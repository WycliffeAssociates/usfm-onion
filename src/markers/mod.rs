use serde::{Deserialize, Serialize};

use crate::internal::marker_defs::{
    InlineContext, NoteFamily, NoteSubkind, lookup_marker_def, marker_default_attribute,
    marker_inline_context, marker_is_note_container, marker_is_note_sub,
};
use crate::internal::markers::{MarkerKind, lookup_marker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerNoteFamily {
    Footnote,
    CrossReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerNoteSubkind {
    Structural,
    StructuralKeepsNestedCharsOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerInlineContext {
    Para,
    Section,
    List,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerInfo {
    pub marker: String,
    pub canonical: Option<String>,
    pub known: bool,
    pub deprecated: bool,
    pub category: MarkerCategory,
    pub note_family: Option<MarkerNoteFamily>,
    pub note_subkind: Option<MarkerNoteSubkind>,
    pub inline_context: Option<MarkerInlineContext>,
    pub default_attribute: Option<String>,
}

pub fn marker_info(marker: &str) -> MarkerInfo {
    let info = lookup_marker(marker);
    let def = lookup_marker_def(marker);
    let note_family = def.and_then(|def| def.note_family).map(map_note_family);
    let note_subkind = def.and_then(|def| def.note_subkind).map(map_note_subkind);
    let inline_context = marker_inline_context(marker).map(map_inline_context);
    let canonical = def.map(|def| def.marker.to_string());
    let deprecated = def.is_some_and(|def| def.deprecated);
    let default_attribute = marker_default_attribute(marker).map(ToOwned::to_owned);

    MarkerInfo {
        marker: marker.to_string(),
        canonical,
        known: info.kind != MarkerKind::Unknown,
        deprecated,
        category: classify_category(marker, info.kind),
        note_family,
        note_subkind,
        inline_context,
        default_attribute,
    }
}

pub fn is_known_marker(marker: &str) -> bool {
    lookup_marker(marker).kind != MarkerKind::Unknown
}

pub fn is_document_marker(marker: &str) -> bool {
    matches!(marker.strip_prefix('+').unwrap_or(marker), "id" | "usfm")
}

pub fn is_paragraph_marker(marker: &str) -> bool {
    matches!(lookup_marker(marker).kind, MarkerKind::Paragraph)
}

pub fn is_body_paragraph_marker(marker: &str) -> bool {
    matches!(
        marker.strip_prefix('+').unwrap_or(marker),
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

pub fn is_poetry_marker(marker: &str) -> bool {
    matches!(
        marker.strip_prefix('+').unwrap_or(marker),
        "q" | "q1" | "q2" | "q3" | "q4" | "qr" | "qc" | "qa" | "qm" | "qm1" | "qm2"
            | "qm3" | "qd"
    )
}

pub fn is_note_container(marker: &str) -> bool {
    marker_is_note_container(marker)
}

pub fn is_note_submarker(marker: &str) -> bool {
    marker_is_note_sub(marker)
}

pub fn is_character_marker(marker: &str) -> bool {
    matches!(lookup_marker(marker).kind, MarkerKind::Character) || is_note_submarker(marker)
}

pub fn is_regular_character_marker(marker: &str) -> bool {
    matches!(lookup_marker(marker).kind, MarkerKind::Character) && !is_note_submarker(marker)
}

pub fn note_marker_family(marker: &str) -> Option<MarkerNoteFamily> {
    lookup_marker_def(marker)
        .and_then(|def| def.note_family)
        .map(map_note_family)
}

pub fn all_markers() -> Vec<String> {
    crate::internal::marker_defs::MARKER_SPECS
        .iter()
        .map(|spec| spec.marker.to_string())
        .collect()
}

pub fn paragraph_markers() -> Vec<String> {
    crate::internal::marker_defs::MARKER_SPECS
        .iter()
        .filter(|spec| matches!(spec.kind, crate::internal::marker_defs::SpecMarkerKind::Paragraph))
        .map(|spec| spec.marker.to_string())
        .collect()
}

pub fn note_markers() -> Vec<String> {
    crate::internal::marker_defs::MARKER_SPECS
        .iter()
        .filter(|spec| matches!(spec.kind, crate::internal::marker_defs::SpecMarkerKind::Note))
        .map(|spec| spec.marker.to_string())
        .collect()
}

pub fn note_submarkers() -> Vec<String> {
    crate::internal::marker_defs::MARKER_SPECS
        .iter()
        .filter(|spec| crate::internal::marker_defs::marker_note_subkind(spec.marker).is_some())
        .map(|spec| spec.marker.to_string())
        .collect()
}

pub fn character_markers() -> Vec<String> {
    crate::internal::marker_defs::MARKER_SPECS
        .iter()
        .filter(|spec| {
            matches!(spec.kind, crate::internal::marker_defs::SpecMarkerKind::Character)
        })
        .map(|spec| spec.marker.to_string())
        .collect()
}

fn classify_category(marker: &str, kind: MarkerKind) -> MarkerCategory {
    if is_document_marker(marker) {
        return MarkerCategory::Document;
    }
    if is_note_submarker(marker) {
        return MarkerCategory::NoteSubmarker;
    }
    match kind {
        MarkerKind::Paragraph => MarkerCategory::Paragraph,
        MarkerKind::Note => MarkerCategory::NoteContainer,
        MarkerKind::Character => MarkerCategory::Character,
        MarkerKind::Header => MarkerCategory::Header,
        MarkerKind::Chapter => MarkerCategory::Chapter,
        MarkerKind::Verse => MarkerCategory::Verse,
        MarkerKind::MilestoneStart => MarkerCategory::MilestoneStart,
        MarkerKind::MilestoneEnd => MarkerCategory::MilestoneEnd,
        MarkerKind::SidebarStart => MarkerCategory::SidebarStart,
        MarkerKind::SidebarEnd => MarkerCategory::SidebarEnd,
        MarkerKind::Figure => MarkerCategory::Figure,
        MarkerKind::Meta => MarkerCategory::Meta,
        MarkerKind::Periph => MarkerCategory::Periph,
        MarkerKind::TableRow => MarkerCategory::TableRow,
        MarkerKind::TableCell => MarkerCategory::TableCell,
        MarkerKind::Unknown => MarkerCategory::Unknown,
    }
}

fn map_note_family(family: NoteFamily) -> MarkerNoteFamily {
    match family {
        NoteFamily::Footnote => MarkerNoteFamily::Footnote,
        NoteFamily::CrossReference => MarkerNoteFamily::CrossReference,
    }
}

fn map_note_subkind(subkind: NoteSubkind) -> MarkerNoteSubkind {
    match subkind {
        NoteSubkind::Structural => MarkerNoteSubkind::Structural,
        NoteSubkind::StructuralKeepsNestedCharsOpen => {
            MarkerNoteSubkind::StructuralKeepsNestedCharsOpen
        }
    }
}

fn map_inline_context(context: InlineContext) -> MarkerInlineContext {
    match context {
        InlineContext::Para => MarkerInlineContext::Para,
        InlineContext::Section => MarkerInlineContext::Section,
        InlineContext::List => MarkerInlineContext::List,
        InlineContext::Table => MarkerInlineContext::Table,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MarkerCategory, MarkerNoteFamily, all_markers, is_note_container, is_note_submarker,
        is_regular_character_marker, marker_info, note_marker_family,
    };

    #[test]
    fn note_container_and_note_submarker_are_distinct() {
        assert!(is_note_container("f"));
        assert_eq!(note_marker_family("f"), Some(MarkerNoteFamily::Footnote));
        assert!(is_note_submarker("ft"));
        assert!(!is_regular_character_marker("ft"));
        assert!(is_regular_character_marker("nd"));
    }

    #[test]
    fn marker_info_distinguishes_regular_char_and_note_submarker() {
        assert_eq!(marker_info("nd").category, MarkerCategory::Character);
        assert_eq!(marker_info("ft").category, MarkerCategory::NoteSubmarker);
        assert_eq!(marker_info("f").category, MarkerCategory::NoteContainer);
    }

    #[test]
    fn all_markers_contains_core_markers() {
        let markers = all_markers();
        assert!(markers.iter().any(|marker| marker == "p"));
        assert!(markers.iter().any(|marker| marker == "f"));
        assert!(markers.iter().any(|marker| marker == "ft"));
    }
}
