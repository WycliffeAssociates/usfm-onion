# Progress: in-library chapter parallelism

Append-only companion to `plan-parallelism.md`.

## 2026-07-20 - initial design review

- Status: planning; no production implementation started.
- Chosen: in-library native Rayon, serial wasm, one public API per operation, caller
  throttling through a Rayon pool, and one cfg-selected ordered-map seam.
- Chosen: canonical source ordering for lint findings, established before parallelism.
- Chosen: lint is the first sufficient delivery; USJ, USX, VREF, HTML, and parse remain
  in scope behind independent correctness and performance gates.
- Chosen: target at least 15-20% representative large-book improvement and no more than
  5% small/single-chapter regression unless explicitly accepted.
- Finding: valid milestone `sid`/`eid` pairs may cross chapters but are independent,
  locally self-closed milestone elements. Onion and the `usfmtc` reference parser agree.
- Finding: isolated slices cannot drain as EOF; non-final segments need a
  `BeforeChapter` terminal that reproduces the whole walker's close reasons.
- Finding: naïve per-chapter lint changes rule ordering and disables document rules.
- Finding: HTML needs prefix-seeded document counters and one final note-section merge;
  parse needs per-slice book seeds and one final ID assignment.
- Deferred separately: loss-aware SID modeling and packed findings wire format; see
  `plan-sid-model-fidelity.md`.
- Verification performed during planning: live native Onion milestone cases and cloned
  `usfmtc` milestone cases; no product code changed.

## 2026-07-20 - Stage 0 executed (pin behavior + performance)

Baseline environment (pinned for Criterion comparisons):

- commit `f70d4f006fcb3980b0f96b4f8321019b28ebb4bb` + uncommitted Stage 0 changes at capture.
- rustc 1.95.0 (59807616e 2026-04-14), host aarch64-apple-darwin, LLVM 22.1.2.
- machine MacBookPro18,4 - 10 physical / 10 logical cores.
- corpus `example-corpora/en_ulb` (66 books); single-book lane `19-PSA.usfm` (Psalms, 150 ch).
- Criterion baseline name **`stage0`** (`cargo bench --bench parallelism -- lint --save-baseline stage0`).

Pinned median numbers:

| lane | time | thrpt |
| --- | --- | --- |
| `parallelism/en_ulb/lint/serial` | 36.5 ms | ~118 MiB/s |
| `parallelism/en_ulb/lint/rayon` (book grain) | 9.0 ms | ~476 MiB/s |
| `chapter_grain/psalms/lint/serial` | 2.35 ms | ~111 MiB/s |

Psalms serial is the intra-book cost Stage 2 chapter-grain must beat (book-grain Rayon can't
help a single book).

Two oracle rebless steps, kept separate on purpose:

1. **Canonical finding sort** - `canonical_sort` in `src/lint_impl.rs`, called in `lint_tokens`
   after suppression. Orders by primary span (spanless last), then `LintCode::code()`, then
   related span; `token_id`/`marker`/`message` are deterministic tie-breakers. NOT ordered by
   SID. Reblessed once; verified **reorder-only** (sorted baseline identical before/after, 0
   findings added/dropped, 490 lines reordered). All 188 lib tests + full suite green.
2. **Export anchors** - added `usj`/`usx`/`vref`/`html` digest lines (default options) to
   `tests/lint_oracle.rs`. Reblessed; verified **additions-only** (0 old lines dropped; new
   lines exclusively usj/usx/vref/html) and deterministic across runs.

Bench instrumentation: added `load_psalms()` (`benches/common.rs`) + `chapter_grain/psalms/
lint/serial` lane (`benches/parallelism.rs`). No production behavior changed; rayon stays a
dev-dependency (target-dep move is Stage 1).

Deferrals from the plan's Stage 0:

- Targeted chapter-boundary fixtures (milestone crossing `\c`, stray `\*`, open scope before
  `\c`) NOT authored yet - they gate `WalkBoundary` equivalence, so they move to **Stage 1**
  where they're exercised. Stage 0 pins current behavior over the existing adversarial corpus.
- HTML option-matrix anchors deferred to Stage 5 (only default `HtmlOptions` pinned now).

Pre-existing issues noted, left untouched (not mine): `cargo fmt --check` flags
`benches/common.rs:24`; `cargo clippy --lib` errors at `src/format/mod.rs:1337`.

Post-Stage-0: ran `cargo fmt` across the whole repo (user-authorized), clearing all
pre-existing drift including the `benches/common.rs` / `crates/usfm_onion_wasm` spots above.

## 2026-07-20 - Stage 1 executed (executor, partitions, walker boundary)

No production entrypoint is parallel yet; this stage only adds the seam, the partition, and
the boundary-correct segmented walk, all proven equivalent to the whole-book walk.

- **`crate::par::map_ordered`** (`src/par.rs`, `pub mod par`): one cfg-selected order-preserving
  map — native `par_iter().map().collect()`, wasm `iter().map().collect()`. Same signature both
  ways. No `par_reduce`/`fold` seam (book reconciliation stays serial). Unused in production so far.
- **Rayon moved to `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`** and removed from
  `[dev-dependencies]` (native tests/benches still get it). wasm build of the main lib links no rayon.
- **`WalkBoundary` + `drain_to_boundary`** (`src/walker/mod.rs`): `drain_to_eof` now delegates to
  `drain_to_boundary(EndOfInput)` (behavior unchanged); `BeforeChapter` closes open scopes with the
  exact reasons the `Chapter` arm of `apply_open_precedence` uses (`ImplicitByOpen`/`RecoveryClosure`).
- **`walk_range(tokens, range, boundary, visitor)`**: walks a range of the *full* slice with a fresh
  stack, so `token_index`es and frame indices stay absolute and lookahead reads the real next token.
  `0..len` + `EndOfInput` == `walk`.
- **`chapter_segments(tokens) -> Vec<ChapterSegment>`**: splits at `c`-marker-followed-by-Number
  (a numberless `\c` opens no scope, so it never starts a segment — matches the walker); front
  segment for pre-first-`\c` matter; last segment drains `EndOfInput`, rest `BeforeChapter`.

**The proof — `tests/walker_segmentation.rs`:** a recording visitor asserts whole-book
`walk_tokens` == concatenated `walk_range` over `chapter_segments`, event-for-event (kinds,
absolute indices, close reasons). Passes over (a) 9 targeted inline boundary cases — open
paragraph/character/note before `\c`, self-closed milestone, logical milestone pair crossing
`\c`, stray `\*`, duplicate `\id` after ch1, `\c` at index 0, no-chapters — and (b) **all 262
`testData/**/*.usfm` fixtures**. This subsumes the Stage-0-deferred boundary fixtures (covered
inline + by the corpus rather than as separate files).

Verification: 188 lib + all integration tests green; **oracle unchanged** (no production behavior
change); `cargo fmt --check` clean; clippy clean on new files (`par.rs`, `walker/mod.rs`);
`wasm32-unknown-unknown` builds the main lib without rayon.

## 2026-07-20 - Stage 2 attempt: correct decomposition, but a perf regression (not shipped)

Implemented the minimal split (range-local rules parallel per chapter segment via
`collect_range_local` + `walk_range`; `lint_structure_rules`, duplicate-chapter, and
number/verse as an explicit whole-book serial pass via `collect_book_serial`;
`collect_issues` split into collect + `finalize_issues`).

- **Correctness: achieved and proven.** Oracle byte-identical at `RAYON_NUM_THREADS=1`
  AND full pool; 188 lib tests green; walker equivalence green. The decomposition is
  behavior-preserving.
- **Performance: regressed hard.** Psalms lint 2.35ms -> 3.46ms full pool (+47%),
  13ms at 1 thread (+456%); en_ulb corpus lint +7% (serial lane) / +17% (rayon lane).
  Fails the acceptance gate (max 5% regression on small input).
- **Cause (samply, lint/tokens on Luke, full pool):** no lint hot path. ~88% of samples
  in `libsystem_kernel` (thread park/wake) with `libsystem_malloc` prominent; lint code
  frames spread thin. Per-chapter granularity (~151 tiny tasks, each allocating a Vec)
  makes thread-sync + allocation overhead dominate the tiny per-chapter compute.
  Isolation: book-serial pass alone = 0.83ms; the per-segment range-local pass is the
  blowup, spread across rules (not one culprit).
**Then the spike overturned that conclusion.** A throwaway spike in a worktree off the
pre-work commit (`examples/spike_chapter_lint.rs`, existing `LintScope::Chapter` API,
`rayon::par_iter` over chapter slices) measured, on 10 cores:

| book | whole-book serial | chapter serial | chapter parallel |
| --- | --- | --- | --- |
| Psalms (30k tok, 151 ch) | 2.30 ms | 2.57 ms (0.89x) | **0.70 ms (3.3x)** |
| Luke (7k tok, 25 ch) | 0.67 ms | 0.67 ms (1.00x) | **0.32 ms (2.1x)** |

So chapter-grain lint **is** worth it — a 2-3.3x win with **negligible** decomposition
overhead (serial split 0.89-1.00x). My +47% production regression was therefore an
**implementation defect in the elaborate `collect_range_local`/`collect_book_serial`
split**, NOT a fundamental limit. The spike's simple shape (par_iter chapters, whole
per-chapter lint) is both faster and simpler than what I built; a correct production impl
should approach ~1ms on Psalms (serial book pass 0.83ms + small parallel range-local).

**Process lesson (see memory [[feedback-spike-perf-hypotheses-first]]):** this spike should
have come FIRST. It cost an hour and would have prevented the whole invasive-refactor +
revert cycle AND my wrong "not worth it" conclusion.

**State:** Stage 1 committed. Stage 2 production decomposition is uncommitted in the working
tree — correct (oracle byte-identical at 1 and full workers) but ~3x slower than the spike
ceiling; keep it to debug the inefficiency, don't ship as-is. Spike lives in the
`usfm_onion-spike` worktree.

Next session: (a) find why the production split is ~3x slower than the spike (compare
against `examples/spike_chapter_lint.rs`) OR rewrite production to the simpler spike shape
(par_iter chapters + serial book reconcile for document/cross-chapter rules); (b) re-bench
against the 15-20% gate. Separately: fix the `DocumentLintState.note_stack` never-popped bug
and intentionally rebaseline the oracle.

## 2026-07-20 - Full spike matrix (worktree `usfm_onion-spike` @ f70d4f0, 10 cores, Psalms)

Every operation spiked via the existing public API (throwaway examples in the worktree:
`spike_chapter_lint`, `spike_chapter_parse`, `spike_parallel_lex`, `spike_chapter_exports`).
Chapter-parallel = split at `\c`, `par_iter`, existing per-slice API; timing/ceiling only.

| op | whole | chapter-parallel | speedup | notes |
| --- | --- | --- | --- | --- |
| lex | 1.05 ms | 0.25 ms | 4.2x | source byte-split at `\c` (line starts = clean boundaries); lex needs no `\id` |
| parse (lex+parse both split) | 2.17 ms | 0.58 ms | 3.75x | byte-split removes the serial-lex floor (parse-step-only was ~1.6x) |
| lint | 2.30 ms | 0.70 ms | 3.3x | ~0 decomposition overhead |
| **html** | **8.02 ms** | **1.84 ms** | **4.36x** | heaviest op AND best speedup — the top target |
| vref | 1.25 ms | 0.42 ms | 2.95x | |
| cst | 0.89 ms | 0.50 ms | 1.78x | light op, least benefit |

Pre-scan `\c` byte offsets: 0.19 ms (negligible). Decomposition overhead is negligible-to-
mild across the board (serial-split ratios 0.84-1.55x). Small books (Luke) are a wash — the
win scales with book size.

**Conclusions:**
- Chapter parallelism is a real 2-4.4x win for every heavy op; my Stage 2 lint regression was
  an impl/measurement artifact, NOT fundamental (proven by the clean spike).
- **HTML is the highest-value target** (heaviest + best speedup: 8ms -> 1.8ms).
- The strongest architecture is a **shared source-level chapter partition**: one cheap byte
  pre-scan at `\c`, then parallel lex+parse+lint+exports per chunk, rather than per-operation
  token-splitting. Merges are all cheap linear passes: span rebasing (+chunk offset), token-id
  reindex, bookcode seed for sids, doc-sequential HTML caller renumber, dup-chapter/verse
  reconcile.
- Real-impl caveats: the `\c` pre-scan must reject false positives (in text/attributes/
  comments) — validate against the lexed stream or a careful line-start scan; spans/ids need
  rebasing; wasm stays serial.
