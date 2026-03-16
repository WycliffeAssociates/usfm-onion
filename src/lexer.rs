use logos::Logos;

use crate::token::{
    AttributeEntryToken, BookCodeToken, MarkerToken, NumberRangeKind, NumberRangeToken,
    ScanResult, ScanToken, ScanTokenKind, Span, TriviaToken, marker_metadata, marker_text_name,
};

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum RawTokenKind {
    #[regex(r"[ \t]+")]
    Whitespace,

    #[regex(r"\r?\n|\r")]
    Newline,

    #[token("//")]
    OptBreak,

    #[token("|")]
    Pipe,

    #[token("\\*", priority = 10)]
    MilestoneEnd,

    #[regex(r"\\\+[a-z]+[0-9]*\*", priority = 9)]
    NestedClosingMarker,

    #[regex(r"\\\+[a-z]+[0-9]*", priority = 8)]
    NestedMarker,

    #[regex(r"\\[a-z]+[0-9]*-[se]", priority = 7)]
    Milestone,

    #[regex(r"\\[a-z]+[0-9]*(-[0-9]+)?\*", priority = 6)]
    ClosingMarker,

    #[regex(r"\\[a-z]+[0-9]*(-[0-9]+)?", priority = 5)]
    Marker,

    #[regex(r#"[^ \t\r\n\\|/][^\\\r\n|/]*"#)]
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingPayload {
    BookCode,
    NumberRange,
}

pub fn lex(source: &str) -> ScanResult<'_> {
    let mut tokens = Vec::new();
    let mut pending = None;
    let mut in_attribute_run = false;
    let mut index = 0usize;

    while index < source.len() {
        if in_attribute_run {
            if let Some((entry, end)) = consume_attribute_entry(source, index) {
                tokens.push(ScanToken::AttributeEntry(entry));
                index = end;
                continue;
            }

            if let Some((ws, end)) = consume_inline_whitespace(source, index) {
                tokens.push(ScanToken::Whitespace(ws));
                index = end;
                continue;
            }

            in_attribute_run = false;
        }

        let mut lexer = RawTokenKind::lexer(&source[index..]);
        let Some(result) = lexer.next() else {
            break;
        };
        let span = lexer.span();
        let raw_span = make_span(index + span.start, index + span.end);
        let slice = &source[raw_span.as_range()];

        match result {
            Ok(kind) => match kind {
                RawTokenKind::Whitespace => {
                    tokens.push(ScanToken::Whitespace(trivia(raw_span, slice)));
                }
                RawTokenKind::Newline => {
                    in_attribute_run = false;
                    tokens.push(ScanToken::Newline(trivia(raw_span, slice)));
                }
                RawTokenKind::OptBreak => {
                    pending = None;
                    in_attribute_run = false;
                    tokens.push(ScanToken::OptBreak(trivia(raw_span, slice)));
                }
                RawTokenKind::Pipe => {
                    pending = None;
                    in_attribute_run = true;
                    tokens.push(ScanToken::Pipe(trivia(raw_span, slice)));
                }
                RawTokenKind::MilestoneEnd => {
                    pending = None;
                    in_attribute_run = false;
                    tokens.push(ScanToken::MilestoneEnd(trivia(raw_span, slice)));
                }
                RawTokenKind::Marker
                | RawTokenKind::NestedMarker
                | RawTokenKind::ClosingMarker
                | RawTokenKind::NestedClosingMarker
                | RawTokenKind::Milestone => {
                    in_attribute_run = false;
                    let marker_name = marker_text_name(raw_kind_to_scan_kind(kind), slice);
                    let token = marker_token(kind, raw_span, slice);
                    pending = pending_payload_for(kind, marker_name);
                    tokens.push(token);
                }
                RawTokenKind::Text => {
                    if in_attribute_run {
                        in_attribute_run = false;
                    }

                    if let Some((matched, rest)) =
                        consume_contextual_payload(source, raw_span, slice, pending)
                    {
                        tokens.push(matched);
                        if !rest.lexeme.is_empty() {
                            tokens.push(ScanToken::Text(rest));
                        }
                        pending = None;
                    } else {
                        pending = None;
                        tokens.push(ScanToken::Text(trivia(raw_span, slice)));
                    }
                }
            },
            Err(()) => {
                pending = None;
                tokens.push(ScanToken::Text(trivia(raw_span, slice)));
            }
        }

        index = raw_span.end as usize;
    }

    ScanResult { tokens }
}

fn marker_token<'a>(kind: RawTokenKind, span: Span, lexeme: &'a str) -> ScanToken<'a> {
    let token_kind = raw_kind_to_scan_kind(kind);
    let name = marker_text_name(token_kind, lexeme);
    let token = MarkerToken {
        span,
        lexeme,
        name,
        metadata: marker_metadata(name),
    };

    match kind {
        RawTokenKind::Marker => ScanToken::Marker(token),
        RawTokenKind::NestedMarker => ScanToken::NestedMarker(token),
        RawTokenKind::ClosingMarker => ScanToken::ClosingMarker(token),
        RawTokenKind::NestedClosingMarker => ScanToken::NestedClosingMarker(token),
        RawTokenKind::Milestone => ScanToken::Milestone(token),
        _ => unreachable!("only marker-like raw tokens reach marker_token"),
    }
}

fn raw_kind_to_scan_kind(kind: RawTokenKind) -> ScanTokenKind {
    match kind {
        RawTokenKind::Whitespace => ScanTokenKind::Whitespace,
        RawTokenKind::Newline => ScanTokenKind::Newline,
        RawTokenKind::OptBreak => ScanTokenKind::OptBreak,
        RawTokenKind::Pipe => ScanTokenKind::Pipe,
        RawTokenKind::Marker => ScanTokenKind::Marker,
        RawTokenKind::NestedMarker => ScanTokenKind::NestedMarker,
        RawTokenKind::ClosingMarker => ScanTokenKind::ClosingMarker,
        RawTokenKind::NestedClosingMarker => ScanTokenKind::NestedClosingMarker,
        RawTokenKind::Milestone => ScanTokenKind::Milestone,
        RawTokenKind::MilestoneEnd => ScanTokenKind::MilestoneEnd,
        RawTokenKind::Text => ScanTokenKind::Text,
    }
}

fn trivia<'a>(span: Span, lexeme: &'a str) -> TriviaToken<'a> {
    TriviaToken { span, lexeme }
}

fn consume_attribute_entry<'a>(
    source: &'a str,
    start: usize,
) -> Option<(AttributeEntryToken<'a>, usize)> {
    let bytes = source.as_bytes();
    if start >= bytes.len() {
        return None;
    }

    let mut cursor = start;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'=' => break,
            b' ' | b'\t' | b'\r' | b'\n' | b'\\' | b'|' => return None,
            _ => cursor += 1,
        }
    }

    if cursor == start || bytes.get(cursor) != Some(&b'=') {
        return None;
    }

    let key = &source[start..cursor];
    cursor += 1;

    if bytes.get(cursor) != Some(&b'"') {
        return None;
    }

    let value_start = cursor + 1;
    cursor = value_start;
    let mut escaped = false;

    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'"' && !escaped {
            let end = cursor + 1;
            let span = Span::new(start as u32, end as u32);
            return Some((
                AttributeEntryToken {
                    span,
                    lexeme: &source[start..end],
                    key,
                    value: &source[value_start..cursor],
                },
                end,
            ));
        }

        escaped = byte == b'\\' && !escaped;
        if byte != b'\\' {
            escaped = false;
        }
        cursor += 1;
    }

    None
}

fn consume_inline_whitespace<'a>(source: &'a str, start: usize) -> Option<(TriviaToken<'a>, usize)> {
    let slice = &source[start..];
    let len = slice
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();
    if len == 0 {
        return None;
    }
    let span = Span::new(start as u32, (start + len) as u32);
    Some((
        TriviaToken {
            span,
            lexeme: &source[span.as_range()],
        },
        start + len,
    ))
}


fn pending_payload_for(kind: RawTokenKind, marker_name: &str) -> Option<PendingPayload> {
    match kind {
        RawTokenKind::Marker | RawTokenKind::NestedMarker => match marker_name {
            "id" => Some(PendingPayload::BookCode),
            "c" | "cp" | "ca" | "v" | "vp" | "va" => Some(PendingPayload::NumberRange),
            _ => None,
        },
        _ => None,
    }
}

fn consume_contextual_payload<'a>(
    source: &'a str,
    span: Span,
    slice: &'a str,
    pending: Option<PendingPayload>,
) -> Option<(ScanToken<'a>, TriviaToken<'a>)> {
    match pending? {
        PendingPayload::BookCode => consume_book_code(source, span, slice),
        PendingPayload::NumberRange => consume_number_range(source, span, slice),
    }
}

fn consume_book_code<'a>(
    source: &'a str,
    span: Span,
    slice: &'a str,
) -> Option<(ScanToken<'a>, TriviaToken<'a>)> {
    if slice.len() < 3 || !slice.is_ascii() {
        return None;
    }

    let code = &slice[..3];
    if !code.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return None;
    }

    let code_end = span.start + 3;
    let rest_span = Span::new(code_end, span.end);
    let rest_slice = &source[rest_span.as_range()];

    Some((
        ScanToken::BookCode(BookCodeToken {
            span: Span::new(span.start, code_end),
            lexeme: code,
            is_valid: is_valid_book_code(code),
        }),
        trivia(rest_span, rest_slice),
    ))
}

fn consume_number_range<'a>(
    source: &'a str,
    span: Span,
    slice: &'a str,
) -> Option<(ScanToken<'a>, TriviaToken<'a>)> {
    let parsed = parse_number_range_prefix(slice)?;
    let matched_text = &slice[..parsed.matched_len];
    let matched_end = span.start + parsed.matched_len as u32;
    let rest_span = Span::new(matched_end, span.end);
    let rest_slice = &source[rest_span.as_range()];

    Some((
        ScanToken::NumberRange(NumberRangeToken {
            span: Span::new(span.start, matched_end),
            lexeme: matched_text,
            start: parsed.start,
            end: parsed.end,
            kind: parsed.kind,
        }),
        trivia(rest_span, rest_slice),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedNumberRange {
    matched_len: usize,
    start: u32,
    end: Option<u32>,
    kind: NumberRangeKind,
}

fn parse_number_range_prefix(slice: &str) -> Option<ParsedNumberRange> {
    let bytes = slice.as_bytes();
    let mut cursor = 0usize;

    let start = parse_ascii_digits(bytes, &mut cursor)?;
    if start == 0 {
        return None;
    }

    consume_number_suffix(slice, &mut cursor);

    let mut kind = NumberRangeKind::Single;
    let mut end = None;

    while let Some(separator_len) = consume_number_separator(slice, &mut cursor) {
        let separator = bytes.get(cursor - 1)?;
        let next = match parse_ascii_digits(bytes, &mut cursor) {
            Some(value) => value,
            None => {
                cursor -= separator_len;
                break;
            }
        };
        consume_number_suffix(slice, &mut cursor);

        match separator {
            b'-' => {
                end = Some(next);
                kind = match kind {
                    NumberRangeKind::Single => NumberRangeKind::Range,
                    NumberRangeKind::Sequence => NumberRangeKind::SequenceWithRange,
                    existing => existing,
                };
            }
            b',' => {
                kind = match kind {
                    NumberRangeKind::Single => NumberRangeKind::Sequence,
                    NumberRangeKind::Range => NumberRangeKind::SequenceWithRange,
                    existing => existing,
                };
            }
            _ => {}
        }
    }

    Some(ParsedNumberRange {
        matched_len: cursor,
        start,
        end,
        kind,
    })
}

fn parse_ascii_digits(bytes: &[u8], cursor: &mut usize) -> Option<u32> {
    let start = *cursor;
    let mut value = 0u32;

    while let Some(&byte) = bytes.get(*cursor) {
        if !byte.is_ascii_digit() {
            break;
        }
        value = value.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
        *cursor += 1;
    }

    (*cursor > start).then_some(value)
}

fn consume_number_suffix(slice: &str, cursor: &mut usize) {
    while *cursor < slice.len() {
        let next = &slice[*cursor..];
        let Some(ch) = next.chars().next() else {
            break;
        };
        if ch.is_alphabetic() || matches!(ch, '\u{0300}'..='\u{036f}' | '\u{1ab0}'..='\u{1aff}' | '\u{1dc0}'..='\u{1dff}' | '\u{20d0}'..='\u{20ff}' | '\u{fe20}'..='\u{fe2f}') {
            *cursor += ch.len_utf8();
        } else {
            break;
        }
    }
}

fn consume_number_separator(slice: &str, cursor: &mut usize) -> Option<usize> {
    let mut consumed_rtl_mark = false;
    if slice[*cursor..].starts_with('\u{200f}') {
        *cursor += '\u{200f}'.len_utf8();
        consumed_rtl_mark = true;
    }

    let bytes = slice.as_bytes();
    match bytes.get(*cursor) {
        Some(b'-' | b',') => {
            *cursor += 1;
            Some(if consumed_rtl_mark { 2 } else { 1 })
        }
        _ => {
            if consumed_rtl_mark {
                *cursor -= '\u{200f}'.len_utf8();
            }
            None
        }
    }
}

fn make_span(start: usize, end: usize) -> Span {
    Span::new(start as u32, end as u32)
}

fn is_valid_book_code(code: &str) -> bool {
    matches!(
        code,
        "FRT"
            | "GEN"
            | "EXO"
            | "LEV"
            | "NUM"
            | "DEU"
            | "JOS"
            | "JDG"
            | "RUT"
            | "1SA"
            | "2SA"
            | "1KI"
            | "2KI"
            | "1CH"
            | "2CH"
            | "EZR"
            | "NEH"
            | "EST"
            | "JOB"
            | "PSA"
            | "PRO"
            | "ECC"
            | "SNG"
            | "ISA"
            | "JER"
            | "LAM"
            | "EZK"
            | "DAN"
            | "HOS"
            | "JOL"
            | "AMO"
            | "OBA"
            | "JON"
            | "MIC"
            | "NAM"
            | "HAB"
            | "ZEP"
            | "HAG"
            | "ZEC"
            | "MAL"
            | "MAT"
            | "MRK"
            | "LUK"
            | "JHN"
            | "ACT"
            | "ROM"
            | "1CO"
            | "2CO"
            | "GAL"
            | "EPH"
            | "PHP"
            | "COL"
            | "1TH"
            | "2TH"
            | "1TI"
            | "2TI"
            | "TIT"
            | "PHM"
            | "HEB"
            | "JAS"
            | "1PE"
            | "2PE"
            | "1JN"
            | "2JN"
            | "3JN"
            | "JUD"
            | "REV"
            | "TOB"
            | "JDT"
            | "ESG"
            | "WIS"
            | "SIR"
            | "BAR"
            | "LJE"
            | "S3Y"
            | "SUS"
            | "BEL"
            | "1MA"
            | "2MA"
            | "3MA"
            | "4MA"
            | "1ES"
            | "2ES"
            | "MAN"
            | "PS2"
            | "ODA"
            | "PSS"
            | "EZA"
            | "5EZ"
            | "6EZ"
            | "DAG"
            | "PS3"
            | "2BA"
            | "LBA"
            | "JUB"
            | "ENO"
            | "1MQ"
            | "2MQ"
            | "3MQ"
            | "REP"
            | "4BA"
            | "LAO"
            | "INT"
            | "CNC"
            | "GLO"
            | "TDX"
            | "NDX"
            | "OTH"
            | "BAK"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<ScanTokenKind> {
        lex(source).tokens.iter().map(ScanToken::kind).collect()
    }

    #[test]
    fn preserves_every_byte_with_contextual_book_code_split() {
        let result = lex("\\id GEN Genesis\n");
        assert_eq!(
            kinds("\\id GEN Genesis\n"),
            vec![
                ScanTokenKind::Marker,
                ScanTokenKind::Whitespace,
                ScanTokenKind::BookCode,
                ScanTokenKind::Text,
                ScanTokenKind::Newline,
            ]
        );
        assert_eq!(result.tokens.iter().map(ScanToken::lexeme).collect::<String>(), "\\id GEN Genesis\n");
    }

    #[test]
    fn recognizes_number_range_after_contextual_markers() {
        let result = lex("\\v 12-14a text");
        assert_eq!(
            result.tokens.iter().map(ScanToken::kind).collect::<Vec<_>>(),
            vec![
                ScanTokenKind::Marker,
                ScanTokenKind::Whitespace,
                ScanTokenKind::NumberRange,
                ScanTokenKind::Text,
            ]
        );

        let ScanToken::NumberRange(number) = result.tokens[2] else {
            panic!("expected number range");
        };
        assert_eq!(number.start, 12);
        assert_eq!(number.end, Some(14));
        assert_eq!(number.kind, NumberRangeKind::Range);
    }

    #[test]
    fn leaves_non_contextual_numbers_as_text() {
        assert_eq!(
            kinds("\\p 12-14"),
            vec![
                ScanTokenKind::Marker,
                ScanTokenKind::Whitespace,
                ScanTokenKind::Text,
            ]
        );
    }

    #[test]
    fn carries_marker_metadata_without_later_lookup() {
        let result = lex("\\q1");
        let ScanToken::Marker(marker) = result.tokens[0] else {
            panic!("expected marker");
        };

        assert_eq!(marker.name, "q1");
        assert_eq!(marker.metadata.canonical, Some("q1"));
        assert_eq!(marker.metadata.kind, Some(crate::marker_defs::SpecMarkerKind::Paragraph));
    }

    #[test]
    fn invalid_book_code_stays_structured() {
        let result = lex("\\id XYZ Stuff");
        let ScanToken::BookCode(book) = result.tokens[2] else {
            panic!("expected book code");
        };
        assert!(!book.is_valid);
    }

    #[test]
    fn numeric_prefixed_book_code_is_recognized() {
        let result = lex("\\id 1JN - Berean Standard Bible\n");
        let ScanToken::BookCode(book) = result.tokens[2] else {
            panic!("expected book code");
        };
        assert_eq!(book.lexeme, "1JN");
        assert!(book.is_valid);
    }

    #[test]
    fn splits_pipe_and_attribute_entries() {
        let result = lex("\\zaln-s |x-strong=\"G42450\" x-lemma=\"πρεσβύτερος\"");
        assert_eq!(
            result.tokens.iter().map(ScanToken::kind).collect::<Vec<_>>(),
            vec![
                ScanTokenKind::Milestone,
                ScanTokenKind::Whitespace,
                ScanTokenKind::Pipe,
                ScanTokenKind::AttributeEntry,
                ScanTokenKind::Whitespace,
                ScanTokenKind::AttributeEntry,
            ]
        );

        let ScanToken::AttributeEntry(entry) = result.tokens[3] else {
            panic!("expected attribute entry");
        };
        assert_eq!(entry.key, "x-strong");
        assert_eq!(entry.value, "G42450");
    }

    #[test]
    fn stops_attribute_run_at_marker_boundaries() {
        let result = lex(
            "\\zaln-s |x-strong=\"G42450\" x-content=\"πρεσβύτερος\"\\*\\w elder|x-occurrence=\"1\" x-occurrences=\"1\"\\w*\\zaln-e\\*,",
        );
        assert_eq!(
            result.tokens.iter().map(ScanToken::kind).collect::<Vec<_>>(),
            vec![
                ScanTokenKind::Milestone,
                ScanTokenKind::Whitespace,
                ScanTokenKind::Pipe,
                ScanTokenKind::AttributeEntry,
                ScanTokenKind::Whitespace,
                ScanTokenKind::AttributeEntry,
                ScanTokenKind::MilestoneEnd,
                ScanTokenKind::Marker,
                ScanTokenKind::Whitespace,
                ScanTokenKind::Text,
                ScanTokenKind::Pipe,
                ScanTokenKind::AttributeEntry,
                ScanTokenKind::Whitespace,
                ScanTokenKind::AttributeEntry,
                ScanTokenKind::ClosingMarker,
                ScanTokenKind::Milestone,
                ScanTokenKind::MilestoneEnd,
                ScanTokenKind::Text,
            ]
        );
        assert_eq!(result.tokens.last().map(ScanToken::lexeme), Some(","));
    }
}
