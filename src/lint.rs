use crate::handle::{ParseHandle, recoveries, tokens};
use crate::markers::{MarkerKind, lookup_marker};
use crate::recovery::{ParseRecovery, RecoveryCode, RecoveryPayload};
use crate::token::{FlatToken, Span, TokenKind, TokenViewOptions, normalized_marker_name};
use crate::transform::{TokenFix, TokenTemplate};
use std::collections::{BTreeMap, BTreeSet};

pub trait LintableFlatToken {
    fn kind(&self) -> &TokenKind;
    fn span(&self) -> &Span;
    fn text(&self) -> &str;
    fn marker(&self) -> Option<&str>;
    fn sid(&self) -> Option<&str> {
        None
    }
    fn id(&self) -> Option<&str> {
        None
    }
}

impl LintableFlatToken for FlatToken {
    fn kind(&self) -> &TokenKind {
        &self.kind
    }

    fn span(&self) -> &Span {
        &self.span
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn sid(&self) -> Option<&str> {
        self.sid.as_deref()
    }

    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LintCode {
    MissingSeparatorAfterMarker,
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
}

impl LintCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingSeparatorAfterMarker => "missing-separator-after-marker",
            Self::NumberRangeAfterChapterMarker => "number-range-after-chapter-marker",
            Self::VerseRangeExpectedAfterVerseMarker => "verse-range-expected-after-verse-marker",
            Self::VerseContentNotEmpty => "verse-content-not-empty",
            Self::UnknownToken => "unknown-token",
            Self::CharNotClosed => "char-not-closed",
            Self::NoteNotClosed => "note-not-closed",
            Self::ParagraphBeforeFirstChapter => "paragraph-before-first-chapter",
            Self::VerseBeforeFirstChapter => "verse-before-first-chapter",
            Self::NoteSubmarkerOutsideNote => "note-submarker-outside-note",
            Self::DuplicateIdMarker => "duplicate-id-marker",
            Self::IdMarkerNotAtFileStart => "id-marker-not-at-file-start",
            Self::ChapterMetadataOutsideChapter => "chapter-metadata-outside-chapter",
            Self::VerseMetadataOutsideVerse => "verse-metadata-outside-verse",
            Self::MissingChapterNumber => "missing-chapter-number",
            Self::MissingVerseNumber => "missing-verse-number",
            Self::MissingMilestoneSelfClose => "missing-milestone-self-close",
            Self::ImplicitlyClosedMarker => "implicitly-closed-marker",
            Self::StrayCloseMarker => "stray-close-marker",
            Self::MisnestedCloseMarker => "misnested-close-marker",
            Self::UnclosedNote => "unclosed-note",
            Self::UnclosedMarkerAtEof => "unclosed-marker-at-eof",
            Self::DuplicateChapterNumber => "duplicate-chapter-number",
            Self::ChapterExpectedIncreaseByOne => "chapter-expected-increase-by-one",
            Self::DuplicateVerseNumber => "duplicate-verse-number",
            Self::VerseExpectedIncreaseByOne => "verse-expected-increase-by-one",
            Self::InvalidNumberRange => "invalid-number-range",
            Self::NumberRangeNotPrecededByMarkerExpectingNumber => {
                "number-range-not-preceded-by-marker-expecting-number"
            }
            Self::VerseTextFollowsVerseRange => "verse-text-follows-verse-range",
            Self::UnknownMarker => "unknown-marker",
            Self::UnknownCloseMarker => "unknown-close-marker",
            Self::InconsistentChapterLabel => "inconsistent-chapter-label",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintIssue {
    pub code: LintCode,
    pub message: String,
    pub span: Span,
    pub related_span: Option<Span>,
    pub token_id: Option<String>,
    pub related_token_id: Option<String>,
    pub sid: Option<String>,
    pub fix: Option<TokenFix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintSuppression {
    pub code: LintCode,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintOptions {
    pub include_parse_recoveries: bool,
    pub token_view: TokenViewOptions,
    pub token_rules: TokenLintOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenLintOptions {
    pub disabled_rules: Vec<LintCode>,
    pub suppressions: Vec<LintSuppression>,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            include_parse_recoveries: true,
            token_view: TokenViewOptions::default(),
            token_rules: TokenLintOptions::default(),
        }
    }
}

pub fn lint(handle: &ParseHandle, options: LintOptions) -> Vec<LintIssue> {
    let projected = tokens(handle, options.token_view);
    let mut issues = lint_tokens(&projected, options.token_rules.clone());

    if options.include_parse_recoveries {
        for recovery in recoveries(handle) {
            if let Some(issue) = lint_issue_from_recovery(&projected, recovery) {
                issues.push(issue);
            }
        }
    }

    dedupe_and_filter_issues(issues, &options.token_rules.suppressions)
}

pub fn lint_tokens<T: LintableFlatToken>(tokens: &[T], options: TokenLintOptions) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let enabled = EnabledRules::new(&options.disabled_rules);

    if enabled.has(LintCode::MissingSeparatorAfterMarker) {
        lint_missing_separator_after_marker(tokens, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::UnknownToken,
        LintCode::NumberRangeAfterChapterMarker,
        LintCode::VerseRangeExpectedAfterVerseMarker,
        LintCode::VerseContentNotEmpty,
    ]) {
        lint_expectation_and_unknown_token_rules(tokens, &enabled, &mut issues);
    }
    if enabled.has_any(&[
        LintCode::ParagraphBeforeFirstChapter,
        LintCode::VerseBeforeFirstChapter,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::DuplicateIdMarker,
        LintCode::IdMarkerNotAtFileStart,
        LintCode::ChapterMetadataOutsideChapter,
        LintCode::VerseMetadataOutsideVerse,
    ]) {
        lint_structure_rules(tokens, &enabled, &mut issues);
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

    dedupe_and_filter_issues(issues, &options.suppressions)
}

fn lint_expectation_and_unknown_token_rules<T: LintableFlatToken>(
    tokens: &[T],
    enabled: &EnabledRules,
    issues: &mut Vec<LintIssue>,
) {
    for index in 0..tokens.len() {
        let token = &tokens[index];

        if enabled.has(LintCode::UnknownToken)
            && token.kind() == &TokenKind::Text
            && let Some(issue) = lint_unknown_token_like(token)
        {
            issues.push(issue);
        }

        if token.kind() != &TokenKind::Marker {
            continue;
        }

        let marker = token.marker().map(normalized_marker_name).unwrap_or_default();
        match marker {
            "c" if enabled.has(LintCode::NumberRangeAfterChapterMarker) => {
                if next_number_token_index(tokens, index + 1).is_none() {
                    issues.push(simple_issue(
                        LintCode::NumberRangeAfterChapterMarker,
                        "number range expected after \\c".to_string(),
                        token,
                    ));
                }
            }
            "v" => {
                if enabled.has(LintCode::VerseRangeExpectedAfterVerseMarker)
                    && next_number_token_index(tokens, index + 1).is_none()
                {
                    issues.push(simple_issue(
                        LintCode::VerseRangeExpectedAfterVerseMarker,
                        "verse number expected after \\v".to_string(),
                        token,
                    ));
                }

                if enabled.has(LintCode::VerseContentNotEmpty)
                    && let Some(next_index) = next_significant_token_index(tokens, index + 1)
                    && tokens[next_index].kind() == &TokenKind::Text
                    && tokens[next_index].text().trim().is_empty()
                {
                    issues.push(LintIssue {
                        code: LintCode::VerseContentNotEmpty,
                        message: "verse content expected after \\v".to_string(),
                        span: tokens[next_index].span().clone(),
                        related_span: Some(token.span().clone()),
                        token_id: tokens[next_index].id().map(ToOwned::to_owned),
                        related_token_id: token.id().map(ToOwned::to_owned),
                        sid: token.sid().map(ToOwned::to_owned),
                        fix: None,
                    });
                }
            }
            _ => {}
        }
    }
}

fn lint_structure_rules<T: LintableFlatToken>(
    tokens: &[T],
    enabled: &EnabledRules,
    issues: &mut Vec<LintIssue>,
) {
    let mut saw_chapter = false;
    let mut saw_content = false;
    let mut id_seen = false;
    let mut note_stack: Vec<String> = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            _ => {}
        }

        if token.kind() == &TokenKind::Marker {
            let marker = token.marker().map(normalized_marker_name).unwrap_or_default();
            let info = lookup_marker(marker);

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

            if marker == "c" {
                saw_chapter = true;
            }

            if enabled.has(LintCode::ParagraphBeforeFirstChapter)
                && !saw_chapter
                && info.kind == MarkerKind::Paragraph
                && is_body_paragraph_marker(marker)
            {
                issues.push(simple_issue(
                    LintCode::ParagraphBeforeFirstChapter,
                    format!("body paragraph marker \\{marker} appears before the first chapter"),
                    token,
                ));
            }

            if enabled.has(LintCode::VerseBeforeFirstChapter) && !saw_chapter && marker == "v" {
                issues.push(simple_issue(
                    LintCode::VerseBeforeFirstChapter,
                    "verse marker appears before the first chapter".to_string(),
                    token,
                ));
            }

            if enabled.has(LintCode::NoteSubmarkerOutsideNote)
                && info.valid_in_note
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

            if info.kind == MarkerKind::Note {
                note_stack.push(marker.to_string());
            }

            saw_content = true;
            continue;
        }

        if token.kind() == &TokenKind::EndMarker {
            if let Some(marker) = token.marker().map(normalized_marker_name)
                && lookup_marker(marker).kind == MarkerKind::Note
            {
                if let Some(idx) = note_stack.iter().rposition(|open| open == marker) {
                    note_stack.truncate(idx);
                } else {
                    note_stack.clear();
                }
            }
            saw_content = true;
            continue;
        }

        if token.kind() != &TokenKind::BookCode {
            saw_content = true;
        }
    }
}

#[derive(Default)]
struct EnabledRules {
    disabled: BTreeSet<LintCode>,
}

impl EnabledRules {
    fn new(disabled_rules: &[LintCode]) -> Self {
        Self {
            disabled: disabled_rules.iter().copied().collect(),
        }
    }

    fn has(&self, code: LintCode) -> bool {
        !self.disabled.contains(&code)
    }

    fn has_any(&self, codes: &[LintCode]) -> bool {
        codes.iter().copied().any(|code| self.has(code))
    }
}

fn lint_missing_separator_after_marker<T: LintableFlatToken>(
    tokens: &[T],
    issues: &mut Vec<LintIssue>,
) {
    for window in tokens.windows(2) {
        let [current, next] = window else {
            continue;
        };

        if current.kind() != &TokenKind::Marker || next.kind() != &TokenKind::Text {
            continue;
        }

        let Some(marker) = current.marker() else {
            continue;
        };
        let marker = normalized_marker_name(marker);
        let marker_kind = lookup_marker(marker).kind;
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

        issues.push(LintIssue {
            code: LintCode::MissingSeparatorAfterMarker,
            message: format!("marker \\{marker} is immediately followed by text"),
            span: current.span().clone(),
            related_span: Some(next.span().clone()),
            token_id: current.id().map(ToOwned::to_owned),
            related_token_id: next.id().map(ToOwned::to_owned),
            sid: current.sid().map(ToOwned::to_owned),
            fix: next.id().map(|id| TokenFix::ReplaceToken {
                label: format!("insert separator after \\{marker}"),
                target_token_id: id.to_string(),
                replacements: vec![TokenTemplate {
                    kind: TokenKind::Text,
                    text: format!(" {}", next.text()),
                    marker: None,
                    sid: current.sid().map(ToOwned::to_owned),
                }],
            }),
        });
    }
}

fn lint_unknown_markers<T: LintableFlatToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for token in tokens {
        if token.kind() != &TokenKind::Marker {
            continue;
        }
        let Some(marker) = token.marker().map(normalized_marker_name) else {
            continue;
        };
        if lookup_marker(marker).kind != MarkerKind::Unknown {
            continue;
        }
        issues.push(simple_issue(
            LintCode::UnknownMarker,
            format!("unknown marker \\{marker}"),
            token,
        ));
    }
}

fn lint_unknown_close_markers<T: LintableFlatToken>(tokens: &[T], issues: &mut Vec<LintIssue>) {
    for token in tokens {
        if token.kind() != &TokenKind::EndMarker {
            continue;
        }
        let Some(marker) = token.marker().map(normalized_marker_name) else {
            continue;
        };
        if lookup_marker(marker).kind != MarkerKind::Unknown {
            continue;
        }
        issues.push(simple_issue(
            LintCode::UnknownCloseMarker,
            format!("unknown closing marker \\{marker}*"),
            token,
        ));
    }
}

fn lint_chapter_rules<T: LintableFlatToken>(
    tokens: &[T],
    enabled: &EnabledRules,
    issues: &mut Vec<LintIssue>,
) {
    let mut seen_chapters = BTreeSet::new();
    let mut last_chapter: Option<u32> = None;
    let mut labels: BTreeMap<String, Vec<(Span, Option<String>, Option<String>)>> = BTreeMap::new();

    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];

        if token.kind() == &TokenKind::Marker
            && token.marker().map(normalized_marker_name) == Some("c")
            && let Some(number_index) = next_number_token_index(tokens, index + 1)
            && let Some(chapter) = parse_primary_number(tokens[number_index].text())
        {
            if enabled.has(LintCode::DuplicateChapterNumber) && seen_chapters.contains(&chapter) {
                issues.push(LintIssue {
                    code: LintCode::DuplicateChapterNumber,
                    message: format!("duplicate chapter number {chapter}"),
                    span: tokens[number_index].span().clone(),
                    related_span: None,
                    token_id: tokens[number_index].id().map(ToOwned::to_owned),
                    related_token_id: None,
                    sid: token.sid().map(ToOwned::to_owned),
                    fix: None,
                });
            }

            if enabled.has(LintCode::ChapterExpectedIncreaseByOne) {
                let expected = last_chapter.map_or(1, |last| last + 1);
                if chapter != expected {
                    issues.push(LintIssue {
                        code: LintCode::ChapterExpectedIncreaseByOne,
                        message: format!("expected chapter number {expected}, found {chapter}"),
                        span: tokens[number_index].span().clone(),
                        related_span: None,
                        token_id: tokens[number_index].id().map(ToOwned::to_owned),
                        related_token_id: None,
                        sid: token.sid().map(ToOwned::to_owned),
                        fix: None,
                    });
                }
            }

            seen_chapters.insert(chapter);
            last_chapter = Some(chapter);
        }

        if enabled.has(LintCode::InconsistentChapterLabel)
            && token.kind() == &TokenKind::Marker
            && token.marker().map(normalized_marker_name) == Some("cl")
            && let Some(text_index) = next_text_token_index(tokens, index + 1)
        {
            let label = strip_digits(tokens[text_index].text().trim()).trim().to_string();
            if !label.is_empty() {
                labels.entry(label).or_default().push((
                    tokens[text_index].span().clone(),
                    tokens[text_index].id().map(ToOwned::to_owned),
                    tokens[text_index].sid().map(ToOwned::to_owned),
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
                        message: format!(
                            "inconsistent chapter label '{label}', expected the canonical label '{canonical}'"
                        ),
                        span,
                        related_span: None,
                        token_id,
                        related_token_id: None,
                        sid,
                        fix: None,
                    });
                }
            }
        }
    }
}

fn lint_number_and_verse_rules<T: LintableFlatToken>(
    tokens: &[T],
    enabled: &EnabledRules,
    issues: &mut Vec<LintIssue>,
) {
    let mut current_chapter = 0u32;
    let mut verse_state_by_chapter: BTreeMap<u32, VerseState> = BTreeMap::new();

    for index in 0..tokens.len() {
        let token = &tokens[index];

        if token.kind() == &TokenKind::Marker
            && token.marker().map(normalized_marker_name) == Some("c")
            && let Some(number_index) = next_number_token_index(tokens, index + 1)
            && let Some(chapter) = parse_primary_number(tokens[number_index].text())
        {
            current_chapter = chapter;
        }

        if enabled.has(LintCode::NumberRangeNotPrecededByMarkerExpectingNumber)
            && token.kind() == &TokenKind::Number
        {
            lint_number_predecessor(tokens, index, issues);
        }

        if token.kind() != &TokenKind::Marker || token.marker().map(normalized_marker_name) != Some("v") {
            continue;
        }

        let Some(number_index) = next_number_token_index(tokens, index + 1) else {
            continue;
        };
        let number_token = &tokens[number_index];
        let value = number_token.text().trim();

        if enabled.has(LintCode::InvalidNumberRange) && parse_number_range(value).is_none() {
            issues.push(LintIssue {
                code: LintCode::InvalidNumberRange,
                message: format!("invalid verse range {value}"),
                span: number_token.span().clone(),
                related_span: None,
                token_id: number_token.id().map(ToOwned::to_owned),
                related_token_id: None,
                sid: number_token.sid().map(ToOwned::to_owned),
                fix: None,
            });
            continue;
        }

        let Some((start, end)) = parse_number_range(value) else {
            continue;
        };

        let chapter = if current_chapter == 0 {
            parse_sid_chapter(number_token.sid()).unwrap_or(0)
        } else {
            current_chapter
        };
        let chapter_state = verse_state_by_chapter.entry(chapter).or_default();

        let mut duplicate = false;
        for verse in start..=end {
            if chapter_state.seen.contains_key(&verse) {
                duplicate = true;
                break;
            }
        }

        if enabled.has(LintCode::DuplicateVerseNumber) && duplicate {
            issues.push(LintIssue {
                code: LintCode::DuplicateVerseNumber,
                message: format!("duplicate verse number {value}"),
                span: number_token.span().clone(),
                related_span: None,
                token_id: number_token.id().map(ToOwned::to_owned),
                related_token_id: None,
                sid: number_token.sid().map(ToOwned::to_owned),
                fix: build_set_number_fix(number_token, chapter_state.last + 1),
            });
        } else if enabled.has(LintCode::VerseExpectedIncreaseByOne) {
            let expected = chapter_state.last + 1;
            if start != expected {
                issues.push(LintIssue {
                    code: LintCode::VerseExpectedIncreaseByOne,
                    message: if chapter_state.last > 0 {
                        format!(
                            "previous verse number was {}, so expected {} here, found {}",
                            chapter_state.last, expected, start
                        )
                    } else {
                        format!("expected verse {expected} here, found {start}")
                    },
                    span: number_token.span().clone(),
                    related_span: None,
                    token_id: number_token.id().map(ToOwned::to_owned),
                    related_token_id: None,
                    sid: number_token.sid().map(ToOwned::to_owned),
                    fix: build_set_number_fix(number_token, expected),
                });
            }
        }

        if enabled.has(LintCode::VerseTextFollowsVerseRange)
            && !verse_has_text_or_note(tokens, number_index + 1)
        {
            issues.push(LintIssue {
                code: LintCode::VerseTextFollowsVerseRange,
                message: "expected verse content after \\v".to_string(),
                span: number_token.span().clone(),
                related_span: None,
                token_id: number_token.id().map(ToOwned::to_owned),
                related_token_id: None,
                sid: number_token.sid().map(ToOwned::to_owned),
                fix: None,
            });
        }

        for verse in start..=end {
            chapter_state.seen.insert(verse, number_token.span().clone());
        }
        chapter_state.last = end;
    }
}

#[derive(Default)]
struct VerseState {
    seen: BTreeMap<u32, Span>,
    last: u32,
}

fn lint_number_predecessor<T: LintableFlatToken>(tokens: &[T], index: usize, issues: &mut Vec<LintIssue>) {
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
    let valid = prev.kind() == &TokenKind::Marker
        && matches!(
            prev.marker().map(normalized_marker_name),
            Some("v" | "vp" | "va" | "c" | "ca" | "cp")
        );
    if valid {
        return;
    }

    issues.push(simple_issue(
        LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        "number range is not preceded by a marker that expects a number".to_string(),
        token,
    ));
}

fn lint_issue_from_recovery<T: LintableFlatToken>(
    projected: &[T],
    recovery: &ParseRecovery,
) -> Option<LintIssue> {
    let code = lint_code_from_recovery(recovery.code.clone(), recovery.payload.as_ref());
    let token_id = find_token_id_for_span(projected, &recovery.span);
    let related_token_id = recovery
        .related_span
        .as_ref()
        .and_then(|span| find_token_id_for_span(projected, span));

        let fix = match (&recovery.code, &recovery.payload) {
        (RecoveryCode::UnclosedNote | RecoveryCode::UnclosedMarkerAtEof, Some(RecoveryPayload::Marker { marker })) => {
            if let Some(related_span) = recovery.related_span.as_ref()
                && let Some(target_token_id) = find_token_id_for_span(projected, related_span)
                && let Some(anchor) = projected.iter().find(|token| token.id() == Some(target_token_id.as_str()))
            {
                Some(TokenFix::ReplaceToken {
                    label: format!("insert \\{marker}*"),
                    target_token_id,
                    replacements: vec![
                        TokenTemplate {
                            kind: TokenKind::EndMarker,
                            text: format!("\\{marker}*"),
                            marker: Some(marker.clone()),
                            sid: anchor.sid().map(ToOwned::to_owned),
                        },
                        TokenTemplate {
                            kind: anchor.kind().clone(),
                            text: anchor.text().to_string(),
                            marker: anchor.marker().map(ToOwned::to_owned),
                            sid: anchor.sid().map(ToOwned::to_owned),
                        },
                    ],
                })
            } else {
                let last_token_id = projected.last().and_then(|token| token.id()).map(ToOwned::to_owned)?;
                Some(TokenFix::InsertAfter {
                    label: format!("insert \\{marker}*"),
                    target_token_id: last_token_id,
                    insert: vec![TokenTemplate {
                        kind: TokenKind::EndMarker,
                        text: format!("\\{marker}*"),
                        marker: Some(marker.clone()),
                        sid: projected.last().and_then(|token| token.sid()).map(ToOwned::to_owned),
                    }],
                })
            }
        }
        _ => None,
    };

    Some(LintIssue {
        code,
        message: format_recovery_message(recovery.code.clone()),
        span: recovery.span.clone(),
        related_span: recovery.related_span.clone(),
        token_id,
        related_token_id,
        sid: find_sid_for_span(projected, &recovery.span),
        fix,
    })
}

fn lint_code_from_recovery(code: RecoveryCode, payload: Option<&RecoveryPayload>) -> LintCode {
    match code {
        RecoveryCode::MissingChapterNumber => LintCode::MissingChapterNumber,
        RecoveryCode::MissingVerseNumber => LintCode::MissingVerseNumber,
        RecoveryCode::MissingMilestoneSelfClose => LintCode::MissingMilestoneSelfClose,
        RecoveryCode::ImplicitlyClosedMarker => {
            if let Some(RecoveryPayload::Marker { marker }) = payload {
                if lookup_marker(normalized_marker_name(marker)).kind == MarkerKind::Note {
                    LintCode::NoteNotClosed
                } else {
                    LintCode::CharNotClosed
                }
            } else {
                LintCode::ImplicitlyClosedMarker
            }
        }
        RecoveryCode::StrayCloseMarker => LintCode::StrayCloseMarker,
        RecoveryCode::MisnestedCloseMarker => LintCode::MisnestedCloseMarker,
        RecoveryCode::UnclosedNote => LintCode::NoteNotClosed,
        RecoveryCode::UnclosedMarkerAtEof => {
            if let Some(RecoveryPayload::Marker { marker }) = payload {
                if lookup_marker(normalized_marker_name(marker)).kind == MarkerKind::Note {
                    LintCode::NoteNotClosed
                } else {
                    LintCode::CharNotClosed
                }
            } else {
                LintCode::UnclosedMarkerAtEof
            }
        }
    }
}

fn format_recovery_message(code: RecoveryCode) -> String {
    match code {
        RecoveryCode::MissingChapterNumber => "chapter marker is missing its number".to_string(),
        RecoveryCode::MissingVerseNumber => "verse marker is missing its number".to_string(),
        RecoveryCode::MissingMilestoneSelfClose => {
            "milestone marker is missing its self-close".to_string()
        }
        RecoveryCode::ImplicitlyClosedMarker => {
            "marker was implicitly closed by later structure".to_string()
        }
        RecoveryCode::StrayCloseMarker => "closing marker has no matching opener".to_string(),
        RecoveryCode::MisnestedCloseMarker => {
            "closing marker mismatches the current open stack".to_string()
        }
        RecoveryCode::UnclosedNote => "note was left open".to_string(),
        RecoveryCode::UnclosedMarkerAtEof => "marker was still open at end of file".to_string(),
    }
}

fn simple_issue<T: LintableFlatToken>(code: LintCode, message: String, token: &T) -> LintIssue {
    LintIssue {
        code,
        message,
        span: token.span().clone(),
        related_span: None,
        token_id: token.id().map(ToOwned::to_owned),
        related_token_id: None,
        sid: token.sid().map(ToOwned::to_owned),
        fix: None,
    }
}

fn build_set_number_fix<T: LintableFlatToken>(token: &T, value: u32) -> Option<TokenFix> {
    let id = token.id()?;
    Some(TokenFix::ReplaceToken {
        label: format!("change number to {value}"),
        target_token_id: id.to_string(),
        replacements: vec![TokenTemplate {
            kind: TokenKind::Number,
            text: value.to_string(),
            marker: None,
            sid: token.sid().map(ToOwned::to_owned),
        }],
    })
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
        "p"
            | "m"
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
    let start = parts.next()?.parse::<u32>().ok()?;
    let end = match parts.next() {
        Some(value) => value.parse::<u32>().ok()?,
        None => start,
    };
    if parts.next().is_some() || start == 0 || end == 0 || start > end {
        return None;
    }
    Some((start, end))
}

fn parse_sid_chapter(sid: Option<&str>) -> Option<u32> {
    let sid = sid?;
    let reference = sid.split("_dup_").next().unwrap_or(sid);
    let (_, chap_and_verse) = reference.rsplit_once(' ')?;
    let (chapter, _) = chap_and_verse.split_once(':')?;
    chapter.parse().ok()
}

fn lint_unknown_token_like<T: LintableFlatToken>(token: &T) -> Option<LintIssue> {
    let text = token.text();
    let slash_index = text.find('\\')?;
    let remainder = &text[slash_index + 1..];
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

    let target_id = token.id()?.to_string();
    let text_after = after.to_string();
    Some(LintIssue {
        code: LintCode::UnknownToken,
        message: format!("unknown token {}", token.text()),
        span: token.span().clone(),
        related_span: None,
        token_id: Some(target_id.clone()),
        related_token_id: None,
        sid: token.sid().map(ToOwned::to_owned),
        fix: Some(TokenFix::ReplaceToken {
            label: format!("split into \\{marker} and text"),
            target_token_id: target_id,
            replacements: vec![
                TokenTemplate {
                    kind: TokenKind::Marker,
                    text: format!("\\{marker}"),
                    marker: Some(marker.to_string()),
                    sid: token.sid().map(ToOwned::to_owned),
                },
                TokenTemplate {
                    kind: TokenKind::Text,
                    text: text_after,
                    marker: None,
                    sid: token.sid().map(ToOwned::to_owned),
                },
            ],
        }),
    })
}

fn verse_has_text_or_note<T: LintableFlatToken>(tokens: &[T], start: usize) -> bool {
    let mut index = start;
    while index < tokens.len() {
        let token = &tokens[index];
        match token.kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => {
                index += 1;
                continue;
            }
            TokenKind::Text => {
                if token.text().trim().is_empty() {
                    index += 1;
                    continue;
                }
                return true;
            }
            TokenKind::Marker => {
                if let Some(marker) = token.marker().map(normalized_marker_name) {
                    if lookup_marker(marker).kind == MarkerKind::Note {
                        return true;
                    }
                }
                return false;
            }
            _ => return false,
        }
    }
    false
}

fn previous_significant_token_index<T: LintableFlatToken>(tokens: &[T], start: usize) -> Option<usize> {
    if start == 0 {
        return None;
    }
    for index in (0..start).rev() {
        match tokens[index].kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            _ => return Some(index),
        }
    }
    None
}

fn next_number_token_index<T: LintableFlatToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            TokenKind::Number => return Some(index),
            _ => return None,
        }
    }
    None
}

fn next_text_token_index<T: LintableFlatToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            TokenKind::Text => return Some(index),
            _ => return None,
        }
    }
    None
}

fn next_significant_token_index<T: LintableFlatToken>(tokens: &[T], start: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind() {
            TokenKind::HorizontalWhitespace | TokenKind::VerticalWhitespace => continue,
            _ => return Some(index),
        }
    }
    None
}

fn matches_previous_marker_and_number<T: LintableFlatToken>(
    tokens: &[T],
    marker_index: usize,
    expected_marker: &str,
) -> bool {
    let Some(prev_index) = previous_significant_token_index(tokens, marker_index) else {
        return false;
    };
    let prev = &tokens[prev_index];
    if prev.kind() != &TokenKind::Number {
        return false;
    }
    let Some(before_number_index) = previous_significant_token_index(tokens, prev_index) else {
        return false;
    };
    let before_number = &tokens[before_number_index];
    before_number.kind() == &TokenKind::Marker
        && before_number.marker().map(normalized_marker_name) == Some(expected_marker)
}

fn find_token_id_for_span<T: LintableFlatToken>(tokens: &[T], span: &Span) -> Option<String> {
    tokens
        .iter()
        .find(|token| token.span().start <= span.start && token.span().end >= span.end)
        .and_then(|token| token.id().map(ToOwned::to_owned))
}

fn find_sid_for_span<T: LintableFlatToken>(tokens: &[T], span: &Span) -> Option<String> {
    tokens
        .iter()
        .find(|token| token.span().start <= span.start && token.span().end >= span.end)
        .and_then(|token| token.sid().map(ToOwned::to_owned))
}

fn dedupe_and_filter_issues(
    issues: Vec<LintIssue>,
    suppressions: &[LintSuppression],
) -> Vec<LintIssue> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();

    for issue in issues {
        if suppressions
            .iter()
            .any(|suppression| suppression.code == issue.code && suppression.span == issue.span)
        {
            continue;
        }

        let identity = (
            issue.code,
            issue.span.start,
            issue.span.end,
            issue.related_span.as_ref().map(|span| (span.start, span.end)),
            issue.token_id.clone(),
        );
        if seen.insert(identity) {
            deduped.push(issue);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::token::{TokenKind, TokenViewOptions, WhitespacePolicy};

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

    impl LintableFlatToken for EditorToken {
        fn kind(&self) -> &TokenKind {
            &self.token_kind
        }

        fn span(&self) -> &Span {
            &self.token_span
        }

        fn text(&self) -> &str {
            &self.token_text
        }

        fn marker(&self) -> Option<&str> {
            self.token_marker.as_deref()
        }

        fn sid(&self) -> Option<&str> {
            self.token_sid.as_deref()
        }

        fn id(&self) -> Option<&str> {
            Some(&self.token_id)
        }
    }

    #[test]
    fn missing_separator_rule_can_be_disabled() {
        let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
        let projected = tokens(&handle, TokenViewOptions::default());

        let issues = lint_tokens(
            &projected,
            TokenLintOptions {
                disabled_rules: vec![LintCode::MissingSeparatorAfterMarker],
                suppressions: Vec::new(),
            },
        );

        assert!(issues
            .iter()
            .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker));
    }

    #[test]
    fn missing_separator_rule_emits_fix() {
        let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
        let projected = tokens(&handle, TokenViewOptions::default());

        let issues = lint_tokens(&projected, TokenLintOptions::default());

        assert!(issues.iter().any(|issue| {
            issue.code == LintCode::MissingSeparatorAfterMarker
                && matches!(issue.fix, Some(TokenFix::ReplaceToken { .. }))
        }));
    }

    #[test]
    fn missing_separator_rule_allows_separator_on_marker_token() {
        let handle = parse("\\id REV\n\\c 19\n\\m (for fine linen)\n");
        let projected = tokens(
            &handle,
            TokenViewOptions {
                whitespace_policy: WhitespacePolicy::Preserve,
            },
        );

        let issues = lint_tokens(&projected, TokenLintOptions::default());

        assert!(issues
            .iter()
            .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker));
    }

    #[test]
    fn verse_continuity_rules_are_reported() {
        let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 text\n\\v 3 text\n");
        let projected = tokens(&handle, TokenViewOptions::default());

        let issues = lint_tokens(&projected, TokenLintOptions::default());

        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::VerseExpectedIncreaseByOne));
    }

    #[test]
    fn suppressions_match_by_span_and_rule() {
        let handle = parse("\\id REV\n\\c 19\n\\m(for fine linen)\n");
        let projected = tokens(&handle, TokenViewOptions::default());
        let target_span = projected
            .iter()
            .find(|token| token.kind == TokenKind::Marker && token.marker.as_deref() == Some("m"))
            .map(|token| token.span.clone())
            .expect("expected marker span");

        let issues = lint_tokens(
            &projected,
            TokenLintOptions {
                disabled_rules: Vec::new(),
                suppressions: vec![LintSuppression {
                    code: LintCode::MissingSeparatorAfterMarker,
                    span: target_span,
                }],
            },
        );

        assert!(issues
            .iter()
            .all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker));
    }

    #[test]
    fn lint_accepts_editor_tokens_without_conversion() {
        let tokens = vec![
            EditorToken {
                token_kind: TokenKind::Marker,
                token_span: 0..2,
                token_text: "\\m".to_string(),
                token_marker: Some("m".to_string()),
                token_sid: Some("REV 19:14".to_string()),
                token_id: "REV-0".to_string(),
                lane: 1,
            },
            EditorToken {
                token_kind: TokenKind::Text,
                token_span: 2..8,
                token_text: "(text)".to_string(),
                token_marker: None,
                token_sid: Some("REV 19:14".to_string()),
                token_id: "REV-1".to_string(),
                lane: 1,
            },
        ];

        let issues = lint_tokens(&tokens, TokenLintOptions::default());

        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::MissingSeparatorAfterMarker));
        assert_eq!(tokens[0].lane, 1);
    }

    #[test]
    fn handle_lint_respects_whitespace_projection() {
        let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
        let merged = lint(
            &handle,
            LintOptions {
                include_parse_recoveries: true,
                token_view: TokenViewOptions {
                    whitespace_policy: WhitespacePolicy::MergeToVisible,
                },
                token_rules: TokenLintOptions::default(),
            },
        );

        assert!(merged.iter().all(|issue| issue.code != LintCode::MissingSeparatorAfterMarker));
    }

    #[test]
    fn body_paragraph_before_first_chapter_is_reported() {
        let handle = parse("\\id GEN Test\n\\p\n\\c 1\n\\v 1 text\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::ParagraphBeforeFirstChapter));
    }

    #[test]
    fn note_submarker_outside_note_is_reported() {
        let handle = parse("\\id GEN Test\n\\c 1\n\\ft outside note\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::NoteSubmarkerOutsideNote));
    }

    #[test]
    fn duplicate_id_marker_is_reported() {
        let handle = parse("\\id GEN Test\n\\c 1\n\\id EXO Other\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues.iter().any(|issue| issue.code == LintCode::DuplicateIdMarker));
        assert!(issues.iter().any(|issue| issue.code == LintCode::IdMarkerNotAtFileStart));
    }

    #[test]
    fn unclosed_note_before_paragraph_boundary_is_fixable() {
        let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v 1 text \\f + \\ft note\n\\p\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues.iter().any(|issue| {
            issue.code == LintCode::NoteNotClosed
                && matches!(issue.fix, Some(TokenFix::ReplaceToken { .. }))
        }));
    }

    #[test]
    fn chapter_marker_missing_number_is_reported_with_explicit_rule() {
        let handle = parse("\\id GEN Test\n\\c\n\\p\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::NumberRangeAfterChapterMarker));
    }

    #[test]
    fn verse_marker_missing_number_is_reported_with_explicit_rule() {
        let handle = parse("\\id GEN Test\n\\c 1\n\\p\n\\v\n");
        let issues = lint(&handle, LintOptions::default());
        assert!(issues
            .iter()
            .any(|issue| issue.code == LintCode::VerseRangeExpectedAfterVerseMarker));
    }

}
