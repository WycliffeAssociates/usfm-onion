// Cross-language token equivalence gate plus the verify surface's rejection
// conformance.
//
// Three things are asserted here, and they are the whole reason two token
// decoders are allowed to exist:
//
//   1. Good goldens and every corpus book: the SAME packed bytes materialized by
//      Rust and by JS produce structurally identical token DTOs. The Rust side is
//      authoritative; a divergence is a JS bug.
//   2. Malformed goldens: the wasm verifier rejects each one with the error name
//      the vector recorded, so they never reach the JS decoder at all.
//   3. Chapter-selective materialize returns exactly the slice of the full pass,
//      and an absent chapter is a typed error.
//
// The Rust reference is streamed as one JSON token per line, so a 280,000-token
// book never needs two full copies in memory. Fixtures are written per book and
// deleted immediately after.
//
//   node scripts/test-packed-equivalence.mjs [bundler|web]

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createReadStream } from "node:fs";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { createInterface } from "node:readline";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  PackedError,
  decodeTokens,
  materialize,
  receiptFor,
  reconcileFindings,
  verifyPackedCorpus,
} from "../js/packed.js";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv.includes("web") ? "web" : "bundler";
const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");
const emitter = path.join(rootDir, "target/release/examples/emit_token_equivalence");

const wasm = await import(pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href);
if (target === "web") {
  const wasmBytes = await readFile(path.join(pkgDir, "usfm_onion_web_bg.wasm"));
  await wasm.default({ module_or_path: wasmBytes });
}

const goldenDir = (kind) => path.join(rootDir, "crates/usfm_onion_wire/golden", kind);

/** Rust `Debug` of a `DecodeError` → the tagged `kind` the boundary reports. */
function expectedKind(recorded) {
  const name = recorded.split(/[\s{]/, 1)[0];
  return name[0].toLowerCase() + name.slice(1);
}

async function goldenVectors(kind) {
  const dir = goldenDir(kind);
  const names = (await readdir(dir))
    .filter((file) => file.endsWith(".json"))
    .map((file) => file.slice(0, -".json".length))
    .sort();
  const out = [];
  for (const name of names) {
    const meta = JSON.parse(await readFile(path.join(dir, `${name}.json`), "utf8"));
    out.push({ ...meta, kind, dir, name });
  }
  return out;
}

/**
 * Deep structural equality with JSON semantics: a key whose value is `undefined`
 * is the same as an absent key, because the Rust side omits `None` fields
 * entirely and key order is not semantics.
 */
function structuralEqual(left, right, trail = "") {
  if (left === right) return null;
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right)) return trail || "type";
    if (left.length !== right.length) return `${trail}.length`;
    for (let index = 0; index < left.length; index += 1) {
      const diff = structuralEqual(left[index], right[index], `${trail}[${index}]`);
      if (diff) return diff;
    }
    return null;
  }
  if (left && right && typeof left === "object" && typeof right === "object") {
    const keys = new Set([...Object.keys(left), ...Object.keys(right)]);
    for (const key of keys) {
      if (left[key] === undefined && right[key] === undefined) continue;
      const diff = structuralEqual(left[key], right[key], `${trail}.${key}`);
      if (diff) return diff;
    }
    return null;
  }
  return trail || "value";
}

function emit(args, outDir) {
  const result = spawnSync(emitter, [...args, outDir], { encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(
      `emit_token_equivalence ${args.join(" ")} failed (${result.status}): ${result.stderr}\n` +
        "build it first: cargo build --release --example emit_token_equivalence -p usfm_onion_wire",
    );
  }
}

function verifyOne(caseName, packed, source) {
  const result = verifyPackedCorpus(wasm, packed, source, [
    {
      path: caseName,
      packed: { byteOffset: 0, byteLength: packed.length },
      source: { byteOffset: 0, byteLength: source.length },
    },
  ]);
  assert.equal(
    result.ok,
    true,
    `${caseName}: wasm verification must accept these bytes, got ${JSON.stringify(result.error)}`,
  );
  return result;
}

/**
 * Streams the Rust reference and compares it token by token against the JS
 * materialization of the same bytes.
 */
async function assertTokenEquivalence(caseName, outDir, jsTokens) {
  let index = 0;
  const stream = createInterface({
    input: createReadStream(path.join(outDir, "tokens.jsonl")),
    crlfDelay: Infinity,
  });
  for await (const line of stream) {
    if (line === "") continue;
    const expected = JSON.parse(line);
    assert.ok(index < jsTokens.length, `${caseName}: JS produced only ${jsTokens.length} tokens`);
    const diff = structuralEqual(expected, jsTokens[index]);
    if (diff) {
      assert.fail(
        `${caseName}: token ${index} differs at ${diff}\n  rust: ${line}\n  js:   ${JSON.stringify(jsTokens[index])}`,
      );
    }
    index += 1;
  }
  assert.equal(index, jsTokens.length, `${caseName}: token count`);
  return index;
}

const stats = {
  cases: 0,
  tokens: 0,
  goldensGood: 0,
  goldensMalformed: 0,
  corpus: 0,
  chapters: 0,
  duplicateBook: 0,
  reconciled: 0,
};

/// The first case that produced more than one finding, kept for the reconciliation
/// checks so they run over shapes the boundary actually emits.
let reconciliationSample;
const workRoot = await mkdtemp(path.join(os.tmpdir(), "usfm-onion-equivalence-"));

/** First discovered corpus book with no `\c` at all, `{caseName, verified}` — or `undefined`. */
let noChapterExample;

function sidChapter(sid) {
  return Number(sid.slice(sid.indexOf(" ") + 1, sid.indexOf(":")));
}

/** Every distinct chapter number any token is anchored to, ascending. */
function allChapters(tokens) {
  const seen = new Set();
  for (const token of tokens) {
    if (token.sid === undefined) continue;
    const chapter = sidChapter(token.sid);
    if (Number.isInteger(chapter) && chapter > 0) seen.add(chapter);
  }
  return [...seen].sort((a, b) => a - b);
}

/** First, middle, and final distinct chapters — deduplicated, so a one- or
 * two-chapter book does not run the same check twice under a different name. */
function representativeChapters(chapters) {
  if (chapters.length === 0) return [];
  return [...new Set([chapters[0], chapters[Math.floor(chapters.length / 2)], chapters.at(-1)])];
}

/**
 * The row range `chapter` occupies, derived solely from `tokens`' own `sid`
 * field — never from anything the materializer itself reports. This is what
 * makes the chapter-selective check non-circular: it independently names the
 * rows a correct implementation must return, so a bug in the materializer's
 * own range-finding (e.g. swallowing a neighboring chapter's boundary token,
 * or including front-matter/trailing content that carries no chapter sid)
 * cannot also produce the "expected" answer.
 */
function expectedChapterRange(tokens, chapter) {
  let first = -1;
  let last = -1;
  for (let index = 0; index < tokens.length; index += 1) {
    const sid = tokens[index].sid;
    if (sid === undefined || sidChapter(sid) !== chapter) continue;
    if (first < 0) first = index;
    last = index;
  }
  return first < 0 ? null : { start: first, end: last };
}

/** One good case: verify, materialize both ways, compare. */
async function runGoodCase(caseName, emitArgs, options = {}) {
  const outDir = path.join(workRoot, "case");
  await rm(outDir, { recursive: true, force: true });
  emit(emitArgs, outDir);
  const packed = new Uint8Array(await readFile(path.join(outDir, "packed.bin")));
  const source = new Uint8Array(await readFile(path.join(outDir, "source.usfm")));
  const expectedStableIds = JSON.parse(await readFile(path.join(outDir, "stable-ids.json"), "utf8"));

  const { verified, findings } = verifyOne(caseName, packed, source);
  const caseFindings = findings.get(caseName) ?? [];
  if (reconciliationSample === undefined && caseFindings.length > 1) {
    reconciliationSample = caseFindings;
  }
  const receipt = receiptFor(verified, caseName).receipt;
  const book = decodeTokens(verified, caseName);
  assert.equal(
    book.tokens.length,
    receipt.tokenCount,
    `${caseName}: receipt tokenCount matches the materialized stream`,
  );
  assert.equal(
    (findings.get(caseName) ?? []).length,
    receipt.findingCount,
    `${caseName}: receipt findingCount matches the returned findings`,
  );
  assert.deepEqual(
    book.stableIds ?? null,
    expectedStableIds,
    `${caseName}: explicit stable ids match the Rust decoder`,
  );
  assert.equal(
    receipt.positionalIds,
    expectedStableIds === null,
    `${caseName}: positionalIds flag agrees with the absence of explicit ids`,
  );
  const count = await assertTokenEquivalence(caseName, outDir, book.tokens);
  stats.cases += 1;
  stats.tokens += count;

  // `materialize()` with no selector must agree with the single-book entry.
  const all = materialize(verified);
  assert.equal(all.size, 1, `${caseName}: one verified book`);
  assert.equal(
    structuralEqual(all.get(caseName).tokens, book.tokens),
    null,
    `${caseName}: materialize() and decodeTokens() agree`,
  );

  // Chapters are derived from the stream rather than assumed: a corpus can
  // legitimately hold a front-matter-only book with no `\\c` at all. Remember
  // the first one found so the explicit no-chapter assertion below (after the
  // corpus loop) has a real case rather than only the passive coverage of it
  // flowing through the full-materialize assertions above.
  const chapterList = allChapters(book.tokens);
  if (options.selective && chapterList.length === 0 && noChapterExample === undefined) {
    noChapterExample = { caseName, verified };
  }
  const chapters = options.selective ? representativeChapters(chapterList) : [];
  for (const chapter of chapters) {
    // Independently derived — never read off the implementation's own result —
    // so this is a real check of where the boundary falls, including that
    // front-matter/trailing content outside any chapter is excluded.
    const expected = expectedChapterRange(book.tokens, chapter);
    assert.ok(expected, `${caseName}: chapter ${chapter} must appear in the full materialize`);
    const selective = materialize(verified, { path: caseName, chapter });
    const slice = selective.get(caseName);
    assert.equal(
      slice.range,
      undefined,
      `${caseName}: MaterializedBook carries no range field (frozen shape)`,
    );
    assert.equal(
      structuralEqual(slice.tokens, book.tokens.slice(expected.start, expected.end + 1)),
      null,
      `${caseName}: chapter ${chapter} is exactly the independently-derived slice of the full materialize`,
    );
    assert.ok(slice.tokens.length > 0, `${caseName}: chapter ${chapter} is non-empty`);
    // Selection by book code works when it is unambiguous.
    const byBook = materialize(verified, { book: receipt.book, chapter });
    assert.equal(
      structuralEqual(byBook.get(caseName).tokens, slice.tokens),
      null,
      `${caseName}: selecting by book code resolves to the same rows`,
    );
    stats.chapters += 1;
  }
  if (chapters.length > 0) {
    assert.throws(
      () => materialize(verified, { path: caseName, chapter: 9999 }),
      (error) => error instanceof PackedError && error.kind === "unknownChapter",
      `${caseName}: an out-of-range chapter is a typed error`,
    );
  }
  await rm(outDir, { recursive: true, force: true });
}

// --- 1 + 3: good goldens ----------------------------------------------------

for (const kind of ["token", "finding"]) {
  for (const vector of await goldenVectors(kind)) {
    if (vector.base !== undefined) continue; // malformed, handled below
    const caseName = `golden/${kind}/${vector.name}`;
    await runGoodCase(caseName, [
      "bin",
      path.join(vector.dir, `${vector.name}.bin`),
      path.join(vector.dir, `${vector.name}.usfm`),
    ]);
    stats.goldensGood += 1;
  }
}

// --- 2: malformed goldens are refused at the trust boundary ------------------

for (const kind of ["token", "finding"]) {
  for (const vector of await goldenVectors(kind)) {
    if (vector.expectedError === undefined) continue;
    const caseName = `golden/${kind}/${vector.name}`;
    const packed = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.bin`)));
    const source = new Uint8Array(
      await readFile(path.join(vector.dir, `${vector.base}.usfm`)),
    );
    const result = verifyPackedCorpus(wasm, packed, source, [
      {
        path: caseName,
        packed: { byteOffset: 0, byteLength: packed.length },
        source: { byteOffset: 0, byteLength: source.length },
      },
    ]);
    assert.equal(result.ok, false, `${caseName}: malformed bytes must be refused`);
    assert.equal(
      result.error.kind,
      expectedKind(vector.expectedError),
      `${caseName}: refused with the recorded error`,
    );
    assert.equal(result.path, caseName, `${caseName}: the rejection names its record`);
    stats.goldensMalformed += 1;
  }
}

// --- 1: corpus books --------------------------------------------------------

async function collectUsfm(dir, out) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await collectUsfm(full, out);
    else if (entry.name.endsWith(".usfm")) out.push(full);
  }
  return out;
}

const corpusRoots = ["testData", "example-corpora/en_ult", "example-corpora/en_ulb"];
let corpusPaths = [];
for (const root of corpusRoots) {
  corpusPaths = await collectUsfm(path.join(rootDir, root), corpusPaths);
}
corpusPaths.sort();
assert.ok(corpusPaths.length > 0, "the corpus must resolve from the repo root");

for (const file of corpusPaths) {
  const caseName = path.relative(rootDir, file);
  await runGoodCase(caseName, ["usfm", file], { selective: true });
  stats.corpus += 1;
}

// --- selector faults and forged-handle rejection -----------------------------

{
  const vector = (await goldenVectors("token")).find((entry) => entry.base === undefined);
  const packed = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.bin`)));
  const source = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.usfm`)));
  const { verified } = verifyOne("only", packed, source);
  assert.throws(
    () => decodeTokens(verified, "nope"),
    (error) => error instanceof PackedError && error.kind === "unknownBook",
    "an unknown path is a typed error",
  );

  // The handle is opaque: verification state lives in a module-private
  // WeakMap keyed by the handle's identity, and the handle itself carries no
  // own properties at all — genuine or not. A look-alike built from the
  // genuine handle's own property *and* symbol keys is therefore
  // indistinguishable by shape (both are empty objects), so its rejection
  // proves the check is real WeakMap membership, not anything inspectable on
  // the value.
  const genuineKeys = [
    ...Object.getOwnPropertyNames(verified),
    ...Object.getOwnPropertySymbols(verified),
  ];
  const forged = Object.freeze(Object.fromEntries(genuineKeys.map((key) => [key, verified[key]])));
  assert.notEqual(forged, verified, "the look-alike is a different object than the genuine handle");
  assert.throws(
    () => materialize(forged),
    (error) => error instanceof PackedError,
    "materialize refuses a look-alike built from the genuine handle's own keys",
  );
  assert.throws(
    () => decodeTokens(forged, vector.name),
    (error) => error instanceof PackedError,
    "decodeTokens refuses the same look-alike",
  );
}

// --- certification outlives caller-side mutation -----------------------------

{
  const vector = (await goldenVectors("token")).find((entry) => entry.base === undefined);
  const packed = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.bin`)));
  const source = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.usfm`)));
  const caseName = "mutation";
  const { verified } = verifyOne(caseName, packed, source);
  const before = decodeTokens(verified, caseName);

  // Mutate the caller's own arrays after minting. If the handle's state held
  // these same buffers rather than copies, the certification would now be
  // stale.
  packed.fill(0xff);
  source.fill(0x2a); // '*' — still valid UTF-8, so a copy bug wouldn't be
  // masked by a decode failure; it would just silently return different text.

  const after = decodeTokens(verified, caseName);
  assert.equal(
    structuralEqual(before.tokens, after.tokens),
    null,
    "mutating the caller's packed/source arrays after minting must not change materialized output",
  );
}

// --- descriptor tree is frozen: marker metadata mutation is rejected --------

{
  const vector = (await goldenVectors("token")).find((entry) => entry.base === undefined);
  const packed = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.bin`)));
  const source = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.usfm`)));
  const caseName = "frozen-metadata";
  const { verified } = verifyOne(caseName, packed, source);
  const before = decodeTokens(verified, caseName);
  const markerToken = before.tokens.find((token) => token.markerMetadata !== undefined);
  assert.ok(
    markerToken,
    `${caseName}: at least one token must carry markerMetadata for this check to mean anything`,
  );
  assert.throws(
    () => {
      markerToken.markerMetadata.canonical = "evil";
    },
    TypeError,
    "mutating a materialized token's markerMetadata throws (the descriptor tree is deep-frozen)",
  );
  // Every token sharing this marker form attaches the same frozen object, so
  // the attempted mutation above (rejected, not silently applied) must not
  // have leaked into a second materialization of the same book either.
  const after = decodeTokens(verified, caseName);
  assert.equal(
    structuralEqual(before.tokens, after.tokens),
    null,
    "materializing again after the rejected mutation attempt is unaffected",
  );
}

// --- duplicate-book ambiguity -------------------------------------------------

{
  const vector = (await goldenVectors("token")).find((entry) => entry.base === undefined);
  const packed = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.bin`)));
  const source = new Uint8Array(await readFile(path.join(vector.dir, `${vector.name}.usfm`)));
  // Two records at different paths naming the same underlying book: a legal
  // corpus (paths, the real key, are distinct) that makes book-code selection
  // ambiguous.
  // Both records name the same underlying bytes, so both extents point at
  // the same range of the one shared buffer -- no need to duplicate them.
  const sharedExtent = (buf) => ({ byteOffset: 0, byteLength: buf.length });
  const result = verifyPackedCorpus(wasm, packed, source, [
    { path: "dup/a", packed: sharedExtent(packed), source: sharedExtent(source) },
    { path: "dup/b", packed: sharedExtent(packed), source: sharedExtent(source) },
  ]);
  assert.equal(result.ok, true, "two records naming the same book at different paths verify fine");
  const book = receiptFor(result.verified, "dup/a").receipt.book;
  assert.throws(
    () => materialize(result.verified, { book }),
    (error) => error instanceof PackedError && error.kind === "ambiguousBook",
    "selecting by book code is a typed error when two records share it",
  );
  stats.duplicateBook += 1;
}

// --- no-\c book: chapter selection is a typed error --------------------------

{
  // A book with no `\c` at all has nothing for `{chapter}` to locate: assert
  // that explicitly rather than only relying on such a book passively flowing
  // through the plain full-materialize assertions. Prefer a real corpus
  // example (`noChapterExample`, captured while the corpus loop ran); fall
  // back to a tiny synthetic fixture if the corpus happens to have none.
  let caseName;
  let verified;
  if (noChapterExample !== undefined) {
    ({ caseName, verified } = noChapterExample);
  } else {
    caseName = "no-chapter-fixture";
    const outDir = path.join(workRoot, "no-chapter-fixture");
    await rm(outDir, { recursive: true, force: true });
    await mkdir(outDir, { recursive: true });
    const fixtureUsfm = path.join(outDir, "fixture.usfm");
    await writeFile(fixtureUsfm, "\\id GEN\n\\h Genesis\n", "utf8");
    emit(["usfm", fixtureUsfm], outDir);
    const packed = new Uint8Array(await readFile(path.join(outDir, "packed.bin")));
    const source = new Uint8Array(await readFile(path.join(outDir, "source.usfm")));
    ({ verified } = verifyOne(caseName, packed, source));
    await rm(outDir, { recursive: true, force: true });
  }
  const book = decodeTokens(verified, caseName);
  assert.equal(
    allChapters(book.tokens).length,
    0,
    `${caseName}: this case must genuinely have no chapter, or the assertion below proves nothing`,
  );
  assert.throws(
    () => materialize(verified, { path: caseName, chapter: 1 }),
    (error) => error instanceof PackedError && error.kind === "unknownChapter",
    `${caseName}: selecting a chapter in a book with no \\c is a typed error`,
  );
}

// Finding reconciliation, over findings that came out of the trust boundary rather
// than hand-built ones: the identity rule has to hold for the real shapes, including
// findings that carry a fix and findings anchored to the same token by two rules.
{
  const findings = reconciliationSample;
  assert.ok(findings.length > 1, "the corpus must supply findings to reconcile");
  const untouched = reconcileFindings(findings, findings.map((finding) => ({ ...finding })));
  assert.equal(
    untouched,
    findings,
    "an unchanged pass returns the previous array itself, so a consumer can skip a re-render",
  );

  const edited = findings.map((finding, index) =>
    index === 0 ? { ...finding, message: `${finding.message} (edited)` } : { ...finding },
  );
  const reconciled = reconcileFindings(findings, edited);
  assert.notEqual(reconciled[0], findings[0], "a changed finding is a fresh object");
  for (let index = 1; index < findings.length; index += 1) {
    assert.equal(
      reconciled[index],
      findings[index],
      "every unchanged finding keeps its identity across a recompute",
    );
  }

  const reAnchored = reconcileFindings(findings, [{ ...findings[0], tokenId: "no-such-token" }]);
  assert.notEqual(
    reAnchored[0],
    findings[0],
    "a finding anchored to a different token is a different finding",
  );
  stats.reconciled = findings.length;
}

// reconcileFindings must not resurrect a finding that is genuinely gone when
// two previous findings share one identity (same code/tokenId/relatedTokenId)
// but differ in content — one-to-one consumption, never a single reusable
// slot per identity.
{
  const base = { code: "test-code", tokenId: "t-1", severity: "warning", marker: null, span: null, relatedSpan: null };
  const findingA = { ...base, message: "message A" };
  const findingB = { ...base, message: "message B" };

  // Clean-room review's exact repro: previous [A, B] (same identity,
  // different messages); next asks for A twice. The second "A" must not
  // resurrect B by matching the wrong pooled candidate.
  const cloneA = () => ({ ...findingA });
  const reconciledAA = reconcileFindings([findingA, findingB], [cloneA(), cloneA()]);
  assert.equal(reconciledAA.length, 2, "two requested findings, two results");
  assert.equal(reconciledAA[0], findingA, "the first A reuses the original A's identity");
  assert.notEqual(
    reconciledAA[1],
    findingB,
    "the second A must never resurrect B -- one-to-one consumption, not a reusable single slot",
  );
  assert.notEqual(reconciledAA[1], findingA, "A was already consumed by the first slot");
  assert.equal(reconciledAA[1].message, "message A");

  // Counts differ in both directions, same identity, same content each time:
  // fewer next findings than previous leaves the extras un-reused (not
  // resurrected into the output); more next findings than previous leaves the
  // extras as fresh objects (nothing left in the pool to reuse).
  const findingA2 = { ...findingA };
  const fewerNext = reconcileFindings([findingA, findingA2, { ...findingA }], [cloneA()]);
  assert.equal(fewerNext.length, 1);
  assert.equal(fewerNext[0], findingA, "the one requested slot reuses the first pooled candidate");

  const moreNext = reconcileFindings([findingA, findingA2], [cloneA(), cloneA(), cloneA()]);
  assert.equal(moreNext.length, 3);
  assert.equal(moreNext[0], findingA);
  assert.equal(moreNext[1], findingA2);
  assert.notEqual(moreNext[2], findingA, "no third pooled candidate exists to reuse");
  assert.notEqual(moreNext[2], findingA2, "no third pooled candidate exists to reuse");
}

// The "return previous itself" shortcut must be slot-based, not count-based:
// a genuine reorder consumes every candidate one-to-one just like the
// unchanged case, so a count-only check can't distinguish them and would
// wrongly hand back `previous` in its stale order.
{
  const base = { code: "test-code", severity: "warning", marker: null, span: null, relatedSpan: null };
  const findingA = { ...base, tokenId: "t-1", message: "message A" };
  const findingB = { ...base, tokenId: "t-2", message: "message B" };

  const reordered = reconcileFindings(
    [findingA, findingB],
    [{ ...findingB }, { ...findingA }],
  );
  assert.equal(reordered.length, 2, "two requested findings, two results");
  assert.equal(reordered[0], findingB, "slot 0 must reuse B, not fall through to previous's slot 0 (A)");
  assert.equal(reordered[1], findingA, "slot 1 must reuse A, not fall through to previous's slot 1 (B)");

  // The unchanged (same order) case still takes the shortcut and returns
  // `previous` itself, object-identical.
  const previous = [findingA, findingB];
  const unchanged = reconcileFindings(previous, [{ ...findingA }, { ...findingB }]);
  assert.equal(unchanged, previous, "an unchanged order still returns previous itself");
}

await rm(workRoot, { recursive: true, force: true });

console.log(
  `${target} packed equivalence passed: ${stats.cases} cases / ${stats.tokens} tokens ` +
    `(${stats.goldensGood} good goldens, ${stats.goldensMalformed} malformed goldens refused, ` +
    `${stats.corpus} corpus books, ` +
    `${stats.chapters} chapter-selective slices, ` +
    `${stats.duplicateBook} duplicate-book ambiguity check, ` +
    `${stats.reconciled} findings reconciled)`,
);
