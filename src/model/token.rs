use std::ops::Range;

use serde::{Deserialize, Serialize};

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanTokenKind {
    Whitespace,
    Newline,
    OptBreak,
    Marker,
    NestedMarker,
    ClosingMarker,
    NestedClosingMarker,
    Milestone,
    MilestoneEnd,
    Attributes,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanToken {
    pub kind: ScanTokenKind,
    pub span: Span,
    pub text: String,
}

impl ScanToken {
    pub fn marker_name(&self) -> Option<&str> {
        match self.kind {
            ScanTokenKind::Marker | ScanTokenKind::NestedMarker | ScanTokenKind::Milestone => {
                Some(strip_marker_backslash(&self.text))
            }
            ScanTokenKind::ClosingMarker | ScanTokenKind::NestedClosingMarker => {
                Some(strip_closing_star(&self.text))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub tokens: Vec<ScanToken>,
}

pub trait SourceTokenText {
    fn source_text(&self) -> &str;
}

impl SourceTokenText for ScanToken {
    fn source_text(&self) -> &str {
        self.text.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    Whitespace,
    Newline,
    OptBreak,
    Marker,
    EndMarker,
    Milestone,
    MilestoneEnd,
    Attributes,
    BookCode,
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatToken {
    pub id: String,
    pub kind: TokenKind,
    pub span: Span,
    pub sid: Option<String>,
    pub marker: Option<String>,
    pub text: String,
}

impl SourceTokenText for FlatToken {
    fn source_text(&self) -> &str {
        self.text.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WhitespacePolicy {
    #[default]
    Preserve,
    MergeToVisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TokenViewOptions {
    pub whitespace_policy: WhitespacePolicy,
}

pub(crate) fn strip_marker_backslash(marker: &str) -> &str {
    marker.strip_prefix('\\').unwrap_or(marker)
}

pub(crate) fn strip_closing_star(marker: &str) -> &str {
    let s = strip_marker_backslash(marker);
    s.strip_suffix('*').unwrap_or(s)
}

pub(crate) fn normalized_marker_name(marker: &str) -> &str {
    marker.strip_prefix('+').unwrap_or(marker)
}

pub(crate) fn number_prefix_len(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let leading_ws = text.len() - trimmed.len();
    let mut len = 0usize;
    let mut seen_digit = false;
    let mut prev_was_sep = false;

    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            prev_was_sep = false;
            len += ch.len_utf8();
            continue;
        }
        if ch == '-' || ch == ',' {
            if !seen_digit || prev_was_sep {
                break;
            }
            prev_was_sep = true;
            len += ch.len_utf8();
            continue;
        }
        if ch == '"' && seen_digit {
            len += ch.len_utf8();
            break;
        }
        if ch.is_ascii_alphabetic() {
            prev_was_sep = false;
            len += ch.len_utf8();
            continue;
        }
        break;
    }

    if seen_digit && !prev_was_sep {
        Some(leading_ws + len)
    } else {
        None
    }
}
