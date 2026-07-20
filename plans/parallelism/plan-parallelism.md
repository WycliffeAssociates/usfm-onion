# Plan: in-library chapter parallelism, oracle-gated

Status: proposed, revised after design review (2026-07-20). Supersedes the
"single-threaded by design, parallelism is the caller's concern" stance in
`benches/parallelism.rs` while keeping IO caller-owned.

## Goal

Make a single native library call use available cores for parse, lint, USJ, USX, VREF,
and HTML. Preserve behavior after one intentional finding-order migration, keep wasm
serial and portable, and let a native caller cap Rayon concurrency without parallel and
serial API twins.

Lint is the first sufficient delivery and proving ground. Every later stage remains in
scope, but ships only after its own correctness and performance gates pass.

## Non-goals

- Parallelizing `lex`; it remains the whole-source serial byte pass.
- Parallelizing diff before the pattern is proven elsewhere.
- Threaded wasm (`wasm-bindgen-rayon`, shared memory, COOP/COEP, `build-std`).
- Moving file IO into the library.
- Validating logical milestone `sid`/`eid` pairing. Each milestone marker's required
  local `\*` is already validated; logical linkage is a separate possible lint rule.
- Changing SID fidelity or building the packed findings wire format; see
  `plan-sid-model-fidelity.md`.

## Locked design decisions

1. **Native Rayon is internal and target-gated, not a public API mode.**

   ```toml
   [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
   rayon = "1.10"
   ```

   Native builds compile the parallel executor; wasm builds compile the serial executor.
   Add a Cargo feature only if a real native consumer later requires a Rayon-free build.

2. **One ordered-map seam, not parallel methods throughout the codebase.**

   `crate::par::map_ordered` has two cfg-selected implementations:

   - native: indexed `par_iter().map(f).collect()`;
   - wasm: `iter().map(f).collect()`.

   Do not add generic `par_reduce`, `fold`, or `in_parallel` helpers until an actual use
   requires them. Book-level reconciliation is small, order-sensitive, and serial.

3. **The caller throttles Rayon rather than selecting a second Onion API.**

   A caller can configure the global pool before first use or call Onion inside a scoped
   `pool.install(..)`. `num_threads(1)` means one-worker Rayon execution, not a distinct
   no-Rayon serial implementation.

4. **Parse-once semantics remain.** Source APIs lex once; no chapter reparses source text.

5. **Lint findings have canonical source order.** Order by primary source span, stable
   lint-code identifier, and related span, with spanless document findings last and a
   deterministic tie order. Establish this contract in the serial implementation before
   parallel lint. Do not order by SID: duplicate, malformed, or decreasing references are
   valid linter inputs.

## Shared chapter boundary model

- Split token streams before each `\c`; material before the first `\c` is the front
  segment. Token IDs and byte spans remain absolute.
- A USFM milestone marker such as `qt-s` or `qt-e` is its own `Milestone` token followed
  by its immediate `MilestoneEnd` (`\*`). Logical `sid`/`eid` partners may live in
  different chapters and do not form a walker scope across the boundary.
- The real walker boundary hazard is the close reason. A whole-book walk reaching `\c`
  closes open scopes as `ImplicitByOpen` or `RecoveryClosure`; ending an isolated slice
  as EOF would report `EndOfInput` and can change lint behavior.
- Add an internal segmented-walk terminal:

  ```rust
  enum WalkBoundary {
      BeforeChapter,
      EndOfInput,
  }
  ```

  Front and non-final chapter segments finish with `BeforeChapter`, which performs the
  same pops and leave reasons as an incoming Chapter without emitting that next chapter.
  The final segment finishes with `EndOfInput`. Existing whole-stream walker APIs retain
  their current behavior.

## Per-operation merge contracts

| Operation | Segment-local product                                 | Ordered document reconciliation                                                    | Difficulty |
| --------- | ----------------------------------------------------- | ---------------------------------------------------------------------------------- | ---------- |
| lint      | findings plus document-rule summary/events            | reconcile book rules; dedupe; suppress; canonical-sort; summarize                  | MEDIUM     |
| USJ       | top-level content nodes                               | concatenate under one document wrapper                                             | LOW        |
| USX       | ordered USJ document                                  | serialize the merged USJ once                                                      | LOW        |
| VREF      | ordered map entries                                   | insert in segment order so later duplicate keys retain current last-write behavior | LOW        |
| HTML      | body, extracted notes, and note counts                | prefix-seed global counters; concatenate body; append note sections once           | HIGH       |
| parse     | tokens plus parse analysis from a seeded lexeme slice | concatenate; merge analysis; assign token IDs once                                 | MEDIUM     |

## Measurements already gathered

On 2026-07-20, on a 10-physical-core machine, `en_ulb` contained 254,559 tokens
across 66 books.

- Book grain: max/median 13.4x; largest task 30,857 tokens (Psalms).
- Chapter grain: max/median 11.1x; largest task 1,969 tokens across 1,255 tasks.
- Largest task shrinks 15.7x. Book-grain work ceiling is 8.25x; chapter grain removes
  that practical straggler ceiling for this corpus.
- Lint measured approximately 33.0 ms serial and 8.05 ms with caller-owned book-grain
  Rayon. The approximately 4 ms chapter-grain number is a hypothesis, not acceptance
  evidence.

## Acceptance gates

- Pin the instrumentation-only baseline by exact commit SHA, dirty status, corpus
  revision, `rustc -Vv`, machine/core information, and exact command.
- Save a named Criterion baseline. Include both a representative single large book and
  whole-corpus throughput; the current 66-book outer Rayon lane alone does not prove a
  single-call chapter win.
- For each operation, establish byte-identical serial segmentation before enabling
  Rayon, then compare one-worker and full-pool output byte-for-byte.
- A stage should normally show at least a 15-20% improvement on representative large
  books and no more than a 5% regression on small/single-chapter input. Numbers outside
  those bounds require an explicit keep/defer decision.
- No stage inherits another stage's performance verdict.

## Oracle

`tests/lint_oracle.rs` and `tests/lint_oracle_baseline.txt` pin behavior over the
adversarial `testData/**/*.usfm` corpus: token digest/count, CST digest/node count, full
lint findings, and summary.

Add direct USJ, USX, VREF, and HTML anchors before their corresponding merge stages.
HTML coverage must include the counter-sensitive option shapes, not only defaults.
Direct output anchors are required because unchanged tokens do not prove a new assembler
or merge is correct.

Required targeted chapter-boundary fixtures:

- open paragraph, character, and note before `\c`;
- properly self-closed milestone before `\c`;
- milestone missing its immediate `\*` before `\c`;
- logical `qt-s`/`qt-e` pair crossing `\c`;
- stray `\*`;
- duplicate/misplaced `\id` after chapter one;
- HTML notes outside a verse and both caller-scope settings;
- repeated VREF keys;
- multiple `\id` values for parse seeding.

## Stages

### Stage 0 - pin behavior and performance before execution changes

1. Land the oracle and benchmark instrumentation without changing production behavior.
2. Record and save the exact Criterion baseline described above.
3. Canonically sort findings in the current serial path, review that ordering-only diff,
   and re-bless once. After this migration, all parallel stages are byte-stable.
4. Add direct output anchors as each walker stage approaches. Keep additions-only oracle
   changes separate from the intentional finding reorder.

Verification: baseline artifacts and environment recorded; serial oracle deterministic;
canonical finding-order diff reviewed; `git diff --check` clean.

### Stage 1 - shared ordered executor, partitions, and walker boundary

1. Add cfg-selected `crate::par::map_ordered`.
2. Move Rayon from dev-dependencies to the native target dependency.
3. Add shared token `chapter_segments` with front/chapter metadata.
4. Add `WalkBoundary` and targeted tests proving segmented serial walks emit the same
   events and close reasons as a whole-book walk.

Verification: no production entrypoint uses parallelism yet; oracle unchanged;
`wasm32-unknown-unknown` builds without Rayon.

### Stage 2 - lint proving ground

**Classify rules by computational shape, not by lint category.** Category
(`Document`/`Structure`/`Context`/`Numbering`) is a taxonomy of *meaning*, not
execution locality; a first attempt that split on category failed the oracle
(the split dropped `duplicate-verse-number`, `marker-not-valid-in-context`,
`unclosed-marker`, whitespace). The correct classification is by how a rule
computes its findings:

| Shape | Rules | Execution |
| --- | --- | --- |
| Range-local (iterate an absolute range; keep full-slice access for predecessor/lookahead) | empty-paragraph, expectation/unknown-token, invalid-range, number-predecessor, unknown-marker(s), whitespace | parallel, per segment |
| Walker-driven | marker-balance (stray/misnest/unclosed/implicit/milestone-self-close/verse-in-section) | parallel, per segment via `walk_range(full, range, boundary)` — NOT a sliced token array |
| Ordered-occurrence + serial reconcile | duplicate-verse-number | per segment emit ordered `(chapter, verse-range, token)` events; reconcile serially with the existing book-wide `seen` logic (duplicate `\c` segments sharing a semantic chapter fall out naturally) |
| Whole-book serial pass | `lint_structure_rules` (document identity, content ordering, paragraph/note context, book invariants), `duplicate-chapter-number` | serial, over the full stream |

The whole-book structure pass is an **explicit, named rule-family contract**, not
the forbidden "silent broad serial fallback." Do not build per-rule prefix-state/
merge machinery for it until benchmarks show this serial pass materially caps the
speedup.

Two range mistakes to avoid (they masqueraded as statefulness in the first
attempt): running the walker over a *sliced* token array throws away the
`BeforeChapter` boundary Stage 1 added — use `walk_range(full, range, boundary)`;
and whitespace/lookahead rules must iterate an absolute range while still holding
the complete token slice for predecessor/next-token access.

Execution shape:

```
parallel chapter segments
  -> range-local findings
  -> walker findings via walk_range(full, range, boundary)
  -> ordered duplicate-verse occurrence events
serial book reconciliation
  -> duplicate verse / duplicate chapter findings
  -> lint_structure_rules over the full stream
combine once
  -> deduplicate -> suppress -> canonical sort -> summarize
```

Steps:

1. Split `collect_issues` into a per-segment collector (range-local + walker +
   occurrence events) and a serial book pass (structure rules + numbering
   reconcile). Range-local rules take `(tokens, range)`, not a sub-slice.
2. Reconcile duplicate chapters and duplicate verses from ordered events in source
   order.
3. Run the decomposed path with a serial ordered map and require oracle identity
   FIRST (equivalently: oracle at `RAYON_NUM_THREADS=1`).
4. Switch the executor to `map_ordered`; confirm oracle identity at the full pool too.
5. Dedupe, suppress, canonical-sort, summarize once after reconciliation.

Verification: oracle identical at one and full worker counts; full lib test suite
(covers `Front`/`Chapter` scopes) green; Criterion clears the acceptance threshold.

**Separate, pre-existing bug to resolve intentionally (NOT parallelism):**
`DocumentLintState.note_stack` (`src/lint_impl.rs`) is pushed on note-open in
`apply_marker` but never popped on note-close — the `EndMarker` branch of
`lint_structure_rules` pops only the *local* `note_stack`. So `current_note_context()`
stays stale after a footnote closes, and post-note markers are validated as if
still inside the note. Some whole-book `marker-not-valid-in-context` findings are
likely spurious because of this. The oracle pins current (buggy) behavior; keeping
the structure pass whole-book preserves it, so Stage 2 stays green. Fix the stack
separately and rebaseline the oracle intentionally.

### Stage 3 - USJ and USX

1. Build segment-local USJ content with the correct `WalkBoundary`.
2. Concatenate content in segment order under one USJ wrapper.
3. Serialize the merged USJ to USX once.
4. Prove serial segmentation before switching to `map_ordered`.

Verification: USJ/USX anchors byte-identical at one and full worker counts; benchmark
each public operation.

### Stage 4 - VREF

1. Produce segment-local ordered VREF entries with the correct boundary.
2. Merge in segment order so duplicate reference keys preserve current later-write-wins
behavior.
3. Prove serial segmentation before switching to `map_ordered`.

Verification: VREF anchors cover both trim settings and duplicate references; outputs
are byte-identical across worker counts; benchmark clears the stage threshold.

### Stage 5 - HTML

HTML remains in scope, but a final string concatenation alone cannot repair labels and
IDs already embedded during rendering. Use a two-phase ordered design:

1. Count per-segment total notes, out-of-verse sequential notes, footnotes, and
   cross-references using a cheap segment pass.
2. Compute serial exclusive-prefix offsets.
3. Render segments in parallel with an `HtmlSeed` containing their initial global
   counters.
4. Return structured `HtmlFragment { body, footnotes, crossrefs }` products.
5. Concatenate bodies in order, append each extracted-note section once, and apply the
   root wrapper once.

Cover all caller styles/scopes, inline and extracted notes, wrap-root, and notes outside
verses. If the count pass costs more than it saves, measure a structured-placeholder
alternative before deferring the stage.

Verification: direct HTML anchors byte-identical for the option matrix and worker counts;
benchmark clears the stage threshold.

### Stage 6 - parse

Parse remains in scope but is separate from the token-walker executor:

1. Lex the whole source serially.
2. During a serial lexeme partition pass, associate every chapter with a private
   `ParseSeed { book_code, book }`, tracking the latest preceding `\id` so malformed and
   multi-book inputs retain current behavior.
3. Parse the front and chapter lexeme slices, first through serial ordered map and then
   through native `map_ordered`.
4. Concatenate tokens, merge `ParseAnalysis` in source order, and run `assign_ids` once.
5. Do not expose a seeded public parser API unless an external caller independently
   needs it.

Verification: token/CST/export/lint oracles remain byte-identical across worker counts;
single-book parse clears the stage threshold. If serial lex dominates and the stage does
not clear the threshold, retain the proven implementation notes and defer activation.

## Risks and stop conditions

- Stop a stage when its serial decomposition cannot reproduce the oracle; do not debug
  partitioning and concurrency simultaneously.
- Stop or redesign HTML if its counter prepass erases the render win.
- Stop parse activation if serial lex dominates enough that chapter parsing misses the
  performance threshold.
- If a future visitor introduces genuine cross-chapter state, give that visitor an
  explicit ordered summary/merge contract; do not silently add a broad serial fallback.
- Keep all public entrypoint signatures unchanged unless a separate caller requirement
  justifies an API addition.
