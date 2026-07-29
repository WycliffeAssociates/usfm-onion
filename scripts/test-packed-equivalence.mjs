// Cross-language token equivalence gate (freeze §H, narrowed to tokens by §I.5)
// plus the verify surface's rejection conformance.
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
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { createInterface } from "node:readline";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { PackedError, decodeTokens, materialize, verifyPackedCorpus } from "../js/packed.js";

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
  const result = verifyPackedCorpus(wasm, [{ path: caseName, packed, source }]);
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

const stats = { cases: 0, tokens: 0, goldensGood: 0, goldensMalformed: 0, corpus: 0, chapters: 0 };
const workRoot = await mkdtemp(path.join(os.tmpdir(), "usfm-onion-equivalence-"));

/** The first chapter any token is anchored to, or `undefined` for a book with none. */
function firstChapter(tokens) {
  for (const token of tokens) {
    if (token.sid === undefined) continue;
    const chapter = Number(token.sid.slice(token.sid.indexOf(" ") + 1, token.sid.indexOf(":")));
    if (Number.isInteger(chapter) && chapter > 0) return chapter;
  }
  return undefined;
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
  const receipt = verified.books.get(caseName).receipt;
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

  // The chapter is derived from the stream rather than assumed: a corpus can
  // legitimately hold a front-matter-only book with no `\\c` at all.
  const chapter = options.selective ? firstChapter(book.tokens) : undefined;
  if (chapter !== undefined) {
    const selective = materialize(verified, { path: caseName, chapter });
    const slice = selective.get(caseName);
    const { start, end } = slice.range;
    assert.equal(
      structuralEqual(slice.tokens, book.tokens.slice(start, end + 1)),
      null,
      `${caseName}: chapter ${chapter} is exactly the slice of the full materialize`,
    );
    assert.ok(slice.tokens.length > 0, `${caseName}: chapter ${chapter} is non-empty`);
    // Selection by book code works when it is unambiguous.
    const byBook = materialize(verified, { book: receipt.book, chapter });
    assert.equal(
      structuralEqual(byBook.get(caseName).tokens, slice.tokens),
      null,
      `${caseName}: selecting by book code resolves to the same rows`,
    );
    assert.throws(
      () => materialize(verified, { path: caseName, chapter: 9999 }),
      (error) => error instanceof PackedError && error.kind === "unknownChapter",
      `${caseName}: an out-of-range chapter is a typed error`,
    );
    stats.chapters += 1;
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
    const result = verifyPackedCorpus(wasm, [{ path: caseName, packed, source }]);
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

// --- selector and brand faults ----------------------------------------------

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
  assert.throws(
    () => materialize({ books: verified.books }),
    (error) => error instanceof PackedError,
    "materialize refuses an unbranded handle",
  );
}

await rm(workRoot, { recursive: true, force: true });

console.log(
  `${target} packed equivalence passed: ${stats.cases} cases / ${stats.tokens} tokens ` +
    `(${stats.goldensGood} good goldens, ${stats.goldensMalformed} malformed goldens refused, ` +
    `${stats.corpus} corpus books, ` +
    `${stats.chapters} chapter-selective slices)`,
);
