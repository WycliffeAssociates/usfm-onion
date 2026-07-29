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
//! Governing documents: `plans/approved/braid/phase0-freeze.md` §F (finding
//! framing), §G (related-span width, materialize boundary), and
//! `plans/approved/braid/gate0-0d-payload-ledger.md` §2 (the per-code payload
//! census this codec must round-trip).
//!
//! Scope, per the frozen §F.3 deferral: `LintIssue.fix` is never encoded.
//! Common-row flag bit 5 (`fix`) always stays clear, and decode always
//! produces `fix: None` — codes 24/25/26 lose their `TokenFix` on a wire round
//! trip, every other field round-trips.
//!
//! `message` is rebuilt **only** via [`LintCode::render_message`] — never a
//! wire-local template renderer — so the English text a decoder produces can
//! never drift from core's own rendering of the same code and params.

use std::collections::BTreeMap;

use usfm_onion::lint::{LintCode, LintIssue, MessageParams};
use usfm_onion::token::{BookId, Sid, Span, Token};

use crate::catalog::{catalog_marker_name, catalog_ordinal};
use crate::container::{read_container, write_container};
use crate::error::{DecodeError, EncodeError};
use crate::finding_section::{
    FindingColumns, FindingDecodeInputs, FindingRowInput, FindingSectionBuffers, MarkerRef,
};
use crate::schema::{LintCodeTag, SectionKind, param_contract};
use crate::token_codec::{DecodedTokens, anchor_fidelity, decode_token_section, encode_token_section};

/// Encodes one book's parsed tokens and lint issues into a single container
/// with a token section and a paired finding section.
///
/// `issues` need not already be in canonical order: this function sorts a
/// local copy before building rows, so the container's finding order is
/// always the §2.2#15 canonical order regardless of the caller's order.
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
pub fn decode_book<'a>(bytes: &'a [u8], source: &'a str) -> Result<Vec<LintIssue>, DecodeError> {
    let container = read_container(bytes)?;
    let toc = container.toc();
    let mut token_index = None;
    let mut finding_index = None;
    let mut book = None;
    for (index, entry) in toc.iter().enumerate() {
        match entry.kind {
            SectionKind::Token => {
                token_index = Some(index);
                book = Some(entry.book);
            }
            SectionKind::Finding => finding_index = Some(index),
        }
    }
    let token_section = container
        .section(token_index.ok_or(DecodeError::InvalidToc)?)
        .ok_or(DecodeError::InvalidToc)??;
    let finding_section = container
        .section(finding_index.ok_or(DecodeError::InvalidToc)?)
        .ok_or(DecodeError::InvalidToc)??;
    let book = book.ok_or(DecodeError::InvalidToc)?;

    let decoded_tokens = decode_token_section(&token_section, source)?;
    let token_ids = resolve_token_ids(&decoded_tokens);
    let inputs = FindingDecodeInputs {
        token_count: u32::try_from(decoded_tokens.tokens.len())
            .map_err(|_| DecodeError::OffsetOverflow)?,
    };
    let columns = FindingColumns::from_section(&finding_section, inputs)?;
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

/// The exact key core's own (private) `canonical_sort` uses (`lint_impl.rs`):
/// span first (spanless last), then the rule's kebab string (not its
/// discriminant), then related span, token id, marker, and finally message.
fn span_key(span: Option<Span>) -> (u8, u32, u32) {
    match span {
        Some(span) => (0, span.start, span.end),
        None => (1, u32::MAX, u32::MAX),
    }
}

/// Sorts `issues` into the §2.2#15 canonical finding order — the same key
/// core's own (private) `canonical_sort` uses. [`encode_book`]/[`encode_findings`]
/// always apply this before building rows, so a caller comparing an
/// unsorted `LintResult` against a decoded one needs it too.
pub fn canonical_order(issues: &mut [LintIssue]) {
    issues.sort_by(|a, b| {
        span_key(a.span)
            .cmp(&span_key(b.span))
            .then_with(|| a.code.code().cmp(b.code.code()))
            .then_with(|| span_key(a.related_span).cmp(&span_key(b.related_span)))
            .then_with(|| a.token_id.cmp(&b.token_id))
            .then_with(|| a.marker.cmp(&b.marker))
            .then_with(|| a.message.cmp(&b.message))
    });
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

    let mut sorted: Vec<LintIssue> = issues.to_vec();
    canonical_order(&mut sorted);

    // A finding's `token_id`/`related_token_id` are opaque strings (positional
    // or caller-supplied); the only general way back to a row index is a
    // reverse lookup built from each token's own `id()`.
    let mut ids: Vec<String> = Vec::with_capacity(tokens.len());
    for token in tokens {
        ids.push(token.id().ok_or(EncodeError::UnboundSpan {
            book,
            token_idx: 0,
        })?);
    }
    let mut resolver: BTreeMap<&str, u32> = BTreeMap::new();
    for (row, id) in ids.iter().enumerate() {
        resolver.insert(id.as_str(), row as u32);
    }

    let fidelity = anchor_fidelity(tokens);
    let mut rows = Vec::with_capacity(sorted.len());
    for issue in &sorted {
        rows.push(issue_to_row(book, source, tokens, &resolver, &fidelity, issue)?);
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
/// row's `(offset, len)` pair. `len == 0` is the frozen "whole token" sentinel
/// (Gate 0D §2.3): every real corpus finding uses it today.
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

    if issue.token_id.is_some() != issue.span.is_some() {
        return Err(err());
    }
    let token_row = match &issue.token_id {
        Some(id) => Some(*resolver.get(id.as_str()).ok_or_else(err)?),
        None => None,
    };
    let (offset, len) = match (issue.span, token_row) {
        (None, None) => (0u32, 0u32),
        (Some(span), Some(row)) => {
            span_within_token(book, code, tokens[row as usize].span, span)?
        }
        _ => return Err(err()),
    };

    // `sid.is_some()` is independent of whether the finding is token-anchored:
    // a token-anchored finding can still have no SID at all (e.g.
    // `content-before-first-chapter`, which fires on real content tokens
    // before any `\c` establishes an anchor) — that is exactly what the
    // `no-anchor` flag exists to distinguish from a legitimate `(0, 0)`
    // front-matter SID (freeze §3.3). What *is* required is a token to read
    // the raw `Sid` from whenever one is present, since `LintIssue.sid` is
    // only a formatted display string.
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
            let source_anchor_only =
                fidelity.get(&sid).copied() == Some(crate::token_section::SidFidelity::AnchorOnly);
            if delta > u16::from(u8::MAX) {
                // Degrade exactly like the token codec's packed SID dictionary:
                // a bridge wider than one byte cannot be stored, so the range
                // is dropped and the row is marked anchor-only. Unreached by
                // any real corpus finding (Gate 0D: max chapter length is far
                // below 255 verses).
                (Some(sid.chapter), Some(sid.verse), None, true)
            } else {
                (
                    Some(sid.chapter),
                    Some(sid.verse),
                    (delta != 0).then_some(delta as u8),
                    source_anchor_only,
                )
            }
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
        let span = token.map(|t| resolve_span(t.span, row.offset, row.len));

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
                let related_token = tokens.get(idx as usize).ok_or(DecodeError::InvalidSection)?;
                Ok(resolve_span(related_token.span, offset, len))
            })
            .transpose()?;

        let sid = match row.chapter {
            None => None,
            Some(chapter) => {
                let verse = row.verse.ok_or(DecodeError::InvalidSection)?;
                let delta = u16::from(row.range_end.unwrap_or(0));
                // The row stores chapter/verse/range only; a finding's SID
                // book is whatever its anchor token's own SID carries (which
                // need not be the section's canonical book — e.g. an
                // unmodified lowercase `\id` book code). The section's `book`
                // is only a fallback for an anchor-only finding that somehow
                // still carries an SID, a combination no current rule
                // produces.
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
            // §F.3: patch framing is deferred; a v1 section never carries one
            // (`FindingColumns::from_section` rejects fields 5/6 outright), so
            // every decoded issue's fix is unconditionally absent.
            fix: None,
        });
    }
    Ok(issues)
}

fn resolve_span(token_span: Span, offset: u32, len: u32) -> Span {
    if len == 0 {
        token_span
    } else {
        Span::new(token_span.start + offset, token_span.start + offset + len)
    }
}

/// Whether `decoded` is what a semantic round trip of `original` must produce:
/// every `LintIssue` field equal except `fix`, which the §F.3 deferral always
/// drops (`decoded.fix` must be `None` regardless of what `original` carried).
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
        canonical_order(&mut original);
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

    /// Hand-built fixtures for the three codes Gate 0D found reachable only
    /// from `lint_tokens(caller_tokens)`, never from `lint_usfm(source)` — no
    /// `.usfm` fixture can produce them, so the corpus gate cannot cover them.
    /// Token shapes mirror the Gate 0D probe (`gate0-0d-payload-ledger.md`
    /// §2.1): a lexer never emits these shapes, but a caller-supplied token
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
        assert_eq!(issue.message_params.get("text").map(String::as_str), Some("\\pWord"));

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
        // `(start, end, kind)` (Gate 0D §1.1 — "number tokens require
        // `number`" is unconditionally true for this type), so the wire token
        // section could not even encode the anchor a `Token<'a>`-based lint
        // pass would need to fire this rule. Gate 0D's own probe needed the
        // more permissive `FormatToken` (a caller-token editor shape, not
        // `Token<'a>`) to reach it.
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
        };

        let bytes =
            encode_book(book("GEN"), source, &parsed.tokens, std::slice::from_ref(&issue))
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
}
