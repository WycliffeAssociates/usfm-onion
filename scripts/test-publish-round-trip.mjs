// The public-API round trip `braid.publish()` exists to close: the v0.1.0
// wasm surface shipped `restoreCorpus` and the `/packed` read layer, but no
// verb that *produces* packed bytes -- `corpus.bin` was production-
// unreachable. This drives the real, built wasm package (no shortcuts
// through Rust-internal test helpers) through:
//
//   braid.publish() -> verifyPublishedCorpus() (independent inspection,
//   never touching resident state) -> new Braid().restorePublishedCorpus()
//   -> identical resident state (books, findings, summary, snapshot id)
//   and identical materialized USFM, both build targets.
//
//   node scripts/test-publish-round-trip.mjs [bundler|web]

import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv.includes("web") ? "web" : "bundler";
const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");

const wasm = await import(pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href);
if (target === "web") {
  const { readFile } = await import("node:fs/promises");
  const wasmBytes = await readFile(path.join(pkgDir, "usfm_onion_web_bg.wasm"));
  await wasm.default({ module_or_path: wasmBytes });
}

function makeMinter() {
  let next = 0;
  return () => {
    next += 1;
    return `minted-${next}`;
  };
}

function unwrap(outcome, label) {
  assert.equal(outcome.status, "ok", `${label}: expected ok, got ${JSON.stringify(outcome)}`);
  return outcome.value;
}

const config = {
  lint: {
    scope: "book",
    enabledCodes: null,
    disabledCodes: [],
    suppressed: [],
    allowImplicitChapterContentVerse: false,
  },
};

const GEN = "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n";
const EXO = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

let checks = 0;
function check(actual, expected, label) {
  assert.deepEqual(actual, expected, label);
  checks += 1;
}

/**
 * One full round trip: publish the given corpus, verify the bytes
 * independently, restore into a fresh handle, and assert every facet of
 * resident state (and materialized USFM) survives -- through the public API
 * only, no internal Rust test helper in the loop.
 */
function roundTrip(label) {
  const original = new wasm.Braid(config, makeMinter());
  const replaced = original.replaceCorpus({
    books: [
      { kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN },
      { kind: "usfm", sourceKey: "EXO.usfm", book: "EXO", source: EXO },
    ],
  });
  unwrap(replaced, `${label}: replaceCorpus`);

  const expectedLint = original.lint();
  assert.ok(
    expectedLint.books.some((book) => book.findings.length > 0),
    `${label}: fixture must carry a real finding`,
  );
  const expectedUsfm = unwrap(original.toUsfm({ kind: "all" }), `${label}: toUsfm`);

  // 1. publish() -- the verb this whole gate exists to prove is reachable.
  const published = unwrap(original.publish(), `${label}: publish`);
  assert.equal(published.books.length, 2, `${label}: two books published`);
  assert.ok(
    published.books.every((book) => book.encoded && book.source !== undefined && book.source !== null),
    `${label}: a first publish encodes every book with its bound source`,
  );

  // 2. verifyPublishedCorpus() -- independent inspection, no resident state
  // touched at all; a genuine second opinion on the same bytes.
  // `PublishedCorpusSourceInput` is a plain (non-tagged) struct crossing via
  // the generic serde-wasm-bindgen path, which represents `Vec<u8>` as a
  // plain JS array (`number[]`), not a `Uint8Array` -- the same rule
  // `RestoreRecord.packed`/`source` follow (confirmed against the generated
  // `.d.ts`).
  const sources = published.books.map((book) => ({
    book: book.book,
    source: Array.from(new TextEncoder().encode(book.source)),
  }));
  const verified = wasm.verifyPublishedCorpus(new Uint8Array(published.bytes), sources);
  assert.equal(verified.status, "verified", `${label}: verifyPublishedCorpus: ${JSON.stringify(verified)}`);
  check(verified.snapshotId, published.snapshotId, `${label}: verified snapshot id matches published`);
  check(
    verified.books.map((book) => book.receipt.book),
    published.books.map((book) => book.book),
    `${label}: verified book order matches published`,
  );

  // 3. new Braid().restorePublishedCorpus() -- the corpus-grain restore
  // counterpart, into a handle that has never seen this corpus before.
  const reopened = new wasm.Braid(config, makeMinter());
  const records = published.books.map((book) => ({
    book: book.book,
    sourceKey: `${book.book}.usfm`,
    source: Array.from(new TextEncoder().encode(book.source)),
  }));
  const restored = unwrap(
    reopened.restorePublishedCorpus(new Uint8Array(published.bytes), records),
    `${label}: restorePublishedCorpus`,
  );
  check(restored.seeded.length, 2, `${label}: both books seeded`);
  check(restored.rejected, [], `${label}: nothing rejected`);

  // 4. Identical resident state: books, findings, summary, snapshot id.
  //
  // Findings are compared on the fields a consumer actually reads off one
  // (code, tokenId, sid, message, messageParams, fix) rather than the whole
  // object: `span`/`relatedSpan` are byte offsets into whichever bytes were
  // last decoded, and a materialized-from-packed token legitimately carries
  // one where a freshly parsed-then-resident token does not -- the same
  // subset the Rust-side publish/restore equivalence test already
  // established (`publication.rs::a_publication_decodes_back_to_the_native_snapshot`),
  // not a new decision made here.
  const comparableFindings = (findings) =>
    findings.map((finding) => ({
      code: finding.code,
      tokenId: finding.tokenId,
      sid: finding.sid,
      message: finding.message,
      messageParams: finding.messageParams,
      fix: finding.fix,
    }));
  const restoredLint = reopened.lint();
  check(restoredLint.snapshotId, expectedLint.snapshotId, `${label}: snapshot id survives the round trip`);
  check(restoredLint.summary, expectedLint.summary, `${label}: summary survives the round trip`);
  check(
    restoredLint.books.map((book) => ({ book: book.book, findings: comparableFindings(book.findings) })),
    expectedLint.books.map((book) => ({ book: book.book, findings: comparableFindings(book.findings) })),
    `${label}: per-book findings survive the round trip`,
  );

  // 5. Materialize equality: the restored corpus reconstructs the same USFM.
  const restoredUsfm = unwrap(reopened.toUsfm({ kind: "all" }), `${label}: restored toUsfm`);
  check(restoredUsfm, expectedUsfm, `${label}: materialized USFM survives the round trip`);
}

roundTrip("plain");

// A suppressing config: packed bytes carry no `suppressedCount`, so the
// restoring side must recompute it honestly rather than adopt a stale `0`
// (P1-B) -- exercised here through the corpus-grain verb, not just the
// per-book one.
const suppressingConfig = {
  lint: {
    scope: "book",
    enabledCodes: null,
    disabledCodes: [],
    suppressed: [{ code: "duplicate-verse-number", sid: "GEN 1:1" }],
    allowImplicitChapterContentVerse: false,
  },
};
{
  const original = new wasm.Braid(suppressingConfig, makeMinter());
  unwrap(
    original.replaceCorpus({ books: [{ kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN }] }),
    "suppressing: replaceCorpus",
  );
  const expectedLint = original.lint();
  assert.ok(expectedLint.summary.suppressedCount >= 1, "suppressing: fixture must suppress a finding");

  const published = unwrap(original.publish(), "suppressing: publish");
  const reopened = new wasm.Braid(suppressingConfig, makeMinter());
  const records = published.books.map((book) => ({
    book: book.book,
    sourceKey: `${book.book}.usfm`,
    source: Array.from(new TextEncoder().encode(book.source)),
  }));
  const restored = unwrap(
    reopened.restorePublishedCorpus(new Uint8Array(published.bytes), records),
    "suppressing: restorePublishedCorpus",
  );
  check(restored.seeded.length, 1, "suppressing: book seeded");
  check(restored.rejected, [], "suppressing: nothing rejected");

  const restoredLint = reopened.lint();
  check(
    restoredLint.summary,
    expectedLint.summary,
    "suppressing: summary (including suppressedCount) survives the round trip via honest recompute",
  );
}

console.log(`${target} publish round trip passed: ${checks} checks`);
