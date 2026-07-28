import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  normalizeTokenSids as normalizePlain,
  normalizeTokenSidsMut,
} from "usfm-onion-web/token-sids";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv[2] ?? "bundler";
const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");
const wasmPackage = await import(
  pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href
);

if (target === "web") {
  const wasmBytes = await readFile(path.join(pkgDir, "usfm_onion_web_bg.wasm"));
  await wasmPackage.default({ module_or_path: wasmBytes });
}

function marker(marker, source = `\\${marker} `) {
  return { id: `marker-${marker}`, kind: "marker", marker, source };
}

function number(start, end, source = String(start)) {
  return {
    id: `number-${start}-${end ?? "single"}`,
    kind: "number",
    source,
    numberInfo: end === undefined
      ? { start, kind: "single" }
      : { start, end, kind: "range" },
  };
}

function text(id, source, extra = {}) {
  return { id, kind: "text", source, ...extra };
}

const fixtures = [
  {
    name: "intro, chapter, verse, bridge, duplicates, and chapter reset",
    bookCode: "GEN",
    tokens: [
      marker("id"),
      { id: "wrong-book", kind: "bookCode", source: "EXO", bookCode: "EXO" },
      { id: "intro-newline", kind: "newline", source: "\r\n" },
      text("intro", "Introduction", { sid: "STALE 9:9", arbitrary: { keep: true } }),
      marker("c"), number(1),
      marker("v"), number(1), text("v1", "one"),
      marker("v"), number(1, 2, "1-2 "), text("bridge-1", "bridge one"),
      marker("v"), number(1), text("v1-dup", "one duplicate"),
      marker("v"), number(1, 2, "1-2 "), text("bridge-dup", "bridge duplicate"),
      marker("c"), number(2),
      marker("v"), number(1), text("chapter-reset", "one again"),
      marker("v"), number(1, 2, "1-2 "), text("bridge-reset", "bridge again"),
    ],
  },
  {
    name: "malformed and absent marker payloads remain on the current boundary",
    bookCode: "MAT",
    tokens: [
      text("leading", "leading", { sid: "", irrelevant: ["preserve"] }),
      marker("c"),
      text("not-a-number", "oops"),
      marker("v"),
      { id: "newline", kind: "newline", source: "\n" },
      marker("c"), number(4),
      marker("v"), text("still-not-a-number", "oops"),
      text("tail", "tail"),
    ],
  },
  {
    name: "a repeated chapter label gets a positional _cdup_ suffix and resets verse dup counting",
    bookCode: "GEN",
    tokens: [
      marker("c"), number(1),
      marker("v"), number(1), text("first-a", "a"),
      marker("v"), number(1), text("first-b", "b"),
      marker("c"), number(1),
      marker("v"), number(1), text("second-a", "c"),
      marker("v"), number(1), text("second-b", "d"),
      marker("c"), number(1),
      marker("v"), number(1), text("third-a", "e"),
    ],
  },
  {
    name: "a stream with no chapter or verse structure stays in intro",
    bookCode: "RUT",
    tokens: [
      marker("id"),
      { id: "book", kind: "bookCode", source: "RUT", bookCode: "RUT" },
      { id: "newline", kind: "newline", source: "\n" },
      marker("h"),
      text("heading", "Ruth", { sid: "STALE 4:2" }),
      marker("p"),
      text("body", "context-free body"),
    ],
  },
];

for (const fixture of fixtures) {
  const before = structuredClone(fixture.tokens);
  const plain = normalizePlain(fixture.tokens, fixture.bookCode);
  const rust = wasmPackage.normalizeTokenSids(fixture.tokens, fixture.bookCode);

  assert.deepEqual(
    plain.map((token) => token.sid),
    rust.map((token) => token.sid),
    `${fixture.name}: plain JS matches live Rust/wasm SID derivation`,
  );
  assert.deepEqual(fixture.tokens, before, `${fixture.name}: input remains untouched`);
  assert.deepEqual(
    normalizePlain(plain, fixture.bookCode),
    plain,
    `${fixture.name}: normalization is idempotent`,
  );
  assert.equal(
    plain.map((token) => token.source).join(""),
    fixture.tokens.map((token) => token.source).join(""),
    `${fixture.name}: serialized USFM bytes remain unchanged`,
  );

  for (const [index, normalized] of plain.entries()) {
    const { sid: _beforeSid, ...beforeWithoutSid } = fixture.tokens[index];
    const { sid: _afterSid, ...afterWithoutSid } = normalized;
    assert.deepEqual(
      afterWithoutSid,
      beforeWithoutSid,
      `${fixture.name}: token ${index} preserves every non-sid field`,
    );
  }

  // Mutable twin: same sids, but mutates the caller's own array in place
  // rather than allocating a clone.
  const mutInput = structuredClone(fixture.tokens);
  const returnValue = normalizeTokenSidsMut(mutInput, fixture.bookCode);
  assert.equal(
    returnValue,
    undefined,
    `${fixture.name}: normalizeTokenSidsMut returns nothing, it mutates`,
  );
  assert.deepEqual(
    mutInput.map((token) => token.sid),
    plain.map((token) => token.sid),
    `${fixture.name}: normalizeTokenSidsMut matches normalizeTokenSids' sids`,
  );
  for (const [index, token] of mutInput.entries()) {
    const { sid: _mutSid, ...mutWithoutSid } = token;
    const { sid: _beforeSid, ...beforeWithoutSid } = fixture.tokens[index];
    assert.deepEqual(
      mutWithoutSid,
      beforeWithoutSid,
      `${fixture.name}: normalizeTokenSidsMut token ${index} preserves every non-sid field`,
    );
  }
}

const repeatedChapterFixture = fixtures.find((fixture) =>
  fixture.name.startsWith("a repeated chapter label")
);
const repeatedChapterNormalized = normalizePlain(
  repeatedChapterFixture.tokens,
  repeatedChapterFixture.bookCode,
);
const sidOfText = (text) =>
  repeatedChapterNormalized.find((token) => token.source === text)?.sid;
assert.deepEqual(
  {
    a: sidOfText("a"),
    b: sidOfText("b"),
    c: sidOfText("c"),
    d: sidOfText("d"),
    e: sidOfText("e"),
  },
  {
    a: "GEN 1:1",
    b: "GEN 1:1_dup_1",
    c: "GEN 1:1_cdup_1",
    d: "GEN 1:1_cdup_1_dup_1",
    e: "GEN 1:1_cdup_2",
  },
  "repeated chapter occurrences get positional _cdup_N suffixes and reset verse dup counting",
);

const noStructureFixture = fixtures.find((fixture) =>
  fixture.name.startsWith("a stream with no chapter or verse structure")
);
assert.ok(
  normalizePlain(noStructureFixture.tokens, noStructureFixture.bookCode)
    .every((token) => token.sid === "RUT 0:0"),
  "a stream with no chapter or verse structure stays entirely at BOOK 0:0",
);

const carriedSidVariants = fixtures[0].tokens.map((token, index) => ({
  ...token,
  sid: index % 2 === 0 ? undefined : `ARBITRARY ${index}:255`,
}));
assert.deepEqual(
  normalizePlain(carriedSidVariants, "GEN").map((token) => token.sid),
  normalizePlain(fixtures[0].tokens, "GEN").map((token) => token.sid),
  "missing and arbitrary carried SIDs do not affect canonical output",
);
assert.ok(
  normalizePlain(fixtures[0].tokens, "LEV").every((token) => token.sid.startsWith("LEV ")),
  "the explicit book code overrides embedded book-code tokens",
);

console.log(`${target} token SID conformance passed (${fixtures.length} fixture streams)`);
