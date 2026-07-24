use serde::Serialize;

use crate::marker_defs::{
    MarkerDefKind, MarkerFamily, MarkerIndex, StructuralMarkerInfo, resolve_marker_metadata,
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
    /// Dense marker-catalog handle resolved once by the lexer (WS2B), so the
    /// parser's per-occurrence structural/delimiter-absorption lookups are
    /// array indexing instead of a string-hashmap probe. Internal perf
    /// handle only — never part of the serialized contract, hence the skip:
    /// its numeric value is not stable across builds and downstream
    /// consumers (lint, export) must keep reading `canonical`/`kind`/
    /// `family`, not this field.
    #[serde(skip)]
    pub(crate) index: MarkerIndex,
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
    let (canonical, kind, family, index) = resolve_marker_metadata(name);
    MarkerMetadata {
        canonical,
        kind,
        family,
        index,
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

/// USFM 3.1 attribute payload for a `Marker`/`Milestone` opener. Boxed and
/// made optional on the token so the common no-attribute case pays no cost:
/// attribute lists are rare, but the two fields below were previously present
/// (empty/`None`) on every token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkerAttrs<'a> {
    /// USFM 3.1 character-level attributes attached to the opening marker.
    /// Empty when the source had no `|...` attribute list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<AttributeItem<'a>>,
    /// Verbatim `|...` attribute-list slice plus its byte span in the source,
    /// kept so `tokens_to_usfm` can re-emit it at exactly the original
    /// position regardless of whether this marker has an explicit closer.
    /// `None` when no attribute list was present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute_source: Option<(Span, &'a str)>,
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
        /// Attribute list attached to this opening marker. `None` when the
        /// source had no `|...` attribute list — the common case, kept cheap
        /// by boxing so it costs a pointer-sized `None` instead of an empty
        /// `Vec` + `Option` pair on every token.
        #[serde(flatten)]
        attrs: Option<Box<MarkerAttrs<'a>>>,
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
        /// Attribute list attached to a milestone-start (e.g. `\zaln-s |...\*`).
        /// `None` for milestone-ends and milestones without an attribute list.
        #[serde(flatten)]
        attrs: Option<Box<MarkerAttrs<'a>>>,
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
            TokenData::Marker { attrs, .. } | TokenData::Milestone { attrs, .. } => Some(
                attrs
                    .as_deref()
                    .map(|a| a.attributes.as_slice())
                    .unwrap_or(&[]),
            ),
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

    /// The dense marker-catalog handle the lexer stamped on this marker
    /// (WS2B), or [`MarkerIndex::UNKNOWN`] for non-marker tokens. Lets marker
    /// consumers (e.g. lint's context check) drive index-keyed catalog lookups
    /// off the already-resolved handle instead of re-hashing the marker name.
    pub(crate) fn marker_index(&self) -> MarkerIndex {
        match self.data {
            TokenData::Marker { metadata, .. }
            | TokenData::EndMarker { metadata, .. }
            | TokenData::Milestone { metadata, .. } => metadata.index,
            _ => MarkerIndex::UNKNOWN,
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
/// without needing to know whether the marker has an explicit closer — even a
/// malformed/unclosed marker's attribute list lands at its original byte
/// position, because this algorithm only needs span order, never a closer
/// match. That span-based guarantee is native `Token`'s only; see
/// [`tokens_to_usfm_reconstruct`] for the spanless equivalent owned/editor
/// tokens use, and why the two aren't interchangeable.
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
            attrs: Some(attrs), ..
        }
        | TokenData::Milestone {
            attrs: Some(attrs), ..
        } = &token.data
            && let Some((span, slice)) = attrs.attribute_source
        {
            pending.push((span, slice));
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

/// Read-only view of an attribute item, generic over the borrowed native
/// representation ([`AttributeItem`], `&str` fields) and owned/wire
/// representations (`String` fields) so [`format_attribute_list`] can serve
/// both without a shared concrete struct across the wire boundary.
pub trait SerializableAttribute {
    fn key(&self) -> &str;
    fn value(&self) -> &str;
    fn is_default(&self) -> bool;
}

impl<'a> SerializableAttribute for AttributeItem<'a> {
    fn key(&self) -> &str {
        self.key
    }

    fn value(&self) -> &str {
        self.value
    }

    fn is_default(&self) -> bool {
        self.is_default
    }
}

/// The minimal contract a token needs to satisfy for lossless token-stream
/// serialization back to USFM. Implemented by native [`Token`] (for
/// contract-reference and parity testing against [`tokens_to_usfm`]) and by
/// owned/editor-authored token representations at the wire boundary, which
/// have no reliable byte spans and so serialize via [`tokens_to_usfm_reconstruct`]
/// instead of the span-based [`tokens_to_usfm`].
///
/// # Five methods
///
/// - `kind()` / `marker()` — drive attribute placement: character markers
///   attach before `\name*`, paragraph markers before the next newline,
///   milestones before their own close.
/// - `source()` — the raw marker + text bytes, **excluding** the `|...`
///   attribute list (that's carried separately, see below). Must stay
///   verbatim (exact whitespace, exact backslash) for byte-losslessness.
/// - `attributes()` — the structured attribute list (`key`/`value`/
///   `is_default`). Used for reading/semantics and for *authoring* new
///   attributes.
/// - `attribute_list()` — the verbatim `|...` slice, when one exists. This is
///   what makes a *structured* token losslessly serializable without the
///   caller ever reconstructing a pipe by hand.
///
/// # Two contracts, not one
///
/// A consumer that can only emit USFM text needs nothing but that: emit ->
/// `parse(&str)` -> full fidelity, attributes inline in the string. That's
/// the **string floor**, always sufficient, and the fallback whenever the
/// token path is inconvenient. `SerializableToken` is the **token path** — the
/// smaller, optional fast path for a consumer that can hand back tokens
/// satisfying this trait instead of paying for a re-parse.
///
/// # Verbatim preferred, reconstruct only for authored attributes
///
/// The emitter always prefers `attribute_list()` when it's `Some`: the
/// original bytes (whitespace, quote style, encoding) ride through untouched.
/// It reconstructs from `attributes()` only when `attribute_list()` is
/// `None` — the editor-authored case, where canonical formatting is correct
/// because there's no original trivia to preserve.
///
/// This is the one rule an editor needs: **touch an attribute, drop its
/// verbatim; onion re-serializes the whole list from structure.** There is
/// no partial/per-item verbatim-preservation — editing one attribute in a
/// list means the entire list is reconstructed, not just the changed entry.
/// An editor that never touches attributes never has to think about this.
///
/// # The one sanctioned divergence between the two emitters
///
/// [`tokens_to_usfm`] (native, span-based) and [`tokens_to_usfm_reconstruct`]
/// (this trait, spanless) agree on every well-formed input, but not on
/// malformed ones: a marker with an attribute list that is never closed (a
/// parser-recovery scenario) keeps its exact byte offset under span-drain,
/// while the spanless/closer-shape emitter — having no matching closer to
/// trigger the drain — pushes it to the end of the token stream instead.
/// Native callers get byte-exact recovery; owned/wire tokens do not, because
/// they have no span to recover a position from.
pub trait SerializableToken {
    type Attr: SerializableAttribute;

    fn kind(&self) -> TokenKind;
    fn marker(&self) -> Option<&str>;
    fn source(&self) -> &str;
    fn attributes(&self) -> &[Self::Attr];
    fn attribute_list(&self) -> Option<&str>;
}

impl<'a> SerializableToken for Token<'a> {
    type Attr = AttributeItem<'a>;

    fn kind(&self) -> TokenKind {
        Token::kind(self)
    }

    fn marker(&self) -> Option<&str> {
        self.marker_name()
    }

    fn source(&self) -> &str {
        self.source
    }

    fn attributes(&self) -> &[Self::Attr] {
        Token::attributes(self).unwrap_or(&[])
    }

    fn attribute_list(&self) -> Option<&str> {
        match &self.data {
            TokenData::Marker {
                attrs: Some(attrs), ..
            }
            | TokenData::Milestone {
                attrs: Some(attrs), ..
            } => attrs.attribute_source.map(|(_, slice)| slice),
            _ => None,
        }
    }
}

/// Re-escapes a logical attribute value for USFM emit. Inverse of the wire
/// boundary's attribute-value decode: `\` → `\\`, `"` → `\"`. Native
/// `AttributeItem::value` is already raw/still-encoded, so this only ever
/// runs on the reconstruct path (`attribute_list()` is `None`) — encoding an
/// already-encoded native value would double-encode it, which is why the
/// verbatim slice always wins when present.
pub fn encode_attr_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

/// Renders a parsed attribute list as a `|key="value" ...` slice. The leading
/// `|` is included; no trailing whitespace. `is_default` items are emitted as
/// the bare value (USFM 3.1 default-attribute shorthand). Used only when a
/// token has no verbatim `attribute_list()` to emit instead (editor-authored
/// attributes).
pub fn format_attribute_list<A: SerializableAttribute>(attrs: &[A]) -> String {
    let mut out = String::from("|");
    for (i, item) in attrs.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        if item.is_default() {
            out.push_str(&encode_attr_value(item.value()));
        } else {
            out.push_str(item.key());
            out.push_str("=\"");
            out.push_str(&encode_attr_value(item.value()));
            out.push('"');
        }
    }
    out
}

#[derive(Clone, Copy)]
enum CloserShape {
    MatchingEndMarker,
    MilestoneEnd,
    ParagraphBoundary,
}

/// Decide how an attribute-bearing marker is closed. Milestones use the
/// token kind (authoritative — the lexer/parser already classified the
/// source as a `Milestone` token). For non-milestone openers we consult
/// the marker catalog by name to distinguish paragraph-style markers
/// (drain before next newline) from character-style markers (drain
/// before matching `\name*` close).
fn closer_shape(token_kind: TokenKind, marker_name: &str) -> CloserShape {
    if matches!(token_kind, TokenKind::Milestone) {
        return CloserShape::MilestoneEnd;
    }
    match crate::marker_defs::lookup_marker_metadata(marker_name).map(|(_, kind, _)| kind) {
        Some(
            MarkerDefKind::Paragraph
            | MarkerDefKind::Periph
            | MarkerDefKind::Header
            | MarkerDefKind::TableRow,
        ) => CloserShape::ParagraphBoundary,
        // Character, Note, Figure, TableCell, Chapter, Verse, Sidebar,
        // Meta, and unknown markers all use the "matching EndMarker"
        // rule. Unknown markers default to character behavior so a
        // round-trip of unrecognized custom markers still positions
        // their attribute lists correctly relative to a `\name*` close.
        _ => CloserShape::MatchingEndMarker,
    }
}

enum PendingAttrs<'t, T: SerializableToken + ?Sized> {
    /// The original `|...` slice, emitted byte-for-byte.
    Verbatim(&'t str),
    /// No verbatim slice available (editor-authored) — reconstruct from
    /// the structured attribute list.
    Structured(&'t [T::Attr]),
}

struct Pending<'t, T: SerializableToken + ?Sized> {
    marker_name: String,
    shape: CloserShape,
    attrs: PendingAttrs<'t, T>,
}

fn is_paragraph_marker<T: SerializableToken>(token: &T) -> bool {
    if !matches!(token.kind(), TokenKind::Marker) {
        return false;
    }
    token
        .marker()
        .map(|name| matches!(closer_shape(TokenKind::Marker, name), CloserShape::ParagraphBoundary))
        .unwrap_or(false)
}

fn token_closes<T: SerializableToken>(pending: &Pending<'_, T>, token: &T) -> bool {
    match pending.shape {
        CloserShape::MatchingEndMarker => {
            matches!(token.kind(), TokenKind::EndMarker)
                && token.marker() == Some(pending.marker_name.as_str())
        }
        CloserShape::MilestoneEnd => matches!(token.kind(), TokenKind::MilestoneEnd),
        CloserShape::ParagraphBoundary => {
            matches!(token.kind(), TokenKind::Newline) || is_paragraph_marker(token)
        }
    }
}

fn emit_pending<T: SerializableToken>(output: &mut String, attrs: PendingAttrs<'_, T>) {
    match attrs {
        PendingAttrs::Verbatim(slice) => output.push_str(slice),
        PendingAttrs::Structured(attrs) => output.push_str(&format_attribute_list(attrs)),
    }
}

/// Serialize a [`SerializableToken`] stream back to USFM without relying on
/// byte spans — the emitter owned/editor-authored tokens use, since they
/// have no reliable span to drive [`tokens_to_usfm`]'s span-drain algorithm.
///
/// Each attribute-bearing marker/milestone is pushed onto a LIFO stack of
/// pending attribute lists on encounter; before emitting each subsequent
/// token we drain any pending entries that token closes (matching
/// `EndMarker`, `MilestoneEnd`, or a paragraph boundary — see
/// [`CloserShape`](enum@CloserShape)). Any pending lists remaining at
/// end-of-stream are flushed in LIFO order. A pending entry emits its
/// verbatim `attribute_list()` slice when present, else reconstructs from
/// `attributes()`.
///
/// Agrees with [`tokens_to_usfm`] on every well-formed input (see the
/// `tokens_to_usfm_reconstruct_parity` test). The one sanctioned divergence:
/// a malformed/unclosed attribute-bearing marker's attribute list lands at
/// end-of-stream here (no closer ever arrives to trigger the drain) instead
/// of at its original byte position — a real difference from `tokens_to_usfm`,
/// but the same recovery behavior the previous wasm-side emitter already had.
pub fn tokens_to_usfm_reconstruct<T: SerializableToken>(tokens: &[T]) -> String {
    let mut output = String::new();
    let mut pending: Vec<Pending<'_, T>> = Vec::new();

    for token in tokens {
        while let Some(top) = pending.last() {
            if token_closes(top, token) {
                let drained = pending.pop().unwrap();
                emit_pending(&mut output, drained.attrs);
            } else {
                break;
            }
        }

        output.push_str(token.source());

        let has_attrs = token.attribute_list().is_some() || !token.attributes().is_empty();
        if matches!(token.kind(), TokenKind::Marker | TokenKind::Milestone)
            && has_attrs
            && let Some(name) = token.marker()
        {
            let attrs = match token.attribute_list() {
                Some(slice) => PendingAttrs::Verbatim(slice),
                None => PendingAttrs::Structured(token.attributes()),
            };
            pending.push(Pending {
                marker_name: name.to_string(),
                shape: closer_shape(token.kind(), name),
                attrs,
            });
        }
    }

    while let Some(drained) = pending.pop() {
        emit_pending(&mut output, drained.attrs);
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

/// Frozen oracle: `parse(source) -> tokens_to_usfm(tokens)` must reproduce
/// `source` byte-for-byte. These pass against today's span-drain emitter
/// (`tokens_to_usfm`) and pin its behavior before the emitter is rewritten
/// to a generic, `closer_shape`-based one — any future rewrite must keep
/// every one of these byte-identical, including the non-canonical-whitespace
/// case, which only a verbatim-slice emitter (not a reconstruct-from-parts
/// one) can reproduce.
#[cfg(test)]
mod tokens_to_usfm_round_trip {
    use super::tokens_to_usfm;
    use crate::parse::parse;

    fn round_trip(source: &str) -> String {
        let tokens = parse(source).tokens;
        tokens_to_usfm(&tokens)
    }

    #[test]
    fn default_attribute_shorthand() {
        let source = "\\w gracious|grace\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn embedded_escaped_quote_in_attribute_value() {
        let source = "\\w word|note=\"a\\\"b\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn multiple_attributes() {
        let source = "\\w word|lemma=\"x\" strong=\"H0430\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn milestone_with_attributes() {
        let source = "\\zaln-s |x-strong=\"H0430\"\\*word\\zaln-e\\*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn non_canonical_attribute_list_whitespace() {
        // Extra inter-item spaces and spaced-out `=` — only a verbatim-slice
        // emitter preserves this; a reconstruct-from-parts emitter would
        // normalize to canonical `key="value"` spacing.
        let source = "\\w x|lemma = \"y\"\\w*";
        assert_eq!(round_trip(source), source);
    }
}

/// Parity between the two emitters: [`tokens_to_usfm`] (span-drain, native
/// `Token` only) and [`tokens_to_usfm_reconstruct`] (spanless, closer-shape
/// — the one owned/wire tokens must use, since they carry no reliable span).
/// The two algorithms are NOT interchangeable in general — span-drain places
/// a malformed/unclosed marker's attribute list at its original byte offset,
/// while closer-shape can only place it once a matching closer is seen and
/// falls back to end-of-stream otherwise (see `tokens_to_usfm_reconstruct`'s
/// doc comment) — but they must agree on every WELL-FORMED input, which is
/// what this suite pins.
#[cfg(test)]
mod tokens_to_usfm_reconstruct_parity {
    use super::{tokens_to_usfm, tokens_to_usfm_reconstruct};
    use crate::parse::parse;

    fn assert_parity(source: &str) {
        let tokens = parse(source).tokens;
        assert_eq!(
            tokens_to_usfm_reconstruct(&tokens),
            tokens_to_usfm(&tokens),
            "reconstruct/span-drain emitters diverged for well-formed input {source:?}"
        );
    }

    #[test]
    fn agrees_on_well_formed_gate_0_corpus() {
        for source in [
            "\\w gracious|grace\\w*",
            "\\w word|note=\"a\\\"b\"\\w*",
            "\\w word|lemma=\"x\" strong=\"H0430\"\\w*",
            "\\zaln-s |x-strong=\"H0430\"\\*word\\zaln-e\\*",
            "\\w x|lemma = \"y\"\\w*",
            // Attribute-bearing marker nested inside another marker — the
            // inner `\nd*` closer must be matched by name, not confused
            // with the outer `\w*`.
            "\\w \\nd x|k=\"v\"\\nd* y\\w*",
            // Paragraph-level attribute list (closer = next newline).
            "\\periph title|periph=\"Title Page\"\n",
        ] {
            assert_parity(source);
        }
    }
}

#[cfg(test)]
mod encode_attr_value_tests {
    use super::encode_attr_value;

    #[test]
    fn escapes_quote_and_backslash() {
        assert_eq!(encode_attr_value("plain"), "plain");
        assert_eq!(encode_attr_value("a\"b"), "a\\\"b");
        assert_eq!(encode_attr_value("a\\b"), "a\\\\b");
    }
}
