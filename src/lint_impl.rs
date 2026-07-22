use std::collections::BTreeMap;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::format::FormatToken;
use crate::format::FormattableToken;
use crate::marker_defs::{
    InlineContext, SpecContext, StructuralMarkerInfo, StructuralScopeKind, lookup_marker_metadata,
    marker_allows_effective_context, marker_inline_context, marker_note_context,
    marker_note_subkind, structural_marker_info,
};
use crate::markers::{MarkerKind, lookup_marker};
use crate::parse::parse;
use crate::token::{NumberRangeKind, Sid, Span, Token, TokenData, TokenId, TokenKind};
use crate::walker::WalkableToken;

/// Public token-shape contract for `lint_tokens` and friends.
///
/// `LintableToken` is a supertrait of [`WalkableToken`] — every
/// `LintableToken` impl can be fed directly to the walker, which means
/// downstream consumers (`Token<'_>`, `FormatToken`, the editor's own
/// `EditorToken`) all flow through the same single scope state machine
/// for the structural rules. The methods on `WalkableToken` (`kind`,
/// `marker`, `text`, `structural`) are inherited; this trait adds the
/// lint-specific fields the walker doesn't need (`span`, `sid`, `id`,
/// `number_info`).
pub trait LintableToken: WalkableToken {
    fn span(&self) -> Option<Span> {
        None
    }
    fn sid(&self) -> Option<String> {
        None
    }
    fn id(&self) -> Option<String> {
        None
    }
    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        None
    }

    /// Whether this token's marker is valid in `context`, applying the same
    /// "effective context" promotions as [`marker_allows_effective_context`].
    /// The default resolves the marker by name (used by editor tokens that
    /// carry no catalog handle); [`Token`] overrides it to bit-test the marker
    /// index the lexer already stamped, avoiding a per-token re-hash on lint's
    /// hot path. Both forms are byte-identical — the stamped index resolves to
    /// the same canonical row the name would.
    fn allows_effective_context(&self, context: SpecContext) -> bool {
        marker_allows_effective_context(self.marker().unwrap_or_default(), context)
    }
}

impl<'a> LintableToken for Token<'a> {
    fn allows_effective_context(&self, context: SpecContext) -> bool {
        crate::marker_defs::marker_allows_effective_context_for_index(self.marker_index(), context)
    }

    fn span(&self) -> Option<Span> {
        Some(self.span)
    }

    fn sid(&self) -> Option<String> {
        self.sid.map(format_sid)
    }

    fn id(&self) -> Option<String> {
        Some(format_token_id(self.id))
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        match self.data {
            TokenData::Number { start, end, kind } => Some((start, end, kind)),
            _ => None,
        }
    }
}

impl WalkableToken for FormatToken {
    fn kind(&self) -> TokenKind {
        self.kind
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural
    }
}

impl LintableToken for FormatToken {
    fn span(&self) -> Option<Span> {
        self.span
    }

    fn sid(&self) -> Option<String> {
        self.sid.clone()
    }

    fn id(&self) -> Option<String> {
        self.id.clone()
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        self.number_info
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintCategory {
    Document,
    Structure,
    Context,
    Numbering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintIssueType {
    Usfm,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintCode {
    MissingIdMarker,
    DuplicateIdMarker,
    IdMarkerNotAtFileStart,
    EmptyParagraph,
    MissingChapterNumber,
    MissingVerseNumber,
    /// Replaces the prior `VerseContentNotEmpty` and `VerseTextFollowsVerseRange`
    /// codes: both fired when a verse had no body content (whitespace-only
    /// text or no significant text at all). Merged because the user-facing
    /// problem is identical.
    VerseIsEmpty,
    UnknownToken,
    UnknownMarker,
    UnknownCloseMarker,
    /// Replaces the prior `ParagraphBeforeFirstChapter` and
    /// `VerseBeforeFirstChapter` codes. The `kind` param (`paragraph` /
    /// `verse`) distinguishes which content arrived early.
    ContentBeforeFirstChapter,
    VerseOutsideExplicitParagraph,
    NoteSubmarkerOutsideNote,
    /// Replaces the prior `ChapterMetadataOutsideChapter` and
    /// `VerseMetadataOutsideVerse` codes. The `target` param
    /// (`chapter` / `verse`) distinguishes the missing target.
    MetadataOutsideTarget,
    MarkerNotValidInContext,
    MissingMilestoneSelfClose,
    StrayCloseMarker,
    MisnestedCloseMarker,
    ImplicitlyClosedMarker,
    /// Replaces the prior `UnclosedNote`, `CharNotClosed`, and
    /// `UnclosedMarker` codes. The `kind` param (`note` / `character` /
    /// `other`) and `location` param (`at-eof` / `at-boundary`) carry
    /// the previously-distinct semantics.
    UnclosedMarker,
    DuplicateChapterNumber,
    DuplicateVerseNumber,
    InvalidNumberRange,
    NumberRangeNotPrecededByMarkerExpectingNumber,
    /// Spec-driven: the `MARKER_WHITESPACE` row for a marker says
    /// whitespace or a newline is required *before* the marker, but
    /// none is present.
    MissingWhitespaceBeforeMarker,
    /// Spec-driven: the marker's row demands horizontal whitespace
    /// (or any whitespace) immediately after the marker name, but the
    /// following token does not start with the required whitespace.
    /// Covers cases like `\c1` (chapter number jammed against marker)
    /// or `\v1Text` (verse marker against number, also against text).
    MissingHorizontalWhitespaceAfterMarkerName,
    /// Spec-driven: the marker's row demands a tag-end delimiter
    /// (whitespace, end-of-input, or `|` attribute opener) immediately
    /// after the marker name, but the following token is none of those.
    /// Covers paragraph / header / sidebar / note markers jammed
    /// against text — e.g. `\p word` is fine, `\pword` is not.
    MissingTagEndDelimiterAfterMarker,
    /// Content-style flag-only rule: a closing character marker
    /// (`\nd*`, `\bk*`, …) is directly followed by alphabetic text
    /// with no space between. No autofix — the shape can be
    /// language-correct (`\nd Lord\nd*'s`) and the linter cannot
    /// distinguish.
    MissingContentSpaceAfterCloseMarker,
    /// USFM 3.2: `\v` is not allowed inside paragraphs of category
    /// `Section` or `Other`. Driven by the walker's
    /// `current_paragraph_category` — fires on `on_enter_scope(Verse)`
    /// when the nearest open paragraph carries one of those categories.
    VerseInSectionOrOtherParagraph,
    /// A no-content block marker (`\b`) carries content on its own line.
    /// `\b` is a blank line and takes no content; the spec table marks it
    /// `SingleNewline` after the name. Trailing horizontal whitespace
    /// before the newline is fine (the formatter normalises it), so this
    /// fires only on genuine content riding the same line — `\b here`,
    /// not `\b ` followed by a newline. Anchored at the content.
    ContentAfterBlankMarker,
}

impl LintCode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingIdMarker => "missing-id-marker",
            Self::DuplicateIdMarker => "duplicate-id-marker",
            Self::IdMarkerNotAtFileStart => "id-marker-not-at-file-start",
            Self::EmptyParagraph => "empty-paragraph",
            Self::MissingChapterNumber => "missing-chapter-number",
            Self::MissingVerseNumber => "missing-verse-number",
            Self::VerseIsEmpty => "verse-is-empty",
            Self::UnknownToken => "unknown-token",
            Self::UnknownMarker => "unknown-marker",
            Self::UnknownCloseMarker => "unknown-close-marker",
            Self::ContentBeforeFirstChapter => "content-before-first-chapter",
            Self::VerseOutsideExplicitParagraph => "verse-outside-explicit-paragraph",
            Self::NoteSubmarkerOutsideNote => "note-submarker-outside-note",
            Self::MetadataOutsideTarget => "metadata-outside-target",
            Self::MarkerNotValidInContext => "marker-not-valid-in-context",
            Self::MissingMilestoneSelfClose => "missing-milestone-self-close",
            Self::StrayCloseMarker => "stray-close-marker",
            Self::MisnestedCloseMarker => "misnested-close-marker",
            Self::ImplicitlyClosedMarker => "implicitly-closed-marker",
            Self::UnclosedMarker => "unclosed-marker",
            Self::DuplicateChapterNumber => "duplicate-chapter-number",
            Self::DuplicateVerseNumber => "duplicate-verse-number",
            Self::InvalidNumberRange => "invalid-number-range",
            Self::NumberRangeNotPrecededByMarkerExpectingNumber => {
                "number-range-not-preceded-by-marker-expecting-number"
            }
            Self::MissingWhitespaceBeforeMarker => "missing-whitespace-before-marker",
            Self::MissingHorizontalWhitespaceAfterMarkerName => {
                "missing-horizontal-whitespace-after-marker-name"
            }
            Self::MissingTagEndDelimiterAfterMarker => "missing-tag-end-delimiter-after-marker",
            Self::MissingContentSpaceAfterCloseMarker => "missing-content-space-after-close-marker",
            Self::VerseInSectionOrOtherParagraph => "verse-in-section-or-other-paragraph",
            Self::ContentAfterBlankMarker => "content-after-blank-marker",
        }
    }

    /// Stable ICU MessageFormat template. Placeholders use the
    /// `{name}` / `{name, number}` / `{name, select, …}` syntax;
    /// resolve at runtime against the issue's `message_params`.
    pub const fn template(self) -> &'static str {
        match self {
            Self::MissingIdMarker => "File is missing its \\id (book identifier).",
            Self::DuplicateIdMarker => "This file has more than one \\id; only one is allowed.",
            Self::IdMarkerNotAtFileStart => "\\id must come before any other content.",
            Self::EmptyParagraph => {
                "\\{marker} starts an empty paragraph — the next paragraph begins right after, with no content in between."
            }
            Self::MissingChapterNumber => "\\c needs a chapter number after it.",
            Self::MissingVerseNumber => "\\v needs a verse number after it.",
            Self::VerseIsEmpty => "This verse has no content.",
            Self::UnknownToken => "Couldn't recognize \"{text}\".",
            Self::UnknownMarker => "\\{marker} is not a known USFM marker.",
            Self::UnknownCloseMarker => "\\{marker}* is not a known closing marker.",
            Self::ContentBeforeFirstChapter => {
                "{kind, select, paragraph {Paragraph marker \\{marker}} verse {Verse marker \\v} other {\\{marker}}} appears before the first \\c."
            }
            Self::VerseOutsideExplicitParagraph => {
                "Verses must appear inside a paragraph, list, or table."
            }
            Self::NoteSubmarkerOutsideNote => {
                "\\{marker} is part of a footnote or cross-reference and must appear inside one."
            }
            Self::MetadataOutsideTarget => {
                "\\{marker} must follow a {target, select, chapter {\\c chapter marker} verse {\\v verse marker} other {target marker}}."
            }
            Self::MarkerNotValidInContext => {
                "\\{marker} is not allowed inside a {context, select, footnote {footnote} cross-reference {cross-reference} table {table cell} chapter-content {chapter} para {paragraph} list {list} section {section heading} sidebar {sidebar} peripheral {peripheral section} peripheral-content {peripheral section} book-headers {book header} book-titles {book title} book-introduction {book introduction} book-introduction-end-titles {book introduction} scripture {scripture text} other {{context}}}."
            }
            Self::MissingMilestoneSelfClose => {
                "\\{marker} is a milestone and needs to end with \\*."
            }
            Self::StrayCloseMarker => {
                "{form, select, milestone-end {Found \\* with no open milestone to close.} other {\\{marker}* has no matching opening \\{marker}.}}"
            }
            Self::MisnestedCloseMarker => {
                "{has_expected, select, true {Expected \\{expected}* here, but found \\{marker}*.} other {\\{marker}* does not match the marker that is currently open.}}"
            }
            Self::ImplicitlyClosedMarker => {
                "\\{marker} was never closed; \\{closer}* closed it indirectly. Add an explicit \\{marker}* before \\{closer}*."
            }
            Self::UnclosedMarker => {
                "{kind, select, note {Note} character {Character marker} other {Marker}} \\{marker} was opened but never closed{location, select, at-eof { before the file ended.} at-boundary { before a new block began.} other {.}}"
            }
            Self::DuplicateChapterNumber => "Chapter {chapter, number} appears more than once.",
            Self::DuplicateVerseNumber => {
                "Verse {verse} appears more than once in chapter {chapter, number}."
            }
            Self::InvalidNumberRange => "'{verse}' is not a valid verse range.",
            Self::NumberRangeNotPrecededByMarkerExpectingNumber => {
                "This number range is not preceded by a marker that expects a number (like \\c or \\v)."
            }
            Self::MissingWhitespaceBeforeMarker => "\\{marker} needs a space or newline before it.",
            Self::MissingHorizontalWhitespaceAfterMarkerName => {
                "\\{marker} needs a space after the marker name."
            }
            Self::MissingTagEndDelimiterAfterMarker => {
                "\\{marker} needs a space before the text that follows."
            }
            Self::MissingContentSpaceAfterCloseMarker => {
                "\\{marker}* is directly followed by text with no space. If this is an intentional contraction (e.g. \\nd Lord\\nd*'s) you can ignore this."
            }
            Self::VerseInSectionOrOtherParagraph => {
                "\\v is not allowed inside a {category, select, section {section heading} other {non-content paragraph}}; verses must appear inside body paragraphs, lists, or tables."
            }
            Self::ContentAfterBlankMarker => {
                "This content shares a line with \\{marker}, but \\{marker} is a blank line that takes no content. Put it in its own paragraph (\\p, \\q, …) on the next line."
            }
        }
    }

    pub fn category(self) -> LintCategory {
        match self {
            Self::MissingIdMarker
            | Self::ContentBeforeFirstChapter
            | Self::DuplicateIdMarker
            | Self::IdMarkerNotAtFileStart => LintCategory::Document,
            Self::EmptyParagraph
            | Self::VerseIsEmpty
            | Self::UnknownToken
            | Self::MissingChapterNumber
            | Self::MissingVerseNumber
            | Self::MissingMilestoneSelfClose
            | Self::ImplicitlyClosedMarker
            | Self::StrayCloseMarker
            | Self::MisnestedCloseMarker
            | Self::UnclosedMarker
            | Self::UnknownMarker
            | Self::UnknownCloseMarker
            | Self::MissingWhitespaceBeforeMarker
            | Self::MissingHorizontalWhitespaceAfterMarkerName
            | Self::MissingTagEndDelimiterAfterMarker
            | Self::MissingContentSpaceAfterCloseMarker
            | Self::ContentAfterBlankMarker => LintCategory::Structure,
            Self::NoteSubmarkerOutsideNote
            | Self::MetadataOutsideTarget
            | Self::MarkerNotValidInContext
            | Self::VerseOutsideExplicitParagraph
            | Self::VerseInSectionOrOtherParagraph => LintCategory::Context,
            Self::DuplicateChapterNumber
            | Self::DuplicateVerseNumber
            | Self::InvalidNumberRange
            | Self::NumberRangeNotPrecededByMarkerExpectingNumber => LintCategory::Numbering,
        }
    }

    pub fn severity(self) -> LintSeverity {
        match self {
            Self::EmptyParagraph | Self::MissingContentSpaceAfterCloseMarker => {
                LintSeverity::Warning
            }
            _ => LintSeverity::Error,
        }
    }

    pub fn issue_type(self) -> LintIssueType {
        match self {
            Self::VerseIsEmpty
            | Self::MissingChapterNumber
            | Self::MissingVerseNumber
            | Self::DuplicateChapterNumber
            | Self::DuplicateVerseNumber
            | Self::InvalidNumberRange
            | Self::NumberRangeNotPrecededByMarkerExpectingNumber => LintIssueType::Content,
            Self::MissingIdMarker
            | Self::EmptyParagraph
            | Self::UnknownToken
            | Self::ContentBeforeFirstChapter
            | Self::NoteSubmarkerOutsideNote
            | Self::DuplicateIdMarker
            | Self::IdMarkerNotAtFileStart
            | Self::MetadataOutsideTarget
            | Self::MissingMilestoneSelfClose
            | Self::ImplicitlyClosedMarker
            | Self::StrayCloseMarker
            | Self::MisnestedCloseMarker
            | Self::UnclosedMarker
            | Self::UnknownMarker
            | Self::UnknownCloseMarker
            | Self::MarkerNotValidInContext
            | Self::VerseOutsideExplicitParagraph
            | Self::MissingWhitespaceBeforeMarker
            | Self::MissingHorizontalWhitespaceAfterMarkerName
            | Self::MissingTagEndDelimiterAfterMarker
            | Self::MissingContentSpaceAfterCloseMarker
            | Self::VerseInSectionOrOtherParagraph
            | Self::ContentAfterBlankMarker => LintIssueType::Usfm,
        }
    }

    /// Single-bit mask for this code, keyed by its enum discriminant. Used by
    /// [`EnabledCodes`] to store rule allow/deny sets as a `u64` bitmask instead
    /// of a `BTreeSet`, turning [`EnabledCodes::has`] into a branchless bit test.
    /// The fieldless enum has `< 64` variants, so every discriminant fits a
    /// `u64`; the drift-guard test `enabled_codes_bitmask_matches_btreeset`
    /// asserts that invariant and that this mask agrees with the old set logic.
    const fn bit(self) -> u64 {
        1u64 << (self as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintIssue {
    pub code: LintCode,
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub issue_type: LintIssueType,
    /// Stable ICU MessageFormat template. Identical to
    /// `LintCode::template(code)`; carried on the issue so consumers
    /// localising messages don't have to look it up from the code
    /// separately.
    pub template: &'static str,
    /// Rendered English message — `template` populated with
    /// `message_params`. Always the same as
    /// `render_template(template, &message_params)`; downstream
    /// localisers can ignore this and re-render from `template` +
    /// `message_params`.
    pub message: String,
    pub message_params: MessageParams,
    pub span: Option<Span>,
    pub related_span: Option<Span>,
    pub token_id: Option<String>,
    pub related_token_id: Option<String>,
    pub sid: Option<String>,
    pub marker: Option<String>,
    pub fix: Option<TokenFix>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LintSummary {
    pub by_category: BTreeMap<LintCategory, usize>,
    pub by_severity: BTreeMap<LintSeverity, usize>,
    pub by_issue_type: BTreeMap<LintIssueType, usize>,
    pub total_count: usize,
    pub suppressed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
    pub summary: LintSummary,
}

mod message;
pub use message::MessageParams;
use message::{message_params, render_template};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenFix {
    ReplaceToken {
        code: String,
        label: String,
        label_params: MessageParams,
        target_token_id: String,
        replacements: Vec<crate::format::TokenTemplate>,
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
        insert: Vec<crate::format::TokenTemplate>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintSuppression {
    pub code: LintCode,
    pub sid: String,
}

/// What the caller is handing the linter. USFM is per-book; the only
/// non-chapter unit in a book is the pre-`\c` front matter. Scope gates the
/// document-level rules (`LintCategory::Document`): they run only when the
/// slice can contain the book head — `Front` or `Book` — never on a bare
/// `Chapter`, so a mid-book chapter slice can't produce a spurious
/// "missing-id". The chapter number on `Chapter` is for the caller's own
/// result keying; the linter itself doesn't branch on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LintScope {
    /// The book's front matter: `\id`, `\h`, `\toc`, `\mt`, intros, periphs —
    /// everything before the first `\c`.
    Front,
    /// A single chapter: `\c n` through the token before the next `\c`.
    Chapter(u32),
    /// A whole book (front matter + all chapters).
    Book,
}

impl LintScope {
    /// Document-level rules run only when the slice can hold the book head.
    fn runs_document_rules(self) -> bool {
        matches!(self, LintScope::Front | LintScope::Book)
    }
}

// No `Default`: `scope` has no sane default — a caller must declare what it
// is sending (see `scoped`). A defaulted scope would let a chapter-grain
// caller silently get whole-book id-behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintOptions {
    pub scope: LintScope,
    pub enabled_codes: Option<Vec<LintCode>>,
    pub disabled_codes: Vec<LintCode>,
    pub suppressed: Vec<LintSuppression>,
    pub allow_implicit_chapter_content_verse: bool,
}

impl LintOptions {
    /// A scope with all rules enabled and no suppressions — the common case.
    /// NOT a default: the caller still names the scope. `scoped(LintScope::Book)`
    /// reproduces whole-book linting (today's behavior).
    pub fn scoped(scope: LintScope) -> Self {
        Self {
            scope,
            enabled_codes: None,
            disabled_codes: Vec::new(),
            suppressed: Vec::new(),
            allow_implicit_chapter_content_verse: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentKind {
    Scripture,
    PeripheralStandalone,
    PeripheralDivided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopLevelSlot {
    AwaitDivision,
    Headers,
    Titles,
    Introduction,
    IntroductionEndTitles,
    Content,
}

#[derive(Debug, Clone)]
struct DocumentLintState {
    kind: DocumentKind,
    slot: TopLevelSlot,
    saw_chapter: bool,
    block_context: Option<SpecContext>,
    note_stack: Vec<SpecContext>,
}

#[derive(Debug, Clone)]
struct EnabledCodes {
    /// `None` = all codes allowed. `Some(mask)` = only the set bits are allowed.
    /// A `u64` bitmask keyed by `LintCode::bit` replaces the old `BTreeSet`, so
    /// `has` is a bit test rather than a per-token B-tree probe.
    allowed: Option<u64>,
    /// Disabled codes as a `u64` bitmask keyed by `LintCode::bit`.
    disabled: u64,
    /// When false (scope is `Chapter`), `LintCategory::Document` codes are
    /// suppressed at the `has` chokepoint — they need the book head.
    run_document_rules: bool,
}

#[derive(Default)]
struct VerseState {
    seen: FxHashSet<u32>,
}

impl Default for DocumentLintState {
    fn default() -> Self {
        Self {
            kind: DocumentKind::Scripture,
            slot: TopLevelSlot::Headers,
            saw_chapter: false,
            block_context: None,
            note_stack: Vec::new(),
        }
    }
}

impl DocumentLintState {
    fn current_root_context(&self) -> SpecContext {
        match (self.kind, self.slot) {
            (DocumentKind::PeripheralDivided, TopLevelSlot::AwaitDivision) => {
                SpecContext::Peripheral
            }
            (_, TopLevelSlot::Headers) => SpecContext::BookHeaders,
            (_, TopLevelSlot::Titles) => SpecContext::BookTitles,
            (_, TopLevelSlot::Introduction) => SpecContext::BookIntroduction,
            (_, TopLevelSlot::IntroductionEndTitles) => SpecContext::BookIntroductionEndTitles,
            (DocumentKind::Scripture, TopLevelSlot::Content) => SpecContext::ChapterContent,
            (DocumentKind::PeripheralStandalone, TopLevelSlot::Content)
            | (DocumentKind::PeripheralDivided, TopLevelSlot::Content) => {
                SpecContext::PeripheralContent
            }
            (_, TopLevelSlot::AwaitDivision) => SpecContext::Peripheral,
        }
    }

    fn current_note_context(&self) -> Option<SpecContext> {
        self.note_stack.last().copied()
    }

    /// Pop the note context a matching note-close (`\f*`, `\x*`, …) ends. Notes
    /// don't nest in USFM, so one pop per close mirrors the one push per note
    /// container in `apply_marker`. Without this, `current_note_context` stayed
    /// stale after a note closed and validated following markers as if still
    /// inside the note.
    fn close_note(&mut self) {
        self.note_stack.pop();
    }

    fn current_validation_context_for_kind(&self, kind: MarkerKind) -> SpecContext {
        let root_context = self.current_root_context();
        let effective = self
            .current_note_context()
            .or(self.block_context)
            .unwrap_or(root_context);

        match kind {
            MarkerKind::Character | MarkerKind::TableCell => effective,
            MarkerKind::Verse => root_context,
            MarkerKind::Meta => effective,
            MarkerKind::Note
            | MarkerKind::Figure
            | MarkerKind::Chapter
            | MarkerKind::Paragraph
            | MarkerKind::Header
            | MarkerKind::SidebarStart
            | MarkerKind::SidebarEnd
            | MarkerKind::Periph
            | MarkerKind::TableRow
            | MarkerKind::MilestoneStart
            | MarkerKind::MilestoneEnd
            | MarkerKind::Unknown => root_context,
        }
    }

    fn select_top_level_slot<T: LintableToken>(&self, token: &T) -> TopLevelSlot {
        if token.marker() == Some("periph") {
            return TopLevelSlot::AwaitDivision;
        }

        let contexts = top_level_contexts_for(self.kind);
        let start = top_level_slot_index(self.slot);

        contexts
            .iter()
            .copied()
            .skip(start)
            .find(|(_, context)| token.allows_effective_context(*context))
            .map(|(slot, _)| slot)
            .unwrap_or(self.slot)
    }

    fn apply_marker<T: LintableToken>(&mut self, tokens: &[T], index: usize, token: &T) {
        let Some(name) = token.marker() else {
            return;
        };

        match token_marker_kind(token) {
            MarkerKind::Header => {
                if name == "id"
                    && let Some(book_code) = next_book_code_after_marker(tokens, index)
                {
                    self.kind = infer_document_kind(book_code);
                    if self.kind == DocumentKind::PeripheralDivided {
                        self.slot = TopLevelSlot::AwaitDivision;
                    } else {
                        self.slot = TopLevelSlot::Headers;
                    }
                    self.block_context = None;
                    self.note_stack.clear();
                } else {
                    self.slot = self.select_top_level_slot(token);
                }
            }
            MarkerKind::Chapter => {
                self.saw_chapter = true;
                self.slot = TopLevelSlot::Content;
                self.block_context = None;
            }
            MarkerKind::Paragraph => {
                if self.current_note_context().is_none() {
                    self.slot = self.select_top_level_slot(token);
                }
                self.block_context = Some(paragraph_block_context_for(token, name));
            }
            MarkerKind::Note => {
                self.note_stack.push(note_context_for_marker(name));
            }
            MarkerKind::Periph => {
                self.kind = DocumentKind::PeripheralDivided;
                self.slot = TopLevelSlot::Headers;
                self.block_context = None;
                self.note_stack.clear();
                self.saw_chapter = false;
            }
            MarkerKind::SidebarStart => {
                self.slot = TopLevelSlot::Content;
                self.block_context = Some(SpecContext::Sidebar);
            }
            MarkerKind::SidebarEnd => {
                self.block_context = None;
            }
            MarkerKind::TableRow | MarkerKind::TableCell => {
                self.slot = TopLevelSlot::Content;
                self.block_context = Some(SpecContext::Table);
            }
            MarkerKind::Verse
            | MarkerKind::Character
            | MarkerKind::Figure
            | MarkerKind::Meta
            | MarkerKind::MilestoneStart
            | MarkerKind::MilestoneEnd
            | MarkerKind::Unknown => {}
        }
    }
}

impl EnabledCodes {
    fn new(options: &LintOptions) -> Self {
        Self {
            allowed: options.enabled_codes.as_ref().map(|codes| {
                codes
                    .iter()
                    .copied()
                    .fold(0u64, |mask, code| mask | code.bit())
            }),
            disabled: options
                .disabled_codes
                .iter()
                .copied()
                .fold(0u64, |mask, code| mask | code.bit()),
            run_document_rules: options.scope.runs_document_rules(),
        }
    }

    fn has(&self, code: LintCode) -> bool {
        if self.disabled & code.bit() != 0 {
            return false;
        }
        if !self.run_document_rules && code.category() == LintCategory::Document {
            return false;
        }
        self.allowed.is_none_or(|allowed| allowed & code.bit() != 0)
    }

    fn has_any(&self, codes: &[LintCode]) -> bool {
        codes.iter().copied().any(|code| self.has(code))
    }
}

pub fn lint_usfm(source: &str, options: LintOptions) -> LintResult {
    let parsed = parse(source);
    lint_tokens(&parsed.tokens, options)
}

pub fn lint_tokens<T: LintableToken + Sync>(tokens: &[T], options: LintOptions) -> LintResult {
    let enabled = EnabledCodes::new(&options);

    // A whole book decomposes at `\c` into independent chapter segments: the
    // range-local rules (empty-paragraph, expectations, unknown/whitespace) and
    // the walker-driven marker-balance rules run per segment, while the rules
    // that thread document state or reconcile across chapters (structure,
    // duplicate-chapter, number/verse) run once over the whole stream. Combining
    // and canonically sorting the findings reproduces the serial result exactly,
    // so this is byte-identical regardless of thread count or target — only
    // wall-clock changes. Small books, non-book scopes, and wasm stay serial:
    // below the segment count/size where the fan-out pays for itself, or with no
    // thread pool to recover it, the decomposition is pure overhead.
    #[cfg(not(target_arch = "wasm32"))]
    let issues = if matches!(options.scope, LintScope::Book) && tokens.len() >= PARALLEL_MIN_TOKENS
    {
        collect_issues_partitioned(tokens, &options, &enabled)
    } else {
        collect_issues_serial(tokens, &options, &enabled)
    };
    #[cfg(target_arch = "wasm32")]
    let issues = collect_issues_serial(tokens, &options, &enabled);

    finalize_issues(issues, &options)
}

/// Shared tail: dedupe, suppress, canonically sort, and summarize. Runs once
/// after collection, so the serial and chapter-parallel collectors converge on
/// byte-identical output here.
fn finalize_issues(issues: Vec<LintIssue>, options: &LintOptions) -> LintResult {
    let unique = dedupe_issues(issues);
    let (mut issues, suppressed_count) = apply_suppressions(unique, &options.suppressed);
    canonical_sort(&mut issues);
    let summary = summarize(&issues, suppressed_count);

    LintResult { issues, summary }
}

/// Token count at or above which a `LintScope::Book` lint fans out across chapter
/// segments. Below it the fixed decomposition cost (segment scan, per-segment
/// allocation, the serial combine) outweighs the parallel gain, so linting stays
/// serial. A conservative proxy — heavy books clear it comfortably and a book
/// left below it lints in well under a millisecond either way.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const PARALLEL_MIN_TOKENS: usize = 4096;

/// The whole-book serial linter: every rule over the full stream, in the fixed
/// order that pins the oracle. Used for small books, non-book scopes, and wasm.
fn collect_issues_serial<T: LintableToken>(
    tokens: &[T],
    options: &LintOptions,
    enabled: &EnabledCodes,
) -> Vec<LintIssue> {
    let full = 0..tokens.len();
    let mut issues = Vec::new();

    if enabled.has(LintCode::EmptyParagraph) {
        lint_empty_paragraphs(tokens, full.clone(), &mut issues);
    }
    if enabled.has_any(&[
        LintCode::UnknownToken,
        LintCode::VerseIsEmpty,
        LintCode::MissingChapterNumber,
        LintCode::MissingVerseNumber,
    ]) {
        lint_expectation_and_unknown_token_rules(tokens, full.clone(), enabled, &mut issues);
    }
    if enabled.has_any(STRUCTURE_CODES) {
        lint_structure_rules(tokens, options, enabled, &mut issues);
    }
    if enabled.has(LintCode::UnknownMarker) {
        lint_unknown_markers(tokens, full.clone(), &mut issues);
    }
    if enabled.has(LintCode::UnknownCloseMarker) {
        lint_unknown_close_markers(tokens, full.clone(), &mut issues);
    }
    if enabled.has(LintCode::DuplicateChapterNumber) {
        lint_chapter_rules(tokens, enabled, &mut issues);
    }
    if enabled.has_any(NUMBER_VERSE_CODES) {
        lint_number_and_verse_rules(tokens, enabled, &mut issues);
    }
    if enabled.has_any(MARKER_BALANCE_CODES) {
        lint_marker_balance_rules(tokens, enabled, &mut issues);
    }
    if enabled.has_any(WHITESPACE_CODES) {
        lint_whitespace_rules(tokens, full.clone(), enabled, &mut issues);
    }

    issues
}

/// Chapter-parallel collection for `LintScope::Book`. Range-local and
/// walker-driven rules run per chapter segment through the order-preserving
/// [`crate::par::map_ordered`] seam; the document/cross-chapter rules run once
/// over the whole stream. The combined findings are byte-identical to
/// [`collect_issues_serial`] after the shared dedupe/sort tail (proven by
/// `partitioned_matches_serial_over_corpora` below and the lint oracle).
#[cfg(not(target_arch = "wasm32"))]
fn collect_issues_partitioned<T: LintableToken + Sync>(
    tokens: &[T],
    options: &LintOptions,
    enabled: &EnabledCodes,
) -> Vec<LintIssue> {
    let segments = crate::walker::chapter_segments(tokens);

    // One work list fed through the order-preserving fan-out: each chapter
    // segment's range-local + walker rules, plus each whole-stream rule family
    // (structure, duplicate-chapter, number/verse) as its own unit. Those three
    // families each still see the entire stream — chapter parallelism doesn't
    // touch their logic — they just run concurrently with each other and the
    // segments instead of in a serial tail, so `structure_rules` (the heaviest
    // single unit) sets the floor rather than their sum. Combined-result order is
    // irrelevant: the finalize tail canonically sorts.
    let mut work = Vec::with_capacity(segments.len() + 3);
    if enabled.has_any(STRUCTURE_CODES) {
        work.push(BookWork::Structure);
    }
    if enabled.has(LintCode::DuplicateChapterNumber) {
        work.push(BookWork::DuplicateChapter);
    }
    if enabled.has_any(NUMBER_VERSE_CODES) {
        work.push(BookWork::NumberVerse);
    }
    work.extend(segments.iter().map(BookWork::Segment));

    let parts = crate::par::map_ordered(&work, |unit| {
        let mut issues = Vec::new();
        match unit {
            BookWork::Structure => lint_structure_rules(tokens, options, enabled, &mut issues),
            BookWork::DuplicateChapter => lint_chapter_rules(tokens, enabled, &mut issues),
            BookWork::NumberVerse => lint_number_and_verse_rules(tokens, enabled, &mut issues),
            BookWork::Segment(segment) => {
                collect_range_local(tokens, segment.range.clone(), enabled, &mut issues);
                collect_marker_balance(
                    tokens,
                    segment.range.clone(),
                    segment.boundary,
                    enabled,
                    &mut issues,
                );
            }
        }
        issues
    });

    let mut issues = Vec::new();
    for part in parts {
        issues.extend(part);
    }
    issues
}

/// One unit of chapter-parallel lint work: a single whole-stream rule family, or
/// one chapter segment's range-local + walker rules.
#[cfg(not(target_arch = "wasm32"))]
enum BookWork<'a> {
    Structure,
    DuplicateChapter,
    NumberVerse,
    Segment(&'a crate::walker::ChapterSegment),
}

/// The range-local rules: each iterates the absolute `range` while reading the
/// full token slice for predecessor/lookahead, so a segment produces exactly the
/// findings the whole-book pass would for tokens in that range.
#[cfg(not(target_arch = "wasm32"))]
fn collect_range_local<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    if enabled.has(LintCode::EmptyParagraph) {
        lint_empty_paragraphs(tokens, range.clone(), issues);
    }
    if enabled.has_any(&[
        LintCode::UnknownToken,
        LintCode::VerseIsEmpty,
        LintCode::MissingChapterNumber,
        LintCode::MissingVerseNumber,
    ]) {
        lint_expectation_and_unknown_token_rules(tokens, range.clone(), enabled, issues);
    }
    if enabled.has(LintCode::UnknownMarker) {
        lint_unknown_markers(tokens, range.clone(), issues);
    }
    if enabled.has(LintCode::UnknownCloseMarker) {
        lint_unknown_close_markers(tokens, range.clone(), issues);
    }
    if enabled.has_any(WHITESPACE_CODES) {
        lint_whitespace_rules(tokens, range, enabled, issues);
    }
}

const STRUCTURE_CODES: &[LintCode] = &[
    LintCode::MissingIdMarker,
    LintCode::ContentBeforeFirstChapter,
    LintCode::NoteSubmarkerOutsideNote,
    LintCode::DuplicateIdMarker,
    LintCode::IdMarkerNotAtFileStart,
    LintCode::MetadataOutsideTarget,
    LintCode::MarkerNotValidInContext,
    LintCode::VerseOutsideExplicitParagraph,
];

const NUMBER_VERSE_CODES: &[LintCode] = &[
    LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
    LintCode::InvalidNumberRange,
    LintCode::DuplicateVerseNumber,
    LintCode::VerseIsEmpty,
];

const MARKER_BALANCE_CODES: &[LintCode] = &[
    LintCode::StrayCloseMarker,
    LintCode::MisnestedCloseMarker,
    LintCode::MissingMilestoneSelfClose,
    LintCode::UnclosedMarker,
    LintCode::ImplicitlyClosedMarker,
    LintCode::VerseInSectionOrOtherParagraph,
];

const WHITESPACE_CODES: &[LintCode] = &[
    LintCode::MissingWhitespaceBeforeMarker,
    LintCode::MissingHorizontalWhitespaceAfterMarkerName,
    LintCode::MissingTagEndDelimiterAfterMarker,
    LintCode::MissingContentSpaceAfterCloseMarker,
    LintCode::ContentAfterBlankMarker,
];

pub fn apply_token_fix<T: FormattableToken>(tokens: &[T], fix: &TokenFix) -> Vec<T> {
    let Some(index) = tokens
        .iter()
        .position(|token| token.id() == Some(fix.target_token_id()))
    else {
        return tokens.to_vec();
    };

    let mut next_tokens = tokens.to_vec();
    let anchor = next_tokens[index].clone();

    match fix {
        TokenFix::ReplaceToken { replacements, .. } => {
            if replacements.is_empty() {
                return next_tokens;
            }
            let replacement_tokens =
                build_replacement_tokens(&anchor, replacements, ReplacementMode::Replace);
            next_tokens.splice(index..=index, replacement_tokens);
        }
        TokenFix::DeleteToken { .. } => {
            next_tokens.remove(index);
        }
        TokenFix::InsertAfter { insert, .. } => {
            if insert.is_empty() {
                return next_tokens;
            }
            let insert_tokens =
                build_replacement_tokens(&anchor, insert, ReplacementMode::InsertAfter);
            next_tokens.splice(index + 1..index + 1, insert_tokens);
        }
    }

    next_tokens
}

impl TokenFix {
    pub fn target_token_id(&self) -> &str {
        match self {
            Self::ReplaceToken {
                target_token_id, ..
            }
            | Self::DeleteToken {
                target_token_id, ..
            }
            | Self::InsertAfter {
                target_token_id, ..
            } => target_token_id,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ReplacementMode {
    Replace,
    InsertAfter,
}

fn build_replacement_tokens<T: FormattableToken>(
    anchor: &T,
    templates: &[crate::format::TokenTemplate],
    mode: ReplacementMode,
) -> Vec<T> {
    let mut built = Vec::with_capacity(templates.len());
    for (index, template) in templates.iter().enumerate() {
        let mut token = if index == 0 && matches!(mode, ReplacementMode::Replace) {
            anchor.clone()
        } else {
            T::synthetic_like(
                Some(anchor),
                template.kind,
                template.text.clone(),
                template.marker.clone(),
                template.sid.clone(),
            )
        };
        token.set_kind(template.kind);
        token.set_text(template.text.clone());
        token.set_marker(template.marker.clone());
        token.set_sid(template.sid.clone());
        built.push(token);
    }
    built
}

fn lint_empty_paragraphs<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    issues: &mut Vec<LintIssue>,
) {
    for index in range {
        let token = &tokens[index];
        if token.kind() != TokenKind::Marker {
            continue;
        }
        let Some(marker) = token.marker() else {
            continue;
        };
        if !is_body_paragraph_marker(marker) || marker_is_intentionally_empty_block(marker) {
            continue;
        }
        let Some(boundary_index) = empty_paragraph_boundary_index(tokens, index) else {
            continue;
        };
        issues.push(issue(
            LintCode::EmptyParagraph,
            marker_params(marker),
            token,
            Some(&tokens[boundary_index]),
        ));
    }
}

fn lint_expectation_and_unknown_token_rules<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    for index in range {
        let token = &tokens[index];

        if enabled.has(LintCode::UnknownToken)
            && token.kind() == TokenKind::Text
            && let Some(issue) = lint_unknown_token_like(token)
        {
            issues.push(issue);
        }

        if token.kind() != TokenKind::Marker {
            continue;
        }

        let marker = token.marker().unwrap_or_default();
        match marker {
            "c" => {
                let missing = next_number_token_index(tokens, index + 1).is_none();
                if enabled.has(LintCode::MissingChapterNumber) && missing {
                    issues.push(simple_issue(
                        LintCode::MissingChapterNumber,
                        MessageParams::default(),
                        token,
                    ));
                }
            }
            "v" => {
                let missing = next_number_token_index(tokens, index + 1).is_none();
                if enabled.has(LintCode::MissingVerseNumber) && missing {
                    issues.push(simple_issue(
                        LintCode::MissingVerseNumber,
                        MessageParams::default(),
                        token,
                    ));
                }
                if enabled.has(LintCode::VerseIsEmpty)
                    && let Some(next_index) = next_significant_token_index(tokens, index + 1)
                    && tokens[next_index].kind() == TokenKind::Text
                    && tokens[next_index].text().trim().is_empty()
                {
                    issues.push(issue(
                        LintCode::VerseIsEmpty,
                        MessageParams::default(),
                        &tokens[next_index],
                        Some(token),
                    ));
                }
            }
            _ => {}
        }
    }
}

fn lint_structure_rules<T: LintableToken>(
    tokens: &[T],
    options: &LintOptions,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut saw_content = false;
    let mut id_seen = false;
    let mut note_stack: Vec<String> = Vec::new();
    let mut document_state = DocumentLintState::default();

    for (index, token) in tokens.iter().enumerate() {
        let token_kind = token.kind();
        if token_kind == TokenKind::Newline {
            continue;
        }

        if token_kind == TokenKind::Marker {
            let marker = token.marker().unwrap_or_default();
            let marker_kind = token_marker_kind(token);

            if enabled.has(LintCode::IdMarkerNotAtFileStart) && marker == "id" && saw_content {
                issues.push(simple_issue(
                    LintCode::IdMarkerNotAtFileStart,
                    MessageParams::default(),
                    token,
                ));
            }
            if enabled.has(LintCode::DuplicateIdMarker) && marker == "id" {
                if id_seen {
                    issues.push(simple_issue(
                        LintCode::DuplicateIdMarker,
                        MessageParams::default(),
                        token,
                    ));
                }
                id_seen = true;
            }

            let prospective_state = if marker_kind == MarkerKind::Note {
                document_state.current_validation_context_for_kind(marker_kind)
            } else if marker_kind == MarkerKind::Paragraph
                && document_state.current_note_context().is_none()
            {
                top_level_root_context(
                    document_state.kind,
                    document_state.select_top_level_slot(token),
                )
            } else if marker_kind == MarkerKind::Periph {
                SpecContext::Peripheral
            } else if marker_kind == MarkerKind::Chapter {
                top_level_root_context(document_state.kind, TopLevelSlot::Content)
            } else {
                document_state.current_root_context()
            };

            if enabled.has(LintCode::ContentBeforeFirstChapter)
                && !document_state.saw_chapter
                && document_state.kind == DocumentKind::Scripture
                && marker_kind == MarkerKind::Paragraph
                && is_body_paragraph_marker(marker)
                && prospective_state == SpecContext::ChapterContent
            {
                issues.push(simple_issue(
                    LintCode::ContentBeforeFirstChapter,
                    message_params([
                        ("kind", "paragraph".to_string()),
                        ("marker", marker.to_string()),
                    ]),
                    token,
                ));
            }

            if enabled.has(LintCode::ContentBeforeFirstChapter)
                && !document_state.saw_chapter
                && document_state.kind == DocumentKind::Scripture
                && marker == "v"
            {
                issues.push(simple_issue(
                    LintCode::ContentBeforeFirstChapter,
                    message_params([("kind", "verse".to_string()), ("marker", "v".to_string())]),
                    token,
                ));
            }

            if enabled.has(LintCode::VerseOutsideExplicitParagraph)
                && !options.allow_implicit_chapter_content_verse
                && marker == "v"
                && document_state.kind == DocumentKind::Scripture
                && document_state.current_root_context() == SpecContext::ChapterContent
                && !matches!(
                    document_state.block_context,
                    Some(SpecContext::Para | SpecContext::List | SpecContext::Table)
                )
            {
                issues.push(simple_issue(
                    LintCode::VerseOutsideExplicitParagraph,
                    MessageParams::default(),
                    token,
                ));
            }

            if enabled.has(LintCode::NoteSubmarkerOutsideNote)
                && marker_note_subkind(marker).is_some()
                && note_stack.is_empty()
            {
                issues.push(simple_issue(
                    LintCode::NoteSubmarkerOutsideNote,
                    marker_params(marker),
                    token,
                ));
            }

            if enabled.has(LintCode::MetadataOutsideTarget)
                && matches!(marker, "ca" | "cp")
                && !matches_previous_marker_and_number(tokens, index, "c")
            {
                issues.push(simple_issue(
                    LintCode::MetadataOutsideTarget,
                    message_params([
                        ("marker", marker.to_string()),
                        ("target", "chapter".to_string()),
                    ]),
                    token,
                ));
            }

            if enabled.has(LintCode::MetadataOutsideTarget)
                && matches!(marker, "va" | "vp")
                && !matches_previous_marker_and_number(tokens, index, "v")
            {
                issues.push(simple_issue(
                    LintCode::MetadataOutsideTarget,
                    message_params([
                        ("marker", marker.to_string()),
                        ("target", "verse".to_string()),
                    ]),
                    token,
                ));
            }

            let validation_context = if marker == "periph" {
                SpecContext::Peripheral
            } else if marker_kind == MarkerKind::Chapter {
                top_level_root_context(document_state.kind, TopLevelSlot::Content)
            } else if document_state.current_note_context().is_none()
                && matches!(
                    marker_kind,
                    MarkerKind::Paragraph
                        | MarkerKind::Header
                        | MarkerKind::SidebarStart
                        | MarkerKind::TableRow
                )
            {
                let next_slot = document_state.select_top_level_slot(token);
                top_level_root_context(document_state.kind, next_slot)
            } else {
                document_state.current_validation_context_for_kind(marker_kind)
            };

            if enabled.has(LintCode::MarkerNotValidInContext)
                && marker_kind != MarkerKind::Unknown
                && !token.allows_effective_context(validation_context)
            {
                issues.push(simple_issue(
                    LintCode::MarkerNotValidInContext,
                    message_params([
                        ("marker", marker.to_string()),
                        ("context", spec_context_name(validation_context).to_string()),
                    ]),
                    token,
                ));
            }

            if marker_kind == MarkerKind::Note {
                note_stack.push(marker.to_string());
            }

            document_state.apply_marker(tokens, index, token);
            saw_content = true;
        } else if token_kind == TokenKind::EndMarker {
            let marker = token.marker().unwrap_or_default();
            if is_note_close_marker(marker) {
                while let Some(open) = note_stack.pop() {
                    if open == marker {
                        break;
                    }
                }
                document_state.close_note();
            }
            saw_content = true;
        } else if token_kind == TokenKind::Text {
            if !token.text().trim().is_empty() {
                saw_content = true;
            }
        } else if !matches!(token_kind, TokenKind::Newline | TokenKind::OptBreak) {
            saw_content = true;
        }
    }

    if enabled.has(LintCode::MissingIdMarker) && !id_seen {
        let code = LintCode::MissingIdMarker;
        let template = code.template();
        let params = MessageParams::default();
        issues.push(LintIssue {
            code,
            category: code.category(),
            severity: code.severity(),
            issue_type: code.issue_type(),
            template,
            message: render_template(template, &params),
            message_params: params,
            span: None,
            related_span: None,
            token_id: None,
            related_token_id: None,
            sid: None,
            marker: Some("id".to_string()),
            fix: None,
        });
    }
}

fn lint_unknown_markers<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    issues: &mut Vec<LintIssue>,
) {
    for token in &tokens[range] {
        if token.kind() != TokenKind::Marker {
            continue;
        }
        let Some(marker) = token.marker() else {
            continue;
        };
        if token_marker_kind(token) != MarkerKind::Unknown {
            continue;
        }
        issues.push(simple_issue(
            LintCode::UnknownMarker,
            marker_params(marker),
            token,
        ));
    }
}

fn lint_unknown_close_markers<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    issues: &mut Vec<LintIssue>,
) {
    for token in &tokens[range] {
        if token.kind() != TokenKind::EndMarker {
            continue;
        }
        let Some(marker) = token.marker() else {
            continue;
        };
        if token_marker_kind(token) != MarkerKind::Unknown {
            continue;
        }
        issues.push(simple_issue(
            LintCode::UnknownCloseMarker,
            marker_params(marker),
            token,
        ));
    }
}

fn lint_chapter_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut seen_chapters = FxHashSet::default();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];

        if token.kind() == TokenKind::Marker
            && token.marker() == Some("c")
            && let Some(number_index) = next_number_token_index(tokens, index + 1)
            && let Some(chapter) = token_primary_number(&tokens[number_index])
        {
            if enabled.has(LintCode::DuplicateChapterNumber) && seen_chapters.contains(&chapter) {
                issues.push(simple_issue_with_marker(
                    LintCode::DuplicateChapterNumber,
                    message_params([
                        ("chapter", chapter.to_string()),
                        ("marker", "c".to_string()),
                    ]),
                    "c",
                    &tokens[number_index],
                ));
            }
            seen_chapters.insert(chapter);
        }

        index += 1;
    }
}

fn lint_number_and_verse_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut current_chapter = 0u32;
    let mut verse_state_by_chapter: FxHashMap<u32, VerseState> = FxHashMap::default();

    for index in 0..tokens.len() {
        let token = &tokens[index];

        if token.kind() == TokenKind::Marker
            && token.marker() == Some("c")
            && let Some(number_index) = next_number_token_index(tokens, index + 1)
            && let Some(chapter) = token_primary_number(&tokens[number_index])
        {
            current_chapter = chapter;
        }

        if enabled.has(LintCode::NumberRangeNotPrecededByMarkerExpectingNumber)
            && token.kind() == TokenKind::Number
        {
            lint_number_predecessor(tokens, index, issues);
        }

        if token.kind() != TokenKind::Marker || token.marker() != Some("v") {
            continue;
        }

        let Some(number_index) = next_number_token_index(tokens, index + 1) else {
            continue;
        };
        let number_token = &tokens[number_index];
        let value = number_token.text().trim();
        let parsed_range = token_number_range(number_token).or_else(|| parse_number_range(value));

        if enabled.has(LintCode::InvalidNumberRange) && parsed_range.is_none() {
            issues.push(simple_issue_with_marker(
                LintCode::InvalidNumberRange,
                message_params([
                    ("found", value.to_string()),
                    ("verse", value.to_string()),
                    ("marker", "v".to_string()),
                    ("context", "verse-range".to_string()),
                ]),
                "v",
                number_token,
            ));
            continue;
        }

        let Some((start, end)) = parsed_range else {
            continue;
        };

        let chapter = if current_chapter == 0 {
            parse_sid_chapter(number_token.sid()).unwrap_or(0)
        } else {
            current_chapter
        };
        let chapter_state = verse_state_by_chapter.entry(chapter).or_default();

        let duplicate = (start..=end).any(|verse| chapter_state.seen.contains(&verse));
        if enabled.has(LintCode::DuplicateVerseNumber) && duplicate {
            issues.push(simple_issue_with_marker(
                LintCode::DuplicateVerseNumber,
                message_params([
                    ("verse", value.to_string()),
                    ("chapter", chapter.to_string()),
                    ("marker", "v".to_string()),
                ]),
                "v",
                number_token,
            ));
        }

        if enabled.has(LintCode::VerseIsEmpty) && !verse_has_text_or_note(tokens, number_index + 1)
        {
            issues.push(simple_issue_with_marker(
                LintCode::VerseIsEmpty,
                MessageParams::default(),
                "v",
                number_token,
            ));
        }

        for verse in start..=end {
            chapter_state.seen.insert(verse);
        }
    }
}

mod marker_balance;
#[cfg(not(target_arch = "wasm32"))]
use marker_balance::collect_marker_balance;
use marker_balance::lint_marker_balance_rules;

/// Whitespace rules driven by `MARKER_WHITESPACE` plus per-
/// `MarkerDefKind` defaults — every known marker has a profile, so
/// these rules cover the full marker catalog rather than just the
/// markers in the explicit table.
///
/// Spec-driven whitespace rules consuming the `MARKER_WHITESPACE` table:
///
/// - **MissingWhitespaceBeforeMarker**: the marker's row says
///   whitespace/newline is required *before*, but the prior token's
///   trailing text does not satisfy.
/// - **MissingHorizontalWhitespaceAfterMarkerName**: the marker's row
///   says HS (or any WS) is required after the marker name, but the
///   following token doesn't start with it.
/// - **MissingTagEndDelimiterAfterMarker**: the marker's row says a
///   tag-end delimiter (WS, EOI, or `|`) is required after the marker
///   name, but the following token is none of those.
/// - **MissingContentSpaceAfterCloseMarker**: a closing character
///   marker (`\nd*`, …) is immediately followed by alphabetic text
///   with no separating whitespace.
fn lint_whitespace_rules<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    use crate::marker_defs::lookup_marker_whitespace;
    use crate::whitespace::{
        StructuralWhitespaceRequirement as Req, is_any_whitespace_char,
        is_horizontal_whitespace_char, is_newline_char,
    };

    // Spec-driven rules (1, 2, 3) gated together — skip the
    // `lookup_marker_whitespace` per-token cost when none are enabled.
    let spec_rules_enabled = enabled.has(LintCode::MissingWhitespaceBeforeMarker)
        || enabled.has(LintCode::MissingHorizontalWhitespaceAfterMarkerName)
        || enabled.has(LintCode::MissingTagEndDelimiterAfterMarker);

    for index in range.clone() {
        let token = &tokens[index];
        let token_kind = token.kind();

        if !spec_rules_enabled {
            continue;
        }
        if !matches!(token_kind, TokenKind::Marker | TokenKind::Milestone) {
            continue;
        }
        let Some(marker) = token.marker() else {
            continue;
        };
        let Some(spec) = lookup_marker_whitespace(marker) else {
            continue;
        };

        // Rule 1: missing whitespace before marker. Read the prior
        // token's trailing source; "start of input" satisfies a
        // newline-before requirement (conceptually preceded by SOF).
        if enabled.has(LintCode::MissingWhitespaceBeforeMarker)
            && requirement_demands_leading_ws(spec.required_before_open)
            && index > 0
        {
            let prev = &tokens[index - 1];
            let prev_text = prev.text();
            let satisfied = match spec.required_before_open {
                Req::SingleNewline | Req::AtLeastOneNewline => {
                    prev_text.chars().next_back().is_some_and(is_newline_char)
                }
                Req::NewlineOrAnyWhitespaceBeforeMarker | Req::AtLeastOneWhitespace => prev_text
                    .chars()
                    .next_back()
                    .is_some_and(is_any_whitespace_char),
                Req::AtLeastOneHorizontalWhitespace => prev_text
                    .chars()
                    .next_back()
                    .is_some_and(is_horizontal_whitespace_char),
                _ => true,
            };
            if !satisfied {
                let prefix = preferred_ws(spec.format_preference_before_open);
                let mut issue = simple_issue(
                    LintCode::MissingWhitespaceBeforeMarker,
                    marker_params(marker),
                    token,
                );
                if !prefix.is_empty() && token.id().is_some() {
                    issue = issue.with_fix(prepend_ws_fix(
                        "insert-whitespace-before-marker",
                        "InsertWhitespaceBeforeMarker",
                        token,
                        prefix,
                    ));
                }
                issues.push(issue);
            }
        }

        // Rules 2 & 3: required whitespace immediately after the marker
        // name. Rule 2 fires when the spec demands HS / newline / any
        // WS; rule 3 fires when the spec demands a tag-end delimiter
        // (WS, EOI, or the `|` attribute opener).
        let marker_tail = token.text().chars().next_back();
        let after_name_satisfied_by_marker_token = match spec.required_after_open_name {
            Req::AtLeastOneHorizontalWhitespace => {
                marker_tail.is_some_and(is_horizontal_whitespace_char)
            }
            Req::AtLeastOneWhitespace => marker_tail.is_some_and(is_any_whitespace_char),
            Req::SingleNewline | Req::AtLeastOneNewline => marker_tail.is_some_and(is_newline_char),
            Req::OptionalHorizontalWhitespace | Req::OptionalWhitespace => true,
            Req::TagEndDelimiter => marker_tail.is_some_and(is_any_whitespace_char),
            _ => true,
        };

        let next_token = tokens.get(index + 1);
        let next_text = next_token.map(|n| n.text()).unwrap_or("");
        let next_first = next_text.chars().next();

        if !after_name_satisfied_by_marker_token && marker_is_intentionally_empty_block(marker) {
            // `\b` and other no-content block markers: the spec table
            // marks them `SingleNewline` after the name, but the real
            // requirement is "nothing on this line." A newline (or
            // end-of-input) must arrive before any content; trailing
            // horizontal whitespace before it is harmless and the
            // formatter normalises it, so `\b \n` is clean. We flag only
            // genuine content riding the same line (`\b here`), anchored
            // at that content rather than at the marker.
            if enabled.has(LintCode::ContentAfterBlankMarker)
                && let Some(content) = content_after_blank_marker(tokens, index)
            {
                issues.push(simple_issue(
                    LintCode::ContentAfterBlankMarker,
                    marker_params(marker),
                    content,
                ));
            }
        } else if !after_name_satisfied_by_marker_token {
            match spec.required_after_open_name {
                Req::AtLeastOneHorizontalWhitespace
                | Req::AtLeastOneWhitespace
                | Req::SingleNewline
                | Req::AtLeastOneNewline
                    if enabled.has(LintCode::MissingHorizontalWhitespaceAfterMarkerName) =>
                {
                    let satisfied = match spec.required_after_open_name {
                        Req::AtLeastOneHorizontalWhitespace => {
                            next_first.is_some_and(is_horizontal_whitespace_char)
                        }
                        Req::AtLeastOneWhitespace => next_first.is_some_and(is_any_whitespace_char),
                        Req::SingleNewline | Req::AtLeastOneNewline => {
                            next_first.is_some_and(is_newline_char)
                        }
                        _ => true,
                    };
                    if !satisfied && let Some(_next) = next_token {
                        let prefix = preferred_ws(spec.format_preference_after_open_name);
                        let mut issue = simple_issue(
                            LintCode::MissingHorizontalWhitespaceAfterMarkerName,
                            marker_params(marker),
                            token,
                        );
                        if !prefix.is_empty() && token.id().is_some() {
                            issue = issue.with_fix(append_ws_fix(
                                "insert-whitespace-after-marker-name",
                                "InsertWhitespaceAfterMarkerName",
                                token,
                                prefix,
                            ));
                        }
                        issues.push(issue);
                    }
                }
                Req::TagEndDelimiter
                    if enabled.has(LintCode::MissingTagEndDelimiterAfterMarker) =>
                {
                    let satisfied = match next_token {
                        None => true, // EOI satisfies tag-end
                        Some(_) => {
                            next_first.is_some_and(is_any_whitespace_char)
                                || matches!(next_first, Some('|'))
                        }
                    };
                    if !satisfied && let Some(_next) = next_token {
                        let prefix = preferred_ws(spec.format_preference_after_open_name);
                        let mut issue = simple_issue(
                            LintCode::MissingTagEndDelimiterAfterMarker,
                            marker_params(marker),
                            token,
                        );
                        if !prefix.is_empty() && token.id().is_some() {
                            issue = issue.with_fix(append_ws_fix(
                                "insert-tag-end-delimiter-after-marker",
                                "InsertTagEndDelimiterAfterMarker",
                                token,
                                prefix,
                            ));
                        }
                        issues.push(issue);
                    }
                }
                _ => {}
            }
        }
    }

    // Rule 6: closing character marker immediately followed by
    // alphabetic text. Pure pairwise check; doesn't consult the spec
    // table (the pattern is content-style, not structural). Reads the
    // following token from the full slice so a close marker at the end of
    // a segment still sees its neighbour; the final token (no next) is
    // skipped, matching the whole-book `0..len-1` bound.
    if enabled.has(LintCode::MissingContentSpaceAfterCloseMarker) {
        for index in range {
            let token = &tokens[index];
            if token.kind() != TokenKind::EndMarker {
                continue;
            }
            let Some(marker) = token.marker() else {
                continue;
            };
            let Some(next) = tokens.get(index + 1) else {
                continue;
            };
            if next.kind() != TokenKind::Text {
                continue;
            }
            let next_text = next.text();
            if next_text.chars().next().is_some_and(|c| c.is_alphabetic()) {
                issues.push(simple_issue(
                    LintCode::MissingContentSpaceAfterCloseMarker,
                    marker_params(marker),
                    token,
                ));
            }
        }
    }
}

fn requirement_demands_leading_ws(req: crate::whitespace::StructuralWhitespaceRequirement) -> bool {
    use crate::whitespace::StructuralWhitespaceRequirement as Req;
    matches!(
        req,
        Req::SingleNewline
            | Req::AtLeastOneNewline
            | Req::NewlineOrAnyWhitespaceBeforeMarker
            | Req::AtLeastOneWhitespace
            | Req::AtLeastOneHorizontalWhitespace
    )
}

fn lint_unknown_token_like<T: LintableToken>(token: &T) -> Option<LintIssue> {
    let text = token.text();
    let trimmed = text.trim_start_matches([' ', '\t']);
    let remainder = trimmed.strip_prefix('\\')?;
    let marker_len = remainder
        .chars()
        .take_while(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || *ch == '-')
        .map(char::len_utf8)
        .sum::<usize>();
    if marker_len == 0 {
        return None;
    }
    let marker = &remainder[..marker_len];
    if lookup_marker(marker).kind == MarkerKind::Unknown {
        return None;
    }
    let after = &remainder[marker_len..];
    if after.is_empty() || after.starts_with(char::is_whitespace) {
        return None;
    }
    Some(simple_issue_with_marker(
        LintCode::UnknownToken,
        message_params([("text", token.text().to_string())]),
        marker,
        token,
    ))
}

fn next_book_code_after_marker<T: LintableToken>(
    tokens: &[T],
    marker_index: usize,
) -> Option<&str> {
    let next_index = next_significant_token_index(tokens, marker_index + 1)?;
    (tokens[next_index].kind() == TokenKind::BookCode).then(|| tokens[next_index].text().trim())
}

fn infer_document_kind(book_code: &str) -> DocumentKind {
    match book_code {
        "FRT" | "INT" | "BAK" | "OTH" => DocumentKind::PeripheralDivided,
        "CNC" | "GLO" | "TDX" | "NDX" => DocumentKind::PeripheralStandalone,
        _ => DocumentKind::Scripture,
    }
}

fn lint_number_predecessor<T: LintableToken>(
    tokens: &[T],
    index: usize,
    issues: &mut Vec<LintIssue>,
) {
    let token = &tokens[index];
    let Some(prev_index) = previous_significant_token_index(tokens, index) else {
        issues.push(simple_issue(
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
            MessageParams::default(),
            token,
        ));
        return;
    };

    let prev = &tokens[prev_index];
    let valid = prev.kind() == TokenKind::Marker
        && matches!(prev.marker(), Some("v" | "vp" | "va" | "c" | "ca" | "cp"));
    if !valid {
        issues.push(simple_issue(
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
            MessageParams::default(),
            token,
        ));
    }
}

fn matches_previous_marker_and_number<T: LintableToken>(
    tokens: &[T],
    marker_index: usize,
    expected_marker: &str,
) -> bool {
    let Some(prev_index) = previous_significant_token_index(tokens, marker_index) else {
        return false;
    };
    if tokens[prev_index].kind() != TokenKind::Number {
        return false;
    }
    let Some(before_number_index) = previous_significant_token_index(tokens, prev_index) else {
        return false;
    };
    tokens[before_number_index].kind() == TokenKind::Marker
        && tokens[before_number_index].marker() == Some(expected_marker)
}

// Note: the helpers that used to live here (close_open_frames_for_boundary,
// handle_close_marker, handle_structural_close, handle_milestone_end,
// close_structural_frames_for_boundary, unclosed_marker_issue) were
// retired when `lint_marker_balance_rules` migrated to a walker
// visitor. The walker's events now drive all unclosed/misnested/stray
// detection; see `MarkerBalanceVisitor`.

fn verse_has_text_or_note<T: LintableToken>(tokens: &[T], start: usize) -> bool {
    for token in tokens.iter().skip(start) {
        match token.kind() {
            TokenKind::Newline => continue,
            TokenKind::Text => {
                if !token.text().trim().is_empty() {
                    return true;
                }
            }
            TokenKind::Marker => {
                if matches!(token.marker(), Some("f" | "fe" | "ef" | "x" | "ex")) {
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }
    false
}

fn token_primary_number<T: LintableToken>(token: &T) -> Option<u32> {
    token
        .number_info()
        .map(|(start, _, _)| start)
        .or_else(|| parse_primary_number(token.text()))
}

fn token_number_range<T: LintableToken>(token: &T) -> Option<(u32, u32)> {
    token
        .number_info()
        .and_then(|(start, end, kind)| match kind {
            NumberRangeKind::Single => Some((start, start)),
            NumberRangeKind::Range => end.map(|end| (start, end)),
            NumberRangeKind::Sequence | NumberRangeKind::SequenceWithRange => {
                Some((start, end.unwrap_or(start)))
            }
        })
}

fn parse_primary_number(text: &str) -> Option<u32> {
    let digits = text
        .trim()
        .split(['-', ','])
        .next()
        .unwrap_or("")
        .trim_matches(|ch: char| !ch.is_ascii_digit());
    digits.parse().ok()
}

fn parse_number_range(text: &str) -> Option<(u32, u32)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('-');
    let start = parts
        .next()?
        .split(',')
        .next()?
        .trim_matches(|ch: char| !ch.is_ascii_digit())
        .parse::<u32>()
        .ok()?;
    let end_raw = match parts.next() {
        Some(value) => value,
        None => trimmed,
    };
    let end = end_raw
        .split(',')
        .next_back()?
        .trim_matches(|ch: char| !ch.is_ascii_digit())
        .parse::<u32>()
        .ok()?;
    if parts.next().is_some() || start == 0 || end == 0 || start > end {
        return None;
    }
    Some((start, end))
}

fn parse_sid_chapter(sid: Option<String>) -> Option<u32> {
    let sid = sid?;
    let reference = sid.split("_dup_").next().unwrap_or(&sid);
    let (_, chap_and_verse) = reference.rsplit_once(' ')?;
    let chapter = chap_and_verse.split(':').next()?;
    chapter.parse().ok()
}

fn top_level_contexts_for(kind: DocumentKind) -> &'static [(TopLevelSlot, SpecContext)] {
    match kind {
        DocumentKind::Scripture => &[
            (TopLevelSlot::Headers, SpecContext::BookHeaders),
            (TopLevelSlot::Titles, SpecContext::BookTitles),
            (TopLevelSlot::Introduction, SpecContext::BookIntroduction),
            (
                TopLevelSlot::IntroductionEndTitles,
                SpecContext::BookIntroductionEndTitles,
            ),
            (TopLevelSlot::Content, SpecContext::ChapterContent),
        ],
        DocumentKind::PeripheralStandalone | DocumentKind::PeripheralDivided => &[
            (TopLevelSlot::Headers, SpecContext::BookHeaders),
            (TopLevelSlot::Titles, SpecContext::BookTitles),
            (TopLevelSlot::Introduction, SpecContext::BookIntroduction),
            (
                TopLevelSlot::IntroductionEndTitles,
                SpecContext::BookIntroductionEndTitles,
            ),
            (TopLevelSlot::Content, SpecContext::PeripheralContent),
        ],
    }
}

fn top_level_slot_index(slot: TopLevelSlot) -> usize {
    match slot {
        TopLevelSlot::AwaitDivision | TopLevelSlot::Headers => 0,
        TopLevelSlot::Titles => 1,
        TopLevelSlot::Introduction => 2,
        TopLevelSlot::IntroductionEndTitles => 3,
        TopLevelSlot::Content => 4,
    }
}

fn top_level_root_context(kind: DocumentKind, slot: TopLevelSlot) -> SpecContext {
    DocumentLintState {
        kind,
        slot,
        ..DocumentLintState::default()
    }
    .current_root_context()
}

fn paragraph_block_context_for<T: LintableToken>(token: &T, marker: &str) -> SpecContext {
    let inline_context = token
        .structural()
        .and_then(|structural| structural.inline_context)
        .or_else(|| marker_inline_context(marker));
    paragraph_block_context_from_inline(inline_context)
}

fn paragraph_block_context_from_inline(inline_context: Option<InlineContext>) -> SpecContext {
    match inline_context.unwrap_or(InlineContext::Para) {
        InlineContext::Para => SpecContext::Para,
        InlineContext::Section => SpecContext::Section,
        InlineContext::List => SpecContext::List,
        InlineContext::Table => SpecContext::Table,
    }
}

fn note_context_for_marker(marker: &str) -> SpecContext {
    marker_note_context(marker).unwrap_or(SpecContext::Footnote)
}

fn token_structural_info<T: LintableToken>(token: &T) -> Option<StructuralMarkerInfo> {
    token.structural().or_else(|| match token.kind() {
        TokenKind::Marker | TokenKind::EndMarker => token.marker().map(|marker| {
            let kind = lookup_marker_metadata(marker).map(|(_, kind, _)| kind);
            structural_marker_info(marker, kind)
        }),
        _ => None,
    })
}

fn token_marker_kind<T: LintableToken>(token: &T) -> MarkerKind {
    if let Some(structural) = token_structural_info(token) {
        return match structural.scope_kind {
            StructuralScopeKind::Unknown => MarkerKind::Unknown,
            StructuralScopeKind::Header => MarkerKind::Header,
            StructuralScopeKind::Block => MarkerKind::Paragraph,
            StructuralScopeKind::Note => MarkerKind::Note,
            StructuralScopeKind::Character => MarkerKind::Character,
            StructuralScopeKind::Milestone => MarkerKind::MilestoneStart,
            StructuralScopeKind::Chapter => MarkerKind::Chapter,
            StructuralScopeKind::Verse => MarkerKind::Verse,
            StructuralScopeKind::TableRow => MarkerKind::TableRow,
            StructuralScopeKind::TableCell => MarkerKind::TableCell,
            StructuralScopeKind::Sidebar => MarkerKind::SidebarStart,
            StructuralScopeKind::Periph => MarkerKind::Periph,
            StructuralScopeKind::Meta => MarkerKind::Meta,
        };
    }
    token
        .marker()
        .map(|name| lookup_marker(name).kind)
        .unwrap_or(MarkerKind::Unknown)
}

fn next_number_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::Newline => continue,
            TokenKind::Number => return Some(index),
            _ => return None,
        }
    }
    None
}

fn next_significant_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind() != TokenKind::Newline {
            return Some(index);
        }
    }
    None
}

fn previous_significant_token_index<T: LintableToken>(tokens: &[T], end: usize) -> Option<usize> {
    let mut index = end;
    while index > 0 {
        index -= 1;
        if tokens[index].kind() != TokenKind::Newline {
            return Some(index);
        }
    }
    None
}

fn is_body_paragraph_marker(marker: &str) -> bool {
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

fn marker_is_intentionally_empty_block(marker: &str) -> bool {
    matches!(marker, "b")
}

/// For a no-content block marker like `\b`, find the first token carrying
/// genuine content on the same line — i.e. before the newline that must
/// follow the marker. Trailing horizontal whitespace before that newline
/// is allowed and skipped, so `\b \n` returns `None` (clean) while
/// `\b here` returns the `here` token (flagged). End-of-input with only
/// horizontal whitespace also counts as clean.
fn content_after_blank_marker<T: LintableToken>(tokens: &[T], marker_index: usize) -> Option<&T> {
    use crate::whitespace::{is_horizontal_whitespace_char, is_newline_char};
    for token in &tokens[marker_index + 1..] {
        for ch in token.text().chars() {
            if is_newline_char(ch) {
                return None;
            }
            if !is_horizontal_whitespace_char(ch) {
                return Some(token);
            }
        }
    }
    None
}

fn empty_paragraph_boundary_index<T: LintableToken>(
    tokens: &[T],
    marker_index: usize,
) -> Option<usize> {
    let mut index = marker_index + 1;
    while index < tokens.len() {
        let token = &tokens[index];
        match token.kind() {
            TokenKind::Newline | TokenKind::OptBreak => index += 1,
            TokenKind::Text if token.text().trim().is_empty() => index += 1,
            TokenKind::Marker => return empty_paragraph_boundary_token(token).then_some(index),
            _ => return None,
        }
    }
    None
}

fn empty_paragraph_boundary_token<T: LintableToken>(token: &T) -> bool {
    let marker = token.marker().unwrap_or_default();
    if is_body_paragraph_marker(marker) {
        return true;
    }
    matches!(
        token_marker_kind(token),
        MarkerKind::Header
            | MarkerKind::Chapter
            | MarkerKind::Periph
            | MarkerKind::SidebarStart
            | MarkerKind::TableRow
            | MarkerKind::Unknown
    )
}

fn is_note_close_marker(marker: &str) -> bool {
    matches!(marker, "f" | "fe" | "ef" | "x" | "ex")
}

/// Lowercase kebab-case identifier for a `SpecContext`. Stable wire
/// value used by the `marker-not-valid-in-context` ICU template's
/// `{context, select, …}` branches and by tooling that filters by
/// context. Keep aligned with the template's branch keys.
fn spec_context_name(context: SpecContext) -> &'static str {
    match context {
        SpecContext::Scripture => "scripture",
        SpecContext::BookIdentification => "book-identification",
        SpecContext::BookHeaders => "book-headers",
        SpecContext::BookTitles => "book-titles",
        SpecContext::BookIntroduction => "book-introduction",
        SpecContext::BookIntroductionEndTitles => "book-introduction-end-titles",
        SpecContext::BookChapterLabel => "book-chapter-label",
        SpecContext::ChapterContent => "chapter-content",
        SpecContext::Peripheral => "peripheral",
        SpecContext::PeripheralContent => "peripheral-content",
        SpecContext::PeripheralDivision => "peripheral-division",
        SpecContext::Chapter => "chapter",
        SpecContext::Verse => "verse",
        SpecContext::Section => "section",
        SpecContext::Para => "para",
        SpecContext::List => "list",
        SpecContext::Table => "table",
        SpecContext::Sidebar => "sidebar",
        SpecContext::Footnote => "footnote",
        SpecContext::CrossReference => "cross-reference",
    }
}

fn issue<T: LintableToken, U: LintableToken>(
    code: LintCode,
    params: MessageParams,
    token: &T,
    related: Option<&U>,
) -> LintIssue {
    let template = code.template();
    let message = render_template(template, &params);
    LintIssue {
        code,
        category: code.category(),
        severity: code.severity(),
        issue_type: code.issue_type(),
        template,
        message,
        message_params: params,
        span: token.span(),
        related_span: related.and_then(LintableToken::span),
        token_id: token.id(),
        related_token_id: related.and_then(LintableToken::id),
        sid: token.sid(),
        marker: token.marker().map(ToOwned::to_owned),
        fix: None,
    }
}

impl LintIssue {
    fn with_fix(mut self, fix: TokenFix) -> Self {
        self.fix = Some(fix);
        self
    }
}

/// Resolve a `FormatWhitespacePreference` to the literal whitespace
/// string the formatter would insert. `None` falls back to a single
/// space — matches the plan's "single space if the preference is unset"
/// rule and the historical `missing_separator_fix` behaviour.
fn preferred_ws(pref: Option<crate::whitespace::FormatWhitespacePreference>) -> &'static str {
    use crate::whitespace::FormatWhitespacePreference as Pref;
    match pref {
        Some(Pref::PreferSingleNewline) => "\n",
        Some(Pref::PreferRemoveAllWhitespace) => "",
        Some(Pref::PreferSingleSpace) | None => " ",
    }
}

/// Replace `target` with a token whose text is `prefix` + the target's
/// existing text. Used by rules 1, 2, 3 to insert the required
/// whitespace immediately before the offending token.
fn prepend_ws_fix<T: LintableToken>(code: &str, label: &str, target: &T, prefix: &str) -> TokenFix {
    TokenFix::ReplaceToken {
        code: code.to_string(),
        label: label.to_string(),
        label_params: MessageParams::default(),
        target_token_id: target.id().expect("fixable token should have id"),
        replacements: vec![crate::format::TokenTemplate {
            kind: target.kind(),
            text: format!("{}{}", prefix, target.text()),
            marker: target.marker().map(ToOwned::to_owned),
            sid: target.sid(),
        }],
    }
}

/// Missing after-name separators now belong to the marker token itself.
/// The fix preserves the following content token exactly as typed.
fn append_ws_fix<T: LintableToken>(code: &str, label: &str, target: &T, suffix: &str) -> TokenFix {
    TokenFix::ReplaceToken {
        code: code.to_string(),
        label: label.to_string(),
        label_params: MessageParams::default(),
        target_token_id: target.id().expect("fixable token should have id"),
        replacements: vec![crate::format::TokenTemplate {
            kind: target.kind(),
            text: format!("{}{}", target.text(), suffix),
            marker: target.marker().map(ToOwned::to_owned),
            sid: target.sid(),
        }],
    }
}

fn simple_issue<T: LintableToken>(code: LintCode, params: MessageParams, token: &T) -> LintIssue {
    issue(code, params, token, None::<&T>)
}

fn simple_issue_with_marker<T: LintableToken>(
    code: LintCode,
    params: MessageParams,
    marker: &str,
    token: &T,
) -> LintIssue {
    let mut issue = simple_issue(code, params, token);
    issue.marker = Some(marker.to_string());
    issue
}

/// Build a single-`marker`-param map, used by every rule that surfaces
/// the offending marker name.
fn marker_params(marker: &str) -> MessageParams {
    message_params([("marker", marker.to_string())])
}

/// Canonical output order for findings. Independent of the order rule groups
/// happen to run in, so callers see a stable sequence and a chapter-parallel
/// linter (which produces findings out of segment order) can sort to the same
/// result. Ordered by primary source position, then the stable lint-code
/// identifier, then related span; spanless document findings sort last. `token_id`
/// / `marker` / `message` are pure tie-breakers for determinism. Deliberately
/// NOT ordered by SID — duplicate, malformed, or decreasing references are valid
/// linter inputs and must not drive output order.
fn canonical_sort(issues: &mut [LintIssue]) {
    fn span_key(span: Option<Span>) -> (u8, u32, u32) {
        match span {
            Some(span) => (0, span.start, span.end),
            None => (1, u32::MAX, u32::MAX),
        }
    }
    issues.sort_by(|a, b| {
        span_key(a.span)
            .cmp(&span_key(b.span))
            .then_with(|| a.code.code().cmp(b.code.code()))
            .then_with(|| span_key(a.related_span).cmp(&span_key(b.related_span)))
            .then_with(|| a.token_id.cmp(&b.token_id))
            .then_with(|| a.marker.cmp(&b.marker))
            .then_with(|| a.message.cmp(&b.message))
    });
}

fn dedupe_issues(issues: Vec<LintIssue>) -> Vec<LintIssue> {
    let mut seen = FxHashSet::default();
    let mut deduped = Vec::new();
    for issue in issues {
        let identity = (
            issue.code,
            issue.span.map(|span| (span.start, span.end)),
            issue.related_span.map(|span| (span.start, span.end)),
            issue.token_id.clone(),
        );
        if seen.insert(identity) {
            deduped.push(issue);
        }
    }
    deduped
}

fn apply_suppressions(
    issues: Vec<LintIssue>,
    suppressions: &[LintSuppression],
) -> (Vec<LintIssue>, usize) {
    let suppression_keys = suppressions
        .iter()
        .map(|suppression| (suppression.code, suppression.sid.as_str()))
        .collect::<FxHashSet<_>>();
    let mut kept = Vec::new();
    let mut suppressed_count = 0;
    for issue in issues {
        if issue
            .sid
            .as_deref()
            .is_some_and(|sid| suppression_keys.contains(&(issue.code, sid)))
        {
            suppressed_count += 1;
        } else {
            kept.push(issue);
        }
    }
    (kept, suppressed_count)
}

fn summarize(issues: &[LintIssue], suppressed_count: usize) -> LintSummary {
    let mut by_category = BTreeMap::new();
    let mut by_severity = BTreeMap::new();
    let mut by_issue_type = BTreeMap::new();

    for issue in issues {
        *by_category.entry(issue.category).or_insert(0) += 1;
        *by_severity.entry(issue.severity).or_insert(0) += 1;
        *by_issue_type.entry(issue.issue_type).or_insert(0) += 1;
    }

    LintSummary {
        by_category,
        by_severity,
        by_issue_type,
        total_count: issues.len(),
        suppressed_count,
    }
}

fn format_sid(sid: Sid) -> String {
    sid.to_string()
}

fn format_token_id(id: TokenId<'_>) -> String {
    format!("{}-{}", id.book_code, id.index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every `LintCode` variant. The bitmask drift guard iterates this; if a
    /// variant is added, the exhaustive match below fails to compile, forcing
    /// this list (and the `< 64` bit-width invariant) to be kept in sync.
    const ALL_LINT_CODES: &[LintCode] = &[
        LintCode::MissingIdMarker,
        LintCode::DuplicateIdMarker,
        LintCode::IdMarkerNotAtFileStart,
        LintCode::EmptyParagraph,
        LintCode::MissingChapterNumber,
        LintCode::MissingVerseNumber,
        LintCode::VerseIsEmpty,
        LintCode::UnknownToken,
        LintCode::UnknownMarker,
        LintCode::UnknownCloseMarker,
        LintCode::ContentBeforeFirstChapter,
        LintCode::VerseOutsideExplicitParagraph,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::MetadataOutsideTarget,
        LintCode::MarkerNotValidInContext,
        LintCode::MissingMilestoneSelfClose,
        LintCode::StrayCloseMarker,
        LintCode::MisnestedCloseMarker,
        LintCode::ImplicitlyClosedMarker,
        LintCode::UnclosedMarker,
        LintCode::DuplicateChapterNumber,
        LintCode::DuplicateVerseNumber,
        LintCode::InvalidNumberRange,
        LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        LintCode::MissingWhitespaceBeforeMarker,
        LintCode::MissingHorizontalWhitespaceAfterMarkerName,
        LintCode::MissingTagEndDelimiterAfterMarker,
        LintCode::MissingContentSpaceAfterCloseMarker,
        LintCode::VerseInSectionOrOtherParagraph,
        LintCode::ContentAfterBlankMarker,
    ];

    /// Compile-time exhaustiveness anchor: adding a `LintCode` variant breaks
    /// this match until `ALL_LINT_CODES` above is updated too.
    fn _all_lint_codes_is_exhaustive(code: LintCode) {
        match code {
            LintCode::MissingIdMarker
            | LintCode::DuplicateIdMarker
            | LintCode::IdMarkerNotAtFileStart
            | LintCode::EmptyParagraph
            | LintCode::MissingChapterNumber
            | LintCode::MissingVerseNumber
            | LintCode::VerseIsEmpty
            | LintCode::UnknownToken
            | LintCode::UnknownMarker
            | LintCode::UnknownCloseMarker
            | LintCode::ContentBeforeFirstChapter
            | LintCode::VerseOutsideExplicitParagraph
            | LintCode::NoteSubmarkerOutsideNote
            | LintCode::MetadataOutsideTarget
            | LintCode::MarkerNotValidInContext
            | LintCode::MissingMilestoneSelfClose
            | LintCode::StrayCloseMarker
            | LintCode::MisnestedCloseMarker
            | LintCode::ImplicitlyClosedMarker
            | LintCode::UnclosedMarker
            | LintCode::DuplicateChapterNumber
            | LintCode::DuplicateVerseNumber
            | LintCode::InvalidNumberRange
            | LintCode::NumberRangeNotPrecededByMarkerExpectingNumber
            | LintCode::MissingWhitespaceBeforeMarker
            | LintCode::MissingHorizontalWhitespaceAfterMarkerName
            | LintCode::MissingTagEndDelimiterAfterMarker
            | LintCode::MissingContentSpaceAfterCloseMarker
            | LintCode::VerseInSectionOrOtherParagraph
            | LintCode::ContentAfterBlankMarker => {}
        }
    }

    /// Reference implementation of the pre-bitmask `EnabledCodes::has`, kept in
    /// the test to assert the `u64` bitmask reproduces the old B-tree logic
    /// exactly across every code and every allowed/disabled/document-scope
    /// combination (drift guard for Task 1).
    fn reference_has(
        allowed: &Option<BTreeSet<LintCode>>,
        disabled: &BTreeSet<LintCode>,
        run_document_rules: bool,
        code: LintCode,
    ) -> bool {
        if disabled.contains(&code) {
            return false;
        }
        if !run_document_rules && code.category() == LintCategory::Document {
            return false;
        }
        allowed.as_ref().is_none_or(|a| a.contains(&code))
    }

    #[test]
    fn enabled_codes_bitmask_matches_btreeset() {
        // Every discriminant must fit a u64 bit index; otherwise `bit()` shifts
        // out of range and the mask silently drops codes.
        for &code in ALL_LINT_CODES {
            assert!(
                (code as u32) < 64,
                "LintCode discriminant {} exceeds u64 bitmask width",
                code as u32
            );
        }

        // A spread of allowed/disabled subsets: none, all, and every prefix.
        let mut allowed_variants: Vec<Option<Vec<LintCode>>> = vec![None];
        for len in 0..=ALL_LINT_CODES.len() {
            allowed_variants.push(Some(ALL_LINT_CODES[..len].to_vec()));
        }
        let mut disabled_variants: Vec<Vec<LintCode>> = vec![Vec::new()];
        for len in 0..=ALL_LINT_CODES.len() {
            // Take from the tail so allowed/disabled sets overlap and diverge.
            disabled_variants.push(ALL_LINT_CODES[ALL_LINT_CODES.len() - len..].to_vec());
        }

        for run_document_rules in [true, false] {
            let scope = if run_document_rules {
                LintScope::Book
            } else {
                LintScope::Chapter(1)
            };
            assert_eq!(scope.runs_document_rules(), run_document_rules);

            for allowed in &allowed_variants {
                for disabled in &disabled_variants {
                    let options = LintOptions {
                        scope,
                        enabled_codes: allowed.clone(),
                        disabled_codes: disabled.clone(),
                        suppressed: Vec::new(),
                        allow_implicit_chapter_content_verse: false,
                    };
                    let bitmask = EnabledCodes::new(&options);

                    let ref_allowed: Option<BTreeSet<LintCode>> =
                        allowed.as_ref().map(|c| c.iter().copied().collect());
                    let ref_disabled: BTreeSet<LintCode> = disabled.iter().copied().collect();

                    for &code in ALL_LINT_CODES {
                        assert_eq!(
                            bitmask.has(code),
                            reference_has(&ref_allowed, &ref_disabled, run_document_rules, code),
                            "has disagreement for {code:?} (allowed={allowed:?}, disabled={disabled:?}, run_document_rules={run_document_rules})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn template_renders_plain_placeholders() {
        let params = message_params([("marker", "p".to_string())]);
        let rendered = render_template("\\{marker} needs a space.", &params);
        assert_eq!(rendered, "\\p needs a space.");
    }

    #[test]
    fn template_renders_number_placeholders() {
        let params = message_params([("chapter", "3".to_string())]);
        let rendered =
            render_template("Chapter {chapter, number} appears more than once.", &params);
        assert_eq!(rendered, "Chapter 3 appears more than once.");
    }

    #[test]
    fn template_renders_select_arms() {
        let template = "{kind, select, note {Note} character {Character marker} other {Marker}} \\{marker} was opened.";
        let note_params =
            message_params([("kind", "note".to_string()), ("marker", "f".to_string())]);
        assert_eq!(
            render_template(template, &note_params),
            "Note \\f was opened."
        );

        let char_params = message_params([
            ("kind", "character".to_string()),
            ("marker", "nd".to_string()),
        ]);
        assert_eq!(
            render_template(template, &char_params),
            "Character marker \\nd was opened."
        );

        let other_params = message_params([
            ("kind", "milestone".to_string()),
            ("marker", "zaln-s".to_string()),
        ]);
        assert_eq!(
            render_template(template, &other_params),
            "Marker \\zaln-s was opened."
        );
    }

    #[test]
    fn template_select_falls_through_when_param_missing() {
        let template = "{form, select, milestone-end {EM} other {NAMED}}";
        let rendered = render_template(template, &MessageParams::default());
        assert_eq!(rendered, "NAMED");
    }

    #[test]
    fn missing_whitespace_before_marker_is_flagged() {
        // `\p` arrives directly after the previous text token with no
        // separating whitespace — `\p`'s row demands
        // NewlineOrAnyWhitespaceBeforeMarker.
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p first\\p second\n",
            LintOptions::scoped(LintScope::Book),
        );
        let flagged = result
            .issues
            .iter()
            .any(|i| i.code == LintCode::MissingWhitespaceBeforeMarker);
        assert!(
            flagged,
            "expected missing-whitespace-before-marker, got: {:?}",
            result.issues.iter().map(|i| i.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn missing_whitespace_before_marker_autofix_prepends_preferred_ws() {
        // `\p`'s row pairs `NewlineOrAnyWhitespaceBeforeMarker` with
        // `PreferSingleNewline`. Autofix should prepend "\n" to the
        // jammed marker token.
        let tokens = vec![
            crate::FormatToken {
                kind: TokenKind::Text,
                text: "tail".to_string(),
                marker: None,
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-0".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
            crate::FormatToken {
                kind: TokenKind::Marker,
                text: "\\p".to_string(),
                marker: Some("p".to_string()),
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-1".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
        ];

        let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        let issue = result
            .issues
            .into_iter()
            .find(|i| i.code == LintCode::MissingWhitespaceBeforeMarker)
            .expect("expected missing-whitespace-before-marker issue");
        let fix = issue.fix.expect("expected autofix");
        let fixed = apply_token_fix(&tokens, &fix);

        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed[0].text, "tail");
        assert_eq!(fixed[1].text, "\n\\p");
    }

    #[test]
    fn missing_horizontal_whitespace_after_marker_name_is_flagged_with_autofix() {
        // Hand-built: `\c` Marker followed by `1` Number with no
        // whitespace between. `\c`'s row demands
        // AtLeastOneHorizontalWhitespace after the marker name.
        let tokens = vec![
            crate::FormatToken {
                kind: TokenKind::Marker,
                text: "\\c".to_string(),
                marker: Some("c".to_string()),
                sid: Some("GEN 1:0".to_string()),
                id: Some("GEN-0".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
            crate::FormatToken {
                kind: TokenKind::Number,
                text: "1".to_string(),
                marker: None,
                sid: Some("GEN 1:0".to_string()),
                id: Some("GEN-1".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
        ];

        let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        let issue = result
            .issues
            .into_iter()
            .find(|i| i.code == LintCode::MissingHorizontalWhitespaceAfterMarkerName)
            .expect("expected missing-horizontal-whitespace-after-marker-name issue");
        let fix = issue.fix.expect("expected autofix");
        let fixed = apply_token_fix(&tokens, &fix);

        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed[0].text, "\\c ");
        assert_eq!(fixed[1].text, "1");
    }

    #[test]
    fn content_after_blank_marker_distinguishes_content_from_trailing_whitespace() {
        // `\b` is a blank line that takes no content. The rule must catch
        // genuine content riding the same line, but must NOT fire on a
        // bare trailing space before the newline (cosmetic — the formatter
        // tidies it) or on content that sits on its own line after `\b`.
        let has_blank = |src: &str| {
            lint_usfm(src, LintOptions::scoped(LintScope::Book))
                .issues
                .iter()
                .any(|i| i.code == LintCode::ContentAfterBlankMarker)
        };

        // Content on `\b`'s own line → flagged.
        assert!(
            has_blank("\\id GEN\n\\c 1\n\\v 1 text\n\\b here asdf\n"),
            "content sharing \\b's line should be flagged"
        );
        // Newline right after `\b`, content on the next line → clean.
        assert!(
            !has_blank("\\id GEN\n\\c 1\n\\v 1 text\n\\b\nhere asdf\n"),
            "content on the line after \\b should not be flagged"
        );
        // Bare trailing space then newline → clean (formatter normalises).
        assert!(
            !has_blank("\\id GEN\n\\c 1\n\\v 1 text\n\\b \nhere asdf\n"),
            "a trailing space before \\b's newline should not be flagged"
        );

        // The reroute means the old, misleading whitespace code must no
        // longer fire for `\b` content — the report names the real issue.
        let case1 = lint_usfm(
            "\\id GEN\n\\c 1\n\\v 1 text\n\\b here asdf\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            !case1
                .issues
                .iter()
                .any(|i| i.code == LintCode::MissingHorizontalWhitespaceAfterMarkerName),
            "\\b content should report content-after-blank-marker, not missing-horizontal-whitespace"
        );
    }

    #[test]
    fn verse_in_section_heading_is_flagged() {
        // `\s1` is a Section-category paragraph. Putting `\v` inside
        // one violates USFM 3.2 §verse-placement.
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\s1 Section\n\\v 1 Text\n",
            LintOptions::scoped(LintScope::Book),
        );
        let flagged = result
            .issues
            .iter()
            .any(|i| i.code == LintCode::VerseInSectionOrOtherParagraph);
        assert!(
            flagged,
            "expected verse-in-section issue; got: {:?}",
            result.issues.iter().map(|i| i.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn verse_in_body_paragraph_is_not_flagged_as_section_violation() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            !result
                .issues
                .iter()
                .any(|i| i.code == LintCode::VerseInSectionOrOtherParagraph),
            "body paragraph must not trigger section-or-other rule"
        );
    }

    #[test]
    fn missing_content_space_after_close_marker_is_flagged() {
        // Closing \nd* immediately followed by alphabetic text. The
        // intentional contraction case is still flagged — autofix is
        // None so the linter is reporting a hint, not changing content.
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 \\nd Lord\\nd*walks.\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.code == LintCode::MissingContentSpaceAfterCloseMarker),
            "expected missing-content-space-after-close-marker"
        );
    }

    #[test]
    fn unclosed_marker_emits_collapsed_code_with_kind_param() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p \\f + \\ft note\n\\p text",
            LintOptions::scoped(LintScope::Book),
        );
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == LintCode::UnclosedMarker)
            .expect("collapsed UnclosedMarker issue");
        assert_eq!(issue.template, LintCode::UnclosedMarker.template());
        assert_eq!(issue.message_params.get("kind"), Some(&"note".to_string()));
        assert!(
            issue.message.contains("Note"),
            "rendered message should select note arm: {}",
            issue.message
        );
    }

    #[derive(Debug)]
    struct EditorToken {
        token_kind: TokenKind,
        token_span: Span,
        token_text: String,
        token_marker: Option<String>,
        token_sid: Option<String>,
        token_id: String,
        lane: u8,
    }

    impl WalkableToken for EditorToken {
        fn kind(&self) -> TokenKind {
            self.token_kind
        }

        fn marker(&self) -> Option<&str> {
            self.token_marker.as_deref()
        }

        fn text(&self) -> &str {
            &self.token_text
        }

        fn structural(&self) -> Option<StructuralMarkerInfo> {
            None
        }
    }

    impl LintableToken for EditorToken {
        fn span(&self) -> Option<Span> {
            Some(self.token_span)
        }

        fn sid(&self) -> Option<String> {
            self.token_sid.clone()
        }

        fn id(&self) -> Option<String> {
            Some(self.token_id.clone())
        }
    }

    #[test]
    fn lint_usfm_matches_lint_tokens() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 text";
        let from_source = lint_usfm(source, LintOptions::scoped(LintScope::Book));
        let parsed = parse(source);
        let from_tokens = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        assert_eq!(from_source, from_tokens);
    }

    #[test]
    fn lint_accepts_editor_tokens_without_conversion() {
        let tokens = vec![
            EditorToken {
                token_kind: TokenKind::Marker,
                token_span: Span::new(0, 2),
                token_text: "\\m".to_string(),
                token_marker: Some("m".to_string()),
                token_sid: Some("REV 19:14".to_string()),
                token_id: "REV-0".to_string(),
                lane: 1,
            },
            EditorToken {
                token_kind: TokenKind::Text,
                token_span: Span::new(2, 8),
                token_text: "(text)".to_string(),
                token_marker: None,
                token_sid: Some("REV 19:14".to_string()),
                token_id: "REV-1".to_string(),
                lane: 1,
            },
        ];

        let issues = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        assert!(
            issues
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MissingTagEndDelimiterAfterMarker)
        );
        assert_eq!(tokens[0].lane, 1);
    }

    #[test]
    fn missing_tag_end_delimiter_carries_concrete_fix_and_apply_token_fix_updates_tokens() {
        let tokens = vec![
            crate::FormatToken {
                kind: TokenKind::Marker,
                text: "\\p".to_string(),
                marker: Some("p".to_string()),
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-0".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
            crate::FormatToken {
                kind: TokenKind::Text,
                text: "Alpha".to_string(),
                marker: None,
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-1".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
        ];

        let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
        let issue = result
            .issues
            .into_iter()
            .find(|issue| issue.code == LintCode::MissingTagEndDelimiterAfterMarker)
            .expect("expected missing-tag-end-delimiter issue");
        let fix = issue.fix.expect("expected concrete token fix");
        let fixed = apply_token_fix(&tokens, &fix);

        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed[0].text, "\\p ");
        assert_eq!(fixed[1].text, "Alpha");
    }

    #[test]
    fn missing_id_is_reported() {
        let result = lint_usfm("\\c 1\n\\v 1 text", LintOptions::scoped(LintScope::Book));
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MissingIdMarker)
        );
    }

    #[test]
    fn duplicate_id_is_reported() {
        let result = lint_usfm(
            "\\id GEN\n\\id EXO\n\\c 1\n\\v 1 text",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::DuplicateIdMarker)
        );
    }

    #[test]
    fn unknown_markers_do_not_also_report_context_errors() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\zzz bogus\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::UnknownMarker)
        );
        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MarkerNotValidInContext)
        );
    }

    #[test]
    fn marker_not_valid_in_context_can_run_as_a_standalone_rule() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\fr orphan\n",
            LintOptions {
                enabled_codes: Some(vec![LintCode::MarkerNotValidInContext]),
                ..LintOptions::scoped(LintScope::Book)
            },
        );
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].code, LintCode::MarkerNotValidInContext);
    }

    #[test]
    fn missing_chapter_and_verse_numbers_are_reported() {
        let result = lint_usfm(
            "\\id GEN\n\\c\n\\v text",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MissingChapterNumber)
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MissingVerseNumber)
        );
    }

    #[test]
    fn note_submarker_outside_note_is_reported() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\ft outside note\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::NoteSubmarkerOutsideNote)
        );
    }

    #[test]
    fn chapter_and_verse_metadata_attachment_is_checked() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\vp 2\n\\ca 3",
            LintOptions::scoped(LintScope::Book),
        );
        let verse_issue = result
            .issues
            .iter()
            .find(|issue| {
                issue.code == LintCode::MetadataOutsideTarget
                    && issue.message_params.get("target").map(String::as_str) == Some("verse")
            })
            .expect("verse-metadata issue");
        assert_eq!(
            verse_issue.message_params.get("marker"),
            Some(&"vp".to_string())
        );
        let chapter_issue = result
            .issues
            .iter()
            .find(|issue| {
                issue.code == LintCode::MetadataOutsideTarget
                    && issue.message_params.get("target").map(String::as_str) == Some("chapter")
            })
            .expect("chapter-metadata issue");
        assert_eq!(
            chapter_issue.message_params.get("marker"),
            Some(&"ca".to_string())
        );
    }

    #[test]
    fn numbering_rules_are_reported() {
        // Uniqueness rules fire: a verse and a chapter that each appear twice.
        // Monotonicity (increment) rules left the library — see
        // the `dropped_consistency_rules_no_longer_fire` test.
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n\\c 1\n",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::DuplicateVerseNumber)
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::DuplicateChapterNumber)
        );
    }

    #[test]
    fn dropped_consistency_rules_no_longer_fire() {
        // Chapter gap (1 -> 5), verse gap (1 -> 3), and inconsistent chapter
        // labels are all valid USFM — content-consistency heuristics, not
        // markup validity. They moved to the consumer;
        // the library must stay silent on them.
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\v 1 a\n\\v 3 b\n\\c 5\n\\cl Chapter 5\n\\c 6\n\\cl Chapitre 6\n",
            LintOptions::scoped(LintScope::Book),
        );
        let dropped = [
            "chapter-expected-increase-by-one",
            "verse-expected-increase-by-one",
            "inconsistent-chapter-label",
        ];
        for issue in &result.issues {
            assert!(
                !dropped.contains(&issue.code.code()),
                "dropped rule still fired: {}",
                issue.code.code()
            );
        }
    }

    #[test]
    fn document_rules_run_only_for_front_and_book_scopes() {
        // Every document-category code must fire under Front and stay silent
        // under Chapter — otherwise a mid-book chapter relint would falsely
        // report e.g. a missing id. One fixture per code (they can't all
        // coexist: "no id" excludes "duplicate id").
        let cases: [(&str, LintCode); 4] = [
            ("\\p\n\\v 1 text\n", LintCode::MissingIdMarker), // no \id at all
            ("\\id GEN\n\\id MAT\n", LintCode::DuplicateIdMarker), // two \id
            ("\\p before\n\\id GEN\n", LintCode::IdMarkerNotAtFileStart), // \id after content
            ("\\p\n\\v 1 text\n", LintCode::ContentBeforeFirstChapter), // content before any \c
        ];
        for (src, code) in cases {
            let front = lint_usfm(src, LintOptions::scoped(LintScope::Front));
            assert!(
                front.issues.iter().any(|i| i.code == code),
                "{} should fire under Front for {src:?}",
                code.code()
            );
            let chapter = lint_usfm(src, LintOptions::scoped(LintScope::Chapter(1)));
            assert!(
                !chapter.issues.iter().any(|i| i.code == code),
                "{} must be suppressed under Chapter for {src:?}",
                code.code()
            );
        }

        // Category-wide guarantee: no document-category finding survives a
        // Chapter-scoped lint, even on a slice that opens with its own \c.
        let chapter = lint_usfm(
            "\\c 5\n\\p\n\\v 1 text\n",
            LintOptions::scoped(LintScope::Chapter(5)),
        );
        assert!(
            !chapter
                .issues
                .iter()
                .any(|i| i.category == LintCategory::Document),
            "Chapter scope must not emit document-category findings, got: {:?}",
            chapter
                .issues
                .iter()
                .filter(|i| i.category == LintCategory::Document)
                .map(|i| i.code.code())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn chapter_slice_and_book_agree_on_unclosed_note_finding() {
        // \c 1 carries an unclosed footnote. A new \c closes everything below
        // it (walker precedence), so the unclosed-marker recovery fires
        // whether the chapter is linted alone (recovers at slice EOF) or
        // inside a book (recovers at the \c 2 boundary). The `location` param
        // may differ; the *finding* must agree.
        let book = "\\id MRK\n\\c 1\n\\v 1 a \\f + \\ft note\n\\c 2\n\\v 1 b\n";
        let chapter = "\\c 1\n\\v 1 a \\f + \\ft note\n";

        let book_flags_it = lint_usfm(book, LintOptions::scoped(LintScope::Book))
            .issues
            .iter()
            .any(|i| i.code == LintCode::UnclosedMarker && i.sid.as_deref() == Some("MRK 1:1"));
        let chapter_flags_it = lint_usfm(chapter, LintOptions::scoped(LintScope::Chapter(1)))
            .issues
            .iter()
            .any(|i| i.code == LintCode::UnclosedMarker);

        assert!(
            book_flags_it,
            "book scope should flag the unclosed note in ch1"
        );
        assert!(
            chapter_flags_it,
            "chapter scope should flag the same unclosed note"
        );
    }

    #[test]
    fn numbering_message_params_are_exposed_for_localized_rendering() {
        let invalid_range_tokens = vec![
            crate::FormatToken {
                kind: TokenKind::Marker,
                text: "\\v".to_string(),
                marker: Some("v".to_string()),
                sid: Some("GEN 1:4".to_string()),
                id: Some("GEN-v".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
            crate::FormatToken {
                kind: TokenKind::Number,
                text: "4-2".to_string(),
                marker: None,
                sid: Some("GEN 1:4".to_string()),
                id: Some("GEN-n".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
        ];
        let invalid_range_result =
            lint_tokens(&invalid_range_tokens, LintOptions::scoped(LintScope::Book));
        let range_issue = invalid_range_result
            .issues
            .iter()
            .find(|issue| issue.code == LintCode::InvalidNumberRange)
            .expect("expected invalid-range issue");
        assert_eq!(
            range_issue.message_params.get("found"),
            Some(&"4-2".to_string())
        );
        assert_eq!(
            range_issue.message_params.get("verse"),
            Some(&"4-2".to_string())
        );
        assert_eq!(
            range_issue.message_params.get("marker"),
            Some(&"v".to_string())
        );
        assert_eq!(
            range_issue.message_params.get("context"),
            Some(&"verse-range".to_string())
        );
    }

    #[test]
    fn structural_balance_rules_are_reported() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p \\f + \\ft note\n\\p text",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::UnclosedMarker)
        );
    }

    #[test]
    fn note_structural_submarkers_do_not_report_implicit_or_misnested_close_on_note_end() {
        let result = lint_usfm(
            "\\id GEN\n\\c 1\n\\p \\f + \\ft note\\f*",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::ImplicitlyClosedMarker)
        );
        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::MisnestedCloseMarker)
        );
    }

    #[test]
    fn rule_filtering_and_suppressions_work() {
        let mut options = LintOptions {
            enabled_codes: Some(vec![LintCode::DuplicateVerseNumber]),
            ..LintOptions::scoped(LintScope::Book)
        };
        let result = lint_usfm("\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text", options.clone());
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].code, LintCode::DuplicateVerseNumber);

        options.enabled_codes = None;
        options.suppressed = vec![LintSuppression {
            code: LintCode::DuplicateVerseNumber,
            sid: "GEN 1:1".to_string(),
        }];
        let suppressed = lint_usfm("\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text", options);
        assert!(
            !suppressed
                .issues
                .iter()
                .any(|issue| issue.code == LintCode::DuplicateVerseNumber)
        );
        assert_eq!(suppressed.summary.suppressed_count, 1);
    }

    #[test]
    fn summary_counts_by_category_and_severity() {
        let result = lint_usfm(
            "\\c 2\n\\v 1 text\n\\v 1 text",
            LintOptions::scoped(LintScope::Book),
        );
        assert!(result.summary.total_count > 0);
        assert!(
            result
                .summary
                .by_category
                .contains_key(&LintCategory::Document)
        );
        assert!(
            result
                .summary
                .by_severity
                .contains_key(&LintSeverity::Error)
        );
    }

    /// The chapter-parallel book collector must be byte-identical to the serial
    /// collector for every corpus fixture, at any thread count — this is the
    /// decomposition proof independent of the `PARALLEL_MIN_TOKENS` routing (it
    /// forces the partitioned path even on inputs `lint_tokens` would run
    /// serially). The lint oracle pins the *whole* pipeline over `testData`; this
    /// isolates the collector split over `testData` + `example-corpora`.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn partitioned_matches_serial_over_corpora() {
        use crate::parse::parse;
        use std::path::{Path, PathBuf};

        fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_usfm(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("usfm") {
                    out.push(path);
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut paths = Vec::new();
        collect_usfm(&root.join("testData"), &mut paths);
        collect_usfm(&root.join("example-corpora"), &mut paths);
        paths.sort();
        assert!(!paths.is_empty(), "expected corpus fixtures");

        let options = LintOptions::scoped(LintScope::Book);
        let enabled = EnabledCodes::new(&options);
        for path in paths {
            let source = std::fs::read_to_string(&path).expect("read fixture");
            let parsed = parse(&source);
            let serial = finalize_issues(
                collect_issues_serial(&parsed.tokens, &options, &enabled),
                &options,
            );
            let partitioned = finalize_issues(
                collect_issues_partitioned(&parsed.tokens, &options, &enabled),
                &options,
            );
            assert_eq!(
                partitioned,
                serial,
                "partitioned lint differs from serial for {}",
                path.display()
            );
        }
    }
}
