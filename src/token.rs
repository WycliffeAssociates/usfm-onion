use serde::Serialize;

use crate::marker_defs::{
    MarkerFamily, MarkerDefKind, StructuralMarkerInfo, lookup_marker_metadata,
};

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
    Pipe,
    Marker,
    NestedMarker,
    ClosingMarker,
    NestedClosingMarker,
    Milestone,
    MilestoneEnd,
    AttributeEntry,
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
    pub kind: Option<MarkerDefKind>,
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
pub struct AttributeEntryToken<'a> {
    pub span: Span,
    pub lexeme: &'a str,
    pub key: &'a str,
    pub value: &'a str,
    /// USFM 3.1 default-attribute shorthand: bare value with no `key=`.
    /// `key` is empty when true; resolves to the marker's default attribute at serialization time.
    pub is_default: bool,
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
    Pipe(TriviaToken<'a>),
    Marker(MarkerToken<'a>),
    NestedMarker(MarkerToken<'a>),
    ClosingMarker(MarkerToken<'a>),
    NestedClosingMarker(MarkerToken<'a>),
    Milestone(MarkerToken<'a>),
    MilestoneEnd(TriviaToken<'a>),
    AttributeEntry(AttributeEntryToken<'a>),
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
            Self::Pipe(_) => ScanTokenKind::Pipe,
            Self::Marker(_) => ScanTokenKind::Marker,
            Self::NestedMarker(_) => ScanTokenKind::NestedMarker,
            Self::ClosingMarker(_) => ScanTokenKind::ClosingMarker,
            Self::NestedClosingMarker(_) => ScanTokenKind::NestedClosingMarker,
            Self::Milestone(_) => ScanTokenKind::Milestone,
            Self::MilestoneEnd(_) => ScanTokenKind::MilestoneEnd,
            Self::AttributeEntry(_) => ScanTokenKind::AttributeEntry,
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
            | Self::Pipe(token)
            | Self::MilestoneEnd(token)
            | Self::Text(token) => token.span,
            Self::Marker(token)
            | Self::NestedMarker(token)
            | Self::ClosingMarker(token)
            | Self::NestedClosingMarker(token)
            | Self::Milestone(token) => token.span,
            Self::AttributeEntry(token) => token.span,
            Self::BookCode(token) => token.span,
            Self::NumberRange(token) => token.span,
        }
    }

    pub fn lexeme(&self) -> &'a str {
        match self {
            Self::Whitespace(token)
            | Self::Newline(token)
            | Self::OptBreak(token)
            | Self::Pipe(token)
            | Self::MilestoneEnd(token)
            | Self::Text(token) => token.lexeme,
            Self::Marker(token)
            | Self::NestedMarker(token)
            | Self::ClosingMarker(token)
            | Self::NestedClosingMarker(token)
            | Self::Milestone(token) => token.lexeme,
            Self::AttributeEntry(token) => token.lexeme,
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
    if let Some((canonical, kind, family)) = lookup_marker_metadata(name) {
        MarkerMetadata {
            canonical: Some(canonical),
            kind: Some(kind),
            family,
        }
    } else {
        MarkerMetadata {
            canonical: None,
            kind: None,
            family: None,
        }
    }
}

pub type Lexeme<'a> = ScanToken<'a>;
pub type LexemeKind = ScanTokenKind;
pub type LexResult<'a> = ScanResult<'a>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TokenKind {
    Newline,
    OptBreak,
    Marker,
    EndMarker,
    Milestone,
    MilestoneEnd,
    BookCode,
    Number,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AttributeItem<'a> {
    pub span: Span,
    pub source: &'a str,
    pub key: &'a str,
    pub value: &'a str,
    /// True when the source used USFM default-attribute shorthand (bare value, no `key=`).
    /// `key` is empty in that case; consumers should expand via `marker_default_attribute`.
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum TokenData<'a> {
    Newline,
    OptBreak,
    Marker {
        name: &'a str,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        nested: bool,
        /// USFM 3.1 character-level attributes attached to this opening marker.
        /// Empty when the source had no `|...` attribute list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<AttributeItem<'a>>,
        /// Verbatim `|...` attribute-list slice plus its byte span in the source,
        /// kept so `tokens_to_usfm` can re-emit it at exactly the original
        /// position regardless of whether this marker has an explicit closer.
        /// `None` when no attribute list was present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribute_source: Option<(Span, &'a str)>,
    },
    EndMarker {
        name: &'a str,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        nested: bool,
    },
    Milestone {
        name: &'a str,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        /// USFM 3.1 attributes attached to a milestone-start (e.g. `\zaln-s |...\*`).
        /// Empty for milestone-ends and milestones without an attribute list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<AttributeItem<'a>>,
        /// Verbatim `|...` attribute-list slice plus its byte span in the source,
        /// kept so `tokens_to_usfm` can re-emit it at exactly the original position.
        /// `None` when no attribute list was present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribute_source: Option<(Span, &'a str)>,
    },
    MilestoneEnd,
    BookCode {
        code: &'a str,
        is_valid: bool,
    },
    Number {
        start: u32,
        end: Option<u32>,
        kind: NumberRangeKind,
    },
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token<'a> {
    pub id: TokenId<'a>,
    pub sid: Option<Sid>,
    pub span: Span,
    pub source: &'a str,
    #[serde(flatten)]
    pub data: TokenData<'a>,
}

impl<'a> Token<'a> {
    pub fn kind(&self) -> TokenKind {
        match self.data {
            TokenData::Newline => TokenKind::Newline,
            TokenData::OptBreak => TokenKind::OptBreak,
            TokenData::Marker { .. } => TokenKind::Marker,
            TokenData::EndMarker { .. } => TokenKind::EndMarker,
            TokenData::Milestone { .. } => TokenKind::Milestone,
            TokenData::MilestoneEnd => TokenKind::MilestoneEnd,
            TokenData::BookCode { .. } => TokenKind::BookCode,
            TokenData::Number { .. } => TokenKind::Number,
            TokenData::Text => TokenKind::Text,
        }
    }

    /// Returns the USFM 3.1 attribute list attached to this marker/milestone, if any.
    /// Returns `None` for tokens that cannot carry attributes (text, numbers, end markers, etc.).
    pub fn attributes(&self) -> Option<&[AttributeItem<'a>]> {
        match &self.data {
            TokenData::Marker { attributes, .. } | TokenData::Milestone { attributes, .. } => {
                Some(attributes.as_slice())
            }
            _ => None,
        }
    }

    pub fn marker_name(&self) -> Option<&'a str> {
        match self.data {
            TokenData::Marker { name, .. }
            | TokenData::EndMarker { name, .. }
            | TokenData::Milestone { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn to_usfm_fragment(&self) -> &'a str {
        self.source
    }
}

/// Serialize a token stream back to USFM, lossless byte-for-byte against the
/// original source.
///
/// Each marker/milestone's `attribute_source` (verbatim `|...` slice) is queued
/// with its original byte span and emitted at the moment we reach a token whose
/// own span starts at or past the attribute list's start position. This works
/// uniformly for character markers (`\w word|attr\w*`), milestones
/// (`\zaln-s |attr\*`), and paragraph-level markers (`\periph title|attr\n`)
/// without needing to know whether the marker has an explicit closer.
pub fn tokens_to_usfm(tokens: &[Token<'_>]) -> String {
    let mut output = String::new();
    let mut pending: Vec<(Span, &str)> = Vec::new();

    for token in tokens {
        // Drain any pending attribute slices whose original position came before
        // this token. `pending` stays sorted because attribute lists are queued
        // in source order during the forward walk.
        while pending
            .first()
            .is_some_and(|(span, _)| span.start <= token.span.start)
        {
            let (_, slice) = pending.remove(0);
            output.push_str(slice);
        }

        output.push_str(token.source);

        if let TokenData::Marker {
            attribute_source: Some((span, slice)),
            ..
        }
        | TokenData::Milestone {
            attribute_source: Some((span, slice)),
            ..
        } = &token.data
        {
            pending.push((*span, slice));
        }
    }

    // Any remaining pending slices live at or beyond the last token's start;
    // emit them in source order.
    pending.sort_by_key(|(span, _)| span.start);
    for (_, slice) in pending {
        output.push_str(slice);
    }

    output
}

/// Three-byte USFM book code (`GEN`, `EXO`, …) stored as raw ASCII bytes.
///
/// `Copy`-friendly and equality is a 24-bit compare. The lexer already
/// enforces 3-byte ASCII alphanumeric at the lex boundary, so any
/// `BookId` constructed via `BookId::from_str` for a parsed file is
/// guaranteed valid. Validation of *canonical* 66-book membership is
/// deliberately pushed to consumers — the type itself stays cheap.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BookId([u8; 3]);

impl BookId {
    /// Sentinel for "book code unknown" — used when materializing a
    /// `Sid` for tokens that have no surrounding `\id`.
    pub const UNKNOWN: BookId = BookId([b'?', b'?', b'?']);

    /// Build a `BookId` from a 3-byte ASCII slice. Returns `None` for
    /// any other length or non-ASCII-alphanumeric content. This mirrors
    /// the lex boundary's validation rule.
    pub fn from_str(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 3 {
            return None;
        }
        if !bytes.iter().all(|b| b.is_ascii_alphanumeric()) {
            return None;
        }
        Some(BookId([bytes[0], bytes[1], bytes[2]]))
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: BookId is always 3 ASCII alphanumeric bytes by construction.
        std::str::from_utf8(&self.0).expect("BookId is valid ASCII by construction")
    }
}

impl std::fmt::Display for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Debug for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BookId({})", self.as_str())
    }
}

impl Serialize for BookId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// Canonical scripture reference: book + chapter + verse. 8 bytes, `Copy`,
/// no lifetime. `verse == 0` means "no verse yet" (chapter-scope sid). Bridge
/// verses (`\v 1-2`) store their range end as `verse_end_delta`, a `u8` offset
/// from `verse` — the longest chapter in the Bible has 176 verses, so a
/// bridge cannot in-domain span more than 255 verses past its start.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize)]
pub struct Sid {
    pub book: BookId,
    pub chapter: u16,
    pub verse: u16,
    verse_end_delta: u8,
}

impl Sid {
    pub const fn new(book: BookId, chapter: u16, verse: u16) -> Self {
        Self {
            book,
            chapter,
            verse,
            verse_end_delta: 0,
        }
    }

    /// A verse sid spanning a range (bridge), e.g. `\v 1-2`. `verse_end`
    /// saturates at `verse + 255`: the ceiling is a stated contract (see the
    /// struct docs), not a silent truncation — callers needing the resolved
    /// end always go through [`Sid::verse_end`].
    pub fn with_range(book: BookId, chapter: u16, verse: u16, verse_end: u16) -> Self {
        let delta = verse_end.saturating_sub(verse).min(u8::MAX as u16) as u8;
        Self {
            book,
            chapter,
            verse,
            verse_end_delta: delta,
        }
    }

    /// The resolved range end (saturating). Equal to `verse` for a single verse.
    /// This is the only public accessor for the range end; the raw delta never
    /// leaks to a serialized/DTO surface.
    pub fn verse_end(&self) -> u16 {
        self.verse.saturating_add(self.verse_end_delta as u16)
    }

    /// The verse-locator fragment: `"1"` for a single verse, `"1-2"` for a range.
    pub fn verse_locator(&self) -> String {
        if self.verse_end_delta == 0 {
            self.verse.to_string()
        } else {
            format!("{}-{}", self.verse, self.verse_end())
        }
    }
}

impl std::fmt::Display for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.verse == 0 {
            write!(f, "{} {}", self.book, self.chapter)
        } else {
            write!(f, "{} {}:{}", self.book, self.chapter, self.verse_locator())
        }
    }
}

impl std::fmt::Debug for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sid({self})")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TokenId<'a> {
    pub book_code: &'a str,
    pub index: u32,
}

impl<'a> TokenId<'a> {
    pub const fn new(book_code: &'a str, index: u32) -> Self {
        Self { book_code, index }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ParseAnalysis<'a> {
    pub book_code: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParseResult<'a> {
    pub tokens: Vec<Token<'a>>,
    pub analysis: ParseAnalysis<'a>,
}

#[cfg(test)]
mod sid_size_guard {
    use super::{BookId, Sid};
    use std::mem::size_of;

    #[test]
    fn sid_stays_pointer_sized() {
        // Sid is the hot path for every chapter/verse the walker tracks.
        // Growing it past 8 bytes (e.g. by adding a heap field) would
        // turn it into a pointer-equivalent and reintroduce the cost
        // the sous-chef pattern was chosen to avoid.
        assert!(
            size_of::<Sid>() <= 8,
            "Sid grew to {} bytes — keep it Copy-cheap",
            size_of::<Sid>()
        );
        assert_eq!(size_of::<BookId>(), 3);
    }

    #[test]
    fn verse_end_saturates_past_255_span() {
        // No versification can produce a bridge wider than 255 verses (the
        // longest chapter in the Bible has 176 verses), so this is a stated
        // ceiling, not a silent truncation. `\v 1-999` pins the behavior.
        let book = BookId::from_str("GEN").unwrap();
        let sid = Sid::with_range(book, 1, 1, 999);
        assert_eq!(sid.verse_end(), 1 + 255);
        assert_eq!(sid.verse_locator(), "1-256");
    }

    #[test]
    fn verse_locator_is_bare_for_a_single_verse() {
        let book = BookId::from_str("GEN").unwrap();
        let sid = Sid::new(book, 1, 1);
        assert_eq!(sid.verse_locator(), "1");
        assert_eq!(sid.to_string(), "GEN 1:1");
    }

    #[test]
    fn display_adds_range_end_when_present() {
        let book = BookId::from_str("GEN").unwrap();
        let sid = Sid::with_range(book, 1, 1, 2);
        assert_eq!(sid.to_string(), "GEN 1:1-2");
    }
}
