use crate::ParseRecovery;
use crate::syntax::Document;
use crate::token::{
    FlatToken, LexToken, RawTokenKind, Span, TokenKind, TokenViewOptions, WhitespacePolicy,
    normalized_marker_name, number_prefix_len, strip_closing_star, strip_marker_backslash,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ParseAnalysis {
    pub book_code: Option<String>,
    pub book_code_token_index: Option<usize>,
    pub book_code_prefix_len: Option<usize>,
    pub number_token_indexes: Vec<usize>,
    pub recoveries: Vec<ParseRecovery>,
    pub document: Document,
}

#[derive(Debug, Clone)]
pub struct ParseHandle {
    source: String,
    raw_tokens: Vec<LexToken>,
    analysis: ParseAnalysis,
}

impl ParseHandle {
    pub(crate) fn new(source: String, raw_tokens: Vec<LexToken>, analysis: ParseAnalysis) -> Self {
        Self {
            source,
            raw_tokens,
            analysis,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn book_code(&self) -> Option<&str> {
        self.analysis.book_code.as_deref()
    }

    pub(crate) fn raw_tokens(&self) -> &[LexToken] {
        &self.raw_tokens
    }

    pub(crate) fn analysis(&self) -> &ParseAnalysis {
        &self.analysis
    }

    #[allow(dead_code)]
    pub(crate) fn document(&self) -> &Document {
        &self.analysis.document
    }
}

pub fn recoveries(handle: &ParseHandle) -> &[ParseRecovery] {
    &handle.analysis.recoveries
}

pub fn tokens(handle: &ParseHandle, options: TokenViewOptions) -> Vec<FlatToken> {
    let mut projected = project_raw_tokens(handle.raw_tokens(), handle.analysis());
    if options.whitespace_policy == WhitespacePolicy::MergeToVisible {
        merge_horizontal_whitespace(&mut projected);
    }

    let book_code = handle.analysis.book_code.as_deref().unwrap_or("unknown");
    assign_ids(&mut projected, book_code);
    assign_sids(&mut projected, book_code);
    projected
}

fn project_raw_tokens(raw_tokens: &[LexToken], analysis: &ParseAnalysis) -> Vec<FlatToken> {
    let mut projected = Vec::new();

    for (index, token) in raw_tokens.iter().enumerate() {
        match token.kind {
            RawTokenKind::Whitespace => projected.push(flat_token(
                TokenKind::HorizontalWhitespace,
                token.span.clone(),
                None,
                token.text.clone(),
            )),
            RawTokenKind::Newline => projected.push(flat_token(
                TokenKind::VerticalWhitespace,
                token.span.clone(),
                None,
                token.text.clone(),
            )),
            RawTokenKind::Marker | RawTokenKind::NestedMarker => projected.push(flat_token(
                TokenKind::Marker,
                token.span.clone(),
                Some(strip_marker_backslash(&token.text).to_string()),
                token.text.clone(),
            )),
            RawTokenKind::ClosingMarker | RawTokenKind::NestedClosingMarker => {
                projected.push(flat_token(
                    TokenKind::EndMarker,
                    token.span.clone(),
                    Some(strip_closing_star(&token.text).to_string()),
                    token.text.clone(),
                ))
            }
            RawTokenKind::Milestone => projected.push(flat_token(
                TokenKind::Milestone,
                token.span.clone(),
                Some(strip_marker_backslash(&token.text).to_string()),
                token.text.clone(),
            )),
            RawTokenKind::MilestoneEnd => projected.push(flat_token(
                TokenKind::MilestoneEnd,
                token.span.clone(),
                Some("*".to_string()),
                token.text.clone(),
            )),
            RawTokenKind::Attributes => projected.push(flat_token(
                TokenKind::Attributes,
                token.span.clone(),
                None,
                token.text.clone(),
            )),
            RawTokenKind::Text => {
                if Some(index) == analysis.book_code_token_index
                    && let Some(prefix_len) = analysis.book_code_prefix_len
                {
                    projected.push(flat_token(
                        TokenKind::BookCode,
                        token.span.start..token.span.start + prefix_len,
                        None,
                        token.text[..prefix_len].to_string(),
                    ));
                    if prefix_len < token.text.len() {
                        projected.push(flat_token(
                            TokenKind::Text,
                            token.span.start + prefix_len..token.span.end,
                            None,
                            token.text[prefix_len..].to_string(),
                        ));
                    }
                    continue;
                }

                if analysis.number_token_indexes.contains(&index)
                    && let Some(prefix_len) = number_prefix_len(&token.text)
                {
                    projected.push(flat_token(
                        TokenKind::Number,
                        token.span.start..token.span.start + prefix_len,
                        None,
                        token.text[..prefix_len].to_string(),
                    ));
                    if prefix_len < token.text.len() {
                        projected.push(flat_token(
                            TokenKind::Text,
                            token.span.start + prefix_len..token.span.end,
                            None,
                            token.text[prefix_len..].to_string(),
                        ));
                    }
                    continue;
                }

                projected.push(flat_token(
                    TokenKind::Text,
                    token.span.clone(),
                    None,
                    token.text.clone(),
                ));
            }
        }
    }

    projected
}

fn flat_token(kind: TokenKind, span: Span, marker: Option<String>, text: String) -> FlatToken {
    FlatToken {
        id: String::new(),
        kind,
        span,
        sid: None,
        marker,
        text,
    }
}

fn merge_horizontal_whitespace(tokens: &mut Vec<FlatToken>) {
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != TokenKind::HorizontalWhitespace {
            index += 1;
            continue;
        }

        let ws = tokens[index].clone();
        if let Some(next) = tokens.get_mut(index + 1) {
            if next.kind != TokenKind::VerticalWhitespace {
                next.text = format!("{}{}", ws.text, next.text);
                next.span = ws.span.start..next.span.end;
                tokens.remove(index);
                continue;
            }
        }

        if index > 0 {
            let prev = &mut tokens[index - 1];
            prev.text.push_str(&ws.text);
            prev.span = prev.span.start..ws.span.end;
            tokens.remove(index);
            continue;
        }

        index += 1;
    }
}

fn assign_ids(tokens: &mut [FlatToken], book_code: &str) {
    for (index, token) in tokens.iter_mut().enumerate() {
        token.id = format!("{book_code}-{index}");
    }
}

fn assign_sids(tokens: &mut [FlatToken], book_code: &str) {
    let mut current_sid = format!("{book_code} 0:0");
    let mut current_chapter = 0u32;
    let mut verse_duplicate_counters = std::collections::BTreeMap::<String, usize>::new();

    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind == TokenKind::Marker {
            let marker = tokens[index].marker.as_deref().map(normalized_marker_name);
            match marker {
                Some("c") => {
                    if let Some(number) = next_number_token(tokens, index + 1)
                        && let Ok(chapter) = parse_primary_number(number.text.trim())
                    {
                        current_chapter = chapter;
                        current_sid = format!("{book_code} {current_chapter}:0");
                        verse_duplicate_counters.clear();
                    }
                }
                Some("v") => {
                    if let Some(number) = next_number_token(tokens, index + 1) {
                        let verse = number.text.trim();
                        if !verse.is_empty() {
                            let base_sid = format!("{book_code} {current_chapter}:{verse}");
                            let seen = verse_duplicate_counters
                                .entry(base_sid.clone())
                                .or_default();
                            if *seen == 0 {
                                current_sid = base_sid.clone();
                            } else {
                                current_sid = format!("{base_sid}_dup_{seen}");
                            }
                            *seen += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        tokens[index].sid = Some(current_sid.clone());
        index += 1;
    }
}

fn parse_primary_number(number: &str) -> Result<u32, std::num::ParseIntError> {
    let primary = number
        .split(['-', ','])
        .next()
        .unwrap_or(number)
        .trim_matches(|ch: char| !ch.is_ascii_digit());
    primary.parse()
}

fn next_number_token(tokens: &[FlatToken], start: usize) -> Option<&FlatToken> {
    for token in tokens.iter().skip(start) {
        match token.kind {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            TokenKind::Number => return Some(token),
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, write_exact};

    #[test]
    fn projection_ids_are_deterministic() {
        let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let projected = tokens(&handle, TokenViewOptions::default());
        assert_eq!(
            projected.first().map(|token| token.id.as_str()),
            Some("GEN-0")
        );
        assert_eq!(
            projected.get(1).map(|token| token.id.as_str()),
            Some("GEN-1")
        );
    }

    #[test]
    fn whitespace_projection_does_not_mutate_canonical_source() {
        let source = "\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n";
        let handle = parse(source);
        let merged = tokens(
            &handle,
            TokenViewOptions {
                whitespace_policy: WhitespacePolicy::MergeToVisible,
            },
        );

        assert!(
            merged
                .iter()
                .all(|token| token.kind != TokenKind::HorizontalWhitespace)
        );
        assert_eq!(write_exact(&handle), source);
    }
}
