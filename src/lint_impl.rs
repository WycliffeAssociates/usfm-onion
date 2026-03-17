use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::marker_defs::{
    InlineContext, SpecContext, StructuralMarkerInfo, StructuralScopeKind,
    marker_allows_effective_context, marker_inline_context, marker_note_context,
    marker_note_subkind,
};
use crate::markers::{MarkerKind, lookup_marker};
use crate::parse::parse;
use crate::format::FormatToken;
use crate::token::{NumberRangeKind, Sid, Span, Token, TokenData, TokenId, TokenKind};

pub trait LintableToken {
    fn kind(&self) -> TokenKind;
    fn span(&self) -> Option<Span> {
        None
    }
    fn text(&self) -> &str;
    fn marker(&self) -> Option<&str>;
    fn sid(&self) -> Option<String> {
        None
    }
    fn id(&self) -> Option<String> {
        None
    }
    fn structural(&self) -> Option<StructuralMarkerInfo> {
        None
    }
    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        None
    }
}

impl<'a> LintableToken for Token<'a> {
    fn kind(&self) -> TokenKind {
        self.kind()
    }

    fn span(&self) -> Option<Span> {
        Some(self.span)
    }

    fn text(&self) -> &str {
        self.source
    }

    fn marker(&self) -> Option<&str> {
        self.marker_name()
    }

    fn sid(&self) -> Option<String> {
        self.sid.map(format_sid)
    }

    fn id(&self) -> Option<String> {
        Some(format_token_id(self.id))
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        match self.data {
            TokenData::Marker { structural, .. }
            | TokenData::EndMarker { structural, .. }
            | TokenData::Milestone { structural, .. } => Some(structural),
            _ => None,
        }
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        match self.data {
            TokenData::Number { start, end, kind } => Some((start, end, kind)),
            _ => None,
        }
    }
}

impl LintableToken for FormatToken {
    fn kind(&self) -> TokenKind {
        self.kind
    }

    fn span(&self) -> Option<Span> {
        self.span
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn sid(&self) -> Option<String> {
        self.sid.clone()
    }

    fn id(&self) -> Option<String> {
        self.id.clone()
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural
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
pub enum LintCode {
    MissingIdMarker,
    MissingSeparatorAfterMarker,
    EmptyParagraph,
    NumberRangeAfterChapterMarker,
    VerseRangeExpectedAfterVerseMarker,
    VerseContentNotEmpty,
    UnknownToken,
    CharNotClosed,
    NoteNotClosed,
    ParagraphBeforeFirstChapter,
    VerseBeforeFirstChapter,
    NoteSubmarkerOutsideNote,
    DuplicateIdMarker,
    IdMarkerNotAtFileStart,
    ChapterMetadataOutsideChapter,
    VerseMetadataOutsideVerse,
    MissingChapterNumber,
    MissingVerseNumber,
    MissingMilestoneSelfClose,
    ImplicitlyClosedMarker,
    StrayCloseMarker,
    MisnestedCloseMarker,
    UnclosedNote,
    UnclosedMarkerAtEof,
    DuplicateChapterNumber,
    ChapterExpectedIncreaseByOne,
    DuplicateVerseNumber,
    VerseExpectedIncreaseByOne,
    InvalidNumberRange,
    NumberRangeNotPrecededByMarkerExpectingNumber,
    VerseTextFollowsVerseRange,
    UnknownMarker,
    UnknownCloseMarker,
    InconsistentChapterLabel,
    MarkerNotValidInContext,
    VerseOutsideExplicitParagraph,
}

impl LintCode {
    pub fn category(self) -> LintCategory {
        match self {
            Self::MissingIdMarker
            | Self::ParagraphBeforeFirstChapter
            | Self::VerseBeforeFirstChapter
            | Self::DuplicateIdMarker
            | Self::IdMarkerNotAtFileStart => LintCategory::Document,
            Self::MissingSeparatorAfterMarker
            | Self::EmptyParagraph
            | Self::NumberRangeAfterChapterMarker
            | Self::VerseRangeExpectedAfterVerseMarker
            | Self::VerseContentNotEmpty
            | Self::UnknownToken
            | Self::CharNotClosed
            | Self::NoteNotClosed
            | Self::MissingChapterNumber
            | Self::MissingVerseNumber
            | Self::MissingMilestoneSelfClose
            | Self::ImplicitlyClosedMarker
            | Self::StrayCloseMarker
            | Self::MisnestedCloseMarker
            | Self::UnclosedNote
            | Self::UnclosedMarkerAtEof
            | Self::UnknownMarker
            | Self::UnknownCloseMarker => LintCategory::Structure,
            Self::NoteSubmarkerOutsideNote
            | Self::ChapterMetadataOutsideChapter
            | Self::VerseMetadataOutsideVerse
            | Self::MarkerNotValidInContext
            | Self::VerseOutsideExplicitParagraph => LintCategory::Context,
            Self::DuplicateChapterNumber
            | Self::ChapterExpectedIncreaseByOne
            | Self::DuplicateVerseNumber
            | Self::VerseExpectedIncreaseByOne
            | Self::InvalidNumberRange
            | Self::NumberRangeNotPrecededByMarkerExpectingNumber
            | Self::VerseTextFollowsVerseRange
            | Self::InconsistentChapterLabel => LintCategory::Numbering,
        }
    }

    pub fn severity(self) -> LintSeverity {
        match self {
            Self::EmptyParagraph => LintSeverity::Warning,
            _ => LintSeverity::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintIssue {
    pub code: LintCode,
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub message: String,
    pub span: Option<Span>,
    pub related_span: Option<Span>,
    pub token_id: Option<String>,
    pub related_token_id: Option<String>,
    pub sid: Option<String>,
    pub marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LintSummary {
    pub by_category: BTreeMap<LintCategory, usize>,
    pub by_severity: BTreeMap<LintSeverity, usize>,
    pub total_count: usize,
    pub suppressed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
    pub summary: LintSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintSuppression {
    pub code: LintCode,
    pub sid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LintOptions {
    pub enabled_codes: Option<Vec<LintCode>>,
    pub disabled_codes: Vec<LintCode>,
    pub suppressed: Vec<LintSuppression>,
    pub allow_implicit_chapter_content_verse: bool,
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
struct OpenMarkerFrame {
    marker: String,
    token_index: usize,
    kind: MarkerKind,
    valid_in_note: bool,
}

#[derive(Debug, Clone)]
struct OpenLintFrame {
    marker: String,
    kind: StructuralScopeKind,
    token_index: usize,
}

#[derive(Debug, Clone)]
struct EnabledCodes {
    allowed: Option<BTreeSet<LintCode>>,
    disabled: BTreeSet<LintCode>,
}

#[derive(Default)]
struct VerseState {
    seen: HashSet<u32>,
    last: u32,
}

type ChapterLabelEntry = (Option<Span>, Option<String>, Option<String>);

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

    fn current_validation_context_for<T: LintableToken>(&self, token: &T) -> SpecContext {
        let root_context = self.current_root_context();
        let effective = self
            .current_note_context()
            .or(self.block_context)
            .unwrap_or(root_context);

        match token_marker_kind(token) {
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

    fn select_top_level_slot(&self, marker_name: &str) -> TopLevelSlot {
        if marker_name == "periph" {
            return TopLevelSlot::AwaitDivision;
        }

        let contexts = top_level_contexts_for(self.kind);
        let start = top_level_slot_index(self.slot);

        contexts
            .iter()
            .copied()
            .skip(start)
            .find(|(_, context)| marker_allows_effective_context(marker_name, *context))
            .map(|(slot, _)| slot)
            .unwrap_or(self.slot)
    }

    fn apply_marker<T: LintableToken>(
        &mut self,
        tokens: &[T],
        index: usize,
        token: &T,
    ) {
        let Some(name) = token.marker() else {
            return;
        };

        match token_marker_kind(token) {
            MarkerKind::Header => {
                if name == "id" && let Some(book_code) = next_book_code_after_marker(tokens, index) {
                    self.kind = infer_document_kind(book_code);
                    if self.kind == DocumentKind::PeripheralDivided {
                        self.slot = TopLevelSlot::AwaitDivision;
                    } else {
                        self.slot = TopLevelSlot::Headers;
                    }
                    self.block_context = None;
                    self.note_stack.clear();
                } else {
                    self.slot = self.select_top_level_slot(name);
                }
            }
            MarkerKind::Chapter => {
                self.saw_chapter = true;
                self.slot = TopLevelSlot::Content;
                self.block_context = None;
            }
            MarkerKind::Paragraph => {
                if self.current_note_context().is_none() {
                    self.slot = self.select_top_level_slot(name);
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
            allowed: options
                .enabled_codes
                .as_ref()
                .map(|codes| codes.iter().copied().collect()),
            disabled: options.disabled_codes.iter().copied().collect(),
        }
    }

    fn has(&self, code: LintCode) -> bool {
        if self.disabled.contains(&code) {
            return false;
        }
        self.allowed
            .as_ref()
            .is_none_or(|allowed| allowed.contains(&code))
    }

    fn has_any(&self, codes: &[LintCode]) -> bool {
        codes.iter().copied().any(|code| self.has(code))
    }
}

pub fn lint_usfm(source: &str, options: LintOptions) -> LintResult {
    let parsed = parse(source);
    lint_tokens(&parsed.tokens, options)
}

pub fn lint_tokens<T: LintableToken>(tokens: &[T], options: LintOptions) -> LintResult {
    let enabled = EnabledCodes::new(&options);
    let mut issues = Vec::new();

    if enabled.has(LintCode::MissingSeparatorAfterMarker) {
        lint_missing_separator_after_marker(tokens, &mut issues);
    }
    if enabled.has(LintCode::EmptyParagraph) {
        lint_empty_paragraphs(tokens, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::UnknownToken,
        LintCode::NumberRangeAfterChapterMarker,
        LintCode::VerseRangeExpectedAfterVerseMarker,
        LintCode::VerseContentNotEmpty,
        LintCode::MissingChapterNumber,
        LintCode::MissingVerseNumber,
    ]) {
        lint_expectation_and_unknown_token_rules(tokens, &enabled, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::MissingIdMarker,
        LintCode::ParagraphBeforeFirstChapter,
        LintCode::VerseBeforeFirstChapter,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::DuplicateIdMarker,
        LintCode::IdMarkerNotAtFileStart,
        LintCode::ChapterMetadataOutsideChapter,
        LintCode::VerseMetadataOutsideVerse,
        LintCode::VerseOutsideExplicitParagraph,
    ]) {
        lint_structure_rules(
            tokens,
            &options,
            &enabled,
            &mut issues,
        );
    }
    if enabled.has(LintCode::UnknownMarker) {
        lint_unknown_markers(tokens, &mut issues);
    }
    if enabled.has(LintCode::UnknownCloseMarker) {
        lint_unknown_close_markers(tokens, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::DuplicateChapterNumber,
        LintCode::ChapterExpectedIncreaseByOne,
        LintCode::InconsistentChapterLabel,
    ]) {
        lint_chapter_rules(tokens, &enabled, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        LintCode::InvalidNumberRange,
        LintCode::DuplicateVerseNumber,
        LintCode::VerseExpectedIncreaseByOne,
        LintCode::VerseTextFollowsVerseRange,
    ]) {
        lint_number_and_verse_rules(tokens, &enabled, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::CharNotClosed,
        LintCode::NoteNotClosed,
        LintCode::StrayCloseMarker,
        LintCode::MisnestedCloseMarker,
        LintCode::MissingMilestoneSelfClose,
        LintCode::UnclosedNote,
        LintCode::UnclosedMarkerAtEof,
    ]) {
        lint_marker_balance_rules(tokens, &enabled, &mut issues);
    }

    let unique = dedupe_issues(issues);
    let (issues, suppressed_count) = apply_suppressions(unique, &options.suppressed);
    let summary = summarize(&issues, suppressed_count);

    LintResult { issues, summary }
}

fn lint_missing_separator_after_marker<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for window in tokens.windows(2) {
        let [current, next] = window else { continue };
        if current.kind() != TokenKind::Marker || next.kind() != TokenKind::Text {
            continue;
        }
        let Some(marker) = current.marker() else {
            continue;
        };
        let marker_kind = token_marker_kind(current);
        if matches!(marker_kind, MarkerKind::MilestoneStart | MarkerKind::MilestoneEnd) {
            continue;
        }
        if matches!(marker_kind, MarkerKind::Unknown) && marker.starts_with('z') {
            continue;
        }
        if ends_with_horizontal_whitespace(current.text()) {
            continue;
        }
        if starts_with_horizontal_whitespace(next.text()) {
            continue;
        }
        issues.push(issue(
            LintCode::MissingSeparatorAfterMarker,
            format!("marker \\{marker} is immediately followed by text"),
            current,
            Some(next),
        ));
    }
}

fn lint_empty_paragraphs<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for index in 0..tokens.len() {
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
            format!("paragraph marker \\{marker} creates an empty block before the next block marker"),
            token,
            Some(&tokens[boundary_index]),
        ));
    }
}

fn lint_expectation_and_unknown_token_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    for index in 0..tokens.len() {
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
                if enabled.has(LintCode::NumberRangeAfterChapterMarker) && missing {
                    issues.push(simple_issue(
                        LintCode::NumberRangeAfterChapterMarker,
                        "number range expected after \\c".to_string(),
                        token,
                    ));
                }
                if enabled.has(LintCode::MissingChapterNumber) && missing {
                    issues.push(simple_issue(
                        LintCode::MissingChapterNumber,
                        "chapter number expected after \\c".to_string(),
                        token,
                    ));
                }
            }
            "v" => {
                let missing = next_number_token_index(tokens, index + 1).is_none();
                if enabled.has(LintCode::VerseRangeExpectedAfterVerseMarker) && missing {
                    issues.push(simple_issue(
                        LintCode::VerseRangeExpectedAfterVerseMarker,
                        "verse number expected after \\v".to_string(),
                        token,
                    ));
                }
                if enabled.has(LintCode::MissingVerseNumber) && missing {
                    issues.push(simple_issue(
                        LintCode::MissingVerseNumber,
                        "verse number expected after \\v".to_string(),
                        token,
                    ));
                }
                if enabled.has(LintCode::VerseContentNotEmpty)
                    && let Some(next_index) = next_significant_token_index(tokens, index + 1)
                    && tokens[next_index].kind() == TokenKind::Text
                    && tokens[next_index].text().trim().is_empty()
                {
                    issues.push(issue(
                        LintCode::VerseContentNotEmpty,
                        "verse content expected after \\v".to_string(),
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
        if token.kind() == TokenKind::Newline {
            continue;
        }

        if token.kind() == TokenKind::Marker {
            let marker = token.marker().unwrap_or_default();

            if enabled.has(LintCode::IdMarkerNotAtFileStart) && marker == "id" && saw_content {
                issues.push(simple_issue(
                    LintCode::IdMarkerNotAtFileStart,
                    "\\id marker appears after book content has already started".to_string(),
                    token,
                ));
            }
            if enabled.has(LintCode::DuplicateIdMarker) && marker == "id" {
                if id_seen {
                    issues.push(simple_issue(
                        LintCode::DuplicateIdMarker,
                        "duplicate \\id marker".to_string(),
                        token,
                    ));
                }
                id_seen = true;
            }

            let prospective_state = if token_marker_kind(token) == MarkerKind::Note {
                document_state.current_validation_context_for(token)
            } else if token_marker_kind(token) == MarkerKind::Paragraph
                && document_state.current_note_context().is_none()
            {
                top_level_root_context(
                    document_state.kind,
                    document_state.select_top_level_slot(marker),
                )
            } else if token_marker_kind(token) == MarkerKind::Periph {
                SpecContext::Peripheral
            } else if token_marker_kind(token) == MarkerKind::Chapter {
                top_level_root_context(document_state.kind, TopLevelSlot::Content)
            } else {
                document_state.current_root_context()
            };

            if enabled.has(LintCode::ParagraphBeforeFirstChapter)
                && !document_state.saw_chapter
                && document_state.kind == DocumentKind::Scripture
                && token_marker_kind(token) == MarkerKind::Paragraph
                && is_body_paragraph_marker(marker)
                && prospective_state == SpecContext::ChapterContent
            {
                issues.push(simple_issue(
                    LintCode::ParagraphBeforeFirstChapter,
                    format!("body paragraph marker \\{marker} appears before the first chapter"),
                    token,
                ));
            }

            if enabled.has(LintCode::VerseBeforeFirstChapter)
                && !document_state.saw_chapter
                && document_state.kind == DocumentKind::Scripture
                && marker == "v"
            {
                issues.push(simple_issue(
                    LintCode::VerseBeforeFirstChapter,
                    "verse marker appears before the first chapter".to_string(),
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
                    "verse marker appears outside an explicit paragraph, list, or table block".to_string(),
                    token,
                ));
            }

            if enabled.has(LintCode::NoteSubmarkerOutsideNote)
                && marker_note_subkind(marker).is_some()
                && note_stack.is_empty()
            {
                issues.push(simple_issue(
                    LintCode::NoteSubmarkerOutsideNote,
                    format!("note submarker \\{marker} appears outside an open note"),
                    token,
                ));
            }

            if enabled.has(LintCode::ChapterMetadataOutsideChapter)
                && matches!(marker, "ca" | "cp")
                && !matches_previous_marker_and_number(tokens, index, "c")
            {
                issues.push(simple_issue(
                    LintCode::ChapterMetadataOutsideChapter,
                    format!("chapter metadata \\{marker} is not attached to a chapter marker"),
                    token,
                ));
            }

            if enabled.has(LintCode::VerseMetadataOutsideVerse)
                && matches!(marker, "va" | "vp")
                && !matches_previous_marker_and_number(tokens, index, "v")
            {
                issues.push(simple_issue(
                    LintCode::VerseMetadataOutsideVerse,
                    format!("verse metadata \\{marker} is not attached to a verse marker"),
                    token,
                ));
            }

            let validation_context = if marker == "periph" {
                SpecContext::Peripheral
            } else if token_marker_kind(token) == MarkerKind::Chapter {
                top_level_root_context(document_state.kind, TopLevelSlot::Content)
            } else if document_state.current_note_context().is_none()
                && matches!(
                    token_marker_kind(token),
                    MarkerKind::Paragraph
                        | MarkerKind::Header
                        | MarkerKind::SidebarStart
                        | MarkerKind::TableRow
                )
            {
                let next_slot = document_state.select_top_level_slot(marker);
                top_level_root_context(document_state.kind, next_slot)
            } else {
                document_state.current_validation_context_for(token)
            };

            if enabled.has(LintCode::MarkerNotValidInContext)
                && !marker_allows_effective_context(marker, validation_context)
            {
                issues.push(simple_issue(
                    LintCode::MarkerNotValidInContext,
                    format!(
                        "marker \\{} is not valid in {}",
                        marker,
                        spec_context_name(validation_context)
                    ),
                    token,
                ));
            }

            if token_marker_kind(token) == MarkerKind::Note {
                note_stack.push(marker.to_string());
            }

            document_state.apply_marker(tokens, index, token);
            saw_content = true;
        } else if token.kind() == TokenKind::EndMarker {
            let marker = token.marker().unwrap_or_default();
            if is_note_close_marker(marker) {
                while let Some(open) = note_stack.pop() {
                    if open == marker {
                        break;
                    }
                }
            }
            saw_content = true;
        } else if token.kind() == TokenKind::Text {
            if !token.text().trim().is_empty() {
                saw_content = true;
            }
        } else if !matches!(token.kind(), TokenKind::Newline | TokenKind::OptBreak) {
            saw_content = true;
        }
    }

    if enabled.has(LintCode::MissingIdMarker) && !id_seen {
        issues.push(LintIssue {
            code: LintCode::MissingIdMarker,
            category: LintCode::MissingIdMarker.category(),
            severity: LintCode::MissingIdMarker.severity(),
            message: "document is missing required \\id marker".to_string(),
            span: None,
            related_span: None,
            token_id: None,
            related_token_id: None,
            sid: None,
            marker: Some("id".to_string()),
        });
    }
}

fn lint_unknown_markers<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for token in tokens {
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
            format!("unknown marker \\{marker}"),
            token,
        ));
    }
}

fn lint_unknown_close_markers<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for token in tokens {
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
            format!("unknown closing marker \\{marker}*"),
            token,
        ));
    }
}

fn lint_chapter_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut seen_chapters = HashSet::new();
    let mut last_chapter: Option<u32> = None;
    let mut labels: BTreeMap<String, Vec<ChapterLabelEntry>> = BTreeMap::new();

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
                    format!("duplicate chapter number {chapter}"),
                    "c",
                    &tokens[number_index],
                ));
            }
            if enabled.has(LintCode::ChapterExpectedIncreaseByOne) {
                let expected = last_chapter.map_or(1, |last| last + 1);
                if chapter != expected {
                    issues.push(simple_issue_with_marker(
                        LintCode::ChapterExpectedIncreaseByOne,
                        format!("expected chapter number {expected}, found {chapter}"),
                        "c",
                        &tokens[number_index],
                    ));
                }
            }
            seen_chapters.insert(chapter);
            last_chapter = Some(chapter);
        }

        if enabled.has(LintCode::InconsistentChapterLabel)
            && token.kind() == TokenKind::Marker
            && token.marker() == Some("cl")
            && let Some(text_index) = next_text_token_index(tokens, index + 1)
        {
            let label = strip_digits(tokens[text_index].text().trim()).trim().to_string();
            if !label.is_empty() {
                labels.entry(label).or_default().push((
                    tokens[text_index].span(),
                    tokens[text_index].id(),
                    tokens[text_index].sid(),
                ));
            }
        }

        index += 1;
    }

    if enabled.has(LintCode::InconsistentChapterLabel) && labels.len() > 1 {
        let canonical = labels
            .iter()
            .max_by_key(|(_, entries)| entries.len())
            .map(|(label, _)| label.clone());
        if let Some(canonical) = canonical {
            for (label, entries) in labels {
                if label == canonical {
                    continue;
                }
                for (span, token_id, sid) in entries {
                    issues.push(LintIssue {
                        code: LintCode::InconsistentChapterLabel,
                        category: LintCode::InconsistentChapterLabel.category(),
                        severity: LintCode::InconsistentChapterLabel.severity(),
                        message: format!(
                            "inconsistent chapter label '{label}', expected the canonical label '{canonical}'"
                        ),
                        span,
                        related_span: None,
                        token_id,
                        related_token_id: None,
                        sid,
                        marker: Some("cl".to_string()),
                    });
                }
            }
        }
    }
}

fn lint_number_and_verse_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut current_chapter = 0u32;
    let mut verse_state_by_chapter: HashMap<u32, VerseState> = HashMap::new();

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
                format!("invalid verse range {value}"),
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
                format!("duplicate verse number {value}"),
                "v",
                number_token,
            ));
        } else if enabled.has(LintCode::VerseExpectedIncreaseByOne) {
            let expected = chapter_state.last + 1;
            if start != expected {
                let message = if chapter_state.last > 0 {
                    format!(
                        "previous verse number was {}, so expected {} here, found {}",
                        chapter_state.last, expected, start
                    )
                } else {
                    format!("expected verse {expected} here, found {start}")
                };
                issues.push(simple_issue_with_marker(
                    LintCode::VerseExpectedIncreaseByOne,
                    message,
                    "v",
                    number_token,
                ));
            }
        }

        if enabled.has(LintCode::VerseTextFollowsVerseRange)
            && !verse_has_text_or_note(tokens, number_index + 1)
        {
            issues.push(simple_issue_with_marker(
                LintCode::VerseTextFollowsVerseRange,
                "expected verse content after \\v".to_string(),
                "v",
                number_token,
            ));
        }

        for verse in start..=end {
            chapter_state.seen.insert(verse);
        }
        chapter_state.last = end;
    }
}

fn lint_marker_balance_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut stack: Vec<OpenMarkerFrame> = Vec::new();
    let mut structural_stack: Vec<OpenLintFrame> = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        let kind = token.kind();
        let marker = match kind {
            TokenKind::Marker | TokenKind::EndMarker => token.marker(),
            _ => None,
        };

        if kind == TokenKind::Marker && closes_inline_stack_at_boundary(token_marker_kind(token)) {
            close_open_frames_for_boundary(tokens, token, &mut stack, enabled, issues);
        }

        if kind == TokenKind::Milestone
            && enabled.has(LintCode::MissingMilestoneSelfClose)
            && next_non_attribute_index(tokens, index + 1)
                .is_none_or(|next| tokens[next].kind() != TokenKind::MilestoneEnd)
        {
            issues.push(simple_issue(
                LintCode::MissingMilestoneSelfClose,
                format!("milestone \\{} is missing a closing \\*", marker.unwrap_or("")),
                token,
            ));
        }

        match kind {
            TokenKind::Marker => {
                let marker_kind = token_marker_kind(token);
                if matches!(marker_kind, MarkerKind::Character | MarkerKind::Note | MarkerKind::Meta) {
                    stack.push(OpenMarkerFrame {
                        marker: marker.unwrap_or_default().to_string(),
                        token_index: index,
                        kind: marker_kind,
                        valid_in_note: marker_note_subkind(marker.unwrap_or_default()).is_some(),
                    });
                }
                if let Some(structural) = token.structural()
                    && matches!(
                        structural.scope_kind,
                        StructuralScopeKind::Note | StructuralScopeKind::Character | StructuralScopeKind::Milestone
                    )
                {
                    structural_stack.push(OpenLintFrame {
                        marker: marker.unwrap_or_default().to_string(),
                        kind: structural.scope_kind,
                        token_index: index,
                    });
                }
            }
            TokenKind::EndMarker => {
                handle_close_marker(token, marker.unwrap_or_default(), &mut stack, enabled, issues);
                handle_structural_close(token, marker.unwrap_or_default(), &mut structural_stack, enabled, issues);
            }
            TokenKind::MilestoneEnd => {
                handle_milestone_end(token, &mut structural_stack, enabled, issues);
            }
            _ => {}
        }

        if matches!(
            kind,
            TokenKind::Marker | TokenKind::Milestone
        ) && matches!(
            token_marker_kind(token),
            MarkerKind::Paragraph
                | MarkerKind::Header
                | MarkerKind::Meta
                | MarkerKind::Chapter
                | MarkerKind::Periph
                | MarkerKind::Unknown
        ) {
            close_structural_frames_for_boundary(tokens, token, &mut structural_stack, enabled, issues);
        }
    }

    if let Some(anchor) = tokens.last() {
        while let Some(frame) = stack.pop() {
            issues.push(unclosed_marker_issue(tokens, &frame, anchor, true));
        }
        for frame in structural_stack {
            let code = if frame.kind == StructuralScopeKind::Note {
                LintCode::UnclosedNote
            } else {
                LintCode::UnclosedMarkerAtEof
            };
            if enabled.has(code) {
                issues.push(LintIssue {
                    code,
                    category: code.category(),
                    severity: code.severity(),
                    message: if code == LintCode::UnclosedNote {
                        format!("note \\{} was not closed before paragraph or chapter boundary", frame.marker)
                    } else {
                        format!("\\{} was still open at end of file", frame.marker)
                    },
                    span: tokens[frame.token_index].span(),
                    related_span: anchor.span(),
                    token_id: tokens[frame.token_index].id(),
                    related_token_id: anchor.id(),
                    sid: tokens[frame.token_index].sid().or_else(|| anchor.sid()),
                    marker: Some(frame.marker),
                });
            }
        }
    }
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
        format!("unknown token {}", token.text()),
        marker,
        token,
    ))
}

fn next_book_code_after_marker<T: LintableToken>(tokens: &[T], marker_index: usize) -> Option<&str> {
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

fn lint_number_predecessor<T: LintableToken>(tokens: &[T], index: usize, issues: &mut Vec<LintIssue>) {
    let token = &tokens[index];
    let Some(prev_index) = previous_significant_token_index(tokens, index) else {
        issues.push(simple_issue(
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
            "number range is not preceded by a marker that expects a number".to_string(),
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
            "number range is not preceded by a marker that expects a number".to_string(),
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

fn close_open_frames_for_boundary<T: LintableToken>(
    tokens: &[T],
    boundary: &T,
    stack: &mut Vec<OpenMarkerFrame>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    while let Some(frame) = stack.pop() {
        if frame.kind == MarkerKind::Character && frame.valid_in_note {
            continue;
        }
        let code = match frame.kind {
            MarkerKind::Note => LintCode::NoteNotClosed,
            MarkerKind::Character => LintCode::CharNotClosed,
            _ => continue,
        };
        if enabled.has(code) {
            issues.push(unclosed_marker_issue(tokens, &frame, boundary, false));
        }
    }
}

fn handle_close_marker<T: LintableToken>(
    token: &T,
    marker: &str,
    stack: &mut Vec<OpenMarkerFrame>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    if stack.is_empty() {
        if enabled.has(LintCode::StrayCloseMarker) {
            issues.push(simple_issue(
                LintCode::StrayCloseMarker,
                format!("closing marker \\{marker}* has no matching opener"),
                token,
            ));
        }
        return;
    }

    if is_note_close_marker(marker) {
        while let Some((frame_kind, _frame_marker)) = stack
            .last()
            .map(|frame| (frame.kind, frame.marker.clone()))
        {
            if frame_kind == MarkerKind::Character
                && stack.last().is_some_and(|frame| frame.valid_in_note)
            {
                stack.pop();
                continue;
            }
            break;
        }
    }

    if stack.last().is_some_and(|frame| frame.marker == marker) {
        stack.pop();
        return;
    }

    if stack.iter().any(|frame| frame.marker == marker) {
        if enabled.has(LintCode::MisnestedCloseMarker) {
            issues.push(simple_issue(
                LintCode::MisnestedCloseMarker,
                format!("closing marker \\{marker}* mismatches the current open stack"),
                token,
            ));
        }
        while let Some(frame) = stack.pop() {
            if frame.marker == marker {
                break;
            }
            if enabled.has(LintCode::ImplicitlyClosedMarker) {
                issues.push(simple_issue_with_marker(
                    LintCode::ImplicitlyClosedMarker,
                    format!("marker \\{} was implicitly closed before \\{}*", frame.marker, marker),
                    &frame.marker,
                    token,
                ));
            }
        }
    } else if enabled.has(LintCode::StrayCloseMarker) {
        issues.push(simple_issue(
            LintCode::StrayCloseMarker,
            format!("closing marker \\{marker}* has no matching opener"),
            token,
        ));
    }
}

fn handle_structural_close<T: LintableToken>(
    token: &T,
    marker: &str,
    stack: &mut Vec<OpenLintFrame>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    if is_note_close_marker(marker) {
        while let Some((frame_kind, frame_marker)) = stack
            .last()
            .map(|frame| (frame.kind, frame.marker.clone()))
        {
            if frame_kind == StructuralScopeKind::Character
                && marker_note_subkind(frame_marker.as_str()).is_some()
            {
                stack.pop();
                continue;
            }
            break;
        }
    }

    if let Some(match_pos) = stack.iter().rposition(|frame| {
        matches!(frame.kind, StructuralScopeKind::Note | StructuralScopeKind::Character)
            && frame.marker == marker
    }) {
        if match_pos + 1 != stack.len() && enabled.has(LintCode::MisnestedCloseMarker) {
            if let Some(open) = stack.last() {
                issues.push(issue(
                    LintCode::MisnestedCloseMarker,
                    format!("expected \\{}* but found \\{}*", open.marker, marker),
                    token,
                    None::<&T>,
                ));
            }
        }
        stack.truncate(match_pos);
    } else if enabled.has(LintCode::StrayCloseMarker) {
        issues.push(simple_issue(
            LintCode::StrayCloseMarker,
            format!("closing marker \\{marker}* has no matching opener"),
            token,
        ));
    }
}

fn handle_milestone_end<T: LintableToken>(
    token: &T,
    stack: &mut Vec<OpenLintFrame>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    if let Some(match_pos) = stack
        .iter()
        .rposition(|frame| frame.kind == StructuralScopeKind::Milestone)
    {
        stack.truncate(match_pos);
    } else if enabled.has(LintCode::StrayCloseMarker) {
        issues.push(simple_issue(
            LintCode::StrayCloseMarker,
            "closing marker \\* has no matching opener".to_string(),
            token,
        ));
    }
}

fn close_structural_frames_for_boundary<T: LintableToken>(
    tokens: &[T],
    boundary: &T,
    stack: &mut Vec<OpenLintFrame>,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    while let Some(frame) = stack.pop() {
        let code = match frame.kind {
            StructuralScopeKind::Note => LintCode::UnclosedNote,
            StructuralScopeKind::Character => LintCode::UnclosedMarkerAtEof,
            _ => continue,
        };
        if enabled.has(code) {
            let open = &tokens[frame.token_index];
            issues.push(LintIssue {
                code,
                category: code.category(),
                severity: code.severity(),
                message: if code == LintCode::UnclosedNote {
                    format!("note \\{} was not closed before paragraph or chapter boundary", frame.marker)
                } else {
                    format!("marker \\{} was not closed before the next block boundary", frame.marker)
                },
                span: open.span(),
                related_span: boundary.span(),
                token_id: open.id(),
                related_token_id: boundary.id(),
                sid: open.sid().or_else(|| boundary.sid()),
                marker: Some(frame.marker),
            });
        }
    }
}

fn unclosed_marker_issue<T: LintableToken>(
    tokens: &[T],
    frame: &OpenMarkerFrame,
    anchor: &T,
    at_eof: bool,
) -> LintIssue {
    let code = match frame.kind {
        MarkerKind::Note => LintCode::NoteNotClosed,
        MarkerKind::Character => LintCode::CharNotClosed,
        _ => LintCode::UnclosedMarkerAtEof,
    };
    let location = if at_eof { "before end of file" } else { "before the next block boundary" };
    let open = &tokens[frame.token_index];
    LintIssue {
        code,
        category: code.category(),
        severity: code.severity(),
        message: format!("marker \\{} was not closed {}", frame.marker, location),
        span: open.span(),
        related_span: anchor.span(),
        token_id: open.id(),
        related_token_id: anchor.id(),
        sid: open.sid().or_else(|| anchor.sid()),
        marker: Some(frame.marker.clone()),
    }
}

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
            TokenKind::AttributeList => continue,
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
    token.number_info().and_then(|(start, end, kind)| match kind {
        NumberRangeKind::Single => Some((start, start)),
        NumberRangeKind::Range => end.map(|end| (start, end)),
        NumberRangeKind::Sequence | NumberRangeKind::SequenceWithRange => Some((start, end.unwrap_or(start))),
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
    let start = parts.next()?.split(',').next()?.trim_matches(|ch: char| !ch.is_ascii_digit()).parse::<u32>().ok()?;
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
            (TopLevelSlot::IntroductionEndTitles, SpecContext::BookIntroductionEndTitles),
            (TopLevelSlot::Content, SpecContext::ChapterContent),
        ],
        DocumentKind::PeripheralStandalone | DocumentKind::PeripheralDivided => &[
            (TopLevelSlot::Headers, SpecContext::BookHeaders),
            (TopLevelSlot::Titles, SpecContext::BookTitles),
            (TopLevelSlot::Introduction, SpecContext::BookIntroduction),
            (TopLevelSlot::IntroductionEndTitles, SpecContext::BookIntroductionEndTitles),
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

fn token_marker_kind<T: LintableToken>(token: &T) -> MarkerKind {
    if let Some(structural) = token.structural() {
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
    token.marker().map(|name| lookup_marker(name).kind).unwrap_or(MarkerKind::Unknown)
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

fn next_text_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::Newline => continue,
            TokenKind::Text => return Some(index),
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

fn next_non_attribute_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind() != TokenKind::AttributeList {
            return Some(index);
        }
    }
    None
}

fn starts_with_horizontal_whitespace(text: &str) -> bool {
    matches!(text.chars().next(), Some(' ' | '\t'))
}

fn ends_with_horizontal_whitespace(text: &str) -> bool {
    matches!(text.chars().next_back(), Some(' ' | '\t'))
}

fn strip_digits(text: &str) -> &str {
    let first_digit = text.find(|ch: char| ch.is_ascii_digit()).unwrap_or(text.len());
    &text[..first_digit]
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

fn empty_paragraph_boundary_index<T: LintableToken>(tokens: &[T], marker_index: usize) -> Option<usize> {
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

fn closes_inline_stack_at_boundary(kind: MarkerKind) -> bool {
    matches!(
        kind,
        MarkerKind::Paragraph
            | MarkerKind::Header
            | MarkerKind::Meta
            | MarkerKind::Chapter
            | MarkerKind::Periph
            | MarkerKind::Unknown
    )
}

fn is_note_close_marker(marker: &str) -> bool {
    matches!(marker, "f" | "fe" | "ef" | "x" | "ex")
}

fn spec_context_name(context: SpecContext) -> &'static str {
    match context {
        SpecContext::Scripture => "Scripture",
        SpecContext::BookIdentification => "BookIdentification",
        SpecContext::BookHeaders => "BookHeaders",
        SpecContext::BookTitles => "BookTitles",
        SpecContext::BookIntroduction => "BookIntroduction",
        SpecContext::BookIntroductionEndTitles => "BookIntroductionEndTitles",
        SpecContext::BookChapterLabel => "BookChapterLabel",
        SpecContext::ChapterContent => "ChapterContent",
        SpecContext::Peripheral => "Peripheral",
        SpecContext::PeripheralContent => "PeripheralContent",
        SpecContext::PeripheralDivision => "PeripheralDivision",
        SpecContext::Chapter => "Chapter",
        SpecContext::Verse => "Verse",
        SpecContext::Section => "Section",
        SpecContext::Para => "Para",
        SpecContext::List => "List",
        SpecContext::Table => "Table",
        SpecContext::Sidebar => "Sidebar",
        SpecContext::Footnote => "Footnote",
        SpecContext::CrossReference => "CrossReference",
    }
}

fn issue<T: LintableToken, U: LintableToken>(
    code: LintCode,
    message: String,
    token: &T,
    related: Option<&U>,
) -> LintIssue {
    LintIssue {
        code,
        category: code.category(),
        severity: code.severity(),
        message,
        span: token.span(),
        related_span: related.and_then(LintableToken::span),
        token_id: token.id(),
        related_token_id: related.and_then(LintableToken::id),
        sid: token.sid(),
        marker: token.marker().map(ToOwned::to_owned),
    }
}

fn simple_issue<T: LintableToken>(code: LintCode, message: String, token: &T) -> LintIssue {
    issue(code, message, token, None::<&T>)
}

fn simple_issue_with_marker<T: LintableToken>(
    code: LintCode,
    message: String,
    marker: &str,
    token: &T,
) -> LintIssue {
    let mut issue = simple_issue(code, message, token);
    issue.marker = Some(marker.to_string());
    issue
}

fn dedupe_issues(issues: Vec<LintIssue>) -> Vec<LintIssue> {
    let mut seen = HashSet::new();
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

fn apply_suppressions(issues: Vec<LintIssue>, suppressions: &[LintSuppression]) -> (Vec<LintIssue>, usize) {
    let suppression_keys = suppressions
        .iter()
        .map(|suppression| (suppression.code, suppression.sid.as_str()))
        .collect::<HashSet<_>>();
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

    for issue in issues {
        *by_category.entry(issue.category).or_insert(0) += 1;
        *by_severity.entry(issue.severity).or_insert(0) += 1;
    }

    LintSummary {
        by_category,
        by_severity,
        total_count: issues.len(),
        suppressed_count,
    }
}

fn format_sid(sid: Sid<'_>) -> String {
    if sid.verse == 0 {
        format!("{} {}", sid.book_code, sid.chapter)
    } else {
        format!("{} {}:{}", sid.book_code, sid.chapter, sid.verse)
    }
}

fn format_token_id(id: TokenId<'_>) -> String {
    format!("{}-{}", id.book_code, id.index)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    impl LintableToken for EditorToken {
        fn kind(&self) -> TokenKind {
            self.token_kind
        }

        fn span(&self) -> Option<Span> {
            Some(self.token_span)
        }

        fn text(&self) -> &str {
            &self.token_text
        }

        fn marker(&self) -> Option<&str> {
            self.token_marker.as_deref()
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
        let from_source = lint_usfm(source, LintOptions::default());
        let parsed = parse(source);
        let from_tokens = lint_tokens(&parsed.tokens, LintOptions::default());
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

        let issues = lint_tokens(&tokens, LintOptions::default());
        assert!(issues
            .issues
            .iter()
            .any(|issue| issue.code == LintCode::MissingSeparatorAfterMarker));
        assert_eq!(tokens[0].lane, 1);
    }

    #[test]
    fn missing_id_is_reported() {
        let result = lint_usfm("\\c 1\n\\v 1 text", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::MissingIdMarker));
    }

    #[test]
    fn duplicate_id_is_reported() {
        let result = lint_usfm("\\id GEN\n\\id EXO\n\\c 1\n\\v 1 text", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::DuplicateIdMarker));
    }

    #[test]
    fn missing_chapter_and_verse_numbers_are_reported() {
        let result = lint_usfm("\\id GEN\n\\c\n\\v text", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::MissingChapterNumber));
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::MissingVerseNumber));
    }

    #[test]
    fn note_submarker_outside_note_is_reported() {
        let result = lint_usfm("\\id GEN\n\\c 1\n\\ft outside note\n", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::NoteSubmarkerOutsideNote));
    }

    #[test]
    fn chapter_and_verse_metadata_attachment_is_checked() {
        let result = lint_usfm("\\id GEN\n\\c 1\n\\vp 2\n\\ca 3", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::VerseMetadataOutsideVerse));
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::ChapterMetadataOutsideChapter));
    }

    #[test]
    fn numbering_rules_are_reported() {
        let result = lint_usfm("\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n\\c 3\n", LintOptions::default());
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::DuplicateVerseNumber));
        assert!(result.issues.iter().any(|issue| issue.code == LintCode::ChapterExpectedIncreaseByOne));
    }

    #[test]
    fn structural_balance_rules_are_reported() {
        let result = lint_usfm("\\id GEN\n\\c 1\n\\p \\f + \\ft note\n\\p text", LintOptions::default());
        assert!(result.issues.iter().any(|issue| {
            matches!(
                issue.code,
                LintCode::UnclosedNote | LintCode::NoteNotClosed | LintCode::CharNotClosed
            )
        }));
    }

    #[test]
    fn note_structural_submarkers_do_not_report_implicit_or_misnested_close_on_note_end() {
        let result = lint_usfm("\\id GEN\n\\c 1\n\\p \\f + \\ft note\\f*", LintOptions::default());
        assert!(!result.issues.iter().any(|issue| issue.code == LintCode::ImplicitlyClosedMarker));
        assert!(!result.issues.iter().any(|issue| issue.code == LintCode::MisnestedCloseMarker));
    }

    #[test]
    fn rule_filtering_and_suppressions_work() {
        let mut options = LintOptions {
            enabled_codes: Some(vec![LintCode::DuplicateVerseNumber]),
            ..LintOptions::default()
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
        assert!(!suppressed
            .issues
            .iter()
            .any(|issue| issue.code == LintCode::DuplicateVerseNumber));
        assert_eq!(suppressed.summary.suppressed_count, 1);
    }

    #[test]
    fn summary_counts_by_category_and_severity() {
        let result = lint_usfm("\\c 2\n\\v 1 text\n\\v 1 text", LintOptions::default());
        assert!(result.summary.total_count > 0);
        assert!(result.summary.by_category.contains_key(&LintCategory::Document));
        assert!(result.summary.by_severity.contains_key(&LintSeverity::Error));
    }
}
