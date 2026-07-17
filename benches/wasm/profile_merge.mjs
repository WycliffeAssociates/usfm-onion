// Node-driven wasm profiling harness for the merge-projection engine
// (`diffUsfm` + `mergeDiffBlocks`), meant to be wrapped by `samply record`
// for a flamegraph. Mirrors `examples/profile_merge.rs` (the native
// counterpart) scenario-for-scenario so the two are directly comparable.
//
// Symbolication note: the checked-in `pkg-bundler`/`pkg-web` are `--release`
// builds run through `wasm-opt -Oz` — great for shipping, useless for
// profiling (no name section, inlined-to-oblivion). Build a `--dev` package
// to a scratch dir first:
//
//   wasm-pack build crates/usfm_onion_wasm --target bundler --dev \
//     --out-dir /tmp/pkg-bundler-profiling --out-name usfm_onion_web
//
// Usage:
//   samply record -- node benches/wasm/profile_merge.mjs /tmp/pkg-bundler-profiling small 300
//   samply record -- node benches/wasm/profile_merge.mjs /tmp/pkg-bundler-profiling medium 100
//   samply record -- node benches/wasm/profile_merge.mjs /tmp/pkg-bundler-profiling large 20
//   samply record -- node benches/wasm/profile_merge.mjs /tmp/pkg-bundler-profiling all

import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const rootDir = path.dirname(path.dirname(path.dirname(fileURLToPath(import.meta.url))));
const pkgDir = process.argv[2];
const scenario = process.argv[3] ?? "all";
const iterations = process.argv[4] ? Number(process.argv[4]) : undefined;

if (!pkgDir) {
  console.error("usage: node profile_merge.mjs <pkg-dir> [small|medium|large|all] [iterations]");
  process.exit(1);
}

const pkg = await import(pathToFileURL(path.join(path.resolve(pkgDir), "usfm_onion_web.js")).href);

const BOOK_PATH = path.join(rootDir, "example-corpora/en_ulb/43-LUK.usfm");
const CORPUS_ROOT = path.join(rootDir, "example-corpora/en_ulb");

async function collectUsfmPaths(dir, out) {
  const { readdir } = await import("node:fs/promises");
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      await collectUsfmPaths(full, out);
    } else if (entry.name.endsWith(".usfm")) {
      out.push(full);
    }
  }
}

async function loadCorpus(root) {
  const paths = [];
  await collectUsfmPaths(root, paths);
  paths.sort();
  return Promise.all(paths.map((p) => readFile(p, "utf8")));
}

// Insert a short marker word into `count` verses, spread roughly evenly
// across the book — mirrors `edit_n_verses` in examples/profile_merge.rs.
function editNVerses(source, count) {
  const totalVerses = (source.match(/\\v /g) ?? []).length;
  const stride = Math.max(Math.floor(totalVerses / Math.max(count, 1)), 1);

  let out = "";
  let verseIndex = 0;
  let edited = 0;
  let rest = source;

  for (;;) {
    const markerPos = rest.indexOf("\\v ");
    if (markerPos === -1) break;
    out += rest.slice(0, markerPos);
    verseIndex += 1;

    const afterMarker = rest.slice(markerPos + 3);
    const numberEnd = (() => {
      const m = afterMarker.match(/[^0-9-]/);
      return m ? m.index : afterMarker.length;
    })();
    const number = afterMarker.slice(0, numberEnd);
    let tail = afterMarker.slice(numberEnd);
    out += "\\v " + number;
    if (tail.startsWith(" ")) {
      out += " ";
      tail = tail.slice(1);
    }

    if (edited < count && verseIndex % stride === 0) {
      out += "edited ";
      edited += 1;
    }
    rest = tail;
  }
  out += rest;
  return out;
}

// One realistic diff+merge round-trip via the wasm surface: source-in diff
// (the realistic JS entry point — see benches/wasm/run.mjs's op list), then
// merge twice (accept-all-incoming and reject-all-incoming), matching the
// native harness's `diff_and_merge_once`.
function diffAndMergeOnce(baseline, current) {
  pkg.diffUsfm(baseline, current);
  const baselineTokens = pkg.parse(baseline).tokens();
  const currentTokens = pkg.parse(current).tokens();
  const acceptIncoming = pkg.mergeDiffBlocks(baselineTokens, currentTokens, {
    decisions: {},
    defaultSide: "current",
  });
  const rejectIncoming = pkg.mergeDiffBlocks(baselineTokens, currentTokens, {
    decisions: {},
    defaultSide: "baseline",
  });
  return acceptIncoming.length + rejectIncoming.length;
}

function runDiffMergeLoop(label, baseline, current, iterations) {
  console.log(
    `=== ${label}: baseline ${baseline.length} bytes, current ${current.length} bytes, ${iterations} iterations ===`,
  );
  const started = performance.now();
  for (let i = 0; i < iterations; i++) {
    diffAndMergeOnce(baseline, current);
  }
  const elapsedMs = performance.now() - started;
  const perIterationMs = elapsedMs / iterations;
  console.log(`  ${iterations} iteration(s) in ${elapsedMs.toFixed(2)}ms (${perIterationMs.toFixed(2)}ms/iteration)`);
}

async function runBookScenario(label, versesChanged, iterations) {
  const baseline = await readFile(BOOK_PATH, "utf8");
  const current = editNVerses(baseline, versesChanged);
  runDiffMergeLoop(label, baseline, current, iterations);
}

// Whole-corpus "reformat then diff" pass — the realistic "did reformatting
// change any content" workflow. Mirrors `run_corpus_scenario`.
async function runCorpusScenario(iterations) {
  const books = await loadCorpus(CORPUS_ROOT);
  const totalBytes = books.reduce((sum, s) => sum + Buffer.byteLength(s, "utf8"), 0);
  console.log(
    `=== large (whole corpus reformat-and-diff, ${books.length} books, ${(totalBytes / (1024 * 1024)).toFixed(2)} MiB) ===`,
  );

  const started = performance.now();
  for (let i = 0; i < iterations; i++) {
    for (const baseline of books) {
      const current = pkg.formatUsfm(baseline);
      diffAndMergeOnce(baseline, current);
    }
  }
  const elapsedMs = performance.now() - started;
  const perIterationMs = elapsedMs / iterations;
  const mibPerSec = (totalBytes / (1024 * 1024)) / (perIterationMs / 1000);
  console.log(
    `  ${iterations} pass(es) over the corpus in ${elapsedMs.toFixed(2)}ms (${perIterationMs.toFixed(2)}ms/pass, ${mibPerSec.toFixed(1)} MiB/s)`,
  );
}

switch (scenario) {
  case "small":
    await runBookScenario("small (3 verses changed)", 3, iterations ?? 300);
    break;
  case "medium":
    await runBookScenario("medium (~50 verses changed)", 50, iterations ?? 100);
    break;
  case "large":
    await runCorpusScenario(iterations ?? 5);
    break;
  case "all":
    await runBookScenario("small (3 verses changed)", 3, iterations ?? 300);
    await runBookScenario("medium (~50 verses changed)", 50, iterations ?? 100);
    await runCorpusScenario(iterations ?? 5);
    break;
  default:
    console.error(`unknown scenario ${JSON.stringify(scenario)}; expected small|medium|large|all`);
    process.exit(1);
}
