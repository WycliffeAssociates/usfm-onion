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


export interface AttributeItem {
    span: Span;
    text: string;
    key: string;
    value: string;
}

export interface BuildSidBlocksOptions {
    allowEmptySid?: boolean | null;
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

export interface CstDocument {
    tokens: Token[];
    roots: CstNode[];
}

export interface CstNode {
    tokenIndex: number;
    children: CstNode[];
}

export interface FormatOptions {
    recoverMalformedMarkers?: boolean | null;
    collapseWhitespaceInText?: boolean | null;
    ensureInlineSeparators?: boolean | null;
    removeDuplicateVerseNumbers?: boolean | null;
    normalizeSpacingAfterParagraphMarkers?: boolean | null;
    removeUnwantedLinebreaks?: boolean | null;
    bridgeConsecutiveVerseMarkers?: boolean | null;
    removeOrphanEmptyVerseBeforeContentfulVerse?: boolean | null;
    removeBridgeVerseEnumerators?: boolean | null;
    moveChapterLabelAfterChapterMarker?: boolean | null;
    insertDefaultParagraphAfterChapterIntro?: boolean | null;
    removeEmptyParagraphs?: boolean | null;
    insertStructuralLinebreaks?: boolean | null;
    collapseConsecutiveLinebreaks?: boolean | null;
    normalizeMarkerWhitespaceAtLineStart?: boolean | null;
}

export interface FormatResult {
    tokens: Token[];
    usfm: string;
}

export interface FormatRuleMeta {
    code: string;
    labelKey: string;
}

export interface HtmlOptions {
    wrapRoot?: boolean;
    preferNativeElements?: boolean | null;
    noteMode?: HtmlNoteMode | null;
    callerStyle?: HtmlCallerStyle | null;
    callerScope?: HtmlCallerScope | null;
}

export interface LintCodeMeta {
    code: LintCode;
    category: LintCategory;
    severity: LintSeverity;
    issueType: LintIssueType;
}

export interface LintIssue {
    code: LintCode;
    category: LintCategory;
    severity: LintSeverity;
    issueType: LintIssueType;
    template: string;
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

export interface LintOptions {
    enabledCodes?: LintCode[] | null;
    disabledCodes?: LintCode[];
    suppressed?: LintSuppression[];
    allowImplicitChapterContentVerse?: boolean;
}

export interface LintResult {
    issues: LintIssue[];
    summary: LintSummary;
}

export interface LintSummary {
    byCategory: Record<LintCategory, number>;
    bySeverity: Record<LintSeverity, number>;
    byIssueType: Record<LintIssueType, number>;
    totalCount: number;
    suppressedCount: number;
}

export interface LintSuppression {
    code: LintCode;
    sid: string;
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
    noteFamily?: NoteFamily;
    noteSubkind?: NoteSubkind;
    inlineContext?: InlineContext;
    defaultAttribute?: string;
    contexts: SpecContext[];
    blockBehavior?: BlockBehavior;
    closingBehavior?: ClosingBehavior;
    source?: string;
}

export interface MarkerMetadata {
    canonical?: string;
    kind?: MarkerDefKind;
    family?: MarkerFamily;
}

export interface NumberInfo {
    start: number;
    end?: number;
    kind: NumberRangeKind;
}

export interface SidBlock {
    blockId: string;
    semanticSid: string;
    start: number;
    endExclusive: number;
    prevBlockId?: string;
    textFull: string;
}

export interface Span {
    start: number;
    end: number;
}

export interface StructuralMarkerInfo {
    scopeKind: StructuralScopeKind;
    inlineContext?: InlineContext;
    noteContext?: SpecContext;
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

export interface TokenAlignment {
    change: DiffTokenChange;
    counterpartIndex?: number;
}

export interface TokenTemplate {
    kind: TokenKind;
    text: string;
    marker?: string;
    sid?: string;
}

export type BlockBehavior = "none" | "paragraph" | "tableRow" | "tableCell" | "sidebarStart" | "sidebarEnd";

export type ClosingBehavior = "none" | "requiredExplicit" | "optionalExplicitUntilNoteEnd" | "selfClosingMilestone";

export type DiffStatus = "added" | "deleted" | "modified" | "unchanged";

export type DiffTokenChange = "unchanged" | "added" | "deleted" | "modified";

export type DiffUndoSide = "original" | "current";

export type HtmlCallerScope = "documentSequential" | "verseSequential";

export type HtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";

export type HtmlNoteMode = "extracted" | "inline";

export type InlineContext = "para" | "section" | "list" | "table";

export type LintCategory = "document" | "structure" | "context" | "numbering";

export type LintCode = "missing-id-marker" | "duplicate-id-marker" | "id-marker-not-at-file-start" | "empty-paragraph" | "missing-chapter-number" | "missing-verse-number" | "verse-is-empty" | "unknown-token" | "unknown-marker" | "unknown-close-marker" | "content-before-first-chapter" | "verse-outside-explicit-paragraph" | "note-submarker-outside-note" | "metadata-outside-target" | "marker-not-valid-in-context" | "missing-milestone-self-close" | "stray-close-marker" | "misnested-close-marker" | "implicitly-closed-marker" | "unclosed-marker" | "duplicate-chapter-number" | "chapter-expected-increase-by-one" | "inconsistent-chapter-label" | "duplicate-verse-number" | "verse-expected-increase-by-one" | "invalid-number-range" | "number-range-not-preceded-by-marker-expecting-number" | "missing-whitespace-before-marker" | "missing-horizontal-whitespace-after-marker-name" | "missing-tag-end-delimiter-after-marker" | "excess-whitespace-around-marker" | "excess-whitespace-in-content" | "missing-content-space-after-close-marker" | "verse-in-section-or-other-paragraph";

export type LintIssueType = "usfm" | "content";

export type LintSeverity = "error" | "warning";

export type MarkerCategory = "document" | "paragraph" | "character" | "noteContainer" | "noteSubmarker" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "figure" | "sidebarStart" | "sidebarEnd" | "periph" | "meta" | "tableRow" | "tableCell" | "header" | "unknown";

export type MarkerDefKind = "paragraph" | "character" | "note" | "chapter" | "verse" | "milestone" | "figure" | "sidebar" | "periph" | "meta" | "tableRow" | "tableCell" | "header";

export type MarkerFamily = "footnote" | "crossReference" | "sectionParagraph" | "listParagraph" | "tableCell" | "milestone" | "sidebar";

export type MarkerFamilyRole = "canonical" | "numberedVariant" | "nestedVariant" | "milestoneStart" | "milestoneEnd" | "alias";

export type MarkerKind = "paragraph" | "note" | "character" | "header" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "sidebarStart" | "sidebarEnd" | "figure" | "meta" | "periph" | "tableRow" | "tableCell" | "unknown";

export type NoteFamily = "footnote" | "crossReference";

export type NoteSubkind = "structural" | "structuralKeepsNestedCharsOpen";

export type NumberRangeKind = "single" | "range" | "sequence" | "sequenceWithRange";

export type SpecContext = "scripture" | "bookIdentification" | "bookHeaders" | "bookTitles" | "bookIntroduction" | "bookIntroductionEndTitles" | "bookChapterLabel" | "chapterContent" | "peripheral" | "peripheralContent" | "peripheralDivision" | "chapter" | "verse" | "section" | "para" | "list" | "table" | "sidebar" | "footnote" | "crossReference";

export type StructuralScopeKind = "unknown" | "header" | "block" | "note" | "character" | "milestone" | "chapter" | "verse" | "tableRow" | "tableCell" | "sidebar" | "periph" | "meta";

export type TokenFix = { type: "replaceToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; replacements: TokenTemplate[] } | { type: "deleteToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string } | { type: "insertAfter"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; insert: TokenTemplate[] };

export type TokenKind = "newline" | "optBreak" | "marker" | "endMarker" | "milestone" | "milestoneEnd" | "bookCode" | "number" | "text";


export function applyTokenFix(tokens: Token[], fix: TokenFix): Token[];

export function diffTokens(left: Token[], right: Token[], options?: BuildSidBlocksOptions | null): ChapterTokenDiff[];

export function diffUsfm(left: string, right: string, options?: BuildSidBlocksOptions | null): ChapterTokenDiff[];

export function formatRuleMeta(): FormatRuleMeta[];

export function formatRules(): string[];

export function formatTokens(tokens: Token[], options?: FormatOptions | null): FormatResult;

export function formatTokensMut(tokens: Token[], options?: FormatOptions | null): Token[];

export function formatUsfm(source: string, options?: FormatOptions | null): string;

export function lintCodeMeta(): LintCodeMeta[];

export function lintCodes(): LintCode[];

export function lintTokens(tokens: Token[], options?: LintOptions | null): LintResult;

export function lintUsfm(source: string, options?: LintOptions | null): LintResult;

export function markerCatalog(): UsfmMarkerCatalog;

export function markerInfo(marker: string): MarkerInfo;

export function revertDiffBlock(baseline: Token[], current: Token[], block_id: string, options?: BuildSidBlocksOptions | null): Token[];

export function revertDiffBlocks(baseline: Token[], current: Token[], block_ids: string[], options?: BuildSidBlocksOptions | null): Token[];

export function tokensToHtml(tokens: Token[], options?: HtmlOptions | null): string;

export function tokensToUsfm(tokens: Token[]): string;
