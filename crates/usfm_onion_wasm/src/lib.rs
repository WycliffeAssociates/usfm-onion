use js_sys::Array;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value as from_js_value, to_value as swb_to_js_value};
use wasm_bindgen::prelude::*;

use usfm_onion::cst::{CstDocument as NativeCstDocument, CstNode as NativeCstNode, parse_cst};
use usfm_onion::diff::{
    BuildSidBlocksOptions as NativeBuildSidBlocksOptions, ChapterTokenDiff as NativeChapterTokenDiff,
    DiffStatus as NativeDiffStatus, DiffTokenChange as NativeDiffTokenChange,
    DiffUndoSide as NativeDiffUndoSide, DiffableToken, DiffsByChapterMap as NativeDiffsByChapterMap,
    SidBlock as NativeSidBlock, TokenAlignment as NativeTokenAlignment, apply_revert_by_block_id,
    apply_reverts_by_block_id, diff_chapter_token_streams, diff_usfm_sources,
    diff_usfm_sources_by_chapter,
};
use usfm_onion::format::{
    FormatOptions as NativeFormatOptions, FormatRule as NativeFormatRule,
    FormatToken as NativeFormatToken, format_tokens as native_format_tokens,
    format_tokens_to_usfm, format_usfm,
};
use usfm_onion::html::{
    HtmlCallerScope as NativeHtmlCallerScope, HtmlCallerStyle as NativeHtmlCallerStyle,
    HtmlNoteMode as NativeHtmlNoteMode, HtmlOptions as NativeHtmlOptions, usfm_to_html,
};
use usfm_onion::lint::{
    ApplyTokenFixesResult as NativeApplyTokenFixesResult, AppliedTokenFix as NativeAppliedTokenFix,
    LintCategory as NativeLintCategory, LintCode as NativeLintCode,
    LintOptions as NativeLintOptions, LintResult as NativeLintResult,
    LintSeverity as NativeLintSeverity, LintSuppression as NativeLintSuppression, LintableToken,
    apply_token_fixes, lint_tokens, lint_usfm,
};
use usfm_onion::marker_defs::{
    BlockBehavior, ClosingBehavior, InlineContext, MarkerFamily, MarkerFamilyRole, SpecContext,
    StructuralMarkerInfo, StructuralScopeKind,
};
use usfm_onion::markers::{
    MarkerCategory as NativeMarkerCategory, MarkerInlineContext as NativeMarkerInlineContext,
    MarkerKind as NativeMarkerKind, MarkerNoteFamily as NativeMarkerNoteFamily,
    MarkerNoteSubkind as NativeMarkerNoteSubkind, UsfmMarkerInfo as NativeUsfmMarkerInfo,
    is_known_marker, marker_catalog, marker_info,
};
use usfm_onion::parse::parse as native_parse;
use usfm_onion::token::{
    AttributeItem as NativeAttributeItem, MarkerMetadata as NativeMarkerMetadata,
    NumberRangeKind as NativeNumberRangeKind, Span as NativeSpan, Token as NativeToken,
    TokenData as NativeTokenData, TokenKind as NativeTokenKind,
    tokens_to_usfm,
};
use usfm_onion::usj::{UsjDocument, usfm_to_usj};
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::{VrefMap, usfm_to_vref_map};

#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
export type Span = { start: number; end: number };
export type TokenKind =
  | "newline"
  | "optBreak"
  | "marker"
  | "endMarker"
  | "milestone"
  | "milestoneEnd"
  | "bookCode"
  | "number"
  | "text"
  | "attributeList";
export type NumberRangeKind = "single" | "range" | "sequence" | "sequenceWithRange";
export type MarkerKind =
  | "paragraph"
  | "note"
  | "character"
  | "header"
  | "chapter"
  | "verse"
  | "milestoneStart"
  | "milestoneEnd"
  | "sidebarStart"
  | "sidebarEnd"
  | "figure"
  | "meta"
  | "periph"
  | "tableRow"
  | "tableCell"
  | "unknown";
export type MarkerCategory =
  | "document"
  | "paragraph"
  | "character"
  | "noteContainer"
  | "noteSubmarker"
  | "chapter"
  | "verse"
  | "milestoneStart"
  | "milestoneEnd"
  | "figure"
  | "sidebarStart"
  | "sidebarEnd"
  | "periph"
  | "meta"
  | "tableRow"
  | "tableCell"
  | "header"
  | "unknown";
export type MarkerNoteFamily = "footnote" | "crossReference";
export type MarkerNoteSubkind = "structural" | "structuralKeepsNestedCharsOpen";
export type MarkerInlineContext = "para" | "section" | "list" | "table";
export type MarkerFamily =
  | "footnote"
  | "crossReference"
  | "sectionParagraph"
  | "listParagraph"
  | "tableCell"
  | "milestone"
  | "sidebar";
export type MarkerFamilyRole =
  | "canonical"
  | "numberedVariant"
  | "nestedVariant"
  | "milestoneStart"
  | "milestoneEnd"
  | "alias";
export type BlockBehavior =
  | "none"
  | "paragraph"
  | "tableRow"
  | "tableCell"
  | "sidebarStart"
  | "sidebarEnd";
export type ClosingBehavior =
  | "none"
  | "requiredExplicit"
  | "optionalExplicitUntilNoteEnd"
  | "selfClosingMilestone";
export type SpecContext =
  | "scripture"
  | "bookIdentification"
  | "bookHeaders"
  | "bookTitles"
  | "bookIntroduction"
  | "bookIntroductionEndTitles"
  | "bookChapterLabel"
  | "chapterContent"
  | "peripheral"
  | "peripheralContent"
  | "peripheralDivision"
  | "chapter"
  | "verse"
  | "section"
  | "para"
  | "list"
  | "table"
  | "sidebar"
  | "footnote"
  | "crossReference";
export type StructuralScopeKind =
  | "unknown"
  | "header"
  | "block"
  | "note"
  | "character"
  | "milestone"
  | "chapter"
  | "verse"
  | "tableRow"
  | "tableCell"
  | "sidebar"
  | "periph"
  | "meta";
export type LintCategory = "document" | "structure" | "context" | "numbering";
export type LintSeverity = "error" | "warning";
export type LintCode =
  | "missing-id-marker"
  | "missing-separator-after-marker"
  | "empty-paragraph"
  | "number-range-after-chapter-marker"
  | "verse-range-expected-after-verse-marker"
  | "verse-content-not-empty"
  | "unknown-token"
  | "char-not-closed"
  | "note-not-closed"
  | "paragraph-before-first-chapter"
  | "verse-before-first-chapter"
  | "note-submarker-outside-note"
  | "duplicate-id-marker"
  | "id-marker-not-at-file-start"
  | "chapter-metadata-outside-chapter"
  | "verse-metadata-outside-verse"
  | "missing-chapter-number"
  | "missing-verse-number"
  | "missing-milestone-self-close"
  | "implicitly-closed-marker"
  | "stray-close-marker"
  | "misnested-close-marker"
  | "unclosed-note"
  | "unclosed-marker-at-eof"
  | "duplicate-chapter-number"
  | "chapter-expected-increase-by-one"
  | "duplicate-verse-number"
  | "verse-expected-increase-by-one"
  | "invalid-number-range"
  | "number-range-not-preceded-by-marker-expecting-number"
  | "verse-text-follows-verse-range"
  | "unknown-marker"
  | "unknown-close-marker"
  | "inconsistent-chapter-label"
  | "marker-not-valid-in-context"
  | "verse-outside-explicit-paragraph";
export type FormatRule =
  | "recover-malformed-markers"
  | "collapse-whitespace-in-text"
  | "ensure-inline-separators"
  | "remove-duplicate-verse-numbers"
  | "normalize-spacing-after-paragraph-markers"
  | "remove-unwanted-linebreaks"
  | "bridge-consecutive-verse-markers"
  | "remove-orphan-empty-verse-before-contentful-verse"
  | "remove-bridge-verse-enumerators"
  | "move-chapter-label-after-chapter-marker"
  | "insert-default-paragraph-after-chapter-intro"
  | "remove-empty-paragraphs"
  | "insert-structural-linebreaks"
  | "collapse-consecutive-linebreaks"
  | "normalize-marker-whitespace-at-line-start";
export type HtmlNoteMode = "extracted" | "inline";
export type HtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";
export type HtmlCallerScope = "documentSequential" | "verseSequential";
export type DiffStatus = "added" | "deleted" | "modified" | "unchanged";
export type DiffTokenChange = "unchanged" | "added" | "deleted" | "modified";
export type DiffUndoSide = "original" | "current";

export interface AttributeItem {
  span: Span;
  text: string;
  key: string;
  value: string;
}

export interface MarkerMetadata {
  canonical?: string;
  kind?: string;
  family?: MarkerFamily;
}

export interface StructuralMarkerInfo {
  scopeKind: StructuralScopeKind;
  inlineContext?: MarkerInlineContext;
  noteContext?: SpecContext;
}

export interface NumberInfo {
  start: number;
  end?: number;
  kind: NumberRangeKind;
}

export interface Token {
  id: string;
  kind: TokenKind;
  text: string;
  span?: Span;
  sid?: string;
  marker?: string;
  nested?: boolean;
  markerMetadata?: MarkerMetadata;
  structural?: StructuralMarkerInfo;
  numberInfo?: NumberInfo;
  bookCode?: string;
  bookCodeValid?: boolean;
  attributes?: AttributeItem[];
}

export type FormatToken = Token;

export interface CstNode {
  tokenIndex: number;
  children: CstNode[];
}

export interface CstDocument {
  tokens: Token[];
  roots: CstNode[];
}

export interface LintSuppression {
  code: LintCode;
  sid: string;
}

export interface LintOptions {
  enabledCodes?: LintCode[];
  disabledCodes?: LintCode[];
  suppressed?: LintSuppression[];
  allowImplicitChapterContentVerse?: boolean;
}

export interface LintIssue {
  code: LintCode;
  category: LintCategory;
  severity: LintSeverity;
  message: string;
  span?: Span;
  relatedSpan?: Span;
  tokenId?: string;
  relatedTokenId?: string;
  sid?: string;
  marker?: string;
}

export interface LintSummary {
  byCategory: Partial<Record<LintCategory, number>>;
  bySeverity: Partial<Record<LintSeverity, number>>;
  totalCount: number;
  suppressedCount: number;
}

export interface LintResult {
  issues: LintIssue[];
  summary: LintSummary;
}

export interface AppliedTokenFix {
  code: LintCode;
  tokenId?: string;
  sid?: string;
  marker?: string;
}

export interface ApplyTokenFixesResult {
  tokens: FormatToken[];
  usfm: string;
  appliedFixes: AppliedTokenFix[];
  remainingIssues: LintIssue[];
  remainingSummary: LintSummary;
}

export interface FormatOptions {
  recoverMalformedMarkers?: boolean;
  collapseWhitespaceInText?: boolean;
  ensureInlineSeparators?: boolean;
  removeDuplicateVerseNumbers?: boolean;
  normalizeSpacingAfterParagraphMarkers?: boolean;
  removeUnwantedLinebreaks?: boolean;
  bridgeConsecutiveVerseMarkers?: boolean;
  removeOrphanEmptyVerseBeforeContentfulVerse?: boolean;
  removeBridgeVerseEnumerators?: boolean;
  moveChapterLabelAfterChapterMarker?: boolean;
  insertDefaultParagraphAfterChapterIntro?: boolean;
  removeEmptyParagraphs?: boolean;
  insertStructuralLinebreaks?: boolean;
  collapseConsecutiveLinebreaks?: boolean;
  normalizeMarkerWhitespaceAtLineStart?: boolean;
}

export interface FormatResult {
  tokens: FormatToken[];
  usfm: string;
}

export interface HtmlOptions {
  wrapRoot?: boolean;
  preferNativeElements?: boolean;
  noteMode?: HtmlNoteMode;
  callerStyle?: HtmlCallerStyle;
  callerScope?: HtmlCallerScope;
}

export interface BuildSidBlocksOptions {
  allowEmptySid?: boolean;
}

export interface SidBlock {
  blockId: string;
  semanticSid: string;
  start: number;
  endExclusive: number;
  prevBlockId?: string;
  textFull: string;
}

export interface TokenAlignment {
  change: DiffTokenChange;
  counterpartIndex?: number;
}

export interface ChapterTokenDiff {
  blockId: string;
  semanticSid: string;
  status: DiffStatus;
  original?: SidBlock;
  current?: SidBlock;
  originalText: string;
  currentText: string;
  originalTextOnly: string;
  currentTextOnly: string;
  isWhitespaceChange: boolean;
  isUsfmStructureChange: boolean;
  originalTokens: Token[];
  currentTokens: Token[];
  originalAlignment: TokenAlignment[];
  currentAlignment: TokenAlignment[];
  undoSide: DiffUndoSide;
}

export type DiffsByChapterMap = Record<string, Record<number, ChapterTokenDiff[]>>;
export type VrefMap = Record<string, string>;

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

export interface LintCodeMeta {
  code: LintCode;
  category: LintCategory;
  severity: LintSeverity;
}

export interface FormatRuleMeta {
  code: FormatRule;
  labelKey: string;
}

export interface MarkerInfo {
  marker: string;
  canonical?: string;
  known: boolean;
  deprecated: boolean;
  category: MarkerCategory;
  kind: MarkerKind;
  family?: MarkerFamily;
  familyRole?: MarkerFamilyRole;
  noteFamily?: MarkerNoteFamily;
  noteSubkind?: MarkerNoteSubkind;
  inlineContext?: MarkerInlineContext;
  defaultAttribute?: string;
  contexts: SpecContext[];
  blockBehavior?: BlockBehavior;
  closingBehavior?: ClosingBehavior;
  source?: string;
}

export interface LintLocalizations extends Partial<Record<LintCode, string>> {}
export interface FormatLocalizations extends Partial<Record<FormatRule, string>> {}

export class ParsedUsfm {
  private constructor();
  tokens(): Token[];
  cst(): CstDocument;
  lint(options?: LintOptions): LintResult;
  applyTokenFixes(lintOptions?: LintOptions, formatOptions?: FormatOptions): ApplyTokenFixesResult;
  revertDiffBlock(current: ParsedUsfm, blockId: string, options?: BuildSidBlocksOptions): Token[];
  format(options?: FormatOptions): string;
  toUsfm(): string;
  toUsj(): UsjDocument;
  toUsx(): string;
  toHtml(options?: HtmlOptions): string;
  toVref(): VrefMap;
  diff(other: ParsedUsfm, options?: BuildSidBlocksOptions): ChapterTokenDiff[];
  diffByChapter(other: ParsedUsfm, options?: BuildSidBlocksOptions): DiffsByChapterMap;
}

export class ParsedUsfmBatch {
  private constructor();
  items(): ParsedUsfm[];
  tokens(): Token[][];
  lint(options?: LintOptions): LintResult[];
  format(options?: FormatOptions): string[];
  toUsfm(): string[];
  toUsj(): UsjDocument[];
  toUsx(): string[];
  toHtml(options?: HtmlOptions): string[];
  toVref(): VrefMap[];
}

export class UsfmMarkerCatalog {
  private constructor();
  all(): MarkerInfo[];
  get(marker: string): MarkerInfo | undefined;
  contains(marker: string): boolean;
}

export function parse(source: string): ParsedUsfm;
export function parseBatch(sources: string[]): ParsedUsfmBatch;
export function lintUsfm(source: string, options?: LintOptions): LintResult;
export function lintTokens(tokens: Token[], options?: LintOptions): LintResult;
export function applyTokenFixes(tokens: Token[], lintOptions?: LintOptions, formatOptions?: FormatOptions): ApplyTokenFixesResult;
export function lintTokenBatch(tokenBatches: Token[][], options?: LintOptions): LintResult[];
export function formatUsfm(source: string, options?: FormatOptions): string;
export function formatTokens(tokens: FormatToken[], options?: FormatOptions): FormatResult;
export function formatTokensMut(tokens: FormatToken[], options?: FormatOptions): FormatToken[];
export function formatTokenBatch(tokenBatches: FormatToken[][], options?: FormatOptions): FormatResult[];
export function tokensToUsfm(tokens: Token[]): string;
export function tokensToHtml(tokens: Token[], options?: HtmlOptions): string;
export function diffUsfm(left: string, right: string, options?: BuildSidBlocksOptions): ChapterTokenDiff[];
export function diffUsfmByChapter(left: string, right: string, options?: BuildSidBlocksOptions): DiffsByChapterMap;
export function diffTokens(left: Token[], right: Token[], options?: BuildSidBlocksOptions): ChapterTokenDiff[];
export function revertDiffBlock(baseline: Token[], current: Token[], blockId: string, options?: BuildSidBlocksOptions): Token[];
export function revertDiffBlocks(baseline: Token[], current: Token[], blockIds: string[], options?: BuildSidBlocksOptions): Token[];
export function markerCatalog(): UsfmMarkerCatalog;
export function markerInfo(marker: string): MarkerInfo;
export function isKnownMarker(marker: string): boolean;
export function lintCodes(): LintCode[];
export function lintCodeMeta(): LintCodeMeta[];
export function formatRules(): FormatRule[];
export function formatRuleMeta(): FormatRuleMeta[];
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpanValue {
    start: u32,
    end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct MarkerMetadataValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttributeItemValue {
    span: SpanValue,
    text: String,
    key: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StructuralMarkerInfoValue {
    scope_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NumberInfoValue {
    start: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end: Option<u32>,
    kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenValue {
    id: String,
    kind: String,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<SpanValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nested: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker_metadata: Option<MarkerMetadataValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    structural: Option<StructuralMarkerInfoValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    number_info: Option<NumberInfoValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    book_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    book_code_valid: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attributes: Vec<AttributeItemValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CstNodeValue {
    token_index: usize,
    children: Vec<CstNodeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CstDocumentValue {
    tokens: Vec<TokenValue>,
    roots: Vec<CstNodeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintSuppressionValue {
    code: String,
    sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LintOptionsValue {
    #[serde(default)]
    enabled_codes: Option<Vec<String>>,
    #[serde(default)]
    disabled_codes: Vec<String>,
    #[serde(default)]
    suppressed: Vec<LintSuppressionValue>,
    #[serde(default)]
    allow_implicit_chapter_content_verse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintIssueValue {
    code: String,
    category: String,
    severity: String,
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<SpanValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    related_span: Option<SpanValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    related_token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintSummaryValue {
    by_category: std::collections::BTreeMap<String, usize>,
    by_severity: std::collections::BTreeMap<String, usize>,
    total_count: usize,
    suppressed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintResultValue {
    issues: Vec<LintIssueValue>,
    summary: LintSummaryValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppliedTokenFixValue {
    code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyTokenFixesResultValue {
    tokens: Vec<TokenValue>,
    usfm: String,
    applied_fixes: Vec<AppliedTokenFixValue>,
    remaining_issues: Vec<LintIssueValue>,
    remaining_summary: LintSummaryValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct FormatOptionsValue {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FormatResultValue {
    tokens: Vec<TokenValue>,
    usfm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct HtmlOptionsValue {
    #[serde(default)]
    wrap_root: bool,
    #[serde(default)]
    prefer_native_elements: Option<bool>,
    #[serde(default)]
    note_mode: Option<String>,
    #[serde(default)]
    caller_style: Option<String>,
    #[serde(default)]
    caller_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BuildSidBlocksOptionsValue {
    #[serde(default)]
    allow_empty_sid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidBlockValue {
    block_id: String,
    semantic_sid: String,
    start: usize,
    end_exclusive: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prev_block_id: Option<String>,
    text_full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenAlignmentValue {
    change: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    counterpart_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChapterTokenDiffValue {
    block_id: String,
    semantic_sid: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original: Option<SidBlockValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current: Option<SidBlockValue>,
    original_text: String,
    current_text: String,
    original_text_only: String,
    current_text_only: String,
    is_whitespace_change: bool,
    is_usfm_structure_change: bool,
    original_tokens: Vec<TokenValue>,
    current_tokens: Vec<TokenValue>,
    original_alignment: Vec<TokenAlignmentValue>,
    current_alignment: Vec<TokenAlignmentValue>,
    undo_side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintCodeMetaValue {
    code: String,
    category: String,
    severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FormatRuleMetaValue {
    code: String,
    label_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarkerInfoValue {
    marker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical: Option<String>,
    known: bool,
    deprecated: bool,
    category: String,
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_subkind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_attribute: Option<String>,
    contexts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    block_behavior: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    closing_behavior: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

#[derive(Debug, Clone)]
struct AdapterToken {
    id: String,
    kind: NativeTokenKind,
    text: String,
    span: Option<NativeSpan>,
    sid: Option<String>,
    marker: Option<String>,
    structural: Option<StructuralMarkerInfo>,
    number_info: Option<(u32, Option<u32>, NativeNumberRangeKind)>,
}

impl LintableToken for AdapterToken {
    fn kind(&self) -> NativeTokenKind {
        self.kind
    }

    fn span(&self) -> Option<NativeSpan> {
        self.span
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn sid(&self) -> Option<String> {
        self.sid.clone()
    }

    fn id(&self) -> Option<String> {
        Some(self.id.clone())
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural
    }

    fn number_info(&self) -> Option<(u32, Option<u32>, NativeNumberRangeKind)> {
        self.number_info
    }
}

impl DiffableToken for AdapterToken {
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
        Some(token_kind_key(self.kind))
    }

    fn marker_key(&self) -> Option<&str> {
        self.marker.as_deref()
    }
}

#[wasm_bindgen(skip_typescript)]
pub struct ParsedUsfm {
    source: String,
}

#[wasm_bindgen(skip_typescript)]
pub struct ParsedUsfmBatch {
    sources: Vec<String>,
}

#[wasm_bindgen(skip_typescript)]
pub struct UsfmMarkerCatalog;

#[wasm_bindgen]
impl ParsedUsfm {
    fn new(source: String) -> Self {
        Self { source }
    }

    pub fn tokens(&self) -> Result<JsValue, JsError> {
        let parsed = native_parse(&self.source);
        to_js_value(&map_tokens(&parsed.tokens))
    }

    pub fn cst(&self) -> Result<JsValue, JsError> {
        let cst = parse_cst(&self.source);
        to_js_value(&map_cst_document(&cst))
    }

    pub fn lint(&self, options: Option<JsValue>) -> Result<JsValue, JsError> {
        let options = parse_lint_options(options)?;
        to_js_value(&map_lint_result(lint_usfm(&self.source, options)))
    }

    #[wasm_bindgen(js_name = applyTokenFixes)]
    pub fn apply_token_fixes(
        &self,
        lint_options: Option<JsValue>,
        format_options: Option<JsValue>,
    ) -> Result<JsValue, JsError> {
        let parsed = native_parse(&self.source);
        let native_tokens = parsed
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let result = apply_token_fixes(
            &native_tokens,
            parse_lint_options(lint_options)?,
            parse_format_options(format_options)?,
        );
        to_js_value(&map_apply_token_fixes_result(result))
    }

    #[wasm_bindgen(js_name = revertDiffBlock)]
    pub fn revert_diff_block(
        &self,
        current: &ParsedUsfm,
        block_id: &str,
        options: Option<JsValue>,
    ) -> Result<JsValue, JsError> {
        let baseline = native_parse(&self.source);
        let current = native_parse(&current.source);
        let baseline = baseline.tokens.iter().map(format_token_with_identity).collect::<Vec<_>>();
        let current = current.tokens.iter().map(format_token_with_identity).collect::<Vec<_>>();
        let reverted = apply_revert_by_block_id(
            block_id,
            &baseline,
            &current,
            &parse_build_options(options)?,
        );
        to_js_value(&reverted.iter().map(map_format_token).collect::<Vec<_>>())
    }

    pub fn format(&self, options: Option<JsValue>) -> Result<String, JsError> {
        let options = parse_format_options(options)?;
        Ok(format_usfm(&self.source, options))
    }

    pub fn to_usfm(&self) -> String {
        let parsed = native_parse(&self.source);
        tokens_to_usfm(&parsed.tokens)
    }

    pub fn to_usj(&self) -> Result<JsValue, JsError> {
        let document = usfm_to_usj(&self.source).map_err(js_error)?;
        to_js_value(&document)
    }

    pub fn to_usx(&self) -> Result<String, JsError> {
        usfm_to_usx(&self.source).map_err(js_error)
    }

    pub fn to_html(&self, options: Option<JsValue>) -> Result<String, JsError> {
        let options = parse_html_options(options)?;
        Ok(usfm_to_html(&self.source, options))
    }

    pub fn to_vref(&self) -> Result<JsValue, JsError> {
        to_js_value(&vref_to_object(usfm_to_vref_map(&self.source)))
    }

    pub fn diff(&self, other: &ParsedUsfm, options: Option<JsValue>) -> Result<JsValue, JsError> {
        let options = parse_build_options(options)?;
        let diffs = diff_usfm_sources(&self.source, &other.source, &options);
        to_js_value(&map_chapter_diffs(&diffs))
    }

    #[wasm_bindgen(js_name = diffByChapter)]
    pub fn diff_by_chapter(
        &self,
        other: &ParsedUsfm,
        options: Option<JsValue>,
    ) -> Result<JsValue, JsError> {
        let options = parse_build_options(options)?;
        let diffs = diff_usfm_sources_by_chapter(&self.source, &other.source, &options);
        to_js_value(&map_diffs_by_chapter(&diffs))
    }
}

#[wasm_bindgen]
impl ParsedUsfmBatch {
    fn new(sources: Vec<String>) -> Self {
        Self { sources }
    }

    pub fn items(&self) -> Array {
        self.sources
            .iter()
            .cloned()
            .map(ParsedUsfm::new)
            .map(JsValue::from)
            .collect()
    }

    pub fn tokens(&self) -> Result<JsValue, JsError> {
        let batches = self
            .sources
            .iter()
            .map(|source| map_tokens(&native_parse(source).tokens))
            .collect::<Vec<_>>();
        to_js_value(&batches)
    }

    pub fn lint(&self, options: Option<JsValue>) -> Result<JsValue, JsError> {
        let options = parse_lint_options(options)?;
        let results = self
            .sources
            .iter()
            .map(|source| map_lint_result(lint_usfm(source, options.clone())))
            .collect::<Vec<_>>();
        to_js_value(&results)
    }

    pub fn format(&self, options: Option<JsValue>) -> Result<JsValue, JsError> {
        let options = parse_format_options(options)?;
        let values = self
            .sources
            .iter()
            .map(|source| format_usfm(source, options))
            .collect::<Vec<_>>();
        to_js_value(&values)
    }

    pub fn to_usfm(&self) -> Result<JsValue, JsError> {
        let values = self
            .sources
            .iter()
            .map(|source| {
                let parsed = native_parse(source);
                tokens_to_usfm(&parsed.tokens)
            })
            .collect::<Vec<_>>();
        to_js_value(&values)
    }

    pub fn to_usj(&self) -> Result<JsValue, JsError> {
        let values = self
            .sources
            .iter()
            .map(|source| usfm_to_usj(source).map_err(js_error))
            .collect::<Result<Vec<UsjDocument>, JsError>>()?;
        to_js_value(&values)
    }

    pub fn to_usx(&self) -> Result<JsValue, JsError> {
        let values = self
            .sources
            .iter()
            .map(|source| usfm_to_usx(source).map_err(js_error))
            .collect::<Result<Vec<String>, JsError>>()?;
        to_js_value(&values)
    }

    pub fn to_html(&self, options: Option<JsValue>) -> Result<JsValue, JsError> {
        let options = parse_html_options(options)?;
        let values = self
            .sources
            .iter()
            .map(|source| usfm_to_html(source, options))
            .collect::<Vec<_>>();
        to_js_value(&values)
    }

    pub fn to_vref(&self) -> Result<JsValue, JsError> {
        let values = self
            .sources
            .iter()
            .map(|source| vref_to_object(usfm_to_vref_map(source)))
            .collect::<Vec<_>>();
        to_js_value(&values)
    }
}

#[wasm_bindgen]
impl UsfmMarkerCatalog {
    fn new() -> Self {
        Self
    }

    pub fn all(&self) -> Result<JsValue, JsError> {
        let entries = marker_catalog()
            .all()
            .iter()
            .cloned()
            .map(map_marker_info)
            .collect::<Vec<_>>();
        to_js_value(&entries)
    }

    pub fn get(&self, marker: &str) -> Result<JsValue, JsError> {
        let value = marker_catalog().get(marker).cloned().map(map_marker_info);
        to_js_value(&value)
    }

    pub fn contains(&self, marker: &str) -> bool {
        marker_catalog().contains(marker)
    }
}

#[wasm_bindgen(skip_typescript, js_name = parse)]
pub fn wasm_parse(source: &str) -> ParsedUsfm {
    ParsedUsfm::new(source.to_string())
}

#[wasm_bindgen(skip_typescript, js_name = parseBatch)]
pub fn wasm_parse_batch(sources: JsValue) -> Result<ParsedUsfmBatch, JsError> {
    let sources = from_js_or_default::<Vec<String>>(sources)?;
    Ok(ParsedUsfmBatch::new(sources))
}

#[wasm_bindgen(skip_typescript, js_name = lintUsfm)]
pub fn wasm_lint_usfm(source: &str, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let options = parse_lint_options(options)?;
    to_js_value(&map_lint_result(lint_usfm(source, options)))
}

#[wasm_bindgen(skip_typescript, js_name = lintTokens)]
pub fn wasm_lint_tokens(tokens: JsValue, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let tokens = parse_adapter_tokens(tokens)?;
    let options = parse_lint_options(options)?;
    to_js_value(&map_lint_result(lint_tokens(&tokens, options)))
}

#[wasm_bindgen(skip_typescript, js_name = applyTokenFixes)]
pub fn wasm_apply_token_fixes(
    tokens: JsValue,
    lint_options: Option<JsValue>,
    format_options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let native_tokens = values
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    let result = apply_token_fixes(
        &native_tokens,
        parse_lint_options(lint_options)?,
        parse_format_options(format_options)?,
    );
    to_js_value(&map_apply_token_fixes_result(result))
}

#[wasm_bindgen(skip_typescript, js_name = lintTokenBatch)]
pub fn wasm_lint_token_batch(
    token_batches: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let token_batches = from_js_or_default::<Vec<Vec<TokenValue>>>(token_batches)?;
    let options = parse_lint_options(options)?;
    let results = token_batches
        .into_iter()
        .map(parse_adapter_tokens_from_values)
        .collect::<Result<Vec<_>, JsError>>()?
        .into_iter()
        .map(|tokens| map_lint_result(lint_tokens(&tokens, options.clone())))
        .collect::<Vec<_>>();
    to_js_value(&results)
}

#[wasm_bindgen(skip_typescript, js_name = formatUsfm)]
pub fn wasm_format_usfm(source: &str, options: Option<JsValue>) -> Result<String, JsError> {
    Ok(format_usfm(source, parse_format_options(options)?))
}

#[wasm_bindgen(skip_typescript, js_name = formatTokens)]
pub fn wasm_format_tokens(tokens: JsValue, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let mut native_tokens = values
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    native_format_tokens(&mut native_tokens, parse_format_options(options)?);
    let formatted = FormatResultValue {
        tokens: native_tokens.iter().map(map_format_token).collect(),
        usfm: format_tokens_to_usfm(&native_tokens),
    };
    to_js_value(&formatted)
}

#[wasm_bindgen(skip_typescript, js_name = formatTokensMut)]
pub fn wasm_format_tokens_mut(tokens: JsValue, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let mut native_tokens = values
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    native_format_tokens(&mut native_tokens, parse_format_options(options)?);
    let formatted = native_tokens.iter().map(map_format_token).collect::<Vec<_>>();
    to_js_value(&formatted)
}

#[wasm_bindgen(skip_typescript, js_name = formatTokenBatch)]
pub fn wasm_format_token_batch(
    token_batches: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let batches = from_js_or_default::<Vec<Vec<TokenValue>>>(token_batches)?;
    let options = parse_format_options(options)?;
    let results = batches
        .into_iter()
        .map(|batch| {
            let mut native_tokens = batch
                .into_iter()
                .map(token_value_to_format_token)
                .collect::<Result<Vec<_>, JsError>>()?;
            native_format_tokens(&mut native_tokens, options);
            Ok(FormatResultValue {
                tokens: native_tokens.iter().map(map_format_token).collect(),
                usfm: format_tokens_to_usfm(&native_tokens),
            })
        })
        .collect::<Result<Vec<_>, JsError>>()?;
    to_js_value(&results)
}

#[wasm_bindgen(skip_typescript, js_name = tokensToUsfm)]
pub fn wasm_tokens_to_usfm(tokens: JsValue) -> Result<String, JsError> {
    let tokens: Vec<TokenValue> = from_js_or_default(tokens)?;
    Ok(token_values_to_usfm(&tokens))
}

#[wasm_bindgen(skip_typescript, js_name = tokensToHtml)]
pub fn wasm_tokens_to_html(tokens: JsValue, options: Option<JsValue>) -> Result<String, JsError> {
    let tokens = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let usfm = token_values_to_usfm(&tokens);
    Ok(usfm_to_html(&usfm, parse_html_options(options)?))
}

#[wasm_bindgen(skip_typescript, js_name = diffUsfm)]
pub fn wasm_diff_usfm(left: &str, right: &str, options: Option<JsValue>) -> Result<JsValue, JsError> {
    let options = parse_build_options(options)?;
    let diffs = diff_usfm_sources(left, right, &options);
    to_js_value(&map_chapter_diffs(&diffs))
}

#[wasm_bindgen(skip_typescript, js_name = diffUsfmByChapter)]
pub fn wasm_diff_usfm_by_chapter(
    left: &str,
    right: &str,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let options = parse_build_options(options)?;
    let diffs = diff_usfm_sources_by_chapter(left, right, &options);
    to_js_value(&map_diffs_by_chapter(&diffs))
}

#[wasm_bindgen(skip_typescript, js_name = diffTokens)]
pub fn wasm_diff_tokens(
    left: JsValue,
    right: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let left = parse_adapter_tokens(left)?;
    let right = parse_adapter_tokens(right)?;
    let options = parse_build_options(options)?;
    let diffs = diff_chapter_token_streams(&left, &right, &options);
    to_js_value(&map_adapter_diffs(&diffs))
}

#[wasm_bindgen(skip_typescript, js_name = revertDiffBlock)]
pub fn wasm_revert_diff_block(
    baseline: JsValue,
    current: JsValue,
    block_id: &str,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let baseline = from_js_or_default::<Vec<TokenValue>>(baseline)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    let current = from_js_or_default::<Vec<TokenValue>>(current)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    let reverted = apply_revert_by_block_id(
        block_id,
        &baseline,
        &current,
        &parse_build_options(options)?,
    );
    to_js_value(&reverted.iter().map(map_format_token).collect::<Vec<_>>())
}

#[wasm_bindgen(skip_typescript, js_name = revertDiffBlocks)]
pub fn wasm_revert_diff_blocks(
    baseline: JsValue,
    current: JsValue,
    block_ids: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let baseline = from_js_or_default::<Vec<TokenValue>>(baseline)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    let current = from_js_or_default::<Vec<TokenValue>>(current)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Result<Vec<_>, JsError>>()?;
    let block_ids = from_js_or_default::<Vec<String>>(block_ids)?;
    let reverted = apply_reverts_by_block_id(
        &block_ids,
        &baseline,
        &current,
        &parse_build_options(options)?,
    );
    to_js_value(&reverted.iter().map(map_format_token).collect::<Vec<_>>())
}

#[wasm_bindgen(skip_typescript, js_name = markerCatalog)]
pub fn wasm_marker_catalog() -> UsfmMarkerCatalog {
    UsfmMarkerCatalog::new()
}

#[wasm_bindgen(skip_typescript, js_name = markerInfo)]
pub fn wasm_marker_info(marker: &str) -> Result<JsValue, JsError> {
    to_js_value(&map_marker_info(marker_info(marker)))
}

#[wasm_bindgen(skip_typescript, js_name = isKnownMarker)]
pub fn wasm_is_known_marker(marker: &str) -> bool {
    is_known_marker(marker)
}

#[wasm_bindgen(skip_typescript, js_name = lintCodes)]
pub fn wasm_lint_codes() -> Result<JsValue, JsError> {
    let codes = lint_code_variants()
        .into_iter()
        .map(lint_code_str)
        .collect::<Vec<_>>();
    to_js_value(&codes)
}

#[wasm_bindgen(skip_typescript, js_name = lintCodeMeta)]
pub fn wasm_lint_code_meta() -> Result<JsValue, JsError> {
    let meta = lint_code_variants()
        .into_iter()
        .map(|code| LintCodeMetaValue {
            code: lint_code_str(code).to_string(),
            category: lint_category_str(code.category()).to_string(),
            severity: lint_severity_str(code.severity()).to_string(),
        })
        .collect::<Vec<_>>();
    to_js_value(&meta)
}

#[wasm_bindgen(skip_typescript, js_name = formatRules)]
pub fn wasm_format_rules() -> Result<JsValue, JsError> {
    let rules = NativeFormatRule::ALL
        .iter()
        .map(|rule| rule.code().to_string())
        .collect::<Vec<_>>();
    to_js_value(&rules)
}

#[wasm_bindgen(skip_typescript, js_name = formatRuleMeta)]
pub fn wasm_format_rule_meta() -> Result<JsValue, JsError> {
    let meta = NativeFormatRule::ALL
        .iter()
        .map(|rule| FormatRuleMetaValue {
            code: rule.code().to_string(),
            label_key: rule.label_key().to_string(),
        })
        .collect::<Vec<_>>();
    to_js_value(&meta)
}

fn parse_lint_options(value: Option<JsValue>) -> Result<NativeLintOptions, JsError> {
    let value = value.unwrap_or(JsValue::UNDEFINED);
    let value: LintOptionsValue = from_js_or_default(value)?;
    Ok(NativeLintOptions {
        enabled_codes: value
            .enabled_codes
            .map(|codes| codes.into_iter().map(parse_lint_code).collect::<Result<Vec<_>, _>>())
            .transpose()?,
        disabled_codes: value
            .disabled_codes
            .into_iter()
            .map(parse_lint_code)
            .collect::<Result<Vec<_>, _>>()?,
        suppressed: value
            .suppressed
            .into_iter()
            .map(|suppression| {
                Ok(NativeLintSuppression {
                    code: parse_lint_code(suppression.code)?,
                    sid: suppression.sid,
                })
            })
            .collect::<Result<Vec<_>, JsError>>()?,
        allow_implicit_chapter_content_verse: value.allow_implicit_chapter_content_verse,
    })
}

fn parse_format_options(value: Option<JsValue>) -> Result<NativeFormatOptions, JsError> {
    let value = value.unwrap_or(JsValue::UNDEFINED);
    let value: FormatOptionsValue = from_js_or_default(value)?;
    let mut options = NativeFormatOptions::default();
    apply_opt(&mut options.recover_malformed_markers, value.recover_malformed_markers);
    apply_opt(
        &mut options.collapse_whitespace_in_text,
        value.collapse_whitespace_in_text,
    );
    apply_opt(&mut options.ensure_inline_separators, value.ensure_inline_separators);
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
    apply_opt(&mut options.remove_empty_paragraphs, value.remove_empty_paragraphs);
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
    Ok(options)
}

fn parse_html_options(value: Option<JsValue>) -> Result<NativeHtmlOptions, JsError> {
    let value = value.unwrap_or(JsValue::UNDEFINED);
    let value: HtmlOptionsValue = from_js_or_default(value)?;
    Ok(NativeHtmlOptions {
        wrap_root: value.wrap_root,
        prefer_native_elements: value.prefer_native_elements.unwrap_or(true),
        note_mode: value
            .note_mode
            .map(|mode| parse_html_note_mode(mode.as_str()))
            .transpose()
            .map_err(js_error)?
            .unwrap_or(NativeHtmlNoteMode::Extracted),
        caller_style: value
            .caller_style
            .map(|style| parse_html_caller_style(style.as_str()))
            .transpose()
            .map_err(js_error)?
            .unwrap_or(NativeHtmlCallerStyle::Numeric),
        caller_scope: value
            .caller_scope
            .map(|scope| parse_html_caller_scope(scope.as_str()))
            .transpose()
            .map_err(js_error)?
            .unwrap_or(NativeHtmlCallerScope::VerseSequential),
    })
}

fn parse_build_options(value: Option<JsValue>) -> Result<NativeBuildSidBlocksOptions, JsError> {
    let value = value.unwrap_or(JsValue::UNDEFINED);
    let value: BuildSidBlocksOptionsValue = from_js_or_default(value)?;
    Ok(NativeBuildSidBlocksOptions {
        allow_empty_sid: value.allow_empty_sid.unwrap_or(true),
    })
}

fn parse_adapter_tokens(value: JsValue) -> Result<Vec<AdapterToken>, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(value)?;
    parse_adapter_tokens_from_values(values)
}

fn parse_adapter_tokens_from_values(values: Vec<TokenValue>) -> Result<Vec<AdapterToken>, JsError> {
    values.into_iter().map(token_value_to_adapter).collect()
}

fn token_value_to_adapter(value: TokenValue) -> Result<AdapterToken, JsError> {
    Ok(AdapterToken {
        id: value.id,
        kind: parse_token_kind(value.kind.as_str())?,
        text: value.text,
        span: value.span.map(native_span),
        sid: value.sid,
        marker: value.marker,
        structural: value.structural.map(parse_structural_info).transpose()?,
        number_info: value.number_info.map(parse_number_info).transpose()?,
    })
}

fn token_value_to_format_token(value: TokenValue) -> Result<NativeFormatToken, JsError> {
    let kind = parse_token_kind(value.kind.as_str())?;
    Ok(NativeFormatToken {
        kind,
        text: value.text,
        marker: value.marker,
        sid: value.sid,
        id: Some(value.id),
        span: value.span.map(native_span),
        structural: value.structural.map(parse_structural_info).transpose()?,
        number_info: value.number_info.map(parse_number_info).transpose()?,
        marker_profile: None,
    })
}

fn format_token_with_identity(token: &NativeToken<'_>) -> NativeFormatToken {
    let mut owned = NativeFormatToken::from(token);
    owned.sid = token.sid.map(|sid| format_sid(sid.book_code, sid.chapter, sid.verse));
    owned.id = Some(format!("{}-{}", token.id.book_code, token.id.index));
    owned
}

fn parse_structural_info(value: StructuralMarkerInfoValue) -> Result<StructuralMarkerInfo, JsError> {
    Ok(StructuralMarkerInfo {
        scope_kind: parse_scope_kind(value.scope_kind.as_str())?,
        inline_context: value
            .inline_context
            .map(|context| parse_inline_context(context.as_str()))
            .transpose()
            .map_err(js_error)?,
        note_context: value
            .note_context
            .map(|context| parse_spec_context(context.as_str()))
            .transpose()
            .map_err(js_error)?,
    })
}

fn parse_number_info(value: NumberInfoValue) -> Result<(u32, Option<u32>, NativeNumberRangeKind), JsError> {
    Ok((value.start, value.end, parse_number_kind(value.kind.as_str())?))
}

fn map_tokens(tokens: &[NativeToken<'_>]) -> Vec<TokenValue> {
    tokens.iter().map(map_token).collect()
}

fn map_token(token: &NativeToken<'_>) -> TokenValue {
    let mut value = TokenValue {
        id: format!("{}-{}", token.id.book_code, token.id.index),
        kind: token_kind_str(token.kind()).to_string(),
        text: token.source.to_string(),
        span: Some(map_span(token.span)),
        sid: token.sid.map(|sid| format_sid(sid.book_code, sid.chapter, sid.verse)),
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
            ..
        }
        | NativeTokenData::EndMarker {
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
            metadata, structural, ..
        } => {
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
        }
        NativeTokenData::BookCode { code, is_valid } => {
            value.book_code = Some((*code).to_string());
            value.book_code_valid = Some(*is_valid);
        }
        NativeTokenData::Number { start, end, kind } => {
            value.number_info = Some(NumberInfoValue {
                start: *start,
                end: *end,
                kind: number_kind_str(*kind).to_string(),
            });
        }
        NativeTokenData::AttributeList { entries } => {
            value.attributes = entries.iter().map(map_attribute_item).collect();
        }
        NativeTokenData::Newline
        | NativeTokenData::OptBreak
        | NativeTokenData::MilestoneEnd
        | NativeTokenData::Text => {}
    }

    value
}

fn map_format_token(token: &NativeFormatToken) -> TokenValue {
    TokenValue {
        id: token.id.clone().unwrap_or_default(),
        kind: token_kind_str(token.kind).to_string(),
        text: token.text.clone(),
        span: token.span.map(map_span),
        sid: token.sid.clone(),
        marker: token.marker.clone(),
        nested: None,
        marker_metadata: None,
        structural: token.structural.map(map_structural_info),
        number_info: token.number_info.map(|(start, end, kind)| NumberInfoValue {
            start,
            end,
            kind: number_kind_str(kind).to_string(),
        }),
        book_code: None,
        book_code_valid: None,
        attributes: Vec::new(),
    }
}

fn map_attribute_item(item: &NativeAttributeItem<'_>) -> AttributeItemValue {
    AttributeItemValue {
        span: map_span(item.span),
        text: item.source.to_string(),
        key: item.key.to_string(),
        value: item.value.to_string(),
    }
}

fn map_marker_metadata(metadata: NativeMarkerMetadata) -> MarkerMetadataValue {
    MarkerMetadataValue {
        canonical: metadata.canonical.map(ToOwned::to_owned),
        kind: metadata.kind.map(spec_marker_kind_str).map(ToOwned::to_owned),
        family: metadata.family.map(marker_family_str).map(ToOwned::to_owned),
    }
}

fn map_structural_info(info: StructuralMarkerInfo) -> StructuralMarkerInfoValue {
    StructuralMarkerInfoValue {
        scope_kind: scope_kind_str(info.scope_kind).to_string(),
        inline_context: info.inline_context.map(inline_context_str).map(ToOwned::to_owned),
        note_context: info.note_context.map(spec_context_str).map(ToOwned::to_owned),
    }
}

fn map_span(span: NativeSpan) -> SpanValue {
    SpanValue {
        start: span.start,
        end: span.end,
    }
}

fn native_span(span: SpanValue) -> NativeSpan {
    NativeSpan {
        start: span.start,
        end: span.end,
    }
}

fn map_cst_document(document: &NativeCstDocument<'_>) -> CstDocumentValue {
    CstDocumentValue {
        tokens: map_tokens(&document.tokens),
        roots: document.roots.iter().map(map_cst_node).collect(),
    }
}

fn map_cst_node(node: &NativeCstNode) -> CstNodeValue {
    CstNodeValue {
        token_index: node.token_index,
        children: node.children.iter().map(map_cst_node).collect(),
    }
}

fn map_lint_result(result: NativeLintResult) -> LintResultValue {
    LintResultValue {
        issues: result.issues.into_iter().map(map_lint_issue).collect(),
        summary: map_lint_summary(result.summary),
    }
}

fn map_lint_summary(summary: usfm_onion::LintSummary) -> LintSummaryValue {
    LintSummaryValue {
        by_category: summary
            .by_category
            .into_iter()
            .map(|(category, count)| (lint_category_str(category).to_string(), count))
            .collect(),
        by_severity: summary
            .by_severity
            .into_iter()
            .map(|(severity, count)| (lint_severity_str(severity).to_string(), count))
            .collect(),
        total_count: summary.total_count,
        suppressed_count: summary.suppressed_count,
    }
}

fn map_applied_token_fix(fix: NativeAppliedTokenFix) -> AppliedTokenFixValue {
    AppliedTokenFixValue {
        code: lint_code_str(fix.code).to_string(),
        token_id: fix.token_id,
        sid: fix.sid,
        marker: fix.marker,
    }
}

fn map_apply_token_fixes_result(
    result: NativeApplyTokenFixesResult<NativeFormatToken>,
) -> ApplyTokenFixesResultValue {
    ApplyTokenFixesResultValue {
        tokens: result.tokens.iter().map(map_format_token).collect(),
        usfm: format_tokens_to_usfm(&result.tokens),
        applied_fixes: result
            .applied_fixes
            .into_iter()
            .map(map_applied_token_fix)
            .collect(),
        remaining_issues: result
            .remaining_issues
            .into_iter()
            .map(map_lint_issue)
            .collect(),
        remaining_summary: map_lint_summary(result.remaining_summary),
    }
}

fn map_lint_issue(issue: usfm_onion::LintIssue) -> LintIssueValue {
    LintIssueValue {
        code: lint_code_str(issue.code).to_string(),
        category: lint_category_str(issue.category).to_string(),
        severity: lint_severity_str(issue.severity).to_string(),
        message: issue.message,
        span: issue.span.map(map_span),
        related_span: issue.related_span.map(map_span),
        token_id: issue.token_id,
        related_token_id: issue.related_token_id,
        sid: issue.sid,
        marker: issue.marker,
    }
}

fn map_chapter_diffs(diffs: &[NativeChapterTokenDiff<NativeToken<'_>>]) -> Vec<ChapterTokenDiffValue> {
    diffs.iter().map(map_native_chapter_diff).collect()
}

fn map_native_chapter_diff(diff: &NativeChapterTokenDiff<NativeToken<'_>>) -> ChapterTokenDiffValue {
    ChapterTokenDiffValue {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff_status_str(diff.status).to_string(),
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
        undo_side: diff_undo_side_str(diff.undo_side).to_string(),
    }
}

fn map_adapter_diffs(diffs: &[NativeChapterTokenDiff<AdapterToken>]) -> Vec<ChapterTokenDiffValue> {
    diffs.iter().map(map_adapter_chapter_diff).collect()
}

fn map_adapter_chapter_diff(diff: &NativeChapterTokenDiff<AdapterToken>) -> ChapterTokenDiffValue {
    ChapterTokenDiffValue {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff_status_str(diff.status).to_string(),
        original: diff.original.as_ref().map(map_sid_block),
        current: diff.current.as_ref().map(map_sid_block),
        original_text: diff.original_text.clone(),
        current_text: diff.current_text.clone(),
        original_text_only: diff.original_text_only.clone(),
        current_text_only: diff.current_text_only.clone(),
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens: diff.original_tokens.iter().map(map_adapter_token).collect(),
        current_tokens: diff.current_tokens.iter().map(map_adapter_token).collect(),
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
        undo_side: diff_undo_side_str(diff.undo_side).to_string(),
    }
}

fn map_adapter_token(token: &AdapterToken) -> TokenValue {
    TokenValue {
        id: token.id.clone(),
        kind: token_kind_str(token.kind).to_string(),
        text: token.text.clone(),
        span: token.span.map(map_span),
        sid: token.sid.clone(),
        marker: token.marker.clone(),
        nested: None,
        marker_metadata: None,
        structural: token.structural.map(map_structural_info),
        number_info: token.number_info.map(|(start, end, kind)| NumberInfoValue {
            start,
            end,
            kind: number_kind_str(kind).to_string(),
        }),
        book_code: None,
        book_code_valid: None,
        attributes: Vec::new(),
    }
}

fn map_sid_block(block: &NativeSidBlock) -> SidBlockValue {
    SidBlockValue {
        block_id: block.block_id.clone(),
        semantic_sid: block.semantic_sid.clone(),
        start: block.start,
        end_exclusive: block.end_exclusive,
        prev_block_id: block.prev_block_id.clone(),
        text_full: block.text_full.clone(),
    }
}

fn map_alignment(alignment: NativeTokenAlignment) -> TokenAlignmentValue {
    TokenAlignmentValue {
        change: diff_token_change_str(alignment.change).to_string(),
        counterpart_index: alignment.counterpart_index,
    }
}

fn map_diffs_by_chapter(
    diffs: &NativeDiffsByChapterMap<NativeChapterTokenDiff<NativeToken<'_>>>,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<u32, Vec<ChapterTokenDiffValue>>> {
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

fn map_marker_info(info: NativeUsfmMarkerInfo) -> MarkerInfoValue {
    MarkerInfoValue {
        marker: info.marker,
        canonical: info.canonical,
        known: info.known,
        deprecated: info.deprecated,
        category: marker_category_str(info.category).to_string(),
        kind: marker_kind_str(info.kind).to_string(),
        family: info.family.map(marker_family_str).map(ToOwned::to_owned),
        family_role: info
            .family_role
            .map(marker_family_role_str)
            .map(ToOwned::to_owned),
        note_family: info
            .note_family
            .map(marker_note_family_str)
            .map(ToOwned::to_owned),
        note_subkind: info
            .note_subkind
            .map(marker_note_subkind_str)
            .map(ToOwned::to_owned),
        inline_context: info
            .inline_context
            .map(marker_inline_context_str)
            .map(ToOwned::to_owned),
        default_attribute: info.default_attribute,
        contexts: info
            .contexts
            .into_iter()
            .map(spec_context_str)
            .map(ToOwned::to_owned)
            .collect(),
        block_behavior: info
            .block_behavior
            .map(block_behavior_str)
            .map(ToOwned::to_owned),
        closing_behavior: info
            .closing_behavior
            .map(closing_behavior_str)
            .map(ToOwned::to_owned),
        source: info.source,
    }
}

fn token_values_to_usfm(tokens: &[TokenValue]) -> String {
    tokens.iter().map(|token| token.text.as_str()).collect()
}

fn vref_to_object(map: VrefMap) -> std::collections::BTreeMap<String, String> {
    map.into_iter().collect()
}

fn from_js_or_default<T>(value: JsValue) -> Result<T, JsError>
where
    T: DeserializeOwned + Default,
{
    if value.is_undefined() || value.is_null() {
        Ok(T::default())
    } else {
        from_js_value(value).map_err(js_serde_error)
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

fn parse_lint_code(value: String) -> Result<NativeLintCode, JsError> {
    lint_code_variants()
        .into_iter()
        .find(|code| lint_code_str(*code) == value)
        .ok_or_else(|| js_error(format!("unknown lint code '{value}'")))
}

fn lint_code_variants() -> Vec<NativeLintCode> {
    vec![
        NativeLintCode::MissingIdMarker,
        NativeLintCode::MissingSeparatorAfterMarker,
        NativeLintCode::EmptyParagraph,
        NativeLintCode::NumberRangeAfterChapterMarker,
        NativeLintCode::VerseRangeExpectedAfterVerseMarker,
        NativeLintCode::VerseContentNotEmpty,
        NativeLintCode::UnknownToken,
        NativeLintCode::CharNotClosed,
        NativeLintCode::NoteNotClosed,
        NativeLintCode::ParagraphBeforeFirstChapter,
        NativeLintCode::VerseBeforeFirstChapter,
        NativeLintCode::NoteSubmarkerOutsideNote,
        NativeLintCode::DuplicateIdMarker,
        NativeLintCode::IdMarkerNotAtFileStart,
        NativeLintCode::ChapterMetadataOutsideChapter,
        NativeLintCode::VerseMetadataOutsideVerse,
        NativeLintCode::MissingChapterNumber,
        NativeLintCode::MissingVerseNumber,
        NativeLintCode::MissingMilestoneSelfClose,
        NativeLintCode::ImplicitlyClosedMarker,
        NativeLintCode::StrayCloseMarker,
        NativeLintCode::MisnestedCloseMarker,
        NativeLintCode::UnclosedNote,
        NativeLintCode::UnclosedMarkerAtEof,
        NativeLintCode::DuplicateChapterNumber,
        NativeLintCode::ChapterExpectedIncreaseByOne,
        NativeLintCode::DuplicateVerseNumber,
        NativeLintCode::VerseExpectedIncreaseByOne,
        NativeLintCode::InvalidNumberRange,
        NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        NativeLintCode::VerseTextFollowsVerseRange,
        NativeLintCode::UnknownMarker,
        NativeLintCode::UnknownCloseMarker,
        NativeLintCode::InconsistentChapterLabel,
        NativeLintCode::MarkerNotValidInContext,
        NativeLintCode::VerseOutsideExplicitParagraph,
    ]
}

fn lint_code_str(code: NativeLintCode) -> &'static str {
    match code {
        NativeLintCode::MissingIdMarker => "missing-id-marker",
        NativeLintCode::MissingSeparatorAfterMarker => "missing-separator-after-marker",
        NativeLintCode::EmptyParagraph => "empty-paragraph",
        NativeLintCode::NumberRangeAfterChapterMarker => "number-range-after-chapter-marker",
        NativeLintCode::VerseRangeExpectedAfterVerseMarker => "verse-range-expected-after-verse-marker",
        NativeLintCode::VerseContentNotEmpty => "verse-content-not-empty",
        NativeLintCode::UnknownToken => "unknown-token",
        NativeLintCode::CharNotClosed => "char-not-closed",
        NativeLintCode::NoteNotClosed => "note-not-closed",
        NativeLintCode::ParagraphBeforeFirstChapter => "paragraph-before-first-chapter",
        NativeLintCode::VerseBeforeFirstChapter => "verse-before-first-chapter",
        NativeLintCode::NoteSubmarkerOutsideNote => "note-submarker-outside-note",
        NativeLintCode::DuplicateIdMarker => "duplicate-id-marker",
        NativeLintCode::IdMarkerNotAtFileStart => "id-marker-not-at-file-start",
        NativeLintCode::ChapterMetadataOutsideChapter => "chapter-metadata-outside-chapter",
        NativeLintCode::VerseMetadataOutsideVerse => "verse-metadata-outside-verse",
        NativeLintCode::MissingChapterNumber => "missing-chapter-number",
        NativeLintCode::MissingVerseNumber => "missing-verse-number",
        NativeLintCode::MissingMilestoneSelfClose => "missing-milestone-self-close",
        NativeLintCode::ImplicitlyClosedMarker => "implicitly-closed-marker",
        NativeLintCode::StrayCloseMarker => "stray-close-marker",
        NativeLintCode::MisnestedCloseMarker => "misnested-close-marker",
        NativeLintCode::UnclosedNote => "unclosed-note",
        NativeLintCode::UnclosedMarkerAtEof => "unclosed-marker-at-eof",
        NativeLintCode::DuplicateChapterNumber => "duplicate-chapter-number",
        NativeLintCode::ChapterExpectedIncreaseByOne => "chapter-expected-increase-by-one",
        NativeLintCode::DuplicateVerseNumber => "duplicate-verse-number",
        NativeLintCode::VerseExpectedIncreaseByOne => "verse-expected-increase-by-one",
        NativeLintCode::InvalidNumberRange => "invalid-number-range",
        NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber => {
            "number-range-not-preceded-by-marker-expecting-number"
        }
        NativeLintCode::VerseTextFollowsVerseRange => "verse-text-follows-verse-range",
        NativeLintCode::UnknownMarker => "unknown-marker",
        NativeLintCode::UnknownCloseMarker => "unknown-close-marker",
        NativeLintCode::InconsistentChapterLabel => "inconsistent-chapter-label",
        NativeLintCode::MarkerNotValidInContext => "marker-not-valid-in-context",
        NativeLintCode::VerseOutsideExplicitParagraph => "verse-outside-explicit-paragraph",
    }
}

fn lint_category_str(category: NativeLintCategory) -> &'static str {
    match category {
        NativeLintCategory::Document => "document",
        NativeLintCategory::Structure => "structure",
        NativeLintCategory::Context => "context",
        NativeLintCategory::Numbering => "numbering",
    }
}

fn lint_severity_str(severity: NativeLintSeverity) -> &'static str {
    match severity {
        NativeLintSeverity::Error => "error",
        NativeLintSeverity::Warning => "warning",
    }
}

fn parse_html_note_mode(value: &str) -> Result<NativeHtmlNoteMode, String> {
    match value {
        "extracted" => Ok(NativeHtmlNoteMode::Extracted),
        "inline" => Ok(NativeHtmlNoteMode::Inline),
        _ => Err(format!("unknown html note mode '{value}'")),
    }
}

fn parse_html_caller_style(value: &str) -> Result<NativeHtmlCallerStyle, String> {
    match value {
        "numeric" => Ok(NativeHtmlCallerStyle::Numeric),
        "alphaLower" => Ok(NativeHtmlCallerStyle::AlphaLower),
        "alphaUpper" => Ok(NativeHtmlCallerStyle::AlphaUpper),
        "romanLower" => Ok(NativeHtmlCallerStyle::RomanLower),
        "romanUpper" => Ok(NativeHtmlCallerStyle::RomanUpper),
        "source" => Ok(NativeHtmlCallerStyle::Source),
        _ => Err(format!("unknown html caller style '{value}'")),
    }
}

fn parse_html_caller_scope(value: &str) -> Result<NativeHtmlCallerScope, String> {
    match value {
        "documentSequential" => Ok(NativeHtmlCallerScope::DocumentSequential),
        "verseSequential" => Ok(NativeHtmlCallerScope::VerseSequential),
        _ => Err(format!("unknown html caller scope '{value}'")),
    }
}

fn parse_token_kind(value: &str) -> Result<NativeTokenKind, JsError> {
    match value {
        "newline" => Ok(NativeTokenKind::Newline),
        "optBreak" => Ok(NativeTokenKind::OptBreak),
        "marker" => Ok(NativeTokenKind::Marker),
        "endMarker" => Ok(NativeTokenKind::EndMarker),
        "milestone" => Ok(NativeTokenKind::Milestone),
        "milestoneEnd" => Ok(NativeTokenKind::MilestoneEnd),
        "bookCode" => Ok(NativeTokenKind::BookCode),
        "number" => Ok(NativeTokenKind::Number),
        "text" => Ok(NativeTokenKind::Text),
        "attributeList" => Ok(NativeTokenKind::AttributeList),
        _ => Err(js_error(format!("unknown token kind '{value}'"))),
    }
}

fn token_kind_str(kind: NativeTokenKind) -> &'static str {
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
        NativeTokenKind::AttributeList => "attributeList",
    }
}

fn token_kind_key(kind: NativeTokenKind) -> &'static str {
    token_kind_str(kind)
}

fn parse_number_kind(value: &str) -> Result<NativeNumberRangeKind, JsError> {
    match value {
        "single" => Ok(NativeNumberRangeKind::Single),
        "range" => Ok(NativeNumberRangeKind::Range),
        "sequence" => Ok(NativeNumberRangeKind::Sequence),
        "sequenceWithRange" => Ok(NativeNumberRangeKind::SequenceWithRange),
        _ => Err(js_error(format!("unknown number kind '{value}'"))),
    }
}

fn number_kind_str(kind: NativeNumberRangeKind) -> &'static str {
    match kind {
        NativeNumberRangeKind::Single => "single",
        NativeNumberRangeKind::Range => "range",
        NativeNumberRangeKind::Sequence => "sequence",
        NativeNumberRangeKind::SequenceWithRange => "sequenceWithRange",
    }
}

fn parse_scope_kind(value: &str) -> Result<StructuralScopeKind, JsError> {
    match value {
        "unknown" => Ok(StructuralScopeKind::Unknown),
        "header" => Ok(StructuralScopeKind::Header),
        "block" => Ok(StructuralScopeKind::Block),
        "note" => Ok(StructuralScopeKind::Note),
        "character" => Ok(StructuralScopeKind::Character),
        "milestone" => Ok(StructuralScopeKind::Milestone),
        "chapter" => Ok(StructuralScopeKind::Chapter),
        "verse" => Ok(StructuralScopeKind::Verse),
        "tableRow" => Ok(StructuralScopeKind::TableRow),
        "tableCell" => Ok(StructuralScopeKind::TableCell),
        "sidebar" => Ok(StructuralScopeKind::Sidebar),
        "periph" => Ok(StructuralScopeKind::Periph),
        "meta" => Ok(StructuralScopeKind::Meta),
        _ => Err(js_error(format!("unknown structural scope kind '{value}'"))),
    }
}

fn scope_kind_str(kind: StructuralScopeKind) -> &'static str {
    match kind {
        StructuralScopeKind::Unknown => "unknown",
        StructuralScopeKind::Header => "header",
        StructuralScopeKind::Block => "block",
        StructuralScopeKind::Note => "note",
        StructuralScopeKind::Character => "character",
        StructuralScopeKind::Milestone => "milestone",
        StructuralScopeKind::Chapter => "chapter",
        StructuralScopeKind::Verse => "verse",
        StructuralScopeKind::TableRow => "tableRow",
        StructuralScopeKind::TableCell => "tableCell",
        StructuralScopeKind::Sidebar => "sidebar",
        StructuralScopeKind::Periph => "periph",
        StructuralScopeKind::Meta => "meta",
    }
}

fn parse_inline_context(value: &str) -> Result<InlineContext, String> {
    match value {
        "para" => Ok(InlineContext::Para),
        "section" => Ok(InlineContext::Section),
        "list" => Ok(InlineContext::List),
        "table" => Ok(InlineContext::Table),
        _ => Err(format!("unknown inline context '{value}'")),
    }
}

fn inline_context_str(context: InlineContext) -> &'static str {
    match context {
        InlineContext::Para => "para",
        InlineContext::Section => "section",
        InlineContext::List => "list",
        InlineContext::Table => "table",
    }
}

fn parse_spec_context(value: &str) -> Result<SpecContext, String> {
    match value {
        "scripture" => Ok(SpecContext::Scripture),
        "bookIdentification" => Ok(SpecContext::BookIdentification),
        "bookHeaders" => Ok(SpecContext::BookHeaders),
        "bookTitles" => Ok(SpecContext::BookTitles),
        "bookIntroduction" => Ok(SpecContext::BookIntroduction),
        "bookIntroductionEndTitles" => Ok(SpecContext::BookIntroductionEndTitles),
        "bookChapterLabel" => Ok(SpecContext::BookChapterLabel),
        "chapterContent" => Ok(SpecContext::ChapterContent),
        "peripheral" => Ok(SpecContext::Peripheral),
        "peripheralContent" => Ok(SpecContext::PeripheralContent),
        "peripheralDivision" => Ok(SpecContext::PeripheralDivision),
        "chapter" => Ok(SpecContext::Chapter),
        "verse" => Ok(SpecContext::Verse),
        "section" => Ok(SpecContext::Section),
        "para" => Ok(SpecContext::Para),
        "list" => Ok(SpecContext::List),
        "table" => Ok(SpecContext::Table),
        "sidebar" => Ok(SpecContext::Sidebar),
        "footnote" => Ok(SpecContext::Footnote),
        "crossReference" => Ok(SpecContext::CrossReference),
        _ => Err(format!("unknown spec context '{value}'")),
    }
}

fn spec_context_str(context: SpecContext) -> &'static str {
    match context {
        SpecContext::Scripture => "scripture",
        SpecContext::BookIdentification => "bookIdentification",
        SpecContext::BookHeaders => "bookHeaders",
        SpecContext::BookTitles => "bookTitles",
        SpecContext::BookIntroduction => "bookIntroduction",
        SpecContext::BookIntroductionEndTitles => "bookIntroductionEndTitles",
        SpecContext::BookChapterLabel => "bookChapterLabel",
        SpecContext::ChapterContent => "chapterContent",
        SpecContext::Peripheral => "peripheral",
        SpecContext::PeripheralContent => "peripheralContent",
        SpecContext::PeripheralDivision => "peripheralDivision",
        SpecContext::Chapter => "chapter",
        SpecContext::Verse => "verse",
        SpecContext::Section => "section",
        SpecContext::Para => "para",
        SpecContext::List => "list",
        SpecContext::Table => "table",
        SpecContext::Sidebar => "sidebar",
        SpecContext::Footnote => "footnote",
        SpecContext::CrossReference => "crossReference",
    }
}

fn spec_marker_kind_str(kind: usfm_onion::marker_defs::SpecMarkerKind) -> &'static str {
    match kind {
        usfm_onion::marker_defs::SpecMarkerKind::Paragraph => "paragraph",
        usfm_onion::marker_defs::SpecMarkerKind::Character => "character",
        usfm_onion::marker_defs::SpecMarkerKind::Note => "note",
        usfm_onion::marker_defs::SpecMarkerKind::Chapter => "chapter",
        usfm_onion::marker_defs::SpecMarkerKind::Verse => "verse",
        usfm_onion::marker_defs::SpecMarkerKind::Milestone => "milestone",
        usfm_onion::marker_defs::SpecMarkerKind::Figure => "figure",
        usfm_onion::marker_defs::SpecMarkerKind::Sidebar => "sidebar",
        usfm_onion::marker_defs::SpecMarkerKind::Periph => "periph",
        usfm_onion::marker_defs::SpecMarkerKind::Meta => "meta",
        usfm_onion::marker_defs::SpecMarkerKind::TableRow => "tableRow",
        usfm_onion::marker_defs::SpecMarkerKind::TableCell => "tableCell",
        usfm_onion::marker_defs::SpecMarkerKind::Header => "header",
    }
}

fn diff_status_str(status: NativeDiffStatus) -> &'static str {
    match status {
        NativeDiffStatus::Added => "added",
        NativeDiffStatus::Deleted => "deleted",
        NativeDiffStatus::Modified => "modified",
        NativeDiffStatus::Unchanged => "unchanged",
    }
}

fn diff_token_change_str(change: NativeDiffTokenChange) -> &'static str {
    match change {
        NativeDiffTokenChange::Unchanged => "unchanged",
        NativeDiffTokenChange::Added => "added",
        NativeDiffTokenChange::Deleted => "deleted",
        NativeDiffTokenChange::Modified => "modified",
    }
}

fn diff_undo_side_str(side: NativeDiffUndoSide) -> &'static str {
    match side {
        NativeDiffUndoSide::Original => "original",
        NativeDiffUndoSide::Current => "current",
    }
}

fn marker_category_str(category: NativeMarkerCategory) -> &'static str {
    match category {
        NativeMarkerCategory::Document => "document",
        NativeMarkerCategory::Paragraph => "paragraph",
        NativeMarkerCategory::Character => "character",
        NativeMarkerCategory::NoteContainer => "noteContainer",
        NativeMarkerCategory::NoteSubmarker => "noteSubmarker",
        NativeMarkerCategory::Chapter => "chapter",
        NativeMarkerCategory::Verse => "verse",
        NativeMarkerCategory::MilestoneStart => "milestoneStart",
        NativeMarkerCategory::MilestoneEnd => "milestoneEnd",
        NativeMarkerCategory::Figure => "figure",
        NativeMarkerCategory::SidebarStart => "sidebarStart",
        NativeMarkerCategory::SidebarEnd => "sidebarEnd",
        NativeMarkerCategory::Periph => "periph",
        NativeMarkerCategory::Meta => "meta",
        NativeMarkerCategory::TableRow => "tableRow",
        NativeMarkerCategory::TableCell => "tableCell",
        NativeMarkerCategory::Header => "header",
        NativeMarkerCategory::Unknown => "unknown",
    }
}

fn marker_kind_str(kind: NativeMarkerKind) -> &'static str {
    match kind {
        NativeMarkerKind::Paragraph => "paragraph",
        NativeMarkerKind::Note => "note",
        NativeMarkerKind::Character => "character",
        NativeMarkerKind::Header => "header",
        NativeMarkerKind::Chapter => "chapter",
        NativeMarkerKind::Verse => "verse",
        NativeMarkerKind::MilestoneStart => "milestoneStart",
        NativeMarkerKind::MilestoneEnd => "milestoneEnd",
        NativeMarkerKind::SidebarStart => "sidebarStart",
        NativeMarkerKind::SidebarEnd => "sidebarEnd",
        NativeMarkerKind::Figure => "figure",
        NativeMarkerKind::Meta => "meta",
        NativeMarkerKind::Periph => "periph",
        NativeMarkerKind::TableRow => "tableRow",
        NativeMarkerKind::TableCell => "tableCell",
        NativeMarkerKind::Unknown => "unknown",
    }
}

fn marker_family_str(family: MarkerFamily) -> &'static str {
    match family {
        MarkerFamily::Footnote => "footnote",
        MarkerFamily::CrossReference => "crossReference",
        MarkerFamily::SectionParagraph => "sectionParagraph",
        MarkerFamily::ListParagraph => "listParagraph",
        MarkerFamily::TableCell => "tableCell",
        MarkerFamily::Milestone => "milestone",
        MarkerFamily::Sidebar => "sidebar",
    }
}

fn marker_family_role_str(role: MarkerFamilyRole) -> &'static str {
    match role {
        MarkerFamilyRole::Canonical => "canonical",
        MarkerFamilyRole::NumberedVariant => "numberedVariant",
        MarkerFamilyRole::NestedVariant => "nestedVariant",
        MarkerFamilyRole::MilestoneStart => "milestoneStart",
        MarkerFamilyRole::MilestoneEnd => "milestoneEnd",
        MarkerFamilyRole::Alias => "alias",
    }
}

fn marker_note_family_str(family: NativeMarkerNoteFamily) -> &'static str {
    match family {
        NativeMarkerNoteFamily::Footnote => "footnote",
        NativeMarkerNoteFamily::CrossReference => "crossReference",
    }
}

fn marker_note_subkind_str(kind: NativeMarkerNoteSubkind) -> &'static str {
    match kind {
        NativeMarkerNoteSubkind::Structural => "structural",
        NativeMarkerNoteSubkind::StructuralKeepsNestedCharsOpen => {
            "structuralKeepsNestedCharsOpen"
        }
    }
}

fn marker_inline_context_str(context: NativeMarkerInlineContext) -> &'static str {
    match context {
        NativeMarkerInlineContext::Para => "para",
        NativeMarkerInlineContext::Section => "section",
        NativeMarkerInlineContext::List => "list",
        NativeMarkerInlineContext::Table => "table",
    }
}

fn block_behavior_str(behavior: BlockBehavior) -> &'static str {
    match behavior {
        BlockBehavior::None => "none",
        BlockBehavior::Paragraph(_) => "paragraph",
        BlockBehavior::TableRow => "tableRow",
        BlockBehavior::TableCell => "tableCell",
        BlockBehavior::SidebarStart => "sidebarStart",
        BlockBehavior::SidebarEnd => "sidebarEnd",
    }
}

fn closing_behavior_str(behavior: ClosingBehavior) -> &'static str {
    match behavior {
        ClosingBehavior::None => "none",
        ClosingBehavior::RequiredExplicit => "requiredExplicit",
        ClosingBehavior::OptionalExplicitUntilNoteEnd => "optionalExplicitUntilNoteEnd",
        ClosingBehavior::SelfClosingMilestone => "selfClosingMilestone",
    }
}

fn format_sid(book: &str, chapter: u32, verse: u32) -> String {
    format!("{book} {chapter}:{verse}")
}
