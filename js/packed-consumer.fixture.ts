// Type-check fixture for js/packed.d.ts. Not a runtime test: `tsc --noEmit`
// over this file is the check (see package.json's `test:packed:types`). It
// exists so a `.d.ts` edit that silently widens a frozen domain type back to
// `unknown`/`Record<string, unknown>` fails a gate instead of only a
// reviewer's eyeball.
//
// A small, realistic consumer: verify a corpus, materialize it two ways, and
// touch the fields whose types are load-bearing — `markerMetadata`,
// `structural`, `messageParams`, `token.kind` — so TypeScript actually checks
// them rather than merely accepting `any`.

import {
  decodeTokens,
  decodeTokensPublished,
  materialize,
  materializePublished,
  receiptFor,
  receiptForPublished,
  verifyPackedCorpus,
  verifyPublishedPacked,
  type MaterializedBook,
  type MaterializedPublishedBook,
  type PackedRecord,
  type PublishedCorpusRecord,
  type VerifiedPacked,
  type VerifiedPublished,
  type VerifyPackedResult,
  type VerifyPublishedResult,
} from "./packed.js";

declare const wasm: {
  verifyPackedBook(packed: Uint8Array, source: Uint8Array): unknown;
  verifyPublishedCorpus(packed: Uint8Array, sources: Uint8Array, records: unknown): unknown;
};
declare const packedAll: Uint8Array;
declare const sources: Uint8Array;
declare const records: readonly PackedRecord[];
declare const publishedPacked: Uint8Array;
declare const publishedSources: Uint8Array;
declare const publishedRecords: readonly PublishedCorpusRecord[];

const result: VerifyPackedResult = verifyPackedCorpus(wasm as never, packedAll, sources, records);

if (result.ok) {
  const verified: VerifiedPacked = result.verified;

  // The handle is opaque: it has no public data members at all.
  // @ts-expect-error `VerifiedPacked` carries no `.books` (or anything else).
  void verified.books;

  // `findings` must be the real `LintIssue[]`, not `unknown[]` — exercise a
  // field that only exists on the frozen DTO.
  for (const issues of result.findings.values()) {
    for (const issue of issues) {
      const params: Record<string, string> = issue.messageParams;
      const marker: string | undefined = issue.marker;
      void params;
      void marker;
    }
  }

  // The receipt's own descriptor rows are the other place a
  // `markerMetadata`/`structural` type lives — separate from `Token`'s copy —
  // so both must be checked, not just the one the fixture happens to reach
  // through a token. `VerifiedPacked` is opaque, so this goes through the
  // explicit snapshot accessor rather than any field on the handle.
  for (const record of records) {
    const { receipt } = receiptFor(verified, record.path);
    for (const descriptor of receipt.descriptors) {
      const canonical: string | undefined = descriptor.markerMetadata.canonical;
      const scopeKind: string = descriptor.structural.scopeKind;
      void canonical;
      void scopeKind;
    }
  }

  const book: MaterializedBook = decodeTokens(verified, "GEN.usfm");
  for (const token of book.tokens) {
    // `Token.kind` is the frozen union, not `string`.
    const kind: "newline" | "optBreak" | "marker" | "endMarker" | "milestone" | "milestoneEnd" | "bookCode" | "number" | "text" =
      token.kind;
    void kind;
    // `markerMetadata`/`structural` are the real onion DTOs, not
    // `Record<string, unknown>` — `canonical` and `scopeKind` only exist if so.
    const canonical: string | undefined = token.markerMetadata?.canonical;
    const scopeKind: string | undefined = token.structural?.scopeKind;
    void canonical;
    void scopeKind;
  }

  // `materialize` returns a `ReadonlyMap`: a caller must not be able to mutate
  // the result in place.
  const all = materialize(verified);
  // @ts-expect-error ReadonlyMap has no `set` — proves the return type isn't `Map`.
  all.set("x", book);

  // `MaterializedBook` has no `range` field (frozen shape).
  // @ts-expect-error `range` was removed from the public per-book result.
  void book.range;
}

// --- the combined-corpus layer -----------------------------------------------

const publishedResult: VerifyPublishedResult = verifyPublishedPacked(
  wasm as never,
  publishedPacked,
  publishedSources,
  publishedRecords,
);

if (publishedResult.ok) {
  const verified: VerifiedPublished = publishedResult.verified;

  // Opaque the same way `VerifiedPacked` is.
  // @ts-expect-error `VerifiedPublished` carries no `.books` (or anything else).
  void verified.books;

  const snapshotId: string = publishedResult.snapshotId;
  void snapshotId;

  for (const issues of publishedResult.findings.values()) {
    for (const issue of issues) {
      const params: Record<string, string> = issue.messageParams;
      void params;
    }
  }

  const { receipt } = receiptForPublished(verified, "GEN");
  for (const descriptor of receipt.descriptors) {
    const canonical: string | undefined = descriptor.markerMetadata.canonical;
    void canonical;
  }

  const publishedBook: MaterializedPublishedBook = decodeTokensPublished(verified, "GEN");
  // No `path` field on a combined-corpus result (frozen shape).
  // @ts-expect-error a combined container has no caller-supplied path.
  void publishedBook.path;
  for (const token of publishedBook.tokens) {
    const canonical: string | undefined = token.markerMetadata?.canonical;
    void canonical;
  }

  const allPublished = materializePublished(verified);
  // @ts-expect-error ReadonlyMap has no `set` — proves the return type isn't `Map`.
  allPublished.set("GEN", publishedBook);
}
