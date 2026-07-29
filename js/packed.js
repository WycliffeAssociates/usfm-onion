// Packed-corpus verification glue and the official pure-JS token materializer.
//
// The split this module implements (freeze §H, adjudicated §I): Rust/wasm is the
// only trust boundary — container structure, XXH3 integrity checksums, exact
// source binding, marker-catalog stamp — and also materializes findings, so
// `LintIssue.message` keeps a single renderer in a single language. JS
// materializes tokens directly out of the certified buffer, which is where the
// volume is (190k-280k tokens for a large book against ~159 findings).
//
// There is no hash, checksum, or source-binding code in this file, and there
// never will be: `verifyPackedBook` already certified the bytes. What the
// decoder below does do is bounds-check every structure it walks, so a buffer
// that somehow reached it uncertified fails loudly instead of reading past an
// array. Every byte offset comes from the generated ./wire-schema constants —
// never a literal written here.

import {
  ATTRIBUTE_ENTRY_LEN,
  ATTRIBUTE_FLAG_DEFAULT,
  ATTRIBUTE_ROW_LEN,
  BOOK_CODE_FLAG_VALID,
  BOOK_CODE_RECORD_LEN,
  CONTAINER_FLAGS_KNOWN,
  CONTAINER_HEADER_LEN,
  CONTAINER_MAGIC,
  DIRECTORY_ENTRY_LEN,
  ELEMENT_WIDTH_VARIABLE,
  ELEMENT_WIDTHS,
  FORMAT_VERSION,
  INDEX_NONE_U16,
  NUMBER_FLAG_HAS_END,
  NUMBER_RANGE_KIND_WIRE,
  NUMBER_RECORD_LEN,
  PACKED_SID_LEN,
  SECTION_FLAG_POSITIONAL_IDS,
  SECTION_HEADER_LEN,
  SECTION_KIND,
  SECTION_MAGIC,
  SECTION_VERSION,
  SID_DELTA_MASK,
  SPAN_ABSENT,
  TOC_ENTRY_LEN,
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

const VERIFIED = Symbol.for("usfm-onion.verifiedPacked");

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
  if (ascii(packed, 0, 4) !== CONTAINER_MAGIC) fail("badMagic");
  if (view.getUint16(4, true) !== FORMAT_VERSION) fail("unsupportedVersion");
  if (view.getUint16(6, true) !== CONTAINER_HEADER_LEN) fail("invalidSection");
  if ((view.getUint32(8, true) & ~CONTAINER_FLAGS_KNOWN) !== 0) fail("unsupportedFlags");
  const sectionCount = view.getUint32(12, true);
  const tocOffset = u64(view, 16);
  const tocLength = sectionCount * TOC_ENTRY_LEN;
  requireRange(packed, tocOffset, tocLength);

  const toc = [];
  for (let index = 0; index < sectionCount; index += 1) {
    const at = tocOffset + index * TOC_ENTRY_LEN;
    const kind = view.getUint8(at);
    if (kind !== SECTION_KIND.Token && kind !== SECTION_KIND.Finding) fail("invalidDiscriminant");
    if (view.getUint16(at + 4, true) !== SECTION_VERSION) fail("unsupportedVersion");
    if ((view.getUint16(at + 6, true) & ~TOC_FLAGS_KNOWN) !== 0) fail("unsupportedFlags");
    const offset = u64(view, at + 8);
    const byteLen = u64(view, at + 16);
    if (byteLen < SECTION_HEADER_LEN) fail("invalidToc");
    requireRange(packed, offset, byteLen);
    toc.push({ kind, book: ascii(packed, at + 1, 3), offset, byteLen });
  }
  return toc;
}

function readSection(packed, entry) {
  const bytes = packed.subarray(entry.offset, entry.offset + entry.byteLen);
  const view = viewOf(bytes);
  if (ascii(bytes, 0, 4) !== SECTION_MAGIC) fail("badMagic");
  if (view.getUint16(4, true) !== FORMAT_VERSION) fail("unsupportedVersion");
  if (view.getUint8(8) !== entry.kind) fail("invalidSection");
  const flags = view.getUint8(9);
  if (ascii(bytes, 10, 3) !== entry.book) fail("invalidSection");
  const recordCount = view.getUint32(16, true);
  const directoryCount = view.getUint16(20, true);
  if (view.getUint16(22, true) !== DIRECTORY_ENTRY_LEN) fail("invalidSection");
  if (u64(view, 32) !== bytes.byteLength) fail("invalidSection");

  const payloadStart = SECTION_HEADER_LEN + directoryCount * DIRECTORY_ENTRY_LEN;
  if (payloadStart > bytes.byteLength) fail("truncated");
  const fields = new Map();
  for (let index = 0; index < directoryCount; index += 1) {
    const at = SECTION_HEADER_LEN + index * DIRECTORY_ENTRY_LEN;
    const id = view.getUint16(at, true);
    const width = view.getUint8(at + 2);
    if (width !== ELEMENT_WIDTH_VARIABLE && !ELEMENT_WIDTHS.includes(width)) fail("invalidSection");
    const offset = view.getUint32(at + 4, true);
    const byteLen = view.getUint32(at + 8, true);
    const count = view.getUint32(at + 12, true);
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
  const offsetsLen = field.count * 4;
  if (field.byteLen < offsetsLen) fail("truncated");
  const view = viewOf(field.bytes);
  const data = field.bytes.subarray(offsetsLen);
  const starts = new Array(field.count);
  let previous = 0;
  for (let index = 0; index < field.count; index += 1) {
    const start = view.getUint32(index * 4, true);
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
  const spanStarts = viewOf(requiredField(section, FIELD.spanStart, rowCount).bytes);
  const spanEnds = viewOf(requiredField(section, FIELD.spanEnd, rowCount).bytes);
  const sidIndices = viewOf(requiredField(section, FIELD.sidIndex, rowCount).bytes);
  const descriptorIndices = viewOf(
    requiredField(section, FIELD.markerDescriptorIndex, rowCount).bytes,
  );
  const sids = requiredField(section, FIELD.packedSidDictionary);
  const sidCount = sids.count;
  const sidView = viewOf(sids.bytes);
  const sidCache = new Array(sidCount).fill(null);

  const strings = readStringDictionary(section.fields.get(FIELD.stringDictionary));
  const descriptors = receipt.descriptors;
  const numbers = sparseColumn(section.fields.get(FIELD.numberRecords), NUMBER_RECORD_LEN);
  const bookCodes = sparseColumn(section.fields.get(FIELD.bookCodeRecords), BOOK_CODE_RECORD_LEN);
  const attributes = attributeColumn(section.fields.get(FIELD.attributeRecords));

  // Explicit ids and their dictionary are present together or absent together;
  // the section flag asserts which, and the two must agree.
  const idIndexField = section.fields.get(FIELD.tokenIdIndex);
  if (section.positionalIds !== !idIndexField) fail("invalidSection");
  const idIndices = idIndexField ? viewOf(requiredField(section, FIELD.tokenIdIndex, rowCount).bytes) : null;
  const idStrings = idIndexField
    ? readStringDictionary(section.fields.get(FIELD.tokenIdDictionary))
    : null;
  if (idIndexField && idStrings === null) fail("invalidSection");

  // `assign_ids`' rule, reproduced: the book label is the nearest preceding
  // book-code token's own text, defaulting to the first one in the stream (or
  // "unknown" when the book has none at all).
  const bookCodeText = (index) => {
    const at = index * BOOK_CODE_RECORD_LEN + 4;
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
    const chapter = sidView.getUint16(at + 3, true);
    const verse = sidView.getUint16(at + 5, true);
    const delta = sidView.getUint8(at + 7) & SID_DELTA_MASK;
    const locator = delta === 0 ? `${verse}` : `${verse}-${verse + delta}`;
    const text = `${ascii(sids.bytes, at, 3)} ${chapter}:${locator}`;
    sidCache[index] = text;
    return text;
  };
  const sidChapterAt = (index) => {
    if (index >= sidCount) fail("invalidSection");
    return sidView.getUint16(index * PACKED_SID_LEN + 3, true);
  };

  const sourceText = (start, end) => {
    if (end < start || end > source.byteLength) fail("invalidSection");
    return decodeUtf8(source.subarray(start, end));
  };

  const materializeRow = (row) => {
    const tag = kinds[row];
    const wireKind = TOKEN_KIND_WIRE[tag];
    if (wireKind === undefined) fail("invalidDiscriminant");
    const start = spanStarts.getUint32(row * 4, true);
    const end = spanEnds.getUint32(row * 4, true);
    const token = {
      id: `${bookLabelAt(row)}-${row}`,
      kind: wireKind,
      source: sourceText(start, end),
      span: { start, end },
    };
    const sid = sidAt(sidIndices.getUint16(row * 2, true));
    if (sid !== null) token.sid = sid;

    const descriptorIndex = descriptorIndices.getUint16(row * 2, true);
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
      const kind = NUMBER_RANGE_KIND_WIRE[numbers.view.getUint8(at + 12)];
      if (kind === undefined) fail("invalidDiscriminant");
      const info = { start: numbers.view.getUint32(at + 4, true), kind };
      if ((numbers.view.getUint8(at + 13) & NUMBER_FLAG_HAS_END) !== 0) {
        info.end = numbers.view.getUint32(at + 8, true);
      }
      token.numberInfo = info;
    } else if (tag === TOKEN_KIND.BookCode) {
      const index = bookCodes.lowerBound(row);
      if (index >= bookCodes.count || bookCodes.tokenIdxAt(index) !== row) fail("invalidSection");
      token.bookCode = bookCodeText(index);
      token.bookCodeValid =
        (bookCodes.view.getUint8(index * BOOK_CODE_RECORD_LEN + 8) & BOOK_CODE_FLAG_VALID) !== 0;
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
          const id = idStrings[idIndices.getUint32(row * 4, true)];
          if (id === undefined || id === "") fail("invalidSection");
          return id;
        }
      : null,
    /**
     * First and last row anchored to `chapter`, inclusive. Reads only the 2-byte
     * SID index column, so locating a viewport costs no token materialization.
     */
    chapterRange(chapter) {
      let first = -1;
      let last = -1;
      for (let row = 0; row < rowCount; row += 1) {
        const index = sidIndices.getUint16(row * 2, true);
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
  const first = view.getUint32(at + 4, true);
  const count = view.getUint32(at + 8, true);
  const listStart = view.getUint32(at + 12, true);
  const listLen = view.getUint32(at + 16, true);
  if (first + count > attributes.entryCount) fail("invalidSection");
  if (listStart !== SPAN_ABSENT) {
    token.attributeSource = sourceText(listStart, listStart + listLen);
  } else if (listLen !== 0) {
    fail("invalidSection");
  }
  if (count === 0) return;
  const items = new Array(count);
  for (let offset = 0; offset < count; offset += 1) {
    const entry = (first + offset) * ATTRIBUTE_ENTRY_LEN;
    const key = strings[attributes.entryView.getUint32(entry, true)];
    const value = strings[attributes.entryView.getUint32(entry + 4, true)];
    if (key === undefined || value === undefined) fail("invalidSection");
    const start = attributes.entryView.getUint32(entry + 8, true);
    const length = attributes.entryView.getUint32(entry + 12, true);
    items[offset] = {
      span: { start, end: start + length },
      text: sourceText(start, start + length),
      key,
      value: decodeAttributeValue(value),
      isDefault:
        (attributes.entryView.getUint8(entry + 16) & ATTRIBUTE_FLAG_DEFAULT) !== 0,
    };
  }
  token.attributes = items;
}

// --- public surface ---------------------------------------------------------

/**
 * Verifies every record through the Rust trust boundary and mints the branded
 * `VerifiedPacked` handle.
 *
 * The first rejected record short-circuits the whole corpus: a partially
 * restored corpus is not a state the caller asked for, and a typed rejection is
 * the signal to fall back to normal USFM ingest/parse. The caller's own
 * `Uint8Array`s are carried through, never copied.
 *
 * @param {{ verifyPackedBook: (packed: Uint8Array, source: Uint8Array) => any }} wasm
 * @param {readonly { path: string, packed: Uint8Array, source: Uint8Array }[]} records
 */
export function verifyPackedCorpus(wasm, records) {
  const books = new Map();
  const findings = new Map();
  for (const record of records) {
    const outcome = wasm.verifyPackedBook(record.packed, record.source);
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
    books.set(record.path, {
      path: record.path,
      packed: record.packed,
      source: record.source,
      receipt: outcome.receipt,
    });
    findings.set(record.path, outcome.findings);
  }
  return { ok: true, verified: { [VERIFIED]: true, books }, findings };
}

function requireVerified(verified) {
  if (!verified || verified[VERIFIED] !== true) {
    fail("invalidSection", "materialize accepts only a VerifiedPacked handle");
  }
  return verified.books;
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
  if (chapter !== undefined) out.range = range;
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
 * @returns {Map<string, { path: string, book: string, tokens: object[], stableIds?: string[], range?: { start: number, end: number } }>}
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
