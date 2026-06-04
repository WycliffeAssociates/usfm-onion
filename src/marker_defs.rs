//! Spec-faithful queries over the USFM marker data table.
//!
//! This module owns the *definition-level* types and predicate helpers used
//! to ask "what does the spec say about this marker?":
//!
//! - [`MarkerDefKind`] — the kind of a marker as defined by the spec
//!   (`Paragraph`, `Character`, `Note`, `Milestone`, `Sidebar`, ...). At the
//!   definition level there is no Start/End split: `qt` is one milestone,
//!   `esb` is one sidebar; the start/end split happens at the *instance*
//!   level (see [`crate::markers::MarkerKind`]).
//! - [`SpecContext`], [`MarkerFamily`], [`NoteFamily`], [`NoteSubkind`],
//!   [`InlineContext`], [`BlockBehavior`], [`ClosingBehavior`],
//!   [`StructuralScopeKind`] — supporting spec-level enums.
//! - [`lookup_marker_def`], [`lookup_spec_marker`], [`structural_marker_info`],
//!   `marker_*` predicate helpers — the query surface that html/usj/lint/format
//!   consume.
//!
//! The static data table lives in [`crate::marker_defs_data`]. The
//! consumer-friendly catalog (with resolved instance-level kinds and
//! categorization) lives in [`crate::markers`].

#![allow(dead_code)]

use serde::Serialize;

use crate::markers::MarkerKind;
use crate::whitespace::{
    FormatWhitespacePreference, StructuralWhitespaceRequirement, WhitespaceFormatCategory,
};
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

/// The kind of a marker as defined by the USFM spec (definition-level).
///
/// At the definition level a milestone marker like `qt` is one `Milestone`
/// regardless of whether it appears in `qt-s` (start) or `qt-e` (end) form;
/// likewise `esb` is one `Sidebar` even though `esb` opens it and `esbe`
/// closes it. The start/end split is an *instance-level* concern and lives
/// on [`crate::markers::MarkerKind`], the public resolved kind that consumers
/// query when they need to know which side of a milestone or sidebar a
/// specific token represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl MarkerDefKind {
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

impl StructuralScopeKind {
    /// Single source of truth for "does opening a marker of this scope kind
    /// terminate an unclosed footnote/cross-reference?" Lossy export surfaces
    /// (vref, html, usj/usx via export_tree) must agree on this boundary or
    /// they will disagree about whether content after a missing `\f*` belongs
    /// inside the note. Lossless surfaces (token stream, CST) do not apply
    /// this rule; they preserve source exactly.
    ///
    /// Note: `export_tree` currently expresses the equivalent rule
    /// imperatively across `MarkerKind` branches rather than calling this
    /// predicate (its dispatch is `MarkerKind`-based and has richer
    /// per-branch logic). If this set changes, audit `export_tree::dispatch`
    /// as well.
    pub fn closes_unclosed_note(self) -> bool {
        matches!(
            self,
            StructuralScopeKind::Block
                | StructuralScopeKind::Chapter
                | StructuralScopeKind::Verse
                | StructuralScopeKind::Sidebar
                | StructuralScopeKind::TableRow
                | StructuralScopeKind::TableCell
                | StructuralScopeKind::Header
                | StructuralScopeKind::Periph
        )
    }
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

/// Category of a paragraph-kind marker per the USFM 3.2 paragraph index.
///
/// Required for every `MarkerDefKind::Paragraph` row; `None` for every other
/// kind. Drives context-validity rules that key off paragraph category — most
/// notably the USFM 3.2 rule that `\v` is not allowed inside paragraphs of
/// category `Section` or `Other`.
///
/// Source: <https://docs.usfm.bible/usfm/3.2/para/index.html>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ParagraphCategory {
    /// File identification (`ide`, `sts`, `rem`, `h`, `toc#`, `toca#`).
    Identification,
    /// Book introduction (`imt#`, `is#`, `ip*`, `im*`, `iq#`, `io#`, etc.).
    Introduction,
    /// Major and book-level titles (`mt#`, `mte#`, `cl`, `cd`).
    Title,
    /// Section headings and descriptors (`ms#`, `s#`, `sr`, `r`, `d`, `sp`, `sd#`, `mr`).
    Section,
    /// Body paragraphs (`p`, `m`, `po`, `pi#`, `mi#`, `nb`, `b`, `ph`, …).
    Body,
    /// Poetry paragraphs (`q#`, `qr`, `qc`, `qa`, `qm#`, `qd`).
    Poetry,
    /// Lists (`lh`, `li#`, `lf`, `lim#`).
    List,
    /// Tables — paragraph-shaped table markers (table rows themselves are
    /// `MarkerDefKind::TableRow`, not Paragraph).
    Table,
    /// Peripheral-only paragraphs (`p1`, `p2`, …).
    Peripheral,
    /// Anything that does not fit the spec categories (e.g. `pb`).
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerSpec {
    pub marker: &'static str,
    pub kind: MarkerDefKind,
    pub contexts: &'static [SpecContext],
    pub deprecated: bool,
    pub source: &'static str,
    /// Paragraph category per USFM 3.2 para index. Required for `Paragraph`
    /// kind rows; `None` for every other kind.
    pub paragraph_category: Option<ParagraphCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NormalizedMarkerRef<'a> {
    pub raw: &'a str,
    pub canonical: &'static str,
    pub nested: bool,
    pub family_role: MarkerFamilyRole,
}

/// Per-marker structural-whitespace rules.
///
/// Declarative companion to [`MarkerSpec`] / [`MarkerDef`] keyed by canonical
/// marker name. Each row says what whitespace is required at each position
/// around the marker (before the open marker, after the marker name, before
/// and after the closing form), what the formatter should insert when
/// normalizing, and which format-profile category the marker belongs to
/// (block markers get their own line in the code-editor profile; inline
/// markers never do).
///
/// See `whitespace.md` in the repo root for the source-of-truth rules,
/// and [`crate::whitespace`] for the requirement / preference / category
/// types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerWhitespace {
    pub marker: &'static str,
    pub required_before_open: StructuralWhitespaceRequirement,
    pub required_after_open_name: StructuralWhitespaceRequirement,
    pub required_before_close: StructuralWhitespaceRequirement,
    pub required_after_close: StructuralWhitespaceRequirement,
    pub format_preference_before_open: Option<FormatWhitespacePreference>,
    pub format_preference_after_open_name: Option<FormatWhitespacePreference>,
    pub category_for_profiles: WhitespaceFormatCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MarkerDef {
    pub id: MarkerId,
    pub marker: &'static str,
    pub kind: MarkerDefKind,
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
    pub paragraph_category: Option<ParagraphCategory>,
}

#[path = "marker_defs_data.rs"]
mod marker_defs_data;

pub(crate) use marker_defs_data::{MARKER_SPECS, MARKER_WHITESPACE};

/// Look up the structural-whitespace rule row for a marker by name.
///
/// Performs the same normalization as [`lookup_spec_marker`] (strips a
/// leading `+`, resolves milestone `-s`/`-e` suffixes back to the base
/// marker name, resolves numbered variants), so callers can pass a marker
/// as it appears in source.
///
/// Returns the explicit `MARKER_WHITESPACE` row when one exists,
/// otherwise synthesizes a sensible default from the marker's
/// `MarkerDefKind` (paragraph/character/note/milestone/…). USFM 3.1
/// defines whitespace shape per-category; this lookup mirrors that so
/// every known marker gets a profile without needing an explicit row.
pub fn lookup_marker_whitespace(marker: &str) -> Option<MarkerWhitespace> {
    let spec = lookup_spec_marker(marker)?;
    if let Some(explicit) = whitespace_index().get(spec.marker).copied() {
        return Some(*explicit);
    }
    Some(default_marker_whitespace_for(spec))
}

/// Per-marker default whitespace row. Used as a fallback when
/// `MARKER_WHITESPACE` doesn't carry an explicit row.
///
/// Drives off the USFM 3.1 marker categorization (`MarkerDefKind` plus
/// `ParagraphCategory`), so each spec category gets the shape the spec
/// actually specifies — not one generic paragraph fallback. The
/// explicit `MARKER_WHITESPACE` rows override these per-marker for
/// cases that deviate (e.g. `\v` permits inline whitespace before it;
/// `\f` does not require a newline).
///
/// Reference for the categories below:
/// <https://docs.usfm.bible/usfm/3.1/index.html>.
fn default_marker_whitespace_for(spec: &MarkerSpec) -> MarkerWhitespace {
    use crate::whitespace::FormatWhitespacePreference as Pref;
    use crate::whitespace::StructuralWhitespaceRequirement as Req;
    use crate::whitespace::WhitespaceFormatCategory as Cat;
    match spec.kind {
        // Paragraph-kind markers: the spec category decides the
        // after-name shape. Identification, title, and section
        // markers take a literal value after the marker name (HS
        // separation). Body/poetry/list/introduction/peripheral
        // markers carry paragraph content (tag-end delimiter).
        MarkerDefKind::Paragraph => {
            let (after_name, after_pref) = match spec.paragraph_category {
                Some(
                    ParagraphCategory::Identification
                    | ParagraphCategory::Title
                    | ParagraphCategory::Section,
                ) => (
                    Req::AtLeastOneHorizontalWhitespace,
                    Some(Pref::PreferSingleSpace),
                ),
                _ => (Req::TagEndDelimiter, None),
            };
            MarkerWhitespace {
                marker: spec.marker,
                required_before_open: Req::NewlineOrAnyWhitespaceBeforeMarker,
                required_after_open_name: after_name,
                required_before_close: Req::NotRequired,
                required_after_close: Req::NotRequired,
                format_preference_before_open: Some(Pref::PreferSingleNewline),
                format_preference_after_open_name: after_pref,
                category_for_profiles: Cat::Block,
            }
        }
        // `\id` is the only Header-kind marker in the catalog; it
        // takes the book code as its value (HS separation).
        MarkerDefKind::Header => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::NewlineOrAnyWhitespaceBeforeMarker,
            required_after_open_name: Req::AtLeastOneHorizontalWhitespace,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: Some(Pref::PreferSingleNewline),
            format_preference_after_open_name: Some(Pref::PreferSingleSpace),
            category_for_profiles: Cat::Block,
        },
        // Sidebar open / table row openers carry no value of their
        // own; the body content starts on the next non-WS token.
        MarkerDefKind::Sidebar | MarkerDefKind::TableRow => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::NewlineOrAnyWhitespaceBeforeMarker,
            required_after_open_name: Req::TagEndDelimiter,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: Some(Pref::PreferSingleNewline),
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Block,
        },
        // Peripheral and chapter take a following identifier / number
        // and therefore require at-least-one HS after the marker name.
        MarkerDefKind::Chapter | MarkerDefKind::Periph => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::NewlineOrAnyWhitespaceBeforeMarker,
            required_after_open_name: Req::AtLeastOneHorizontalWhitespace,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: Some(Pref::PreferSingleNewline),
            format_preference_after_open_name: Some(Pref::PreferSingleSpace),
            category_for_profiles: Cat::Block,
        },
        // Verse markers sit inline within a paragraph; the explicit
        // `v` row overrides this with `OptionalWhitespace` before, but
        // for any other Verse-kind marker (rare/none today) require WS.
        MarkerDefKind::Verse => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::AtLeastOneWhitespace,
            required_after_open_name: Req::AtLeastOneHorizontalWhitespace,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: Some(Pref::PreferSingleNewline),
            format_preference_after_open_name: Some(Pref::PreferSingleSpace),
            category_for_profiles: Cat::Block,
        },
        // Character markers run inline; explicit closing `\X*` form
        // means the open marker's after-name delimiter is tag-end
        // (whitespace, EOI, or `|` for attributes).
        MarkerDefKind::Character | MarkerDefKind::TableCell => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::OptionalWhitespace,
            required_after_open_name: Req::TagEndDelimiter,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: None,
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Inline,
        },
        // Note containers (footnote, cross-reference) match `\f` / `\x`.
        MarkerDefKind::Note => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::OptionalWhitespace,
            required_after_open_name: Req::TagEndDelimiter,
            required_before_close: Req::NotRequired,
            required_after_close: Req::OptionalWhitespace,
            format_preference_before_open: None,
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Inline,
        },
        // Milestones are terminated by `|` (attributes) or `\*` (self-close),
        // never by whitespace — the ms railroad
        // (docs.usfm.bible/usfm/3.1/ms/index.html) puts `${Hs}` (zero or
        // more) between the name and what follows. Covers all forms:
        // `\ts\*`, `\ts |sid="…"\*`, `\ts-s |sid="…"\*`, `\qt#-s`,
        // `\zaln-s`, …. (Corrected from TagEndDelimiter 2026-06-04;
        // TagEnd's whitespace branch is required-when-text-follows, which
        // is a char-marker shape, not a milestone shape. Note suffixed
        // forms like `ts-s`/`ts-e` resolve to their own spec entries and
        // take THIS default — explicit base-name rows don't reach them.)
        MarkerDefKind::Milestone => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::OptionalWhitespace,
            required_after_open_name: Req::OptionalHorizontalWhitespace,
            required_before_close: Req::NotRequired,
            required_after_close: Req::OptionalWhitespace,
            format_preference_before_open: None,
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Inline,
        },
        // Figure markers take `|attribute=...|...` shape after the
        // marker name; tag-end allows the `|` to follow directly.
        MarkerDefKind::Figure => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::OptionalWhitespace,
            required_after_open_name: Req::TagEndDelimiter,
            required_before_close: Req::NotRequired,
            required_after_close: Req::NotRequired,
            format_preference_before_open: None,
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Inline,
        },
        // Metadata markers (`cat`, `va`, `vp`, `ca`, `cp`) — the
        // explicit table covers the common ones with `OptionalHs`
        // after name; default to a tolerant inline shape for any
        // future addition.
        MarkerDefKind::Meta => MarkerWhitespace {
            marker: spec.marker,
            required_before_open: Req::OptionalWhitespace,
            required_after_open_name: Req::OptionalHorizontalWhitespace,
            required_before_close: Req::NotRequired,
            required_after_close: Req::OptionalWhitespace,
            format_preference_before_open: None,
            format_preference_after_open_name: None,
            category_for_profiles: Cat::Inline,
        },
    }
}

fn whitespace_index() -> &'static HashMap<&'static str, &'static MarkerWhitespace> {
    static INDEX: OnceLock<HashMap<&'static str, &'static MarkerWhitespace>> = OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_WHITESPACE
            .iter()
            .map(|row| (row.marker, row))
            .collect::<HashMap<_, _>>()
    })
}

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
            .filter(|spec| spec.kind == MarkerDefKind::TableCell)
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
            && spec.kind == MarkerDefKind::Milestone
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
        paragraph_category: spec.paragraph_category,
    })
}

pub fn lookup_marker_metadata(
    marker: &str,
) -> Option<(&'static str, MarkerDefKind, Option<MarkerFamily>)> {
    // Resolve canonical name + kind via the fast path when possible, but always
    // derive `family` from `marker_family_for` so this path cannot disagree with
    // the catalog (`lookup_marker_def`), which is the single source for family.
    let (canonical, kind) = match fast_marker_metadata(marker) {
        Some(metadata) => metadata,
        None => {
            let normalized = normalized_marker(marker)?;
            let spec = lookup_spec_marker(normalized.canonical)?;
            (spec.marker, spec.kind)
        }
    };
    Some((canonical, kind, marker_family_for(canonical, kind)))
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
        (spec.kind == MarkerDefKind::Paragraph
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
            def.kind == MarkerDefKind::Paragraph
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
        .map(|def| def.kind == MarkerDefKind::Note)
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
            def.kind == MarkerDefKind::Character
                && def
                    .contexts
                    .iter()
                    .any(|ctx| matches!(ctx, SpecContext::Footnote | SpecContext::CrossReference))
        })
        .unwrap_or(false)
}

pub fn structural_marker_info(marker: &str, kind: Option<MarkerDefKind>) -> StructuralMarkerInfo {
    if let Some(info) = fast_structural_marker_info(marker, kind) {
        return info;
    }

    match kind {
        None => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Unknown,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Header) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Header,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Paragraph) => match marker_block_behavior(marker) {
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
        Some(MarkerDefKind::Note) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Note,
            inline_context: None,
            note_context: marker_note_context(marker),
        },
        Some(MarkerDefKind::Character) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Character,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Milestone) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Milestone,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Chapter) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Chapter,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Verse) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Verse,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Periph) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Periph,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Meta) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Meta,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::Sidebar) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Sidebar,
            inline_context: None,
            note_context: None,
        },
        Some(MarkerDefKind::TableRow) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableRow,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        Some(MarkerDefKind::TableCell) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableCell,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        Some(MarkerDefKind::Figure) => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: None,
            note_context: None,
        },
    }
}

fn fast_structural_marker_info(
    marker: &str,
    kind: Option<MarkerDefKind>,
) -> Option<StructuralMarkerInfo> {
    let kind = kind?;
    let info = match kind {
        MarkerDefKind::Header => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Header,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Note => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Note,
            inline_context: None,
            note_context: marker_note_context(marker),
        },
        MarkerDefKind::Character => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Character,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Milestone => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Milestone,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Chapter => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Chapter,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Verse => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Verse,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Periph => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Periph,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Meta => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Meta,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Sidebar => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Sidebar,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::TableRow => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableRow,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        MarkerDefKind::TableCell => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::TableCell,
            inline_context: Some(InlineContext::Table),
            note_context: None,
        },
        MarkerDefKind::Figure => StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Block,
            inline_context: None,
            note_context: None,
        },
        MarkerDefKind::Paragraph => return fast_paragraph_structural_info(marker),
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

    spec.kind == MarkerDefKind::Character
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

fn marker_family_for(marker: &str, kind: MarkerDefKind) -> Option<MarkerFamily> {
    if matches!(marker_note_family(marker), Some(NoteFamily::Footnote)) {
        return Some(MarkerFamily::Footnote);
    }
    if matches!(marker_note_family(marker), Some(NoteFamily::CrossReference)) {
        return Some(MarkerFamily::CrossReference);
    }
    if kind == MarkerDefKind::Milestone {
        return Some(MarkerFamily::Milestone);
    }
    if kind == MarkerDefKind::Sidebar {
        return Some(MarkerFamily::Sidebar);
    }
    if kind == MarkerDefKind::TableCell {
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

/// Fast canonical-name + kind resolution for the hottest markers, skipping the
/// general normalization path. Deliberately does **not** return `family`:
/// `family` has a single source (`marker_family_for`), which
/// [`lookup_marker_metadata`] applies to this function's result. Returning a
/// hardcoded family here previously let the two paths disagree (e.g. `r` / `s`).
fn fast_marker_metadata(marker: &str) -> Option<(&'static str, MarkerDefKind)> {
    match marker {
        "id" => Some(("id", MarkerDefKind::Header)),
        "h" => Some(("h", MarkerDefKind::Paragraph)),
        "c" => Some(("c", MarkerDefKind::Chapter)),
        "v" => Some(("v", MarkerDefKind::Verse)),
        "p" => Some(("p", MarkerDefKind::Paragraph)),
        "m" => Some(("m", MarkerDefKind::Paragraph)),
        "b" => Some(("b", MarkerDefKind::Paragraph)),
        "r" => Some(("r", MarkerDefKind::Paragraph)),
        "mt" => Some(("mt", MarkerDefKind::Paragraph)),
        "mt1" => Some(("mt1", MarkerDefKind::Paragraph)),
        "mt2" => Some(("mt2", MarkerDefKind::Paragraph)),
        "mt3" => Some(("mt3", MarkerDefKind::Paragraph)),
        "mt4" => Some(("mt4", MarkerDefKind::Paragraph)),
        "s" => Some(("s", MarkerDefKind::Paragraph)),
        "s1" => Some(("s1", MarkerDefKind::Paragraph)),
        "s2" => Some(("s2", MarkerDefKind::Paragraph)),
        "s3" => Some(("s3", MarkerDefKind::Paragraph)),
        "s4" => Some(("s4", MarkerDefKind::Paragraph)),
        "q" => Some(("q", MarkerDefKind::Paragraph)),
        "q1" => Some(("q1", MarkerDefKind::Paragraph)),
        "q2" => Some(("q2", MarkerDefKind::Paragraph)),
        "q3" => Some(("q3", MarkerDefKind::Paragraph)),
        "q4" => Some(("q4", MarkerDefKind::Paragraph)),
        "f" | "fe" | "ef" => Some(("f", MarkerDefKind::Note)),
        "x" | "ex" => Some(("x", MarkerDefKind::Note)),
        "ft" => Some(("ft", MarkerDefKind::Character)),
        "fr" => Some(("fr", MarkerDefKind::Character)),
        "fq" => Some(("fq", MarkerDefKind::Character)),
        "fqa" => Some(("fqa", MarkerDefKind::Character)),
        "fk" => Some(("fk", MarkerDefKind::Character)),
        "fl" => Some(("fl", MarkerDefKind::Character)),
        "fw" => Some(("fw", MarkerDefKind::Character)),
        "fp" => Some(("fp", MarkerDefKind::Character)),
        "fv" => Some(("fv", MarkerDefKind::Character)),
        "fdc" => Some(("fdc", MarkerDefKind::Character)),
        "fm" => Some(("fm", MarkerDefKind::Character)),
        "xo" => Some(("xo", MarkerDefKind::Character)),
        "xop" => Some(("xop", MarkerDefKind::Character)),
        "xk" => Some(("xk", MarkerDefKind::Character)),
        "xq" => Some(("xq", MarkerDefKind::Character)),
        "xt" => Some(("xt", MarkerDefKind::Character)),
        "xta" => Some(("xta", MarkerDefKind::Character)),
        "xot" => Some(("xot", MarkerDefKind::Character)),
        "xnt" => Some(("xnt", MarkerDefKind::Character)),
        "xdc" => Some(("xdc", MarkerDefKind::Character)),
        "w" => Some(("w", MarkerDefKind::Character)),
        "jmp" => Some(("jmp", MarkerDefKind::Character)),
        "ref" => Some(("ref", MarkerDefKind::Character)),
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
    kind: MarkerDefKind,
    note_subkind: Option<NoteSubkind>,
) -> ClosingBehavior {
    match kind {
        MarkerDefKind::Milestone => ClosingBehavior::SelfClosingMilestone,
        MarkerDefKind::Character => {
            if note_subkind.is_some() {
                ClosingBehavior::OptionalExplicitUntilNoteEnd
            } else {
                ClosingBehavior::RequiredExplicit
            }
        }
        MarkerDefKind::Note => ClosingBehavior::RequiredExplicit,
        _ => ClosingBehavior::None,
    }
}

fn derive_block_behavior(
    kind: MarkerDefKind,
    inline_context: Option<InlineContext>,
    marker: &str,
) -> BlockBehavior {
    match kind {
        MarkerDefKind::Paragraph => inline_context
            .map(BlockBehavior::Paragraph)
            .unwrap_or(BlockBehavior::None),
        MarkerDefKind::TableRow => BlockBehavior::TableRow,
        MarkerDefKind::TableCell => BlockBehavior::TableCell,
        MarkerDefKind::Sidebar => {
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
    // Section markers are an explicit set, not "anything starting with s". The
    // former `starts_with('s')` clause wrongly swept in character markers
    // (`sc`, `sig`, `sls`, `sup`) and the identification marker `sts`. The `s`,
    // `ms`, and `sd` families carry numbered variants (`s1`..`s4`, `ms1`.., `sd1`..).
    matches!(marker, "cd" | "cl" | "d" | "mr" | "r" | "sp" | "sr")
        || matches!(marker, "s" | "s1" | "s2" | "s3" | "s4")
        || marker == "ms"
        || marker.starts_with("ms")
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
        MarkerDefKind, SpecContext, lookup_marker_whitespace, lookup_spec_marker,
        marker_allows_context, marker_allows_effective_context,
    };
    use crate::whitespace::{
        FormatWhitespacePreference, StructuralWhitespaceRequirement, WhitespaceFormatCategory,
    };

    #[test]
    fn marker_whitespace_lookup_resolves_canonical_and_variants() {
        let chapter = lookup_marker_whitespace("c").expect("c whitespace row should exist");
        assert_eq!(
            chapter.required_after_open_name,
            StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
        );
        assert_eq!(
            chapter.format_preference_before_open,
            Some(FormatWhitespacePreference::PreferSingleNewline)
        );
        assert_eq!(
            chapter.category_for_profiles,
            WhitespaceFormatCategory::Block
        );

        let nested =
            lookup_marker_whitespace("+f").expect("+ prefix should resolve to canonical f row");
        assert_eq!(nested.marker, "f");
        assert_eq!(
            nested.category_for_profiles,
            WhitespaceFormatCategory::Inline
        );

        assert!(
            lookup_marker_whitespace("zzzzz").is_none(),
            "unknown marker should return None"
        );

        // `\h` is Paragraph kind / Identification category — no
        // explicit `MARKER_WHITESPACE` row, but the category-driven
        // default gives it newline-before + at-least-one-HS after
        // (since `\h` takes a string value: the running header text).
        let header = lookup_marker_whitespace("h")
            .expect("marker known to spec should resolve to a default WS row");
        assert_eq!(
            header.required_before_open,
            StructuralWhitespaceRequirement::NewlineOrAnyWhitespaceBeforeMarker
        );
        assert_eq!(
            header.required_after_open_name,
            StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
        );
        assert_eq!(header.category_for_profiles, WhitespaceFormatCategory::Block);

        // `\mt1` is Paragraph kind / Title category — takes a title
        // string after the marker name (HS required).
        let title = lookup_marker_whitespace("mt1").expect("mt1 should resolve via numbered variant");
        assert_eq!(
            title.required_after_open_name,
            StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
        );

        // `\ip` is Paragraph kind / Introduction category — paragraph
        // content follows, so tag-end-delimiter after name.
        let intro = lookup_marker_whitespace("ip").expect("ip should resolve to intro default");
        assert_eq!(
            intro.required_after_open_name,
            StructuralWhitespaceRequirement::TagEndDelimiter
        );

        // `\id` is Header kind — book code value follows (HS).
        let id = lookup_marker_whitespace("id").expect("id should resolve to header default");
        assert_eq!(
            id.required_after_open_name,
            StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
        );

        // `\nd` (character marker) takes the inline default — tag-end
        // after name, optional WS before, no format preference.
        let nd = lookup_marker_whitespace("nd")
            .expect("character marker should resolve to inline default");
        assert_eq!(
            nd.required_after_open_name,
            StructuralWhitespaceRequirement::TagEndDelimiter
        );
        assert_eq!(nd.category_for_profiles, WhitespaceFormatCategory::Inline);
    }

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
        assert_eq!(spec.kind, MarkerDefKind::TableCell);
        assert!(marker_allows_context("tc2", SpecContext::Table));
    }

    #[test]
    fn list_value_chars_resolve_from_numbered_variants() {
        let spec = lookup_spec_marker("liv1").expect("liv1 should resolve");
        assert_eq!(spec.kind, MarkerDefKind::Character);
        assert!(marker_allows_context("liv1", SpecContext::List));
    }

    #[test]
    fn imte_uses_its_own_marker_metadata() {
        let spec = lookup_spec_marker("imte").expect("imte should resolve");
        assert_eq!(spec.kind, MarkerDefKind::Paragraph);
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
}
