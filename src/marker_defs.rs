#![allow(dead_code)]

use serde::Serialize;

use crate::markers::MarkerKind;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MarkerId(&'static str);

impl MarkerId {
    pub const fn new(marker: &'static str) -> Self {
        Self(marker)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

pub const MARKER_C: MarkerId = MarkerId::new("c");
pub const MARKER_V: MarkerId = MarkerId::new("v");
pub const MARKER_ID: MarkerId = MarkerId::new("id");
pub const MARKER_PERIPH: MarkerId = MarkerId::new("periph");
pub const MARKER_REM: MarkerId = MarkerId::new("rem");
pub const MARKER_CAT: MarkerId = MarkerId::new("cat");
pub const MARKER_F: MarkerId = MarkerId::new("f");
pub const MARKER_FE: MarkerId = MarkerId::new("fe");
pub const MARKER_EF: MarkerId = MarkerId::new("ef");
pub const MARKER_X: MarkerId = MarkerId::new("x");
pub const MARKER_EX: MarkerId = MarkerId::new("ex");
pub const MARKER_FT: MarkerId = MarkerId::new("ft");
pub const MARKER_FK: MarkerId = MarkerId::new("fk");
pub const MARKER_XQ: MarkerId = MarkerId::new("xq");
pub const MARKER_FV: MarkerId = MarkerId::new("fv");
pub const MARKER_REF: MarkerId = MarkerId::new("ref");
pub const MARKER_JMP: MarkerId = MarkerId::new("jmp");
pub const MARKER_W: MarkerId = MarkerId::new("w");
pub const MARKER_XT: MarkerId = MarkerId::new("xt");
pub const MARKER_FIG: MarkerId = MarkerId::new("fig");
pub const MARKER_ESB: MarkerId = MarkerId::new("esb");
pub const MARKER_ESBE: MarkerId = MarkerId::new("esbe");
pub const MARKER_TR: MarkerId = MarkerId::new("tr");
pub const MARKER_PN: MarkerId = MarkerId::new("pn");
pub const MARKER_PNG: MarkerId = MarkerId::new("png");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum SpecMarkerKind {
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

impl SpecMarkerKind {
    pub fn to_marker_kind(self, marker: &str) -> MarkerKind {
        match self {
            Self::Paragraph => MarkerKind::Paragraph,
            Self::Character => MarkerKind::Character,
            Self::Note => MarkerKind::Note,
            Self::Chapter => MarkerKind::Chapter,
            Self::Verse => MarkerKind::Verse,
            Self::Milestone => {
                if marker.ends_with("-e") {
                    MarkerKind::MilestoneEnd
                } else {
                    MarkerKind::MilestoneStart
                }
            }
            Self::Figure => MarkerKind::Figure,
            Self::Sidebar => {
                if marker == "esbe" {
                    MarkerKind::SidebarEnd
                } else {
                    MarkerKind::SidebarStart
                }
            }
            Self::Periph => MarkerKind::Periph,
            Self::Meta => MarkerKind::Meta,
            Self::TableRow => MarkerKind::TableRow,
            Self::TableCell => MarkerKind::TableCell,
            Self::Header => MarkerKind::Header,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MarkerFamily {
    Footnote,
    CrossReference,
    SectionParagraph,
    ListParagraph,
    TableCell,
    Milestone,
    Sidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MarkerFamilyRole {
    Canonical,
    NumberedVariant,
    NestedVariant,
    MilestoneStart,
    MilestoneEnd,
    Alias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NoteFamily {
    Footnote,
    CrossReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NoteSubkind {
    Structural,
    StructuralKeepsNestedCharsOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum InlineContext {
    Para,
    Section,
    List,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BlockBehavior {
    None,
    Paragraph(InlineContext),
    TableRow,
    TableCell,
    SidebarStart,
    SidebarEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StructuralMarkerInfo {
    pub scope_kind: StructuralScopeKind,
    pub inline_context: Option<InlineContext>,
    pub note_context: Option<SpecContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ClosingBehavior {
    None,
    RequiredExplicit,
    OptionalExplicitUntilNoteEnd,
    SelfClosingMilestone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerSpec {
    pub marker: &'static str,
    pub kind: SpecMarkerKind,
    pub contexts: &'static [SpecContext],
    pub deprecated: bool,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NormalizedMarkerRef<'a> {
    pub raw: &'a str,
    pub canonical: &'static str,
    pub nested: bool,
    pub family_role: MarkerFamilyRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerDef {
    pub id: MarkerId,
    pub marker: &'static str,
    pub kind: SpecMarkerKind,
    pub contexts: &'static [SpecContext],
    pub deprecated: bool,
    pub family: Option<MarkerFamily>,
    pub family_role: MarkerFamilyRole,
    pub default_attribute: Option<&'static str>,
    pub note_family: Option<NoteFamily>,
    pub note_subkind: Option<NoteSubkind>,
    pub inline_context: Option<InlineContext>,
    pub block_behavior: BlockBehavior,
    pub closing_behavior: ClosingBehavior,
    pub source: &'static str,
}

#[path = "marker_defs_data.rs"]
mod marker_defs_data;

pub(crate) use marker_defs_data::MARKER_SPECS;

fn exact_spec_index() -> &'static HashMap<&'static str, &'static MarkerSpec> {
    static INDEX: OnceLock<HashMap<&'static str, &'static MarkerSpec>> = OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .map(|spec| (spec.marker, spec))
            .collect::<HashMap<_, _>>()
    })
}

fn table_cell_spec_index() -> &'static HashMap<&'static str, &'static MarkerSpec> {
    static INDEX: OnceLock<HashMap<&'static str, &'static MarkerSpec>> = OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .filter(|spec| spec.kind == SpecMarkerKind::TableCell)
            .map(|spec| (spec.marker, spec))
            .collect::<HashMap<_, _>>()
    })
}

pub fn normalized_marker(marker: &str) -> Option<NormalizedMarkerRef<'_>> {
    let nested = marker.starts_with('+');
    let normalized = marker.strip_prefix('+').unwrap_or(marker);
    let spec = lookup_spec_marker(normalized)?;
    let family_role = marker_family_role(normalized, spec.marker);
    Some(NormalizedMarkerRef {
        raw: marker,
        canonical: spec.marker,
        nested,
        family_role,
    })
}

pub fn lookup_spec_marker(marker: &str) -> Option<&'static MarkerSpec> {
    let normalized = marker.strip_prefix('+').unwrap_or(marker);

    if let Some(spec) = exact_spec_index().get(normalized).copied() {
        return Some(spec);
    }

    if let Some(base) = normalized
        .strip_suffix("-s")
        .or_else(|| normalized.strip_suffix("-e"))
    {
        let milestone_base = base.trim_end_matches(|ch: char| ch.is_ascii_digit());
        if let Some(spec) = exact_spec_index().get(milestone_base).copied()
            && spec.kind == SpecMarkerKind::Milestone
        {
            return Some(spec);
        }
    }

    if let Some(table_cell_base) = table_cell_base(normalized)
        && let Some(spec) = table_cell_spec_index().get(table_cell_base).copied()
    {
        return Some(spec);
    }

    if normalized == "esbe" {
        return exact_spec_index().get("esb").copied();
    }

    if let Some(base) = numbered_marker_base(normalized) {
        return exact_spec_index().get(base).copied();
    }

    None
}

pub fn lookup_marker_def(marker: &str) -> Option<MarkerDef> {
    let normalized = normalized_marker(marker)?;
    let spec = lookup_spec_marker(normalized.canonical)?;
    let note_family = marker_note_family(spec.marker);
    let note_subkind = marker_note_subkind(spec.marker);
    let inline_context = marker_inline_context(spec.marker);
    Some(MarkerDef {
        id: MarkerId::new(spec.marker),
        marker: spec.marker,
        kind: spec.kind,
        contexts: spec.contexts,
        deprecated: spec.deprecated,
        family: marker_family_for(spec.marker, spec.kind),
        family_role: normalized.family_role,
        default_attribute: derive_default_attribute(spec.marker),
        note_family,
        note_subkind,
        inline_context,
        block_behavior: derive_block_behavior(spec.kind, inline_context, spec.marker),
        closing_behavior: derive_closing_behavior(spec.kind, note_subkind),
        source: spec.source,
    })
}

pub fn lookup_marker_metadata(
    marker: &str,
) -> Option<(&'static str, SpecMarkerKind, Option<MarkerFamily>)> {
    if let Some(metadata) = fast_marker_metadata(marker) {
        return Some(metadata);
    }

    let normalized = normalized_marker(marker)?;
    let spec = lookup_spec_marker(normalized.canonical)?;
    Some((
        spec.marker,
        spec.kind,
        marker_family_for(spec.marker, spec.kind),
    ))
}

pub fn lookup_marker_id(marker: &str) -> Option<MarkerId> {
    normalized_marker(marker).map(|normalized| MarkerId::new(normalized.canonical))
}

pub fn spec_marker_kind(marker: &str) -> Option<MarkerKind> {
    lookup_marker_def(marker).map(|def| def.kind.to_marker_kind(marker))
}

pub fn marker_default_attribute(marker: &str) -> Option<&'static str> {
    lookup_marker_def(marker).and_then(|def| def.default_attribute)
}

pub fn marker_family(marker: &str) -> Option<MarkerFamily> {
    lookup_marker_def(marker).and_then(|def| def.family)
}

pub fn marker_note_family(marker: &str) -> Option<NoteFamily> {
    let marker = marker.strip_prefix('+').unwrap_or(marker);
    match marker {
        "f" | "fe" | "ef" => Some(NoteFamily::Footnote),
        "x" | "ex" => Some(NoteFamily::CrossReference),
        _ if marker.starts_with('f') => Some(NoteFamily::Footnote),
        _ if marker.starts_with('x') => Some(NoteFamily::CrossReference),
        _ => None,
    }
}

pub fn marker_note_subkind(marker: &str) -> Option<NoteSubkind> {
    let marker = marker.strip_prefix('+').unwrap_or(marker);
    match marker {
        "fr" | "fq" | "fqa" | "fl" | "fw" | "fp" | "fv" | "fdc" | "fm" | "xo" | "xop" | "xk"
        | "xt" | "xta" | "xot" | "xnt" | "xdc" => Some(NoteSubkind::Structural),
        "ft" | "fk" | "xq" => Some(NoteSubkind::StructuralKeepsNestedCharsOpen),
        _ => None,
    }
}

pub fn marker_inline_context(marker: &str) -> Option<InlineContext> {
    let marker = marker.strip_prefix('+').unwrap_or(marker);
    if marker == "tr" || table_cell_base(marker).is_some() {
        return Some(InlineContext::Table);
    }
    if is_list_marker_name(marker) {
        return Some(InlineContext::List);
    }
    if is_section_marker_name(marker) {
        return Some(InlineContext::Section);
    }

    lookup_spec_marker(marker).and_then(|spec| {
        (spec.kind == SpecMarkerKind::Paragraph
            && spec.contexts.iter().any(|ctx| {
                matches!(
                    ctx,
                    SpecContext::ChapterContent | SpecContext::PeripheralContent
                )
            }))
        .then_some(InlineContext::Para)
    })
}

pub fn marker_block_behavior(marker: &str) -> BlockBehavior {
    lookup_marker_def(marker)
        .map(|def| def.block_behavior)
        .unwrap_or(BlockBehavior::None)
}

pub fn marker_paragraph_supports_verse(marker: &str) -> bool {
    if marker.strip_prefix('+').unwrap_or(marker) == "lit" {
        return true;
    }
    lookup_marker_def(marker)
        .map(|def| {
            def.kind == SpecMarkerKind::Paragraph
                && matches!(
                    def.inline_context,
                    Some(InlineContext::Para | InlineContext::List)
                )
                && def.contexts.iter().any(|ctx| {
                    matches!(
                        ctx,
                        SpecContext::ChapterContent | SpecContext::PeripheralContent
                    )
                })
        })
        .unwrap_or(false)
}

pub fn marker_is_heading_bridge(marker: &str) -> bool {
    let marker = marker.strip_prefix('+').unwrap_or(marker);
    marker == "s" || marker.starts_with('s')
}

pub fn marker_note_context(marker: &str) -> Option<SpecContext> {
    match marker_note_family(marker) {
        Some(NoteFamily::Footnote) => Some(SpecContext::Footnote),
        Some(NoteFamily::CrossReference) => Some(SpecContext::CrossReference),
        None => None,
    }
}

pub fn marker_is_note_container(marker: &str) -> bool {
    lookup_marker_def(marker)
        .map(|def| def.kind == SpecMarkerKind::Note)
        .unwrap_or(false)
}

pub fn marker_forbidden_in_note_context(marker: &str) -> bool {
    matches!(lookup_marker_id(marker), Some(MARKER_PN | MARKER_PNG))
}

pub fn marker_allows_context(marker: &str, context: SpecContext) -> bool {
    lookup_spec_marker(marker)
        .map(|spec| spec.contexts.contains(&context))
        .unwrap_or(false)
}

pub fn marker_allows_effective_context(marker: &str, context: SpecContext) -> bool {
    marker_allows_context(marker, context)
        || (context == SpecContext::PeripheralContent
            && marker_allows_context(marker, SpecContext::ChapterContent))
        || marker_allows_embedded_char_context(marker, context)
}

pub fn marker_is_note_sub(marker: &str) -> bool {
    lookup_marker_def(marker)
        .map(|def| {
            def.kind == SpecMarkerKind::Character
                && def
                    .contexts
                    .iter()
                    .any(|ctx| matches!(ctx, SpecContext::Footnote | SpecContext::CrossReference))
        })
        .unwrap_or(false)
}

pub fn structural_marker_info(marker: &str, kind: Option<SpecMarkerKind>) -> StructuralMarkerInfo {
    if let Some(info) = fast_structural_marker_info(marker, kind) {
        return info;
    }

    match kind {
        None => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Unknown,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Header) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Header,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Paragraph) => match marker_block_behavior(marker) {
            BlockBehavior::TableRow => StructuralMarkerInfo {
                scope_kind: StructuralScopeKind::TableRow,
                inline_context: Some(InlineContext::Table),
                note_context: None,
            },
            BlockBehavior::TableCell => StructuralMarkerInfo {
                scope_kind: StructuralScopeKind::TableCell,
                inline_context: Some(InlineContext::Table),
                note_context: None,
            },
            BlockBehavior::SidebarStart | BlockBehavior::SidebarEnd => StructuralMarkerInfo {
                scope_kind: StructuralScopeKind::Sidebar,
                inline_context: None,
                note_context: None,
            },
            BlockBehavior::Paragraph(inline_context) => StructuralMarkerInfo {
                scope_kind: StructuralScopeKind::Block,
                inline_context: Some(inline_context),
                note_context: None,
            },
            BlockBehavior::None => StructuralMarkerInfo {
                scope_kind: StructuralScopeKind::Block,
                inline_context: None,
                note_context: None,
            },
        },
        Some(SpecMarkerKind::Note) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Note,
            inline_context: None,
            note_context: marker_note_context(marker),
        },
        Some(SpecMarkerKind::Character) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Character,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Milestone) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Milestone,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Chapter) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Chapter,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Verse) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Verse,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Periph) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Periph,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Meta) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Meta,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::Sidebar) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Sidebar,
            inline_context: None,
            note_context: None,
        },
        Some(SpecMarkerKind::TableRow) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableRow,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        Some(SpecMarkerKind::TableCell) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableCell,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        Some(SpecMarkerKind::Figure) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: None,
            note_context: None,
        },
    }
}

fn fast_structural_marker_info(
    marker: &str,
    kind: Option<SpecMarkerKind>,
) -> Option<StructuralMarkerInfo> {
    let kind = kind?;
    let info = match kind {
        SpecMarkerKind::Header => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Header,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Note => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Note,
            inline_context: None,
            note_context: marker_note_context(marker),
        },
        SpecMarkerKind::Character => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Character,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Milestone => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Milestone,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Chapter => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Chapter,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Verse => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Verse,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Periph => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Periph,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Meta => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Meta,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Sidebar => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Sidebar,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::TableRow => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableRow,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        SpecMarkerKind::TableCell => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableCell,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        SpecMarkerKind::Figure => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: None,
            note_context: None,
        },
        SpecMarkerKind::Paragraph => return fast_paragraph_structural_info(marker),
    };

    Some(info)
}

fn fast_paragraph_structural_info(marker: &str) -> Option<StructuralMarkerInfo> {
    let info = if marker == "tr" {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableRow,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        }
    } else if table_cell_base(marker).is_some() {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableCell,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        }
    } else if matches!(marker, "esb" | "esbe") {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Sidebar,
            inline_context: None,
            note_context: None,
        }
    } else if is_list_marker_name(marker) {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: Some(InlineContext::List),
            note_context: None,
        }
    } else if is_section_marker_name(marker) {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: Some(InlineContext::Section),
            note_context: None,
        }
    } else if is_para_marker_name(marker) {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: Some(InlineContext::Para),
            note_context: None,
        }
    } else if is_non_inline_paragraph_marker_name(marker) {
        StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: None,
            note_context: None,
        }
    } else {
        return None;
    };

    Some(info)
}

fn marker_allows_embedded_char_context(marker: &str, context: SpecContext) -> bool {
    if !matches!(context, SpecContext::Footnote | SpecContext::CrossReference) {
        return false;
    }

    let Some(spec) = lookup_spec_marker(marker) else {
        return false;
    };

    spec.kind == SpecMarkerKind::Character
        && spec.contexts.iter().any(|ctx| {
            matches!(
                ctx,
                SpecContext::Section | SpecContext::Para | SpecContext::List | SpecContext::Table
            )
        })
}

fn table_cell_base(marker: &str) -> Option<&str> {
    for prefix in ["th", "thr", "thc", "tc", "tcr", "tcc"] {
        if let Some(suffix) = marker.strip_prefix(prefix)
            && !suffix.is_empty()
            && suffix.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
        {
            return Some(prefix);
        }
    }
    None
}

fn numbered_marker_base(marker: &str) -> Option<&str> {
    let split_at = marker.find(|ch: char| ch.is_ascii_digit())?;
    let (base, suffix) = marker.split_at(split_at);
    if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let value = suffix.parse::<usize>().ok()?;
    let max = match base {
        "h" => 3,
        "toc" | "toca" => 3,
        "is" | "ili" | "imte" | "mte" => 2,
        "liv" => 5,
        "imt" | "mt" | "q" | "s" | "io" | "li" => 4,
        "iq" | "mi" | "ph" | "pi" | "qm" | "ms" | "lim" | "sd" => 3,
        _ => return None,
    };

    (value <= max).then_some(base)
}

fn marker_family_for(marker: &str, kind: SpecMarkerKind) -> Option<MarkerFamily> {
    if matches!(marker_note_family(marker), Some(NoteFamily::Footnote)) {
        return Some(MarkerFamily::Footnote);
    }
    if matches!(marker_note_family(marker), Some(NoteFamily::CrossReference)) {
        return Some(MarkerFamily::CrossReference);
    }
    if kind == SpecMarkerKind::Milestone {
        return Some(MarkerFamily::Milestone);
    }
    if kind == SpecMarkerKind::Sidebar {
        return Some(MarkerFamily::Sidebar);
    }
    if kind == SpecMarkerKind::TableCell {
        return Some(MarkerFamily::TableCell);
    }
    if is_list_marker_name(marker) {
        return Some(MarkerFamily::ListParagraph);
    }
    if is_section_marker_name(marker) {
        return Some(MarkerFamily::SectionParagraph);
    }
    None
}

fn fast_marker_metadata(
    marker: &str,
) -> Option<(&'static str, SpecMarkerKind, Option<MarkerFamily>)> {
    match marker {
        "id" => Some(("id", SpecMarkerKind::Header, None)),
        "h" => Some(("h", SpecMarkerKind::Paragraph, None)),
        "c" => Some(("c", SpecMarkerKind::Chapter, None)),
        "v" => Some(("v", SpecMarkerKind::Verse, None)),
        "p" => Some(("p", SpecMarkerKind::Paragraph, None)),
        "m" => Some(("m", SpecMarkerKind::Paragraph, None)),
        "b" => Some(("b", SpecMarkerKind::Paragraph, None)),
        "r" => Some(("r", SpecMarkerKind::Paragraph, None)),
        "mt" => Some(("mt", SpecMarkerKind::Paragraph, None)),
        "mt1" => Some(("mt1", SpecMarkerKind::Paragraph, None)),
        "mt2" => Some(("mt2", SpecMarkerKind::Paragraph, None)),
        "mt3" => Some(("mt3", SpecMarkerKind::Paragraph, None)),
        "mt4" => Some(("mt4", SpecMarkerKind::Paragraph, None)),
        "s" => Some(("s", SpecMarkerKind::Paragraph, None)),
        "s1" => Some(("s1", SpecMarkerKind::Paragraph, None)),
        "s2" => Some(("s2", SpecMarkerKind::Paragraph, None)),
        "s3" => Some(("s3", SpecMarkerKind::Paragraph, None)),
        "s4" => Some(("s4", SpecMarkerKind::Paragraph, None)),
        "q" => Some(("q", SpecMarkerKind::Paragraph, None)),
        "q1" => Some(("q1", SpecMarkerKind::Paragraph, None)),
        "q2" => Some(("q2", SpecMarkerKind::Paragraph, None)),
        "q3" => Some(("q3", SpecMarkerKind::Paragraph, None)),
        "q4" => Some(("q4", SpecMarkerKind::Paragraph, None)),
        "f" | "fe" | "ef" => Some(("f", SpecMarkerKind::Note, Some(MarkerFamily::Footnote))),
        "x" | "ex" => Some((
            "x",
            SpecMarkerKind::Note,
            Some(MarkerFamily::CrossReference),
        )),
        "ft" => Some((
            "ft",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fr" => Some((
            "fr",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fq" => Some((
            "fq",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fqa" => Some((
            "fqa",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fk" => Some((
            "fk",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fl" => Some((
            "fl",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fw" => Some((
            "fw",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fp" => Some((
            "fp",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fv" => Some((
            "fv",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fdc" => Some((
            "fdc",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "fm" => Some((
            "fm",
            SpecMarkerKind::Character,
            Some(MarkerFamily::Footnote),
        )),
        "xo" => Some((
            "xo",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xop" => Some((
            "xop",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xk" => Some((
            "xk",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xq" => Some((
            "xq",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xt" => Some((
            "xt",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xta" => Some((
            "xta",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xot" => Some((
            "xot",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xnt" => Some((
            "xnt",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "xdc" => Some((
            "xdc",
            SpecMarkerKind::Character,
            Some(MarkerFamily::CrossReference),
        )),
        "w" => Some(("w", SpecMarkerKind::Character, None)),
        "jmp" => Some(("jmp", SpecMarkerKind::Character, None)),
        "ref" => Some(("ref", SpecMarkerKind::Character, None)),
        _ => None,
    }
}

fn marker_family_role(marker: &str, canonical: &'static str) -> MarkerFamilyRole {
    if marker == "esbe" {
        return MarkerFamilyRole::Alias;
    }
    if marker.ends_with("-e") {
        return MarkerFamilyRole::MilestoneEnd;
    }
    if marker.ends_with("-s") {
        return MarkerFamilyRole::MilestoneStart;
    }
    if marker != canonical {
        return MarkerFamilyRole::NumberedVariant;
    }
    MarkerFamilyRole::Canonical
}

fn derive_closing_behavior(
    kind: SpecMarkerKind,
    note_subkind: Option<NoteSubkind>,
) -> ClosingBehavior {
    match kind {
        SpecMarkerKind::Milestone => ClosingBehavior::SelfClosingMilestone,
        SpecMarkerKind::Character => {
            if note_subkind.is_some() {
                ClosingBehavior::OptionalExplicitUntilNoteEnd
            } else {
                ClosingBehavior::RequiredExplicit
            }
        }
        SpecMarkerKind::Note => ClosingBehavior::RequiredExplicit,
        _ => ClosingBehavior::None,
    }
}

fn derive_block_behavior(
    kind: SpecMarkerKind,
    inline_context: Option<InlineContext>,
    marker: &str,
) -> BlockBehavior {
    match kind {
        SpecMarkerKind::Paragraph => inline_context
            .map(BlockBehavior::Paragraph)
            .unwrap_or(BlockBehavior::None),
        SpecMarkerKind::TableRow => BlockBehavior::TableRow,
        SpecMarkerKind::TableCell => BlockBehavior::TableCell,
        SpecMarkerKind::Sidebar => {
            if marker == "esbe" {
                BlockBehavior::SidebarEnd
            } else {
                BlockBehavior::SidebarStart
            }
        }
        _ => BlockBehavior::None,
    }
}

fn derive_default_attribute(marker: &str) -> Option<&'static str> {
    match marker {
        "w" => Some("lemma"),
        "rb" => Some("gloss"),
        "jmp" | "xt" => Some("link-href"),
        "ref" => Some("loc"),
        "fig" => Some("src"),
        _ => {
            let base = marker
                .strip_suffix("-s")
                .or_else(|| marker.strip_suffix("-e"));
            if let Some(base) = base {
                let base = base.trim_end_matches(|ch: char| ch.is_ascii_digit());
                if base == "qt" {
                    return Some("who");
                }
            }
            None
        }
    }
}

fn is_list_marker_name(marker: &str) -> bool {
    matches!(marker, "lf" | "lh")
        || marker == "li"
        || marker.starts_with("li")
        || marker == "lim"
        || marker.starts_with("lim")
}

fn is_section_marker_name(marker: &str) -> bool {
    matches!(marker, "cd" | "cl" | "d" | "mr" | "r" | "sp" | "sr")
        || marker == "ms"
        || marker.starts_with("ms")
        || marker == "s"
        || (marker.starts_with('s') && marker != "sts")
        || marker == "sd"
        || marker.starts_with("sd")
}

fn is_para_marker_name(marker: &str) -> bool {
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
            | "mi"
            | "nb"
            | "pc"
            | "ph"
            | "phi"
            | "pi"
            | "pii"
            | "b"
            | "q"
            | "qr"
            | "qc"
            | "qa"
            | "qm"
            | "qd"
            | "lh"
            | "lf"
            | "lit"
    ) || marker.starts_with("q")
        || marker.starts_with("pi")
        || marker.starts_with("ph")
        || marker.starts_with("mi")
}

fn is_non_inline_paragraph_marker_name(marker: &str) -> bool {
    matches!(
        marker,
        "h" | "toc"
            | "toca"
            | "imt"
            | "imte"
            | "is"
            | "ip"
            | "ipi"
            | "im"
            | "imi"
            | "imq"
            | "ipq"
            | "ipr"
            | "ib"
            | "ili"
            | "iot"
            | "io"
            | "iex"
            | "mt"
            | "mte"
            | "ms"
            | "mr"
            | "cl"
            | "cd"
            | "s"
            | "sr"
            | "r"
            | "d"
            | "sp"
            | "restore"
    ) || marker.starts_with("mt")
        || marker.starts_with("mte")
        || marker.starts_with("is")
        || marker.starts_with("ili")
        || marker.starts_with("io")
}

#[cfg(test)]
mod tests {
    use super::{
        SpecContext, SpecMarkerKind, lookup_spec_marker, marker_allows_context,
        marker_allows_effective_context,
    };
    use std::path::PathBuf;

    #[test]
    fn official_markers_exist_in_spec_lookup() {
        for marker in ["p", "ip", "f", "w", "ca"] {
            assert!(lookup_spec_marker(marker).is_some(), "missing {marker}");
        }
    }

    #[test]
    fn undocumented_s5_is_not_in_spec_lookup() {
        assert!(lookup_spec_marker("s5").is_none());
    }

    #[test]
    fn table_cells_resolve_from_numbered_variants() {
        let spec = lookup_spec_marker("tc2").expect("tc2 should resolve");
        assert_eq!(spec.kind, SpecMarkerKind::TableCell);
        assert!(marker_allows_context("tc2", SpecContext::Table));
    }

    #[test]
    fn list_value_chars_resolve_from_numbered_variants() {
        let spec = lookup_spec_marker("liv1").expect("liv1 should resolve");
        assert_eq!(spec.kind, SpecMarkerKind::Character);
        assert!(marker_allows_context("liv1", SpecContext::List));
    }

    #[test]
    fn imte_uses_its_own_marker_metadata() {
        let spec = lookup_spec_marker("imte").expect("imte should resolve");
        assert_eq!(spec.kind, SpecMarkerKind::Paragraph);
        assert!(marker_allows_context("imte", SpecContext::BookIntroduction));
        assert!(spec.source.ends_with("markers/para/imte.adoc"));
    }

    #[test]
    fn peripheral_content_reuses_chapter_content_markers() {
        assert!(marker_allows_effective_context(
            "p",
            SpecContext::PeripheralContent
        ));
        assert!(marker_allows_effective_context(
            "s1",
            SpecContext::PeripheralContent
        ));
        assert!(marker_allows_effective_context(
            "tr",
            SpecContext::PeripheralContent
        ));
        assert!(marker_allows_effective_context(
            "esb",
            SpecContext::PeripheralContent
        ));
    }

    #[test]
    fn ordinary_character_markup_is_allowed_in_notes_via_embedded_char_semantics() {
        assert!(marker_allows_effective_context("nd", SpecContext::Footnote));
        assert!(marker_allows_effective_context(
            "nd",
            SpecContext::CrossReference
        ));
    }

    #[test]
    fn pi_numbered_variants_resolve_to_chapter_content() {
        assert!(marker_allows_context("pi1", SpecContext::ChapterContent));
        assert!(marker_allows_context("pi2", SpecContext::ChapterContent));
        assert!(marker_allows_context("pi3", SpecContext::ChapterContent));
    }

    #[test]
    fn generated_specs_reference_existing_tcdocs_files_when_checkout_is_present() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let docs_root = repo_root.join("repos_to_compare/tcdocs-main");
        if !docs_root.exists() {
            return;
        }

        for spec in super::MARKER_SPECS {
            let source = repo_root.join(spec.source);
            assert!(source.exists(), "missing source {}", source.display());
        }
    }
}
