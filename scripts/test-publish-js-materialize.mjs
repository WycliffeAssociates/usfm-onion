// Cross-lane equivalence for the combined `publish()` container: the SAME
// packed bytes must be materializable both ways the editor actually uses
// them --
//
//   worker: publish() -> transfer -> restorePublishedCorpus() (wasm, seeds
//   braid) for mutation
//
//   main thread: publish() -> verifyPublishedPacked() -> materializePublished()
//   (pure JS, no wasm call) for rendering
//
// -- and both must agree with each other and with the original resident
// corpus's own braid.toTokens()/lint(). Before this gate, only the wasm-seed
// lane was exercised (test-publish-round-trip.mjs); this is the render lane
// closed, and the equivalence between the two that makes closing it safe.
//
//   node scripts/test-publish-js-materialize.mjs [bundler|web]

import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { materializePublished, verifyPublishedPacked, decodeTokens, materialize, verifyPackedCorpus } from "../js/packed.js";

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
const EXO = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\\p\n\\v 2 And the earth.\\q1\n\\v 3 Light.\n";

let checks = 0;
function check(actual, expected, label) {
  assert.deepEqual(actual, expected, label);
  checks += 1;
}

/** The fields a consumer actually reads off a token, matching the same
 * comparable subset `test-publish-round-trip.mjs` established for findings:
 * `span`/`relatedSpan` are byte offsets into whichever bytes were last
 * decoded, and are not expected to agree between a freshly-parsed-then-
 * resident token and one materialized from packed bytes. */
function comparableTokens(tokens) {
  return tokens.map((token) => ({
    id: token.id,
    kind: token.kind,
    source: token.source,
    sid: token.sid,
    marker: token.marker,
  }));
}

function comparableFindings(findings) {
  return findings.map((finding) => ({
    code: finding.code,
    tokenId: finding.tokenId,
    sid: finding.sid,
    message: finding.message,
    messageParams: finding.messageParams,
    fix: finding.fix,
  }));
}

/**
 * Publishes a corpus, then materializes the SAME bytes three ways: braid's
 * own live state, the JS pure-materializer over the combined container, and
 * (for the round trip) a wasm-side restore into a fresh handle. All three
 * must agree, token for token and finding for finding.
 */
async function crossLaneEquivalence(books, label) {
  const original = new wasm.Braid(config, makeMinter());
  unwrap(original.replaceCorpus({ books }), `${label}: replaceCorpus`);
  const expectedLint = original.lint();
  const expectedTokensByBook = new Map();
  for (const book of books) {
    const scoped = unwrap(original.toTokens([{ book: book.book }]), `${label}: toTokens ${book.book}`);
    expectedTokensByBook.set(book.book, scoped[0].tokens);
  }

  const published = unwrap(original.publish(), `${label}: publish`);
  assert.ok(published.books.every((b) => b.encoded), `${label}: first publish encodes every book`);

  // Main-thread lane: verify + materialize in pure JS, no wasm call for the
  // materialization step itself (verifyPublishedPacked's one wasm call is
  // the certifier -- §H: no JS hashing/validation, ever).
  const sources = published.books.map((b) => ({
    book: b.book,
    source: new TextEncoder().encode(b.source),
  }));
  const verifiedResult = verifyPublishedPacked(wasm, new Uint8Array(published.bytes), sources);
  assert.ok(verifiedResult.ok, `${label}: verifyPublishedPacked: ${JSON.stringify(verifiedResult)}`);
  check(verifiedResult.snapshotId, published.snapshotId, `${label}: snapshot id matches published`);

  const materializedAll = materializePublished(verifiedResult.verified);
  for (const book of books) {
    const jsTokens = materializedAll.get(book.book).tokens;
    check(
      comparableTokens(jsTokens),
      comparableTokens(expectedTokensByBook.get(book.book)),
      `${label}: ${book.book} JS-materialized tokens match braid.toTokens`,
    );
    const expectedFindings = expectedLint.books.find((b) => b.book === book.book).findings;
    check(
      comparableFindings(verifiedResult.findings.get(book.book)),
      comparableFindings(expectedFindings),
      `${label}: ${book.book} JS-verified findings match braid.lint()`,
    );
  }

  // Selective (single-book, single-chapter) materialization must be
  // identical to the corresponding slice of the full pass -- the same
  // guarantee the per-book layer makes.
  const oneBook = materializePublished(verifiedResult.verified, { book: books[0].book });
  check(
    comparableTokens(oneBook.get(books[0].book).tokens),
    comparableTokens(materializedAll.get(books[0].book).tokens),
    `${label}: selective single-book materialize matches the full pass`,
  );

  // Worker lane: the same bytes restored into a fresh handle via wasm.
  const reopened = new wasm.Braid(config, makeMinter());
  const records = published.books.map((b) => ({
    book: b.book,
    sourceKey: `${b.book}.usfm`,
    source: Array.from(new TextEncoder().encode(b.source)),
  }));
  const restored = unwrap(
    reopened.restorePublishedCorpus(new Uint8Array(published.bytes), records),
    `${label}: restorePublishedCorpus`,
  );
  check(restored.seeded.length, books.length, `${label}: every book seeded`);
  check(restored.rejected, [], `${label}: nothing rejected`);

  const restoredLint = reopened.lint();
  for (const book of books) {
    const restoredScoped = unwrap(
      reopened.toTokens([{ book: book.book }]),
      `${label}: restored toTokens ${book.book}`,
    );
    check(
      comparableTokens(restoredScoped[0].tokens),
      comparableTokens(expectedTokensByBook.get(book.book)),
      `${label}: ${book.book} restored (wasm) tokens match the original`,
    );
    const restoredFindings = restoredLint.books.find((b) => b.book === book.book).findings;
    const expectedFindings = expectedLint.books.find((b) => b.book === book.book).findings;
    check(
      comparableFindings(restoredFindings),
      comparableFindings(expectedFindings),
      `${label}: ${book.book} restored (wasm) findings match the original`,
    );
  }

  // Three-way close: the JS-materialized tokens and the wasm-restored
  // tokens must also agree with each other directly, not merely both with
  // the original -- the actual cross-lane guarantee this gate exists for.
  for (const book of books) {
    const restoredScoped = unwrap(reopened.toTokens([{ book: book.book }]), `${label}: re-fetch`);
    check(
      comparableTokens(materializedAll.get(book.book).tokens),
      comparableTokens(restoredScoped[0].tokens),
      `${label}: ${book.book} JS materialization matches the wasm-restored tokens directly`,
    );
  }
}

await crossLaneEquivalence(
  [{ kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN }],
  "single-book",
);
await crossLaneEquivalence(
  [
    { kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN },
    { kind: "usfm", sourceKey: "EXO.usfm", book: "EXO", source: EXO },
  ],
  "two-book",
);

// Equivalence with the per-book lane, where both can express the same
// corpus: a ONE-book combined container is, by construction (the writer
// lays out a single-book publication exactly as the single-book verifier
// expects it -- the same fact the native equivalence gate proves), also a
// valid input to the per-book layer (`verifyPackedCorpus`/`materialize`).
// Both layers must decode it to the same tokens.
{
  const original = new wasm.Braid(config, makeMinter());
  unwrap(
    original.replaceCorpus({ books: [{ kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN }] }),
    "per-book equivalence: replaceCorpus",
  );
  const published = unwrap(original.publish(), "per-book equivalence: publish");
  const genSource = published.books[0].source;

  const publishedResult = verifyPublishedPacked(wasm, new Uint8Array(published.bytes), [
    { book: "GEN", source: new TextEncoder().encode(genSource) },
  ]);
  assert.ok(publishedResult.ok, "per-book equivalence: verifyPublishedPacked");
  const combinedTokens = materializePublished(publishedResult.verified).get("GEN").tokens;

  const perBookResult = verifyPackedCorpus(wasm, [
    { path: "GEN.usfm", packed: new Uint8Array(published.bytes), source: new TextEncoder().encode(genSource) },
  ]);
  assert.ok(perBookResult.ok, `per-book equivalence: verifyPackedCorpus: ${JSON.stringify(perBookResult)}`);
  const perBookTokens = decodeTokens(perBookResult.verified, "GEN.usfm").tokens;

  check(
    comparableTokens(combinedTokens),
    comparableTokens(perBookTokens),
    "per-book equivalence: a one-book combined container decodes identically through both layers",
  );

  const allPerBook = materialize(perBookResult.verified);
  check(
    comparableTokens(allPerBook.get("GEN.usfm").tokens),
    comparableTokens(perBookTokens),
    "per-book equivalence: materialize (all) matches decodeTokens for the same book",
  );
}

console.log(`${target} publish JS-materialize cross-lane equivalence passed: ${checks} checks`);
