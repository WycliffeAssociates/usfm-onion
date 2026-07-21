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
use rustc_hash::FxHashMap;
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

/// Argument payload a marker's opening form consumes immediately after its
/// name, before any content: the book code for `\id`, a chapter/verse
/// number-range for the chapter/verse family (`\c`, `\cp`, `\ca`, `\v`,
/// `\vp`, `\va`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MarkerPayload {
    BookCode,
    NumberRange,
}

/// Single source of truth for marker argument payloads. The lexer's
/// contextual-payload consumption (`pending_payload_for` in `lexer.rs`) and
/// the marker catalog (`UsfmMarkerInfo.payload`) both read THIS function, so
/// the two surfaces cannot drift.
pub fn marker_payload(marker: &str) -> Option<MarkerPayload> {
    match marker {
        "id" => Some(MarkerPayload::BookCode),
        "c" | "cp" | "ca" | "v" | "vp" | "va" => Some(MarkerPayload::NumberRange),
        _ => None,
    }
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

fn whitespace_index() -> &'static FxHashMap<&'static str, &'static MarkerWhitespace> {
    static INDEX: OnceLock<FxHashMap<&'static str, &'static MarkerWhitespace>> = OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_WHITESPACE
            .iter()
            .map(|row| (row.marker, row))
            .collect::<FxHashMap<_, _>>()
    })
}

/// Canonical marker name -> `(MarkerIndex, &'static MarkerSpec)`. The index
/// is each row's position in `MARKER_SPECS` — the same iteration
/// `marker_index_by_canonical()`/`marker_rows()` use — captured here so
/// `lookup_spec_marker_indexed` gets it from the same hash probe that finds
/// the spec, instead of a second hash on the canonical name afterward.
fn exact_spec_index() -> &'static FxHashMap<&'static str, (MarkerIndex, &'static MarkerSpec)> {
    static INDEX: OnceLock<FxHashMap<&'static str, (MarkerIndex, &'static MarkerSpec)>> =
        OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .enumerate()
            .filter_map(|(i, spec)| MarkerIndex::new(i).map(|idx| (spec.marker, (idx, spec))))
            .collect()
    })
}

fn table_cell_spec_index() -> &'static FxHashMap<&'static str, (MarkerIndex, &'static MarkerSpec)> {
    static INDEX: OnceLock<FxHashMap<&'static str, (MarkerIndex, &'static MarkerSpec)>> =
        OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.kind == MarkerDefKind::TableCell)
            .filter_map(|(i, spec)| MarkerIndex::new(i).map(|idx| (spec.marker, (idx, spec))))
            .collect()
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
    lookup_spec_marker_indexed(marker).map(|(_, spec)| spec)
}

/// Same resolution as [`lookup_spec_marker`], but also returns the
/// [`MarkerIndex`] the same hash probe already found — the position this
/// canonical row occupies in `MARKER_SPECS`/`marker_rows()`. Lets
/// [`resolve_marker_metadata`]'s slow path get canonical/kind and the index
/// from a single lookup, instead of hashing the canonical name a second time
/// via `marker_index_by_canonical()`.
fn lookup_spec_marker_indexed(marker: &str) -> Option<(MarkerIndex, &'static MarkerSpec)> {
    let normalized = marker.strip_prefix('+').unwrap_or(marker);

    if let Some(found) = exact_spec_index().get(normalized).copied() {
        return Some(found);
    }

    if let Some(base) = normalized
        .strip_suffix("-s")
        .or_else(|| normalized.strip_suffix("-e"))
    {
        let milestone_base = base.trim_end_matches(|ch: char| ch.is_ascii_digit());
        if let Some(found) = exact_spec_index().get(milestone_base).copied()
            && found.1.kind == MarkerDefKind::Milestone
        {
            return Some(found);
        }
    }

    if let Some(table_cell_base) = table_cell_base(normalized)
        && let Some(found) = table_cell_spec_index().get(table_cell_base).copied()
    {
        return Some(found);
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
        Some((canonical, kind, _ordinal)) => (canonical, kind),
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

/// One consolidated resolution of the marker facts the parser's hot path
/// needs, built once per canonical marker from the existing spec-derivation
/// functions (`structural_marker_info`, `marker_family_for`, `marker_payload`,
/// `lookup_marker_whitespace`) rather than from `MARKER_SPECS` /
/// `MARKER_WHITESPACE` directly — those helpers already carry the extra
/// derivation logic (defaults, aliases, family) a raw two-table read would
/// miss.
///
/// Canonical-row facts only: every field here is the same for every raw
/// spelling that resolves to a given canonical marker. Occurrence facts that
/// can differ between two raw spellings of the same canonical marker —
/// nested (`+`) prefix, milestone start/end side, alias/numbered family role,
/// and the raw spelling itself — are deliberately NOT stored here; callers
/// read those straight off the raw lexeme. See WS2A in
/// `plans/plan-parse-hot-path.md`.
///
/// Deliberately keyed by *canonical* name only, never re-derived from a raw
/// occurrence spelling: the lexer already resolves canonical/kind/family via
/// [`lookup_marker_metadata`] (unchanged — its `fast_marker_metadata` alias
/// table is the ground truth for cases like `fe`/`ef` collapsing to `f`,
/// which this row table must not second-guess). Re-normalizing raw text here
/// too risked disagreeing with that alias table; keying strictly off the
/// already-resolved canonical name avoids the whole class of bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedMarker {
    pub(crate) structural: StructuralMarkerInfo,
    /// Kept for parity-testing against `marker_payload` and for future
    /// callers; the lexer's own payload consumption intentionally does NOT
    /// route through this row (see the note on `resolved_marker_metadata`
    /// in `lexer.rs`).
    #[allow(dead_code)]
    pub(crate) payload: Option<MarkerPayload>,
    /// Whether this marker's opening form absorbs the first whitespace byte
    /// that follows its name as a tag-end delimiter — the same predicate
    /// pre-WS2A `delimiter_absorption` (`parse/mod.rs`) computed per
    /// occurrence via `lookup_marker_whitespace`, now precomputed once per
    /// canonical row.
    pub(crate) absorbs_delimiter_whitespace: bool,
}

/// Dense-table handle into [`marker_rows`]: the position of a canonical
/// marker's row, resolved once in the lexer and stamped onto
/// [`crate::token::MarkerMetadata`] so the parser's per-occurrence structural
/// and delimiter-absorption lookups (`structural_info_for_index`,
/// `absorbs_delimiter_whitespace_for_index`) become array indexing instead of
/// a string-hashmap probe. WS2B; see `plans/plan-parse-hot-path.md`.
///
/// Distinct from the public, string-backed [`MarkerId`]: this is a private
/// performance handle with no catalog/semantic meaning of its own. Its
/// numeric value is not a supported contract (not serialized, not compared
/// across builds) — only [`MarkerId`] and `canonical`/`kind`/`family` are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MarkerIndex(u16);

impl MarkerIndex {
    /// Sentinel for "no resolved canonical row" (unknown/unresolved marker).
    /// `marker_rows()` never has anywhere near `u16::MAX` entries, so this
    /// value can never collide with a real row position.
    pub(crate) const UNKNOWN: MarkerIndex = MarkerIndex(u16::MAX);

    /// Checked constructor: `None` if `i` would collide with the `UNKNOWN`
    /// sentinel. `MARKER_SPECS` has on the order of 200 rows, far below the
    /// u16 index space, so this never actually rejects a real row today.
    fn new(i: usize) -> Option<Self> {
        (i < u16::MAX as usize).then_some(MarkerIndex(i as u16))
    }

    fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for MarkerIndex {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

/// The dense, canonical-row table: one [`ResolvedMarker`] per `MARKER_SPECS`
/// entry, in `MARKER_SPECS` order, so a [`MarkerIndex`] is just that entry's
/// position. Built once from the same per-canonical derivation WS2A used.
fn marker_rows() -> &'static [ResolvedMarker] {
    static ROWS: OnceLock<Vec<ResolvedMarker>> = OnceLock::new();
    ROWS.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .map(|spec| {
                let structural = structural_marker_info(spec.marker, Some(spec.kind));
                let payload = marker_payload(spec.marker);
                let absorbs_delimiter_whitespace = absorbs_delimiter_whitespace(spec.marker);
                ResolvedMarker {
                    structural,
                    payload,
                    absorbs_delimiter_whitespace,
                }
            })
            .collect()
    })
}

/// Canonical marker name to its [`MarkerIndex`] (its position in
/// [`marker_rows`]). Built once, in lockstep with `marker_rows()` (same
/// `MARKER_SPECS` iteration), so the two can never drift apart. Used to seed
/// [`fast_marker_index_table`] (once, cold) and by the WS2A parity surface
/// (`resolved_marker_for_canonical`) — not on `resolve_marker_metadata`'s
/// hot path, which resolves its index from the same hash that already found
/// canonical/kind (see `fast_marker_index_table` and
/// `lookup_spec_marker_indexed`).
fn marker_index_by_canonical() -> &'static FxHashMap<&'static str, MarkerIndex> {
    static INDEX: OnceLock<FxHashMap<&'static str, MarkerIndex>> = OnceLock::new();
    INDEX.get_or_init(|| {
        MARKER_SPECS
            .iter()
            .enumerate()
            .filter_map(|(i, spec)| MarkerIndex::new(i).map(|idx| (spec.marker, idx)))
            .collect()
    })
}

/// Array-index accessor for an already-resolved [`MarkerIndex`] — no
/// hashing. `None` for `MarkerIndex::UNKNOWN` or any other out-of-range value.
pub(crate) fn resolved_marker_row(index: MarkerIndex) -> Option<&'static ResolvedMarker> {
    marker_rows().get(index.as_usize())
}

/// Canonical marker name per ordinal returned by [`fast_marker_metadata`] —
/// index `i` here is the ordinal `i` a match arm returns. Only consulted
/// once, to build [`fast_marker_index_table`]; never hashed at lex time.
/// Kept in lockstep with `fast_marker_metadata`'s arms by
/// `resolve_marker_metadata_index_matches_marker_index_by_canonical` (see
/// tests below).
const FAST_MARKER_CANONICALS: [&str; 48] = [
    "id", "h", "c", "v", "p", "m", "b", "r", "mt", "mt1", "mt2", "mt3", "mt4", "s", "s1", "s2",
    "s3", "s4", "q", "q1", "q2", "q3", "q4", "f", "x", "ft", "fr", "fq", "fqa", "fk", "fl", "fw",
    "fp", "fv", "fdc", "fm", "xo", "xop", "xk", "xq", "xt", "xta", "xot", "xnt", "xdc", "w", "jmp",
    "ref",
];

/// `fast_marker_metadata`'s ordinal -> [`MarkerIndex`], resolved once (one
/// `marker_index_by_canonical()` hash per canonical, ever) so
/// `resolve_marker_metadata`'s fast path turns the ordinal a match arm
/// already returns into a `MarkerIndex` by array indexing — never a hash.
fn fast_marker_index_table() -> &'static [MarkerIndex; 48] {
    static TABLE: OnceLock<[MarkerIndex; 48]> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|i| {
            marker_index_by_canonical()
                .get(FAST_MARKER_CANONICALS[i])
                .copied()
                .unwrap_or(MarkerIndex::UNKNOWN)
        })
    })
}

/// The lexer's single canonical-resolution pass: one hash to resolve a raw
/// marker spelling, from which `canonical`/`kind`/`family`/[`MarkerIndex`]
/// all fall out without a second hash. Returns `(canonical, kind, family,
/// index)`; `index` is `MarkerIndex::UNKNOWN` whenever the marker doesn't
/// resolve.
///
/// - Fast path (`fast_marker_metadata`, the ~48 hottest markers, including
///   the `fe`/`ef`→`f` and `x`/`ex`→`x` alias collapses): zero hashing.
///   `fast_marker_metadata` already resolves canonical/kind via a plain
///   `match`; its ordinal turns into a `MarkerIndex` via array indexing into
///   [`fast_marker_index_table`].
/// - Slow path: [`lookup_spec_marker_indexed`] resolves canonical/kind *and*
///   `MarkerIndex` from the single hash probe that finds the spec, instead
///   of a second hash on the canonical name afterward.
pub(crate) fn resolve_marker_metadata(
    marker: &str,
) -> (
    Option<&'static str>,
    Option<MarkerDefKind>,
    Option<MarkerFamily>,
    MarkerIndex,
) {
    if let Some((canonical, kind, ordinal)) = fast_marker_metadata(marker) {
        let index = fast_marker_index_table()[ordinal as usize];
        return (
            Some(canonical),
            Some(kind),
            marker_family_for(canonical, kind),
            index,
        );
    }

    let normalized = marker.strip_prefix('+').unwrap_or(marker);
    match lookup_spec_marker_indexed(normalized) {
        Some((index, spec)) => (
            Some(spec.marker),
            Some(spec.kind),
            marker_family_for(spec.marker, spec.kind),
            index,
        ),
        None => (None, None, None, MarkerIndex::UNKNOWN),
    }
}

/// [`structural_marker_info`] equivalent for the parser, driven off an
/// already-resolved [`MarkerIndex`] — array indexing, no hashing. Falls back
/// to `Unknown` for an unresolved marker, exactly like `structural_marker_info`'s
/// `None`-kind arm.
pub(crate) fn structural_info_for_index(index: MarkerIndex) -> StructuralMarkerInfo {
    resolved_marker_row(index)
        .map(|row| row.structural)
        .unwrap_or(StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Unknown,
            inline_context: None,
            note_context: None,
        })
}

/// Delimiter-absorption predicate for the parser, equivalent to pre-WS2A
/// `delimiter_absorption`'s `lookup_marker_whitespace(name).is_some_and(...)`
/// check, but driven off an already-resolved [`MarkerIndex`] — array
/// indexing, no hashing.
pub(crate) fn absorbs_delimiter_whitespace_for_index(index: MarkerIndex) -> bool {
    resolved_marker_row(index).is_some_and(|row| row.absorbs_delimiter_whitespace)
}

/// Resolve an already-canonical marker name (e.g. a lexer-resolved
/// `MarkerMetadata.canonical`) directly, with no renormalization. Kept for the
/// WS2A drift/parity tests, which assert behavior over the raw-spelling-keyed
/// surface; the parser's hot path uses [`structural_info_for_index`] /
/// [`absorbs_delimiter_whitespace_for_index`] instead (WS2B), which take the
/// `MarkerIndex` already stamped on the token and never hash.
pub(crate) fn resolved_marker_for_canonical(canonical: Option<&str>) -> Option<ResolvedMarker> {
    let index = marker_index_by_canonical().get(canonical?).copied()?;
    resolved_marker_row(index).copied()
}

/// [`structural_marker_info`] equivalent for the parser, driven off an
/// already-resolved canonical name instead of re-normalizing the raw
/// occurrence spelling and re-deriving structural info. Falls back to
/// `Unknown` for an unresolved marker, exactly like `structural_marker_info`'s
/// `None`-kind arm.
///
/// Kept for the WS2A parity tests (see `structural_info_for_index` above for
/// the WS2B hot path this predates).
pub(crate) fn structural_info_for_canonical(canonical: Option<&str>) -> StructuralMarkerInfo {
    resolved_marker_for_canonical(canonical)
        .map(|row| row.structural)
        .unwrap_or(StructuralMarkerInfo {
            scope_kind: StructuralScopeKind::Unknown,
            inline_context: None,
            note_context: None,
        })
}

/// Delimiter-absorption predicate for the parser, equivalent to pre-WS2A
/// `delimiter_absorption`'s `lookup_marker_whitespace(name).is_some_and(...)`
/// check, but driven off an already-resolved canonical name instead of
/// renormalizing the raw occurrence spelling.
///
/// Kept for the WS2A parity tests (see `absorbs_delimiter_whitespace_for_index`
/// above for the WS2B hot path this predates).
pub(crate) fn absorbs_delimiter_whitespace_for_canonical(canonical: Option<&str>) -> bool {
    resolved_marker_for_canonical(canonical).is_some_and(|row| row.absorbs_delimiter_whitespace)
}

/// The same "does this marker's opening form absorb its trailing tag-end
/// delimiter whitespace" predicate pre-WS2A `delimiter_absorption`
/// (`parse/mod.rs`) computed inline via `lookup_marker_whitespace`, factored
/// out so [`marker_row_index`] can precompute it once per canonical row.
fn absorbs_delimiter_whitespace(marker: &str) -> bool {
    lookup_marker_whitespace(marker).is_some_and(|whitespace| {
        matches!(
            whitespace.required_after_open_name,
            StructuralWhitespaceRequirement::TagEndDelimiter
                | StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
                | StructuralWhitespaceRequirement::AtLeastOneWhitespace
        )
    })
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
///
/// The third tuple element is an ordinal into [`FAST_MARKER_CANONICALS`] /
/// [`fast_marker_index_table`] (not a [`MarkerIndex`] itself) — a plain
/// `match` result [`resolve_marker_metadata`] turns into a `MarkerIndex` via
/// array indexing, with no hashing.
fn fast_marker_metadata(marker: &str) -> Option<(&'static str, MarkerDefKind, u8)> {
    match marker {
        "id" => Some(("id", MarkerDefKind::Header, 0)),
        "h" => Some(("h", MarkerDefKind::Paragraph, 1)),
        "c" => Some(("c", MarkerDefKind::Chapter, 2)),
        "v" => Some(("v", MarkerDefKind::Verse, 3)),
        "p" => Some(("p", MarkerDefKind::Paragraph, 4)),
        "m" => Some(("m", MarkerDefKind::Paragraph, 5)),
        "b" => Some(("b", MarkerDefKind::Paragraph, 6)),
        "r" => Some(("r", MarkerDefKind::Paragraph, 7)),
        "mt" => Some(("mt", MarkerDefKind::Paragraph, 8)),
        "mt1" => Some(("mt1", MarkerDefKind::Paragraph, 9)),
        "mt2" => Some(("mt2", MarkerDefKind::Paragraph, 10)),
        "mt3" => Some(("mt3", MarkerDefKind::Paragraph, 11)),
        "mt4" => Some(("mt4", MarkerDefKind::Paragraph, 12)),
        "s" => Some(("s", MarkerDefKind::Paragraph, 13)),
        "s1" => Some(("s1", MarkerDefKind::Paragraph, 14)),
        "s2" => Some(("s2", MarkerDefKind::Paragraph, 15)),
        "s3" => Some(("s3", MarkerDefKind::Paragraph, 16)),
        "s4" => Some(("s4", MarkerDefKind::Paragraph, 17)),
        "q" => Some(("q", MarkerDefKind::Paragraph, 18)),
        "q1" => Some(("q1", MarkerDefKind::Paragraph, 19)),
        "q2" => Some(("q2", MarkerDefKind::Paragraph, 20)),
        "q3" => Some(("q3", MarkerDefKind::Paragraph, 21)),
        "q4" => Some(("q4", MarkerDefKind::Paragraph, 22)),
        "f" => Some(("f", MarkerDefKind::Note, 23)),
        "x" => Some(("x", MarkerDefKind::Note, 24)),
        "ft" => Some(("ft", MarkerDefKind::Character, 25)),
        "fr" => Some(("fr", MarkerDefKind::Character, 26)),
        "fq" => Some(("fq", MarkerDefKind::Character, 27)),
        "fqa" => Some(("fqa", MarkerDefKind::Character, 28)),
        "fk" => Some(("fk", MarkerDefKind::Character, 29)),
        "fl" => Some(("fl", MarkerDefKind::Character, 30)),
        "fw" => Some(("fw", MarkerDefKind::Character, 31)),
        "fp" => Some(("fp", MarkerDefKind::Character, 32)),
        "fv" => Some(("fv", MarkerDefKind::Character, 33)),
        "fdc" => Some(("fdc", MarkerDefKind::Character, 34)),
        "fm" => Some(("fm", MarkerDefKind::Character, 35)),
        "xo" => Some(("xo", MarkerDefKind::Character, 36)),
        "xop" => Some(("xop", MarkerDefKind::Character, 37)),
        "xk" => Some(("xk", MarkerDefKind::Character, 38)),
        "xq" => Some(("xq", MarkerDefKind::Character, 39)),
        "xt" => Some(("xt", MarkerDefKind::Character, 40)),
        "xta" => Some(("xta", MarkerDefKind::Character, 41)),
        "xot" => Some(("xot", MarkerDefKind::Character, 42)),
        "xnt" => Some(("xnt", MarkerDefKind::Character, 43)),
        "xdc" => Some(("xdc", MarkerDefKind::Character, 44)),
        "w" => Some(("w", MarkerDefKind::Character, 45)),
        "jmp" => Some(("jmp", MarkerDefKind::Character, 46)),
        "ref" => Some(("ref", MarkerDefKind::Character, 47)),
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
        MARKER_SPECS, MarkerDefKind, MarkerIndex, SpecContext,
        absorbs_delimiter_whitespace_for_canonical, lookup_marker_metadata,
        lookup_marker_whitespace, lookup_spec_marker, marker_allows_context,
        marker_allows_effective_context, marker_index_by_canonical, marker_payload, marker_rows,
        resolve_marker_metadata, resolved_marker_for_canonical, resolved_marker_row,
        structural_info_for_canonical, structural_marker_info,
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
        assert_eq!(
            header.category_for_profiles,
            WhitespaceFormatCategory::Block
        );

        // `\mt1` is Paragraph kind / Title category — takes a title
        // string after the marker name (HS required).
        let title =
            lookup_marker_whitespace("mt1").expect("mt1 should resolve via numbered variant");
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

    // --- WS2A drift/parity tests -------------------------------------------
    //
    // WS2A leaves the lexer's canonical/kind/family resolution untouched
    // (still `lookup_marker_metadata`, unchanged). The new consolidation is
    // on the parser side: `structural_info_for_canonical` and
    // `absorbs_delimiter_whitespace_for_canonical` replace
    // `structural_marker_info(raw, kind)` and the
    // `lookup_marker_whitespace(raw)`-driven absorption check, but read off
    // the canonical name the lexer already resolved instead of
    // renormalizing `raw`. These tests assert that substitution never
    // disagrees with the pre-WS2A raw-spelling-driven originals — over every
    // canonical spec row, explicit normalization-family cases, every
    // distinct marker spelling seen in the corpora, and unknown/malformed
    // input.

    /// Re-derive what the pre-WS2A parser call sites would have produced for
    /// `raw` (via `structural_marker_info`/`lookup_marker_whitespace`, called
    /// with `raw` directly) and assert the canonical-keyed WS2A replacements
    /// agree, using exactly the canonical name `lookup_marker_metadata`
    /// resolves for `raw` — i.e. exactly what `marker.metadata.canonical`
    /// would hold on the real token. `raw` is passed exactly as the lexer
    /// would see it (may carry a `+` nesting prefix, milestone `-s`/`-e`
    /// suffix, numbered/hyphenated table-cell suffix, or be
    /// unknown/malformed).
    fn assert_resolution_parity(raw: &str) {
        let old_metadata = lookup_marker_metadata(raw);
        let old_kind = old_metadata.map(|(_, kind, _)| kind);
        let canonical = old_metadata.map(|(canonical, _, _)| canonical);

        let old_structural = structural_marker_info(raw, old_kind);
        assert_eq!(
            structural_info_for_canonical(canonical),
            old_structural,
            "structural mismatch for {raw:?} (canonical {canonical:?})"
        );

        let old_absorbs = lookup_marker_whitespace(raw).is_some_and(|whitespace| {
            matches!(
                whitespace.required_after_open_name,
                StructuralWhitespaceRequirement::TagEndDelimiter
                    | StructuralWhitespaceRequirement::AtLeastOneHorizontalWhitespace
                    | StructuralWhitespaceRequirement::AtLeastOneWhitespace
            )
        });
        assert_eq!(
            absorbs_delimiter_whitespace_for_canonical(canonical),
            old_absorbs,
            "absorption mismatch for {raw:?} (canonical {canonical:?})"
        );
    }

    #[test]
    fn resolved_marker_matches_every_canonical_spec_row() {
        // Every literal `MARKER_SPECS` row, resolved by its own canonical
        // spelling, must agree with the pre-WS2A structural/payload/
        // whitespace predicates called with that same canonical spelling.
        for spec in MARKER_SPECS.iter() {
            assert_resolution_parity(spec.marker);

            let row = resolved_marker_for_canonical(Some(spec.marker))
                .unwrap_or_else(|| panic!("canonical marker {:?} should have a row", spec.marker));
            assert_eq!(
                row.payload,
                marker_payload(spec.marker),
                "payload mismatch for {:?}",
                spec.marker
            );
        }
    }

    #[test]
    fn structural_and_absorption_agree_with_old_lookup_for_normalization_families() {
        // Nested `+`, numbered paragraph variants, numbered/hyphenated table
        // cells, milestone `-s`/`-e`, and the `esbe` alias — occurrence-level
        // normalizations that `lookup_marker_metadata` (unchanged by WS2A)
        // still performs before the parser's canonical-keyed lookups run.
        let cases = [
            "+f", "+fq", "+xt", "+nd", // nested character/note markers
            "q1", "q2", "q3", "q4", "s1", "s2", "s3", "s4", "li1", "li2", "li3", "li4", "lim1",
            "lim4", "liv1", "liv5", // numbered paragraph/character variants
            "tc2", "tc3", "tcr2", "tcc3", "th1", "thr2-3", // numbered/hyphenated table cells
            "qt-s", "qt-e", "qt1-s", "qt2-e", "ts-s", "ts-e", "zaln-s",
            "zaln-e", // milestone start/end
            "esbe", "esb", // sidebar alias + its canonical
            "fe", "ef", "ex", // metadata-fast-path aliases (collapse to "f"/"x")
        ];
        for raw in cases {
            assert_resolution_parity(raw);
        }

        // `fe` (endnote), `ef` (extended footnote), and `ex` (extended
        // cross-reference) are distinct markers with their own `MARKER_SPECS`
        // rows, so they resolve to their OWN canonical — not collapsed to
        // `"f"`/`"x"`. (They stay in the footnote/cross-reference note *family*
        // via `marker_note_family`, which is a coarser axis than canonical.)
        assert_eq!(lookup_marker_metadata("fe").map(|(c, ..)| c), Some("fe"));
        assert_eq!(lookup_marker_metadata("ef").map(|(c, ..)| c), Some("ef"));
        assert_eq!(lookup_marker_metadata("ex").map(|(c, ..)| c), Some("ex"));
    }

    #[test]
    fn parser_lookups_keep_unknown_and_malformed_markers_unknown() {
        // Unknown/malformed marker names must not be silently promoted to
        // "known" by the canonical-keyed parser lookups.
        let cases = [
            "zzzzz",
            "s5",
            "+bogus",
            "q99",
            "notareal-s",
            "123abc",
            "",
            "+",
            "-s",
            "tcz",
        ];
        for raw in cases {
            assert_resolution_parity(raw);
            assert!(
                lookup_marker_metadata(raw).is_none(),
                "expected {raw:?} to stay unresolved"
            );
        }

        // A malformed nested occurrence of a marker that only ever carries a
        // contextual payload as a bare top-level marker (`\id`, `\c`, `\v`, …)
        // still classifies as its canonical marker for metadata purposes
        // (`lookup_marker_metadata` has always stripped a leading `+`,
        // unchanged by WS2A) — but payload *consumption* in the lexer stays
        // gated on the literal, unstripped spelling (`marker_payload` never
        // matches a `+`-prefixed string), so a malformed `\+id` still must
        // not start consuming a book code. This is a deliberate WS2A
        // decision, not an oversight: see the note on `ResolvedMarker`.
        assert_eq!(
            lookup_marker_metadata("+id").map(|(canonical, ..)| canonical),
            Some("id")
        );
        assert_eq!(marker_payload("+id"), None);
    }

    #[test]
    fn parser_lookups_match_old_lookup_for_every_corpus_marker_spelling() {
        use std::collections::BTreeSet;
        use std::path::Path;

        let mut names: BTreeSet<String> = BTreeSet::new();
        for root in ["testData", "example-corpora"] {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(root);
            collect_marker_spellings(&root, &mut names);
        }
        assert!(
            names.len() > 50,
            "expected the corpora to exercise a healthy number of distinct \
             marker spellings, got {}",
            names.len()
        );
        for name in &names {
            assert_resolution_parity(name);
        }
    }

    // --- WS2C drift guard: single-hash MarkerIndex resolution --------------
    //
    // `resolve_marker_metadata` now resolves `MarkerIndex` alongside
    // canonical/kind/family from the *same* hash probe, instead of hashing
    // the canonical name a second time via `marker_index_by_canonical()`.
    // The fast path (`fast_marker_metadata`'s ~48 hottest markers) gets its
    // index from a hand-written ordinal array indexed into
    // `fast_marker_index_table()` — zero hashing, but only correct if that
    // ordinal table stays in lockstep with `fast_marker_metadata`'s match
    // arms. This test is the guard: it walks every raw spelling
    // `fast_marker_metadata` special-cases (including the `fe`/`ef`→`f` and
    // `x`/`ex`→`x` alias collapses) and every literal `MARKER_SPECS` row,
    // and asserts `resolve_marker_metadata`'s index always agrees with the
    // canonical-keyed `marker_index_by_canonical()` hash — so a
    // `MARKER_SPECS` reorder, or an ordinal typo, fails loudly here instead
    // of silently mis-resolving a token's structural facts.

    #[test]
    fn resolve_marker_metadata_index_matches_marker_index_by_canonical() {
        // Fast path: every raw spelling `fast_marker_metadata` special-cases
        // must resolve to the same `MarkerIndex` its canonical would via the
        // canonical-keyed hash — including the alias collapses, where the
        // raw spelling (`fe`, `ef`, `ex`) differs from the canonical (`f`,
        // `x`) whose index actually applies.
        let fast_path_raw_markers = [
            "id", "h", "c", "v", "p", "m", "b", "r", "mt", "mt1", "mt2", "mt3", "mt4", "s", "s1",
            "s2", "s3", "s4", "q", "q1", "q2", "q3", "q4", "f", "fe", "ef", "x", "ex", "ft", "fr",
            "fq", "fqa", "fk", "fl", "fw", "fp", "fv", "fdc", "fm", "xo", "xop", "xk", "xq", "xt",
            "xta", "xot", "xnt", "xdc", "w", "jmp", "ref",
        ];
        for raw in fast_path_raw_markers {
            let (canonical, _, _, index) = resolve_marker_metadata(raw);
            let canonical =
                canonical.unwrap_or_else(|| panic!("{raw:?} should resolve on the fast path"));
            let expected = marker_index_by_canonical()
                .get(canonical)
                .copied()
                .unwrap_or_else(|| {
                    panic!("canonical {canonical:?} missing from marker_index_by_canonical")
                });
            assert_eq!(
                index, expected,
                "fast-path MarkerIndex drift for raw {raw:?} (canonical {canonical:?})"
            );
        }

        // Slow (and fast-path-overlapping) coverage: every literal
        // `MARKER_SPECS` row, resolved by its own spelling, must agree with
        // `lookup_marker_metadata`'s canonical/kind/family (the existing
        // single source of truth this refactor must not drift from) and
        // with the canonical-keyed index.
        for spec in MARKER_SPECS.iter() {
            let (canonical, kind, family, index) = resolve_marker_metadata(spec.marker);
            let expected_metadata = lookup_marker_metadata(spec.marker);
            assert_eq!(
                canonical,
                expected_metadata.map(|(c, ..)| c),
                "canonical drift for {:?}",
                spec.marker
            );
            assert_eq!(
                kind,
                expected_metadata.map(|(_, k, _)| k),
                "kind drift for {:?}",
                spec.marker
            );
            assert_eq!(
                family,
                expected_metadata.and_then(|(_, _, f)| f),
                "family drift for {:?}",
                spec.marker
            );
            let expected_index = canonical
                .and_then(|c| marker_index_by_canonical().get(c).copied())
                .unwrap_or(MarkerIndex::UNKNOWN);
            assert_eq!(index, expected_index, "index drift for {:?}", spec.marker);
        }

        // `marker_rows()` (ROWS) stays one-to-one with `MARKER_SPECS`: same
        // length, same order, so a `MarkerIndex` from either resolution path
        // always lands on the row describing that canonical marker.
        assert_eq!(marker_rows().len(), MARKER_SPECS.len());
        for spec in MARKER_SPECS.iter() {
            let index = marker_index_by_canonical()
                .get(spec.marker)
                .copied()
                .unwrap_or_else(|| {
                    panic!("{:?} missing from marker_index_by_canonical", spec.marker)
                });
            let row = resolved_marker_row(index)
                .unwrap_or_else(|| panic!("{:?} should have a row", spec.marker));
            assert_eq!(
                row.structural,
                structural_marker_info(spec.marker, Some(spec.kind)),
                "row for {:?} disagrees with structural_marker_info",
                spec.marker
            );
        }
    }

    fn collect_marker_spellings(
        dir: &std::path::Path,
        out: &mut std::collections::BTreeSet<String>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_marker_spellings(&path, out);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("usfm") {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            for token in crate::lexer::lex(&source).tokens {
                match token {
                    crate::token::ScanToken::Marker(m)
                    | crate::token::ScanToken::NestedMarker(m)
                    | crate::token::ScanToken::ClosingMarker(m)
                    | crate::token::ScanToken::NestedClosingMarker(m)
                    | crate::token::ScanToken::Milestone(m) => {
                        out.insert(m.name.to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}
