use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value as from_js_value, to_value as swb_to_js_value};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use usfm_onion::cst::{CstDocument as NativeCstDocument, CstNode as NativeCstNode, parse_cst};
use usfm_onion::diff::{
    BuildSidBlocksOptions as NativeBuildSidBlocksOptions,
    ChapterTokenDiff as NativeChapterTokenDiff, DiffStatus as NativeDiffStatus,
    DiffTokenChange as NativeDiffTokenChange, DiffUndoSide as NativeDiffUndoSide, DiffableToken,
    DiffsByChapterMap as NativeDiffsByChapterMap, SidBlock as NativeSidBlock,
    TokenAlignment as NativeTokenAlignment, apply_revert_by_block_id, apply_reverts_by_block_id,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
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
    LintCategory as NativeLintCategory, LintCode as NativeLintCode,
    LintIssueType as NativeLintIssueType, LintOptions as NativeLintOptions,
    LintResult as NativeLintResult,
    LintSeverity as NativeLintSeverity, LintSuppression as NativeLintSuppression, LintableToken,
    TokenFix as NativeTokenFix, apply_token_fix, lint_tokens, lint_usfm,
};
use usfm_onion::marker_defs::{
    BlockBehavior, ClosingBehavior, InlineContext, MarkerFamily, MarkerFamilyRole, NoteFamily,
    NoteSubkind, SpecContext, StructuralMarkerInfo, StructuralScopeKind,
};
use usfm_onion::walker::WalkableToken;
use usfm_onion::markers::{
    MarkerCategory as NativeMarkerCategory, MarkerKind as NativeMarkerKind,
    UsfmMarkerInfo as NativeUsfmMarkerInfo, is_known_marker, marker_catalog, marker_info,
};
use usfm_onion::parse::parse as native_parse;
use usfm_onion::token::{
    AttributeItem as NativeAttributeItem, MarkerMetadata as NativeMarkerMetadata,
    NumberRangeKind as NativeNumberRangeKind, Span as NativeSpan, Token as NativeToken,
    TokenData as NativeTokenData, TokenKind as NativeTokenKind, tokens_to_usfm,
};
use usfm_onion::usj::usfm_to_usj;
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
export type LintIssueType = "usfm" | "content";
export type LintCode =
  | "missing-id-marker"
  | "duplicate-id-marker"
  | "id-marker-not-at-file-start"
  | "empty-paragraph"
  | "missing-chapter-number"
  | "missing-verse-number"
  | "verse-is-empty"
  | "unknown-token"
  | "unknown-marker"
  | "unknown-close-marker"
  | "content-before-first-chapter"
  | "verse-outside-explicit-paragraph"
  | "note-submarker-outside-note"
  | "metadata-outside-target"
  | "marker-not-valid-in-context"
  | "missing-milestone-self-close"
  | "stray-close-marker"
  | "misnested-close-marker"
  | "implicitly-closed-marker"
  | "unclosed-marker"
  | "duplicate-chapter-number"
  | "chapter-expected-increase-by-one"
  | "inconsistent-chapter-label"
  | "duplicate-verse-number"
  | "verse-expected-increase-by-one"
  | "invalid-number-range"
  | "number-range-not-preceded-by-marker-expecting-number"
  | "missing-whitespace-before-marker"
  | "missing-horizontal-whitespace-after-marker-name"
  | "missing-tag-end-delimiter-after-marker"
  | "excess-whitespace-around-marker"
  | "excess-whitespace-in-content"
  | "missing-content-space-after-close-marker"
  | "verse-in-section-or-other-paragraph";
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
  issueType: LintIssueType;
  message: string;
  messageParams: Record<string, string>;
  span?: Span;
  relatedSpan?: Span;
  tokenId?: string;
  relatedTokenId?: string;
  sid?: string;
  marker?: string;
  fix?: TokenFix;
}

export interface LintSummary {
  byCategory: Partial<Record<LintCategory, number>>;
  bySeverity: Partial<Record<LintSeverity, number>>;
  byIssueType: Partial<Record<LintIssueType, number>>;
  totalCount: number;
  suppressedCount: number;
}

export interface LintResult {
  issues: LintIssue[];
  summary: LintSummary;
}

export type TokenFix =
  | {
      type: "replaceToken";
      code: string;
      label: string;
      labelParams: Record<string, string>;
      targetTokenId: string;
      replacements: { kind: TokenKind; text: string; marker?: string; sid?: string }[];
    }
  | {
      type: "deleteToken";
      code: string;
      label: string;
      labelParams: Record<string, string>;
      targetTokenId: string;
    }
  | {
      type: "insertAfter";
      code: string;
      label: string;
      labelParams: Record<string, string>;
      targetTokenId: string;
      insert: { kind: TokenKind; text: string; marker?: string; sid?: string }[];
    };


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
  issueType: LintIssueType;
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
  applyTokenFix(fix: TokenFix): Token[];
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

export class UsfmMarkerCatalog {
  private constructor();
  all(): MarkerInfo[];
  get(marker: string): MarkerInfo | undefined;
  contains(marker: string): boolean;
}

export function parse(source: string): ParsedUsfm;
export function lintUsfm(source: string, options?: LintOptions): LintResult;
export function lintTokens(tokens: Token[], options?: LintOptions): LintResult;
export function applyTokenFix(tokens: Token[], fix: TokenFix): Token[];
export function formatUsfm(source: string, options?: FormatOptions): string;
export function formatTokens(tokens: FormatToken[], options?: FormatOptions): FormatResult;
export function formatTokensMut(tokens: FormatToken[], options?: FormatOptions): FormatToken[];
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

// ---------------------------------------------------------------------------
// FFI enums (tsify-derived) — one source of truth for the JS-side string
// unions that used to be hand-written in TS_TYPES.
//
// Each enum here mirrors a native enum and exposes the same wire format
// the previous stringify pairs produced. From impls in both directions keep
// the boundary between native and FFI fully typed; no string parsing.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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
    ChapterExpectedIncreaseByOne,
    InconsistentChapterLabel,
    DuplicateVerseNumber,
    VerseExpectedIncreaseByOne,
    InvalidNumberRange,
    NumberRangeNotPrecededByMarkerExpectingNumber,
    MissingWhitespaceBeforeMarker,
    MissingHorizontalWhitespaceAfterMarkerName,
    MissingTagEndDelimiterAfterMarker,
    ExcessWhitespaceAroundMarker,
    ExcessWhitespaceInContent,
    MissingContentSpaceAfterCloseMarker,
    VerseInSectionOrOtherParagraph,
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
            NativeLintCode::ChapterExpectedIncreaseByOne => Self::ChapterExpectedIncreaseByOne,
            NativeLintCode::InconsistentChapterLabel => Self::InconsistentChapterLabel,
            NativeLintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            NativeLintCode::VerseExpectedIncreaseByOne => Self::VerseExpectedIncreaseByOne,
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
            NativeLintCode::ExcessWhitespaceAroundMarker => Self::ExcessWhitespaceAroundMarker,
            NativeLintCode::ExcessWhitespaceInContent => Self::ExcessWhitespaceInContent,
            NativeLintCode::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            NativeLintCode::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
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
            LintCode::ChapterExpectedIncreaseByOne => Self::ChapterExpectedIncreaseByOne,
            LintCode::InconsistentChapterLabel => Self::InconsistentChapterLabel,
            LintCode::DuplicateVerseNumber => Self::DuplicateVerseNumber,
            LintCode::VerseExpectedIncreaseByOne => Self::VerseExpectedIncreaseByOne,
            LintCode::InvalidNumberRange => Self::InvalidNumberRange,
            LintCode::NumberRangeNotPrecededByMarkerExpectingNumber => {
                Self::NumberRangeNotPrecededByMarkerExpectingNumber
            }
            LintCode::MissingWhitespaceBeforeMarker => Self::MissingWhitespaceBeforeMarker,
            LintCode::MissingHorizontalWhitespaceAfterMarkerName => {
                Self::MissingHorizontalWhitespaceAfterMarkerName
            }
            LintCode::MissingTagEndDelimiterAfterMarker => Self::MissingTagEndDelimiterAfterMarker,
            LintCode::ExcessWhitespaceAroundMarker => Self::ExcessWhitespaceAroundMarker,
            LintCode::ExcessWhitespaceInContent => Self::ExcessWhitespaceInContent,
            LintCode::MissingContentSpaceAfterCloseMarker => {
                Self::MissingContentSpaceAfterCloseMarker
            }
            LintCode::VerseInSectionOrOtherParagraph => Self::VerseInSectionOrOtherParagraph,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

impl From<NativeDiffStatus> for DiffStatus {
    fn from(value: NativeDiffStatus) -> Self {
        match value {
            NativeDiffStatus::Added => Self::Added,
            NativeDiffStatus::Deleted => Self::Deleted,
            NativeDiffStatus::Modified => Self::Modified,
            NativeDiffStatus::Unchanged => Self::Unchanged,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffTokenChange {
    Unchanged,
    Added,
    Deleted,
    Modified,
}

impl From<NativeDiffTokenChange> for DiffTokenChange {
    fn from(value: NativeDiffTokenChange) -> Self {
        match value {
            NativeDiffTokenChange::Unchanged => Self::Unchanged,
            NativeDiffTokenChange::Added => Self::Added,
            NativeDiffTokenChange::Deleted => Self::Deleted,
            NativeDiffTokenChange::Modified => Self::Modified,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum DiffUndoSide {
    Original,
    Current,
}

impl From<NativeDiffUndoSide> for DiffUndoSide {
    fn from(value: NativeDiffUndoSide) -> Self {
        match value {
            NativeDiffUndoSide::Original => Self::Original,
            NativeDiffUndoSide::Current => Self::Current,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiStructuralScopeKind {
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

impl From<StructuralScopeKind> for FfiStructuralScopeKind {
    fn from(value: StructuralScopeKind) -> Self {
        match value {
            StructuralScopeKind::Unknown => Self::Unknown,
            StructuralScopeKind::Header => Self::Header,
            StructuralScopeKind::Block => Self::Block,
            StructuralScopeKind::Note => Self::Note,
            StructuralScopeKind::Character => Self::Character,
            StructuralScopeKind::Milestone => Self::Milestone,
            StructuralScopeKind::Chapter => Self::Chapter,
            StructuralScopeKind::Verse => Self::Verse,
            StructuralScopeKind::TableRow => Self::TableRow,
            StructuralScopeKind::TableCell => Self::TableCell,
            StructuralScopeKind::Sidebar => Self::Sidebar,
            StructuralScopeKind::Periph => Self::Periph,
            StructuralScopeKind::Meta => Self::Meta,
        }
    }
}

impl From<FfiStructuralScopeKind> for StructuralScopeKind {
    fn from(value: FfiStructuralScopeKind) -> Self {
        match value {
            FfiStructuralScopeKind::Unknown => Self::Unknown,
            FfiStructuralScopeKind::Header => Self::Header,
            FfiStructuralScopeKind::Block => Self::Block,
            FfiStructuralScopeKind::Note => Self::Note,
            FfiStructuralScopeKind::Character => Self::Character,
            FfiStructuralScopeKind::Milestone => Self::Milestone,
            FfiStructuralScopeKind::Chapter => Self::Chapter,
            FfiStructuralScopeKind::Verse => Self::Verse,
            FfiStructuralScopeKind::TableRow => Self::TableRow,
            FfiStructuralScopeKind::TableCell => Self::TableCell,
            FfiStructuralScopeKind::Sidebar => Self::Sidebar,
            FfiStructuralScopeKind::Periph => Self::Periph,
            FfiStructuralScopeKind::Meta => Self::Meta,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiSpecContext {
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

impl From<SpecContext> for FfiSpecContext {
    fn from(value: SpecContext) -> Self {
        match value {
            SpecContext::Scripture => Self::Scripture,
            SpecContext::BookIdentification => Self::BookIdentification,
            SpecContext::BookHeaders => Self::BookHeaders,
            SpecContext::BookTitles => Self::BookTitles,
            SpecContext::BookIntroduction => Self::BookIntroduction,
            SpecContext::BookIntroductionEndTitles => Self::BookIntroductionEndTitles,
            SpecContext::BookChapterLabel => Self::BookChapterLabel,
            SpecContext::ChapterContent => Self::ChapterContent,
            SpecContext::Peripheral => Self::Peripheral,
            SpecContext::PeripheralContent => Self::PeripheralContent,
            SpecContext::PeripheralDivision => Self::PeripheralDivision,
            SpecContext::Chapter => Self::Chapter,
            SpecContext::Verse => Self::Verse,
            SpecContext::Section => Self::Section,
            SpecContext::Para => Self::Para,
            SpecContext::List => Self::List,
            SpecContext::Table => Self::Table,
            SpecContext::Sidebar => Self::Sidebar,
            SpecContext::Footnote => Self::Footnote,
            SpecContext::CrossReference => Self::CrossReference,
        }
    }
}

impl From<FfiSpecContext> for SpecContext {
    fn from(value: FfiSpecContext) -> Self {
        match value {
            FfiSpecContext::Scripture => Self::Scripture,
            FfiSpecContext::BookIdentification => Self::BookIdentification,
            FfiSpecContext::BookHeaders => Self::BookHeaders,
            FfiSpecContext::BookTitles => Self::BookTitles,
            FfiSpecContext::BookIntroduction => Self::BookIntroduction,
            FfiSpecContext::BookIntroductionEndTitles => Self::BookIntroductionEndTitles,
            FfiSpecContext::BookChapterLabel => Self::BookChapterLabel,
            FfiSpecContext::ChapterContent => Self::ChapterContent,
            FfiSpecContext::Peripheral => Self::Peripheral,
            FfiSpecContext::PeripheralContent => Self::PeripheralContent,
            FfiSpecContext::PeripheralDivision => Self::PeripheralDivision,
            FfiSpecContext::Chapter => Self::Chapter,
            FfiSpecContext::Verse => Self::Verse,
            FfiSpecContext::Section => Self::Section,
            FfiSpecContext::Para => Self::Para,
            FfiSpecContext::List => Self::List,
            FfiSpecContext::Table => Self::Table,
            FfiSpecContext::Sidebar => Self::Sidebar,
            FfiSpecContext::Footnote => Self::Footnote,
            FfiSpecContext::CrossReference => Self::CrossReference,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum FfiInlineContext {
    Para,
    Section,
    List,
    Table,
}

impl From<InlineContext> for FfiInlineContext {
    fn from(value: InlineContext) -> Self {
        match value {
            InlineContext::Para => Self::Para,
            InlineContext::Section => Self::Section,
            InlineContext::List => Self::List,
            InlineContext::Table => Self::Table,
        }
    }
}

impl From<FfiInlineContext> for InlineContext {
    fn from(value: FfiInlineContext) -> Self {
        match value {
            FfiInlineContext::Para => Self::Para,
            FfiInlineContext::Section => Self::Section,
            FfiInlineContext::List => Self::List,
            FfiInlineContext::Table => Self::Table,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiBlockBehavior {
    None,
    Paragraph,
    TableRow,
    TableCell,
    SidebarStart,
    SidebarEnd,
}

impl From<BlockBehavior> for FfiBlockBehavior {
    fn from(value: BlockBehavior) -> Self {
        match value {
            BlockBehavior::None => Self::None,
            BlockBehavior::Paragraph(_) => Self::Paragraph,
            BlockBehavior::TableRow => Self::TableRow,
            BlockBehavior::TableCell => Self::TableCell,
            BlockBehavior::SidebarStart => Self::SidebarStart,
            BlockBehavior::SidebarEnd => Self::SidebarEnd,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiClosingBehavior {
    None,
    RequiredExplicit,
    OptionalExplicitUntilNoteEnd,
    SelfClosingMilestone,
}

impl From<ClosingBehavior> for FfiClosingBehavior {
    fn from(value: ClosingBehavior) -> Self {
        match value {
            ClosingBehavior::None => Self::None,
            ClosingBehavior::RequiredExplicit => Self::RequiredExplicit,
            ClosingBehavior::OptionalExplicitUntilNoteEnd => Self::OptionalExplicitUntilNoteEnd,
            ClosingBehavior::SelfClosingMilestone => Self::SelfClosingMilestone,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiMarkerCategory {
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

impl From<NativeMarkerCategory> for FfiMarkerCategory {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiMarkerKind {
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

impl From<NativeMarkerKind> for FfiMarkerKind {
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

// `MarkerDefKind` (spec-level kind on tokens, distinct from `MarkerKind` which
// is the catalog-level kind on `UsfmMarkerInfo`). Smaller variant set — the
// spec doesn't distinguish milestone-start/-end or sidebar-start/-end.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiMarkerDefKind {
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

impl From<usfm_onion::marker_defs::MarkerDefKind> for FfiMarkerDefKind {
    fn from(value: usfm_onion::marker_defs::MarkerDefKind) -> Self {
        use usfm_onion::marker_defs::MarkerDefKind as K;
        match value {
            K::Paragraph => Self::Paragraph,
            K::Character => Self::Character,
            K::Note => Self::Note,
            K::Chapter => Self::Chapter,
            K::Verse => Self::Verse,
            K::Milestone => Self::Milestone,
            K::Figure => Self::Figure,
            K::Sidebar => Self::Sidebar,
            K::Periph => Self::Periph,
            K::Meta => Self::Meta,
            K::TableRow => Self::TableRow,
            K::TableCell => Self::TableCell,
            K::Header => Self::Header,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiMarkerFamily {
    Footnote,
    CrossReference,
    SectionParagraph,
    ListParagraph,
    TableCell,
    Milestone,
    Sidebar,
}

impl From<MarkerFamily> for FfiMarkerFamily {
    fn from(value: MarkerFamily) -> Self {
        match value {
            MarkerFamily::Footnote => Self::Footnote,
            MarkerFamily::CrossReference => Self::CrossReference,
            MarkerFamily::SectionParagraph => Self::SectionParagraph,
            MarkerFamily::ListParagraph => Self::ListParagraph,
            MarkerFamily::TableCell => Self::TableCell,
            MarkerFamily::Milestone => Self::Milestone,
            MarkerFamily::Sidebar => Self::Sidebar,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiMarkerFamilyRole {
    Canonical,
    NumberedVariant,
    NestedVariant,
    MilestoneStart,
    MilestoneEnd,
    Alias,
}

impl From<MarkerFamilyRole> for FfiMarkerFamilyRole {
    fn from(value: MarkerFamilyRole) -> Self {
        match value {
            MarkerFamilyRole::Canonical => Self::Canonical,
            MarkerFamilyRole::NumberedVariant => Self::NumberedVariant,
            MarkerFamilyRole::NestedVariant => Self::NestedVariant,
            MarkerFamilyRole::MilestoneStart => Self::MilestoneStart,
            MarkerFamilyRole::MilestoneEnd => Self::MilestoneEnd,
            MarkerFamilyRole::Alias => Self::Alias,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiNoteFamily {
    Footnote,
    CrossReference,
}

impl From<NoteFamily> for FfiNoteFamily {
    fn from(value: NoteFamily) -> Self {
        match value {
            NoteFamily::Footnote => Self::Footnote,
            NoteFamily::CrossReference => Self::CrossReference,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FfiNoteSubkind {
    Structural,
    StructuralKeepsNestedCharsOpen,
}

impl From<NoteSubkind> for FfiNoteSubkind {
    fn from(value: NoteSubkind) -> Self {
        match value {
            NoteSubkind::Structural => Self::Structural,
            NoteSubkind::StructuralKeepsNestedCharsOpen => Self::StructuralKeepsNestedCharsOpen,
        }
    }
}

// ---------------------------------------------------------------------------
// Value types — string fields below will migrate to the FFI enums above in
// phase 4. The shape is preserved during the enum migration so goldens stay
// byte-identical at this step.
// ---------------------------------------------------------------------------

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
    kind: Option<FfiMarkerDefKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<FfiMarkerFamily>,
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
    scope_kind: FfiStructuralScopeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<FfiInlineContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_context: Option<FfiSpecContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NumberInfoValue {
    start: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end: Option<u32>,
    kind: NumberRangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenValue {
    id: String,
    kind: TokenKind,
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
    code: LintCode,
    sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LintOptionsValue {
    #[serde(default)]
    enabled_codes: Option<Vec<LintCode>>,
    #[serde(default)]
    disabled_codes: Vec<LintCode>,
    #[serde(default)]
    suppressed: Vec<LintSuppressionValue>,
    #[serde(default)]
    allow_implicit_chapter_content_verse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintIssueValue {
    code: LintCode,
    category: LintCategory,
    severity: LintSeverity,
    issue_type: LintIssueType,
    template: String,
    message: String,
    message_params: std::collections::BTreeMap<String, String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fix: Option<TokenFixValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintSummaryValue {
    by_category: std::collections::BTreeMap<LintCategory, usize>,
    by_severity: std::collections::BTreeMap<LintSeverity, usize>,
    by_issue_type: std::collections::BTreeMap<LintIssueType, usize>,
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
#[serde(tag = "type", rename_all = "camelCase")]
enum TokenFixValue {
    ReplaceToken {
        code: String,
        label: String,
        label_params: std::collections::BTreeMap<String, String>,
        target_token_id: String,
        replacements: Vec<TokenTemplateValue>,
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
        insert: Vec<TokenTemplateValue>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenTemplateValue {
    kind: TokenKind,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
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
    note_mode: Option<HtmlNoteMode>,
    #[serde(default)]
    caller_style: Option<HtmlCallerStyle>,
    #[serde(default)]
    caller_scope: Option<HtmlCallerScope>,
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
    change: DiffTokenChange,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    counterpart_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChapterTokenDiffValue {
    block_id: String,
    semantic_sid: String,
    status: DiffStatus,
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
    undo_side: DiffUndoSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LintCodeMetaValue {
    code: LintCode,
    category: LintCategory,
    severity: LintSeverity,
    issue_type: LintIssueType,
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
    category: FfiMarkerCategory,
    kind: FfiMarkerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<FfiMarkerFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family_role: Option<FfiMarkerFamilyRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_family: Option<FfiNoteFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_subkind: Option<FfiNoteSubkind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline_context: Option<FfiInlineContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_attribute: Option<String>,
    contexts: Vec<FfiSpecContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    block_behavior: Option<FfiBlockBehavior>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    closing_behavior: Option<FfiClosingBehavior>,
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

impl WalkableToken for AdapterToken {
    fn kind(&self) -> NativeTokenKind {
        self.kind
    }

    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        self.structural
    }

    fn text(&self) -> &str {
        &self.text
    }
}

impl LintableToken for AdapterToken {
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
        Some(token_kind_wire_key(self.kind))
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

    #[wasm_bindgen(js_name = applyTokenFix)]
    pub fn apply_token_fix(&self, fix: JsValue) -> Result<JsValue, JsError> {
        let parsed = native_parse(&self.source);
        let native_tokens = parsed
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let fix = parse_token_fix(fix)?;
        let result = apply_token_fix(&native_tokens, &fix);
        to_js_value(&result.iter().map(map_format_token).collect::<Vec<_>>())
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
    pub fn to_html(&self, options: Option<JsValue>) -> Result<String, JsError> {
        let options = parse_html_options(options)?;
        Ok(usfm_to_html(&self.source, options))
    }

    #[wasm_bindgen(js_name = toVref)]
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

#[wasm_bindgen(skip_typescript, js_name = applyTokenFix)]
pub fn wasm_apply_token_fix(tokens: JsValue, fix: JsValue) -> Result<JsValue, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let native_tokens = values
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let fix = parse_token_fix(fix)?;
    let result = apply_token_fix(&native_tokens, &fix);
    to_js_value(&result.iter().map(map_format_token).collect::<Vec<_>>())
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
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, parse_format_options(options)?);
    let formatted = FormatResultValue {
        tokens: native_tokens.iter().map(map_format_token).collect(),
        usfm: format_tokens_to_usfm(&native_tokens),
    };
    to_js_value(&formatted)
}

#[wasm_bindgen(skip_typescript, js_name = formatTokensMut)]
pub fn wasm_format_tokens_mut(
    tokens: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
    let values = from_js_or_default::<Vec<TokenValue>>(tokens)?;
    let mut native_tokens = values
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, parse_format_options(options)?);
    let formatted = native_tokens
        .iter()
        .map(map_format_token)
        .collect::<Vec<_>>();
    to_js_value(&formatted)
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
pub fn wasm_diff_usfm(
    left: &str,
    right: &str,
    options: Option<JsValue>,
) -> Result<JsValue, JsError> {
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
        .collect::<Vec<_>>();
    let current = from_js_or_default::<Vec<TokenValue>>(current)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
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
        .collect::<Vec<_>>();
    let current = from_js_or_default::<Vec<TokenValue>>(current)?
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
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
        .map(LintCode::from)
        .collect::<Vec<_>>();
    to_js_value(&codes)
}

#[wasm_bindgen(skip_typescript, js_name = lintCodeMeta)]
pub fn wasm_lint_code_meta() -> Result<JsValue, JsError> {
    let meta = lint_code_variants()
        .into_iter()
        .map(|code| LintCodeMetaValue {
            code: code.into(),
            category: code.category().into(),
            severity: code.severity().into(),
            issue_type: code.issue_type().into(),
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
    })
}

fn parse_format_options(value: Option<JsValue>) -> Result<NativeFormatOptions, JsError> {
    let value = value.unwrap_or(JsValue::UNDEFINED);
    let value: FormatOptionsValue = from_js_or_default(value)?;
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
    Ok(parse_adapter_tokens_from_values(values))
}

fn parse_adapter_tokens_from_values(values: Vec<TokenValue>) -> Vec<AdapterToken> {
    values.into_iter().map(token_value_to_adapter).collect()
}

fn token_value_to_adapter(value: TokenValue) -> AdapterToken {
    AdapterToken {
        id: value.id,
        kind: value.kind.into(),
        text: value.text,
        span: value.span.map(native_span),
        sid: value.sid,
        marker: value.marker,
        structural: value.structural.map(parse_structural_info),
        number_info: value.number_info.map(parse_number_info),
    }
}

fn token_value_to_format_token(value: TokenValue) -> NativeFormatToken {
    NativeFormatToken {
        kind: value.kind.into(),
        text: value.text,
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
    owned.sid = token
        .sid
        .map(|sid| format_sid(sid.book.as_str(), sid.chapter, sid.verse));
    owned.id = Some(format!("{}-{}", token.id.book_code, token.id.index));
    owned
}

fn parse_token_fix(value: JsValue) -> Result<NativeTokenFix, JsError> {
    let value: TokenFixValue = from_js_value(value).map_err(js_serde_error)?;
    Ok(match value {
        TokenFixValue::ReplaceToken {
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
        TokenFixValue::DeleteToken {
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
        TokenFixValue::InsertAfter {
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
    })
}

fn parse_token_template(value: TokenTemplateValue) -> usfm_onion::TokenTemplate {
    usfm_onion::TokenTemplate {
        kind: value.kind.into(),
        text: value.text,
        marker: value.marker,
        sid: value.sid,
    }
}

fn parse_structural_info(value: StructuralMarkerInfoValue) -> StructuralMarkerInfo {
    StructuralMarkerInfo {
        scope_kind: value.scope_kind.into(),
        inline_context: value.inline_context.map(Into::into),
        note_context: value.note_context.map(Into::into),
    }
}

fn parse_number_info(value: NumberInfoValue) -> (u32, Option<u32>, NativeNumberRangeKind) {
    (value.start, value.end, value.kind.into())
}

fn map_tokens(tokens: &[NativeToken<'_>]) -> Vec<TokenValue> {
    tokens.iter().map(map_token).collect()
}

fn map_token(token: &NativeToken<'_>) -> TokenValue {
    let mut value = TokenValue {
        id: format!("{}-{}", token.id.book_code, token.id.index),
        kind: token.kind().into(),
        text: token.source.to_string(),
        span: Some(map_span(token.span)),
        sid: token
            .sid
            .map(|sid| format_sid(sid.book.as_str(), sid.chapter, sid.verse)),
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
            attributes,
            ..
        } => {
            value.nested = Some(*nested);
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
            value.attributes = attributes.iter().map(map_attribute_item).collect();
        }
        NativeTokenData::EndMarker {
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
            metadata,
            structural,
            attributes,
            ..
        } => {
            value.marker_metadata = Some(map_marker_metadata(*metadata));
            value.structural = Some(map_structural_info(*structural));
            value.attributes = attributes.iter().map(map_attribute_item).collect();
        }
        NativeTokenData::BookCode { code, is_valid } => {
            value.book_code = Some((*code).to_string());
            value.book_code_valid = Some(*is_valid);
        }
        NativeTokenData::Number { start, end, kind } => {
            value.number_info = Some(NumberInfoValue {
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

fn map_format_token(token: &NativeFormatToken) -> TokenValue {
    TokenValue {
        id: token.id.clone().unwrap_or_default(),
        kind: token.kind.into(),
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
            kind: kind.into(),
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
        kind: metadata.kind.map(Into::into),
        family: metadata.family.map(Into::into),
    }
}

fn map_structural_info(info: StructuralMarkerInfo) -> StructuralMarkerInfoValue {
    StructuralMarkerInfoValue {
        scope_kind: info.scope_kind.into(),
        inline_context: info.inline_context.map(Into::into),
        note_context: info.note_context.map(Into::into),
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

fn map_token_fix(fix: NativeTokenFix) -> TokenFixValue {
    match fix {
        NativeTokenFix::ReplaceToken {
            code,
            label,
            label_params,
            target_token_id,
            replacements,
        } => TokenFixValue::ReplaceToken {
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
        } => TokenFixValue::DeleteToken {
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
        } => TokenFixValue::InsertAfter {
            code,
            label,
            label_params,
            target_token_id,
            insert: insert.into_iter().map(map_token_template).collect(),
        },
    }
}

fn map_token_template(template: usfm_onion::TokenTemplate) -> TokenTemplateValue {
    TokenTemplateValue {
        kind: template.kind.into(),
        text: template.text,
        marker: template.marker,
        sid: template.sid,
    }
}

fn map_lint_issue(issue: usfm_onion::LintIssue) -> LintIssueValue {
    LintIssueValue {
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

fn map_chapter_diffs(
    diffs: &[NativeChapterTokenDiff<NativeToken<'_>>],
) -> Vec<ChapterTokenDiffValue> {
    diffs.iter().map(map_native_chapter_diff).collect()
}

fn map_native_chapter_diff(
    diff: &NativeChapterTokenDiff<NativeToken<'_>>,
) -> ChapterTokenDiffValue {
    ChapterTokenDiffValue {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff.status.into(),
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
        undo_side: diff.undo_side.into(),
    }
}

fn map_adapter_diffs(diffs: &[NativeChapterTokenDiff<AdapterToken>]) -> Vec<ChapterTokenDiffValue> {
    diffs.iter().map(map_adapter_chapter_diff).collect()
}

fn map_adapter_chapter_diff(diff: &NativeChapterTokenDiff<AdapterToken>) -> ChapterTokenDiffValue {
    ChapterTokenDiffValue {
        block_id: diff.block_id.clone(),
        semantic_sid: diff.semantic_sid.clone(),
        status: diff.status.into(),
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
        undo_side: diff.undo_side.into(),
    }
}

fn map_adapter_token(token: &AdapterToken) -> TokenValue {
    TokenValue {
        id: token.id.clone(),
        kind: token.kind.into(),
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
            kind: kind.into(),
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
        change: alignment.change.into(),
        counterpart_index: alignment.counterpart_index,
    }
}

fn map_diffs_by_chapter(
    diffs: &NativeDiffsByChapterMap<NativeChapterTokenDiff<NativeToken<'_>>>,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<u32, Vec<ChapterTokenDiffValue>>>
{
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
        NativeLintCode::ChapterExpectedIncreaseByOne,
        NativeLintCode::InconsistentChapterLabel,
        NativeLintCode::DuplicateVerseNumber,
        NativeLintCode::VerseExpectedIncreaseByOne,
        NativeLintCode::InvalidNumberRange,
        NativeLintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        NativeLintCode::MissingWhitespaceBeforeMarker,
        NativeLintCode::MissingHorizontalWhitespaceAfterMarkerName,
        NativeLintCode::MissingTagEndDelimiterAfterMarker,
        NativeLintCode::ExcessWhitespaceAroundMarker,
        NativeLintCode::ExcessWhitespaceInContent,
        NativeLintCode::MissingContentSpaceAfterCloseMarker,
        NativeLintCode::VerseInSectionOrOtherParagraph,
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

fn format_sid(book: &str, chapter: u16, verse: u16) -> String {
    format!("{book} {chapter}:{verse}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_tokens_accepts_parsed_footnote_submarker_streams() {
        let source = "\\id GEN\n\\c 1\n\\pi1\n\\v 26 Then God said, “Let Us make man in Our image, after Our likeness, to rule over the fish of the sea and the birds of the air, over the livestock, and over all the earth itself\\f + \\fr 1:26 \\ft MT; Syriac \\fqa and over all the beasts of the earth\\f* and every creature that crawls upon it.”\n\\q1\n";
        let token_values = map_tokens(&native_parse(source).tokens);
        let adapter_tokens = parse_adapter_tokens_from_values(token_values);
        let result = lint_tokens(&adapter_tokens, NativeLintOptions::default());

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
        let adapter_tokens = parse_adapter_tokens_from_values(token_values);
        let result = lint_tokens(&adapter_tokens, NativeLintOptions::default());

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
        let batches: Vec<Vec<AdapterToken>> = sources
            .iter()
            .map(|source| map_tokens(&native_parse(source).tokens))
            .map(parse_adapter_tokens_from_values)
            .collect();

        let results = batches
            .iter()
            .map(|tokens| lint_tokens(tokens, NativeLintOptions::default()))
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
        let code = usfm_onion::LintCode::VerseExpectedIncreaseByOne;
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
        assert_eq!(mapped.message_params.get("expected"), Some(&"2".to_string()));
        assert_eq!(mapped.message_params.get("found"), Some(&"3".to_string()));
    }
}
