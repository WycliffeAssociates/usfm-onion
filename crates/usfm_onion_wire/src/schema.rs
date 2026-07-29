//! Frozen v1 wire schema constants.
//!
//! Stable integer assignments use declaration order, ascend from zero, and are
//! append-only. Removed values are tombstoned rather than reused. A wire integer
//! that changes meaning would silently reinterpret every already-encoded
//! container.
//!
//! v1 is little-endian only. There is no endianness flag and no byte-order
//! heuristic: a big-endian producer fails the magic/version checks, which is
//! the intended rejection path.

use usfm_onion::lint::LintCode;
use usfm_onion::token::{NumberRangeKind, TokenKind};

/// Container magic, ASCII `uson` (container header offset 0).
pub const CONTAINER_MAGIC: [u8; 4] = *b"uson";

/// Section magic, ASCII `usos` (section header offset 0).
pub const SECTION_MAGIC: [u8; 4] = *b"usos";

/// The only container/section layout version this build encodes or decodes.
pub const FORMAT_VERSION: u16 = 1;

/// The only TOC-entry section version this build encodes or decodes.
pub const SECTION_VERSION: u16 = 1;

/// `rules_version` value required of token sections; findings carry the real
/// rule-catalog version instead.
pub const TOKEN_SECTION_RULES_VERSION: u16 = 0;

/// Current finding-catalog and message-parameter contract revision.
pub const FINDING_SECTION_RULES_VERSION: u16 = 1;

/// Container header size. Fixed at 48 in v1, and also written into the header's
/// own `header length` field so a future longer header stays self-describing.
pub const CONTAINER_HEADER_LEN: usize = 48;

/// TOC entry size. Fixed in v1; not self-described, so it can only change with
/// the format version.
pub const TOC_ENTRY_LEN: usize = 32;

/// Section header size, excluding the field directory that follows it. A
/// multiple of [`SECTION_ALIGN`], which is what lets a section-relative field
/// offset carry the same alignment as its absolute offset.
pub const SECTION_HEADER_LEN: usize = 64;

/// Field directory entry size, written into the section header's `directory
/// entry size` field so a decoder can reject a mismatched producer before
/// walking the directory.
pub const DIRECTORY_ENTRY_LEN: usize = 16;

/// Top-level sections start at 16-byte-aligned absolute offsets. 16 is the
/// widest alignment any column needs, so one container-wide rule lets every
/// section-relative field offset be validated without knowing the section's
/// absolute placement.
pub const SECTION_ALIGN: u64 = 16;

/// Byte offset of the container integrity checksum inside the container
/// header. The checksum is computed with these 8 bytes read as zero.
pub const CONTAINER_CHECKSUM_OFFSET: usize = 24;

/// Byte offset of the section integrity checksum inside a section header,
/// relative to the section start. Same zero-the-hole rule as the container.
pub const SECTION_CHECKSUM_OFFSET: usize = 40;

/// A zero integrity checksum means "omitted", not "hash of zeros" — reserved
/// for transient output the API explicitly requests unchecked.
pub const CHECKSUM_OMITTED: u64 = 0;

/// Container-header reserved range (offset 40, 8 bytes). Reserved bytes must be
/// zero on read: accepting nonzero would let a later version's field pass
/// silently through a build that cannot honour it.
pub const CONTAINER_RESERVED_OFFSET: usize = 40;
pub const CONTAINER_RESERVED_LEN: usize = 8;

/// Container header `flags:u32`. No container-level flag is defined in v1;
/// every set bit therefore rejects.
pub const CONTAINER_FLAGS_KNOWN: u32 = 0;

/// TOC entry `flags:u16`. Kind-specific by spec, but v1 defines none for
/// either kind.
pub const TOC_FLAGS_KNOWN: u16 = 0;

/// Section header `flags:u8` bit 0 — ids are positional (`{book}-{index}`), so
/// the explicit id column and its dictionary are omitted and the decoder
/// synthesizes ids from book + row. This belongs to the section header because
/// it changes one token section, not the container as a whole.
pub const SECTION_FLAG_POSITIONAL_IDS: u8 = 1 << 0;

/// Token sections may only set [`SECTION_FLAG_POSITIONAL_IDS`]; finding
/// sections define no flag in v1.
pub const TOKEN_SECTION_FLAGS_KNOWN: u8 = SECTION_FLAG_POSITIONAL_IDS;
pub const FINDING_SECTION_FLAGS_KNOWN: u8 = 0;

/// Field directory entry `flags:u8` bit 0 — the field is required for a
/// correct reading of this section. It is the only signal that lets a decoder
/// honour "unknown required field ids reject, unknown optional field ids are
/// skipped" for an id it has never heard of.
pub const FIELD_FLAG_REQUIRED: u8 = 1 << 0;

/// Field directory entry flags defined in v1.
pub const FIELD_FLAGS_KNOWN: u8 = FIELD_FLAG_REQUIRED;

/// `element_width` value for a field whose payload is not a uniform array
/// (string dictionaries and sparse keyed records). `byte_len` is authoritative
/// for such a field and no alignment is implied.
pub const ELEMENT_WIDTH_VARIABLE: u8 = 0;

/// Uniform column widths a v1 field entry may declare, besides
/// [`ELEMENT_WIDTH_VARIABLE`]. Anything else is a producer this build cannot
/// interpret.
pub const ELEMENT_WIDTHS: [u8; 5] = [1, 2, 4, 8, 16];

/// One known field-directory entry. `element_width = None` means the semantic
/// field codec owns its mixed record shape; the generic container still checks
/// its declared width, extent, range, alignment, and overlap.
///
/// A field whose frozen record shape is a uniform array declares that width
/// here, so `count * width == byte_len` is enforced generically instead of once
/// per codec. Only genuinely mixed payloads — the string dictionaries, whose
/// bytes are an offset array followed by character data, and the attribute
/// records, which are two arrays of different record sizes — are `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpec {
    pub id: u16,
    pub element_width: Option<u8>,
    pub required: bool,
}

/// `sid_index` / `marker_descriptor_index` "none" sentinel. Because the
/// sentinel consumes a value, a book with more than [`MAX_DISTINCT_SIDS`]
/// distinct SIDs cannot be encoded at all.
pub const INDEX_NONE_U16: u16 = 0xffff;

/// `token_idx` "anchor-only finding" and `patch_id` "no patch" sentinel.
pub const INDEX_NONE_U32: u32 = u32::MAX;

/// Distinct SIDs a single book may reference, capped by the `u16` index column
/// minus its sentinel. Real scripture peaks around 2,600 per book, so crossing
/// this is a loud refusal for structurally legal non-scriptural input rather
/// than a reason to widen the column.
pub const MAX_DISTINCT_SIDS: u32 = INDEX_NONE_U16 as u32;

/// Fixed record widths and flag bits for the mixed and sparse token payloads.
/// Every value is frozen by the framing specification; a change here silently
/// reinterprets already-encoded sections.
pub const DESCRIPTOR_RECORD_LEN: usize = 8;
pub const NUMBER_RECORD_LEN: usize = 16;
pub const BOOK_CODE_RECORD_LEN: usize = 16;
pub const ATTRIBUTE_ROW_LEN: usize = 24;
pub const ATTRIBUTE_ENTRY_LEN: usize = 20;

/// Marker descriptor `flags:u8` bit 0 — the occurrence was written nested
/// (`\+add`). Carried on the descriptor, so the dictionary keys on
/// `(name, nested)`.
pub const DESCRIPTOR_FLAG_NESTED: u8 = 1 << 0;
pub const DESCRIPTOR_FLAGS_KNOWN: u8 = DESCRIPTOR_FLAG_NESTED;

/// Number record `flags:u8` bit 0 — the range end is meaningful. When clear the
/// end field must be zero, so an absent end has exactly one encoding.
pub const NUMBER_FLAG_HAS_END: u8 = 1 << 0;
pub const NUMBER_FLAGS_KNOWN: u8 = NUMBER_FLAG_HAS_END;

/// Book-code record `flags:u8` bit 0. Stored rather than recomputed: the
/// canonical book list is not covered by the marker-catalog stamp, so deriving
/// it on decode could silently rewrite an already-encoded token.
pub const BOOK_CODE_FLAG_VALID: u8 = 1 << 0;

/// Attribute entry `flags:u8` bit 0 — USFM 3.1 default-attribute shorthand. The
/// key is empty whenever it is set.
pub const ATTRIBUTE_FLAG_DEFAULT: u8 = 1 << 0;

/// Sentinel offset meaning "this span is absent", distinct from a present empty
/// span. Used by the attribute row's whole-list source, which core models as an
/// `Option` and which must round-trip as one.
pub const SPAN_ABSENT: u32 = u32::MAX;

/// Marker descriptors a single section may hold, capped by the `u16`
/// `marker_descriptor_index` column minus its sentinel.
pub const MAX_MARKER_DESCRIPTORS: u32 = INDEX_NONE_U16 as u32;

/// Packed SID dictionary record width and final-byte bit allocation.
pub const PACKED_SID_LEN: usize = 8;
pub const SID_FIDELITY_BIT: u8 = 1 << 7;
pub const SID_DELTA_MASK: u8 = SID_FIDELITY_BIT - 1;

/// Stable token-row discriminant, separate from Rust's enum layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKindTag {
    Newline = 0,
    OptBreak = 1,
    Marker = 2,
    EndMarker = 3,
    Milestone = 4,
    MilestoneEnd = 5,
    BookCode = 6,
    Number = 7,
    Text = 8,
}

impl TokenKindTag {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Newline),
            1 => Some(Self::OptBreak),
            2 => Some(Self::Marker),
            3 => Some(Self::EndMarker),
            4 => Some(Self::Milestone),
            5 => Some(Self::MilestoneEnd),
            6 => Some(Self::BookCode),
            7 => Some(Self::Number),
            8 => Some(Self::Text),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<TokenKind> for TokenKindTag {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Newline => Self::Newline,
            TokenKind::OptBreak => Self::OptBreak,
            TokenKind::Marker => Self::Marker,
            TokenKind::EndMarker => Self::EndMarker,
            TokenKind::Milestone => Self::Milestone,
            TokenKind::MilestoneEnd => Self::MilestoneEnd,
            TokenKind::BookCode => Self::BookCode,
            TokenKind::Number => Self::Number,
            TokenKind::Text => Self::Text,
        }
    }
}

impl From<TokenKindTag> for TokenKind {
    fn from(value: TokenKindTag) -> Self {
        match value {
            TokenKindTag::Newline => Self::Newline,
            TokenKindTag::OptBreak => Self::OptBreak,
            TokenKindTag::Marker => Self::Marker,
            TokenKindTag::EndMarker => Self::EndMarker,
            TokenKindTag::Milestone => Self::Milestone,
            TokenKindTag::MilestoneEnd => Self::MilestoneEnd,
            TokenKindTag::BookCode => Self::BookCode,
            TokenKindTag::Number => Self::Number,
            TokenKindTag::Text => Self::Text,
        }
    }
}

/// Stable number-payload discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NumberRangeKindTag {
    Single = 0,
    Range = 1,
    Sequence = 2,
    SequenceWithRange = 3,
}

impl NumberRangeKindTag {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Single),
            1 => Some(Self::Range),
            2 => Some(Self::Sequence),
            3 => Some(Self::SequenceWithRange),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<NumberRangeKind> for NumberRangeKindTag {
    fn from(value: NumberRangeKind) -> Self {
        match value {
            NumberRangeKind::Single => Self::Single,
            NumberRangeKind::Range => Self::Range,
            NumberRangeKind::Sequence => Self::Sequence,
            NumberRangeKind::SequenceWithRange => Self::SequenceWithRange,
        }
    }
}

impl From<NumberRangeKindTag> for NumberRangeKind {
    fn from(value: NumberRangeKindTag) -> Self {
        match value {
            NumberRangeKindTag::Single => Self::Single,
            NumberRangeKindTag::Range => Self::Range,
            NumberRangeKindTag::Sequence => Self::Sequence,
            NumberRangeKindTag::SequenceWithRange => Self::SequenceWithRange,
        }
    }
}

/// Top-level section kind (TOC entry offset 0, section header offset 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SectionKind {
    Token = 0,
    Finding = 1,
}

impl SectionKind {
    /// Returns `None` for any value outside the frozen table; callers turn that
    /// into a typed decode rejection rather than a fallback kind.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Token),
            1 => Some(Self::Finding),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Section-header flag bits this kind may set.
    pub const fn known_section_flags(self) -> u8 {
        match self {
            Self::Token => TOKEN_SECTION_FLAGS_KNOWN,
            Self::Finding => FINDING_SECTION_FLAGS_KNOWN,
        }
    }

    /// Field ids, fixed widths where applicable, and requiredness. Unknown ids
    /// are resolved through [`FIELD_FLAG_REQUIRED`] instead.
    pub const fn field_table(self) -> &'static [FieldSpec] {
        match self {
            Self::Token => token_field::TABLE,
            Self::Finding => finding_field::TABLE,
        }
    }
}

/// Token-section field ids. The id space is **per section kind**: the same
/// integer in a finding section names an unrelated field.
pub mod token_field {
    use super::FieldSpec;

    pub const KIND: u16 = 0;
    pub const SPAN_START: u16 = 1;
    pub const SPAN_END: u16 = 2;
    pub const TOKEN_ID_INDEX: u16 = 3;
    pub const SID_INDEX: u16 = 4;
    pub const MARKER_DESCRIPTOR_INDEX: u16 = 5;
    pub const NUMBER_RECORDS: u16 = 6;
    pub const BOOK_CODE_RECORDS: u16 = 7;
    pub const ATTRIBUTE_RECORDS: u16 = 8;
    pub const TOKEN_ID_DICTIONARY: u16 = 9;
    pub const STRING_DICTIONARY: u16 = 10;
    pub const MARKER_DESCRIPTOR_DICTIONARY: u16 = 11;
    /// Appended by the 2026-07-28 layout amendment: the eight-byte packed-SID
    /// records that [`SID_INDEX`] points into. Required even when empty, so a
    /// decoder never has to infer the dictionary's presence from index values.
    pub const PACKED_SID_DICTIONARY: u16 = 12;

    /// Field requirements and any fixed uniform width, in stable id order.
    pub const TABLE: &[FieldSpec] = &[
        FieldSpec {
            id: KIND,
            element_width: Some(1),
            required: true,
        },
        FieldSpec {
            id: SPAN_START,
            element_width: Some(4),
            required: true,
        },
        FieldSpec {
            id: SPAN_END,
            element_width: Some(4),
            required: true,
        },
        FieldSpec {
            id: TOKEN_ID_INDEX,
            element_width: Some(4),
            required: false,
        },
        FieldSpec {
            id: SID_INDEX,
            element_width: Some(2),
            required: true,
        },
        FieldSpec {
            id: MARKER_DESCRIPTOR_INDEX,
            element_width: Some(2),
            required: true,
        },
        FieldSpec {
            id: NUMBER_RECORDS,
            element_width: Some(16),
            required: false,
        },
        FieldSpec {
            id: BOOK_CODE_RECORDS,
            element_width: Some(16),
            required: false,
        },
        FieldSpec {
            id: ATTRIBUTE_RECORDS,
            element_width: None,
            required: false,
        },
        FieldSpec {
            id: TOKEN_ID_DICTIONARY,
            element_width: None,
            required: false,
        },
        FieldSpec {
            id: STRING_DICTIONARY,
            element_width: None,
            required: true,
        },
        FieldSpec {
            id: MARKER_DESCRIPTOR_DICTIONARY,
            element_width: Some(8),
            required: true,
        },
        FieldSpec {
            id: PACKED_SID_DICTIONARY,
            element_width: Some(super::PACKED_SID_LEN as u8),
            required: true,
        },
    ];

    /// Fields that exist only when ids are explicit; they are omitted together
    /// when the section sets `positional_ids`, and their presence alongside
    /// that flag is a contradiction, not a redundancy.
    pub const POSITIONAL_ID_EXCLUSIVE: [u16; 2] = [TOKEN_ID_INDEX, TOKEN_ID_DICTIONARY];
}

/// Finding-section field ids.
pub mod finding_field {
    use super::FieldSpec;

    pub const COMMON_ROW: u16 = 0;
    /// 16-byte record: `{token_idx: u32, offset: u32, len: u32, reserved: u32}`.
    /// Full `u32` width for offset and length, independent of the primary
    /// row's own `overflow` sidecar (which covers only the primary span, not
    /// this one). `reserved` MUST be zero on encode and MUST be rejected
    /// non-zero on decode.
    pub const RELATED_TOKEN_IDX: u16 = 1;
    pub const OVERFLOW_SPAN: u16 = 2;
    pub const MESSAGE_PAYLOAD_IDX: u16 = 3;
    /// Tagged 8-byte marker reference: `{tag:u8, span_len:u8, ordinal:u16,
    /// span_offset:u32}` over anchored-token / catalog-ordinal / source-span /
    /// explicitly-absent. Renamed from the original `u32` string-index field when
    /// the evidence showed no finding-section string dictionary is needed.
    pub const MARKER_REF: u16 = 4;
    pub const PATCH_ID: u16 = 5;
    pub const PATCH_TABLE: u16 = 6;
    /// Generic UTF-8 dictionary for message parameter keys and values.
    pub const STRING_DICTIONARY: u16 = 7;
    /// Message payload rows followed by their key/value pairs.
    pub const MESSAGE_PAYLOAD_TABLE: u16 = 8;

    /// Field requirements and any fixed uniform width, in stable id order.
    pub const TABLE: &[FieldSpec] = &[
        FieldSpec {
            id: COMMON_ROW,
            element_width: Some(16),
            required: true,
        },
        FieldSpec {
            id: RELATED_TOKEN_IDX,
            element_width: Some(16),
            required: false,
        },
        FieldSpec {
            id: OVERFLOW_SPAN,
            element_width: Some(8),
            required: false,
        },
        FieldSpec {
            id: MESSAGE_PAYLOAD_IDX,
            element_width: Some(4),
            required: false,
        },
        FieldSpec {
            id: MARKER_REF,
            element_width: Some(8),
            required: false,
        },
        FieldSpec {
            id: PATCH_ID,
            element_width: Some(4),
            required: false,
        },
        FieldSpec {
            id: PATCH_TABLE,
            element_width: None,
            required: false,
        },
        FieldSpec {
            id: STRING_DICTIONARY,
            element_width: None,
            required: false,
        },
        FieldSpec {
            id: MESSAGE_PAYLOAD_TABLE,
            element_width: None,
            required: false,
        },
    ];
}

/// Stable `LintCode` → `u8` discriminant, frozen by the Phase 0 freeze
/// (`phase0-freeze.md` §1) in `LintCode`'s current declaration order
/// (`src/lint_impl.rs`), starting at 0. Append-only, exactly like
/// [`TokenKindTag`]: a removed rule tombstones its integer rather than
/// reusing it. This is the discriminant table only — the finding-record codec
/// that reads/writes it is Phase B — but it lives here, not in a future
/// finding-codec module or a hand-copied list, so the eventual codec and the
/// generated JS/TS schema constants both read the one frozen mapping and
/// cannot drift from each other or from the freeze.
///
/// The kebab string, not the `u8`, is the canonical sort key (§2.2#15): a
/// decoder that sorted by `u8` would silently reorder findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LintCodeTag {
    MissingIdMarker = 0,
    DuplicateIdMarker = 1,
    IdMarkerNotAtFileStart = 2,
    EmptyParagraph = 3,
    MissingChapterNumber = 4,
    MissingVerseNumber = 5,
    VerseIsEmpty = 6,
    UnknownToken = 7,
    UnknownMarker = 8,
    UnknownCloseMarker = 9,
    ContentBeforeFirstChapter = 10,
    VerseOutsideExplicitParagraph = 11,
    NoteSubmarkerOutsideNote = 12,
    MetadataOutsideTarget = 13,
    MarkerNotValidInContext = 14,
    MissingMilestoneSelfClose = 15,
    StrayCloseMarker = 16,
    MisnestedCloseMarker = 17,
    ImplicitlyClosedMarker = 18,
    UnclosedMarker = 19,
    DuplicateChapterNumber = 20,
    DuplicateVerseNumber = 21,
    InvalidNumberRange = 22,
    NumberRangeNotPrecededByMarkerExpectingNumber = 23,
    MissingWhitespaceBeforeMarker = 24,
    MissingHorizontalWhitespaceAfterMarkerName = 25,
    MissingTagEndDelimiterAfterMarker = 26,
    MissingContentSpaceAfterCloseMarker = 27,
    VerseInSectionOrOtherParagraph = 28,
    ContentAfterBlankMarker = 29,
    InvalidBookCode = 30,
    BookCodeNotUppercase = 31,
}

impl LintCodeTag {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::MissingIdMarker),
            1 => Some(Self::DuplicateIdMarker),
            2 => Some(Self::IdMarkerNotAtFileStart),
            3 => Some(Self::EmptyParagraph),
            4 => Some(Self::MissingChapterNumber),
            5 => Some(Self::MissingVerseNumber),
            6 => Some(Self::VerseIsEmpty),
            7 => Some(Self::UnknownToken),
            8 => Some(Self::UnknownMarker),
            9 => Some(Self::UnknownCloseMarker),
            10 => Some(Self::ContentBeforeFirstChapter),
            11 => Some(Self::VerseOutsideExplicitParagraph),
            12 => Some(Self::NoteSubmarkerOutsideNote),
            13 => Some(Self::MetadataOutsideTarget),
            14 => Some(Self::MarkerNotValidInContext),
            15 => Some(Self::MissingMilestoneSelfClose),
            16 => Some(Self::StrayCloseMarker),
            17 => Some(Self::MisnestedCloseMarker),
            18 => Some(Self::ImplicitlyClosedMarker),
            19 => Some(Self::UnclosedMarker),
            20 => Some(Self::DuplicateChapterNumber),
            21 => Some(Self::DuplicateVerseNumber),
            22 => Some(Self::InvalidNumberRange),
            23 => Some(Self::NumberRangeNotPrecededByMarkerExpectingNumber),
            24 => Some(Self::MissingWhitespaceBeforeMarker),
            25 => Some(Self::MissingHorizontalWhitespaceAfterMarkerName),
            26 => Some(Self::MissingTagEndDelimiterAfterMarker),
            27 => Some(Self::MissingContentSpaceAfterCloseMarker),
            28 => Some(Self::VerseInSectionOrOtherParagraph),
            29 => Some(Self::ContentAfterBlankMarker),
            30 => Some(Self::InvalidBookCode),
            31 => Some(Self::BookCodeNotUppercase),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// The kebab-case wire/JS code string, e.g. `"missing-id-marker"`.
    pub const fn kebab(self) -> &'static str {
        match self {
            Self::MissingIdMarker => "missing-id-marker",
            Self::DuplicateIdMarker => "duplicate-id-marker",
            Self::IdMarkerNotAtFileStart => "id-marker-not-at-file-start",
            Self::EmptyParagraph => "empty-paragraph",
            Self::MissingChapterNumber => "missing-chapter-number",
            Self::MissingVerseNumber => "missing-verse-number",
            Self::VerseIsEmpty => "verse-is-empty",
            Self::UnknownToken => "unknown-token",
            Self::UnknownMarker => "unknown-marker",
            Self::UnknownCloseMarker => "unknown-close-marker",
            Self::ContentBeforeFirstChapter => "content-before-first-chapter",
            Self::VerseOutsideExplicitParagraph => "verse-outside-explicit-paragraph",
            Self::NoteSubmarkerOutsideNote => "note-submarker-outside-note",
            Self::MetadataOutsideTarget => "metadata-outside-target",
            Self::MarkerNotValidInContext => "marker-not-valid-in-context",
            Self::MissingMilestoneSelfClose => "missing-milestone-self-close",
            Self::StrayCloseMarker => "stray-close-marker",
            Self::MisnestedCloseMarker => "misnested-close-marker",
            Self::ImplicitlyClosedMarker => "implicitly-closed-marker",
            Self::UnclosedMarker => "unclosed-marker",
            Self::DuplicateChapterNumber => "duplicate-chapter-number",
            Self::DuplicateVerseNumber => "duplicate-verse-number",
            Self::InvalidNumberRange => "invalid-number-range",
            Self::NumberRangeNotPrecededByMarkerExpectingNumber => {
                "number-range-not-preceded-by-marker-expecting-number"
            }
            Self::MissingWhitespaceBeforeMarker => "missing-whitespace-before-marker",
            Self::MissingHorizontalWhitespaceAfterMarkerName => {
                "missing-horizontal-whitespace-after-marker-name"
            }
            Self::MissingTagEndDelimiterAfterMarker => "missing-tag-end-delimiter-after-marker",
            Self::MissingContentSpaceAfterCloseMarker => "missing-content-space-after-close-marker",
            Self::VerseInSectionOrOtherParagraph => "verse-in-section-or-other-paragraph",
            Self::ContentAfterBlankMarker => "content-after-blank-marker",
            Self::InvalidBookCode => "invalid-book-code",
            Self::BookCodeNotUppercase => "book-code-not-uppercase",
        }
    }
}

impl From<LintCode> for LintCodeTag {
    fn from(value: LintCode) -> Self {
        match value {
            LintCode::MissingIdMarker => Self::MissingIdMarker,
            LintCode::DuplicateIdMarker => Self::DuplicateIdMarker,
            LintCode::IdMarkerNotAtFileStart => Self::IdMarkerNotAtFileStart,
            LintCode::EmptyParagraph => Self::EmptyParagraph,
            LintCode::MissingChapterNumber => Self::MissingChapterNumber,
            LintCode::MissingVerseNumber => Self::MissingVerseNumber,
            LintCode::VerseIsEmpty => Self::VerseIsEmpty,
            LintCode::UnknownToken => Self::UnknownToken,
            LintCode::UnknownMarker => Self::UnknownMarker,
            LintCode::UnknownCloseMarker => Self::UnknownCloseMarker,
            LintCode::ContentBeforeFirstChapter => Self::ContentBeforeFirstChapter,
            LintCode::VerseOutsideExplicitParagraph => Self::VerseOutsideExplicitParagraph,
            LintCode::NoteSubmarkerOutsideNote => Self::NoteSubmarkerOutsideNote,
            LintCode::MetadataOutsideTarget => Self::MetadataOutsideTarget,
            LintCode::MarkerNotValidInContext => Self::MarkerNotValidInContext,
            LintCode::MissingMilestoneSelfClose => Self::MissingMilestoneSelfClose,
            LintCode::StrayCloseMarker => Self::StrayCloseMarker,
            LintCode::MisnestedCloseMarker => Self::MisnestedCloseMarker,
            LintCode::ImplicitlyClosedMarker => Self::ImplicitlyClosedMarker,
            LintCode::UnclosedMarker => Self::UnclosedMarker,
            LintCode::DuplicateChapterNumber => Self::DuplicateChapterNumber,
            LintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            LintCode::InvalidNumberRange => Self::InvalidNumberRange,
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber => {
                Self::NumberRangeNotPrecededByMarkerExpectingNumber
            }
            LintCode::MissingWhitespaceBeforeMarker => Self::MissingWhitespaceBeforeMarker,
            LintCode::MissingHorizontalWhitespaceAfterMarkerName => {
                Self::MissingHorizontalWhitespaceAfterMarkerName
            }
            LintCode::MissingTagEndDelimiterAfterMarker => Self::MissingTagEndDelimiterAfterMarker,
            LintCode::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            LintCode::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
            LintCode::ContentAfterBlankMarker => Self::ContentAfterBlankMarker,
            LintCode::InvalidBookCode => Self::InvalidBookCode,
            LintCode::BookCodeNotUppercase => Self::BookCodeNotUppercase,
        }
    }
}

impl From<LintCodeTag> for LintCode {
    fn from(value: LintCodeTag) -> Self {
        match value {
            LintCodeTag::MissingIdMarker => Self::MissingIdMarker,
            LintCodeTag::DuplicateIdMarker => Self::DuplicateIdMarker,
            LintCodeTag::IdMarkerNotAtFileStart => Self::IdMarkerNotAtFileStart,
            LintCodeTag::EmptyParagraph => Self::EmptyParagraph,
            LintCodeTag::MissingChapterNumber => Self::MissingChapterNumber,
            LintCodeTag::MissingVerseNumber => Self::MissingVerseNumber,
            LintCodeTag::VerseIsEmpty => Self::VerseIsEmpty,
            LintCodeTag::UnknownToken => Self::UnknownToken,
            LintCodeTag::UnknownMarker => Self::UnknownMarker,
            LintCodeTag::UnknownCloseMarker => Self::UnknownCloseMarker,
            LintCodeTag::ContentBeforeFirstChapter => Self::ContentBeforeFirstChapter,
            LintCodeTag::VerseOutsideExplicitParagraph => Self::VerseOutsideExplicitParagraph,
            LintCodeTag::NoteSubmarkerOutsideNote => Self::NoteSubmarkerOutsideNote,
            LintCodeTag::MetadataOutsideTarget => Self::MetadataOutsideTarget,
            LintCodeTag::MarkerNotValidInContext => Self::MarkerNotValidInContext,
            LintCodeTag::MissingMilestoneSelfClose => Self::MissingMilestoneSelfClose,
            LintCodeTag::StrayCloseMarker => Self::StrayCloseMarker,
            LintCodeTag::MisnestedCloseMarker => Self::MisnestedCloseMarker,
            LintCodeTag::ImplicitlyClosedMarker => Self::ImplicitlyClosedMarker,
            LintCodeTag::UnclosedMarker => Self::UnclosedMarker,
            LintCodeTag::DuplicateChapterNumber => Self::DuplicateChapterNumber,
            LintCodeTag::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            LintCodeTag::InvalidNumberRange => Self::InvalidNumberRange,
            LintCodeTag::NumberRangeNotPrecededByMarkerExpectingNumber => {
                Self::NumberRangeNotPrecededByMarkerExpectingNumber
            }
            LintCodeTag::MissingWhitespaceBeforeMarker => Self::MissingWhitespaceBeforeMarker,
            LintCodeTag::MissingHorizontalWhitespaceAfterMarkerName => {
                Self::MissingHorizontalWhitespaceAfterMarkerName
            }
            LintCodeTag::MissingTagEndDelimiterAfterMarker => {
                Self::MissingTagEndDelimiterAfterMarker
            }
            LintCodeTag::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            LintCodeTag::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
            LintCodeTag::ContentAfterBlankMarker => Self::ContentAfterBlankMarker,
            LintCodeTag::InvalidBookCode => Self::InvalidBookCode,
            LintCodeTag::BookCodeNotUppercase => Self::BookCodeNotUppercase,
        }
    }
}

/// All 32 `LintCode` variants in frozen `u8` order — the table the generated
/// JS/TS schema constants and any future finding catalog both iterate.
pub const LINT_CODE_TABLE: [LintCodeTag; 32] = [
    LintCodeTag::MissingIdMarker,
    LintCodeTag::DuplicateIdMarker,
    LintCodeTag::IdMarkerNotAtFileStart,
    LintCodeTag::EmptyParagraph,
    LintCodeTag::MissingChapterNumber,
    LintCodeTag::MissingVerseNumber,
    LintCodeTag::VerseIsEmpty,
    LintCodeTag::UnknownToken,
    LintCodeTag::UnknownMarker,
    LintCodeTag::UnknownCloseMarker,
    LintCodeTag::ContentBeforeFirstChapter,
    LintCodeTag::VerseOutsideExplicitParagraph,
    LintCodeTag::NoteSubmarkerOutsideNote,
    LintCodeTag::MetadataOutsideTarget,
    LintCodeTag::MarkerNotValidInContext,
    LintCodeTag::MissingMilestoneSelfClose,
    LintCodeTag::StrayCloseMarker,
    LintCodeTag::MisnestedCloseMarker,
    LintCodeTag::ImplicitlyClosedMarker,
    LintCodeTag::UnclosedMarker,
    LintCodeTag::DuplicateChapterNumber,
    LintCodeTag::DuplicateVerseNumber,
    LintCodeTag::InvalidNumberRange,
    LintCodeTag::NumberRangeNotPrecededByMarkerExpectingNumber,
    LintCodeTag::MissingWhitespaceBeforeMarker,
    LintCodeTag::MissingHorizontalWhitespaceAfterMarkerName,
    LintCodeTag::MissingTagEndDelimiterAfterMarker,
    LintCodeTag::MissingContentSpaceAfterCloseMarker,
    LintCodeTag::VerseInSectionOrOtherParagraph,
    LintCodeTag::ContentAfterBlankMarker,
    LintCodeTag::InvalidBookCode,
    LintCodeTag::BookCodeNotUppercase,
];

/// One message parameter. An empty `allowed_values` admits any UTF-8 string;
/// otherwise it is a closed semantic domain that both decoders must check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamSpec {
    pub key: &'static str,
    pub allowed_values: &'static [&'static str],
}

/// One exact map arm. More than one arm represents a discriminated union, not
/// an optional bag: the decoded map must match one arm's keys and domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamVariant {
    pub params: &'static [ParamSpec],
}

/// Rust owns the full validation data; generated JS consumes this same table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamContract {
    pub code: LintCodeTag,
    pub variants: &'static [ParamVariant],
}

impl ParamContract {
    pub fn accepts(self, params: &std::collections::BTreeMap<String, String>) -> bool {
        self.variants.iter().any(|variant| {
            params.len() == variant.params.len()
                && variant.params.iter().all(|spec| {
                    params.get(spec.key).is_some_and(|value| {
                        spec.allowed_values.is_empty()
                            || spec.allowed_values.contains(&value.as_str())
                    })
                })
        })
    }
}

macro_rules! contract {
    ($code:expr, [$(($key:expr, [$($value:expr),*])),* $(,)?]) => {
        ParamContract { code: $code, variants: &[ParamVariant { params: &[$(ParamSpec { key: $key, allowed_values: &[$($value),*] }),*] }] }
    };
}

/// In tag order. Empty-map codes are absent because field 3 must not be set.
pub const PARAM_CONTRACTS: &[ParamContract] = &[
    contract!(LintCodeTag::EmptyParagraph, [("marker", [])]),
    contract!(LintCodeTag::UnknownToken, [("text", [])]),
    contract!(LintCodeTag::UnknownMarker, [("marker", [])]),
    contract!(LintCodeTag::UnknownCloseMarker, [("marker", [])]),
    contract!(
        LintCodeTag::ContentBeforeFirstChapter,
        [("kind", ["paragraph", "verse"]), ("marker", [])]
    ),
    contract!(LintCodeTag::NoteSubmarkerOutsideNote, [("marker", [])]),
    contract!(
        LintCodeTag::MetadataOutsideTarget,
        [("marker", []), ("target", ["chapter", "verse"])]
    ),
    contract!(
        LintCodeTag::MarkerNotValidInContext,
        [
            ("marker", []),
            (
                "context",
                [
                    "scripture",
                    "book-identification",
                    "book-headers",
                    "book-titles",
                    "book-introduction",
                    "book-introduction-end-titles",
                    "book-chapter-label",
                    "chapter-content",
                    "peripheral",
                    "peripheral-content",
                    "peripheral-division",
                    "chapter",
                    "verse",
                    "section",
                    "para",
                    "list",
                    "table",
                    "sidebar",
                    "footnote",
                    "cross-reference"
                ]
            )
        ]
    ),
    contract!(LintCodeTag::MissingMilestoneSelfClose, [("marker", [])]),
    ParamContract {
        code: LintCodeTag::StrayCloseMarker,
        variants: &[
            ParamVariant {
                params: &[ParamSpec {
                    key: "form",
                    allowed_values: &["milestone-end"],
                }],
            },
            ParamVariant {
                params: &[
                    ParamSpec {
                        key: "form",
                        allowed_values: &["named"],
                    },
                    ParamSpec {
                        key: "marker",
                        allowed_values: &[],
                    },
                ],
            },
        ],
    },
    contract!(
        LintCodeTag::MisnestedCloseMarker,
        [("has_expected", ["true"]), ("expected", []), ("marker", [])]
    ),
    contract!(
        LintCodeTag::ImplicitlyClosedMarker,
        [("marker", []), ("closer", [])]
    ),
    contract!(
        LintCodeTag::UnclosedMarker,
        [
            ("kind", ["note", "character", "other"]),
            ("marker", []),
            ("location", ["at-eof", "at-boundary"])
        ]
    ),
    contract!(
        LintCodeTag::DuplicateChapterNumber,
        [("chapter", []), ("marker", [])]
    ),
    contract!(
        LintCodeTag::DuplicateVerseNumber,
        [("verse", []), ("chapter", []), ("marker", [])]
    ),
    contract!(
        LintCodeTag::InvalidNumberRange,
        [
            ("found", []),
            ("verse", []),
            ("marker", []),
            ("context", [])
        ]
    ),
    contract!(LintCodeTag::MissingWhitespaceBeforeMarker, [("marker", [])]),
    contract!(
        LintCodeTag::MissingHorizontalWhitespaceAfterMarkerName,
        [("marker", [])]
    ),
    contract!(
        LintCodeTag::MissingTagEndDelimiterAfterMarker,
        [("marker", [])]
    ),
    contract!(
        LintCodeTag::MissingContentSpaceAfterCloseMarker,
        [("marker", [])]
    ),
    contract!(
        LintCodeTag::VerseInSectionOrOtherParagraph,
        [("category", ["section", "other"])]
    ),
    contract!(LintCodeTag::ContentAfterBlankMarker, [("marker", [])]),
    contract!(LintCodeTag::InvalidBookCode, [("code", [])]),
    contract!(
        LintCodeTag::BookCodeNotUppercase,
        [("code", []), ("uppercase", [])]
    ),
];

pub const fn param_contract(code: LintCodeTag) -> Option<&'static ParamContract> {
    let mut index = 0;
    while index < PARAM_CONTRACTS.len() {
        if PARAM_CONTRACTS[index].code as u8 == code as u8 {
            return Some(&PARAM_CONTRACTS[index]);
        }
        index += 1;
    }
    None
}

/// Finding common-row `flags:u8` bits (finding record offset 14).
pub mod finding_flag {
    /// Set = `AnchorOnly` fidelity; clear = `Exact`.
    pub const ANCHOR_ONLY: u8 = 1 << 0;
    /// Set = the finding has no SID at all. Required because `(chapter 0,
    /// verse 0)` is itself a legal front-matter anchor and cannot double as
    /// "absent".
    pub const NO_ANCHOR: u8 = 1 << 1;
    /// Set = the row's range-end byte is meaningful (a verse bridge).
    pub const RANGE: u8 = 1 << 2;
    pub const RELATED: u8 = 1 << 3;
    pub const PAYLOAD: u8 = 1 << 4;
    pub const FIX: u8 = 1 << 5;
    /// Set = the overflow-span sidecar supersedes the row's `u16` offset and
    /// length for this finding.
    pub const OVERFLOW: u8 = 1 << 6;

    /// Bit 7 is reserved and zero in v1.
    pub const KNOWN: u8 = ANCHOR_ONLY | NO_ANCHOR | RANGE | RELATED | PAYLOAD | FIX | OVERFLOW;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_kind_round_trips_only_frozen_values() {
        assert_eq!(SectionKind::from_u8(0), Some(SectionKind::Token));
        assert_eq!(SectionKind::from_u8(1), Some(SectionKind::Finding));
        for value in 2..=u8::MAX {
            assert_eq!(SectionKind::from_u8(value), None);
        }
    }

    #[test]
    fn field_tables_are_dense_and_ordered() {
        // A gap or reorder here means a decoder and an encoder built from this
        // table would disagree about what an id means.
        for table in [token_field::TABLE, finding_field::TABLE] {
            for (index, field) in table.iter().enumerate() {
                assert_eq!(usize::from(field.id), index);
            }
        }
    }

    #[test]
    fn finding_flags_reserve_the_top_bit() {
        assert_eq!(finding_flag::KNOWN, 0x7f);
    }

    #[test]
    fn token_kind_tags_match_the_stable_table() {
        let kinds = [
            TokenKind::Newline,
            TokenKind::OptBreak,
            TokenKind::Marker,
            TokenKind::EndMarker,
            TokenKind::Milestone,
            TokenKind::MilestoneEnd,
            TokenKind::BookCode,
            TokenKind::Number,
            TokenKind::Text,
        ];
        for (value, kind) in kinds.into_iter().enumerate() {
            let tag = TokenKindTag::from(kind);
            assert_eq!(usize::from(tag.as_u8()), value);
            assert_eq!(
                TokenKind::from(TokenKindTag::from_u8(tag.as_u8()).unwrap()),
                kind
            );
        }
        assert_eq!(TokenKindTag::from_u8(9), None);
    }

    #[test]
    fn number_range_tags_match_the_stable_table() {
        let kinds = [
            NumberRangeKind::Single,
            NumberRangeKind::Range,
            NumberRangeKind::Sequence,
            NumberRangeKind::SequenceWithRange,
        ];
        for (value, kind) in kinds.into_iter().enumerate() {
            let tag = NumberRangeKindTag::from(kind);
            assert_eq!(usize::from(tag.as_u8()), value);
            assert_eq!(
                NumberRangeKind::from(NumberRangeKindTag::from_u8(tag.as_u8()).unwrap()),
                kind
            );
        }
        assert_eq!(NumberRangeKindTag::from_u8(4), None);
    }

    #[test]
    fn lint_code_tags_match_the_frozen_table() {
        // `From<LintCode> for LintCodeTag` is an exhaustive match, so a new
        // core `LintCode` variant fails to compile here until mirrored — the
        // same drift guard `dto.rs` documents for its own conversions. This
        // test additionally proves the declaration order the freeze assigned
        // (`phase0-freeze.md` §1) and that `LINT_CODE_TABLE` walks it in
        // ascending `u8` order.
        let codes = [
            LintCode::MissingIdMarker,
            LintCode::DuplicateIdMarker,
            LintCode::IdMarkerNotAtFileStart,
            LintCode::EmptyParagraph,
            LintCode::MissingChapterNumber,
            LintCode::MissingVerseNumber,
            LintCode::VerseIsEmpty,
            LintCode::UnknownToken,
            LintCode::UnknownMarker,
            LintCode::UnknownCloseMarker,
            LintCode::ContentBeforeFirstChapter,
            LintCode::VerseOutsideExplicitParagraph,
            LintCode::NoteSubmarkerOutsideNote,
            LintCode::MetadataOutsideTarget,
            LintCode::MarkerNotValidInContext,
            LintCode::MissingMilestoneSelfClose,
            LintCode::StrayCloseMarker,
            LintCode::MisnestedCloseMarker,
            LintCode::ImplicitlyClosedMarker,
            LintCode::UnclosedMarker,
            LintCode::DuplicateChapterNumber,
            LintCode::DuplicateVerseNumber,
            LintCode::InvalidNumberRange,
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
            LintCode::MissingWhitespaceBeforeMarker,
            LintCode::MissingHorizontalWhitespaceAfterMarkerName,
            LintCode::MissingTagEndDelimiterAfterMarker,
            LintCode::MissingContentSpaceAfterCloseMarker,
            LintCode::VerseInSectionOrOtherParagraph,
            LintCode::ContentAfterBlankMarker,
            LintCode::InvalidBookCode,
            LintCode::BookCodeNotUppercase,
        ];
        assert_eq!(codes.len(), 32);
        assert_eq!(LINT_CODE_TABLE.len(), 32);

        let mut seen_kebab = Vec::new();
        for (value, code) in codes.into_iter().enumerate() {
            let tag = LintCodeTag::from(code);
            assert_eq!(usize::from(tag.as_u8()), value);
            assert_eq!(LintCodeTag::from_u8(tag.as_u8()), Some(tag));
            assert_eq!(LintCode::from(tag), code);
            assert_eq!(LINT_CODE_TABLE[value], tag);
            assert!(!tag.kebab().is_empty());
            assert!(!seen_kebab.contains(&tag.kebab()));
            seen_kebab.push(tag.kebab());
        }
        assert_eq!(LintCodeTag::from_u8(32), None);
        assert_eq!(LintCodeTag::MissingIdMarker.kebab(), "missing-id-marker");
        assert_eq!(
            LintCodeTag::BookCodeNotUppercase.kebab(),
            "book-code-not-uppercase"
        );
    }

    #[test]
    fn parameter_contracts_have_exact_union_arms_and_closed_domains() {
        let named = param_contract(LintCodeTag::StrayCloseMarker).unwrap();
        let milestone = std::collections::BTreeMap::from([(
            String::from("form"),
            String::from("milestone-end"),
        )]);
        let named_close = std::collections::BTreeMap::from([
            (String::from("form"), String::from("named")),
            (String::from("marker"), String::from("p")),
        ]);
        assert!(named.accepts(&milestone));
        assert!(named.accepts(&named_close));
        assert!(!named.accepts(&std::collections::BTreeMap::from([(
            String::from("form"),
            String::from("named")
        )])));
        let context = param_contract(LintCodeTag::MarkerNotValidInContext).unwrap();
        assert!(context.accepts(&std::collections::BTreeMap::from([
            (String::from("marker"), String::from("p")),
            (String::from("context"), String::from("cross-reference")),
        ])));
        assert!(!context.accepts(&std::collections::BTreeMap::from([
            (String::from("marker"), String::from("p")),
            (String::from("context"), String::from("other")),
        ])));
    }
}
