// Packed-corpus verification glue and the official pure-JS token materializer.
//
// Rust/wasm is the only trust boundary — container structure, XXH3 integrity
// checksums, exact source binding, marker-catalog stamp — and also
// materializes findings, so `LintIssue.message` keeps a single renderer in a
// single language. JS materializes tokens directly out of the certified
// buffer, which is where the volume is (190k-280k tokens for a large book
// against ~159 findings).
//
// There is no hash, checksum, or source-binding code in this file, and there
// never will be: `verifyPackedBook` already certified the bytes. What the
// decoder below does do is bounds-check every structure it walks, so a buffer
// that somehow reached it uncertified fails loudly instead of reading past an
// array. Every field *position* comes from the generated ./wire-schema
// constants, and every per-row *stride* comes from the container's own
// directory-reported field width (never a hardcoded 2/4) — the sole exception
// is `u64()`'s high/low split, 4 bytes forward by the definition of reading a
// 64-bit value as two little-endian 32-bit halves, not a schema position.

import {
  ATTRIBUTE_ENTRY_LEN,
  ATTRIBUTE_ENTRY_OFFSET,
  ATTRIBUTE_FLAG_DEFAULT,
  ATTRIBUTE_ROW_LEN,
  ATTRIBUTE_ROW_OFFSET,
  BOOK_CODE_FLAG_VALID,
  BOOK_CODE_RECORD_LEN,
  BOOK_CODE_RECORD_OFFSET,
  CONTAINER_FLAGS_KNOWN,
  CONTAINER_HEADER_LEN,
  CONTAINER_HEADER_OFFSET,
  CONTAINER_MAGIC,
  DIRECTORY_ENTRY_LEN,
  DIRECTORY_ENTRY_OFFSET,
  ELEMENT_WIDTH_VARIABLE,
  ELEMENT_WIDTHS,
  FORMAT_VERSION,
  INDEX_NONE_U16,
  NUMBER_FLAG_HAS_END,
  NUMBER_RANGE_KIND_WIRE,
  NUMBER_RECORD_LEN,
  NUMBER_RECORD_OFFSET,
  PACKED_SID_LEN,
  PACKED_SID_OFFSET,
  SECTION_FLAG_POSITIONAL_IDS,
  SECTION_HEADER_LEN,
  SECTION_HEADER_OFFSET,
  SECTION_KIND,
  SECTION_MAGIC,
  SECTION_VERSION,
  SID_DELTA_MASK,
  SPAN_ABSENT,
  STRING_DICTIONARY_ENTRY_LEN,
  TOC_ENTRY_LEN,
  TOC_ENTRY_OFFSET,
  TOC_FLAGS_KNOWN,
  TOKEN_FIELD,
  TOKEN_KIND,
  TOKEN_KIND_WIRE,
} from "./wire-schema.js";

/** Token field ids by generated name, so no id is written out here. */
const FIELD = Object.fromEntries(TOKEN_FIELD.map((field) => [field.name, field.id]));
const FIELD_WIDTH = Object.fromEntries(
  TOKEN_FIELD.map((field) => [field.id, field.elementWidth]),
);

/** Kinds that carry a marker descriptor, and the subset that may carry attributes. */
const MARKER_BEARING = new Set([TOKEN_KIND.Marker, TOKEN_KIND.EndMarker, TOKEN_KIND.Milestone]);
const ATTRIBUTE_BEARING = new Set([TOKEN_KIND.Marker, TOKEN_KIND.Milestone]);

// `fatal` is load-bearing: Rust resolves a token's text with `str::get`, which
// refuses a span that splits a character. A lenient decoder would substitute
// U+FFFD and hand back a token that never existed.
const TEXT = new TextDecoder("utf-8", { fatal: true });
const ASCII = new TextDecoder("ascii");

/**
 * A structural failure in packed bytes, or a caller naming something the
 * verified corpus does not contain. `kind` mirrors the Rust `PackedDecodeError`
 * tags, plus `unknownBook` / `ambiguousBook` / `unknownChapter` for selector
 * mistakes.
 */
export class PackedError extends Error {
  constructor(kind, detail) {
    super(detail ? `${kind}: ${detail}` : kind);
    this.name = "PackedError";
    this.kind = kind;
  }
}

function fail(kind, detail) {
  throw new PackedError(kind, detail);
}

function viewOf(bytes) {
  return new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
}

/**
 * Recursively freezes an object/array and everything reachable from it via
 * its own property names (`Object.freeze` alone is shallow). Used once per
 * book, at verify time, on the receipt's marker-descriptor tree — never per
 * token and never per call — so every token that attaches
 * `descriptor.markerMetadata`/`.structural` by reference (tokenReader, below)
 * shares one frozen object instead of one a caller could mutate and have that
 * mutation silently reappear in every later materialization of the same book.
 */
function deepFreeze(value) {
  if (
    value !== null &&
    (typeof value === "object" || typeof value === "function") &&
    !Object.isFrozen(value)
  ) {
    Object.freeze(value);
    for (const key of Object.getOwnPropertyNames(value)) {
      deepFreeze(value[key]);
    }
  }
  return value;
}

function ascii(bytes, at, len) {
  if (at + len > bytes.byteLength) fail("truncated");
  return ASCII.decode(bytes.subarray(at, at + len));
}

/**
 * A `u64` offset or length narrowed to a JS number. A non-zero high word cannot
 * describe anything inside a buffer this runtime can hold, so it is the
 * `offsetOverflow` case rather than a silent precision loss.
 */
function u64(view, at) {
  if (view.getUint32(at + 4, true) !== 0) fail("offsetOverflow");
  return view.getUint32(at, true);
}

function requireRange(bytes, offset, length) {
  if (offset < 0 || length < 0 || offset + length > bytes.byteLength) fail("truncated");
}

// --- container / section structure ------------------------------------------

function readContainer(packed) {
  if (packed.byteLength < CONTAINER_HEADER_LEN) fail("truncated");
  const view = viewOf(packed);
  const H = CONTAINER_HEADER_OFFSET;
  if (ascii(packed, H.magic, 4) !== CONTAINER_MAGIC) fail("badMagic");
  if (view.getUint16(H.formatVersion, true) !== FORMAT_VERSION) fail("unsupportedVersion");
  if (view.getUint16(H.headerLen, true) !== CONTAINER_HEADER_LEN) fail("invalidSection");
  if ((view.getUint32(H.flags, true) & ~CONTAINER_FLAGS_KNOWN) !== 0) fail("unsupportedFlags");
  const sectionCount = view.getUint32(H.sectionCount, true);
  const tocOffset = u64(view, H.tocOffset);
  const tocLength = sectionCount * TOC_ENTRY_LEN;
  requireRange(packed, tocOffset, tocLength);

  const T = TOC_ENTRY_OFFSET;
  const toc = [];
  for (let index = 0; index < sectionCount; index += 1) {
    const at = tocOffset + index * TOC_ENTRY_LEN;
    const kind = view.getUint8(at + T.kind);
    if (kind !== SECTION_KIND.Token && kind !== SECTION_KIND.Finding) fail("invalidDiscriminant");
    if (view.getUint16(at + T.sectionVersion, true) !== SECTION_VERSION) fail("unsupportedVersion");
    if ((view.getUint16(at + T.flags, true) & ~TOC_FLAGS_KNOWN) !== 0) fail("unsupportedFlags");
    const offset = u64(view, at + T.offset);
    const byteLen = u64(view, at + T.byteLen);
    if (byteLen < SECTION_HEADER_LEN) fail("invalidToc");
    requireRange(packed, offset, byteLen);
    toc.push({ kind, book: ascii(packed, at + T.book, 3), offset, byteLen });
  }
  return toc;
}

function readSection(packed, entry) {
  const bytes = packed.subarray(entry.offset, entry.offset + entry.byteLen);
  const view = viewOf(bytes);
  const S = SECTION_HEADER_OFFSET;
  if (ascii(bytes, S.magic, 4) !== SECTION_MAGIC) fail("badMagic");
  if (view.getUint16(S.formatVersion, true) !== FORMAT_VERSION) fail("unsupportedVersion");
  if (view.getUint8(S.kind) !== entry.kind) fail("invalidSection");
  const flags = view.getUint8(S.flags);
  if (ascii(bytes, S.book, 3) !== entry.book) fail("invalidSection");
  const recordCount = view.getUint32(S.recordCount, true);
  const directoryCount = view.getUint16(S.directoryCount, true);
  if (view.getUint16(S.directoryEntrySize, true) !== DIRECTORY_ENTRY_LEN) fail("invalidSection");
  if (u64(view, S.sectionLen) !== bytes.byteLength) fail("invalidSection");

  const payloadStart = SECTION_HEADER_LEN + directoryCount * DIRECTORY_ENTRY_LEN;
  if (payloadStart > bytes.byteLength) fail("truncated");
  const D = DIRECTORY_ENTRY_OFFSET;
  const fields = new Map();
  for (let index = 0; index < directoryCount; index += 1) {
    const at = SECTION_HEADER_LEN + index * DIRECTORY_ENTRY_LEN;
    const id = view.getUint16(at + D.fieldId, true);
    const width = view.getUint8(at + D.elementWidth);
    if (width !== ELEMENT_WIDTH_VARIABLE && !ELEMENT_WIDTHS.includes(width)) fail("invalidSection");
    const offset = view.getUint32(at + D.offset, true);
    const byteLen = view.getUint32(at + D.byteLen, true);
    const count = view.getUint32(at + D.count, true);
    if (offset < payloadStart) fail("invalidSection");
    requireRange(bytes, offset, byteLen);
    if (width !== ELEMENT_WIDTH_VARIABLE && count * width !== byteLen) fail("invalidSection");
    // A known id's width is fixed by the schema; a producer disagreeing about it
    // is describing a different format, not a variant of this one.
    const expected = FIELD_WIDTH[id];
    if (expected != null && expected !== width) fail("invalidSection");
    if (fields.has(id)) fail("invalidSection");
    fields.set(id, { id, width, count, byteLen, bytes: bytes.subarray(offset, offset + byteLen) });
  }
  return {
    book: entry.book,
    flags,
    recordCount,
    positionalIds: (flags & SECTION_FLAG_POSITIONAL_IDS) !== 0,
    fields,
  };
}

function requiredField(section, id, recordCount) {
  const field = section.fields.get(id);
  if (!field) fail("invalidSection", `missing token field ${id}`);
  if (recordCount !== undefined && field.count !== recordCount) fail("invalidSection");
  return field;
}

// --- dictionaries and sparse columns ----------------------------------------

/** `[u32; count]` ascending start offsets followed by concatenated UTF-8. */
function readStringDictionary(field) {
  if (!field || field.count === 0) {
    if (field && field.byteLen !== 0) fail("invalidSection");
    return [];
  }
  const offsetsLen = field.count * STRING_DICTIONARY_ENTRY_LEN;
  if (field.byteLen < offsetsLen) fail("truncated");
  const view = viewOf(field.bytes);
  const data = field.bytes.subarray(offsetsLen);
  const starts = new Array(field.count);
  let previous = 0;
  for (let index = 0; index < field.count; index += 1) {
    const start = view.getUint32(index * STRING_DICTIONARY_ENTRY_LEN, true);
    if (start < previous || start > data.byteLength) fail("invalidSection");
    starts[index] = start;
    previous = start;
  }
  if (starts[0] !== 0) fail("invalidSection");
  const out = new Array(field.count);
  for (let index = 0; index < field.count; index += 1) {
    const end = index + 1 < field.count ? starts[index + 1] : data.byteLength;
    out[index] = decodeUtf8(data.subarray(starts[index], end));
  }
  return out;
}

function decodeUtf8(bytes) {
  try {
    return TEXT.decode(bytes);
  } catch {
    return fail("invalidUtf8");
  }
}

/**
 * A fixed-width sparse column keyed by an ascending `token_idx` in its first
 * `u32`. Lookup is a binary search rather than a map, which is also what lets a
 * chapter-selective pass start anywhere in the stream for free.
 */
function sparseColumn(field, recordLen) {
  const bytes = field ? field.bytes : new Uint8Array(0);
  const count = field ? field.count : 0;
  if (field && field.byteLen !== count * recordLen) fail("invalidSection");
  const view = viewOf(bytes);
  return {
    count,
    view,
    tokenIdxAt(index) {
      return view.getUint32(index * recordLen, true);
    },
    /** Index of the first record whose `token_idx` is >= `row`. */
    lowerBound(row) {
      let low = 0;
      let high = count;
      while (low < high) {
        const mid = (low + high) >> 1;
        if (view.getUint32(mid * recordLen, true) < row) low = mid + 1;
        else high = mid;
      }
      return low;
    },
  };
}

/** Attribute rows (24 bytes) followed by the entries they partition (20 bytes). */
function attributeColumn(field) {
  if (!field) return null;
  const rowsLen = field.count * ATTRIBUTE_ROW_LEN;
  if (field.byteLen < rowsLen) fail("truncated");
  const entries = field.bytes.subarray(rowsLen);
  if (entries.byteLength % ATTRIBUTE_ENTRY_LEN !== 0) fail("invalidSection");
  return {
    rows: sparseColumn(
      { bytes: field.bytes.subarray(0, rowsLen), count: field.count, byteLen: rowsLen },
      ATTRIBUTE_ROW_LEN,
    ),
    entries,
    entryView: viewOf(entries),
    entryCount: entries.byteLength / ATTRIBUTE_ENTRY_LEN,
  };
}

/**
 * Undoes the two USFM attribute-value escapes on read, mirroring
 * `usfm_onion_wire::dto::decode_attr_value` (`\\` → `\`, `\"` → `"`; any other
 * backslash sequence is preserved verbatim). The equivalence gate is what keeps
 * the two in step.
 */
function decodeAttributeValue(raw) {
  if (!raw.includes("\\")) return raw;
  let out = "";
  for (let index = 0; index < raw.length; index += 1) {
    const char = raw[index];
    if (char === "\\") {
      const next = raw[index + 1];
      if (next === "\\" || next === '"') {
        out += next;
        index += 1;
        continue;
      }
    }
    out += char;
  }
  return out;
}

// --- the token materializer --------------------------------------------------

/**
 * Everything needed to materialize any row of one book, built once per book and
 * reused by the full and chapter-selective passes so both produce byte-identical
 * tokens for the same row.
 */
function tokenReader(book) {
  const { packed, source, receipt } = book;
  const toc = readContainer(packed);
  const tokenEntries = toc.filter((entry) => entry.kind === SECTION_KIND.Token);
  if (tokenEntries.length !== 1) fail("invalidToc");
  const section = readSection(packed, tokenEntries[0]);
  if (source.byteLength !== receipt.sourceLen) fail("sourceLengthMismatch");

  const rowCount = section.recordCount;
  const kinds = requiredField(section, FIELD.kind, rowCount).bytes;
  // Each column's own directory-reported `width` is its per-row byte stride —
  // read off the field the container actually declared, not a hardcoded 2/4.
  // The stride is therefore never a literal; the fixed getUint16/getUint32
  // calls below still assume today's widths and would need to change to match
  // a genuinely widened column.
  const spanStartField = requiredField(section, FIELD.spanStart, rowCount);
  const spanStarts = viewOf(spanStartField.bytes);
  const spanEndField = requiredField(section, FIELD.spanEnd, rowCount);
  const spanEnds = viewOf(spanEndField.bytes);
  const sidIndexField = requiredField(section, FIELD.sidIndex, rowCount);
  const sidIndices = viewOf(sidIndexField.bytes);
  const descriptorIndexField = requiredField(section, FIELD.markerDescriptorIndex, rowCount);
  const descriptorIndices = viewOf(descriptorIndexField.bytes);
  const sids = requiredField(section, FIELD.packedSidDictionary);
  const sidCount = sids.count;
  const sidView = viewOf(sids.bytes);
  const sidCache = new Array(sidCount).fill(null);

  const strings = readStringDictionary(section.fields.get(FIELD.stringDictionary));
  // Deep-frozen once at verify time (verifyPackedCorpus); every materialized
  // token below attaches `.markerMetadata`/`.structural` from these same
  // objects by reference, never a copy, so the freeze is what keeps a mutation
  // on one materialized token from silently reappearing on every other.
  const descriptors = receipt.descriptors;
  const numbers = sparseColumn(section.fields.get(FIELD.numberRecords), NUMBER_RECORD_LEN);
  const bookCodes = sparseColumn(section.fields.get(FIELD.bookCodeRecords), BOOK_CODE_RECORD_LEN);
  const attributes = attributeColumn(section.fields.get(FIELD.attributeRecords));

  // Explicit ids and their dictionary are present together or absent together;
  // the section flag asserts which, and the two must agree.
  const idIndexField = section.fields.get(FIELD.tokenIdIndex);
  if (section.positionalIds !== !idIndexField) fail("invalidSection");
  if (idIndexField && idIndexField.count !== rowCount) fail("invalidSection");
  const idIndices = idIndexField ? viewOf(idIndexField.bytes) : null;
  const idStrings = idIndexField
    ? readStringDictionary(section.fields.get(FIELD.tokenIdDictionary))
    : null;
  if (idIndexField && idStrings === null) fail("invalidSection");

  // `assign_ids`' rule, reproduced: the book label is the nearest preceding
  // book-code token's own text, defaulting to the first one in the stream (or
  // "unknown" when the book has none at all).
  const bookCodeText = (index) => {
    const at = index * BOOK_CODE_RECORD_LEN + BOOK_CODE_RECORD_OFFSET.codeIndex;
    const text = strings[bookCodes.view.getUint32(at, true)];
    if (text === undefined) fail("invalidSection");
    return text;
  };
  const defaultBook = bookCodes.count > 0 ? bookCodeText(0) : "unknown";
  const bookLabelAt = (row) => {
    const next = bookCodes.lowerBound(row + 1);
    return next === 0 ? defaultBook : bookCodeText(next - 1);
  };

  const sidAt = (index) => {
    if (index === INDEX_NONE_U16) return null;
    if (index >= sidCount) fail("invalidSection");
    const cached = sidCache[index];
    if (cached !== null) return cached;
    const at = index * PACKED_SID_LEN;
    const chapter = sidView.getUint16(at + PACKED_SID_OFFSET.chapter, true);
    const verse = sidView.getUint16(at + PACKED_SID_OFFSET.verse, true);
    const delta = sidView.getUint8(at + PACKED_SID_OFFSET.delta) & SID_DELTA_MASK;
    const locator = delta === 0 ? `${verse}` : `${verse}-${verse + delta}`;
    const text = `${ascii(sids.bytes, at + PACKED_SID_OFFSET.book, 3)} ${chapter}:${locator}`;
    sidCache[index] = text;
    return text;
  };
  const sidChapterAt = (index) => {
    if (index >= sidCount) fail("invalidSection");
    return sidView.getUint16(index * PACKED_SID_LEN + PACKED_SID_OFFSET.chapter, true);
  };

  const sourceText = (start, end) => {
    if (end < start || end > source.byteLength) fail("invalidSection");
    return decodeUtf8(source.subarray(start, end));
  };

  const materializeRow = (row) => {
    const tag = kinds[row];
    const wireKind = TOKEN_KIND_WIRE[tag];
    if (wireKind === undefined) fail("invalidDiscriminant");
    const start = spanStarts.getUint32(row * spanStartField.width, true);
    const end = spanEnds.getUint32(row * spanEndField.width, true);
    const token = {
      id: `${bookLabelAt(row)}-${row}`,
      kind: wireKind,
      source: sourceText(start, end),
      span: { start, end },
    };
    const sid = sidAt(sidIndices.getUint16(row * sidIndexField.width, true));
    if (sid !== null) token.sid = sid;

    const descriptorIndex = descriptorIndices.getUint16(row * descriptorIndexField.width, true);
    const markerBearing = MARKER_BEARING.has(tag);
    if (markerBearing !== (descriptorIndex !== INDEX_NONE_U16)) fail("invalidSection");
    if (markerBearing) {
      const descriptor = descriptors[descriptorIndex];
      if (descriptor === undefined) fail("invalidSection");
      // A milestone has no `nested` field at all, so a descriptor claiming one
      // cannot be describing this row.
      if (tag === TOKEN_KIND.Milestone && descriptor.nested) fail("invalidSection");
      token.marker = descriptor.name;
      if (tag !== TOKEN_KIND.Milestone) token.nested = descriptor.nested;
      token.markerMetadata = descriptor.markerMetadata;
      token.structural = descriptor.structural;
    }

    if (tag === TOKEN_KIND.Number) {
      const index = numbers.lowerBound(row);
      if (index >= numbers.count || numbers.tokenIdxAt(index) !== row) fail("invalidSection");
      const at = index * NUMBER_RECORD_LEN;
      const kind = NUMBER_RANGE_KIND_WIRE[numbers.view.getUint8(at + NUMBER_RECORD_OFFSET.kind)];
      if (kind === undefined) fail("invalidDiscriminant");
      const info = { start: numbers.view.getUint32(at + NUMBER_RECORD_OFFSET.start, true), kind };
      if ((numbers.view.getUint8(at + NUMBER_RECORD_OFFSET.flags) & NUMBER_FLAG_HAS_END) !== 0) {
        info.end = numbers.view.getUint32(at + NUMBER_RECORD_OFFSET.end, true);
      }
      token.numberInfo = info;
    } else if (tag === TOKEN_KIND.BookCode) {
      const index = bookCodes.lowerBound(row);
      if (index >= bookCodes.count || bookCodes.tokenIdxAt(index) !== row) fail("invalidSection");
      token.bookCode = bookCodeText(index);
      token.bookCodeValid =
        (bookCodes.view.getUint8(index * BOOK_CODE_RECORD_LEN + BOOK_CODE_RECORD_OFFSET.flags) &
          BOOK_CODE_FLAG_VALID) !==
        0;
    }

    if (attributes) {
      const index = attributes.rows.lowerBound(row);
      if (index < attributes.rows.count && attributes.rows.tokenIdxAt(index) === row) {
        if (!ATTRIBUTE_BEARING.has(tag)) fail("invalidSection");
        applyAttributes(token, attributes, index, strings, sourceText);
      }
    }
    return token;
  };

  return {
    book: section.book,
    rowCount,
    positionalIds: section.positionalIds,
    materializeRow,
    stableIdAt: idIndices
      ? (row) => {
          const id = idStrings[idIndices.getUint32(row * idIndexField.width, true)];
          if (id === undefined || id === "") fail("invalidSection");
          return id;
        }
      : null,
    /**
     * First and last row anchored to `chapter`, inclusive. Reads only the SID
     * index column, so locating a viewport costs no token materialization.
     */
    chapterRange(chapter) {
      let first = -1;
      let last = -1;
      for (let row = 0; row < rowCount; row += 1) {
        const index = sidIndices.getUint16(row * sidIndexField.width, true);
        if (index === INDEX_NONE_U16) continue;
        if (sidChapterAt(index) !== chapter) continue;
        if (first < 0) first = row;
        last = row;
      }
      if (first < 0) fail("unknownChapter", `chapter ${chapter}`);
      return { start: first, end: last };
    },
  };
}

function applyAttributes(token, attributes, rowIndex, strings, sourceText) {
  const view = attributes.rows.view;
  const at = rowIndex * ATTRIBUTE_ROW_LEN;
  const first = view.getUint32(at + ATTRIBUTE_ROW_OFFSET.firstEntry, true);
  const count = view.getUint32(at + ATTRIBUTE_ROW_OFFSET.entryCount, true);
  const listStart = view.getUint32(at + ATTRIBUTE_ROW_OFFSET.listStart, true);
  const listLen = view.getUint32(at + ATTRIBUTE_ROW_OFFSET.listLen, true);
  if (first + count > attributes.entryCount) fail("invalidSection");
  if (listStart !== SPAN_ABSENT) {
    token.attributeSource = sourceText(listStart, listStart + listLen);
    // Where the list sat, as a distance from this token's own end — the other half
    // of what the verbatim slice promises, since one placement rule cannot express
    // every real layout. A list recorded as starting before its owner is not a
    // distance this can state, so it is left absent and the emitter falls back to
    // placing at the marker's closer.
    if (token.span !== undefined && listStart >= token.span.end) {
      token.attributeOffset = listStart - token.span.end;
    }
  } else if (listLen !== 0) {
    fail("invalidSection");
  }
  if (count === 0) return;
  const items = new Array(count);
  for (let offset = 0; offset < count; offset += 1) {
    const entry = (first + offset) * ATTRIBUTE_ENTRY_LEN;
    const key = strings[attributes.entryView.getUint32(entry + ATTRIBUTE_ENTRY_OFFSET.keyIndex, true)];
    const value = strings[attributes.entryView.getUint32(entry + ATTRIBUTE_ENTRY_OFFSET.valueIndex, true)];
    if (key === undefined || value === undefined) fail("invalidSection");
    const start = attributes.entryView.getUint32(entry + ATTRIBUTE_ENTRY_OFFSET.spanStart, true);
    const length = attributes.entryView.getUint32(entry + ATTRIBUTE_ENTRY_OFFSET.spanLen, true);
    items[offset] = {
      span: { start, end: start + length },
      text: sourceText(start, start + length),
      key,
      value: decodeAttributeValue(value),
      isDefault:
        (attributes.entryView.getUint8(entry + ATTRIBUTE_ENTRY_OFFSET.flags) & ATTRIBUTE_FLAG_DEFAULT) !== 0,
    };
  }
  token.attributes = items;
}

// --- public surface ---------------------------------------------------------

/**
 * Verifies every record through the Rust trust boundary and mints the opaque
 * `VerifiedPacked` handle.
 *
 * The first rejected record short-circuits the whole corpus: a partially
 * restored corpus is not a state the caller asked for, and a typed rejection is
 * the signal to fall back to normal USFM ingest/parse.
 *
 * The caller's `packed`/`source` bytes are copied (`Uint8Array#slice`, never an
 * `ArrayBuffer` transfer/detach) into handle-private state before they are
 * verified. Without this, a caller could mutate its own array after minting
 * and pair a still-valid certification with changed bytes.
 *
 * Threat model (dated 2026-07-29): this module protects HONEST use — a
 * caller that mutates its own buffers by accident, or passes an
 * already-verified handle around a codebase without re-checking it. The
 * handle's opacity (below) is a footgun-elimination device, not a security
 * boundary: it is not meant to stop code in the same JS process that is
 * deliberately trying to subvert this module's own state.
 *
 * @param {{ verifyPackedBook: (packed: Uint8Array, source: Uint8Array) => any }} wasm
 * @param {readonly { path: string, packed: Uint8Array, source: Uint8Array }[]} records
 */
export function verifyPackedCorpus(wasm, records) {
  const books = new Map();
  const findings = new Map();
  for (const record of records) {
    const packed = record.packed.slice();
    const source = record.source.slice();
    const outcome = wasm.verifyPackedBook(packed, source);
    if (outcome.status !== "verified") {
      return { ok: false, path: record.path, error: outcome.error };
    }
    if (books.has(record.path)) {
      return {
        ok: false,
        path: record.path,
        error: { kind: "invalidToc" },
      };
    }
    // One-time O(descriptor-count) freeze (a few dozen rows for a whole
    // scripture book) — not O(token-count): every token that attaches these
    // objects by reference does so many times over, so cloning per token or
    // per materialize call would be wasted work the freeze avoids entirely
    // while still making mutation loud instead of silently shared.
    deepFreeze(outcome.receipt.descriptors);
    books.set(record.path, {
      path: record.path,
      packed,
      source,
      receipt: outcome.receipt,
    });
    findings.set(record.path, outcome.findings);
  }
  // The handle is a plain frozen object with no own properties at all — every
  // decoder input lives in `STATE`, keyed by the handle's identity. Membership
  // in `STATE` *is* the mint check: there is nothing on the handle itself to
  // fake, and nothing public to read state off of by accident.
  const verified = Object.freeze({});
  STATE.set(verified, books);
  return { ok: true, verified, findings };
}

/** Handle identity → its books, private to this module. */
const STATE = new WeakMap();

function requireVerified(verified) {
  const books = STATE.get(verified);
  if (!books) {
    fail("invalidSection", "materialize accepts only a VerifiedPacked handle");
  }
  return books;
}

/**
 * A detached snapshot of what verification certified for one book —
 * `structuredClone` produces an ordinary mutable object, not a read-only one.
 * It is detached, not read-only: `materialize`/`decodeTokens` read their own
 * copy of this same data out of `STATE`, never this function's return value,
 * so mutating what this returns cannot affect materialized output either way.
 * Exists because tests and callers legitimately need to inspect a receipt
 * (book code, counts, positional-ids flag) without the handle exposing its
 * internal state.
 *
 * @returns {{ path: string, receipt: object }}
 */
export function receiptFor(verified, path) {
  const books = requireVerified(verified);
  const book = books.get(path);
  if (!book) fail("unknownBook", path);
  return structuredClone({ path: book.path, receipt: book.receipt });
}

function resolveBook(books, selector) {
  if (selector.path !== undefined) {
    const book = books.get(selector.path);
    if (!book) fail("unknownBook", selector.path);
    return book;
  }
  const matches = [];
  for (const book of books.values()) {
    if (book.receipt.book === selector.book) matches.push(book);
  }
  if (matches.length === 0) fail("unknownBook", selector.book);
  // Two corpora in one restore can legitimately both carry GEN, so a book code
  // is not an identity — the caller has to name the path it supplied.
  if (matches.length > 1) fail("ambiguousBook", selector.book);
  return matches[0];
}

function materializeBook(book, chapter) {
  const reader = tokenReader(book);
  const range =
    chapter === undefined ? { start: 0, end: reader.rowCount - 1 } : reader.chapterRange(chapter);
  const length = reader.rowCount === 0 ? 0 : range.end - range.start + 1;
  const tokens = new Array(length);
  for (let offset = 0; offset < length; offset += 1) {
    tokens[offset] = reader.materializeRow(range.start + offset);
  }
  const out = { path: book.path, book: reader.book, tokens };
  if (reader.stableIdAt) {
    const ids = new Array(length);
    for (let offset = 0; offset < length; offset += 1) {
      ids[offset] = reader.stableIdAt(range.start + offset);
    }
    out.stableIds = ids;
  }
  return out;
}

/**
 * Materializes tokens from certified bytes, in the JS engine, with no wasm call.
 *
 * With no selector: every verified book, keyed by the path the caller supplied.
 * With `{path}` or `{book}`: that one book. Adding `{chapter}` materializes only
 * that chapter's contiguous row range — located through the packed SID column,
 * so a viewport costs no whole-book work — and the result is guaranteed to be
 * the same tokens the full pass produces for those rows, `id` included.
 *
 * Findings are not here: they arrive already materialized on the verify result.
 *
 * @returns {Map<string, { path: string, book: string, tokens: object[], stableIds?: string[] }>}
 */
export function materialize(verified, selector) {
  const books = requireVerified(verified);
  const out = new Map();
  if (selector === undefined || (selector.path === undefined && selector.book === undefined)) {
    if (selector !== undefined && selector.chapter !== undefined) {
      fail("unknownBook", "a chapter selector must name a path or book");
    }
    for (const book of books.values()) {
      out.set(book.path, materializeBook(book, undefined));
    }
    return out;
  }
  const book = resolveBook(books, selector);
  out.set(book.path, materializeBook(book, selector.chapter));
  return out;
}

/** Tokens-only entry for one book, by the path the caller supplied. */
export function decodeTokens(verified, path) {
  const books = requireVerified(verified);
  const book = books.get(path);
  if (!book) fail("unknownBook", path);
  return materializeBook(book, undefined);
}
