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

// Rich enough to exercise every field the comparators below have to carry
// through, not just id/kind/source/sid/marker: a default-shorthand AND an
// explicit-key attribute (`\w`'s default key is "lemma" -- marker_defs.rs),
// a footnote with a GENUINELY nested `\+nd ... \+nd*` inside it (nested
// markers require the `\+` spelling; a plain footnote alone leaves every
// token `nested: false`), a duplicate verse number (a finding anchored on
// the duplicate token), and a verse range (`\v 1-2`, a number payload with
// `NumberRangeKind::Range`, not just a bare `Verse`).
const GEN =
  '\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning \\w gracious|grace\\w* \\w noble|lemma="honor"\\w* God created' +
  ".\\f + \\ft A note about \\+nd Lord\\+nd* origins.\\f*\n\\v 1 A duplicate verse for a finding.\n\\c 2\n\\v 1-2 And the earth was without form.\n";
// An invalid `\id` book code (bookCodeValid = false), independent of the
// resident/declared book -- exercises the token-level bookCode payload's
// other branch, which the GEN fixture above never hits. The UNCLOSED `\f`
// at the end is deliberate: the unclosed-marker rule emits a finding whose
// `relatedTokenId` anchors the opening marker (the same shape the wire
// golden `related-span-unclosed-marker` pins), which the duplicate-verse
// finding alone never produces.
const EXO =
  "\\id ZZZ\n\\c 1\n\\p\n\\v 1 These are the names.\\p\n\\v 2 And the earth.\\q1\n\\v 3 Light.\\f + no closing marker\n";

let checks = 0;
function check(actual, expected, label) {
  assert.deepEqual(actual, expected, label);
  checks += 1;
}

/**
 * Complete objects, minus only the explicitly non-comparable fields --
 * destructured away by name, never an allowlist, so a field this file does
 * not yet know about enters the comparison automatically instead of being
 * silently dropped the way a five-field allowlist was.
 *
 * `span` is the only drop: braid's resident `OwnedToken` model is spanless
 * regardless of ingestion lane (established when `publish`/
 * `restorePublishedCorpus` were added — see the ledger), while a token
 * materialized straight from packed bytes carries a real span computed from
 * the container's own byte offsets. That is the one asymmetry allowed to
 * matter here. `attributeOffset` is NOT dropped: it is a remembered
 * placement distance carried by resident tokens and packed bytes alike
 * (it is even part of wire identity), and all three lanes agree on it.
 */
function comparableTokens(tokens) {
  return tokens.map((token) => {
    const { span, ...comparable } = token;
    return comparable;
  });
}

/** Same rule as `comparableTokens`, for the same reason: `span`/`relatedSpan`
 * are byte offsets that do not survive the spanless resident model. */
function comparableFindings(findings) {
  return findings.map((finding) => {
    const { span, relatedSpan, ...comparable } = finding;
    return comparable;
  });
}

/**
 * Concatenates every published book's own source into one buffer plus
 * per-book extent records (v0.1.5, bytes-at-boundary convention) -- the
 * exact pairing `verifyPublishedPacked`/`restorePublishedCorpus` both take,
 * so the SAME two values feed both lanes below.
 */
function concatPublishedSources(publishedBooks) {
  const encoder = new TextEncoder();
  const chunks = publishedBooks.map((book) => encoder.encode(book.source));
  const sources = new Uint8Array(chunks.reduce((total, chunk) => total + chunk.length, 0));
  const records = [];
  let offset = 0;
  for (let index = 0; index < publishedBooks.length; index += 1) {
    const chunk = chunks[index];
    sources.set(chunk, offset);
    records.push({
      book: publishedBooks[index].book,
      sourceKey: `${publishedBooks[index].book}.usfm`,
      byteOffset: offset,
      byteLength: chunk.length,
    });
    offset += chunk.length;
  }
  return { sources, records };
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
  const { sources, records } = concatPublishedSources(published.books);
  const verifiedResult = verifyPublishedPacked(
    wasm,
    published.bytes,
    sources,
    records,
  );
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

  // Worker lane: the same bytes restored into a fresh handle via wasm --
  // reusing the SAME `sources`/`records` the main-thread lane just verified
  // with, since both take the identical buffer-plus-extents pairing.
  const reopened = new wasm.Braid(config, makeMinter());
  const restored = unwrap(
    reopened.restorePublishedCorpus(published.bytes, sources, records),
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

// Fixture preconditions: prove the fixtures actually produce the branches
// the comparators claim to cover, so a fixture edit (or rule change) that
// silently loses a branch fails HERE, loudly, instead of leaving the
// cross-lane checks green-but-vacuous. This is the trap the first two
// rounds of this gate fell into.
{
  const probe = new wasm.Braid(config, makeMinter());
  unwrap(
    probe.replaceCorpus({
      books: [
        { kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN },
        { kind: "usfm", sourceKey: "EXO.usfm", book: "EXO", source: EXO },
      ],
    }),
    "precondition: replaceCorpus",
  );
  const lint = probe.lint();
  const genTokens = unwrap(probe.toTokens([{ book: "GEN" }]), "precondition: GEN tokens")[0].tokens;
  assert.ok(
    genTokens.some((t) => t.nested === true),
    "precondition: GEN must contain a genuinely nested (\\+) marker token",
  );
  const allFindings = lint.books.flatMap((b) => b.findings);
  assert.ok(
    allFindings.some((f) => f.relatedTokenId != null),
    "precondition: the corpus must produce a finding carrying relatedTokenId",
  );
  checks += 2;
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
  const genSource = new TextEncoder().encode(published.books[0].source);

  const publishedResult = verifyPublishedPacked(
    wasm,
    published.bytes,
    genSource,
    [{ book: "GEN", sourceKey: "GEN.usfm", byteOffset: 0, byteLength: genSource.length }],
  );
  assert.ok(publishedResult.ok, "per-book equivalence: verifyPublishedPacked");
  const combinedTokens = materializePublished(publishedResult.verified).get("GEN").tokens;

  const perBookResult = verifyPackedCorpus(wasm, published.bytes, genSource, [
    {
      path: "GEN.usfm",
      packed: { byteOffset: 0, byteLength: published.bytes.length },
      source: { byteOffset: 0, byteLength: genSource.length },
    },
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
