//! Frozen v1 wire schema constants.
//!
//! Every value here is frozen by `plans/approved/braid/phase0-freeze.md`
//! (assignment rule: declaration/text order, ascending from 0, append-only
//! forever, tombstone-on-removal). Layout offsets and widths come from the
//! container specification the freeze document numbers. Nothing in this module
//! may be renumbered: a wire integer that changes meaning silently reinterprets
//! every already-encoded container.
//!
//! v1 is little-endian only. There is no endianness flag and no byte-order
//! heuristic: a big-endian producer fails the magic/version checks, which is
//! the intended rejection path.

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

/// Container header size. Fixed at 32 in v1, and also written into the header's
/// own `header length` field so a future longer header stays self-describing.
pub const CONTAINER_HEADER_LEN: usize = 32;

/// TOC entry size. Fixed in v1; not self-described, so it can only change with
/// the format version.
pub const TOC_ENTRY_LEN: usize = 32;

/// Section header size, excluding the field directory that follows it.
pub const SECTION_HEADER_LEN: usize = 48;

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

/// Container header `flags:u32`. No container-level flag is defined in v1;
/// every set bit therefore rejects.
pub const CONTAINER_FLAGS_KNOWN: u32 = 0;

/// TOC entry `flags:u16`. Kind-specific by spec, but v1 defines none for
/// either kind.
pub const TOC_FLAGS_KNOWN: u16 = 0;

/// Section header `flags:u8` bit 0 — ids are positional (`{book}-{index}`), so
/// the explicit id column and its dictionary are omitted and the decoder
/// synthesizes ids from book + row. Location adjudicated in the freeze
/// document's §3.2 (section header, not the container or TOC flags field).
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
/// (string dictionaries, sparse keyed records — the freeze document's field
/// tables call these "mixed"). `byte_len` is authoritative for such a field
/// and no alignment is implied.
pub const ELEMENT_WIDTH_VARIABLE: u8 = 0;

/// Uniform column widths a v1 field entry may declare, besides
/// [`ELEMENT_WIDTH_VARIABLE`]. Anything else is a producer this build cannot
/// interpret.
pub const ELEMENT_WIDTHS: [u8; 4] = [1, 2, 4, 8];

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

    /// Field ids defined for this kind, and whether each is required. Unknown
    /// ids are not in this table and are resolved through
    /// [`FIELD_FLAG_REQUIRED`] instead.
    pub const fn field_table(self) -> &'static [(u16, bool)] {
        match self {
            Self::Token => token_field::TABLE,
            Self::Finding => finding_field::TABLE,
        }
    }
}

/// Token-section field ids. The id space is **per section kind**: the same
/// integer in a finding section names an unrelated field.
pub mod token_field {
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

    /// `(field_id, required)`, in frozen id order.
    pub const TABLE: &[(u16, bool)] = &[
        (KIND, true),
        (SPAN_START, true),
        (SPAN_END, true),
        (TOKEN_ID_INDEX, false),
        (SID_INDEX, true),
        (MARKER_DESCRIPTOR_INDEX, true),
        (NUMBER_RECORDS, false),
        (BOOK_CODE_RECORDS, false),
        (ATTRIBUTE_RECORDS, false),
        (TOKEN_ID_DICTIONARY, false),
        (STRING_DICTIONARY, true),
        (MARKER_DESCRIPTOR_DICTIONARY, true),
    ];

    /// Fields that exist only when ids are explicit; they are omitted together
    /// when the section sets `positional_ids`, and their presence alongside
    /// that flag is a contradiction, not a redundancy.
    pub const POSITIONAL_ID_EXCLUSIVE: [u16; 2] = [TOKEN_ID_INDEX, TOKEN_ID_DICTIONARY];
}

/// Finding-section field ids.
pub mod finding_field {
    pub const COMMON_ROW: u16 = 0;
    pub const RELATED_TOKEN_IDX: u16 = 1;
    pub const OVERFLOW_SPAN: u16 = 2;
    pub const MESSAGE_PAYLOAD_IDX: u16 = 3;
    pub const MARKER_STRING_IDX: u16 = 4;
    pub const PATCH_ID: u16 = 5;
    pub const PATCH_TABLE: u16 = 6;

    /// `(field_id, required)`, in frozen id order.
    pub const TABLE: &[(u16, bool)] = &[
        (COMMON_ROW, true),
        (RELATED_TOKEN_IDX, false),
        (OVERFLOW_SPAN, false),
        (MESSAGE_PAYLOAD_IDX, false),
        (MARKER_STRING_IDX, false),
        (PATCH_ID, false),
        (PATCH_TABLE, false),
    ];
}

/// Finding common-row `flags:u8` bits (finding record offset 14). Bit order is
/// the freeze document's §3.3 assignment.
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
            for (index, (field_id, _)) in table.iter().enumerate() {
                assert_eq!(usize::from(*field_id), index);
            }
        }
    }

    #[test]
    fn finding_flags_reserve_the_top_bit() {
        assert_eq!(finding_flag::KNOWN, 0x7f);
    }
}
