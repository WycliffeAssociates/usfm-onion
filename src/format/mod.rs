use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::marker_defs::StructuralMarkerInfo;
use crate::markers::{MarkerKind, lookup_marker};
use crate::parse::parse;
use crate::token::{
    NumberRangeKind, OwnedAttribute, SerializableToken, Span, Token, TokenData, TokenKind,
};

const POETRY_MARKERS: &[&str] = &[
    "q", "q1", "q2", "q3", "q4", "q5", "qc", "qa", "qm", "qm1", "qm2", "qm3", "qd",
];

const LINEBREAK_BEFORE_AND_AFTER_MARKERS: &[&str] = &[
    "p", "m", "pi", "pi1", "pi2", "pi3", "pi4", "ms", "ms1", "ms2", "ms3", "li", "li1", "li2",
    "li3", "li4", "b",
];

const LINEBREAK_BEFORE_ONLY_MARKERS: &[&str] = &[
    "cl", "cd", "d", "sp", "r", "mr", "sr", "s", "s1", "s2", "s3", "s4",
];

pub type MessageParams = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum FormatRule {
    RecoverMalformedMarkers,
    CollapseWhitespaceInText,
    EnsureInlineSeparators,
    RemoveDuplicateVerseNumbers,
    NormalizeSpacingAfterParagraphMarkers,
    RemoveUnwantedLinebreaks,
    BridgeConsecutiveVerseMarkers,
    RemoveOrphanEmptyVerseBeforeContentfulVerse,
    RemoveBridgeVerseEnumerators,
    MoveChapterLabelAfterChapterMarker,
    InsertDefaultParagraphAfterChapterIntro,
    RemoveEmptyParagraphs,
    InsertStructuralLinebreaks,
    CollapseConsecutiveLinebreaks,
    NormalizeMarkerWhitespaceAtLineStart,
}

impl FormatRule {
    pub const ALL: &'static [FormatRule] = &[
        Self::RecoverMalformedMarkers,
        Self::CollapseWhitespaceInText,
        Self::EnsureInlineSeparators,
        Self::RemoveDuplicateVerseNumbers,
        Self::NormalizeSpacingAfterParagraphMarkers,
        Self::RemoveUnwantedLinebreaks,
        Self::BridgeConsecutiveVerseMarkers,
        Self::RemoveOrphanEmptyVerseBeforeContentfulVerse,
        Self::RemoveBridgeVerseEnumerators,
        Self::MoveChapterLabelAfterChapterMarker,
        Self::InsertDefaultParagraphAfterChapterIntro,
        Self::RemoveEmptyParagraphs,
        Self::InsertStructuralLinebreaks,
        Self::CollapseConsecutiveLinebreaks,
        Self::NormalizeMarkerWhitespaceAtLineStart,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::RecoverMalformedMarkers => "recover-malformed-markers",
            Self::CollapseWhitespaceInText => "collapse-whitespace-in-text",
            Self::EnsureInlineSeparators => "ensure-inline-separators",
            Self::RemoveDuplicateVerseNumbers => "remove-duplicate-verse-numbers",
            Self::NormalizeSpacingAfterParagraphMarkers => {
                "normalize-spacing-after-paragraph-markers"
            }
            Self::RemoveUnwantedLinebreaks => "remove-unwanted-linebreaks",
            Self::BridgeConsecutiveVerseMarkers => "bridge-consecutive-verse-markers",
            Self::RemoveOrphanEmptyVerseBeforeContentfulVerse => {
                "remove-orphan-empty-verse-before-contentful-verse"
            }
            Self::RemoveBridgeVerseEnumerators => "remove-bridge-verse-enumerators",
            Self::MoveChapterLabelAfterChapterMarker => "move-chapter-label-after-chapter-marker",
            Self::InsertDefaultParagraphAfterChapterIntro => {
                "insert-default-paragraph-after-chapter-intro"
            }
            Self::RemoveEmptyParagraphs => "remove-empty-paragraphs",
            Self::InsertStructuralLinebreaks => "insert-structural-linebreaks",
            Self::CollapseConsecutiveLinebreaks => "collapse-consecutive-linebreaks",
            Self::NormalizeMarkerWhitespaceAtLineStart => {
                "normalize-marker-whitespace-at-line-start"
            }
        }
    }

    pub const fn label_key(self) -> &'static str {
        match self {
            Self::RecoverMalformedMarkers => "format.rule.recoverMalformedMarkers",
            Self::CollapseWhitespaceInText => "format.rule.collapseWhitespaceInText",
            Self::EnsureInlineSeparators => "format.rule.ensureInlineSeparators",
            Self::RemoveDuplicateVerseNumbers => "format.rule.removeDuplicateVerseNumbers",
            Self::NormalizeSpacingAfterParagraphMarkers => {
                "format.rule.normalizeSpacingAfterParagraphMarkers"
            }
            Self::RemoveUnwantedLinebreaks => "format.rule.removeUnwantedLinebreaks",
            Self::BridgeConsecutiveVerseMarkers => "format.rule.bridgeConsecutiveVerseMarkers",
            Self::RemoveOrphanEmptyVerseBeforeContentfulVerse => {
                "format.rule.removeOrphanEmptyVerseBeforeContentfulVerse"
            }
            Self::RemoveBridgeVerseEnumerators => "format.rule.removeBridgeVerseEnumerators",
            Self::MoveChapterLabelAfterChapterMarker => {
                "format.rule.moveChapterLabelAfterChapterMarker"
            }
            Self::InsertDefaultParagraphAfterChapterIntro => {
                "format.rule.insertDefaultParagraphAfterChapterIntro"
            }
            Self::RemoveEmptyParagraphs => "format.rule.removeEmptyParagraphs",
            Self::InsertStructuralLinebreaks => "format.rule.insertStructuralLinebreaks",
            Self::CollapseConsecutiveLinebreaks => "format.rule.collapseConsecutiveLinebreaks",
            Self::NormalizeMarkerWhitespaceAtLineStart => {
                "format.rule.normalizeMarkerWhitespaceAtLineStart"
            }
        }
    }
}

/// A named format profile that selects a curated set of format rules.
///
/// Profiles are the user-facing way to ask for a specific kind of formatted
/// output without having to know which individual `FormatRule`s to enable.
/// They are inspired by USFM 3.1's distinction between source-readability
/// concerns (code-editor view) and content-readability concerns (preview /
/// WYSIWYG view). Both profiles collapse excess whitespace and normalize
/// intra-content whitespace by default; see [`FormatOptions::for_profile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatProfile {
    /// Marker-per-line for block markers; preserves inline markers
    /// (characters and notes) inline. Optimized for reading USFM in a
    /// code editor.
    CodeEditor,
    /// Removes non-required structural newlines so paragraph blocks render
    /// as continuous text. Optimized for previewing what a WYSIWYG editor
    /// would show.
    Reading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FormatOptions {
    pub recover_malformed_markers: bool,
    pub collapse_whitespace_in_text: bool,
    pub ensure_inline_separators: bool,
    pub remove_duplicate_verse_numbers: bool,
    pub normalize_spacing_after_paragraph_markers: bool,
    pub remove_unwanted_linebreaks: bool,
    pub bridge_consecutive_verse_markers: bool,
    pub remove_orphan_empty_verse_before_contentful_verse: bool,
    pub remove_bridge_verse_enumerators: bool,
    pub move_chapter_label_after_chapter_marker: bool,
    pub insert_default_paragraph_after_chapter_intro: bool,
    pub remove_empty_paragraphs: bool,
    pub insert_structural_linebreaks: bool,
    pub collapse_consecutive_linebreaks: bool,
    pub normalize_marker_whitespace_at_line_start: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self::all_enabled()
    }
}

impl FormatOptions {
    pub const fn all_enabled() -> Self {
        Self {
            recover_malformed_markers: true,
            collapse_whitespace_in_text: true,
            ensure_inline_separators: true,
            remove_duplicate_verse_numbers: true,
            normalize_spacing_after_paragraph_markers: true,
            remove_unwanted_linebreaks: true,
            bridge_consecutive_verse_markers: true,
            remove_orphan_empty_verse_before_contentful_verse: true,
            remove_bridge_verse_enumerators: true,
            move_chapter_label_after_chapter_marker: true,
            insert_default_paragraph_after_chapter_intro: true,
            remove_empty_paragraphs: false,
            insert_structural_linebreaks: true,
            collapse_consecutive_linebreaks: true,
            normalize_marker_whitespace_at_line_start: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            recover_malformed_markers: false,
            collapse_whitespace_in_text: false,
            ensure_inline_separators: false,
            remove_duplicate_verse_numbers: false,
            normalize_spacing_after_paragraph_markers: false,
            remove_unwanted_linebreaks: false,
            bridge_consecutive_verse_markers: false,
            remove_orphan_empty_verse_before_contentful_verse: false,
            remove_bridge_verse_enumerators: false,
            move_chapter_label_after_chapter_marker: false,
            insert_default_paragraph_after_chapter_intro: false,
            remove_empty_paragraphs: false,
            insert_structural_linebreaks: false,
            collapse_consecutive_linebreaks: false,
            normalize_marker_whitespace_at_line_start: false,
        }
    }

    pub fn only(rules: &[FormatRule]) -> Self {
        let mut options = Self::none();
        for rule in rules {
            options.set(*rule, true);
        }
        options
    }

    pub fn excluding(rules: &[FormatRule]) -> Self {
        let mut options = Self::all_enabled();
        for rule in rules {
            options.set(*rule, false);
        }
        options
    }

    /// Build a curated `FormatOptions` for a named [`FormatProfile`].
    ///
    /// Both profiles enable the structural whitespace cleanup rules
    /// (collapse excess whitespace, normalize tag-end delimiters, collapse
    /// consecutive linebreaks, normalize marker whitespace at line start)
    /// and intra-content collapse, since neither view wants noisy artifact
    /// whitespace. They differ on whether block markers should sit on their
    /// own line:
    ///
    /// - [`FormatProfile::CodeEditor`] inserts structural linebreaks before
    ///   block markers — paragraph, chapter, sidebar, etc. each start a
    ///   new line, which is what you want when reading USFM source.
    /// - [`FormatProfile::Reading`] removes non-required structural linebreaks
    ///   so paragraph blocks read as continuous text — the way a WYSIWYG
    ///   preview would render them.
    ///
    /// Inline markers (character markers and note containers) never get
    /// their own line in either profile.
    pub const fn for_profile(profile: FormatProfile) -> Self {
        match profile {
            FormatProfile::CodeEditor => Self {
                recover_malformed_markers: true,
                collapse_whitespace_in_text: true,
                ensure_inline_separators: true,
                remove_duplicate_verse_numbers: true,
                normalize_spacing_after_paragraph_markers: true,
                remove_unwanted_linebreaks: false,
                bridge_consecutive_verse_markers: true,
                remove_orphan_empty_verse_before_contentful_verse: true,
                remove_bridge_verse_enumerators: true,
                move_chapter_label_after_chapter_marker: true,
                insert_default_paragraph_after_chapter_intro: true,
                remove_empty_paragraphs: false,
                insert_structural_linebreaks: true,
                collapse_consecutive_linebreaks: true,
                normalize_marker_whitespace_at_line_start: true,
            },
            FormatProfile::Reading => Self {
                recover_malformed_markers: true,
                collapse_whitespace_in_text: true,
                ensure_inline_separators: true,
                remove_duplicate_verse_numbers: true,
                normalize_spacing_after_paragraph_markers: true,
                remove_unwanted_linebreaks: true,
                bridge_consecutive_verse_markers: true,
                remove_orphan_empty_verse_before_contentful_verse: true,
                remove_bridge_verse_enumerators: true,
                move_chapter_label_after_chapter_marker: true,
                insert_default_paragraph_after_chapter_intro: true,
                remove_empty_paragraphs: false,
                insert_structural_linebreaks: false,
                collapse_consecutive_linebreaks: true,
                normalize_marker_whitespace_at_line_start: true,
            },
        }
    }

    pub fn set(&mut self, rule: FormatRule, enabled: bool) {
        match rule {
            FormatRule::RecoverMalformedMarkers => self.recover_malformed_markers = enabled,
            FormatRule::CollapseWhitespaceInText => self.collapse_whitespace_in_text = enabled,
            FormatRule::EnsureInlineSeparators => self.ensure_inline_separators = enabled,
            FormatRule::RemoveDuplicateVerseNumbers => {
                self.remove_duplicate_verse_numbers = enabled
            }
            FormatRule::NormalizeSpacingAfterParagraphMarkers => {
                self.normalize_spacing_after_paragraph_markers = enabled
            }
            FormatRule::RemoveUnwantedLinebreaks => self.remove_unwanted_linebreaks = enabled,
            FormatRule::BridgeConsecutiveVerseMarkers => {
                self.bridge_consecutive_verse_markers = enabled
            }
            FormatRule::RemoveOrphanEmptyVerseBeforeContentfulVerse => {
                self.remove_orphan_empty_verse_before_contentful_verse = enabled
            }
            FormatRule::RemoveBridgeVerseEnumerators => {
                self.remove_bridge_verse_enumerators = enabled
            }
            FormatRule::MoveChapterLabelAfterChapterMarker => {
                self.move_chapter_label_after_chapter_marker = enabled
            }
            FormatRule::InsertDefaultParagraphAfterChapterIntro => {
                self.insert_default_paragraph_after_chapter_intro = enabled
            }
            FormatRule::RemoveEmptyParagraphs => self.remove_empty_paragraphs = enabled,
            FormatRule::InsertStructuralLinebreaks => self.insert_structural_linebreaks = enabled,
            FormatRule::CollapseConsecutiveLinebreaks => {
                self.collapse_consecutive_linebreaks = enabled
            }
            FormatRule::NormalizeMarkerWhitespaceAtLineStart => {
                self.normalize_marker_whitespace_at_line_start = enabled
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatLabel {
    pub code: String,
    pub key: String,
    pub params: MessageParams,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenTemplate {
    pub kind: TokenKind,
    pub text: String,
    pub marker: Option<String>,
    pub sid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FormatFix {
    ReplaceToken {
        code: String,
        label: String,
        label_params: MessageParams,
        target_token_id: String,
        replacements: Vec<TokenTemplate>,
    },
    DeleteToken {
        code: String,
        label: String,
        label_params: MessageParams,
        target_token_id: String,
    },
    InsertAfter {
        code: String,
        label: String,
        label_params: MessageParams,
        target_token_id: String,
        insert: Vec<TokenTemplate>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct FormatTimings {
    #[serde(skip)]
    pub normalize: Duration,
    #[serde(skip)]
    pub verse_normalize: Duration,
    #[serde(skip)]
    pub default_paragraphs: Duration,
    #[serde(skip)]
    pub structural_linebreaks: Duration,
    #[serde(skip)]
    pub collapse_linebreaks: Duration,
    #[serde(skip)]
    pub normalize_line_start: Duration,
    #[serde(skip)]
    pub total: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LinebreakBehavior {
    None,
    BeforeOnly,
    BeforeAndAfter,
    BeforeIfNextMarker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FormatMarkerProfile {
    pub is_char_or_note_like: bool,
    pub linebreak_behavior: LinebreakBehavior,
    pub empty_paragraph_candidate: bool,
    pub empty_paragraph_boundary: bool,
    pub valid_paragraph_or_heading: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatToken {
    pub kind: TokenKind,
    pub text: String,
    pub marker: Option<String>,
    pub sid: Option<String>,
    pub id: Option<String>,
    pub span: Option<Span>,
    pub structural: Option<StructuralMarkerInfo>,
    pub number_info: Option<(u32, Option<u32>, NumberRangeKind)>,
    pub marker_profile: Option<FormatMarkerProfile>,
    /// Verbatim `|...` attribute-list text (the same shape as
    /// `dto::Token.attributeSource`/`OwnedToken::attribute_list()`), carried
    /// as one opaque string rather than parsed apart. Format never inspects
    /// or edits attribute content — no rule needs to — so a single string
    /// field that survives `Clone` and gets placed back on emission (see
    /// [`format_tokens_to_usfm`]) is the whole fix: nothing downstream has
    /// to special-case it. `None` when the token has no attribute list.
    pub attribute_source: Option<String>,
}

impl FormatToken {
    pub fn to_usfm_fragment(&self) -> &str {
        &self.text
    }
}

impl<'a> From<&Token<'a>> for FormatToken {
    fn from(token: &Token<'a>) -> Self {
        let structural = match &token.data {
            TokenData::Marker { structural, .. }
            | TokenData::EndMarker { structural, .. }
            | TokenData::Milestone { structural, .. } => Some(*structural),
            _ => None,
        };
        let number_info = match &token.data {
            TokenData::Number { start, end, kind } => Some((*start, *end, *kind)),
            _ => None,
        };
        Self {
            kind: token.kind(),
            text: token.source.to_string(),
            marker: token.marker_name().map(ToOwned::to_owned),
            sid: None,
            id: None,
            span: Some(token.span),
            structural,
            number_info,
            marker_profile: token
                .marker_name()
                .map(|marker| build_marker_profile(marker, token.kind(), structural)),
            attribute_source: token.attribute_list().map(ToOwned::to_owned),
        }
    }
}

impl SerializableToken for FormatToken {
    // FormatToken never carries a structured attribute list — only the
    // verbatim `attribute_source` string (attribute content is not
    // something any format rule inspects or edits) — so `Self::Attr` is
    // never actually populated; `OwnedAttribute` just satisfies the bound.
    type Attr = OwnedAttribute;

    fn attributes(&self) -> &[Self::Attr] {
        &[]
    }

    fn attribute_list(&self) -> Option<&str> {
        self.attribute_source.as_deref()
    }

    // No `attribute_offset` override: `None` (the default) means every
    // attribute list places at its marker's closer, never at a remembered
    // byte distance. Format is a normalizing pass, not a byte-exact one —
    // token order can shift — so the closer rule is the correct choice
    // here, not a fallback for a missing feature.
}

/// An attribute-bearing marker's list does not sit next to the marker's own
/// text in USFM — `\w gracious|lemma="x" \w*` stores the list on the
/// *opening* token but it reads after the content, right before `\w*`.
/// [`crate::token::tokens_to_usfm_reconstruct`] already solves exactly this
/// placement problem generically over any [`SerializableToken`] (it is what
/// keeps owned/editor-authored streams, which have no reliable span, byte-
/// correct); reusing it here — rather than a second, naive "append after
/// this token" concatenation — is the whole attribute-passthrough fix. No
/// rule in this module reads or writes attribute content anywhere.
pub fn format_tokens_to_usfm(tokens: &[FormatToken]) -> String {
    crate::token::tokens_to_usfm_reconstruct(tokens)
}

pub trait FormattableToken: Clone {
    fn id(&self) -> Option<&str> {
        None
    }
    /// No default: a no-op default here would let an implementor pass the
    /// `_with_minter` seam's guarantee silently — the minter mints an id,
    /// `set_id` discards it, and callers relying on every token being
    /// addressable would find out only when something downstream can't
    /// find the token it was promised. Every implementor must decide, in
    /// its own visible code, what happens to a set id — even if that
    /// decision is "nothing" — rather than inherit silence from the trait.
    fn set_id(&mut self, id: String);
    fn kind(&self) -> TokenKind;
    fn set_kind(&mut self, kind: TokenKind);
    fn text(&self) -> &str;
    fn set_text(&mut self, text: String);
    fn marker(&self) -> Option<&str>;
    fn set_marker(&mut self, marker: Option<String>);
    fn sid(&self) -> Option<&str> {
        None
    }
    fn set_sid(&mut self, _sid: Option<String>) {}
    fn span(&self) -> Option<Span> {
        None
    }
    fn structural(&self) -> Option<StructuralMarkerInfo> {
        None
    }
    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        None
    }
    fn marker_profile(&self) -> Option<FormatMarkerProfile> {
        None
    }
    /// Verbatim `|...` attribute-list text. Defaults to `None`/no-op so a
    /// token type that never carries attributes (most `LintableToken`-only
    /// test fixtures, editor types that model attributes some other way)
    /// implements nothing extra — the same shape `sid`/`structural` already
    /// use. A type that does carry attributes overrides both, and gets them
    /// preserved through every format/fix pass with zero changes to the
    /// pass itself: passthrough is `Clone` plus this one pair of accessors,
    /// not per-rule bookkeeping.
    fn attribute_source(&self) -> Option<&str> {
        None
    }
    fn set_attribute_source(&mut self, _source: Option<String>) {}
    fn synthetic_like(
        anchor: Option<&Self>,
        kind: TokenKind,
        text: String,
        marker: Option<String>,
        sid: Option<String>,
    ) -> Self;
}

/// Attaches a fresh id to a token the formatter/fix-applier just synthesized,
/// when a minter is supplied.
///
/// Core never invents an id itself (address-agnostic — ids come from a
/// caller or from `assign_ids` at parse time, never fabricated mid-pipeline)
/// and never uses randomness (`std` has none, and determinism is a format
/// invariant, not an implementation detail). With no minter — every call
/// site before this seam existed, and every call site today unless a caller
/// opts in — a synthesized token keeps its historical no-id shape
/// (`synthetic_like` itself never calls `set_id`). A caller that needs every
/// resident token addressable supplies a minter that is a pure function of
/// how many times it has already been called in this pass, e.g.
/// `{book}-p{patch}-{n}` — reproducible across runs, unique per book by
/// construction; ingest/apply validation remains the collision backstop.
pub(crate) fn mint_synthetic_id<T: FormattableToken>(
    token: &mut T,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) {
    if let Some(mint) = minter.as_deref_mut() {
        token.set_id(mint());
    }
}

impl FormattableToken for FormatToken {
    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    fn kind(&self) -> TokenKind {
        self.kind
    }

    fn set_kind(&mut self, kind: TokenKind) {
        self.kind = kind;
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn set_marker(&mut self, marker: Option<String>) {
        self.marker = marker;
    }

    fn sid(&self) -> Option<&str> {
        self.sid.as_deref()
    }

    fn set_sid(&mut self, sid: Option<String>) {
        self.sid = sid;
    }

    fn span(&self) -> Option<Span> {
        self.span
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        self.number_info
    }

    fn marker_profile(&self) -> Option<FormatMarkerProfile> {
        self.marker_profile
    }

    fn attribute_source(&self) -> Option<&str> {
        self.attribute_source.as_deref()
    }

    fn set_attribute_source(&mut self, source: Option<String>) {
        self.attribute_source = source;
    }

    fn synthetic_like(
        anchor: Option<&Self>,
        kind: TokenKind,
        text: String,
        marker: Option<String>,
        sid: Option<String>,
    ) -> Self {
        let marker_profile = marker
            .as_deref()
            .map(|value| build_marker_profile(value, kind, None));
        Self {
            kind,
            text,
            marker,
            sid,
            id: None,
            span: anchor.and_then(|token| token.span),
            structural: None,
            number_info: None,
            marker_profile,
            // A synthesized token (an inserted linebreak, a default `\p`, a
            // template-built fix replacement) is new content that never had
            // an attribute list of its own.
            attribute_source: None,
        }
    }
}

pub fn into_format_tokens(tokens: &[Token<'_>]) -> Vec<FormatToken> {
    tokens.iter().map(FormatToken::from).collect()
}

pub fn format<T: FormattableToken + Clone>(tokens: &[T], options: FormatOptions) -> Vec<T> {
    let mut working = tokens.to_vec();
    format_tokens(&mut working, options);
    working
}

pub fn format_mut<T: FormattableToken>(tokens: &mut Vec<T>, options: FormatOptions) {
    format_tokens(tokens, options);
}

pub fn format_mut_default<T: FormattableToken>(tokens: &mut Vec<T>) {
    format_mut(tokens, FormatOptions::default());
}

pub fn format_tokens<T: FormattableToken>(tokens: &mut Vec<T>, options: FormatOptions) {
    let mut minter: Option<&mut dyn FnMut() -> String> = None;
    format_tokens_owned(tokens, options, &mut minter);
}

pub fn format_tokens_profile<T: FormattableToken>(
    tokens: &[T],
    options: FormatOptions,
) -> (Vec<T>, FormatTimings) {
    let mut working = tokens.to_vec();
    let mut minter: Option<&mut dyn FnMut() -> String> = None;
    let profile = format_tokens_owned(&mut working, options, &mut minter);
    (working, profile)
}

pub fn format_usfm(source: &str, options: FormatOptions) -> String {
    let parsed = parse(source);
    let mut tokens = into_format_tokens(&parsed.tokens);
    format_tokens(&mut tokens, options);
    format_tokens_to_usfm(&tokens)
}

/// [`format`], but every token the formatter synthesizes (inserted structural
/// linebreaks, a default `\p` after a bare chapter intro, a marker token
/// recovered from malformed text) is minted a fresh id via `minter`. See
/// [`mint_synthetic_id`] for the seam's contract; `format`/`format_tokens`
/// are unchanged and still pass no minter.
pub fn format_with_minter<T: FormattableToken + Clone>(
    tokens: &[T],
    options: FormatOptions,
    minter: &mut dyn FnMut() -> String,
) -> Vec<T> {
    let mut working = tokens.to_vec();
    format_tokens_with_minter(&mut working, options, minter);
    working
}

/// [`format_mut`] with a synthetic-id minter — see [`format_with_minter`].
pub fn format_mut_with_minter<T: FormattableToken>(
    tokens: &mut Vec<T>,
    options: FormatOptions,
    minter: &mut dyn FnMut() -> String,
) {
    format_tokens_with_minter(tokens, options, minter);
}

/// [`format_tokens`] with a synthetic-id minter — see [`format_with_minter`].
pub fn format_tokens_with_minter<T: FormattableToken>(
    tokens: &mut Vec<T>,
    options: FormatOptions,
    minter: &mut dyn FnMut() -> String,
) {
    let mut minter: Option<&mut dyn FnMut() -> String> = Some(minter);
    format_tokens_owned(tokens, options, &mut minter);
}

fn push_token_merging_text<T: FormattableToken>(tokens: &mut Vec<T>, token: T) {
    if let Some(last) = tokens.last_mut()
        && token.kind() == TokenKind::Text
        && last.kind() == TokenKind::Text
        && last.sid() == token.sid()
        && last.marker() == token.marker()
    {
        let mut text = String::with_capacity(last.text().len() + token.text().len());
        text.push_str(last.text());
        text.push_str(token.text());
        last.set_text(text);
        return;
    }

    tokens.push(token);
}

fn rewrite_tokens<T, F>(tokens: &mut Vec<T>, scratch: &mut Vec<T>, mut rewrite: F)
where
    T: FormattableToken,
    F: FnMut(&[T], &mut Vec<T>),
{
    std::mem::swap(tokens, scratch);
    tokens.clear();
    tokens.reserve(scratch.len());
    rewrite(scratch.as_slice(), tokens);
    scratch.clear();
}

fn format_tokens_owned<T: FormattableToken>(
    tokens: &mut Vec<T>,
    options: FormatOptions,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) -> FormatTimings {
    let profile = FormatTimings::default();
    let mut scratch = Vec::new();

    normalize_tokens_in_place(tokens, &mut scratch, options, &mut *minter);

    if options.bridge_consecutive_verse_markers
        || options.remove_orphan_empty_verse_before_contentful_verse
        || options.remove_bridge_verse_enumerators
    {
        normalize_verse_sequences_in_place(
            tokens,
            options.bridge_consecutive_verse_markers,
            options.remove_orphan_empty_verse_before_contentful_verse,
            options.remove_bridge_verse_enumerators,
        );
    }

    if options.move_chapter_label_after_chapter_marker
        || options.insert_default_paragraph_after_chapter_intro
    {
        if options.move_chapter_label_after_chapter_marker
            && has_movable_chapter_label(tokens.as_slice())
        {
            rewrite_tokens(tokens, &mut scratch, move_chapter_labels_after_chapter_into);
        }
        if options.insert_default_paragraph_after_chapter_intro
            && needs_default_paragraph_after_chapter_intro(tokens.as_slice())
        {
            // Inlined rather than routed through `rewrite_tokens`'s generic
            // `FnMut` closure: a closure capturing `minter` (a `&mut Option<
            // &mut dyn FnMut() -> String>`) by move would only be callable
            // once, and reborrowing it back out on every call is exactly the
            // friction `rewrite_tokens`'s single-call sites don't otherwise
            // pay for. Same swap/clear shape `rewrite_tokens` uses.
            std::mem::swap(tokens, &mut scratch);
            tokens.clear();
            tokens.reserve(scratch.len());
            insert_default_paragraph_after_chapter_intro_into(
                scratch.as_slice(),
                tokens,
                &mut *minter,
            );
            scratch.clear();
        }
    }

    if options.remove_empty_paragraphs {
        remove_empty_paragraphs_in_place(tokens);
    }

    if options.insert_structural_linebreaks {
        insert_structural_linebreaks_in_place(tokens, &mut scratch, minter);
    }

    if options.collapse_consecutive_linebreaks {
        collapse_consecutive_linebreaks_in_place(tokens);
    }

    if options.normalize_marker_whitespace_at_line_start {
        normalize_marker_whitespace_at_line_start_in_place(tokens);
    }

    profile
}

fn normalize_tokens_in_place<T: FormattableToken>(
    tokens: &mut Vec<T>,
    scratch: &mut Vec<T>,
    options: FormatOptions,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) {
    let mut input = std::mem::take(tokens)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    tokens.reserve(input.len());

    let len = input.len();
    for index in 0..len {
        let token = input[index].take().expect("token slot should be populated");
        let prev = tokens.last();
        let next = input.get(index + 1).and_then(|token| token.as_ref());
        let next_after_next = input.get(index + 2).and_then(|token| token.as_ref());

        if options.recover_malformed_markers
            && let Some(recovered) = recover_malformed_markers(&token, &mut *minter)
        {
            for recovered_token in recovered {
                push_token_merging_text(tokens, recovered_token);
            }
            continue;
        }

        let mut current = token;

        if options.ensure_inline_separators {
            current = ensure_space_between_nodes(current, prev);
        }

        if current.kind() == TokenKind::Text {
            if options.collapse_whitespace_in_text {
                current = collapse_whitespace_in_text_node(current);
            }
            if options.remove_duplicate_verse_numbers {
                current = remove_duplicate_verse_numbers(current, prev, tokens);
            }
            if options.normalize_spacing_after_paragraph_markers {
                current = normalize_spacing_after_paragraph_markers(current, prev);
            }
        }

        if current.kind() == TokenKind::Newline
            && options.remove_unwanted_linebreaks
            && should_remove_unwanted_linebreak(prev, next, tokens, next_after_next)
        {
            continue;
        }

        push_token_merging_text(tokens, current);
    }

    scratch.clear();
}

fn insert_structural_linebreaks_in_place<T: FormattableToken>(
    tokens: &mut Vec<T>,
    scratch: &mut Vec<T>,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) {
    std::mem::swap(tokens, scratch);
    tokens.clear();
    tokens.reserve(scratch.len().saturating_mul(2));

    let len = scratch.len();
    for index in 0..len {
        let token = std::mem::replace(
            &mut scratch[index],
            T::synthetic_like(None, TokenKind::Text, String::new(), None, None),
        );
        let next_in = scratch.get(index + 1);
        let prev_out = tokens.last();

        if token.kind() == TokenKind::Marker
            && token
                .marker()
                .is_some_and(|marker| linebreak_before_marker_token::<T>(&token, marker))
            && prev_out.is_some()
            && !prev_out.is_some_and(|t| t.kind() == TokenKind::Newline)
        {
            tokens.push(new_newline_like(&token, &mut *minter));
        }

        let kind = token.kind();
        let needs_newline_after = if kind == TokenKind::Marker {
            if let Some(marker) = token.marker() {
                if linebreak_before_if_next_marker_token(&token, marker) {
                    next_in.is_some_and(|t| t.kind() == TokenKind::Marker)
                        && !next_in.is_some_and(|t| t.kind() == TokenKind::Newline)
                } else {
                    linebreak_before_and_after_marker_token(&token, marker)
                        && !next_in.is_some_and(|t| t.kind() == TokenKind::Newline)
                }
            } else {
                false
            }
        } else {
            kind == TokenKind::Number
                && number_belongs_to_marker(scratch.as_slice(), index, "c")
                && !next_in.is_some_and(|t| t.kind() == TokenKind::Newline)
        };

        tokens.push(token);

        if needs_newline_after {
            let anchor = tokens.last().expect("pushed token should exist");
            tokens.push(new_newline_like(anchor, &mut *minter));
        }
    }

    scratch.clear();
}

fn recover_malformed_markers<T: FormattableToken>(
    token: &T,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) -> Option<Vec<T>> {
    if token.kind() != TokenKind::Text {
        return None;
    }

    let text = token.text();
    let slash_index = text.find('\\')?;
    let mut chars = text[slash_index + 1..].chars().peekable();
    let mut marker = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
            marker.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if marker.is_empty() {
        return None;
    }
    let rest = &text[slash_index + 1 + marker.len()..];
    let first_rest = rest.chars().next()?;
    if !matches!(first_rest, ' ' | '\t') {
        return None;
    }
    if lookup_marker(&marker).kind == MarkerKind::Unknown {
        return None;
    }

    // Splitting one token into several must not duplicate its identity onto
    // every fragment: the original id (if any) goes to whichever fragment
    // is emitted first — mirroring the same "first replacement reuses the
    // target's own clone" rule `build_replacement_tokens` already applies —
    // and every fragment after that is synthetic and gets minted, never a
    // second copy of an id something else in the pipeline may already be
    // addressing.
    let mut original_id = token.id().map(ToOwned::to_owned);
    let mut out = Vec::new();

    if slash_index > 0 {
        let mut prefix = T::synthetic_like(
            Some(token),
            TokenKind::Text,
            text[..slash_index].to_string(),
            None,
            token.sid().map(ToOwned::to_owned),
        );
        if let Some(id) = original_id.take() {
            prefix.set_id(id);
        }
        out.push(prefix);
    }

    let mut recovered_marker = T::synthetic_like(
        Some(token),
        TokenKind::Marker,
        format!("\\{marker}"),
        Some(marker.clone()),
        token.sid().map(ToOwned::to_owned),
    );
    match original_id.take() {
        Some(id) => recovered_marker.set_id(id),
        None => mint_synthetic_id(&mut recovered_marker, minter),
    }
    out.push(recovered_marker);

    if rest.len() > 1 {
        let mut suffix = T::synthetic_like(
            Some(token),
            TokenKind::Text,
            rest[1..].to_string(),
            None,
            token.sid().map(ToOwned::to_owned),
        );
        mint_synthetic_id(&mut suffix, minter);
        out.push(suffix);
    }

    Some(out)
}

fn ensure_space_between_nodes<T: FormattableToken>(mut token: T, prev: Option<&T>) -> T {
    if token.kind() == TokenKind::Newline {
        return token;
    }
    let Some(prev) = prev else {
        return token;
    };
    if prev.kind() == TokenKind::Newline {
        return token;
    }
    if !is_text_like(prev.kind()) || !is_text_like(token.kind()) {
        return token;
    }
    if is_protected_whitespace_boundary(prev, &token) {
        return token;
    }

    if !ends_with_whitespace(prev.text()) && !starts_with_whitespace(token.text()) {
        let mut text = String::with_capacity(token.text().len() + 1);
        text.push(' ');
        text.push_str(token.text());
        token.set_text(text);
    }
    token
}

fn collapse_whitespace_in_text_node<T: FormattableToken>(mut token: T) -> T {
    let Some(collapsed) = collapse_horizontal_whitespace(token.text()) else {
        return token;
    };
    token.set_text(collapsed);
    token
}

fn collapse_horizontal_whitespace(text: &str) -> Option<String> {
    let mut output = String::with_capacity(text.len());
    let mut previous_was_horizontal_ws = false;
    let mut changed = false;

    for ch in text.chars() {
        match ch {
            '\t' => {
                changed = true;
                if !previous_was_horizontal_ws {
                    output.push(' ');
                }
                previous_was_horizontal_ws = true;
            }
            ' ' => {
                if previous_was_horizontal_ws {
                    changed = true;
                } else {
                    output.push(' ');
                    previous_was_horizontal_ws = true;
                }
            }
            _ => {
                previous_was_horizontal_ws = false;
                output.push(ch);
            }
        }
    }

    changed.then_some(output)
}

fn remove_duplicate_verse_numbers<T: FormattableToken>(
    mut token: T,
    prev: Option<&T>,
    cleaned: &[T],
) -> T {
    let Some(prev) = prev else {
        return token;
    };
    if prev.kind() != TokenKind::Number {
        return token;
    }
    if !number_belongs_to_marker(cleaned, cleaned.len().saturating_sub(1), "v") {
        return token;
    }

    let verse_number = prev.text().trim();
    if verse_number.is_empty() {
        return token;
    }

    let trimmed_start = token.text().trim_start_matches([' ', '\t']);
    if let Some(remainder) = trimmed_start.strip_prefix(verse_number) {
        let leading_len = token.text().len() - trimmed_start.len();
        let leading = &token.text()[..leading_len];
        let mut text = String::with_capacity(leading.len() + remainder.len());
        text.push_str(leading);
        text.push_str(remainder);
        token.set_text(text);
    }
    token
}

fn normalize_spacing_after_paragraph_markers<T: FormattableToken>(
    mut token: T,
    prev: Option<&T>,
) -> T {
    let Some(prev) = prev else {
        return token;
    };
    if prev.kind() != TokenKind::Marker {
        return token;
    }
    let Some(marker) = prev.marker() else {
        return token;
    };
    if !linebreak_before_marker_token(prev, marker) {
        return token;
    }

    let rest = token.text().trim_start_matches(' ');
    if rest.len() != token.text().len() {
        let mut text = String::with_capacity(rest.len() + 1);
        text.push(' ');
        text.push_str(rest);
        token.set_text(text);
    }
    token
}

fn should_remove_unwanted_linebreak<T: FormattableToken>(
    prev: Option<&T>,
    next: Option<&T>,
    cleaned: &[T],
    next_after_next: Option<&T>,
) -> bool {
    let prev_marker = prev
        .filter(|token| token.kind() == TokenKind::Marker)
        .and_then(|token| token.marker());
    let next_is_marker = next.is_some_and(|token| token.kind() == TokenKind::Marker);
    let next_marker = next
        .filter(|token| token.kind() == TokenKind::Marker)
        .and_then(|t| t.marker());

    if let Some(marker) = prev_marker {
        if linebreak_before_and_after_marker(marker) {
            return false;
        }
        if linebreak_before_if_next_marker(marker) {
            return !next_is_marker;
        }
        if linebreak_before_marker(marker) {
            return true;
        }
    }

    if next_marker == Some("v") {
        if let Some(prev) = prev
            && prev.kind() == TokenKind::Number
            && number_belongs_to_marker(cleaned, cleaned.len().saturating_sub(1), "c")
        {
            return false;
        }
        if next_after_next.is_some_and(|token| token.kind() == TokenKind::Number) {
            return true;
        }
    }

    false
}

fn normalize_verse_sequences_in_place<T: FormattableToken>(
    tokens: &mut Vec<T>,
    enable_bridge: bool,
    enable_orphan_cleanup: bool,
    enable_enumerator_cleanup: bool,
) {
    let mut index = 0usize;
    while index + 1 < tokens.len() {
        if !is_immediate_verse_pair(tokens, index) {
            index += 1;
            continue;
        }

        if enable_bridge && bridge_verse_run(tokens, index) {
            if enable_enumerator_cleanup {
                cleanup_bridge_enumerator_at(tokens, index);
            }
            continue;
        }

        if enable_orphan_cleanup
            && let Some(next_marker_index) = orphan_next_marker_index(tokens, index)
        {
            tokens.drain(index..next_marker_index);
            continue;
        }

        if enable_enumerator_cleanup {
            cleanup_bridge_enumerator_at(tokens, index);
        }

        index += 1;
    }
}

fn is_immediate_verse_pair<T: FormattableToken>(tokens: &[T], index: usize) -> bool {
    tokens
        .get(index)
        .is_some_and(|token| token.kind() == TokenKind::Marker && token.marker() == Some("v"))
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.kind() == TokenKind::Number)
}

fn bridge_verse_run<T: FormattableToken>(tokens: &mut Vec<T>, index: usize) -> bool {
    let Some(first_verse) = tokens.get(index + 1).and_then(|token| {
        token
            .number_info()
            .map(|(start, _, _)| start)
            .or_else(|| parse_plain_verse(token.text()))
    }) else {
        return false;
    };

    let mut end_verse = first_verse;
    let mut scan = index + 2;

    while scan + 1 < tokens.len() {
        let mut candidate_marker_index = scan;
        while candidate_marker_index < tokens.len()
            && tokens[candidate_marker_index].kind() == TokenKind::Text
            && tokens[candidate_marker_index].text().trim().is_empty()
        {
            candidate_marker_index += 1;
        }

        if !is_immediate_verse_pair(tokens, candidate_marker_index) {
            break;
        }

        let Some(next_verse) = tokens.get(candidate_marker_index + 1).and_then(|token| {
            token
                .number_info()
                .map(|(start, _, _)| start)
                .or_else(|| parse_plain_verse(token.text()))
        }) else {
            break;
        };
        if next_verse != end_verse + 1 {
            break;
        }

        end_verse = next_verse;
        scan = candidate_marker_index + 2;
    }

    if end_verse == first_verse {
        return false;
    }

    let range = bridge_range_string(first_verse, end_verse);
    let updated = with_original_spacing(tokens[index + 1].text(), &range);
    tokens[index + 1].set_text(updated);
    tokens.drain(index + 2..scan);
    true
}

fn orphan_next_marker_index<T: FormattableToken>(tokens: &[T], index: usize) -> Option<usize> {
    let mut next_marker_index = index + 2;
    while next_marker_index < tokens.len()
        && tokens[next_marker_index].kind() == TokenKind::Text
        && tokens[next_marker_index].text().trim().is_empty()
    {
        next_marker_index += 1;
    }

    if !is_immediate_verse_pair(tokens, next_marker_index) {
        return None;
    }

    let next_text = tokens.get(next_marker_index + 2)?;
    if next_text.kind() == TokenKind::Text && !next_text.text().trim().is_empty() {
        Some(next_marker_index)
    } else {
        None
    }
}

fn cleanup_bridge_enumerator_at<T: FormattableToken>(tokens: &mut [T], index: usize) {
    if !is_immediate_verse_pair(tokens, index) {
        return;
    }
    let Some(range_token) = tokens.get(index + 1) else {
        return;
    };
    let Some(next) = tokens.get(index + 2) else {
        return;
    };
    if next.kind() != TokenKind::Text {
        return;
    }
    let Some((start, end)) = parse_bridge_range(range_token.text()) else {
        return;
    };
    let updated = strip_bridge_enumerators(next.text(), start, end);
    if updated != next.text() {
        tokens[index + 2].set_text(updated);
    }
}

fn insert_default_paragraph_after_chapter_intro_into<T: FormattableToken>(
    tokens: &[T],
    out: &mut Vec<T>,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) {
    let mut in_chapter_intro = false;
    let mut saw_para_marker_in_intro = false;
    let mut saw_chapter_marker = false;
    let mut saw_chapter_number = false;

    for token in tokens {
        let is_chapter_marker = token.kind() == TokenKind::Marker && token.marker() == Some("c");
        let is_verse_marker = token.kind() == TokenKind::Marker && token.marker() == Some("v");
        let is_paragraph_marker = token.kind() == TokenKind::Marker
            && token
                .marker()
                .is_some_and(|marker| is_valid_paragraph_or_heading_marker_token(token, marker));

        if is_chapter_marker {
            saw_chapter_marker = true;
            saw_chapter_number = false;
            in_chapter_intro = false;
            saw_para_marker_in_intro = false;
            out.push(token.clone());
            continue;
        }

        if saw_chapter_marker && !saw_chapter_number {
            if token.kind() == TokenKind::Number {
                saw_chapter_number = true;
            }
            out.push(token.clone());
            continue;
        }

        if saw_chapter_marker && saw_chapter_number && !in_chapter_intro {
            in_chapter_intro = true;
        }

        if in_chapter_intro {
            if is_paragraph_marker {
                saw_para_marker_in_intro = true;
            }

            if is_verse_marker && !saw_para_marker_in_intro {
                let mut default_paragraph = T::synthetic_like(
                    Some(token),
                    TokenKind::Marker,
                    "\\p".to_string(),
                    Some("p".to_string()),
                    token.sid().map(ToOwned::to_owned),
                );
                mint_synthetic_id(&mut default_paragraph, &mut *minter);
                out.push(default_paragraph);
                saw_para_marker_in_intro = true;
            }

            if is_verse_marker {
                in_chapter_intro = false;
            }
        }

        out.push(token.clone());
    }
}

fn has_movable_chapter_label<T: FormattableToken>(tokens: &[T]) -> bool {
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind() != TokenKind::Marker || token.marker() != Some("cl") {
            index += 1;
            continue;
        }

        let mut chapter_marker_index = index + 1;
        while chapter_marker_index < tokens.len() {
            let probe = &tokens[chapter_marker_index];
            match probe.kind() {
                TokenKind::Newline | TokenKind::Text => chapter_marker_index += 1,
                TokenKind::Marker if probe.marker() == Some("c") => return true,
                _ => break,
            }
        }

        index += 1;
    }

    false
}

fn needs_default_paragraph_after_chapter_intro<T: FormattableToken>(tokens: &[T]) -> bool {
    let mut saw_chapter_marker = false;
    let mut saw_chapter_number = false;
    let mut in_chapter_intro = false;
    let mut saw_para_marker_in_intro = false;

    for token in tokens {
        let is_chapter_marker = token.kind() == TokenKind::Marker && token.marker() == Some("c");
        let is_verse_marker = token.kind() == TokenKind::Marker && token.marker() == Some("v");
        let is_paragraph_marker = token.kind() == TokenKind::Marker
            && token
                .marker()
                .is_some_and(|marker| is_valid_paragraph_or_heading_marker_token(token, marker));

        if is_chapter_marker {
            saw_chapter_marker = true;
            saw_chapter_number = false;
            in_chapter_intro = false;
            saw_para_marker_in_intro = false;
            continue;
        }

        if saw_chapter_marker && !saw_chapter_number {
            if token.kind() == TokenKind::Number {
                saw_chapter_number = true;
            }
            continue;
        }

        if saw_chapter_marker && saw_chapter_number && !in_chapter_intro {
            in_chapter_intro = true;
        }

        if !in_chapter_intro {
            continue;
        }

        if is_paragraph_marker {
            saw_para_marker_in_intro = true;
            continue;
        }

        if is_verse_marker {
            return !saw_para_marker_in_intro;
        }
    }

    false
}

fn move_chapter_labels_after_chapter_into<T: FormattableToken>(tokens: &[T], out: &mut Vec<T>) {
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        let is_chapter_label = token.kind() == TokenKind::Marker && token.marker() == Some("cl");
        if !is_chapter_label {
            out.push(token.clone());
            index += 1;
            continue;
        }

        let mut chapter_marker_index = index + 1;
        let mut movable = true;
        while chapter_marker_index < tokens.len() {
            let probe = &tokens[chapter_marker_index];
            match probe.kind() {
                TokenKind::Newline | TokenKind::Text => chapter_marker_index += 1,
                TokenKind::Marker if probe.marker() == Some("c") => break,
                _ => {
                    movable = false;
                    break;
                }
            }
        }

        if !movable || chapter_marker_index >= tokens.len() {
            out.push(token.clone());
            index += 1;
            continue;
        }

        let mut chapter_block_end = chapter_marker_index + 1;
        if chapter_block_end < tokens.len() {
            let probe = &tokens[chapter_block_end];
            if probe.kind() == TokenKind::Number {
                chapter_block_end += 1;
            }
        }

        out.extend(
            tokens[chapter_marker_index..chapter_block_end]
                .iter()
                .cloned(),
        );
        out.extend(tokens[index..chapter_marker_index].iter().cloned());
        index = chapter_block_end;
    }
}

fn collapse_consecutive_linebreaks_in_place<T: FormattableToken>(tokens: &mut Vec<T>) {
    let mut write = 0usize;
    let mut previous_was_linebreak = false;

    for read in 0..tokens.len() {
        let is_linebreak = tokens[read].kind() == TokenKind::Newline;
        if is_linebreak && previous_was_linebreak {
            continue;
        }
        if write != read {
            tokens.swap(write, read);
        }
        previous_was_linebreak = is_linebreak;
        write += 1;
    }

    tokens.truncate(write);
}

fn normalize_marker_whitespace_at_line_start_in_place<T: FormattableToken>(tokens: &mut [T]) {
    for index in 0..tokens.len() {
        if tokens[index].kind() != TokenKind::Marker {
            continue;
        }
        let at_line_start = index == 0 || tokens[index - 1].kind() == TokenKind::Newline;
        if !at_line_start {
            continue;
        }
        let trimmed = tokens[index].text().trim_start();
        if trimmed.len() == tokens[index].text().len() {
            continue;
        }
        tokens[index].set_text(trimmed.to_string());
    }
}

fn new_newline_like<T: FormattableToken>(
    anchor: &T,
    minter: &mut Option<&mut dyn FnMut() -> String>,
) -> T {
    let mut newline = T::synthetic_like(
        Some(anchor),
        TokenKind::Newline,
        "\n".to_string(),
        None,
        anchor.sid().map(ToOwned::to_owned),
    );
    mint_synthetic_id(&mut newline, minter);
    newline
}

fn is_text_like(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Marker | TokenKind::EndMarker | TokenKind::Number | TokenKind::Text
    )
}

fn is_protected_whitespace_boundary<T: FormattableToken>(prev: &T, curr: &T) -> bool {
    is_char_or_note_markerish(prev) || is_char_or_note_markerish(curr)
}

fn is_char_or_note_markerish<T: FormattableToken>(token: &T) -> bool {
    if let Some(profile) = token.marker_profile() {
        return profile.is_char_or_note_like;
    }
    if !matches!(
        token.kind(),
        TokenKind::Marker | TokenKind::EndMarker | TokenKind::Milestone | TokenKind::MilestoneEnd
    ) {
        return false;
    }
    let Some(marker) = token.marker() else {
        return false;
    };
    matches!(
        token.structural().map(|info| info.scope_kind),
        Some(
            crate::marker_defs::StructuralScopeKind::Character
                | crate::marker_defs::StructuralScopeKind::Note
                | crate::marker_defs::StructuralScopeKind::Milestone
        )
    ) || matches!(
        lookup_marker(marker).kind,
        MarkerKind::Character
            | MarkerKind::Note
            | MarkerKind::MilestoneStart
            | MarkerKind::MilestoneEnd
    )
}

fn linebreak_before_and_after_marker(marker: &str) -> bool {
    contains_marker(LINEBREAK_BEFORE_AND_AFTER_MARKERS, marker)
        || unknown_marker_defaults_to_own_line(marker)
}

fn linebreak_before_and_after_marker_token<T: FormattableToken>(token: &T, marker: &str) -> bool {
    token
        .marker_profile()
        .map(|profile| profile.linebreak_behavior == LinebreakBehavior::BeforeAndAfter)
        .unwrap_or_else(|| linebreak_before_and_after_marker(marker))
}

fn linebreak_before_if_next_marker(marker: &str) -> bool {
    contains_marker(POETRY_MARKERS, marker)
}

fn linebreak_before_if_next_marker_token<T: FormattableToken>(token: &T, marker: &str) -> bool {
    token
        .marker_profile()
        .map(|profile| profile.linebreak_behavior == LinebreakBehavior::BeforeIfNextMarker)
        .unwrap_or_else(|| linebreak_before_if_next_marker(marker))
}

fn linebreak_before_marker(marker: &str) -> bool {
    linebreak_before_and_after_marker(marker)
        || contains_marker(LINEBREAK_BEFORE_ONLY_MARKERS, marker)
        || linebreak_before_if_next_marker(marker)
}

fn linebreak_before_marker_token<T: FormattableToken>(token: &T, marker: &str) -> bool {
    token
        .marker_profile()
        .map(|profile| profile.linebreak_behavior != LinebreakBehavior::None)
        .unwrap_or_else(|| linebreak_before_marker(marker))
}

fn contains_marker(markers: &[&str], marker: &str) -> bool {
    markers.contains(&marker)
}

fn is_empty_paragraph_candidate(marker: &str) -> bool {
    matches!(
        marker,
        "p" | "m"
            | "po"
            | "pr"
            | "cls"
            | "pmo"
            | "pm"
            | "pmc"
            | "pmr"
            | "pi"
            | "pi1"
            | "pi2"
            | "pi3"
            | "mi"
            | "nb"
            | "pc"
            | "ph"
            | "ph1"
            | "ph2"
            | "ph3"
            | "b"
            | "pb"
            | "q"
            | "q1"
            | "q2"
            | "q3"
            | "q4"
            | "qr"
            | "qc"
            | "qa"
            | "qm"
            | "qm1"
            | "qm2"
            | "qm3"
            | "qd"
            | "lh"
            | "li"
            | "li1"
            | "li2"
            | "li3"
            | "li4"
            | "lf"
            | "lim"
            | "lim1"
            | "lim2"
            | "lim3"
    )
}

fn is_empty_paragraph_boundary_marker(marker: &str) -> bool {
    if is_empty_paragraph_candidate(marker) {
        return true;
    }
    matches!(
        lookup_marker(marker).kind,
        MarkerKind::Header
            | MarkerKind::Chapter
            | MarkerKind::Periph
            | MarkerKind::SidebarStart
            | MarkerKind::TableRow
            | MarkerKind::Unknown
    )
}

fn is_empty_paragraph_boundary_marker_token<T: FormattableToken>(token: &T, marker: &str) -> bool {
    token
        .marker_profile()
        .map(|profile| profile.empty_paragraph_boundary)
        .unwrap_or_else(|| is_empty_paragraph_boundary_marker(marker))
}

fn remove_empty_paragraphs_in_place<T: FormattableToken>(tokens: &mut Vec<T>) {
    let mut write = 0usize;
    let mut read = 0usize;

    while read < tokens.len() {
        let token = &tokens[read];
        let Some(marker) = (token.kind() == TokenKind::Marker)
            .then(|| token.marker())
            .flatten()
        else {
            if write != read {
                tokens.swap(write, read);
            }
            write += 1;
            read += 1;
            continue;
        };

        if !token
            .marker_profile()
            .map(|profile| profile.empty_paragraph_candidate)
            .unwrap_or_else(|| is_empty_paragraph_candidate(marker))
        {
            if write != read {
                tokens.swap(write, read);
            }
            write += 1;
            read += 1;
            continue;
        }

        let mut probe = read + 1;
        let mut remove_until = None;
        while probe < tokens.len() {
            let next = &tokens[probe];
            match next.kind() {
                TokenKind::Newline | TokenKind::OptBreak => probe += 1,
                TokenKind::Text if next.text().trim().is_empty() => probe += 1,
                TokenKind::Marker
                    if next.marker().is_some_and(|marker| {
                        is_empty_paragraph_boundary_marker_token(next, marker)
                    }) =>
                {
                    remove_until = Some(probe);
                    break;
                }
                _ => break,
            }
        }

        if let Some(next_boundary) = remove_until {
            read = next_boundary;
            continue;
        }

        if write != read {
            tokens.swap(write, read);
        }
        write += 1;
        read += 1;
    }

    tokens.truncate(write);
}

fn unknown_marker_defaults_to_own_line(marker: &str) -> bool {
    !marker.starts_with('z') && lookup_marker(marker).kind == MarkerKind::Unknown
}

fn is_valid_paragraph_or_heading_marker(marker: &str) -> bool {
    matches!(
        lookup_marker(marker).kind,
        MarkerKind::Paragraph | MarkerKind::Header | MarkerKind::Meta
    )
}

fn is_valid_paragraph_or_heading_marker_token<T: FormattableToken>(
    token: &T,
    marker: &str,
) -> bool {
    token
        .marker_profile()
        .map(|profile| profile.valid_paragraph_or_heading)
        .unwrap_or_else(|| is_valid_paragraph_or_heading_marker(marker))
}

fn parse_plain_verse(text: &str) -> Option<u32> {
    let trimmed = text.trim();
    if !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    trimmed.parse().ok()
}

fn bridge_range_string(start: u32, end: u32) -> String {
    format!("{start}-{end}")
}

fn with_original_spacing(original: &str, normalized: &str) -> String {
    let leading_len = original.len() - original.trim_start().len();
    let trailing_len = original.len() - original.trim_end().len();
    let leading = &original[..leading_len];
    let trailing = &original[original.len() - trailing_len..];
    let mut text = String::with_capacity(leading.len() + normalized.len() + trailing.len());
    text.push_str(leading);
    text.push_str(normalized);
    text.push_str(trailing);
    text
}

fn parse_bridge_range(text: &str) -> Option<(u32, u32)> {
    let trimmed = text.trim();
    let (start, end) = trimmed.split_once('-')?;
    let start: u32 = start.trim().parse().ok()?;
    let end: u32 = end.trim().parse().ok()?;
    (start <= end).then_some((start, end))
}

fn strip_bridge_enumerators(text: &str, start: u32, end: u32) -> String {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut last_copied = 0usize;
    let mut output = String::with_capacity(text.len());

    while index < bytes.len() {
        let current = bytes[index];
        let at_boundary = index == 0 || current_boundary_byte(bytes[index - 1]);
        if !at_boundary || !current.is_ascii_digit() {
            index += 1;
            continue;
        }

        let digit_start = index;
        let mut verse_num = 0u32;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            verse_num = verse_num
                .saturating_mul(10)
                .saturating_add((bytes[index] - b'0') as u32);
            index += 1;
        }

        let mut after_digits = index;
        while after_digits < bytes.len() && bytes[after_digits].is_ascii_whitespace() {
            after_digits += 1;
        }
        if after_digits >= bytes.len() || !is_enumerator_punctuation(bytes[after_digits] as char) {
            index = after_digits;
            continue;
        }

        let mut after_enum = after_digits + 1;
        while after_enum < bytes.len() && bytes[after_enum].is_ascii_whitespace() {
            after_enum += 1;
        }

        if verse_num >= start && verse_num <= end {
            output.push_str(&text[last_copied..digit_start]);
            last_copied = after_enum;
        }

        index = after_enum;
    }

    if last_copied == 0 {
        return text.to_string();
    }

    output.push_str(&text[last_copied..]);
    output
}

fn current_boundary_byte(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'('
}

fn is_enumerator_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
            | '-'
    )
}

fn number_belongs_to_marker<T: FormattableToken>(tokens: &[T], index: usize, marker: &str) -> bool {
    if index == 0 {
        return false;
    }
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        match tokens[cursor].kind() {
            TokenKind::Newline => continue,
            TokenKind::Marker => return tokens[cursor].marker() == Some(marker),
            _ => return false,
        }
    }
    false
}

fn starts_with_whitespace(text: &str) -> bool {
    text.chars().next().is_some_and(char::is_whitespace)
}

fn ends_with_whitespace(text: &str) -> bool {
    text.chars().last().is_some_and(char::is_whitespace)
}

fn build_marker_profile(
    marker: &str,
    kind: TokenKind,
    structural: Option<StructuralMarkerInfo>,
) -> FormatMarkerProfile {
    let looked_up_kind = lookup_marker(marker).kind;
    let is_char_or_note_like = matches!(
        structural.map(|info| info.scope_kind),
        Some(
            crate::marker_defs::StructuralScopeKind::Character
                | crate::marker_defs::StructuralScopeKind::Note
                | crate::marker_defs::StructuralScopeKind::Milestone
        )
    ) || matches!(
        looked_up_kind,
        MarkerKind::Character
            | MarkerKind::Note
            | MarkerKind::MilestoneStart
            | MarkerKind::MilestoneEnd
    );

    let linebreak_behavior = if contains_marker(LINEBREAK_BEFORE_AND_AFTER_MARKERS, marker)
        || (!marker.starts_with('z') && looked_up_kind == MarkerKind::Unknown)
    {
        LinebreakBehavior::BeforeAndAfter
    } else if contains_marker(POETRY_MARKERS, marker) {
        LinebreakBehavior::BeforeIfNextMarker
    } else if contains_marker(LINEBREAK_BEFORE_ONLY_MARKERS, marker) {
        LinebreakBehavior::BeforeOnly
    } else {
        LinebreakBehavior::None
    };

    let empty_paragraph_candidate = matches!(
        marker,
        "p" | "m"
            | "po"
            | "pr"
            | "cls"
            | "pmo"
            | "pm"
            | "pmc"
            | "pmr"
            | "pi"
            | "pi1"
            | "pi2"
            | "pi3"
            | "mi"
            | "nb"
            | "pc"
            | "ph"
            | "ph1"
            | "ph2"
            | "ph3"
            | "b"
            | "pb"
            | "q"
            | "q1"
            | "q2"
            | "q3"
            | "q4"
            | "qr"
            | "qc"
            | "qa"
            | "qm"
            | "qm1"
            | "qm2"
            | "qm3"
            | "qd"
            | "lh"
            | "li"
            | "li1"
            | "li2"
            | "li3"
            | "li4"
            | "lf"
            | "lim"
            | "lim1"
            | "lim2"
            | "lim3"
    );

    let empty_paragraph_boundary = empty_paragraph_candidate
        || matches!(
            looked_up_kind,
            MarkerKind::Header
                | MarkerKind::Chapter
                | MarkerKind::Periph
                | MarkerKind::SidebarStart
                | MarkerKind::TableRow
                | MarkerKind::Unknown
        );

    let valid_paragraph_or_heading = matches!(
        looked_up_kind,
        MarkerKind::Paragraph | MarkerKind::Header | MarkerKind::Meta
    );

    let _ = kind;
    FormatMarkerProfile {
        is_char_or_note_like,
        linebreak_behavior,
        empty_paragraph_candidate,
        empty_paragraph_boundary,
        valid_paragraph_or_heading,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct EditorToken {
        kind: TokenKind,
        text: String,
        marker: Option<String>,
        sid: Option<String>,
        id: String,
        lane: u8,
    }

    impl FormattableToken for EditorToken {
        fn id(&self) -> Option<&str> {
            Some(&self.id)
        }

        fn set_id(&mut self, id: String) {
            self.id = id;
        }

        fn kind(&self) -> TokenKind {
            self.kind
        }

        fn set_kind(&mut self, kind: TokenKind) {
            self.kind = kind;
        }

        fn text(&self) -> &str {
            &self.text
        }

        fn set_text(&mut self, text: String) {
            self.text = text;
        }

        fn marker(&self) -> Option<&str> {
            self.marker.as_deref()
        }

        fn set_marker(&mut self, marker: Option<String>) {
            self.marker = marker;
        }

        fn sid(&self) -> Option<&str> {
            self.sid.as_deref()
        }

        fn set_sid(&mut self, sid: Option<String>) {
            self.sid = sid;
        }

        fn synthetic_like(
            anchor: Option<&Self>,
            kind: TokenKind,
            text: String,
            marker: Option<String>,
            sid: Option<String>,
        ) -> Self {
            let lane = anchor.map(|token| token.lane).unwrap_or(0);
            Self {
                kind,
                text,
                marker,
                sid,
                id: String::new(),
                lane,
            }
        }
    }

    fn token(kind: TokenKind, text: &str, marker: Option<&str>) -> EditorToken {
        EditorToken {
            kind,
            text: text.to_string(),
            marker: marker.map(ToOwned::to_owned),
            sid: None,
            id: String::new(),
            lane: 1,
        }
    }

    #[test]
    fn preserves_unknown_metadata() {
        let tokens = vec![EditorToken {
            kind: TokenKind::Text,
            text: String::new(),
            marker: None,
            sid: None,
            id: "custom".to_string(),
            lane: 7,
        }];

        let result = format(&tokens, FormatOptions::default());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].lane, 7);
        assert_eq!(result[0].id, "custom");
    }

    #[test]
    fn default_format_bridges_consecutive_verse_markers_into_range() {
        let tokens = vec![
            token(TokenKind::Marker, "\\v", Some("v")),
            token(TokenKind::Number, "1", None),
            token(TokenKind::Marker, "\\v", Some("v")),
            token(TokenKind::Number, "2", None),
            token(TokenKind::Marker, "\\v", Some("v")),
            token(TokenKind::Number, "3", None),
            token(TokenKind::Text, "  asdf asdf", None),
        ];

        let result = format(&tokens, FormatOptions::default());

        assert_eq!(result.len(), 3);
        assert_eq!(result[1].text, " 1-3");
        assert_eq!(result[2].text, " asdf asdf");
    }

    #[test]
    fn remove_empty_paragraphs_is_rule_gated() {
        let tokens = vec![
            token(TokenKind::Marker, "\\p", Some("p")),
            token(TokenKind::Newline, "\n", None),
            token(TokenKind::Marker, "\\c", Some("c")),
        ];

        let result = format(
            &tokens,
            FormatOptions::only(&[FormatRule::RemoveEmptyParagraphs]),
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].marker(), Some("c"));
    }

    #[test]
    fn format_token_from_parse_tokens_roundtrips_to_usfm() {
        let source = "\\c 1\\cl Chapter 1\n\\v 1  Text";
        let parsed = parse(source);
        let mut tokens = into_format_tokens(&parsed.tokens);
        format_tokens(&mut tokens, FormatOptions::default());
        let output = format_tokens_to_usfm(&tokens);
        assert!(output.contains("\\c 1"));
        assert!(output.contains("\\cl Chapter 1"));
        assert!(output.contains("\\v 1"));
    }

    /// The F1 gate: `\w gracious|lemma="grace" \w*` must survive a full
    /// `format` pass. Before `FormatToken` carried `attribute_source`, this
    /// silently emitted `\w gracious\w*` — the attribute list vanished
    /// because it was never in any field `format_tokens_to_usfm` read, not
    /// because any rule deliberately stripped it.
    #[test]
    fn format_preserves_attribute_bearing_tokens_round_trip() {
        let source =
            "\\id GEN\n\\c 1\n\\p\n\\v 2 the second verse \\w gracious|lemma=\"grace\" \\w*";
        let parsed = parse(source);
        let mut tokens = into_format_tokens(&parsed.tokens);
        assert!(
            tokens.iter().any(|token| token.attribute_source.is_some()),
            "into_format_tokens must actually carry the attribute list, or this test proves nothing"
        );
        format_tokens(&mut tokens, FormatOptions::default());
        let output = format_tokens_to_usfm(&tokens);
        assert!(
            output.contains("\\w gracious|lemma=\"grace\" \\w*"),
            "attribute list must survive a format pass: {output:?}"
        );
    }

    /// Same fixture, but every token is id-less (as `into_format_tokens`
    /// itself already produces) — attribute passthrough must not depend on
    /// id presence, since the two are unrelated fields on `FormatToken`.
    #[test]
    fn format_preserves_attributes_on_id_less_tokens() {
        let source = "\\w gracious|lemma=\"grace\" \\w*";
        let parsed = parse(source);
        let tokens = into_format_tokens(&parsed.tokens);
        assert!(tokens.iter().all(|token| token.id.is_none()));
        let mut tokens = tokens;
        format_tokens(&mut tokens, FormatOptions::default());
        assert_eq!(
            format_tokens_to_usfm(&tokens),
            "\\w gracious|lemma=\"grace\" \\w*"
        );
    }

    /// Same fixture, but every token is minted an id first (the C1 seam) —
    /// attribute passthrough must not depend on id *absence* either.
    #[test]
    fn format_preserves_attributes_on_id_bearing_tokens() {
        let source = "\\w gracious|lemma=\"grace\" \\w*";
        let parsed = parse(source);
        let mut tokens = into_format_tokens(&parsed.tokens);
        for (index, token) in tokens.iter_mut().enumerate() {
            token.set_id(format!("synthetic-{index}"));
        }
        assert!(tokens.iter().all(|token| token.id.is_some()));
        format_tokens(&mut tokens, FormatOptions::default());
        assert_eq!(
            format_tokens_to_usfm(&tokens),
            "\\w gracious|lemma=\"grace\" \\w*"
        );
    }

    /// Tokens the formatter itself synthesizes (inserted structural
    /// linebreaks, a default `\p`) are new content and must never invent an
    /// attribute list — `synthetic_like`'s `attribute_source: None` is the
    /// whole rule, with no per-call-site bookkeeping needed. Proven by
    /// count: this fixture has none to begin with, format adds tokens, and
    /// the attribute-bearing count must stay exactly zero throughout.
    #[test]
    fn synthesized_tokens_never_carry_an_attribute_list() {
        let source = "\\id GEN\n\\c 1\\v 1 In the beginning.";
        let parsed = parse(source);
        let mut tokens = into_format_tokens(&parsed.tokens);
        assert!(tokens.iter().all(|token| token.attribute_source.is_none()));
        let original_count = tokens.len();
        format_tokens(&mut tokens, FormatOptions::all_enabled());
        assert!(
            tokens.len() > original_count,
            "this fixture must actually synthesize tokens, or the test proves nothing"
        );
        assert!(
            tokens.iter().all(|token| token.attribute_source.is_none()),
            "no synthesized token may invent an attribute list"
        );
    }

    #[test]
    fn format_rule_has_stable_machine_identifiers() {
        let rule = FormatRule::InsertDefaultParagraphAfterChapterIntro;
        assert_eq!(rule.code(), "insert-default-paragraph-after-chapter-intro");
        assert_eq!(
            rule.label_key(),
            "format.rule.insertDefaultParagraphAfterChapterIntro"
        );
    }

    #[test]
    fn format_profiles_differ_on_structural_linebreaks_only() {
        let editor = FormatOptions::for_profile(FormatProfile::CodeEditor);
        let reading = FormatOptions::for_profile(FormatProfile::Reading);

        // CodeEditor inserts structural linebreaks; Reading strips
        // unwanted ones to keep paragraphs flowing.
        assert!(editor.insert_structural_linebreaks);
        assert!(!reading.insert_structural_linebreaks);
        assert!(reading.remove_unwanted_linebreaks);
        assert!(!editor.remove_unwanted_linebreaks);

        // Both profiles share the structural-WS cleanup defaults.
        for opts in [editor, reading] {
            assert!(opts.collapse_whitespace_in_text);
            assert!(opts.collapse_consecutive_linebreaks);
            assert!(opts.normalize_marker_whitespace_at_line_start);
            assert!(opts.ensure_inline_separators);
        }
    }

    /// A minter that counts its own calls — a pure function of call count,
    /// e.g. `{book}-p{patch}-{n}`, the shape a deterministic minter needs.
    fn counting_minter() -> impl FnMut() -> String {
        let mut count: u32 = 0;
        move || {
            count += 1;
            format!("synthetic-{count}")
        }
    }

    /// With no minter — every call site before this seam existed, and every
    /// call site today unless a caller opts in — synthesized tokens keep
    /// their historical no-id shape. This is the byte-identity half of the
    /// seam: nothing changes for `format_tokens`'s existing callers.
    #[test]
    fn format_tokens_without_a_minter_leaves_synthesized_tokens_with_no_id() {
        // Triggers both structural-linebreak insertion (`\c 1\p\v 1 …` has no
        // newlines between markers) and default-paragraph insertion (a verse
        // directly after `\c 1` with no `\p`).
        let source = "\\id GEN\n\\c 1\\v 1 In the beginning.";
        let parsed = parse(source);
        let mut tokens = into_format_tokens(&parsed.tokens);
        format_tokens(&mut tokens, FormatOptions::all_enabled());
        assert!(
            tokens.iter().all(|token| token.id.is_none()),
            "no synthesized token should carry an id without a minter"
        );
    }

    /// With a minter, every token the formatter synthesizes — an inserted
    /// structural linebreak and a default `\p` after a bare chapter intro —
    /// gets a fresh id. Pre-existing (non-synthetic) tokens are untouched.
    #[test]
    fn format_tokens_with_minter_mints_every_synthesized_token() {
        let source = "\\id GEN\n\\c 1\\v 1 In the beginning.";
        let parsed = parse(source);
        let tokens = into_format_tokens(&parsed.tokens);
        let original_count = tokens.len();

        let mut minter = counting_minter();
        let mut minted = tokens.clone();
        format_tokens_with_minter(&mut minted, FormatOptions::all_enabled(), &mut minter);

        // Original tokens carried no id; the only tokens with one afterward
        // are the ones the formatter created.
        let with_id: Vec<_> = minted.iter().filter(|token| token.id.is_some()).collect();
        assert!(
            !with_id.is_empty(),
            "this fixture must actually exercise synthesis, or the test proves nothing"
        );
        assert!(
            minted.len() > original_count,
            "synthesis must have added tokens"
        );
        for token in &with_id {
            assert!(
                token
                    .id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("synthetic-")),
                "every id must come from the supplied minter, not be invented"
            );
        }
        // Minted ids are unique — one per synthesized token, in call order.
        let mut ids: Vec<&str> = with_id.iter().filter_map(|t| t.id.as_deref()).collect();
        let unique_count = {
            ids.sort_unstable();
            ids.dedup();
            ids.len()
        };
        assert_eq!(unique_count, with_id.len());

        // Formatted USFM text is identical whether or not a minter was
        // supplied: minting only touches the id field, never source/kind/
        // marker/text, so this is oracle-neutral by construction.
        let mut unminted = tokens;
        format_tokens(&mut unminted, FormatOptions::all_enabled());
        assert_eq!(
            format_tokens_to_usfm(&unminted),
            format_tokens_to_usfm(&minted)
        );
    }

    /// Two independent runs over the same input with freshly-constructed,
    /// identically-behaved minters produce identical ids — the seam adds no
    /// hidden nondeterminism (no clock, no randomness, just call count).
    #[test]
    fn format_tokens_with_minter_is_deterministic_across_runs() {
        let source = "\\id GEN\n\\c 1\\v 1 In the beginning.";
        let parsed = parse(source);
        let tokens = into_format_tokens(&parsed.tokens);

        let mut first = tokens.clone();
        format_tokens_with_minter(
            &mut first,
            FormatOptions::all_enabled(),
            &mut counting_minter(),
        );
        let mut second = tokens;
        format_tokens_with_minter(
            &mut second,
            FormatOptions::all_enabled(),
            &mut counting_minter(),
        );

        let ids = |v: &[FormatToken]| v.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&first), ids(&second));
    }

    /// `FormattableToken::set_id` has no default body specifically so a
    /// minted id can't be silently dropped by an implementor that never
    /// wrote a real one. This exercises the trait-level guarantee directly,
    /// through `id()`, for both in-tree implementors — not just by reading
    /// `FormatToken`'s own `id` field, which would only prove the field
    /// exists, not that the trait contract holds.
    #[test]
    fn a_minted_id_is_observable_through_the_trait_after_set_id() {
        fn assert_round_trips<T: FormattableToken>(mut token: T) {
            token.set_id("synthetic-1".to_string());
            assert_eq!(token.id(), Some("synthetic-1"));
        }

        assert_round_trips(FormatToken {
            kind: TokenKind::Text,
            text: String::new(),
            marker: None,
            sid: None,
            id: None,
            span: None,
            structural: None,
            number_info: None,
            marker_profile: None,
            attribute_source: None,
        });
        assert_round_trips(EditorToken {
            kind: TokenKind::Text,
            text: String::new(),
            marker: None,
            sid: None,
            id: String::new(),
            lane: 0,
        });
    }

    /// Direct unit test of the malformed-marker recovery path: a `Text`
    /// token whose content embeds an unescaped marker the parser didn't
    /// split out gets recovered into its own token, which is exactly as
    /// synthetic as an inserted linebreak and must be minted the same way.
    #[test]
    fn recover_malformed_markers_mints_the_recovered_marker_token() {
        let glued = FormatToken {
            kind: TokenKind::Text,
            text: "before\\q1 after".to_string(),
            marker: None,
            sid: None,
            id: None,
            span: None,
            structural: None,
            number_info: None,
            marker_profile: None,
            attribute_source: None,
        };
        let mut mint_fn = counting_minter();
        let mut minter: Option<&mut dyn FnMut() -> String> = Some(&mut mint_fn);
        let recovered =
            recover_malformed_markers(&glued, &mut minter).expect("embedded \\q1 must recover");
        assert_eq!(recovered.len(), 3, "prefix text, marker, suffix text");
        assert_eq!(recovered[0].text, "before");
        assert!(
            recovered[0].id.is_none(),
            "the prefix clone is not synthetic"
        );
        assert_eq!(recovered[1].kind, TokenKind::Marker);
        assert_eq!(recovered[1].marker.as_deref(), Some("q1"));
        assert_eq!(
            recovered[1].id.as_deref(),
            Some("synthetic-1"),
            "the recovered marker token is synthetic and must be minted"
        );
        assert_eq!(recovered[2].text, "after");
        assert_eq!(
            recovered[2].id.as_deref(),
            Some("synthetic-2"),
            "the suffix is synthetic too and must be minted, not left with the (absent) original id"
        );
    }

    /// An *identified* token split three ways must not hand the same stable
    /// id to more than one fragment — duplicate ids are an error condition
    /// downstream, so this has to be fixed at the source. The first
    /// fragment (the prefix, here) keeps the original identity, and every
    /// fragment after it is synthetic and gets its own minted id.
    #[test]
    fn recover_malformed_markers_does_not_duplicate_the_original_id() {
        let identified = FormatToken {
            kind: TokenKind::Text,
            text: "before\\q1 after".to_string(),
            marker: None,
            sid: None,
            id: Some("GEN-7".to_string()),
            span: None,
            structural: None,
            number_info: None,
            marker_profile: None,
            attribute_source: None,
        };
        let mut mint_fn = counting_minter();
        let mut minter: Option<&mut dyn FnMut() -> String> = Some(&mut mint_fn);
        let recovered = recover_malformed_markers(&identified, &mut minter)
            .expect("embedded \\q1 must recover");

        assert_eq!(recovered.len(), 3);
        assert_eq!(
            recovered[0].id.as_deref(),
            Some("GEN-7"),
            "the first fragment (the prefix) keeps the original identity"
        );
        assert_ne!(recovered[1].id.as_deref(), Some("GEN-7"));
        assert_ne!(recovered[2].id.as_deref(), Some("GEN-7"));
        assert_ne!(
            recovered[1].id, recovered[2].id,
            "no two fragments may share an id, minted or otherwise"
        );
        let ids: std::collections::HashSet<_> =
            recovered.iter().map(|token| token.id.clone()).collect();
        assert_eq!(ids.len(), 3, "all three fragment ids must be distinct");
    }

    /// Same recovery, but with no prefix (the malformed marker starts the
    /// text): the recovered marker token itself is the first fragment and
    /// must inherit the identity instead of the (nonexistent) prefix.
    #[test]
    fn recover_malformed_markers_gives_the_original_id_to_the_marker_when_there_is_no_prefix() {
        let identified = FormatToken {
            kind: TokenKind::Text,
            text: "\\q1 after".to_string(),
            marker: None,
            sid: None,
            id: Some("GEN-9".to_string()),
            span: None,
            structural: None,
            number_info: None,
            marker_profile: None,
            attribute_source: None,
        };
        let mut mint_fn = counting_minter();
        let mut minter: Option<&mut dyn FnMut() -> String> = Some(&mut mint_fn);
        let recovered = recover_malformed_markers(&identified, &mut minter)
            .expect("embedded \\q1 must recover");

        assert_eq!(recovered.len(), 2, "marker, suffix text — no prefix");
        assert_eq!(recovered[0].kind, TokenKind::Marker);
        assert_eq!(
            recovered[0].id.as_deref(),
            Some("GEN-9"),
            "with no prefix, the marker itself is the first fragment"
        );
        assert_ne!(recovered[1].id.as_deref(), Some("GEN-9"));
    }

    /// `apply_token_fix_with_minter`: an `InsertAfter` fix's new token is
    /// synthetic and gets minted; a `ReplaceToken` fix's first replacement
    /// reuses the target's own clone (its existing id, never re-minted),
    /// while any additional replacement is synthetic and gets minted.
    #[test]
    fn apply_token_fix_with_minter_mints_only_the_synthesized_tokens() {
        use crate::format::TokenTemplate;

        let target = FormatToken {
            kind: TokenKind::Marker,
            text: "\\p".to_string(),
            marker: Some("p".to_string()),
            sid: None,
            id: Some("GEN-3".to_string()),
            span: None,
            structural: None,
            number_info: None,
            marker_profile: None,
            attribute_source: None,
        };
        let tokens = vec![target.clone()];

        let fix = TokenFixForTest::insert_after(
            "GEN-3",
            vec![TokenTemplate {
                kind: TokenKind::Newline,
                text: "\n".to_string(),
                marker: None,
                sid: None,
            }],
        );
        let mut minter = counting_minter();
        let mut minter_opt: Option<&mut dyn FnMut() -> String> = Some(&mut minter);
        let result = crate::lint::apply_token_fix_with_minter(&tokens, &fix, &mut minter_opt);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].id.as_deref(),
            Some("GEN-3"),
            "target is untouched"
        );
        assert_eq!(
            result[1].id.as_deref(),
            Some("synthetic-1"),
            "the inserted token is synthetic and must be minted"
        );
    }

    /// Builds a `TokenFix::InsertAfter` for the test above without depending
    /// on `lint_impl`'s private constructors.
    struct TokenFixForTest;
    impl TokenFixForTest {
        fn insert_after(
            target_token_id: &str,
            insert: Vec<crate::format::TokenTemplate>,
        ) -> crate::lint::TokenFix {
            crate::lint::TokenFix::InsertAfter {
                code: "test".to_string(),
                label: "test".to_string(),
                label_params: Default::default(),
                target_token_id: target_token_id.to_string(),
                insert,
            }
        }
    }
}
