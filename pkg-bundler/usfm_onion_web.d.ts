/* tslint:disable */
/* eslint-disable */

export function intoAst(document: WebParsedDocument): AstDocument;
export function usfmToAst(content: string): AstDocument;
export function usjToAst(content: string): AstDocument;
export function usxToAst(content: string): AstDocument;
export function tokensToAst(tokens: WebToken[]): AstDocument;
export function astToTokens(document: AstDocument): WebToken[];
export function astToUsj(document: AstDocument): UsjDocument;
export function astToUsx(document: AstDocument): string;
export function astToHtml(document: AstDocument, options?: WebHtmlOptions | null): string;
export function astToVref(document: AstDocument): WebVrefEntry[];
export function intoUsj(document: WebParsedDocument): UsjDocument;
export function tokensToUsj(tokens: WebToken[]): UsjDocument;
export function usfmToUsj(content: string): UsjDocument;
export function fromUsj(document: UsjDocument): string;
export function cstToken(document: CstDocument, tokenRef: CstTokenRef): WebToken;
export function cstTokenText(document: CstDocument, tokenRef: CstTokenRef): string;
export function cstTokenValue(document: CstDocument, tokenRef: CstTokenRef): string;



export type MaybeString = string | null | undefined;
export type Span = WebSpan;
export type BatchExecutionOptions = WebBatchExecutionOptions;
export type IntoTokensOptions = WebIntoTokensOptions;
export type TokenViewOptions = WebTokenViewOptions;
export type LintSuppression = WebLintSuppression;
export type TokenLintOptions = WebTokenLintOptions;
export type LintOptions = WebLintOptions;
export type ProjectUsfmOptions = WebProjectUsfmOptions;
export type FormatOptions = WebFormatOptions;
export type BuildSidBlocksOptions = WebBuildSidBlocksOptions;
export type HtmlOptions = WebHtmlOptions;
export type Token = Omit<WebToken, "sid" | "marker"> & { sid?: MaybeString; marker?: MaybeString };
export type TokenTemplate = Omit<WebTokenTemplate, "sid" | "marker"> & { sid?: MaybeString; marker?: MaybeString };
export type TokenFix = WebTokenFix;
export type LintIssue = Omit<WebLintIssue, "marker" | "tokenId" | "relatedTokenId" | "sid"> & {
    marker?: MaybeString;
    tokenId?: MaybeString;
    relatedTokenId?: MaybeString;
    sid?: MaybeString;
};
export type ProjectedUsfmDocument = WebProjectedUsfmDocument;
export type TokenTransformChange = WebTokenTransformChange;
export type SkippedTokenTransform = Omit<WebSkippedTokenTransform, "targetTokenId"> & {
    targetTokenId?: MaybeString;
};
export type TokenTransformResult = WebTokenTransformResult;
export type Diff = WebChapterTokenDiff;
export type DiffTokenAlignment = WebTokenAlignment;
export type SidBlock = WebSidBlock;
export type SidBlockDiff = WebSidBlockDiff;
export type VrefEntry = WebVrefEntry;
export type ParseRecovery = WebParseRecovery;
export type ParsedDocument = WebParsedDocument;
export type DocumentFormat = WebDocumentFormat;
export type WhitespacePolicy = WebWhitespacePolicy;
export type HtmlNoteMode = WebHtmlNoteMode;
export type HtmlCallerStyle = WebHtmlCallerStyle;
export type HtmlCallerScope = WebHtmlCallerScope;
export type MarkerCategory = WebMarkerCategory;
export type MarkerNoteFamily = WebMarkerNoteFamily;
export type MarkerNoteSubkind = WebMarkerNoteSubkind;
export type MarkerInlineContext = WebMarkerInlineContext;
export type MarkerInfo = WebMarkerInfo;
export type AstDocument = {
    type: string;
    version: string;
    content: AstNode[];
};
export type AstNode = AstElement;
export type AstElement =
| ({ type: "text"; value: string } & Record<string, Value>)
| ({ type: "book"; marker: string; code: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "chapter"; marker: string; number: string } & Record<string, Value>)
| ({ type: "verse"; marker: string; number: string } & Record<string, Value>)
| ({ type: "para"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "char"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "note"; marker: string; caller: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "ms"; marker: string } & Record<string, Value>)
| ({ type: "figure"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "sidebar"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "periph"; content?: AstNode[] } & Record<string, Value>)
| ({ type: "table"; content?: AstNode[] } & Record<string, Value>)
| ({ type: "table:row"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "table:cell"; marker: string; align: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "ref"; content?: AstNode[] } & Record<string, Value>)
| ({ type: "unknown"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "unmatched"; marker: string; content?: AstNode[] } & Record<string, Value>)
| ({ type: "optbreak" } & Record<string, Value>)
| ({ type: "linebreak"; value: string } & Record<string, Value>);
export type UsjDocument = {
    type: string;
    version: string;
    content: UsjNode[];
};
export type UsjNode = string | UsjElement;
export type UsjElement =
| ({ type: "book"; marker: string; code: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "chapter"; marker: string; number: string } & Record<string, Value>)
| ({ type: "verse"; marker: string; number: string } & Record<string, Value>)
| ({ type: "para"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "char"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "note"; marker: string; caller: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "ms"; marker: string } & Record<string, Value>)
| ({ type: "figure"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "sidebar"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "periph"; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "table"; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "table:row"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "table:cell"; marker: string; align: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "ref"; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "unknown"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "unmatched"; marker: string; content?: UsjNode[] } & Record<string, Value>)
| ({ type: "optbreak" } & Record<string, Value>);
export type CstDocument = {
    type: string;
    sourceUsfm: string;
    bookCode?: MaybeString;
    recoveries: ParseRecovery[];
    tokens: Token[];
    content: CstNode[];
};
export type CstTokenRef = {
    token: number;
    span: Span;
};
export type CstContainerKind =
| "book"
| "paragraph"
| "character"
| "note"
| "figure"
| "sidebar"
| "periph"
| "tableRow"
| "tableCell"
| "header"
| "meta"
| "unknown";
export type CstLeafKind =
| "text"
| "whitespace"
| "newline"
| "optBreak"
| "attributes";
export type CstContainer = {
    type: "container";
    kind: CstContainerKind;
    marker: string;
    markerToken?: CstTokenRef | null;
    closeToken?: CstTokenRef | null;
    specialToken?: CstTokenRef | null;
    attributeTokens?: CstTokenRef[];
    children?: CstNode[];
};
export type CstChapter = {
    type: "chapter";
    markerToken: CstTokenRef;
    numberToken?: CstTokenRef | null;
};
export type CstVerse = {
    type: "verse";
    markerToken: CstTokenRef;
    numberToken?: CstTokenRef | null;
};
export type CstMilestone = {
    type: "milestone";
    marker: string;
    markerToken: CstTokenRef;
    attributeTokens?: CstTokenRef[];
    endToken?: CstTokenRef | null;
    closed: boolean;
};
export type CstLeaf = {
    type: "leaf";
    kind: CstLeafKind;
    token: CstTokenRef;
};
export type CstElement =
| CstContainer
| CstChapter
| CstVerse
| CstMilestone
| CstLeaf;
export type CstNode = CstElement;



export type Value =
| string
| number
| boolean
| null
| Value[]
| { [key: string]: Value };


export interface WebApplyRevertsByBlockIdRequest {
    diffBlockIds: string[];
    baselineTokens: WebToken[];
    currentTokens: WebToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebApplyTokenFixesRequest {
    tokens: WebToken[];
    fixes: WebTokenFix[];
}

export interface WebBatchExecutionOptions {
    parallel?: boolean;
}

export interface WebBuildSidBlocksOptions {
    allowEmptySid?: boolean;
}

export interface WebBuildSidBlocksRequest {
    tokens: WebToken[];
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
    originalTokens: WebToken[];
    currentTokens: WebToken[];
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
    baselineTokens: WebToken[];
    currentTokens: WebToken[];
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
    baselineTokens: WebToken[];
    currentTokens: WebToken[];
    buildOptions?: WebBuildSidBlocksOptions | null;
}

export interface WebDiffUsfmRequest {
    baselineUsfm: string;
    currentUsfm: string;
    tokenView?: WebTokenViewOptions | null;
    buildOptions?: WebBuildSidBlocksOptions | null;
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
    removeEmptyParagraphs?: boolean;
    insertStructuralLinebreaks?: boolean;
    collapseConsecutiveLinebreaks?: boolean;
    normalizeMarkerWhitespaceAtLineStart?: boolean;
}

export interface WebFormatTokenBatchesRequest {
    tokenBatches: WebToken[][];
    formatOptions?: WebFormatOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebFormatTokensRequest {
    tokens: WebToken[];
    formatOptions?: WebFormatOptions | null;
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

export interface WebLintIssue {
    code: string;
    severity: string;
    marker: string | null;
    message: string;
    messageParams: Record<string, string>;
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
    sid: string;
}

export interface WebLintTokenBatchesRequest {
    tokenBatches: WebToken[][];
    options?: WebTokenLintOptions | null;
    batchOptions?: WebBatchExecutionOptions | null;
}

export interface WebLintTokensRequest {
    tokens: WebToken[];
    options?: WebTokenLintOptions | null;
}

export interface WebMarkerInfo {
    marker: string;
    canonical: string | null;
    known: boolean;
    deprecated: boolean;
    category: WebMarkerCategory;
    noteFamily: WebMarkerNoteFamily | null;
    noteSubkind: WebMarkerNoteSubkind | null;
    inlineContext: WebMarkerInlineContext | null;
    defaultAttribute: string | null;
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
    tokens: WebToken[];
    ast: AstDocument;
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
    baselineTokens: WebToken[];
    currentTokens: WebToken[];
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
    code: string;
    label: string;
    labelParams: Record<string, string>;
    reasonCode: string;
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

export interface WebToken {
    id: string;
    kind: string;
    span: WebSpan;
    sid: string | null;
    marker: string | null;
    text: string;
}

export interface WebTokenAlignment {
    change: string;
    counterpartIndex: number | null;
}

export interface WebTokenBatch {
    tokens: WebToken[];
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
    code: string;
    label: string;
    labelParams: Record<string, string>;
    targetTokenId: string | null;
}

export interface WebTokenTransformResult {
    tokens: WebToken[];
    appliedChanges: WebTokenTransformChange[];
    skippedChanges: WebSkippedTokenTransform[];
}

export interface WebTokenViewOptions {
    whitespacePolicy?: WebWhitespacePolicy | null;
}

export interface WebTokensOpResult {
    value: WebToken[] | null;
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

export type WebDocumentFormat = "usfm" | "usj" | "usx";

export type WebHtmlCallerScope = "documentSequential" | "verseSequential";

export type WebHtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";

export type WebHtmlNoteMode = "extracted" | "inline";

export type WebMarkerCategory = "document" | "paragraph" | "character" | "noteContainer" | "noteSubmarker" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "figure" | "sidebarStart" | "sidebarEnd" | "periph" | "meta" | "tableRow" | "tableCell" | "header" | "unknown";

export type WebMarkerInlineContext = "para" | "section" | "list" | "table";

export type WebMarkerNoteFamily = "footnote" | "crossReference";

export type WebMarkerNoteSubkind = "structural" | "structuralKeepsNestedCharsOpen";

export type WebRecoveryPayload = { type: "marker"; marker: string } | { type: "close"; open: string; close: string };

export type WebTokenFix = { type: "replaceToken"; code: string; label: string; label_params: Record<string, string>; targetTokenId: string; replacements: WebTokenTemplate[] } | { type: "deleteToken"; code: string; label: string; label_params: Record<string, string>; targetTokenId: string } | { type: "insertAfter"; code: string; label: string; label_params: Record<string, string>; targetTokenId: string; insert: WebTokenTemplate[] };

export type WebTokenVariant = { type: "newline"; id: string; span: WebSpan; sid: string | null; text: string } | { type: "optBreak"; id: string; span: WebSpan; sid: string | null; text: string } | { type: "marker"; id: string; span: WebSpan; sid: string | null; marker: string; text: string } | { type: "endMarker"; id: string; span: WebSpan; sid: string | null; marker: string; text: string } | { type: "milestone"; id: string; span: WebSpan; sid: string | null; marker: string; text: string } | { type: "milestoneEnd"; id: string; span: WebSpan; sid: string | null; marker: string | null; text: string } | { type: "attributes"; id: string; span: WebSpan; sid: string | null; text: string } | { type: "bookCode"; id: string; span: WebSpan; sid: string | null; text: string } | { type: "number"; id: string; span: WebSpan; sid: string | null; text: string } | { type: "text"; id: string; span: WebSpan; sid: string | null; text: string };

export type WebWhitespacePolicy = "mergeToVisible";


export function allMarkers(): string[];

export function applyRevertByBlockId(request: WebRevertDiffBlockRequest): WebToken[];

export function applyRevertsByBlockId(request: WebApplyRevertsByBlockIdRequest): WebToken[];

export function applyTokenFixes(request: WebApplyTokenFixesRequest): WebTokenTransformResult;

export function buildSidBlocks(request: WebBuildSidBlocksRequest): WebSidBlock[];

export function characterMarkers(): string[];

export function classifyTokens(tokens: WebToken[]): WebTokenVariant[];

export function convertContent(request: WebContentRequest): string;

export function diffChapterTokenStreams(request: WebDiffChapterTokenStreamsRequest): WebChapterTokenDiff[];

/**
 * Parse both sources, project canonical flat tokens, then diff.
 *
 * If you already have canonical flat tokens, prefer `diffTokens(...)`.
 */
export function diffContent(request: WebDiffContentRequest): WebChapterTokenDiff[];

export function diffSidBlocks(request: WebDiffSidBlocksRequest): WebSidBlockDiff[];

/**
 * Diff canonical flat token streams without reparsing source content.
 */
export function diffTokens(request: WebDiffTokensRequest): WebChapterTokenDiff[];

export function diffUsfm(request: WebDiffUsfmRequest): WebChapterTokenDiff[];

export function diffUsfmByChapter(request: WebDiffUsfmRequest): WebChapterDiffGroup[];

export function diffUsfmSources(request: WebDiffUsfmRequest): WebChapterTokenDiff[];

export function diffUsfmSourcesByChapter(request: WebDiffUsfmRequest): WebChapterDiffGroup[];

export function flattenDiffMap(groups: WebChapterDiffGroup[]): WebChapterTokenDiff[];

/**
 * Parse content, project tokens, then run the formatter.
 *
 * If you already have canonical flat tokens, prefer `formatFlatTokens(...)`.
 */
export function formatContent(request: WebFormatContentRequest): WebTokenTransformResult;

export function formatContents(request: WebFormatContentsRequest): WebTransformOpResult[];

/**
 * Format canonical flat tokens without reparsing source content.
 */
export function formatFlatTokens(request: WebFormatTokensRequest): WebTokenTransformResult;

/**
 * Format batches of canonical flat token streams without reparsing source content.
 */
export function formatTokenBatches(request: WebFormatTokenBatchesRequest): WebTokenTransformResult[];

export function fromUsx(content: string): string;

export function intoHtml(document: WebParsedDocument, options?: WebHtmlOptions | null): string;

/**
 * Project a previously parsed document into canonical flat tokens.
 */
export function intoTokens(request: WebIntoTokensRequest): WebToken[];

export function intoTokensBatch(request: WebIntoTokensBatchRequest): WebTokenBatch[];

/**
 * Parse raw content and immediately project flat tokens.
 *
 * Prefer `parseContent(...)` plus `intoTokens(...)` when you will also lint,
 * format, diff, or project other views from the same source.
 */
export function intoTokensFromContent(request: WebIntoTokensFromContentRequest): WebToken[];

export function intoTokensFromContents(request: WebIntoTokensFromContentsRequest): WebTokensOpResult[];

export function intoUsx(request: WebIntoUsxRequest): string;

export function intoVref(document: WebParsedDocument): WebVrefEntry[];

export function isBodyParagraphMarker(marker: string): boolean;

export function isCharacterMarker(marker: string): boolean;

export function isDocumentMarker(marker: string): boolean;

export function isKnownMarker(marker: string): boolean;

export function isNoteContainer(marker: string): boolean;

export function isNoteSubmarker(marker: string): boolean;

export function isParagraphMarker(marker: string): boolean;

export function isPoetryMarker(marker: string): boolean;

export function isRegularCharacterMarker(marker: string): boolean;

export function lexSources(request: WebLexSourcesRequest): WebScanResult[];

export function lintCodes(): string[];

/**
 * Parse content, project tokens, then lint.
 *
 * If you already have canonical flat tokens, prefer `lintFlatTokens(...)`.
 */
export function lintContent(request: WebLintContentRequest): WebLintIssue[];

export function lintContents(request: WebLintContentsRequest): WebLintOpResult[];

export function lintDocument(request: WebLintDocumentRequest): WebLintIssue[];

export function lintDocumentBatch(request: WebLintDocumentBatchRequest): WebLintBatch[];

/**
 * Lint canonical flat tokens without reparsing source content.
 */
export function lintFlatTokens(request: WebLintTokensRequest): WebLintIssue[];

/**
 * Lint batches of canonical flat token streams without reparsing source content.
 */
export function lintTokenBatches(request: WebLintTokenBatchesRequest): WebLintBatch[];

export function markerInfo(marker: string): WebMarkerInfo;

export function noteMarkerFamily(marker: string): WebMarkerNoteFamily | undefined;

export function noteMarkers(): string[];

export function noteSubmarkers(): string[];

export function packageVersion(): string;

export function paragraphMarkers(): string[];

/**
 * Parse raw content once and keep the returned document if you plan to project
 * multiple views from it.
 */
export function parseContent(request: WebParseContentRequest): WebParsedDocument;

export function parseContents(request: WebParseContentsRequest): WebParsedOpResult[];

export function parseSources(request: WebLexSourcesRequest): WebParsedDocument[];

export function projectContent(request: WebProjectContentRequest): WebProjectedUsfmDocument;

export function projectContents(request: WebProjectContentsRequest): WebProjectedOpResult[];

export function projectDocument(request: WebProjectDocumentRequest): WebProjectedUsfmDocument;

export function projectUsfmBatch(request: WebProjectContentsRequest): WebProjectedUsfmDocument[];

export function pushWhitespace(tokens: WebToken[]): WebToken[];

export function replaceChapterDiffsInMap(request: WebReplaceChapterDiffsInMapRequest): WebChapterDiffGroup[];

export function replaceManyChapterDiffsInMap(request: WebReplaceManyChapterDiffsInMapRequest): WebChapterDiffGroup[];

export function revertDiffBlock(request: WebRevertDiffBlockRequest): WebToken[];

export function revertDiffBlocks(request: WebApplyRevertsByBlockIdRequest): WebToken[];

export function tokenFixCodes(): string[];

export function tokenTransformChangeCodes(): string[];

export function tokenTransformSkipReasonCodes(): string[];

export function tokensToHtml(tokens: WebToken[], options?: WebHtmlOptions | null): string;

export function tokensToUsfm(tokens: WebToken[]): string;

export function tokensToUsx(tokens: WebToken[]): string;

export function tokensToVref(tokens: WebToken[]): WebVrefEntry[];

export function usfmToHtml(content: string, options?: WebHtmlOptions | null): string;

export function usfmToTokenVariants(content: string): WebTokenVariant[];

export function usfmToTokens(content: string, token_options?: WebIntoTokensOptions | null): WebToken[];

export function usfmToUsx(content: string): string;

export function usfmToVref(content: string): WebVrefEntry[];

export function usjToTokens(content: string, token_options?: WebIntoTokensOptions | null): WebToken[];

export function usjToUsfm(content: string): string;

export function usxToTokens(content: string, token_options?: WebIntoTokensOptions | null): WebToken[];

export function usxToUsfm(content: string): string;
