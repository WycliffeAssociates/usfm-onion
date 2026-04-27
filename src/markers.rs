use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::marker_defs::{
    BlockBehavior, ClosingBehavior, InlineContext, MarkerDef, MarkerFamily, MarkerFamilyRole,
    NoteFamily, NoteSubkind, SpecContext, lookup_marker_def, marker_is_note_sub,
};
use crate::marker_defs_data::MARKER_SPECS;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MarkerInfo {
    pub kind: MarkerKind,
    #[allow(dead_code)]
    pub valid_in_note: bool,
}

impl MarkerInfo {
    const fn new(kind: MarkerKind) -> Self {
        Self {
            kind,
            valid_in_note: false,
        }
    }

    const fn note_sub(kind: MarkerKind) -> Self {
        Self {
            kind,
            valid_in_note: true,
        }
    }
}

pub fn lookup_marker(name: &str) -> MarkerInfo {
    if name.starts_with('z') {
        return MarkerInfo::new(MarkerKind::Unknown);
    }

    if let Some(def) = lookup_marker_def(name) {
        let kind = def.kind.to_marker_kind(name);
        if def.note_subkind.is_some() {
            return MarkerInfo::note_sub(kind);
        }
        return MarkerInfo::new(kind);
    }

    if is_table_cell_marker(name) {
        return MarkerInfo::new(MarkerKind::TableCell);
    }

    MarkerInfo::new(MarkerKind::Unknown)
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsfmMarkerInfo {
    pub marker: String,
    pub canonical: Option<String>,
    pub known: bool,
    pub deprecated: bool,
    pub category: MarkerCategory,
    pub kind: MarkerKind,
    pub family: Option<MarkerFamily>,
    pub family_role: Option<MarkerFamilyRole>,
    pub note_family: Option<MarkerNoteFamily>,
    pub note_subkind: Option<MarkerNoteSubkind>,
    pub inline_context: Option<MarkerInlineContext>,
    pub default_attribute: Option<String>,
    pub contexts: Vec<SpecContext>,
    pub block_behavior: Option<BlockBehavior>,
    pub closing_behavior: Option<ClosingBehavior>,
    pub source: Option<String>,
}

#[derive(Debug)]
pub struct UsfmMarkerCatalog {
    entries: Vec<UsfmMarkerInfo>,
    exact_index: HashMap<String, usize>,
}

impl UsfmMarkerCatalog {
    pub fn all(&self) -> &[UsfmMarkerInfo] {
        &self.entries
    }

    pub fn get(&self, marker: &str) -> Option<&UsfmMarkerInfo> {
        let canonical = lookup_marker_def(marker).map(|def| def.marker)?;
        self.exact_index
            .get(canonical)
            .and_then(|index| self.entries.get(*index))
    }

    pub fn contains(&self, marker: &str) -> bool {
        self.get(marker).is_some()
    }
}

pub fn marker_catalog() -> &'static UsfmMarkerCatalog {
    static CATALOG: OnceLock<UsfmMarkerCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let entries = MARKER_SPECS
            .iter()
            .filter_map(|spec| lookup_marker_def(spec.marker))
            .map(marker_def_to_info)
            .collect::<Vec<_>>();
        let exact_index = entries
            .iter()
            .enumerate()
            .map(|(index, info)| (info.marker.clone(), index))
            .collect::<HashMap<_, _>>();
        UsfmMarkerCatalog {
            entries,
            exact_index,
        }
    })
}

pub fn marker_info(marker: &str) -> UsfmMarkerInfo {
    if let Some(def) = lookup_marker_def(marker) {
        let mut info = marker_def_to_info(def);
        info.marker = marker.to_string();
        info.category = classify_category(marker, info.kind);
        info
    } else {
        let looked_up = lookup_marker(marker);
        UsfmMarkerInfo {
            marker: marker.to_string(),
            canonical: None,
            known: false,
            deprecated: false,
            category: classify_category(marker, looked_up.kind),
            kind: looked_up.kind,
            family: None,
            family_role: None,
            note_family: None,
            note_subkind: None,
            inline_context: None,
            default_attribute: None,
            contexts: Vec::new(),
            block_behavior: None,
            closing_behavior: None,
            source: None,
        }
    }
}

pub fn is_known_marker(marker: &str) -> bool {
    lookup_marker(marker).kind != MarkerKind::Unknown
}

fn marker_def_to_info(def: MarkerDef) -> UsfmMarkerInfo {
    UsfmMarkerInfo {
        marker: def.marker.to_string(),
        canonical: Some(def.marker.to_string()),
        known: true,
        deprecated: def.deprecated,
        category: classify_category(def.marker, def.kind.to_marker_kind(def.marker)),
        kind: def.kind.to_marker_kind(def.marker),
        family: def.family,
        family_role: Some(def.family_role),
        note_family: def.note_family.map(map_note_family),
        note_subkind: def.note_subkind.map(map_note_subkind),
        inline_context: def.inline_context.map(map_inline_context),
        default_attribute: def.default_attribute.map(ToOwned::to_owned),
        contexts: def.contexts.to_vec(),
        block_behavior: Some(def.block_behavior),
        closing_behavior: Some(def.closing_behavior),
        source: Some(def.source.to_string()),
    }
}

fn classify_category(marker: &str, kind: MarkerKind) -> MarkerCategory {
    if is_document_marker(marker) {
        return MarkerCategory::Document;
    }
    if marker_is_note_sub(marker) {
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

fn is_document_marker(marker: &str) -> bool {
    matches!(marker.strip_prefix('+').unwrap_or(marker), "id" | "usfm")
}

fn is_table_cell_marker(name: &str) -> bool {
    let prefixes = ["th", "thr", "thc", "tc", "tcr", "tcc"];
    prefixes.iter().any(|prefix| {
        name.strip_prefix(prefix)
            .map(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
            })
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::{MarkerCategory, MarkerNoteFamily, is_known_marker, marker_catalog, marker_info};

    #[test]
    fn note_container_and_note_submarker_are_distinct() {
        assert!(is_known_marker("f"));
        assert_eq!(
            marker_info("f").note_family,
            Some(MarkerNoteFamily::Footnote)
        );
        assert_eq!(marker_info("ft").category, MarkerCategory::NoteSubmarker);
        assert_eq!(marker_info("nd").category, MarkerCategory::Character);
    }

    #[test]
    fn catalog_contains_core_markers() {
        let catalog = marker_catalog();
        assert!(catalog.contains("p"));
        assert!(catalog.contains("f"));
        assert!(catalog.contains("ft"));
    }
}
