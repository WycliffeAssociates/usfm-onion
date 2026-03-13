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

const source = "\\\\id GEN\\n\\\\c 1\\n\\\\p\\n\\\\v 1 In the beginning\\n";
const parsed = pkg.parseContent({
  source,
  format: "usfm",
});

assert.equal(parsed.bookCode, "GEN");

const html = pkg.intoHtml(parsed, {
  noteMode: "extracted",
});
assert.match(html, /<article|<main|<section|<p/);

const usj = pkg.intoUsj(parsed);
assert.equal(usj.type, "USJ");

const ast = pkg.intoAst(parsed);
assert.ok(Array.isArray(ast.content));

const directAst = pkg.usfmToAst(source);
assert.ok(Array.isArray(directAst.content));

const usx = pkg.intoUsx({ document: parsed });
assert.match(usx, /<usx/);

const tokenList = pkg.usfmToTokens(source);
assert.ok(Array.isArray(tokenList));
assert.equal(pkg.tokensToUsfm(tokenList), source);

const variants = pkg.classifyTokens(tokenList);
assert.ok(Array.isArray(variants));
assert.ok(variants.some((variant) => variant.type === "marker"));

const tokenAst = pkg.tokensToAst(tokenList);
assert.ok(Array.isArray(tokenAst.content));

const flattenedAstTokens = pkg.astToTokens(tokenAst);
assert.ok(Array.isArray(flattenedAstTokens));

const issues = pkg.lintContent({
  source,
  format: "usfm",
});
assert.ok(Array.isArray(issues));

const tokenIssues = pkg.lintFlatTokens({
  tokens: tokenList,
});
assert.ok(Array.isArray(tokenIssues));

const fixableIssues = pkg.lintContent({
  source: "\\\\id GEN\\n\\\\c 1\\n\\\\ptext\\n",
  format: "usfm",
});
assert.ok(
  fixableIssues.some((issue) => issue.fix && issue.fix.type === "replaceToken"),
);

const formatted = pkg.formatContent({
  source,
  format: "usfm",
});
assert.ok(formatted.tokens.length > 0);

const formattedTokens = pkg.formatFlatTokens({
  tokens: tokenList,
});
assert.ok(formattedTokens.tokens.length > 0);

const diffs = pkg.diffUsfm({
  baselineUsfm: source,
  currentUsfm: `${source}God created\\n`,
});
assert.ok(Array.isArray(diffs));

const tokenDiffs = pkg.diffTokens({
  baselineTokens: tokenList,
  currentTokens: pkg.usfmToTokens(`${source}God created\\n`),
});
assert.ok(Array.isArray(tokenDiffs));

const reverted = pkg.revertDiffBlock({
  blockId: diffs[0]?.blockId ?? "GEN 1:1",
  baselineTokens: tokenList,
  currentTokens: pkg.usfmToTokens(`${source}God created\\n`),
});
assert.ok(Array.isArray(reverted));

console.log(`${target} package smoke test passed`);
