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
 *
 * Takes an already-resolved token `section` rather than deriving one from a
 * whole `packed` buffer itself, so this same core serves two callers: the
 * per-book layer (`tokenReader`, below — one book per buffer, so it locates
 * the section by requiring there be exactly one) and the combined-corpus
 * layer (`corpusTokenReader` — many books share one buffer and one TOC
 * already read once at verify time, so it locates the section by book code
 * instead). Neither is a second decoder; both hand the identical section
 * shape to this one function.
 */
function buildTokenReader(section, source, receipt) {
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

  // v2 layout (v0.1.6): widens the packed entry to carry the two occurrence
  // ordinals phase-1 sids can spell (`_cdup_N`/`_dup_N`). The book stays
  // inline per entry — a resident sid can legitimately name a book other
  // than the section's own (a non-canonical `\id`), so the section's book is
  // not a safe substitute; see the Rust `packed_sid` module doc.
  const sidAt = (index) => {
    if (index === INDEX_NONE_U16) return null;
    if (index >= sidCount) fail("invalidSection");
    const cached = sidCache[index];
    if (cached !== null) return cached;
    const at = index * PACKED_SID_LEN;
    const chapter = sidView.getUint16(at + PACKED_SID_OFFSET.chapter, true);
    const verse = sidView.getUint16(at + PACKED_SID_OFFSET.verse, true);
    const delta = sidView.getUint8(at + PACKED_SID_OFFSET.delta);
    const chapterOccurrence = sidView.getUint8(at + PACKED_SID_OFFSET.chapterOccurrence);
    const verseOccurrence = sidView.getUint8(at + PACKED_SID_OFFSET.verseOccurrence);
    const locator = delta === 0 ? `${verse}` : `${verse}-${verse + delta}`;
    // Suffix order/spelling mirrors core `Sid`'s own `Display`: "_cdup_N" then
    // "_dup_N", each only when its ordinal is nonzero.
    let text = `${ascii(sids.bytes, at + PACKED_SID_OFFSET.book, 3)} ${chapter}:${locator}`;
    if (chapterOccurrence > 0) text += `_cdup_${chapterOccurrence}`;
    if (verseOccurrence > 0) text += `_dup_${verseOccurrence}`;
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

/** Per-book layer: one book per `packed` buffer, so its own TOC must name
 * exactly one token section. */
function tokenReader(book) {
  const { packed, source, receipt } = book;
  const toc = readContainer(packed);
  const tokenEntries = toc.filter((entry) => entry.kind === SECTION_KIND.Token);
  if (tokenEntries.length !== 1) fail("invalidToc");
  const section = readSection(packed, tokenEntries[0]);
  return buildTokenReader(section, source, receipt);
}

/** Combined-corpus layer: many books share one `packed` buffer and one TOC
 * (read once, at verify time — see `verifyPublishedPacked`); this book's own
 * token section is whichever TOC entry names its book code. Position-
 * independent by design (§P.3), so locating it is a filter, not a search
 * that depends on any other book's presence or order. */
function corpusTokenReader(state, book) {
  const entry = state.toc.find(
    (candidate) => candidate.kind === SECTION_KIND.Token && candidate.book === book.book,
  );
  if (!entry) fail("invalidToc", book.book);
  const section = readSection(state.packed, entry);
  return buildTokenReader(section, book.source, book.receipt);
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
 * Slices `buf` at `extent` (`{byteOffset, byteLength}`), refusing -- never
 * clamping or truncating -- when the extent falls outside `buf`. `subarray`
 * itself clamps silently, which is exactly the failure mode this bounds
 * check exists to intercept before it ever reaches one: an out-of-range or
 * overflowing extent returns `null` rather than a shorter-than-requested (or
 * wrapped) view (v0.1.5, bytes-at-boundary convention).
 *
 * @param {Uint8Array} buf
 * @param {{ byteOffset: number, byteLength: number }} extent
 * @returns {Uint8Array | null}
 */
function sliceExtent(buf, extent) {
  const { byteOffset, byteLength } = extent;
  if (
    !Number.isInteger(byteOffset)
    || !Number.isInteger(byteLength)
    || byteOffset < 0
    || byteLength < 0
    || byteOffset + byteLength > buf.length
  ) {
    return null;
  }
  return buf.subarray(byteOffset, byteOffset + byteLength);
}

/**
 * Verifies every record through the Rust trust boundary and mints the opaque
 * `VerifiedPacked` handle.
 *
 * `packedAll`/`sources` are two single buffers -- every record's own
 * container concatenated into the first, every record's own source
 * concatenated into the second -- with `records` naming each one's extent
 * into whichever buffer it belongs to (v0.1.5, bytes-at-boundary
 * convention): the exact shape `wasm.restoreCorpus` takes, and exactly what
 * `publishScope`'s wasm output already is, so a scoped publication forwards
 * here with zero reshaping. An extent outside its buffer refuses
 * (`{kind: "invalidExtent"}`, naming the record's own `path`) before any
 * bytes reach wasm at all -- no `Array.from`, no per-record copy: each
 * record's own `packed`/`source` is a zero-copy `subarray` view into the one
 * defensive copy this function takes of each whole buffer up front.
 *
 * The first rejected record short-circuits the whole corpus: a partially
 * restored corpus is not a state the caller asked for, and a typed rejection is
 * the signal to fall back to normal USFM ingest/parse.
 *
 * `packedAll`/`sources` are copied (`Uint8Array#slice`, never an
 * `ArrayBuffer` transfer/detach) into handle-private state before they are
 * verified. Without this, a caller could mutate its own buffer after minting
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
 * @param {Uint8Array} packedAll
 * @param {Uint8Array} sources
 * @param {readonly { path: string, packed: { byteOffset: number, byteLength: number }, source: { byteOffset: number, byteLength: number } }[]} records
 */
export function verifyPackedCorpus(wasm, packedAll, sources, records) {
  const packedAllCopy = packedAll.slice();
  const sourcesCopy = sources.slice();
  const books = new Map();
  const findings = new Map();
  for (const record of records) {
    const packed = sliceExtent(packedAllCopy, record.packed);
    const source = sliceExtent(sourcesCopy, record.source);
    if (packed === null || source === null) {
      return { ok: false, path: record.path, error: { kind: "invalidExtent" } };
    }
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

/** Shared by both layers: given an already-built reader, slices and
 * materializes exactly the rows a full or chapter-selective pass needs, so
 * both produce byte-identical tokens for the same row (the reader is the
 * one thing that differs between the per-book and combined-corpus layers). */
function materializeFromReader(reader, chapter) {
  const range =
    chapter === undefined ? { start: 0, end: reader.rowCount - 1 } : reader.chapterRange(chapter);
  const length = reader.rowCount === 0 ? 0 : range.end - range.start + 1;
  const tokens = new Array(length);
  for (let offset = 0; offset < length; offset += 1) {
    tokens[offset] = reader.materializeRow(range.start + offset);
  }
  const out = { book: reader.book, tokens };
  if (reader.stableIdAt) {
    const ids = new Array(length);
    for (let offset = 0; offset < length; offset += 1) {
      ids[offset] = reader.stableIdAt(range.start + offset);
    }
    out.stableIds = ids;
  }
  return out;
}

function materializeBook(book, chapter) {
  const result = materializeFromReader(tokenReader(book), chapter);
  return { path: book.path, ...result };
}

function materializeCorpusBook(state, book, chapter) {
  return materializeFromReader(corpusTokenReader(state, book), chapter);
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

// --- the combined-corpus layer -----------------------------------------------
//
// `publish()` produces one packed `corpus.bin` container carrying every
// resident book's sections in one buffer -- the shape a worker transfers to
// seed braid (`restorePublishedCorpus`, wasm). Main-thread rendering needs
// the same certified bytes materialized in pure JS with no wasm call, the
// same reason the per-book layer above exists; this is that layer's
// corpus-shaped twin, not a second decoder — it locates each book's own
// section in the shared TOC (§P.3: sections are position-independent) and
// hands it to the exact same `buildTokenReader`/`materializeFromReader` core
// the per-book layer uses.

/** Handle identity → its corpus-wide state, private to this module. Separate
 * from `STATE` (the per-book `WeakMap`) because the two handles are not
 * interchangeable: `VerifiedPublished` addresses books by code (a combined
 * container has no per-book caller-supplied path), `VerifiedPacked` by path. */
const CORPUS_STATE = new WeakMap();

function requireVerifiedCorpus(verified) {
  const state = CORPUS_STATE.get(verified);
  if (!state) {
    fail("invalidSection", "materializePublished accepts only a VerifiedPublished handle");
  }
  return state;
}

/**
 * Verifies a combined `publish()` container through the Rust trust boundary
 * (`wasm.verifyPublishedCorpus`, the sole certifier -- no JS hashing or
 * validation, ever) and mints the opaque `VerifiedPublished` handle.
 *
 * `packed` is the one whole-corpus container. `sources` is every named
 * book's source bytes concatenated into one buffer; `records` supplies each
 * book's own code and its extent into `sources` (v0.1.5, bytes-at-boundary
 * convention) -- the same pairing `wasm.restorePublishedCorpus` takes. An
 * extent outside `sources` refuses (`{kind: "invalidExtent"}`, naming the
 * record's own `book`) before any bytes reach wasm.
 *
 * `packed`/`sources` are copied (`Uint8Array#slice`) into handle-private
 * state before they are verified, the same copy-at-mint rule
 * `verifyPackedCorpus` follows and for the same reason: a caller mutating
 * its own buffers after minting must not silently invalidate what was
 * already certified out from under a still-valid handle. No `Array.from`
 * anywhere in this path: every book's own source is a zero-copy `subarray`
 * view into that one defensive copy.
 *
 * @param {{ verifyPublishedCorpus: (packed: Uint8Array, sources: Uint8Array, records: { book: string, sourceKey: string, byteOffset: number, byteLength: number }[]) => any }} wasm
 * @param {Uint8Array} packed
 * @param {Uint8Array} sources
 * @param {readonly { book: string, sourceKey: string, byteOffset: number, byteLength: number }[]} records
 */
export function verifyPublishedPacked(wasm, packed, sources, records) {
  const packedCopy = packed.slice();
  const sourcesCopy = sources.slice();
  for (const record of records) {
    if (sliceExtent(sourcesCopy, record) === null) {
      return { ok: false, error: { kind: "invalidExtent", book: record.book } };
    }
  }
  const outcome = wasm.verifyPublishedCorpus(packedCopy, sourcesCopy, records);
  if (outcome.status === "invalidExtent") {
    // Normally unreachable: every extent was pre-checked against `sources`
    // above. Kept honest anyway -- this arm carries `book`, not `error`.
    return { ok: false, error: { kind: "invalidExtent", book: outcome.book } };
  }
  if (outcome.status !== "verified") {
    return { ok: false, error: outcome.error };
  }
  const toc = readContainer(packedCopy);
  const recordByBook = new Map(records.map((record) => [record.book, record]));
  const books = new Map();
  const findings = new Map();
  for (const entry of outcome.books) {
    const book = entry.receipt.book;
    const record = recordByBook.get(book);
    // Unreachable given `wasm.verifyPublishedCorpus` already enforces exactly
    // one source per book -- checked anyway, since this function's own
    // contract (never index a book it cannot find a source for) should not
    // depend on that invariant holding on trust.
    if (record === undefined) fail("invalidToc", book);
    const source = sliceExtent(sourcesCopy, record);
    // Same one-time freeze `verifyPackedCorpus` performs, same reason: every
    // token that attaches these objects by reference shares one frozen tree.
    deepFreeze(entry.receipt.descriptors);
    books.set(book, { book, source, receipt: entry.receipt });
    findings.set(book, entry.findings);
  }
  const verified = Object.freeze({});
  CORPUS_STATE.set(verified, { packed: packedCopy, toc, books, snapshotId: outcome.snapshotId });
  return { ok: true, verified, findings, snapshotId: outcome.snapshotId };
}

/**
 * A detached snapshot of what verification certified for one book, by book
 * code (a combined container has no caller-supplied path — see
 * {@link receiptFor} for the per-book layer's path-keyed equivalent).
 */
export function receiptForPublished(verified, book) {
  const state = requireVerifiedCorpus(verified);
  const found = state.books.get(book);
  if (!found) fail("unknownBook", book);
  return structuredClone({ book: found.book, receipt: found.receipt });
}

/**
 * Materializes tokens from a certified combined corpus, in the JS engine,
 * with no wasm call -- the same guarantee `materialize` makes for per-book
 * containers, over the container `publish()`/`restorePublishedCorpus`
 * produce and consume.
 *
 * With no selector: every verified book, keyed by book code. With `{book}`:
 * that one book; adding `{chapter}` materializes only that chapter's
 * contiguous row range, guaranteed identical to the corresponding slice of
 * the full pass.
 *
 * Findings are not here: they arrive already materialized on the verify
 * result, keyed by book code the same way.
 *
 * @returns {Map<string, { book: string, tokens: object[], stableIds?: string[] }>}
 */
export function materializePublished(verified, selector) {
  const state = requireVerifiedCorpus(verified);
  const out = new Map();
  if (selector === undefined || selector.book === undefined) {
    if (selector !== undefined && selector.chapter !== undefined) {
      fail("unknownBook", "a chapter selector must name a book");
    }
    for (const book of state.books.values()) {
      out.set(book.book, materializeCorpusBook(state, book, undefined));
    }
    return out;
  }
  const book = state.books.get(selector.book);
  if (!book) fail("unknownBook", selector.book);
  out.set(book.book, materializeCorpusBook(state, book, selector.chapter));
  return out;
}

/** Tokens-only entry for one book, by book code. */
export function decodeTokensPublished(verified, book) {
  const state = requireVerifiedCorpus(verified);
  const found = state.books.get(book);
  if (!found) fail("unknownBook", book);
  return materializeCorpusBook(state, found, undefined);
}

/**
 * Reuses the previous findings array's objects wherever a finding is unchanged.
 *
 * The finding counterpart of token reconciliation, and it exists for the same
 * reason: a consumer that re-renders on every lint pass wants to know which
 * findings are *the same finding*, so it can keep whatever it attached to them —
 * a DOM node, a dismissal, a scroll position — instead of rebuilding all of it
 * because one verse changed.
 *
 * Identity is the rule code plus the tokens the finding is anchored to, which is
 * the only address stable across a recompute: a byte span moves when anything
 * earlier in the book is edited, a token id does not. Message text and fix payload
 * are deliberately not identity — a rule whose wording changed is the same finding
 * — but a change in either still yields a fresh object, because what a consumer
 * reads did change.
 *
 * Never validates and never decodes: these findings already came out of the trust
 * boundary.
 */
export function reconcileFindings(previous, next) {
  // A pool of *not-yet-consumed* candidates per identity key, not a single
  // "first wins" slot: two same-identity findings in `previous` (a real,
  // supported case — duplicate logical identity is deterministic occurrence,
  // not a collision) are two distinct objects, and each may be reused by at
  // most one `next` finding. A single-slot map here previously let a later
  // `next` finding matching the SAME key re-match that one slot a second
  // time, which could report the shortcut "nothing changed, return `previous`
  // itself" even though a real prior finding (never re-matched, never
  // consumed) had actually disappeared from `next`.
  const pools = new Map();
  for (const finding of previous ?? []) {
    const key = findingIdentity(finding);
    let pool = pools.get(key);
    if (pool === undefined) {
      pool = [];
      pools.set(key, pool);
    }
    pool.push(finding);
  }

  const out = (next ?? []).map((finding) => {
    const pool = pools.get(findingIdentity(finding));
    const index = pool?.findIndex((candidate) => sameFindingValue(candidate, finding)) ?? -1;
    if (index === -1) return finding;
    const [candidate] = pool.splice(index, 1);
    return candidate;
  });
  // The shortcut — return `previous` itself so a caller can skip a re-render
  // on identity alone — is only sound when the array is unchanged in BOTH
  // membership and order: a reorder (e.g. previous [A, B], next asking for
  // [B, A]) consumes every candidate one-to-one just like the unchanged
  // case, so a count-only check can't tell them apart and would return
  // `previous` with the wrong order. Require every slot to be the same
  // object at the same position.
  if (out.length === (previous?.length ?? 0) && out.every((finding, i) => finding === previous[i])) {
    return previous;
  }
  return out;
}

function findingIdentity(finding) {
  return finding.code + " " + (finding.tokenId ?? "") + " " + (finding.relatedTokenId ?? "");
}

/** Whether two same-identity findings also read identically. */
function sameFindingValue(a, b) {
  return (
    a.severity === b.severity &&
    a.message === b.message &&
    a.sid === b.sid &&
    a.marker === b.marker &&
    spanEq(a.span, b.span) &&
    spanEq(a.relatedSpan, b.relatedSpan) &&
    JSON.stringify(a.messageParams ?? null) === JSON.stringify(b.messageParams ?? null) &&
    JSON.stringify(a.fix ?? null) === JSON.stringify(b.fix ?? null)
  );
}

function spanEq(a, b) {
  if (a === undefined || a === null) return b === undefined || b === null;
  if (b === undefined || b === null) return false;
  return a.start === b.start && a.end === b.end;
}
