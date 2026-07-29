//! Whole token-section codec: parsed tokens in, wire bytes out, and a validated
//! borrowed decode back against the exact source.
//!
//! The decode side reconstructs core's own `Token<'source>`, not a wire-shaped
//! lookalike, and recovers marker metadata by calling core's registry functions
//! rather than storing or reimplementing them. That is why a section carries a
//! marker-catalog stamp: the recovery is only sound while the registry the
//! encoder saw and the registry the decoder calls agree.

use std::collections::BTreeMap;

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
    /// Present only for a section whose ids are opaque. Omitted together with the
    /// dictionary when ids are positional, which is what the section flag asserts.
    token_id_indices: Vec<u8>,
    token_ids: Vec<u8>,
    token_id_count: u32,
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
        if !self.token_id_indices.is_empty() {
            fields.push(FieldPayload {
                id: token_field::TOKEN_ID_INDEX,
                width: ElementWidth::Four,
                count: self.record_count,
                bytes: &self.token_id_indices,
            });
            fields.push(FieldPayload {
                id: token_field::TOKEN_ID_DICTIONARY,
                width: ElementWidth::Variable,
                count: self.token_id_count,
                bytes: &self.token_ids,
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
            // The flag and the two id fields are one decision: set means the ids
            // are `{book}-{index}` and both fields are absent, clear means they are
            // opaque and both are present.
            variant: SectionVariant::Token {
                positional_ids: self.token_id_indices.is_empty(),
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
    encode_token_section_with_ids(book, source, tokens, None)
}

/// [`encode_token_section`], with opaque stable ids to carry.
///
/// `stable_ids` is `Some` only when the ids are not the positional
/// `{book_code}-{index}` form the decoder can synthesize; passing them otherwise
/// would store a dictionary that is 100% redundant, which is exactly what the
/// `positional_ids` flag exists to avoid.
pub(crate) fn encode_token_section_with_ids(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
    stable_ids: Option<&[&str]>,
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
        token_id_indices: Vec::new(),
        token_ids: Vec::new(),
        token_id_count: 0,
    };
    let mut sids = SidDictionaryBuilder::new(book);
    let mut strings = StringDictionaryBuilder::default();
    let mut descriptors = MarkerDescriptorBuilder::default();
    let fidelity = anchor_fidelity(tokens);

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
            // Fidelity comes from the designator that established this anchor, not
            // from the anchor itself — see `anchor_fidelity`. An anchor no number
            // token established (front matter, chapter scope before any verse) is
            // exact by construction: there is no designator to have spelled wrong.
            Some(sid) => sids.intern(
                sid,
                fidelity.get(&sid).copied().unwrap_or(SidFidelity::Exact),
            )?,
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
    if let Some(ids) = stable_ids {
        if ids.len() != tokens.len() {
            return Err(unbound(tokens.len()));
        }
        let mut dictionary = StringDictionaryBuilder::default();
        for (row, id) in ids.iter().enumerate() {
            // An empty id is not identity: it cannot be told apart from a missing
            // one, and core's `StableTokenId` refuses to hold it.
            if id.is_empty() {
                return Err(unbound(row));
            }
            let index = dictionary.intern(id).map_err(|_| unbound(row))?;
            buffers
                .token_id_indices
                .extend_from_slice(&index.to_le_bytes());
        }
        buffers.token_ids = dictionary.bytes();
        buffers.token_id_count = dictionary.len();
    }
    buffers.sparse.seal_attributes();
    Ok(buffers)
}

/// Fidelity of one number token's anchor, derived from its **source text**.
///
/// Required by the frozen rule: a sequence (`\v 1,3`) is byte-identical to a
/// single verse in core `Sid`, and a suffixed verse (`\v 1a`) is identical to an
/// unsuffixed one in both `Sid` and `NumberRangeKind`, so an encoder reading only
/// the semantic payload would mark both `Exact`. Only the source distinguishes
/// them.
///
/// `Exact` is therefore the narrow case: a bare number, or two bare numbers around
/// a single `-`. A comma (sequence), a letter (suffix), or anything else that does
/// not parse that way is `AnchorOnly` — the anchor is still the correct first
/// canonical reference, but it does not spell the whole designator. A bridge wider
/// than the seven delta bits also degrades, which `PackedSid::encode` applies on
/// its own.
pub(crate) fn source_fidelity(text: &str) -> SidFidelity {
    let trimmed = text.trim();
    let mut parts = trimmed.split('-');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    let bare = |value: &str| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit());
    if parts.next().is_some() || !bare(first) || second.is_some_and(|value| !bare(value)) {
        return SidFidelity::AnchorOnly;
    }
    SidFidelity::Exact
}

/// Fidelity per anchor, keyed by the anchor the establishing number token carries.
///
/// A number token carries the anchor it establishes, and every token after it
/// shares that anchor, so one lookup by `Sid` gives each row the fidelity of the
/// designator it came from. `AnchorOnly` wins a collision: two number tokens can
/// legitimately resolve to the same anchor (a duplicate verse number is a lint
/// finding, not a parse failure), and the inexact reading is the safe one.
pub(crate) fn anchor_fidelity(tokens: &[Token<'_>]) -> BTreeMap<Sid, SidFidelity> {
    let mut table = BTreeMap::new();
    for token in tokens {
        if token.kind() != TokenKind::Number {
            continue;
        }
        if let Some(sid) = token.sid {
            let derived = source_fidelity(token.source);
            let slot = table.entry(sid).or_insert(derived);
            if derived == SidFidelity::AnchorOnly {
                *slot = SidFidelity::AnchorOnly;
            }
        }
    }
    table
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
    // Opaque ids are carried; positional ones are omitted and re-synthesized. The
    // test is what the flag actually claims — that every id is the positional form
    // this stream's own `assign_ids` pass produced — rather than a caller promise.
    let positional = tokens.iter().zip(&borrowed).all(|(owned, token)| {
        owned.id().as_str() == format!("{}-{}", token.id.book_code, token.id.index)
    });
    let stable_ids: Option<Vec<&str>> =
        (!positional).then(|| tokens.iter().map(|token| token.id().as_str()).collect());
    let buffers = encode_token_section_with_ids(book, &source, &borrowed, stable_ids.as_deref())?;
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

/// One decoded token section.
///
/// `stable_ids` is `Some` exactly when the section carried explicit ids, and holds
/// one per token in row order. They are returned alongside rather than inside the
/// tokens because core's `TokenId` is a structured positional label
/// (`{book_code}, {index}`) and cannot hold an opaque caller id — and dropping them
/// would break the identity-keyed reconciliation they exist for. `Token::id` is
/// still filled with that positional label in both cases; for an explicit-id
/// section it is a derived convenience, not the identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedTokens<'a> {
    pub(crate) tokens: Vec<Token<'a>>,
    pub(crate) stable_ids: Option<Vec<&'a str>>,
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
) -> Result<DecodedTokens<'a>, DecodeError> {
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
        Some(field) => NumberRecords::from_field(field, record_count, columns.kinds())?,
        None => NumberRecords::default(),
    };
    let book_codes = match section.field(token_field::BOOK_CODE_RECORDS) {
        Some(field) => BookCodeRecords::from_field(field, record_count, columns.kinds(), strings)?,
        None => BookCodeRecords::default(),
    };
    let attributes = match section.field(token_field::ATTRIBUTE_RECORDS) {
        Some(field) => AttributeRecords::from_field(
            field,
            record_count,
            columns.kinds(),
            section.header.source_len,
            strings,
        )?,
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
    // during parsing, so a positional section stores none of them and cannot drift
    // from it. An explicit-id section also gets the positional label — it is the
    // only thing `TokenId` can hold — but its opaque ids are returned separately
    // and are the identity.
    assign_ids(&mut tokens);
    let stable_ids = if columns.has_explicit_ids() {
        Some(
            (0..record_count)
                .map(|row| columns.token_id(row).ok_or(DecodeError::InvalidSection))
                .collect::<Result<Vec<_>, _>>()?,
        )
    } else {
        None
    };
    Ok(DecodedTokens { tokens, stable_ids })
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
