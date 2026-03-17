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
export function markerCatalog(): UsfmMarkerCatalog;
export function markerInfo(marker: string): MarkerInfo;
export function isKnownMarker(marker: string): boolean;
export function lintCodes(): LintCode[];
export function lintCodeMeta(): LintCodeMeta[];
export function formatRules(): FormatRule[];
export function formatRuleMeta(): FormatRuleMeta[];



export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_parsedusfm_free: (a: number, b: number) => void;
    readonly __wbg_parsedusfmbatch_free: (a: number, b: number) => void;
    readonly __wbg_usfmmarkercatalog_free: (a: number, b: number) => void;
    readonly diffTokens: (a: any, b: any, c: number) => [number, number, number];
    readonly diffUsfm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly diffUsfmByChapter: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly formatRuleMeta: () => [number, number, number];
    readonly formatRules: () => [number, number, number];
    readonly formatTokenBatch: (a: any, b: number) => [number, number, number];
    readonly formatTokens: (a: any, b: number) => [number, number, number];
    readonly formatTokensMut: (a: any, b: number) => [number, number, number];
    readonly formatUsfm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly isKnownMarker: (a: number, b: number) => number;
    readonly lintCodeMeta: () => [number, number, number];
    readonly lintCodes: () => [number, number, number];
    readonly lintTokenBatch: (a: any, b: number) => [number, number, number];
    readonly lintTokens: (a: any, b: number) => [number, number, number];
    readonly lintUsfm: (a: number, b: number, c: number) => [number, number, number];
    readonly markerInfo: (a: number, b: number) => [number, number, number];
    readonly parse: (a: number, b: number) => number;
    readonly parseBatch: (a: any) => [number, number, number];
    readonly parsedusfm_cst: (a: number) => [number, number, number];
    readonly parsedusfm_diff: (a: number, b: number, c: number) => [number, number, number];
    readonly parsedusfm_diffByChapter: (a: number, b: number, c: number) => [number, number, number];
    readonly parsedusfm_format: (a: number, b: number) => [number, number, number, number];
    readonly parsedusfm_lint: (a: number, b: number) => [number, number, number];
    readonly parsedusfm_to_html: (a: number, b: number) => [number, number, number, number];
    readonly parsedusfm_to_usfm: (a: number) => [number, number];
    readonly parsedusfm_to_usj: (a: number) => [number, number, number];
    readonly parsedusfm_to_usx: (a: number) => [number, number, number, number];
    readonly parsedusfm_to_vref: (a: number) => [number, number, number];
    readonly parsedusfm_tokens: (a: number) => [number, number, number];
    readonly parsedusfmbatch_format: (a: number, b: number) => [number, number, number];
    readonly parsedusfmbatch_items: (a: number) => any;
    readonly parsedusfmbatch_lint: (a: number, b: number) => [number, number, number];
    readonly parsedusfmbatch_to_html: (a: number, b: number) => [number, number, number];
    readonly parsedusfmbatch_to_usfm: (a: number) => [number, number, number];
    readonly parsedusfmbatch_to_usj: (a: number) => [number, number, number];
    readonly parsedusfmbatch_to_usx: (a: number) => [number, number, number];
    readonly parsedusfmbatch_to_vref: (a: number) => [number, number, number];
    readonly parsedusfmbatch_tokens: (a: number) => [number, number, number];
    readonly tokensToHtml: (a: any, b: number) => [number, number, number, number];
    readonly tokensToUsfm: (a: any) => [number, number, number, number];
    readonly usfmmarkercatalog_all: (a: number) => [number, number, number];
    readonly usfmmarkercatalog_contains: (a: number, b: number, c: number) => number;
    readonly usfmmarkercatalog_get: (a: number, b: number, c: number) => [number, number, number];
    readonly markerCatalog: () => number;
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
