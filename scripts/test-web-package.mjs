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

const usx = pkg.intoUsx({ document: parsed });
assert.match(usx, /<usx/);

const issues = pkg.lintContent({
  source,
  format: "usfm",
});
assert.ok(Array.isArray(issues));

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

const diffs = pkg.diffUsfm({
  baselineUsfm: source,
  currentUsfm: `${source}God created\\n`,
});
assert.ok(Array.isArray(diffs));

const reverted = pkg.revertDiffBlock({
  blockId: diffs[0]?.blockId ?? "GEN 1:1",
  baselineTokens: pkg.intoTokensFromContent({ source, format: "usfm" }),
  currentTokens: pkg.intoTokensFromContent({
    source: `${source}God created\\n`,
    format: "usfm",
  }),
});
assert.ok(Array.isArray(reverted));

console.log(`${target} package smoke test passed`);
