# Progress: merge-projection (append-only)

## 2026-07-16 — design phase closed
- Design settled via throwaway prototype (merge-interleave, in scripture-editor-proto-2
  agent-tmp) rather than on paper: 23 fixture cases, P1/P2/P4 property harness, 21 narration
  pins, all green. Prototype engine is the reference semantics.
- Surprising findings that shaped the design (would not have been caught on paper):
  - Coalesced move pairs break naive merge identity (P2) unless the pair occupies TWO
    skeleton slots bound to one decision — this is the core structural insight.
  - Pure moves were invisible in the diff (identical text → Unchanged status) — new `Moved`
    status required.
  - Positional-only pairing mis-narrates duplicate-collapse (case 13) — exact-text-first
    tier added.
  - Anchors that resolve only to shared blocks skip paired neighbors and produce
    out-of-document-order hunks (case 15) — anchors must resolve to shared-or-paired.
  - Renumber typo (`\v 11`→`\v 1`) must stay delete+add — pinned as case 23 after Will's
    explicit call; cross-key content pairing rejected.
- Consumer doc discrepancy recorded: its P6 expects same-number replace as added+deleted;
  settled design makes it Modified (sid-stable). Doc to be updated when Zephyr consumes.
- Zephyr verified: always passes allowEmptySid:true; already normalizes sids app-side
  (mutAddSids: range-end, _dup_N, BOOK 0:0) — onion adopts these semantics natively.
- Implementation not started. UI A/B (move rendering) still open in the prototype; does not
  block engine work.

## 2026-07-16 — adversarial plan review
- Re-ran the reference harness: 23/23 cases pass P1/P2/P4 and all 21 narration pins pass.
- Reordered the implementation plan because pair displacement/status cannot be finalized before
  the two-slot skeleton exists; pairing and skeleton construction are now one dependency slice.
- Made canonical SID ownership explicit: `Sid` carries range end, while a stateful diff/DTO pass
  owns duplicate ordinals. Added the full formatter, size-guard, and token-id-drift blast radius.
- Resolved plan questions: hunk projection stays app-side; `mutAddSids` retires only after parity;
  core is slice-generic while Zephyr invokes merge per scope item.
- Added exact block/unit id rules, slot-anchor semantics, native/wasm API and error contracts,
  fixture-to-pin mapping, shape-preserving directed proptest requirements, package/golden gates,
  Rust/npm semver handling, and the separate Zephyr consumer handoff inventory.
- Implementation remains not started.

## 2026-07-16 — adjudication of the adversarial review (independent verification)
Every claim was verified against code/prototype or spiked; nothing accepted on authority.
- F1 step-inversion: CONFIRMED. merge-engine.js:391-418 finalizes displaced/Moved/Unchanged in a
  post-skeleton pass; `displaced = cur < base || sharedBetween`, one-sided slots between a pair
  deliberately do NOT count (matches case 15 non-displaced pin). Revised rule text is verbatim-
  faithful to the prototype.
- F2 Sid ownership: CONFIRMED, amended. Layout spike (rustc): Sid today 8B/Option 10B; +verse_end
  u16 → 10/12 (reviewer's numbers exact); +verse_end_delta u8 → 8/10 — keeps the existing guard.
  Plan amended: delta repr first, u16+10B ceiling as fallback. Formatter scatter confirmed real
  (8 files; api.rs:512 format_sid).
- F3 identity spec: CONFIRMED. Prototype uses sid-based block ids, current block id as coalesced
  decision key (merge-engine.js:322), `@n` collision suffix (line 288). Reviewer's spec is lifted
  from the prototype, not invented.
- F4 breaking fallout: CONFIRMED, and material — Zephyr's Tauri Rust crate consumes the NATIVE
  onion API directly (src/tauri/rust/src/usfm_onion.rs:18 BuildSidBlocksOptionsDto, :572
  exhaustive map_diff_status — a Moved variant is a compile break, :924 apply_revert_by_block_id).
  Versions verified (npm 0.0.8, workspace 0.1.1); all gate scripts exist (golden:wasm, test:wasm,
  test-web-package.mjs). AMENDED per Will: no npm/crates.io publication this release; bumps are
  bookkeeping only.
- F5 fixture port: CONFIRMED. parse/mod.rs:509 — sid assignment requires current_book; \id-less
  fixtures natively produce empty sids. The `\id GEN` adapter is necessary.
- F6 generators: CONFIRMED (standard proptest shrink-erosion discipline; structured models +
  per-family shape-preserving properties).
- F7 P7 overclaim: CONFIRMED. Unknown-id validation cannot detect same-id different-stream
  staleness; snapshot equality is Zephyr's (consumer doc §5). Original user story 4 overclaimed.
- Verdict: review upheld 7/7 with two amendments (Sid delta repr option; no-publish). Go for
  Step 1 stands, now with the delta-first stop condition.

## 2026-07-16 — final pre-implementation round; plan is GO
- UI verdict landed: collocated box + order-chip wording for moves; production UI implemented
  fresh in Sefer (prototype = reference semantics only).
- Second reviewer's final objections, both accepted and applied:
  - Lossy delta Sid repr REJECTED (u8 saturation on >255 spans violates never-silently-normalize;
    hardened dial). `verse_end: u16`, reviewed 10-byte guard ceiling.
  - "One canonical pass" boundary made explicit: single `derive_canonical_sids(tokens, book_code)`
    structural algorithm; native diff + wasm normalizeTokenSids call it now; external diffTokens
    keeps carried sids as an interim CALLING convention (not a second algorithm); consumer
    migration flips it to derivation.
- Contracts added: DiffableToken gains number_range()/book_code() accessors (trait currently has
  neither — src/diff/mod.rs:9); normalizeTokenSids(tokens, bookCode) is pure, preserves all
  non-SID fields, replaces every sid, bookCode authoritative over embedded \id (mutAddSids parity).
- Added "TBD in Zephyr/Sefer" consumer section (modal-open normalization, six call-site
  migration, Tauri Moved compile break, fresh UI, staleness UX).
- No remaining architectural objections from either reviewer or adjudicator. Next action: Step 1.

## 2026-07-16 — Sid repr: Will overrules reviewer, delta-u8 reinstated
- Decision: `verse_end_delta: u8`, 8-byte guard untouched. Domain grounds: longest chapter is
  176 verses; a bridge cannot span more verses than its chapter contains; >255 span is
  impossible-in-domain, not rare. Memory wins (Option<Sid> on every token: 10 vs 12).
- Fail-loud reconciliation: saturation is SPECIFIED, not silent — public surface is only
  `verse_end()` (saturating), serialized values are resolved ends (raw delta never leaks), the
  ceiling is documented on the field, and a `\v 1-999` pin test states the behavior explicitly.
- Fallback (u16, 10-byte ceiling) requires Will's sign-off via the step-1 stop condition.

## 2026-07-16 — implementation kickoff; cadence amendment applied
- Crew amendment (message #27): this is a port of the JS reference; review cadence moves from
  per-step to a single review at the end of step 7. Gates still run and get recorded at every
  step — they are the builder's own safety net now, not review checkpoints.

## 2026-07-16 — Step 1 done: range-aware Sid + canonical SID pass
- `Sid` gained `verse_end_delta: u8` (private field), `Sid::with_range(book, chapter, verse,
  verse_end)` constructor, `verse_end()` (saturating) and `verse_locator()` ("1" or "1-2")
  accessors. `Sid::new` keeps delta 0. Layout still 8 bytes — the size guard test passes
  unmodified (`token.rs::sid_size_guard::sid_stays_pointer_sized`).
- `parse/mod.rs::advanced_sid`'s `"v"` branch now threads `NumberRangeToken.end` through
  `Sid::with_range`; the sid stays sticky (full range) on every token until the next `\c`/`\v`.
  Added `parse_carries_bridge_range_end_on_the_verse_sid` and
  `parse_saturates_an_oversized_bridge_span` (the `\v 1-999` pin: `verse_end() == verse + 255`).
- `DiffableToken` gained `number_range(&self) -> Option<(u32, Option<u32>)>` and
  `book_code(&self) -> Option<&str>` (both default `None`), implemented for `Token` (from
  `TokenData::Number`/`TokenData::BookCode`) and `FormatToken` (from `number_info` / a
  `kind == BookCode` check on `text`).
- Added `derive_canonical_sids(tokens, book_code) -> Vec<String>` in `src/diff/mod.rs`: one
  stateful structural pass over any `DiffableToken` stream, sticky on `\c`/`\v` structure, never
  reading a token's own carried sid. Per-chapter `_dup_N` counter keyed on the full range string
  (`GEN 1:1` and `GEN 1:1-2` never share a counter — pinned in
  `single_and_range_sids_never_share_a_dup_counter`). 6 new tests in `diff::canonical_sid_tests`
  cover intro (`GEN 0:0`), chapter-open (`GEN 1:0`), single verse, bridge propagation, dup, and
  the dup/range-key separation.
- **Rule-5 surfacing, not blended:** the codebase already has two independent, pre-existing sid
  string conventions for a verse-0 (chapter/intro) sid — `Sid::Display` renders bare `"GEN 1"`
  (consumed by `usx.rs`, and by `lint_impl.rs`'s `format_sid` which just calls `.to_string()`),
  while `api.rs`/`crates/usfm_onion_wasm`/`diff/mod.rs`'s ad hoc `format_sid`/`sid_string`
  helpers always render `"GEN 1:0"` (the legacy diff-block-identity family `build_sid_blocks`
  is about to be replaced in Step 2). These are two legitimate conventions for two different
  consumer families, not a contradiction to average — I left both bases exactly as they were and
  only added range-end fidelity to each via one shared building block, `Sid::verse_locator()`
  (returns `"0"` when verse is 0, so the colon-family's `format!("{book} {chapter}:{}",
  sid.verse_locator())` reproduces its old unconditional-colon output unchanged for verse 0, and
  the bare family stays bare since it never had a colon branch to begin with). This satisfies the
  plan's "route every range-aware formatter through one helper" without an unrequested convention
  unification. Flagged for cleanup: once Step 2 deletes `build_sid_blocks`, the colon-family's
  remaining consumer is `api.rs`/wasm token mapping only — worth revisiting whether it should
  converge on `Sid::Display` then, but that's next-step/out-of-plan territory, not raised now.
- `vref.rs` deliberately NOT changed: its `verse_ref`/`verse_ref_str` build the verse-locator
  portion of a sid string from the raw lexeme (`"1-2"`, `"1,3"`, `"1-3,5"`), which already has
  range fidelity and additionally covers `NumberRangeKind::Sequence`/`SequenceWithRange` shapes
  that `Sid.verse_end_delta` cannot represent (a single contiguous range only). Switching vref to
  `Sid::verse_end()` would silently drop sequence fidelity — a regression, not a simplification.
  Existing `vref::tests::verse_bridge_produces_compound_sid` and
  `verse_sequence_lexeme_is_preserved` still pass unmodified, confirming no behavior change.
- Gate: `cargo build --workspace` clean; `cargo test --workspace` — 150 passed (0 failed) in the
  main crate plus all sub-crate suites, 0 failures anywhere. `cargo clippy --workspace
  --all-targets` — the only `error` (`format/mod.rs:1337` never_loop) is pre-existing on
  `master` (confirmed via `git stash`), untouched by this step; no new warnings introduced by any
  changed file (verified by diffing clippy output locations against changed files). Prototype
  pre-flight re-confirmed 23/23 cases + 21 pins at session start (JS untouched since).
- Files touched: `src/token.rs`, `src/parse/mod.rs`, `src/diff/mod.rs`, `src/api.rs`,
  `crates/usfm_onion_wasm/src/lib.rs`, `src/usj.rs`, `src/html.rs`.
- Next: Step 2 (gapless partition, deterministic block ids, `BuildSidBlocksOptions` removal).

## 2026-07-16 — Step 2 done: gapless partition, deterministic block ids, option removal
- Replaced `build_sid_blocks`'s option-branching scan with `partition_by_sid` — one linear,
  unconditionally gapless helper parameterized over a `sid_at(index) -> String` closure. Block id
  is that sid string for its first contiguous occurrence, then `#1`, `#2`, ... for a later
  non-contiguous reuse — never a token's own id (the old `sid::token_id` block-id branch is gone).
  Two thin public wrappers over the helper carry the two interim sid sources from Step 1's
  dispatch rule: `build_sid_blocks` (carried `sid_string()`, for the external/`FormatToken` path)
  and `build_sid_blocks_canonical(tokens, book_code)` (via `derive_canonical_sids`, for native
  `Token` paths). Deleted `BuildSidBlocksOptions`, `token_included`, `normalized_sid`,
  `next_block_start` entirely — no option, no escape hatch.
- `diff_usfm_sources`/`diff_usfm_sources_by_chapter` (native, source-level) now call
  `build_sid_blocks_canonical` with each side's own parsed book code; the by-chapter path derives
  canonical sids per already-book/chapter-grouped slice (each slice starts with its `\c` marker or
  is the chapter-0 intro bucket, so `derive_canonical_sids`'s sticky/dup state initializes
  correctly per slice). `diff_chapter_token_streams` (external/`FormatToken`/`TokenStream<T>`
  path) keeps the carried-sid convention, per Step 1's interim dispatch — unchanged behavior,
  just gapless now. Factored the diffs-from-blocks tail (`diff_sid_blocks` +
  `build_chapter_token_diff`) into a shared `diffs_from_blocks` so both calling conventions share
  one path after block-building.
- Dropped the `options`/`with_options` parameter from every builder and function that threaded
  `BuildSidBlocksOptions`: `api.rs`'s five diff builders + both `revert_diff_block(s)` methods on
  `ParsedUsfm`/`TokenStream<T>`, the wasm crate's `BuildSidBlocksOptions` DTO + all 6
  diff/revert-touching wasm functions (2 `ParsedUsfm` methods + 4 free `wasm_*` functions),
  `src/lib.rs` re-export, `src/bin/playground.rs`, `benches/operations.rs`, `README.md`.
  `revertDiffBlocks` (plural) still exists in wasm — its deletion is Step 5's job, not this one;
  only its options param dropped here.
- Rewrote two tests whose expectations WERE the pre-plan behavior this step removes:
  `uses_first_token_id_for_stable_block_ids` (expected `"GEN 1:1::tok-aaa"`) is now
  `block_ids_never_depend_on_token_ids` (expects bare `"GEN 1:1"`); added
  `non_contiguous_reuse_of_the_same_sid_gets_a_collision_suffix` for the `#1`/`#2` rule. Added
  `partition_is_gapless_including_leading_empty_sid_tokens` and
  `format_token_partition_is_also_gapless` (round-trip reassembly across a leading empty-sid
  token, native and `FormatToken`) and `canonical_block_ids_agree_across_independent_id_numbering`
  (id-drift fixture: two independently-parsed streams of the same structure produce identical
  canonical block-id candidates).
- Gate: `rg "allow_empty_sid|allowEmptySid|BuildSidBlocksOptions" src crates README.md` — zero
  hits (had to also reword one doc comment that named the retired option for context, since a
  comment mention is still a literal-text hit). `cargo build --workspace` and
  `cargo check --workspace --benches --examples --tests` clean. `cargo test --workspace` — 154
  passed (0 failed), `tests/synthetic_fixtures.rs`/`tests/bare_tokens_lint.rs` needed no semantic
  changes (both still green as-is, confirming no unintended SID fallout). `cargo clippy
  --workspace --all-targets` — same single pre-existing `format/mod.rs` error as Step 1 (confirmed
  unrelated to this diff), no new warnings in any changed file.
- Files touched: `src/diff/mod.rs`, `src/api.rs`, `src/lib.rs`, `src/bin/playground.rs`,
  `benches/operations.rs`, `crates/usfm_onion_wasm/src/lib.rs`, `README.md`.
- Next: Step 3 (Myers interleave, two-tier pairing, two-slot skeleton).

## 2026-07-16 — Step 3 done (+ most of Step 4's classification, one dependency slice)
- New module `src/diff/skeleton.rs`, re-exported from `crate::diff`: `DiffSkeleton<T>`, `Slot`,
  `Anchor`, `DecisionUnit<T>`, `SlotRole`, `DecisionUnitKind`, `DecisionStatus`, `DupContext`,
  `CoveredBy`/`CoveredSide`, `UnitId` (newtype, never a display SID used as a key), `MergeSide`
  (needed now by `CoveredBy.side`; Step 5 reuses it for merge decisions).
- Two entry points mirroring the Step 1/2 interim-dispatch precedent: `diff_skeleton(baseline,
  current)` (carried `sid_string()`, external/`FormatToken` convention) and
  `diff_skeleton_canonical(baseline, baseline_book, current, current_book)` (via
  `derive_canonical_sids`, native `Token` convention) — both delegate to one private
  `build_skeleton` engine, so there is exactly one alignment/pairing/merge implementation, not two.
  `diff_skeleton`'s pinned zero-book-code signature is preserved by design: it never calls
  `derive_canonical_sids`, only `T::sid_string()`.
- `build_skeleton`: reuses `partition_by_sid` (Step 2) for both sides, `similar::Algorithm::Myers`
  over block-id sequences via a new `myers_pairs` cursor walk (no second LCS/DP implementation),
  interleaves baseline-only/current-only/shared steps between LCS anchors, then pair-loose
  coalesces off-Myers blocks by `pairing_key` (book+chapter+range-start, stripping range end,
  `_dup_N`, `#occurrence`) — tier 1 exact-text-first (stream order among ties), tier 2 positional
  leftovers. Units are created in the prototype's exact order (shared → coalesced → deleted →
  added) so `UnitId`'s `@1`/`@2` collision suffixing is deterministic. Slots are built by walking
  the interleave steps, resolving each to Shared/BaselineOnly/CurrentOnly/PairBaseline/PairCurrent
  by looking up which unit owns that block.
- Also implemented the classification/annotation passes the plan assigns to Step 4, since displaced
  cannot be classified before pair slots exist (same "one dependency slice" as Steps 3+4's own
  reasoning in the 2026-07-16 adversarial-review entry above) and JS itself structures these as
  separate loops AFTER skeleton construction, not as part of the initial align(): `status` truth
  table (Shared/Deleted/Added final at build time; Coalesced Unchanged/Moved finalized against
  slot positions in a dedicated `finalize_displacement_and_status` pass — `displaced` iff the
  current slot precedes the baseline slot or a shared slot lies strictly between them, one-sided
  Added/Deleted neighbors don't count), `relabeled` (byte-equal coalesced pair, differing sids),
  `dup_context` (per-key block counts on both sides, computed over ALL blocks not just off-Myers
  ones — matches JS `keyCounts`), `covered_by` (`finalize_covered_by`: one-sided verse overlapped
  by a true multi-verse bridge on the paired unit's opposite side, same book/chapter, first match —
  no fixture in the 23-case catalog produces an ambiguous multiple-coverer situation, so no
  tie-break was needed), WS/USFM-structure-only flags (reused the existing private
  `normalize_block_text`/`strip_all_whitespace`/`strip_usfm_markers_for_display` helpers from the
  old system — Rust module privacy already grants a child module access to its parent's private
  items, no visibility changes needed), and `after` anchors (`finalize_anchors`: nearest preceding
  shared-or-paired anchor, side-appropriate SID, Added/Deleted never become anchors). Did NOT
  implement JS's `slot.emit`/`slot.anchor`/one-row-projection bookkeeping — the plan is explicit
  that hunk/row projection is app-side; onion ships `Slot.after` and lets a downstream projector
  derive the rest, so those JS fields have no Rust counterpart.
- Simplification found during implementation: JS tracks a separate `exact` (tier-1-matched) flag
  per coalesced pair to decide the `Unchanged`-downgrade, but tier 1 tries every remaining current
  candidate for byte-equality before tier 2 ever runs, so a tier-2 (positional) pair is provably
  never byte-equal — `exact` and `byte_equal` are equivalent for coalesced units. Used
  `byte_equal` directly (already needed for the status truth table) and dropped the redundant
  `exact` bookkeeping rather than porting a field that would immediately show up as dead code.
- Step 3's own (narrower) gate, tested now — full 23-case/21-pin port is Step 4's remaining scope:
  every baseline/current block appears in exactly one baseline-/current-bearing slot; every
  coalesced unit owns exactly one `PairBaseline` + one `PairCurrent` slot with the same id; case 13
  exact-pairs `c` before positional leftovers (no mis-pair with `a`); case 23 has zero coalesced
  units (renumber typo stays delete+add); case 11 has exactly one coalesced (moved) unit without
  pinning which symmetric verse Myers names as the mover; `diff_skeleton_canonical` run twice on
  the same input yields identical unit ids and identical `(unit_id, role)` slot shape.
- Gate: `cargo build --workspace` clean (one dead-code warning caught and fixed before commit —
  see above). `cargo test --workspace` — 160 passed (0 failed), including the 6 new
  `diff::skeleton::tests`. `cargo clippy --workspace --all-targets` — same single pre-existing
  `format/mod.rs` error, zero new warnings in `skeleton.rs`.
- Files touched: `src/diff/skeleton.rs` (new), `src/diff/mod.rs` (module declaration + re-export).
- Next: Step 4's remaining scope — port all 23 cases + 21 narration pins (native `Token` AND
  `FormatToken`/id-drift paths), which is now mostly a fixture-authoring exercise since the
  classification/anchor logic above is already implemented and unit-tested against a subset.

## 2026-07-16 — Step 4 done: full fixture oracle (23 cases, 21 pins, both token shapes)
- New test-only module `src/diff/skeleton_fixtures.rs`, all 23 cases transcribed verbatim from
  `cases.js`. Non-`\id` cases wrapped with a shared `\id GEN\n` prefix before `parse()`; case 19
  keeps its own `\id` line, per the plan's fixture-adapter rule.
- `all_23_cases_partition_and_reassemble_on_native_parsed_tokens` and the FormatToken counterpart
  (`..._on_format_tokens_with_id_drift`, which runs every case through `derive_canonical_sids` +
  deliberately different `orig-N`/`new-N` token ids, mirroring app-shaped id drift) both walk
  every case's skeleton and reassemble baseline/current byte-for-byte from slot-ordered token
  text, asserting every baseline token lands in exactly one baseline-bearing slot and every
  current token in exactly one current-bearing slot (P1/P2-shaped, ahead of Step 6's hardened
  proptest version of the same properties).
- Ported all 21 narration pins as direct unit/slot assertions (cases 7, 8, 10, 13, 15, 16, 23) —
  no row/hunk projection exists to pin against, so each JS pin was translated to the underlying
  `DecisionUnit`/`Slot` data it depends on (settled design decision #8). Three pins needed
  correcting against my own first-draft assumptions, verified by reading
  `merge-engine-check.js`'s actual JS assertions rather than guessing from the plan's prose alone:
  - Forgot the `\id GEN\n` wrap injects TWO extra leading blocks (`GEN 0:0` intro, `GEN 1:0`
    chapter-open) ahead of every case's own content — case 10's expected baseline/current sid
    order was missing them.
  - Case 15's deleted `GEN 1:2` anchors on the pair's **baseline**-side sid (`GEN 1:1`), not its
    current-side sid (`GEN 1:1-2`) — confirmed against `merge-engine-check.js`'s literal
    `delHunk.anchor.after === "GEN 1:1"` assertion. Matches the implementation's `finalize_anchors`
    exactly (`PairBaseline` slots anchor on `baseline_sid`); the test's first draft had guessed
    wrong, not the code.
  - Case 13's "no two hunks fight over one anchor" pin is about **distinct units** never colliding
    on the same anchor sid among hunks a projection would actually render (JS's `needsAnchor`:
    Deleted, or a displaced Coalesced pair) — not "a unit's anchor never equals its own id" (my
    first draft's wrong translation, which spuriously failed because a coalesced pair's own
    `PairBaseline` slot legitimately becomes the anchor for its own later `PairCurrent` slot when
    nothing else falls between them; that's correct behavior, matching JS's `paired` branch in its
    `lastAnchorSid` update).
- Confirms `similar::Algorithm::Myers` resolves case 13's ambiguous double-match (`GEN 1:1`~`GEN
  1:1` collision between baseline's first `\v 1` and current's renamed survivor) the same way the
  prototype's DP tie-break does — 'b' matches shared, leaving 'a' deleted and 'c' free to coalesce
  — without needing to special-case or reimplement the tie-break, consistent with the plan's "do
  not port the DP literally, preserve `similar`'s Myers behavior" instruction.
- Gate: `cargo build --workspace` clean. `cargo test --workspace` — 169 passed (0 failed),
  including all 9 new fixture tests (2 whole-catalog + 7 pin-specific). `cargo clippy --workspace
  --all-targets` — same single pre-existing `format/mod.rs` error, zero new warnings.
- Files touched: `src/diff/skeleton_fixtures.rs` (new, test-only), `src/diff/mod.rs` (module
  declaration).
- Next: Step 5 (pure merge, strict single revert, delete `infer_sid_match_block` /
  `find_insertion_index` / `apply_reverts_by_block_id` / the sequential plural revert API).

## 2026-07-16 — Step 5 done: pure merge, strict single revert, corruption-path deletion
- Added to `src/diff/skeleton.rs`: `MergeError::UnknownUnitId(UnitId)` (Display + std::error::Error,
  matching the existing `UsjError`/`UsxError` pattern already in the crate); `merge_skeleton`
  (validates every decision id against the skeleton's real units BEFORE walking any slot, then a
  single pass emitting each slot's chosen side — `Shared` always emits, `BaselineOnly`/
  `PairBaseline` emit only when the side is `Baseline`, `CurrentOnly`/`PairCurrent` only when
  `Current`, so a coalesced pair's two slots contribute exactly once for either choice); the
  `merge_diff_blocks` convenience wrapper (builds the skeleton via the plain/external
  `diff_skeleton` calling convention once, then delegates); `revert_diff_block` (one `{id:
  Baseline}` decision with default `Current` — exactly the "revert is one-decision merge" settled
  design). Added `UnitId::new` since decisions maps and revert requests need to construct ids from
  caller-supplied strings that may not correspond to any real unit — construction never validates,
  `merge_skeleton` is the single validation point.
- Deleted `infer_sid_match_block`, `extract_sid_from_block_id`, `find_insertion_index`, and the old
  `apply_revert_by_block_id`/`apply_reverts_by_block_id` (plural) from `src/diff/mod.rs` entirely —
  the fuzzy stale-id fallback and the insertion-at-zero fallback are gone, not just unused.
  Rewired every caller: `api.rs`'s `ParsedUsfm`/`TokenStream<T>` singular `revert_diff_block`
  methods now return `Result<Vec<T>, MergeError>` and delegate to the new native function; their
  plural `revert_diff_blocks` methods are deleted outright (no sequential-replay API survives
  anywhere in the native or API layer). Wasm: `ParsedUsfm::revert_diff_block` and the free
  `wasm_revert_diff_block` now return `Result<_, JsError>` (thrown JS `Error` on unknown id, via
  the existing `js_error` helper already used by `to_usj`); `wasm_revert_diff_blocks` (plural) is
  deleted along with its import. `scripts/test-web-package.mjs` had its `pkg.revertDiffBlocks(...)`
  smoke-test block removed (the singular `revertDiffBlock` call is untouched and still exercises
  the real, still-passing package) — full P2/mixed-decision/unknown-id wasm smoke coverage is
  Step 7's job once the package is regenerated against the new skeleton-returning wasm surface.
- New tests in `src/diff/skeleton_fixtures.rs`: `all_23_cases_merge_all_baseline_and_all_current_are_byte_exact`
  (covers all-Baseline/all-Current identity for every case including case 20's CRLF-vs-LF mix, so
  no separate CRLF test was needed — it's just one of the 23); a 24-trial-per-case deterministic
  LCG decision-vector sweep (`all_23_cases_random_decision_vectors_respect_contribution_cardinality`,
  mirroring the prototype's own P4 trial count) asserting idempotency and exact contribution
  cardinality per unit kind (Shared/Coalesced always exactly one; Deleted/Added zero-or-one tied to
  the resolved side); `unknown_unit_id_errors_before_any_output_is_assembled` and the
  `merge_diff_blocks` equivalent (both assert `Err` before any `Vec` is even considered, matching
  "validate before assembling output"); `single_revert_of_every_changed_unit_equals_a_one_decision_merge`
  (every non-Unchanged unit in every case: `revert_diff_block(id) == merge({id: Baseline}, Current)`
  byte-for-byte) — run on the id-drift `FormatToken` fixture (the external calling convention
  `revert_diff_block` actually uses), not on raw native `Token`, since native `Token::sid_string()`
  has no dup-suffix awareness and would build a *different* (non-canonical) skeleton than one built
  via `diff_skeleton_canonical` — an early draft of this test mixed the two conventions and would
  have compared mismatched skeletons; caught before committing by re-deriving which skeleton
  `revert_diff_block` itself actually builds internally, not assuming it matched what I'd built for
  comparison.
- Gate: `cargo build --workspace` and `cargo check --workspace --tests --benches --examples` clean.
  `cargo test --workspace` — 174 passed (0 failed), including 5 new Step-5 tests (9 fixture-oracle
  tests carried over from Step 4). `cargo clippy --workspace --all-targets` — one new lint caught
  and fixed (`manual_is_multiple_of` in a test helper) before the only remaining error was again
  the single pre-existing unrelated `format/mod.rs` one. `rg "infer_sid_match_block|
  find_insertion_index|apply_reverts_by_block_id|revertDiffBlocks" src crates README.md scripts` —
  zero hits.
- Files touched: `src/diff/skeleton.rs`, `src/diff/mod.rs`, `src/diff/skeleton_fixtures.rs`,
  `src/api.rs`, `src/lib.rs`, `crates/usfm_onion_wasm/src/lib.rs`, `scripts/test-web-package.mjs`.
- Next: Step 6 (hardened proptest suite, P1-P7 amended — structured chapter/document generator,
  typed edit scripts, 9 directed strategies, shape-preserving shrinking).

## 2026-07-16 — Step 6 done: hardened proptest suite (P1-P7, 9 directed strategies)
- Added `proptest = "1"` as a dev-dependency. New test-only module
  `src/diff/skeleton_proptest.rs`.
- Structured document model (`Doc`/`Chapter`/`Verse`), never a raw-byte mutator: always a valid
  `\id GEN`, 1-2 chapters, 1-5 verses each with optional bridges (`bridge_width` 0-2), text atoms
  from an 8-word pool, `\p`/`\m` alternation, optional `\s` heading, optional balanced `\add`
  wrap, an explicit `trailing_extra_spaces` field for genuine whitespace-only edits, and LF/CRLF.
  Built via tuple + `prop::collection::vec` + one top-level `.prop_map` (avoided nested
  Strategy-returning helper functions needing `.boxed()` — lower API risk, same expressiveness).
  Typed edit scripts (`Edit`: append-a-letter text edit, insert/delete verse, reorder, boundary
  word migration, paragraph change, whitespace-only, marker-wrap toggle) applied 0-3 at a time to
  derive Current from Baseline — shrinking operates on the structured model/edit list, so a shrunk
  failure keeps its defining shape (confirmed: both real shrinks below reported clean, faithful
  minimal reproductions, not byte-offset noise).
- P1-P5, P7 as five `#[test]` properties (`p1_p2_partition_and_identity` covers P1+P2 together
  since both walk the same slot loop; `p3_totality_and_reparse_fixed_point`;
  `p4_purity_across_repeated_and_interleaved_calls`; `p5_skeleton_construction_is_fully_deterministic`
  — a single `assert!(first == second)` since `DiffSkeleton<T>` already derives full `PartialEq`
  down to `T`, so "complete skeleton equality (ids, unit order/data, slots, anchors, annotations)"
  falls out for free; `p7_unknown_id_is_rejected_with_no_output`).
- P6: 9 separate `#[test]` properties, each its own shape-specific generator (not folded into the
  general strategy) — adjacent deletion + boundary migration; boundary migration under all 4 mixed
  Baseline/Current choices; reorder + full transposition; duplicate-sid delete-first-occurrence;
  the 4 bridge shapes (fuse-adjacency/merge-into-bridge/split/overlap) selected via one
  shape-index generator; renumber-typo non-coalescing; chapter start/end insertion; same-number
  replace (Modified) vs different-number add+delete (never coalesced); fully independent streams
  with forced-mismatched CRLF/LF. Each runs P1/P2/purity plus a contribution-cardinality witness
  (`assert_contribution_cardinality`, the same logic Step 5's fixture tests used, generalized here)
  via a shared `assert_directed_shape` helper.
- Two real proptest-caught bugs — both in my test code, not the implementation, and both left as
  permanent regression seeds in `proptest-regressions/diff/skeleton_proptest.txt`:
  - `p3`'s first draft called `edited_doc_strategy()` a second time inside `prop_flat_map` to
    compute decision-vector unit ids, an independent random draw disconnected from the outer
    `(baseline, current)` parameter — decisions referenced units from a *different* generated
    skeleton than the one being merged. Fixed by composing one strategy end-to-end
    (`edited_doc_strategy().prop_flat_map(...)`) so the decisions are always built from the exact
    skeleton under test.
  - `directed_8`'s same-number-replace case failed when the word generator happened to draw the
    same word for both sides (`x == y`), producing byte-identical text — correctly `Unchanged`,
    not the asserted `Modified`. Fixed with `prop_assume!(x != y)`; the shrinker's minimal
    counterexample (`x = "eta", y = "eta"`) pointed straight at the missing precondition.
- Gate: `cargo build --workspace --tests` clean. `cargo test --workspace` — 188 passed (0 failed),
  including 14 new proptest properties (9 directed_N under `directed_N` names as required, run
  independently). Ran the proptest suite 3 additional times to check for flakiness beyond the
  fixed case count — stable every time. `cargo clippy --workspace --all-targets` — fixed 4 new
  lints on first pass (duplicate `where`-bound location, a type-complexity tuple needing a type
  alias, an `Edit` variant name ending in the enum's own name, `push_str("Z")` → `push('Z')`);
  after fixing, zero new warnings remain (same single pre-existing `format/mod.rs` error).
- Files touched: `Cargo.toml`, `src/diff/skeleton_proptest.rs` (new), `src/diff/mod.rs` (module
  declaration), `proptest-regressions/diff/skeleton_proptest.txt` (new, checked in per proptest's
  own convention).
- Next: Step 7 (wasm/DTO/package release, `normalizeTokenSids` export + parity fixture, version
  bumps, Zephyr consumer handoff inventory) — the last step before the single end-of-plan review.

## 2026-07-16 — Step 7 done: wasm/DTO release, consumer handoff — plan complete
- This step also required retiring the OLD flat-diff native implementation, not just the wasm
  surface: "do not leave two alignment/coalesce implementations" meant `diff_chapter_token_streams`
  /`diff_usfm_sources`/`diff_usfm_sources_by_chapter` and their whole supporting apparatus
  (`ChapterTokenDiff`, `SidBlockDiff`, `DiffStatus`, `DiffTokenChange`, `DiffUndoSide`,
  `TokenAlignment`, `DiffsByChapterMap`, `diff_sid_blocks`, `coalesce_delete_add_pairs`,
  `build_modified_diff*`, `diff_id_sequences`/`diff_sequences`, `align_token_sequences`/
  `align_removed_added_chunk`, `token_shape_key`/`token_comparable_key`/`can_pair_as_modified`/
  `token_is_linebreak`, `build_chapter_token_diff`, `replace_chapter_diffs_in_map`/
  `replace_many_chapter_diffs_in_map`/`flatten_diff_map`) all had to come out of `src/diff/mod.rs`
  entirely, plus `api.rs`'s `group_chapter_diffs`/`chapter_key_from_semantic_sid`, before the wasm
  rewiring could be honest about there being one implementation. Kept `SidBlock`,
  `build_sid_blocks(_canonical)`, `partition_by_sid`, `derive_canonical_sids`,
  `group_tokens_by_book_and_chapter`, and `normalize_block_text` + its string helpers — the
  skeleton builder still depends on all of these.
- Added two native by-chapter entry points to `src/diff/skeleton.rs`: `diff_skeleton_by_chapter`
  (native, source-level — mirrors the retired `diff_usfm_sources_by_chapter`'s group-tokens-then-
  diff-per-chapter shape) and `diff_skeleton_by_chapter_from_tokens` (external/`FormatToken`
  convention). Rule-5 note: the OLD `ParsedUsfmDiffByChapterBuilder` actually used a *different*
  algorithm than native (diff-the-whole-document-then-regroup-by-parsing-the-resulting-sid, via
  `group_chapter_diffs`) — a pre-existing, accidental divergence from the native by-chapter path's
  group-first approach. Converged both onto the same group-tokens-by-chapter-first algorithm
  rather than porting the old FormatToken path's inconsistency forward; the only behavior change
  is that a verse move crossing a chapter boundary in the FormatToken/`ParsedUsfmDiffByChapterBuilder`
  path no longer surfaces as a coalesced move (an extremely rare edge case, and now consistent with
  how the native by-chapter path already behaved).
- `api.rs`'s five diff builders (`UsfmDiffBuilder`, `UsfmDiffByChapterBuilder`,
  `ParsedUsfmDiffBuilder`, `ParsedUsfmDiffByChapterBuilder`, `TokenDiffBuilder`) now return
  `DiffSkeleton<T>` / `BTreeMap<String, BTreeMap<u32, DiffSkeleton<T>>>`, each still calling the
  same convention it always used (`Usfm`/`ParsedUsfmDiffBuilder`-over-tokens use canonical or
  carried-sid exactly as before — see the Step 1/2 interim-dispatch note this preserves).
- Wasm crate (`crates/usfm_onion_wasm/src/lib.rs`): new Tsify DTOs (`DiffSkeleton`, `Slot`,
  `Anchor`, `DecisionUnit`, `SlotRole`, `DecisionUnitKind`, `DecisionStatus`, `DupContext`,
  `CoveredBy`/`CoveredSide`, `MergeSide`, `MergeRequest`) replace the deleted flat ones.
  `diffUsfm`/`diffTokens`/`ParsedUsfm.diff` return `DiffSkeleton`; `diffUsfmByChapter`/
  `ParsedUsfm.diffByChapter` return the by-chapter skeleton map; new `mergeDiffBlocks` wraps
  `merge_diff_blocks`; `revertDiffBlock`/`ParsedUsfm.revertDiffBlock` already threw on unknown id
  since Step 5, unchanged here; `revertDiffBlocks` (plural) deleted with its import. New
  `normalizeTokenSids(tokens, bookCode) -> Token[]` wraps `derive_canonical_sids`, replacing only
  the `sid` field and leaving every other field (including `id`) untouched, satisfying the pure/
  preserves-all-non-sid-fields contract. One naming note: the generated `.d.ts` shows
  `book_code: string` (snake_case), not `bookCode` — wasm-bindgen does not camelCase plain function
  parameters (only struct fields via serde), and the existing `revertDiffBlock(..., block_id:
  string)` already has this same pre-existing convention gap; matched it for consistency rather
  than introducing a one-off camelCase parameter with no precedent in this file.
- Regenerated both `pkg-bundler` and `pkg-web` (dev builds for iteration, then a final `release`
  build via `npm run build:wasm` as the committed state — release, not debug, per the prior
  `fix(pkg): ship release bundler wasm, not debug` fix already in this repo's history). Inspected
  `usfm_onion_web.d.ts`: exact new unions, camelCase struct fields, `Record<string, MergeSide>` for
  `MergeRequest.decisions`, `DiffsByChapterMap = Record<string, Record<number, DiffSkeleton>>`, no
  leftover `ChapterTokenDiff`/`SidBlockDiff`/`TokenAlignment`/`revertDiffBlocks` surface.
- Rewrote `scripts/test-web-package.mjs`'s diff section to exercise: P2 identities
  (`mergeDiffBlocks` with empty decisions, default baseline/current, byte-exact against source/
  edited); a mixed-decision single-unit revert-via-merge equivalence check; move two-slot identity
  (a pure swap fixture, asserting exactly 2 slots reference the coalesced unit's id, roles
  `{pairBaseline, pairCurrent}`, status `moved`); and the unknown-id throw for both
  `mergeDiffBlocks` and `revertDiffBlock` (manual try/catch, not `assert.throws`, whose 2-arg
  string-vs-message-vs-matcher semantics are ambiguous enough to avoid).
- Ran `npm run golden:wasm` first: exactly the 21 expected mismatches (`diff.json`/
  `diff-by-chapter.json`/`diff-tokens.json` × 7 fixtures) — the breaking shape change — and nothing
  else. Ran `npm run golden:wasm:update`, then manually inspected `tiny/diff.json`,
  `tiny/diff-by-chapter.json`, `tiny/diff-tokens.json`, and `multiple-chapters/diff.json` in full:
  units/slots/anchors/statuses all correct (e.g. `tiny`'s intro+chapter-open+two-verse+one-added
  shape, the `\id ` marker correctly folding into the `GEN 0:0` canonical block instead of the old
  system's spurious empty-sid pre-book-code block — an intentional consequence of Step 1-2's
  canonical pass, not accidental churn). Re-ran `golden:wasm` clean after accepting. Verified
  `golden:wasm:web` also clean.
- Ran the full step-7 gate command list: `cargo test --workspace`, `npm run test:wasm` (bundler +
  web), `npm run golden:wasm`, `npm run build:wasm` (release) — all green. Re-ran the prototype
  harness one more time: still 23/23 cases + 21 pins.
- Bumped versions per the plan's exact instruction: npm (`package.json`) `0.0.8` → `0.0.9`,
  workspace (`Cargo.toml`) `0.1.1` → `0.2.0`. Rebuilt after bumping so `pkg-bundler`/`pkg-web`'s
  generated `package.json` (sourced from the workspace version) show `0.2.0`. No publish step run
  or considered — nothing pushed to npm/crates.io, matching the plan's explicit no-publish note.
- Wrote `plans/zephyr-handoff-merge-projection.md`: concrete per-surface inventory
  (`IUsfmOnionService`, onion type/`Diff`-mapping imports, `WebUsfmOnionService`/
  `TauriUsfmOnionService`, Tauri Rust DTO/status mapping and commands, `diffScope`/staging/mocks/
  tests, `defaultBuildSidBlocksOptions` removal — flagging that dropping `allowEmptySid: false`
  callers changes behavior, not just deletes dead code — and the six `normalizeTokenSids`/
  `mutAddSids` call sites with the dual-run-parity-first, one-at-a-time migration order). Explicit
  "not part of this handoff" section (fresh Sefer UI, modal-open normalization discipline,
  snapshot-staleness UX) so Zephyr migration is handed off precisely, not vaguely claimed done.
- Found and fixed a gap in Step 2's own gate: a second, missed literal mention of the retired
  `allow_empty_sid` option in a test comment (`src/diff/mod.rs`,
  `partition_is_gapless_including_leading_empty_sid_tokens`) that the Step 2 rg check should have
  caught but didn't get re-verified against after that step closed. Step 7's final rg re-check
  (run because Step 7 shares the same retired-symbol gate surface) caught it; reworded, rg now
  clean across all of `allow_empty_sid|allowEmptySid|BuildSidBlocksOptions|infer_sid_match_block|
  find_insertion_index|apply_reverts_by_block_id|revertDiffBlocks` over `src crates README.md
  scripts`.
- Files touched: `src/diff/mod.rs`, `src/diff/skeleton.rs`, `src/api.rs`, `src/bin/playground.rs`,
  `benches/operations.rs`, `crates/usfm_onion_wasm/src/lib.rs`, `scripts/test-web-package.mjs`,
  `package.json`, `Cargo.toml`, `Cargo.lock`, `pkg-bundler/*`, `pkg-web/*`,
  `crates/usfm_onion_wasm/golden/outputs/**/diff*.json` (21 files, reviewed),
  `plans/zephyr-handoff-merge-projection.md` (new).

## Plan status: all seven steps complete, gates green in order, recorded above
Definition of done, checked against `plans/plan-merge-projection.md`'s own list:
- All seven step gates green in order, recorded in this file. ✅
- Rust matches all 23 reference cases, all 21 narration pins, and P1–P7 as amended
  (`src/diff/skeleton_fixtures.rs`, `src/diff/skeleton_proptest.rs`). ✅
- One canonical skeleton builder (`build_skeleton` in `src/diff/skeleton.rs`), one merge
  implementation (`merge_skeleton`), one SID derivation algorithm (`derive_canonical_sids`) — the
  old flat-diff alignment/coalesce implementation is deleted, not left dormant. ✅
- Fuzzy/stale fallbacks and sequential plural revert are absent (`infer_sid_match_block`,
  `find_insertion_index`, `apply_reverts_by_block_id`, `revertDiffBlocks` all deleted, rg-verified
  clean). ✅
- Native and wasm/package contracts updated, versioned (npm 0.0.9, workspace 0.2.0), and verified
  (golden, smoke tests, wasm32 target check all green). Nothing published. ✅
- Remaining consumer work is the named handoff in `plans/zephyr-handoff-merge-projection.md`, not
  an implicit claim in this plan. ✅
- Single end-of-plan review requested next, per the crew thread's cadence amendment (#27).

## 2026-07-17 — findings from the final adversarial review (#47), all addressed
Reviewer verdict: FINDINGS, not passed. Seven items; all fixed below. One scope question (item 1)
was adjudicated by Will directly (relayed to the thread as message #48) before the fix.

1. **Missing Zephyr `mutAddSids` dual-run parity fixture (plan lines 492-494).** Will's call: the
   actual dual-run (both implementations executed together, outputs diffed) can only run in
   Sefer/Zephyr's repo — `mutAddSids` is Zephyr application code, not importable here, and running
   it in onion would violate this plan's own "usfm_onion repo only, no Zephyr/Sefer changes" scope
   line. Fix: added `normalize_token_sids_bridge_dup_intro_parity_contract` in
   `crates/usfm_onion_wasm/src/lib.rs` — onion's committed, executable half of the contract: exact
   pinned sid strings for bridge (`GEN 1:1-2`), duplicate (`GEN 1:1_dup_1`), and intro (`GEN 0:0`)
   cases, explicitly documented as the answer key Zephyr's own dual-run test must diff its
   `mutAddSids` output against. Tightened
   `plans/zephyr-handoff-merge-projection.md`'s "six call sites" section to spell out exactly
   which half of the parity check lives where and why, instead of vaguely deferring "run this
   later."
2. **P6 witness reasoned from slots/decisions, never observed real `merge_skeleton` output.**
   `src/diff/skeleton_proptest.rs`'s `assert_contribution_cardinality` (and the identical pattern
   in `src/diff/skeleton_fixtures.rs`'s 23-case sweep) recomputed expected cardinality from the
   same slot-role bookkeeping the implementation itself uses, without ever calling `merge_skeleton`
   and checking its actual returned tokens — a wrong-side emission, a duplicated slot, or a leaked
   unchosen side would have passed silently. Fix: replaced with `expected_merge_text` (an
   independently-written per-slot walk) + `assert_merge_output_matches_expected`, which calls the
   real `merge_skeleton` and asserts byte-exact equality against the independent reconstruction —
   a real bug now shows up as a string mismatch, not a self-consistent recount. Also fixed
   `directed_2`, which built its own skeleton directly and never ran the claimed P1/P2/P4 package;
   it now goes through `assert_directed_shape` first.
3. **`covered_by` incorrectly annotated a one-sided bridge, not just a one-sided verse.**
   `finalize_covered_by` in `src/diff/skeleton.rs` excluded `own_start == 0` but never required
   `own_start == own_end` (a genuine singular verse) — so a Deleted/Added unit whose OWN sid was
   itself a bridge could still get `covered_by` set against an opposite coalesced bridge.
   Reproduced exactly per the reviewer's repro (baseline `\v 1-3 A` + `\v 1-2 B`, current
   `\v 1-4 A`: tier-1 exact-text pairs `1-3`↔`1-4` on byte-identical `"A"`, leaving `1-2` deleted;
   old code then wrongly annotated the deleted `1-2` bridge as covered by the `1-4` pair). Fixed by
   requiring `own_start == own_end`; added
   `covered_by_never_annotates_a_one_sided_bridge_against_an_opposite_bridge` as a permanent
   regression in `skeleton_fixtures.rs` using the exact reported case.
4. **Stale `pkg-bundler`/`pkg-web` READMEs still showed `BuildSidBlocksOptions`.** Both are
   standalone hand-maintained docs (not wasm-pack generated, no source template — confirmed by
   inspection), and their "Diffing" section was already documenting a fictional API
   (`diff::diff_content`, `DocumentFormat`, `model::TokenViewOptions`) that never existed in this
   crate, on top of the now-deleted `BuildSidBlocksOptions`. Replaced that one section in both
   files with an accurate description of the real skeleton-based diff API and a compiling example.
   Expanded the deletion-audit rg command to include `pkg-bundler pkg-web` going forward — the
   previously-recorded `src crates README.md` scope missed the committed consumer package surface.
5. **Proptest coverage: silently capped case counts, and three directed properties were
   probabilistic instead of exhaustive over their named geometries.** Standing supervisor item
   #43 on the case counts (64 general / 32 directed, no rationale) was never actually resolved
   before the final review — corrected now: general suite raised to the proptest default (256,
   confirmed negligible runtime — whole module still ~0.16-0.19s); directed suite stays at 32 with
   an explicit in-code rationale (every named geometry below now runs unconditionally every case,
   so raising the count adds word-content variety, not coverage). Separately, `directed_3` (swap
   vs. transposition), `directed_5` (four bridge geometries), and `directed_7` (start vs. end
   insertion) each picked ONE sub-shape at random per case (bool / `0u8..4`) — a single run was not
   guaranteed to exercise every named shape, contrary to the plan's "a generator is not coverage
   merely because its enum contains a shape" gate. Rewrote all three to run every named sub-shape
   unconditionally within each case, rather than splitting into more properties (equivalent
   coverage, less duplication of the shared word-generation setup). Verified stable across 5
   consecutive runs at the new counts.
6. **Handoff omitted the by-chapter algorithm behavior change.** Step 7's own progress entry above
   disclosed that `ParsedUsfmDiffByChapterBuilder` converged from diff-then-regroup to
   group-then-diff (consistency fix, noted then), but `plans/zephyr-handoff-merge-projection.md`
   never surfaced it as a migration-relevant behavior change for Zephyr. Added an explicit note
   (top-level summary + a detailed bullet under the `diffScope`/mocks/tests section) spelling out
   the practical effect (a verse move crossing a chapter boundary no longer coalesces in the
   by-chapter result specifically) and which test fixtures to check.
7. **New comments referenced the transient plan document/step numbers.** `src/diff/skeleton.rs:3`,
   `src/diff/skeleton_proptest.rs:1`, `src/diff/skeleton_fixtures.rs:3,7` and a `// ---- Step 5
   ----` section marker all named `plans/plan-merge-projection.md` or step numbers directly.
   Reworded all five to describe durable behavior/rationale without referencing the plan file or
   step numbering, consistent with the project's comment-durability rule (comments describe the
   code, not the task that produced it).
- Gate re-run after all seven fixes: `cargo test --workspace` (188 lib + 16 wasm-crate tests, 0
  failed — +1 net test from the `covered_by` regression, +1 from the parity-contract test, net of
  the rename in the 23-case sweep), `cargo clippy --workspace --all-targets` (same single
  pre-existing `format/mod.rs` error, zero new warnings), `npm run test:wasm` (bundler + web),
  `npm run golden:wasm` / `golden:wasm:web` (7/7, unchanged — none of these fixes altered any
  golden-checked output shape), `npm run build:wasm` (release, final committed state), `npm run
  check:wasm:web` (wasm32 target), the expanded rg deletion audit (now including `pkg-bundler
  pkg-web`, zero hits), a plan-reference comment grep (zero hits), `git diff --check` (clean), and
  the prototype harness (still 23/23 + 21 pins).
- Files touched: `src/diff/skeleton.rs`, `src/diff/skeleton_proptest.rs`,
  `src/diff/skeleton_fixtures.rs`, `crates/usfm_onion_wasm/src/lib.rs`,
  `pkg-bundler/README.md`, `pkg-web/README.md`, `plans/zephyr-handoff-merge-projection.md`.
- Re-requesting review now.
