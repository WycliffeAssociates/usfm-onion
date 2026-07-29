// Types for ./packed.js — packed-corpus verification glue and the pure-JS token
// materializer. See epic §8.1 and freeze §H/§I for the boundary contract: Rust
// certifies bytes and materializes findings; JS materializes tokens.

/** One record to restore: the caller's key, the packed container, its exact source. */
export type PackedRecord = Readonly<{
  path: string;
  packed: Uint8Array;
  source: Uint8Array;
}>;

/** Resolved name-derived marker facts the packed bytes deliberately do not store. */
export type PackedMarkerDescriptor = Readonly<{
  name: string;
  nested: boolean;
  markerMetadata: Readonly<Record<string, unknown>>;
  structural: Readonly<Record<string, unknown>>;
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
 * Certified packed bytes. Mintable only by {@link verifyPackedCorpus} — the
 * brand is a compile-time guardrail (not a security boundary) that keeps
 * unverified buffers out of the materializer's public type.
 */
export type VerifiedPacked = Readonly<{
  [verifiedBrand]: true;
  books: ReadonlyMap<string, Readonly<{
    path: string;
    packed: Uint8Array;
    source: Uint8Array;
    receipt: PackedBookReceipt;
  }>>;
}>;

/** What the wasm export returns for one record. A rejection is a value, not a throw. */
export type PackedBookOutcome =
  | Readonly<{ status: "verified"; receipt: PackedBookReceipt; findings: readonly unknown[] }>
  | Readonly<{ status: "rejected"; error: PackedDecodeError }>;

export type VerifyPackedResult =
  | Readonly<{
      ok: true;
      verified: VerifiedPacked;
      /** Rust-materialized `LintIssue` DTOs, keyed by `path`. */
      findings: ReadonlyMap<string, readonly unknown[]>;
    }>
  | Readonly<{ ok: false; path: string; error: PackedDecodeError }>;

/**
 * One book's tokens.
 *
 * `stableIds` is present exactly when the section carried explicit ids; `Token.id`
 * still carries the positional `{book}-{index}` label in that case, matching the
 * Rust decoder. `range` is present only for a chapter-selective materialize.
 */
export type MaterializedBook = Readonly<{
  path: string;
  book: string;
  tokens: readonly Record<string, unknown>[];
  stableIds?: readonly string[];
  range?: Readonly<{ start: number; end: number }>;
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
 * Verifies every record through the Rust trust boundary and mints the
 * `VerifiedPacked` brand. The first rejected record short-circuits the corpus.
 */
export declare function verifyPackedCorpus(
  wasm: { verifyPackedBook(packed: Uint8Array, source: Uint8Array): PackedBookOutcome },
  records: readonly PackedRecord[],
): VerifyPackedResult;

/**
 * Materializes tokens from certified bytes in the JS engine, with no wasm call.
 * No selector materializes every book; `{path}`/`{book}` one book; adding
 * `{chapter}` materializes only that chapter's contiguous row range, which is
 * guaranteed identical to the corresponding slice of the full pass.
 */
export declare function materialize(
  verified: VerifiedPacked,
  selector?: MaterializeSelector,
): Map<string, MaterializedBook>;

/** Tokens-only entry for one book, by the path the caller supplied. */
export declare function decodeTokens(verified: VerifiedPacked, path: string): MaterializedBook;
