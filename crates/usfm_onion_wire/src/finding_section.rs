//! Checked structural views and deterministic buffers for finding sections.
//!
//! This module deliberately stops before `LintIssue` materialization.  The
//! common rows and sidecars establish byte-level invariants here; the next
//! layer supplies the rule catalog, token views, and exact source needed to
//! turn those validated references into core values.

use std::collections::BTreeMap;

use usfm_onion::token::BookId;

use crate::container::{ElementWidth, FieldPayload, Section, SectionPayload, SectionVariant};
use crate::error::{DecodeError, EncodeError};
use crate::schema::{
    FINDING_SECTION_RULES_VERSION, LintCodeTag, SectionKind, finding_field, finding_flag,
    param_contract,
};
use crate::token_payload::{StringDictionary, StringDictionaryBuilder};

const ROW_LEN: usize = 16;
const RELATED_LEN: usize = 16;
const OVERFLOW_LEN: usize = 8;
const MARKER_REF_LEN: usize = 8;
const PAYLOAD_ROW_LEN: usize = 8;
const PAYLOAD_PAIR_LEN: usize = 8;
const TOKEN_NONE: u32 = u32::MAX;

/// The token-section fact required to validate finding token indices.  Source
/// bytes and catalog lookup intentionally remain outside this structural layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FindingDecodeInputs {
    pub token_count: u32,
}

/// A source-relative marker representation.  The catalog ordinal and source
/// span are retained as references; their semantic resolution is a later step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkerRef {
    AnchoredToken,
    CatalogOrdinal(u16),
    SourceSpan { offset: u32, len: u8 },
    Absent,
}

/// One checked common row with its record-aligned sidecar values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindingRow<'wire> {
    pub token_idx: Option<u32>,
    pub offset: u32,
    pub len: u32,
    pub chapter: Option<u16>,
    pub verse: Option<u16>,
    pub range_end: Option<u8>,
    pub code: LintCodeTag,
    pub anchor_only: bool,
    pub related: Option<(u32, u32, u32)>,
    pub marker: MarkerRef,
    pub params: Option<Vec<(&'wire str, &'wire str)>>,
}

/// Borrowed, fully checked finding-section columns.  This is intentionally a
/// narrow structural seam for a later semantic decoder, not a parallel lint
/// implementation.
#[derive(Debug, Clone)]
pub(crate) struct FindingColumns<'wire> {
    rows: Vec<FindingRow<'wire>>,
    pub source_len: u64,
    pub source_hash: u64,
    pub catalog_stamp: u64,
    pub rules_version: u16,
}

impl<'wire> FindingColumns<'wire> {
    pub(crate) fn from_section(
        section: &Section<'wire>,
        inputs: FindingDecodeInputs,
    ) -> Result<Self, DecodeError> {
        if section.header.kind != SectionKind::Finding {
            return Err(DecodeError::InvalidSection);
        }
        if section.header.rules_version != FINDING_SECTION_RULES_VERSION {
            return Err(DecodeError::UnsupportedVersion {
                found: section.header.rules_version,
            });
        }
        // Patch framing is deliberately deferred.  Rejecting both ids prevents
        // a newer producer from being mistaken for a v1 section with no fix.
        if section.field(finding_field::PATCH_ID).is_some()
            || section.field(finding_field::PATCH_TABLE).is_some()
        {
            return Err(DecodeError::InvalidSection);
        }

        let count = section.header.record_count;
        let common = per_row(section, finding_field::COMMON_ROW, count)?;
        let related = optional_per_row(section, finding_field::RELATED_TOKEN_IDX, count)?;
        let overflow = optional_per_row(section, finding_field::OVERFLOW_SPAN, count)?;
        let payload_indices = optional_per_row(section, finding_field::MESSAGE_PAYLOAD_IDX, count)?;
        let marker_refs = optional_per_row(section, finding_field::MARKER_REF, count)?;

        let payloads = match (
            payload_indices,
            section.field(finding_field::STRING_DICTIONARY),
            section.field(finding_field::MESSAGE_PAYLOAD_TABLE),
        ) {
            (None, None, None) => None,
            (Some(indices), Some(strings), Some(table)) => {
                Some(PayloadTable::from_fields(indices, *strings, *table)?)
            }
            _ => return Err(DecodeError::InvalidSection),
        };

        let mut any_related = false;
        let mut any_overflow = false;
        let mut any_payload = false;
        let mut any_marker_override = false;
        let mut rows =
            Vec::with_capacity(usize::try_from(count).map_err(|_| DecodeError::OffsetOverflow)?);

        for row in 0..count {
            let record = record_at(common, row, ROW_LEN)?;
            let token_raw = u32_at(record, 0)?;
            let short_offset = u16_at(record, 4)?;
            let short_len = u16_at(record, 6)?;
            let chapter = u16_at(record, 8)?;
            let verse = u16_at(record, 10)?;
            let range_end = record[12];
            let code = LintCodeTag::from_u8(record[13]).ok_or(DecodeError::InvalidDiscriminant)?;
            let flags = record[14];
            if flags & !finding_flag::KNOWN != 0 {
                return Err(DecodeError::UnsupportedFlags {
                    found: u32::from(flags),
                });
            }
            if record[15] != 0 || flags & finding_flag::FIX != 0 {
                return Err(DecodeError::InvalidSection);
            }

            let no_anchor = flags & finding_flag::NO_ANCHOR != 0;
            let anchor_only = flags & finding_flag::ANCHOR_ONLY != 0;
            let range = flags & finding_flag::RANGE != 0;
            if (range && range_end == 0) || (!range && range_end != 0) {
                return Err(DecodeError::InvalidSection);
            }
            if no_anchor && (chapter != 0 || verse != 0 || range_end != 0 || range) {
                return Err(DecodeError::InvalidSection);
            }
            // `no_anchor` means no SID exists at all; `anchor_only` describes
            // the fidelity of an *existing* SID. A row cannot claim both.
            if no_anchor && anchor_only {
                return Err(DecodeError::InvalidSection);
            }
            let token_idx = if token_raw == TOKEN_NONE {
                None
            } else {
                if token_raw >= inputs.token_count {
                    return Err(DecodeError::InvalidSection);
                }
                Some(token_raw)
            };

            let related_flag = flags & finding_flag::RELATED != 0;
            let related_value = match (related_flag, related) {
                (false, None) => None,
                (false, Some(bytes)) if record_at(bytes, row, RELATED_LEN)? == [0; RELATED_LEN] => {
                    None
                }
                (true, Some(bytes)) => {
                    any_related = true;
                    let related = record_at(bytes, row, RELATED_LEN)?;
                    let related_idx = u32_at(related, 0)?;
                    if related_idx >= inputs.token_count {
                        return Err(DecodeError::InvalidSection);
                    }
                    // `reserved` (offset 12) must be zero: a nonzero value would
                    // be a future producer's field this build cannot honour.
                    if u32_at(related, 12)? != 0 {
                        return Err(DecodeError::InvalidSection);
                    }
                    Some((related_idx, u32_at(related, 4)?, u32_at(related, 8)?))
                }
                _ => return Err(DecodeError::InvalidSection),
            };

            let overflow_flag = flags & finding_flag::OVERFLOW != 0;
            let (offset, len) = match (overflow_flag, overflow) {
                (false, None) => (u32::from(short_offset), u32::from(short_len)),
                (false, Some(bytes))
                    if record_at(bytes, row, OVERFLOW_LEN)? == [0; OVERFLOW_LEN] =>
                {
                    (u32::from(short_offset), u32::from(short_len))
                }
                (true, Some(bytes)) => {
                    any_overflow = true;
                    let span = record_at(bytes, row, OVERFLOW_LEN)?;
                    (u32_at(span, 0)?, u32_at(span, 4)?)
                }
                _ => return Err(DecodeError::InvalidSection),
            };

            let payload_flag = flags & finding_flag::PAYLOAD != 0;
            let params = match (payload_flag, payloads.as_ref()) {
                (false, None) => checked_params(code, None)?,
                (false, Some(payloads)) if payloads.is_zero_filler(row)? => {
                    checked_params(code, None)?
                }
                (true, Some(payloads)) => {
                    any_payload = true;
                    checked_params(code, Some(payloads.params(row)?))?
                }
                _ => return Err(DecodeError::InvalidSection),
            };

            let marker = match marker_refs {
                None => MarkerRef::AnchoredToken,
                Some(bytes) => {
                    let marker = decode_marker_ref(
                        record_at(bytes, row, MARKER_REF_LEN)?,
                        section.header.source_len,
                    )?;
                    any_marker_override |= marker != MarkerRef::AnchoredToken;
                    marker
                }
            };

            rows.push(FindingRow {
                token_idx,
                offset,
                len,
                chapter: (!no_anchor).then_some(chapter),
                verse: (!no_anchor).then_some(verse),
                range_end: range.then_some(range_end),
                code,
                anchor_only,
                related: related_value,
                marker,
                params,
            });
        }
        if related.is_some() != any_related
            || overflow.is_some() != any_overflow
            || payload_indices.is_some() != any_payload
            || marker_refs.is_some() != any_marker_override
        {
            return Err(DecodeError::InvalidSection);
        }
        Ok(Self {
            rows,
            source_len: section.header.source_len,
            source_hash: section.header.source_hash,
            catalog_stamp: section.header.catalog_stamp,
            rules_version: section.header.rules_version,
        })
    }

    pub(crate) fn rows(&self) -> &[FindingRow<'wire>] {
        &self.rows
    }
}

/// Builder input for one finding.  It intentionally contains only wire-owned
/// structure; semantic rule validation and `LintIssue` materialization follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindingRowInput {
    pub token_idx: Option<u32>,
    pub offset: u32,
    pub len: u32,
    pub chapter: Option<u16>,
    pub verse: Option<u16>,
    pub range_end: Option<u8>,
    pub code: LintCodeTag,
    pub anchor_only: bool,
    pub related: Option<(u32, u32, u32)>,
    pub marker: MarkerRef,
    pub params: Option<BTreeMap<String, String>>,
}

/// Owned finding columns that can lend a `SectionPayload` to the generic
/// container writer.  Input order is retained: callers own canonical finding
/// ordering, while all dictionary and field ordering is deterministic here.
#[derive(Debug)]
pub(crate) struct FindingSectionBuffers {
    book: BookId,
    source_hash: u64,
    source_len: u64,
    catalog_stamp: u64,
    common: Vec<u8>,
    related: Option<Vec<u8>>,
    overflow: Option<Vec<u8>>,
    payload_indices: Option<Vec<u8>>,
    marker_refs: Option<Vec<u8>>,
    strings: Option<Vec<u8>>,
    string_count: u32,
    payload_table: Option<Vec<u8>>,
    payload_count: u32,
    record_count: u32,
}

impl FindingSectionBuffers {
    pub(crate) fn new(
        book: BookId,
        source_hash: u64,
        source_len: u64,
        catalog_stamp: u64,
        rows: &[FindingRowInput],
    ) -> Result<Self, EncodeError> {
        let record_count =
            u32::try_from(rows.len()).map_err(|_| EncodeError::UnrepresentablePayload {
                book,
                code: u8::MAX,
            })?;
        let mut common = Vec::with_capacity(rows.len().saturating_mul(ROW_LEN));
        let mut related = vec![0; rows.len().saturating_mul(RELATED_LEN)];
        let mut overflow = vec![0; rows.len().saturating_mul(OVERFLOW_LEN)];
        let mut payload_indices = vec![0; rows.len().saturating_mul(4)];
        let mut marker_refs = vec![0; rows.len().saturating_mul(MARKER_REF_LEN)];
        let mut has_related = false;
        let mut has_overflow = false;
        let mut has_payload = false;
        let mut has_marker_override = false;
        let mut strings = StringDictionaryBuilder::default();
        let mut payload_rows = Vec::new();
        let mut payload_pairs = Vec::new();

        for (row_index, row) in rows.iter().enumerate() {
            let mut flags = 0u8;
            let (chapter, verse) = match (row.chapter, row.verse) {
                (Some(chapter), Some(verse)) => (chapter, verse),
                (None, None) => {
                    flags |= finding_flag::NO_ANCHOR;
                    (0, 0)
                }
                _ => return Err(unrepresentable(book, row.code)),
            };
            if row.anchor_only {
                // `anchor_only` describes the fidelity of an *existing* SID;
                // a row with no SID at all (`chapter: None`) cannot also
                // claim one.
                if row.chapter.is_none() {
                    return Err(unrepresentable(book, row.code));
                }
                flags |= finding_flag::ANCHOR_ONLY;
            }
            let range_end = match row.range_end {
                Some(value) if value != 0 && row.chapter.is_some() => {
                    flags |= finding_flag::RANGE;
                    value
                }
                None => 0,
                _ => return Err(unrepresentable(book, row.code)),
            };
            if row.related.is_some() {
                flags |= finding_flag::RELATED;
                has_related = true;
            }
            if row.params.is_some() {
                if !params_are_representable(row.code, row.params.as_ref()) {
                    return Err(unrepresentable(book, row.code));
                }
                flags |= finding_flag::PAYLOAD;
                has_payload = true;
            } else if param_contract(row.code).is_some() {
                return Err(unrepresentable(book, row.code));
            }
            if row.offset > u32::from(u16::MAX) || row.len > u32::from(u16::MAX) {
                flags |= finding_flag::OVERFLOW;
                has_overflow = true;
            }

            common.extend_from_slice(&row.token_idx.unwrap_or(TOKEN_NONE).to_le_bytes());
            common.extend_from_slice(&(row.offset.min(u32::from(u16::MAX)) as u16).to_le_bytes());
            common.extend_from_slice(&(row.len.min(u32::from(u16::MAX)) as u16).to_le_bytes());
            common.extend_from_slice(&chapter.to_le_bytes());
            common.extend_from_slice(&verse.to_le_bytes());
            common.push(range_end);
            common.push(row.code as u8);
            common.push(flags);
            common.push(0);

            if let Some((token_idx, offset, len)) = row.related {
                related[row_index * RELATED_LEN..row_index * RELATED_LEN + 4]
                    .copy_from_slice(&token_idx.to_le_bytes());
                related[row_index * RELATED_LEN + 4..row_index * RELATED_LEN + 8]
                    .copy_from_slice(&offset.to_le_bytes());
                related[row_index * RELATED_LEN + 8..row_index * RELATED_LEN + 12]
                    .copy_from_slice(&len.to_le_bytes());
                // Bytes [12..16] stay zero: `reserved`.
            }
            if flags & finding_flag::OVERFLOW != 0 {
                overflow[row_index * OVERFLOW_LEN..row_index * OVERFLOW_LEN + 4]
                    .copy_from_slice(&row.offset.to_le_bytes());
                overflow[row_index * OVERFLOW_LEN + 4..row_index * OVERFLOW_LEN + 8]
                    .copy_from_slice(&row.len.to_le_bytes());
            }
            if let Some(params) = &row.params {
                let payload_index = u32::try_from(payload_rows.len() / PAYLOAD_ROW_LEN)
                    .map_err(|_| unrepresentable(book, row.code))?;
                payload_indices[row_index * 4..row_index * 4 + 4]
                    .copy_from_slice(&payload_index.to_le_bytes());
                let first_pair = u32::try_from(payload_pairs.len() / PAYLOAD_PAIR_LEN)
                    .map_err(|_| unrepresentable(book, row.code))?;
                let pair_count =
                    u32::try_from(params.len()).map_err(|_| unrepresentable(book, row.code))?;
                payload_rows.extend_from_slice(&first_pair.to_le_bytes());
                payload_rows.extend_from_slice(&pair_count.to_le_bytes());
                // BTreeMap iteration is the documented canonical key order.
                for (key, value) in params {
                    let key_index = strings
                        .intern(key)
                        .map_err(|_| unrepresentable(book, row.code))?;
                    let value_index = strings
                        .intern(value)
                        .map_err(|_| unrepresentable(book, row.code))?;
                    payload_pairs.extend_from_slice(&key_index.to_le_bytes());
                    payload_pairs.extend_from_slice(&value_index.to_le_bytes());
                }
            }
            let encoded_marker = encode_marker_ref(row.marker, source_len, book, row.code)?;
            if encoded_marker != [0; MARKER_REF_LEN] {
                has_marker_override = true;
            }
            marker_refs[row_index * MARKER_REF_LEN..(row_index + 1) * MARKER_REF_LEN]
                .copy_from_slice(&encoded_marker);
        }

        let payload_count = u32::try_from(payload_rows.len() / PAYLOAD_ROW_LEN).map_err(|_| {
            EncodeError::UnrepresentablePayload {
                book,
                code: u8::MAX,
            }
        })?;
        let payload_table = has_payload.then(|| {
            payload_rows.extend_from_slice(&payload_pairs);
            payload_rows
        });
        let string_count = if has_payload { strings.len() } else { 0 };
        let string_bytes = has_payload.then(|| strings.bytes());
        Ok(Self {
            book,
            source_hash,
            source_len,
            catalog_stamp,
            common,
            related: has_related.then_some(related),
            overflow: has_overflow.then_some(overflow),
            payload_indices: has_payload.then_some(payload_indices),
            marker_refs: has_marker_override.then_some(marker_refs),
            strings: string_bytes,
            string_count,
            payload_table,
            payload_count,
            record_count,
        })
    }

    pub(crate) fn payload(&self) -> SectionPayload<'_> {
        let mut fields = vec![FieldPayload {
            id: finding_field::COMMON_ROW,
            width: ElementWidth::Sixteen,
            count: self.record_count,
            bytes: &self.common,
        }];
        if let Some(bytes) = &self.related {
            fields.push(FieldPayload {
                id: finding_field::RELATED_TOKEN_IDX,
                width: ElementWidth::Sixteen,
                count: self.record_count,
                bytes,
            });
        }
        if let Some(bytes) = &self.overflow {
            fields.push(FieldPayload {
                id: finding_field::OVERFLOW_SPAN,
                width: ElementWidth::Eight,
                count: self.record_count,
                bytes,
            });
        }
        if let Some(bytes) = &self.payload_indices {
            fields.push(FieldPayload {
                id: finding_field::MESSAGE_PAYLOAD_IDX,
                width: ElementWidth::Four,
                count: self.record_count,
                bytes,
            });
        }
        if let Some(bytes) = &self.marker_refs {
            fields.push(FieldPayload {
                id: finding_field::MARKER_REF,
                width: ElementWidth::Eight,
                count: self.record_count,
                bytes,
            });
        }
        if let Some(bytes) = &self.strings {
            fields.push(FieldPayload {
                id: finding_field::STRING_DICTIONARY,
                width: ElementWidth::Variable,
                count: self.string_count,
                bytes,
            });
        }
        if let Some(bytes) = &self.payload_table {
            fields.push(FieldPayload {
                id: finding_field::MESSAGE_PAYLOAD_TABLE,
                width: ElementWidth::Variable,
                count: self.payload_count,
                bytes,
            });
        }
        SectionPayload {
            variant: SectionVariant::Finding {
                rules_version: FINDING_SECTION_RULES_VERSION,
            },
            book: self.book,
            source_hash: self.source_hash,
            source_len: self.source_len,
            catalog_stamp: self.catalog_stamp,
            record_count: self.record_count,
            fields,
        }
    }
}

fn unrepresentable(book: BookId, code: LintCodeTag) -> EncodeError {
    EncodeError::UnrepresentablePayload {
        book,
        code: code as u8,
    }
}

fn per_row<'wire>(
    section: &Section<'wire>,
    id: u16,
    count: u32,
) -> Result<&'wire [u8], DecodeError> {
    let field = section.field(id).ok_or(DecodeError::InvalidSection)?;
    if field.count != count {
        return Err(DecodeError::InvalidSection);
    }
    Ok(field.bytes)
}

fn optional_per_row<'section, 'wire>(
    section: &'section Section<'wire>,
    id: u16,
    count: u32,
) -> Result<Option<&'wire [u8]>, DecodeError> {
    match section.field(id) {
        Some(field) if field.count == count => Ok(Some(field.bytes)),
        Some(_) => Err(DecodeError::InvalidSection),
        None => Ok(None),
    }
}

fn record_at(bytes: &[u8], index: u32, len: usize) -> Result<&[u8], DecodeError> {
    let start = usize::try_from(index)
        .map_err(|_| DecodeError::OffsetOverflow)?
        .checked_mul(len)
        .ok_or(DecodeError::OffsetOverflow)?;
    bytes
        .get(start..start.checked_add(len).ok_or(DecodeError::OffsetOverflow)?)
        .ok_or(DecodeError::Truncated)
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, DecodeError> {
    let raw: [u8; 2] = bytes
        .get(offset..offset.checked_add(2).ok_or(DecodeError::OffsetOverflow)?)
        .ok_or(DecodeError::Truncated)?
        .try_into()
        .map_err(|_| DecodeError::Truncated)?;
    Ok(u16::from_le_bytes(raw))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, DecodeError> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.checked_add(4).ok_or(DecodeError::OffsetOverflow)?)
        .ok_or(DecodeError::Truncated)?
        .try_into()
        .map_err(|_| DecodeError::Truncated)?;
    Ok(u32::from_le_bytes(raw))
}

fn decode_marker_ref(bytes: &[u8], source_len: u64) -> Result<MarkerRef, DecodeError> {
    let tag = bytes[0];
    let len = bytes[1];
    let ordinal = u16_at(bytes, 2)?;
    let offset = u32_at(bytes, 4)?;
    match tag {
        0 if len == 0 && ordinal == 0 && offset == 0 => Ok(MarkerRef::AnchoredToken),
        1 if len == 0 && offset == 0 => Ok(MarkerRef::CatalogOrdinal(ordinal)),
        2 if ordinal == 0
            && len != 0
            && u64::from(offset)
                .checked_add(u64::from(len))
                .is_some_and(|end| end <= source_len) =>
        {
            Ok(MarkerRef::SourceSpan { offset, len })
        }
        3 if len == 0 && ordinal == 0 && offset == 0 => Ok(MarkerRef::Absent),
        0..=3 => Err(DecodeError::InvalidSection),
        _ => Err(DecodeError::InvalidDiscriminant),
    }
}

fn encode_marker_ref(
    marker: MarkerRef,
    source_len: u64,
    book: BookId,
    code: LintCodeTag,
) -> Result<[u8; MARKER_REF_LEN], EncodeError> {
    let mut bytes = [0; MARKER_REF_LEN];
    match marker {
        MarkerRef::AnchoredToken => {}
        MarkerRef::CatalogOrdinal(ordinal) => {
            bytes[0] = 1;
            bytes[2..4].copy_from_slice(&ordinal.to_le_bytes());
        }
        MarkerRef::SourceSpan { offset, len }
            if len != 0
                && u64::from(offset)
                    .checked_add(u64::from(len))
                    .is_some_and(|end| end <= source_len) =>
        {
            bytes[0] = 2;
            bytes[1] = len;
            bytes[4..8].copy_from_slice(&offset.to_le_bytes());
        }
        MarkerRef::Absent => bytes[0] = 3,
        MarkerRef::SourceSpan { .. } => return Err(unrepresentable(book, code)),
    }
    Ok(bytes)
}

/// The generic payload table is shared by all rules, but each code owns one
/// exact argument contract. Keeping that validation beside the structural
/// reader prevents a later semantic layer from accepting bytes this codec did
/// not promise to preserve.
fn checked_params<'wire>(
    code: LintCodeTag,
    params: Option<Vec<(&'wire str, &'wire str)>>,
) -> Result<Option<Vec<(&'wire str, &'wire str)>>, DecodeError> {
    match (param_contract(code), params) {
        (None, None) => Ok(None),
        (Some(contract), Some(params)) if decoded_params_match_contract(contract, &params) => {
            Ok(Some(params))
        }
        _ => Err(DecodeError::InvalidSection),
    }
}

fn params_are_representable(code: LintCodeTag, params: Option<&BTreeMap<String, String>>) -> bool {
    match (param_contract(code), params) {
        (None, None) => true,
        (Some(contract), Some(params)) => contract.accepts(params),
        _ => false,
    }
}

fn decoded_params_match_contract(
    contract: &crate::schema::ParamContract,
    params: &[(&str, &str)],
) -> bool {
    contract.variants.iter().any(|variant| {
        params.len() == variant.params.len()
            && variant.params.iter().all(|spec| {
                params
                    .iter()
                    .find(|(key, _)| *key == spec.key)
                    .is_some_and(|(_, value)| {
                        spec.allowed_values.is_empty() || spec.allowed_values.contains(value)
                    })
            })
    })
}

struct PayloadTable<'wire> {
    indices: &'wire [u8],
    strings: StringDictionary<'wire>,
    rows: &'wire [u8],
    pairs: &'wire [u8],
    count: u32,
}

impl<'wire> PayloadTable<'wire> {
    fn from_fields(
        indices: &'wire [u8],
        strings: crate::container::SectionField<'wire>,
        table: crate::container::SectionField<'wire>,
    ) -> Result<Self, DecodeError> {
        let strings = StringDictionary::from_field(&strings)?;
        let rows_len = usize::try_from(u64::from(table.count) * PAYLOAD_ROW_LEN as u64)
            .map_err(|_| DecodeError::OffsetOverflow)?;
        let (rows, pairs) = table
            .bytes
            .split_at_checked(rows_len)
            .ok_or(DecodeError::Truncated)?;
        if pairs.len() % PAYLOAD_PAIR_LEN != 0 {
            return Err(DecodeError::InvalidSection);
        }
        let pair_count = u32::try_from(pairs.len() / PAYLOAD_PAIR_LEN)
            .map_err(|_| DecodeError::OffsetOverflow)?;
        let mut expected_first = 0u32;
        for row in 0..table.count {
            let record = record_at(rows, row, PAYLOAD_ROW_LEN)?;
            let first = u32_at(record, 0)?;
            let count = u32_at(record, 4)?;
            if first != expected_first {
                return Err(DecodeError::InvalidSection);
            }
            expected_first = expected_first
                .checked_add(count)
                .ok_or(DecodeError::OffsetOverflow)?;
            if expected_first > pair_count {
                return Err(DecodeError::InvalidSection);
            }
            let mut previous_key: Option<&str> = None;
            for pair in first..expected_first {
                let pair = record_at(pairs, pair, PAYLOAD_PAIR_LEN)?;
                let key = strings
                    .get(u32_at(pair, 0)?)
                    .ok_or(DecodeError::InvalidSection)?;
                strings
                    .get(u32_at(pair, 4)?)
                    .ok_or(DecodeError::InvalidSection)?;
                if previous_key.is_some_and(|previous| previous >= key) {
                    return Err(DecodeError::InvalidSection);
                }
                previous_key = Some(key);
            }
        }
        if expected_first != pair_count {
            return Err(DecodeError::InvalidSection);
        }
        for index in indices.chunks_exact(4) {
            if u32::from_le_bytes(index.try_into().map_err(|_| DecodeError::Truncated)?)
                >= table.count
            {
                return Err(DecodeError::InvalidSection);
            }
        }
        Ok(Self {
            indices,
            strings,
            rows,
            pairs,
            count: table.count,
        })
    }

    fn params(&self, row: u32) -> Result<Vec<(&'wire str, &'wire str)>, DecodeError> {
        let payload_index = u32_at(
            self.indices,
            usize::try_from(row)
                .map_err(|_| DecodeError::OffsetOverflow)?
                .checked_mul(4)
                .ok_or(DecodeError::OffsetOverflow)?,
        )?;
        if payload_index >= self.count {
            return Err(DecodeError::InvalidSection);
        }
        let payload = record_at(self.rows, payload_index, PAYLOAD_ROW_LEN)?;
        let first = u32_at(payload, 0)?;
        let count = u32_at(payload, 4)?;
        let mut out =
            Vec::with_capacity(usize::try_from(count).map_err(|_| DecodeError::OffsetOverflow)?);
        for pair in first
            ..first
                .checked_add(count)
                .ok_or(DecodeError::OffsetOverflow)?
        {
            let pair = record_at(self.pairs, pair, PAYLOAD_PAIR_LEN)?;
            out.push((
                self.strings
                    .get(u32_at(pair, 0)?)
                    .ok_or(DecodeError::InvalidSection)?,
                self.strings
                    .get(u32_at(pair, 4)?)
                    .ok_or(DecodeError::InvalidSection)?,
            ));
        }
        Ok(out)
    }

    fn is_zero_filler(&self, row: u32) -> Result<bool, DecodeError> {
        let offset = usize::try_from(row)
            .map_err(|_| DecodeError::OffsetOverflow)?
            .checked_mul(4)
            .ok_or(DecodeError::OffsetOverflow)?;
        Ok(u32_at(self.indices, offset)? == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::{read_container, read_container_unchecked, write_container};
    use crate::schema::{
        CONTAINER_CHECKSUM_OFFSET, LINT_CODE_TABLE, PARAM_CONTRACTS, SECTION_CHECKSUM_OFFSET,
        SECTION_MAGIC,
    };

    fn book() -> BookId {
        BookId::from_str("GEN").unwrap()
    }
    fn input() -> FindingRowInput {
        FindingRowInput {
            token_idx: Some(0),
            offset: 0,
            len: 0,
            chapter: Some(1),
            verse: Some(1),
            range_end: None,
            code: LintCodeTag::MissingIdMarker,
            anchor_only: false,
            related: None,
            marker: MarkerRef::AnchoredToken,
            params: None,
        }
    }
    fn section(rows: &[FindingRowInput]) -> FindingSectionBuffers {
        FindingSectionBuffers::new(book(), 7, 99, 11, rows).unwrap()
    }
    fn encoded(rows: &[FindingRowInput]) -> Vec<u8> {
        // Pair with a minimal token section: the container's TOC policy requires it.
        let finding = section(rows);
        let token = SectionPayload {
            variant: SectionVariant::Token {
                positional_ids: true,
            },
            book: book(),
            source_hash: 7,
            source_len: 99,
            catalog_stamp: 11,
            record_count: 1,
            fields: vec![
                FieldPayload {
                    id: crate::schema::token_field::KIND,
                    width: ElementWidth::One,
                    count: 1,
                    bytes: &[0],
                },
                FieldPayload {
                    id: crate::schema::token_field::SPAN_START,
                    width: ElementWidth::Four,
                    count: 1,
                    bytes: &[0; 4],
                },
                FieldPayload {
                    id: crate::schema::token_field::SPAN_END,
                    width: ElementWidth::Four,
                    count: 1,
                    bytes: &[0; 4],
                },
                FieldPayload {
                    id: crate::schema::token_field::SID_INDEX,
                    width: ElementWidth::Two,
                    count: 1,
                    bytes: &[0xff, 0xff],
                },
                FieldPayload {
                    id: crate::schema::token_field::MARKER_DESCRIPTOR_INDEX,
                    width: ElementWidth::Two,
                    count: 1,
                    bytes: &[0xff, 0xff],
                },
                FieldPayload {
                    id: crate::schema::token_field::STRING_DICTIONARY,
                    width: ElementWidth::Variable,
                    count: 0,
                    bytes: &[],
                },
                FieldPayload {
                    id: crate::schema::token_field::MARKER_DESCRIPTOR_DICTIONARY,
                    width: ElementWidth::Eight,
                    count: 0,
                    bytes: &[],
                },
                FieldPayload {
                    id: crate::schema::token_field::PACKED_SID_DICTIONARY,
                    width: ElementWidth::Eight,
                    count: 0,
                    bytes: &[],
                },
            ],
        };
        write_container(1, &[token, finding.payload()]).unwrap()
    }
    fn decoded(bytes: &[u8]) -> Result<FindingColumns<'_>, DecodeError> {
        let container = read_container(bytes)?;
        let section = container.section(1).unwrap()?;
        FindingColumns::from_section(&section, FindingDecodeInputs { token_count: 1 })
    }

    fn decoded_unchecked(bytes: &[u8]) -> Result<FindingColumns<'_>, DecodeError> {
        let container = read_container_unchecked(bytes)?;
        let section = container.section(1).unwrap()?;
        FindingColumns::from_section(&section, FindingDecodeInputs { token_count: 1 })
    }

    fn omit_checksums(bytes: &mut [u8]) {
        bytes[CONTAINER_CHECKSUM_OFFSET..CONTAINER_CHECKSUM_OFFSET + 8].fill(0);
        let section_starts: Vec<_> = bytes
            .windows(SECTION_MAGIC.len())
            .enumerate()
            .filter_map(|(start, window)| (window == SECTION_MAGIC).then_some(start))
            .collect();
        for start in section_starts {
            bytes[start + SECTION_CHECKSUM_OFFSET..start + SECTION_CHECKSUM_OFFSET + 8].fill(0);
        }
    }

    fn corrupt_first_filler(bytes: &mut [u8], field_id: u16) {
        let filler_offset = {
            let container = read_container(bytes).unwrap();
            let section = container.section(1).unwrap().unwrap();
            section.field(field_id).unwrap().bytes.as_ptr() as usize - bytes.as_ptr() as usize
        };
        bytes[filler_offset] = 1;
        omit_checksums(bytes);
    }

    fn replace_first_code(bytes: &mut [u8], code: LintCodeTag) {
        let row_offset = {
            let container = read_container(bytes).unwrap();
            let section = container.section(1).unwrap().unwrap();
            section
                .field(finding_field::COMMON_ROW)
                .unwrap()
                .bytes
                .as_ptr() as usize
                - bytes.as_ptr() as usize
        };
        bytes[row_offset + 13] = code as u8;
        omit_checksums(bytes);
    }

    fn params_for(variant: &crate::schema::ParamVariant) -> BTreeMap<String, String> {
        variant
            .params
            .iter()
            .map(|spec| {
                (
                    spec.key.into(),
                    spec.allowed_values
                        .first()
                        .copied()
                        .unwrap_or("value")
                        .into(),
                )
            })
            .collect()
    }

    fn decoded_pairs(params: &BTreeMap<String, String>) -> Vec<(&str, &str)> {
        params
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect()
    }

    #[test]
    fn zero_rows_and_deterministic_bytes_decode() {
        let first = encoded(&[]);
        assert_eq!(first, encoded(&[]));
        assert!(decoded(&first).unwrap().rows().is_empty());
    }

    #[test]
    fn every_sidecar_and_unicode_payload_round_trip() {
        let mut row = input();
        row.offset = 70_000;
        row.len = 80_000;
        // Exercises the widened related record: offset/len above `u16::MAX`,
        // which the old 8-byte shape could not have stored at all.
        row.related = Some((0, 70_002, 80_003));
        row.marker = MarkerRef::CatalogOrdinal(4);
        row.code = LintCodeTag::UnknownToken;
        row.params = Some(BTreeMap::from([("text".into(), "日本語".into())]));
        let bytes = encoded(&[row]);
        let decoded = decoded(&bytes).unwrap();
        assert_eq!(decoded.rows()[0].offset, 70_000);
        assert_eq!(decoded.rows()[0].related, Some((0, 70_002, 80_003)));
        assert_eq!(decoded.rows()[0].marker, MarkerRef::CatalogOrdinal(4));
        assert_eq!(
            decoded.rows()[0].params.as_ref().unwrap()[0],
            ("text", "日本語")
        );
    }

    #[test]
    fn section_wide_sidecars_zero_fill_clear_rows() {
        let base = input();

        let mut related = input();
        related.related = Some((0, 2, 3));
        let bytes = encoded(&[base.clone(), related]);
        let views = decoded(&bytes).unwrap();
        assert_eq!(views.rows()[0].related, None);
        assert_eq!(views.rows()[1].related, Some((0, 2, 3)));

        let mut overflow = input();
        overflow.offset = 70_000;
        let bytes = encoded(&[base.clone(), overflow]);
        let views = decoded(&bytes).unwrap();
        assert_eq!(views.rows()[0].offset, 0);
        assert_eq!(views.rows()[1].offset, 70_000);

        let mut payload = input();
        payload.code = LintCodeTag::EmptyParagraph;
        payload.params = Some(BTreeMap::from([("marker".into(), "p".into())]));
        let bytes = encoded(&[base, payload]);
        let views = decoded(&bytes).unwrap();
        assert_eq!(views.rows()[0].params, None);
        assert_eq!(views.rows()[1].params.as_ref().unwrap(), &[("marker", "p")]);
    }

    #[test]
    fn decoder_rejects_nonzero_sidecar_fillers() {
        let base = input();

        let mut related = input();
        related.related = Some((0, 2, 3));
        let mut bytes = encoded(&[base.clone(), related]);
        corrupt_first_filler(&mut bytes, finding_field::RELATED_TOKEN_IDX);
        assert!(matches!(
            decoded_unchecked(&bytes),
            Err(DecodeError::InvalidSection)
        ));

        let mut overflow = input();
        overflow.offset = 70_000;
        let mut bytes = encoded(&[base.clone(), overflow]);
        corrupt_first_filler(&mut bytes, finding_field::OVERFLOW_SPAN);
        assert!(matches!(
            decoded_unchecked(&bytes),
            Err(DecodeError::InvalidSection)
        ));

        let mut payload = input();
        payload.code = LintCodeTag::EmptyParagraph;
        payload.params = Some(BTreeMap::from([("marker".into(), "p".into())]));
        let mut bytes = encoded(&[base, payload]);
        corrupt_first_filler(&mut bytes, finding_field::MESSAGE_PAYLOAD_IDX);
        assert!(matches!(
            decoded_unchecked(&bytes),
            Err(DecodeError::InvalidSection)
        ));
    }

    #[test]
    fn decoder_rejects_nonzero_related_reserved_word() {
        // The related record's trailing `reserved:u32` must be zero on encode
        // and rejected non-zero on decode, independent of every other
        // sidecar-filler check above (which only cover the *unused* rows).
        let mut related = input();
        related.related = Some((0, 2, 3));
        let mut bytes = encoded(&[related]);
        let reserved_offset = {
            let container = read_container(&bytes).unwrap();
            let section = container.section(1).unwrap().unwrap();
            section
                .field(finding_field::RELATED_TOKEN_IDX)
                .unwrap()
                .bytes
                .as_ptr() as usize
                - bytes.as_ptr() as usize
                + 12
        };
        bytes[reserved_offset] = 1;
        omit_checksums(&mut bytes);
        assert!(matches!(
            decoded_unchecked(&bytes),
            Err(DecodeError::InvalidSection)
        ));
    }

    #[test]
    fn generated_param_contracts_gate_encode_and_decode() {
        for contract in PARAM_CONTRACTS {
            for variant in contract.variants {
                let params = params_for(variant);
                let mut row = input();
                row.code = contract.code;
                row.params = Some(params.clone());
                let bytes = encoded(&[row]);
                let views = decoded(&bytes).unwrap();
                assert_eq!(
                    views.rows()[0].params.as_ref().unwrap(),
                    &decoded_pairs(&params)
                );
                assert!(checked_params(contract.code, Some(decoded_pairs(&params))).is_ok());
            }
        }
    }

    #[test]
    fn generated_param_contracts_refuse_wrong_missing_extra_and_closed_values() {
        for contract in PARAM_CONTRACTS {
            let variant = &contract.variants[0];
            let valid = params_for(variant);
            let first_key = variant.params[0].key;

            let mut missing = valid.clone();
            missing.remove(first_key);
            assert!(!params_are_representable(contract.code, Some(&missing)));
            assert!(checked_params(contract.code, Some(decoded_pairs(&missing))).is_err());

            let mut extra = valid.clone();
            extra.insert("unexpected".into(), "value".into());
            assert!(!params_are_representable(contract.code, Some(&extra)));
            assert!(checked_params(contract.code, Some(decoded_pairs(&extra))).is_err());

            let mut wrong = valid.clone();
            let value = wrong.remove(first_key).unwrap();
            wrong.insert("wrong".into(), value);
            assert!(!params_are_representable(contract.code, Some(&wrong)));
            assert!(checked_params(contract.code, Some(decoded_pairs(&wrong))).is_err());

            for variant in contract.variants {
                for spec in variant
                    .params
                    .iter()
                    .filter(|spec| !spec.allowed_values.is_empty())
                {
                    let mut closed = params_for(variant);
                    closed.insert(spec.key.into(), "not-a-contract-value".into());
                    assert!(!params_are_representable(contract.code, Some(&closed)));
                    assert!(checked_params(contract.code, Some(decoded_pairs(&closed))).is_err());
                }
            }
        }
    }

    #[test]
    fn zero_param_codes_refuse_payloads() {
        for code in LINT_CODE_TABLE {
            if param_contract(code).is_some() {
                continue;
            }
            let mut row = input();
            row.code = code;
            row.params = Some(BTreeMap::new());
            assert!(matches!(
                FindingSectionBuffers::new(book(), 7, 99, 11, &[row]),
                Err(EncodeError::UnrepresentablePayload { .. })
            ));
            assert!(checked_params(code, Some(Vec::new())).is_err());
        }
    }

    #[test]
    fn decoder_rechecks_payload_contract_after_structural_decode() {
        let mut row = input();
        row.code = LintCodeTag::UnknownToken;
        row.params = Some(BTreeMap::from([("text".into(), "x".into())]));

        let mut wrong_key = encoded(&[row.clone()]);
        replace_first_code(&mut wrong_key, LintCodeTag::UnknownMarker);
        assert!(matches!(
            decoded_unchecked(&wrong_key),
            Err(DecodeError::InvalidSection)
        ));

        let mut zero_param = encoded(&[row]);
        replace_first_code(&mut zero_param, LintCodeTag::MissingIdMarker);
        assert!(matches!(
            decoded_unchecked(&zero_param),
            Err(DecodeError::InvalidSection)
        ));
    }

    #[test]
    fn source_span_and_absent_marker_refs_round_trip() {
        let mut span = input();
        span.marker = MarkerRef::SourceSpan { offset: 2, len: 4 };
        let mut absent = input();
        absent.marker = MarkerRef::Absent;
        let bytes = encoded(&[span, absent]);
        let views = decoded(&bytes).unwrap();
        assert_eq!(
            views.rows()[0].marker,
            MarkerRef::SourceSpan { offset: 2, len: 4 }
        );
        assert_eq!(views.rows()[1].marker, MarkerRef::Absent);
    }

    #[test]
    fn builder_refuses_unrepresentable_marker_and_anchor_shapes() {
        let mut row = input();
        row.marker = MarkerRef::SourceSpan { offset: 99, len: 1 };
        assert!(FindingSectionBuffers::new(book(), 1, 99, 1, &[row]).is_err());
        let mut bad_anchor = input();
        bad_anchor.chapter = None;
        assert!(FindingSectionBuffers::new(book(), 1, 99, 1, &[bad_anchor]).is_err());
    }

    #[test]
    fn all_truncations_and_single_byte_corruption_are_survivable() {
        let mut row = input();
        row.code = LintCodeTag::EmptyParagraph;
        row.params = Some(BTreeMap::from([("marker".into(), "p".into())]));
        let bytes = encoded(&[row]);
        for end in 0..bytes.len() {
            assert!(decoded(&bytes[..end]).is_err());
        }
        for index in 0..bytes.len() {
            let mut corrupt = bytes.clone();
            corrupt[index] ^= 0xff;
            let _ = decoded(&corrupt);
        }
    }
}
