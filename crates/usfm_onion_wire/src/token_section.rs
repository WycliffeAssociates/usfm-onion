//! Checked views over the fixed token columns and the packed SID dictionary.
//!
//! This layer knows token semantics but does not yet decode dictionaries or
//! materialize core tokens. It is the narrow boundary between a structurally
//! valid generic section and token-specific readers.

use usfm_onion::token::{BookId, Sid};

use crate::container::{Section, SectionField};
use crate::error::DecodeError;
use crate::schema::{
    PACKED_SID_LEN, SID_DELTA_MASK, SID_FIDELITY_BIT, SectionKind, TokenKindTag, token_field,
};

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
    sid_indices: &'wire [u8],
    marker_descriptor_indices: &'wire [u8],
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
        let marker_descriptor_indices =
            per_row(section, token_field::MARKER_DESCRIPTOR_INDEX, record_count)?.bytes;
        let token_id_indices = match section.field(token_field::TOKEN_ID_INDEX) {
            Some(field) => {
                require_count(field, record_count)?;
                if section.field(token_field::TOKEN_ID_DICTIONARY).is_none() {
                    return Err(DecodeError::InvalidSection);
                }
                Some(field.bytes)
            }
            None if section.positional_ids() => None,
            None => return Err(DecodeError::InvalidSection),
        };

        Ok(Self {
            record_count,
            kinds,
            span_starts,
            span_ends,
            token_id_indices,
            sid_indices,
            marker_descriptor_indices,
        })
    }

    pub(crate) const fn len(&self) -> u32 {
        self.record_count
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.record_count == 0
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

    pub(crate) fn sid_index(&self, row: u32) -> Option<u16> {
        u16_at(self.sid_indices, row)
    }

    pub(crate) fn marker_descriptor_index(&self, row: u32) -> Option<u16> {
        u16_at(self.marker_descriptor_indices, row)
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
    fn packed_sid_rejects_an_invalid_book() {
        let packed = PackedSid::from_bytes(*b"G-N\x01\x00\x01\x00\x00");
        assert_eq!(packed.decode(), Err(DecodeError::InvalidSection));
    }
}
