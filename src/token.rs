use serde::Serialize;

use crate::marker_defs::{MarkerFamily, SpecMarkerKind, lookup_marker_def};

pub type BytePos = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub const fn new(start: BytePos, end: BytePos) -> Self {
        Self { start, end }
    }

    pub fn as_range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    BookCode,
    NumberRange,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TriviaToken<'a> {
    pub span: Span,
    pub lexeme: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerMetadata {
    pub canonical: Option<&'static str>,
    pub kind: Option<SpecMarkerKind>,
    pub family: Option<MarkerFamily>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerToken<'a> {
    pub span: Span,
    pub lexeme: &'a str,
    pub name: &'a str,
    pub metadata: MarkerMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BookCodeToken<'a> {
    pub span: Span,
    pub lexeme: &'a str,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NumberRangeKind {
    Single,
    Range,
    Sequence,
    SequenceWithRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NumberRangeToken<'a> {
    pub span: Span,
    pub lexeme: &'a str,
    pub start: u32,
    pub end: Option<u32>,
    pub kind: NumberRangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScanToken<'a> {
    Whitespace(TriviaToken<'a>),
    Newline(TriviaToken<'a>),
    OptBreak(TriviaToken<'a>),
    Marker(MarkerToken<'a>),
    NestedMarker(MarkerToken<'a>),
    ClosingMarker(MarkerToken<'a>),
    NestedClosingMarker(MarkerToken<'a>),
    Milestone(MarkerToken<'a>),
    MilestoneEnd(TriviaToken<'a>),
    Attributes(TriviaToken<'a>),
    BookCode(BookCodeToken<'a>),
    NumberRange(NumberRangeToken<'a>),
    Text(TriviaToken<'a>),
}

impl<'a> ScanToken<'a> {
    pub fn kind(&self) -> ScanTokenKind {
        match self {
            Self::Whitespace(_) => ScanTokenKind::Whitespace,
            Self::Newline(_) => ScanTokenKind::Newline,
            Self::OptBreak(_) => ScanTokenKind::OptBreak,
            Self::Marker(_) => ScanTokenKind::Marker,
            Self::NestedMarker(_) => ScanTokenKind::NestedMarker,
            Self::ClosingMarker(_) => ScanTokenKind::ClosingMarker,
            Self::NestedClosingMarker(_) => ScanTokenKind::NestedClosingMarker,
            Self::Milestone(_) => ScanTokenKind::Milestone,
            Self::MilestoneEnd(_) => ScanTokenKind::MilestoneEnd,
            Self::Attributes(_) => ScanTokenKind::Attributes,
            Self::BookCode(_) => ScanTokenKind::BookCode,
            Self::NumberRange(_) => ScanTokenKind::NumberRange,
            Self::Text(_) => ScanTokenKind::Text,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Whitespace(token)
            | Self::Newline(token)
            | Self::OptBreak(token)
            | Self::MilestoneEnd(token)
            | Self::Attributes(token)
            | Self::Text(token) => token.span,
            Self::Marker(token)
            | Self::NestedMarker(token)
            | Self::ClosingMarker(token)
            | Self::NestedClosingMarker(token)
            | Self::Milestone(token) => token.span,
            Self::BookCode(token) => token.span,
            Self::NumberRange(token) => token.span,
        }
    }

    pub fn lexeme(&self) -> &'a str {
        match self {
            Self::Whitespace(token)
            | Self::Newline(token)
            | Self::OptBreak(token)
            | Self::MilestoneEnd(token)
            | Self::Attributes(token)
            | Self::Text(token) => token.lexeme,
            Self::Marker(token)
            | Self::NestedMarker(token)
            | Self::ClosingMarker(token)
            | Self::NestedClosingMarker(token)
            | Self::Milestone(token) => token.lexeme,
            Self::BookCode(token) => token.lexeme,
            Self::NumberRange(token) => token.lexeme,
        }
    }

    pub fn marker_metadata(&self) -> Option<MarkerMetadata> {
        match self {
            Self::Marker(token)
            | Self::NestedMarker(token)
            | Self::ClosingMarker(token)
            | Self::NestedClosingMarker(token)
            | Self::Milestone(token) => Some(token.metadata),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanResult<'a> {
    pub tokens: Vec<ScanToken<'a>>,
}

pub fn strip_marker_backslash(marker: &str) -> &str {
    marker.strip_prefix('\\').unwrap_or(marker)
}

pub fn strip_closing_star(marker: &str) -> &str {
    let s = strip_marker_backslash(marker);
    s.strip_suffix('*').unwrap_or(s)
}

pub fn marker_text_name(kind: ScanTokenKind, lexeme: &str) -> &str {
    match kind {
        ScanTokenKind::ClosingMarker | ScanTokenKind::NestedClosingMarker => {
            strip_closing_star(lexeme)
        }
        ScanTokenKind::Marker | ScanTokenKind::NestedMarker | ScanTokenKind::Milestone => {
            strip_marker_backslash(lexeme)
        }
        _ => lexeme,
    }
}

pub fn marker_metadata(name: &str) -> MarkerMetadata {
    if let Some(def) = lookup_marker_def(name) {
        MarkerMetadata {
            canonical: Some(def.marker),
            kind: Some(def.kind),
            family: def.family,
        }
    } else {
        MarkerMetadata {
            canonical: None,
            kind: None,
            family: None,
        }
    }
}
