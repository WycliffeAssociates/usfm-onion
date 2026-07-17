# Zephyr/Sefer handoff: merge-as-projection

Onion-side work from `plans/plan-merge-projection.md` is complete and reviewed in this repo.
This document is the consumer migration inventory referenced by that plan's Definition of Done —
**none of this is onion's work**; it lists what Zephyr/Sefer must change to consume the new
`usfm_onion_web` package (npm `0.0.9`, workspace `0.2.0`). Nothing was published to npm or
crates.io; Zephyr consumes the regenerated, committed `pkg-bundler`/`pkg-web` directories directly.

## What changed on the onion side (context for the sections below)

- The flat `ChapterTokenDiff[]` / `SidBlockDiff` / `DiffStatus` / `DiffTokenChange` /
  `DiffUndoSide` / `TokenAlignment` wasm return shapes are **gone** — replaced everywhere by a
  `DiffSkeleton` (`{ slots: Slot[], units: DecisionUnit[] }`).
- `diffUsfm`, `diffTokens`, `ParsedUsfm.diff` all return `DiffSkeleton` now. `diffUsfmByChapter` /
  `ParsedUsfm.diffByChapter` return `Record<string, Record<number, DiffSkeleton>>` (one skeleton
  per chapter) instead of `Record<string, Record<number, ChapterTokenDiff[]>>`.
- New `mergeDiffBlocks(baseline, current, { decisions, defaultSide }) -> Token[]` — pure
  projection merge. `decisions` is `Record<string, "baseline" | "current">`.
- `revertDiffBlock(...)` keeps its name and shape but now **throws** a JS `Error` on an unknown
  block id, instead of silently falling back to a fuzzy sid match or splicing at index 0.
- `revertDiffBlocks` (plural) and `BuildSidBlocksOptions` (and its `allowEmptySid` field) are
  **deleted** — no replacement, no option. Partition is now unconditionally gapless.
- New `normalizeTokenSids(tokens, bookCode) -> Token[]` exported as a standalone utility — the
  canonical sid-derivation pass, pure, preserves every non-`sid` field, `bookCode` authoritative
  over any embedded `\id`.
- `DecisionUnit` in the new model always identifies a unit by `id: string` (never depends on an
  app token id); `Slot.role` is one of `shared | baselineOnly | currentOnly | pairBaseline |
  pairCurrent`; a moved/coalesced pair occupies exactly two slots bound to one `id`.
- **Behavior change, not just a shape change:** `ParsedUsfm.diffByChapter`'s by-chapter grouping
  algorithm changed (now groups tokens by chapter before diffing, matching the native by-chapter
  path, instead of diffing the whole document then regrouping) — a verse move that crosses a
  chapter boundary no longer coalesces in the by-chapter result. Full detail and affected-test
  guidance under "`diffScope`, save/external-compare staging, mocks, and tests" below.

## Required changes, by surface

### `IUsfmOnionService` (interface)

- Replace every method returning `ChapterTokenDiff[]` / `DiffsByChapterMap` with the
  `DiffSkeleton` / by-chapter-skeleton-map shapes above.
- Add a `mergeDiffBlocks` method mirroring the wasm export.
- `revertDiffBlock`'s return type stays `Token[]`, but its contract becomes throwing (or
  Promise-rejecting, depending on how the interface wraps sync wasm calls) on an unknown id —
  callers must handle that instead of silently getting a no-op array back.
- Remove `revertDiffBlocks` (plural) and any `BuildSidBlocksOptions`-shaped parameter from every
  method signature in the interface.

### Onion generated-type imports / `Diff` domain mapping

- Delete or replace every hand-written TS type that mirrors the old `ChapterTokenDiff` /
  `SidBlockDiff` / `DiffStatus` / `DiffTokenChange` / `DiffUndoSide` / `TokenAlignment` shapes —
  regenerate from the new `.d.ts` (`pkg-bundler/usfm_onion_web.d.ts` / `pkg-web/...`) rather than
  hand-porting field-by-field.
- Any app-side `Diff` domain model built on top of the old flat shape needs its own mapping layer
  rewritten against `DiffSkeleton`'s slots/units instead of a flat list with `status`/`blockId`.

### `WebUsfmOnionService` / `TauriUsfmOnionService`

- Update both adapters' diff/merge/revert method bodies to call the new wasm exports and return
  the new shapes — this is a mechanical follow of the interface change above, but each adapter
  currently has its own marshaling code for the old flat shape that needs deleting, not patching.
- `TauriUsfmOnionService` additionally talks to the Tauri Rust side (next section) — its TS-side
  method bodies must match whatever the Rust commands now return.

### Tauri Rust DTO/status mapping and commands

- `src/tauri/rust/src/usfm_onion.rs` (per the plan's own adjudication notes,
  `plans/progress-merge-projection.md`'s 2026-07-16 adjudication entry): `BuildSidBlocksOptionsDto`
  must be deleted; every command taking it needs the parameter removed.
- `map_diff_status` (or equivalent exhaustive match over the old `DiffStatus`) is a **compile
  break**: the new `DecisionStatus` adds `Moved` and drops nothing, but the match arms must be
  rewritten against the new enum regardless, since the whole return type changed shape.
- `apply_revert_by_block_id`-equivalent command(s) need their signature/behavior updated to match
  the new throwing-on-unknown-id contract; any plural revert command should be deleted.
- Re-check every exhaustive `match` over the diff/merge domain types in this file — the compiler
  will find them once the DTOs update, but do not assume the list above is exhaustive.

### `diffScope`, save/external-compare staging, mocks, and tests

- Anywhere `diffScope` stages a comparison and later calls revert/merge per-block, the staged
  block ids now come from `DecisionUnit.id` (skeleton units), not the old flat diff's `blockId`.
  Since unit ids never depend on app token ids (by design — id drift is a supported input), any
  staging logic that assumed a stable relationship between a token's own id and its diff block id
  must be re-verified.
- All mocks that fabricate `ChapterTokenDiff`-shaped fixtures need rewriting to
  `DiffSkeleton`-shaped fixtures. Do not patch field names in place — the shape is genuinely
  different (flat array vs. slots+units).
- Any test asserting the old plural-revert or `allowEmptySid` behavior should be deleted, not
  adapted — those code paths no longer exist anywhere in onion.
- **Behavior change in by-chapter diffing on the `ParsedUsfm`/token-diff path specifically:** the
  old `ParsedUsfmDiffByChapterBuilder` (the FormatToken/carried-sid convention `IUsfmOnionService`
  adapters likely call through `ParsedUsfm.diffByChapter`) diffed the *whole* multi-chapter
  document first and only afterward regrouped the flat result by parsing each unit's own sid
  string into a book/chapter bucket. The new `diff_skeleton_by_chapter_from_tokens` groups tokens
  by book/chapter *first*, then diffs each chapter independently — the same algorithm the native
  by-chapter path (`diffUsfmByChapter`) already used, so this converges the two by-chapter paths
  onto one algorithm rather than porting the old inconsistency forward. **Practical effect:** a
  verse that moves *across* a chapter boundary (e.g. content migrating from the end of one
  chapter's last verse into the start of the next chapter, or a genuine cross-chapter reorder) no
  longer surfaces as a single coalesced `moved`/`Modified` unit in the by-chapter map — each
  chapter now sees it as an unpaired `Deleted` in one bucket and an unpaired `Added` in the other,
  since the two chapters are diffed as independent documents. This is a narrow, rare edge case
  (verses essentially never move between chapters in real translation work), but if any existing
  Zephyr test fixture exercises a cross-chapter move specifically through the by-chapter diff
  path, that test's expectation needs updating, not just its shape. The single-skeleton path
  (`ParsedUsfm.diff` / `diffUsfm`, not by-chapter) is unaffected — it still diffs the whole
  document as one skeleton and would still coalesce a genuine cross-chapter move.

### `defaultBuildSidBlocksOptions` removal

- Delete this constant/default entirely along with every call site that passes it. There is no
  replacement option — partition is now unconditionally gapless in onion; if some caller actually
  relied on `allowEmptySid: false` to *drop* empty-sid tokens, that caller's behavior changes:
  empty-sid tokens are now always included in the diff (in their own partitioned block) rather
  than silently excluded. Audit call sites for this before assuming it's a no-op removal.

### The six `normalizeTokenSids` / `mutAddSids` call sites

Per the plan's interim design: onion's native diff paths and the new `normalizeTokenSids` wasm
export both call the same canonical sid-derivation pass now; Zephyr's own `mutAddSids` remains the
sid source for these six call sites until the dual-run parity check below gates the swap.
**Do not migrate these to `normalizeTokenSids` yet** — this list is the target set for a *later*
consumer-side change, not something to do as part of this handoff:

- `tokenReplace`
- `scriptureProjectToParsedFiles`
- `usfmTokenStreamSerializedAdapter`
- `maintainMetadata` (includes the Lexical listener at `maintainMetadata.ts:148` mentioned in the
  plan's design notes)
- `rebuildParsedFileFromUsfm`
- `parseRecoveredBookContents`

**Where each half of the parity check lives, and why it's split this way:** `mutAddSids` is
Zephyr application code — it does not exist in the onion repo, and this plan's own scope line
("usfm_onion repo only, no Zephyr/Sefer changes") rules out vendoring or importing it here to run
a literal side-by-side comparison in onion's test suite. So the check is split across both repos:

- **Onion's half (done, shipped):** `crates/usfm_onion_wasm/src/lib.rs`'s
  `normalize_token_sids_bridge_dup_intro_parity_contract` test pins the exact expected sid strings
  `normalizeTokenSids(tokens, bookCode)` produces for bridge (`GEN 1:1-2`), duplicate
  (`GEN 1:1_dup_1`), and intro (`GEN 0:0`) cases. This is onion's committed, tested answer key —
  not a promise, an executable pinned test that fails loudly if onion's derivation ever drifts.
- **Zephyr's half (not started, this repo cannot do it):** before migrating any of the six call
  sites, add a Zephyr-side test that runs the *same* representative bridge/duplicate/intro token
  fixtures through both `mutAddSids(tokens, bookCode)` (Zephyr's current implementation) and the
  wasm-exported `normalizeTokenSids(tokens, bookCode)` (import the package this handoff regenerated),
  and assert the `sid` values agree token-for-token. This is the actual dual-run — it can only
  execute where both implementations are importable, which is Zephyr's repo, not onion's.

Only after that Zephyr-side dual-run check is green should each call site be migrated one at a
time (not all six at once), followed eventually by retiring `mutAddSids` — and only after that,
flipping onion's external `diffTokens`/`mergeDiffBlocks` calling convention from "trust the carried
sid" to "always derive" (deleting the carried-sid interim branch in onion — a follow-up onion
change, not a Zephyr-side one).

## Explicitly NOT part of this handoff (later work, tracked in the plan)

- The fresh Sefer review UI (moves as a collocated box with order-chip wording; the prototype's
  `projectRows`/`projectHunks` are reference semantics only, not code to port).
- The review-modal open-sequence normalization discipline (normalize both streams while taking the
  frozen snapshot) — belongs to the Zephyr consumer plan, not onion.
- Sefer-side snapshot-staleness UX (freeze/lock typing while a review modal is open, abort on
  buffer change) — onion's `UnknownUnitId`/throw-on-unknown-id only covers "this id doesn't exist
  in this skeleton," not "this skeleton is stale relative to the live buffer."

## Seed for a follow-up plan: `diffTokens` always deriving sids (2026-07-17)

Not decided, not scoped, not started — this is the raw material for a future coordinated plan
(onion + scripture-editor-proto-2), captured here so it isn't lost. Traced against
`scripture-editor-proto-2` as of this date:

**Confirmed: `diffTokens`/`revertDiffBlock` are the only real diff/merge call sites in that repo.**
`diffUsfm`, `diffUsfmByChapter`, `mergeDiffBlocks`, and onion's own `normalizeTokenSids` wasm export
have zero callers there, prototype or production. Real call sites, all passing already-tokenized
arrays (never raw source):
- `src/web/domain/usfm/WebUsfmOnionService.ts:379-381` (`diffTokens`), `:395-400`
  (`revertDiffBlock`) — the wasm-side service.
- `src/web/domain/usfm/TauriUsfmOnionService.ts:463` (`diffTokens` mirror), `:478`
  (`revertDiffBlock` mirror) — IPC to the native side.
- `src/app/ui/hooks/save/useDiffModalState.ts:127` — computes the unsaved-changes diff shown in the
  save/review modal, **per dirty chapter** (not whole-book).
- `src/app/domain/project/saveAndRevertService.ts:45` (`revertChapterDiffByBlockId`),
  `src/app/domain/project/compare/compareMutations.ts:116` (`applyIncomingHunk`).
- `versionSnapshotAdapter.ts:103-104` — pass-through wrapper over `diffTokens`.
- Native mirrors in `src/tauri/rust/src/usfm_onion.rs:870-906` (the `.diff().with_options().run()`
  builder backing the token-array diff command) and `:924` (`apply_revert_by_block_id`).
- The prototype at `agent-tmp/prototypes/merge-interleave/merge-engine.js` does **not** import
  onion at all — it's a from-scratch reference reimplementation, not evidence of a real call site.

**The gap:** `diffTokens`'s carried-sid trust isn't trusting onion's own derivation twice — it's
trusting a second, independently-maintained sid computation. The app has its own local
`normalizeTokenSids`/`mutAddSids` (`src/core/domain/usfm/tokenSidNormalization.ts:34`, same name as
onion's export, unrelated code), called from `tokenReplace.ts:258`,
`parseRecoveredBookContents.ts:52`, `scriptureProjectToParsedFiles.ts:167`,
`usfmTokenStreamSerializedAdapter.ts:298`, `rebuildParsedFileFromUsfm.ts:41` — none of which touch
onion. That's the sid `diffTokens` currently trusts, with no cross-check against onion's own
`derive_canonical_sids` anywhere. Given `diffTokens`/`revertDiffBlock` drive real save/revert/merge
paths (data going to and from disk), this is worth closing even at the cost of re-deriving sids on
every call — the duplicate work is cheap relative to the correctness risk.

**Concrete blocker, not just a policy flip:** `wasm_diff_tokens(left: Vec<Token>, right: Vec<Token>)
-> DiffSkeleton` has no `book_code` parameter, and `derive_canonical_sids` requires one. The native
canonical path gets it for free by parsing raw source (the `\id` marker). `diffTokens` never sees
raw source, and — critically — `useDiffModalState.ts:127` diffs **per dirty chapter**, so a
chapter-2+ token slice won't carry a `\id`/`BookCode` token at all (that only appears once, at the
top of chapter 1). Making `diffTokens` always derive means adding a `book_code` parameter to the
signature (both wasm and native), not just an internal onion flip — every call site above needs
updating to pass it.

**Design intent floated, not decided:**
- Shape the new signature so the caller supplies `book_code` explicitly —
  `diffTokens(left, right, options)` with `options.bookCode` — rather than onion trying to infer it
  from a possibly-bookCode-less chapter slice. The editor already knows the book code for any given
  chapter (can pluck it from a sid it already has), so this is a reasonable constraint to impose on
  the consumer. Chapter-level granularity must be preserved either way — no forcing a whole-book
  diff over postMessage/Tauri-IPC serialization just because one chapter is dirty.
- Whether the sid-derivation pass should also ship as plain JS utilities, not only via the wasm
  bundle — desktop (Tauri/webview) doesn't import the wasm package at all today, so parity between
  web and desktop for this specific behavior might mean exporting equivalent JS, not assuming wasm
  is always in reach.

**Next step:** flesh this into an actual scoped plan (line numbers re-verified at that time, not
assumed stable) before touching any code — this needs its own review pass, the same way the
merge-projection plan itself went through crew review, not a quick patch bundled into unrelated
work.

## Verification for whoever picks this up

- Re-run the prototype harness (`node merge-engine-check.js` in
  `../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave`) if in doubt about expected
  narration — it is still the reference semantics and was 23/23 cases + 21 pins green as of this
  handoff.
- Inspect `pkg-bundler/usfm_onion_web.d.ts` and `pkg-web/usfm_onion_web.d.ts` directly for the
  exact generated shapes — they are the wire contract, not this document's prose.
