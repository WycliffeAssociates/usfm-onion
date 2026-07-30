//! Semantic finding codec: `LintIssue` values in, packed finding-section bytes
//! out, and back.
//!
//! This is the layer [`crate::finding_section`] deliberately stopped short of:
//! that module proves the byte-level shape of a finding section (fixed rows,
//! sidecars, dictionaries), but a checked row is not yet a `LintIssue` — it
//! still needs the paired token section (to resolve `token_idx` back to a
//! `token_id`/span/SID/marker) and the rule catalog (to rebuild `message` and
//! validate `message_params`). This module is that seam.
//!
//! Scope: `LintIssue.fix` is never encoded. Common-row flag bit 5 (`fix`)
//! always stays clear, and decode always produces `fix: None` — codes with a
//! `TokenFix` lose it on a wire round trip, every other field round-trips.
//!
//! `message` is rebuilt **only** via [`LintCode::render_message`] — never a
//! wire-local template renderer — so the English text a decoder produces can
//! never drift from core's own rendering of the same code and params.
//! `encode_book` conversely refuses (rather than silently overwrites) a
//! caller-supplied `LintIssue` whose derived fields (`category`, `severity`,
//! `issue_type`, `template`, `message`) do not match what the catalog would
//! produce for that code and its own `message_params` — the wire format has
//! no storage for a divergent value, so accepting one and rewriting it on
//! decode would silently change the caller's data.

use std::collections::BTreeMap;

use usfm_onion::lint::{LintCode, LintIssue, MessageParams, NO_TOKEN_POSITION};
use usfm_onion::token::{BookId, Sid, Span, Token};

use crate::catalog::{catalog_marker_name, catalog_ordinal};
use crate::container::{read_container, write_container};
use crate::error::{DecodeError, EncodeError};
use crate::finding_section::{
    FindingColumns, FindingDecodeInputs, FindingRowInput, FindingSectionBuffers, MarkerRef,
};
use crate::schema::{LintCodeTag, SectionKind, param_contract};
use crate::token_codec::{
    DecodedTokens, anchor_fidelity, decode_token_section, encode_token_section,
};

/// Encodes one book's parsed tokens and lint issues into a single container
/// with a token section and a paired finding section.
///
/// `issues` need not already be in canonical order: this function sorts a
/// local copy before building rows, so the container's finding order is
/// always canonical order regardless of the caller's order.
pub fn encode_book(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
    issues: &[LintIssue],
) -> Result<Vec<u8>, EncodeError> {
    let token_buffers = encode_token_section(book, source, tokens)?;
    let finding_buffers = encode_findings(book, source, tokens, issues)?;
    write_container(0, &[token_buffers.payload(), finding_buffers.payload()])
}

/// Decodes a container written by [`encode_book`] (or any producer of the
/// same paired token+finding section shape) back into `LintIssue` values,
/// against the exact source the container was bound to.
///
/// The container must carry exactly one token section and exactly one
/// finding section, naming the same book — a multi-book container, a
/// container missing either section kind, or one whose token/finding
/// sections disagree about which book they belong to, all reject rather than
/// silently pairing sections that do not actually belong together.
pub fn decode_book<'a>(bytes: &'a [u8], source: &'a str) -> Result<Vec<LintIssue>, DecodeError> {
    let container = read_container(bytes)?;
    let toc = container.toc();
    let mut token_index = None;
    let mut finding_index = None;
    for (index, entry) in toc.iter().enumerate() {
        let slot = match entry.kind {
            SectionKind::Token => &mut token_index,
            SectionKind::Finding => &mut finding_index,
        };
        if slot.is_some() {
            return Err(DecodeError::InvalidToc);
        }
        *slot = Some(index);
    }
    let token_index = token_index.ok_or(DecodeError::InvalidToc)?;
    let finding_index = finding_index.ok_or(DecodeError::InvalidToc)?;
    if toc[token_index].book != toc[finding_index].book {
        return Err(DecodeError::InvalidToc);
    }
    let book = toc[token_index].book;
    let token_section = container
        .section(token_index)
        .ok_or(DecodeError::InvalidToc)??;
    let finding_section = container
        .section(finding_index)
        .ok_or(DecodeError::InvalidToc)??;

    let decoded_tokens = decode_token_section(&token_section, source)?;
    decode_finding_section(&finding_section, book, source, &decoded_tokens)
}

/// Trust-checks one finding section against the live source and registry, then
/// materializes its rows against the already-decoded token section.
///
/// Shared by [`decode_book`] and [`crate::verify::verify_book`] so the three
/// checks below exist once: a second copy is exactly how one caller would
/// quietly end up trusting a catalog-derived value the other refuses.
pub(crate) fn decode_finding_section<'a>(
    finding_section: &crate::container::Section<'a>,
    book: BookId,
    source: &'a str,
    decoded_tokens: &DecodedTokens<'a>,
) -> Result<Vec<LintIssue>, DecodeError> {
    let token_ids = resolve_token_ids(decoded_tokens);
    let inputs = FindingDecodeInputs {
        token_count: u32::try_from(decoded_tokens.tokens.len())
            .map_err(|_| DecodeError::OffsetOverflow)?,
    };
    let columns = FindingColumns::from_section(finding_section, inputs)?;
    // The finding section carries its own copy of the exact facts the token
    // section already bound to (source length/hash, marker-catalog stamp);
    // both must be checked against the live source and registry before any
    // catalog-derived value (e.g. a marker looked up by ordinal) is trusted.
    if columns.source_len != source.len() as u64 {
        return Err(DecodeError::SourceLengthMismatch);
    }
    if columns.source_hash != crate::primitives::source_hash(source) {
        return Err(DecodeError::SourceHashMismatch);
    }
    if columns.catalog_stamp != crate::catalog::catalog_stamp() {
        return Err(DecodeError::CatalogMismatch);
    }
    decode_findings(book, source, &decoded_tokens.tokens, &token_ids, &columns)
}

/// The token id every row would report through [`usfm_onion::lint::LintableToken::id`]:
/// the section's own opaque ids when it carries them, else the positional
/// label `assign_ids` (already re-applied by `decode_token_section`) stamped
/// on every decoded token.
fn resolve_token_ids(decoded: &DecodedTokens<'_>) -> Vec<String> {
    use usfm_onion::lint::LintableToken;
    match &decoded.stable_ids {
        Some(ids) => ids.iter().map(|id| (*id).to_string()).collect(),
        None => decoded
            .tokens
            .iter()
            .map(|token| token.id().expect("Token::id always resolves"))
            .collect(),
    }
}

/// Sorts `issues` into canonical finding order: the primary anchor token's
/// row position (a finding with no anchor token sorts last), then the rule's
/// kebab code string (not its numeric discriminant), then the related anchor
/// token's row position (a finding with no related token sorts last).
///
/// `resolve_row` maps a finding's own `token_id`/`related_token_id` string to
/// the row index that identifies its position in the token stream; it
/// returns `None` for an absent id or one that names no token. This is the
/// key the packed finding section's rows are physically stored in, so a
/// caller comparing an unsorted `LintResult` against a decoded one needs the
/// same key — [`encode_book`]/[`encode_findings`] always apply it before
/// building rows.
///
/// Same 3-key shape as core's own (private) `canonical_sort` — the two are
/// not shared code, since this function is deliberately generic over an
/// arbitrary row-resolution closure (built once and reused for both encoding
/// and test/reproduction callers, a shape core's `LintableToken`-only sort
/// has no reason to grow) and each is pinned by its own gate (core's lint
/// oracle; this crate's finding corpus round-trip gate). They agree by
/// construction, not by delegation.
pub fn canonical_order(
    issues: &mut [LintIssue],
    resolve_row: impl Fn(Option<&str>) -> Option<u32>,
) {
    let key = |issue: &LintIssue| -> (u32, &str, u32) {
        (
            resolve_row(issue.token_id.as_deref()).unwrap_or(u32::MAX),
            issue.code.code(),
            resolve_row(issue.related_token_id.as_deref()).unwrap_or(u32::MAX),
        )
    };
    issues.sort_by(|a, b| key(a).cmp(&key(b)));
}

/// Convenience wrapper over [`canonical_order`] for a caller that has the
/// same token slice [`encode_book`] would encode against: builds the
/// `token_id -> row` lookup the same way the encoder does, then sorts by it.
/// Intended for tests and callers reproducing the encoder's order externally,
/// not for the encoder itself (which needs the same lookup for row
/// resolution too, and builds it once for both uses).
pub fn canonical_order_for_tokens(issues: &mut [LintIssue], tokens: &[Token<'_>]) {
    use usfm_onion::lint::LintableToken;
    let mut rows: BTreeMap<String, u32> = BTreeMap::new();
    for (row, token) in tokens.iter().enumerate() {
        if let Some(id) = token.id() {
            rows.insert(id, row as u32);
        }
    }
    canonical_order(issues, |id| id.and_then(|id| rows.get(id).copied()));
}

/// Builds the finding-section buffers for one book's issues, in canonical
/// order, against the same source/tokens the paired token section binds to.
pub(crate) fn encode_findings(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
    issues: &[LintIssue],
) -> Result<FindingSectionBuffers, EncodeError> {
    use usfm_onion::lint::LintableToken;

    // A finding's `token_id`/`related_token_id` are opaque strings (positional
    // or caller-supplied); the only general way back to a row index is a
    // reverse lookup built from each token's own `id()`. Built before sorting
    // because canonical order is itself keyed on these row positions.
    let mut ids: Vec<String> = Vec::with_capacity(tokens.len());
    for token in tokens {
        ids.push(
            token
                .id()
                .ok_or(EncodeError::UnboundSpan { book, token_idx: 0 })?,
        );
    }
    let mut resolver: BTreeMap<&str, u32> = BTreeMap::new();
    for (row, id) in ids.iter().enumerate() {
        resolver.insert(id.as_str(), row as u32);
    }

    let mut sorted: Vec<LintIssue> = issues.to_vec();
    canonical_order(&mut sorted, |id| {
        id.and_then(|id| resolver.get(id).copied())
    });

    let fidelity = anchor_fidelity(tokens);
    let mut rows = Vec::with_capacity(sorted.len());
    for issue in &sorted {
        rows.push(issue_to_row(
            book, source, tokens, &resolver, &fidelity, issue,
        )?);
    }
    FindingSectionBuffers::new(
        book,
        crate::primitives::source_hash(source),
        source.len() as u64,
        crate::catalog::catalog_stamp(),
        &rows,
    )
}

fn unrepresentable(book: BookId, code: LintCode) -> EncodeError {
    EncodeError::UnrepresentablePayload {
        book,
        code: LintCodeTag::from(code) as u8,
    }
}

/// Resolves an anchor's whole-token span vs. a sub-range within it into the
/// row's `(offset, len)` pair. `len == 0` is the "whole token" sentinel: every
/// real corpus finding uses it today, since a finding's span is always
/// exactly its anchor token's own span.
fn span_within_token(
    book: BookId,
    code: LintCode,
    token_span: Span,
    span: Span,
) -> Result<(u32, u32), EncodeError> {
    if span == token_span {
        return Ok((0, 0));
    }
    if span.start >= token_span.start && span.end <= token_span.end && span.end > span.start {
        return Ok((span.start - token_span.start, span.end - span.start));
    }
    Err(unrepresentable(book, code))
}

fn issue_to_row(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
    resolver: &BTreeMap<&str, u32>,
    fidelity: &BTreeMap<Sid, crate::token_section::SidFidelity>,
    issue: &LintIssue,
) -> Result<FindingRowInput, EncodeError> {
    let code = issue.code;
    let err = || unrepresentable(book, code);

    // The wire format has no storage for a caller-supplied `category`/
    // `severity`/`issue_type`/`template`/`message` that disagrees with what
    // the catalog derives for this code — decode always recomputes them from
    // `code`, so accepting a divergent value here would silently discard it
    // rather than round-trip it.
    if issue.category != code.category()
        || issue.severity != code.severity()
        || issue.issue_type != code.issue_type()
        || issue.template != code.template()
        || issue.message != code.render_message(&issue.message_params)
    {
        return Err(err());
    }

    if issue.token_id.is_some() != issue.span.is_some() {
        return Err(err());
    }
    let token_row = match &issue.token_id {
        Some(id) => Some(*resolver.get(id.as_str()).ok_or_else(err)?),
        None => None,
    };
    let (offset, len) = match (issue.span, token_row) {
        (None, None) => (0u32, 0u32),
        (Some(span), Some(row)) => span_within_token(book, code, tokens[row as usize].span, span)?,
        _ => return Err(err()),
    };

    // Whether a finding has an SID is independent of whether it is
    // token-anchored: a token-anchored finding can still have no SID at all
    // (e.g. content fired on before any chapter marker establishes an
    // anchor) — that is exactly what distinguishes "no SID" from a
    // legitimate chapter-0/verse-0 front-matter SID. What *is* required is a
    // token to read the raw `Sid` from whenever one is present, since
    // `LintIssue.sid` is only a formatted display string, not a structured
    // value this codec could otherwise reconstruct.
    let (chapter, verse, range_end, anchor_only) = match issue.sid.as_deref() {
        None => (None, None, None, false),
        Some(text) => {
            let row = token_row.ok_or_else(err)?;
            let sid = tokens[row as usize].sid.ok_or_else(err)?;
            if sid.to_string() != text {
                // The finding's SID must be exactly its anchor token's own
                // current SID; a caller that violates that has no raw `Sid`
                // for this codec to encode from a formatted string alone.
                return Err(err());
            }
            let delta = sid.verse_end().saturating_sub(sid.verse);
            if delta > u16::from(u8::MAX) {
                // The row's range-end column is one byte; a bridge wider than
                // that has no storage at all, and unlike a token's packed SID
                // (which still keeps the verbatim source text to fall back
                // on) a finding's SID is semantic data with no backup
                // representation, so refuse rather than silently drop the
                // range. In practice `Sid::verse_end()` itself saturates at
                // `verse + 255` (a stated contract of `Sid::with_range`), so
                // `delta` can never actually exceed 255 for any real `Sid`
                // value — this branch is unreachable today and exists only so
                // a future widening of that ceiling could not silently start
                // losing data here instead of refusing.
                return Err(err());
            }
            let source_anchor_only =
                fidelity.get(&sid).copied() == Some(crate::token_section::SidFidelity::AnchorOnly);
            (
                Some(sid.chapter),
                Some(sid.verse),
                (delta != 0).then_some(delta as u8),
                source_anchor_only,
            )
        }
    };

    let related_token_row = match &issue.related_token_id {
        Some(id) => Some(*resolver.get(id.as_str()).ok_or_else(err)?),
        None => None,
    };
    if issue.related_token_id.is_some() != issue.related_span.is_some() {
        return Err(err());
    }
    let related = match (issue.related_span, related_token_row) {
        (None, None) => None,
        (Some(span), Some(row)) => {
            let (offset, len) = span_within_token(book, code, tokens[row as usize].span, span)?;
            Some((row, offset, len))
        }
        _ => return Err(err()),
    };

    let natural_marker = token_row.and_then(|row| tokens[row as usize].marker_name());
    let marker = match (issue.marker.as_deref(), natural_marker) {
        (None, None) => MarkerRef::AnchoredToken,
        (Some(m), Some(n)) if m == n => MarkerRef::AnchoredToken,
        (None, Some(_)) => MarkerRef::Absent,
        (Some(m), _) => {
            if let Some(ordinal) = catalog_ordinal(m) {
                MarkerRef::CatalogOrdinal(ordinal)
            } else if let Some(offset) = source.find(m) {
                let len = u8::try_from(m.len()).map_err(|_| err())?;
                MarkerRef::SourceSpan {
                    offset: offset as u32,
                    len,
                }
            } else {
                return Err(err());
            }
        }
    };

    let params = if param_contract(LintCodeTag::from(code)).is_some() {
        Some(issue.message_params.clone())
    } else if issue.message_params.is_empty() {
        None
    } else {
        return Err(err());
    };

    Ok(FindingRowInput {
        token_idx: token_row,
        offset,
        len,
        chapter,
        verse,
        range_end,
        code: LintCodeTag::from(code),
        anchor_only,
        related,
        marker,
        params,
    })
}

/// Rebuilds `LintIssue` values from checked finding-section rows, resolving
/// each row's token references against the paired, already-decoded token
/// section.
pub(crate) fn decode_findings(
    book: BookId,
    source: &str,
    tokens: &[Token<'_>],
    token_ids: &[String],
    columns: &FindingColumns<'_>,
) -> Result<Vec<LintIssue>, DecodeError> {
    let mut issues = Vec::with_capacity(columns.rows().len());
    for row in columns.rows() {
        let code: LintCode = row.code.into();
        let token = row
            .token_idx
            .map(|idx| tokens.get(idx as usize).ok_or(DecodeError::InvalidSection))
            .transpose()?;
        let token_id = row
            .token_idx
            .map(|idx| {
                token_ids
                    .get(idx as usize)
                    .cloned()
                    .ok_or(DecodeError::InvalidSection)
            })
            .transpose()?;
        let span = token
            .map(|t| resolve_span(t.span, row.offset, row.len))
            .transpose()?;

        let related_token_id = row
            .related
            .map(|(idx, ..)| {
                token_ids
                    .get(idx as usize)
                    .cloned()
                    .ok_or(DecodeError::InvalidSection)
            })
            .transpose()?;
        let related_span = row
            .related
            .map(|(idx, offset, len)| -> Result<Span, DecodeError> {
                let related_token = tokens
                    .get(idx as usize)
                    .ok_or(DecodeError::InvalidSection)?;
                resolve_span(related_token.span, offset, len)
            })
            .transpose()?;

        let sid = match row.chapter {
            None => None,
            Some(chapter) => {
                let verse = row.verse.ok_or(DecodeError::InvalidSection)?;
                let delta = u16::from(row.range_end.unwrap_or(0));
                // The row stores chapter/verse/range only; a finding's SID
                // book is whatever its anchor token's own SID carries (which
                // need not be the section's own book — e.g. an unmodified
                // lowercase book code, still a legal anchor). The section's
                // `book` is only a fallback for a finding that carries an SID
                // but has no anchor token to read one from.
                let row_book = token
                    .and_then(|t| t.sid)
                    .map(|sid| sid.book)
                    .unwrap_or(book);
                let sid = if delta == 0 {
                    Sid::new(row_book, chapter, verse)
                } else {
                    Sid::with_range(row_book, chapter, verse, verse.saturating_add(delta))
                };
                Some(sid.to_string())
            }
        };

        let marker = match row.marker {
            MarkerRef::AnchoredToken => token.and_then(|t| t.marker_name()).map(str::to_owned),
            MarkerRef::CatalogOrdinal(ordinal) => Some(
                catalog_marker_name(ordinal)
                    .ok_or(DecodeError::InvalidSection)?
                    .to_owned(),
            ),
            MarkerRef::SourceSpan { offset, len } => {
                let start = usize::try_from(offset).map_err(|_| DecodeError::OffsetOverflow)?;
                let end = start
                    .checked_add(usize::from(len))
                    .ok_or(DecodeError::OffsetOverflow)?;
                Some(
                    source
                        .get(start..end)
                        .ok_or(DecodeError::InvalidUtf8)?
                        .to_owned(),
                )
            }
            MarkerRef::Absent => None,
        };

        let message_params: MessageParams = match &row.params {
            Some(pairs) => pairs
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
            None => MessageParams::default(),
        };

        issues.push(LintIssue {
            code,
            category: code.category(),
            severity: code.severity(),
            issue_type: code.issue_type(),
            template: code.template(),
            message: code.render_message(&message_params),
            message_params,
            span,
            related_span,
            token_id,
            related_token_id,
            sid,
            marker,
            // Patch/fix framing is not encoded at all — a section carrying
            // either the patch-id or patch-table field is rejected outright
            // by the structural decoder before rows are even read — so every
            // decoded issue's fix is unconditionally absent.
            fix: None,
            // The row's own token index IS the position — more direct than
            // core's own id-string resolution, since wire stores it as a
            // number already.
            position: row.token_idx.unwrap_or(NO_TOKEN_POSITION),
            related_position: row.related.map_or(NO_TOKEN_POSITION, |(idx, ..)| idx),
        });
    }
    Ok(issues)
}

/// Resolves a stored `(offset, len)` pair against the token span it is
/// relative to. `len == 0` is the "whole token" sentinel (the token's own
/// span, verbatim); otherwise the pair names a byte range that must fit
/// entirely inside the token's own span — an offset or length wide enough to
/// escape it names bytes this finding was never bound to, so it is rejected
/// rather than silently read.
fn resolve_span(token_span: Span, offset: u32, len: u32) -> Result<Span, DecodeError> {
    if len == 0 {
        return Ok(token_span);
    }
    let token_len = token_span
        .end
        .checked_sub(token_span.start)
        .ok_or(DecodeError::InvalidSection)?;
    let end_offset = offset.checked_add(len).ok_or(DecodeError::OffsetOverflow)?;
    if end_offset > token_len {
        return Err(DecodeError::InvalidSection);
    }
    let start = token_span
        .start
        .checked_add(offset)
        .ok_or(DecodeError::OffsetOverflow)?;
    let end = token_span
        .start
        .checked_add(end_offset)
        .ok_or(DecodeError::OffsetOverflow)?;
    Ok(Span::new(start, end))
}

/// Whether `decoded` is what a semantic round trip of `original` must
/// produce: every `LintIssue` field equal except `fix`, which is never
/// encoded (`decoded.fix` must be `None` regardless of what `original`
/// carried).
pub fn issue_round_trips(original: &LintIssue, decoded: &LintIssue) -> bool {
    decoded.fix.is_none()
        && original.code == decoded.code
        && original.category == decoded.category
        && original.severity == decoded.severity
        && original.issue_type == decoded.issue_type
        && original.template == decoded.template
        && original.message == decoded.message
        && original.message_params == decoded.message_params
        && original.span == decoded.span
        && original.related_span == decoded.related_span
        && original.token_id == decoded.token_id
        && original.related_token_id == decoded.related_token_id
        && original.sid == decoded.sid
        && original.marker == decoded.marker
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding_goldens::{restamp_container, restamp_section};
    use usfm_onion::LintableToken;
    use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
    use usfm_onion::parse::parse;
    use usfm_onion::token::{TokenData, TokenId};

    fn book(code: &str) -> BookId {
        BookId::from_str(code).unwrap()
    }

    #[test]
    fn round_trips_a_small_book_with_several_finding_codes() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n\\v 1 duplicate\n";
        let parsed = parse(source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        assert!(!result.issues.is_empty());

        let bytes = encode_book(book("GEN"), source, &parsed.tokens, &result.issues).unwrap();
        let decoded = decode_book(&bytes, source).unwrap();

        let mut original = result.issues.clone();
        canonical_order_for_tokens(&mut original, &parsed.tokens);
        assert_eq!(original.len(), decoded.len());
        for (original, decoded) in original.iter().zip(&decoded) {
            assert!(
                issue_round_trips(original, decoded),
                "mismatch: {original:?} vs {decoded:?}"
            );
        }
    }

    #[test]
    fn encoding_is_deterministic() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
        let parsed = parse(source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        let first = encode_book(book("GEN"), source, &parsed.tokens, &result.issues).unwrap();
        let second = encode_book(book("GEN"), source, &parsed.tokens, &result.issues).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn zero_issues_round_trip_to_an_empty_list() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
        let parsed = parse(source);
        let bytes = encode_book(book("GEN"), source, &parsed.tokens, &[]).unwrap();
        let decoded = decode_book(&bytes, source).unwrap();
        assert!(decoded.is_empty());
    }

    /// Builds a marker/end-marker-free `Token` backed by an exact slice of
    /// `source`, mirroring what `token_codec::decode_token_section` builds for
    /// non-marker-bearing kinds.
    fn plain_token<'a>(source: &'a str, id: u32, span: Span, data: TokenData<'a>) -> Token<'a> {
        Token {
            id: TokenId::new("GEN", id),
            sid: None,
            span,
            source: &source[span.start as usize..span.end as usize],
            data,
        }
    }

    fn marker_token<'a>(source: &'a str, id: u32, span: Span, name: &'a str) -> Token<'a> {
        let metadata = usfm_onion::token::marker_metadata(name);
        let structural = usfm_onion::marker_defs::structural_marker_info(name, metadata.kind);
        Token {
            id: TokenId::new("GEN", id),
            sid: None,
            span,
            source: &source[span.start as usize..span.end as usize],
            data: TokenData::Marker {
                name,
                metadata,
                structural,
                nested: false,
                attrs: None,
            },
        }
    }

    /// Hand-built fixtures for codes reachable only from
    /// `lint_tokens(caller_tokens)`, never from `lint_usfm(source)` — no
    /// `.usfm` fixture can produce them, so the corpus gate cannot cover them.
    /// A lexer never emits these token shapes, but a caller-supplied token
    /// stream can.
    #[test]
    fn hand_built_unknown_token_round_trips() {
        // A single `Text` token whose source is a backslash, a known marker
        // name, and a non-`[a-z0-9-]` character jammed together — the lexer
        // always splits this into Marker+Text, so only a caller-built stream
        // reaches `lint_unknown_token_like`.
        let source = "\\id GEN\n\\pWord\n";
        let tokens = vec![
            marker_token(source, 0, Span::new(0, 4), "id"),
            plain_token(
                source,
                1,
                Span::new(4, 7),
                TokenData::BookCode {
                    code: "GEN",
                    is_valid: true,
                },
            ),
            plain_token(source, 2, Span::new(7, 8), TokenData::Newline),
            plain_token(source, 3, Span::new(8, 14), TokenData::Text),
            plain_token(source, 4, Span::new(14, 15), TokenData::Newline),
        ];
        let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        let issue = result
            .issues
            .iter()
            .find(|issue| issue.code == LintCode::UnknownToken)
            .expect("jammed marker+text fires unknown-token");
        assert_eq!(
            issue.message_params.get("text").map(String::as_str),
            Some("\\pWord")
        );

        let bytes = encode_book(book("GEN"), source, &tokens, std::slice::from_ref(issue)).unwrap();
        let decoded = decode_book(&bytes, source).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!(issue_round_trips(issue, &decoded[0]));
    }

    #[test]
    fn hand_built_invalid_number_range_round_trips() {
        // `invalid-number-range` requires a `Number` token whose text does not
        // parse as a range and which carries no `number_info` at all. That is
        // not just corpus-unreachable, it is unreachable from *any*
        // `Token<'a>`: `TokenData::Number` always carries a parsed
        // `(start, end, kind)` — there is no variant shape for "not actually
        // parsed" — so the wire token section could not even encode the
        // anchor a `Token<'a>`-based lint pass would need to fire this rule.
        // Reaching it at all requires a more permissive caller-token editor
        // shape than `Token<'a>`.
        //
        // What this codec owns is the *finding* payload shape, not core's
        // rule-firing logic, so this fixture builds the `LintIssue` directly
        // against a real anchor token from a real parse, proving the codec
        // round-trips this code's exact payload contract even though no
        // `Token<'a>` stream can make core produce one live.
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 4 body\n";
        let parsed = parse(source);
        let anchor = parsed
            .tokens
            .iter()
            .find(|token| token.kind() == usfm_onion::token::TokenKind::Number)
            .expect("a verse number token exists");
        let params = MessageParams::from([
            ("found".to_string(), "4-2x".to_string()),
            ("verse".to_string(), "4".to_string()),
            ("marker".to_string(), "v".to_string()),
            ("context".to_string(), "numbering".to_string()),
        ]);
        let issue = LintIssue {
            code: LintCode::InvalidNumberRange,
            category: LintCode::InvalidNumberRange.category(),
            severity: LintCode::InvalidNumberRange.severity(),
            issue_type: LintCode::InvalidNumberRange.issue_type(),
            template: LintCode::InvalidNumberRange.template(),
            message: LintCode::InvalidNumberRange.render_message(&params),
            message_params: params,
            span: Some(anchor.span),
            related_span: None,
            token_id: anchor.id(),
            related_token_id: None,
            sid: anchor.sid.map(|sid| sid.to_string()),
            marker: Some("v".to_string()),
            fix: None,
            position: NO_TOKEN_POSITION,
            related_position: NO_TOKEN_POSITION,
        };

        let bytes = encode_book(
            book("GEN"),
            source,
            &parsed.tokens,
            std::slice::from_ref(&issue),
        )
        .unwrap();
        let decoded = decode_book(&bytes, source).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!(issue_round_trips(&issue, &decoded[0]));
    }

    #[test]
    fn hand_built_number_range_not_preceded_by_marker_round_trips() {
        // A `Number` token whose previous significant token is not one of
        // \c/\ca/\cp/\v/\va/\vp — the lexer only mints `Number` after those.
        let source = "\\id GEN\n\\p\n1-3 body\n";
        let tokens = vec![
            marker_token(source, 0, Span::new(0, 4), "id"),
            plain_token(
                source,
                1,
                Span::new(4, 7),
                TokenData::BookCode {
                    code: "GEN",
                    is_valid: true,
                },
            ),
            plain_token(source, 2, Span::new(7, 8), TokenData::Newline),
            marker_token(source, 3, Span::new(8, 10), "p"),
            plain_token(source, 4, Span::new(10, 11), TokenData::Newline),
            plain_token(
                source,
                5,
                Span::new(11, 14),
                TokenData::Number {
                    start: 1,
                    end: Some(3),
                    kind: usfm_onion::token::NumberRangeKind::Range,
                },
            ),
            plain_token(source, 6, Span::new(14, 19), TokenData::Text),
            plain_token(source, 7, Span::new(19, 20), TokenData::Newline),
        ];
        let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        let issue = result
            .issues
            .iter()
            .find(|issue| issue.code == LintCode::NumberRangeNotPrecededByMarkerExpectingNumber)
            .unwrap_or_else(|| {
                panic!(
                    "expected number-range-not-preceded-by-marker-expecting-number; got {:?}",
                    result.issues.iter().map(|i| i.code).collect::<Vec<_>>()
                )
            });

        let bytes = encode_book(book("GEN"), source, &tokens, std::slice::from_ref(issue)).unwrap();
        let decoded = decode_book(&bytes, source).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!(issue_round_trips(issue, &decoded[0]));
    }

    /// Corruption battery, part 1: every prefix of a real encoded book must
    /// either fail to decode or decode to something — never panic.
    #[test]
    fn truncating_the_wire_never_panics() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n\\v 1 duplicate\n";
        let parsed = parse(source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        let bytes = encode_book(book("GEN"), source, &parsed.tokens, &result.issues).unwrap();
        for len in 0..bytes.len() {
            let _ = decode_book(&bytes[..len], source);
        }
    }

    /// Corruption battery, part 2: flipping any single byte to any of a few
    /// representative values must never panic, only ever return a typed error
    /// or (rarely, for a mutation the checksum cannot see) a still-valid
    /// decode.
    #[test]
    fn corrupting_any_wire_byte_never_panics() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n\\v 1 duplicate\n";
        let parsed = parse(source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        let base = encode_book(book("GEN"), source, &parsed.tokens, &result.issues).unwrap();
        for index in 0..base.len() {
            for value in [0x00u8, 0x01, 0x7f, 0xff] {
                let mut bytes = base.clone();
                bytes[index] = value;
                let _ = decode_book(&bytes, source);
            }
        }
    }

    #[test]
    fn derived_fields_that_disagree_with_the_catalog_refuse_to_encode() {
        let source = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 a\n";
        let parsed = parse(source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        let mut issue = result
            .issues
            .iter()
            .find(|issue| issue.code == LintCode::BookCodeNotUppercase)
            .expect("lower-case book code is flagged")
            .clone();

        let mut wrong_message = issue.clone();
        wrong_message.message = "not what the catalog would render".to_string();
        assert!(matches!(
            encode_book(
                book("GEN"),
                source,
                &parsed.tokens,
                std::slice::from_ref(&wrong_message)
            ),
            Err(EncodeError::UnrepresentablePayload { .. })
        ));

        issue.severity = usfm_onion::lint::LintSeverity::Error;
        assert_ne!(issue.severity, LintCode::BookCodeNotUppercase.severity());
        assert!(matches!(
            encode_book(
                book("GEN"),
                source,
                &parsed.tokens,
                std::slice::from_ref(&issue)
            ),
            Err(EncodeError::UnrepresentablePayload { .. })
        ));
    }

    /// A container naming two books, each with its own valid token+finding
    /// pair, is a perfectly legal *corpus* container — nothing in the
    /// container format itself is single-book. `decode_book` names one book,
    /// so it must refuse such a container rather than pick a token section
    /// from one book and a finding section from another. Building the
    /// sections in "crossed" order — token A, finding B, token B, finding A —
    /// reproduces exactly the scenario a last-one-wins TOC walk would get
    /// wrong (pairing token B with finding A), while still passing every
    /// container-level structural check (no duplicate (kind, book) pair, and
    /// every finding has a same-book, same-hash token to pair with).
    #[test]
    fn decode_book_rejects_a_two_book_container() {
        let source_a = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 a\n";
        let source_b = "\\id exo Exodus\n\\c 1\n\\p\n\\v 1 b\n";
        let parsed_a = parse(source_a);
        let parsed_b = parse(source_b);
        let issues_a = lint_tokens(&parsed_a.tokens, LintOptions::scoped(LintScope::Book)).issues;
        let issues_b = lint_tokens(&parsed_b.tokens, LintOptions::scoped(LintScope::Book)).issues;
        let token_a = encode_token_section(book("GEN"), source_a, &parsed_a.tokens).unwrap();
        let token_b = encode_token_section(book("EXO"), source_b, &parsed_b.tokens).unwrap();
        let finding_a =
            encode_findings(book("GEN"), source_a, &parsed_a.tokens, &issues_a).unwrap();
        let finding_b =
            encode_findings(book("EXO"), source_b, &parsed_b.tokens, &issues_b).unwrap();
        let bytes = write_container(
            0,
            &[
                token_a.payload(),
                finding_b.payload(),
                token_b.payload(),
                finding_a.payload(),
            ],
        )
        .unwrap();
        assert_eq!(decode_book(&bytes, source_a), Err(DecodeError::InvalidToc));
        assert_eq!(decode_book(&bytes, source_b), Err(DecodeError::InvalidToc));
    }

    /// Locates the one finding section's absolute byte offset and its
    /// common-row field's absolute byte offset in a two-section container
    /// `encode_book` produced.
    fn locate_finding_section(bytes: &[u8]) -> (usize, usize) {
        let finding_offset = crate::finding_goldens::section_bounds(bytes)[1].0;
        let common_row_offset = {
            let container = read_container(bytes).unwrap();
            let section = container.section(1).unwrap().unwrap();
            let field = section
                .field(crate::schema::finding_field::COMMON_ROW)
                .unwrap();
            field.bytes.as_ptr() as usize - bytes.as_ptr() as usize
        };
        (finding_offset, common_row_offset)
    }

    /// Semantic (not merely bit-flip) mutations, each followed by recomputing
    /// both integrity checksums, so what is actually exercised is the trust
    /// checks *downstream* of the checksum rather than the checksum itself.
    /// Every case must yield its named typed error — never a panic, never a
    /// silently different `LintIssue`.
    #[test]
    fn restamped_hostile_mutations_yield_typed_errors() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 4 body\n";
        let parsed = parse(source);
        let anchor = parsed
            .tokens
            .iter()
            .find(|token| token.kind() == usfm_onion::token::TokenKind::Number)
            .unwrap();
        let params = MessageParams::from([
            ("found".to_string(), "4-2x".to_string()),
            ("verse".to_string(), "4".to_string()),
            ("marker".to_string(), "v".to_string()),
            ("context".to_string(), "numbering".to_string()),
        ]);
        let issue = LintIssue {
            code: LintCode::InvalidNumberRange,
            category: LintCode::InvalidNumberRange.category(),
            severity: LintCode::InvalidNumberRange.severity(),
            issue_type: LintCode::InvalidNumberRange.issue_type(),
            template: LintCode::InvalidNumberRange.template(),
            message: LintCode::InvalidNumberRange.render_message(&params),
            message_params: params,
            span: Some(anchor.span),
            related_span: None,
            token_id: anchor.id(),
            related_token_id: None,
            sid: anchor.sid.map(|sid| sid.to_string()),
            marker: Some("v".to_string()),
            fix: None,
            position: NO_TOKEN_POSITION,
            related_position: NO_TOKEN_POSITION,
        };
        let base = encode_book(
            book("GEN"),
            source,
            &parsed.tokens,
            std::slice::from_ref(&issue),
        )
        .unwrap();

        // (a) primary span offset/len escape the anchor token's own span.
        {
            let mut bytes = base.clone();
            let (finding_offset, row_offset) = locate_finding_section(&bytes);
            bytes[row_offset + 6..row_offset + 8].copy_from_slice(&60_000u16.to_le_bytes());
            restamp_section(&mut bytes, finding_offset);
            restamp_container(&mut bytes);
            assert_eq!(
                decode_book(&bytes, source),
                Err(DecodeError::InvalidSection)
            );
        }

        // (b) stale source_len: the finding section's own recorded source
        // length disagrees with the real source it is decoded against. This
        // field has no container-level cross-check (unlike source_hash, which
        // the container's token/finding pairing rule already forces to agree
        // with the token section's own — itself already checked against the
        // real source before the finding section is ever reached), so this
        // is the one of the three finding-section trust fields a bad
        // producer could get wrong independently of everything else already
        // validated.
        {
            let mut bytes = base.clone();
            let (finding_offset, _) = locate_finding_section(&bytes);
            bytes[finding_offset + 48..finding_offset + 56]
                .copy_from_slice(&(source.len() as u64 + 1).to_le_bytes());
            restamp_section(&mut bytes, finding_offset);
            restamp_container(&mut bytes);
            assert_eq!(
                decode_book(&bytes, source),
                Err(DecodeError::SourceLengthMismatch)
            );
        }

        // (c) stale catalog_stamp: the finding section claims a registry
        // version other than the one this build actually has.
        {
            let mut bytes = base.clone();
            let (finding_offset, _) = locate_finding_section(&bytes);
            bytes[finding_offset + 56..finding_offset + 64]
                .copy_from_slice(&0xdead_beef_dead_beefu64.to_le_bytes());
            restamp_section(&mut bytes, finding_offset);
            restamp_container(&mut bytes);
            assert_eq!(
                decode_book(&bytes, source),
                Err(DecodeError::CatalogMismatch)
            );
        }

        // (d) contradictory flags: force ANCHOR_ONLY on top of a row whose
        // NO_ANCHOR bit is already set, starting from a real no-SID finding
        // (missing-id-marker) so every *other* no-anchor invariant already
        // holds and only the new contradiction is under test.
        {
            let no_sid_issue = LintIssue {
                code: LintCode::MissingIdMarker,
                category: LintCode::MissingIdMarker.category(),
                severity: LintCode::MissingIdMarker.severity(),
                issue_type: LintCode::MissingIdMarker.issue_type(),
                template: LintCode::MissingIdMarker.template(),
                message: LintCode::MissingIdMarker.render_message(&MessageParams::default()),
                message_params: MessageParams::default(),
                span: None,
                related_span: None,
                token_id: None,
                related_token_id: None,
                sid: None,
                marker: Some("id".to_string()),
                fix: None,
                position: NO_TOKEN_POSITION,
                related_position: NO_TOKEN_POSITION,
            };
            let mut bytes = encode_book(
                book("GEN"),
                source,
                &parsed.tokens,
                std::slice::from_ref(&no_sid_issue),
            )
            .unwrap();
            let (finding_offset, row_offset) = locate_finding_section(&bytes);
            bytes[row_offset + 14] |= 0x01; // ANCHOR_ONLY, on top of the existing NO_ANCHOR bit
            restamp_section(&mut bytes, finding_offset);
            restamp_container(&mut bytes);
            assert_eq!(
                decode_book(&bytes, source),
                Err(DecodeError::InvalidSection)
            );
        }
    }
}
