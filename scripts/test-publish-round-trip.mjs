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
// v0.1.5 bytes-at-boundary convention: every corpus-grain byte payload
// crosses as ONE real `Uint8Array` per side (never a per-book array, never a
// JS `number[]`), with extent records (`{byteOffset, byteLength}`) naming
// each book's own slice into it -- this file's own diff (against the
// pre-v0.1.5 version) is the editor's migration guide for every one of
// `restoreCorpus`/`restorePublishedCorpus`/`verifyPublishedCorpus`/
// `publishScope`.
//
//   node scripts/test-publish-round-trip.mjs [bundler|web]

import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { verifyPackedCorpus, materialize } from "../js/packed.js";

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
 * Concatenates every published book's own source into one buffer plus
 * per-book extent records (v0.1.5, bytes-at-boundary convention) -- the
 * exact pairing `wasm.verifyPublishedCorpus`/`restorePublishedCorpus` both
 * take.
 */
function concatSources(publishedBooks) {
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
  // `published.bytes` is already a real `Uint8Array` (`serde_bytes`, honored
  // now that this workspace's `tsify` dependency resolves its `js` feature)
  // -- one whole-corpus buffer, no wrap needed.
  const published = unwrap(original.publish(), `${label}: publish`);
  assert.ok(published.bytes instanceof Uint8Array, `${label}: publish().bytes is a real Uint8Array`);
  assert.equal(published.books.length, 2, `${label}: two books published`);
  assert.ok(
    published.books.every((book) => book.encoded && book.source !== undefined && book.source !== null),
    `${label}: a first publish encodes every book with its bound source`,
  );

  // 2. verifyPublishedCorpus() -- independent inspection, no resident state
  // touched at all; a genuine second opinion on the same bytes. `sources`/
  // `records` is the buffer-plus-extents pairing every corpus-grain verb in
  // this file now shares.
  const { sources, records } = concatSources(published.books);
  const verified = wasm.verifyPublishedCorpus(published.bytes, sources, records);
  assert.equal(verified.status, "verified", `${label}: verifyPublishedCorpus: ${JSON.stringify(verified)}`);
  check(verified.snapshotId, published.snapshotId, `${label}: verified snapshot id matches published`);
  check(
    verified.books.map((book) => book.receipt.book),
    published.books.map((book) => book.book),
    `${label}: verified book order matches published`,
  );

  // 3. new Braid().restorePublishedCorpus() -- the corpus-grain restore
  // counterpart, into a handle that has never seen this corpus before.
  // Reuses the SAME `sources`/`records` step 2 just verified with.
  const reopened = new wasm.Braid(config, makeMinter());
  const restored = unwrap(
    reopened.restorePublishedCorpus(published.bytes, sources, records),
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

/**
 * Builds a two-book corpus and its `publishScope({kind:"all"})` output --
 * shared by the verbatim-forward round trip and the adversarial extent
 * section below.
 */
function scopedTwoBookCorpus() {
  const original = new wasm.Braid(config, makeMinter());
  unwrap(
    original.replaceCorpus({
      books: [
        { kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN },
        { kind: "usfm", sourceKey: "EXO.usfm", book: "EXO", source: EXO },
      ],
    }),
    "publishScope: replaceCorpus",
  );
  const scoped = unwrap(original.publishScope({ kind: "all" }), "publishScope: all");
  // `packed`/`sources` are already real `Uint8Array`s (`ScopedPublication`'s
  // own doc comment) -- no wrap needed.
  return { original, scoped };
}

// publishScope: the per-book verb -- its output must be feedable straight
// into the pure-JS render lane (verifyPackedCorpus/materialize from
// ./packed, no wasm call in that half), and its tokens must agree with the
// same handle's own toTokens() for every book in scope.
{
  const { original, scoped } = scopedTwoBookCorpus();
  assert.equal(scoped.books.length, 2, "publishScope: both books in scope");
  assert.ok(scoped.packed instanceof Uint8Array, "publishScope: packed is a real Uint8Array");
  assert.ok(scoped.sources instanceof Uint8Array, "publishScope: sources is a real Uint8Array");
  assert.ok(
    scoped.books.every((book) => book.packed.byteLength > 0 && book.source.byteLength > 0),
    "publishScope: every book is always encoded with its source",
  );
  check(
    scoped.snapshotId,
    original.expectedSnapshotId(),
    "publishScope: snapshotId matches the handle's own corpus identity",
  );

  // The exact shape verifyPackedCorpus/restoreCorpus take, forwarded with
  // zero reshaping (only the book -> path rename, since a per-book
  // container's caller-supplied key is a path, not the book code alone).
  const records = scoped.books.map((book) => ({
    path: `${book.book}.usfm`,
    packed: book.packed,
    source: book.source,
  }));
  const verified = verifyPackedCorpus(wasm, scoped.packed, scoped.sources, records);
  assert.ok(verified.ok, `publishScope: verifyPackedCorpus: ${JSON.stringify(verified)}`);
  const materialized = materialize(verified.verified);

  for (const book of scoped.books) {
    const path = `${book.book}.usfm`;
    const jsTokens = materialized.get(path).tokens;
    const nativeTokens = unwrap(
      original.toTokens([{ book: book.book }]),
      `publishScope: toTokens ${book.book}`,
    )[0].tokens;
    check(jsTokens.length, nativeTokens.length, `publishScope: ${book.book} token count`);
    const comparable = (tokens) =>
      tokens.map((token) => ({
        id: token.id,
        kind: token.kind,
        source: token.source,
        sid: token.sid ?? null,
        marker: token.marker ?? null,
      }));
    check(
      comparable(jsTokens),
      comparable(nativeTokens),
      `publishScope: ${book.book} tokens (pure-JS materialize) equal toTokens()`,
    );
  }
}

// Key symmetry: publishScope's output buffers+records forward UNMODIFIED
// into restoreCorpus, into a FRESH handle -- token and finding equality
// against the handle that produced the scoped publication. This is the
// verbatim-forward contract ScopedPublication's own doc comment names.
{
  const { original, scoped } = scopedTwoBookCorpus();
  const expectedLint = original.lint();

  const reopened = new wasm.Braid(config, makeMinter());
  const records = scoped.books.map((book) => ({
    path: `${book.book}.usfm`,
    packed: book.packed,
    source: book.source,
  }));
  const restored = unwrap(
    reopened.restoreCorpus(scoped.packed, scoped.sources, records),
    "verbatim-forward: restoreCorpus",
  );
  check(restored.seeded.length, 2, "verbatim-forward: both books seeded");
  check(restored.rejected, [], "verbatim-forward: nothing rejected");

  const restoredLint = reopened.lint();
  const comparableFindings = (findings) =>
    findings.map((finding) => ({ code: finding.code, tokenId: finding.tokenId, sid: finding.sid }));
  for (const book of scoped.books) {
    const originalTokens = unwrap(
      original.toTokens([{ book: book.book }]),
      `verbatim-forward: original toTokens ${book.book}`,
    )[0].tokens;
    const restoredTokens = unwrap(
      reopened.toTokens([{ book: book.book }]),
      `verbatim-forward: restored toTokens ${book.book}`,
    )[0].tokens;
    check(
      restoredTokens.map((t) => ({ id: t.id, kind: t.kind, source: t.source })),
      originalTokens.map((t) => ({ id: t.id, kind: t.kind, source: t.source })),
      `verbatim-forward: ${book.book} tokens survive the forward`,
    );
    const originalFindings = expectedLint.books.find((b) => b.book === book.book).findings;
    const restoredFindings = restoredLint.books.find((b) => b.book === book.book).findings;
    check(
      comparableFindings(restoredFindings),
      comparableFindings(originalFindings),
      `verbatim-forward: ${book.book} findings survive the forward`,
    );
  }
}

// --- adversarial: extents must refuse, never clamp/truncate ------------------
//
// Every case below names the offending book/record in its refusal, and none
// of them may throw -- a bad extent is caller data, not a programmer error.
{
  const { scoped } = scopedTwoBookCorpus();
  const genRecord = scoped.books.find((book) => book.book === "GEN");

  // Out-of-bounds: byteLength runs past the end of `scoped.packed`.
  {
    const reopened = new wasm.Braid(config, makeMinter());
    const outcome = reopened.restoreCorpus(scoped.packed, scoped.sources, [
      {
        path: "GEN.usfm",
        packed: { byteOffset: genRecord.packed.byteOffset, byteLength: scoped.packed.length + 1 },
        source: genRecord.source,
      },
    ]);
    assert.equal(outcome.status, "error", "adversarial: out-of-bounds packed extent must refuse");
    check(
      outcome.error,
      { kind: "invalidExtent", book: "GEN.usfm" },
      "adversarial: out-of-bounds packed extent names the record",
    );
  }

  // Overflowing: byteOffset + byteLength overflows a naive same-width sum,
  // including an overflow-y `u32::MAX`-scale value.
  {
    const reopened = new wasm.Braid(config, makeMinter());
    const outcome = reopened.restoreCorpus(scoped.packed, scoped.sources, [
      {
        path: "GEN.usfm",
        packed: { byteOffset: 0xffffffff, byteLength: 0xffffffff },
        source: genRecord.source,
      },
    ]);
    assert.equal(outcome.status, "error", "adversarial: overflowing packed extent must refuse");
    check(
      outcome.error,
      { kind: "invalidExtent", book: "GEN.usfm" },
      "adversarial: overflowing packed extent names the record",
    );
  }

  // Invalid UTF-8: a source extent whose bytes are not valid UTF-8. Native
  // takes `&str` for source bytes, so this must refuse rather than reach a
  // native call with a bad slice.
  {
    const reopened = new wasm.Braid(config, makeMinter());
    const badSources = new Uint8Array([0xff, 0xfe, 0xfd]);
    const outcome = reopened.restoreCorpus(scoped.packed, badSources, [
      {
        path: "GEN.usfm",
        packed: genRecord.packed,
        source: { byteOffset: 0, byteLength: 3 },
      },
    ]);
    assert.equal(outcome.status, "error", "adversarial: invalid-UTF-8 source extent must refuse");
    // Refused at this boundary itself (`InvalidExtent`, naming the record),
    // never reaching a native call that would classify it as a generic
    // decode defect with no book identifier attached.
    check(
      outcome.error,
      { kind: "invalidExtent", book: "GEN.usfm" },
      "adversarial: invalid-UTF-8 source extent names the record",
    );
  }

  // The same three cases through verifyPublishedCorpus/restorePublishedCorpus
  // (the single-extent-per-record pairing), and through the pure-JS
  // verifyPackedCorpus lane.
  {
    const { sources, records } = concatSources([{ book: "GEN", source: GEN }]);
    const genPublish = unwrap(
      (() => {
        const b = new wasm.Braid(config, makeMinter());
        unwrap(b.replaceCorpus({ books: [{ kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN }] }), "adversarial seed");
        return b.publish();
      })(),
      "adversarial: publish",
    );
    const outOfBounds = wasm.verifyPublishedCorpus(genPublish.bytes, sources, [
      { ...records[0], byteLength: sources.length + 1 },
    ]);
    assert.equal(outOfBounds.status, "invalidExtent", "adversarial: verifyPublishedCorpus out-of-bounds refuses");
    check(outOfBounds.book, "GEN", "adversarial: verifyPublishedCorpus names the book");

    const overflowing = wasm.verifyPublishedCorpus(genPublish.bytes, sources, [
      { ...records[0], byteOffset: 0xffffffff, byteLength: 0xffffffff },
    ]);
    assert.equal(overflowing.status, "invalidExtent", "adversarial: verifyPublishedCorpus overflow refuses");
    check(overflowing.book, "GEN", "adversarial: verifyPublishedCorpus overflow names the book");

    const reopened = new wasm.Braid(config, makeMinter());
    const restoreOutOfBounds = reopened.restorePublishedCorpus(genPublish.bytes, sources, [
      { ...records[0], byteLength: sources.length + 1 },
    ]);
    assert.equal(restoreOutOfBounds.status, "error", "adversarial: restorePublishedCorpus out-of-bounds refuses");
    check(
      restoreOutOfBounds.error,
      { kind: "invalidExtent", book: "GEN" },
      "adversarial: restorePublishedCorpus out-of-bounds names the book",
    );

    const jsOutOfBounds = verifyPackedCorpus(wasm, genPublish.bytes, sources, [
      {
        path: "GEN.usfm",
        packed: { byteOffset: 0, byteLength: genPublish.bytes.length },
        source: { byteOffset: 0, byteLength: sources.length + 1 },
      },
    ]);
    assert.equal(jsOutOfBounds.ok, false, "adversarial: pure-JS verifyPackedCorpus out-of-bounds refuses");
    check(
      jsOutOfBounds.error,
      { kind: "invalidExtent" },
      "adversarial: pure-JS verifyPackedCorpus refusal is typed",
    );
    check(jsOutOfBounds.path, "GEN.usfm", "adversarial: pure-JS verifyPackedCorpus names the record");
  }
}

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
  const { sources, records } = concatSources(published.books);
  const restored = unwrap(
    reopened.restorePublishedCorpus(published.bytes, sources, records),
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

// Clean-room re-review P1: an empty sourceKey must classify as
// {kind: "ingest", error: {kind: "duplicateSourceKey", source: ""}} through
// the actual built package -- the pre-extraction wasm behavior, reproduced
// via braid::RestoreError::EmptySourceKey rather than silently
// reclassified as a decode defect. This is also pinned in the parity
// transcript (restore_published_corpus_empty_source_key); this check
// exercises the same case through the real npm package end to end.
{
  const original = new wasm.Braid(config, makeMinter());
  unwrap(
    original.replaceCorpus({ books: [{ kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN }] }),
    "empty source key: replaceCorpus",
  );
  const published = unwrap(original.publish(), "empty source key: publish");
  const sources = new TextEncoder().encode(published.books[0].source);
  const records = [{ book: "GEN", sourceKey: "", byteOffset: 0, byteLength: sources.length }];
  const reopened = new wasm.Braid(config, makeMinter());
  const outcome = reopened.restorePublishedCorpus(published.bytes, sources, records);
  assert.equal(outcome.status, "error", `empty source key: expected a refusal, got ${JSON.stringify(outcome)}`);
  check(
    outcome.error,
    { kind: "ingest", error: { kind: "duplicateSourceKey", source: "" } },
    "empty source key: classified as the pre-extraction ingest/duplicateSourceKey shape",
  );
}

// --- shape pin: tsify's `js` feature, permanently -----------------------
//
// Pins the actual JS shapes the v0.1.5 bytes-at-boundary convention (plus
// this round's tsify `js`-feature migration) depends on, forever -- not
// just today's manual verification. If a future dependency bump or Cargo
// feature change silently drags `tsify` back onto its legacy `json`
// feature (`JsValue::from_serde`), every byte field here regresses to a
// plain `number[]` and every map-shaped field regresses to an ES `Map`, and
// this section is what catches it.
{
  const original = new wasm.Braid(config, makeMinter());
  unwrap(
    original.replaceCorpus({
      books: [
        // A duplicate verse number (GEN 1:1 twice) so `findings`/`summary`
        // are non-empty -- an always-empty map would trivially pass an
        // "is it a plain object" check without proving anything.
        { kind: "usfm", sourceKey: "GEN.usfm", book: "GEN", source: GEN },
      ],
    }),
    "shape pin: replaceCorpus",
  );

  // Bytes: real Uint8Array, not number[], on every corpus-grain byte field.
  const published = unwrap(original.publish(), "shape pin: publish");
  assert.ok(published.bytes instanceof Uint8Array, "shape pin: PublishedCorpus.bytes is a Uint8Array");
  const scoped = unwrap(original.publishScope({ kind: "all" }), "shape pin: publishScope");
  assert.ok(scoped.packed instanceof Uint8Array, "shape pin: ScopedPublication.packed is a Uint8Array");
  assert.ok(scoped.sources instanceof Uint8Array, "shape pin: ScopedPublication.sources is a Uint8Array");

  // Maps: plain JS object (constructor === Object), never an ES Map.
  const lint = original.lint();
  assert.ok(lint.summary.byCategory.constructor === Object, "shape pin: LintSummary.byCategory is a plain object");
  assert.ok(!(lint.summary.byCategory instanceof Map), "shape pin: LintSummary.byCategory is not a Map");
  assert.ok(
    Object.keys(lint.summary.byCategory).length > 0,
    "shape pin: fixture actually produces a non-empty byCategory (not a vacuous pass)",
  );
  const finding = lint.books[0].findings.find((f) => Object.keys(f.messageParams).length > 0);
  assert.ok(finding, "shape pin: fixture must produce a finding with non-empty messageParams");
  assert.ok(finding.messageParams.constructor === Object, "shape pin: LintIssue.messageParams is a plain object");
  assert.ok(!(finding.messageParams instanceof Map), "shape pin: LintIssue.messageParams is not a Map");

  // Enum-keyed and nested maps: still plain objects with string keys.
  const left = wasm.parse(GEN);
  const right = wasm.parse(GEN.replace("text\n", "TEXT\n"));
  const byChapter = left.diffByChapter(right, undefined);
  assert.ok(byChapter.constructor === Object, "shape pin: DiffsByChapterMap outer level is a plain object");
  const firstBook = Object.values(byChapter)[0];
  assert.ok(firstBook.constructor === Object, "shape pin: DiffsByChapterMap inner (chapter) level is a plain object");
  assert.ok(
    Object.keys(firstBook).every((key) => typeof key === "string"),
    "shape pin: chapter keys are strings, not numbers wrapped in a Map",
  );

  // VrefMap newtype: Record<string,string>, not wrapped in an extra layer.
  const vrefMap = left.toVref(undefined);
  assert.ok(vrefMap.constructor === Object, "shape pin: VrefMap is a plain object");
  assert.ok(!(vrefMap instanceof Map), "shape pin: VrefMap is not a Map");
  checks += 1;
}

// --- undefined tolerance: own properties holding `undefined` -------------
// The legacy JSON serializer erased `{ attributes: undefined }` before the
// boundary; serde-wasm-bindgen reads it with Reflect.get and hands it to the
// field's deserializer. Structured clone PRESERVES such properties, so
// editor tokens genuinely arrive in this shape (v0.1.5 regression, reported
// by the editor: TypeError out of tokensToUsfm/lintTokens). Every declared
// `?:` field must accept present-but-undefined, and defaultable options
// members must too.
{
  const parsed = wasm.parse(GEN);
  const tokens = parsed.tokens();
  const optional = [
    "span",
    "sid",
    "marker",
    "nested",
    "markerMetadata",
    "structural",
    "numberInfo",
    "bookCode",
    "bookCodeValid",
    "attributes",
    "attributeSource",
    "attributeOffset",
  ];
  const undef = tokens.map((token) => {
    const out = { ...token };
    for (const field of optional) out[field] = out[field] ?? undefined;
    return out;
  });
  assert.ok(
    undef.some((token) => Object.hasOwn(token, "attributes") && token.attributes === undefined),
    "undefined tolerance: fixture actually carries own-property undefined (not a vacuous pass)",
  );
  const viaClean = wasm.tokensToUsfm(tokens);
  assert.equal(
    wasm.tokensToUsfm(undef),
    viaClean,
    "undefined tolerance: tokensToUsfm treats present-but-undefined as absent",
  );
  const lintClean = wasm.lintTokens(tokens, { scope: "book" });
  const lintUndef = wasm.lintTokens(undef, {
    scope: "book",
    enabledCodes: undefined,
    disabledCodes: undefined,
    suppressed: undefined,
    allowImplicitChapterContentVerse: undefined,
  });
  assert.deepEqual(
    lintUndef.summary,
    lintClean.summary,
    "undefined tolerance: lintTokens summary identical with undefined-holding tokens and options",
  );
  wasm.tokensToHtml(tokens, { wrapRoot: undefined });
  checks += 1;
}

console.log(`${target} publish round trip passed: ${checks} checks`);
