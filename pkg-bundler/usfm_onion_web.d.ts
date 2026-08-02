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
 * A baseline comparison that cannot be answered.
 */
export type BaselineError = { kind: "scope"; error: ScopeError } | { kind: "missingBaseline"; books: string[] };

/**
 * A baseline that could not be recorded.
 */
export type SetBaselineError = { kind: "bookNotResident"; book: string } | { kind: "invalid"; error: IngestError };

/**
 * A chapter run\'s label, exactly as the source spells it.
 */
export type ChapterLabel = { kind: "frontMatter" } | { kind: "number"; label: string };

/**
 * A complete ordered corpus. Caller order is preserved exactly.
 */
export interface CorpusInput {
    books: BookInput[];
}

/**
 * A mutation refused before it touched resident state.
 */
export type IngestError = { kind: "duplicateBook"; book: string; sources: string[] } | { kind: "duplicateSourceKey"; source: string } | { kind: "duplicateTokenId"; book: string; id: string } | { kind: "chapterNotFound"; target: ChapterTarget } | { kind: "ambiguousChapter"; target: ChapterTarget; matches: number } | { kind: "replacementLabelMismatch"; target: ChapterTarget; found: ChapterLabel } | { kind: "invalidToken"; message: string };

/**
 * A packed corpus, ready to persist as `corpus.bin`, plus what the caller
 * needs to restore or re-publish it later.
 */
export interface PublishedCorpus {
    bytes: number[];
    snapshotId: string;
    /**
     * One entry per resident book, in corpus order -- not only the freshly
     * encoded ones, so a caller always has the complete bookkeeping set for
     * what this publication now contains.
     */
    books: PublishedBookInfo[];
}

/**
 * A patch that could not be looked up or applied.
 */
export type PatchError = { kind: "staleSnapshot"; expected: string; found: string } | { kind: "unknownPatch"; id: PatchId } | { kind: "invalidResult"; error: IngestError };

/**
 * A patch\'s snapshot-bound identity.
 */
export interface PatchId {
    /**
     * The corpus identity the patch was resolved against, as 16 hex digits.
     */
    snapshot: string;
    ordinal: number;
}

/**
 * A prepared format patch that could not be looked up or applied.
 */
export type FormatPatchError = { kind: "staleSnapshot"; expected: string; found: string } | { kind: "unknownPatch"; id: FormatPatchId } | { kind: "bookNotResident"; book: string } | { kind: "invalidResult"; error: IngestError };

/**
 * A prepared format patch\'s snapshot-bound identity.
 */
export interface FormatPatchId {
    snapshot: string;
    ordinal: number;
}

/**
 * A projection over one scope, or over every resident book in corpus order.
 *
 * Ordered pairs for the `all` case rather than an object keyed by source key:
 * corpus order is a contract, and an object\'s key enumeration is not.
 */
export type ScopedOutput<T> = { kind: "single"; value: T } | { kind: "all"; books: SourceOutput<T>[] };

/**
 * A read or projection selector over resident data.
 */
export type CorpusScope = { kind: "all" } | { kind: "book"; book: string } | { kind: "chapter"; target: ChapterTarget };

/**
 * A scope that does not resolve against the resident corpus.
 */
export type ScopeError = { kind: "bookNotFound"; book: string } | { kind: "chapterNotFound"; target: ChapterTarget } | { kind: "ambiguousChapter"; target: ChapterTarget; matches: number };

/**
 * A scope that does not resolve, on the way to preparing a format patch.
 */
export type FormatError = { kind: "scope"; error: ScopeError };

/**
 * A verb\'s outcome: the value it produced, or the typed reason it was refused.
 *
 * Tagged on a string rather than a boolean, matching the crate\'s existing packed
 * outcome type: a string tag is what makes this a real discriminated union in
 * TypeScript, so a consumer that checks `status` has the other field narrowed for
 * it rather than asserted by hand.
 */
export type ApiResult<T, E> = { status: "ok"; value: T } | { status: "error"; error: E };

/**
 * A wasm-facing projection of [`crate::schema::SectionKind`] — only ever seen
 * nested inside [`PackedLayoutRefusal::DuplicateSection`].
 */
export type PackedSectionKind = "token" | "finding";

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
 * Every verse\'s lossless projection, as `[sid, projection]` pairs in the order
 * the document itself puts them — including deliberately out-of-order content
 * (`\\v 19` before `\\v 2`, chapter 10 before chapter 2).
 *
 * One authoritative sequence, not a sequence plus a lookup: an object keyed by
 * SID enumerates its keys sorted, which is the silent re-ordering this shape
 * exists to prevent, and carrying both would mean documenting that one of the two
 * views is meaningless. A consumer that wants O(1) lookup writes
 * `new Map(entries)` and owns that choice.
 */
export type VrefIndex = [string, VerseProjection][];

/**
 * How a book\'s exact bytes end their lines when they have to be re-emitted.
 */
export type LineEnding = "lf" | "crlf";

/**
 * Lossless plain-text projection of one verse plus its segment map back to
 * source / token coordinates.
 */
export interface VerseProjection {
    text: string;
    segments: Segment[];
}

/**
 * One book a warm seed would not fully accept.
 */
export interface PrimeRejection {
    book: string;
    reason: PrimeRejectReason;
}

/**
 * One book\'s attestation of what the Rust trust boundary checked.
 *
 * The three `u64` integrity values are 16-character lowercase hex strings.
 * They are an audit record, never an input to a check: a consumer of this
 * receipt has already been told the bytes are certified, and there is no JS
 * hash implementation to re-derive them with.
 */
export interface PackedBookReceipt {
    book: string;
    sourceLen: number;
    tokenCount: number;
    findingCount: number;
    /**
     * True when token ids are `{book}-{index}` and the id column/dictionary are
     * absent, so a materializer synthesizes them.
     */
    positionalIds: boolean;
    sourceHash: string;
    catalogStamp: string;
    snapshotId: string;
    /**
     * Descriptor-ordinal order: the packed marker-descriptor-index column
     * indexes straight into this list.
     */
    descriptors: PackedMarkerDescriptor[];
}

/**
 * One book\'s exact source, keyed the same way a corpus-grain restore or
 * re-publish needs to address it: by its resident book code *and* its own
 * source key (a packed container names the book but not the key a corpus
 * was originally addressed by).
 */
export interface PublishedCorpusSource {
    book: string;
    sourceKey: string;
    source: number[];
}

/**
 * One book\'s lint contribution.
 */
export interface BookLintSnapshot {
    sourceKey: string;
    book: string;
    sourceHash: string;
    tokenIdentity: string;
    findings: LintIssue[];
    summary: LintSummary;
}

/**
 * One book\'s own bookkeeping from a publish -- never the reuse-cache\'s
 * internal sections/bytes, which stay behind `PublicationCache`.
 *
 * `source` is present exactly when `encoded` is `true`: a reused (spliced)
 * book\'s source did not change and wire never saw it this round, so the
 * caller is expected to already hold it from whichever earlier publish
 * first reported `encoded: true` for that book -- the same asymmetry
 * `EncodedCorpus::sources` documents natively.
 */
export interface PublishedBookInfo {
    book: string;
    sourceHash: string;
    /**
     * `true` when this book was freshly re-encoded this call; `false` when
     * its previous publication\'s sections were spliced in unchanged.
     */
    encoded: boolean;
    source: string | null;
}

/**
 * One book\'s own source, for [`wasm_verify_published_corpus`] -- addressed by
 * book code alone, since verifying a corpus-wide container needs no source
 * key (that is a resident-corpus concept; the container itself never names
 * one).
 */
export interface PublishedCorpusSourceInput {
    book: string;
    source: number[];
}

/**
 * One book\'s packed bytes and the source they were bound to.
 */
export interface RestoreRecord {
    /**
     * The caller\'s own binding for where the book came from — normally a path.
     */
    path: string;
    packed: number[];
    /**
     * The exact bytes the container was bound to. Bytes rather than a string so a
     * host can hand over what it read from disk without a UTF-16 round trip.
     */
    source: number[];
}

/**
 * One book\'s receipt and findings out of a verified corpus container, in
 * the container\'s own (corpus) order.
 */
export interface PublishedCorpusBook {
    receipt: PackedBookReceipt;
    findings: LintIssue[];
}

/**
 * One book\'s value in an `all`-scoped projection.
 */
export interface SourceOutput<T> {
    sourceKey: string;
    book: string;
    value: T;
}

/**
 * One book\'s worth of resident input.
 */
export type BookInput = { kind: "usfm"; sourceKey: string; book: string; source: string } | { kind: "tokens"; sourceKey: string; book: string; tokens: Token[]; lineEnding: LineEnding };

/**
 * One chapter run\'s address.
 */
export interface ChapterTarget {
    book: string;
    label: ChapterLabel;
}

/**
 * One chapter run\'s replacement content.
 */
export type ChapterInput = { kind: "tokens"; tokens: Token[] };

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
 * One pulled scope\'s current tokens.
 */
export interface ScopeTokens {
    book: string;
    chapter?: ChapterLabel;
    tokens: Token[];
}

/**
 * One resident book\'s identity and derived stamps, in corpus order.
 */
export interface BookEntry {
    sourceKey: string;
    book: string;
    /**
     * 16 hex digits over the book\'s exact bytes.
     */
    sourceHash: string;
    /**
     * 16 hex digits over everything the book\'s tokens carry that its bytes do
     * not — the fact a consumer caching anything token-derived keys on.
     */
    tokenIdentity: string;
    lineEnding: LineEnding;
}

/**
 * One resolved fix, addressable and inspectable without applying it.
 */
export interface Patch {
    id: PatchId;
    book: string;
    /**
     * The target book\'s hash at resolution time — the second half of the
     * staleness check, since a book can be rewritten and restored inside a corpus
     * that hashes the same overall.
     */
    sourceHash: string;
    /**
     * The fix\'s own remedy code, which is not the finding\'s lint code.
     */
    code: string;
    label: string;
    labelParams: Record<string, string>;
    rows: PatchRow[];
}

/**
 * One token operation. `position` addresses the token stream of the snapshot the
 * owning patch is bound to, never the post-patch stream.
 */
export interface PatchRow {
    op: PatchOp;
    position: number;
    /**
     * Absent exactly for a delete, which places nothing.
     */
    template?: TokenTemplate;
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
 * The complete resident lint snapshot, in corpus order.
 *
 * Findings, not packed bytes: a finding\'s message is rendered by exactly one
 * renderer in one language, so findings cross this boundary already materialized.
 * Packed bytes are a separate question with a separate verb.
 */
export interface LintSnapshot {
    snapshotId: string;
    books: BookLintSnapshot[];
    summary: LintSummary;
}

/**
 * The frozen [`crate::error::DecodeError`] set as a tagged boundary value.
 *
 * Variant names and payloads are the same contract the Rust enum carries; the
 * tag exists so a TypeScript consumer can narrow instead of string-matching a
 * message.
 */
export type PackedDecodeError = { kind: "truncated" } | { kind: "badMagic" } | { kind: "unsupportedVersion"; found: number } | { kind: "unsupportedFlags"; found: number } | { kind: "invalidToc" } | { kind: "invalidSection" } | { kind: "invalidUtf8" } | { kind: "invalidDiscriminant" } | { kind: "offsetOverflow" } | { kind: "tooManySids"; found: number } | { kind: "checksumMismatch" } | { kind: "catalogMismatch" } | { kind: "sourceLengthMismatch" } | { kind: "sourceHashMismatch" };

/**
 * The frozen [`crate::error::EncodeError`] set as a tagged boundary value.
 *
 * Every variant here is a pathological-input safety net (a book past an
 * index ceiling, a synthetically-built token whose span does not bind, a
 * layout the writer\'s own reader would reject) rather than something a
 * normal publish hits — but the packed boundary\'s rule is refuse-typed-never-
 * panic regardless of how rare the refusal is.
 */
export type PackedEncodeError = { kind: "tooManySids"; book: string; found: number } | { kind: "unrepresentablePayload"; book: string; code: number } | { kind: "tooManyDescriptors"; book: string; found: number } | { kind: "unboundSpan"; book: string; tokenIdx: number } | { kind: "invalidSectionLayout"; book: string; reason: PackedLayoutRefusal } | { kind: "emptyFix"; book: string; code: number };

/**
 * The frozen [`crate::error::LayoutRefusal`] set as a tagged boundary value.
 */
export type PackedLayoutRefusal = { kind: "duplicateSection"; sectionKind: PackedSectionKind } | { kind: "orphanFindingSection" } | { kind: "duplicateField"; fieldId: number } | { kind: "fieldExtentMismatch"; fieldId: number } | { kind: "missingRequiredField"; fieldId: number } | { kind: "cachedSectionUnreadable" } | { kind: "cachedSectionMismatch" } | { kind: "cachedSectionStampMismatch" } | { kind: "positionalIdConflict"; fieldId: number } | { kind: "sectionTooLarge" } | { kind: "tooManyFields" } | { kind: "tooManySections" };

/**
 * The resident configuration.
 */
export interface BraidConfig {
    lint: LintOptions;
}

/**
 * The value every mutating verb returns, after it has already applied.
 *
 * `changed` is exact — what was rewritten, not what was inspected — so an empty
 * one means nothing needs re-pulling. Findings are absent by design: lint is an
 * explicit separate call.
 */
export interface MutationEffect {
    /**
     * The corpus identity *after* the mutation, as 16 hex digits.
     */
    snapshotId: string;
    changed: Scope[];
    removed: string[];
    /**
     * The full new book order, when the relative order of the books present both
     * before and after actually changed. A pure reorder rewrites no tokens, so it
     * appears here and nowhere else.
     */
    reordered?: string[];
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
 * What [`wasm_verify_packed_book`] reports for one record.
 *
 * A rejection is a *value*, not a thrown exception: it is an expected outcome
 * (the caller falls back to normal USFM ingest) and it carries the frozen
 * `DecodeError` variant rather than a message a consumer would have to parse.
 */
export type PackedBookOutcome = { status: "verified"; receipt: PackedBookReceipt; findings: LintIssue[] } | { status: "rejected"; error: PackedDecodeError };

/**
 * What [`wasm_verify_published_corpus`] reports.
 *
 * The corpus-grain twin of [`PackedBookOutcome`], for exactly the same
 * reason [`usfm_onion_wire::corpus_codec::verify_corpus`] exists beside
 * [`usfm_onion_wire::verify::verify_book`]: a whole `corpus.bin` container
 * is verified as one unit -- every book must have exactly one source
 * supplied, and findings that carry stamps must all carry the *same*
 * stamps -- rather than book by book.
 */
export type PublishedCorpusOutcome = { status: "verified"; snapshotId: string; books: PublishedCorpusBook[] } | { status: "rejected"; error: PackedDecodeError };

/**
 * What one mutation rewrote. `chapter` absent means the whole book.
 */
export interface Scope {
    book: string;
    chapter?: ChapterLabel;
}

/**
 * What one patch row does at its own position.
 */
export type PatchOp = "insert" | "replace" | "delete";

/**
 * What one warm restore installed, and what it would not take.
 *
 * A book can appear in both lists: residency and lint-priming are independent, so a
 * book whose cached findings were refused still seeds — with no lex or parse — and
 * is simply awaiting recompute.
 */
export interface RestoreReport {
    seeded: string[];
    rejected: PrimeRejection[];
}

/**
 * What the caller is linting. Gates the document-level rules: they run only
 * for `\"front\"` and `\"book\"`, never a bare `{ chapter }` slice. TS shape:
 * `\"front\" | { chapter: number } | \"book\"`.
 */
export type LintScope = "front" | { chapter: number } | "book";

/**
 * Whether preparing a format patch found anything to change.
 */
export type PatchPreparation = { kind: "unchanged" } | { kind: "ready"; id: FormatPatchId };

/**
 * Why a publish could not produce packed bytes.
 *
 * Every variant is a pathological-input safety net (see
 * [`crate::dto::PackedEncodeError`]\'s own doc comment) rather than something
 * a normal publish hits; surfaced as a typed refusal regardless, never a
 * panic.
 */
export type PublishError = { kind: "encode"; error: PackedEncodeError };

/**
 * Why a warm restore was refused outright.
 *
 * A refusal here is about the *call*: bytes that do not verify, or a corpus that
 * cannot be installed. A single book whose cached findings are not adoptable is not
 * a refusal — it seeds anyway and appears in the report\'s rejections.
 */
export type RestoreError = { kind: "decode"; error: PackedDecodeError } | { kind: "ingest"; error: IngestError };

/**
 * Why one book\'s cached lint contribution was not adopted.
 */
export type PrimeRejectReason = "bookNotResident" | "sourceHashMismatch" | "configFingerprintMismatch" | "engineStampMismatch" | "invalidPatch" | "sourceTokenMismatch";

/**
 *r" A baseline diff, or the reason it cannot be answered.
 */
export type DiffBaselineOutcome = ApiResult<ScopedOutput<DiffSkeleton>, BaselineError>;

/**
 *r" A mutation addressed by scope, or the reason the scope does not resolve.
 */
export type ScopedMutationOutcome = ApiResult<MutationEffect, ScopeError>;

/**
 *r" A mutation, or the reason the input was refused.
 */
export type MutationOutcome = ApiResult<MutationEffect, IngestError>;

/**
 *r" A packed corpus, or the reason it could not be produced.
 */
export type PublishOutcome = ApiResult<PublishedCorpus, PublishError>;

/**
 *r" A patch's projected tokens, or the reason it is not addressable.
 */
export type PatchPreviewOutcome = ApiResult<Token[], PatchError>;

/**
 *r" A prepared format patch, or the reason the scope does not resolve.
 */
export type FormatPreparationOutcome = ApiResult<PatchPreparation, FormatError>;

/**
 *r" A recorded baseline, or the reason it could not be.
 */
export type BaselineMutationOutcome = ApiResult<MutationEffect, SetBaselineError>;

/**
 *r" A scope's exact bytes, or the reason it does not resolve.
 */
export type UsfmOutcome = ApiResult<ScopedOutput<string>, ScopeError>;

/**
 *r" A scope's verse index, or the reason the scope does not resolve.
 */
export type VrefIndexOutcome = ApiResult<ScopedOutput<VrefIndex>, ScopeError>;

/**
 *r" A warm restore's report, or the reason the bytes were refused.
 */
export type RestoreOutcome = ApiResult<RestoreReport, RestoreError>;

/**
 *r" An applied format patch, or the reason it was refused.
 */
export type FormatMutationOutcome = ApiResult<MutationEffect, FormatPatchError>;

/**
 *r" An applied patch, or the reason it was refused.
 */
export type PatchMutationOutcome = ApiResult<MutationEffect, PatchError>;

/**
 *r" Hydrated tokens, or the reason a scope does not resolve.
 */
export type ScopeTokensOutcome = ApiResult<ScopeTokens[], ScopeError>;

/**
 *r" One book's chapter labels, or the reason the book does not resolve.
 */
export type ChapterLabelsOutcome = ApiResult<ChapterLabel[], ScopeError>;

/**
 *r" One patch, or the reason it is not addressable.
 */
export type PatchOutcome = ApiResult<Patch, PatchError>;

/**
 *r" Whether a scope differs from its baseline, or the reason it does not resolve.
 */
export type DirtyOutcome = ApiResult<boolean, ScopeError>;

export interface Anchor {
    unitId: string;
    sid: string;
}

export interface AttributeItem {
    /**
     * `None` for an attribute an editor synthesized or structurally edited —
     * same convention as `Token.span`, and never fabricated from some other
     * token\'s span, which would misreport a position this attribute never
     * actually occupied.
     */
    span?: Span;
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

export interface PackedMarkerDescriptor {
    name: string;
    nested: boolean;
    markerMetadata: MarkerMetadata;
    structural: StructuralMarkerInfo;
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
    /**
     * Bytes from the end of this token\'s own source to the start of its attribute
     * list, when the token remembers a placement.
     *
     * The other half of what `attribute_source` promises: an attribute list does
     * not sit next to the marker that owns it, and one placement rule cannot
     * express every real layout — an alignment list sits at the opener, a wordlist
     * list can sit past a nested closer. Carrying the verbatim text without its
     * position makes a round trip byte-lossless only for the layouts that happen
     * to match the fallback. `None` means no remembered placement, and an emitter
     * places the list at the marker\'s closer.
     */
    attributeOffset?: number;
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

export type LintCode = "missing-id-marker" | "duplicate-id-marker" | "id-marker-not-at-file-start" | "empty-paragraph" | "missing-chapter-number" | "missing-verse-number" | "verse-is-empty" | "unknown-token" | "unknown-marker" | "unknown-close-marker" | "content-before-first-chapter" | "verse-outside-explicit-paragraph" | "note-submarker-outside-note" | "metadata-outside-target" | "marker-not-valid-in-context" | "missing-milestone-self-close" | "stray-close-marker" | "misnested-close-marker" | "implicitly-closed-marker" | "unclosed-marker" | "duplicate-chapter-number" | "duplicate-verse-number" | "invalid-number-range" | "number-range-not-preceded-by-marker-expecting-number" | "missing-whitespace-before-marker" | "missing-horizontal-whitespace-after-marker-name" | "missing-tag-end-delimiter-after-marker" | "missing-content-space-after-close-marker" | "verse-in-section-or-other-paragraph" | "content-after-blank-marker" | "invalid-book-code" | "book-code-not-uppercase" | "book-id-mismatch";

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


/**
 * The resident corpus handle.
 */
export class Braid {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Applies a prepared format patch. All-or-nothing across every book it covers.
     */
    applyFormatPatch(id: FormatPatchId): FormatMutationOutcome;
    /**
     * Applies a patch as an ordinary mutation, atomically.
     */
    applyPatch(id: PatchId): PatchMutationOutcome;
    /**
     * Resident books with their derived stamps, in corpus order.
     */
    books(): BookEntry[];
    /**
     * Books whose findings are stale, in corpus order. Derived from authoritative
     * stamps rather than drained from a queue, so reading it twice is safe.
     */
    booksAwaitingLint(): string[];
    /**
     * One book's chapter-run labels in source order, duplicates included.
     */
    chapterLabels(book: string): ChapterLabelsOutcome;
    /**
     * Drops every resident book. Clearing an empty corpus is a no-op.
     */
    clear(): MutationEffect;
    /**
     * Forgets one book's baseline. Clearing an absent one is a no-op.
     */
    clearBaseline(book: string): MutationOutcome;
    /**
     * The resident diff against the baseline.
     */
    diffBaseline(scope: CorpusScope): DiffBaselineOutcome;
    /**
     * The corpus's content-derived identity, as a 16-digit hex string.
     *
     * Hex rather than a number because the value is 64 bits: a JS `number` cannot
     * hold it without silently rounding, and a `bigint` does not survive every
     * structured clone a worker boundary performs.
     */
    expectedSnapshotId(): string;
    /**
     * Whether a scope differs from its baseline, by exact serialized equality.
     */
    isDirty(scope: CorpusScope): DirtyOutcome;
    /**
     * Recomputes every book awaiting it and returns the complete snapshot.
     *
     * The only recompute verb, and always explicit: no mutation lints implicitly
     * and no effect carries findings. Exactly the stale books run rules — a clean
     * corpus runs none.
     */
    lint(): LintSnapshot;
    /**
     * Creates an empty handle bound to the application's own id minter.
     *
     * The minter is a JS callback returning a string, held for the life of the
     * handle: core never invents a token id, so every token a fix or format pass
     * synthesizes gets one from here. Speed, spelling, and collision resistance
     * are the application's trade — uniqueness is not assumed but enforced at the
     * residency boundary, where a collision is a typed rejection rather than a
     * corrupted book.
     *
     * Throws only for a programmer error: a minter that throws, or one that
     * returns something other than a string.
     */
    constructor(config: BraidConfig, minter: Function);
    /**
     * One patch by id, refusing a stale or unknown one.
     */
    patch(id: PatchId): PatchOutcome;
    /**
     * Every patch of the current snapshot, in corpus order and then each book's own
     * canonical finding order — which is what assigns each one its ordinal.
     *
     * A book awaiting recompute contributes none: its stored positions address the
     * token stream it held when its findings were computed.
     */
    patches(): Patch[];
    /**
     * Prepares a formatting pass over a scope without applying it.
     */
    prepareFormatPatch(scope: CorpusScope, options?: FormatOptions | null): FormatPreparationOutcome;
    /**
     * The token stream the patch would produce, without applying it.
     *
     * A preview is a projection and is never admitted to residency, so it mints
     * nothing: a surviving token carries the id it already had, and a token the fix
     * would synthesize carries none until an apply grants it one.
     */
    previewPatch(id: PatchId): PatchPreviewOutcome;
    /**
     * Publishes the resident corpus as one packed `corpus.bin` container.
     *
     * A thin projection of `PublicationCache::publish` (this handle's own
     * cache, so a repeat publish gets the adapter's whole point -- splice-
     * reuse of whatever did not change -- automatically): dirty books are
     * linted first (the adapter's own rule, via the `lint()` it runs
     * internally), every book's bytes and stamps decide reuse vs. re-encode,
     * and the reuse-cache's own sections/bytes never cross this boundary --
     * only the per-book bookkeeping in [`PublishedBookInfo`] does.
     */
    publish(): PublishOutcome;
    /**
     * Removes a book. Removing an absent book is a no-op, not an error: the
     * requested end state already holds.
     */
    removeBook(book: string): MutationOutcome;
    /**
     * Removes one chapter run's tokens from its book. The effect is whole-book:
     * the address the caller used no longer exists.
     */
    removeChapter(target: ChapterTarget): ScopedMutationOutcome;
    /**
     * Replaces the whole corpus with a validated candidate.
     *
     * Every book is built, validated, and hashed before resident state is touched,
     * so a rejection leaves the corpus, its stamps, and its identity exactly as
     * they were.
     */
    replaceCorpus(corpus: CorpusInput): MutationOutcome;
    /**
     * Seeds the whole corpus from packed bytes plus the sources they were bound to
     * — the warm cold-open.
     *
     * Composed here because this is the only layer allowed to know both halves: the
     * bytes are verified and decoded by the wire codec, and the results are handed
     * to the resident corpus, which never sees a packed byte itself. Verification is
     * the full trust boundary — structure, both checksums, exact source length and
     * content hash, the catalog stamp, every discriminant and index — so a container
     * that does not check out is refused before anything is installed.
     *
     * A book whose cached findings cannot be adopted still seeds: residency and
     * lint-priming are independent facts, so that book arrives with no lex or parse
     * and is simply awaiting recompute.
     */
    restoreCorpus(records: RestoreRecord[]): RestoreOutcome;
    /**
     * Restores the whole resident corpus from one packed `corpus.bin`
     * container -- the corpus-grain counterpart to [`Self::publish`], as
     * [`Self::restore_corpus`] is to a per-book publication.
     *
     * `records` supplies each book's own source key and exact bound source
     * (a packed container names the book but never the key a corpus was
     * addressed by, and a freshly-encoded book's bound source is wire's own
     * serialization, not necessarily any file on disk -- see
     * [`PublishedBookInfo::source`]). Verification is corpus-wide
     * (`verify_corpus`): every book must have exactly one source supplied,
     * and findings that carry stamps must all carry the *same* stamps,
     * checked atomically before anything installs.
     */
    restorePublishedCorpus(packed: Uint8Array, records: PublishedCorpusSource[]): RestoreOutcome;
    /**
     * Records one book's baseline — the state later comparisons are against.
     *
     * Only for a book that is already resident: a baseline is what the *current*
     * state is compared against, so installing one for a book with no current
     * state would invent the comparison rather than record it.
     */
    setBaseline(book: BookInput): BaselineMutationOutcome;
    /**
     * Current tokens for the requested scopes — the single hydration verb.
     *
     * Returns current truth, not state as of any earlier effect. The input is
     * normalized first (duplicates collapse, a whole-book scope absorbs that
     * book's chapter scopes), so concatenating several effects' `changed` lists is
     * always correct.
     */
    toTokens(scopes: Scope[]): ScopeTokensOutcome;
    /**
     * The exact bytes a scope would be saved as.
     */
    toUsfm(scope: CorpusScope): UsfmOutcome;
    /**
     * Replaces one book, or appends it when it is not resident yet.
     *
     * Whole-book replacement is the structural escape hatch: chapter insertion,
     * deletion, reordering, and duplicate resolution all go through here.
     */
    updateBook(book: BookInput): MutationOutcome;
    /**
     * Replaces exactly one existing chapter run with the caller's content.
     *
     * The replacement must be that same one run: no matching run is not found,
     * several is ambiguous, and content that is a different or additional chapter
     * is a label mismatch. The book's stored line ending is inherited.
     */
    updateChapter(target: ChapterTarget, replacement: ChapterInput): MutationOutcome;
    /**
     * Replaces the resident configuration.
     *
     * No tokens are rewritten, so nothing needs re-pulling and the identity — which
     * covers source bytes only — is unchanged. What changes is staleness: every
     * book is marked for recompute, because the configuration its cached findings
     * were produced under no longer applies.
     */
    updateConfig(config: BraidConfig): MutationEffect;
    /**
     * Every verse's lossless text projection for a scope, in document order.
     *
     * The resident answer to what the stateless projection computes from scratch:
     * identical entries, but a read after a one-chapter edit recomputes only that
     * chapter and takes the rest from cache — which is what makes this callable on
     * a keystroke instead of once a document.
     *
     * Entries are `[sid, projection]` pairs in first-seen token order, the same
     * shape the stateless `vrefIndexUsfm`/`vrefIndexTokens` exports return: one
     * authoritative sequence, since an object keyed by sid enumerates its keys
     * sorted and would silently reorder a document that is deliberately not.
     */
    vrefIndex(scope: CorpusScope): VrefIndexOutcome;
}

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
 * The packed trust boundary: verifies one book's container against its exact
 * source and returns the receipt plus that book's findings.
 *
 * This runs the whole Rust boundary — container/section structure, both
 * integrity checksums, exact source length and XXH3 content hash, the
 * marker-catalog stamp, every discriminant, index range, and reserved byte.
 * Nothing but tokens is left for the caller to materialize, and no token
 * object crosses this boundary. Findings are materialized here so
 * `LintIssue.message` keeps a single renderer (core's), in a single language.
 *
 * `source` is bytes rather than a string so the caller can hand over the same
 * buffer it read from disk without a UTF-16 round trip; non-UTF-8 source is a
 * rejection, not a panic.
 */
export function verifyPackedBook(packed: Uint8Array, source: Uint8Array): PackedBookOutcome;

/**
 * Verifies a whole packed corpus container against the exact sources every
 * book was bound to -- the read-only inspection counterpart to
 * [`crate::resident::Braid::restore_published_corpus`], useful to a host
 * that wants to validate a `corpus.bin` before deciding whether to restore
 * it into a resident handle at all.
 *
 * Runs the same corpus-wide trust boundary `restorePublishedCorpus` does
 * (container/section structure, both integrity checksums, exact source
 * length and content hash, the marker-catalog stamp, the all-or-none lint
 * stamp invariant), and nothing more: no resident state is read or
 * mutated, and no token crosses this boundary.
 */
export function verifyPublishedCorpus(packed: Uint8Array, sources: PublishedCorpusSourceInput[]): PublishedCorpusOutcome;

/**
 * Build the vref index from an existing token stream (the editor's live
 * path) — same rehydration as `lintTokens`, no reparse. Segment ids match
 * the tokens passed in, so they line up with the editor's DOM `data-id`s.
 */
export function vrefIndexTokens(tokens: Token[]): VrefIndex;

export function vrefIndexUsfm(source: string): VrefIndex;
