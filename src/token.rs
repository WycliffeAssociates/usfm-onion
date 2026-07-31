use serde::{Deserialize, Serialize};

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

/// `Deserialize` as well as `Serialize`: a token kind travels back *into* the
/// library on the resident boundary (a patch's replacement template, a native
/// host's IPC payload), not only out of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Opaque identifier supplied by a resident token source.
///
/// Identity is stable only within one book. Onion compares it byte-for-byte
/// but does not assign meaning to its contents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableTokenId(Box<str>);

impl StableTokenId {
    /// Returns `None` for an empty identifier, which cannot address a token.
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for StableTokenId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for StableTokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One USFM attribute entry retained with its source spelling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedAttribute {
    pub source: Box<str>,
    pub key: Box<str>,
    pub value: Box<str>,
    pub is_default: bool,
    /// Byte span in the source this attribute was parsed from. `None` for an
    /// attribute an editor synthesized or structurally edited — never
    /// fabricated from some other token's span, since that would misreport a
    /// position the attribute never actually occupied.
    pub span: Option<Span>,
}

/// Parsed number payload carried by a `TokenKind::Number` token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedNumberInfo {
    pub start: u32,
    pub end: Option<u32>,
    pub kind: NumberRangeKind,
}

/// Parsed book-code payload carried by a `TokenKind::BookCode` token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedBookCode {
    pub code: Box<str>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedMarkerAttrs {
    attributes: Box<[OwnedAttribute]>,
    attribute_source: Option<Box<str>>,
    /// Bytes from the end of the owning token's own source to the start of the
    /// attribute list, in the stream this token came from.
    ///
    /// The verbatim attribute text always survived the drop to owned tokens; its
    /// *position* did not, and one placement rule cannot express every real
    /// layout — an alignment list sits at the opener, a wordlist list can sit
    /// past a nested closer, and an unclosed `\fig`'s list sits in the middle of
    /// the following text. A distance from the owner is the smallest fact that
    /// covers all of them, and unlike an absolute offset it stays meaningful
    /// after tokens elsewhere in the stream are edited.
    ///
    /// `None` for tokens built without positions (editor- or DTO-authored): those
    /// never had a position to remember, and the emitter falls back to placing
    /// the list at the marker's closer.
    attribute_offset: Option<BytePos>,
}

/// The payload variant follows `kind` exactly. Keeping it private prevents a
/// boundary caller from creating impossible marker/attribute combinations.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OwnedTokenPayload {
    Plain,
    Marker {
        marker: Box<str>,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        nested: bool,
        attrs: Option<OwnedMarkerAttrs>,
    },
    EndMarker {
        marker: Box<str>,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        nested: bool,
    },
    Milestone {
        marker: Box<str>,
        metadata: MarkerMetadata,
        structural: StructuralMarkerInfo,
        attrs: Option<OwnedMarkerAttrs>,
    },
    BookCode(OwnedBookCode),
    Number(OwnedNumberInfo),
}

/// Owned semantic token for token streams that outlive their parsed source.
///
/// Parsed tokens borrow source text and carry byte spans; this type retains
/// the semantic payload and any verbatim attribute list needed to emit the
/// token stream without retaining the original source buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedToken {
    id: StableTokenId,
    kind: TokenKind,
    source: Box<str>,
    sid: Option<Box<str>>,
    parsed_sid: Option<Sid>,
    payload: OwnedTokenPayload,
}

/// Why a working token could not become an owned resident token.
///
/// Every variant names a payload fact the working shape
/// ([`crate::format::FormattableToken`]) does not carry and that this token
/// cannot be given honestly — so the conversion refuses instead of inventing
/// one. All three are checkpoint failures at a residency boundary, never
/// something a well-formed format or fix pass produces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenBuildError {
    /// The working token carries no id, so nothing could address it.
    MissingId,
    /// This kind's payload (a book code, a parsed number) is not reconstructible
    /// from the working shape, and no anchor supplied it either.
    MissingPayload { id: Box<str>, kind: TokenKind },
    /// The working token names a canonical anchor no anchor token can supply the
    /// structured form of. Keeping the formatted string without it would leave a
    /// token whose spelling and structured anchor disagree.
    UnresolvableSid { id: Box<str>, sid: Box<str> },
}

impl std::fmt::Display for TokenBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingId => f.write_str("working token carries no id"),
            Self::MissingPayload { id, kind } => {
                write!(
                    f,
                    "token {id} of kind {kind:?} has no reconstructible payload"
                )
            }
            Self::UnresolvableSid { id, sid } => {
                write!(
                    f,
                    "token {id} names the anchor {sid} with no structured form"
                )
            }
        }
    }
}

impl std::error::Error for TokenBuildError {}

impl OwnedToken {
    /// Rebuilds an owned resident token from a format/fix pass's working token.
    ///
    /// This is the residency checkpoint for the id-optional working type: a
    /// token with no id cannot be addressed by anything downstream, so it is
    /// refused here rather than stored unaddressable.
    ///
    /// `anchor` is the resident token this working token descends from — the
    /// same-id token for one the pass modified, or the fix's target for one it
    /// synthesized. It supplies exactly the payload facts a working token has no
    /// field for: marker nesting, a book code, a parsed number, an attribute
    /// list's remembered placement, and the structured form of a canonical
    /// anchor. Passing `None` states there is no such predecessor, and then a
    /// kind that needs one of those facts is refused rather than guessed.
    ///
    /// Facts the working token *does* carry always win: kind, source text,
    /// marker name, and the formatted anchor are read from it, never from the
    /// anchor token.
    pub fn from_format_token(
        token: &crate::format::FormatToken,
        anchor: Option<&Self>,
    ) -> Result<Self, TokenBuildError> {
        // Field access, not the `FormattableToken` accessors: `FormatToken`
        // implements two traits that both declare `marker`, so the fields are
        // the unambiguous reading.
        let id = StableTokenId::new(token.id.clone().ok_or(TokenBuildError::MissingId)?)
            .ok_or(TokenBuildError::MissingId)?;
        let kind = token.kind;
        let missing_payload = || TokenBuildError::MissingPayload {
            id: Box::from(id.as_str()),
            kind,
        };

        // The formatted spelling comes from the working token; its structured
        // form can only come from an anchor that spells the same anchor, since
        // re-parsing the display string here would fork core's own formatting.
        let (sid, parsed_sid) = match token.sid.as_deref() {
            None => (None, None),
            Some(text) => {
                let inherited = anchor
                    .filter(|anchor| anchor.sid() == Some(text))
                    .and_then(|anchor| anchor.parsed_sid);
                match inherited {
                    Some(parsed) => (Some(Box::from(text)), Some(parsed)),
                    None => {
                        return Err(TokenBuildError::UnresolvableSid {
                            id: Box::from(id.as_str()),
                            sid: Box::from(text),
                        });
                    }
                }
            }
        };

        let payload = match kind {
            TokenKind::Newline
            | TokenKind::OptBreak
            | TokenKind::MilestoneEnd
            | TokenKind::Text => OwnedTokenPayload::Plain,
            // A book code and a parsed number are payloads the working shape
            // either omits entirely or carries verbatim; an anchor may only
            // supply one while the token's own text still spells it.
            TokenKind::BookCode => OwnedTokenPayload::BookCode(
                anchor
                    .filter(|anchor| anchor.source() == token.text)
                    .and_then(Self::book_code)
                    .cloned()
                    .ok_or_else(missing_payload)?,
            ),
            TokenKind::Number => OwnedTokenPayload::Number(match token.number_info {
                Some((start, end, kind)) => OwnedNumberInfo { start, end, kind },
                None => anchor
                    .filter(|anchor| anchor.source() == token.text)
                    .and_then(Self::number_info)
                    .cloned()
                    .ok_or_else(missing_payload)?,
            }),
            TokenKind::Marker | TokenKind::EndMarker | TokenKind::Milestone => {
                let marker = token.marker.as_deref().ok_or_else(missing_payload)?;
                let metadata = marker_metadata(marker);
                let structural = token.structural.unwrap_or_else(|| {
                    crate::marker_defs::structural_marker_info(marker, metadata.kind)
                });
                // Nesting is a spelling fact (`\+add`) the working type has no
                // field for, so it survives only from an anchor spelling the
                // same marker. A synthesized token is new content, and new
                // content is not nested.
                let nested = anchor
                    .filter(|anchor| anchor.marker_name() == Some(marker))
                    .is_some_and(|anchor| anchor.nested());
                let attrs = rebuilt_marker_attrs(token, anchor);
                match kind {
                    TokenKind::Marker => OwnedTokenPayload::Marker {
                        marker: Box::from(marker),
                        metadata,
                        structural,
                        nested,
                        attrs,
                    },
                    TokenKind::EndMarker => OwnedTokenPayload::EndMarker {
                        marker: Box::from(marker),
                        metadata,
                        structural,
                        nested,
                    },
                    _ => OwnedTokenPayload::Milestone {
                        marker: Box::from(marker),
                        metadata,
                        structural,
                        attrs,
                    },
                }
            }
        };

        Ok(Self {
            id,
            kind,
            source: Box::from(token.text.as_str()),
            sid,
            parsed_sid,
            payload,
        })
    }

    /// Feeds everything the packed wire encoding derives from this token into
    /// `state`, exhaustively.
    ///
    /// This exists so a consumer can key a cache on "the same token stream" rather
    /// than on the book's bytes, which pin far less than they look like they do: a
    /// book's source is its tokens' text concatenated, so two streams can
    /// serialize identically while differing in every fact a serializer never
    /// writes — stable ids above all, and the canonical anchors, nesting,
    /// book-code validity, parsed numbers, and attribute spellings with them.
    ///
    /// # Why it lives here, and why it cannot silently rot
    ///
    /// The payload is private to this module, so an outside crate can only reach it
    /// through accessors — and an accessor-based projection is exactly the thing
    /// that goes quietly stale, because adding a field to a private struct compiles
    /// fine everywhere that never learned to ask for it. Every value below is
    /// reached by **destructuring with no `..` anywhere**: this token, every payload
    /// variant, the marker-attribute record, each attribute, and each name-derived
    /// struct. Adding a field to any of them is therefore a compile error at this
    /// site, which is the whole guarantee — a caller does not have to trust that
    /// this list is complete, because it cannot compile while being incomplete.
    ///
    /// Fields that are *deliberately* excluded are destructured and ignored by
    /// name, with the reason, rather than skipped: an exclusion is a decision, and
    /// a new field must force that decision to be made again.
    ///
    /// Every `Option` gets a presence byte and every variable-length value is
    /// length-framed, so `None` cannot hash like `Some("")` and `("ab", "c")`
    /// cannot hash like `("a", "bc")`. The hasher, and therefore the algorithm and
    /// the digest's meaning, belong to the caller.
    pub fn hash_wire_identity<H: std::hash::Hasher>(&self, state: &mut H) {
        // No `..`: a new field here is a compile error, by design.
        let Self {
            id,
            kind,
            source,
            sid,
            parsed_sid,
            payload,
        } = self;
        framed(state, id.as_str().as_bytes());
        state.write_u8(*kind as u8);
        framed(state, source.as_bytes());
        optional(state, sid.as_deref(), |state, sid| {
            framed(state, sid.as_bytes())
        });
        optional(state, parsed_sid.as_ref(), |state, sid| {
            let Sid {
                book,
                chapter,
                verse,
                verse_end_delta,
            } = sid;
            framed(state, book.as_str().as_bytes());
            state.write_u16(*chapter);
            state.write_u16(*verse);
            state.write_u8(*verse_end_delta);
        });

        match payload {
            OwnedTokenPayload::Plain => state.write_u8(0),
            OwnedTokenPayload::Marker {
                marker,
                metadata,
                structural,
                nested,
                attrs,
            } => {
                state.write_u8(1);
                framed(state, marker.as_bytes());
                ignore_name_derived(metadata, structural);
                state.write_u8(u8::from(*nested));
                hash_marker_attrs(state, attrs.as_ref());
            }
            OwnedTokenPayload::EndMarker {
                marker,
                metadata,
                structural,
                nested,
            } => {
                state.write_u8(2);
                framed(state, marker.as_bytes());
                ignore_name_derived(metadata, structural);
                state.write_u8(u8::from(*nested));
            }
            OwnedTokenPayload::Milestone {
                marker,
                metadata,
                structural,
                attrs,
            } => {
                state.write_u8(3);
                framed(state, marker.as_bytes());
                ignore_name_derived(metadata, structural);
                hash_marker_attrs(state, attrs.as_ref());
            }
            OwnedTokenPayload::BookCode(OwnedBookCode { code, is_valid }) => {
                state.write_u8(4);
                framed(state, code.as_bytes());
                state.write_u8(u8::from(*is_valid));
            }
            OwnedTokenPayload::Number(OwnedNumberInfo { start, end, kind }) => {
                state.write_u8(5);
                state.write_u32(*start);
                optional(state, end.as_ref(), |state, end| state.write_u32(*end));
                state.write_u8(*kind as u8);
            }
        }
    }

    /// Copies one parsed token into the owned semantic representation.
    pub fn from_parsed(value: &Token<'_>) -> Self {
        let kind = value.kind();
        let payload = match &value.data {
            TokenData::Newline
            | TokenData::OptBreak
            | TokenData::MilestoneEnd
            | TokenData::Text => OwnedTokenPayload::Plain,
            TokenData::Marker {
                name,
                metadata,
                structural,
                nested,
                attrs,
            } => OwnedTokenPayload::Marker {
                marker: Box::from(*name),
                metadata: *metadata,
                structural: *structural,
                nested: *nested,
                attrs: owned_marker_attrs(attrs, value.span.end),
            },
            TokenData::EndMarker {
                name,
                metadata,
                structural,
                nested,
            } => OwnedTokenPayload::EndMarker {
                marker: Box::from(*name),
                metadata: *metadata,
                structural: *structural,
                nested: *nested,
            },
            TokenData::Milestone {
                name,
                metadata,
                structural,
                attrs,
            } => OwnedTokenPayload::Milestone {
                marker: Box::from(*name),
                metadata: *metadata,
                structural: *structural,
                attrs: owned_marker_attrs(attrs, value.span.end),
            },
            TokenData::BookCode { code, is_valid } => OwnedTokenPayload::BookCode(OwnedBookCode {
                code: Box::from(*code),
                is_valid: *is_valid,
            }),
            TokenData::Number { start, end, kind } => OwnedTokenPayload::Number(OwnedNumberInfo {
                start: *start,
                end: *end,
                kind: *kind,
            }),
        };

        Self {
            id: StableTokenId(Box::from(format!(
                "{}-{}",
                value.id.book_code, value.id.index
            ))),
            kind,
            source: Box::from(value.source),
            sid: value.sid.map(|sid| Box::from(sid.to_string())),
            parsed_sid: value.sid,
            payload,
        }
    }

    pub fn id(&self) -> &StableTokenId {
        &self.id
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn sid(&self) -> Option<&str> {
        self.sid.as_deref()
    }

    pub fn marker_name(&self) -> Option<&str> {
        match &self.payload {
            OwnedTokenPayload::Marker { marker, .. }
            | OwnedTokenPayload::EndMarker { marker, .. }
            | OwnedTokenPayload::Milestone { marker, .. } => Some(marker),
            _ => None,
        }
    }

    pub fn nested(&self) -> bool {
        match self.payload {
            OwnedTokenPayload::Marker { nested, .. }
            | OwnedTokenPayload::EndMarker { nested, .. } => nested,
            _ => false,
        }
    }

    /// The compact canonical anchor, when this token has one.
    ///
    /// Distinct from [`Self::sid`], which is the formatted spelling: the packed
    /// wire anchor is eight bytes built from the structured value, and
    /// re-parsing the string form to recover it would fork core's formatting.
    pub fn parsed_sid(&self) -> Option<Sid> {
        self.parsed_sid
    }

    pub fn number_info(&self) -> Option<&OwnedNumberInfo> {
        match &self.payload {
            OwnedTokenPayload::Number(number) => Some(number),
            _ => None,
        }
    }

    pub fn book_code(&self) -> Option<&OwnedBookCode> {
        match &self.payload {
            OwnedTokenPayload::BookCode(book_code) => Some(book_code),
            _ => None,
        }
    }

    pub fn attributes(&self) -> &[OwnedAttribute] {
        match &self.payload {
            OwnedTokenPayload::Marker {
                attrs: Some(attrs), ..
            }
            | OwnedTokenPayload::Milestone {
                attrs: Some(attrs), ..
            } => &attrs.attributes,
            _ => &[],
        }
    }

    /// Distance from the end of this token's own source to the start of its
    /// attribute list, when the token remembers one. `None` means no remembered
    /// position, and a serializer places the list at the marker's closer.
    pub fn attribute_offset(&self) -> Option<BytePos> {
        match &self.payload {
            OwnedTokenPayload::Marker {
                attrs: Some(attrs), ..
            }
            | OwnedTokenPayload::Milestone {
                attrs: Some(attrs), ..
            } => attrs.attribute_offset,
            _ => None,
        }
    }

    pub fn attribute_list(&self) -> Option<&str> {
        match &self.payload {
            OwnedTokenPayload::Marker {
                attrs: Some(attrs), ..
            }
            | OwnedTokenPayload::Milestone {
                attrs: Some(attrs), ..
            } => attrs.attribute_source.as_deref(),
            _ => None,
        }
    }

    /// The structural facts parse recorded for this marker. Crate-visible for
    /// the format boundary, which must hand a working token exactly what parse
    /// recorded rather than re-deriving it from the marker name.
    pub(crate) fn structural(&self) -> Option<StructuralMarkerInfo> {
        match self.payload {
            OwnedTokenPayload::Marker { structural, .. }
            | OwnedTokenPayload::EndMarker { structural, .. }
            | OwnedTokenPayload::Milestone { structural, .. } => Some(structural),
            _ => None,
        }
    }

    fn marker_index(&self) -> MarkerIndex {
        match self.payload {
            OwnedTokenPayload::Marker { metadata, .. }
            | OwnedTokenPayload::EndMarker { metadata, .. }
            | OwnedTokenPayload::Milestone { metadata, .. } => metadata.index,
            _ => MarkerIndex::UNKNOWN,
        }
    }
}

/// Writes a length-framed byte string, so concatenations cannot collide.
fn framed<H: std::hash::Hasher>(state: &mut H, bytes: &[u8]) {
    state.write_u64(bytes.len() as u64);
    state.write(bytes);
}

/// Writes a presence byte, then the value when there is one: `None` and an empty
/// `Some` are different facts and must hash differently.
fn optional<H: std::hash::Hasher, T>(
    state: &mut H,
    value: Option<T>,
    write: impl FnOnce(&mut H, T),
) {
    match value {
        None => state.write_u8(0),
        Some(value) => {
            state.write_u8(1);
            write(state, value);
        }
    }
}

/// The two marker facts the wire encoder deliberately re-derives from the marker
/// name rather than reading off the token (`marker_metadata` /
/// `structural_marker_info`, exactly as decoding does), so they cannot make one
/// name's encoding differ and must not make its identity differ either.
///
/// Destructured with no `..` and ignored by name: adding a field to either struct
/// breaks this line, which forces the "does the encoder read this?" question to be
/// answered again instead of inheriting a stale answer.
fn ignore_name_derived(metadata: &MarkerMetadata, structural: &StructuralMarkerInfo) {
    let MarkerMetadata {
        canonical: _,
        kind: _,
        family: _,
        index: _,
    } = metadata;
    let StructuralMarkerInfo {
        scope_kind: _,
        inline_context: _,
        note_context: _,
    } = structural;
}

/// One marker/milestone's attribute record, exhaustively.
fn hash_marker_attrs<H: std::hash::Hasher>(state: &mut H, attrs: Option<&OwnedMarkerAttrs>) {
    optional(state, attrs, |state, attrs| {
        let OwnedMarkerAttrs {
            attributes,
            attribute_source,
            attribute_offset,
        } = attrs;
        state.write_u64(attributes.len() as u64);
        for attribute in attributes.iter() {
            let OwnedAttribute {
                source,
                key,
                value,
                is_default,
                // The parse-time span of an attribute inside its original
                // document. The owned encoder never reads it: it locates each
                // attribute by finding `source` in the text it just emitted, so a
                // remembered span cannot change the bytes and must not change the
                // identity either.
                span: _,
            } = attribute;
            // `source` is load-bearing for exactly that reason — it is the
            // spelling the encoder searches for — so two attributes that agree on
            // key, value, and default while spelling themselves differently are
            // not the same token.
            framed(state, source.as_bytes());
            framed(state, key.as_bytes());
            framed(state, value.as_bytes());
            state.write_u8(u8::from(*is_default));
        }
        // Presence is the distinction the wire draws too: no verbatim list at all
        // is a token with no attribute row, while an empty one is a row with an
        // empty span.
        optional(state, attribute_source.as_deref(), |state, source| {
            framed(state, source.as_bytes())
        });
        optional(state, attribute_offset.as_ref(), |state, offset| {
            state.write_u32(*offset)
        });
    });
}

/// The attribute record a rebuilt token keeps.
///
/// An untouched list survives verbatim from its anchor — including the
/// remembered placement distance, which is the one attribute fact a working
/// token has no field for. A list the caller actually edited is rebuilt from the
/// working token with no placement, which is exactly the "touch an attribute,
/// drop its verbatim position" rule the emitter already falls back on.
fn rebuilt_marker_attrs(
    token: &crate::format::FormatToken,
    anchor: Option<&OwnedToken>,
) -> Option<OwnedMarkerAttrs> {
    if token.attributes.is_empty() && token.attribute_source.is_none() {
        return None;
    }
    let unchanged = anchor.filter(|anchor| {
        anchor.attribute_list() == token.attribute_source.as_deref()
            && anchor.attributes() == token.attributes.as_slice()
    });
    if let Some(anchor) = unchanged {
        match &anchor.payload {
            OwnedTokenPayload::Marker {
                attrs: Some(attrs), ..
            }
            | OwnedTokenPayload::Milestone {
                attrs: Some(attrs), ..
            } => return Some(attrs.clone()),
            _ => {}
        }
    }
    Some(OwnedMarkerAttrs {
        attributes: token.attributes.clone().into(),
        attribute_source: token.attribute_source.as_deref().map(Box::from),
        attribute_offset: None,
    })
}

fn owned_marker_attrs(
    attrs: &Option<Box<MarkerAttrs<'_>>>,
    owner_end: BytePos,
) -> Option<OwnedMarkerAttrs> {
    attrs.as_deref().map(|attrs| OwnedMarkerAttrs {
        attributes: attrs
            .attributes
            .iter()
            .map(|attribute| OwnedAttribute {
                source: Box::from(attribute.source),
                key: Box::from(attribute.key),
                value: Box::from(attribute.value),
                is_default: attribute.is_default,
                span: Some(attribute.span),
            })
            .collect(),
        attribute_source: attrs.attribute_source.map(|(_, source)| Box::from(source)),
        // `checked_sub` rather than a saturating one: a list recorded as starting
        // before the token that owns it is not a distance this can represent, so
        // it falls back to the closer rule instead of pretending to be zero.
        attribute_offset: attrs
            .attribute_source
            .and_then(|(span, _)| span.start.checked_sub(owner_end)),
    })
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
/// # Where the list lands
///
/// [`tokens_to_usfm`] (native, span-based) and [`tokens_to_usfm_reconstruct`]
/// (this trait, spanless) agree byte for byte whenever the token remembers where
/// its list sat — see [`SerializableToken::attribute_offset`]. That covers every
/// parse-origin stream, including the malformed shapes an unclosed marker
/// produces: the list is placed at its recorded distance from the owning token
/// rather than at a closer that never arrives.
///
/// A token with no remembered position — editor- or DTO-authored, which never had
/// one — falls back to the closer rule: the list is emitted at the marker's
/// closer, or at end of stream if none arrives.
/// Base contract every token shape satisfies, regardless of which
/// higher-level trait (`SerializableToken`, `WalkableToken`, …) a consumer
/// needs on top. `marker()` is `None` for non-marker kinds (Text, Number,
/// Newline, OptBreak, BookCode) — that's honest, not a lying default.
pub trait UsfmToken {
    fn kind(&self) -> TokenKind;
    fn source(&self) -> &str;
    fn marker(&self) -> Option<&str>;
}

pub trait SerializableToken: UsfmToken {
    type Attr: SerializableAttribute;

    fn attributes(&self) -> &[Self::Attr];
    fn attribute_list(&self) -> Option<&str>;

    /// Bytes from the end of this token's own source to the start of its
    /// attribute list, for tokens that remember where the list sat.
    ///
    /// Defaults to `None`, which means "no remembered position" and leaves
    /// [`tokens_to_usfm_reconstruct`] placing the list at the marker's closer —
    /// the behavior every implementor had before this existed.
    fn attribute_offset(&self) -> Option<BytePos> {
        None
    }
}

impl<'a> UsfmToken for Token<'a> {
    fn kind(&self) -> TokenKind {
        Token::kind(self)
    }

    fn source(&self) -> &str {
        self.source
    }

    fn marker(&self) -> Option<&str> {
        self.marker_name()
    }
}

impl<'a> SerializableToken for Token<'a> {
    type Attr = AttributeItem<'a>;

    fn attributes(&self) -> &[Self::Attr] {
        Token::attributes(self).unwrap_or(&[])
    }

    fn attribute_offset(&self) -> Option<BytePos> {
        match &self.data {
            TokenData::Marker { attrs, .. } | TokenData::Milestone { attrs, .. } => attrs
                .as_deref()
                .and_then(|attrs| attrs.attribute_source)
                .and_then(|(span, _)| span.start.checked_sub(self.span.end)),
            _ => None,
        }
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

impl SerializableAttribute for OwnedAttribute {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn is_default(&self) -> bool {
        self.is_default
    }
}

impl UsfmToken for OwnedToken {
    fn kind(&self) -> TokenKind {
        self.kind()
    }

    fn source(&self) -> &str {
        self.source()
    }

    fn marker(&self) -> Option<&str> {
        self.marker_name()
    }
}

impl SerializableToken for OwnedToken {
    type Attr = OwnedAttribute;

    fn attributes(&self) -> &[Self::Attr] {
        self.attributes()
    }

    fn attribute_list(&self) -> Option<&str> {
        self.attribute_list()
    }

    fn attribute_offset(&self) -> Option<BytePos> {
        OwnedToken::attribute_offset(self)
    }
}

impl crate::walker::WalkableToken for OwnedToken {
    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural()
    }
}

impl crate::lint::LintableToken for OwnedToken {
    fn sid(&self) -> Option<String> {
        self.sid().map(ToOwned::to_owned)
    }

    fn id(&self) -> Option<String> {
        Some(self.id().to_string())
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        self.number_info()
            .map(|number| (number.start, number.end, number.kind))
    }

    fn allows_effective_context(&self, context: crate::marker_defs::SpecContext) -> bool {
        crate::marker_defs::marker_allows_effective_context_for_index(self.marker_index(), context)
    }
}

impl crate::diff::DiffableToken for OwnedToken {
    fn sid(&self) -> Option<&str> {
        self.sid()
    }

    fn sid_string(&self) -> Option<String> {
        self.parsed_sid
            .map(|sid| format!("{} {}:{}", sid.book, sid.chapter, sid.verse_locator()))
            .or_else(|| self.sid().map(ToOwned::to_owned))
    }

    fn sid_key(&self) -> crate::diff::SidKey<'_> {
        match self.parsed_sid {
            Some(sid) => crate::diff::SidKey::Compact(sid),
            None => match self.sid() {
                Some(sid) => crate::diff::SidKey::Text(sid),
                None => crate::diff::SidKey::Empty,
            },
        }
    }

    fn text(&self) -> &str {
        self.source()
    }

    fn id(&self) -> Option<&str> {
        Some(self.id().as_str())
    }

    fn kind_key(&self) -> Option<&str> {
        Some(owned_token_kind_key(self.kind()))
    }

    fn marker_key(&self) -> Option<&str> {
        self.marker_name()
    }

    fn number_range(&self) -> Option<(u32, Option<u32>)> {
        self.number_info().map(|number| (number.start, number.end))
    }

    fn book_code(&self) -> Option<&str> {
        self.book_code().map(|book_code| book_code.code.as_ref())
    }
}

fn owned_token_kind_key(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Newline => "verticalWhitespace",
        TokenKind::OptBreak => "optBreak",
        TokenKind::Marker => "marker",
        TokenKind::EndMarker => "endMarker",
        TokenKind::Milestone => "milestone",
        TokenKind::MilestoneEnd => "milestoneEnd",
        TokenKind::BookCode => "bookCode",
        TokenKind::Number => "number",
        TokenKind::Text => "text",
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
    /// Row that queued this list. A list is not emitted next to its marker, so a
    /// span recorder cannot infer the owner from emission order.
    row: usize,
    /// Output position this list must be emitted at, for a token that remembered
    /// where it sat. `None` falls back to the closer rule.
    target: Option<usize>,
}

fn is_paragraph_marker<T: SerializableToken>(token: &T) -> bool {
    if !matches!(token.kind(), TokenKind::Marker) {
        return false;
    }
    token
        .marker()
        .map(|name| {
            matches!(
                closer_shape(TokenKind::Marker, name),
                CloserShape::ParagraphBoundary
            )
        })
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
/// A token that remembers where its list sat ([`SerializableToken::attribute_offset`],
/// which every parse-origin token carries) has the list placed at that distance
/// instead, which is what makes this agree with [`tokens_to_usfm`] byte for byte —
/// including for unclosed markers, where the closer rule alone had to fall back to
/// end-of-stream. See the `tokens_to_usfm_reconstruct_parity` tests.
pub fn tokens_to_usfm_reconstruct<T: SerializableToken>(tokens: &[T]) -> String {
    reconstruct(tokens, None, None)
}

/// The line ending a token stream serializes its newline tokens with.
///
/// Two variants only: these are the endings a USFM file is written with. A
/// lone `\r` is a legal newline to the lexer but is not a writable choice
/// here, so [`Self::detect`] reports it as [`Self::Lf`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }

    /// The first line ending in `source` wins; `Lf` when there is none.
    ///
    /// A file with mixed endings has no single answer, and this deliberately
    /// does not invent one — it reports what the file leads with, and a
    /// caller keeping the exact source bytes never re-emits until an edit
    /// forces it.
    pub fn detect(source: &str) -> Self {
        match source.find('\n') {
            Some(0) | None => Self::Lf,
            Some(index) if source.as_bytes()[index - 1] == b'\r' => Self::CrLf,
            Some(_) => Self::Lf,
        }
    }
}

/// [`tokens_to_usfm_reconstruct`] with every newline token emitted as
/// `line_ending` instead of its own source.
///
/// Exists because a token stream and the line ending it must be saved with are
/// independent facts once tokens outlive their file: an editor pushing tokens
/// into a resident CRLF book supplies `\n` newline tokens, and a book that
/// serialized them verbatim would flip its own file's endings on first edit.
/// Emission stays here, in the one emitter, rather than in a caller
/// post-processing newline tokens it does not own.
///
/// Only `TokenKind::Newline` tokens are affected; no other token source,
/// attribute list, or whitespace is touched.
pub fn tokens_to_usfm_reconstruct_with_eol<T: SerializableToken>(
    tokens: &[T],
    line_ending: LineEnding,
) -> String {
    reconstruct(tokens, None, Some(line_ending))
}

/// Where one token's bytes landed in a reconstructed source.
///
/// `token` and `attribute_list` are not adjacent and cannot be derived from one
/// another: a token that remembers where its list sat
/// ([`SerializableToken::attribute_offset`]) has it placed at that remembered
/// distance; a positionless token falls back to the historical rule of its
/// closer, or end-of-stream for an unclosed marker — either way, the list may
/// land thousands of bytes after the marker that owns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconstructedSpans {
    /// The token's own text, excluding any attribute list — the same slice a
    /// parsed [`Token`]'s `span` covers.
    pub token: Span,
    /// Where this token's attribute list was emitted, when it has one.
    pub attribute_list: Option<Span>,
}

/// [`tokens_to_usfm_reconstruct`], plus where every token's bytes landed.
///
/// Exists because spanless owned tokens cannot be encoded to the packed wire
/// format, which stores span columns: the caller serializes and gets the spans
/// as a by-product of the single emission pass, rather than re-lexing the result
/// or having owned tokens carry span state that could go stale. This emitter is
/// the only code that knows where a deferred attribute list actually lands.
///
/// Spans index the returned `String`, so they are only meaningful against it.
pub fn tokens_to_usfm_reconstruct_spanned<T: SerializableToken>(
    tokens: &[T],
) -> (String, Vec<ReconstructedSpans>) {
    let mut spans = vec![
        ReconstructedSpans {
            token: Span::new(0, 0),
            attribute_list: None,
        };
        tokens.len()
    ];
    let output = reconstruct(tokens, Some(&mut spans), None);
    (output, spans)
}

/// The one implementation both entry points use, so the spanned variant cannot
/// drift from the plain one. `spans` is `None` on the plain path, which is the
/// per-keystroke editor path and pays nothing for recording it would discard.
fn reconstruct<T: SerializableToken>(
    tokens: &[T],
    mut spans: Option<&mut Vec<ReconstructedSpans>>,
    line_ending: Option<LineEnding>,
) -> String {
    let mut output = String::new();
    let mut pending: Vec<Pending<'_, T>> = Vec::new();

    for (row, token) in tokens.iter().enumerate() {
        // Positioned lists first, in ascending target order, so a stream that
        // remembers its layout reproduces it byte for byte. This is the same rule
        // the span-based emitter applies to absolute offsets, expressed as a
        // distance from the owning token instead.
        while let Some(index) = due_pending(&pending, output.len()) {
            let drained = pending.remove(index);
            emit_pending_recorded(&mut output, drained, spans.as_deref_mut());
        }
        // Then the closer rule, for lists with no remembered position.
        while let Some(top) = pending.last() {
            if top.target.is_none() && token_closes(top, token) {
                let drained = pending.pop().unwrap();
                emit_pending_recorded(&mut output, drained, spans.as_deref_mut());
            } else {
                break;
            }
        }

        let start = output.len();
        match line_ending {
            Some(ending) if token.kind() == TokenKind::Newline => output.push_str(ending.as_str()),
            _ => output.push_str(token.source()),
        }
        if let Some(spans) = spans.as_deref_mut() {
            spans[row].token = Span::new(start as BytePos, output.len() as BytePos);
        }

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
                row,
                target: token
                    .attribute_offset()
                    .map(|offset| output.len() + offset as usize),
            });
        }
    }

    // End of stream: anything positioned is due by definition, then the
    // closer-ruled remainder flushes LIFO as it always has.
    while let Some(index) = due_pending(&pending, output.len()) {
        let drained = pending.remove(index);
        emit_pending_recorded(&mut output, drained, spans.as_deref_mut());
    }
    while let Some(drained) = pending.pop() {
        emit_pending_recorded(&mut output, drained, spans.as_deref_mut());
    }

    output
}

/// Index of the positioned pending list with the smallest target that `position`
/// has reached. Linear because `pending` is bounded by marker nesting depth.
fn due_pending<T: SerializableToken>(pending: &[Pending<'_, T>], position: usize) -> Option<usize> {
    pending
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.target.is_some_and(|target| target <= position))
        .min_by_key(|(_, entry)| entry.target)
        .map(|(index, _)| index)
}

fn emit_pending_recorded<T: SerializableToken>(
    output: &mut String,
    pending: Pending<'_, T>,
    spans: Option<&mut Vec<ReconstructedSpans>>,
) {
    let start = output.len();
    let row = pending.row;
    emit_pending(output, pending.attrs);
    if let Some(spans) = spans {
        spans[row].attribute_list = Some(Span::new(start as BytePos, output.len() as BytePos));
    }
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

/// Mirrors [`Self::serialize`]: a 3-byte ASCII-alphanumeric string, validated
/// the same way [`Self::from_str`] validates one. Added for the declared-book
/// lint context (`LintOptions::declared_book`), the first `BookId` field on a
/// type that also derives `Deserialize`.
impl<'de> Deserialize<'de> for BookId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        BookId::from_str(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid BookId: {s:?}")))
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

#[cfg(test)]
mod owned_token_tests {
    use super::{
        OwnedAttribute, OwnedToken, StableTokenId, TokenKind, tokens_to_usfm_reconstruct,
        tokens_to_usfm_reconstruct_spanned,
    };
    use crate::diff::{DiffableToken, derive_canonical_sids};
    use crate::lint::{LintOptions, LintScope, LintableToken, lint_tokens};
    use crate::parse::parse;

    #[test]
    fn stable_token_id_rejects_an_empty_address() {
        assert!(StableTokenId::new("").is_none());
        assert_eq!(
            StableTokenId::new("editor-42").unwrap().as_str(),
            "editor-42"
        );
    }

    #[test]
    fn owned_tokens_preserve_semantics_and_verbatim_attributes() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 \\w grace|lemma=\"grace\"\\w*\n";
        let parsed = parse(source);
        let expected_word_id = parsed
            .tokens
            .iter()
            .find(|token| token.kind() == TokenKind::Marker && token.marker_name() == Some("w"))
            .map(|token| format!("{}-{}", token.id.book_code, token.id.index))
            .expect("parsed character marker");
        let tokens = parsed
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>();

        let word = tokens
            .iter()
            .find(|token| token.kind() == TokenKind::Marker && token.marker_name() == Some("w"))
            .expect("parsed character marker");
        assert_eq!(word.id().as_str(), expected_word_id);
        assert_eq!(word.attributes().len(), 1);
        assert_eq!(word.attributes()[0].key.as_ref(), "lemma");
        assert_eq!(word.attribute_list(), Some("|lemma=\"grace\""));

        assert_eq!(tokens_to_usfm_reconstruct(&tokens), source);
        assert_eq!(
            derive_canonical_sids(&tokens, "GEN"),
            derive_canonical_sids(&parsed.tokens, "GEN")
        );
        assert_eq!(
            lint_tokens(&tokens, LintOptions::scoped(LintScope::Book))
                .issues
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>(),
            lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book))
                .issues
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
        );
    }

    /// A token stream with no remembered positions — the shape an editor or a
    /// wire DTO produces. Exercises the `SerializableToken::attribute_offset`
    /// default rather than asserting it indirectly.
    struct PositionlessToken {
        kind: TokenKind,
        source: String,
        marker: Option<String>,
        attribute_list: Option<String>,
    }

    impl super::UsfmToken for PositionlessToken {
        fn kind(&self) -> TokenKind {
            self.kind
        }

        fn source(&self) -> &str {
            &self.source
        }

        fn marker(&self) -> Option<&str> {
            self.marker.as_deref()
        }
    }

    impl super::SerializableToken for PositionlessToken {
        type Attr = OwnedAttribute;

        fn attributes(&self) -> &[Self::Attr] {
            &[]
        }

        fn attribute_list(&self) -> Option<&str> {
            self.attribute_list.as_deref()
        }
        // `attribute_offset` deliberately not implemented: the default is what
        // every ingest path relies on.
    }

    #[test]
    fn owned_tokens_remember_where_their_attribute_list_sat() {
        // The four shapes an emitter with only a closer rule could not reproduce.
        for source in [
            "\\p And\\fig something | and some more text\n",
            "\\p \\w \\+pn Proper Noun\\+pn*|keyword\\w* def\n",
            "\\p \\qt1-s |sid=\"a\"\nwho=\"Paul\"\\*said\n",
            "\\k-s | x-tw=\"a\"\n\\w b|c=\"d\"\\w*\n\\k-e\\*\n",
        ] {
            let owned = parse(source)
                .tokens
                .iter()
                .map(OwnedToken::from_parsed)
                .collect::<Vec<_>>();
            assert_eq!(
                tokens_to_usfm_reconstruct(&owned),
                source,
                "owned round trip for {source:?}"
            );
        }
    }

    #[test]
    fn attribute_offset_is_the_distance_from_the_owning_token() {
        // `\\w ` ends at 3 and its list starts at 7, four bytes later, past the
        // intervening text token. An offset from the owner — not an absolute
        // position — is what stays meaningful when other tokens are edited.
        let source = "\\p \\w abc|k=\"v\"\\w*";
        let owned = parse(source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>();
        let word = owned
            .iter()
            .find(|token| token.marker_name() == Some("w"))
            .expect("character marker");
        assert_eq!(word.attribute_offset(), Some(3));
        assert_eq!(word.attribute_list(), Some("|k=\"v\""));

        // A milestone whose list sits immediately after the opener remembers zero.
        let owned = parse("\\k-s | x=\"a\"\ntext\\k-e\\*")
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>();
        let opener = owned
            .iter()
            .find(|token| token.marker_name() == Some("k-s"))
            .expect("milestone opener");
        assert_eq!(opener.attribute_offset(), Some(0));
    }

    #[test]
    fn tokens_without_a_remembered_position_still_emit_at_the_closer() {
        let token =
            |kind, source: &str, marker: Option<&str>, list: Option<&str>| PositionlessToken {
                kind,
                source: source.to_string(),
                marker: marker.map(ToOwned::to_owned),
                attribute_list: list.map(ToOwned::to_owned),
            };
        let tokens = [
            token(TokenKind::Marker, "\\w ", Some("w"), Some("|k=\"v\"")),
            token(TokenKind::Text, "word", None, None),
            token(TokenKind::EndMarker, "\\w*", Some("w"), None),
        ];
        // Unchanged behavior: with nothing remembered, the list goes to the closer.
        assert_eq!(tokens_to_usfm_reconstruct(&tokens), "\\w word|k=\"v\"\\w*");
    }

    #[test]
    fn spanned_reconstruct_agrees_with_the_plain_emitter_and_locates_every_token() {
        // An attribute-bearing marker is the case that matters: its list is
        // emitted at the closer, so the concatenation of token sources is not
        // the source and a span cannot be a running offset.
        let source = "\\id GEN\n\\c 1\n\\p \\w grace|lemma=\"grace\"\\w* and more\n";
        let parsed = parse(source);
        let tokens = parsed
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>();

        let (rebuilt, spans) = tokens_to_usfm_reconstruct_spanned(&tokens);
        assert_eq!(rebuilt, source);
        assert_eq!(rebuilt, tokens_to_usfm_reconstruct(&tokens));
        assert_eq!(spans.len(), tokens.len());

        for (token, span) in tokens.iter().zip(&spans) {
            assert_eq!(
                &rebuilt[span.token.as_range()],
                token.source(),
                "token span must name the token's own text"
            );
            assert_eq!(
                span.attribute_list.map(|list| &rebuilt[list.as_range()]),
                token.attribute_list(),
                "attribute-list span must name the verbatim list"
            );
        }

        // The list lands after the marker's own text, not adjacent to it.
        let word = spans
            .iter()
            .zip(&tokens)
            .find(|(_, token)| token.marker_name() == Some("w"))
            .map(|(span, _)| *span)
            .expect("character marker");
        let list = word.attribute_list.expect("verbatim list");
        assert!(list.start > word.token.end);
    }

    #[test]
    fn owned_token_keeps_verse_zero_diff_spelling() {
        let parsed = parse("\\id GEN\n\\c 1\n\\p\n");
        let tokens = parsed
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>();
        let (parsed_token, owned_token) = parsed
            .tokens
            .iter()
            .zip(&tokens)
            .find(|(parsed_token, _)| {
                parsed_token
                    .sid
                    .is_some_and(|sid| sid.chapter == 1 && sid.verse == 0)
            })
            .expect("chapter-scope token");

        assert_eq!(
            DiffableToken::sid_string(owned_token),
            DiffableToken::sid_string(parsed_token)
        );
        assert_eq!(
            DiffableToken::sid_string(owned_token).as_deref(),
            Some("GEN 1:0")
        );
        assert_eq!(
            LintableToken::sid(owned_token),
            parsed_token.sid.map(|sid| sid.to_string())
        );
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

    #[test]
    fn agrees_where_the_list_is_nowhere_near_the_closer() {
        // Each of these used to diverge, because the emitter's only rule was "at
        // the marker's closer" and none of them puts the list there. They agree now
        // that the token remembers the distance to its own list.
        for source in [
            // Unclosed marker: the list sits mid-text and no closer ever arrives.
            "\\p And\\fig something | and some more text\n",
            // List belongs to the nested marker but sits *after* its closer.
            "\\p \\w \\+pn Proper Noun\\+pn*|keyword\\w* def\n",
            // List split by a newline: the remainder parses as text.
            "\\p \\qt1-s |sid=\"a\"\nwho=\"Paul\"\\*said\n",
            // Old-format alignment: list at the opener, closer lines below.
            "\\k-s | x-tw=\"a\"\n\\w b|c=\"d\"\\w*\n\\k-e\\*\n",
        ] {
            assert_parity(source);
        }
    }
}

#[cfg(test)]
mod line_ending_tests {
    use super::{
        LineEnding, OwnedToken, tokens_to_usfm_reconstruct, tokens_to_usfm_reconstruct_with_eol,
    };
    use crate::parse::parse;

    fn owned(source: &str) -> Vec<OwnedToken> {
        parse(source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect()
    }

    #[test]
    fn detect_reports_what_the_source_leads_with() {
        assert_eq!(LineEnding::detect("\\id GEN\r\n\\c 1\n"), LineEnding::CrLf);
        assert_eq!(LineEnding::detect("\\id GEN\n\\c 1\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("\\id GEN"), LineEnding::Lf);
        // A leading newline has no preceding byte to inspect, and a lone `\r`
        // is not a writable ending — both report Lf rather than panicking or
        // inventing a third variant.
        assert_eq!(LineEnding::detect("\n\\id GEN"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("\\id GEN\r\\c 1"), LineEnding::Lf);
    }

    #[test]
    fn the_override_rewrites_newline_tokens_and_nothing_else() {
        // The editor's push direction: tokens carry `\n` newlines because that
        // is what a JS editor produces, while the resident book must keep
        // saving CRLF. Only the newline tokens change — the `\w` attribute
        // list, its spacing, and every other token source survive verbatim.
        let tokens = owned("\\id GEN\n\\c 1\n\\p\n\\v 1 \\w a|lemma=\"b\"\\w*\n");
        assert_eq!(
            tokens_to_usfm_reconstruct_with_eol(&tokens, LineEnding::CrLf),
            "\\id GEN\r\n\\c 1\r\n\\p\r\n\\v 1 \\w a|lemma=\"b\"\\w*\r\n"
        );
        assert_eq!(
            tokens_to_usfm_reconstruct_with_eol(&tokens, LineEnding::Lf),
            tokens_to_usfm_reconstruct(&tokens)
        );
    }

    #[test]
    fn the_override_normalizes_a_mixed_ending_stream_both_ways() {
        // A stream that came from a mixed-EOL file: the override is what makes
        // one book's saved bytes consistent, in either direction.
        let tokens = owned("\\id GEN\r\n\\c 1\n\\p\r\n");
        assert_eq!(
            tokens_to_usfm_reconstruct_with_eol(&tokens, LineEnding::Lf),
            "\\id GEN\n\\c 1\n\\p\n"
        );
        assert_eq!(
            tokens_to_usfm_reconstruct_with_eol(&tokens, LineEnding::CrLf),
            "\\id GEN\r\n\\c 1\r\n\\p\r\n"
        );
        // Without an override the emitter stays verbatim — mixed input is
        // never silently normalized.
        assert_eq!(
            tokens_to_usfm_reconstruct(&tokens),
            "\\id GEN\r\n\\c 1\n\\p\r\n"
        );
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
