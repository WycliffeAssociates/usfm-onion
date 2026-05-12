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

const lintResult = parsed.lint();
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

const diffs = parsed.diff(editedParsed);
assert.ok(Array.isArray(diffs), "diff() returns an array");

const diffsByChapter = parsed.diffByChapter(editedParsed);
assert.equal(typeof diffsByChapter, "object", "diffByChapter() returns object");

// --- standalone source-in functions ------------------------------------

const lintFromSource = pkg.lintUsfm(source);
assert.deepEqual(
  lintFromSource.issues.map((i) => i.code).sort(),
  lintResult.issues.map((i) => i.code).sort(),
  "lintUsfm() and parsed.lint() agree",
);

const formattedFromSource = pkg.formatUsfm(source);
assert.equal(formattedFromSource, formatted, "formatUsfm() matches parsed.format()");

const diffsFromSource = pkg.diffUsfm(source, edited);
assert.equal(
  diffsFromSource.length,
  diffs.length,
  "diffUsfm() and parsed.diff() agree on block count",
);

// --- token-in functions ------------------------------------------------

const usfmFromTokens = pkg.tokensToUsfm(tokens);
assert.equal(usfmFromTokens, source, "tokensToUsfm() round-trips");

const htmlFromTokens = pkg.tokensToHtml(tokens);
assert.match(htmlFromTokens, /<(article|main|section|p)/);

const lintFromTokens = pkg.lintTokens(tokens);
assert.ok(Array.isArray(lintFromTokens.issues), "lintTokens() returns issues");

const formattedTokens = pkg.formatTokens(tokens);
assert.ok(Array.isArray(formattedTokens.tokens), "formatTokens().tokens is array");
assert.equal(typeof formattedTokens.usfm, "string");

const formattedTokensMut = pkg.formatTokensMut(tokens);
assert.ok(Array.isArray(formattedTokensMut), "formatTokensMut() returns array");

const editedTokens = editedParsed.tokens();
const tokenDiffs = pkg.diffTokens(tokens, editedTokens);
assert.ok(Array.isArray(tokenDiffs), "diffTokens() returns an array");

// --- token fix flow ----------------------------------------------------

const fixableSource = "\\id GEN\n\\c 1\n\\p\\v 1 Word\n";
const fixableLint = pkg.lintUsfm(fixableSource);
const fixableIssue = fixableLint.issues.find((i) => i.fix);
assert.ok(fixableIssue, "fixture produces at least one issue with a fix");
const fixedTokens = pkg.parse(fixableSource).applyTokenFix(fixableIssue.fix);
assert.ok(Array.isArray(fixedTokens), "applyTokenFix() returns tokens");

const standaloneFixed = pkg.applyTokenFix(
  pkg.parse(fixableSource).tokens(),
  fixableIssue.fix,
);
assert.ok(Array.isArray(standaloneFixed), "standalone applyTokenFix() returns tokens");

// --- revert diff block -------------------------------------------------

if (diffs.length > 0) {
  const reverted = parsed.revertDiffBlock(editedParsed, diffs[0].blockId);
  assert.ok(Array.isArray(reverted), "revertDiffBlock() returns tokens");

  const standaloneReverted = pkg.revertDiffBlock(
    tokens,
    editedTokens,
    diffs[0].blockId,
  );
  assert.ok(Array.isArray(standaloneReverted));

  const standaloneRevertedMany = pkg.revertDiffBlocks(
    tokens,
    editedTokens,
    [diffs[0].blockId],
  );
  assert.ok(Array.isArray(standaloneRevertedMany));
}

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
assert.ok(lintCodes.length > 30, "many lint codes registered");
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
