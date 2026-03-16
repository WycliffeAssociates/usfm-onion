// use crate::internal::marker_defs::{
//     InlineContext, MARKER_ID, MARKER_PERIPH, MarkerSpec, SpecContext, lookup_marker_id,
//     lookup_spec_marker, marker_allows_effective_context, marker_inline_context,
//     marker_is_note_container, marker_note_context, marker_note_subkind,
// };
// use crate::internal::markers::{MarkerKind, lookup_marker};
// use crate::internal::transform::{TokenFix, TokenTemplate};
// use crate::model::token::{Span, Token, TokenKind, TokenViewOptions, normalized_marker_name};
// use crate::parse::handle::{ParseHandle, tokens};
// use serde::{Deserialize, Serialize};
// use std::collections::{BTreeMap, HashSet};

// type ChapterLabelEntry = (Span, Option<String>, Option<String>);
// pub type MessageParams = BTreeMap<String, String>;

// pub trait LintableToken {
//     fn kind(&self) -> &TokenKind;
//     fn span(&self) -> &Span;
//     fn text(&self) -> &str;
//     fn marker(&self) -> Option<&str>;
//     fn sid(&self) -> Option<&str> {
//         None
//     }
//     fn id(&self) -> Option<&str> {
//         None
//     }
// }

// impl LintableToken for Token {
//     fn kind(&self) -> &TokenKind {
//         &self.kind
//     }

//     fn span(&self) -> &Span {
//         &self.span
//     }

//     fn text(&self) -> &str {
//         &self.text
//     }

//     fn marker(&self) -> Option<&str> {
//         self.marker.as_deref()
//     }

//     fn sid(&self) -> Option<&str> {
//         self.sid.as_deref()
//     }

//     fn id(&self) -> Option<&str> {
//         Some(&self.id)
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// pub enum LintCode {
//     MissingSeparatorAfterMarker,
//     EmptyParagraph,
//     NumberRangeAfterChapterMarker,
//     VerseRangeExpectedAfterVerseMarker,
//     VerseContentNotEmpty,
//     UnknownToken,
//     CharNotClosed,
//     NoteNotClosed,
//     ParagraphBeforeFirstChapter,
//     VerseBeforeFirstChapter,
//     NoteSubmarkerOutsideNote,
//     DuplicateIdMarker,
//     IdMarkerNotAtFileStart,
//     ChapterMetadataOutsideChapter,
//     VerseMetadataOutsideVerse,
//     MissingChapterNumber,
//     MissingVerseNumber,
//     MissingMilestoneSelfClose,
//     ImplicitlyClosedMarker,
//     StrayCloseMarker,
//     MisnestedCloseMarker,
//     UnclosedNote,
//     UnclosedMarkerAtEof,
//     DuplicateChapterNumber,
//     ChapterExpectedIncreaseByOne,
//     DuplicateVerseNumber,
//     VerseExpectedIncreaseByOne,
//     InvalidNumberRange,
//     NumberRangeNotPrecededByMarkerExpectingNumber,
//     VerseTextFollowsVerseRange,
//     UnknownMarker,
//     UnknownCloseMarker,
//     InconsistentChapterLabel,
//     MarkerNotValidInContext,
//     VerseOutsideExplicitParagraph,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// pub enum LintSeverity {
//     Error,
//     Warning,
// }

// impl LintSeverity {
//     pub fn as_str(self) -> &'static str {
//         match self {
//             Self::Error => "error",
//             Self::Warning => "warning",
//         }
//     }
// }

// impl LintCode {
//     pub const ALL: &'static [LintCode] = &[
//         LintCode::MissingSeparatorAfterMarker,
//         LintCode::EmptyParagraph,
//         LintCode::NumberRangeAfterChapterMarker,
//         LintCode::VerseRangeExpectedAfterVerseMarker,
//         LintCode::VerseContentNotEmpty,
//         LintCode::UnknownToken,
//         LintCode::CharNotClosed,
//         LintCode::NoteNotClosed,
//         LintCode::ParagraphBeforeFirstChapter,
//         LintCode::VerseBeforeFirstChapter,
//         LintCode::NoteSubmarkerOutsideNote,
//         LintCode::DuplicateIdMarker,
//         LintCode::IdMarkerNotAtFileStart,
//         LintCode::ChapterMetadataOutsideChapter,
//         LintCode::VerseMetadataOutsideVerse,
//         LintCode::MissingChapterNumber,
//         LintCode::MissingVerseNumber,
//         LintCode::MissingMilestoneSelfClose,
//         LintCode::ImplicitlyClosedMarker,
//         LintCode::StrayCloseMarker,
//         LintCode::MisnestedCloseMarker,
//         LintCode::UnclosedNote,
//         LintCode::UnclosedMarkerAtEof,
//         LintCode::DuplicateChapterNumber,
//         LintCode::ChapterExpectedIncreaseByOne,
//         LintCode::DuplicateVerseNumber,
//         LintCode::VerseExpectedIncreaseByOne,
//         LintCode::InvalidNumberRange,
//         LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
//         LintCode::VerseTextFollowsVerseRange,
//         LintCode::UnknownMarker,
//         LintCode::UnknownCloseMarker,
//         LintCode::InconsistentChapterLabel,
//         LintCode::MarkerNotValidInContext,
//         LintCode::VerseOutsideExplicitParagraph,
//     ];

//     pub fn as_str(self) -> &'static str {
//         match self {
//             Self::MissingSeparatorAfterMarker => "missing-separator-after-marker",
//             Self::EmptyParagraph => "empty-paragraph",
//             Self::NumberRangeAfterChapterMarker => "number-range-after-chapter-marker",
//             Self::VerseRangeExpectedAfterVerseMarker => "verse-range-expected-after-verse-marker",
//             Self::VerseContentNotEmpty => "verse-content-not-empty",
//             Self::UnknownToken => "unknown-token",
//             Self::CharNotClosed => "char-not-closed",
//             Self::NoteNotClosed => "note-not-closed",
//             Self::ParagraphBeforeFirstChapter => "paragraph-before-first-chapter",
//             Self::VerseBeforeFirstChapter => "verse-before-first-chapter",
//             Self::NoteSubmarkerOutsideNote => "note-submarker-outside-note",
//             Self::DuplicateIdMarker => "duplicate-id-marker",
//             Self::IdMarkerNotAtFileStart => "id-marker-not-at-file-start",
//             Self::ChapterMetadataOutsideChapter => "chapter-metadata-outside-chapter",
//             Self::VerseMetadataOutsideVerse => "verse-metadata-outside-verse",
//             Self::MissingChapterNumber => "missing-chapter-number",
//             Self::MissingVerseNumber => "missing-verse-number",
//             Self::MissingMilestoneSelfClose => "missing-milestone-self-close",
//             Self::ImplicitlyClosedMarker => "implicitly-closed-marker",
//             Self::StrayCloseMarker => "stray-close-marker",
//             Self::MisnestedCloseMarker => "misnested-close-marker",
//             Self::UnclosedNote => "unclosed-note",
//             Self::UnclosedMarkerAtEof => "unclosed-marker-at-eof",
//             Self::DuplicateChapterNumber => "duplicate-chapter-number",
//             Self::ChapterExpectedIncreaseByOne => "chapter-expected-increase-by-one",
//             Self::DuplicateVerseNumber => "duplicate-verse-number",
//             Self::VerseExpectedIncreaseByOne => "verse-expected-increase-by-one",
//             Self::InvalidNumberRange => "invalid-number-range",
//             Self::NumberRangeNotPrecededByMarkerExpectingNumber => {
//                 "number-range-not-preceded-by-marker-expecting-number"
//             }
//             Self::VerseTextFollowsVerseRange => "verse-text-follows-verse-range",
//             Self::UnknownMarker => "unknown-marker",
//             Self::UnknownCloseMarker => "unknown-close-marker",
//             Self::InconsistentChapterLabel => "inconsistent-chapter-label",
//             Self::MarkerNotValidInContext => "marker-not-valid-in-context",
//             Self::VerseOutsideExplicitParagraph => "verse-outside-explicit-paragraph",
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct LintIssue {
//     pub code: LintCode,
//     pub severity: LintSeverity,
//     pub marker: Option<String>,
//     pub message: String,
//     pub message_params: MessageParams,
//     pub span: Span,
//     pub related_span: Option<Span>,
//     pub token_id: Option<String>,
//     pub related_token_id: Option<String>,
//     pub sid: Option<String>,
//     pub fix: Option<TokenFix>,
// }

// fn default_severity(code: LintCode) -> LintSeverity {
//     match code {
//         LintCode::EmptyParagraph => LintSeverity::Warning,
//         _ => LintSeverity::Error,
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct LintSuppression {
//     pub code: LintCode,
//     pub sid: String,
// }

// /// Linting options for parsed content.
// ///
// /// `token_rules` governs the token-first lint engine. `include_parse_recoveries`
// /// additionally exposes parser recovery events as lint issues.
// #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
// pub struct LintOptions {
//     pub include_parse_recoveries: bool,
//     pub token_view: TokenViewOptions,
//     pub token_rules: TokenLintOptions,
// }

// /// Token-first lint configuration.
// ///
// /// There is no custom lint-pass plugin API yet. Today you can:
// /// - disable built-in rules entirely with `disabled_rules`
// /// - suppress exact `(code, sid)` findings with `suppressions`
// /// - opt into a small amount of permissive structural handling with
// ///   `allow_implicit_chapter_content_verse`
// #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
// pub struct TokenLintOptions {
//     pub disabled_rules: Vec<LintCode>,
//     pub suppressions: Vec<LintSuppression>,
//     pub allow_implicit_chapter_content_verse: bool,
// }

// impl From<TokenLintOptions> for LintOptions {
//     fn from(value: TokenLintOptions) -> Self {
//         Self {
//             include_parse_recoveries: false,
//             token_view: TokenViewOptions::default(),
//             token_rules: value,
//         }
//     }
// }

// pub fn lint(handle: &ParseHandle, options: LintOptions) -> Vec<LintIssue> {
//     let projected = tokens(handle, options.token_view);
//     lint_tokens(&projected, options.token_rules)
// }

// pub fn lint_tokens<T: LintableToken>(tokens: &[T], options: TokenLintOptions) -> Vec<LintIssue> {
//     let mut issues = Vec::new();
//     let enabled = EnabledRules::new(&options.disabled_rules);

//     if enabled.has(LintCode::MissingSeparatorAfterMarker) {
//         lint_missing_separator_after_marker(tokens, &mut issues);
//     }
//     if enabled.has(LintCode::EmptyParagraph) {
//         lint_empty_paragraphs(tokens, &mut issues);
//     }
//     if enabled.has_any(&[
//         LintCode::UnknownToken,
//         LintCode::NumberRangeAfterChapterMarker,
//         LintCode::VerseRangeExpectedAfterVerseMarker,
//         LintCode::VerseContentNotEmpty,
//     ]) {
//         lint_expectation_and_unknown_token_rules(tokens, &enabled, &mut issues);
//     }
//     if enabled.has_any(&[
//         LintCode::ParagraphBeforeFirstChapter,
//         LintCode::VerseBeforeFirstChapter,
//         LintCode::NoteSubmarkerOutsideNote,
//         LintCode::DuplicateIdMarker,
//         LintCode::IdMarkerNotAtFileStart,
//         LintCode::ChapterMetadataOutsideChapter,
//         LintCode::VerseMetadataOutsideVerse,
//         LintCode::VerseOutsideExplicitParagraph,
//     ]) {
//         lint_structure_rules(tokens, &options, &enabled, &mut issues);
//     }
//     if enabled.has(LintCode::UnknownMarker) {
//         lint_unknown_markers(tokens, &mut issues);
//     }
//     if enabled.has(LintCode::UnknownCloseMarker) {
//         lint_unknown_close_markers(tokens, &mut issues);
//     }
//     if enabled.has_any(&[
//         LintCode::DuplicateChapterNumber,
//         LintCode::ChapterExpectedIncreaseByOne,
//         LintCode::InconsistentChapterLabel,
//     ]) {
//         lint_chapter_rules(tokens, &enabled, &mut issues);
//     }
//     if enabled.has_any(&[
//         LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
//         LintCode::InvalidNumberRange,
//         LintCode::DuplicateVerseNumber,
//         LintCode::VerseExpectedIncreaseByOne,
//         LintCode::VerseTextFollowsVerseRange,
//     ]) {
//         lint_number_and_verse_rules(tokens, &enabled, &mut issues);
//     }
//     if enabled.has(LintCode::MarkerNotValidInContext) {
//         lint_marker_context_rules(tokens, &mut issues);
//     }
//     if enabled.has_any(&[
//         LintCode::CharNotClosed,
//         LintCode::NoteNotClosed,
//         LintCode::StrayCloseMarker,
//         LintCode::MisnestedCloseMarker,
//     ]) {
//         lint_marker_balance_rules(tokens, &enabled, &mut issues);
//     }

//     dedupe_and_filter_issues(issues, &options.suppressions)
// }

// fn lint_expectation_and_unknown_token_rules<T: LintableToken>(
//     tokens: &[T],
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     for index in 0..tokens.len() {
//         let token = &tokens[index];

//         if enabled.has(LintCode::UnknownToken)
//             && token.kind() == &TokenKind::Text
//             && let Some(issue) = lint_unknown_token_like(token)
//         {
//             issues.push(issue);
//         }

//         if token.kind() != &TokenKind::Marker {
//             continue;
//         }

//         let marker = token
//             .marker()
//             .map(normalized_marker_name)
//             .unwrap_or_default();
//         match marker {
//             "c" if enabled.has(LintCode::NumberRangeAfterChapterMarker) => {
//                 if next_number_token_index(tokens, index + 1).is_none() {
//                     issues.push(simple_issue(
//                         LintCode::NumberRangeAfterChapterMarker,
//                         "number range expected after \\c".to_string(),
//                         token,
//                     ));
//                 }
//             }
//             "v" => {
//                 if enabled.has(LintCode::VerseRangeExpectedAfterVerseMarker)
//                     && next_number_token_index(tokens, index + 1).is_none()
//                 {
//                     issues.push(simple_issue(
//                         LintCode::VerseRangeExpectedAfterVerseMarker,
//                         "verse number expected after \\v".to_string(),
//                         token,
//                     ));
//                 }

//                 if enabled.has(LintCode::VerseContentNotEmpty)
//                     && let Some(next_index) = next_significant_token_index(tokens, index + 1)
//                     && tokens[next_index].kind() == &TokenKind::Text
//                     && tokens[next_index].text().trim().is_empty()
//                 {
//                     issues.push(LintIssue {
//                         code: LintCode::VerseContentNotEmpty,
//                         severity: default_severity(LintCode::VerseContentNotEmpty),
//                         marker: Some("v".to_string()),
//                         message: "verse content expected after \\v".to_string(),
//                         message_params: MessageParams::new(),
//                         span: tokens[next_index].span().clone(),
//                         related_span: Some(token.span().clone()),
//                         token_id: tokens[next_index].id().map(ToOwned::to_owned),
//                         related_token_id: token.id().map(ToOwned::to_owned),
//                         sid: token.sid().map(ToOwned::to_owned),
//                         fix: None,
//                     });
//                 }
//             }
//             _ => {}
//         }
//     }
// }

// fn lint_structure_rules<T: LintableToken>(
//     tokens: &[T],
//     options: &TokenLintOptions,
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     let mut saw_content = false;
//     let mut id_seen = false;
//     let mut note_stack: Vec<String> = Vec::new();
//     let mut document_state = DocumentLintState::default();

//     for (index, token) in tokens.iter().enumerate() {
//         match token.kind() {
//             TokenKind::Newline => continue,
//             _ => {}
//         }

//         if token.kind() == &TokenKind::Marker {
//             let marker = token
//                 .marker()
//                 .map(normalized_marker_name)
//                 .unwrap_or_default();
//             let resolved = ResolvedContextMarker::new(marker);

//             if enabled.has(LintCode::IdMarkerNotAtFileStart) && marker == "id" && saw_content {
//                 issues.push(simple_issue(
//                     LintCode::IdMarkerNotAtFileStart,
//                     "\\id marker appears after book content has already started".to_string(),
//                     token,
//                 ));
//             }
//             if enabled.has(LintCode::DuplicateIdMarker) && marker == "id" {
//                 if id_seen {
//                     issues.push(simple_issue(
//                         LintCode::DuplicateIdMarker,
//                         "duplicate \\id marker".to_string(),
//                         token,
//                     ));
//                 }
//                 id_seen = true;
//             }

//             let prospective_state =
//                 if resolved.is_some_and(|resolved| resolved.kind == MarkerKind::Note) {
//                     resolved.map_or(document_state.current_root_context(), |resolved| {
//                         document_state.current_validation_context(resolved)
//                     })
//                 } else {
//                     let mut next_state = document_state.clone();
//                     if let Some(resolved) = resolved {
//                         next_state.apply_marker(tokens, index, resolved);
//                     }
//                     next_state.current_root_context()
//                 };

//             if enabled.has(LintCode::ParagraphBeforeFirstChapter)
//                 && !document_state.saw_chapter
//                 && document_state.kind == DocumentKind::Scripture
//                 && resolved.is_some_and(|resolved| resolved.kind == MarkerKind::Paragraph)
//                 && is_body_paragraph_marker(marker)
//                 && prospective_state == SpecContext::ChapterContent
//             {
//                 issues.push(simple_issue(
//                     LintCode::ParagraphBeforeFirstChapter,
//                     format!("body paragraph marker \\{marker} appears before the first chapter"),
//                     token,
//                 ));
//             }

//             if enabled.has(LintCode::VerseBeforeFirstChapter)
//                 && !document_state.saw_chapter
//                 && document_state.kind == DocumentKind::Scripture
//                 && marker == "v"
//             {
//                 issues.push(simple_issue(
//                     LintCode::VerseBeforeFirstChapter,
//                     "verse marker appears before the first chapter".to_string(),
//                     token,
//                 ));
//             }

//             if enabled.has(LintCode::VerseOutsideExplicitParagraph)
//                 && !options.allow_implicit_chapter_content_verse
//                 && marker == "v"
//                 && document_state.kind == DocumentKind::Scripture
//                 && document_state.current_root_context() == SpecContext::ChapterContent
//                 && !matches!(
//                     document_state.block_context,
//                     Some(SpecContext::Para | SpecContext::List | SpecContext::Table)
//                 )
//             {
//                 issues.push(simple_issue(
//                     LintCode::VerseOutsideExplicitParagraph,
//                     "verse marker appears outside an explicit paragraph, list, or table block"
//                         .to_string(),
//                     token,
//                 ));
//             }

//             if enabled.has(LintCode::NoteSubmarkerOutsideNote)
//                 && resolved.is_some_and(|resolved| resolved.valid_in_note)
//                 && note_stack.is_empty()
//             {
//                 issues.push(simple_issue(
//                     LintCode::NoteSubmarkerOutsideNote,
//                     format!("note submarker \\{marker} appears outside an open note"),
//                     token,
//                 ));
//             }

//             if enabled.has(LintCode::ChapterMetadataOutsideChapter)
//                 && matches!(marker, "ca" | "cp")
//                 && !matches_previous_marker_and_number(tokens, index, "c")
//             {
//                 issues.push(simple_issue(
//                     LintCode::ChapterMetadataOutsideChapter,
//                     format!("chapter metadata \\{marker} is not attached to a chapter marker"),
//                     token,
//                 ));
//             }

//             if enabled.has(LintCode::VerseMetadataOutsideVerse)
//                 && matches!(marker, "va" | "vp")
//                 && !matches_previous_marker_and_number(tokens, index, "v")
//             {
//                 issues.push(simple_issue(
//                     LintCode::VerseMetadataOutsideVerse,
//                     format!("verse metadata \\{marker} is not attached to a verse marker"),
//                     token,
//                 ));
//             }

//             if resolved.is_some_and(|resolved| resolved.kind == MarkerKind::Note) {
//                 note_stack.push(marker.to_string());
//             }

//             if let Some(resolved) = resolved {
//                 document_state.apply_marker(tokens, index, resolved);
//             }

//             saw_content = true;
//             continue;
//         }

//         if token.kind() == &TokenKind::EndMarker {
//             if let Some(marker) = token.marker().map(normalized_marker_name)
//                 && lookup_marker(marker).kind == MarkerKind::Note
//             {
//                 if let Some(idx) = note_stack.iter().rposition(|open| open == marker) {
//                     note_stack.truncate(idx);
//                 } else {
//                     note_stack.clear();
//                 }
//                 let _ = document_state.note_stack.pop();
//             }
//             saw_content = true;
//             continue;
//         }

//         if token.kind() != &TokenKind::BookCode {
//             saw_content = true;
//         }
//     }
// }

// #[derive(Default)]
// struct EnabledRules {
//     disabled: HashSet<LintCode>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum DocumentKind {
//     Scripture,
//     PeripheralStandalone,
//     PeripheralDivided,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum TopLevelSlot {
//     AwaitDivision,
//     Headers,
//     Titles,
//     Introduction,
//     IntroductionEndTitles,
//     Content,
// }

// #[derive(Clone)]
// struct DocumentLintState {
//     kind: DocumentKind,
//     slot: TopLevelSlot,
//     saw_chapter: bool,
//     block_context: Option<SpecContext>,
//     note_stack: Vec<SpecContext>,
// }

// #[derive(Clone, Copy)]
// struct ResolvedContextMarker<'a> {
//     marker: &'a str,
//     spec: &'static MarkerSpec,
//     kind: MarkerKind,
//     inline_context: Option<InlineContext>,
//     note_context: Option<SpecContext>,
//     valid_in_note: bool,
// }

// impl<'a> ResolvedContextMarker<'a> {
//     fn new(marker: &'a str) -> Option<Self> {
//         let spec = lookup_spec_marker(marker)?;
//         Some(Self {
//             marker,
//             spec,
//             kind: spec.kind.to_marker_kind(marker),
//             inline_context: marker_inline_context(marker),
//             note_context: marker_note_context(marker),
//             valid_in_note: marker_note_subkind(marker).is_some(),
//         })
//     }

//     fn id(self) -> crate::internal::marker_defs::MarkerId {
//         crate::internal::marker_defs::MarkerId::new(self.spec.marker)
//     }

//     fn allows_effective_context(self, context: SpecContext) -> bool {
//         marker_spec_allows_effective_context(self.spec, self.kind, context)
//     }

//     fn paragraph_block_context(self) -> SpecContext {
//         paragraph_block_context_from_inline(self.inline_context)
//     }

//     fn note_context(self) -> SpecContext {
//         self.note_context.unwrap_or(SpecContext::Footnote)
//     }
// }

// impl Default for DocumentLintState {
//     fn default() -> Self {
//         Self {
//             kind: DocumentKind::Scripture,
//             slot: TopLevelSlot::Headers,
//             saw_chapter: false,
//             block_context: None,
//             note_stack: Vec::new(),
//         }
//     }
// }

// impl DocumentLintState {
//     fn current_root_context(&self) -> SpecContext {
//         match (self.kind, self.slot) {
//             (DocumentKind::PeripheralDivided, TopLevelSlot::AwaitDivision) => {
//                 SpecContext::Peripheral
//             }
//             (_, TopLevelSlot::Headers) => SpecContext::BookHeaders,
//             (_, TopLevelSlot::Titles) => SpecContext::BookTitles,
//             (_, TopLevelSlot::Introduction) => SpecContext::BookIntroduction,
//             (_, TopLevelSlot::IntroductionEndTitles) => SpecContext::BookIntroductionEndTitles,
//             (DocumentKind::Scripture, TopLevelSlot::Content) => SpecContext::ChapterContent,
//             (DocumentKind::PeripheralStandalone, TopLevelSlot::Content)
//             | (DocumentKind::PeripheralDivided, TopLevelSlot::Content) => {
//                 SpecContext::PeripheralContent
//             }
//             (_, TopLevelSlot::AwaitDivision) => SpecContext::Peripheral,
//         }
//     }

//     fn current_note_context(&self) -> Option<SpecContext> {
//         self.note_stack.last().copied()
//     }

//     fn current_validation_context(&self, marker: ResolvedContextMarker<'_>) -> SpecContext {
//         let root_context = self.current_root_context();
//         marker_validation_context(
//             marker.kind,
//             root_context,
//             self.block_context,
//             self.current_note_context(),
//         )
//     }

//     fn select_top_level_slot(&self, marker: ResolvedContextMarker<'_>) -> TopLevelSlot {
//         if marker.id() == MARKER_PERIPH {
//             return TopLevelSlot::AwaitDivision;
//         }

//         let contexts = top_level_contexts_for(self.kind);
//         let start = top_level_slot_index(self.slot);

//         contexts
//             .iter()
//             .copied()
//             .skip(start)
//             .find(|(_, context)| marker.allows_effective_context(*context))
//             .map(|(slot, _)| slot)
//             .unwrap_or(self.slot)
//     }

//     fn apply_marker<T: LintableToken>(
//         &mut self,
//         tokens: &[T],
//         index: usize,
//         marker: ResolvedContextMarker<'_>,
//     ) {
//         match marker.kind {
//             MarkerKind::Header => {
//                 if marker.id() == MARKER_ID
//                     && let Some(book_code) = next_book_code_after_marker(tokens, index)
//                 {
//                     self.kind = infer_document_kind(book_code);
//                     if self.kind == DocumentKind::PeripheralDivided {
//                         self.slot = TopLevelSlot::AwaitDivision;
//                     } else {
//                         self.slot = TopLevelSlot::Headers;
//                     }
//                     self.block_context = None;
//                     self.note_stack.clear();
//                 }
//             }
//             MarkerKind::Chapter => {
//                 self.saw_chapter = true;
//                 self.slot = TopLevelSlot::Content;
//                 self.block_context = None;
//             }
//             MarkerKind::Paragraph => {
//                 if self.current_note_context().is_none() {
//                     self.slot = self.select_top_level_slot(marker);
//                 }
//                 self.block_context = Some(marker.paragraph_block_context());
//             }
//             MarkerKind::Meta => {}
//             MarkerKind::Note => {
//                 self.note_stack.push(marker.note_context());
//             }
//             MarkerKind::Periph => {
//                 self.kind = DocumentKind::PeripheralDivided;
//                 self.slot = TopLevelSlot::Headers;
//                 self.block_context = None;
//                 self.note_stack.clear();
//                 self.saw_chapter = false;
//             }
//             MarkerKind::SidebarStart => {
//                 self.slot = TopLevelSlot::Content;
//                 self.block_context = Some(SpecContext::Sidebar);
//             }
//             MarkerKind::SidebarEnd => {
//                 self.block_context = None;
//             }
//             MarkerKind::TableRow | MarkerKind::TableCell => {
//                 self.slot = TopLevelSlot::Content;
//                 self.block_context = Some(SpecContext::Table);
//             }
//             MarkerKind::Verse
//             | MarkerKind::Character
//             | MarkerKind::Figure
//             | MarkerKind::MilestoneStart
//             | MarkerKind::MilestoneEnd
//             | MarkerKind::Unknown => {}
//         }
//     }
// }

// fn top_level_contexts_for(kind: DocumentKind) -> &'static [(TopLevelSlot, SpecContext)] {
//     match kind {
//         DocumentKind::Scripture => &[
//             (TopLevelSlot::Headers, SpecContext::BookHeaders),
//             (TopLevelSlot::Titles, SpecContext::BookTitles),
//             (TopLevelSlot::Introduction, SpecContext::BookIntroduction),
//             (
//                 TopLevelSlot::IntroductionEndTitles,
//                 SpecContext::BookIntroductionEndTitles,
//             ),
//             (TopLevelSlot::Content, SpecContext::ChapterContent),
//         ],
//         DocumentKind::PeripheralStandalone | DocumentKind::PeripheralDivided => &[
//             (TopLevelSlot::Headers, SpecContext::BookHeaders),
//             (TopLevelSlot::Titles, SpecContext::BookTitles),
//             (TopLevelSlot::Introduction, SpecContext::BookIntroduction),
//             (
//                 TopLevelSlot::IntroductionEndTitles,
//                 SpecContext::BookIntroductionEndTitles,
//             ),
//             (TopLevelSlot::Content, SpecContext::PeripheralContent),
//         ],
//     }
// }

// fn top_level_slot_index(slot: TopLevelSlot) -> usize {
//     match slot {
//         TopLevelSlot::AwaitDivision | TopLevelSlot::Headers => 0,
//         TopLevelSlot::Titles => 1,
//         TopLevelSlot::Introduction => 2,
//         TopLevelSlot::IntroductionEndTitles => 3,
//         TopLevelSlot::Content => 4,
//     }
// }

// fn infer_document_kind(book_code: &str) -> DocumentKind {
//     match book_code {
//         "FRT" | "INT" | "BAK" | "OTH" => DocumentKind::PeripheralDivided,
//         "CNC" | "GLO" | "TDX" | "NDX" => DocumentKind::PeripheralStandalone,
//         _ => DocumentKind::Scripture,
//     }
// }

// fn next_book_code_after_marker<T: LintableToken>(
//     tokens: &[T],
//     marker_index: usize,
// ) -> Option<&str> {
//     let next_index = next_significant_token_index(tokens, marker_index + 1)?;
//     (tokens[next_index].kind() == &TokenKind::BookCode).then(|| tokens[next_index].text().trim())
// }

// impl EnabledRules {
//     fn new(disabled_rules: &[LintCode]) -> Self {
//         Self {
//             disabled: disabled_rules.iter().copied().collect(),
//         }
//     }

//     fn has(&self, code: LintCode) -> bool {
//         !self.disabled.contains(&code)
//     }

//     fn has_any(&self, codes: &[LintCode]) -> bool {
//         codes.iter().copied().any(|code| self.has(code))
//     }
// }

// fn lint_missing_separator_after_marker<T: LintableToken>(
//     tokens: &[T],
//     issues: &mut Vec<LintIssue>,
// ) {
//     for window in tokens.windows(2) {
//         let [current, next] = window else {
//             continue;
//         };

//         if current.kind() != &TokenKind::Marker || next.kind() != &TokenKind::Text {
//             continue;
//         }

//         let Some(marker) = current.marker() else {
//             continue;
//         };
//         let marker = normalized_marker_name(marker);
//         let marker_kind = lookup_marker(marker).kind;
//         if matches!(
//             marker_kind,
//             MarkerKind::MilestoneStart | MarkerKind::MilestoneEnd
//         ) {
//             continue;
//         }
//         if matches!(marker_kind, MarkerKind::Unknown) && marker.starts_with('z') {
//             continue;
//         }
//         if ends_with_horizontal_whitespace(current.text()) {
//             continue;
//         }
//         if starts_with_horizontal_whitespace(next.text()) {
//             continue;
//         }

//         issues.push(LintIssue {
//             code: LintCode::MissingSeparatorAfterMarker,
//             severity: default_severity(LintCode::MissingSeparatorAfterMarker),
//             marker: Some(marker.to_string()),
//             message: format!("marker \\{marker} is immediately followed by text"),
//             message_params: MessageParams::from([("marker".to_string(), marker.to_string())]),
//             span: current.span().clone(),
//             related_span: Some(next.span().clone()),
//             token_id: current.id().map(ToOwned::to_owned),
//             related_token_id: next.id().map(ToOwned::to_owned),
//             sid: current.sid().map(ToOwned::to_owned),
//             fix: next.id().map(|id| TokenFix::ReplaceToken {
//                 code: "insert-separator-after-marker".to_string(),
//                 label: format!("insert separator after \\{marker}"),
//                 label_params: MessageParams::from([("marker".to_string(), marker.to_string())]),
//                 target_token_id: id.to_string(),
//                 replacements: vec![TokenTemplate {
//                     kind: TokenKind::Text,
//                     text: format!(" {}", next.text()),
//                     marker: None,
//                     sid: current.sid().map(ToOwned::to_owned),
//                 }],
//             }),
//         });
//     }
// }

// fn lint_empty_paragraphs<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
//     for index in 0..tokens.len() {
//         let token = &tokens[index];
//         if token.kind() != &TokenKind::Marker {
//             continue;
//         }
//         let Some(marker) = token.marker().map(normalized_marker_name) else {
//             continue;
//         };
//         if !is_body_paragraph_marker(marker) {
//             continue;
//         }
//         if marker_is_intentionally_empty_block(marker) {
//             continue;
//         }
//         let Some(boundary_index) = empty_paragraph_boundary_index(tokens, index) else {
//             continue;
//         };

//         issues.push(LintIssue {
//             code: LintCode::EmptyParagraph,
//             severity: default_severity(LintCode::EmptyParagraph),
//             marker: Some(marker.to_string()),
//             message: format!(
//                 "paragraph marker \\{marker} creates an empty block before the next block marker"
//             ),
//             message_params: MessageParams::from([("marker".to_string(), marker.to_string())]),
//             span: token.span().clone(),
//             related_span: Some(tokens[boundary_index].span().clone()),
//             token_id: token.id().map(ToOwned::to_owned),
//             related_token_id: tokens[boundary_index].id().map(ToOwned::to_owned),
//             sid: token.sid().map(ToOwned::to_owned),
//             fix: token.id().map(|id| TokenFix::DeleteToken {
//                 code: "remove-empty-paragraph".to_string(),
//                 label: format!("remove empty \\{marker} paragraph"),
//                 label_params: MessageParams::from([("marker".to_string(), marker.to_string())]),
//                 target_token_id: id.to_string(),
//             }),
//         });
//     }
// }

// fn lint_unknown_markers<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
//     for token in tokens {
//         if token.kind() != &TokenKind::Marker {
//             continue;
//         }
//         let Some(marker) = token.marker().map(normalized_marker_name) else {
//             continue;
//         };
//         if lookup_marker(marker).kind != MarkerKind::Unknown {
//             continue;
//         }
//         issues.push(simple_issue(
//             LintCode::UnknownMarker,
//             format!("unknown marker \\{marker}"),
//             token,
//         ));
//     }
// }

// fn lint_unknown_close_markers<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
//     for token in tokens {
//         if token.kind() != &TokenKind::EndMarker {
//             continue;
//         }
//         let Some(marker) = token.marker().map(normalized_marker_name) else {
//             continue;
//         };
//         if lookup_marker(marker).kind != MarkerKind::Unknown {
//             continue;
//         }
//         issues.push(simple_issue(
//             LintCode::UnknownCloseMarker,
//             format!("unknown closing marker \\{marker}*"),
//             token,
//         ));
//     }
// }

// fn lint_chapter_rules<T: LintableToken>(
//     tokens: &[T],
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     let mut seen_chapters = HashSet::new();
//     let mut last_chapter: Option<u32> = None;
//     let mut labels: BTreeMap<String, Vec<ChapterLabelEntry>> = BTreeMap::new();

//     let mut index = 0usize;
//     while index < tokens.len() {
//         let token = &tokens[index];

//         if token.kind() == &TokenKind::Marker
//             && token.marker().map(normalized_marker_name) == Some("c")
//             && let Some(number_index) = next_number_token_index(tokens, index + 1)
//             && let Some(chapter) = parse_primary_number(tokens[number_index].text())
//         {
//             if enabled.has(LintCode::DuplicateChapterNumber) && seen_chapters.contains(&chapter) {
//                 issues.push(LintIssue {
//                     code: LintCode::DuplicateChapterNumber,
//                     severity: default_severity(LintCode::DuplicateChapterNumber),
//                     marker: Some("c".to_string()),
//                     message: format!("duplicate chapter number {chapter}"),
//                     message_params: MessageParams::from([(
//                         "chapter".to_string(),
//                         chapter.to_string(),
//                     )]),
//                     span: tokens[number_index].span().clone(),
//                     related_span: None,
//                     token_id: tokens[number_index].id().map(ToOwned::to_owned),
//                     related_token_id: None,
//                     sid: token.sid().map(ToOwned::to_owned),
//                     fix: None,
//                 });
//             }

//             if enabled.has(LintCode::ChapterExpectedIncreaseByOne) {
//                 let expected = last_chapter.map_or(1, |last| last + 1);
//                 if chapter != expected {
//                     issues.push(LintIssue {
//                         code: LintCode::ChapterExpectedIncreaseByOne,
//                         severity: default_severity(LintCode::ChapterExpectedIncreaseByOne),
//                         marker: Some("c".to_string()),
//                         message: format!("expected chapter number {expected}, found {chapter}"),
//                         message_params: MessageParams::from([
//                             ("expected".to_string(), expected.to_string()),
//                             ("found".to_string(), chapter.to_string()),
//                         ]),
//                         span: tokens[number_index].span().clone(),
//                         related_span: None,
//                         token_id: tokens[number_index].id().map(ToOwned::to_owned),
//                         related_token_id: None,
//                         sid: token.sid().map(ToOwned::to_owned),
//                         fix: None,
//                     });
//                 }
//             }

//             seen_chapters.insert(chapter);
//             last_chapter = Some(chapter);
//         }

//         if enabled.has(LintCode::InconsistentChapterLabel)
//             && token.kind() == &TokenKind::Marker
//             && token.marker().map(normalized_marker_name) == Some("cl")
//             && let Some(text_index) = next_text_token_index(tokens, index + 1)
//         {
//             let label = strip_digits(tokens[text_index].text().trim())
//                 .trim()
//                 .to_string();
//             if !label.is_empty() {
//                 labels.entry(label).or_default().push((
//                     tokens[text_index].span().clone(),
//                     tokens[text_index].id().map(ToOwned::to_owned),
//                     tokens[text_index].sid().map(ToOwned::to_owned),
//                 ));
//             }
//         }

//         index += 1;
//     }

//     if enabled.has(LintCode::InconsistentChapterLabel) && labels.len() > 1 {
//         let canonical = labels
//             .iter()
//             .max_by_key(|(_, entries)| entries.len())
//             .map(|(label, _)| label.clone());
//         if let Some(canonical) = canonical {
//             for (label, entries) in labels {
//                 if label == canonical {
//                     continue;
//                 }
//                 for (span, token_id, sid) in entries {
//                     issues.push(LintIssue {
//                         code: LintCode::InconsistentChapterLabel,
//                         severity: default_severity(LintCode::InconsistentChapterLabel),
//                         marker: Some("cl".to_string()),
//                         message: format!(
//                             "inconsistent chapter label '{label}', expected the canonical label '{canonical}'"
//                         ),
//                         message_params: MessageParams::from([
//                             ("label".to_string(), label.clone()),
//                             ("canonical".to_string(), canonical.clone()),
//                         ]),
//                         span,
//                         related_span: None,
//                         token_id,
//                         related_token_id: None,
//                         sid,
//                         fix: None,
//                     });
//                 }
//             }
//         }
//     }
// }

// fn lint_number_and_verse_rules<T: LintableToken>(
//     tokens: &[T],
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     let mut current_chapter = 0u32;
//     let mut verse_state_by_chapter: BTreeMap<u32, VerseState> = BTreeMap::new();

//     for index in 0..tokens.len() {
//         let token = &tokens[index];

//         if token.kind() == &TokenKind::Marker
//             && token.marker().map(normalized_marker_name) == Some("c")
//             && let Some(number_index) = next_number_token_index(tokens, index + 1)
//             && let Some(chapter) = parse_primary_number(tokens[number_index].text())
//         {
//             current_chapter = chapter;
//         }

//         if enabled.has(LintCode::NumberRangeNotPrecededByMarkerExpectingNumber)
//             && token.kind() == &TokenKind::Number
//         {
//             lint_number_predecessor(tokens, index, issues);
//         }

//         if token.kind() != &TokenKind::Marker
//             || token.marker().map(normalized_marker_name) != Some("v")
//         {
//             continue;
//         }

//         let Some(number_index) = next_number_token_index(tokens, index + 1) else {
//             continue;
//         };
//         let number_token = &tokens[number_index];
//         let value = number_token.text().trim();

//         if enabled.has(LintCode::InvalidNumberRange) && parse_number_range(value).is_none() {
//             issues.push(LintIssue {
//                 code: LintCode::InvalidNumberRange,
//                 severity: default_severity(LintCode::InvalidNumberRange),
//                 marker: Some("v".to_string()),
//                 message: format!("invalid verse range {value}"),
//                 message_params: MessageParams::from([("value".to_string(), value.to_string())]),
//                 span: number_token.span().clone(),
//                 related_span: None,
//                 token_id: number_token.id().map(ToOwned::to_owned),
//                 related_token_id: None,
//                 sid: number_token.sid().map(ToOwned::to_owned),
//                 fix: None,
//             });
//             continue;
//         }

//         let Some((start, end)) = parse_number_range(value) else {
//             continue;
//         };

//         let chapter = if current_chapter == 0 {
//             parse_sid_chapter(number_token.sid()).unwrap_or(0)
//         } else {
//             current_chapter
//         };
//         let chapter_state = verse_state_by_chapter.entry(chapter).or_default();

//         let mut duplicate = false;
//         for verse in start..=end {
//             if chapter_state.seen.contains_key(&verse) {
//                 duplicate = true;
//                 break;
//             }
//         }

//         if enabled.has(LintCode::DuplicateVerseNumber) && duplicate {
//             issues.push(LintIssue {
//                 code: LintCode::DuplicateVerseNumber,
//                 severity: default_severity(LintCode::DuplicateVerseNumber),
//                 marker: Some("v".to_string()),
//                 message: format!("duplicate verse number {value}"),
//                 message_params: MessageParams::from([("value".to_string(), value.to_string())]),
//                 span: number_token.span().clone(),
//                 related_span: None,
//                 token_id: number_token.id().map(ToOwned::to_owned),
//                 related_token_id: None,
//                 sid: number_token.sid().map(ToOwned::to_owned),
//                 fix: build_set_number_fix(number_token, chapter_state.last + 1),
//             });
//         } else if enabled.has(LintCode::VerseExpectedIncreaseByOne) {
//             let expected = chapter_state.last + 1;
//             if start != expected {
//                 issues.push(LintIssue {
//                     code: LintCode::VerseExpectedIncreaseByOne,
//                     severity: default_severity(LintCode::VerseExpectedIncreaseByOne),
//                     marker: Some("v".to_string()),
//                     message: if chapter_state.last > 0 {
//                         format!(
//                             "previous verse number was {}, so expected {} here, found {}",
//                             chapter_state.last, expected, start
//                         )
//                     } else {
//                         format!("expected verse {expected} here, found {start}")
//                     },
//                     message_params: MessageParams::from([
//                         ("previous".to_string(), chapter_state.last.to_string()),
//                         ("expected".to_string(), expected.to_string()),
//                         ("found".to_string(), start.to_string()),
//                     ]),
//                     span: number_token.span().clone(),
//                     related_span: None,
//                     token_id: number_token.id().map(ToOwned::to_owned),
//                     related_token_id: None,
//                     sid: number_token.sid().map(ToOwned::to_owned),
//                     fix: None,
//                 });
//             }
//         }

//         if enabled.has(LintCode::VerseTextFollowsVerseRange)
//             && !verse_has_text_or_note(tokens, number_index + 1)
//         {
//             issues.push(LintIssue {
//                 code: LintCode::VerseTextFollowsVerseRange,
//                 severity: default_severity(LintCode::VerseTextFollowsVerseRange),
//                 marker: Some("v".to_string()),
//                 message: "expected verse content after \\v".to_string(),
//                 message_params: MessageParams::new(),
//                 span: number_token.span().clone(),
//                 related_span: None,
//                 token_id: number_token.id().map(ToOwned::to_owned),
//                 related_token_id: None,
//                 sid: number_token.sid().map(ToOwned::to_owned),
//                 fix: None,
//             });
//         }

//         for verse in start..=end {
//             chapter_state
//                 .seen
//                 .insert(verse, number_token.span().clone());
//         }
//         chapter_state.last = end;
//     }
// }

// #[derive(Default)]
// struct VerseState {
//     seen: BTreeMap<u32, Span>,
//     last: u32,
// }

// fn lint_number_predecessor<T: LintableToken>(
//     tokens: &[T],
//     index: usize,
//     issues: &mut Vec<LintIssue>,
// ) {
//     let token = &tokens[index];
//     let Some(prev_index) = previous_significant_token_index(tokens, index) else {
//         issues.push(simple_issue(
//             LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
//             "number range is not preceded by a marker that expects a number".to_string(),
//             token,
//         ));
//         return;
//     };

//     let prev = &tokens[prev_index];
//     let valid = prev.kind() == &TokenKind::Marker
//         && matches!(
//             prev.marker().map(normalized_marker_name),
//             Some("v" | "vp" | "va" | "c" | "ca" | "cp")
//         );
//     if valid {
//         return;
//     }

//     issues.push(simple_issue(
//         LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
//         "number range is not preceded by a marker that expects a number".to_string(),
//         token,
//     ));
// }

// fn simple_issue<T: LintableToken>(code: LintCode, message: String, token: &T) -> LintIssue {
//     LintIssue {
//         code,
//         severity: default_severity(code),
//         marker: token
//             .marker()
//             .map(normalized_marker_name)
//             .map(ToOwned::to_owned),
//         message,
//         message_params: MessageParams::new(),
//         span: token.span().clone(),
//         related_span: None,
//         token_id: token.id().map(ToOwned::to_owned),
//         related_token_id: None,
//         sid: token.sid().map(ToOwned::to_owned),
//         fix: None,
//     }
// }

// fn build_set_number_fix<T: LintableToken>(token: &T, value: u32) -> Option<TokenFix> {
//     let id = token.id()?;
//     Some(TokenFix::ReplaceToken {
//         code: "set-number".to_string(),
//         label: format!("change number to {value}"),
//         label_params: MessageParams::from([("value".to_string(), value.to_string())]),
//         target_token_id: id.to_string(),
//         replacements: vec![TokenTemplate {
//             kind: TokenKind::Number,
//             text: value.to_string(),
//             marker: None,
//             sid: token.sid().map(ToOwned::to_owned),
//         }],
//     })
// }

// fn starts_with_horizontal_whitespace(text: &str) -> bool {
//     matches!(text.chars().next(), Some(' ' | '\t'))
// }

// fn ends_with_horizontal_whitespace(text: &str) -> bool {
//     matches!(text.chars().next_back(), Some(' ' | '\t'))
// }

// fn strip_digits(text: &str) -> &str {
//     let first_digit = text
//         .find(|ch: char| ch.is_ascii_digit())
//         .unwrap_or(text.len());
//     &text[..first_digit]
// }

// fn is_body_paragraph_marker(marker: &str) -> bool {
//     matches!(
//         marker,
//         "p" | "m"
//             | "po"
//             | "pr"
//             | "cls"
//             | "pmo"
//             | "pm"
//             | "pmc"
//             | "pmr"
//             | "pi"
//             | "pi1"
//             | "pi2"
//             | "pi3"
//             | "mi"
//             | "nb"
//             | "pc"
//             | "ph"
//             | "ph1"
//             | "ph2"
//             | "ph3"
//             | "b"
//             | "pb"
//             | "q"
//             | "q1"
//             | "q2"
//             | "q3"
//             | "q4"
//             | "qr"
//             | "qc"
//             | "qa"
//             | "qm"
//             | "qm1"
//             | "qm2"
//             | "qm3"
//             | "qd"
//             | "lh"
//             | "li"
//             | "li1"
//             | "li2"
//             | "li3"
//             | "li4"
//             | "lf"
//             | "lim"
//             | "lim1"
//             | "lim2"
//             | "lim3"
//     )
// }

// fn marker_is_intentionally_empty_block(marker: &str) -> bool {
//     matches!(marker, "b")
// }

// fn empty_paragraph_boundary_index<T: LintableToken>(
//     tokens: &[T],
//     marker_index: usize,
// ) -> Option<usize> {
//     let mut index = marker_index + 1;
//     while index < tokens.len() {
//         let token = &tokens[index];
//         match token.kind() {
//             TokenKind::Newline | TokenKind::OptBreak => {
//                 index += 1;
//                 continue;
//             }
//             TokenKind::Text if token.text().trim().is_empty() => {
//                 index += 1;
//                 continue;
//             }
//             TokenKind::Marker => {
//                 let marker = token.marker().map(normalized_marker_name)?;
//                 return empty_paragraph_boundary_marker(marker).then_some(index);
//             }
//             _ => return None,
//         }
//     }
//     None
// }

// fn empty_paragraph_boundary_marker(marker: &str) -> bool {
//     if is_body_paragraph_marker(marker) {
//         return true;
//     }
//     matches!(
//         lookup_marker(marker).kind,
//         MarkerKind::Header
//             | MarkerKind::Chapter
//             | MarkerKind::Periph
//             | MarkerKind::SidebarStart
//             | MarkerKind::TableRow
//             | MarkerKind::Unknown
//     )
// }

// fn parse_primary_number(text: &str) -> Option<u32> {
//     let digits = text
//         .trim()
//         .split(['-', ','])
//         .next()
//         .unwrap_or("")
//         .trim_matches(|ch: char| !ch.is_ascii_digit());
//     digits.parse().ok()
// }

// fn parse_number_range(text: &str) -> Option<(u32, u32)> {
//     let trimmed = text.trim();
//     if trimmed.is_empty() {
//         return None;
//     }
//     let mut parts = trimmed.split('-');
//     let start = parts.next()?.parse::<u32>().ok()?;
//     let end = match parts.next() {
//         Some(value) => value.parse::<u32>().ok()?,
//         None => start,
//     };
//     if parts.next().is_some() || start == 0 || end == 0 || start > end {
//         return None;
//     }
//     Some((start, end))
// }

// fn parse_sid_chapter(sid: Option<&str>) -> Option<u32> {
//     let sid = sid?;
//     let reference = sid.split("_dup_").next().unwrap_or(sid);
//     let (_, chap_and_verse) = reference.rsplit_once(' ')?;
//     let (chapter, _) = chap_and_verse.split_once(':')?;
//     chapter.parse().ok()
// }

// fn lint_unknown_token_like<T: LintableToken>(token: &T) -> Option<LintIssue> {
//     let text = token.text();
//     let slash_index = text.find('\\')?;
//     let remainder = &text[slash_index + 1..];
//     let marker_len = remainder
//         .chars()
//         .take_while(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || *ch == '-')
//         .map(char::len_utf8)
//         .sum::<usize>();
//     if marker_len == 0 {
//         return None;
//     }
//     let marker = &remainder[..marker_len];
//     if lookup_marker(marker).kind == MarkerKind::Unknown {
//         return None;
//     }
//     let after = &remainder[marker_len..];
//     if after.is_empty() || after.starts_with(char::is_whitespace) {
//         return None;
//     }

//     let target_id = token.id()?.to_string();
//     let text_after = after.to_string();
//     Some(LintIssue {
//         code: LintCode::UnknownToken,
//         severity: default_severity(LintCode::UnknownToken),
//         marker: Some(marker.to_string()),
//         message: format!("unknown token {}", token.text()),
//         message_params: MessageParams::from([
//             ("marker".to_string(), marker.to_string()),
//             ("text".to_string(), token.text().to_string()),
//         ]),
//         span: token.span().clone(),
//         related_span: None,
//         token_id: Some(target_id.clone()),
//         related_token_id: None,
//         sid: token.sid().map(ToOwned::to_owned),
//         fix: Some(TokenFix::ReplaceToken {
//             code: "split-unknown-token".to_string(),
//             label: format!("split into \\{marker} and text"),
//             label_params: MessageParams::from([("marker".to_string(), marker.to_string())]),
//             target_token_id: target_id,
//             replacements: vec![
//                 TokenTemplate {
//                     kind: TokenKind::Marker,
//                     text: format!("\\{marker}"),
//                     marker: Some(marker.to_string()),
//                     sid: token.sid().map(ToOwned::to_owned),
//                 },
//                 TokenTemplate {
//                     kind: TokenKind::Text,
//                     text: text_after,
//                     marker: None,
//                     sid: token.sid().map(ToOwned::to_owned),
//                 },
//             ],
//         }),
//     })
// }

// fn verse_has_text_or_note<T: LintableToken>(tokens: &[T], start: usize) -> bool {
//     let mut index = start;
//     while index < tokens.len() {
//         let token = &tokens[index];
//         match token.kind() {
//             TokenKind::Newline
//             | TokenKind::Attributes
//             | TokenKind::Milestone
//             | TokenKind::MilestoneEnd => {
//                 index += 1;
//                 continue;
//             }
//             TokenKind::Text => {
//                 if token.text().trim().is_empty() {
//                     index += 1;
//                     continue;
//                 }
//                 return true;
//             }
//             TokenKind::Marker => {
//                 if let Some(marker) = token.marker().map(normalized_marker_name) {
//                     match lookup_marker(marker).kind {
//                         MarkerKind::Note | MarkerKind::Figure => return true,
//                         MarkerKind::Character | MarkerKind::Meta => {
//                             index += 1;
//                             continue;
//                         }
//                         MarkerKind::Paragraph
//                         | MarkerKind::Header
//                         | MarkerKind::Chapter
//                         | MarkerKind::Verse
//                         | MarkerKind::SidebarStart
//                         | MarkerKind::SidebarEnd
//                         | MarkerKind::Periph
//                         | MarkerKind::TableRow
//                         | MarkerKind::TableCell => return false,
//                         MarkerKind::MilestoneStart
//                         | MarkerKind::MilestoneEnd
//                         | MarkerKind::Unknown => {
//                             index += 1;
//                             continue;
//                         }
//                     }
//                 }
//                 return false;
//             }
//             TokenKind::EndMarker => {
//                 if let Some(marker) = token.marker().map(normalized_marker_name) {
//                     match lookup_marker(marker).kind {
//                         MarkerKind::Character
//                         | MarkerKind::Note
//                         | MarkerKind::Meta
//                         | MarkerKind::Figure
//                         | MarkerKind::MilestoneStart
//                         | MarkerKind::MilestoneEnd
//                         | MarkerKind::Unknown => {
//                             index += 1;
//                             continue;
//                         }
//                         MarkerKind::Paragraph
//                         | MarkerKind::Header
//                         | MarkerKind::Chapter
//                         | MarkerKind::Verse
//                         | MarkerKind::SidebarStart
//                         | MarkerKind::SidebarEnd
//                         | MarkerKind::Periph
//                         | MarkerKind::TableRow
//                         | MarkerKind::TableCell => return false,
//                     }
//                 }
//                 return false;
//             }
//             TokenKind::BookCode | TokenKind::Number | TokenKind::OptBreak => return false,
//         }
//     }
//     false
// }

// fn previous_significant_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
//     if start == 0 {
//         return None;
//     }
//     for index in (0..start).rev() {
//         match tokens[index].kind() {
//             TokenKind::Newline => continue,
//             _ => return Some(index),
//         }
//     }
//     None
// }

// fn next_number_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
//     for (index, token) in tokens.iter().enumerate().skip(start) {
//         match token.kind() {
//             TokenKind::Newline => continue,
//             TokenKind::Number => return Some(index),
//             _ => return None,
//         }
//     }
//     None
// }

// fn next_text_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
//     for (index, token) in tokens.iter().enumerate().skip(start) {
//         match token.kind() {
//             TokenKind::Newline => continue,
//             TokenKind::Text => return Some(index),
//             _ => return None,
//         }
//     }
//     None
// }

// fn next_significant_token_index<T: LintableToken>(tokens: &[T], start: usize) -> Option<usize> {
//     for (index, token) in tokens.iter().enumerate().skip(start) {
//         match token.kind() {
//             TokenKind::Newline => continue,
//             _ => return Some(index),
//         }
//     }
//     None
// }

// fn lint_marker_context_rules<T: LintableToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
//     let mut document_state = DocumentLintState::default();

//     for (index, token) in tokens.iter().enumerate() {
//         let marker = match token.kind() {
//             TokenKind::Marker | TokenKind::Milestone => token.marker().map(normalized_marker_name),
//             TokenKind::EndMarker | TokenKind::MilestoneEnd => {
//                 token.marker().map(normalized_marker_name)
//             }
//             _ => None,
//         };
//         let Some(marker) = marker else {
//             continue;
//         };
//         let resolved = ResolvedContextMarker::new(marker);

//         match token.kind() {
//             TokenKind::EndMarker => {
//                 if resolved.is_some_and(|resolved| resolved.kind == MarkerKind::Note) {
//                     let _ = document_state.note_stack.pop();
//                 }
//                 continue;
//             }
//             TokenKind::MilestoneEnd => continue,
//             _ => {}
//         }

//         let Some(resolved) = resolved else {
//             continue;
//         };

//         let validation_context = if resolved.id() == MARKER_PERIPH {
//             SpecContext::Peripheral
//         } else if resolved.kind == MarkerKind::Chapter {
//             top_level_root_context(document_state.kind, TopLevelSlot::Content)
//         } else if document_state.current_note_context().is_none()
//             && matches!(
//                 resolved.kind,
//                 MarkerKind::Paragraph
//                     | MarkerKind::Header
//                     | MarkerKind::SidebarStart
//                     | MarkerKind::TableRow
//             )
//         {
//             let next_slot = document_state.select_top_level_slot(resolved);
//             top_level_root_context(document_state.kind, next_slot)
//         } else {
//             document_state.current_validation_context(resolved)
//         };
//         validate_context_marker_for_token(resolved, validation_context, token, issues);
//         document_state.apply_marker(tokens, index, resolved);
//     }
// }

// #[derive(Clone)]
// struct OpenMarkerFrame {
//     marker: String,
//     span: Span,
//     token_id: Option<String>,
//     sid: Option<String>,
//     kind: MarkerKind,
// }

// fn lint_marker_balance_rules<T: LintableToken>(
//     tokens: &[T],
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     let mut stack: Vec<OpenMarkerFrame> = Vec::new();

//     for token in tokens {
//         let kind = token.kind().clone();
//         let marker = match kind {
//             TokenKind::Marker | TokenKind::EndMarker => token.marker().map(normalized_marker_name),
//             _ => None,
//         };
//         let Some(marker) = marker else {
//             continue;
//         };
//         let marker_kind = lookup_marker(marker).kind;

//         if kind == TokenKind::Marker && closes_inline_stack_at_boundary(marker_kind) {
//             close_open_frames_for_boundary(token, &mut stack, enabled, issues);
//         }

//         match kind {
//             TokenKind::Marker => {
//                 if matches!(
//                     marker_kind,
//                     MarkerKind::Character | MarkerKind::Note | MarkerKind::Meta
//                 ) {
//                     stack.push(OpenMarkerFrame {
//                         marker: marker.to_string(),
//                         span: token.span().clone(),
//                         token_id: token.id().map(ToOwned::to_owned),
//                         sid: token.sid().map(ToOwned::to_owned),
//                         kind: marker_kind,
//                     });
//                 }
//             }
//             TokenKind::EndMarker => {
//                 handle_close_marker(token, marker, &mut stack, enabled, issues);
//             }
//             _ => {}
//         }
//     }

//     if let Some(anchor) = tokens.last() {
//         while let Some(frame) = stack.pop() {
//             issues.push(unclosed_marker_issue(&frame, anchor, true));
//         }
//     }
// }

// fn closes_inline_stack_at_boundary(kind: MarkerKind) -> bool {
//     matches!(
//         kind,
//         MarkerKind::Paragraph
//             | MarkerKind::Header
//             | MarkerKind::Meta
//             | MarkerKind::Chapter
//             | MarkerKind::Periph
//             | MarkerKind::Unknown
//     )
// }

// fn close_open_frames_for_boundary<T: LintableToken>(
//     boundary: &T,
//     stack: &mut Vec<OpenMarkerFrame>,
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     while let Some(frame) = stack.pop() {
//         if frame.kind == MarkerKind::Character && lookup_marker(frame.marker.as_str()).valid_in_note
//         {
//             continue;
//         }
//         let code = match frame.kind {
//             MarkerKind::Note => LintCode::NoteNotClosed,
//             MarkerKind::Character => LintCode::CharNotClosed,
//             _ => continue,
//         };
//         if !enabled.has(code) {
//             continue;
//         }
//         issues.push(unclosed_marker_issue(&frame, boundary, false));
//     }
// }

// fn handle_close_marker<T: LintableToken>(
//     token: &T,
//     marker: &str,
//     stack: &mut Vec<OpenMarkerFrame>,
//     enabled: &EnabledRules,
//     issues: &mut Vec<LintIssue>,
// ) {
//     if stack.is_empty() {
//         if enabled.has(LintCode::StrayCloseMarker) {
//             issues.push(simple_issue(
//                 LintCode::StrayCloseMarker,
//                 format!("closing marker \\{marker}* has no matching opener"),
//                 token,
//             ));
//         }
//         return;
//     }

//     if is_note_close_marker(marker) {
//         while let Some(frame) = stack.last() {
//             if frame.kind == MarkerKind::Character
//                 && lookup_marker(frame.marker.as_str()).valid_in_note
//             {
//                 stack.pop();
//                 continue;
//             }
//             break;
//         }
//     }

//     if stack.last().is_some_and(|frame| frame.marker == marker) {
//         stack.pop();
//         return;
//     }

//     if stack.iter().any(|frame| frame.marker == marker) {
//         if enabled.has(LintCode::MisnestedCloseMarker) {
//             issues.push(simple_issue(
//                 LintCode::MisnestedCloseMarker,
//                 format!("closing marker \\{marker}* mismatches the current open stack"),
//                 token,
//             ));
//         }
//         while let Some(frame) = stack.pop() {
//             if frame.marker == marker {
//                 break;
//             }
//         }
//     } else if enabled.has(LintCode::StrayCloseMarker) {
//         issues.push(simple_issue(
//             LintCode::StrayCloseMarker,
//             format!("closing marker \\{marker}* has no matching opener"),
//             token,
//         ));
//     }
// }

// fn is_note_close_marker(marker: &str) -> bool {
//     marker_is_note_container(marker)
// }

// fn unclosed_marker_issue<T: LintableToken>(
//     frame: &OpenMarkerFrame,
//     anchor: &T,
//     at_eof: bool,
// ) -> LintIssue {
//     let code = match frame.kind {
//         MarkerKind::Note => LintCode::NoteNotClosed,
//         MarkerKind::Character => LintCode::CharNotClosed,
//         _ => LintCode::UnclosedMarkerAtEof,
//     };
//     let fix = anchor.id().map(|target_token_id| TokenFix::ReplaceToken {
//         code: "insert-close-marker".to_string(),
//         label: format!("insert \\{}*", frame.marker),
//         label_params: MessageParams::from([("marker".to_string(), frame.marker.clone())]),
//         target_token_id: target_token_id.to_string(),
//         replacements: vec![
//             TokenTemplate {
//                 kind: TokenKind::EndMarker,
//                 text: format!("\\{}*", frame.marker),
//                 marker: Some(frame.marker.clone()),
//                 sid: frame
//                     .sid
//                     .clone()
//                     .or_else(|| anchor.sid().map(ToOwned::to_owned)),
//             },
//             TokenTemplate {
//                 kind: anchor.kind().clone(),
//                 text: anchor.text().to_string(),
//                 marker: anchor.marker().map(ToOwned::to_owned),
//                 sid: anchor.sid().map(ToOwned::to_owned),
//             },
//         ],
//     });

//     let location = if at_eof {
//         "before end of file"
//     } else {
//         "before the next block boundary"
//     };

//     LintIssue {
//         code,
//         severity: default_severity(code),
//         marker: Some(frame.marker.clone()),
//         message: format!("marker \\{} was not closed {}", frame.marker, location),
//         message_params: MessageParams::from([
//             ("marker".to_string(), frame.marker.clone()),
//             ("location".to_string(), location.to_string()),
//         ]),
//         span: frame.span.clone(),
//         related_span: Some(anchor.span().clone()),
//         token_id: frame.token_id.clone(),
//         related_token_id: anchor.id().map(ToOwned::to_owned),
//         sid: frame
//             .sid
//             .clone()
//             .or_else(|| anchor.sid().map(ToOwned::to_owned)),
//         fix,
//     }
// }

// fn marker_validation_context(
//     marker_kind: MarkerKind,
//     root_context: SpecContext,
//     block_context: Option<SpecContext>,
//     note_context: Option<SpecContext>,
// ) -> SpecContext {
//     let effective = note_context.or(block_context).unwrap_or(root_context);
//     match marker_kind {
//         MarkerKind::Character | MarkerKind::TableCell => effective,
//         MarkerKind::Verse => root_context,
//         MarkerKind::Meta => effective,
//         MarkerKind::Note | MarkerKind::Figure | MarkerKind::Chapter => root_context,
//         MarkerKind::Paragraph
//         | MarkerKind::Header
//         | MarkerKind::SidebarStart
//         | MarkerKind::SidebarEnd
//         | MarkerKind::Periph
//         | MarkerKind::TableRow
//         | MarkerKind::MilestoneStart
//         | MarkerKind::MilestoneEnd
//         | MarkerKind::Unknown => root_context,
//     }
// }

// fn validate_context_marker_for_token<T: LintableToken>(
//     marker: ResolvedContextMarker<'_>,
//     context: SpecContext,
//     token: &T,
//     issues: &mut Vec<LintIssue>,
// ) {
//     if marker.allows_effective_context(context) {
//         return;
//     }

//     issues.push(LintIssue {
//         code: LintCode::MarkerNotValidInContext,
//         severity: default_severity(LintCode::MarkerNotValidInContext),
//         marker: Some(marker.marker.to_string()),
//         message: format!(
//             "marker \\{} is not valid in {}",
//             marker.marker,
//             spec_context_name(context)
//         ),
//         message_params: MessageParams::from([
//             ("marker".to_string(), marker.marker.to_string()),
//             (
//                 "context".to_string(),
//                 spec_context_name(context).to_string(),
//             ),
//         ]),
//         span: token.span().clone(),
//         related_span: None,
//         token_id: token.id().map(ToOwned::to_owned),
//         related_token_id: None,
//         sid: token.sid().map(ToOwned::to_owned),
//         fix: None,
//     });
// }

// fn marker_spec_allows_effective_context(
//     spec: &MarkerSpec,
//     kind: MarkerKind,
//     context: SpecContext,
// ) -> bool {
//     spec.contexts.contains(&context)
//         || (context == SpecContext::PeripheralContent
//             && spec.contexts.contains(&SpecContext::ChapterContent))
//         || marker_spec_allows_embedded_char_context(spec, kind, context)
// }

// fn marker_spec_allows_embedded_char_context(
//     spec: &MarkerSpec,
//     kind: MarkerKind,
//     context: SpecContext,
// ) -> bool {
//     if !matches!(context, SpecContext::Footnote | SpecContext::CrossReference) {
//         return false;
//     }

//     kind == MarkerKind::Character
//         && spec.contexts.iter().any(|ctx| {
//             matches!(
//                 ctx,
//                 SpecContext::Section | SpecContext::Para | SpecContext::List | SpecContext::Table
//             )
//         })
// }

// fn matches_previous_marker_and_number<T: LintableToken>(
//     tokens: &[T],
//     marker_index: usize,
//     expected_marker: &str,
// ) -> bool {
//     let Some(prev_index) = previous_significant_token_index(tokens, marker_index) else {
//         return false;
//     };
//     let prev = &tokens[prev_index];
//     if prev.kind() != &TokenKind::Number {
//         return false;
//     }
//     let Some(before_number_index) = previous_significant_token_index(tokens, prev_index) else {
//         return false;
//     };
//     let before_number = &tokens[before_number_index];
//     before_number.kind() == &TokenKind::Marker
//         && before_number.marker().map(normalized_marker_name) == Some(expected_marker)
// }

// fn top_level_root_context(kind: DocumentKind, slot: TopLevelSlot) -> SpecContext {
//     DocumentLintState {
//         kind,
//         slot,
//         ..DocumentLintState::default()
//     }
//     .current_root_context()
// }

// fn paragraph_block_context(marker: &str) -> SpecContext {
//     paragraph_block_context_from_inline(marker_inline_context(marker))
// }

// fn paragraph_block_context_from_inline(inline_context: Option<InlineContext>) -> SpecContext {
//     match inline_context.unwrap_or(InlineContext::Para) {
//         InlineContext::Para => SpecContext::Para,
//         InlineContext::Section => SpecContext::Section,
//         InlineContext::List => SpecContext::List,
//         InlineContext::Table => SpecContext::Table,
//     }
// }

// fn note_context_for_marker(marker: &str) -> SpecContext {
//     marker_note_context(marker).unwrap_or(SpecContext::Footnote)
// }

// fn spec_context_name(context: SpecContext) -> &'static str {
//     match context {
//         SpecContext::Scripture => "Scripture",
//         SpecContext::BookIdentification => "BookIdentification",
//         SpecContext::BookHeaders => "BookHeaders",
//         SpecContext::BookTitles => "BookTitles",
//         SpecContext::BookIntroduction => "BookIntroduction",
//         SpecContext::BookIntroductionEndTitles => "BookIntroductionEndTitles",
//         SpecContext::BookChapterLabel => "BookChapterLabel",
//         SpecContext::ChapterContent => "ChapterContent",
//         SpecContext::Peripheral => "Peripheral",
//         SpecContext::PeripheralContent => "PeripheralContent",
//         SpecContext::PeripheralDivision => "PeripheralDivision",
//         SpecContext::Chapter => "Chapter",
//         SpecContext::Verse => "Verse",
//         SpecContext::Section => "Section",
//         SpecContext::Para => "Para",
//         SpecContext::List => "List",
//         SpecContext::Table => "Table",
//         SpecContext::Sidebar => "Sidebar",
//         SpecContext::Footnote => "Footnote",
//         SpecContext::CrossReference => "CrossReference",
//     }
// }

// fn dedupe_and_filter_issues(
//     issues: Vec<LintIssue>,
//     suppressions: &[LintSuppression],
// ) -> Vec<LintIssue> {
//     let suppression_keys = suppressions
//         .iter()
//         .map(|suppression| (suppression.code, suppression.sid.as_str()))
//         .collect::<HashSet<_>>();
//     let mut seen = HashSet::new();
//     let mut deduped = Vec::new();

//     for issue in issues {
//         if issue
//             .sid
//             .as_deref()
//             .is_some_and(|sid| suppression_keys.contains(&(issue.code, sid)))
//         {
//             continue;
//         }

//         let identity = (
//             issue.code,
//             issue.span.start,
//             issue.span.end,
//             issue
//                 .related_span
//                 .as_ref()
//                 .map(|span| (span.start, span.end)),
//             issue.token_id.clone(),
//         );
//         if seen.insert(identity) {
//             deduped.push(issue);
//         }
//     }

//     deduped
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::model::token::{TokenKind, TokenViewOptions, WhitespacePolicy};
//     use crate::parse::parse;

//     #[derive(Debug)]
//     struct EditorToken {
//         token_kind: TokenKind,
//         token_span: Span,
//         token_text: String,
//         token_marker: Option<String>,
//         token_sid: Option<String>,
//         token_id: String,
//         lane: u8,
//     }

//     impl LintableToken for EditorToken {
//         fn kind(&self) -> &TokenKind {
//             &self.token_kind
//         }

//         fn span(&self) -> &Span {
//             &self.token_span
//         }

//         fn text(&self) -> &str {
//             &self.token_text
//         }

//         fn marker(&self) -> Option<&str> {
//             self.token_marker.as_deref()
//         }

//         fn sid(&self) -> Option<&str> {
//             self.token_sid.as_deref()
//         }

//         fn id(&self) -> Option<&str> {
//             Some(&self.token_id)
//         }
//     }

//     fn assert_no_invalid_context_for_markers(source: &str, markers: &[&str]) {
//         let handle = parse(source);
//         let issues = lint(&handle, LintOptions::default());

//         for marker in markers {
//             assert!(
//                 !issues.iter().any(|issue| {
//                     issue.code == LintCode::MarkerNotValidInContext
//                         && issue.message.contains(&format!("\\{marker}"))
//                 }),
//                 "unexpected invalid-context issue for \\{marker}: {issues:?}"
//             );
//         }
//     }

//     #[test]
//     fn missing_separator_rule_can_be_disabled() {
//         let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issues = lint_tokens(
//             &projected,
//             TokenLintOptions {
//                 disabled_rules: vec![LintCode::MissingSeparatorAfterMarker],
//                 suppressions: Vec::new(),
//                 allow_implicit_chapter_content_verse: false,
//             },
//         );

//         assert!(
//             issues
//                 .iter()
//                 .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker)
//         );
//     }

//     #[test]
//     fn missing_separator_rule_emits_fix() {
//         let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::MissingSeparatorAfterMarker
//                 && matches!(issue.fix, Some(TokenFix::ReplaceToken { .. }))
//         }));
//     }

//     #[test]
//     fn lint_issues_default_to_error_severity() {
//         let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         assert!(!issues.is_empty());
//         assert!(
//             issues
//                 .iter()
//                 .all(|issue| issue.severity == LintSeverity::Error)
//         );
//     }

//     #[test]
//     fn missing_separator_rule_allows_separator_on_marker_token() {
//         let handle = parse("\\id REV\n\\c 19\n\\m (for fine linen)\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         assert!(
//             issues
//                 .iter()
//                 .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker)
//         );
//     }

//     #[test]
//     fn verse_continuity_rules_are_reported() {
//         let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 text\n\\v 3 text\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         let issue = issues
//             .iter()
//             .find(|issue| issue.code == LintCode::VerseExpectedIncreaseByOne)
//             .expect("expected verse continuity issue");

//         assert!(issue.fix.is_none());
//     }

//     #[test]
//     fn empty_paragraph_before_poetry_marker_is_reported_as_warning() {
//         let handle = parse("\\id PSA\n\\c 2\n\\m\n\\q\n\\v 1 text\n");
//         let projected = tokens(&handle, TokenViewOptions::default());

//         let issue = lint_tokens(&projected, TokenLintOptions::default())
//             .into_iter()
//             .find(|issue| issue.code == LintCode::EmptyParagraph)
//             .expect("expected empty paragraph issue");

//         assert_eq!(issue.severity, LintSeverity::Warning);
//         assert!(matches!(issue.fix, Some(TokenFix::DeleteToken { .. })));
//     }

//     #[test]
//     fn paragraph_before_verse_is_not_reported_empty() {
//         let handle = parse("\\id MAT\n\\c 1\n\\p\n\\v 1 text\n");
//         let projected = tokens(&handle, TokenViewOptions::default());
//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         assert!(
//             issues
//                 .iter()
//                 .all(|issue| issue.code != LintCode::EmptyParagraph)
//         );
//     }

//     #[test]
//     fn blank_break_marker_is_not_reported_empty() {
//         let handle = parse("\\id PSA\n\\c 2\n\\q text\n\\b\n\\q text\n");
//         let projected = tokens(&handle, TokenViewOptions::default());
//         let issues = lint_tokens(&projected, TokenLintOptions::default());

//         assert!(issues.iter().all(|issue| {
//             !(issue.code == LintCode::EmptyParagraph && issue.marker.as_deref() == Some("b"))
//         }));
//     }

//     #[test]
//     fn suppressions_match_by_sid_and_rule() {
//         let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
//         let projected = tokens(&handle, TokenViewOptions::default());
//         let target_sid = projected
//             .iter()
//             .find(|token| token.kind == TokenKind::Marker && token.marker.as_deref() == Some("m"))
//             .and_then(|token| token.sid.clone())
//             .expect("expected marker sid");

//         let issues = lint_tokens(
//             &projected,
//             TokenLintOptions {
//                 disabled_rules: Vec::new(),
//                 suppressions: vec![LintSuppression {
//                     code: LintCode::MissingSeparatorAfterMarker,
//                     sid: target_sid,
//                 }],
//                 allow_implicit_chapter_content_verse: false,
//             },
//         );

//         assert!(
//             issues
//                 .iter()
//                 .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker)
//         );
//     }

//     #[test]
//     fn lint_accepts_editor_tokens_without_conversion() {
//         let tokens = vec![
//             EditorToken {
//                 token_kind: TokenKind::Marker,
//                 token_span: 0..2,
//                 token_text: "\\m".to_string(),
//                 token_marker: Some("m".to_string()),
//                 token_sid: Some("REV 19:14".to_string()),
//                 token_id: "REV-0".to_string(),
//                 lane: 1,
//             },
//             EditorToken {
//                 token_kind: TokenKind::Text,
//                 token_span: 2..8,
//                 token_text: "(text)".to_string(),
//                 token_marker: None,
//                 token_sid: Some("REV 19:14".to_string()),
//                 token_id: "REV-1".to_string(),
//                 lane: 1,
//             },
//         ];

//         let issues = lint_tokens(&tokens, TokenLintOptions::default());

//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::MissingSeparatorAfterMarker)
//         );
//         assert_eq!(tokens[0].lane, 1);
//     }

//     #[test]
//     fn handle_lint_respects_whitespace_projection() {
//         let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
//         let merged = lint(
//             &handle,
//             LintOptions {
//                 include_parse_recoveries: true,
//                 token_view: TokenViewOptions {
//                     whitespace_policy: WhitespacePolicy::MergeToVisible,
//                 },
//                 token_rules: TokenLintOptions::default(),
//             },
//         );

//         assert!(
//             merged
//                 .iter()
//                 .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker)
//         );
//     }

//     #[test]
//     fn body_paragraph_before_first_chapter_is_reported() {
//         let handle = parse("\\id GEN Test\n\\p\n\\c 1\n\\v 1 text\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::ParagraphBeforeFirstChapter)
//         );
//     }

//     #[test]
//     fn note_submarker_outside_note_is_reported() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\ft outside note\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::NoteSubmarkerOutsideNote)
//         );
//     }

//     #[test]
//     fn unknown_ix_marker_is_reported_as_error() {
//         let handle = parse("\\id GEN\n\\c 1\n\\ix text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::UnknownMarker && issue.severity == LintSeverity::Error
//         }));
//     }

//     #[test]
//     fn undocumented_s5_marker_is_reported_as_error() {
//         let handle = parse("\\id GEN\n\\s5\n\\c 1\n\\p\n\\v 1 text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::UnknownMarker
//                 && issue.severity == LintSeverity::Error
//                 && issue.message.contains("\\s5")
//         }));
//     }

//     #[test]
//     fn duplicate_id_marker_is_reported() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\id EXO Other\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::DuplicateIdMarker)
//         );
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::IdMarkerNotAtFileStart)
//         );
//     }

//     #[test]
//     fn rem_in_chapter_content_reports_invalid_context() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\rem comment\n\\p\n\\v 1 text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext
//                 && issue.severity == LintSeverity::Error
//                 && issue.message.contains("\\rem")
//                 && issue.message.contains("ChapterContent")
//         }));
//     }

//     #[test]
//     fn book_title_marker_after_headers_is_not_reported_invalid() {
//         let handle = parse(
//             "\\id 2JN Test\n\\h 2 John\n\\toc1 The Second Letter of John\n\\toc2 2 John\n\\mt Second John\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext && issue.message.contains("\\mt")
//         }));
//     }

//     #[test]
//     fn intro_end_title_marker_after_introduction_is_not_reported_invalid() {
//         let handle = parse(
//             "\\id GEN Test\n\\mt Genesis\n\\ip Intro paragraph\n\\mt End of intro title\n\\c 1\n\\p\n\\v 1 text\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext && issue.message.contains("\\mt")
//         }));
//     }

//     #[test]
//     fn transition_headers_to_titles_is_valid() {
//         assert_no_invalid_context_for_markers(
//             "\\id JHN Test\n\\h John\n\\toc1 The Gospel of John\n\\toc2 John\n\\mt John\n",
//             &["h", "toc1", "toc2", "mt"],
//         );
//     }

//     #[test]
//     fn transition_titles_to_intro_is_valid() {
//         assert_no_invalid_context_for_markers(
//             "\\id MRK Test\n\\h Mark\n\\mt Mark\n\\imt1 Introduction\n\\is1 Intro Section\n\\ip Intro paragraph\n",
//             &["mt", "imt1", "is1", "ip"],
//         );
//     }

//     #[test]
//     fn transition_intro_to_intro_end_titles_to_content_is_valid() {
//         assert_no_invalid_context_for_markers(
//             "\\id GEN Test\n\\mt Genesis\n\\ip Intro paragraph\n\\mt End Title\n\\c 1\n\\p\n\\v 1 text\n",
//             &["mt", "ip", "c", "p", "v"],
//         );
//     }

//     #[test]
//     fn chapter_label_before_first_chapter_is_not_reported_invalid() {
//         let handle = parse("\\id PSA Test\n\\cl Psalm\n\\c 1\n\\q1\n\\v 1 text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext && issue.message.contains("\\cl")
//         }));
//     }

//     #[test]
//     fn standalone_peripheral_paragraph_before_first_chapter_is_allowed() {
//         let handle = parse("\\id GLO Test\n\\h Glossary\n\\mt Glossary\n\\p Entry text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::ParagraphBeforeFirstChapter
//                 || (issue.code == LintCode::MarkerNotValidInContext
//                     && issue.message.contains("\\p"))
//         }));
//     }

//     #[test]
//     fn divided_peripheral_divisions_reset_top_level_context() {
//         let handle = parse(
//             "\\id FRT Test\n\\periph Foreword|id=\"foreword\"\n\\h Foreword\n\\mt1 Foreword\n\\p Text\n\\periph Contents|id=\"contents\"\n\\h Contents\n\\mt Contents\n\\s Section\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext
//                 && (issue.message.contains("\\h")
//                     || issue.message.contains("\\mt")
//                     || issue.message.contains("\\p")
//                     || issue.message.contains("\\s"))
//         }));
//         assert!(
//             !issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::ParagraphBeforeFirstChapter)
//         );
//     }

//     #[test]
//     fn transition_periph_reset_to_headers_titles_and_content_is_valid() {
//         assert_no_invalid_context_for_markers(
//             "\\id FRT Test\n\\periph Title Page|id=\"title\"\n\\mt1 Holy Bible\n\\periph Foreword|id=\"foreword\"\n\\h Foreword\n\\mt1 Foreword\n\\p Text\n",
//             &["periph", "mt1", "h", "p"],
//         );
//     }

//     #[test]
//     fn content_block_transitions_are_valid_inside_content() {
//         assert_no_invalid_context_for_markers(
//             "\\id GEN Test\n\\c 1\n\\s1 Section\n\\p Paragraph\n\\li1 Item\n\\tr \\tc1 Cell\n\\esb \\cat study\\cat* Sidebar text\\esbe\n",
//             &["c", "s1", "p", "li1", "tr", "tc1", "esb"],
//         );
//     }

//     #[test]
//     fn footnote_submarker_outside_note_reports_invalid_context() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\fr 1.1\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext && issue.message.contains("\\fr")
//         }));
//     }

//     #[test]
//     fn cross_reference_submarker_outside_note_reports_invalid_context() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\xo 1.1\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::MarkerNotValidInContext && issue.message.contains("\\xo")
//         }));
//     }

//     #[test]
//     fn ordinary_char_markup_inside_footnote_is_allowed() {
//         assert_no_invalid_context_for_markers(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 1 text \\f + \\ft note about the \\nd Lord\\nd*\\f*\n",
//             &["f", "ft", "nd"],
//         );
//     }

//     #[test]
//     fn pi1_paragraph_in_chapter_content_is_allowed() {
//         assert_no_invalid_context_for_markers(
//             "\\id GEN Test\n\\c 1\n\\pi1 Indented paragraph\n",
//             &["c", "pi1"],
//         );
//     }

//     #[test]
//     fn verse_after_section_heading_paragraphs_is_allowed() {
//         assert_no_invalid_context_for_markers(
//             "\\id JDG Test\n\\c 13\n\\s1 The Birth of Samson\n\\r (Numbers 6:1-21)\n\\v 1 Again the Israelites did evil.\n",
//             &["c", "s1", "r", "v"],
//         );
//     }

//     #[test]
//     fn verse_after_section_reference_paragraph_is_allowed() {
//         assert_no_invalid_context_for_markers(
//             "\\id ZEC Test\n\\c 12\n\\s1 Jerusalem will be Attacked\n\\r (Zechariah 12:1-9)\n\\v 1 This is the burden of the word of the LORD.\n",
//             &["c", "s1", "r", "v"],
//         );
//     }

//     #[test]
//     fn nested_plus_prefixed_xt_can_close_without_closing_ft_or_fr() {
//         let handle = parse(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 3 And God said, \"Let there be light,\"\\f + \\fr 1:3 \\ft Cited in \\+xt 2 Corinthians 4:6\\+xt*\\f* and there was light.\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             matches!(
//                 issue.code,
//                 LintCode::MisnestedCloseMarker
//                     | LintCode::StrayCloseMarker
//                     | LintCode::CharNotClosed
//                     | LintCode::NoteNotClosed
//                     | LintCode::MarkerNotValidInContext
//             )
//         }));
//     }

//     #[test]
//     fn explicit_fqa_close_inside_footnote_is_not_stray() {
//         let handle = parse(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 26 text\\f + \\ft Some ancient copies have: \\fqa quoted text \\fqa* \\f*\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             matches!(
//                 issue.code,
//                 LintCode::StrayCloseMarker
//                     | LintCode::MisnestedCloseMarker
//                     | LintCode::CharNotClosed
//                     | LintCode::NoteNotClosed
//             )
//         }));
//     }

//     #[test]
//     fn unclosed_note_before_paragraph_boundary_is_fixable() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v 1 text \\f + \\ft note\n\\p\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::NoteNotClosed
//                 && matches!(issue.fix, Some(TokenFix::ReplaceToken { .. }))
//         }));
//     }

//     #[test]
//     fn unclosed_note_before_unknown_marker_boundary_targets_unknown_marker() {
//         let handle = parse("\\id MRK Test\n\\c 15\n\\p\n\\v 28 text \\f + \\ft note\n\\s5\n\\p\n");
//         let issues = lint(&handle, LintOptions::default());

//         let issue = issues
//             .iter()
//             .find(|issue| issue.code == LintCode::NoteNotClosed)
//             .expect("note-not-closed issue");

//         match issue.fix.as_ref() {
//             Some(TokenFix::ReplaceToken { replacements, .. }) => {
//                 assert_eq!(replacements.len(), 2);
//                 assert_eq!(replacements[0].kind, TokenKind::EndMarker);
//                 assert_eq!(replacements[0].text, "\\f*");
//                 assert_eq!(replacements[1].kind, TokenKind::Marker);
//                 assert_eq!(replacements[1].text, "\\s5");
//                 assert_eq!(replacements[1].marker.as_deref(), Some("s5"));
//             }
//             other => panic!("unexpected fix payload: {other:?}"),
//         }
//     }

//     #[test]
//     fn chapter_marker_missing_number_is_reported_with_explicit_rule() {
//         let handle = parse("\\id GEN Test\n\\c\n\\p\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::NumberRangeAfterChapterMarker)
//         );
//     }

//     #[test]
//     fn verse_marker_missing_number_is_reported_with_explicit_rule() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v\n");
//         let issues = lint(&handle, LintOptions::default());
//         assert!(
//             issues
//                 .iter()
//                 .any(|issue| issue.code == LintCode::VerseRangeExpectedAfterVerseMarker)
//         );
//     }

//     #[test]
//     fn verse_outside_explicit_paragraph_is_reported_by_default() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\v 1 text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::VerseOutsideExplicitParagraph
//                 && issue.marker.as_deref() == Some("v")
//         }));
//     }

//     #[test]
//     fn verse_outside_explicit_paragraph_can_be_allowed() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\v 1 text\n");
//         let issues = lint(
//             &handle,
//             LintOptions {
//                 include_parse_recoveries: false,
//                 token_view: TokenViewOptions::default(),
//                 token_rules: TokenLintOptions {
//                     disabled_rules: Vec::new(),
//                     suppressions: Vec::new(),
//                     allow_implicit_chapter_content_verse: true,
//                 },
//             },
//         );

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::VerseOutsideExplicitParagraph
//                 && issue.marker.as_deref() == Some("v")
//         }));
//     }

//     #[test]
//     fn verse_with_leading_alignment_milestones_is_not_reported_empty() {
//         let handle = parse(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 1 \\zaln-s |x-strong=\"H7225\" x-content=\"בְּ⁠רֵאשִׁ֖ית\"\\*\\w In|x-occurrence=\"1\" x-occurrences=\"1\"\\w*\\zaln-e\\*\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::VerseTextFollowsVerseRange
//                 && issue.sid.as_deref() == Some("GEN 1:1")
//         }));
//     }

//     #[test]
//     fn verse_with_only_wrappers_and_no_text_still_reports_empty() {
//         let handle = parse(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 1 \\zaln-s |x-strong=\"H7225\"\\*\\zaln-e\\*\n\\v 2 text\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::VerseTextFollowsVerseRange
//                 && issue.sid.as_deref() == Some("GEN 1:1")
//         }));
//     }

//     #[test]
//     fn verse_with_leading_word_wrapper_before_text_is_not_reported_empty() {
//         let handle = parse(
//             "\\id GEN Test\n\\c 1\n\\p\n\\v 1 \\w Word|x-occurrence=\"1\" x-occurrences=\"1\"\\w*\n",
//         );
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::VerseTextFollowsVerseRange
//                 && issue.sid.as_deref() == Some("GEN 1:1")
//         }));
//     }

//     #[test]
//     fn empty_verse_followed_by_next_block_still_reports_empty() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v 1\n\\p\n\\v 2 text\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(issues.iter().any(|issue| {
//             issue.code == LintCode::VerseTextFollowsVerseRange
//                 && issue.sid.as_deref() == Some("GEN 1:1")
//         }));
//     }

//     #[test]
//     fn verse_with_note_only_still_counts_as_content() {
//         let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v 1 \\f + \\ft note\\f*\n");
//         let issues = lint(&handle, LintOptions::default());

//         assert!(!issues.iter().any(|issue| {
//             issue.code == LintCode::VerseTextFollowsVerseRange
//                 && issue.sid.as_deref() == Some("GEN 1:1")
//         }));
//     }
// }
