use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value as swb_to_js_value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use usfm_onion::cst::{CstDocument as NativeCstDocument, CstNode as NativeCstNode, parse_cst};
use usfm_onion::diff::{
    BuildSidBlocksOptions as NativeBuildSidBlocksOptions,
    ChapterTokenDiff as NativeChapterTokenDiff, DiffStatus as NativeDiffStatus,
    DiffTokenChange as NativeDiffTokenChange, DiffUndoSide as NativeDiffUndoSide, DiffableToken,
    DiffsByChapterMap as NativeDiffsByChapterMap, SidBlock as NativeSidBlock,
    TokenAlignment as NativeTokenAlignment, apply_revert_by_block_id, apply_reverts_by_block_id,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
};
use usfm_onion::format::{
    FormatOptions as NativeFormatOptions, FormatRule as NativeFormatRule,
    FormatToken as NativeFormatToken, format_tokens as native_format_tokens, format_tokens_to_usfm,
    format_usfm,
};
use usfm_onion::html::{
    HtmlCallerScope as NativeHtmlCallerScope, HtmlCallerStyle as NativeHtmlCallerStyle,
    HtmlNoteMode as NativeHtmlNoteMode, HtmlOptions as NativeHtmlOptions, usfm_to_html,
};
use usfm_onion::lint::{
    LintCategory as NativeLintCategory, LintCode as NativeLintCode,
    LintIssueType as NativeLintIssueType, LintOptions as NativeLintOptions,
    LintResult as NativeLintResult, LintSeverity as NativeLintSeverity,
    LintSuppression as NativeLintSuppression, LintableToken, TokenFix as NativeTokenFix,
    apply_token_fix, lint_tokens, lint_usfm,
};
use usfm_onion::marker_defs::{
    BlockBehavior as NativeBlockBehavior, ClosingBehavior as NativeClosingBehavior,
    InlineContext as NativeInlineContext, MarkerFamily as NativeMarkerFamily,
    MarkerFamilyRole as NativeMarkerFamilyRole, NoteFamily as NativeNoteFamily,
    NoteSubkind as NativeNoteSubkind, ParagraphCategory as NativeParagraphCategory,
    MarkerPayload as NativeMarkerPayload, SpecContext as NativeSpecContext,
    StructuralMarkerInfo as NativeStructuralMarkerInfo,
    StructuralScopeKind as NativeStructuralScopeKind,
};
use usfm_onion::markers::{
    MarkerCategory as NativeMarkerCategory, MarkerKind as NativeMarkerKind,
    UsfmMarkerInfo as NativeUsfmMarkerInfo, is_known_marker, marker_catalog, marker_info,
};
use usfm_onion::parse::parse as native_parse;
use usfm_onion::token::{
    AttributeItem as NativeAttributeItem, MarkerMetadata as NativeMarkerMetadata,
    NumberRangeKind as NativeNumberRangeKind, Span as NativeSpan, Token as NativeToken,
    TokenData as NativeTokenData, TokenKind as NativeTokenKind, tokens_to_usfm,
};
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::{
    Segment as NativeSegment, Utf16Span as NativeUtf16Span,
    VerseProjection as NativeVerseProjection, VrefIndex as NativeVrefIndex, VrefMap as NativeVrefMap,
    tokens_to_vref_index, usfm_to_vref_index, usfm_to_vref_map,
};
use usfm_onion::walker::WalkableToken;

// TODO: eventually move off of this ideally
#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
// JSON Value type and USJ document tree.
//
// Mirroring the native `UsjDocument` shape as a real Tsify type is
// follow-up work; until that lands, USJ is the one return type that
// is still emitted as a TS_TYPES hand-written declaration. Every
// other public shape is tsify-derived.

export type Value =
  | string
  | number
  | boolean
  | null
  | Value[]
  | { [key: string]: Value };

export type UsjDocument = {
  type: string;
  version: string;
  content: UsjNode[];
};

export type UsjNode = string | UsjElement;

export type UsjElement =
  | ({ type: "book"; marker: string; code: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "chapter"; marker: string; number: string; sid?: string } & Record<string, Value>)
  | ({ type: "verse"; marker: string; number: string; sid?: string } & Record<string, Value>)
  | ({ type: "para"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "char"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "ref"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "note"; marker: string; caller: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "ms"; marker: string } & Record<string, Value>)
  | ({ type: "figure"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "sidebar"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "periph"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table:row"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table:cell"; marker: string; align?: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "unknown"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "unmatched"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "optbreak" } & Record<string, Value>);
"#;

// ---------------------------------------------------------------------------
// FFI enums (tsify-derived) — one source of truth for the JS-side string
// unions that used to be hand-written in TS_TYPES.
//
// Each enum here mirrors a native enum and exposes the same wire format
// the previous stringify pairs produced. From impls in both directions keep
// the boundary between native and FFI fully typed; no string parsing.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
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

impl From<NativeTokenKind> for TokenKind {
    fn from(value: NativeTokenKind) -> Self {
        match value {
            NativeTokenKind::Newline => Self::Newline,
            NativeTokenKind::OptBreak => Self::OptBreak,
            NativeTokenKind::Marker => Self::Marker,
            NativeTokenKind::EndMarker => Self::EndMarker,
            NativeTokenKind::Milestone => Self::Milestone,
            NativeTokenKind::MilestoneEnd => Self::MilestoneEnd,
            NativeTokenKind::BookCode => Self::BookCode,
            NativeTokenKind::Number => Self::Number,
            NativeTokenKind::Text => Self::Text,
        }
    }
}

impl From<TokenKind> for NativeTokenKind {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Newline => Self::Newline,
            TokenKind::OptBreak => Self::OptBreak,
            TokenKind::Marker => Self::Marker,
            TokenKind::EndMarker => Self::EndMarker,
            TokenKind::Milestone => Self::Milestone,
            TokenKind::MilestoneEnd => Self::MilestoneEnd,
            TokenKind::BookCode => Self::BookCode,
            TokenKind::Number => Self::Number,
            TokenKind::Text => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum NumberRangeKind {
    Single,
    Range,
    Sequence,
    SequenceWithRange,
}

impl From<NativeNumberRangeKind> for NumberRangeKind {
    fn from(value: NativeNumberRangeKind) -> Self {
        match value {
            NativeNumberRangeKind::Single => Self::Single,
            NativeNumberRangeKind::Range => Self::Range,
            NativeNumberRangeKind::Sequence => Self::Sequence,
            NativeNumberRangeKind::SequenceWithRange => Self::SequenceWithRange,
        }
    }
}

impl From<NumberRangeKind> for NativeNumberRangeKind {
    fn from(value: NumberRangeKind) -> Self {
        match value {
            NumberRangeKind::Single => Self::Single,
            NumberRangeKind::Range => Self::Range,
            NumberRangeKind::Sequence => Self::Sequence,
            NumberRangeKind::SequenceWithRange => Self::SequenceWithRange,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "kebab-case")]
pub enum LintCategory {
    Document,
    Structure,
    Context,
    Numbering,
}

impl From<NativeLintCategory> for LintCategory {
    fn from(value: NativeLintCategory) -> Self {
        match value {
            NativeLintCategory::Document => Self::Document,
            NativeLintCategory::Structure => Self::Structure,
            NativeLintCategory::Context => Self::Context,
            NativeLintCategory::Numbering => Self::Numbering,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "kebab-case")]
pub enum LintSeverity {
    Error,
    Warning,
}

impl From<NativeLintSeverity> for LintSeverity {
    fn from(value: NativeLintSeverity) -> Self {
        match value {
            NativeLintSeverity::Error => Self::Error,
            NativeLintSeverity::Warning => Self::Warning,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "kebab-case")]
pub enum LintIssueType {
    Usfm,
    Content,
}

impl From<NativeLintIssueType> for LintIssueType {
    fn from(value: NativeLintIssueType) -> Self {
        match value {
            NativeLintIssueType::Usfm => Self::Usfm,
            NativeLintIssueType::Content => Self::Content,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "kebab-case")]
pub enum LintCode {
    MissingIdMarker,
    DuplicateIdMarker,
    IdMarkerNotAtFileStart,
    EmptyParagraph,
    MissingChapterNumber,
    MissingVerseNumber,
    VerseIsEmpty,
    UnknownToken,
    UnknownMarker,
    UnknownCloseMarker,
    ContentBeforeFirstChapter,
    VerseOutsideExplicitParagraph,
    NoteSubmarkerOutsideNote,
    MetadataOutsideTarget,
    MarkerNotValidInContext,
    MissingMilestoneSelfClose,
    StrayCloseMarker,
    MisnestedCloseMarker,
    ImplicitlyClosedMarker,
    UnclosedMarker,
    DuplicateChapterNumber,
    ChapterExpectedIncreaseByOne,
    InconsistentChapterLabel,
    DuplicateVerseNumber,
    VerseExpectedIncreaseByOne,
    InvalidNumberRange,
    NumberRangeNotPrecededByMarkerExpectingNumber,
    MissingWhitespaceBeforeMarker,
    MissingHorizontalWhitespaceAfterMarkerName,
    MissingTagEndDelimiterAfterMarker,
    MissingContentSpaceAfterCloseMarker,
    VerseInSectionOrOtherParagraph,
    ContentAfterBlankMarker,
}

impl From<NativeLintCode> for LintCode {
    fn from(value: NativeLintCode) -> Self {
        match value {
            NativeLintCode::MissingIdMarker => Self::MissingIdMarker,
            NativeLintCode::DuplicateIdMarker => Self::DuplicateIdMarker,
            NativeLintCode::IdMarkerNotAtFileStart => Self::IdMarkerNotAtFileStart,
            NativeLintCode::EmptyParagraph => Self::EmptyParagraph,
            NativeLintCode::MissingChapterNumber => Self::MissingChapterNumber,
            NativeLintCode::MissingVerseNumber => Self::MissingVerseNumber,
            NativeLintCode::VerseIsEmpty => Self::VerseIsEmpty,
            NativeLintCode::UnknownToken => Self::UnknownToken,
            NativeLintCode::UnknownMarker => Self::UnknownMarker,
            NativeLintCode::UnknownCloseMarker => Self::UnknownCloseMarker,
            NativeLintCode::ContentBeforeFirstChapter => Self::ContentBeforeFirstChapter,
            NativeLintCode::VerseOutsideExplicitParagraph => Self::VerseOutsideExplicitParagraph,
            NativeLintCode::NoteSubmarkerOutsideNote => Self::NoteSubmarkerOutsideNote,
            NativeLintCode::MetadataOutsideTarget => Self::MetadataOutsideTarget,
            NativeLintCode::MarkerNotValidInContext => Self::MarkerNotValidInContext,
            NativeLintCode::MissingMilestoneSelfClose => Self::MissingMilestoneSelfClose,
            NativeLintCode::StrayCloseMarker => Self::StrayCloseMarker,
            NativeLintCode::MisnestedCloseMarker => Self::MisnestedCloseMarker,
            NativeLintCode::ImplicitlyClosedMarker => Self::ImplicitlyClosedMarker,
            NativeLintCode::UnclosedMarker => Self::UnclosedMarker,
            NativeLintCode::DuplicateChapterNumber => Self::DuplicateChapterNumber,
            NativeLintCode::ChapterExpectedIncreaseByOne => Self::ChapterExpectedIncreaseByOne,
            NativeLintCode::InconsistentChapterLabel => Self::InconsistentChapterLabel,
            NativeLintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            NativeLintCode::VerseExpectedIncreaseByOne => Self::VerseExpectedIncreaseByOne,
            NativeLintCode::InvalidNumberRange => Self::InvalidNumberRange,
            NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber => {
                Self::NumberRangeNotPrecededByMarkerExpectingNumber
            }
            NativeLintCode::MissingWhitespaceBeforeMarker => Self::MissingWhitespaceBeforeMarker,
            NativeLintCode::MissingHorizontalWhitespaceAfterMarkerName => {
                Self::MissingHorizontalWhitespaceAfterMarkerName
            }
            NativeLintCode::MissingTagEndDelimiterAfterMarker => {
                Self::MissingTagEndDelimiterAfterMarker
            }
            NativeLintCode::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            NativeLintCode::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
            NativeLintCode::ContentAfterBlankMarker => Self::ContentAfterBlankMarker,
        }
    }
}

impl From<LintCode> for NativeLintCode {
    fn from(value: LintCode) -> Self {
        match value {
            LintCode::MissingIdMarker => Self::MissingIdMarker,
            LintCode::DuplicateIdMarker => Self::DuplicateIdMarker,
            LintCode::IdMarkerNotAtFileStart => Self::IdMarkerNotAtFileStart,
            LintCode::EmptyParagraph => Self::EmptyParagraph,
            LintCode::MissingChapterNumber => Self::MissingChapterNumber,
            LintCode::MissingVerseNumber => Self::MissingVerseNumber,
            LintCode::VerseIsEmpty => Self::VerseIsEmpty,
            LintCode::UnknownToken => Self::UnknownToken,
            LintCode::UnknownMarker => Self::UnknownMarker,
            LintCode::UnknownCloseMarker => Self::UnknownCloseMarker,
            LintCode::ContentBeforeFirstChapter => Self::ContentBeforeFirstChapter,
            LintCode::VerseOutsideExplicitParagraph => Self::VerseOutsideExplicitParagraph,
            LintCode::NoteSubmarkerOutsideNote => Self::NoteSubmarkerOutsideNote,
            LintCode::MetadataOutsideTarget => Self::MetadataOutsideTarget,
            LintCode::MarkerNotValidInContext => Self::MarkerNotValidInContext,
            LintCode::MissingMilestoneSelfClose => Self::MissingMilestoneSelfClose,
            LintCode::StrayCloseMarker => Self::StrayCloseMarker,
            LintCode::MisnestedCloseMarker => Self::MisnestedCloseMarker,
            LintCode::ImplicitlyClosedMarker => Self::ImplicitlyClosedMarker,
            LintCode::UnclosedMarker => Self::UnclosedMarker,
            LintCode::DuplicateChapterNumber => Self::DuplicateChapterNumber,
            LintCode::ChapterExpectedIncreaseByOne => Self::ChapterExpectedIncreaseByOne,
            LintCode::InconsistentChapterLabel => Self::InconsistentChapterLabel,
            LintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            LintCode::VerseExpectedIncreaseByOne => Self::VerseExpectedIncreaseByOne,
            LintCode::InvalidNumberRange => Self::InvalidNumberRange,
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber => {
                Self::NumberRangeNotPrecededByMarkerExpectingNumber
            }
            LintCode::MissingWhitespaceBeforeMarker => Self::MissingWhitespaceBeforeMarker,
            LintCode::MissingHorizontalWhitespaceAfterMarkerName => {
                Self::MissingHorizontalWhitespaceAfterMarkerName
            }
            LintCode::MissingTagEndDelimiterAfterMarker => Self::MissingTagEndDelimiterAfterMarker,
            LintCode::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            LintCode::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
            LintCode::ContentAfterBlankMarker => Self::ContentAfterBlankMarker,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

impl From<NativeDiffStatus> for DiffStatus {
    fn from(value: NativeDiffStatus) -> Self {
        match value {
            NativeDiffStatus::Added => Self::Added,
            NativeDiffStatus::Deleted => Self::Deleted,
            NativeDiffStatus::Modified => Self::Modified,
            NativeDiffStatus::Unchanged => Self::Unchanged,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffTokenChange {
    Unchanged,
    Added,
    Deleted,
    Modified,
}

impl From<NativeDiffTokenChange> for DiffTokenChange {
    fn from(value: NativeDiffTokenChange) -> Self {
        match value {
            NativeDiffTokenChange::Unchanged => Self::Unchanged,
            NativeDiffTokenChange::Added => Self::Added,
            NativeDiffTokenChange::Deleted => Self::Deleted,
            NativeDiffTokenChange::Modified => Self::Modified,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffUndoSide {
    Original,
    Current,
}

impl From<NativeDiffUndoSide> for DiffUndoSide {
    fn from(value: NativeDiffUndoSide) -> Self {
        match value {
            NativeDiffUndoSide::Original => Self::Original,
            NativeDiffUndoSide::Current => Self::Current,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum HtmlNoteMode {
    Extracted,
    Inline,
}

impl From<HtmlNoteMode> for NativeHtmlNoteMode {
    fn from(value: HtmlNoteMode) -> Self {
        match value {
            HtmlNoteMode::Extracted => Self::Extracted,
            HtmlNoteMode::Inline => Self::Inline,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum HtmlCallerStyle {
    Numeric,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
    Source,
}

impl From<HtmlCallerStyle> for NativeHtmlCallerStyle {
    fn from(value: HtmlCallerStyle) -> Self {
        match value {
            HtmlCallerStyle::Numeric => Self::Numeric,
            HtmlCallerStyle::AlphaLower => Self::AlphaLower,
            HtmlCallerStyle::AlphaUpper => Self::AlphaUpper,
            HtmlCallerStyle::RomanLower => Self::RomanLower,
            HtmlCallerStyle::RomanUpper => Self::RomanUpper,
            HtmlCallerStyle::Source => Self::Source,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum HtmlCallerScope {
    DocumentSequential,
    VerseSequential,
}

impl From<HtmlCallerScope> for NativeHtmlCallerScope {
    fn from(value: HtmlCallerScope) -> Self {
        match value {
            HtmlCallerScope::DocumentSequential => Self::DocumentSequential,
            HtmlCallerScope::VerseSequential => Self::VerseSequential,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum StructuralScopeKind {
    Unknown,
    Header,
    Block,
    Note,
    Character,
    Milestone,
    Chapter,
    Verse,
    TableRow,
    TableCell,
    Sidebar,
    Periph,
    Meta,
}

impl From<NativeStructuralScopeKind> for StructuralScopeKind {
    fn from(value: NativeStructuralScopeKind) -> Self {
        match value {
            NativeStructuralScopeKind::Unknown => Self::Unknown,
            NativeStructuralScopeKind::Header => Self::Header,
            NativeStructuralScopeKind::Block => Self::Block,
            NativeStructuralScopeKind::Note => Self::Note,
            NativeStructuralScopeKind::Character => Self::Character,
            NativeStructuralScopeKind::Milestone => Self::Milestone,
            NativeStructuralScopeKind::Chapter => Self::Chapter,
            NativeStructuralScopeKind::Verse => Self::Verse,
            NativeStructuralScopeKind::TableRow => Self::TableRow,
            NativeStructuralScopeKind::TableCell => Self::TableCell,
            NativeStructuralScopeKind::Sidebar => Self::Sidebar,
            NativeStructuralScopeKind::Periph => Self::Periph,
            NativeStructuralScopeKind::Meta => Self::Meta,
        }
    }
}

impl From<StructuralScopeKind> for NativeStructuralScopeKind {
    fn from(value: StructuralScopeKind) -> Self {
        match value {
            StructuralScopeKind::Unknown => NativeStructuralScopeKind::Unknown,
            StructuralScopeKind::Header => NativeStructuralScopeKind::Header,
            StructuralScopeKind::Block => NativeStructuralScopeKind::Block,
            StructuralScopeKind::Note => NativeStructuralScopeKind::Note,
            StructuralScopeKind::Character => NativeStructuralScopeKind::Character,
            StructuralScopeKind::Milestone => NativeStructuralScopeKind::Milestone,
            StructuralScopeKind::Chapter => NativeStructuralScopeKind::Chapter,
            StructuralScopeKind::Verse => NativeStructuralScopeKind::Verse,
            StructuralScopeKind::TableRow => NativeStructuralScopeKind::TableRow,
            StructuralScopeKind::TableCell => NativeStructuralScopeKind::TableCell,
            StructuralScopeKind::Sidebar => NativeStructuralScopeKind::Sidebar,
            StructuralScopeKind::Periph => NativeStructuralScopeKind::Periph,
            StructuralScopeKind::Meta => NativeStructuralScopeKind::Meta,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum SpecContext {
    Scripture,
    BookIdentification,
    BookHeaders,
    BookTitles,
    BookIntroduction,
    BookIntroductionEndTitles,
    BookChapterLabel,
    ChapterContent,
    Peripheral,
    PeripheralContent,
    PeripheralDivision,
    Chapter,
    Verse,
    Section,
    Para,
    List,
    Table,
    Sidebar,
    Footnote,
    CrossReference,
}

impl From<NativeSpecContext> for SpecContext {
    fn from(value: NativeSpecContext) -> Self {
        match value {
            NativeSpecContext::Scripture => Self::Scripture,
            NativeSpecContext::BookIdentification => Self::BookIdentification,
            NativeSpecContext::BookHeaders => Self::BookHeaders,
            NativeSpecContext::BookTitles => Self::BookTitles,
            NativeSpecContext::BookIntroduction => Self::BookIntroduction,
            NativeSpecContext::BookIntroductionEndTitles => Self::BookIntroductionEndTitles,
            NativeSpecContext::BookChapterLabel => Self::BookChapterLabel,
            NativeSpecContext::ChapterContent => Self::ChapterContent,
            NativeSpecContext::Peripheral => Self::Peripheral,
            NativeSpecContext::PeripheralContent => Self::PeripheralContent,
            NativeSpecContext::PeripheralDivision => Self::PeripheralDivision,
            NativeSpecContext::Chapter => Self::Chapter,
            NativeSpecContext::Verse => Self::Verse,
            NativeSpecContext::Section => Self::Section,
            NativeSpecContext::Para => Self::Para,
            NativeSpecContext::List => Self::List,
            NativeSpecContext::Table => Self::Table,
            NativeSpecContext::Sidebar => Self::Sidebar,
            NativeSpecContext::Footnote => Self::Footnote,
            NativeSpecContext::CrossReference => Self::CrossReference,
        }
    }
}

impl From<SpecContext> for NativeSpecContext {
    fn from(value: SpecContext) -> Self {
        match value {
            SpecContext::Scripture => NativeSpecContext::Scripture,
            SpecContext::BookIdentification => NativeSpecContext::BookIdentification,
            SpecContext::BookHeaders => NativeSpecContext::BookHeaders,
            SpecContext::BookTitles => NativeSpecContext::BookTitles,
            SpecContext::BookIntroduction => NativeSpecContext::BookIntroduction,
            SpecContext::BookIntroductionEndTitles => NativeSpecContext::BookIntroductionEndTitles,
            SpecContext::BookChapterLabel => NativeSpecContext::BookChapterLabel,
            SpecContext::ChapterContent => NativeSpecContext::ChapterContent,
            SpecContext::Peripheral => NativeSpecContext::Peripheral,
            SpecContext::PeripheralContent => NativeSpecContext::PeripheralContent,
            SpecContext::PeripheralDivision => NativeSpecContext::PeripheralDivision,
            SpecContext::Chapter => NativeSpecContext::Chapter,
            SpecContext::Verse => NativeSpecContext::Verse,
            SpecContext::Section => NativeSpecContext::Section,
            SpecContext::Para => NativeSpecContext::Para,
            SpecContext::List => NativeSpecContext::List,
            SpecContext::Table => NativeSpecContext::Table,
            SpecContext::Sidebar => NativeSpecContext::Sidebar,
            SpecContext::Footnote => NativeSpecContext::Footnote,
            SpecContext::CrossReference => NativeSpecContext::CrossReference,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum InlineContext {
    Para,
    Section,
    List,
    Table,
}

impl From<NativeInlineContext> for InlineContext {
    fn from(value: NativeInlineContext) -> Self {
        match value {
            NativeInlineContext::Para => Self::Para,
            NativeInlineContext::Section => Self::Section,
            NativeInlineContext::List => Self::List,
            NativeInlineContext::Table => Self::Table,
        }
    }
}

impl From<InlineContext> for NativeInlineContext {
    fn from(value: InlineContext) -> Self {
        match value {
            InlineContext::Para => NativeInlineContext::Para,
            InlineContext::Section => NativeInlineContext::Section,
            InlineContext::List => NativeInlineContext::List,
            InlineContext::Table => NativeInlineContext::Table,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum BlockBehavior {
    None,
    Paragraph,
    TableRow,
    TableCell,
    SidebarStart,
    SidebarEnd,
}

impl From<NativeBlockBehavior> for BlockBehavior {
    fn from(value: NativeBlockBehavior) -> Self {
        match value {
            NativeBlockBehavior::None => Self::None,
            NativeBlockBehavior::Paragraph(_) => Self::Paragraph,
            NativeBlockBehavior::TableRow => Self::TableRow,
            NativeBlockBehavior::TableCell => Self::TableCell,
            NativeBlockBehavior::SidebarStart => Self::SidebarStart,
            NativeBlockBehavior::SidebarEnd => Self::SidebarEnd,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum ClosingBehavior {
    None,
    RequiredExplicit,
    OptionalExplicitUntilNoteEnd,
    SelfClosingMilestone,
}

impl From<NativeClosingBehavior> for ClosingBehavior {
    fn from(value: NativeClosingBehavior) -> Self {
        match value {
            NativeClosingBehavior::None => Self::None,
            NativeClosingBehavior::RequiredExplicit => Self::RequiredExplicit,
            NativeClosingBehavior::OptionalExplicitUntilNoteEnd => {
                Self::OptionalExplicitUntilNoteEnd
            }
            NativeClosingBehavior::SelfClosingMilestone => Self::SelfClosingMilestone,
        }
    }
}

/// Argument payload a marker's opening form consumes right after its name:
/// `"bookCode"` for `\id`, `"numberRange"` for the chapter/verse family
/// (`\c`, `\cp`, `\ca`, `\v`, `\vp`, `\va`). Shares one table with the
/// lexer (`marker_defs::marker_payload`), so catalog and tokenization cannot
/// drift.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerPayload {
    BookCode,
    NumberRange,
}

impl From<NativeMarkerPayload> for MarkerPayload {
    fn from(value: NativeMarkerPayload) -> Self {
        match value {
            NativeMarkerPayload::BookCode => Self::BookCode,
            NativeMarkerPayload::NumberRange => Self::NumberRange,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerCategory {
    Document,
    Paragraph,
    Character,
    NoteContainer,
    NoteSubmarker,
    Chapter,
    Verse,
    MilestoneStart,
    MilestoneEnd,
    Figure,
    SidebarStart,
    SidebarEnd,
    Periph,
    Meta,
    TableRow,
    TableCell,
    Header,
    Unknown,
}

impl From<NativeMarkerCategory> for MarkerCategory {
    fn from(value: NativeMarkerCategory) -> Self {
        match value {
            NativeMarkerCategory::Document => Self::Document,
            NativeMarkerCategory::Paragraph => Self::Paragraph,
            NativeMarkerCategory::Character => Self::Character,
            NativeMarkerCategory::NoteContainer => Self::NoteContainer,
            NativeMarkerCategory::NoteSubmarker => Self::NoteSubmarker,
            NativeMarkerCategory::Chapter => Self::Chapter,
            NativeMarkerCategory::Verse => Self::Verse,
            NativeMarkerCategory::MilestoneStart => Self::MilestoneStart,
            NativeMarkerCategory::MilestoneEnd => Self::MilestoneEnd,
            NativeMarkerCategory::Figure => Self::Figure,
            NativeMarkerCategory::SidebarStart => Self::SidebarStart,
            NativeMarkerCategory::SidebarEnd => Self::SidebarEnd,
            NativeMarkerCategory::Periph => Self::Periph,
            NativeMarkerCategory::Meta => Self::Meta,
            NativeMarkerCategory::TableRow => Self::TableRow,
            NativeMarkerCategory::TableCell => Self::TableCell,
            NativeMarkerCategory::Header => Self::Header,
            NativeMarkerCategory::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerKind {
    Paragraph,
    Note,
    Character,
    Header,
    Chapter,
    Verse,
    MilestoneStart,
    MilestoneEnd,
    SidebarStart,
    SidebarEnd,
    Figure,
    Meta,
    Periph,
    TableRow,
    TableCell,
    Unknown,
}

impl From<NativeMarkerKind> for MarkerKind {
    fn from(value: NativeMarkerKind) -> Self {
        match value {
            NativeMarkerKind::Paragraph => Self::Paragraph,
            NativeMarkerKind::Note => Self::Note,
            NativeMarkerKind::Character => Self::Character,
            NativeMarkerKind::Header => Self::Header,
            NativeMarkerKind::Chapter => Self::Chapter,
            NativeMarkerKind::Verse => Self::Verse,
            NativeMarkerKind::MilestoneStart => Self::MilestoneStart,
            NativeMarkerKind::MilestoneEnd => Self::MilestoneEnd,
            NativeMarkerKind::SidebarStart => Self::SidebarStart,
            NativeMarkerKind::SidebarEnd => Self::SidebarEnd,
            NativeMarkerKind::Figure => Self::Figure,
            NativeMarkerKind::Meta => Self::Meta,
            NativeMarkerKind::Periph => Self::Periph,
            NativeMarkerKind::TableRow => Self::TableRow,
            NativeMarkerKind::TableCell => Self::TableCell,
            NativeMarkerKind::Unknown => Self::Unknown,
        }
    }
}

// `MarkerDefKind` (spec-level kind on tokens, distinct from `MarkerKind` which
// is the catalog-level kind on `UsfmMarkerInfo`). Smaller variant set — the
// spec doesn't distinguish milestone-start/-end or sidebar-start/-end.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerDefKind {
    Paragraph,
    Character,
    Note,
    Chapter,
    Verse,
    Milestone,
    Figure,
    Sidebar,
    Periph,
    Meta,
    TableRow,
    TableCell,
    Header,
}

impl From<usfm_onion::marker_defs::MarkerDefKind> for MarkerDefKind {
    fn from(value: usfm_onion::marker_defs::MarkerDefKind) -> Self {
        use usfm_onion::marker_defs::MarkerDefKind as K;
        match value {
            K::Paragraph => Self::Paragraph,
            K::Character => Self::Character,
            K::Note => Self::Note,
            K::Chapter => Self::Chapter,
            K::Verse => Self::Verse,
            K::Milestone => Self::Milestone,
            K::Figure => Self::Figure,
            K::Sidebar => Self::Sidebar,
            K::Periph => Self::Periph,
            K::Meta => Self::Meta,
            K::TableRow => Self::TableRow,
            K::TableCell => Self::TableCell,
            K::Header => Self::Header,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerFamily {
    Footnote,
    CrossReference,
    SectionParagraph,
    ListParagraph,
    TableCell,
    Milestone,
    Sidebar,
}

impl From<NativeMarkerFamily> for MarkerFamily {
    fn from(value: NativeMarkerFamily) -> Self {
        match value {
            NativeMarkerFamily::Footnote => Self::Footnote,
            NativeMarkerFamily::CrossReference => Self::CrossReference,
            NativeMarkerFamily::SectionParagraph => Self::SectionParagraph,
            NativeMarkerFamily::ListParagraph => Self::ListParagraph,
            NativeMarkerFamily::TableCell => Self::TableCell,
            NativeMarkerFamily::Milestone => Self::Milestone,
            NativeMarkerFamily::Sidebar => Self::Sidebar,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum MarkerFamilyRole {
    Canonical,
    NumberedVariant,
    NestedVariant,
    MilestoneStart,
    MilestoneEnd,
    Alias,
}

impl From<NativeMarkerFamilyRole> for MarkerFamilyRole {
    fn from(value: NativeMarkerFamilyRole) -> Self {
        match value {
            NativeMarkerFamilyRole::Canonical => Self::Canonical,
            NativeMarkerFamilyRole::NumberedVariant => Self::NumberedVariant,
            NativeMarkerFamilyRole::NestedVariant => Self::NestedVariant,
            NativeMarkerFamilyRole::MilestoneStart => Self::MilestoneStart,
            NativeMarkerFamilyRole::MilestoneEnd => Self::MilestoneEnd,
            NativeMarkerFamilyRole::Alias => Self::Alias,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum ParagraphCategory {
    Identification,
    Introduction,
    Title,
    Section,
    Body,
    Poetry,
    List,
    Table,
    Peripheral,
    Other,
}

impl From<NativeParagraphCategory> for ParagraphCategory {
    fn from(value: NativeParagraphCategory) -> Self {
        match value {
            NativeParagraphCategory::Identification => Self::Identification,
            NativeParagraphCategory::Introduction => Self::Introduction,
            NativeParagraphCategory::Title => Self::Title,
            NativeParagraphCategory::Section => Self::Section,
            NativeParagraphCategory::Body => Self::Body,
            NativeParagraphCategory::Poetry => Self::Poetry,
            NativeParagraphCategory::List => Self::List,
            NativeParagraphCategory::Table => Self::Table,
            NativeParagraphCategory::Peripheral => Self::Peripheral,
            NativeParagraphCategory::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum NoteFamily {
    Footnote,
    CrossReference,
}

impl From<NativeNoteFamily> for NoteFamily {
    fn from(value: NativeNoteFamily) -> Self {
        match value {
            NativeNoteFamily::Footnote => Self::Footnote,
            NativeNoteFamily::CrossReference => Self::CrossReference,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum NoteSubkind {
    Structural,
    StructuralKeepsNestedCharsOpen,
}

impl From<NativeNoteSubkind> for NoteSubkind {
    fn from(value: NativeNoteSubkind) -> Self {
        match value {
            NativeNoteSubkind::Structural => Self::Structural,
            NativeNoteSubkind::StructuralKeepsNestedCharsOpen => {
                Self::StructuralKeepsNestedCharsOpen
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Value types — schema-of-record for the JS surface. tsify-derived TS types
// emit from these directly; their wire format is the byte-for-byte
// contract recorded under `crates/usfm_onion_wasm/golden/outputs/`.
// ---------------------------------------------------------------------------

/// Diff result grouped by book and chapter: `{ "GEN": { 1: [...], 2: [...] } }`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct DiffsByChapterMap(
    pub std::collections::BTreeMap<String, std::collections::BTreeMap<u32, Vec<ChapterTokenDiff>>>,
);

/// Verse-reference map: `{ "GEN 1:1": "...", "GEN 1:2": "...", ... }`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct VrefMap(pub std::collections::BTreeMap<String, String>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    start: u32,
    end: u32,
}

/// UTF-16 code-unit offsets into a `VerseProjection.text`. Deliberately a
/// distinct type from `Span` (byte offsets into the source) so the unit is
/// unmistakable on the wire — JS/DOM consumers index `text` in UTF-16.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Utf16Span {
    start: u32,
    end: u32,
}

/// One in-scope text token's contribution to a verse projection, with both
/// resolution anchors: `sourceSpan` (bytes into source, for raw buffers) and
/// `tokenId` (== the editor's DOM `data-id`). `textSpan` is UTF-16 into `text`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    token_id: String,
    source_span: Span,
    text_span: Utf16Span,
}

/// Lossless plain-text projection of one verse plus its segment map back to
/// source / token coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct VerseProjection {
    text: String,
    segments: Vec<Segment>,
}

/// `sid` -> lossless verse projection. Same key set as `VrefMap`; the
/// difference is losslessness plus the segment map.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct VrefIndex(pub std::collections::BTreeMap<String, VerseProjection>);

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MarkerMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<MarkerDefKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<MarkerFamily>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AttributeItem {
    span: Span,
    text: String,
    key: String,
    value: String,
    #[serde(default)]
    is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StructuralMarkerInfo {
    scope_kind: StructuralScopeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<InlineContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_context: Option<SpecContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct NumberInfo {
    start: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end: Option<u32>,
    kind: NumberRangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    id: String,
    kind: TokenKind,
    source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nested: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker_metadata: Option<MarkerMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    structural: Option<StructuralMarkerInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number_info: Option<NumberInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    book_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    book_code_valid: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attributes: Vec<AttributeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
    token_index: usize,
    children: Vec<CstNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CstDocument {
    tokens: Vec<Token>,
    roots: Vec<CstNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSuppression {
    code: LintCode,
    sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintOptions {
    #[serde(default)]
    enabled_codes: Option<Vec<LintCode>>,
    #[serde(default)]
    disabled_codes: Vec<LintCode>,
    #[serde(default)]
    suppressed: Vec<LintSuppression>,
    #[serde(default)]
    allow_implicit_chapter_content_verse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintIssue {
    code: LintCode,
    category: LintCategory,
    severity: LintSeverity,
    issue_type: LintIssueType,
    template: String,
    message: String,
    message_params: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    related_span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    related_token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fix: Option<TokenFix>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSummary {
    by_category: std::collections::BTreeMap<LintCategory, usize>,
    by_severity: std::collections::BTreeMap<LintSeverity, usize>,
    by_issue_type: std::collections::BTreeMap<LintIssueType, usize>,
    total_count: usize,
    suppressed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintResult {
    issues: Vec<LintIssue>,
    summary: LintSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TokenFix {
    ReplaceToken {
        code: String,
        label: String,
        label_params: std::collections::BTreeMap<String, String>,
        target_token_id: String,
        replacements: Vec<TokenTemplate>,
    },
    DeleteToken {
        code: String,
        label: String,
        label_params: std::collections::BTreeMap<String, String>,
        target_token_id: String,
    },
    InsertAfter {
        code: String,
        label: String,
        label_params: std::collections::BTreeMap<String, String>,
        target_token_id: String,
        insert: Vec<TokenTemplate>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TokenTemplate {
    kind: TokenKind,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    #[serde(default)]
    recover_malformed_markers: Option<bool>,
    #[serde(default)]
    collapse_whitespace_in_text: Option<bool>,
    #[serde(default)]
    ensure_inline_separators: Option<bool>,
    #[serde(default)]
    remove_duplicate_verse_numbers: Option<bool>,
    #[serde(default)]
    normalize_spacing_after_paragraph_markers: Option<bool>,
    #[serde(default)]
    remove_unwanted_linebreaks: Option<bool>,
    #[serde(default)]
    bridge_consecutive_verse_markers: Option<bool>,
    #[serde(default)]
    remove_orphan_empty_verse_before_contentful_verse: Option<bool>,
    #[serde(default)]
    remove_bridge_verse_enumerators: Option<bool>,
    #[serde(default)]
    move_chapter_label_after_chapter_marker: Option<bool>,
    #[serde(default)]
    insert_default_paragraph_after_chapter_intro: Option<bool>,
    #[serde(default)]
    remove_empty_paragraphs: Option<bool>,
    #[serde(default)]
    insert_structural_linebreaks: Option<bool>,
    #[serde(default)]
    collapse_consecutive_linebreaks: Option<bool>,
    #[serde(default)]
    normalize_marker_whitespace_at_line_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatResult {
    tokens: Vec<Token>,
    usfm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct HtmlOptions {
    #[serde(default)]
    wrap_root: bool,
    #[serde(default)]
    prefer_native_elements: Option<bool>,
    #[serde(default)]
    note_mode: Option<HtmlNoteMode>,
    #[serde(default)]
    caller_style: Option<HtmlCallerStyle>,
    #[serde(default)]
    caller_scope: Option<HtmlCallerScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BuildSidBlocksOptions {
    #[serde(default)]
    allow_empty_sid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SidBlock {
    block_id: String,
    semantic_sid: String,
    start: usize,
    end_exclusive: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prev_block_id: Option<String>,
    text_full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TokenAlignment {
    change: DiffTokenChange,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    counterpart_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTokenDiff {
    block_id: String,
    semantic_sid: String,
    status: DiffStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original: Option<SidBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current: Option<SidBlock>,
    original_text: String,
    current_text: String,
    original_text_only: String,
    current_text_only: String,
    is_whitespace_change: bool,
    is_usfm_structure_change: bool,
    original_tokens: Vec<Token>,
    current_tokens: Vec<Token>,
    original_alignment: Vec<TokenAlignment>,
    current_alignment: Vec<TokenAlignment>,
    undo_side: DiffUndoSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintCodeMeta {
    code: LintCode,
    category: LintCategory,
    severity: LintSeverity,
    issue_type: LintIssueType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatRuleMeta {
    code: String,
    label_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MarkerInfo {
    marker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical: Option<String>,
    known: bool,
    deprecated: bool,
    category: MarkerCategory,
    kind: MarkerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<MarkerFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family_role: Option<MarkerFamilyRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_family: Option<NoteFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_subkind: Option<NoteSubkind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<InlineContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_attribute: Option<String>,
    contexts: Vec<SpecContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    block_behavior: Option<BlockBehavior>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    closing_behavior: Option<ClosingBehavior>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payload: Option<MarkerPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    paragraph_category: Option<ParagraphCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

/// Owned token shape that satisfies the library's `WalkableToken`,
/// `LintableToken`, and `DiffableToken` traits with **native** enum
/// types. The JS-facing `Token` carries the FFI mirror enums
/// (`TokenKind`, `StructuralMarkerInfo`, …); converting them on every
/// trait-method access inside a walker would cost several enum
/// conversions per token per walker pass — tens of thousands of extra
/// match statements for a Luke-sized document. So token-in entry
/// points do one linear `Token` → `WalkToken` pass at call time and
/// the walker runs over these.
#[derive(Debug, Clone)]
struct WalkToken {
    id: String,
    kind: NativeTokenKind,
    text: String,
    span: Option<NativeSpan>,
    sid: Option<String>,
    marker: Option<String>,
    structural: Option<NativeStructuralMarkerInfo>,
    number_info: Option<(u32, Option<u32>, NativeNumberRangeKind)>,
}

impl WalkableToken for WalkToken {
    fn kind(&self) -> NativeTokenKind {
        self.kind
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn structural(&self) -> Option<NativeStructuralMarkerInfo> {
        self.structural
    }

    fn text(&self) -> &str {
        &self.text
    }
}

impl LintableToken for WalkToken {
    fn span(&self) -> Option<NativeSpan> {
        self.span
    }

    fn sid(&self) -> Option<String> {
        self.sid.clone()
    }

    fn id(&self) -> Option<String> {
        Some(self.id.clone())
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NativeNumberRangeKind)> {
        self.number_info
    }
}

impl DiffableToken for WalkToken {
    fn sid(&self) -> Option<&str> {
        self.sid.as_deref()
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }

    fn kind_key(&self) -> Option<&str> {
        Some(token_kind_wire_key(self.kind))
    }

    fn marker_key(&self) -> Option<&str> {
        self.marker.as_deref()
    }
}

#[wasm_bindgen]
pub struct ParsedUsfm {
    source: String,
}

#[wasm_bindgen]
pub struct UsfmMarkerCatalog;

#[wasm_bindgen]
impl ParsedUsfm {
    fn new(source: String) -> Self {
        Self { source }
    }

    pub fn tokens(&self) -> Vec<Token> {
        let parsed = native_parse(&self.source);
        map_tokens(&parsed.tokens)
    }

    pub fn cst(&self) -> CstDocument {
        let cst = parse_cst(&self.source);
        map_cst_document(&cst)
    }

    pub fn lint(&self, options: Option<LintOptions>) -> LintResult {
        map_lint_result(lint_usfm(&self.source, lint_options_into_native(options)))
    }

    #[wasm_bindgen(js_name = applyTokenFix)]
    pub fn apply_token_fix(&self, fix: TokenFix) -> Vec<Token> {
        let parsed = native_parse(&self.source);
        let native_tokens = parsed
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let result = apply_token_fix(&native_tokens, &token_fix_into_native(fix));
        result.iter().map(map_format_token).collect()
    }

    #[wasm_bindgen(js_name = revertDiffBlock)]
    pub fn revert_diff_block(
        &self,
        current: &ParsedUsfm,
        block_id: &str,
        options: Option<BuildSidBlocksOptions>,
    ) -> Vec<Token> {
        let baseline = native_parse(&self.source);
        let current = native_parse(&current.source);
        let baseline = baseline
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let current = current
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let reverted = apply_revert_by_block_id(
            block_id,
            &baseline,
            &current,
            &build_options_into_native(options),
        );
        reverted.iter().map(map_format_token).collect()
    }

    pub fn format(&self, options: Option<FormatOptions>) -> String {
        format_usfm(&self.source, format_options_into_native(options))
    }

    #[wasm_bindgen(js_name = toUsfm)]
    pub fn to_usfm(&self) -> String {
        let parsed = native_parse(&self.source);
        tokens_to_usfm(&parsed.tokens)
    }

    #[wasm_bindgen(js_name = toUsj)]
    pub fn to_usj(&self) -> Result<JsValue, JsError> {
        let document = usfm_to_usj(&self.source).map_err(js_error)?;
        to_js_value(&document)
    }

    #[wasm_bindgen(js_name = toUsx)]
    pub fn to_usx(&self) -> Result<String, JsError> {
        usfm_to_usx(&self.source).map_err(js_error)
    }

    #[wasm_bindgen(js_name = toHtml)]
    pub fn to_html(&self, options: Option<HtmlOptions>) -> String {
        usfm_to_html(&self.source, html_options_into_native(options))
    }

    #[wasm_bindgen(js_name = toVref)]
    pub fn to_vref(&self) -> VrefMap {
        VrefMap(vref_to_object(usfm_to_vref_map(&self.source)))
    }

    #[wasm_bindgen(js_name = vrefIndex)]
    pub fn vref_index(&self) -> VrefIndex {
        map_vref_index(usfm_to_vref_index(&self.source))
    }

    pub fn diff(
        &self,
        other: &ParsedUsfm,
        options: Option<BuildSidBlocksOptions>,
    ) -> Vec<ChapterTokenDiff> {
        let options = build_options_into_native(options);
        let diffs = diff_usfm_sources(&self.source, &other.source, &options);
        map_chapter_diffs(&diffs)
    }

    #[wasm_bindgen(js_name = diffByChapter)]
    pub fn diff_by_chapter(
        &self,
        other: &ParsedUsfm,
        options: Option<BuildSidBlocksOptions>,
    ) -> DiffsByChapterMap {
        let options = build_options_into_native(options);
        let diffs = diff_usfm_sources_by_chapter(&self.source, &other.source, &options);
        DiffsByChapterMap(map_diffs_by_chapter(&diffs))
    }
}

#[wasm_bindgen]
impl UsfmMarkerCatalog {
    fn new() -> Self {
        Self
    }

    pub fn all(&self) -> Vec<MarkerInfo> {
        marker_catalog()
            .all()
            .iter()
            .cloned()
            .map(map_marker_info)
            .collect()
    }

    pub fn get(&self, marker: &str) -> Option<MarkerInfo> {
        marker_catalog().get(marker).cloned().map(map_marker_info)
    }

    pub fn contains(&self, marker: &str) -> bool {
        marker_catalog().contains(marker)
    }
}

#[wasm_bindgen(js_name = parse)]
pub fn wasm_parse(source: &str) -> ParsedUsfm {
    ParsedUsfm::new(source.to_string())
}

#[wasm_bindgen(js_name = lintUsfm)]
pub fn wasm_lint_usfm(source: &str, options: Option<LintOptions>) -> LintResult {
    map_lint_result(lint_usfm(source, lint_options_into_native(options)))
}

#[wasm_bindgen(js_name = vrefIndexUsfm)]
pub fn wasm_vref_index_usfm(source: &str) -> VrefIndex {
    map_vref_index(usfm_to_vref_index(source))
}

/// Build the vref index from an existing token stream (the editor's live
/// path) — same rehydration as `lintTokens`, no reparse. Segment ids match
/// the tokens passed in, so they line up with the editor's DOM `data-id`s.
#[wasm_bindgen(js_name = vrefIndexTokens)]
pub fn wasm_vref_index_tokens(tokens: Vec<Token>) -> VrefIndex {
    let tokens = parse_walk_tokens_from_values(tokens);
    map_vref_index(tokens_to_vref_index(&tokens))
}

#[wasm_bindgen(js_name = lintTokens)]
pub fn wasm_lint_tokens(tokens: Vec<Token>, options: Option<LintOptions>) -> LintResult {
    let tokens = parse_walk_tokens_from_values(tokens);
    map_lint_result(lint_tokens(&tokens, lint_options_into_native(options)))
}

#[wasm_bindgen(js_name = applyTokenFix)]
pub fn wasm_apply_token_fix(tokens: Vec<Token>, fix: TokenFix) -> Vec<Token> {
    let native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let result = apply_token_fix(&native_tokens, &token_fix_into_native(fix));
    result.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = formatUsfm)]
pub fn wasm_format_usfm(source: &str, options: Option<FormatOptions>) -> String {
    format_usfm(source, format_options_into_native(options))
}

#[wasm_bindgen(js_name = formatTokens)]
pub fn wasm_format_tokens(tokens: Vec<Token>, options: Option<FormatOptions>) -> FormatResult {
    let mut native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, format_options_into_native(options));
    FormatResult {
        tokens: native_tokens.iter().map(map_format_token).collect(),
        usfm: format_tokens_to_usfm(&native_tokens),
    }
}

#[wasm_bindgen(js_name = formatTokensMut)]
pub fn wasm_format_tokens_mut(tokens: Vec<Token>, options: Option<FormatOptions>) -> Vec<Token> {
    let mut native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, format_options_into_native(options));
    native_tokens.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = tokensToUsfm)]
pub fn wasm_tokens_to_usfm(tokens: Vec<Token>) -> String {
    token_values_to_usfm(&tokens)
}

#[wasm_bindgen(js_name = tokensToHtml)]
pub fn wasm_tokens_to_html(tokens: Vec<Token>, options: Option<HtmlOptions>) -> String {
    let usfm = token_values_to_usfm(&tokens);
    usfm_to_html(&usfm, html_options_into_native(options))
}

#[wasm_bindgen(js_name = diffUsfm)]
pub fn wasm_diff_usfm(
    left: &str,
    right: &str,
    options: Option<BuildSidBlocksOptions>,
) -> Vec<ChapterTokenDiff> {
    let options = build_options_into_native(options);
    let diffs = diff_usfm_sources(left, right, &options);
    map_chapter_diffs(&diffs)
}

#[wasm_bindgen(js_name = diffUsfmByChapter)]
pub fn wasm_diff_usfm_by_chapter(
    left: &str,
    right: &str,
    options: Option<BuildSidBlocksOptions>,
) -> DiffsByChapterMap {
    let options = build_options_into_native(options);
    let diffs = diff_usfm_sources_by_chapter(left, right, &options);
    DiffsByChapterMap(map_diffs_by_chapter(&diffs))
}

#[wasm_bindgen(js_name = diffTokens)]
pub fn wasm_diff_tokens(
    left: Vec<Token>,
    right: Vec<Token>,
    options: Option<BuildSidBlocksOptions>,
) -> Vec<ChapterTokenDiff> {
    let left = parse_walk_tokens_from_values(left);
    let right = parse_walk_tokens_from_values(right);
    let options = build_options_into_native(options);
    let diffs = diff_chapter_token_streams(&left, &right, &options);
    map_walk_token_diffs(&diffs)
}

#[wasm_bindgen(js_name = revertDiffBlock)]
pub fn wasm_revert_diff_block(
    baseline: Vec<Token>,
    current: Vec<Token>,
    block_id: &str,
    options: Option<BuildSidBlocksOptions>,
) -> Vec<Token> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let reverted = apply_revert_by_block_id(
        block_id,
        &baseline,
        &current,
        &build_options_into_native(options),
    );
    reverted.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = revertDiffBlocks)]
pub fn wasm_revert_diff_blocks(
    baseline: Vec<Token>,
    current: Vec<Token>,
    block_ids: Vec<String>,
    options: Option<BuildSidBlocksOptions>,
) -> Vec<Token> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let reverted = apply_reverts_by_block_id(
        &block_ids,
        &baseline,
        &current,
        &build_options_into_native(options),
    );
    reverted.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = markerCatalog)]
pub fn wasm_marker_catalog() -> UsfmMarkerCatalog {
    UsfmMarkerCatalog::new()
}

#[wasm_bindgen(js_name = markerInfo)]
pub fn wasm_marker_info(marker: &str) -> MarkerInfo {
    map_marker_info(marker_info(marker))
}

#[wasm_bindgen(js_name = isKnownMarker)]
pub fn wasm_is_known_marker(marker: &str) -> bool {
    is_known_marker(marker)
}

#[wasm_bindgen(js_name = lintCodes)]
pub fn wasm_lint_codes() -> Vec<LintCode> {
    lint_code_variants()
        .into_iter()
        .map(LintCode::from)
        .collect()
}

#[wasm_bindgen(js_name = lintCodeMeta)]
pub fn wasm_lint_code_meta() -> Vec<LintCodeMeta> {
    lint_code_variants()
        .into_iter()
        .map(|code| LintCodeMeta {
            code: code.into(),
            category: code.category().into(),
            severity: code.severity().into(),
            issue_type: code.issue_type().into(),
        })
        .collect()
}

#[wasm_bindgen(js_name = formatRules)]
pub fn wasm_format_rules() -> Vec<String> {
    NativeFormatRule::ALL
        .iter()
        .map(|rule| rule.code().to_string())
        .collect()
}

#[wasm_bindgen(js_name = formatRuleMeta)]
pub fn wasm_format_rule_meta() -> Vec<FormatRuleMeta> {
    NativeFormatRule::ALL
        .iter()
        .map(|rule| FormatRuleMeta {
            code: rule.code().to_string(),
            label_key: rule.label_key().to_string(),
        })
        .collect()
}

fn lint_options_into_native(value: Option<LintOptions>) -> NativeLintOptions {
    let value = value.unwrap_or_default();
    NativeLintOptions {
        enabled_codes: value
            .enabled_codes
            .map(|codes| codes.into_iter().map(NativeLintCode::from).collect()),
        disabled_codes: value
            .disabled_codes
            .into_iter()
            .map(NativeLintCode::from)
            .collect(),
        suppressed: value
            .suppressed
            .into_iter()
            .map(|suppression| NativeLintSuppression {
                code: suppression.code.into(),
                sid: suppression.sid,
            })
            .collect(),
        allow_implicit_chapter_content_verse: value.allow_implicit_chapter_content_verse,
    }
}

fn format_options_into_native(value: Option<FormatOptions>) -> NativeFormatOptions {
    let value = value.unwrap_or_default();
    let mut options = NativeFormatOptions::default();
    apply_opt(
        &mut options.recover_malformed_markers,
        value.recover_malformed_markers,
    );
    apply_opt(
        &mut options.collapse_whitespace_in_text,
        value.collapse_whitespace_in_text,
    );
    apply_opt(
        &mut options.ensure_inline_separators,
        value.ensure_inline_separators,
    );
    apply_opt(
        &mut options.remove_duplicate_verse_numbers,
        value.remove_duplicate_verse_numbers,
    );
    apply_opt(
        &mut options.normalize_spacing_after_paragraph_markers,
        value.normalize_spacing_after_paragraph_markers,
    );
    apply_opt(
        &mut options.remove_unwanted_linebreaks,
        value.remove_unwanted_linebreaks,
    );
    apply_opt(
        &mut options.bridge_consecutive_verse_markers,
        value.bridge_consecutive_verse_markers,
    );
    apply_opt(
        &mut options.remove_orphan_empty_verse_before_contentful_verse,
        value.remove_orphan_empty_verse_before_contentful_verse,
    );
    apply_opt(
        &mut options.remove_bridge_verse_enumerators,
        value.remove_bridge_verse_enumerators,
    );
    apply_opt(
        &mut options.move_chapter_label_after_chapter_marker,
        value.move_chapter_label_after_chapter_marker,
    );
    apply_opt(
        &mut options.insert_default_paragraph_after_chapter_intro,
        value.insert_default_paragraph_after_chapter_intro,
    );
    apply_opt(
        &mut options.remove_empty_paragraphs,
        value.remove_empty_paragraphs,
    );
    apply_opt(
        &mut options.insert_structural_linebreaks,
        value.insert_structural_linebreaks,
    );
    apply_opt(
        &mut options.collapse_consecutive_linebreaks,
        value.collapse_consecutive_linebreaks,
    );
    apply_opt(
        &mut options.normalize_marker_whitespace_at_line_start,
        value.normalize_marker_whitespace_at_line_start,
    );
    options
}

fn html_options_into_native(value: Option<HtmlOptions>) -> NativeHtmlOptions {
    let value = value.unwrap_or_default();
    NativeHtmlOptions {
        wrap_root: value.wrap_root,
        prefer_native_elements: value.prefer_native_elements.unwrap_or(true),
        note_mode: value
            .note_mode
            .map(Into::into)
            .unwrap_or(NativeHtmlNoteMode::Extracted),
        caller_style: value
            .caller_style
            .map(Into::into)
            .unwrap_or(NativeHtmlCallerStyle::Numeric),
        caller_scope: value
            .caller_scope
            .map(Into::into)
            .unwrap_or(NativeHtmlCallerScope::VerseSequential),
    }
}

fn build_options_into_native(value: Option<BuildSidBlocksOptions>) -> NativeBuildSidBlocksOptions {
    let value = value.unwrap_or_default();
    NativeBuildSidBlocksOptions {
        allow_empty_sid: value.allow_empty_sid.unwrap_or(true),
    }
}

fn parse_walk_tokens_from_values(values: Vec<Token>) -> Vec<WalkToken> {
    values.into_iter().map(token_to_walk_token).collect()
}

fn token_to_walk_token(value: Token) -> WalkToken {
    WalkToken {
        id: value.id,
        kind: value.kind.into(),
        text: value.source,
        span: value.span.map(native_span),
        sid: value.sid,
        marker: value.marker,
        structural: value.structural.map(parse_structural_info),
        number_info: value.number_info.map(parse_number_info),
    }
}

fn token_value_to_format_token(value: Token) -> NativeFormatToken {
    NativeFormatToken {
        kind: value.kind.into(),
        text: value.source,
        marker: value.marker,
        sid: value.sid,
        id: Some(value.id),
        span: value.span.map(native_span),
        structural: value.structural.map(parse_structural_info),
        number_info: value.number_info.map(parse_number_info),
        marker_profile: None,
    }
}

fn format_token_with_identity(token: &NativeToken<'_>) -> NativeFormatToken {
    let mut owned = NativeFormatToken::from(token);
    owned.sid = token
        .sid
        .map(|sid| format_sid(sid.book.as_str(), sid.chapter, sid.verse));
    owned.id = Some(format!("{}-{}", token.id.book_code, token.id.index));
    owned
}

fn token_fix_into_native(value: TokenFix) -> NativeTokenFix {
    match value {
        TokenFix::ReplaceToken {
            code,
            label,
            label_params,
            target_token_id,
            replacements,
        } => NativeTokenFix::ReplaceToken {
            code,
            label,
            label_params,
            target_token_id,
            replacements: replacements.into_iter().map(parse_token_template).collect(),
        },
        TokenFix::DeleteToken {
            code,
            label,
            label_params,
            target_token_id,
        } => NativeTokenFix::DeleteToken {
            code,
            label,
            label_params,
            target_token_id,
        },
        TokenFix::InsertAfter {
            code,
            label,
            label_params,
            target_token_id,
            insert,
        } => NativeTokenFix::InsertAfter {
            code,
            label,
            label_params,
            target_token_id,
            insert: insert.into_iter().map(parse_token_template).collect(),
        },
    }
}

fn parse_token_template(value: TokenTemplate) -> usfm_onion::TokenTemplate {
    usfm_onion::TokenTemplate {
        kind: value.kind.into(),
        text: value.text,
        marker: value.marker,
        sid: value.sid,
    }
}

fn parse_structural_info(value: StructuralMarkerInfo) -> NativeStructuralMarkerInfo {
    NativeStructuralMarkerInfo {
        scope_kind: value.scope_kind.into(),
        inline_context: value.inline_context.map(Into::into),
        note_context: value.note_context.map(Into::into),
    }
}

fn parse_number_info(value: NumberInfo) -> (u32, Option<u32>, NativeNumberRangeKind) {
    (value.start, value.end, value.kind.into())
}

fn map_tokens(tokens: &[NativeToken<'_>]) -> Vec<Token> {
    tokens.iter().map(map_token).collect()
}

fn map_token(token: &NativeToken<'_>) -> Token {
    let mut value = Token {
        id: format!("{}-{}", token.id.book_code, token.id.index),
        kind: token.kind().into(),
        source: token.source.to_string(),
        span: Some(map_span(token.span)),
        sid: token
            .sid
            .map(|sid| format_sid(sid.book.as_str(), sid.chapter, sid.verse)),
        marker: token.marker_name().map(ToOwned::to_owned),
        nested: None,
        marker_metadata: None,
        structural: None,
        number_info: None,
        book_code: None,
        book_code_valid: None,
        attributes: Vec::new(),
    };

    match &token.data {
        NativeTokenData::Marker {
            metadata,
            structural,
            nested,
            attributes,
            ..
        } => {
            value.nested = Some(*nested);
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
            value.attributes = attributes.iter().map(map_attribute_item).collect();
        }
        NativeTokenData::EndMarker {
            metadata,
            structural,
            nested,
            ..
        } => {
            value.nested = Some(*nested);
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
        }
        NativeTokenData::Milestone {
            metadata,
            structural,
            attributes,
            ..
        } => {
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
            value.attributes = attributes.iter().map(map_attribute_item).collect();
        }
        NativeTokenData::BookCode { code, is_valid } => {
            value.book_code = Some((*code).to_string());
            value.book_code_valid = Some(*is_valid);
        }
        NativeTokenData::Number { start, end, kind } => {
            value.number_info = Some(NumberInfo {
                start: *start,
                end: *end,
                kind: (*kind).into(),
            });
        }
        NativeTokenData::Newline
        | NativeTokenData::OptBreak
        | NativeTokenData::MilestoneEnd
        | NativeTokenData::Text => {}
    }

    value
}

fn map_format_token(token: &NativeFormatToken) -> Token {
    Token {
        id: token.id.clone().unwrap_or_default(),
        kind: token.kind.into(),
        source: token.text.clone(),
        span: token.span.map(map_span),
        sid: token.sid.clone(),
        marker: token.marker.clone(),
        nested: None,
        marker_metadata: None,
        structural: token.structural.map(map_structural_info),
        number_info: token.number_info.map(|(start, end, kind)| NumberInfo {
            start,
            end,
            kind: kind.into(),
        }),
        book_code: None,
        book_code_valid: None,
        attributes: Vec::new(),
    }
}

fn map_attribute_item(item: &NativeAttributeItem<'_>) -> AttributeItem {
    AttributeItem {
        span: map_span(item.span),
        text: item.source.to_string(),
        key: item.key.to_string(),
        value: decode_attr_value(item.value),
        is_default: item.is_default,
    }
}

/// Decodes USFM-wire escapes in an attribute value so JS consumers see the
/// logical string. Mirrors `encode_attr_value` on the emit side: `\\` → `\`,
/// `\"` → `"`. Other backslash sequences are preserved verbatim (the lexer
/// only recognizes those two escapes today).
fn decode_attr_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.clone().next() {
                Some('\\') => {
                    chars.next();
                    out.push('\\');
                    continue;
                }
                Some('"') => {
                    chars.next();
                    out.push('"');
                    continue;
                }
                _ => {}
            }
        }
        out.push(ch);
    }
    out
}

/// Re-escapes a logical attribute value for USFM emit. Inverse of
/// `decode_attr_value`. Editors that hand back a value containing a literal
/// `"` or `\` get a well-formed `\"` / `\\` on the wire — the usfm-onion
/// extension to USFM 3.1 covered by [D3] in `rfc.md`.
fn encode_attr_value(value: &str) -> String {
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
/// the bare value (USFM 3.1 default-attribute shorthand).
fn format_attribute_list(attrs: &[AttributeItem]) -> String {
    let mut out = String::from("|");
    for (i, item) in attrs.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        if item.is_default {
            out.push_str(&encode_attr_value(&item.value));
        } else {
            out.push_str(&item.key);
            out.push_str("=\"");
            out.push_str(&encode_attr_value(&item.value));
            out.push('"');
        }
    }
    out
}

fn map_marker_metadata(metadata: NativeMarkerMetadata) -> MarkerMetadata {
    MarkerMetadata {
        canonical: metadata.canonical.map(ToOwned::to_owned),
        kind: metadata.kind.map(Into::into),
        family: metadata.family.map(Into::into),
    }
}

fn map_structural_info(info: NativeStructuralMarkerInfo) -> StructuralMarkerInfo {
    StructuralMarkerInfo {
        scope_kind: info.scope_kind.into(),
        inline_context: info.inline_context.map(Into::into),
        note_context: info.note_context.map(Into::into),
    }
}

fn map_span(span: NativeSpan) -> Span {
    Span {
        start: span.start,
        end: span.end,
    }
}

fn native_span(span: Span) -> NativeSpan {
    NativeSpan {
        start: span.start,
        end: span.end,
    }
}

fn map_cst_document(document: &NativeCstDocument<'_>) -> CstDocument {
    CstDocument {
        tokens: map_tokens(&document.tokens),
        roots: document.roots.iter().map(map_cst_node).collect(),
    }
}

fn map_cst_node(node: &NativeCstNode) -> CstNode {
    CstNode {
        token_index: node.token_index,
        children: node.children.iter().map(map_cst_node).collect(),
    }
}

fn map_lint_result(result: NativeLintResult) -> LintResult {
    LintResult {
        issues: result.issues.into_iter().map(map_lint_issue).collect(),
        summary: map_lint_summary(result.summary),
    }
}

fn map_lint_summary(summary: usfm_onion::LintSummary) -> LintSummary {
    LintSummary {
        by_category: summary
            .by_category
            .into_iter()
            .map(|(category, count)| (category.into(), count))
            .collect(),
        by_severity: summary
            .by_severity
            .into_iter()
            .map(|(severity, count)| (severity.into(), count))
            .collect(),
        by_issue_type: summary
            .by_issue_type
            .into_iter()
            .map(|(issue_type, count)| (issue_type.into(), count))
            .collect(),
        total_count: summary.total_count,
        suppressed_count: summary.suppressed_count,
    }
}

fn map_token_fix(fix: NativeTokenFix) -> TokenFix {
    match fix {
        NativeTokenFix::ReplaceToken {
            code,
            label,
            label_params,
            target_token_id,
            replacements,
        } => TokenFix::ReplaceToken {
            code,
            label,
            label_params,
            target_token_id,
            replacements: replacements.into_iter().map(map_token_template).collect(),
        },
        NativeTokenFix::DeleteToken {
            code,
            label,
            label_params,
            target_token_id,
        } => TokenFix::DeleteToken {
            code,
            label,
            label_params,
            target_token_id,
        },
        NativeTokenFix::InsertAfter {
            code,
            label,
            label_params,
            target_token_id,
            insert,
        } => TokenFix::InsertAfter {
            code,
            label,
            label_params,
            target_token_id,
            insert: insert.into_iter().map(map_token_template).collect(),
        },
    }
}

fn map_token_template(template: usfm_onion::TokenTemplate) -> TokenTemplate {
    TokenTemplate {
        kind: template.kind.into(),
        text: template.text,
        marker: template.marker,
        sid: template.sid,
    }
}

fn map_lint_issue(issue: usfm_onion::LintIssue) -> LintIssue {
    LintIssue {
        code: issue.code.into(),
        category: issue.category.into(),
        severity: issue.severity.into(),
        issue_type: issue.issue_type.into(),
        template: issue.template.to_string(),
        message: issue.message,
        message_params: issue.message_params,
        span: issue.span.map(map_span),
        related_span: issue.related_span.map(map_span),
        token_id: issue.token_id,
        related_token_id: issue.related_token_id,
        sid: issue.sid,
        marker: issue.marker,
        fix: issue.fix.map(map_token_fix),
    }
}

fn map_chapter_diffs(diffs: &[NativeChapterTokenDiff<NativeToken<'_>>]) -> Vec<ChapterTokenDiff> {
    diffs.iter().map(map_native_chapter_diff).collect()
}

fn map_native_chapter_diff(diff: &NativeChapterTokenDiff<NativeToken<'_>>) -> ChapterTokenDiff {
    ChapterTokenDiff {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff.status.into(),
        original: diff.original.as_ref().map(map_sid_block),
        current: diff.current.as_ref().map(map_sid_block),
        original_text: diff.original_text.clone(),
        current_text: diff.current_text.clone(),
        original_text_only: diff.original_text_only.clone(),
        current_text_only: diff.current_text_only.clone(),
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens: map_tokens(&diff.original_tokens),
        current_tokens: map_tokens(&diff.current_tokens),
        original_alignment: diff
            .original_alignment
            .iter()
            .copied()
            .map(map_alignment)
            .collect(),
        current_alignment: diff
            .current_alignment
            .iter()
            .copied()
            .map(map_alignment)
            .collect(),
        undo_side: diff.undo_side.into(),
    }
}

fn map_walk_token_diffs(diffs: &[NativeChapterTokenDiff<WalkToken>]) -> Vec<ChapterTokenDiff> {
    diffs.iter().map(map_walk_token_chapter_diff).collect()
}

fn map_walk_token_chapter_diff(diff: &NativeChapterTokenDiff<WalkToken>) -> ChapterTokenDiff {
    ChapterTokenDiff {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff.status.into(),
        original: diff.original.as_ref().map(map_sid_block),
        current: diff.current.as_ref().map(map_sid_block),
        original_text: diff.original_text.clone(),
        current_text: diff.current_text.clone(),
        original_text_only: diff.original_text_only.clone(),
        current_text_only: diff.current_text_only.clone(),
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens: diff.original_tokens.iter().map(map_walk_token).collect(),
        current_tokens: diff.current_tokens.iter().map(map_walk_token).collect(),
        original_alignment: diff
            .original_alignment
            .iter()
            .copied()
            .map(map_alignment)
            .collect(),
        current_alignment: diff
            .current_alignment
            .iter()
            .copied()
            .map(map_alignment)
            .collect(),
        undo_side: diff.undo_side.into(),
    }
}

fn map_walk_token(token: &WalkToken) -> Token {
    Token {
        id: token.id.clone(),
        kind: token.kind.into(),
        source: token.text.clone(),
        span: token.span.map(map_span),
        sid: token.sid.clone(),
        marker: token.marker.clone(),
        nested: None,
        marker_metadata: None,
        structural: token.structural.map(map_structural_info),
        number_info: token.number_info.map(|(start, end, kind)| NumberInfo {
            start,
            end,
            kind: kind.into(),
        }),
        book_code: None,
        book_code_valid: None,
        attributes: Vec::new(),
    }
}

fn map_sid_block(block: &NativeSidBlock) -> SidBlock {
    SidBlock {
        block_id: block.block_id.clone(),
        semantic_sid: block.semantic_sid.clone(),
        start: block.start,
        end_exclusive: block.end_exclusive,
        prev_block_id: block.prev_block_id.clone(),
        text_full: block.text_full.clone(),
    }
}

fn map_alignment(alignment: NativeTokenAlignment) -> TokenAlignment {
    TokenAlignment {
        change: alignment.change.into(),
        counterpart_index: alignment.counterpart_index,
    }
}

fn map_diffs_by_chapter(
    diffs: &NativeDiffsByChapterMap<NativeChapterTokenDiff<NativeToken<'_>>>,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<u32, Vec<ChapterTokenDiff>>> {
    diffs
        .iter()
        .map(|(book, chapters)| {
            (
                book.clone(),
                chapters
                    .iter()
                    .map(|(chapter, diffs)| (*chapter, map_chapter_diffs(diffs)))
                    .collect(),
            )
        })
        .collect()
}

fn map_marker_info(info: NativeUsfmMarkerInfo) -> MarkerInfo {
    MarkerInfo {
        marker: info.marker,
        canonical: info.canonical,
        known: info.known,
        deprecated: info.deprecated,
        category: info.category.into(),
        kind: info.kind.into(),
        family: info.family.map(Into::into),
        family_role: info.family_role.map(Into::into),
        note_family: info.note_family.map(Into::into),
        note_subkind: info.note_subkind.map(Into::into),
        inline_context: info.inline_context.map(Into::into),
        default_attribute: info.default_attribute,
        contexts: info.contexts.into_iter().map(Into::into).collect(),
        block_behavior: info.block_behavior.map(Into::into),
        closing_behavior: info.closing_behavior.map(Into::into),
        payload: info.payload.map(Into::into),
        paragraph_category: info.paragraph_category.map(Into::into),
        source: info.source,
    }
}

/// Stream-level emitter that re-attaches parsed attribute lists at their
/// correct positions during USFM serialization. Replaces the previous
/// `tokens.map(t => t.text).join("")` pattern, which silently dropped
/// `|key="value"` slices because attributes are not part of any single
/// token's `text` / `source`.
///
/// Algorithm: walk forward, push each Marker/Milestone opener with non-empty
/// `attributes` onto a LIFO stack of pending attribute lists. Before
/// emitting each token, drain any pending lists whose owning marker is
/// closed by this token (matching `EndMarker`, `MilestoneEnd`,
/// paragraph-terminating `Newline`, or another paragraph-level `Marker`).
/// Any pending lists remaining at end-of-stream are flushed in LIFO order.
///
/// Marker form is resolved from `lookup_marker_metadata` so consumers do
/// not need to preserve `marker_metadata` or `structural` through their
/// intermediate representation — only `marker` and `attributes`.
fn token_values_to_usfm(tokens: &[Token]) -> String {
    use usfm_onion::marker_defs::{MarkerDefKind, lookup_marker_metadata};

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
        match lookup_marker_metadata(marker_name).map(|(_, kind, _)| kind) {
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

    struct Pending<'a> {
        marker_name: String,
        shape: CloserShape,
        attributes: &'a [AttributeItem],
    }

    fn is_paragraph_marker(token: &Token) -> bool {
        if !matches!(token.kind, TokenKind::Marker) {
            return false;
        }
        token
            .marker
            .as_deref()
            .map(|name| {
                matches!(
                    closer_shape(TokenKind::Marker, name),
                    CloserShape::ParagraphBoundary
                )
            })
            .unwrap_or(false)
    }

    fn token_closes(pending: &Pending<'_>, token: &Token) -> bool {
        match pending.shape {
            CloserShape::MatchingEndMarker => {
                matches!(token.kind, TokenKind::EndMarker)
                    && token.marker.as_deref() == Some(pending.marker_name.as_str())
            }
            CloserShape::MilestoneEnd => matches!(token.kind, TokenKind::MilestoneEnd),
            CloserShape::ParagraphBoundary => {
                matches!(token.kind, TokenKind::Newline) || is_paragraph_marker(token)
            }
        }
    }

    let mut output = String::new();
    let mut pending: Vec<Pending<'_>> = Vec::new();

    for token in tokens {
        while let Some(top) = pending.last() {
            if token_closes(top, token) {
                let drained = pending.pop().unwrap();
                output.push_str(&format_attribute_list(drained.attributes));
            } else {
                break;
            }
        }

        output.push_str(&token.source);

        if matches!(token.kind, TokenKind::Marker | TokenKind::Milestone)
            && !token.attributes.is_empty()
            && let Some(name) = token.marker.as_deref()
        {
            pending.push(Pending {
                marker_name: name.to_string(),
                shape: closer_shape(token.kind, name),
                attributes: &token.attributes,
            });
        }
    }

    while let Some(drained) = pending.pop() {
        output.push_str(&format_attribute_list(drained.attributes));
    }

    output
}

fn vref_to_object(map: NativeVrefMap) -> std::collections::BTreeMap<String, String> {
    map.into_iter().collect()
}

fn map_vref_index(index: NativeVrefIndex) -> VrefIndex {
    VrefIndex(
        index
            .into_iter()
            .map(|(sid, projection)| (sid, map_verse_projection(projection)))
            .collect(),
    )
}

fn map_verse_projection(projection: NativeVerseProjection) -> VerseProjection {
    VerseProjection {
        text: projection.text,
        segments: projection.segments.into_iter().map(map_segment).collect(),
    }
}

fn map_segment(segment: NativeSegment) -> Segment {
    Segment {
        token_id: segment.token_id,
        source_span: map_span(segment.source_span),
        text_span: map_utf16_span(segment.text_span),
    }
}

fn map_utf16_span(span: NativeUtf16Span) -> Utf16Span {
    Utf16Span {
        start: span.start,
        end: span.end,
    }
}

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    swb_to_js_value(value).map_err(js_serde_error)
}

fn apply_opt(target: &mut bool, value: Option<bool>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

fn js_serde_error(error: serde_wasm_bindgen::Error) -> JsError {
    js_error(error)
}

fn lint_code_variants() -> Vec<NativeLintCode> {
    vec![
        NativeLintCode::MissingIdMarker,
        NativeLintCode::DuplicateIdMarker,
        NativeLintCode::IdMarkerNotAtFileStart,
        NativeLintCode::EmptyParagraph,
        NativeLintCode::MissingChapterNumber,
        NativeLintCode::MissingVerseNumber,
        NativeLintCode::VerseIsEmpty,
        NativeLintCode::UnknownToken,
        NativeLintCode::UnknownMarker,
        NativeLintCode::UnknownCloseMarker,
        NativeLintCode::ContentBeforeFirstChapter,
        NativeLintCode::VerseOutsideExplicitParagraph,
        NativeLintCode::NoteSubmarkerOutsideNote,
        NativeLintCode::MetadataOutsideTarget,
        NativeLintCode::MarkerNotValidInContext,
        NativeLintCode::MissingMilestoneSelfClose,
        NativeLintCode::StrayCloseMarker,
        NativeLintCode::MisnestedCloseMarker,
        NativeLintCode::ImplicitlyClosedMarker,
        NativeLintCode::UnclosedMarker,
        NativeLintCode::DuplicateChapterNumber,
        NativeLintCode::ChapterExpectedIncreaseByOne,
        NativeLintCode::InconsistentChapterLabel,
        NativeLintCode::DuplicateVerseNumber,
        NativeLintCode::VerseExpectedIncreaseByOne,
        NativeLintCode::InvalidNumberRange,
        NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        NativeLintCode::MissingWhitespaceBeforeMarker,
        NativeLintCode::MissingHorizontalWhitespaceAfterMarkerName,
        NativeLintCode::MissingTagEndDelimiterAfterMarker,
        NativeLintCode::MissingContentSpaceAfterCloseMarker,
        NativeLintCode::VerseInSectionOrOtherParagraph,
        NativeLintCode::ContentAfterBlankMarker,
    ]
}

// Stable wire-format key for a native TokenKind. DiffableToken's kind_key
// returns a borrowed `&str`, so this stays as a small lookup rather than
// going through a serde round-trip on each token.
fn token_kind_wire_key(kind: NativeTokenKind) -> &'static str {
    match kind {
        NativeTokenKind::Newline => "newline",
        NativeTokenKind::OptBreak => "optBreak",
        NativeTokenKind::Marker => "marker",
        NativeTokenKind::EndMarker => "endMarker",
        NativeTokenKind::Milestone => "milestone",
        NativeTokenKind::MilestoneEnd => "milestoneEnd",
        NativeTokenKind::BookCode => "bookCode",
        NativeTokenKind::Number => "number",
        NativeTokenKind::Text => "text",
    }
}

fn format_sid(book: &str, chapter: u16, verse: u16) -> String {
    format!("{book} {chapter}:{verse}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(source: &str) -> String {
        let tokens = map_tokens(&native_parse(source).tokens);
        token_values_to_usfm(&tokens)
    }

    #[test]
    fn round_trips_character_marker_with_attributes() {
        // The canonical case the RFC was written to fix.
        let source = "\\w word|lemma=\"lemma\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_all_kitchen_sink_attribute_markers() {
        for source in [
            "\\w word|lemma=\"lemma\"\\w*",
            "\\rb gloss|gloss=\"gloss\"\\rb*",
            "\\wl word|index=\"x\"\\wl*",
            "\\jmp text|link-href=\"#x\"\\jmp*",
        ] {
            assert_eq!(round_trip(source), source, "round-trip failed for {source}");
        }
    }

    #[test]
    fn round_trips_milestone_with_attributes() {
        let source = "\\zaln-s |x-strong=\"H0430\"\\*word\\zaln-e\\*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_default_attribute_shorthand() {
        // `\w word|lemma\w*` — bare value, no key=. Round-trip preserves shorthand.
        let source = "\\w word|lemma\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_multiple_attributes() {
        let source = "\\w word|lemma=\"x\" strong=\"H0430\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_after_edit_inserting_text_in_marker_run() {
        // Edit scenario: parse, then splice a synthetic Text token between
        // the word content and the `\w*` closer. Span on the synthetic
        // token is None (consumer-fabricated). The structural emitter
        // must still drain the attribute slice before the closer.
        let source = "\\w hello|lemma=\"x\"\\w*";
        let mut tokens = map_tokens(&native_parse(source).tokens);
        let closer_idx = tokens
            .iter()
            .position(|t| matches!(t.kind, TokenKind::EndMarker))
            .expect("expected \\w* closer");
        tokens.insert(
            closer_idx,
            Token {
                id: "synthetic-1".into(),
                kind: TokenKind::Text,
                source: " world".into(),
                span: None,
                sid: None,
                marker: None,
                nested: None,
                marker_metadata: None,
                structural: None,
                number_info: None,
                book_code: None,
                book_code_valid: None,
                attributes: Vec::new(),
            },
        );
        // Attribute slice still lands right before \w*, independent of
        // the inserted Text token between word content and closer.
        assert_eq!(
            token_values_to_usfm(&tokens),
            "\\w hello world|lemma=\"x\"\\w*"
        );
    }

    #[test]
    fn round_trips_embedded_double_quote_in_attribute_value() {
        // Source uses USFM 3.1 + the usfm-onion `\"` extension (D3).
        // Decode-on-map turns `\"` into `"` on the JS-facing value; the
        // emitter re-escapes back to `\"` for byte-identical output.
        let source = "\\w word|note=\"a\\\"b\"\\w*";
        let tokens = map_tokens(&native_parse(source).tokens);
        // The JS-facing AttributeItem.value is decoded.
        let attr_token = tokens
            .iter()
            .find(|t| !t.attributes.is_empty())
            .expect("expected attribute-bearing token");
        assert_eq!(attr_token.attributes[0].value, "a\"b");
        // Emit re-encodes the literal quote.
        assert_eq!(token_values_to_usfm(&tokens), source);
    }

    #[test]
    fn encode_attr_value_escapes_quote_and_backslash() {
        assert_eq!(encode_attr_value("plain"), "plain");
        assert_eq!(encode_attr_value("a\"b"), "a\\\"b");
        assert_eq!(encode_attr_value("a\\b"), "a\\\\b");
    }

    #[test]
    fn decode_attr_value_unescapes_quote_and_backslash() {
        assert_eq!(decode_attr_value("plain"), "plain");
        assert_eq!(decode_attr_value("a\\\"b"), "a\"b");
        assert_eq!(decode_attr_value("a\\\\b"), "a\\b");
    }

    #[test]
    fn empty_token_stream_emits_empty_string() {
        assert_eq!(token_values_to_usfm(&[]), "");
    }

    #[test]
    fn round_trip_no_attributes_unchanged() {
        // Sanity: structural emit must not regress the no-attribute path.
        let source = "\\id GEN\n\\c 1\n\\v 1 In the beginning.\n";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn lint_tokens_accepts_parsed_footnote_submarker_streams() {
        let source = "\\id GEN\n\\c 1\n\\pi1\n\\v 26 Then God said, “Let Us make man in Our image, after Our likeness, to rule over the fish of the sea and the birds of the air, over the livestock, and over all the earth itself\\f + \\fr 1:26 \\ft MT; Syriac \\fqa and over all the beasts of the earth\\f* and every creature that crawls upon it.”\n\\q1\n";
        let token_values = map_tokens(&native_parse(source).tokens);
        let adapter_tokens = parse_walk_tokens_from_values(token_values);
        let result = lint_tokens(&adapter_tokens, NativeLintOptions::default());

        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker),
            "unexpected stray-close issues: {:?}",
            result.issues
        );
    }

    // Note: a prior test fed `lint_tokens` a payload with `structural`,
    // `marker_metadata`, and `nested` stripped, and expected no stray-close
    // issues. After the walker migration in e1e6011 the lint visitor needs
    // `structural` to recover open scopes — with it missing, every closer
    // (\f*, \x*, \+xt*) appears stray. Callers must pass through the
    // canonical token shape produced by `parse()`; tearing it down before
    // round-tripping is no longer a supported contract.

    #[test]
    fn lint_tokens_accepts_parsed_nested_cross_reference_char_streams() {
        let source = "\\id GEN\n\\c 1\n\\v 1 In the beginning\\x + \\xo 1:1 \\xt cross ref \\+xt nested\\+xt* tail\\x*\n";
        let token_values = map_tokens(&native_parse(source).tokens);
        let adapter_tokens = parse_walk_tokens_from_values(token_values);
        let result = lint_tokens(&adapter_tokens, NativeLintOptions::default());

        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker),
            "unexpected stray-close issues: {:?}",
            result.issues
        );
    }

    #[test]
    fn lint_token_batch_accepts_parsed_source_faithful_streams() {
        let sources = [
            "\\id GEN\n\\c 1\n\\pi1\n\\v 26 Then God said\\f + \\fr 1:26 \\ft MT; Syriac \\fqa and over all the beasts of the earth\\f*\n",
            "\\id GEN\n\\c 1\n\\v 1 In the beginning\\x + \\xo 1:1 \\xt cross ref \\+xt nested\\+xt* tail\\x*\n",
        ];
        let batches: Vec<Vec<WalkToken>> = sources
            .iter()
            .map(|source| map_tokens(&native_parse(source).tokens))
            .map(parse_walk_tokens_from_values)
            .collect();

        let results = batches
            .iter()
            .map(|tokens| lint_tokens(tokens, NativeLintOptions::default()))
            .collect::<Vec<_>>();

        assert!(results.iter().all(|result| {
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker)
        }));
    }

    #[test]
    fn lint_issue_types_and_mapping_include_message_params() {
        let code = usfm_onion::LintCode::VerseExpectedIncreaseByOne;
        let issue = usfm_onion::LintIssue {
            code,
            category: code.category(),
            severity: code.severity(),
            issue_type: code.issue_type(),
            template: code.template(),
            message: "expected verse 2 here, found 3".to_string(),
            message_params: std::collections::BTreeMap::from([
                ("expected".to_string(), "2".to_string()),
                ("found".to_string(), "3".to_string()),
            ]),
            span: None,
            related_span: None,
            token_id: None,
            related_token_id: None,
            sid: None,
            marker: Some("v".to_string()),
            fix: None,
        };

        let mapped = map_lint_issue(issue);
        assert_eq!(mapped.issue_type, LintIssueType::Content);
        assert_eq!(
            mapped.message_params.get("expected"),
            Some(&"2".to_string())
        );
        assert_eq!(mapped.message_params.get("found"), Some(&"3".to_string()));
    }
}
