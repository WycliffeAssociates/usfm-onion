// GENERATED FILE — DO NOT EDIT.
// Regenerate with: cargo run --example generate_js_schema -p usfm_onion_wire
// Source of truth: crates/usfm_onion_wire/src/schema.rs
//
// Why codegen: single-source from the compiled Rust schema constants so
// JS never hand-mirrors the contract; a drift test fails if this file
// diverges from schema.rs.
//
// Why not wasm-bindgen: this is static schema data, not a decoder.
// Reading it through wasm would mean instantiating the module and
// crossing a runtime boundary just for constants.
//
// Consumers: the byte-layout tables (field ids/widths, magics) are
// load-bearing production data for the official pure-JS `materialize`
// decoder, which decodes wasm-certified packed buffers directly in the
// JS engine (Rust/wasm remains the sole trust boundary — validation,
// XXH3 checksums, source binding; there is never a JS hash
// implementation). The semantic catalog (LINT_CODES, PARAM_CONTRACTS,
// rules version) additionally serves runtime JS like message
// localization. Both decoders are held identical by golden vectors and
// a serde-JSON equivalence gate.

export declare const CONTAINER_MAGIC: string;
export declare const SECTION_MAGIC: string;
export declare const FORMAT_VERSION: number;
export declare const SECTION_VERSION: number;
export declare const TOKEN_SECTION_RULES_VERSION: number;
export declare const FINDING_SECTION_RULES_VERSION: number;
export declare const MARKER_CATALOG_STAMP: bigint;
export declare const CONTAINER_HEADER_LEN: number;
export declare const TOC_ENTRY_LEN: number;
export declare const SECTION_HEADER_LEN: number;
export declare const DIRECTORY_ENTRY_LEN: number;
export declare const SECTION_ALIGN: number;
export declare const CONTAINER_CHECKSUM_OFFSET: number;
export declare const SECTION_CHECKSUM_OFFSET: number;
export declare const CHECKSUM_OMITTED: number;
export declare const CONTAINER_RESERVED_OFFSET: number;
export declare const CONTAINER_RESERVED_LEN: number;
export declare const CONTAINER_FLAGS_KNOWN: number;
export declare const TOC_FLAGS_KNOWN: number;
export declare const SECTION_FLAG_POSITIONAL_IDS: number;
export declare const TOKEN_SECTION_FLAGS_KNOWN: number;
export declare const FINDING_SECTION_FLAGS_KNOWN: number;
export declare const FIELD_FLAG_REQUIRED: number;
export declare const ELEMENT_WIDTH_VARIABLE: number;
export declare const INDEX_NONE_U16: number;
export declare const INDEX_NONE_U32: number;
export declare const MAX_DISTINCT_SIDS: number;
export declare const DESCRIPTOR_RECORD_LEN: number;
export declare const NUMBER_RECORD_LEN: number;
export declare const BOOK_CODE_RECORD_LEN: number;
export declare const ATTRIBUTE_ROW_LEN: number;
export declare const ATTRIBUTE_ENTRY_LEN: number;
export declare const DESCRIPTOR_FLAG_NESTED: number;
export declare const NUMBER_FLAG_HAS_END: number;
export declare const BOOK_CODE_FLAG_VALID: number;
export declare const ATTRIBUTE_FLAG_DEFAULT: number;
export declare const SPAN_ABSENT: number;
export declare const MAX_MARKER_DESCRIPTORS: number;
export declare const PACKED_SID_LEN: number;
export declare const SID_FIDELITY_BIT: number;
export declare const STRING_DICTIONARY_ENTRY_LEN: number;
export declare const ELEMENT_WIDTHS: readonly number[];

export declare const FINDING_FLAG: Readonly<{
  anchorOnly: number;
  noAnchor: number;
  range: number;
  related: number;
  payload: number;
  fix: number;
  overflow: number;
}>;

export declare const TOKEN_KIND: Readonly<{
  Newline: 0;
  OptBreak: 1;
  Marker: 2;
  EndMarker: 3;
  Milestone: 4;
  MilestoneEnd: 5;
  BookCode: 6;
  Number: 7;
  Text: 8;
}>;

export declare const NUMBER_RANGE_KIND: Readonly<{
  Single: 0;
  Range: 1;
  Sequence: 2;
  SequenceWithRange: 3;
}>;

export declare const CONTAINER_HEADER_OFFSET: Readonly<{
  magic: number;
  formatVersion: number;
  headerLen: number;
  flags: number;
  sectionCount: number;
  tocOffset: number;
  checksum: number;
  snapshotId: number;
  reserved: number;
}>;

export declare const TOC_ENTRY_OFFSET: Readonly<{
  kind: number;
  book: number;
  sectionVersion: number;
  flags: number;
  offset: number;
  byteLen: number;
  sourceHash: number;
}>;

export declare const SECTION_HEADER_OFFSET: Readonly<{
  magic: number;
  formatVersion: number;
  rulesVersion: number;
  kind: number;
  flags: number;
  book: number;
  reserved: number;
  recordCount: number;
  directoryCount: number;
  directoryEntrySize: number;
  sourceHash: number;
  sectionLen: number;
  checksum: number;
  sourceLen: number;
  catalogStamp: number;
}>;

export declare const DIRECTORY_ENTRY_OFFSET: Readonly<{
  fieldId: number;
  elementWidth: number;
  flags: number;
  offset: number;
  byteLen: number;
  count: number;
}>;

export declare const PACKED_SID_OFFSET: Readonly<{
  book: number;
  chapter: number;
  verse: number;
  delta: number;
  verseOccurrence: number;
  chapterOccurrence: number;
  flags: number;
}>;

export declare const DESCRIPTOR_RECORD_OFFSET: Readonly<{
  nameIndex: number;
  flags: number;
}>;

export declare const NUMBER_RECORD_OFFSET: Readonly<{
  tokenIdx: number;
  start: number;
  end: number;
  kind: number;
  flags: number;
}>;

export declare const BOOK_CODE_RECORD_OFFSET: Readonly<{
  tokenIdx: number;
  codeIndex: number;
  flags: number;
}>;

export declare const ATTRIBUTE_ROW_OFFSET: Readonly<{
  tokenIdx: number;
  firstEntry: number;
  entryCount: number;
  listStart: number;
  listLen: number;
}>;

export declare const ATTRIBUTE_ENTRY_OFFSET: Readonly<{
  keyIndex: number;
  valueIndex: number;
  spanStart: number;
  spanLen: number;
  flags: number;
}>;

export declare const TOKEN_KIND_WIRE: readonly string[];

export declare const NUMBER_RANGE_KIND_WIRE: readonly string[];

export declare const SECTION_KIND: Readonly<{ Token: 0; Finding: 1 }>;

export declare const TOKEN_FIELD: readonly Readonly<{ id: number; name: string; elementWidth: number | null; required: boolean }>[];

export declare const FINDING_FIELD: readonly Readonly<{ id: number; name: string; elementWidth: number | null; required: boolean }>[];

export declare const LINT_CODES: readonly Readonly<{ code: number; kebab: string }>[];

export declare const PARAM_CONTRACTS: readonly Readonly<{ code: number; variants: readonly Readonly<{ params: readonly Readonly<{ key: string; allowedValues: readonly string[] }>[] }>[] }>[];
