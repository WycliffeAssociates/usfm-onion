// Native-vs-wasm lifecycle parity gate.
//
// The Rust generator (`crates/usfm_onion_wasm/src/parity.rs`,
// `generate_parity_transcript`, `#[ignore]`d) drives the *native*
// `braid::Braid` through a scripted lifecycle and writes
// `crates/usfm_onion_wasm/tests/fixtures/parity-transcript.json`: one entry
// per step, each carrying the wasm-shaped JSON arguments and the expected
// wasm-shaped output, produced by calling the exact same `pub(crate)` DTO
// conversions the real wasm bindings call. This script drives the *real*
// wasm `Braid` class (bundler or web build) through the identical sequence
// and deep-compares its actual output against that transcript.
//
// The only mapping this script does is unwrapping the `ApiResult`
// `{status, value|error}` envelope that fallible verbs return — the
// transcript's "output" is always the bare semantic value (or the bare
// error value for an intentionally-failing step), never the envelope
// itself. Every other field (hex snapshot/source-hash ids, tuple-shaped
// `VrefIndex` entries, tagged `ScopedOutput`/`ChapterLabel`/error unions) is
// produced by the same Rust `Serialize` impl on both sides, so there is
// nothing else to translate — a real divergence shows up as a plain
// structural diff, not a mapping bug.
//
//   node scripts/test-parity.mjs [bundler|web]

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const rootDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const target = process.argv.includes("web") ? "web" : "bundler";
const pkgDir = path.join(rootDir, target === "bundler" ? "pkg-bundler" : "pkg-web");

const wasm = await import(pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href);
if (target === "web") {
  const { readFile: read } = await import("node:fs/promises");
  const wasmBytes = await read(path.join(pkgDir, "usfm_onion_web_bg.wasm"));
  await wasm.default({ module_or_path: wasmBytes });
}

const transcriptPath = path.join(
  rootDir,
  "crates/usfm_onion_wasm/tests/fixtures/parity-transcript.json",
);
const transcript = JSON.parse(await readFile(transcriptPath, "utf8"));

/**
 * Deep structural equality with JSON semantics: a key whose value is
 * `undefined` is the same as an absent key (the Rust side omits `None`
 * fields entirely via `skip_serializing_if`), and key order is not
 * semantics.
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

/** Every method this transcript exercises: its JS name, whether it returns
 * an `ApiResult`-wrapped `*Outcome` (needs unwrapping) or a plain value, and
 * how to read its arguments out of one step's `args` object. */
const METHODS = {
  replace_corpus: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  books: { js: "books", wrapped: false, args: () => [] },
  chapter_labels: { js: "chapterLabels", wrapped: true, args: (a) => [a.book] },
  chapter_labels_unknown_book: { js: "chapterLabels", wrapped: true, args: (a) => [a.book] },
  update_chapter: { js: "updateChapter", wrapped: true, args: (a) => [a.target, a.replacement] },
  update_chapter_ambiguous_seed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  update_chapter_ambiguous: {
    js: "updateChapter",
    wrapped: true,
    args: (a) => [a.target, a.replacement],
  },
  lint: { js: "lint", wrapped: false, args: () => [] },
  patches: { js: "patches", wrapped: false, args: () => [] },
  preview_patch: { js: "previewPatch", wrapped: true, args: (a) => [a.id] },
  apply_patch: { js: "applyPatch", wrapped: true, args: (a) => [a.id] },
  apply_patch_stale: { js: "applyPatch", wrapped: true, args: (a) => [a.id] },
  to_tokens_pull: { js: "toTokens", wrapped: true, args: (a) => [a.scopes] },
  set_baseline: { js: "setBaseline", wrapped: true, args: (a) => [a.book] },
  set_baseline_not_resident: { js: "setBaseline", wrapped: true, args: (a) => [a.book] },
  is_dirty_clean: { js: "isDirty", wrapped: true, args: (a) => [a.scope] },
  is_dirty_dirty: { js: "isDirty", wrapped: true, args: (a) => [a.scope] },
  update_book: { js: "updateBook", wrapped: true, args: (a) => [a.book] },
  diff_baseline: { js: "diffBaseline", wrapped: true, args: (a) => [a.scope] },
  prepare_format_patch: {
    js: "prepareFormatPatch",
    wrapped: true,
    args: (a) => [a.scope, a.options ?? undefined],
  },
  apply_format_patch: { js: "applyFormatPatch", wrapped: true, args: (a) => [a.id] },
  vref_index: { js: "vrefIndex", wrapped: true, args: (a) => [a.scope] },
  to_usfm: { js: "toUsfm", wrapped: true, args: (a) => [a.scope] },
  remove_book: { js: "removeBook", wrapped: true, args: (a) => [a.book] },
  clear: { js: "clear", wrapped: false, args: () => [] },
  replace_corpus_malformed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  replace_corpus_duplicate_token_id: {
    js: "replaceCorpus",
    wrapped: true,
    args: (a) => [a.corpus],
  },
  // The reopen-from-packed gate. `RestoreRecord` is a plain (non-tagged)
  // struct that crosses the wasm boundary through the generic
  // serde-wasm-bindgen path, which represents `Vec<u8>` as `number[]`
  // (confirmed against the generated `.d.ts`: `packed: number[]`), not a
  // `Uint8Array` — so the transcript's own plain JSON number arrays are
  // already the right shape and need no conversion here.
  restore_corpus: { js: "restoreCorpus", wrapped: true, args: (a) => [a.records] },
  // Continues on the same just-restored instance (see FRESH_INSTANCE_STEPS)
  // rather than replaying restore_corpus's own args.
  restore_corpus_then_lint: { js: "lint", wrapped: false, args: () => [] },
  // The suppressed-config case: a restoring config with any suppression
  // configured must decline to prime a cached summary rather than claim a
  // stale suppressedCount of 0. This step's own `args.config` (not the
  // shared `transcript.config`) is what the fresh instance below is built
  // from, since it deliberately differs from every other step's config.
  restore_corpus_suppressed: { js: "restoreCorpus", wrapped: true, args: (a) => [a.records] },
  restore_corpus_suppressed_then_lint: { js: "lint", wrapped: false, args: () => [] },
  // The publish gate: pins the wasm `publish()` verb's projection against the
  // native adapter it wraps (asserted byte-identical in the Rust generator
  // itself before this step is ever recorded). `publish_seed` builds the
  // fresh instance `publish` then continues on.
  publish_seed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  publish: { js: "publish", wrapped: true, args: () => [] },
  // Clean-room re-review P1: pins the empty-sourceKey classification
  // (`{kind: "ingest", error: {kind: "duplicateSourceKey", source: ""}}`),
  // the pre-extraction wasm behavior `braid::RestoreError::
  // EmptySourceKey` now reproduces. `packed` is a direct `Vec<u8>` function
  // parameter (not a struct field), which crosses as `Uint8Array` -- unlike
  // `records[].source`, a plain struct field that is already the right
  // shape (`number[]`) with no conversion needed.
  restore_published_corpus_empty_source_key: {
    js: "restorePublishedCorpus",
    wrapped: true,
    args: (a) => [new Uint8Array(a.packed), a.records],
  },
  // publish_scope: its own fresh handle (`publish_scope_seed` builds it),
  // then the scoped publish itself.
  publish_scope_seed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  publish_scope: { js: "publishScope", wrapped: true, args: (a) => [a.scope] },
  // revert_to_baseline: its own fresh handle, seeded/baselined/edited via
  // their own recorded steps, then the revert itself and the chapter-scope
  // refusal continuing on that same instance.
  revert_seed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  revert_set_baseline: { js: "setBaseline", wrapped: true, args: (a) => [a.book] },
  revert_update_book: { js: "updateBook", wrapped: true, args: (a) => [a.book] },
  revert_to_baseline: { js: "revertToBaseline", wrapped: true, args: (a) => [a.scope] },
  revert_to_baseline_chapter_unsupported: {
    js: "revertToBaseline",
    wrapped: true,
    args: (a) => [a.scope],
  },
  // set_baseline_to_current: its own fresh handle, seeded/edited via their
  // own recorded steps, then the no-parse baseline declaration, its
  // idempotence, and the chapter-scope refusal, all on that same instance.
  set_baseline_to_current_seed: { js: "replaceCorpus", wrapped: true, args: (a) => [a.corpus] },
  set_baseline_to_current_edit: { js: "updateBook", wrapped: true, args: (a) => [a.book] },
  set_baseline_to_current: { js: "setBaselineToCurrent", wrapped: true, args: (a) => [a.scope] },
  set_baseline_to_current_idempotent: {
    js: "setBaselineToCurrent",
    wrapped: true,
    args: (a) => [a.scope],
  },
  set_baseline_to_current_chapter_unsupported: {
    js: "setBaselineToCurrent",
    wrapped: true,
    args: (a) => [a.scope],
  },
  // All scope on an empty corpus, on its own never-seeded fresh handle.
  set_baseline_to_current_empty_corpus: {
    js: "setBaselineToCurrent",
    wrapped: true,
    args: (a) => [a.scope],
  },
};

/** Steps that build their own fresh `Braid` rather than continuing the
 * lane's ongoing one. `update_chapter_ambiguous_seed` and the two
 * `replace_corpus_*` failure cases exist only to prove one refusal and must
 * not perturb the lane's main sequence; `restore_corpus` is a genuine cold
 * reopen and must start from an empty handle, exactly as the Rust generator's
 * `reopened` does. `restore_corpus_then_lint` deliberately continues on that
 * same restored instance, so it is not in this set. */
const FRESH_INSTANCE_STEPS = new Set([
  "update_chapter_ambiguous_seed",
  "replace_corpus_malformed",
  "replace_corpus_duplicate_token_id",
  "restore_corpus",
  "restore_corpus_suppressed",
  "publish_seed",
  "restore_published_corpus_empty_source_key",
  "publish_scope_seed",
  "revert_seed",
  "set_baseline_to_current_seed",
  "set_baseline_to_current_empty_corpus",
]);

function makeMinter() {
  let next = 0;
  return () => {
    next += 1;
    return `minted-${next}`;
  };
}

function unwrap(outcome, expectedIsError) {
  assert.ok(
    outcome && typeof outcome === "object" && "status" in outcome,
    `expected an ApiResult envelope, got ${JSON.stringify(outcome)}`,
  );
  if (expectedIsError) {
    assert.equal(outcome.status, "error", `expected an error outcome, got ${JSON.stringify(outcome)}`);
    return outcome.error;
  }
  assert.equal(outcome.status, "ok", `expected an ok outcome, got ${JSON.stringify(outcome)}`);
  return outcome.value;
}

let checked = 0;
let failures = 0;

function runLane(lane, steps) {
  let braid = new wasm.Braid(transcript.config, makeMinter());
  for (const entry of steps) {
    const method = METHODS[entry.step];
    assert.ok(method, `${target}/${lane}/${entry.step}: no method mapping registered`);
    if (FRESH_INSTANCE_STEPS.has(entry.step)) {
      // A step can carry its own config (see `restore_corpus_suppressed`)
      // when it deliberately needs one different from every other step's
      // shared `transcript.config`; otherwise fall back to that shared one.
      braid = new wasm.Braid(entry.args.config ?? transcript.config, makeMinter());
    }
    let actual;
    try {
      const raw = braid[method.js](...method.args(entry.args));
      actual = method.wrapped ? unwrap(raw, isErrorStep(entry)) : raw;
    } catch (error) {
      throw new Error(
        `${target}/${lane}/${entry.step}: threw calling ${method.js} — ${error.stack ?? error}`,
      );
    }
    const diff = structuralEqual(entry.output, actual);
    checked += 1;
    if (diff) {
      failures += 1;
      console.error(
        `DIVERGENCE ${target}/${lane}/${entry.step} at ${diff}\n  expected: ${JSON.stringify(entry.output)}\n  actual:   ${JSON.stringify(actual)}`,
      );
    }
  }
}

/** Whether a step is an intentional-refusal case, by name rather than by
 * guessing from its output shape (some ok outcomes — `ScopedOutput`,
 * `PatchPreparation` — are themselves tagged with `kind`, which is not the
 * same thing as an error). */
function isErrorStep(entry) {
  return entry.step.endsWith("_unknown_book")
    || entry.step.endsWith("_stale")
    || entry.step.endsWith("_not_resident")
    || entry.step.endsWith("_ambiguous")
    || entry.step.endsWith("_malformed")
    || entry.step.endsWith("_duplicate_token_id")
    || entry.step.endsWith("_empty_source_key")
    || entry.step.endsWith("_unsupported");
}

const byLane = new Map();
for (const entry of transcript.steps) {
  if (!byLane.has(entry.lane)) byLane.set(entry.lane, []);
  byLane.get(entry.lane).push(entry);
}
for (const [lane, steps] of byLane) {
  runLane(lane, steps);
}

console.log(
  `parity (${target}): ${checked} steps checked across ${byLane.size} lane(s), ${failures} divergence(s)`,
);
if (failures > 0) {
  process.exitCode = 1;
}
