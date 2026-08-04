// Types for ./packed.js — packed-corpus verification glue and the pure-JS token
// materializer. The boundary contract: Rust certifies bytes and materializes
// findings; JS materializes tokens.

import type {
  LintIssue,
  MarkerMetadata,
  StructuralMarkerInfo,
  Token,
} from "../pkg-bundler/usfm_onion_web.js";

/**
 * A byte range `[byteOffset, byteOffset + byteLength)` into whichever
 * sibling buffer the record pairs it with -- boundary-only (v0.1.5,
 * bytes-at-boundary convention): bytes never cross as a JS `number[]`, so a
 * corpus-grain payload crosses as one buffer plus extent records like this
 * one into it.
 */
export type ByteExtent = Readonly<{
  byteOffset: number;
  byteLength: number;
}>;

/** One book's own extent for a combined-corpus verify: addressed by book
 * code and its extent into the `sources` buffer -- a combined container has
 * no per-book caller-supplied path. */
export type PublishedCorpusRecord = Readonly<{
  book: string;
  sourceKey: string;
}> &
  ByteExtent;

/** One record to restore: the caller's key, plus its own `packed`/`source`
 * extents into the two buffers {@link verifyPackedCorpus} takes alongside
 * `records` -- deliberately the same shape a `publishScope` result's own
 * `books[]` already is. */
export type PackedRecord = Readonly<{
  path: string;
  packed: ByteExtent;
  source: ByteExtent;
}>;

/**
 * Resolved name-derived marker facts the packed bytes deliberately do not
 * store. `markerMetadata`/`structural` are deep-frozen at verification time
 * and shared by reference across every materialized token for the same
 * marker form: attempting to mutate one throws under strict mode, and a
 * consumer that wants a decorated copy clones it itself.
 */
export type PackedMarkerDescriptor = Readonly<{
  name: string;
  nested: boolean;
  markerMetadata: MarkerMetadata;
  structural: StructuralMarkerInfo;
}>;

/**
 * What the Rust trust boundary attests for one book. The three integrity values
 * are 16-character lowercase hex: an audit record of what Rust checked, never an
 * input to a check here — this package contains no hash implementation.
 */
export type PackedBookReceipt = Readonly<{
  book: string;
  sourceLen: number;
  tokenCount: number;
  findingCount: number;
  positionalIds: boolean;
  sourceHash: string;
  catalogStamp: string;
  snapshotId: string;
  descriptors: readonly PackedMarkerDescriptor[];
}>;

/** The frozen `DecodeError` set, tagged so a consumer narrows instead of string-matching. */
export type PackedDecodeError =
  | Readonly<{ kind: "truncated" }>
  | Readonly<{ kind: "badMagic" }>
  | Readonly<{ kind: "unsupportedVersion"; found: number }>
  | Readonly<{ kind: "unsupportedFlags"; found: number }>
  | Readonly<{ kind: "invalidToc" }>
  | Readonly<{ kind: "invalidSection" }>
  | Readonly<{ kind: "invalidUtf8" }>
  | Readonly<{ kind: "invalidDiscriminant" }>
  | Readonly<{ kind: "offsetOverflow" }>
  | Readonly<{ kind: "tooManySids"; found: number }>
  | Readonly<{ kind: "checksumMismatch" }>
  | Readonly<{ kind: "catalogMismatch" }>
  | Readonly<{ kind: "sourceLengthMismatch" }>
  | Readonly<{ kind: "sourceHashMismatch" }>;

declare const verifiedBrand: unique symbol;

/**
 * Certified packed bytes. Opaque: mintable only by {@link verifyPackedCorpus},
 * and exposes no data members at all — every decoder input lives in a
 * module-private `WeakMap` keyed by the handle's identity, not on the handle
 * itself. Use {@link receiptFor} to inspect a book's receipt.
 *
 * This opacity is a footgun-elimination device, not a security boundary: see
 * the threat-model note on {@link verifyPackedCorpus}.
 */
export type VerifiedPacked = Readonly<{ readonly [verifiedBrand]: true }>;

declare const verifiedPublishedBrand: unique symbol;

/**
 * Certified combined-corpus bytes (a `publish()` container). Opaque, the
 * same footgun-elimination shape as {@link VerifiedPacked}: mintable only
 * by {@link verifyPublishedPacked}, no data members, every decoder input
 * lives in a module-private `WeakMap`. Use {@link receiptForPublished} to
 * inspect a book's receipt.
 */
export type VerifiedPublished = Readonly<{ readonly [verifiedPublishedBrand]: true }>;

/** What `wasm.verifyPublishedCorpus` returns for a combined container. */
export type PublishedCorpusOutcome =
  | Readonly<{
      status: "verified";
      snapshotId: string;
      books: readonly Readonly<{ receipt: PackedBookReceipt; findings: readonly LintIssue[] }>[];
    }>
  | Readonly<{ status: "rejected"; error: PackedDecodeError }>;

export type VerifyPublishedResult =
  | Readonly<{
      ok: true;
      verified: VerifiedPublished;
      snapshotId: string;
      /** Rust-materialized `LintIssue` DTOs, keyed by book code. */
      findings: ReadonlyMap<string, readonly LintIssue[]>;
    }>
  | Readonly<{ ok: false; error: PackedDecodeError | Readonly<{ kind: "invalidExtent"; book: string }> }>;

/**
 * One book's tokens out of a combined-corpus materialization: `{book,
 * tokens, stableIds?}` -- no `path` (a combined container has none) where
 * {@link MaterializedBook} has one.
 */
export type MaterializedPublishedBook = Readonly<{
  book: string;
  tokens: readonly Token[];
  stableIds?: readonly string[];
}>;

export type MaterializePublishedSelector = Readonly<{
  book?: string;
  chapter?: number;
}>;

/** What the wasm export returns for one record. A rejection is a value, not a throw. */
export type PackedBookOutcome =
  | Readonly<{ status: "verified"; receipt: PackedBookReceipt; findings: readonly LintIssue[] }>
  | Readonly<{ status: "rejected"; error: PackedDecodeError }>;

export type VerifyPackedResult =
  | Readonly<{
      ok: true;
      verified: VerifiedPacked;
      /** Rust-materialized `LintIssue` DTOs, keyed by `path`. */
      findings: ReadonlyMap<string, readonly LintIssue[]>;
    }>
  | Readonly<{
      ok: false;
      path: string;
      error: PackedDecodeError | Readonly<{ kind: "invalidExtent" }>;
    }>;

/**
 * One book's tokens (frozen shape: `{path, book, tokens, stableIds?}` — no
 * `range` field, selective materialize is a slicing strategy, not part of the
 * public result).
 *
 * `stableIds` is present exactly when the section carried explicit ids; `Token.id`
 * still carries the positional `{book}-{index}` label in that case, matching the
 * Rust decoder.
 */
export type MaterializedBook = Readonly<{
  path: string;
  book: string;
  tokens: readonly Token[];
  stableIds?: readonly string[];
}>;

export type MaterializeSelector = Readonly<{
  /** The key supplied in the record. The primary selector: book codes are not unique. */
  path?: string;
  /** A three-letter book code; rejected as `ambiguousBook` if two records share it. */
  book?: string;
  chapter?: number;
}>;

/** Thrown by the materializer. `kind` mirrors `PackedDecodeError["kind"]`, plus selector faults. */
export declare class PackedError extends Error {
  readonly kind:
    | PackedDecodeError["kind"]
    | "unknownBook"
    | "ambiguousBook"
    | "unknownChapter";
  constructor(kind: string, detail?: string);
}

/**
 * Verifies every record through the Rust trust boundary and mints the opaque
 * `VerifiedPacked` handle. The first rejected record short-circuits the corpus.
 *
 * Threat model (dated 2026-07-29): this module protects honest use —
 * certification, typed errors, copy-at-mint — not deliberate subversion of its
 * own in-process state. The handle's opacity is a footgun-elimination device,
 * not a security boundary.
 */
export declare function verifyPackedCorpus(
  wasm: { verifyPackedBook(packed: Uint8Array, source: Uint8Array): PackedBookOutcome },
  packedAll: Uint8Array,
  sources: Uint8Array,
  records: readonly PackedRecord[],
): VerifyPackedResult;

/**
 * A detached snapshot of what verification certified for one book —
 * `structuredClone` produces an ordinary mutable object, not a read-only one.
 * It is detached, not read-only: the decoder never reads this function's
 * return value back, so mutating it cannot affect `materialize`/
 * `decodeTokens` either way. For inspection only; not part of the decode path.
 */
export declare function receiptFor(
  verified: VerifiedPacked,
  path: string,
): Readonly<{ path: string; receipt: PackedBookReceipt }>;

/**
 * Materializes tokens from certified bytes in the JS engine, with no wasm call.
 * No selector materializes every book; `{path}`/`{book}` one book; adding
 * `{chapter}` materializes only that chapter's contiguous row range, which is
 * guaranteed identical to the corresponding slice of the full pass.
 */
export declare function materialize(
  verified: VerifiedPacked,
  selector?: MaterializeSelector,
): ReadonlyMap<string, MaterializedBook>;

/** Tokens-only entry for one book, by the path the caller supplied. */
export declare function decodeTokens(verified: VerifiedPacked, path: string): MaterializedBook;

// --- the combined-corpus layer -----------------------------------------------

/**
 * Verifies a combined `publish()` container through the Rust trust boundary
 * (`wasm.verifyPublishedCorpus`, the sole certifier) and mints the opaque
 * `VerifiedPublished` handle.
 */
export declare function verifyPublishedPacked(
  wasm: {
    verifyPublishedCorpus(
      packed: Uint8Array,
      sources: Uint8Array,
      records: readonly PublishedCorpusRecord[],
    ): PublishedCorpusOutcome;
  },
  packed: Uint8Array,
  sources: Uint8Array,
  records: readonly PublishedCorpusRecord[],
): VerifyPublishedResult;

/**
 * A detached snapshot of what verification certified for one book, by book
 * code -- a combined container has no caller-supplied path (contrast
 * {@link receiptFor}, keyed by path).
 */
export declare function receiptForPublished(
  verified: VerifiedPublished,
  book: string,
): Readonly<{ book: string; receipt: PackedBookReceipt }>;

/**
 * Materializes tokens from a certified combined corpus in the JS engine,
 * with no wasm call. No selector materializes every book, keyed by book
 * code; `{book}` one book; adding `{chapter}` materializes only that
 * chapter's contiguous row range, identical to the corresponding slice of
 * the full pass.
 */
export declare function materializePublished(
  verified: VerifiedPublished,
  selector?: MaterializePublishedSelector,
): ReadonlyMap<string, MaterializedPublishedBook>;

/** Tokens-only entry for one book, by book code. */
export declare function decodeTokensPublished(
  verified: VerifiedPublished,
  book: string,
): MaterializedPublishedBook;

/**
 * Reuses `previous`'s finding objects wherever a finding is unchanged, so a consumer
 * keeps whatever it attached to them. Identity is the rule code plus the anchored
 * token ids — the only address stable across a recompute. Returns `previous` itself
 * when nothing moved.
 */
export declare function reconcileFindings<T>(
  previous: readonly T[] | undefined,
  next: readonly T[],
): readonly T[];
