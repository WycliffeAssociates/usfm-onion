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
pub struct Token {
    pub id: String,
    pub kind: TokenKind,
    pub span: Span,
    pub sid: Option<String>,
    pub marker: Option<String>,
    pub text: String,
}

impl SourceTokenText for Token {
    fn source_text(&self) -> &str {
        self.text.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TokenVariant {
    Newline {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
    OptBreak {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
    Marker {
        id: String,
        span: Span,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    EndMarker {
        id: String,
        span: Span,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    Milestone {
        id: String,
        span: Span,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    MilestoneEnd {
        id: String,
        span: Span,
        sid: Option<String>,
        marker: Option<String>,
        text: String,
    },
    Attributes {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
    BookCode {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
    Number {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
    Text {
        id: String,
        span: Span,
        sid: Option<String>,
        text: String,
    },
}

impl Token {
    pub fn variant(&self) -> TokenVariant {
        let id = self.id.clone();
        let span = self.span.clone();
        let sid = self.sid.clone();
        let text = self.text.clone();

        match self.kind {
            TokenKind::Newline => TokenVariant::Newline {
                id,
                span,
                sid,
                text,
            },
            TokenKind::OptBreak => TokenVariant::OptBreak {
                id,
                span,
                sid,
                text,
            },
            TokenKind::Marker => TokenVariant::Marker {
                id,
                span,
                sid,
                marker: self.marker.clone().unwrap_or_default(),
                text,
            },
            TokenKind::EndMarker => TokenVariant::EndMarker {
                id,
                span,
                sid,
                marker: self.marker.clone().unwrap_or_default(),
                text,
            },
            TokenKind::Milestone => TokenVariant::Milestone {
                id,
                span,
                sid,
                marker: self.marker.clone().unwrap_or_default(),
                text,
            },
            TokenKind::MilestoneEnd => TokenVariant::MilestoneEnd {
                id,
                span,
                sid,
                marker: self.marker.clone(),
                text,
            },
            TokenKind::Attributes => TokenVariant::Attributes {
                id,
                span,
                sid,
                text,
            },
            TokenKind::BookCode => TokenVariant::BookCode {
                id,
                span,
                sid,
                text,
            },
            TokenKind::Number => TokenVariant::Number {
                id,
                span,
                sid,
                text,
            },
            TokenKind::Text => TokenVariant::Text {
                id,
                span,
                sid,
                text,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WhitespacePolicy {
    #[default]
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
