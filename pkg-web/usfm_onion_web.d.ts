/* tslint:disable */
/* eslint-disable */

export type Value =
| string
| number
| boolean
| null
| Value[]
| { [key: string]: Value };


export interface EditorTreeDocument {
    type: string;
    version: string;
    content: EditorTreeNode[];
}

export interface UsjDocument {
    type: string;
    version: string;
    content: UsjNode[];
    _lossless_roundtrip?: UsjRoundtrip;
}

export interface UsjRoundtrip {
    source: string;
    fingerprint: string;
}

export interface WebApplyRevertsByBlockIdRequest {
    diffBlockIds: string[];
    baselineTokens: WebFlatToken[];
    currentTokens: WebFlatToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebApplyTokenFixesRequest {
    tokens: WebFlatToken[];
    fixes: WebTokenFix[];
}

export interface WebBatchExecutionOptions {
    parallel?: boolean;
}

export interface WebBuildSidBlocksOptions {
    allowEmptySid?: boolean;
}

export interface WebBuildSidBlocksRequest {
    tokens: WebFlatToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebChapterDiffGroup {
    book: string;
    chapter: number;
    diffs: WebChapterTokenDiff[];
}

export interface WebChapterDiffReplacement {
    book: string;
    chapter: number;
    diffs: WebChapterTokenDiff[];
}

export interface WebChapterTokenDiff {
    blockId: string;
    semanticSid: string;
    status: string;
    original: WebSidBlock | null;
    current: WebSidBlock | null;
    originalText: string;
    currentText: string;
    originalTextOnly: string;
    currentTextOnly: string;
    isWhitespaceChange: boolean;
    isUsfmStructureChange: boolean;
    originalTokens: WebFlatToken[];
    currentTokens: WebFlatToken[];
    originalAlignment: WebTokenAlignment[];
    currentAlignment: WebTokenAlignment[];
    undoSide: string;
}

export interface WebContentRequest {
    source: string;
    sourceFormat: WebDocumentFormat;
    targetFormat: WebDocumentFormat;
}

export interface WebDiffChapterTokenStreamsRequest {
    baselineTokens: WebFlatToken[];
    currentTokens: WebFlatToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebDiffContentRequest {
    baselineSource: string;
    baselineFormat: WebDocumentFormat;
    currentSource: string;
    currentFormat: WebDocumentFormat;
    tokenView?: WebTokenViewOptions | null;
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebDiffSidBlocksRequest {
    baselineBlocks: WebSidBlock[];
    currentBlocks: WebSidBlock[];
}

export interface WebDiffTokensRequest {
    baselineTokens: WebFlatToken[];
    currentTokens: WebFlatToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebDiffUsfmRequest {
    baselineUsfm: string;
    currentUsfm: string;
    tokenView?: WebTokenViewOptions | null;
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebFlatToken {
    id: string;
    kind: string;
    span: WebSpan;
    sid: string | null;
    marker: string | null;
    text: string;
}

export interface WebFormatContentRequest {
    source: string;
    format: WebDocumentFormat;
    tokenOptions?: WebIntoTokensOptions | null;
    formatOptions?: WebFormatOptions | null;
}

export interface WebFormatContentsRequest {
    sources: string[];
    format: WebDocumentFormat;
    tokenOptions?: WebIntoTokensOptions | null;
    formatOptions?: WebFormatOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebFormatFlatTokenBatchesRequest {
    tokenBatches: WebFlatToken[][];
    formatOptions?: WebFormatOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebFormatFlatTokensRequest {
    tokens: WebFlatToken[];
    formatOptions?: WebFormatOptions | null;
}

export interface WebFormatOptions {
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
    insertStructuralLinebreaks?: boolean;
    collapseConsecutiveLinebreaks?: boolean;
    normalizeMarkerWhitespaceAtLineStart?: boolean;
}

export interface WebHtmlOptions {
    wrapRoot?: boolean;
    preferNativeElements?: boolean;
    noteMode?: WebHtmlNoteMode | null;
    callerStyle?: WebHtmlCallerStyle | null;
    callerScope?: WebHtmlCallerScope | null;
}

export interface WebIntoTokensBatchRequest {
    documents: WebParsedDocument[];
    tokenOptions?: WebIntoTokensOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebIntoTokensFromContentRequest {
    source: string;
    format: WebDocumentFormat;
    tokenOptions?: WebIntoTokensOptions | null;
}

export interface WebIntoTokensFromContentsRequest {
    sources: string[];
    format: WebDocumentFormat;
    tokenOptions?: WebIntoTokensOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebIntoTokensOptions {
    mergeHorizontalWhitespace?: boolean;
}

export interface WebIntoTokensRequest {
    document: WebParsedDocument;
    tokenOptions?: WebIntoTokensOptions | null;
}

export interface WebIntoUsxRequest {
    document: WebParsedDocument;
}

export interface WebLexSourcesRequest {
    sources: string[];
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebLintBatch {
    issues: WebLintIssue[];
}

export interface WebLintContentRequest {
    source: string;
    format: WebDocumentFormat;
    options?: WebLintOptions | null;
}

export interface WebLintContentsRequest {
    sources: string[];
    format: WebDocumentFormat;
    options?: WebLintOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebLintDocumentBatchRequest {
    documents: WebParsedDocument[];
    options?: WebLintOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebLintDocumentRequest {
    document: WebParsedDocument;
    options?: WebLintOptions | null;
}

export interface WebLintFlatTokenBatchesRequest {
    tokenBatches: WebFlatToken[][];
    options?: WebTokenLintOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebLintFlatTokensRequest {
    tokens: WebFlatToken[];
    options?: WebTokenLintOptions | null;
}

export interface WebLintIssue {
    code: string;
    severity: string;
    marker: string | null;
    message: string;
    span: WebSpan;
    relatedSpan: WebSpan | null;
    tokenId: string | null;
    relatedTokenId: string | null;
    sid: string | null;
    fix: WebTokenFix | null;
}

export interface WebLintOpResult {
    value: WebLintIssue[] | null;
    error: string | null;
}

export interface WebLintOptions {
    includeParseRecoveries?: boolean;
    tokenView?: WebTokenViewOptions | null;
    tokenRules?: WebTokenLintOptions | null;
}

export interface WebLintSuppression {
    code: string;
    spanStart: number;
    spanEnd: number;
}

export interface WebParseContentRequest {
    source: string;
    format: WebDocumentFormat;
}

export interface WebParseContentsRequest {
    sources: string[];
    format: WebDocumentFormat;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebParseRecovery {
    code: string;
    span: WebSpan;
    relatedSpan: WebSpan | null;
    payload: WebRecoveryPayload | null;
}

export interface WebParsedDocument {
    sourceUsfm: string;
    bookCode: string | null;
    recoveries: WebParseRecovery[];
}

export interface WebParsedOpResult {
    value: WebParsedDocument | null;
    error: string | null;
}

export interface WebProjectContentRequest {
    source: string;
    format: WebDocumentFormat;
    options?: WebProjectUsfmOptions | null;
}

export interface WebProjectContentsRequest {
    sources: string[];
    format: WebDocumentFormat;
    options?: WebProjectUsfmOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebProjectDocumentRequest {
    document: WebParsedDocument;
    options?: WebProjectUsfmOptions | null;
}

export interface WebProjectUsfmOptions {
    tokenOptions?: WebIntoTokensOptions | null;
    lintOptions?: WebLintOptions | null;
}

export interface WebProjectedOpResult {
    value: WebProjectedUsfmDocument | null;
    error: string | null;
}

export interface WebProjectedUsfmDocument {
    tokens: WebFlatToken[];
    editorTree: EditorTreeDocument;
    lintIssues: WebLintIssue[] | null;
}

export interface WebReplaceChapterDiffsInMapRequest {
    groups: WebChapterDiffGroup[];
    book: string;
    chapter: number;
    diffs: WebChapterTokenDiff[];
}

export interface WebReplaceManyChapterDiffsInMapRequest {
    groups: WebChapterDiffGroup[];
    replacements: WebChapterDiffReplacement[];
}

export interface WebRevertDiffBlockRequest {
    blockId: string;
    baselineTokens: WebFlatToken[];
    currentTokens: WebFlatToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebScanResult {
    tokens: WebScanToken[];
}

export interface WebScanToken {
    kind: string;
    span: WebSpan;
    text: string;
}

export interface WebSidBlock {
    blockId: string;
    semanticSid: string;
    start: number;
    endExclusive: number;
    prevBlockId: string | null;
    textFull: string;
}

export interface WebSidBlockDiff {
    blockId: string;
    semanticSid: string;
    status: string;
    original: WebSidBlock | null;
    current: WebSidBlock | null;
    originalText: string;
    currentText: string;
    originalTextOnly: string;
    currentTextOnly: string;
    isWhitespaceChange: boolean;
    isUsfmStructureChange: boolean;
}

export interface WebSkippedTokenTransform {
    kind: string;
    label: string;
    targetTokenId: string | null;
    reason: string;
}

export interface WebSpan {
    start: number;
    end: number;
}

export interface WebStringOpResult {
    value: string | null;
    error: string | null;
}

export interface WebTokenAlignment {
    change: string;
    counterpartIndex: number | null;
}

export interface WebTokenBatch {
    tokens: WebFlatToken[];
}

export interface WebTokenLintOptions {
    disabledRules?: string[];
    suppressions?: WebLintSuppression[];
    allowImplicitChapterContentVerse?: boolean;
}

export interface WebTokenTemplate {
    kind: string;
    text: string;
    marker: string | null;
    sid: string | null;
}

export interface WebTokenTransformChange {
    kind: string;
    label: string;
    targetTokenId: string | null;
}

export interface WebTokenTransformResult {
    tokens: WebFlatToken[];
    appliedChanges: WebTokenTransformChange[];
    skippedChanges: WebSkippedTokenTransform[];
}

export interface WebTokenViewOptions {
    whitespacePolicy?: WebWhitespacePolicy | null;
}

export interface WebTokensOpResult {
    value: WebFlatToken[] | null;
    error: string | null;
}

export interface WebTransformOpResult {
    value: WebTokenTransformResult | null;
    error: string | null;
}

export interface WebVrefEntry {
    reference: string;
    text: string;
}

export type EditorTreeElement = ({ type: "book" } & { marker: string; code: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "chapter" } & { marker: string; number: string } & Record<string, Value>) | ({ type: "verse" } & { marker: string; number: string } & Record<string, Value>) | ({ type: "para" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "char" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "note" } & { marker: string; caller: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "ms" } & { marker: string } & Record<string, Value>) | ({ type: "figure" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "sidebar" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "periph" } & { content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "table" } & { content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "table:row" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "table:cell" } & { marker: string; align: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "ref" } & { content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "unknown" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | ({ type: "unmatched" } & { marker: string; content?: EditorTreeNode[] } & Record<string, Value>) | { type: "optbreak" } | { type: "linebreak" };

export type EditorTreeNode = string | EditorTreeElement;

export type UsjElement = ({ type: "book" } & { marker: string; code: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "chapter" } & { marker: string; number: string } & Record<string, Value>) | ({ type: "verse" } & { marker: string; number: string } & Record<string, Value>) | ({ type: "para" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "char" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "note" } & { marker: string; caller: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "ms" } & { marker: string } & Record<string, Value>) | ({ type: "figure" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "sidebar" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "periph" } & { content?: UsjNode[] } & Record<string, Value>) | ({ type: "table" } & { content?: UsjNode[] } & Record<string, Value>) | ({ type: "table:row" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "table:cell" } & { marker: string; align: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "ref" } & { content?: UsjNode[] } & Record<string, Value>) | ({ type: "unknown" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | ({ type: "unmatched" } & { marker: string; content?: UsjNode[] } & Record<string, Value>) | { type: "optbreak" };

export type UsjNode = string | UsjElement;

export type WebDocumentFormat = "usfm" | "usj" | "usx";

export type WebHtmlCallerScope = "documentSequential" | "verseSequential";

export type WebHtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";

export type WebHtmlNoteMode = "extracted" | "inline";

export type WebRecoveryPayload = { type: "marker"; marker: string } | { type: "close"; open: string; close: string };

export type WebTokenFix = { type: "replaceToken"; label: string; targetTokenId: string; replacements: WebTokenTemplate[] } | { type: "insertAfter"; label: string; targetTokenId: string; insert: WebTokenTemplate[] };

export type WebWhitespacePolicy = "preserve" | "mergeToVisible";


export function applyRevertByBlockId(request: WebRevertDiffBlockRequest): WebFlatToken[];

export function applyRevertsByBlockId(request: WebApplyRevertsByBlockIdRequest): WebFlatToken[];

export function applyTokenFixes(request: WebApplyTokenFixesRequest): WebTokenTransformResult;

export function buildSidBlocks(request: WebBuildSidBlocksRequest): WebSidBlock[];

export function convertContent(request: WebContentRequest): string;

export function diffChapterTokenStreams(request: WebDiffChapterTokenStreamsRequest): WebChapterTokenDiff[];

export function diffContent(request: WebDiffContentRequest): WebChapterTokenDiff[];

export function diffSidBlocks(request: WebDiffSidBlocksRequest): WebSidBlockDiff[];

export function diffTokens(request: WebDiffTokensRequest): WebChapterTokenDiff[];

export function diffUsfm(request: WebDiffUsfmRequest): WebChapterTokenDiff[];

export function diffUsfmByChapter(request: WebDiffUsfmRequest): WebChapterDiffGroup[];

export function diffUsfmSources(request: WebDiffUsfmRequest): WebChapterTokenDiff[];

export function diffUsfmSourcesByChapter(request: WebDiffUsfmRequest): WebChapterDiffGroup[];

export function flattenDiffMap(groups: WebChapterDiffGroup[]): WebChapterTokenDiff[];

export function formatContent(request: WebFormatContentRequest): WebTokenTransformResult;

export function formatContents(request: WebFormatContentsRequest): WebTransformOpResult[];

export function formatFlatTokenBatches(request: WebFormatFlatTokenBatchesRequest): WebTokenTransformResult[];

export function formatFlatTokens(request: WebFormatFlatTokensRequest): WebTokenTransformResult;

export function fromUsj(document: UsjDocument): string;

export function fromUsx(content: string): string;

export function intoEditorTree(document: WebParsedDocument): EditorTreeDocument;

export function intoHtml(document: WebParsedDocument, options?: WebHtmlOptions | null): string;

export function intoTokens(request: WebIntoTokensRequest): WebFlatToken[];

export function intoTokensBatch(request: WebIntoTokensBatchRequest): WebTokenBatch[];

export function intoTokensFromContent(request: WebIntoTokensFromContentRequest): WebFlatToken[];

export function intoTokensFromContents(request: WebIntoTokensFromContentsRequest): WebTokensOpResult[];

export function intoUsfmFromTokens(tokens: WebFlatToken[]): string;

export function intoUsj(document: WebParsedDocument): UsjDocument;

export function intoUsjFromTokens(tokens: WebFlatToken[]): UsjDocument;

export function intoUsjLossless(document: WebParsedDocument): UsjDocument;

export function intoUsjLosslessFromTokens(tokens: WebFlatToken[]): UsjDocument;

export function intoUsx(request: WebIntoUsxRequest): string;

export function intoUsxFromTokens(tokens: WebFlatToken[]): string;

export function intoUsxLossless(request: WebIntoUsxRequest): string;

export function intoUsxLosslessFromTokens(tokens: WebFlatToken[]): string;

export function intoVref(document: WebParsedDocument): WebVrefEntry[];

export function intoVrefFromTokens(tokens: WebFlatToken[]): WebVrefEntry[];

export function lexSources(request: WebLexSourcesRequest): WebScanResult[];

export function lintContent(request: WebLintContentRequest): WebLintIssue[];

export function lintContents(request: WebLintContentsRequest): WebLintOpResult[];

export function lintDocument(request: WebLintDocumentRequest): WebLintIssue[];

export function lintDocumentBatch(request: WebLintDocumentBatchRequest): WebLintBatch[];

export function lintFlatTokenBatches(request: WebLintFlatTokenBatchesRequest): WebLintBatch[];

export function lintFlatTokens(request: WebLintFlatTokensRequest): WebLintIssue[];

export function packageVersion(): string;

export function parseContent(request: WebParseContentRequest): WebParsedDocument;

export function parseContents(request: WebParseContentsRequest): WebParsedOpResult[];

export function parseSources(request: WebLexSourcesRequest): WebParsedDocument[];

export function projectContent(request: WebProjectContentRequest): WebProjectedUsfmDocument;

export function projectContents(request: WebProjectContentsRequest): WebProjectedOpResult[];

export function projectDocument(request: WebProjectDocumentRequest): WebProjectedUsfmDocument;

export function projectUsfmBatch(request: WebProjectContentsRequest): WebProjectedUsfmDocument[];

export function pushWhitespace(tokens: WebFlatToken[]): WebFlatToken[];

export function replaceChapterDiffsInMap(request: WebReplaceChapterDiffsInMapRequest): WebChapterDiffGroup[];

export function replaceManyChapterDiffsInMap(request: WebReplaceManyChapterDiffsInMapRequest): WebChapterDiffGroup[];

export function revertDiffBlock(request: WebRevertDiffBlockRequest): WebFlatToken[];

export function revertDiffBlocks(request: WebApplyRevertsByBlockIdRequest): WebFlatToken[];

export function usfmToHtml(content: string, options?: WebHtmlOptions | null): string;

export function usfmToUsj(content: string): string;

export function usfmToUsx(content: string): string;

export function usjToUsfm(content: string): string;

export function usxToUsfm(content: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly applyRevertByBlockId: (a: any) => [number, number];
    readonly applyRevertsByBlockId: (a: any) => [number, number];
    readonly applyTokenFixes: (a: any) => any;
    readonly buildSidBlocks: (a: any) => [number, number];
    readonly convertContent: (a: any) => [number, number, number, number];
    readonly diffChapterTokenStreams: (a: any) => [number, number];
    readonly diffContent: (a: any) => [number, number, number, number];
    readonly diffSidBlocks: (a: any) => [number, number];
    readonly diffTokens: (a: any) => [number, number];
    readonly diffUsfm: (a: any) => [number, number];
    readonly diffUsfmByChapter: (a: any) => [number, number];
    readonly diffUsfmSources: (a: any) => [number, number];
    readonly diffUsfmSourcesByChapter: (a: any) => [number, number];
    readonly flattenDiffMap: (a: number, b: number) => [number, number];
    readonly formatContent: (a: any) => [number, number, number];
    readonly formatContents: (a: any) => [number, number];
    readonly formatFlatTokenBatches: (a: any) => [number, number];
    readonly formatFlatTokens: (a: any) => any;
    readonly fromUsj: (a: any) => [number, number, number, number];
    readonly fromUsx: (a: number, b: number) => [number, number, number, number];
    readonly intoEditorTree: (a: any) => any;
    readonly intoHtml: (a: any, b: number) => [number, number];
    readonly intoTokens: (a: any) => [number, number];
    readonly intoTokensBatch: (a: any) => [number, number];
    readonly intoTokensFromContent: (a: any) => [number, number, number, number];
    readonly intoTokensFromContents: (a: any) => [number, number];
    readonly intoUsfmFromTokens: (a: number, b: number) => [number, number];
    readonly intoUsj: (a: any) => any;
    readonly intoUsjFromTokens: (a: number, b: number) => any;
    readonly intoUsjLossless: (a: any) => any;
    readonly intoUsjLosslessFromTokens: (a: number, b: number) => any;
    readonly intoUsx: (a: any) => [number, number, number, number];
    readonly intoUsxFromTokens: (a: number, b: number) => [number, number, number, number];
    readonly intoUsxLossless: (a: any) => [number, number, number, number];
    readonly intoUsxLosslessFromTokens: (a: number, b: number) => [number, number, number, number];
    readonly intoVref: (a: any) => [number, number];
    readonly intoVrefFromTokens: (a: number, b: number) => [number, number];
    readonly lexSources: (a: any) => [number, number];
    readonly lintContent: (a: any) => [number, number, number, number];
    readonly lintContents: (a: any) => [number, number];
    readonly lintDocument: (a: any) => [number, number];
    readonly lintDocumentBatch: (a: any) => [number, number];
    readonly lintFlatTokenBatches: (a: any) => [number, number];
    readonly lintFlatTokens: (a: any) => [number, number];
    readonly packageVersion: () => [number, number];
    readonly parseContent: (a: any) => [number, number, number];
    readonly parseContents: (a: any) => [number, number];
    readonly parseSources: (a: any) => [number, number];
    readonly projectContent: (a: any) => [number, number, number];
    readonly projectContents: (a: any) => [number, number];
    readonly projectDocument: (a: any) => any;
    readonly projectUsfmBatch: (a: any) => [number, number];
    readonly pushWhitespace: (a: number, b: number) => [number, number];
    readonly replaceChapterDiffsInMap: (a: any) => [number, number];
    readonly replaceManyChapterDiffsInMap: (a: any) => [number, number];
    readonly revertDiffBlock: (a: any) => [number, number];
    readonly revertDiffBlocks: (a: any) => [number, number];
    readonly usfmToHtml: (a: number, b: number, c: number) => [number, number];
    readonly usfmToUsj: (a: number, b: number) => [number, number, number, number];
    readonly usfmToUsx: (a: number, b: number) => [number, number, number, number];
    readonly usjToUsfm: (a: number, b: number) => [number, number, number, number];
    readonly usxToUsfm: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
