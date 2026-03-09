use crate::model::token::{ScanResult, ScanToken, ScanTokenKind};

pub fn lex(source: &str) -> ScanResult {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        let token = if source[index..].starts_with("\r\n") {
            let token = build_token(source, ScanTokenKind::Newline, index, index + 2);
            index += 2;
            token
        } else {
            let ch = source[index..].chars().next().expect("valid utf-8");
            match ch {
                '\n' => {
                    let token = build_token(source, ScanTokenKind::Newline, index, index + 1);
                    index += 1;
                    token
                }
                ' ' | '\t' => {
                    let end = consume_while(source, index, |next| matches!(next, ' ' | '\t'));
                    let token = build_token(source, ScanTokenKind::Whitespace, index, end);
                    index = end;
                    token
                }
                '\r' => {
                    let token = build_token(source, ScanTokenKind::Newline, index, index + 1);
                    index += 1;
                    token
                }
                '/' if source[index..].starts_with("//") => {
                    let token = build_token(source, ScanTokenKind::OptBreak, index, index + 2);
                    index += 2;
                    token
                }
                '|' => {
                    let end = scan_attributes(source, index);
                    let token = build_token(source, ScanTokenKind::Attributes, index, end);
                    index = end;
                    token
                }
                '\\' => {
                    if source[index..].starts_with("\\*") {
                        let token =
                            build_token(source, ScanTokenKind::MilestoneEnd, index, index + 2);
                        index += 2;
                        token
                    } else if escaped_text_width(source, index).is_some() {
                        let end = scan_text(source, index);
                        let token = build_token(source, ScanTokenKind::Text, index, end);
                        index = end;
                        token
                    } else {
                        let (kind, end) = scan_marker(source, index);
                        let token = build_token(source, kind, index, end);
                        index = end;
                        token
                    }
                }
                _ => {
                    let end = scan_text(source, index);
                    let token = build_token(source, ScanTokenKind::Text, index, end);
                    index = end;
                    token
                }
            }
        };
        tokens.push(token);
    }

    ScanResult { tokens }
}

fn build_token(source: &str, kind: ScanTokenKind, start: usize, end: usize) -> ScanToken {
    ScanToken {
        kind,
        span: start..end,
        text: source[start..end].to_string(),
    }
}

fn consume_while(source: &str, start: usize, predicate: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    for (offset, ch) in source[start..].char_indices() {
        if !predicate(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    end
}

fn scan_text(source: &str, start: usize) -> usize {
    let mut end = start;
    let mut index = start;

    while index < source.len() {
        if source[index..].starts_with("//") {
            break;
        }

        let ch = source[index..].chars().next().expect("valid utf-8");
        if matches!(ch, '\n' | '\r' | '|') {
            break;
        }

        if ch == '\\' {
            if let Some(width) = escaped_text_width(source, index) {
                index += width;
                end = index;
                continue;
            }
            break;
        }

        index += ch.len_utf8();
        end = index;
    }

    end
}

fn scan_marker(source: &str, start: usize) -> (ScanTokenKind, usize) {
    let bytes = source.as_bytes();
    let mut index = start + 1;
    let mut nested = false;

    if bytes.get(index) == Some(&b'+') {
        nested = true;
        index += 1;
    }

    if !bytes.get(index).is_some_and(u8::is_ascii_lowercase) {
        return (ScanTokenKind::Text, start + 1);
    }

    let name_start = index;
    while let Some(byte) = bytes.get(index) {
        let is_valid = byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-';
        if !is_valid {
            break;
        }
        index += 1;
    }

    if index == name_start {
        return (ScanTokenKind::Text, start + 1);
    }

    if bytes.get(index) == Some(&b'*') {
        index += 1;
        return if nested {
            (ScanTokenKind::NestedClosingMarker, index)
        } else {
            (ScanTokenKind::ClosingMarker, index)
        };
    }

    let marker_text = &source[start..index];
    let marker_name = marker_text.strip_prefix('\\').unwrap_or(marker_text);
    if !nested
        && source[index..].starts_with("\\*")
        && is_self_closing_milestone_marker(marker_name)
    {
        return (ScanTokenKind::Milestone, index);
    }
    if marker_text.ends_with("-s") || marker_text.ends_with("-e") {
        return (ScanTokenKind::Milestone, index);
    }

    if nested {
        (ScanTokenKind::NestedMarker, index)
    } else {
        (ScanTokenKind::Marker, index)
    }
}

fn is_self_closing_milestone_marker(marker: &str) -> bool {
    matches!(marker, "ts" | "zms")
}

fn escaped_text_width(source: &str, index: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(index) != Some(&b'\\') {
        return None;
    }

    match bytes.get(index + 1) {
        Some(b'\\') => {
            if bytes.get(index + 2).is_some_and(u8::is_ascii_lowercase) {
                None
            } else {
                Some(2)
            }
        }
        Some(b'/' | b'~' | b'|') => Some(2),
        Some(b'u') if has_hex_digits(bytes, index + 2, 4) => Some(6),
        Some(b'U') if has_hex_digits(bytes, index + 2, 8) => Some(10),
        _ => None,
    }
}

fn has_hex_digits(bytes: &[u8], start: usize, len: usize) -> bool {
    bytes
        .get(start..start + len)
        .is_some_and(|slice| slice.iter().all(u8::is_ascii_hexdigit))
}

fn scan_attributes(source: &str, start: usize) -> usize {
    let mut end = source.len();
    let mut in_quotes = false;
    let mut previous_was_escape = false;

    for (offset, ch) in source[start + 1..].char_indices() {
        if in_quotes {
            if ch == '"' && !previous_was_escape {
                in_quotes = false;
            }
            previous_was_escape = ch == '\\' && !previous_was_escape;
            continue;
        }

        match ch {
            '"' => {
                in_quotes = true;
                previous_was_escape = false;
            }
            '\n' | '\r' | '\\' => {
                end = start + 1 + offset;
                break;
            }
            _ => {
                previous_was_escape = false;
            }
        }
    }

    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::token::ScanTokenKind;

    #[test]
    fn preserves_spans_across_whitespace_and_newlines() {
        let result = lex("\\p \r\nText");
        assert_eq!(result.tokens.len(), 4);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Marker);
        assert_eq!(result.tokens[0].span, 0..2);
        assert_eq!(result.tokens[1].kind, ScanTokenKind::Whitespace);
        assert_eq!(result.tokens[1].span, 2..3);
        assert_eq!(result.tokens[2].kind, ScanTokenKind::Newline);
        assert_eq!(result.tokens[2].span, 3..5);
        assert_eq!(result.tokens[3].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[3].span, 5..9);
    }

    #[test]
    fn carriage_return_is_tokenized_as_newline() {
        let result = lex("\\p\rText");
        assert_eq!(result.tokens[1].kind, ScanTokenKind::Newline);
    }

    #[test]
    fn optbreak_is_tokenized_as_explicit_token() {
        let result = lex("a//b");
        assert_eq!(result.tokens.len(), 3);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[0].text, "a");
        assert_eq!(result.tokens[1].kind, ScanTokenKind::OptBreak);
        assert_eq!(result.tokens[1].text, "//");
        assert_eq!(result.tokens[2].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[2].text, "b");
    }

    #[test]
    fn self_closing_milestones_split_into_marker_and_milestone_end() {
        let result = lex("\\ts\\* \\zms\\*");
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Milestone);
        assert_eq!(result.tokens[0].text, "\\ts");
        assert_eq!(result.tokens[1].kind, ScanTokenKind::MilestoneEnd);
        assert_eq!(result.tokens[1].text, "\\*");
        assert_eq!(result.tokens[3].kind, ScanTokenKind::Milestone);
        assert_eq!(result.tokens[3].text, "\\zms");
        assert_eq!(result.tokens[4].kind, ScanTokenKind::MilestoneEnd);
    }

    #[test]
    fn attributes_preserve_escaped_quotes_inside_values() {
        let result = lex(
            "\\fig Caption|alt=\"He said: \\\"this is interesting\\\"\" src=\"IMG_0195.JPG\"\\fig*",
        );
        let attributes = result
            .tokens
            .iter()
            .find(|token| token.kind == ScanTokenKind::Attributes)
            .expect("expected attribute token");
        assert_eq!(
            attributes.text,
            "|alt=\"He said: \\\"this is interesting\\\"\" src=\"IMG_0195.JPG\""
        );
    }

    #[test]
    fn escaped_pipe_and_slash_stay_in_text() {
        let result = lex("a\\|b \\/ c");
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[0].text, "a\\|b \\/ c");
    }

    #[test]
    fn escaped_backslash_before_marker_text_starts_marker() {
        let result = lex("\\\\v 16");
        assert_eq!(result.tokens.len(), 4);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[0].text, "\\");
        assert_eq!(result.tokens[1].kind, ScanTokenKind::Marker);
        assert_eq!(result.tokens[1].text, "\\v");
        assert_eq!(result.tokens[2].kind, ScanTokenKind::Whitespace);
        assert_eq!(result.tokens[3].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[3].text, "16");
    }

    #[test]
    fn usv_escapes_stay_in_text() {
        let result = lex("pre \\u1234 \\U0001F600 post");
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[0].text, "pre \\u1234 \\U0001F600 post");
    }

    #[test]
    fn marker_must_start_with_letter_after_backslash() {
        let result = lex("\\1abc");
        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[0].text, "\\");
        assert_eq!(result.tokens[1].kind, ScanTokenKind::Text);
        assert_eq!(result.tokens[1].text, "1abc");
    }
}
