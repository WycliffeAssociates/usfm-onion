use crate::lexer::lex;
use crate::marker_defs::structural_marker_info;
use crate::token::{
    AttributeItem, Lexeme, NumberRangeToken, ParseAnalysis, ParseResult, ScanToken, Sid, Span,
    Token, TokenData, TokenId, TokenKind, tokens_to_usfm,
};

pub fn parse(source: &str) -> ParseResult<'_> {
    let lexed = lex(source);
    parse_lexemes(source, &lexed.tokens)
}

pub fn parse_lexemes<'a>(source: &'a str, lexemes: &[Lexeme<'a>]) -> ParseResult<'a> {
    let mut analysis = ParseAnalysis::default();
    let mut state = ParseState::default();
    let mut tokens = Vec::new();
    let mut cursor = 0usize;

    while cursor < lexemes.len() {
        if matches!(lexemes[cursor], ScanToken::Pipe(_)) {
            flush_pending_whitespace(source, &mut state, &mut tokens);
            let (token, next_cursor) = build_attribute_list(source, lexemes, cursor, &state);
            push_token(source, &mut tokens, token);
            cursor = next_cursor;
            continue;
        }

        match lexemes[cursor] {
            ScanToken::Whitespace(ws) => {
                state.pending_ws = Some(ws.span);
                cursor += 1;
            }
            ScanToken::Newline(token) => {
                flush_pending_whitespace(source, &mut state, &mut tokens);
                let token = Token {
                    id: TokenId::new("", 0),
                    sid: state.current_sid.clone(),
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
                if let Some((next_cursor, book_code)) = try_consume_book_code(lexemes, cursor, &mut state)
                {
                    analysis.book_code = Some(book_code.lexeme);
                    let marker_token = token_with_current_ws_and_sid(
                        source,
                        &mut state,
                        marker.span,
                        TokenData::Marker {
                            name: marker.name,
                            metadata: marker.metadata,
                            structural: structural_marker_info(marker.name, marker.metadata.kind),
                            nested: false,
                        },
                    );
                    push_token(source, &mut tokens, marker_token);

                    state.pending_ws = book_code.leading_ws;
                    state.current_book_code = Some(book_code.lexeme);
                    state.current_sid = Some(Sid::new(book_code.lexeme, 0, 0));
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
                            structural: structural_marker_info(marker.name, marker.metadata.kind),
                            nested: false,
                        },
                        next_sid.clone(),
                    );
                    push_token(source, &mut tokens, marker_token);
                    state.pending_ws = number.leading_ws;
                    update_sid_state(&mut state, marker.name, &number.number);
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
                        structural: structural_marker_info(marker.name, marker.metadata.kind),
                        nested: false,
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
                        structural: structural_marker_info(marker.name, marker.metadata.kind),
                        nested: true,
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
                        structural: structural_marker_info(marker.name, marker.metadata.kind),
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
                        structural: structural_marker_info(marker.name, marker.metadata.kind),
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
                        structural: structural_marker_info(marker.name, marker.metadata.kind),
                    },
                );
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::MilestoneEnd(token) => {
                let token =
                    token_with_current_ws_and_sid(source, &mut state, token.span, TokenData::MilestoneEnd);
                push_token(source, &mut tokens, token);
                cursor += 1;
            }
            ScanToken::BookCode(book) => {
                if analysis.book_code.is_none() {
                    analysis.book_code = Some(book.lexeme);
                }
                state.current_book_code = Some(book.lexeme);
                state.current_sid = Some(Sid::new(book.lexeme, 0, 0));
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
                let token = token_with_current_ws_and_sid(
                    source,
                    &mut state,
                    text.span,
                    TokenData::Text,
                );
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
    current_chapter: u32,
    current_sid: Option<Sid<'a>>,
    pending_ws: Option<Span>,
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

fn build_attribute_list<'a>(
    source: &'a str,
    lexemes: &[Lexeme<'a>],
    start: usize,
    state: &ParseState<'a>,
) -> (Token<'a>, usize) {
    let ScanToken::Pipe(pipe) = lexemes[start] else {
        unreachable!("attribute list must start with pipe");
    };

    let mut entries = Vec::new();
    let mut end_span = pipe.span;
    let mut cursor = start + 1;

    while cursor < lexemes.len() {
        match lexemes[cursor] {
            ScanToken::Whitespace(ws) => {
                end_span = ws.span;
                cursor += 1;
            }
            ScanToken::AttributeEntry(entry) => {
                entries.push(AttributeItem {
                    span: entry.span,
                    source: entry.lexeme,
                    key: entry.key,
                    value: entry.value,
                });
                end_span = entry.span;
                cursor += 1;
            }
            _ => break,
        }
    }

    let start_span = state.pending_ws.map(|ws| ws.start).unwrap_or(pipe.span.start);
    let span = Span::new(start_span, end_span.end);
    let token = Token {
        id: TokenId::new("", 0),
        sid: state.current_sid.clone(),
        span,
        source: &source[span.as_range()],
        data: TokenData::AttributeList { entries },
    };
    (token, cursor)
}

fn advanced_sid<'a>(
    state: &ParseState<'a>,
    marker_name: &str,
    number: &NumberRangeToken<'_>,
) -> Option<Sid<'a>> {
    let book = state.current_book_code?;
    match marker_name {
        "c" => Some(Sid::new(book, number.start, 0)),
        "v" => Some(Sid::new(book, state.current_chapter, number.start)),
        _ => state.current_sid.clone(),
    }
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
    token_with_current_ws(source, state, span, data, state.current_sid.clone())
}

fn token_with_current_ws<'a>(
    source: &'a str,
    state: &mut ParseState<'a>,
    span: Span,
    data: TokenData<'a>,
    sid: Option<Sid<'a>>,
) -> Token<'a> {
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

fn flush_pending_whitespace<'a>(source: &'a str, state: &mut ParseState<'a>, tokens: &mut Vec<Token<'a>>) {
    let Some(ws) = state.pending_ws.take() else {
        return;
    };
    if let Some(last) = tokens.last_mut() {
        last.span = Span::new(last.span.start, ws.end);
        last.source = &source[last.span.as_range()];
    } else {
        tokens.push(Token {
            id: TokenId::new("", 0),
            sid: state.current_sid.clone(),
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

fn assign_ids(tokens: &mut [Token<'_>]) {
    let default_book = tokens
        .iter()
        .find_map(|token| match token.data {
            TokenData::BookCode { code, .. } => Some(code),
            _ => None,
        })
        .unwrap_or("unknown");

    for (index, token) in tokens.iter_mut().enumerate() {
        let book = token
            .sid
            .map(|sid| sid.book_code)
            .unwrap_or(default_book);
        token.id = TokenId::new(book, index as u32);
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
        assert_eq!(parsed.tokens.first().map(|token| token.id), Some(TokenId::new("GEN", 0)));

        let book = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::BookCode { code: "GEN", .. }))
            .expect("book code token");
        assert_eq!(book.sid, Some(Sid::new("GEN", 0, 0)));

        let chapter_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "c", .. }))
            .expect("chapter marker token");
        assert_eq!(chapter_marker.sid, Some(Sid::new("GEN", 1, 0)));

        let verse_marker = parsed
            .tokens
            .iter()
            .find(|token| matches!(token.data, TokenData::Marker { name: "v", .. }))
            .expect("verse marker token");
        assert_eq!(verse_marker.sid, Some(Sid::new("GEN", 1, 2)));
    }

    #[test]
    fn parse_groups_attribute_list() {
        let parsed = parse("\\w gracious|lemma=\"grace\" strong=\"H1\"\\w*.");
        assert!(parsed
            .tokens
            .iter()
            .any(|token| matches!(token.data, TokenData::AttributeList { .. })));
        assert_eq!(into_usfm_from_tokens(&parsed.tokens), "\\w gracious|lemma=\"grace\" strong=\"H1\"\\w*.");
    }

    #[test]
    fn parse_keeps_marker_number_split() {
        let parsed = parse("\\v 12 text");
        assert!(matches!(parsed.tokens[0].data, TokenData::Marker { name: "v", .. }));
        assert!(matches!(
            parsed.tokens[1].data,
            TokenData::Number {
                start: 12,
                end: None,
                ..
            }
        ));
        assert_eq!(parsed.tokens[1].source, " 12");
    }
}
