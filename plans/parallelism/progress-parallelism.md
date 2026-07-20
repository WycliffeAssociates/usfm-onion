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

## Next: Stage 2 — lint proving ground (decompose Book lint into segment-local + book
reconciliation; serial ordered map first, then `map_ordered`; oracle identity at 1 and full workers)
