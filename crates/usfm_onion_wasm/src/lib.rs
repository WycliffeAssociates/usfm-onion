use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value as swb_to_js_value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use usfm_onion::cst::{CstDocument as NativeCstDocument, CstNode as NativeCstNode, parse_cst};
use usfm_onion::diff::{
    Anchor as NativeAnchor, CoveredBy as NativeCoveredBy, DecisionUnit as NativeDecisionUnit,
    DiffSkeleton as NativeDiffSkeleton, DiffableToken, Slot as NativeSlot,
    TextDiffMode as NativeTextDiffMode, UnitId as NativeUnitId,
    derive_canonical_sids as native_derive_canonical_sids,
    diff_skeleton as native_diff_skeleton,
    diff_skeleton_by_chapter as native_diff_skeleton_by_chapter,
    diff_skeleton_canonical as native_diff_skeleton_canonical,
    merge_diff_blocks as native_merge_diff_blocks, revert_diff_block as native_revert_diff_block,
    unit_text_diff as native_unit_text_diff,
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
    LintCode as NativeLintCode, LintOptions as NativeLintOptions, LintResult as NativeLintResult,
    LintScope as NativeLintScope, LintSuppression as NativeLintSuppression, LintableToken,
    TokenFix as NativeTokenFix, apply_token_fix, lint_tokens, lint_usfm,
};
use usfm_onion::marker_defs::StructuralMarkerInfo as NativeStructuralMarkerInfo;
use usfm_onion::markers::{is_known_marker, marker_catalog, marker_info};
use usfm_onion::parse::parse as native_parse;
use usfm_onion::token::{
    NumberRangeKind as NativeNumberRangeKind, Span as NativeSpan, Token as NativeToken,
    TokenKind as NativeTokenKind, tokens_to_usfm, tokens_to_usfm_reconstruct,
};
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::{
    Segment as NativeSegment, Utf16Span as NativeUtf16Span,
    VerseProjection as NativeVerseProjection, VrefIndex as NativeVrefIndex,
    VrefMap as NativeVrefMap, VrefOptions as NativeVrefOptions, tokens_to_vref_index,
    usfm_to_vref_index, usfm_to_vref_map_with_options,
};
use usfm_onion::walker::WalkableToken;
pub use usfm_onion_dto::{
    AttributeItem, BlockBehavior, ClosingBehavior, CoveredSide, DecisionStatus, DecisionUnitKind,
    DiffOptions, HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, InlineContext, LintCategory,
    LintCode, LintIssueType, LintSeverity, MarkerCategory, MarkerDefKind, MarkerFamily,
    MarkerFamilyRole, MarkerInfo, MarkerKind, MarkerMetadata, MarkerPayload, MergeSide,
    NoteFamily, NoteSubkind, NumberInfo, NumberRangeKind, ParagraphCategory, SlotRole, Span,
    SpecContext, StructuralMarkerInfo, StructuralScopeKind, TextDiffMode, TextDiffRun,
    TextDiffRunKind, Token, TokenKind, UnitTextDiff, format_sid, map_marker_info,
};

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
    trim: Option<bool>,
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
    scope: LintScope,
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

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    unit_id: String,
    sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    unit_id: String,
    role: SlotRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after: Option<Anchor>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DupContext {
    baseline_count: u32,
    current_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CoveredBy {
    unit_id: String,
    sid: String,
    side: CoveredSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DecisionUnit {
    id: String,
    kind: DecisionUnitKind,
    status: DecisionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    baseline_sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_sid: Option<String>,
    baseline_tokens: Vec<Token>,
    current_tokens: Vec<Token>,
    displaced: bool,
    relabeled: bool,
    dup_context: DupContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    covered_by: Option<CoveredBy>,
    is_whitespace_change: bool,
    is_usfm_structure_change: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_diff: Option<UnitTextDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DiffSkeleton {
    slots: Vec<Slot>,
    units: Vec<DecisionUnit>,
}

/// Staged decisions for [`wasm_merge_diff_blocks`]: `Record<string,
/// MergeSide>` plus the default applied to any unit not present in the map.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MergeRequest {
    decisions: std::collections::BTreeMap<String, MergeSide>,
    default_side: MergeSide,
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

    pub fn lint(&self, options: LintOptions) -> LintResult {
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
    ) -> Result<Vec<Token>, JsError> {
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
        let reverted = native_revert_diff_block(block_id, &baseline, &current).map_err(js_error)?;
        Ok(reverted.iter().map(map_format_token).collect())
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
    pub fn to_vref(&self, options: Option<VrefOptions>) -> VrefMap {
        let options = vref_options_into_native(options);
        VrefMap(vref_to_object(usfm_to_vref_map_with_options(
            &self.source,
            options,
        )))
    }

    #[wasm_bindgen(js_name = vrefIndex)]
    pub fn vref_index(&self) -> VrefIndex {
        map_vref_index(usfm_to_vref_index(&self.source))
    }

    pub fn diff(&self, other: &ParsedUsfm, options: Option<DiffOptions>) -> DiffSkeleton {
        map_native_skeleton(
            &native_diff_usfm(&self.source, &other.source),
            map_token,
            diff_options_into_native(options),
        )
    }

    #[wasm_bindgen(js_name = diffByChapter)]
    pub fn diff_by_chapter(
        &self,
        other: &ParsedUsfm,
        options: Option<DiffOptions>,
    ) -> DiffsByChapterMap {
        DiffsByChapterMap(map_diffs_by_chapter(
            &native_diff_skeleton_by_chapter(&self.source, &other.source),
            map_token,
            diff_options_into_native(options),
        ))
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
pub fn wasm_lint_usfm(source: &str, options: LintOptions) -> LintResult {
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
pub fn wasm_lint_tokens(tokens: Vec<Token>, options: LintOptions) -> LintResult {
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
    tokens_to_usfm_reconstruct(&tokens)
}

#[wasm_bindgen(js_name = tokensToHtml)]
pub fn wasm_tokens_to_html(tokens: Vec<Token>, options: Option<HtmlOptions>) -> String {
    let usfm = tokens_to_usfm_reconstruct(&tokens);
    usfm_to_html(&usfm, html_options_into_native(options))
}

fn native_diff_usfm<'a>(
    baseline_usfm: &'a str,
    current_usfm: &'a str,
) -> NativeDiffSkeleton<NativeToken<'a>> {
    let baseline = native_parse(baseline_usfm);
    let current = native_parse(current_usfm);
    let baseline_book = baseline.analysis.book_code.unwrap_or("unknown");
    let current_book = current.analysis.book_code.unwrap_or("unknown");
    native_diff_skeleton_canonical(
        &baseline.tokens,
        baseline_book,
        &current.tokens,
        current_book,
    )
}

#[wasm_bindgen(js_name = diffUsfm)]
pub fn wasm_diff_usfm(left: &str, right: &str, options: Option<DiffOptions>) -> DiffSkeleton {
    map_native_skeleton(
        &native_diff_usfm(left, right),
        map_token,
        diff_options_into_native(options),
    )
}

#[wasm_bindgen(js_name = diffUsfmByChapter)]
pub fn wasm_diff_usfm_by_chapter(
    left: &str,
    right: &str,
    options: Option<DiffOptions>,
) -> DiffsByChapterMap {
    DiffsByChapterMap(map_diffs_by_chapter(
        &native_diff_skeleton_by_chapter(left, right),
        map_token,
        diff_options_into_native(options),
    ))
}

#[wasm_bindgen(js_name = diffTokens)]
pub fn wasm_diff_tokens(
    left: Vec<Token>,
    right: Vec<Token>,
    options: Option<DiffOptions>,
) -> DiffSkeleton {
    let left = parse_walk_tokens_from_values(left);
    let right = parse_walk_tokens_from_values(right);
    map_native_skeleton(
        &native_diff_skeleton(&left, &right),
        map_walk_token,
        diff_options_into_native(options),
    )
}

#[wasm_bindgen(js_name = mergeDiffBlocks)]
pub fn wasm_merge_diff_blocks(
    baseline: Vec<Token>,
    current: Vec<Token>,
    request: MergeRequest,
) -> Result<Vec<Token>, JsError> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let decisions = request
        .decisions
        .into_iter()
        .map(|(id, side)| (NativeUnitId::new(id), side.into()))
        .collect();
    let merged =
        native_merge_diff_blocks(&baseline, &current, &decisions, request.default_side.into())
            .map_err(js_error)?;
    Ok(merged.iter().map(map_format_token).collect())
}

#[wasm_bindgen(js_name = revertDiffBlock)]
pub fn wasm_revert_diff_block(
    baseline: Vec<Token>,
    current: Vec<Token>,
    block_id: &str,
) -> Result<Vec<Token>, JsError> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let reverted = native_revert_diff_block(block_id, &baseline, &current).map_err(js_error)?;
    Ok(reverted.iter().map(map_format_token).collect())
}

#[wasm_bindgen(js_name = normalizeTokenSids)]
pub fn wasm_normalize_token_sids(tokens: Vec<Token>, book_code: &str) -> Vec<Token> {
    let native_tokens: Vec<NativeFormatToken> = tokens
        .iter()
        .cloned()
        .map(token_value_to_format_token)
        .collect();
    let canonical = native_derive_canonical_sids(&native_tokens, book_code);
    tokens
        .into_iter()
        .zip(canonical)
        .map(|(mut token, sid)| {
            token.sid = Some(sid);
            token
        })
        .collect()
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

fn lint_scope_into_native(value: LintScope) -> NativeLintScope {
    match value {
        LintScope::Front => NativeLintScope::Front,
        LintScope::Chapter(n) => NativeLintScope::Chapter(n),
        LintScope::Book => NativeLintScope::Book,
    }
}

fn lint_options_into_native(value: LintOptions) -> NativeLintOptions {
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

/// Omitting `options` (or omitting `textDiff` on a supplied `DiffOptions`)
/// resolves to `TextDiffMode::None` — today's behavior, computing nothing.
fn diff_options_into_native(value: Option<DiffOptions>) -> NativeTextDiffMode {
    value.unwrap_or_default().into()
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
    owned.sid = token.sid.map(format_sid);
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

// Native token → wire DTO. The conversion body lives in `usfm_onion_dto`
// (`From<&Token> for Token`) so the wasm and native-Tauri consumers share one
// definition; this wrapper stays for the fn-pointer call sites (`map_token`
// passed into `map_native_skeleton` / `map_diffs_by_chapter`).
fn map_token(token: &NativeToken<'_>) -> Token {
    token.into()
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
        attribute_source: None,
    }
}

fn map_structural_info(info: NativeStructuralMarkerInfo) -> StructuralMarkerInfo {
    info.into()
}

fn map_span(span: NativeSpan) -> Span {
    span.into()
}

fn native_span(span: Span) -> NativeSpan {
    span.into()
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

fn map_native_skeleton<T: DiffableToken>(
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

fn map_native_slot(slot: &NativeSlot) -> Slot {
    Slot {
        unit_id: slot.unit_id.to_string(),
        role: slot.role.into(),
        after: slot.after.as_ref().map(map_native_anchor),
    }
}

fn map_native_anchor(anchor: &NativeAnchor) -> Anchor {
    Anchor {
        unit_id: anchor.unit_id.to_string(),
        sid: anchor.sid.clone(),
    }
}

fn map_native_decision_unit<T: DiffableToken>(
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

fn map_native_covered_by(covered_by: &NativeCoveredBy) -> CoveredBy {
    CoveredBy {
        unit_id: covered_by.unit_id.to_string(),
        sid: covered_by.sid.clone(),
        side: covered_by.side.into(),
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
        attribute_source: None,
    }
}

fn map_diffs_by_chapter<T: DiffableToken>(
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
                        (*chapter, map_native_skeleton(skeleton, &map_token, text_diff_mode))
                    })
                    .collect(),
            )
        })
        .collect()
}

fn vref_to_object(map: NativeVrefMap) -> std::collections::BTreeMap<String, String> {
    map.into_iter().collect()
}

fn vref_options_into_native(options: Option<VrefOptions>) -> NativeVrefOptions {
    let defaults = NativeVrefOptions::default();
    let Some(options) = options else {
        return defaults;
    };
    NativeVrefOptions {
        trim: options.trim.unwrap_or(defaults.trim),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(source: &str) -> String {
        let tokens = map_tokens(&native_parse(source).tokens);
        tokens_to_usfm_reconstruct(&tokens)
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
    fn round_trips_non_canonical_attribute_whitespace_via_wire() {
        // Extra inter-item spacing (`lemma = "y"` instead of `lemma="y"`) is
        // trivia only a verbatim slice preserves. Before `attributeSource`
        // was added to the wire DTO, this normalized to canonical spacing
        // when it crossed native -> DTO -> wasm; now the verbatim
        // `attribute_source` rides along the wire and this round-trips
        // byte-identical.
        let source = "\\w x|lemma = \"y\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    /// Parity contract for `normalizeTokenSids`, pinned on the onion side.
    ///
    /// The other half of this contract — comparing these exact values
    /// against a live Zephyr `mutAddSids(tokens, bookCode)` call — cannot
    /// run in this repo: `mutAddSids` is Zephyr application code, not a
    /// dependency of onion, and vendoring or importing it here would defeat
    /// the point of keeping the two implementations independent. This test
    /// is onion's half of the parity gate: the exact sid strings
    /// `normalizeTokenSids` must produce for bridge, duplicate, and intro
    /// streams, which Zephyr's consumer migration diffs its own
    /// `mutAddSids` output against before retiring it (see
    /// `plans/zephyr-handoff-merge-projection.md`). If this test ever
    /// changes, that is a signal the Zephyr-side comparison needs
    /// re-running, not just a golden update.
    #[test]
    fn normalize_token_sids_bridge_dup_intro_parity_contract() {
        let source = "\\id GEN\n\\h Genesis\n\\c 1\n\\v 1 a\n\\v 1-2 b\n\\v 1 c\n";
        let tokens = map_tokens(&native_parse(source).tokens);
        let normalized = wasm_normalize_token_sids(tokens.clone(), "GEN");

        assert_eq!(
            tokens.len(),
            normalized.len(),
            "normalizeTokenSids must preserve length/order"
        );
        for (before, after) in tokens.iter().zip(&normalized) {
            assert_eq!(before.id, after.id, "id must be untouched");
            assert_eq!(before.source, after.source, "source/text must be untouched");
            assert_eq!(
                format!("{:?}", before.kind),
                format!("{:?}", after.kind),
                "kind must be untouched"
            );
        }

        let sid_of = |text: &str| -> String {
            normalized
                .iter()
                .find(|t| t.source.trim() == text)
                .unwrap_or_else(|| panic!("no token with text {text:?}"))
                .sid
                .clone()
                .unwrap_or_else(|| panic!("token {text:?} has no sid"))
        };

        // intro: book code precedes any \c.
        assert_eq!(sid_of("Genesis"), "GEN 0:0");
        // single verse.
        assert_eq!(sid_of("a"), "GEN 1:1");
        // bridge: range end encoded.
        assert_eq!(sid_of("b"), "GEN 1:1-2");
        // duplicate: second occurrence of the same base sid gets _dup_1.
        assert_eq!(sid_of("c"), "GEN 1:1_dup_1");
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
                attribute_source: None,
            },
        );
        // Attribute slice still lands right before \w*, independent of
        // the inserted Text token between word content and closer.
        assert_eq!(
            tokens_to_usfm_reconstruct(&tokens),
            "\\w hello world|lemma=\"x\"\\w*"
        );
    }

    #[test]
    fn round_trips_embedded_double_quote_in_attribute_value_via_reconstruct() {
        // Source uses USFM 3.1 + the usfm-onion `\"` extension (D3). Decode-on-map
        // turns `\"` into `"` on the JS-facing value. Clearing `attributeSource`
        // simulates an editor having touched this attribute (the documented
        // rule: touch an attribute -> drop its verbatim, onion reconstructs
        // from structure) — the reconstruct path must re-escape the literal
        // quote for byte-identical output.
        let source = "\\w word|note=\"a\\\"b\"\\w*";
        let mut tokens = map_tokens(&native_parse(source).tokens);
        let attr_token = tokens
            .iter_mut()
            .find(|t| !t.attributes.is_empty())
            .expect("expected attribute-bearing token");
        assert_eq!(attr_token.attributes[0].value, "a\"b");
        attr_token.attribute_source = None;
        assert_eq!(tokens_to_usfm_reconstruct(&tokens), source);
    }

    #[test]
    fn empty_token_stream_emits_empty_string() {
        assert_eq!(tokens_to_usfm_reconstruct::<Token>(&[]), "");
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
        let result = lint_tokens(
            &adapter_tokens,
            NativeLintOptions::scoped(NativeLintScope::Book),
        );

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
        let result = lint_tokens(
            &adapter_tokens,
            NativeLintOptions::scoped(NativeLintScope::Book),
        );

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
            .map(|tokens| lint_tokens(tokens, NativeLintOptions::scoped(NativeLintScope::Book)))
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
        let code = usfm_onion::LintCode::DuplicateVerseNumber;
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

    // --- DiffOptions / textDiff wire threading (Gate 4) ---
    //
    // `DiffOptions`'s `text_diff` field is intentionally private (mirroring
    // the plan's sketch) — the wasm boundary only ever builds one by
    // deserializing the JS-facing wire shape, never by Rust field
    // construction. These tests do the same.

    fn diff_options(text_diff: &str) -> DiffOptions {
        serde_json::from_value(serde_json::json!({ "textDiff": text_diff }))
            .expect("DiffOptions must deserialize from its wire shape")
    }

    #[test]
    fn omitted_and_explicit_none_diff_options_produce_no_text_diff_and_match_byte_for_byte() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n\\v 2 old text\n\\v 3 same text\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n\\v 3 same text\n\\v 4 new text\n";

        let omitted = wasm_diff_usfm(baseline, current, None);
        let explicit_none = wasm_diff_usfm(baseline, current, Some(diff_options("none")));

        for skeleton in [&omitted, &explicit_none] {
            assert!(
                skeleton.units.iter().all(|u| u.text_diff.is_none()),
                "omitted/\"none\" DiffOptions must compute no text_diff on any unit"
            );
        }
        // Additive/back-compatible per the plan: omitting the option must be
        // byte-identical to passing an explicit "none" — and, since neither
        // computes a text_diff, both must be byte-identical to a caller that
        // never knew this feature existed.
        assert_eq!(
            serde_json::to_string(&omitted).unwrap(),
            serde_json::to_string(&explicit_none).unwrap()
        );
    }

    #[test]
    fn words_mode_populates_text_diff_on_changed_units_only() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n\\v 2 old text\n\\v 3 same text\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n\\v 3 same text\n\\v 4 new text\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));

        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit (v1 heaven -> heavens)");
        assert!(modified.text_diff.is_some(), "Modified must carry a text_diff");

        let added = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Added))
            .expect("expected one Added unit (v4)");
        assert!(added.text_diff.is_some(), "Added must carry a text_diff");

        let deleted = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Deleted))
            .expect("expected one Deleted unit (v2)");
        assert!(deleted.text_diff.is_some(), "Deleted must carry a text_diff");

        let unchanged = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Unchanged))
            .expect("expected one Unchanged unit (v3)");
        assert!(
            unchanged.text_diff.is_none(),
            "Unchanged (byte-equal) must not carry a text_diff"
        );
    }

    #[test]
    fn words_mode_leaves_a_pure_move_without_a_text_diff() {
        // Verses swapped, byte-identical text either side -> Moved.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n";
        let current = "\\id GEN\n\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));
        let moved = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Moved))
            .expect("expected one Moved unit");
        assert!(
            moved.text_diff.is_none(),
            "a pure move (byte-equal text) must not read as a content change"
        );
    }

    #[test]
    fn words_mode_current_and_baseline_runs_highlight_the_changed_word() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));
        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit");
        let text_diff = modified
            .text_diff
            .as_ref()
            .expect("Modified unit must carry a text_diff under \"words\"");

        assert!(
            text_diff
                .current
                .iter()
                .any(|r| r.text == "heavens" && matches!(r.kind, TextDiffRunKind::Added)),
            "current runs must highlight the new word \"heavens\": {:?}",
            text_diff.current
        );
        assert!(
            text_diff
                .baseline
                .iter()
                .any(|r| r.text == "heaven" && matches!(r.kind, TextDiffRunKind::Removed)),
            "baseline runs must highlight the removed word \"heaven\": {:?}",
            text_diff.baseline
        );
    }

    #[test]
    fn chars_mode_is_also_threaded_through_diff_options() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("chars")));
        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit");
        let text_diff = modified
            .text_diff
            .as_ref()
            .expect("Modified unit must carry a text_diff under \"chars\"");
        assert!(
            text_diff
                .current
                .iter()
                .any(|r| r.text == "s" && matches!(r.kind, TextDiffRunKind::Added)),
            "grapheme-level diff must isolate the added \"s\": {:?}",
            text_diff.current
        );
    }

    // --- native <-> wasm parity fixture (Gate 5) ---
    //
    // The Gate 4 tests above prove the DTO path *populates* text_diff and pin
    // a few ASCII runs. This fixture proves the wasm DTO path is byte-identical
    // to the native `unit_text_diff` computation for the HARDENED multilingual
    // pins that `src/diff/text_diff_fixtures.rs` gates (apostrophe, Hebrew
    // niqqud, Arabic RTL, Thai, punctuation-adjacent). The wasm entry point
    // (`wasm_diff_usfm`) and the native reference (`native_diff_usfm` +
    // `native_unit_text_diff`) walk the SAME canonical-sid skeleton, so any
    // drift in `map_native_skeleton`, the `From<Native*>` conversions, or the
    // serde rename would surface here — and it's a plain cargo test, no pkg
    // build. This is the seam the fixture oracle can't reach: those run the
    // native fn directly; these run it through the wasm boundary's DTO path.

    /// The single Modified unit's `text_diff` as produced by the wasm DTO
    /// entry point.
    fn wasm_path_modified_text_diff(
        baseline: &str,
        current: &str,
        mode: &str,
    ) -> Option<UnitTextDiff> {
        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options(mode)));
        let modified = skeleton
            .units
            .into_iter()
            .filter(|u| matches!(u.status, DecisionStatus::Modified))
            .collect::<Vec<_>>();
        assert_eq!(modified.len(), 1, "fixture must have exactly one Modified unit");
        modified.into_iter().next().unwrap().text_diff
    }

    /// The same unit's `text_diff` computed natively (canonical-sid skeleton,
    /// exactly as the wasm path builds it) and converted into the wire DTO.
    fn native_path_modified_text_diff(
        baseline: &str,
        current: &str,
        mode: &str,
    ) -> Option<UnitTextDiff> {
        use usfm_onion::diff::DecisionStatus as NativeDecisionStatus;
        let native_mode = NativeTextDiffMode::from(diff_options(mode));
        let skeleton = native_diff_usfm(baseline, current);
        let modified = skeleton
            .units
            .iter()
            .filter(|u| matches!(u.status, NativeDecisionStatus::Modified))
            .collect::<Vec<_>>();
        assert_eq!(modified.len(), 1, "fixture must have exactly one Modified unit");
        native_unit_text_diff(modified[0], native_mode).map(Into::into)
    }

    fn assert_wasm_matches_native(baseline: &str, current: &str, mode: &str, label: &str) {
        let wasm = wasm_path_modified_text_diff(baseline, current, mode);
        let native = native_path_modified_text_diff(baseline, current, mode);
        // Non-trivial: two Nones would pass vacuously, but every pin here is a
        // Modified content change, so both sides MUST carry runs.
        assert!(
            wasm.is_some(),
            "{label}: wasm path produced no text_diff for a Modified unit"
        );
        assert_eq!(
            wasm, native,
            "{label}: wasm DTO text_diff must byte-match the native computation"
        );
    }

    #[test]
    fn parity_apostrophe_straight_vs_curly_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 They don't know.\n",
            "\\id GEN\n\\c 1\n\\v 1 They don\u{2019}t know.\n",
            "words",
            "apostrophe straight vs curly",
        );
    }

    #[test]
    fn parity_hebrew_combining_niqqud_chars() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05E8}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05BC}\u{05E8}\n",
            "chars",
            "Hebrew combining niqqud",
        );
    }

    #[test]
    fn parity_arabic_rtl_word_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0644}\u{0647}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0631}\u{0628}\n",
            "words",
            "Arabic RTL word edit",
        );
    }

    #[test]
    fn parity_thai_no_space_segment_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E42}\u{0E25}\u{0E01}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E1B}\u{0E23}\u{0E30}\u{0E40}\u{0E17}\u{0E28}\n",
            "words",
            "Thai no-space segment edit",
        );
    }

    #[test]
    fn parity_latin_punctuation_adjacent_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 Hello, world!\n",
            "\\id GEN\n\\c 1\n\\v 1 Hello, there!\n",
            "words",
            "Latin punctuation-adjacent edit",
        );
    }
}
