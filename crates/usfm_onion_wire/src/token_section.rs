//! Checked views over the fixed token columns and the packed SID dictionary.
//!
//! This layer knows token semantics but does not yet decode dictionaries or
//! materialize core tokens. It is the narrow boundary between a structurally
//! valid generic section and token-specific readers.

use std::collections::BTreeMap;

use usfm_onion::token::{BookId, Sid};

use crate::container::{Section, SectionField};
use crate::error::{DecodeError, EncodeError};
use crate::schema::{
    INDEX_NONE_U16, MAX_DISTINCT_SIDS, PACKED_SID_LEN, SID_DELTA_MASK, SID_FIDELITY_BIT,
    SectionKind, TokenKindTag, token_field,
};
use crate::token_payload::StringDictionary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SidFidelity {
    Exact,
    AnchorOnly,
}

/// Eight-byte wire SID. Its representation is explicit rather than `repr(Rust)`
/// so core layout changes cannot silently alter persisted bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PackedSid([u8; PACKED_SID_LEN]);

impl PackedSid {
    pub(crate) fn encode(sid: Sid, requested_fidelity: SidFidelity) -> Self {
        // The caller derives requested_fidelity from exact number-token source;
        // core Sid alone cannot distinguish sequences or suffixed verses.
        let source_delta = sid.verse_end().saturating_sub(sid.verse);
        let (delta, fidelity) = if source_delta > u16::from(SID_DELTA_MASK) {
            (0, SidFidelity::AnchorOnly)
        } else {
            (source_delta as u8, requested_fidelity)
        };
        let mut bytes = [0u8; PACKED_SID_LEN];
        bytes[..3].copy_from_slice(sid.book.as_str().as_bytes());
        bytes[3..5].copy_from_slice(&sid.chapter.to_le_bytes());
        bytes[5..7].copy_from_slice(&sid.verse.to_le_bytes());
        bytes[7] = delta
            | if fidelity == SidFidelity::AnchorOnly {
                SID_FIDELITY_BIT
            } else {
                0
            };
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; PACKED_SID_LEN] {
        &self.0
    }

    pub(crate) fn decode(self) -> Result<(Sid, SidFidelity), DecodeError> {
        let book_text = std::str::from_utf8(&self.0[..3]).map_err(|_| DecodeError::InvalidUtf8)?;
        let book = BookId::from_str(book_text).ok_or(DecodeError::InvalidSection)?;
        let chapter = u16::from_le_bytes([self.0[3], self.0[4]]);
        let verse = u16::from_le_bytes([self.0[5], self.0[6]]);
        let delta = self.0[7] & SID_DELTA_MASK;
        let fidelity = if self.0[7] & SID_FIDELITY_BIT == 0 {
            SidFidelity::Exact
        } else {
            SidFidelity::AnchorOnly
        };
        let sid = if delta == 0 {
            Sid::new(book, chapter, verse)
        } else {
            Sid::with_range(book, chapter, verse, verse.saturating_add(u16::from(delta)))
        };
        Ok((sid, fidelity))
    }

    pub(crate) const fn from_bytes(bytes: [u8; PACKED_SID_LEN]) -> Self {
        Self(bytes)
    }
}

/// Validated packed-SID dictionary — token field 12.
///
/// Construction proves the whole column: its entry count fits what the `u16`
/// `sid_index` column can name, and every record decodes. Lookup afterwards is
/// infallible, so no caller has to re-handle a dictionary error per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SidDictionary<'wire> {
    records: &'wire [u8],
    count: u16,
}

impl<'wire> SidDictionary<'wire> {
    fn from_section(section: &Section<'wire>) -> Result<Self, DecodeError> {
        let field = section
            .field(token_field::PACKED_SID_DICTIONARY)
            .ok_or(DecodeError::InvalidSection)?;
        // The `0xffff` sentinel consumes one index value, so the highest
        // nameable record is 65,534 and the count ceiling is 65,535. Checked
        // before the records are walked, so an inflated count costs one compare.
        if field.count > MAX_DISTINCT_SIDS {
            return Err(DecodeError::TooManySids { found: field.count });
        }
        let count = field.count as u16;
        // The container already proved `count * 8 == byte_len` for this
        // fixed-width field; what remains is that each record's book code is a
        // legal one, which a decoded `Sid` cannot represent otherwise.
        for record in field.bytes.chunks_exact(PACKED_SID_LEN) {
            let raw: [u8; PACKED_SID_LEN] =
                record.try_into().map_err(|_| DecodeError::InvalidSection)?;
            PackedSid::from_bytes(raw).decode()?;
        }
        Ok(Self {
            records: field.bytes,
            count,
        })
    }

    pub(crate) const fn len(&self) -> u16 {
        self.count
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// `None` for the "no SID for this token" sentinel. Any other out-of-range
    /// index was rejected when the columns were built, so a present index always
    /// resolves.
    pub(crate) fn get(&self, index: u16) -> Option<(Sid, SidFidelity)> {
        if index == INDEX_NONE_U16 {
            return None;
        }
        let start = usize::from(index) * PACKED_SID_LEN;
        let raw: [u8; PACKED_SID_LEN] = self
            .records
            .get(start..start.checked_add(PACKED_SID_LEN)?)?
            .try_into()
            .ok()?;
        PackedSid::from_bytes(raw).decode().ok()
    }
}

/// Interns SIDs into a packed dictionary, returning the `sid_index` for each.
///
/// Ordinals are assigned in first-use order, which makes the output a function
/// of the token order alone. The `BTreeMap` is a lookup index only — it is never
/// iterated, so no map ordering reaches the bytes.
pub(crate) struct SidDictionaryBuilder {
    book: BookId,
    records: Vec<u8>,
    ordinals: BTreeMap<[u8; PACKED_SID_LEN], u16>,
}

impl SidDictionaryBuilder {
    pub(crate) fn new(book: BookId) -> Self {
        Self {
            book,
            records: Vec::new(),
            ordinals: BTreeMap::new(),
        }
    }

    /// Returns the index to store in the row's `sid_index`. Refuses rather than
    /// truncating: a book past the ceiling is structurally legal but not
    /// scriptural, and silently reusing the sentinel would corrupt every row.
    pub(crate) fn intern(&mut self, sid: Sid, fidelity: SidFidelity) -> Result<u16, EncodeError> {
        let packed = PackedSid::encode(sid, fidelity);
        if let Some(ordinal) = self.ordinals.get(packed.as_bytes()) {
            return Ok(*ordinal);
        }
        let next = self.ordinals.len();
        if next >= MAX_DISTINCT_SIDS as usize {
            return Err(EncodeError::TooManySids {
                book: self.book,
                found: MAX_DISTINCT_SIDS.saturating_add(1),
            });
        }
        let ordinal = next as u16;
        self.records.extend_from_slice(packed.as_bytes());
        self.ordinals.insert(*packed.as_bytes(), ordinal);
        Ok(ordinal)
    }

    /// Directory `count` for field 12.
    pub(crate) fn len(&self) -> u32 {
        self.ordinals.len() as u32
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.records
    }
}

/// Borrowed, token-specific view over per-row columns. Dictionary and sparse
/// sidecar validation belongs to their codecs; this view guarantees that each
/// fixed column has exactly one entry per token and every kind tag is known.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TokenColumns<'wire> {
    record_count: u32,
    kinds: &'wire [u8],
    span_starts: &'wire [u8],
    span_ends: &'wire [u8],
    token_id_indices: Option<&'wire [u8]>,
    token_ids: Option<StringDictionary<'wire>>,
    sid_indices: &'wire [u8],
    marker_descriptor_indices: &'wire [u8],
    sids: SidDictionary<'wire>,
}

impl<'wire> TokenColumns<'wire> {
    pub(crate) fn from_section(section: &Section<'wire>) -> Result<Self, DecodeError> {
        if section.header.kind != SectionKind::Token {
            return Err(DecodeError::InvalidSection);
        }
        let record_count = section.header.record_count;
        let kinds = per_row(section, token_field::KIND, record_count)?.bytes;
        if kinds
            .iter()
            .copied()
            .any(|value| TokenKindTag::from_u8(value).is_none())
        {
            return Err(DecodeError::InvalidDiscriminant);
        }
        let span_starts = per_row(section, token_field::SPAN_START, record_count)?.bytes;
        let span_ends = per_row(section, token_field::SPAN_END, record_count)?.bytes;
        let sid_indices = per_row(section, token_field::SID_INDEX, record_count)?.bytes;
        let sids = SidDictionary::from_section(section)?;
        // Establish the index invariant for the whole column once: a row's SID
        // index is either the sentinel or names a record that exists. Doing it
        // here is what lets the per-row accessor be infallible.
        for index in sid_indices.chunks_exact(2) {
            let index = u16::from_le_bytes([index[0], index[1]]);
            if index != INDEX_NONE_U16 && index >= sids.len() {
                return Err(DecodeError::InvalidSection);
            }
        }
        let marker_descriptor_indices =
            per_row(section, token_field::MARKER_DESCRIPTOR_INDEX, record_count)?.bytes;
        let (token_id_indices, token_ids) = match section.field(token_field::TOKEN_ID_INDEX) {
            Some(field) => {
                require_count(field, record_count)?;
                let dictionary = section
                    .field(token_field::TOKEN_ID_DICTIONARY)
                    .ok_or(DecodeError::InvalidSection)?;
                // The id dictionary carries opaque caller identity, so it earns the
                // same scrutiny as every other dictionary: UTF-8 per string,
                // ascending in-range offsets, and every index resolving. Accepting
                // it unvalidated would hand out a view over unchecked bytes.
                let dictionary = StringDictionary::from_field(dictionary)?;
                for index in field.bytes.chunks_exact(4) {
                    let index = u32::from_le_bytes([index[0], index[1], index[2], index[3]]);
                    // Non-empty because a stable token id is identity: an empty one
                    // cannot be told apart from a missing one, and core's
                    // `StableTokenId` refuses to hold it.
                    match dictionary.get(index) {
                        Some(id) if !id.is_empty() => {}
                        _ => return Err(DecodeError::InvalidSection),
                    }
                }
                (Some(field.bytes), Some(dictionary))
            }
            None if section.positional_ids() => (None, None),
            None => return Err(DecodeError::InvalidSection),
        };

        Ok(Self {
            record_count,
            kinds,
            span_starts,
            span_ends,
            token_id_indices,
            token_ids,
            sid_indices,
            marker_descriptor_indices,
            sids,
        })
    }

    pub(crate) const fn len(&self) -> u32 {
        self.record_count
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.record_count == 0
    }

    /// The raw kind column, for the sparse codecs that must check a record's
    /// target row against it.
    pub(crate) const fn kinds(&self) -> &'wire [u8] {
        self.kinds
    }

    pub(crate) fn kind(&self, row: u32) -> Option<TokenKindTag> {
        self.kinds
            .get(usize::try_from(row).ok()?)
            .copied()
            .and_then(TokenKindTag::from_u8)
    }

    pub(crate) fn span(&self, row: u32) -> Option<(u32, u32)> {
        Some((u32_at(self.span_starts, row)?, u32_at(self.span_ends, row)?))
    }

    pub(crate) fn token_id_index(&self, row: u32) -> Option<u32> {
        u32_at(self.token_id_indices?, row)
    }

    /// The row's explicit stable id, for a section that carries opaque ids.
    /// `None` for a positional-id section, where ids are synthesized instead.
    /// Validated at construction, so a present column always resolves.
    pub(crate) fn token_id(&self, row: u32) -> Option<&'wire str> {
        self.token_ids?.get(self.token_id_index(row)?)
    }

    /// True when this section carries opaque caller ids rather than positional
    /// ones.
    pub(crate) const fn has_explicit_ids(&self) -> bool {
        self.token_ids.is_some()
    }

    pub(crate) fn sid_index(&self, row: u32) -> Option<u16> {
        u16_at(self.sid_indices, row)
    }

    pub(crate) fn marker_descriptor_index(&self, row: u32) -> Option<u16> {
        u16_at(self.marker_descriptor_indices, row)
    }

    /// The row's anchor, or `None` when the row has no SID or is out of range.
    pub(crate) fn sid(&self, row: u32) -> Option<(Sid, SidFidelity)> {
        self.sids.get(self.sid_index(row)?)
    }

    pub(crate) const fn sids(&self) -> SidDictionary<'wire> {
        self.sids
    }
}

fn per_row<'section, 'wire>(
    section: &'section Section<'wire>,
    field_id: u16,
    record_count: u32,
) -> Result<&'section SectionField<'wire>, DecodeError> {
    let field = section.field(field_id).ok_or(DecodeError::InvalidSection)?;
    require_count(field, record_count)?;
    Ok(field)
}

fn require_count(field: &SectionField<'_>, record_count: u32) -> Result<(), DecodeError> {
    if field.count != record_count {
        return Err(DecodeError::InvalidSection);
    }
    Ok(())
}

fn u16_at(bytes: &[u8], row: u32) -> Option<u16> {
    let start = usize::try_from(row).ok()?.checked_mul(2)?;
    let raw: [u8; 2] = bytes.get(start..start.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(raw))
}

fn u32_at(bytes: &[u8], row: u32) -> Option<u32> {
    let start = usize::try_from(row).ok()?.checked_mul(4)?;
    let raw: [u8; 4] = bytes.get(start..start.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(code: &str) -> BookId {
        BookId::from_str(code).unwrap()
    }

    #[test]
    fn packed_sid_round_trips_exact_and_anchor_only_ranges() {
        let exact = PackedSid::encode(Sid::with_range(book("GEN"), 1, 1, 128), SidFidelity::Exact);
        assert_eq!(exact.as_bytes()[7], 127);
        let (sid, fidelity) = exact.decode().unwrap();
        assert_eq!(sid.verse_end(), 128);
        assert_eq!(fidelity, SidFidelity::Exact);

        let anchor_only = PackedSid::encode(
            Sid::with_range(book("GEN"), 1, 1, 2),
            SidFidelity::AnchorOnly,
        );
        assert_eq!(anchor_only.as_bytes()[7], 0x81);
        let (sid, fidelity) = anchor_only.decode().unwrap();
        assert_eq!(sid.verse_end(), 2);
        assert_eq!(fidelity, SidFidelity::AnchorOnly);
    }

    #[test]
    fn packed_sid_wider_than_127_becomes_anchor_only() {
        let packed = PackedSid::encode(Sid::with_range(book("GEN"), 1, 1, 129), SidFidelity::Exact);
        assert_eq!(packed.as_bytes()[7], 0x80);
        let (sid, fidelity) = packed.decode().unwrap();
        assert_eq!(sid.verse, 1);
        assert_eq!(sid.verse_end(), 1);
        assert_eq!(fidelity, SidFidelity::AnchorOnly);
    }

    #[test]
    fn builder_dedupes_and_assigns_ordinals_in_first_use_order() {
        let mut builder = SidDictionaryBuilder::new(book("GEN"));
        let first = Sid::new(book("GEN"), 1, 1);
        let second = Sid::new(book("GEN"), 1, 2);
        assert_eq!(builder.intern(second, SidFidelity::Exact), Ok(0));
        assert_eq!(builder.intern(first, SidFidelity::Exact), Ok(1));
        // Same anchor again is the same ordinal, and adds no record.
        assert_eq!(builder.intern(second, SidFidelity::Exact), Ok(0));
        // Same anchor, different fidelity, is a different dictionary entry: the
        // fidelity bit is part of the record, not metadata beside it.
        assert_eq!(builder.intern(second, SidFidelity::AnchorOnly), Ok(2));
        assert_eq!(builder.len(), 3);
        assert_eq!(builder.bytes().len(), 3 * PACKED_SID_LEN);
        assert_eq!(
            PackedSid::from_bytes(builder.bytes()[..PACKED_SID_LEN].try_into().unwrap())
                .decode()
                .unwrap(),
            (second, SidFidelity::Exact)
        );
    }

    #[test]
    fn builder_output_depends_only_on_intern_order() {
        let build = |verses: &[u16]| {
            let mut builder = SidDictionaryBuilder::new(book("GEN"));
            for verse in verses {
                builder
                    .intern(Sid::new(book("GEN"), 1, *verse), SidFidelity::Exact)
                    .unwrap();
            }
            builder.bytes().to_vec()
        };
        assert_eq!(build(&[3, 1, 2]), build(&[3, 1, 2, 3, 1]));
        assert_ne!(build(&[3, 1, 2]), build(&[1, 2, 3]));
    }

    #[test]
    fn builder_refuses_more_sids_than_the_index_column_can_name() {
        let mut builder = SidDictionaryBuilder::new(book("GEN"));
        // Distinct anchors are cheap to generate across chapters; the ceiling is
        // the sentinel-adjusted count, not a byte budget.
        let mut interned = 0u32;
        'fill: for chapter in 1..=u16::MAX {
            for verse in 1..=u16::MAX {
                match builder.intern(Sid::new(book("GEN"), chapter, verse), SidFidelity::Exact) {
                    Ok(_) => interned += 1,
                    Err(error) => {
                        assert_eq!(
                            error,
                            EncodeError::TooManySids {
                                book: book("GEN"),
                                found: MAX_DISTINCT_SIDS + 1,
                            }
                        );
                        break 'fill;
                    }
                }
            }
        }
        assert_eq!(interned, MAX_DISTINCT_SIDS);
        assert_eq!(builder.len(), MAX_DISTINCT_SIDS);
    }

    #[test]
    fn packed_sid_rejects_an_invalid_book() {
        let packed = PackedSid::from_bytes(*b"G-N\x01\x00\x01\x00\x00");
        assert_eq!(packed.decode(), Err(DecodeError::InvalidSection));
    }
}
