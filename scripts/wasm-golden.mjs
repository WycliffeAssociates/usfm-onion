// Golden parity harness for the wasm package.
//
// Captures the wire-format output of every public wasm function on a fixed
// fixture corpus. The on-disk goldens under `crates/usfm_onion_wasm/golden/`
// are the fidelity contract for `docs/plan-wasm-bindings.md` — any schema or
// value change inside the wasm crate that alters output is caught here.
//
// Usage:
//   node --experimental-wasm-modules scripts/wasm-golden.mjs           verify
//   UPDATE_GOLDEN=1 node --experimental-wasm-modules scripts/wasm-golden.mjs   refresh
//
// Defaults to the bundler target; pass `web` as argv[2] to switch.

import { readFile, readdir, writeFile, mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv[2] ?? "bundler";
const update = process.env.UPDATE_GOLDEN === "1";

const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");
const inputsDir = path.join(rootDir, "crates/usfm_onion_wasm/golden/inputs");
const outputsDir = path.join(rootDir, "crates/usfm_onion_wasm/golden/outputs");

const pkg = await import(pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href);
if (target === "web") {
  const wasmBytes = await readFile(path.join(pkgDir, "usfm_onion_web_bg.wasm"));
  await pkg.default({ module_or_path: wasmBytes });
}

await mkdir(outputsDir, { recursive: true });

// Deterministic JSON serialization: sort keys recursively.
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object") {
    const keys = Object.keys(value).sort();
    const out = {};
    for (const k of keys) out[k] = canonical(value[k]);
    return out;
  }
  return value;
}

function pretty(value) {
  return JSON.stringify(canonical(value), null, 2) + "\n";
}

const failures = [];
const written = [];

async function check(name, content) {
  const file = path.join(outputsDir, name);
  if (update) {
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, content);
    written.push(name);
    return;
  }
  let existing;
  try {
    existing = await readFile(file, "utf8");
  } catch {
    failures.push(`${name}: missing golden (run with UPDATE_GOLDEN=1)`);
    return;
  }
  if (existing !== content) {
    failures.push(`${name}: output differs from golden`);
  }
}

// --- corpus-wide (no-input) outputs ----------------------------------------

await check("marker-catalog.json", pretty(pkg.markerCatalog().all()));
await check("lint-codes.json", pretty(pkg.lintCodes()));
await check("lint-code-meta.json", pretty(pkg.lintCodeMeta()));
await check("format-rules.json", pretty(pkg.formatRules()));
await check("format-rule-meta.json", pretty(pkg.formatRuleMeta()));

// --- per-fixture outputs ---------------------------------------------------

const fixtures = (await readdir(inputsDir))
  .filter((n) => n.endsWith(".usfm"))
  .sort();

for (const fixture of fixtures) {
  const stem = fixture.replace(/\.usfm$/, "");
  const source = await readFile(path.join(inputsDir, fixture), "utf8");
  const parsed = pkg.parse(source);

  const tokens = parsed.tokens();
  await check(`${stem}/tokens.json`, pretty(tokens));
  await check(`${stem}/cst.json`, pretty(parsed.cst()));
  await check(`${stem}/lint.json`, pretty(parsed.lint({ scope: "book" })));
  await check(`${stem}/usj.json`, pretty(parsed.toUsj()));
  await check(`${stem}/usx.xml`, parsed.toUsx());
  await check(`${stem}/html.html`, parsed.toHtml());
  await check(`${stem}/vref.json`, pretty(parsed.toVref()));
  await check(`${stem}/format.usfm`, parsed.format());
  await check(`${stem}/to-usfm.usfm`, parsed.toUsfm());

  // Token-in pathways must agree with their source-in counterparts.
  await check(`${stem}/lint-from-tokens.json`, pretty(pkg.lintTokens(tokens, { scope: "book" })));
  await check(`${stem}/tokens-to-usfm.usfm`, pkg.tokensToUsfm(tokens));
  await check(`${stem}/tokens-to-html.html`, pkg.tokensToHtml(tokens));
  await check(`${stem}/format-tokens.json`, pretty(pkg.formatTokens(tokens)));

  // Diff each fixture against itself with one trailing edit to exercise the
  // diff machinery without exploding golden sizes.
  const edited = source + "\\p\n\\v 99 trailing addition\n";
  const editedParsed = pkg.parse(edited);
  const editedTokens = editedParsed.tokens();
  await check(`${stem}/diff.json`, pretty(parsed.diff(editedParsed)));
  await check(`${stem}/diff-by-chapter.json`, pretty(parsed.diffByChapter(editedParsed)));
  await check(`${stem}/diff-tokens.json`, pretty(pkg.diffTokens(tokens, editedTokens)));
}

if (update) {
  console.log(`baked ${written.length} golden files`);
} else if (failures.length) {
  console.error(`golden parity FAILED (${failures.length} mismatches):`);
  for (const f of failures) console.error(`  - ${f}`);
  process.exit(1);
} else {
  console.log(`golden parity passed (${fixtures.length} fixtures)`);
}
