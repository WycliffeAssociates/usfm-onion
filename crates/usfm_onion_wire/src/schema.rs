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
    pub const RELATED_TOKEN_IDX: u16 = 1;
    pub const OVERFLOW_SPAN: u16 = 2;
    pub const MESSAGE_PAYLOAD_IDX: u16 = 3;
    pub const MARKER_STRING_IDX: u16 = 4;
    pub const PATCH_ID: u16 = 5;
    pub const PATCH_TABLE: u16 = 6;

    /// Field requirements and any fixed uniform width, in stable id order.
    pub const TABLE: &[FieldSpec] = &[
        FieldSpec {
            id: COMMON_ROW,
            element_width: Some(16),
            required: true,
        },
        FieldSpec {
            id: RELATED_TOKEN_IDX,
            element_width: None,
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
            id: MARKER_STRING_IDX,
            element_width: Some(4),
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
    ];
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
}
