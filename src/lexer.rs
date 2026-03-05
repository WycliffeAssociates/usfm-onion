use crate::token::{LexResult, LexToken, RawTokenKind};

pub fn lex(source: &str) -> LexResult {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        let token = if source[index..].starts_with("\r\n") {
            let token = build_token(source, RawTokenKind::Newline, index, index + 2);
            index += 2;
            token
        } else {
            let ch = source[index..].chars().next().expect("valid utf-8");
            match ch {
                '\n' => {
                    let token = build_token(source, RawTokenKind::Newline, index, index + 1);
                    index += 1;
                    token
                }
                ' ' | '\t' => {
                    let end = consume_while(source, index, |next| matches!(next, ' ' | '\t'));
                    let token = build_token(source, RawTokenKind::Whitespace, index, end);
                    index = end;
                    token
                }
                '\r' => {
                    let token = build_token(source, RawTokenKind::Whitespace, index, index + 1);
                    index += 1;
                    token
                }
                '|' => {
                    let end = scan_attributes(source, index);
                    let token = build_token(source, RawTokenKind::Attributes, index, end);
                    index = end;
                    token
                }
                '\\' => {
                    if source[index..].starts_with("\\*") {
                        let token =
                            build_token(source, RawTokenKind::MilestoneEnd, index, index + 2);
                        index += 2;
                        token
                    } else {
                        let (kind, end) = scan_marker(source, index);
                        let token = build_token(source, kind, index, end);
                        index = end;
                        token
                    }
                }
                _ => {
                    let end = consume_until(source, index, |next| {
                        next == '\n' || next == '\r' || next == '\\' || next == '|'
                    });
                    let token = build_token(source, RawTokenKind::Text, index, end);
                    index = end;
                    token
                }
            }
        };
        tokens.push(token);
    }

    LexResult { tokens }
}

fn build_token(source: &str, kind: RawTokenKind, start: usize, end: usize) -> LexToken {
    LexToken {
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

fn consume_until(source: &str, start: usize, predicate: impl Fn(char) -> bool) -> usize {
    let mut end = source.len();
    for (offset, ch) in source[start..].char_indices() {
        if predicate(ch) {
            end = start + offset;
            break;
        }
    }
    end
}

fn scan_marker(source: &str, start: usize) -> (RawTokenKind, usize) {
    let bytes = source.as_bytes();
    let mut index = start + 1;
    let mut nested = false;

    if bytes.get(index) == Some(&b'+') {
        nested = true;
        index += 1;
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
        return (RawTokenKind::Text, start + 1);
    }

    if bytes.get(index) == Some(&b'*') {
        index += 1;
        return if nested {
            (RawTokenKind::NestedClosingMarker, index)
        } else {
            (RawTokenKind::ClosingMarker, index)
        };
    }

    let marker_text = &source[start..index];
    let marker_name = marker_text.strip_prefix('\\').unwrap_or(marker_text);
    if !nested
        && source[index..].starts_with("\\*")
        && is_self_closing_milestone_marker(marker_name)
    {
        return (RawTokenKind::Milestone, index);
    }
    if marker_text.ends_with("-s") || marker_text.ends_with("-e") {
        return (RawTokenKind::Milestone, index);
    }

    if nested {
        (RawTokenKind::NestedMarker, index)
    } else {
        (RawTokenKind::Marker, index)
    }
}

fn is_self_closing_milestone_marker(marker: &str) -> bool {
    matches!(marker, "ts" | "zms")
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
    use crate::token::RawTokenKind;

    #[test]
    fn preserves_spans_across_whitespace_and_newlines() {
        let result = lex("\\p \r\nText");
        assert_eq!(result.tokens.len(), 4);
        assert_eq!(result.tokens[0].kind, RawTokenKind::Marker);
        assert_eq!(result.tokens[0].span, 0..2);
        assert_eq!(result.tokens[1].kind, RawTokenKind::Whitespace);
        assert_eq!(result.tokens[1].span, 2..3);
        assert_eq!(result.tokens[2].kind, RawTokenKind::Newline);
        assert_eq!(result.tokens[2].span, 3..5);
        assert_eq!(result.tokens[3].kind, RawTokenKind::Text);
        assert_eq!(result.tokens[3].span, 5..9);
    }

    #[test]
    fn self_closing_milestones_split_into_marker_and_milestone_end() {
        let result = lex("\\ts\\* \\zms\\*");
        assert_eq!(result.tokens[0].kind, RawTokenKind::Milestone);
        assert_eq!(result.tokens[0].text, "\\ts");
        assert_eq!(result.tokens[1].kind, RawTokenKind::MilestoneEnd);
        assert_eq!(result.tokens[1].text, "\\*");
        assert_eq!(result.tokens[3].kind, RawTokenKind::Milestone);
        assert_eq!(result.tokens[3].text, "\\zms");
        assert_eq!(result.tokens[4].kind, RawTokenKind::MilestoneEnd);
    }

    #[test]
    fn attributes_preserve_escaped_quotes_inside_values() {
        let result = lex(
            "\\fig Caption|alt=\"He said: \\\"this is interesting\\\"\" src=\"IMG_0195.JPG\"\\fig*",
        );
        let attributes = result
            .tokens
            .iter()
            .find(|token| token.kind == RawTokenKind::Attributes)
            .expect("expected attribute token");
        assert_eq!(
            attributes.text,
            "|alt=\"He said: \\\"this is interesting\\\"\" src=\"IMG_0195.JPG\""
        );
    }
}
