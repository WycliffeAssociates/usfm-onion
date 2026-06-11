use serde::{Deserialize, Serialize};

use usfm_onion::marker_defs::{
    BlockBehavior as NativeBlockBehavior, ClosingBehavior as NativeClosingBehavior,
    InlineContext as NativeInlineContext, MarkerFamily as NativeMarkerFamily,
    MarkerFamilyRole as NativeMarkerFamilyRole, MarkerPayload as NativeMarkerPayload,
    NoteFamily as NativeNoteFamily, NoteSubkind as NativeNoteSubkind,
    ParagraphCategory as NativeParagraphCategory, SpecContext as NativeSpecContext,
};
use usfm_onion::markers::{
    MarkerCategory as NativeMarkerCategory, MarkerKind as NativeMarkerKind,
    UsfmMarkerInfo as NativeUsfmMarkerInfo,
};

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
}
