// Type-check fixture for js/packed.d.ts. Not a runtime test: `tsc --noEmit`
// over this file is the check (see package.json's `test:packed:types`). It
// exists so a `.d.ts` edit that silently widens a frozen §8.1 type back to
// `unknown`/`Record<string, unknown>` fails a gate instead of only a reviewer's
// eyeball.
//
// A small, realistic consumer: verify a corpus, materialize it two ways, and
// touch the fields whose types are load-bearing (P1-2) — `markerMetadata`,
// `structural`, `messageParams`, `token.kind` — so TypeScript actually checks
// them rather than merely accepting `any`.

import {
  decodeTokens,
  materialize,
  verifyPackedCorpus,
  type MaterializedBook,
  type PackedRecord,
  type VerifiedPacked,
  type VerifyPackedResult,
} from "./packed.js";

declare const wasm: { verifyPackedBook(packed: Uint8Array, source: Uint8Array): unknown };
declare const records: readonly PackedRecord[];

const result: VerifyPackedResult = verifyPackedCorpus(wasm as never, records);

if (result.ok) {
  const verified: VerifiedPacked = result.verified;

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

  // The receipt's own descriptor rows (freeze §I.3) are the other place a
  // `markerMetadata`/`structural` type lives — separate from `Token`'s copy —
  // so both must be checked, not just the one the fixture happens to reach
  // through a token.
  for (const b of verified.books.values()) {
    for (const descriptor of b.receipt.descriptors) {
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
