//! Whole token-section codec: parsed tokens in, wire bytes out, and a validated
//! borrowed decode back against the exact source.
//!
//! The decode side reconstructs core's own `Token<'source>`, not a wire-shaped
//! lookalike, and recovers marker metadata by calling core's registry functions
//! rather than storing or reimplementing them. That is why a section carries a
//! marker-catalog stamp: the recovery is only sound while the registry the
//! encoder saw and the registry the decoder calls agree.

use usfm_onion::marker_defs::structural_marker_info;
use usfm_onion::parse::assign_ids;
use usfm_onion::token::{
    AttributeItem, BookId, MarkerAttrs, OwnedToken, ReconstructedSpans, Sid, Span, Token,
    TokenData, TokenId, TokenKind, marker_metadata, tokens_to_usfm_reconstruct_spanned,
};

use crate::catalog::catalog_stamp;
use crate::container::{ElementWidth, FieldPayload, Section, SectionPayload, SectionVariant};
use crate::error::{DecodeError, EncodeError};
use crate::primitives::source_hash;
use crate::schema::{INDEX_NONE_U16, NumberRangeKindTag, SectionKind, TokenKindTag, token_field};
use crate::token_payload::{
    AttributeRecords, BookCodeRecords, MarkerDescriptorBuilder, MarkerDescriptors, NumberRecords,
    SourceSpan, SparseBuilders, StringDictionary, StringDictionaryBuilder, WireNumber,
};
use crate::token_section::{SidDictionaryBuilder, SidFidelity, TokenColumns};

/// Owned column buffers for one encoded token section. Held separately from the
/// container writer so the borrowed [`SectionPayload`] it lends out stays a view
/// rather than a copy.
#[derive(Debug)]
pub(crate) struct TokenSectionBuffers {
    book: BookId,
    source_hash: u64,
    source_len: u64,
    catalog_stamp: u64,
    record_count: u32,
    kinds: Vec<u8>,
    span_starts: Vec<u8>,
    span_ends: Vec<u8>,
    sid_indices: Vec<u8>,
    descriptor_indices: Vec<u8>,
    sids: Vec<u8>,
    sid_count: u32,
    strings: Vec<u8>,
    string_count: u32,
    descriptors: Vec<u8>,
    descriptor_count: u32,
    sparse: SparseBuilders,
}

impl TokenSectionBuffers {
    pub(crate) fn payload(&self) -> SectionPayload<'_> {
        let mut fields = vec![
            FieldPayload {
                id: token_field::KIND,
                width: ElementWidth::One,
                count: self.record_count,
                bytes: &self.kinds,
            },
            FieldPayload {
                id: token_field::SPAN_START,
                width: ElementWidth::Four,
                count: self.record_count,
                bytes: &self.span_starts,
            },
            FieldPayload {
                id: token_field::SPAN_END,
                width: ElementWidth::Four,
                count: self.record_count,
                bytes: &self.span_ends,
            },
            FieldPayload {
                id: token_field::SID_INDEX,
                width: ElementWidth::Two,
                count: self.record_count,
                bytes: &self.sid_indices,
            },
            FieldPayload {
                id: token_field::MARKER_DESCRIPTOR_INDEX,
                width: ElementWidth::Two,
                count: self.record_count,
                bytes: &self.descriptor_indices,
            },
            FieldPayload {
                id: token_field::STRING_DICTIONARY,
                width: ElementWidth::Variable,
                count: self.string_count,
                bytes: &self.strings,
            },
            FieldPayload {
                id: token_field::MARKER_DESCRIPTOR_DICTIONARY,
                width: ElementWidth::Eight,
                count: self.descriptor_count,
                bytes: &self.descriptors,
            },
            FieldPayload {
                id: token_field::PACKED_SID_DICTIONARY,
                width: ElementWidth::Eight,
                count: self.sid_count,
                bytes: &self.sids,
            },
        ];
        // Sparse fields are omitted entirely when nothing in the book uses them,
        // which is the common case for book codes and attributes.
        if self.sparse.number_count() > 0 {
            fields.push(FieldPayload {
                id: token_field::NUMBER_RECORDS,
                width: ElementWidth::Sixteen,
                count: self.sparse.number_count(),
                bytes: &self.sparse.numbers,
            });
        }
        if self.sparse.book_code_count() > 0 {
            fields.push(FieldPayload {
                id: token_field::BOOK_CODE_RECORDS,
                width: ElementWidth::Sixteen,
                count: self.sparse.book_code_count(),
                bytes: &self.sparse.book_codes,
            });
        }
        if self.sparse.attribute_row_count() > 0 {
            fields.push(FieldPayload {
                id: token_field::ATTRIBUTE_RECORDS,
                width: ElementWidth::Variable,
                count: self.sparse.attribute_row_count(),
                bytes: &self.sparse.attributes,
            });
        }
        SectionPayload {
            // Parsed tokens always carry positional ids, so the explicit id
            // column and its dictionary are omitted; the decoder rebuilds them
            // with core's own assignment pass.
            variant: SectionVariant::Token {
                positional_ids: true,
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

/// Encodes one book's parsed token stream.
///
/// `source` must be the exact bytes the tokens were parsed from: every span is
/// written as-is and verified to still name the token's own text, so a mismatched
/// source is refused here rather than becoming a decode-time surprise.
pub(crate) fn encode_token_section(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
) -> Result<TokenSectionBuffers, EncodeError> {
    let unbound = |token_idx: usize| EncodeError::UnboundSpan {
        book,
        token_idx: token_idx as u32,
    };
    let record_count = u32::try_from(tokens.len()).map_err(|_| unbound(tokens.len()))?;
    u32::try_from(source.len()).map_err(|_| unbound(0))?;

    let mut buffers = TokenSectionBuffers {
        book,
        source_hash: source_hash(source),
        source_len: source.len() as u64,
        catalog_stamp: catalog_stamp(),
        record_count,
        kinds: Vec::with_capacity(tokens.len()),
        span_starts: Vec::with_capacity(tokens.len() * 4),
        span_ends: Vec::with_capacity(tokens.len() * 4),
        sid_indices: Vec::with_capacity(tokens.len() * 2),
        descriptor_indices: Vec::with_capacity(tokens.len() * 2),
        sids: Vec::new(),
        sid_count: 0,
        strings: Vec::new(),
        string_count: 0,
        descriptors: Vec::new(),
        descriptor_count: 0,
        sparse: SparseBuilders::default(),
    };
    let mut sids = SidDictionaryBuilder::new(book);
    let mut strings = StringDictionaryBuilder::default();
    let mut descriptors = MarkerDescriptorBuilder::default();

    for (row, token) in tokens.iter().enumerate() {
        let index = row as u32;
        buffers.kinds.push(TokenKindTag::from(token.kind()).as_u8());
        let span = bind_span(source, token.span, token.source).ok_or_else(|| unbound(row))?;
        buffers
            .span_starts
            .extend_from_slice(&span.start.to_le_bytes());
        buffers
            .span_ends
            .extend_from_slice(&(span.start + span.len).to_le_bytes());

        let sid_index = match token.sid {
            // Token anchors are the canonical sid core computed, so they are
            // interned as exact; the packed codec still degrades a bridge wider
            // than its seven delta bits on its own. Sequence/suffix fidelity is
            // a finding-level concern, derived there from the number token's
            // source text.
            Some(sid) => sids.intern(sid, SidFidelity::Exact)?,
            None => INDEX_NONE_U16,
        };
        buffers
            .sid_indices
            .extend_from_slice(&sid_index.to_le_bytes());

        let descriptor_index = match marker_of(token) {
            Some((name, nested)) => {
                let name_index = strings.intern(name).map_err(|_| unbound(row))?;
                descriptors
                    .intern(name_index, nested)
                    .ok_or(EncodeError::TooManyDescriptors {
                        book,
                        found: descriptors.len(),
                    })?
            }
            None => INDEX_NONE_U16,
        };
        buffers
            .descriptor_indices
            .extend_from_slice(&descriptor_index.to_le_bytes());

        match &token.data {
            TokenData::Number { start, end, kind } => buffers.sparse.push_number(
                index,
                WireNumber {
                    start: *start,
                    end: *end,
                    kind: NumberRangeKindTag::from(*kind),
                },
            ),
            TokenData::BookCode { code, is_valid } => {
                let code_index = strings.intern(code).map_err(|_| unbound(row))?;
                buffers.sparse.push_book_code(index, code_index, *is_valid);
            }
            _ => {}
        }

        if let Some(attrs) = token_attrs(token) {
            let list_source = match attrs.attribute_source {
                Some((span, text)) => {
                    Some(bind_span(source, span, text).ok_or_else(|| unbound(row))?)
                }
                None => None,
            };
            let first = buffers.sparse.push_attribute_row(index, list_source);
            for attribute in &attrs.attributes {
                let key = strings.intern(attribute.key).map_err(|_| unbound(row))?;
                let value = strings.intern(attribute.value).map_err(|_| unbound(row))?;
                let source_span = bind_span(source, attribute.span, attribute.source)
                    .ok_or_else(|| unbound(row))?;
                buffers
                    .sparse
                    .push_attribute_entry(key, value, source_span, attribute.is_default);
            }
            buffers.sparse.finish_attribute_row(first);
        }
    }

    buffers.sids = sids.bytes().to_vec();
    buffers.sid_count = sids.len();
    buffers.strings = strings.bytes();
    buffers.string_count = strings.len();
    buffers.descriptors = descriptors.bytes().to_vec();
    buffers.descriptor_count = descriptors.len();
    buffers.sparse.seal_attributes();
    Ok(buffers)
}

/// Encodes one book from spanless owned tokens.
///
/// Owned tokens carry no spans, and the wire's span columns are required, so the
/// source is serialized first and the spans come back from that same emission
/// pass. Returns the derived source because it — not the caller's original file,
/// which may not exist — is what the section's spans and hash are bound to, and
/// what a decoder must be handed back.
///
/// This is the cold path: a resident book is encoded once per snapshot, not per
/// keystroke.
pub(crate) fn encode_owned_token_section(
    book: BookId,
    tokens: &[OwnedToken],
) -> Result<(String, TokenSectionBuffers), EncodeError> {
    let (source, spans) = tokens_to_usfm_reconstruct_spanned(tokens);
    let borrowed = owned_to_borrowed(book, tokens, &spans, &source)?;
    let buffers = encode_token_section(book, &source, &borrowed)?;
    Ok((source, buffers))
}

/// Rebuilds borrowed tokens over the serialized source, so the owned path feeds
/// the same encoder the parsed path uses instead of a second one.
///
/// Marker metadata and structural info are recovered from the name through
/// core's registry, exactly as decoding does — an owned token's copies are the
/// same values, and reading them back from core keeps one source of truth.
pub(crate) fn owned_to_borrowed<'a>(
    book: BookId,
    tokens: &'a [OwnedToken],
    spans: &[ReconstructedSpans],
    source: &'a str,
) -> Result<Vec<Token<'a>>, EncodeError> {
    let unbound = |row: usize| EncodeError::UnboundSpan {
        book,
        token_idx: row as u32,
    };
    let mut out = Vec::with_capacity(tokens.len());
    for (row, (token, span)) in tokens.iter().zip(spans).enumerate() {
        let text = source
            .get(span.token.as_range())
            .ok_or_else(|| unbound(row))?;
        let attrs = owned_attrs(book, token, span, source, row)?;
        let data = match token.kind() {
            TokenKind::Newline => TokenData::Newline,
            TokenKind::OptBreak => TokenData::OptBreak,
            TokenKind::MilestoneEnd => TokenData::MilestoneEnd,
            TokenKind::Text => TokenData::Text,
            TokenKind::BookCode => {
                let code = token.book_code().ok_or_else(|| unbound(row))?;
                TokenData::BookCode {
                    code: code.code.as_ref(),
                    is_valid: code.is_valid,
                }
            }
            TokenKind::Number => {
                let number = token.number_info().ok_or_else(|| unbound(row))?;
                TokenData::Number {
                    start: number.start,
                    end: number.end,
                    kind: number.kind,
                }
            }
            kind @ (TokenKind::Marker | TokenKind::EndMarker | TokenKind::Milestone) => {
                let name = token.marker_name().ok_or_else(|| unbound(row))?;
                let metadata = marker_metadata(name);
                let structural = structural_marker_info(name, metadata.kind);
                match kind {
                    TokenKind::EndMarker => TokenData::EndMarker {
                        name,
                        metadata,
                        structural,
                        nested: token.nested(),
                    },
                    TokenKind::Milestone => TokenData::Milestone {
                        name,
                        metadata,
                        structural,
                        attrs,
                    },
                    _ => TokenData::Marker {
                        name,
                        metadata,
                        structural,
                        nested: token.nested(),
                        attrs,
                    },
                }
            }
        };
        out.push(Token {
            id: TokenId::new("", 0),
            sid: token.parsed_sid(),
            span: span.token,
            source: text,
            data,
        });
    }
    assign_ids(&mut out);
    Ok(out)
}

/// Locates an owned token's attribute list and each of its entries inside the
/// serialized source.
///
/// The list span comes from the emitter, which is the only thing that knows where
/// a deferred list landed. Entry spans are then found within it by a
/// left-to-right scan, because entries are substrings of a verbatim list. A
/// synthetically-built token whose entry text is not in its own list has no span
/// to record and is refused — the guard for the formatter-synthetic edge.
fn owned_attrs<'a>(
    book: BookId,
    token: &'a OwnedToken,
    span: &ReconstructedSpans,
    source: &'a str,
    row: usize,
) -> Result<Option<Box<MarkerAttrs<'a>>>, EncodeError> {
    let unbound = EncodeError::UnboundSpan {
        book,
        token_idx: row as u32,
    };
    // Mirrors the emitter's own condition: with nothing to emit there is nothing
    // to locate, and a re-parse of the serialized source would report no list
    // either.
    if token.attribute_list().is_none() && token.attributes().is_empty() {
        return Ok(None);
    }
    let Some(list) = span.attribute_list else {
        return Err(unbound);
    };
    let list_text = source.get(list.as_range()).ok_or(unbound)?;

    let mut cursor = 0usize;
    let mut attributes = Vec::with_capacity(token.attributes().len());
    for attribute in token.attributes() {
        let offset = list_text
            .get(cursor..)
            .and_then(|rest| rest.find(attribute.source.as_ref()))
            .ok_or(unbound)?;
        let start = list.start as usize + cursor + offset;
        let end = start + attribute.source.len();
        cursor += offset + attribute.source.len();
        attributes.push(AttributeItem {
            span: Span::new(start as u32, end as u32),
            source: source.get(start..end).ok_or(unbound)?,
            key: attribute.key.as_ref(),
            value: attribute.value.as_ref(),
            is_default: attribute.is_default,
        });
    }
    Ok(Some(Box::new(MarkerAttrs {
        attributes,
        // The list is read back out of the serialized source rather than off the
        // input token. After serialization it is genuinely there, so recording it
        // makes the loop idempotent: decoding yields what a parse of this source
        // would. A caller who supplied structured attributes with no verbatim
        // slice therefore gets the serialized spelling back.
        //
        // The absent case is still distinct: a token with neither a list nor any
        // entry returned early above with no attribute row at all, so `None` and
        // `Some("")` do not collapse — the latter keeps a present, empty span.
        attribute_source: Some((list, list_text)),
    })))
}

/// The attribute payload of a marker or milestone that carries one.
fn token_attrs<'t, 'a>(token: &'t Token<'a>) -> Option<&'t MarkerAttrs<'a>> {
    match &token.data {
        TokenData::Marker { attrs, .. } | TokenData::Milestone { attrs, .. } => attrs.as_deref(),
        _ => None,
    }
}

/// The marker name and nesting of a token that has one.
fn marker_of<'a>(token: &Token<'a>) -> Option<(&'a str, bool)> {
    match token.data {
        TokenData::Marker { name, nested, .. } | TokenData::EndMarker { name, nested, .. } => {
            Some((name, nested))
        }
        TokenData::Milestone { name, .. } => Some((name, false)),
        _ => None,
    }
}

/// Confirms a span still names `text` in `source` and narrows it to the wire's
/// `u32` fields. Returning `None` is the encoder's refusal signal: a span that
/// does not bind would decode to different bytes than it was written from.
fn bind_span(source: &str, span: Span, text: &str) -> Option<SourceSpan> {
    if span.end < span.start || source.get(span.start as usize..span.end as usize)? != text {
        return None;
    }
    Some(SourceSpan {
        start: span.start,
        len: span.end - span.start,
    })
}

/// Decodes one token section against the exact source it was encoded from.
///
/// The returned tokens borrow from **both** buffers — marker names and attribute
/// text come from the wire's dictionaries, token text from the source — so the
/// two share one lifetime parameter here. Callers with separately-owned buffers
/// simply get the shorter of the two.
///
/// Binding happens before any span is dereferenced: length first because it is a
/// single compare, then the content hash, which is what catches a same-length
/// different-bytes source. A hash match is not treated as proof — every span is
/// still range- and character-boundary-checked.
pub(crate) fn decode_token_section<'a>(
    section: &Section<'a>,
    source: &'a str,
) -> Result<Vec<Token<'a>>, DecodeError> {
    if section.header.kind != SectionKind::Token {
        return Err(DecodeError::InvalidSection);
    }
    if section.header.source_len != source.len() as u64 {
        return Err(DecodeError::SourceLengthMismatch);
    }
    if section.header.source_hash != source_hash(source) {
        return Err(DecodeError::SourceHashMismatch);
    }
    // Marker ordinals and name-based metadata recovery are only meaningful under
    // the registry the encoder saw.
    if section.header.catalog_stamp != catalog_stamp() {
        return Err(DecodeError::CatalogMismatch);
    }

    let columns = TokenColumns::from_section(section)?;
    let strings = match section.field(token_field::STRING_DICTIONARY) {
        Some(field) => StringDictionary::from_field(field)?,
        None => return Err(DecodeError::InvalidSection),
    };
    let descriptors = match section.field(token_field::MARKER_DESCRIPTOR_DICTIONARY) {
        Some(field) => MarkerDescriptors::from_field(field, strings)?,
        None => return Err(DecodeError::InvalidSection),
    };
    let record_count = section.header.record_count;
    let numbers = match section.field(token_field::NUMBER_RECORDS) {
        Some(field) => NumberRecords::from_field(field, record_count)?,
        None => NumberRecords::default(),
    };
    let book_codes = match section.field(token_field::BOOK_CODE_RECORDS) {
        Some(field) => BookCodeRecords::from_field(field, record_count, strings)?,
        None => BookCodeRecords::default(),
    };
    let attributes = match section.field(token_field::ATTRIBUTE_RECORDS) {
        Some(field) => {
            AttributeRecords::from_field(field, record_count, section.header.source_len, strings)?
        }
        None => AttributeRecords::default(),
    };

    let mut tokens = Vec::with_capacity(record_count as usize);
    for row in 0..record_count {
        let kind = columns.kind(row).ok_or(DecodeError::InvalidSection)?;
        let (start, end) = columns.span(row).ok_or(DecodeError::InvalidSection)?;
        // `str::get` rejects both an out-of-range span and one that splits a
        // character, so one lookup covers the range and the encoding.
        let text = source
            .get(start as usize..end as usize)
            .ok_or(DecodeError::InvalidUtf8)?;
        let data = decode_payload(
            kind,
            row,
            &columns,
            &descriptors,
            &numbers,
            &book_codes,
            &attributes,
            source,
        )?;
        tokens.push(Token {
            // Filled by core's own assignment pass below.
            id: TokenId::new("", 0),
            sid: decode_sid(&columns, row)?,
            span: Span::new(start, end),
            source: text,
            data,
        });
    }
    // Positional ids are reproduced by the same core function that assigned them
    // during parsing, so the wire stores none of them and cannot drift from it.
    assign_ids(&mut tokens);
    Ok(tokens)
}

fn decode_sid(columns: &TokenColumns<'_>, row: u32) -> Result<Option<Sid>, DecodeError> {
    let index = columns.sid_index(row).ok_or(DecodeError::InvalidSection)?;
    if index == INDEX_NONE_U16 {
        return Ok(None);
    }
    columns
        .sids()
        .get(index)
        .map(|(sid, _)| Some(sid))
        .ok_or(DecodeError::InvalidSection)
}

#[allow(clippy::too_many_arguments)]
fn decode_payload<'source>(
    kind: TokenKindTag,
    row: u32,
    columns: &TokenColumns<'_>,
    descriptors: &MarkerDescriptors<'source>,
    numbers: &NumberRecords<'_>,
    book_codes: &BookCodeRecords<'source>,
    attributes: &AttributeRecords<'source>,
    source: &'source str,
) -> Result<TokenData<'source>, DecodeError> {
    let descriptor_index = columns
        .marker_descriptor_index(row)
        .ok_or(DecodeError::InvalidSection)?;
    let descriptor = match descriptor_index {
        INDEX_NONE_U16 => None,
        index => Some(descriptors.get(index).ok_or(DecodeError::InvalidSection)?),
    };
    // The legal-payload table pairs each kind with exactly one shape. A row that
    // carries a descriptor it may not have, or lacks one it must, is a different
    // format rather than a recoverable row.
    let marker_bearing = matches!(
        kind,
        TokenKindTag::Marker | TokenKindTag::EndMarker | TokenKindTag::Milestone
    );
    if marker_bearing != descriptor.is_some() {
        return Err(DecodeError::InvalidSection);
    }
    let attribute_list = attributes.get(row);
    if attribute_list.is_some() && !matches!(kind, TokenKindTag::Marker | TokenKindTag::Milestone) {
        return Err(DecodeError::InvalidSection);
    }

    let data = match kind {
        TokenKindTag::Newline => TokenData::Newline,
        TokenKindTag::OptBreak => TokenData::OptBreak,
        TokenKindTag::MilestoneEnd => TokenData::MilestoneEnd,
        TokenKindTag::Text => TokenData::Text,
        TokenKindTag::Marker | TokenKindTag::EndMarker | TokenKindTag::Milestone => {
            let (name, nested) = descriptor.ok_or(DecodeError::InvalidSection)?;
            // Metadata and structural info are pure functions of the name in
            // core, so they are recovered rather than stored — the reason the
            // catalog stamp gates this whole section.
            let metadata = marker_metadata(name);
            let structural = structural_marker_info(name, metadata.kind);
            match kind {
                TokenKindTag::EndMarker => TokenData::EndMarker {
                    name,
                    metadata,
                    structural,
                    nested,
                },
                TokenKindTag::Milestone => {
                    // A milestone has no `nested` field at all, so a descriptor
                    // claiming one cannot be describing this row.
                    if nested {
                        return Err(DecodeError::InvalidSection);
                    }
                    TokenData::Milestone {
                        name,
                        metadata,
                        structural,
                        attrs: decode_attrs(attribute_list, source)?,
                    }
                }
                _ => TokenData::Marker {
                    name,
                    metadata,
                    structural,
                    nested,
                    attrs: decode_attrs(attribute_list, source)?,
                },
            }
        }
        TokenKindTag::BookCode => {
            let (code, is_valid) = book_codes.get(row).ok_or(DecodeError::InvalidSection)?;
            TokenData::BookCode { code, is_valid }
        }
        TokenKindTag::Number => {
            let number = numbers.get(row).ok_or(DecodeError::InvalidSection)?;
            TokenData::Number {
                start: number.start,
                end: number.end,
                kind: number.kind.into(),
            }
        }
    };
    Ok(data)
}

fn decode_attrs<'source>(
    list: Option<crate::token_payload::WireAttributeList<'source>>,
    source: &'source str,
) -> Result<Option<Box<MarkerAttrs<'source>>>, DecodeError> {
    let Some(list) = list else {
        return Ok(None);
    };
    let attribute_source = match list.list_source {
        Some(span) => Some((
            Span::new(span.start, span.start + span.len),
            slice(source, span)?,
        )),
        None => None,
    };
    let mut attributes = Vec::with_capacity(list.attributes.len());
    for attribute in list.attributes {
        attributes.push(AttributeItem {
            span: Span::new(
                attribute.source.start,
                attribute.source.start + attribute.source.len,
            ),
            source: slice(source, attribute.source)?,
            key: attribute.key,
            value: attribute.value,
            is_default: attribute.is_default,
        });
    }
    Ok(Some(Box::new(MarkerAttrs {
        attributes,
        attribute_source,
    })))
}

fn slice(source: &str, span: SourceSpan) -> Result<&str, DecodeError> {
    let start = span.start as usize;
    let end = start + span.len as usize;
    source.get(start..end).ok_or(DecodeError::InvalidUtf8)
}
