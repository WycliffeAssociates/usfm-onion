//! Typed codec failures.
//!
//! Variant names and payloads are an append-only wire/API contract. Both enums
//! are expected-failure *values*: no panic, no string message as the
//! carrier of meaning, no Rust debug formatting exposed as API. `Display` text
//! is for logs only; callers match on variants.
//!
//! Which check maps to which `DecodeError` variant is a decoder decision, not a
//! frozen one, so the mapping is stated here once rather than rediscovered per
//! call site:
//!
//! - [`DecodeError::Truncated`] — a validated in-range offset/length pair
//!   reaches past the end of the buffer or the enclosing section.
//! - [`DecodeError::OffsetOverflow`] — the offset/length arithmetic itself
//!   overflows, or a value exceeds this platform's addressable range. Kept
//!   distinct from `Truncated` so an adversarial count is distinguishable from
//!   an honestly short buffer.
//! - [`DecodeError::InvalidToc`] — a TOC-level rule: misaligned or
//!   header-overlapping section offset, overlapping sections, duplicate
//!   `(kind, book)`, a finding section with no matching token section, or a
//!   book code that is well-formed UTF-8 but not a legal three-character code.
//! - [`DecodeError::InvalidSection`] — a header or directory that contradicts
//!   itself or its TOC entry: wrong declared header length, non-zero reserved
//!   bytes, wrong directory entry size, a section whose kind/book/source hash
//!   disagrees with its TOC entry, a duplicate or missing required field, a
//!   misaligned or overlapping field payload, or a field whose declared width
//!   and byte length disagree with its count.
//! - [`DecodeError::UnsupportedFlags`] — any set flag bit not defined in v1, at
//!   the container, TOC-entry, section-header, or field-entry level. The frozen
//!   payload is `u32`, so narrower flag fields widen into it.

use usfm_onion::token::BookId;

use crate::schema::SectionKind;

/// Every way a container can fail to decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    BadMagic,
    UnsupportedVersion { found: u16 },
    UnsupportedFlags { found: u32 },
    InvalidToc,
    InvalidSection,
    InvalidUtf8,
    InvalidDiscriminant,
    OffsetOverflow,
    TooManySids { found: u32 },
    ChecksumMismatch,
    CatalogMismatch,
    SourceLengthMismatch,
    SourceHashMismatch,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => f.write_str("wire buffer ends inside a declared range"),
            Self::BadMagic => f.write_str("wire buffer does not start with the expected magic"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported wire layout version {found}")
            }
            Self::UnsupportedFlags { found } => {
                write!(
                    f,
                    "wire flags {found:#x} contain bits undefined in this version"
                )
            }
            Self::InvalidToc => f.write_str("wire table of contents is not canonical"),
            Self::InvalidSection => {
                f.write_str("wire section header or field directory is invalid")
            }
            Self::InvalidUtf8 => f.write_str("wire string data is not valid UTF-8"),
            Self::InvalidDiscriminant => f.write_str("wire discriminant is outside its known set"),
            Self::OffsetOverflow => f.write_str("wire offset or length arithmetic overflows"),
            Self::TooManySids { found } => {
                write!(
                    f,
                    "wire section declares {found} distinct sids, above the index ceiling"
                )
            }
            Self::ChecksumMismatch => f.write_str("wire integrity checksum does not match content"),
            Self::CatalogMismatch => {
                f.write_str("wire marker catalog stamp does not match this engine")
            }
            Self::SourceLengthMismatch => {
                f.write_str("supplied source length does not match the wire section")
            }
            Self::SourceHashMismatch => {
                f.write_str("supplied source hash does not match the wire section")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Every way an encode is refused. Encoders refuse; they never truncate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// More distinct SIDs in one book than the `u16` index column can name.
    TooManySids { book: BookId, found: u32 },
    /// A finding payload or token shape this schema version cannot express.
    UnrepresentablePayload { book: BookId, code: u8 },
    /// More distinct marker descriptors in one book than the `u16` index column
    /// can name.
    TooManyDescriptors { book: BookId, found: u32 },
    /// A token whose recorded span does not name its own text in the supplied
    /// source, or an attribute whose verbatim source does not bind there. Writing
    /// it would produce a section that decodes to different bytes than it was
    /// built from, so the encoder refuses. Reachable only for
    /// synthetically-built tokens; a serialize-then-encode path cannot hit it.
    UnboundSpan { book: BookId, token_idx: u32 },
    /// A section set this writer refuses to lay out, because the result would
    /// be a container its own reader rejects as non-canonical.
    ///
    /// Appended after the two payload refusals; no existing variant is
    /// renumbered.
    InvalidSectionLayout { book: BookId, reason: LayoutRefusal },
    /// A `TokenFix` that edits nothing — no replacement, no insertion, no
    /// deletion.
    ///
    /// Semantically a no-op fix, and no rule produces one. It is refused rather
    /// than stored because the patch table addresses a fix by a *run* of rows: a
    /// zero-row run is indistinguishable from the next fix's run, so writing one
    /// would produce a table this format's own decoder cannot read back as the
    /// fix that went in.
    EmptyFix { book: BookId, code: u8 },
}

/// Why a section set cannot be laid out canonically. Each variant mirrors a
/// check the reader performs, so the writer can never emit a container its own
/// reader would reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutRefusal {
    /// Two sections share a `(kind, book)` key. Corpus validation is supposed to
    /// have rejected duplicate books already; the writer refuses rather than
    /// emitting a container that fails its own TOC validation.
    DuplicateSection { kind: SectionKind },
    /// A finding section names no token section with the same book *and* source
    /// hash, which is the pairing key, so the pair a reader looks for would not
    /// exist.
    OrphanFindingSection,
    /// Two field entries share a field id.
    DuplicateField { field_id: u16 },
    /// A uniform-width field's byte length does not equal `count * width`.
    FieldExtentMismatch { field_id: u16 },
    /// A field the schema marks required for this section kind is absent.
    MissingRequiredField { field_id: u16 },
    /// Bytes offered back for reuse are not a readable section: truncated, not a
    /// section at all, or their integrity checksum does not hold. Reuse never
    /// means trust, so this is a refusal of the whole publication rather than a
    /// silent re-encode.
    CachedSectionUnreadable,
    /// Bytes offered back for reuse are a readable section describing a different
    /// book, source, or kind than the caller claims they are.
    CachedSectionMismatch,
    /// A section claiming positional token ids also carries the explicit id
    /// column or its dictionary, which that flag asserts are omitted.
    PositionalIdConflict { field_id: u16 },
    /// The section does not fit the `u32` section-relative offset, byte length,
    /// and count fields of its own directory.
    SectionTooLarge,
    /// More field entries than the section header's `u16` directory count can
    /// name.
    TooManyFields,
    /// More sections than the container header's `u32` section count can name.
    TooManySections,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManySids { book, found } => {
                write!(
                    f,
                    "book {book} declares {found} distinct sids, above the index ceiling"
                )
            }
            Self::TooManyDescriptors { book, found } => {
                write!(
                    f,
                    "book {book} declares {found} marker descriptors, above the index ceiling"
                )
            }
            Self::UnboundSpan { book, token_idx } => {
                write!(
                    f,
                    "book {book} token {token_idx} has a span that does not bind to the supplied source"
                )
            }
            Self::UnrepresentablePayload { book, code } => {
                write!(
                    f,
                    "book {book} carries a payload for rule code {code} this schema cannot encode"
                )
            }
            Self::InvalidSectionLayout { book, reason } => {
                write!(f, "book {book} section layout refused: {reason}")
            }
            Self::EmptyFix { book, code } => {
                write!(
                    f,
                    "book {book} carries a fix for rule code {code} that edits nothing"
                )
            }
        }
    }
}

impl std::fmt::Display for LayoutRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateSection { kind } => write!(f, "duplicate {kind:?} section"),
            Self::OrphanFindingSection => {
                f.write_str("finding section without a matching token section")
            }
            Self::DuplicateField { field_id } => write!(f, "duplicate field {field_id}"),
            Self::FieldExtentMismatch { field_id } => {
                write!(
                    f,
                    "field {field_id} byte length disagrees with its count and width"
                )
            }
            Self::MissingRequiredField { field_id } => {
                write!(f, "missing required field {field_id}")
            }
            Self::PositionalIdConflict { field_id } => {
                write!(
                    f,
                    "field {field_id} is present despite positional token ids"
                )
            }
            Self::SectionTooLarge => {
                f.write_str("section exceeds the 32-bit field directory range")
            }
            Self::TooManyFields => f.write_str("more fields than the directory count can name"),
            Self::TooManySections => f.write_str("more sections than the container count can name"),
            Self::CachedSectionUnreadable => {
                f.write_str("a section offered for reuse is not a readable section")
            }
            Self::CachedSectionMismatch => {
                f.write_str("a section offered for reuse describes different content than claimed")
            }
        }
    }
}

impl std::error::Error for EncodeError {}
