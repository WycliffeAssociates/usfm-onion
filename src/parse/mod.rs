#[cfg(not(target_arch = "wasm32"))]
mod parallel;

use crate::lexer::lex;
use crate::marker_defs::{absorbs_delimiter_whitespace_for_index, structural_info_for_index};
use crate::token::{
    AttributeItem, BookId, Lexeme, MarkerAttrs, NumberRangeToken, ParseAnalysis, ParseResult,
    ScanToken, Sid, Span, Token, TokenData, TokenId, TokenKind, tokens_to_usfm,
};

pub fn parse(source: &str) -> ParseResult<'_> {
    // Large books parse chapter-parallel; the result is byte-identical to the
    // serial path (proven by the corpus test in `parallel`), so callers get the
    // speedup transparently with no API change. Small books and wasm stay serial —
    // below the threshold the partition's fixed cost outweighs the gain, and wasm
    // has no thread pool to recover it.
    #[cfg(not(target_arch = "wasm32"))]
    if source.len() >= parallel::PARALLEL_MIN_BYTES {
        return parallel::parse_partitioned(source);
    }
    let lexed = lex(source);
    parse_lexemes(source, &lexed.tokens)
}

pub fn parse_lexemes<'a>(source: &'a str, lexemes: &[Lexeme<'a>]) -> ParseResult<'a> {
    parse_lexemes_seeded(source, lexemes, None)
}

/// Like [`parse_lexemes`], but primes the initial book so a chapter slice lexed
/// in isolation (with no preceding `\id`) still resolves `(book, chapter, verse)`
/// sids. The seed touches only book state; it never sets `analysis.book_code`,
/// which stays owned by the actual `\id`/book-code tokens present in the slice.
pub(crate) fn parse_lexemes_seeded<'a>(
    source: &'a str,
    lexemes: &[Lexeme<'a>],
    seed_book: Option<&'a str>,
) -> ParseResult<'a> {
    let mut analysis = ParseAnalysis::default();
    let mut state = ParseState::default();
    if let Some(code) = seed_book {
        state.current_book_code = Some(code);
        state.current_book = BookId::from_str(code);
        state.current_sid = state.current_book.map(|book| Sid::new(book, 0, 0));
    }
    // Presize from the lexeme count: parsing emits at most one token per lexeme
    // (adjacent text runs merge in `push_token`, so usually fewer; the empty-doc
    // `flush_pending_whitespace` edge adds at most one), so `lexemes.len()` is a
    // tight, never-under upper bound. This eliminates the realloc `memmove`
    // ladder from growing a zero-capacity Vec — the parallel path presizes each
    // chapter chunk the same way from its own lexeme slice.
    let mut tokens = Vec::with_capacity(lexemes.len());
    let mut cursor = 0usize;

    while cursor < lexemes.len() {
        if matches!(lexemes[cursor], ScanToken::Pipe(_)) {
            flush_pending_whitespace(source, &mut state, &mut tokens);
            let (entries, attr_span, attr_source, next_cursor) =
                consume_attribute_list(source, lexemes, cursor);
            if entries.is_empty() {
                // Malformed input (e.g. a lone `|` with no key="value" after it).
                // Preserve the bytes as a Text token so round-trip stays lossless;
                // do not pollute the preceding marker's `attribute_source`.
                let token = Token {
                    id: TokenId::new("", 0),
                    sid: state.current_sid,
                    span: attr_span,
                    source: attr_source,
                    data: TokenData::Text,
                };
                push_token(source, &mut tokens, token);
            } else {
                attach_attributes_to_preceding_marker(&mut tokens, entries, attr_span, attr_source);
            }
            cursor = next_cursor;
            continue;
        }

        match lexemes[cursor] {
            ScanToken::Whitespace(ws) => {
                park_ws(source, &mut state, &mut tokens, ws.span);
                cursor += 1;
            }
            ScanToken::Newline(token) => {
                flush_pending_whitespace(source, &mut state, &mut tokens);
                let token = Token {
                    id: TokenId::new("", 0),
                    sid: state.current_sid,
                    span: token.span,
                    source: token.lexeme,
                    data: TokenData::Newline,
                };
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::OptBreak(token) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    token.span,
                    TokenData::OptBreak,
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::Marker(marker) => {
                if let Some((next_cursor, book_code)) =
                    try_consume_book_code(lexemes, cursor, &mut state)
                {
                    analysis.book_code = Some(book_code.lexeme);
                    let marker_token = token_with_current_ws_and_sid(
                        source,
                        &mut state,
                        marker.span,
                        TokenData::Marker {
                            name: marker.name,
                            metadata: marker.metadata,
                            structural: structural_info_for_index(marker.metadata.index),
                            nested: false,
                            attrs: None,
                        },
                    );
                    push_token(source, &mut tokens, marker_token);

                    if let Some(ws) = book_code.leading_ws {
                        park_ws(source, &mut state, &mut tokens, ws);
                    }
                    state.current_book_code = Some(book_code.lexeme);
                    state.current_book = BookId::from_str(book_code.lexeme);
                    state.current_sid = state.current_book.map(|book| Sid::new(book, 0, 0));
                    let book_token = token_with_current_ws_and_sid(
                        source,
                        &mut state,
                        book_code.span,
                        TokenData::BookCode {
                            code: book_code.lexeme,
                            is_valid: book_code.is_valid,
                        },
                    );
                    push_token(source, &mut tokens, book_token);
                    cursor = next_cursor;
                    continue;
                }

                if let Some((next_cursor, number)) =
                    try_consume_number(lexemes, cursor, marker.name, &mut state)
                {
                    let next_sid = advanced_sid(&state, marker.name, &number.number);
                    let marker_token = token_with_current_ws(
                        source,
                        &mut state,
                        marker.span,
                        TokenData::Marker {
                            name: marker.name,
                            metadata: marker.metadata,
                            structural: structural_info_for_index(marker.metadata.index),
                            nested: false,
                            attrs: None,
                        },
                        next_sid,
                    );
                    push_token(source, &mut tokens, marker_token);
                    if let Some(ws) = number.leading_ws {
                        park_ws(source, &mut state, &mut tokens, ws);
                    }
                    update_sid_state(&mut state, marker.name, &number.number);
                    state.cv_number = CvNumber::ExpectingNumber;
                    let number_token = token_with_current_ws_and_sid(
                        source,
                        &mut state,
                        number.number.span,
                        TokenData::Number {
                            start: number.number.start,
                            end: number.number.end,
                            kind: number.number.kind,
                        },
                    );
                    push_token(source, &mut tokens, number_token);
                    cursor = next_cursor;
                    continue;
                }

                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    marker.span,
                    TokenData::Marker {
                        name: marker.name,
                        metadata: marker.metadata,
                        structural: structural_info_for_index(marker.metadata.index),
                        nested: false,
                        attrs: None,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::NestedMarker(marker) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    marker.span,
                    TokenData::Marker {
                        name: marker.name,
                        metadata: marker.metadata,
                        structural: structural_info_for_index(marker.metadata.index),
                        nested: true,
                        attrs: None,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::ClosingMarker(marker) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    marker.span,
                    TokenData::EndMarker {
                        name: marker.name,
                        metadata: marker.metadata,
                        structural: structural_info_for_index(marker.metadata.index),
                        nested: false,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::NestedClosingMarker(marker) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    marker.span,
                    TokenData::EndMarker {
                        name: marker.name,
                        metadata: marker.metadata,
                        structural: structural_info_for_index(marker.metadata.index),
                        nested: true,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::Milestone(marker) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    marker.span,
                    TokenData::Milestone {
                        name: marker.name,
                        metadata: marker.metadata,
                        structural: structural_info_for_index(marker.metadata.index),
                        attrs: None,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::MilestoneEnd(token) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    token.span,
                    TokenData::MilestoneEnd,
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::BookCode(book) => {
                if analysis.book_code.is_none() {
                    analysis.book_code = Some(book.lexeme);
                }
                state.current_book_code = Some(book.lexeme);
                state.current_book = BookId::from_str(book.lexeme);
                state.current_sid = state.current_book.map(|b| Sid::new(b, 0, 0));
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    book.span,
                    TokenData::BookCode {
                        code: book.lexeme,
                        is_valid: book.is_valid,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::NumberRange(number) => {
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    number.span,
                    TokenData::Number {
                        start: number.start,
                        end: number.end,
                        kind: number.kind,
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::Text(text) => {
                let span =
                    if let Some(ws) = leading_horizontal_whitespace_span(text.span, text.lexeme) {
                        park_ws(source, &mut state, &mut tokens, ws);
                        if ws.end == text.span.end {
                            cursor += 1;
                            continue;
                        }
                        Span::new(ws.end, text.span.end)
                    } else {
                        text.span
                    };
                let token =
                    token_with_current_ws_and_sid(source, &mut state, span, TokenData::Text);
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::AttributeEntry(_) | ScanToken::Pipe(_) => unreachable!("handled above"),
        }
    }

    flush_pending_whitespace(source, &mut state, &mut tokens);
    assign_ids(&mut tokens);

    ParseResult { tokens, analysis }
}

#[derive(Default)]
struct ParseState<'a> {
    current_book_code: Option<&'a str>,
    current_book: Option<BookId>,
    current_chapter: u32,
    current_sid: Option<Sid>,
    pending_ws: Option<Span>,
    cv_number: CvNumber,
}

/// One-shot tracker for the number argument of a chapter/verse marker.
///
/// `delimiter_absorption` needs to distinguish a just-emitted `Number` token
/// that is the argument of `\c`/`\v` (whose trailing space is a tag-end
/// delimiter it should own) from an ordinary number in content (which owns
/// nothing). Lifecycle: armed when the marker's number argument is
/// recognized, latched for exactly the next emitted token, then back to idle.
#[derive(Default, Clone, Copy, PartialEq)]
enum CvNumber {
    #[default]
    Idle,
    /// The next emitted token, if a `Number`, is a chapter/verse argument.
    ExpectingNumber,
    /// The token just emitted was a chapter/verse argument number.
    JustEmitted,
}

#[derive(Clone, Copy)]
struct PendingBookCode<'a> {
    leading_ws: Option<Span>,
    lexeme: &'a str,
    span: Span,
    is_valid: bool,
}

#[derive(Clone, Copy)]
struct PendingNumber<'a> {
    leading_ws: Option<Span>,
    number: NumberRangeToken<'a>,
}

fn try_consume_book_code<'a>(
    lexemes: &[Lexeme<'a>],
    marker_index: usize,
    _state: &mut ParseState<'a>,
) -> Option<(usize, PendingBookCode<'a>)> {
    let marker = match lexemes.get(marker_index)? {
        ScanToken::Marker(marker) => marker,
        _ => return None,
    };
    if marker.name != "id" {
        return None;
    }

    let next_index = next_significant_lexeme_index(lexemes, marker_index + 1)?;
    let book = match lexemes.get(next_index)? {
        ScanToken::BookCode(book) => *book,
        _ => return None,
    };
    Some((
        next_index + 1,
        PendingBookCode {
            leading_ws: pending_whitespace_between(lexemes, marker_index + 1, next_index),
            lexeme: book.lexeme,
            span: book.span,
            is_valid: book.is_valid,
        },
    ))
}

fn try_consume_number<'a>(
    lexemes: &[Lexeme<'a>],
    marker_index: usize,
    marker_name: &'a str,
    _state: &mut ParseState<'a>,
) -> Option<(usize, PendingNumber<'a>)> {
    if !matches!(marker_name, "c" | "v") {
        return None;
    }

    let next_index = next_significant_lexeme_index(lexemes, marker_index + 1)?;
    let number = match lexemes.get(next_index)? {
        ScanToken::NumberRange(number) => *number,
        _ => return None,
    };
    Some((
        next_index + 1,
        PendingNumber {
            leading_ws: pending_whitespace_between(lexemes, marker_index + 1, next_index),
            number,
        },
    ))
}

fn next_significant_lexeme_index(lexemes: &[Lexeme<'_>], start: usize) -> Option<usize> {
    for (index, lexeme) in lexemes.iter().enumerate().skip(start) {
        match lexeme {
            ScanToken::Whitespace(_) => continue,
            ScanToken::Newline(_) => return None,
            _ => return Some(index),
        }
    }
    None
}

fn pending_whitespace_between(lexemes: &[Lexeme<'_>], start: usize, end: usize) -> Option<Span> {
    let mut begin = None;
    let mut finish = None;
    for lexeme in lexemes.iter().take(end).skip(start) {
        if let ScanToken::Whitespace(ws) = lexeme {
            begin.get_or_insert(ws.span.start);
            finish = Some(ws.span.end);
        }
    }
    match (begin, finish) {
        (Some(start), Some(end)) => Some(Span::new(start, end)),
        _ => None,
    }
}

/// Consume a `|attr="v" ...` block from the lexeme stream and return the parsed
/// entries plus the verbatim source slice (`|...`, including any internal
/// whitespace) so the caller can attach both to the owning marker. No standalone
/// token is emitted into the stream — `tokens_to_usfm` reproduces the source by
/// emitting the slice at the correct position relative to the owning marker.
fn consume_attribute_list<'a>(
    source: &'a str,
    lexemes: &[Lexeme<'a>],
    start: usize,
) -> (Vec<AttributeItem<'a>>, Span, &'a str, usize) {
    let ScanToken::Pipe(pipe) = lexemes[start] else {
        unreachable!("attribute list must start with pipe");
    };

    let mut entries = Vec::new();
    let mut end = pipe.span.end;
    let mut cursor = start + 1;

    while cursor < lexemes.len() {
        match lexemes[cursor] {
            ScanToken::Whitespace(ws) => {
                end = ws.span.end;
                cursor += 1;
            }
            ScanToken::AttributeEntry(entry) => {
                entries.push(AttributeItem {
                    span: entry.span,
                    source: entry.lexeme,
                    key: entry.key,
                    value: entry.value,
                    is_default: entry.is_default,
                });
                end = entry.span.end;
                cursor += 1;
            }
            _ => break,
        }
    }

    let span = Span::new(pipe.span.start, end);
    let raw = &source[span.as_range()];
    (entries, span, raw, cursor)
}

/// Attach a parsed attribute list to the most recent Marker or Milestone token.
/// Stores both the semantic entries (for API/USJ/USX/HTML access) and the raw
/// source slice (for byte-identical USFM round-trip via `tokens_to_usfm`).
/// Orphan attribute lists (no preceding marker) are dropped — they are malformed
/// USFM and there is no semantically meaningful place to attach them.
fn attach_attributes_to_preceding_marker<'a>(
    tokens: &mut [Token<'a>],
    entries: Vec<AttributeItem<'a>>,
    span: Span,
    raw_source: &'a str,
) {
    let Some(target) = tokens.iter_mut().rev().find(|token| {
        matches!(
            token.data,
            TokenData::Marker { .. } | TokenData::Milestone { .. }
        )
    }) else {
        return;
    };
    match &mut target.data {
        TokenData::Marker { attrs, .. } | TokenData::Milestone { attrs, .. } => {
            match attrs {
                Some(existing) => existing.attributes.extend(entries),
                None => {
                    *attrs = Some(Box::new(MarkerAttrs {
                        attributes: entries,
                        attribute_source: None,
                    }))
                }
            }
            attrs.as_mut().unwrap().attribute_source = Some((span, raw_source));
        }
        _ => unreachable!("filtered above"),
    }
}

fn advanced_sid(
    state: &ParseState<'_>,
    marker_name: &str,
    number: &NumberRangeToken<'_>,
) -> Option<Sid> {
    let book = state.current_book?;
    match marker_name {
        "c" => Some(Sid::new(book, saturating_u16(number.start), 0)),
        "v" => {
            let verse = saturating_u16(number.start);
            let verse_end = number.end.map(saturating_u16).unwrap_or(verse);
            Some(Sid::with_range(
                book,
                saturating_u16(state.current_chapter),
                verse,
                verse_end,
            ))
        }
        _ => state.current_sid,
    }
}

fn saturating_u16(value: u32) -> u16 {
    value.min(u16::MAX as u32) as u16
}

fn update_sid_state(state: &mut ParseState<'_>, marker_name: &str, number: &NumberRangeToken<'_>) {
    match marker_name {
        "c" => {
            state.current_chapter = number.start;
            state.current_sid = advanced_sid(state, marker_name, number);
        }
        "v" => {
            state.current_sid = advanced_sid(state, marker_name, number);
        }
        _ => {}
    }
}

fn token_with_current_ws_and_sid<'a>(
    source: &'a str,
    state: &mut ParseState<'a>,
    span: Span,
    data: TokenData<'a>,
) -> Token<'a> {
    token_with_current_ws(source, state, span, data, state.current_sid)
}

fn token_with_current_ws<'a>(
    source: &'a str,
    state: &mut ParseState<'a>,
    span: Span,
    data: TokenData<'a>,
    sid: Option<Sid>,
) -> Token<'a> {
    let is_cv_argument_number =
        state.cv_number == CvNumber::ExpectingNumber && matches!(data, TokenData::Number { .. });
    state.cv_number = if is_cv_argument_number {
        CvNumber::JustEmitted
    } else {
        CvNumber::Idle
    };
    let start = state
        .pending_ws
        .map(|ws| ws.start.min(span.start))
        .unwrap_or(span.start);
    state.pending_ws = None;
    let span = Span::new(start, span.end);
    Token {
        id: TokenId::new("", 0),
        sid,
        span,
        source: &source[span.as_range()],
        data,
    }
}

/// Decides who owns whitespace that follows a token, then parks it.
///
/// In USFM, the first space/tab after a marker name (`\v •1`) or after a
/// chapter/verse number or `\id` book code (`\v 1•In`) is the tag-end
/// delimiter — markup required by the syntax, not content. We absorb that one
/// byte onto the owning token's span so following content tokens are
/// content-pure: text projections (`to_vref`, `vref_index`) would otherwise
/// leak the delimiter into verse text as a phantom leading space. Only the
/// single required byte is absorbed; any excess whitespace flows forward to
/// the next token as before, where lint can see and flag it.
/// Byte-losslessness holds either way — the byte changes owner, it is never
/// dropped.
///
/// The `+ 1` below is safe: whitespace spans are built exclusively from ASCII
/// space/tab (`consume_inline_whitespace`,
/// `leading_horizontal_whitespace_span`), so the first char is always one
/// byte.
fn park_ws<'a>(source: &'a str, state: &mut ParseState<'a>, tokens: &mut Vec<Token<'a>>, ws: Span) {
    let should_absorb = tokens
        .last()
        .is_some_and(|token| delimiter_absorption(token, state));

    if should_absorb {
        let absorbed_end = ws.start + 1;
        let last = tokens
            .last_mut()
            .expect("last token exists when absorption is requested");
        last.span = Span::new(last.span.start, absorbed_end);
        last.source = &source[last.span.as_range()];

        state.pending_ws = (absorbed_end < ws.end).then_some(Span::new(absorbed_end, ws.end));
    } else {
        state.pending_ws = Some(ws);
    }
}

/// Whether `token` owns the whitespace that follows it as its tag-end
/// delimiter.
///
/// Driven by the marker whitespace table: a marker absorbs only when the spec
/// *requires* trailing whitespace after its name. Closers, milestones,
/// optional-whitespace markers, and unknown markers never absorb — and
/// "the token happens to end at a space" is deliberately NOT the predicate;
/// absorption is a property of the marker's syntax, not of the bytes.
/// A `Number` absorbs only as the argument of `\c`/`\v` (which the delimiter
/// terminates), never as an in-content number.
fn delimiter_absorption(token: &Token<'_>, state: &ParseState<'_>) -> bool {
    match &token.data {
        TokenData::Marker { metadata, .. } => {
            absorbs_delimiter_whitespace_for_index(metadata.index)
        }
        TokenData::Number { .. } => state.cv_number == CvNumber::JustEmitted,
        TokenData::BookCode { .. } => true,
        _ => false,
    }
}

fn leading_horizontal_whitespace_span(span: Span, source: &str) -> Option<Span> {
    let len = source
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();
    (len > 0).then_some(Span::new(span.start, span.start + len as u32))
}

fn flush_pending_whitespace<'a>(
    source: &'a str,
    state: &mut ParseState<'a>,
    tokens: &mut Vec<Token<'a>>,
) {
    let Some(ws) = state.pending_ws.take() else {
        return;
    };
    if let Some(last) = tokens.last_mut() {
        last.span = Span::new(last.span.start, ws.end);
        last.source = &source[last.span.as_range()];
    } else {
        tokens.push(Token {
            id: TokenId::new("", 0),
            sid: state.current_sid,
            span: ws,
            source: &source[ws.as_range()],
            data: TokenData::Text,
        });
    }
}

fn push_token<'a>(source: &'a str, tokens: &mut Vec<Token<'a>>, token: Token<'a>) {
    if let Some(last) = tokens.last_mut()
        && matches!(last.kind(), TokenKind::Text)
        && matches!(token.kind(), TokenKind::Text)
        && last.span.end == token.span.start
    {
        last.span = Span::new(last.span.start, token.span.end);
        last.source = &source[last.span.as_range()];
        return;
    }
    tokens.push(token);
}

fn assign_ids<'a>(tokens: &mut [Token<'a>]) {
    let default_book: &'a str = tokens
        .iter()
        .find_map(|token| match token.data {
            TokenData::BookCode { code, .. } => Some(code),
            _ => None,
        })
        .unwrap_or("unknown");

    // Track the current source-borrowed book lexeme as we walk so a
    // multi-book document (rare) carries the right book in TokenId.
    // Pre-walker code keyed this off `sid.book_code` (also source-
    // borrowed); the sous-chef `Sid` now stores `BookId([u8; 3])` whose
    // `as_str()` does not have source lifetime, so we read it from the
    // BookCode tokens directly.
    let mut current_book: &'a str = default_book;
    for (index, token) in tokens.iter_mut().enumerate() {
        if let TokenData::BookCode { code, .. } = token.data {
            current_book = code;
        }
        token.id = TokenId::new(current_book, index as u32);
    }
}

pub fn into_usfm_from_tokens(tokens: &[Token<'_>]) -> String {
    tokens_to_usfm(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_assigns_ids_and_sids() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 2 text\n");
        assert_eq!(parsed.analysis.book_code, Some("GEN"));
        assert_eq!(
            parsed.tokens.first().map(|token| token.id),
            Some(TokenId::new("GEN", 0))
        );

        let gen_book = BookId::from_str("GEN").expect("GEN parses");
        let book = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::BookCode { code: "GEN", .. }))
            .expect("book code token");
        assert_eq!(book.sid, Some(Sid::new(gen_book, 0, 0)));

        let chapter_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "c", .. }))
            .expect("chapter marker token");
        assert_eq!(chapter_marker.sid, Some(Sid::new(gen_book, 1, 0)));

        let verse_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "v", .. }))
            .expect("verse marker token");
        assert_eq!(verse_marker.sid, Some(Sid::new(gen_book, 1, 2)));
    }

    #[test]
    fn parse_carries_bridge_range_end_on_the_verse_sid() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1-2 text\n\\v 3 more\n");
        let gen_book = BookId::from_str("GEN").expect("GEN parses");

        let verse_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "v", .. }))
            .expect("verse marker token");
        assert_eq!(verse_marker.sid, Some(Sid::with_range(gen_book, 1, 1, 2)));
        assert_eq!(verse_marker.sid.unwrap().to_string(), "GEN 1:1-2");

        // Every following token keeps the full-range sid until the next
        // chapter/verse marker advances it.
        let text_after_bridge = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Text) && token.source.trim() == "text")
            .expect("text token inside the bridge verse");
        assert_eq!(
            text_after_bridge.sid,
            Some(Sid::with_range(gen_book, 1, 1, 2))
        );
    }

    #[test]
    fn parse_saturates_an_oversized_bridge_span() {
        // \v 1-999 pin: no real chapter can span 999 verses, so the u8 delta
        // saturates at verse + 255 rather than silently truncating elsewhere.
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1-999 text\n");
        let gen_book = BookId::from_str("GEN").expect("GEN parses");
        let verse_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "v", .. }))
            .expect("verse marker token");
        let sid = verse_marker.sid.expect("verse sid");
        assert_eq!(sid.book, gen_book);
        assert_eq!(sid.verse, 1);
        assert_eq!(sid.verse_end(), 1 + 255);
    }

    #[test]
    fn parse_attaches_attributes_to_owning_marker() {
        let parsed = parse("\\w gracious|lemma=\"grace\" strong=\"H1\"\\w*.");
        let word_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "w", .. }))
            .expect("\\w marker token");
        let attrs = word_marker.attributes().expect("\\w carries attributes");
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key, "lemma");
        assert_eq!(attrs[0].value, "grace");
        assert!(!attrs[0].is_default);
        assert_eq!(attrs[1].key, "strong");
        assert_eq!(attrs[1].value, "H1");
        assert_eq!(
            into_usfm_from_tokens(&parsed.tokens),
            "\\w gracious|lemma=\"grace\" strong=\"H1\"\\w*."
        );
    }

    #[test]
    fn parse_keeps_marker_number_split() {
        let parsed = parse("\\v 12 text");
        assert!(matches!(
            parsed.tokens[0].data,
            TokenData::Marker { name: "v", .. }
        ));
        assert!(matches!(
            parsed.tokens[1].data,
            TokenData::Number {
                start: 12,
                end: None,
                ..
            }
        ));
        assert_eq!(parsed.tokens[0].source, "\\v ");
        assert_eq!(parsed.tokens[1].source, "12 ");
        assert_eq!(parsed.tokens[2].source, "text");
        assert_eq!(into_usfm_from_tokens(&parsed.tokens), "\\v 12 text");
    }
}
