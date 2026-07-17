import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv[2] ?? "web";
const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");
const packageUrl = pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href;
const wasmPath = path.join(pkgDir, "usfm_onion_web_bg.wasm");

const pkg = await import(packageUrl);
if (target === "web") {
  const wasmBytes = await readFile(wasmPath);
  await pkg.default({ module_or_path: wasmBytes });
}

const source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";
const edited = `${source}God created the heavens.\n`;

// --- parse + ParsedUsfm methods ----------------------------------------

const parsed = pkg.parse(source);

const tokens = parsed.tokens();
assert.ok(Array.isArray(tokens), "tokens() returns an array");
assert.ok(tokens.length > 0, "tokens() non-empty");
assert.ok(
  tokens.some((t) => t.kind === "marker" && t.marker === "id"),
  "tokens include \\id marker",
);

const cst = parsed.cst();
assert.ok(Array.isArray(cst.tokens), "cst.tokens is an array");
assert.ok(Array.isArray(cst.roots), "cst.roots is an array");

const lintResult = parsed.lint({ scope: "book" });
assert.ok(Array.isArray(lintResult.issues), "lint().issues is an array");
assert.ok(typeof lintResult.summary === "object", "lint().summary is an object");
assert.equal(typeof lintResult.summary.totalCount, "number");

const usfmRoundtrip = parsed.toUsfm();
assert.equal(usfmRoundtrip, source, "toUsfm() round-trips byte-exact");

const usj = parsed.toUsj();
assert.equal(usj.type, "USJ", "toUsj() has type=USJ");
assert.ok(Array.isArray(usj.content), "toUsj().content is an array");

const usx = parsed.toUsx();
assert.match(usx, /<usx/, "toUsx() emits <usx>");

const html = parsed.toHtml({ noteMode: "extracted" });
assert.match(html, /<(article|main|section|p)/, "toHtml() emits a block element");

const vref = parsed.toVref();
assert.equal(typeof vref, "object", "toVref() returns an object");

// --- format ------------------------------------------------------------

const formatted = parsed.format();
assert.equal(typeof formatted, "string");

// --- diff (instance method + standalone) -------------------------------

const editedParsed = pkg.parse(edited);

const skeleton = parsed.diff(editedParsed);
assert.ok(Array.isArray(skeleton.slots), "diff().slots is an array");
assert.ok(Array.isArray(skeleton.units), "diff().units is an array");
assert.ok(skeleton.units.length > 0, "diff() finds at least one unit");

const diffsByChapter = parsed.diffByChapter(editedParsed);
assert.equal(typeof diffsByChapter, "object", "diffByChapter() returns object");
const genChapters = diffsByChapter["GEN"];
assert.ok(genChapters && Array.isArray(genChapters["1"].units), "diffByChapter() nests a skeleton per chapter");

// --- standalone source-in functions ------------------------------------

const lintFromSource = pkg.lintUsfm(source, { scope: "book" });
assert.deepEqual(
  lintFromSource.issues.map((i) => i.code).sort(),
  lintResult.issues.map((i) => i.code).sort(),
  "lintUsfm() and parsed.lint() agree",
);

const formattedFromSource = pkg.formatUsfm(source);
assert.equal(formattedFromSource, formatted, "formatUsfm() matches parsed.format()");

const skeletonFromSource = pkg.diffUsfm(source, edited);
assert.equal(
  skeletonFromSource.units.length,
  skeleton.units.length,
  "diffUsfm() and parsed.diff() agree on unit count",
);

// --- token-in functions ------------------------------------------------

const usfmFromTokens = pkg.tokensToUsfm(tokens);
assert.equal(usfmFromTokens, source, "tokensToUsfm() round-trips");

const htmlFromTokens = pkg.tokensToHtml(tokens);
assert.match(htmlFromTokens, /<(article|main|section|p)/);

const lintFromTokens = pkg.lintTokens(tokens, { scope: "book" });
assert.ok(Array.isArray(lintFromTokens.issues), "lintTokens() returns issues");

const formattedTokens = pkg.formatTokens(tokens);
assert.ok(Array.isArray(formattedTokens.tokens), "formatTokens().tokens is array");
assert.equal(typeof formattedTokens.usfm, "string");

const formattedTokensMut = pkg.formatTokensMut(tokens);
assert.ok(Array.isArray(formattedTokensMut), "formatTokensMut() returns array");

const editedTokens = editedParsed.tokens();
const tokenSkeleton = pkg.diffTokens(tokens, editedTokens);
assert.ok(Array.isArray(tokenSkeleton.units), "diffTokens() returns a skeleton");

// --- token fix flow ----------------------------------------------------

const fixableSource = "\\id GEN\n\\c 1\n\\p\\v 1 Word\n";
const fixableLint = pkg.lintUsfm(fixableSource, { scope: "book" });
const fixableIssue = fixableLint.issues.find((i) => i.fix);
assert.ok(fixableIssue, "fixture produces at least one issue with a fix");
const fixedTokens = pkg.parse(fixableSource).applyTokenFix(fixableIssue.fix);
assert.ok(Array.isArray(fixedTokens), "applyTokenFix() returns tokens");

const standaloneFixed = pkg.applyTokenFix(
  pkg.parse(fixableSource).tokens(),
  fixableIssue.fix,
);
assert.ok(Array.isArray(standaloneFixed), "standalone applyTokenFix() returns tokens");

// --- merge: P2 identities, mixed decisions, revert equivalence ---------

const allBaseline = pkg.mergeDiffBlocks(tokens, editedTokens, { decisions: {}, defaultSide: "baseline" });
assert.equal(pkg.tokensToUsfm(allBaseline), source, "merge all-baseline reproduces source byte-exact (P2)");

const allCurrent = pkg.mergeDiffBlocks(tokens, editedTokens, { decisions: {}, defaultSide: "current" });
assert.equal(pkg.tokensToUsfm(allCurrent), edited, "merge all-current reproduces edited byte-exact (P2)");

const changedUnit = tokenSkeleton.units.find((u) => u.status === "modified");
assert.ok(changedUnit, "fixture produces a modified unit to test mixed decisions against");

const revertedViaMerge = pkg.mergeDiffBlocks(tokens, editedTokens, {
  decisions: { [changedUnit.id]: "baseline" },
  defaultSide: "current",
});
assert.equal(
  pkg.tokensToUsfm(revertedViaMerge),
  source,
  "mixed decision (single unit -> baseline, default current) reverts exactly like source",
);

const reverted = parsed.revertDiffBlock(editedParsed, changedUnit.id);
assert.ok(Array.isArray(reverted), "revertDiffBlock() returns tokens");
assert.equal(
  pkg.tokensToUsfm(reverted),
  pkg.tokensToUsfm(revertedViaMerge),
  "revertDiffBlock() equals a one-decision merge({id: baseline}, current)",
);

const standaloneReverted = pkg.revertDiffBlock(tokens, editedTokens, changedUnit.id);
assert.ok(Array.isArray(standaloneReverted));

// --- move: two-slot identity --------------------------------------------

const moveBaseline = "\\id GEN\n\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n";
const moveCurrent = "\\id GEN\n\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n";
const moveSkeleton = pkg.diffUsfm(moveBaseline, moveCurrent);
const movedUnit = moveSkeleton.units.find((u) => u.kind === "coalesced");
assert.ok(movedUnit, "a pure swap must surface as one coalesced (moved) unit");
assert.equal(movedUnit.status, "moved", "a byte-identical displaced pair is Moved, not Modified");
const movedSlots = moveSkeleton.slots.filter((s) => s.unitId === movedUnit.id);
assert.equal(movedSlots.length, 2, "a moved unit spans exactly two linked slots (one decision, two ghosts)");
assert.deepEqual(
  movedSlots.map((s) => s.role).sort(),
  ["pairBaseline", "pairCurrent"],
  "the two slots are the pair's baseline and current sides",
);

// --- unknown id: strict throw, not a fuzzy fallback ---------------------

function assertThrows(fn, description) {
  try {
    fn();
    assert.fail(`expected ${description} to throw`);
  } catch (error) {
    assert.ok(error instanceof Error, `${description} throws a real Error`);
  }
}

assertThrows(
  () => pkg.mergeDiffBlocks(tokens, editedTokens, { decisions: { "__no_such_unit__": "baseline" }, defaultSide: "current" }),
  "mergeDiffBlocks() on an unknown decision unit id",
);
assertThrows(
  () => pkg.revertDiffBlock(tokens, editedTokens, "__no_such_unit__"),
  "revertDiffBlock() on an unknown block id",
);

// --- marker catalog ----------------------------------------------------

const catalog = pkg.markerCatalog();
const all = catalog.all();
assert.ok(Array.isArray(all), "markerCatalog().all() returns an array");
assert.ok(all.length > 50, "catalog has many entries");
assert.equal(catalog.contains("p"), true, "catalog contains \\p");
assert.equal(catalog.contains("__nonexistent__"), false);
const pInfo = catalog.get("p");
assert.equal(pInfo.marker, "p");

const directInfo = pkg.markerInfo("p");
assert.equal(directInfo.marker, "p");
assert.equal(pkg.isKnownMarker("p"), true);
assert.equal(pkg.isKnownMarker("__nope__"), false);

// --- lint / format introspection --------------------------------------

const lintCodes = pkg.lintCodes();
assert.ok(Array.isArray(lintCodes));
assert.ok(lintCodes.length > 25, "many lint codes registered");
assert.ok(lintCodes.includes("verse-is-empty"), "new lint codes present");

const lintMeta = pkg.lintCodeMeta();
assert.ok(Array.isArray(lintMeta));
assert.equal(lintMeta.length, lintCodes.length);
assert.ok(lintMeta.every((m) => m.category && m.severity && m.issueType));

const formatRules = pkg.formatRules();
assert.ok(Array.isArray(formatRules));
assert.ok(formatRules.length > 5);

const formatMeta = pkg.formatRuleMeta();
assert.ok(Array.isArray(formatMeta));
assert.equal(formatMeta.length, formatRules.length);
assert.ok(formatMeta.every((m) => m.code && m.labelKey));

console.log(`${target} package smoke test passed`);
