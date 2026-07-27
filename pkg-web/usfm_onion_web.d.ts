/* tslint:disable */
/* eslint-disable */

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


/**
 * Argument payload a marker\'s opening form consumes right after its name:
 * `\"bookCode\"` for `\\id`, `\"numberRange\"` for the chapter/verse family
 * (`\\c`, `\\cp`, `\\ca`, `\\v`, `\\vp`, `\\va`). Shares one table with the
 * lexer (`marker_defs::marker_payload`), so catalog and tokenization cannot
 * drift.
 */
export type MarkerPayload = "bookCode" | "numberRange";

/**
 * Diff result grouped by book and chapter: `{ \"GEN\": { 1: [...], 2: [...] } }`.
 */
export type DiffsByChapterMap = Record<string, Record<number, DiffSkeleton>>;

/**
 * Lossless plain-text projection of one verse plus its segment map back to
 * source / token coordinates.
 */
export interface VerseProjection {
    text: string;
    segments: Segment[];
}

/**
 * One in-scope text token\'s contribution to a verse projection, with both
 * resolution anchors: `sourceSpan` (bytes into source, for raw buffers) and
 * `tokenId` (== the editor\'s DOM `data-id`). `textSpan` is UTF-16 into `text`.
 */
export interface Segment {
    tokenId: string;
    sourceSpan: Span;
    textSpan: Utf16Span;
}

/**
 * Optional trailing argument on the wasm diff entry points. Omitting it (or
 * omitting `textDiff`) is `\"none\"` — today\'s behavior, computing nothing.
 */
export interface DiffOptions {
    textDiff?: TextDiffMode;
}

/**
 * Staged decisions for [`wasm_merge_diff_blocks`]: `Record<string,
 * MergeSide>` plus the default applied to any unit not present in the map.
 */
export interface MergeRequest {
    decisions: Record<string, MergeSide>;
    defaultSide: MergeSide;
}

/**
 * UTF-16 code-unit offsets into a `VerseProjection.text`. Deliberately a
 * distinct type from `Span` (byte offsets into the source) so the unit is
 * unmistakable on the wire — JS/DOM consumers index `text` in UTF-16.
 */
export interface Utf16Span {
    start: number;
    end: number;
}

/**
 * Verse-reference map: `{ \"GEN 1:1\": \"...\", \"GEN 1:2\": \"...\", ... }`.
 */
export type VrefMap = Record<string, string>;

/**
 * What the caller is linting. Gates the document-level rules: they run only
 * for `\"front\"` and `\"book\"`, never a bare `{ chapter }` slice. TS shape:
 * `\"front\" | { chapter: number } | \"book\"`.
 */
export type LintScope = "front" | { chapter: number } | "book";

/**
 * `sid` -> lossless verse projection. Same key set as `VrefMap`; the
 * difference is losslessness plus the segment map.
 */
export type VrefIndex = Record<string, VerseProjection>;

export interface Anchor {
    unitId: string;
    sid: string;
}

export interface AttributeItem {
    span: Span;
    text: string;
    key: string;
    value: string;
    isDefault?: boolean;
}

export interface CoveredBy {
    unitId: string;
    sid: string;
    side: CoveredSide;
}

export interface CstDocument {
    tokens: Token[];
    roots: CstNode[];
}

export interface CstNode {
    tokenIndex: number;
    children: CstNode[];
}

export interface DecisionUnit {
    id: string;
    kind: DecisionUnitKind;
    status: DecisionStatus;
    baselineSid?: string;
    currentSid?: string;
    baselineTokens: Token[];
    currentTokens: Token[];
    displaced: boolean;
    relabeled: boolean;
    dupContext: DupContext;
    coveredBy?: CoveredBy;
    isWhitespaceChange: boolean;
    isUsfmStructureChange: boolean;
    textDiff?: UnitTextDiff;
}

export interface DiffSkeleton {
    slots: Slot[];
    units: DecisionUnit[];
}

export interface DupContext {
    baselineCount: number;
    currentCount: number;
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
    /**
     * Required — no default. A defaulted scope would let a chapter-grain
     * caller silently get whole-book id-behavior.
     */
    scope: LintScope;
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
    payload?: MarkerPayload;
    paragraphCategory?: ParagraphCategory;
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

export interface Slot {
    unitId: string;
    role: SlotRole;
    after?: Anchor;
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

export interface TextDiffRun {
    text: string;
    kind: TextDiffRunKind;
}

export interface Token {
    id: string;
    kind: TokenKind;
    source: string;
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
    /**
     * Verbatim `|...` attribute-list slice (native `MarkerAttrs.attribute_source`),
     * carried across the wire so a passed-through token stays byte-lossless
     * through `tokens_to_usfm_reconstruct`. `None` when the source had no
     * attribute list, or when an editor authored/edited the attributes
     * itself — see [`SerializableToken`] for the \"touch an attribute, drop
     * its verbatim\" rule.
     */
    attributeSource?: string;
}

export interface TokenTemplate {
    kind: TokenKind;
    text: string;
    marker?: string;
    sid?: string;
}

export interface UnitTextDiff {
    /**
     * Kinds: `Unchanged` | `Removed`.
     */
    baseline: TextDiffRun[];
    /**
     * Kinds: `Unchanged` | `Added`.
     */
    current: TextDiffRun[];
}

export interface VrefOptions {
    trim?: boolean | null;
}

export type BlockBehavior = "none" | "paragraph" | "tableRow" | "tableCell" | "sidebarStart" | "sidebarEnd";

export type ClosingBehavior = "none" | "requiredExplicit" | "optionalExplicitUntilNoteEnd" | "selfClosingMilestone";

export type CoveredSide = "baseline" | "current";

export type DecisionStatus = "unchanged" | "modified" | "added" | "deleted" | "moved";

export type DecisionUnitKind = "shared" | "added" | "deleted" | "coalesced";

export type HtmlCallerScope = "documentSequential" | "verseSequential";

export type HtmlCallerStyle = "numeric" | "alphaLower" | "alphaUpper" | "romanLower" | "romanUpper" | "source";

export type HtmlNoteMode = "extracted" | "inline";

export type InlineContext = "para" | "section" | "list" | "table";

export type LintCategory = "document" | "structure" | "context" | "numbering";

export type LintCode = "missing-id-marker" | "duplicate-id-marker" | "id-marker-not-at-file-start" | "empty-paragraph" | "missing-chapter-number" | "missing-verse-number" | "verse-is-empty" | "unknown-token" | "unknown-marker" | "unknown-close-marker" | "content-before-first-chapter" | "verse-outside-explicit-paragraph" | "note-submarker-outside-note" | "metadata-outside-target" | "marker-not-valid-in-context" | "missing-milestone-self-close" | "stray-close-marker" | "misnested-close-marker" | "implicitly-closed-marker" | "unclosed-marker" | "duplicate-chapter-number" | "duplicate-verse-number" | "invalid-number-range" | "number-range-not-preceded-by-marker-expecting-number" | "missing-whitespace-before-marker" | "missing-horizontal-whitespace-after-marker-name" | "missing-tag-end-delimiter-after-marker" | "missing-content-space-after-close-marker" | "verse-in-section-or-other-paragraph" | "content-after-blank-marker" | "invalid-book-code" | "book-code-not-uppercase";

export type LintIssueType = "usfm" | "content";

export type LintSeverity = "error" | "warning";

export type MarkerCategory = "document" | "paragraph" | "character" | "noteContainer" | "noteSubmarker" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "figure" | "sidebarStart" | "sidebarEnd" | "periph" | "meta" | "tableRow" | "tableCell" | "header" | "unknown";

export type MarkerDefKind = "paragraph" | "character" | "note" | "chapter" | "verse" | "milestone" | "figure" | "sidebar" | "periph" | "meta" | "tableRow" | "tableCell" | "header";

export type MarkerFamily = "footnote" | "crossReference" | "sectionParagraph" | "listParagraph" | "tableCell" | "milestone" | "sidebar";

export type MarkerFamilyRole = "canonical" | "numberedVariant" | "nestedVariant" | "milestoneStart" | "milestoneEnd" | "alias";

export type MarkerKind = "paragraph" | "note" | "character" | "header" | "chapter" | "verse" | "milestoneStart" | "milestoneEnd" | "sidebarStart" | "sidebarEnd" | "figure" | "meta" | "periph" | "tableRow" | "tableCell" | "unknown";

export type MergeSide = "baseline" | "current";

export type NoteFamily = "footnote" | "crossReference";

export type NoteSubkind = "structural" | "structuralKeepsNestedCharsOpen";

export type NumberRangeKind = "single" | "range" | "sequence" | "sequenceWithRange";

export type ParagraphCategory = "identification" | "introduction" | "title" | "section" | "body" | "poetry" | "list" | "table" | "peripheral" | "other";

export type SlotRole = "shared" | "baselineOnly" | "currentOnly" | "pairBaseline" | "pairCurrent";

export type SpecContext = "scripture" | "bookIdentification" | "bookHeaders" | "bookTitles" | "bookIntroduction" | "bookIntroductionEndTitles" | "bookChapterLabel" | "chapterContent" | "peripheral" | "peripheralContent" | "peripheralDivision" | "chapter" | "verse" | "section" | "para" | "list" | "table" | "sidebar" | "footnote" | "crossReference";

export type StructuralScopeKind = "unknown" | "header" | "block" | "note" | "character" | "milestone" | "chapter" | "verse" | "tableRow" | "tableCell" | "sidebar" | "periph" | "meta";

export type TextDiffMode = "none" | "words" | "chars";

export type TextDiffRunKind = "unchanged" | "added" | "removed";

export type TokenFix = { type: "replaceToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; replacements: TokenTemplate[] } | { type: "deleteToken"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string } | { type: "insertAfter"; code: string; label: string; labelParams: Record<string, string>; targetTokenId: string; insert: TokenTemplate[] };

export type TokenKind = "newline" | "optBreak" | "marker" | "endMarker" | "milestone" | "milestoneEnd" | "bookCode" | "number" | "text";


export class ParsedUsfm {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    applyTokenFix(fix: TokenFix): Token[];
    cst(): CstDocument;
    diff(other: ParsedUsfm, options?: DiffOptions | null): DiffSkeleton;
    diffByChapter(other: ParsedUsfm, options?: DiffOptions | null): DiffsByChapterMap;
    format(options?: FormatOptions | null): string;
    lint(options: LintOptions): LintResult;
    revertDiffBlock(current: ParsedUsfm, block_id: string): Token[];
    toHtml(options?: HtmlOptions | null): string;
    toUsfm(): string;
    toUsj(): any;
    toUsx(): string;
    toVref(options?: VrefOptions | null): VrefMap;
    tokens(): Token[];
    vrefIndex(): VrefIndex;
}

export class UsfmMarkerCatalog {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    all(): MarkerInfo[];
    contains(marker: string): boolean;
    get(marker: string): MarkerInfo | undefined;
}

export function applyTokenFix(tokens: Token[], fix: TokenFix): Token[];

export function diffTokens(left: Token[], right: Token[], options?: DiffOptions | null): DiffSkeleton;

export function diffUsfm(left: string, right: string, options?: DiffOptions | null): DiffSkeleton;

export function diffUsfmByChapter(left: string, right: string, options?: DiffOptions | null): DiffsByChapterMap;

export function formatRuleMeta(): FormatRuleMeta[];

export function formatRules(): string[];

export function formatTokens(tokens: Token[], options?: FormatOptions | null): FormatResult;

export function formatTokensMut(tokens: Token[], options?: FormatOptions | null): Token[];

export function formatUsfm(source: string, options?: FormatOptions | null): string;

export function isKnownMarker(marker: string): boolean;

export function lintCodeMeta(): LintCodeMeta[];

export function lintCodes(): LintCode[];

export function lintTokens(tokens: Token[], options: LintOptions): LintResult;

export function lintUsfm(source: string, options: LintOptions): LintResult;

export function markerCatalog(): UsfmMarkerCatalog;

export function markerInfo(marker: string): MarkerInfo;

export function mergeDiffBlocks(baseline: Token[], current: Token[], request: MergeRequest): Token[];

export function normalizeTokenSids(tokens: Token[], book_code: string): Token[];

export function parse(source: string): ParsedUsfm;

export function revertDiffBlock(baseline: Token[], current: Token[], block_id: string): Token[];

export function tokensToHtml(tokens: Token[], options?: HtmlOptions | null): string;

export function tokensToUsfm(tokens: Token[]): string;

/**
 * Build the vref index from an existing token stream (the editor's live
 * path) — same rehydration as `lintTokens`, no reparse. Segment ids match
 * the tokens passed in, so they line up with the editor's DOM `data-id`s.
 */
export function vrefIndexTokens(tokens: Token[]): VrefIndex;

export function vrefIndexUsfm(source: string): VrefIndex;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_parsedusfm_free: (a: number, b: number) => void;
    readonly __wbg_usfmmarkercatalog_free: (a: number, b: number) => void;
    readonly applyTokenFix: (a: number, b: number, c: number, d: number) => void;
    readonly diffTokens: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly diffUsfm: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly diffUsfmByChapter: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly formatRuleMeta: (a: number) => void;
    readonly formatRules: (a: number) => void;
    readonly formatTokens: (a: number, b: number, c: number) => number;
    readonly formatTokensMut: (a: number, b: number, c: number, d: number) => void;
    readonly formatUsfm: (a: number, b: number, c: number, d: number) => void;
    readonly isKnownMarker: (a: number, b: number) => number;
    readonly lintCodeMeta: (a: number) => void;
    readonly lintCodes: (a: number) => void;
    readonly lintTokens: (a: number, b: number, c: number) => number;
    readonly lintUsfm: (a: number, b: number, c: number) => number;
    readonly markerInfo: (a: number, b: number) => number;
    readonly mergeDiffBlocks: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly normalizeTokenSids: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly parse: (a: number, b: number) => number;
    readonly parsedusfm_applyTokenFix: (a: number, b: number, c: number) => void;
    readonly parsedusfm_cst: (a: number) => number;
    readonly parsedusfm_diff: (a: number, b: number, c: number) => number;
    readonly parsedusfm_diffByChapter: (a: number, b: number, c: number) => number;
    readonly parsedusfm_format: (a: number, b: number, c: number) => void;
    readonly parsedusfm_lint: (a: number, b: number) => number;
    readonly parsedusfm_revertDiffBlock: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly parsedusfm_toHtml: (a: number, b: number, c: number) => void;
    readonly parsedusfm_toUsfm: (a: number, b: number) => void;
    readonly parsedusfm_toUsj: (a: number, b: number) => void;
    readonly parsedusfm_toUsx: (a: number, b: number) => void;
    readonly parsedusfm_toVref: (a: number, b: number) => number;
    readonly parsedusfm_tokens: (a: number, b: number) => void;
    readonly parsedusfm_vrefIndex: (a: number) => number;
    readonly revertDiffBlock: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly tokensToHtml: (a: number, b: number, c: number, d: number) => void;
    readonly tokensToUsfm: (a: number, b: number, c: number) => void;
    readonly usfmmarkercatalog_all: (a: number, b: number) => void;
    readonly usfmmarkercatalog_contains: (a: number, b: number, c: number) => number;
    readonly usfmmarkercatalog_get: (a: number, b: number, c: number) => number;
    readonly vrefIndexTokens: (a: number, b: number) => number;
    readonly vrefIndexUsfm: (a: number, b: number) => number;
    readonly markerCatalog: () => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
