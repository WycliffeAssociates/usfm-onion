// GENERATED FILE — DO NOT EDIT.
// Regenerate with: cargo run --example generate_js_schema -p usfm_onion_wire
// Source of truth: crates/usfm_onion_wire/src/schema.rs

export const CONTAINER_MAGIC = "uson";
export const SECTION_MAGIC = "usos";
export const FORMAT_VERSION = 1;
export const SECTION_VERSION = 1;
export const TOKEN_SECTION_RULES_VERSION = 0;
export const FINDING_SECTION_RULES_VERSION = 1;
export const MARKER_CATALOG_STAMP = 11438527644260983257n;
export const CONTAINER_HEADER_LEN = 48;
export const TOC_ENTRY_LEN = 32;
export const SECTION_HEADER_LEN = 64;
export const DIRECTORY_ENTRY_LEN = 16;
export const SECTION_ALIGN = 16;
export const CONTAINER_CHECKSUM_OFFSET = 24;
export const SECTION_CHECKSUM_OFFSET = 40;
export const CHECKSUM_OMITTED = 0;
export const CONTAINER_RESERVED_OFFSET = 40;
export const CONTAINER_RESERVED_LEN = 8;
export const CONTAINER_FLAGS_KNOWN = 0;
export const TOC_FLAGS_KNOWN = 0;
export const SECTION_FLAG_POSITIONAL_IDS = 1;
export const TOKEN_SECTION_FLAGS_KNOWN = 1;
export const FINDING_SECTION_FLAGS_KNOWN = 0;
export const FIELD_FLAG_REQUIRED = 1;
export const ELEMENT_WIDTH_VARIABLE = 0;
export const INDEX_NONE_U16 = 65535;
export const INDEX_NONE_U32 = 4294967295;
export const MAX_DISTINCT_SIDS = 65535;
export const DESCRIPTOR_RECORD_LEN = 8;
export const NUMBER_RECORD_LEN = 16;
export const BOOK_CODE_RECORD_LEN = 16;
export const ATTRIBUTE_ROW_LEN = 24;
export const ATTRIBUTE_ENTRY_LEN = 20;
export const DESCRIPTOR_FLAG_NESTED = 1;
export const NUMBER_FLAG_HAS_END = 1;
export const BOOK_CODE_FLAG_VALID = 1;
export const ATTRIBUTE_FLAG_DEFAULT = 1;
export const SPAN_ABSENT = 4294967295;
export const MAX_MARKER_DESCRIPTORS = 65535;
export const PACKED_SID_LEN = 8;
export const SID_FIDELITY_BIT = 128;
export const SID_DELTA_MASK = 127;
export const ELEMENT_WIDTHS = [1, 2, 4, 8, 16];

export const FINDING_FLAG = {
  anchorOnly: 1,
  noAnchor: 2,
  range: 4,
  related: 8,
  payload: 16,
  fix: 32,
  overflow: 64,
};

export const TOKEN_KIND = {
  Newline: 0,
  OptBreak: 1,
  Marker: 2,
  EndMarker: 3,
  Milestone: 4,
  MilestoneEnd: 5,
  BookCode: 6,
  Number: 7,
  Text: 8,
};

export const NUMBER_RANGE_KIND = {
  Single: 0,
  Range: 1,
  Sequence: 2,
  SequenceWithRange: 3,
};

export const SECTION_KIND = {
  Token: 0,
  Finding: 1,
};

export const TOKEN_FIELD = [
  { id: 0, name: "kind", elementWidth: 1, required: true },
  { id: 1, name: "spanStart", elementWidth: 4, required: true },
  { id: 2, name: "spanEnd", elementWidth: 4, required: true },
  { id: 3, name: "tokenIdIndex", elementWidth: 4, required: false },
  { id: 4, name: "sidIndex", elementWidth: 2, required: true },
  { id: 5, name: "markerDescriptorIndex", elementWidth: 2, required: true },
  { id: 6, name: "numberRecords", elementWidth: 16, required: false },
  { id: 7, name: "bookCodeRecords", elementWidth: 16, required: false },
  { id: 8, name: "attributeRecords", elementWidth: null, required: false },
  { id: 9, name: "tokenIdDictionary", elementWidth: null, required: false },
  { id: 10, name: "stringDictionary", elementWidth: null, required: true },
  { id: 11, name: "markerDescriptorDictionary", elementWidth: 8, required: true },
  { id: 12, name: "packedSidDictionary", elementWidth: 8, required: true },
];

export const FINDING_FIELD = [
  { id: 0, name: "commonRow", elementWidth: 16, required: true },
  { id: 1, name: "relatedTokenIdx", elementWidth: null, required: false },
  { id: 2, name: "overflowSpan", elementWidth: 8, required: false },
  { id: 3, name: "messagePayloadIdx", elementWidth: 4, required: false },
  { id: 4, name: "markerRef", elementWidth: 8, required: false },
  { id: 5, name: "patchId", elementWidth: 4, required: false },
  { id: 6, name: "patchTable", elementWidth: null, required: false },
  { id: 7, name: "stringDictionary", elementWidth: null, required: false },
  { id: 8, name: "messagePayloadTable", elementWidth: null, required: false },
];

export const LINT_CODES = [
  { code: 0, kebab: "missing-id-marker" },
  { code: 1, kebab: "duplicate-id-marker" },
  { code: 2, kebab: "id-marker-not-at-file-start" },
  { code: 3, kebab: "empty-paragraph" },
  { code: 4, kebab: "missing-chapter-number" },
  { code: 5, kebab: "missing-verse-number" },
  { code: 6, kebab: "verse-is-empty" },
  { code: 7, kebab: "unknown-token" },
  { code: 8, kebab: "unknown-marker" },
  { code: 9, kebab: "unknown-close-marker" },
  { code: 10, kebab: "content-before-first-chapter" },
  { code: 11, kebab: "verse-outside-explicit-paragraph" },
  { code: 12, kebab: "note-submarker-outside-note" },
  { code: 13, kebab: "metadata-outside-target" },
  { code: 14, kebab: "marker-not-valid-in-context" },
  { code: 15, kebab: "missing-milestone-self-close" },
  { code: 16, kebab: "stray-close-marker" },
  { code: 17, kebab: "misnested-close-marker" },
  { code: 18, kebab: "implicitly-closed-marker" },
  { code: 19, kebab: "unclosed-marker" },
  { code: 20, kebab: "duplicate-chapter-number" },
  { code: 21, kebab: "duplicate-verse-number" },
  { code: 22, kebab: "invalid-number-range" },
  { code: 23, kebab: "number-range-not-preceded-by-marker-expecting-number" },
  { code: 24, kebab: "missing-whitespace-before-marker" },
  { code: 25, kebab: "missing-horizontal-whitespace-after-marker-name" },
  { code: 26, kebab: "missing-tag-end-delimiter-after-marker" },
  { code: 27, kebab: "missing-content-space-after-close-marker" },
  { code: 28, kebab: "verse-in-section-or-other-paragraph" },
  { code: 29, kebab: "content-after-blank-marker" },
  { code: 30, kebab: "invalid-book-code" },
  { code: 31, kebab: "book-code-not-uppercase" },
];

export const PARAM_CONTRACTS = [
  { code: 3, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 7, variants: [
    { params: [
      { key: "text", allowedValues: [] },
    ] },
  ] },
  { code: 8, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 9, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 10, variants: [
    { params: [
      { key: "kind", allowedValues: ["paragraph", "verse"] },
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 12, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 13, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
      { key: "target", allowedValues: ["chapter", "verse"] },
    ] },
  ] },
  { code: 14, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
      { key: "context", allowedValues: ["scripture", "book-identification", "book-headers", "book-titles", "book-introduction", "book-introduction-end-titles", "book-chapter-label", "chapter-content", "peripheral", "peripheral-content", "peripheral-division", "chapter", "verse", "section", "para", "list", "table", "sidebar", "footnote", "cross-reference"] },
    ] },
  ] },
  { code: 15, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 16, variants: [
    { params: [
      { key: "form", allowedValues: ["milestone-end"] },
    ] },
    { params: [
      { key: "form", allowedValues: ["named"] },
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 17, variants: [
    { params: [
      { key: "has_expected", allowedValues: ["true"] },
      { key: "expected", allowedValues: [] },
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 18, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
      { key: "closer", allowedValues: [] },
    ] },
  ] },
  { code: 19, variants: [
    { params: [
      { key: "kind", allowedValues: ["note", "character", "other"] },
      { key: "marker", allowedValues: [] },
      { key: "location", allowedValues: ["at-eof", "at-boundary"] },
    ] },
  ] },
  { code: 20, variants: [
    { params: [
      { key: "chapter", allowedValues: [] },
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 21, variants: [
    { params: [
      { key: "verse", allowedValues: [] },
      { key: "chapter", allowedValues: [] },
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 22, variants: [
    { params: [
      { key: "found", allowedValues: [] },
      { key: "verse", allowedValues: [] },
      { key: "marker", allowedValues: [] },
      { key: "context", allowedValues: [] },
    ] },
  ] },
  { code: 24, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 25, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 26, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 27, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 28, variants: [
    { params: [
      { key: "category", allowedValues: ["section", "other"] },
    ] },
  ] },
  { code: 29, variants: [
    { params: [
      { key: "marker", allowedValues: [] },
    ] },
  ] },
  { code: 30, variants: [
    { params: [
      { key: "code", allowedValues: [] },
    ] },
  ] },
  { code: 31, variants: [
    { params: [
      { key: "code", allowedValues: [] },
      { key: "uppercase", allowedValues: [] },
    ] },
  ] },
];
