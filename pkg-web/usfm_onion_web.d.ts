/* tslint:disable */
/* eslint-disable */

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


export interface AttributeItemValue {
    span: SpanValue;
    text: string;
    key: string;
    value: string;
}

export interface ChapterTokenDiffValue {
    blockId: string;
    semanticSid: string;
    status: DiffStatus;
    original?: SidBlockValue;
    current?: SidBlockValue;
    originalText: string;
    currentText: string;
    originalTextOnly: string;
    currentTextOnly: string;
    isWhitespaceChange: boolean;
    isUsfmStructureChange: boolean;
    originalTokens: TokenValue[];
    currentTokens: TokenValue[];
    originalAlignment: TokenAlignmentValue[];
    currentAlignment: TokenAlignmentValue[];
    undoSide: DiffUndoSide;
}

export interface CstDocumentValue {
    tokens: TokenValue[];
    roots: CstNodeValue[];
}

export interface CstNodeValue {
    tokenIndex: number;
    children: CstNodeValue[];
}

export interface FormatResultValue {
    tokens: TokenValue[];
    usfm: string;
}

export interface FormatRuleMetaValue {
    code: string;
    labelKey: string;
}

export interface LintCodeMetaValue {
    code: LintCode;
    category: LintCategory;
    severity: LintSeverity;
    issueType: LintIssueType;
}

export interface LintIssueValue {
    code: LintCode;
    category: LintCategory;
    severity: LintSeverity;
    issueType: LintIssueType;
    template: string;
    message: string;
    messageParams: Record<string, string>;
    span?: SpanValue;
    relatedSpan?: SpanValue;
    tokenId?: string;
    relatedTokenId?: string;
    sid?: string;
    marker?: string;
    fix?: TokenFixValue;
}

export interface LintResultValue {
    issues: LintIssueValue[];
    summary: LintSummaryValue;
}

export interface LintSummaryValue {
    byCategory: Record<LintCategory, number>;
    bySeverity: Record<LintSeverity, number>;
    byIssueType: Record<LintIssueType, number>;
    totalCount: number;
    suppressedCount: number;
}

export interface LintSuppressionValue {
    code: LintCode;
    sid: string;
}

export interface MarkerInfoValue {
    marker: string;
    canonical?: string;
    known: boolean;
    deprecated: boolean;
    category: FfiMarkerCategory;
    kind: FfiMarkerKind;
    family?: FfiMarkerFamily;
    familyRole?: FfiMarkerFamilyRole;
    noteFamily?: FfiNoteFamily;
    noteSubkind?: FfiNoteSubkind;
    inlineContext?: FfiInlineContext;
    defaultAttribute?: string;
    contexts: FfiSpecContext[];
    blockBehavior?: FfiBlockBehavior;
    closingBehavior?: FfiClosingBehavior;
    source?: string;
}

export interface NumberInfoValue {
    start: number;
    end?: number;
    kind: NumberRangeKind;
}

export interface SidBlockValue {
    blockId: string;
    semanticSid: string;
    start: number;
    endExclusive: number;
    prevBlockId?: string;
    textFull: string;
}

export interface SpanValue {
    start: number;
    end: number;
}

export interface StructuralMarkerInfoValue {
    scopeKind: FfiStructuralScopeKind;
    inlineContext?: FfiInlineContext;
    noteContext?: FfiSpecContext;
}

export interface TokenAlignmentValue {
    change: DiffTokenChange;
    counterpartIndex?: number;
}

export interface TokenTemplateValue {
    kind: TokenKind;
    text: string;
    marker?: string;
    sid?: string;
}

export interface TokenValue {
    id: string;
    kind: TokenKind;
    text: string;
    span?: SpanValue;
    sid?: string;
    marker?: string;
    nested?: boolean;
    markerMetadata?: MarkerMetadataValue;
    structural?: StructuralMarkerInfoValue;
    numberInfo?: NumberInfoValue;
    bookCode?: string;
    bookCodeValid?: boolean;
    attributes?: AttributeItemValue[];
}

export type DiffStatus = "added" | "deleted" | "modified" | "unchanged";

export type DiffTokenChange = "unchanged" | "added" | "deleted" | "modified";

export type DiffUndoSide = "original" | "current";

export type FfiBlockBehavior = "none" | "paragraph" | "tableRow" | "tableCell" | "sidebarStart" | "sidebarEnd";

export type FfiClosingBehavior = "none" | "requiredExplicit" | "optionalExplicitUntilNoteEnd" | "selfClosingMilestone";

export type FfiInlineContext = "para" | "section" | "list" | "table";

export type FfiMarkerCategory = "document" | "paragraph" | "character" | "noteContainer" | "noteSubmarker" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "figure" | "sidebarStart" | "sidebarEnd" | "periph" | "meta" | "tableRow" | "tableCell" | "header" | "unknown";

export type FfiMarkerDefKind = "paragraph" | "character" | "note" | "chapter" | "verse" | "milestone" | "figure" | "sidebar" | "periph" | "meta" | "tableRow" | "tableCell" | "header";

export type FfiMarkerFamily = "footnote" | "crossReference" | "sectionParagraph" | "listParagraph" | "tableCell" | "milestone" | "sidebar";

export type FfiMarkerFamilyRole = "canonical" | "numberedVariant" | "nestedVariant" | "milestoneStart" | "milestoneEnd" | "alias";

export type FfiMarkerKind = "paragraph" | "note" | "character" | "header" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "sidebarStart" | "sidebarEnd" | "figure" | "meta" | "periph" | "tableRow" | "tableCell" | "unknown";

export type FfiNoteFamily = "footnote" | "crossReference";

export type FfiNoteSubkind = "structural" | "structuralKeepsNestedCharsOpen";

export type FfiSpecContext = "scripture" | "bookIdentification" | "bookHeaders" | "bookTitles" | "bookIntroduction" | "bookIntroductionEndTitles" | "bookChapterLabel" | "chapterContent" | "peripheral" | "peripheralContent" | "peripheralDivision" | "chapter" | "verse" | "section" | "para" | "list" | "table" | "sidebar" | "footnote" | "crossReference";

export type FfiStructuralScopeKind = "unknown" | "header" | "block" | "note" | "character" | "milestone" | "chapter" | "verse" | "tableRow" | "tableCell" | "sidebar" | "periph" | "meta";

export type HtmlCallerScope = "documentSequential" | "verseSequential";

export type HtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";

export type HtmlNoteMode = "extracted" | "inline";

export type LintCategory = "document" | "structure" | "context" | "numbering";

export type LintCode = "missing-id-marker" | "duplicate-id-marker" | "id-marker-not-at-file-start" | "empty-paragraph" | "missing-chapter-number" | "missing-verse-number" | "verse-is-empty" | "unknown-token" | "unknown-marker" | "unknown-close-marker" | "content-before-first-chapter" | "verse-outside-explicit-paragraph" | "note-submarker-outside-note" | "metadata-outside-target" | "marker-not-valid-in-context" | "missing-milestone-self-close" | "stray-close-marker" | "misnested-close-marker" | "implicitly-closed-marker" | "unclosed-marker" | "duplicate-chapter-number" | "chapter-expected-increase-by-one" | "inconsistent-chapter-label" | "duplicate-verse-number" | "verse-expected-increase-by-one" | "invalid-number-range" | "number-range-not-preceded-by-marker-expecting-number" | "missing-whitespace-before-marker" | "missing-horizontal-whitespace-after-marker-name" | "missing-tag-end-delimiter-after-marker" | "excess-whitespace-around-marker" | "excess-whitespace-in-content" | "missing-content-space-after-close-marker" | "verse-in-section-or-other-paragraph";

export type LintIssueType = "usfm" | "content";

export type LintSeverity = "error" | "warning";

export type NumberRangeKind = "single" | "range" | "sequence" | "sequenceWithRange";

export type TokenFixValue = { type: "replaceToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; replacements: TokenTemplateValue[] } | { type: "deleteToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string } | { type: "insertAfter"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; insert: TokenTemplateValue[] };

export type TokenKind = "newline" | "optBreak" | "marker" | "endMarker" | "milestone" | "milestoneEnd" | "bookCode" | "number" | "text";


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_parsedusfm_free: (a: number, b: number) => void;
    readonly __wbg_usfmmarkercatalog_free: (a: number, b: number) => void;
    readonly applyTokenFix: (a: any, b: any) => [number, number, number];
    readonly diffTokens: (a: any, b: any, c: number) => [number, number, number];
    readonly diffUsfm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly diffUsfmByChapter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly formatRuleMeta: () => [number, number, number];
    readonly formatRules: () => [number, number, number];
    readonly formatTokens: (a: any, b: number) => [number, number, number];
    readonly formatTokensMut: (a: any, b: number) => [number, number, number];
    readonly formatUsfm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly isKnownMarker: (a: number, b: number) => number;
    readonly lintCodeMeta: () => [number, number, number];
    readonly lintCodes: () => [number, number, number];
    readonly lintTokens: (a: any, b: number) => [number, number, number];
    readonly lintUsfm: (a: number, b: number, c: number) => [number, number, number];
    readonly markerCatalog: () => number;
    readonly markerInfo: (a: number, b: number) => [number, number, number];
    readonly parse: (a: number, b: number) => number;
    readonly parsedusfm_applyTokenFix: (a: number, b: any) => [number, number, number];
    readonly parsedusfm_cst: (a: number) => [number, number, number];
    readonly parsedusfm_diff: (a: number, b: number, c: number) => [number, number, number];
    readonly parsedusfm_diffByChapter: (a: number, b: number, c: number) => [number, number, number];
    readonly parsedusfm_format: (a: number, b: number) => [number, number, number, number];
    readonly parsedusfm_lint: (a: number, b: number) => [number, number, number];
    readonly parsedusfm_revertDiffBlock: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly parsedusfm_toHtml: (a: number, b: number) => [number, number, number, number];
    readonly parsedusfm_toUsfm: (a: number) => [number, number];
    readonly parsedusfm_toUsj: (a: number) => [number, number, number];
    readonly parsedusfm_toUsx: (a: number) => [number, number, number, number];
    readonly parsedusfm_toVref: (a: number) => [number, number, number];
    readonly parsedusfm_tokens: (a: number) => [number, number, number];
    readonly revertDiffBlock: (a: any, b: any, c: number, d: number, e: number) => [number, number, number];
    readonly revertDiffBlocks: (a: any, b: any, c: any, d: number) => [number, number, number];
    readonly tokensToHtml: (a: any, b: number) => [number, number, number, number];
    readonly tokensToUsfm: (a: any) => [number, number, number, number];
    readonly usfmmarkercatalog_all: (a: number) => [number, number, number];
    readonly usfmmarkercatalog_contains: (a: number, b: number, c: number) => number;
    readonly usfmmarkercatalog_get: (a: number, b: number, c: number) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
