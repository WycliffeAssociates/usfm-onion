//! The JS boundary's value types, and the conversions between them and core's
//! own.
//!
//! These declarations are the schema of record for the generated TypeScript:
//! tsify emits from them directly, and their JSON shape is pinned byte-for-byte by
//! the golden outputs. Nothing here is a second definition of a *wire* type —
//! tokens, findings, and their payloads come from `usfm_onion_wire::dto`, which
//! this crate and a native host both read.
//!
//! Conversions live beside the shapes they convert, in both directions: a boundary
//! type is only half-defined without how a native value becomes one and how one
//! becomes native again.

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value as swb_to_js_value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use usfm_onion::cst::{CstDocument as NativeCstDocument, CstNode as NativeCstNode};
use usfm_onion::diff::{
    Anchor as NativeAnchor, CoveredBy as NativeCoveredBy, DecisionUnit as NativeDecisionUnit,
    DiffSkeleton as NativeDiffSkeleton, DiffableToken, Slot as NativeSlot,
    TextDiffMode as NativeTextDiffMode, unit_text_diff as native_unit_text_diff,
};
use usfm_onion::format::{FormatOptions as NativeFormatOptions, FormatToken as NativeFormatToken};
use usfm_onion::html::{
    HtmlCallerScope as NativeHtmlCallerScope, HtmlCallerStyle as NativeHtmlCallerStyle,
    HtmlNoteMode as NativeHtmlNoteMode, HtmlOptions as NativeHtmlOptions,
};
use usfm_onion::lint::{
    LintCode as NativeLintCode, LintOptions as NativeLintOptions, LintResult as NativeLintResult,
    LintScope as NativeLintScope, LintSuppression as NativeLintSuppression, LintableToken,
    TokenFix as NativeTokenFix,
};
use usfm_onion::marker_defs::StructuralMarkerInfo as NativeStructuralMarkerInfo;
use usfm_onion::token::{
    NumberRangeKind as NativeNumberRangeKind, OwnedAttribute as NativeOwnedAttribute,
    Span as NativeSpan, Token as NativeToken, TokenKind as NativeTokenKind, UsfmToken,
};
use usfm_onion::vref::{
    Segment as NativeSegment, Utf16Span as NativeUtf16Span,
    VerseProjection as NativeVerseProjection, VrefIndex as NativeVrefIndex,
    VrefMap as NativeVrefMap, VrefOptions as NativeVrefOptions,
};
use usfm_onion::walker::WalkableToken;
pub use usfm_onion_wire::dto::{
    AttributeItem, BlockBehavior, ClosingBehavior, CoveredSide, DecisionStatus, DecisionUnitKind,
    DiffOptions, HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, InlineContext, LintCategory,
    LintCode, LintIssueType, LintSeverity, MarkerCategory, MarkerDefKind, MarkerFamily,
    MarkerFamilyRole, MarkerInfo, MarkerKind, MarkerMetadata, MarkerPayload, MergeSide, NoteFamily,
    NoteSubkind, NumberInfo, NumberRangeKind, PackedBookReceipt, PackedDecodeError,
    PackedMarkerDescriptor, ParagraphCategory, SlotRole, Span, SpecContext, StructuralMarkerInfo,
    StructuralScopeKind, TextDiffMode, TextDiffRun, TextDiffRunKind, Token, TokenKind,
    UnitTextDiff, format_sid, map_marker_info,
};

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
    pub std::collections::BTreeMap<String, std::collections::BTreeMap<u32, DiffSkeleton>>,
);

/// Verse-reference map: `{ "GEN 1:1": "...", "GEN 1:2": "...", ... }`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct VrefMap(pub std::collections::BTreeMap<String, String>);

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct VrefOptions {
    #[serde(default)]
    pub(crate) trim: Option<bool>,
}

/// UTF-16 code-unit offsets into a `VerseProjection.text`. Deliberately a
/// distinct type from `Span` (byte offsets into the source) so the unit is
/// unmistakable on the wire — JS/DOM consumers index `text` in UTF-16.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Utf16Span {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

/// One in-scope text token's contribution to a verse projection, with both
/// resolution anchors: `sourceSpan` (bytes into source, for raw buffers) and
/// `tokenId` (== the editor's DOM `data-id`). `textSpan` is UTF-16 into `text`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub(crate) token_id: String,
    pub(crate) source_span: Span,
    pub(crate) text_span: Utf16Span,
}

/// Lossless plain-text projection of one verse plus its segment map back to
/// source / token coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct VerseProjection {
    pub(crate) text: String,
    pub(crate) segments: Vec<Segment>,
}

/// Every verse's lossless projection, as `[sid, projection]` pairs in the order
/// the document itself puts them — including deliberately out-of-order content
/// (`\v 19` before `\v 2`, chapter 10 before chapter 2).
///
/// One authoritative sequence, not a sequence plus a lookup: an object keyed by
/// SID enumerates its keys sorted, which is the silent re-ordering this shape
/// exists to prevent, and carrying both would mean documenting that one of the two
/// views is meaningless. A consumer that wants O(1) lookup writes
/// `new Map(entries)` and owns that choice.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct VrefIndex(pub(crate) Vec<(String, VerseProjection)>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
    pub(crate) token_index: usize,
    pub(crate) children: Vec<CstNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CstDocument {
    pub(crate) tokens: Vec<Token>,
    pub(crate) roots: Vec<CstNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSuppression {
    pub(crate) code: LintCode,
    pub(crate) sid: String,
}

/// What the caller is linting. Gates the document-level rules: they run only
/// for `"front"` and `"book"`, never a bare `{ chapter }` slice. TS shape:
/// `"front" | { chapter: number } | "book"`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum LintScope {
    Front,
    Chapter(u32),
    Book,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintOptions {
    /// Required — no default. A defaulted scope would let a chapter-grain
    /// caller silently get whole-book id-behavior.
    pub(crate) scope: LintScope,
    #[serde(default)]
    pub(crate) enabled_codes: Option<Vec<LintCode>>,
    #[serde(default)]
    pub(crate) disabled_codes: Vec<LintCode>,
    #[serde(default)]
    pub(crate) suppressed: Vec<LintSuppression>,
    #[serde(default)]
    pub(crate) allow_implicit_chapter_content_verse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintIssue {
    pub(crate) code: LintCode,
    pub(crate) category: LintCategory,
    pub(crate) severity: LintSeverity,
    pub(crate) issue_type: LintIssueType,
    pub(crate) template: String,
    pub(crate) message: String,
    pub(crate) message_params: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) related_span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) related_token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fix: Option<TokenFix>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSummary {
    pub(crate) by_category: std::collections::BTreeMap<LintCategory, usize>,
    pub(crate) by_severity: std::collections::BTreeMap<LintSeverity, usize>,
    pub(crate) by_issue_type: std::collections::BTreeMap<LintIssueType, usize>,
    pub(crate) total_count: usize,
    pub(crate) suppressed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintResult {
    pub(crate) issues: Vec<LintIssue>,
    pub(crate) summary: LintSummary,
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
    pub(crate) kind: TokenKind,
    pub(crate) text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) sid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    #[serde(default)]
    pub(crate) recover_malformed_markers: Option<bool>,
    #[serde(default)]
    pub(crate) collapse_whitespace_in_text: Option<bool>,
    #[serde(default)]
    pub(crate) ensure_inline_separators: Option<bool>,
    #[serde(default)]
    pub(crate) remove_duplicate_verse_numbers: Option<bool>,
    #[serde(default)]
    pub(crate) normalize_spacing_after_paragraph_markers: Option<bool>,
    #[serde(default)]
    pub(crate) remove_unwanted_linebreaks: Option<bool>,
    #[serde(default)]
    pub(crate) bridge_consecutive_verse_markers: Option<bool>,
    #[serde(default)]
    pub(crate) remove_orphan_empty_verse_before_contentful_verse: Option<bool>,
    #[serde(default)]
    pub(crate) remove_bridge_verse_enumerators: Option<bool>,
    #[serde(default)]
    pub(crate) move_chapter_label_after_chapter_marker: Option<bool>,
    #[serde(default)]
    pub(crate) insert_default_paragraph_after_chapter_intro: Option<bool>,
    #[serde(default)]
    pub(crate) remove_empty_paragraphs: Option<bool>,
    #[serde(default)]
    pub(crate) insert_structural_linebreaks: Option<bool>,
    #[serde(default)]
    pub(crate) collapse_consecutive_linebreaks: Option<bool>,
    #[serde(default)]
    pub(crate) normalize_marker_whitespace_at_line_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatResult {
    pub(crate) tokens: Vec<Token>,
    pub(crate) usfm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct HtmlOptions {
    #[serde(default)]
    pub(crate) wrap_root: bool,
    #[serde(default)]
    pub(crate) prefer_native_elements: Option<bool>,
    #[serde(default)]
    pub(crate) note_mode: Option<HtmlNoteMode>,
    #[serde(default)]
    pub(crate) caller_style: Option<HtmlCallerStyle>,
    #[serde(default)]
    pub(crate) caller_scope: Option<HtmlCallerScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub(crate) unit_id: String,
    pub(crate) sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub(crate) unit_id: String,
    pub(crate) role: SlotRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) after: Option<Anchor>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DupContext {
    pub(crate) baseline_count: u32,
    pub(crate) current_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CoveredBy {
    pub(crate) unit_id: String,
    pub(crate) sid: String,
    pub(crate) side: CoveredSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DecisionUnit {
    pub(crate) id: String,
    pub(crate) kind: DecisionUnitKind,
    pub(crate) status: DecisionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) baseline_sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) current_sid: Option<String>,
    pub(crate) baseline_tokens: Vec<Token>,
    pub(crate) current_tokens: Vec<Token>,
    pub(crate) displaced: bool,
    pub(crate) relabeled: bool,
    pub(crate) dup_context: DupContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) covered_by: Option<CoveredBy>,
    pub(crate) is_whitespace_change: bool,
    pub(crate) is_usfm_structure_change: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_diff: Option<UnitTextDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DiffSkeleton {
    pub(crate) slots: Vec<Slot>,
    pub(crate) units: Vec<DecisionUnit>,
}

/// Staged decisions for [`wasm_merge_diff_blocks`]: `Record<string,
/// MergeSide>` plus the default applied to any unit not present in the map.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MergeRequest {
    // Crate-visible rather than private now that the exports live in their own
    // module: the fields are still not part of the Rust API, only reachable by the
    // one export that deserializes this shape.
    pub(crate) decisions: std::collections::BTreeMap<String, MergeSide>,
    pub(crate) default_side: MergeSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintCodeMeta {
    pub(crate) code: LintCode,
    pub(crate) category: LintCategory,
    pub(crate) severity: LintSeverity,
    pub(crate) issue_type: LintIssueType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatRuleMeta {
    pub(crate) code: String,
    pub(crate) label_key: String,
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
pub(crate) struct WalkToken {
    id: String,
    kind: NativeTokenKind,
    text: String,
    span: Option<NativeSpan>,
    sid: Option<String>,
    marker: Option<String>,
    structural: Option<NativeStructuralMarkerInfo>,
    number_info: Option<(u32, Option<u32>, NativeNumberRangeKind)>,
}

impl UsfmToken for WalkToken {
    fn kind(&self) -> NativeTokenKind {
        self.kind
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn source(&self) -> &str {
        &self.text
    }
}

impl WalkableToken for WalkToken {
    fn structural(&self) -> Option<NativeStructuralMarkerInfo> {
        self.structural
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

pub(crate) fn lint_scope_into_native(value: LintScope) -> NativeLintScope {
    match value {
        LintScope::Front => NativeLintScope::Front,
        LintScope::Chapter(n) => NativeLintScope::Chapter(n),
        LintScope::Book => NativeLintScope::Book,
    }
}

pub(crate) fn lint_options_into_native(value: LintOptions) -> NativeLintOptions {
    NativeLintOptions {
        scope: lint_scope_into_native(value.scope),
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
        // The wasm-facing `LintOptions` DTO has no declared-book field yet —
        // this is core's seam for a future direct-Rust caller, not a
        // wasm/JS surface addition. Every wasm-driven lint call keeps
        // today's stateless behavior until a JS-facing field is deliberately
        // added.
        declared_book: None,
    }
}

pub(crate) fn format_options_into_native(value: Option<FormatOptions>) -> NativeFormatOptions {
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

pub(crate) fn html_options_into_native(value: Option<HtmlOptions>) -> NativeHtmlOptions {
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

/// Omitting `options` (or omitting `textDiff` on a supplied `DiffOptions`)
/// resolves to `TextDiffMode::None` — today's behavior, computing nothing.
pub(crate) fn diff_options_into_native(value: Option<DiffOptions>) -> NativeTextDiffMode {
    value.unwrap_or_default().into()
}

pub(crate) fn parse_walk_tokens_from_values(values: Vec<Token>) -> Vec<WalkToken> {
    values.into_iter().map(token_to_walk_token).collect()
}

pub(crate) fn token_to_walk_token(value: Token) -> WalkToken {
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

/// Wire `AttributeItem` -> native `OwnedAttribute`. The caller's own span
/// rides along verbatim (`None` for an attribute the caller synthesized or
/// structurally edited) rather than being discarded and reinvented on the way
/// back out; `key`/`value`/`is_default` are the only other fields
/// `format_attribute_list` reads, and `text` is the verbatim per-attribute
/// source `OwnedAttribute` calls `source`.
pub(crate) fn wire_attribute_to_owned(item: &AttributeItem) -> NativeOwnedAttribute {
    NativeOwnedAttribute {
        source: Box::from(item.text.as_str()),
        key: Box::from(item.key.as_str()),
        value: Box::from(item.value.as_str()),
        is_default: item.is_default,
        span: item.span.clone().map(native_span),
    }
}

/// Native `OwnedAttribute` -> wire `AttributeItem`. Emits the preserved span
/// (or `None`) rather than substituting the owning token's — the marker's own
/// span is a different byte range than the attribute list's, and using it
/// would misreport a position the attribute never occupied.
pub(crate) fn owned_attribute_to_wire(attribute: &NativeOwnedAttribute) -> AttributeItem {
    AttributeItem {
        span: attribute.span.map(map_span),
        text: attribute.source.to_string(),
        key: attribute.key.to_string(),
        value: attribute.value.to_string(),
        is_default: attribute.is_default,
    }
}

pub(crate) fn token_value_to_format_token(value: Token) -> NativeFormatToken {
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
        attribute_source: value.attribute_source,
        attributes: value
            .attributes
            .iter()
            .map(wire_attribute_to_owned)
            .collect(),
    }
}

pub(crate) fn format_token_with_identity(token: &NativeToken<'_>) -> NativeFormatToken {
    let mut owned = NativeFormatToken::from(token);
    owned.sid = token.sid.map(format_sid);
    owned.id = Some(format!("{}-{}", token.id.book_code, token.id.index));
    owned
}

pub(crate) fn token_fix_into_native(value: TokenFix) -> NativeTokenFix {
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

pub(crate) fn parse_token_template(value: TokenTemplate) -> usfm_onion::TokenTemplate {
    usfm_onion::TokenTemplate {
        kind: value.kind.into(),
        text: value.text,
        marker: value.marker,
        sid: value.sid,
    }
}

pub(crate) fn parse_structural_info(value: StructuralMarkerInfo) -> NativeStructuralMarkerInfo {
    NativeStructuralMarkerInfo {
        scope_kind: value.scope_kind.into(),
        inline_context: value.inline_context.map(Into::into),
        note_context: value.note_context.map(Into::into),
    }
}

pub(crate) fn parse_number_info(value: NumberInfo) -> (u32, Option<u32>, NativeNumberRangeKind) {
    (value.start, value.end, value.kind.into())
}

pub(crate) fn map_tokens(tokens: &[NativeToken<'_>]) -> Vec<Token> {
    tokens.iter().map(map_token).collect()
}

// Native token → wire DTO. The conversion body lives in `usfm_onion_wire::dto`
// (`From<&Token> for Token`) so the wasm and native-Tauri consumers share one
// definition; this wrapper stays for the fn-pointer call sites (`map_token`
// passed into `map_native_skeleton` / `map_diffs_by_chapter`).
pub(crate) fn map_token(token: &NativeToken<'_>) -> Token {
    token.into()
}

pub(crate) fn map_format_token(token: &NativeFormatToken) -> Token {
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
        // The real structured list, not an unconditional empty vec: an
        // editor that edits an attribute's value clears `attributeSource`
        // (the whole-list attr-edit contract) and relies on `attributes`
        // being the authoritative copy from then on. `FormatToken` carries
        // both (see `attribute_source` and `attributes` there); this must
        // return both or a structurally-edited attribute silently reverts.
        attributes: token
            .attributes
            .iter()
            .map(owned_attribute_to_wire)
            .collect(),
        attribute_source: token.attribute_source.clone(),
        // A format pass's working token carries no remembered placement by design —
        // formatting may move tokens, so a distance from an old neighbour would be a
        // lie — and the emitter places its lists at the marker's closer.
        attribute_offset: None,
    }
}

pub(crate) fn map_structural_info(info: NativeStructuralMarkerInfo) -> StructuralMarkerInfo {
    info.into()
}

pub(crate) fn map_span(span: NativeSpan) -> Span {
    span.into()
}

pub(crate) fn native_span(span: Span) -> NativeSpan {
    span.into()
}

pub(crate) fn map_cst_document(document: &NativeCstDocument<'_>) -> CstDocument {
    CstDocument {
        tokens: map_tokens(&document.tokens),
        roots: document.roots.iter().map(map_cst_node).collect(),
    }
}

pub(crate) fn map_cst_node(node: &NativeCstNode) -> CstNode {
    CstNode {
        token_index: node.token_index,
        children: node.children.iter().map(map_cst_node).collect(),
    }
}

pub(crate) fn map_lint_result(result: NativeLintResult) -> LintResult {
    LintResult {
        issues: result.issues.into_iter().map(map_lint_issue).collect(),
        summary: map_lint_summary(result.summary),
    }
}

pub(crate) fn map_lint_summary(summary: usfm_onion::LintSummary) -> LintSummary {
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

pub(crate) fn map_token_fix(fix: NativeTokenFix) -> TokenFix {
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

pub(crate) fn map_token_template(template: usfm_onion::TokenTemplate) -> TokenTemplate {
    TokenTemplate {
        kind: template.kind.into(),
        text: template.text,
        marker: template.marker,
        sid: template.sid,
    }
}

pub(crate) fn map_lint_issue(issue: usfm_onion::LintIssue) -> LintIssue {
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

pub(crate) fn map_native_skeleton<T: DiffableToken>(
    skeleton: &NativeDiffSkeleton<T>,
    map_token: impl Fn(&T) -> Token,
    text_diff_mode: NativeTextDiffMode,
) -> DiffSkeleton {
    DiffSkeleton {
        slots: skeleton.slots.iter().map(map_native_slot).collect(),
        units: skeleton
            .units
            .iter()
            .map(|unit| map_native_decision_unit(unit, &map_token, text_diff_mode))
            .collect(),
    }
}

pub(crate) fn map_native_slot(slot: &NativeSlot) -> Slot {
    Slot {
        unit_id: slot.unit_id.to_string(),
        role: slot.role.into(),
        after: slot.after.as_ref().map(map_native_anchor),
    }
}

pub(crate) fn map_native_anchor(anchor: &NativeAnchor) -> Anchor {
    Anchor {
        unit_id: anchor.unit_id.to_string(),
        sid: anchor.sid.clone(),
    }
}

pub(crate) fn map_native_decision_unit<T: DiffableToken>(
    unit: &NativeDecisionUnit<T>,
    map_token: &impl Fn(&T) -> Token,
    text_diff_mode: NativeTextDiffMode,
) -> DecisionUnit {
    DecisionUnit {
        id: unit.id.to_string(),
        kind: unit.kind.into(),
        status: unit.status.into(),
        baseline_sid: unit.baseline_sid.clone(),
        current_sid: unit.current_sid.clone(),
        baseline_tokens: unit.baseline_tokens.iter().map(map_token).collect(),
        current_tokens: unit.current_tokens.iter().map(map_token).collect(),
        displaced: unit.displaced,
        relabeled: unit.relabeled,
        dup_context: DupContext {
            baseline_count: unit.dup_context.baseline_count,
            current_count: unit.dup_context.current_count,
        },
        covered_by: unit.covered_by.as_ref().map(map_native_covered_by),
        is_whitespace_change: unit.is_whitespace_change,
        is_usfm_structure_change: unit.is_usfm_structure_change,
        text_diff: native_unit_text_diff(unit, text_diff_mode).map(Into::into),
    }
}

pub(crate) fn map_native_covered_by(covered_by: &NativeCoveredBy) -> CoveredBy {
    CoveredBy {
        unit_id: covered_by.unit_id.to_string(),
        sid: covered_by.sid.clone(),
        side: covered_by.side.into(),
    }
}

pub(crate) fn map_walk_token(token: &WalkToken) -> Token {
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
        attribute_source: None,
        attribute_offset: None,
    }
}

pub(crate) fn map_diffs_by_chapter<T: DiffableToken>(
    by_chapter: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<u32, NativeDiffSkeleton<T>>,
    >,
    map_token: impl Fn(&T) -> Token,
    text_diff_mode: NativeTextDiffMode,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<u32, DiffSkeleton>> {
    by_chapter
        .iter()
        .map(|(book, chapters)| {
            (
                book.clone(),
                chapters
                    .iter()
                    .map(|(chapter, skeleton)| {
                        (
                            *chapter,
                            map_native_skeleton(skeleton, &map_token, text_diff_mode),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

pub(crate) fn vref_to_object(map: NativeVrefMap) -> std::collections::BTreeMap<String, String> {
    map.into_iter().collect()
}

pub(crate) fn vref_options_into_native(options: Option<VrefOptions>) -> NativeVrefOptions {
    let defaults = NativeVrefOptions::default();
    let Some(options) = options else {
        return defaults;
    };
    NativeVrefOptions {
        trim: options.trim.unwrap_or(defaults.trim),
    }
}

pub(crate) fn map_vref_index(index: NativeVrefIndex) -> VrefIndex {
    VrefIndex(
        index
            .iter()
            .map(|(sid, projection)| (sid.to_string(), map_verse_projection(projection.clone())))
            .collect(),
    )
}

pub(crate) fn map_verse_projection(projection: NativeVerseProjection) -> VerseProjection {
    VerseProjection {
        text: projection.text,
        segments: projection.segments.into_iter().map(map_segment).collect(),
    }
}

pub(crate) fn map_segment(segment: NativeSegment) -> Segment {
    Segment {
        token_id: segment.token_id,
        source_span: map_span(segment.source_span),
        text_span: map_utf16_span(segment.text_span),
    }
}

pub(crate) fn map_utf16_span(span: NativeUtf16Span) -> Utf16Span {
    Utf16Span {
        start: span.start,
        end: span.end,
    }
}

pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    swb_to_js_value(value).map_err(js_serde_error)
}

pub(crate) fn apply_opt(target: &mut bool, value: Option<bool>) {
    if let Some(value) = value {
        *target = value;
    }
}

pub(crate) fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

pub(crate) fn js_serde_error(error: serde_wasm_bindgen::Error) -> JsError {
    js_error(error)
}

pub(crate) fn lint_code_variants() -> Vec<NativeLintCode> {
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
        NativeLintCode::DuplicateVerseNumber,
        NativeLintCode::InvalidNumberRange,
        NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        NativeLintCode::MissingWhitespaceBeforeMarker,
        NativeLintCode::MissingHorizontalWhitespaceAfterMarkerName,
        NativeLintCode::MissingTagEndDelimiterAfterMarker,
        NativeLintCode::MissingContentSpaceAfterCloseMarker,
        NativeLintCode::VerseInSectionOrOtherParagraph,
        NativeLintCode::ContentAfterBlankMarker,
        NativeLintCode::InvalidBookCode,
        NativeLintCode::BookCodeNotUppercase,
    ]
}

// Stable wire-format key for a native TokenKind. DiffableToken's kind_key
// returns a borrowed `&str`, so this stays as a small lookup rather than
// going through a serde round-trip on each token.
pub(crate) fn token_kind_wire_key(kind: NativeTokenKind) -> &'static str {
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
