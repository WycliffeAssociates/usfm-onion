//! Target-agnostic serde/tsify wire DTOs for the usfm_onion boundary — the
//! **single source** for every type that crosses the wasm/TS and native-Rust
//! (Tauri) boundaries. Never hand-mirror one of these on the far side; the
//! `usfm_onion_wasm` crate re-exports them (`pub use usfm_onion_wire::dto::…`).
//!
//! ## Adding or changing a boundary type without drift
//!
//! 1. **Define it once, here.** `#[derive(Serialize, Deserialize)]` always;
//!    `#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]` +
//!    `#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]` for
//!    the TS/ABI. Keep core (`usfm_onion`) wasm-bindgen-free — it stays the
//!    native representation; this crate is its wire form.
//! 2. **serde `rename_all` = the WIRE contract, not the native enum's.** Some
//!    native types serialize differently internally (e.g. the diff enums are
//!    PascalCase natively but camelCase on the wire). Copy the wire casing.
//! 3. **Add the `From<Native…>` conversion(s) here** (both directions if the
//!    type is also an input). For enums these matches are *exhaustive*, so
//!    adding a native variant fails to compile until you mirror it — that is
//!    the primary drift guard; lean on it.
//! 4. **Hand-lists that are NOT compiler-guarded** must be updated manually:
//!    the native `ALL_LINT_CODES` test array and `usfm_onion_wasm::lint_code_variants`.
//! 5. **Prove it:** a serde round-trip / wire-string test here (see the
//!    `boundary_enums_*` tests), then `npm run golden:wasm` (+ `:web`). A new
//!    variant makes `lint-codes.json` and the generated `.d.ts` grow — inspect,
//!    then `golden:wasm:update`. The golden is a real gate; it rots silently if
//!    not run.
//!
//! Structs have **no** exhaustiveness link across the boundary — guard those
//! with a same-payload deserialize test (see `token_deserializes_without_span_field`).

use serde::{Deserialize, Serialize};

use usfm_onion::diff::{
    CoveredSide as NativeCoveredSide, DecisionStatus as NativeDecisionStatus,
    DecisionUnitKind as NativeDecisionUnitKind, MergeSide as NativeMergeSide,
    SlotRole as NativeSlotRole, TextDiffMode as NativeTextDiffMode,
    TextDiffRun as NativeTextDiffRun, TextDiffRunKind as NativeTextDiffRunKind,
    UnitTextDiff as NativeUnitTextDiff,
};
use usfm_onion::html::{
    HtmlCallerScope as NativeHtmlCallerScope, HtmlCallerStyle as NativeHtmlCallerStyle,
    HtmlNoteMode as NativeHtmlNoteMode,
};
use usfm_onion::lint::{
    LintCategory as NativeLintCategory, LintCode as NativeLintCode,
    LintIssueType as NativeLintIssueType, LintSeverity as NativeLintSeverity,
};
use usfm_onion::marker_defs::{
    BlockBehavior as NativeBlockBehavior, ClosingBehavior as NativeClosingBehavior,
    InlineContext as NativeInlineContext, MarkerDefKind as NativeMarkerDefKind,
    MarkerFamily as NativeMarkerFamily, MarkerFamilyRole as NativeMarkerFamilyRole,
    MarkerPayload as NativeMarkerPayload, NoteFamily as NativeNoteFamily,
    NoteSubkind as NativeNoteSubkind, ParagraphCategory as NativeParagraphCategory,
    SpecContext as NativeSpecContext, StructuralMarkerInfo as NativeStructuralMarkerInfo,
    StructuralScopeKind as NativeStructuralScopeKind,
};
use usfm_onion::markers::{
    MarkerCategory as NativeMarkerCategory, MarkerKind as NativeMarkerKind,
    UsfmMarkerInfo as NativeUsfmMarkerInfo,
};
use usfm_onion::token::{
    AttributeItem as NativeAttributeItem, MarkerMetadata as NativeMarkerMetadata,
    NumberRangeKind as NativeNumberRangeKind, OwnedAttribute as NativeOwnedAttribute,
    OwnedNumberInfo as NativeOwnedNumberInfo, OwnedToken, OwnedTokenParts, SerializableAttribute,
    SerializableToken, Sid as NativeSid, Span as NativeSpan, Token as NativeToken,
    TokenData as NativeTokenData, TokenKind as NativeTokenKind, UsfmToken, encode_attr_value,
};

use crate::error::TokenInputError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

// SpecContext and InlineContext are the only marker fields that also travel
// JS → native (parse_structural_info in the wasm crate deserializes a
// StructuralMarkerInfo back into the parser's types), so they carry the
// reverse conversion as well. The other DTO enums are write-only (native → JS).
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

impl From<NativeUsfmMarkerInfo> for MarkerInfo {
    fn from(info: NativeUsfmMarkerInfo) -> Self {
        Self {
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
}

pub fn map_marker_info(info: NativeUsfmMarkerInfo) -> MarkerInfo {
    info.into()
}

// ---------------------------------------------------------------------------
// Token wire DTO — single source of truth for the flat token stream that
// travels native → JS (wasm) and native → editor (Tauri commands). The
// editor's hand-written parallel copy drifted and crashed the v0.0.9 desktop
// nightly (`missing field 'span'`); defining the shape once here — with the
// real contract, `span: Option<Span>` and its skip-if — removes that seam.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

// `MarkerDefKind` (spec-level kind on tokens, distinct from `MarkerKind` which
// is the catalog-level kind on `UsfmMarkerInfo`). Smaller variant set — the
// spec doesn't distinguish milestone-start/-end or sidebar-start/-end.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

impl From<NativeMarkerDefKind> for MarkerDefKind {
    fn from(value: NativeMarkerDefKind) -> Self {
        match value {
            NativeMarkerDefKind::Paragraph => Self::Paragraph,
            NativeMarkerDefKind::Character => Self::Character,
            NativeMarkerDefKind::Note => Self::Note,
            NativeMarkerDefKind::Chapter => Self::Chapter,
            NativeMarkerDefKind::Verse => Self::Verse,
            NativeMarkerDefKind::Milestone => Self::Milestone,
            NativeMarkerDefKind::Figure => Self::Figure,
            NativeMarkerDefKind::Sidebar => Self::Sidebar,
            NativeMarkerDefKind::Periph => Self::Periph,
            NativeMarkerDefKind::Meta => Self::Meta,
            NativeMarkerDefKind::TableRow => Self::TableRow,
            NativeMarkerDefKind::TableCell => Self::TableCell,
            NativeMarkerDefKind::Header => Self::Header,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl From<NativeSpan> for Span {
    fn from(span: NativeSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for NativeSpan {
    fn from(span: Span) -> Self {
        NativeSpan {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct MarkerMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<MarkerDefKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<MarkerFamily>,
}

impl From<NativeMarkerMetadata> for MarkerMetadata {
    fn from(metadata: NativeMarkerMetadata) -> Self {
        Self {
            canonical: metadata.canonical.map(ToOwned::to_owned),
            kind: metadata.kind.map(Into::into),
            family: metadata.family.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct AttributeItem {
    /// `None` for an attribute an editor synthesized or structurally edited —
    /// same convention as `Token.span`, and never fabricated from some other
    /// token's span, which would misreport a position this attribute never
    /// actually occupied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    pub text: String,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_default: bool,
}

impl From<&NativeAttributeItem<'_>> for AttributeItem {
    fn from(item: &NativeAttributeItem<'_>) -> Self {
        Self {
            span: Some(item.span.into()),
            text: item.source.to_string(),
            key: item.key.to_string(),
            value: decode_attr_value(item.value),
            is_default: item.is_default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct StructuralMarkerInfo {
    pub scope_kind: StructuralScopeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_context: Option<InlineContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_context: Option<SpecContext>,
}

impl From<NativeStructuralMarkerInfo> for StructuralMarkerInfo {
    fn from(info: NativeStructuralMarkerInfo) -> Self {
        Self {
            scope_kind: info.scope_kind.into(),
            inline_context: info.inline_context.map(Into::into),
            note_context: info.note_context.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct NumberInfo {
    pub start: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
    pub kind: NumberRangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub kind: TokenKind,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nested: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker_metadata: Option<MarkerMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structural: Option<StructuralMarkerInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_info: Option<NumberInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub book_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub book_code_valid: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<AttributeItem>,
    /// Verbatim `|...` attribute-list slice (native `MarkerAttrs.attribute_source`),
    /// carried across the wire so a passed-through token stays byte-lossless
    /// through `tokens_to_usfm_reconstruct`. `None` when the source had no
    /// attribute list, or when an editor authored/edited the attributes
    /// itself — see [`SerializableToken`] for the "touch an attribute, drop
    /// its verbatim" rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute_source: Option<String>,
    /// Bytes from the end of this token's own source to the start of its attribute
    /// list, when the token remembers a placement.
    ///
    /// The other half of what `attribute_source` promises: an attribute list does
    /// not sit next to the marker that owns it, and one placement rule cannot
    /// express every real layout — an alignment list sits at the opener, a wordlist
    /// list can sit past a nested closer. Carrying the verbatim text without its
    /// position makes a round trip byte-lossless only for the layouts that happen
    /// to match the fallback. `None` means no remembered placement, and an emitter
    /// places the list at the marker's closer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute_offset: Option<u32>,
}

/// The remembered distance from a token's own end to its attribute list.
///
/// `checked_sub` rather than a saturating one, mirroring core: a list recorded as
/// starting before the token that owns it is not a distance this can represent, so
/// it falls back to the closer rule instead of pretending to be zero.
fn attribute_offset(
    attrs: Option<&usfm_onion::token::MarkerAttrs<'_>>,
    owner_end: u32,
) -> Option<u32> {
    attrs
        .and_then(|attrs| attrs.attribute_source)
        .and_then(|(span, _)| span.start.checked_sub(owner_end))
}

/// Turns one boundary token into a resident token.
///
/// The inbound half of the token boundary, and the exact counterpart of
/// `From<&Token>` going out. Two layers do two jobs here: this function reads the
/// DTO — whether it said enough, whether what it said parses — while core's
/// `OwnedToken::from_parts` decides whether the facts together describe a token the
/// library can hold. Neither guesses: a DTO that is short a field and a DTO whose
/// fields contradict each other are both refusals, with the second carrying core's
/// own verdict rather than a re-worded one.
///
/// `token_idx` names the position in the caller's array, which is the only address
/// a caller can act on when one token out of a book's worth is wrong.
pub fn owned_token_from_dto(token: &Token, token_idx: u32) -> Result<OwnedToken, TokenInputError> {
    if token.id.is_empty() {
        return Err(TokenInputError::MissingId { token_idx });
    }
    let book_code = match (token.book_code.as_deref(), token.book_code_valid) {
        (None, None) => None,
        (Some(code), Some(is_valid)) => Some((code, is_valid)),
        // Half a book code is not a usable fact, and picking a default validity
        // would be inventing the answer the caller failed to give.
        _ => return Err(TokenInputError::IncompleteBookCode { token_idx }),
    };
    let attributes: Vec<NativeOwnedAttribute> =
        token.attributes.iter().map(owned_attribute).collect();

    OwnedToken::from_parts(OwnedTokenParts {
        id: &token.id,
        kind: token.kind.into(),
        source: &token.source,
        sid: token.sid.as_deref(),
        marker: token.marker.as_deref(),
        // Absent means not nested: a token that never mentions nesting is the
        // ordinary case, and every marker-free kind refuses a `true` outright.
        nested: token.nested.unwrap_or(false),
        book_code,
        number: token
            .number_info
            .as_ref()
            .map(|number| NativeOwnedNumberInfo {
                start: number.start,
                end: number.end,
                kind: number.kind.into(),
            }),
        attributes: &attributes,
        attribute_source: token.attribute_source.as_deref(),
        attribute_offset: token.attribute_offset,
    })
    .map_err(|reason| TokenInputError::Illegal { token_idx, reason })
}

/// One boundary attribute as core holds it. The span travels as-is — including
/// absent, which is what an editor-authored attribute honestly has.
///
/// The value needs recovering rather than copying: the outbound conversion decodes
/// USFM escapes so a JS consumer sees the logical string (`\"` becomes `"`), while
/// core holds the source spelling, which is what the emitter writes back. Re-encoding
/// the decoded string would be exact only for the escapes this library recognizes,
/// so the verbatim `text` — the attribute's own `key="value"` slice — is the
/// authority when it carries one, and re-encoding is the fallback for an attribute
/// the caller authored and therefore has no verbatim spelling of.
fn owned_attribute(attribute: &AttributeItem) -> NativeOwnedAttribute {
    let value = verbatim_value(attribute)
        .map(Box::from)
        .unwrap_or_else(|| Box::from(encode_attr_value(&attribute.value).as_str()));
    NativeOwnedAttribute {
        source: Box::from(attribute.text.as_str()),
        key: Box::from(attribute.key.as_str()),
        value,
        is_default: attribute.is_default,
        span: attribute
            .span
            .as_ref()
            .map(|span| NativeSpan::new(span.start, span.end)),
    }
}

/// The value exactly as it is spelled in an attribute's verbatim slice.
///
/// Read structurally, from the attribute's own key rather than by hunting for a
/// `="` in the text: a default attribute's slice *is* its value and may contain
/// anything at all — quotes of its own (`|"caption"`), or, when the source is
/// malformed, several `key="value"` pairs the parser could not split
/// (`|lemma= strong="l" …` parses as one default attribute). Searching for a
/// delimiter would cut those in the wrong place; the key says which shape this is.
///
/// `None` only when there is no verbatim slice at all, which is what an attribute
/// the caller authored has.
fn verbatim_value(attribute: &AttributeItem) -> Option<&str> {
    if attribute.text.is_empty() {
        return None;
    }
    if attribute.key.is_empty() {
        return Some(&attribute.text);
    }
    let rest = attribute
        .text
        .strip_prefix(attribute.key.as_str())?
        .strip_prefix('=')?;
    // Quoted is the well-formed spelling; an unquoted remainder is kept verbatim
    // rather than "corrected", since the emitter has to write back what was read.
    Some(
        rest.strip_prefix('"')
            .and_then(|inner| inner.strip_suffix('"'))
            .unwrap_or(rest),
    )
}

/// One resident token as the boundary shape — the outbound counterpart of
/// [`owned_token_from_dto`], and the pair that makes a round trip meaningful.
///
/// `span` is absent because an owned token has none: spanlessness is what lets it
/// outlive the source it came from. The two name-derived fields are recomputed from
/// the marker rather than carried, exactly as decoding does, so a consumer sees the
/// same resolved metadata whichever path produced its token.
impl From<&OwnedToken> for Token {
    fn from(token: &OwnedToken) -> Self {
        let marker = token.marker_name();
        let marker_like = marker.is_some();
        Token {
            id: token.id().as_str().to_string(),
            kind: token.kind().into(),
            source: token.source().to_string(),
            span: None,
            sid: token.sid().map(ToOwned::to_owned),
            marker: marker.map(ToOwned::to_owned),
            nested: marker_like.then(|| token.nested()),
            marker_metadata: marker.map(|name| usfm_onion::token::marker_metadata(name).into()),
            structural: marker.map(|name| {
                usfm_onion::marker_defs::structural_marker_info(
                    name,
                    usfm_onion::token::marker_metadata(name).kind,
                )
                .into()
            }),
            number_info: token.number_info().map(|number| NumberInfo {
                start: number.start,
                end: number.end,
                kind: number.kind.into(),
            }),
            book_code: token.book_code().map(|code| code.code.as_ref().to_string()),
            book_code_valid: token.book_code().map(|code| code.is_valid),
            attributes: token
                .attributes()
                .iter()
                .map(|attribute| AttributeItem {
                    span: attribute.span.map(Into::into),
                    text: attribute.source.as_ref().to_string(),
                    key: attribute.key.as_ref().to_string(),
                    value: decode_attr_value(attribute.value.as_ref()),
                    is_default: attribute.is_default,
                })
                .collect(),
            attribute_source: token.attribute_list().map(ToOwned::to_owned),
            attribute_offset: token.attribute_offset(),
        }
    }
}

impl<'a> From<&NativeToken<'a>> for Token {
    fn from(token: &NativeToken<'a>) -> Self {
        let mut value = Token {
            id: format!("{}-{}", token.id.book_code, token.id.index),
            kind: token.kind().into(),
            source: token.source.to_string(),
            span: Some(token.span.into()),
            sid: token.sid.map(format_sid),
            marker: token.marker_name().map(ToOwned::to_owned),
            nested: None,
            marker_metadata: None,
            structural: None,
            number_info: None,
            book_code: None,
            book_code_valid: None,
            attributes: Vec::new(),
            attribute_source: None,
            attribute_offset: None,
        };

        match &token.data {
            NativeTokenData::Marker {
                metadata,
                structural,
                nested,
                attrs,
                ..
            } => {
                value.nested = Some(*nested);
                value.marker_metadata = Some((*metadata).into());
                value.structural = Some((*structural).into());
                value.attributes = token
                    .attributes()
                    .map(|a| a.iter().map(Into::into).collect())
                    .unwrap_or_default();
                value.attribute_source = attrs
                    .as_deref()
                    .and_then(|a| a.attribute_source)
                    .map(|(_, slice)| slice.to_string());
                value.attribute_offset = attribute_offset(attrs.as_deref(), token.span.end);
            }
            NativeTokenData::EndMarker {
                metadata,
                structural,
                nested,
                ..
            } => {
                value.nested = Some(*nested);
                value.marker_metadata = Some((*metadata).into());
                value.structural = Some((*structural).into());
            }
            NativeTokenData::Milestone {
                metadata,
                structural,
                attrs,
                ..
            } => {
                value.marker_metadata = Some((*metadata).into());
                value.structural = Some((*structural).into());
                value.attributes = token
                    .attributes()
                    .map(|a| a.iter().map(Into::into).collect())
                    .unwrap_or_default();
                value.attribute_source = attrs
                    .as_deref()
                    .and_then(|a| a.attribute_source)
                    .map(|(_, slice)| slice.to_string());
                value.attribute_offset = attribute_offset(attrs.as_deref(), token.span.end);
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
}

impl SerializableAttribute for AttributeItem {
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

impl UsfmToken for Token {
    fn kind(&self) -> NativeTokenKind {
        self.kind.into()
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn source(&self) -> &str {
        &self.source
    }
}

impl SerializableToken for Token {
    type Attr = AttributeItem;

    fn attributes(&self) -> &[Self::Attr] {
        &self.attributes
    }

    fn attribute_list(&self) -> Option<&str> {
        self.attribute_source.as_deref()
    }
}

/// Formats a native `Sid` as its wire string (`"GEN 1:1"`, bridge/dup forms
/// included via `verse_locator`). Shared so the wasm crate's native-token
/// identity path and this DTO conversion produce byte-identical sids.
pub fn format_sid(sid: NativeSid) -> String {
    sid.to_string()
}

/// Decodes USFM-wire escapes in an attribute value so JS consumers see the
/// logical string. Mirrors `encode_attr_value` on the emit side: `\\` → `\`,
/// `\"` → `"`. Other backslash sequences are preserved verbatim (the lexer
/// only recognizes those two escapes today).
pub fn decode_attr_value(raw: &str) -> String {
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

// ---------------------------------------------------------------------------
// Lint enums — output wire contract (kebab-case, matching the native serde).
// `LintCode` is also an input (enabled/disabled/suppressed), so it converts
// both directions; the other three are output-only (native → wire).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
    DuplicateVerseNumber,
    InvalidNumberRange,
    NumberRangeNotPrecededByMarkerExpectingNumber,
    MissingWhitespaceBeforeMarker,
    MissingHorizontalWhitespaceAfterMarkerName,
    MissingTagEndDelimiterAfterMarker,
    MissingContentSpaceAfterCloseMarker,
    VerseInSectionOrOtherParagraph,
    ContentAfterBlankMarker,
    InvalidBookCode,
    BookCodeNotUppercase,
    BookIdMismatch,
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
            NativeLintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
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
            NativeLintCode::InvalidBookCode => Self::InvalidBookCode,
            NativeLintCode::BookCodeNotUppercase => Self::BookCodeNotUppercase,
            NativeLintCode::BookIdMismatch => Self::BookIdMismatch,
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
            LintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
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
            LintCode::InvalidBookCode => Self::InvalidBookCode,
            LintCode::BookCodeNotUppercase => Self::BookCodeNotUppercase,
            LintCode::BookIdMismatch => Self::BookIdMismatch,
        }
    }
}

// ---------------------------------------------------------------------------
// Diff enums — the native enums serialize as PascalCase (an internal form);
// the wire contract the JS boundary sees is camelCase, so these wire types
// carry the camelCase serde attrs. `MergeSide` is also an input (chosen side),
// so it converts both directions; the rest are output-only (native → wire).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum SlotRole {
    Shared,
    BaselineOnly,
    CurrentOnly,
    PairBaseline,
    PairCurrent,
}

impl From<NativeSlotRole> for SlotRole {
    fn from(value: NativeSlotRole) -> Self {
        match value {
            NativeSlotRole::Shared => Self::Shared,
            NativeSlotRole::BaselineOnly => Self::BaselineOnly,
            NativeSlotRole::CurrentOnly => Self::CurrentOnly,
            NativeSlotRole::PairBaseline => Self::PairBaseline,
            NativeSlotRole::PairCurrent => Self::PairCurrent,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum DecisionUnitKind {
    Shared,
    Added,
    Deleted,
    Coalesced,
}

impl From<NativeDecisionUnitKind> for DecisionUnitKind {
    fn from(value: NativeDecisionUnitKind) -> Self {
        match value {
            NativeDecisionUnitKind::Shared => Self::Shared,
            NativeDecisionUnitKind::Added => Self::Added,
            NativeDecisionUnitKind::Deleted => Self::Deleted,
            NativeDecisionUnitKind::Coalesced => Self::Coalesced,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum DecisionStatus {
    Unchanged,
    Modified,
    Added,
    Deleted,
    Moved,
}

impl From<NativeDecisionStatus> for DecisionStatus {
    fn from(value: NativeDecisionStatus) -> Self {
        match value {
            NativeDecisionStatus::Unchanged => Self::Unchanged,
            NativeDecisionStatus::Modified => Self::Modified,
            NativeDecisionStatus::Added => Self::Added,
            NativeDecisionStatus::Deleted => Self::Deleted,
            NativeDecisionStatus::Moved => Self::Moved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum MergeSide {
    Baseline,
    Current,
}

impl From<NativeMergeSide> for MergeSide {
    fn from(value: NativeMergeSide) -> Self {
        match value {
            NativeMergeSide::Baseline => Self::Baseline,
            NativeMergeSide::Current => Self::Current,
        }
    }
}

impl From<MergeSide> for NativeMergeSide {
    fn from(value: MergeSide) -> Self {
        match value {
            MergeSide::Baseline => Self::Baseline,
            MergeSide::Current => Self::Current,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum CoveredSide {
    Baseline,
    Current,
}

impl From<NativeCoveredSide> for CoveredSide {
    fn from(value: NativeCoveredSide) -> Self {
        match value {
            NativeCoveredSide::Baseline => Self::Baseline,
            NativeCoveredSide::Current => Self::Current,
        }
    }
}

// ---------------------------------------------------------------------------
// Intra-unit text diff wire types (2026-07-24) — presentation-only word/char
// runs attached to a Modified/Added/Deleted `DecisionUnit`. `TextDiffMode`
// and `DiffOptions` are inputs (the caller picks a granularity), so they
// convert wire → native; `TextDiffRunKind`/`TextDiffRun`/`UnitTextDiff` are
// outputs (native → wire) only. See
// `plans/approved/plan-intra-unit-text-diff-2026-07-20.md`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum TextDiffMode {
    #[default]
    None,
    Words,
    Chars,
}

impl From<TextDiffMode> for NativeTextDiffMode {
    fn from(value: TextDiffMode) -> Self {
        match value {
            TextDiffMode::None => Self::None,
            TextDiffMode::Words => Self::Words,
            TextDiffMode::Chars => Self::Chars,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum TextDiffRunKind {
    Unchanged,
    Added,
    Removed,
}

impl From<NativeTextDiffRunKind> for TextDiffRunKind {
    fn from(value: NativeTextDiffRunKind) -> Self {
        match value {
            NativeTextDiffRunKind::Unchanged => Self::Unchanged,
            NativeTextDiffRunKind::Added => Self::Added,
            NativeTextDiffRunKind::Removed => Self::Removed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct TextDiffRun {
    pub text: String,
    pub kind: TextDiffRunKind,
}

impl From<NativeTextDiffRun> for TextDiffRun {
    fn from(run: NativeTextDiffRun) -> Self {
        Self {
            text: run.text,
            kind: run.kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct UnitTextDiff {
    /// Kinds: `Unchanged` | `Removed`.
    pub baseline: Vec<TextDiffRun>,
    /// Kinds: `Unchanged` | `Added`.
    pub current: Vec<TextDiffRun>,
}

impl From<NativeUnitTextDiff> for UnitTextDiff {
    fn from(value: NativeUnitTextDiff) -> Self {
        Self {
            baseline: value.baseline.into_iter().map(Into::into).collect(),
            current: value.current.into_iter().map(Into::into).collect(),
        }
    }
}

/// Optional trailing argument on the wasm diff entry points. Omitting it (or
/// omitting `textDiff`) is `"none"` — today's behavior, computing nothing.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct DiffOptions {
    #[serde(default)]
    text_diff: TextDiffMode,
}

impl From<DiffOptions> for NativeTextDiffMode {
    fn from(value: DiffOptions) -> Self {
        value.text_diff.into()
    }
}

// ---------------------------------------------------------------------------
// HTML option enums — input-only wire contract (JS → native). The native
// enums (`usfm_onion::html`) deliberately derive no `Serialize`; these wire
// types carry the serde/tsify boundary shape and convert one way into native.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

// ---------------------------------------------------------------------------
// Packed-verification DTOs — the receipt a non-Rust materializer stands on.
//
// The packed bytes deliberately store no `markerMetadata`/`structural`: both
// are pure functions of the marker name under the section's verified
// `catalog_stamp`, so the Rust decoder recalls them from the registry instead.
// A JS decoder has no registry, so the verifier hands back the resolved rows
// for the marker forms this book actually references — 25-31 for a whole
// scripture book, against hundreds of thousands of tokens. No token and no
// finding object is in this receipt.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct PackedMarkerDescriptor {
    pub name: String,
    pub nested: bool,
    pub marker_metadata: MarkerMetadata,
    pub structural: StructuralMarkerInfo,
}

/// One book's attestation of what the Rust trust boundary checked.
///
/// The three `u64` integrity values are 16-character lowercase hex strings.
/// They are an audit record, never an input to a check: a consumer of this
/// receipt has already been told the bytes are certified, and there is no JS
/// hash implementation to re-derive them with.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct PackedBookReceipt {
    pub book: String,
    pub source_len: u32,
    pub token_count: u32,
    pub finding_count: u32,
    /// True when token ids are `{book}-{index}` and the id column/dictionary are
    /// absent, so a materializer synthesizes them.
    pub positional_ids: bool,
    pub source_hash: String,
    pub catalog_stamp: String,
    pub snapshot_id: String,
    /// Descriptor-ordinal order: the packed marker-descriptor-index column
    /// indexes straight into this list.
    pub descriptors: Vec<PackedMarkerDescriptor>,
}

/// The frozen [`crate::error::DecodeError`] set as a tagged boundary value.
///
/// Variant names and payloads are the same contract the Rust enum carries; the
/// tag exists so a TypeScript consumer can narrow instead of string-matching a
/// message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PackedDecodeError {
    Truncated,
    BadMagic,
    UnsupportedVersion { found: u16 },
    UnsupportedFlags { found: u32 },
    InvalidToc,
    InvalidSection,
    InvalidUtf8,
    InvalidDiscriminant,
    OffsetOverflow,
    TooManySids { found: u32 },
    ChecksumMismatch,
    CatalogMismatch,
    SourceLengthMismatch,
    SourceHashMismatch,
}

impl From<crate::error::DecodeError> for PackedDecodeError {
    fn from(value: crate::error::DecodeError) -> Self {
        use crate::error::DecodeError as E;
        match value {
            E::Truncated => Self::Truncated,
            E::BadMagic => Self::BadMagic,
            E::UnsupportedVersion { found } => Self::UnsupportedVersion { found },
            E::UnsupportedFlags { found } => Self::UnsupportedFlags { found },
            E::InvalidToc => Self::InvalidToc,
            E::InvalidSection => Self::InvalidSection,
            E::InvalidUtf8 => Self::InvalidUtf8,
            E::InvalidDiscriminant => Self::InvalidDiscriminant,
            E::OffsetOverflow => Self::OffsetOverflow,
            E::TooManySids { found } => Self::TooManySids { found },
            E::ChecksumMismatch => Self::ChecksumMismatch,
            E::CatalogMismatch => Self::CatalogMismatch,
            E::SourceLengthMismatch => Self::SourceLengthMismatch,
            E::SourceHashMismatch => Self::SourceHashMismatch,
        }
    }
}

/// A wasm-facing projection of [`crate::schema::SectionKind`] — only ever seen
/// nested inside [`PackedLayoutRefusal::DuplicateSection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub enum PackedSectionKind {
    Token,
    Finding,
}

impl From<crate::schema::SectionKind> for PackedSectionKind {
    fn from(value: crate::schema::SectionKind) -> Self {
        match value {
            crate::schema::SectionKind::Token => Self::Token,
            crate::schema::SectionKind::Finding => Self::Finding,
        }
    }
}

/// The frozen [`crate::error::LayoutRefusal`] set as a tagged boundary value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PackedLayoutRefusal {
    DuplicateSection { section_kind: PackedSectionKind },
    OrphanFindingSection,
    DuplicateField { field_id: u16 },
    FieldExtentMismatch { field_id: u16 },
    MissingRequiredField { field_id: u16 },
    CachedSectionUnreadable,
    CachedSectionMismatch,
    CachedSectionStampMismatch,
    PositionalIdConflict { field_id: u16 },
    SectionTooLarge,
    TooManyFields,
    TooManySections,
}

impl From<crate::error::LayoutRefusal> for PackedLayoutRefusal {
    fn from(value: crate::error::LayoutRefusal) -> Self {
        use crate::error::LayoutRefusal as R;
        match value {
            R::DuplicateSection { kind } => Self::DuplicateSection {
                section_kind: kind.into(),
            },
            R::OrphanFindingSection => Self::OrphanFindingSection,
            R::DuplicateField { field_id } => Self::DuplicateField { field_id },
            R::FieldExtentMismatch { field_id } => Self::FieldExtentMismatch { field_id },
            R::MissingRequiredField { field_id } => Self::MissingRequiredField { field_id },
            R::CachedSectionUnreadable => Self::CachedSectionUnreadable,
            R::CachedSectionMismatch => Self::CachedSectionMismatch,
            R::CachedSectionStampMismatch => Self::CachedSectionStampMismatch,
            R::PositionalIdConflict { field_id } => Self::PositionalIdConflict { field_id },
            R::SectionTooLarge => Self::SectionTooLarge,
            R::TooManyFields => Self::TooManyFields,
            R::TooManySections => Self::TooManySections,
        }
    }
}

/// The frozen [`crate::error::EncodeError`] set as a tagged boundary value.
///
/// Every variant here is a pathological-input safety net (a book past an
/// index ceiling, a synthetically-built token whose span does not bind, a
/// layout the writer's own reader would reject) rather than something a
/// normal publish hits — but the packed boundary's rule is refuse-typed-never-
/// panic regardless of how rare the refusal is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PackedEncodeError {
    TooManySids {
        book: String,
        found: u32,
    },
    UnrepresentablePayload {
        book: String,
        code: u8,
    },
    TooManyDescriptors {
        book: String,
        found: u32,
    },
    UnboundSpan {
        book: String,
        token_idx: u32,
    },
    InvalidSectionLayout {
        book: String,
        reason: PackedLayoutRefusal,
    },
    EmptyFix {
        book: String,
        code: u8,
    },
}

impl From<crate::error::EncodeError> for PackedEncodeError {
    fn from(value: crate::error::EncodeError) -> Self {
        use crate::error::EncodeError as E;
        match value {
            E::TooManySids { book, found } => Self::TooManySids {
                book: book.as_str().to_string(),
                found,
            },
            E::UnrepresentablePayload { book, code } => Self::UnrepresentablePayload {
                book: book.as_str().to_string(),
                code,
            },
            E::TooManyDescriptors { book, found } => Self::TooManyDescriptors {
                book: book.as_str().to_string(),
                found,
            },
            E::UnboundSpan { book, token_idx } => Self::UnboundSpan {
                book: book.as_str().to_string(),
                token_idx,
            },
            E::InvalidSectionLayout { book, reason } => Self::InvalidSectionLayout {
                book: book.as_str().to_string(),
                reason: reason.into(),
            },
            E::EmptyFix { book, code } => Self::EmptyFix {
                book: book.as_str().to_string(),
                code,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::map_marker_info;
    use usfm_onion::markers::marker_info;

    fn marker_value(marker: &str) -> Value {
        serde_json::to_value(map_marker_info(marker_info(marker))).unwrap()
    }

    fn assert_field(marker: &str, field: &str, expected: Value) {
        let value = marker_value(marker);
        assert_eq!(
            value.get(field),
            Some(&expected),
            "{marker}.{field} serialized unexpectedly: {value}"
        );
    }

    #[test]
    fn marker_info_serializes_js_wire_contract_without_wasm() {
        assert_field("p", "paragraphCategory", json!("body"));
        assert_field("q1", "paragraphCategory", json!("poetry"));
        assert_field("s1", "paragraphCategory", json!("section"));
        assert_field("li1", "paragraphCategory", json!("list"));
        assert_field("mt1", "paragraphCategory", json!("title"));
        assert_field("pb", "paragraphCategory", json!("other"));

        assert_field("c", "payload", json!("numberRange"));
        assert_field("v", "payload", json!("numberRange"));
        assert_field("id", "payload", json!("bookCode"));

        assert_field("nd", "closingBehavior", json!("requiredExplicit"));
    }

    #[test]
    fn absent_optional_marker_fields_are_omitted() {
        let value = marker_value("nd");
        assert_eq!(value.get("payload"), None);
        assert_eq!(value.get("paragraphCategory"), None);
    }

    /// Regression for the v0.0.9 desktop nightly crash (`missing field 'span'`).
    /// A frontend-shaped token with NO `span` field must still deserialize into
    /// `Token` — `span` is `Option<Span>` with a skip-if, so its absence on the
    /// wire is the contract, not an error. This is the exact seam the editor's
    /// hand-written parallel copy drifted on; single-sourcing `Token` here
    /// makes both consumers share this shape.
    #[test]
    fn token_deserializes_without_span_field() {
        // Fixture: the minimal frontend token shape, span omitted entirely.
        let json = r#"{"id":"GEN-3","kind":"text","source":"In the beginning"}"#;
        let token: super::Token = serde_json::from_str(json).expect("token must deserialize");
        assert!(token.span.is_none(), "absent span must decode to None");
        assert_eq!(token.id, "GEN-3");
        assert_eq!(token.source, "In the beginning");
        assert!(matches!(token.kind, super::TokenKind::Text));
    }

    #[test]
    fn decode_attr_value_unescapes_quote_and_backslash() {
        use super::decode_attr_value;
        assert_eq!(decode_attr_value("plain"), "plain");
        assert_eq!(decode_attr_value("a\\\"b"), "a\"b");
        assert_eq!(decode_attr_value("a\\\\b"), "a\\b");
    }

    /// Boundary-enum wire contract. These were hand-mirrored in the wasm crate
    /// before being single-sourced here; the exact JS-facing strings are the
    /// contract the golden outputs enforce, pinned at the dto layer too. Lint
    /// enums are kebab-case (matching native serde); diff/html enums are the
    /// camelCase/lowercase the boundary sees, which for the diff enums differs
    /// from the native PascalCase `Serialize`.
    #[test]
    fn boundary_enums_serialize_the_js_wire_strings() {
        use super::{
            CoveredSide, DecisionStatus, DecisionUnitKind, HtmlCallerScope, HtmlCallerStyle,
            HtmlNoteMode, LintCategory, LintCode, LintSeverity, MergeSide, SlotRole,
        };

        // Lint: kebab-case.
        assert_eq!(
            serde_json::to_value(LintCategory::Numbering).unwrap(),
            json!("numbering")
        );
        assert_eq!(
            serde_json::to_value(LintSeverity::Warning).unwrap(),
            json!("warning")
        );
        assert_eq!(
            serde_json::to_value(LintCode::IdMarkerNotAtFileStart).unwrap(),
            json!("id-marker-not-at-file-start")
        );
        assert_eq!(
            serde_json::to_value(LintCode::InvalidBookCode).unwrap(),
            json!("invalid-book-code")
        );
        assert_eq!(
            serde_json::to_value(LintCode::BookCodeNotUppercase).unwrap(),
            json!("book-code-not-uppercase")
        );

        // Diff: camelCase wire (native serializes these PascalCase — different form).
        assert_eq!(
            serde_json::to_value(SlotRole::BaselineOnly).unwrap(),
            json!("baselineOnly")
        );
        assert_eq!(
            serde_json::to_value(DecisionUnitKind::Coalesced).unwrap(),
            json!("coalesced")
        );
        assert_eq!(
            serde_json::to_value(DecisionStatus::Moved).unwrap(),
            json!("moved")
        );
        assert_eq!(
            serde_json::to_value(MergeSide::Baseline).unwrap(),
            json!("baseline")
        );
        assert_eq!(
            serde_json::to_value(CoveredSide::Current).unwrap(),
            json!("current")
        );

        // Html input config.
        assert_eq!(
            serde_json::to_value(HtmlNoteMode::Extracted).unwrap(),
            json!("extracted")
        );
        assert_eq!(
            serde_json::to_value(HtmlCallerStyle::AlphaLower).unwrap(),
            json!("alphaLower")
        );
        assert_eq!(
            serde_json::to_value(HtmlCallerScope::VerseSequential).unwrap(),
            json!("verseSequential")
        );
    }

    /// The input-direction enums must round-trip from their wire string into the
    /// native type (the conversion the wasm boundary relies on).
    #[test]
    fn input_enums_deserialize_and_convert_into_native() {
        use super::{HtmlNoteMode, LintCode, MergeSide};
        use usfm_onion::html::HtmlNoteMode as NativeHtmlNoteMode;
        use usfm_onion::lint::LintCode as NativeLintCode;

        let mode: HtmlNoteMode = serde_json::from_value(json!("inline")).unwrap();
        assert!(matches!(
            NativeHtmlNoteMode::from(mode),
            NativeHtmlNoteMode::Inline
        ));

        let code: LintCode = serde_json::from_value(json!("unclosed-marker")).unwrap();
        assert!(matches!(
            NativeLintCode::from(code),
            NativeLintCode::UnclosedMarker
        ));

        let side: MergeSide = serde_json::from_value(json!("current")).unwrap();
        assert!(matches!(
            usfm_onion::diff::MergeSide::from(side),
            usfm_onion::diff::MergeSide::Current
        ));
    }

    /// Text-diff wire strings (Gate 4). `TextDiffMode`/`TextDiffRunKind` are
    /// camelCase like the other diff enums; single-word variants read the
    /// same either way, so this pins the exact set of accepted strings.
    #[test]
    fn text_diff_enums_serialize_the_js_wire_strings() {
        use super::{TextDiffMode, TextDiffRunKind};

        assert_eq!(
            serde_json::to_value(TextDiffMode::None).unwrap(),
            json!("none")
        );
        assert_eq!(
            serde_json::to_value(TextDiffMode::Words).unwrap(),
            json!("words")
        );
        assert_eq!(
            serde_json::to_value(TextDiffMode::Chars).unwrap(),
            json!("chars")
        );

        assert_eq!(
            serde_json::to_value(TextDiffRunKind::Unchanged).unwrap(),
            json!("unchanged")
        );
        assert_eq!(
            serde_json::to_value(TextDiffRunKind::Added).unwrap(),
            json!("added")
        );
        assert_eq!(
            serde_json::to_value(TextDiffRunKind::Removed).unwrap(),
            json!("removed")
        );
    }

    /// `TextDiffMode` is an input (the caller picks a granularity), so it must
    /// round-trip from its wire string into the native enum the same way
    /// `MergeSide`/`LintCode` do above.
    #[test]
    fn text_diff_mode_deserializes_and_converts_into_native() {
        use super::TextDiffMode;
        use usfm_onion::diff::TextDiffMode as NativeTextDiffMode;

        let mode: TextDiffMode = serde_json::from_value(json!("words")).unwrap();
        assert!(matches!(
            NativeTextDiffMode::from(mode),
            NativeTextDiffMode::Words
        ));

        let mode: TextDiffMode = serde_json::from_value(json!("chars")).unwrap();
        assert!(matches!(
            NativeTextDiffMode::from(mode),
            NativeTextDiffMode::Chars
        ));

        let mode: TextDiffMode = serde_json::from_value(json!("none")).unwrap();
        assert!(matches!(
            NativeTextDiffMode::from(mode),
            NativeTextDiffMode::None
        ));
    }

    /// `DiffOptions` omitted entirely (empty object) must resolve to `"none"`
    /// — the additive/back-compatible contract the plan requires: an absent
    /// option must be byte-identical to today's no-text-diff behavior.
    #[test]
    fn diff_options_default_and_omitted_field_resolve_to_none_mode() {
        use super::DiffOptions;
        use usfm_onion::diff::TextDiffMode as NativeTextDiffMode;

        let options: DiffOptions = serde_json::from_value(json!({})).unwrap();
        assert!(matches!(
            NativeTextDiffMode::from(options),
            NativeTextDiffMode::None
        ));
        assert!(matches!(
            NativeTextDiffMode::from(DiffOptions::default()),
            NativeTextDiffMode::None
        ));
    }

    /// `UnitTextDiff`/`TextDiffRun` are outputs, so the guard is a native →
    /// wire mapping test rather than a round trip: the exact runs a native
    /// `unit_text_diff` call produces must survive the DTO conversion.
    #[test]
    fn unit_text_diff_maps_native_runs_into_wire_runs() {
        use super::{TextDiffRun, TextDiffRunKind, UnitTextDiff};
        use usfm_onion::diff::{
            TextDiffRun as NativeTextDiffRun, TextDiffRunKind as NativeTextDiffRunKind,
            UnitTextDiff as NativeUnitTextDiff,
        };

        let native = NativeUnitTextDiff {
            baseline: vec![NativeTextDiffRun {
                text: "heaven".to_string(),
                kind: NativeTextDiffRunKind::Removed,
            }],
            current: vec![NativeTextDiffRun {
                text: "heavens".to_string(),
                kind: NativeTextDiffRunKind::Added,
            }],
        };
        let wire: UnitTextDiff = native.into();
        assert_eq!(
            wire.baseline,
            vec![TextDiffRun {
                text: "heaven".to_string(),
                kind: TextDiffRunKind::Removed
            }]
        );
        assert_eq!(
            wire.current,
            vec![TextDiffRun {
                text: "heavens".to_string(),
                kind: TextDiffRunKind::Added
            }]
        );
    }
}

#[cfg(test)]
mod owned_token_from_dto_tests {
    use super::*;
    use usfm_onion::parse::parse;

    fn dto(kind: TokenKind, source: &str) -> Token {
        Token {
            id: "editor-1".to_string(),
            kind,
            source: source.to_string(),
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
            attribute_source: None,
            attribute_offset: None,
        }
    }

    /// The inbound and outbound halves of the token boundary must describe the same
    /// token. Both id lanes are exercised: parse-assigned positional ids, and a
    /// caller's own opaque ids — the editor's case, and the one where nothing but
    /// the DTO's own `id` field can carry the identity.
    #[test]
    fn a_token_survives_the_round_trip_in_both_id_lanes() {
        let source = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1-2 a \\w gracious|lemma=\"grace\"\\w* \\add \\+add b\\+add*\\add* c\n";
        let parsed = parse(source);
        let owned: Vec<OwnedToken> = parsed.tokens.iter().map(OwnedToken::from_parsed).collect();
        // The fixture has to actually exercise the payload-bearing kinds, or the
        // round trip proves only that plain text survives.
        assert!(owned.iter().any(|token| token.book_code().is_some()));
        assert!(owned.iter().any(|token| token.number_info().is_some()));
        assert!(owned.iter().any(|token| token.nested()));
        assert!(owned.iter().any(|token| token.attribute_list().is_some()));
        assert!(owned.iter().any(|token| token.sid().is_some()));

        for relabel in [false, true] {
            for (index, native) in parsed.tokens.iter().enumerate() {
                let mut wire = Token::from(native);
                if relabel {
                    wire.id = format!("editor-{index}");
                }
                let rebuilt = owned_token_from_dto(&wire, index as u32)
                    .unwrap_or_else(|error| panic!("token {index} must convert: {error}"));

                let expected = &owned[index];
                assert_eq!(rebuilt.kind(), expected.kind());
                assert_eq!(rebuilt.source(), expected.source());
                assert_eq!(rebuilt.sid(), expected.sid());
                assert_eq!(rebuilt.parsed_sid(), expected.parsed_sid());
                assert_eq!(rebuilt.marker_name(), expected.marker_name());
                assert_eq!(rebuilt.nested(), expected.nested());
                assert_eq!(
                    rebuilt.book_code().map(|code| (&code.code, code.is_valid)),
                    expected.book_code().map(|code| (&code.code, code.is_valid))
                );
                assert_eq!(rebuilt.number_info(), expected.number_info());
                assert_eq!(rebuilt.attribute_list(), expected.attribute_list());
                assert_eq!(rebuilt.attributes(), expected.attributes());
                assert_eq!(
                    rebuilt.id().as_str(),
                    if relabel {
                        format!("editor-{index}")
                    } else {
                        expected.id().as_str().to_string()
                    }
                );
            }
        }
    }

    /// Shape problems belong to this layer; legality belongs to core's builder. Each
    /// refusal must name the one that actually applies, so a caller can tell "you
    /// sent me half a fact" from "those facts cannot coexist".
    #[test]
    fn hostile_dtos_are_refused_by_the_layer_that_owns_the_complaint() {
        let mut idless = dto(TokenKind::Text, "x");
        idless.id = String::new();
        assert_eq!(
            owned_token_from_dto(&idless, 7),
            Err(TokenInputError::MissingId { token_idx: 7 })
        );

        // Half a book code: shape, not legality.
        let mut half = dto(TokenKind::BookCode, "GEN");
        half.book_code = Some("GEN".to_string());
        assert_eq!(
            owned_token_from_dto(&half, 1),
            Err(TokenInputError::IncompleteBookCode { token_idx: 1 })
        );
        let mut other_half = dto(TokenKind::BookCode, "GEN");
        other_half.book_code_valid = Some(true);
        assert_eq!(
            owned_token_from_dto(&other_half, 1),
            Err(TokenInputError::IncompleteBookCode { token_idx: 1 })
        );

        // A book-code payload on a text token: core's verdict, carried verbatim.
        let mut wrong_kind = dto(TokenKind::Text, "GEN");
        wrong_kind.book_code = Some("GEN".to_string());
        wrong_kind.book_code_valid = Some(true);
        assert!(matches!(
            owned_token_from_dto(&wrong_kind, 2),
            Err(TokenInputError::Illegal {
                token_idx: 2,
                reason: usfm_onion::token::TokenBuildError::UnexpectedPayload { .. }
            })
        ));

        // An anchor this library would never have written.
        let mut bad_sid = dto(TokenKind::Text, "x");
        bad_sid.sid = Some("GENESIS 1:1".to_string());
        assert!(matches!(
            owned_token_from_dto(&bad_sid, 3),
            Err(TokenInputError::Illegal {
                reason: usfm_onion::token::TokenBuildError::UnresolvableSid { .. },
                ..
            })
        ));

        // An attribute list on an end marker, which structurally cannot hold one.
        let mut closing = dto(TokenKind::EndMarker, "\\w*");
        closing.marker = Some("w".to_string());
        closing.attribute_source = Some("|lemma=\"grace\"".to_string());
        assert!(matches!(
            owned_token_from_dto(&closing, 4),
            Err(TokenInputError::Illegal {
                reason: usfm_onion::token::TokenBuildError::UnexpectedPayload { .. },
                ..
            })
        ));

        // A marker-bearing kind with no marker name.
        assert!(matches!(
            owned_token_from_dto(&dto(TokenKind::Marker, "\\p "), 5),
            Err(TokenInputError::Illegal {
                reason: usfm_onion::token::TokenBuildError::MissingPayload { .. },
                ..
            })
        ));
    }

    /// Three shapes the corpus gate caught and the hand-written cases had not: an
    /// escaped value, a bare default attribute whose own value is quoted, and a
    /// malformed slice the parser could only read as one default attribute. Each
    /// must come back spelled exactly as the source spelled it, because the
    /// emitter writes that spelling back out.
    #[test]
    fn an_attributes_value_comes_back_spelled_as_the_source_spelled_it() {
        for source in [
            // Escapes: the outbound DTO decodes them for JS, so rebuilding has to
            // recover the source spelling rather than re-encode the logical string.
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\fig caption|alt=\"He said: \\\"hi\\\"\" src=\"x.jpg\"\\fig* b\n",
            // A default attribute whose value carries quotes of its own.
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\w word|\"quoted\"\\w* b\n",
            // Malformed: one unquoted key swallows the rest as a default value.
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\w word|lemma= strong=\"l\"\\w* b\n",
            // Keyed and empty, which must stay empty rather than becoming absent.
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\w word|lemma=\"\"\\w* b\n",
        ] {
            let parsed = parse(source);
            let attributed = parsed
                .tokens
                .iter()
                .position(|token| token.attributes().is_some_and(|list| !list.is_empty()))
                .unwrap_or_else(|| panic!("fixture must carry attributes: {source:?}"));
            let native = &parsed.tokens[attributed];
            let expected = OwnedToken::from_parsed(native);
            let rebuilt =
                owned_token_from_dto(&Token::from(native), attributed as u32).expect("converts");
            assert_eq!(
                rebuilt.attributes(),
                expected.attributes(),
                "attribute spelling changed for {source:?}"
            );
            // Placement is the other half of byte-losslessness, and it has a DTO
            // field for exactly this reason.
            assert_eq!(rebuilt.attribute_offset(), expected.attribute_offset());
            assert_eq!(rebuilt.attribute_list(), expected.attribute_list());
        }
    }

    /// Attribute presence is three distinct facts, and each has to survive the
    /// boundary as itself: no list at all, an empty verbatim list, and a structured
    /// list with no verbatim spelling.
    #[test]
    fn attribute_presence_permutations_survive_the_boundary() {
        let mut none = dto(TokenKind::Marker, "\\w ");
        none.marker = Some("w".to_string());
        let mut empty = none.clone();
        empty.attribute_source = Some(String::new());
        let mut structured = none.clone();
        structured.attributes = vec![AttributeItem {
            span: None,
            text: "lemma=\"grace\"".to_string(),
            key: "lemma".to_string(),
            value: "grace".to_string(),
            is_default: false,
        }];

        let none = owned_token_from_dto(&none, 0).expect("converts");
        let empty = owned_token_from_dto(&empty, 0).expect("converts");
        let structured = owned_token_from_dto(&structured, 0).expect("converts");
        assert_eq!(none.attribute_list(), None);
        assert!(none.attributes().is_empty());
        assert_eq!(empty.attribute_list(), Some(""));
        assert_eq!(structured.attribute_list(), None);
        assert_eq!(structured.attributes().len(), 1);
        // A boundary token states no remembered placement, so the emitter places
        // the list at the marker's closer rather than at a distance nobody gave.
        assert_eq!(structured.attribute_offset(), None);
    }
}
