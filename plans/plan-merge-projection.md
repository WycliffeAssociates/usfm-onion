# Plan: merge-as-projection (interleave skeleton + `merge_diff_blocks`)

Status: design settled by prototype; adversarial plan review incorporated; implementation not started.
Repo scope: **usfm_onion only.** Every step and gate in this plan lands in this repository.
Zephyr/Sefer consumer work is out of scope end-to-end — this plan's only consumer deliverable is
the step-7 handoff inventory, collected in "TBD in Zephyr/Sefer" at the bottom.
Dials: interview = done (2026-07-16, prototype-driven); testing = **hardened**.

Reference semantics, in priority order:

1. `../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave/merge-engine-check.js`
   and `cases.js` (23 cases, P1/P2/P4, 21 narration pins).
2. `../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave/merge-engine.js`.
3. `../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave/NOTES.md`.
4. `../scripture-editor-proto-2/agent-tmp/prototypes/diff-review-redesign/merge-projection-properties.md`
   (its P6 same-number replace wording is superseded below).

Before changing Rust, run:

```sh
cd ../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave
node merge-engine-check.js
```

Stop if any of the 23 cases or 21 pins fails. Do not “correct” the Rust port away from a pin.

## Problem

Zephyr currently applies staged block decisions by calling `revert_diff_block` repeatedly. Each
call mutates the current stream before the next block id is resolved. A frozen diff id can then
miss, and the existing fallbacks (`infer_sid_match_block` and `find_insertion_index` returning
index 0) turn that miss into silent corruption: the wrong duplicate occurrence can be restored,
or content can be spliced at the start of a chapter.

Zephyr is moving to staged reconcile: freeze the comparison, stage all per-unit choices, and use
one Apply to materialize the result. Onion must provide a pure projection over one frozen diff
model. It must never replay edits sequentially and must never guess when an id is unknown.

## Solution

The canonical model is a gapless Myers interleave of baseline and current SID blocks. A shared or
one-sided unit occupies one slot. A coalesced pair occupies two slots—its baseline position and
its current position—bound to one decision id. Merge walks those slots once and emits the chosen
side. Therefore all-Baseline reproduces baseline byte-for-byte and all-Current reproduces current
byte-for-byte, including moves and mixed line endings.

The native core builds and merges a `DiffSkeleton<T>`. The wasm entry point may rebuild that
skeleton from the same token inputs, but it must call the same native builder and merger; there is
only one alignment/pairing/merge implementation.

## Scope and assumptions

- The prototype's engine model and pins are the behavior spec. Its UI modes and copy are not.
- Zephyr owns preview-equals-Apply, working-buffer snapshot validation, and semantic warnings on
  mixed decisions. Onion's P7 detects an **unknown decision id**; it does not prove that two
  different token streams with coincidentally identical ids are the same app snapshot.
- The core accepts any flat token slice, including multiple chapters or fully independent streams.
  Zephyr v1 calls it once per existing `diffScope` item (normally one chapter). Do not add a
  batched multi-chapter merge API in this plan.
- Native parsed tokens derive canonical SID strings in onion. App-supplied `FormatToken.sid`
  strings are trusted as already normalized during this release. Missing/empty external SIDs are
  still included in explicit empty-SID blocks so partition remains gapless; they are not guessed.
- **Same/same direction (Will, 2026-07-16):** the canonical pass must be written against the
  `DiffableToken` structural surface (kind/marker/number payload/text), never `Token`-only
  internals, so one implementation covers both token shapes. Zephyr today re-stamps sids at six
  lifecycle points (incl. a Lexical listener, `maintainMetadata.ts:148`) — the end state is those
  call sites invoking onion's exported pass, then diff/merge deriving internally and ignoring
  carried sids entirely. This release ships the shared implementation + wasm export + parity
  fixture; the always-derive flip is consumer-migration work.
- **Zephyr interim discipline (Will, 2026-07-16):** the review modal's open sequence normalizes
  BOTH streams (workingFilesStore buffer + baseline) as part of taking the frozen snapshot, then
  diffs and later merges those exact normalized streams from modal state. This makes sid
  correctness a boundary property of the modal rather than a promise every Lexical mutation path
  must keep, and it guarantees merge sees the same inputs the diff saw. Scaffolding: it dissolves
  into the engine when always-derive lands. Belongs in the Zephyr consumer plan; recorded here so
  the handoff carries it.
- Duplicate ordinals are occurrence context, not part of the compact native `Sid` value. They are
  added by a stateful canonical-SID pass used by diff/token DTO projection.
- Footnotes remain inside verse-grain units. JSON patch transfer and cross-verse-number content
  pairing remain out of scope.

## Settled design decisions

1. **SID semantics match Zephyr.** SID assignment is sticky per token. `Sid` keeps `verse` as the
   range start and gains `verse_end`; a single verse stores `verse_end == verse`, while chapter and
   intro SIDs store `verse == verse_end == 0`. Canonical text includes a range end when it differs
   (`GEN 1:1-2`), uses `BOOK 0:0` for intro material, and adds `_dup_N` with a per-chapter,
   per-full-range positional counter. The counter key includes the range end, so `1` and `1-2`
   are not duplicates of one another. Cases 12–17 and 23.
2. **Partition is strict and gapless.** Every input token belongs to exactly one contiguous
   same-canonical-SID block. Delete `BuildSidBlocksOptions` and `allow_empty_sid`; the option's
   false branch dropped tokens and no current Zephyr call uses it.
3. **Alignment is Myers over canonical block ids, followed by two-tier pairing.** Pairing key is
   book + chapter + range start; it strips range end, `_dup_N`, and occurrence suffix, but never
   crosses a verse number. Within a key: pair byte-identical block texts first (stream order among
   ties), then pair remaining blocks positionally. A tier-2 pair is byte-different by construction.
   Case 13 requires exact-text-first; case 23 forbids cross-key content pairing.
4. **Statuses follow the prototype truth table.** Shared byte-equal = `Unchanged`; shared
   byte-different = `Modified`; unpaired sides = `Added`/`Deleted`; exact coalesced pair in the
   same relational position = `Unchanged`; exact displaced pair = `Moved`; byte-different
   coalesced pair = `Modified`, with `displaced == true` when it also moved. Same-number content
   replacement is `Modified`, not delete + add.
5. **The skeleton is canonical.** Slots are in interleave order. A coalesced pair has two slots
   and one decision. Every slot carries its side-appropriate nearest preceding shared-or-paired
   anchor. Deleted/added neighbors never become anchor targets and never by themselves make a pair
   displaced. Both side SIDs are retained on pairs.
6. **Merge is pure projection.** Unknown ids are validated before any output is assembled. Shared
   units emit the chosen side; Added emits only for Current; Deleted emits only for Baseline;
   pair-baseline emits only for Baseline; pair-current emits only for Current. Output tokens are
   clones/slices of the inputs—never trimmed, normalized, or reserialized during merge.
7. **Revert is one-decision merge.** `revert_diff_block(X)` is `{X: Baseline}` with default
   Current. Delete `infer_sid_match_block`, the insertion-at-zero fallback, and the sequential
   plural revert API. An unknown id is an error; a caller must abort and re-diff.
8. **Presentation stays downstream.** Onion returns enough model data to project rows/hunks, but
   does not ship `projectRows`/`projectHunks`. Rust tests translate every engine pin to direct
   skeleton/unit assertions so app-side projection is still fully specified.

## Exact domain contracts

### Canonical SID pass

There is exactly ONE derivation algorithm; interim behavior differs only in WHO calls it.

- `derive_canonical_sids(tokens, book_code) -> Vec<String>`: a single stateful structural pass —
  sticky sid assignment from marker/number structure (`\c`, `\v`, range payloads), range-aware
  base SID (`GEN 1:1-2`), `BOOK 0:0` intro, per-chapter full-range duplicate counters
  (`_dup_1`, `_dup_2`, …; `1` and `1-2` never share a counter). It never trusts carried sids.
- **Interim dispatch (explicit):**
  1. Native diff paths (`diffUsfm`, parsed methods) call `derive_canonical_sids` now.
  2. Wasm `normalizeTokenSids` calls the same function now (contract below).
  3. External token diff (`diffTokens` / `mergeDiffBlocks` on app tokens) temporarily uses the
     carried `FormatToken.sid` strings UNCHANGED — verbatim, no re-suffixing; empty stays empty
     and remains in the partition.
  4. The Zephyr/Sefer consumer migration later switches path 3 to derivation, deleting the
     carried-sid branch. Until then, paths 1+2 vs 3 are two CALLING conventions, not two
     algorithms — there is no second derivation implementation to drift.
- Apply the resulting canonical string to every token in its sticky run for partition/DTO use.
- `Sid` deliberately does not store `_dup_N`; this keeps occurrence identity out of scripture
  reference value semantics.
- Layout (spiked 2026-07-16, `rustc -O`): today `Sid` = 8 bytes, `Option<Sid>` = 10. With
  `verse_end: u16` → 10 / `Option` 12; with `verse_end_delta: u8` → **8 / 10, guard unchanged**.
  **Decision (Will, 2026-07-16, overruling the reviewer's lossiness objection on domain
  grounds): use `verse_end_delta: u8`.** No real versification can approach the limit — the
  longest chapter in the Bible has 176 verses, and a bridge cannot span more verses than its
  chapter contains — so a >255 span is impossible-in-domain, not merely rare. Memory wins:
  `Option<Sid>` rides on every token. The u8 constraint is SPECIFIED, not silent: expose
  `verse_end()` (= verse + delta, saturating) as the only public accessor, serialize the
  resolved end (never the raw delta), document the ceiling on the field, and PIN the saturation
  behavior with an explicit test (`\v 1-999` fixture) so it is a stated contract rather than a
  surprise. The 8-byte guard stays byte-for-byte as it is today.
- Route every range-aware native formatter through one helper. Inventory includes
  `DiffableToken::sid_string`, `api::format_sid`, wasm token mapping, HTML SID attributes, USJ SID
  projection, lint SID formatting, and vref helpers. Duplicate suffixes are required on diff/token
  DTO projection; other semantic exporters need range fidelity but need not expose occurrence
  suffixes unless they already promise diff-address identity.

### Block and unit ids

IDs must not depend on app token ids; id drift is a supported input.

- Block id = canonical SID for its first contiguous occurrence, then `#1`, `#2`, ... for a later
  non-contiguous reuse of the exact same canonical SID. (Duplicate verses normally already differ
  by `_dup_N`; the occurrence suffix is a collision safety net.)
- Shared/deleted unit id comes from the baseline block id. Added/coalesced unit id comes from the
  current block id, matching the prototype's current-major decision key.
- If that desired unit id already exists, append `@1`, `@2`, ... in deterministic unit-creation
  order. Unit creation and `units` output order must be deterministic.
- `UnitId` should be a Rust newtype internally even if serialized as a string. Slot references and
  decision-map keys use it; never use display SID as an implicit foreign key.

### Pairing, displacement, and statuses

- Run Myers on block ids and emit the baseline-only run, then current-only run, then shared slot
  around each Myers match, as the current interleave does.
- Pair only off-Myers blocks within the same pairing key.
- Build both pair slots before classifying displacement.
- A pair is displaced iff its current slot is before its baseline slot **or** at least one shared
  slot lies strictly between its slots. One-sided Added/Deleted slots between the pair do not count.
- `relabeled` is true only for a byte-equal coalesced pair whose canonical SIDs differ.
- `dup_context` reports key-group counts on both sides and is true if either count exceeds one.
- `is_whitespace_change` / `is_usfm_structure_change` retain current onion behavior and are false
  when one side is absent. USFM-structure-only is mutually exclusive with whitespace-only.
- `covered_by` is only for a one-sided verse overlapped by a true multi-verse range on the opposite
  side of a coalesced pair, in the same book/chapter. If a new fixture produces multiple possible
  coverers, stop and add a prototype pin before choosing a tie-break; do not improvise.
- Preserve the existing `similar::Algorithm::Myers` behavior. Do not port the prototype's O(n*m)
  DP literally. Symmetric reorder tests assert the one-mover and column-order invariants, not which
  symmetric verse Myers names as the mover. Do not implement the deferred canonical-order
  tie-break in this plan.

### Slots and anchors

- Slot roles: `Shared`, `BaselineOnly`, `CurrentOnly`, `PairBaseline`, `PairCurrent`.
- Every slot stores `unit_id` and `after: Option<Anchor>`, where `Anchor` contains both the
  preceding unit id and that predecessor's SID at this slot's side/position.
- Update `after` only after Shared or Pair slots. Added/Deleted slots do not become anchors.
- For a pair slot, use that slot's side SID (`baseline_sid` at PairBaseline, `current_sid` at
  PairCurrent).
- A downstream one-row projection emits a unit at its first skeleton slot. This rule plus the
  per-slot anchors must reproduce case 13 and 15 narration order.
- Reading nonempty baseline-side cells in slot order must equal baseline block order; same for
  current-side cells and current block order. Case 10 pins both.

### Merge and errors

- Validate the entire decisions map before walking slots. `UnknownUnitId(id)` returns no output.
- Missing map entries use the explicit `default_side`; do not give the native API an implicit
  default.
- A coalesced pair must contribute exactly once for either choice. Shared contributes exactly once;
  Added/Deleted contributes zero or once according to presence on the chosen side.
- Native `merge_skeleton` consumes the already-built model. `merge_diff_blocks` is a convenience
  wrapper that builds the skeleton once and delegates. `revert_diff_block` delegates to that
  wrapper.
- The wasm error surface is a thrown JS `Error` (Rust `Result<_, JsValue>`), with an error message
  containing the unknown id. The web-package smoke test must assert the throw.

## API contract (names may move; shapes and semantics may not)

Use a status enum dedicated to decision units so the implementer does not accidentally force the
legacy flat diff DTO through half-migrated exhaustive matches.

```rust
pub struct DiffSkeleton<T> {
    pub slots: Vec<Slot>,
    pub units: Vec<DecisionUnit<T>>,
}

pub struct Slot {
    pub unit_id: UnitId,
    pub role: SlotRole,
    pub after: Option<Anchor>,
}

pub struct Anchor {
    pub unit_id: UnitId,
    pub sid: String,
}

pub struct DecisionUnit<T> {
    pub id: UnitId,
    pub kind: DecisionUnitKind, // Shared | Added | Deleted | Coalesced
    pub status: DecisionStatus, // Unchanged | Modified | Added | Deleted | Moved
    pub baseline_sid: Option<String>,
    pub current_sid: Option<String>,
    pub baseline_tokens: Vec<T>,
    pub current_tokens: Vec<T>,
    pub displaced: bool,
    pub relabeled: bool,
    pub dup_context: DupContext,
    pub covered_by: Option<CoveredBy>,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
}

pub fn diff_skeleton<T: DiffableToken>(
    baseline: &[T],
    current: &[T],
) -> DiffSkeleton<T>;

pub fn merge_skeleton<T: Clone>(
    skeleton: &DiffSkeleton<T>,
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) -> Result<Vec<T>, MergeError>;

pub fn merge_diff_blocks<T: DiffableToken>(
    baseline: &[T],
    current: &[T],
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) -> Result<Vec<T>, MergeError>;
```

Wasm wire contract:

- `diffTokens(baseline, current) -> DiffSkeleton<Token>` (breaking replacement of the old flat
  `Diff[]` result).
- `mergeDiffBlocks(baseline, current, { decisions, defaultSide }) -> Token[]`, where decisions is
  `Record<string, "baseline" | "current">`.
- `revertDiffBlock(...) -> Token[]` remains but now throws on unknown id.
- Remove `revertDiffBlocks` and `BuildSidBlocksOptions` from Rust, wasm, generated declarations,
  package smoke tests, README, and playground examples.
- `diffUsfm`, `ParsedUsfm.diff`, and by-chapter variants must return the same canonical skeleton
  model (or a book/chapter map of skeletons); do not leave a second flat-diff algorithm alive.

This intentionally breaks Zephyr's current `IUsfmOnionService`, web adapter, Tauri adapter and Rust
commands. Updating that sibling repo is a separate consumer plan, not work to hide inside this
onion implementation. Step 7 must leave an exact handoff inventory.

## User stories / acceptance behavior

1. A translator stages Baseline/Current choices against one frozen skeleton and applies them once.
2. A move is one decision with two positions; it cannot half-apply, duplicate, or disappear.
3. A comparison with id drift or no shared ancestry still aligns by canonical scripture address.
4. An unknown staged unit id fails before output is produced; the caller aborts and re-diffs.
5. All-Baseline and all-Current reproduce their input bytes exactly, including CRLF/LF.
6. Zephyr can derive its row/hunk views, both-side labels, duplicate warnings, bridge coverage, and
   WS/USFM filters from the returned model without onion owning presentation.

## Seven implementation steps (execute in order)

Each step starts by adding/updating the smallest failing tests for that step, then implementing,
then running its gate. Append surprises, deviations, and verification results to
`plans/progress-merge-projection.md`. Do not begin the next step while the current gate is red.

### 1. Range-aware SID value and one canonical SID pass

Files/surfaces:

- `src/token.rs`: `Sid.verse_end_delta: u8` + `verse_end()` accessor, constructors/invariants,
  Display/Debug, size guard untouched.
- `src/parse/mod.rs`: pass `NumberRangeToken.end` through sticky SID assignment.
- Central range-aware SID formatter; update every native formatter found in the inventory above.
- Stateful canonical SID pass for native diff/token DTO use; per-chapter full-range dup counters.
- **`DiffableToken` trait additions** (the current trait exposes no number or book data —
  `src/diff/mod.rs:9`): add `number_range(&self) -> Option<(u32, Option<u32>)>` and
  `book_code(&self) -> Option<&str>`, both defaulting to `None`. Implement for native `Token`
  (from `TokenData::Number` / `TokenData::BookCode`) and for `FormatToken` (from its number
  metadata). This is what lets `derive_canonical_sids` run identically over both token shapes.
- **`normalizeTokenSids` wasm contract** (exported in Step 7, defined here):
  `normalizeTokenSids(tokens: Token[], bookCode: string): Token[]` — pure (input untouched);
  preserves length, order, `text`/`source`, `id`, and every non-SID field exactly; replaces
  EVERY token's `sid` with the derived canonical string (carried sids are never consulted).
  `bookCode` is authoritative for all derived SIDs; an embedded `\id` book code is not consulted
  and a mismatch is not an error — parity with Zephyr's `mutAddSids(tokens, bookCode)`, which
  behaves identically. Callers who want id/bookCode agreement lint it separately.

Gate:

- Unit tests prove single, bridge, duplicate, chapter, and intro strings.
- A bridge marker and all following tokens keep the full range SID until the next chapter/verse.
- `1` and `1-2` do not share a duplicate counter; repeated `1-2` gets `_dup_1`.
- `size_of::<Sid>() == 8` — the existing guard passes unmodified. The saturation pin test
  (`\v 1-999` → `verse_end() == verse + 255`, documented) is green.
- Existing parse, vref, HTML, USJ, lint, and API SID tests pass with only intentional range changes.

Stop condition: if the delta representation fails to keep `Sid` at 8 bytes, or the accessor
indirection leaks the raw delta into any serialized/DTO surface, stop and report rather than
weakening the guard or shipping a leaky repr. (Fallback, requiring Will's sign-off: `verse_end:
u16` with a reviewed 10-byte ceiling.)

### 2. Gapless partition, deterministic block ids, and option removal

Files/surfaces:

- `src/diff/mod.rs`: replace `next_block_start`/`token_included` option behavior with one linear,
  gapless partition over the canonical SID vector.
- Remove `BuildSidBlocksOptions` and option arguments from native builders/APIs.
- Remove wasm option DTO and root exports now so subsequent steps compile against the final shape.
- Update `src/api.rs`, `src/lib.rs`, playground, README, wasm bindings, and directly affected tests.

Gate:

- Concatenating every block's token slice reproduces the exact input for native and FormatToken
  paths, including leading empty-SID tokens.
- Block ids ignore token ids and use the canonical SID/occurrence rules above.
- An id-drift fixture yields identical block/unit-address candidates on both sides.
- `rg "allow_empty_sid|allowEmptySid|BuildSidBlocksOptions" src crates README.md` has no
  product-code/documentation hits (generated packages are regenerated and checked in Step 7;
  historical plan text may remain).
- `cargo test --workspace` passes. `tests/synthetic_fixtures.rs` and
  `tests/bare_tokens_lint.rs` need no semantic rewrite; if they fail, treat that as unintended SID
  fallout, not permission to change their expectations.

### 3. Myers interleave, two-tier pairing, units, and the two-slot skeleton

Implement as one dependency slice: displacement cannot be classified before pair slots exist.

- Reuse the current Myers sequence diff; do not introduce another LCS implementation.
- Produce interleave steps and the complete block order for both sides.
- Build per-key off-Myers groups, exact-text pair first, then positional leftovers.
- Create deterministic unit ids and one/two slot roles. Store both side SIDs and token slices.
- Do not finalize `Moved`/`Unchanged` for exact pairs until Step 4 scans slot positions.

Gate:

- Structural tests prove every baseline block appears in exactly one baseline-bearing slot and
  every current block in exactly one current-bearing slot.
- Every coalesced unit owns exactly one PairBaseline and one PairCurrent slot with the same id.
- Case 13 pairs `c↔c` before positional leftovers; case 23 has no coalesced pair.
- Case 10/11 prove the one-mover invariant without pinning which symmetric item Myers chooses.
- Running `diff_skeleton` twice yields identical block ids, units, and slot order.

### 4. Classification, annotations, anchors, and the complete fixture oracle

- Finalize `displaced` from pair slot positions, then apply the status truth table.
- Add `relabeled`, dup counts/context, `covered_by`, and existing WS/USFM flags.
- Compute side-appropriate `after` anchors for every slot.
- Port all 23 cases and all 21 narration pins. The Rust fixture adapter must provide a book to the
  real parser: prototype cases without `\\id` are wrapped with a shared `\\id GEN\n` prefix before
  `parse()`, while case 19 keeps its existing id line. Expected P2 bytes include that prefix.
- Run the same catalog through app-shaped `FormatToken`s with normalized SIDs and deliberately
  different token ids.

Pin translation (do not omit because presentation is app-side):

- case 10: baseline/current slot subsequences equal source orders; moved unit has two linked slots.
- case 13: deleted `a`; surviving `c` exact-pairs as Unchanged+relabeled; dup counts 2×1; deletion
  anchors after chapter-open unit; no first-slot anchor collision.
- case 15: pair exposes both SIDs; deleted verse 2 anchors after the pair; pair precedes deletion;
  deleted verse 2 is covered by current `1-2`.
- case 16: added verse 3 is covered by baseline `1-3`.
- case 23: one Deleted 11, one Added duplicate 1, no pair.
- cases 7/8: the exact WS/USFM flag truth table from the JS pins.

Gate: the ported fixture suite is green on native parsed tokens and FormatToken/id-drift tokens.

### 5. Pure merge, strict single revert, and corruption-path deletion

- Implement `merge_skeleton`, then the raw-input `merge_diff_blocks` wrapper.
- Validate all decision ids first; implement `UnknownUnitId` as a typed native error.
- Rewrite single revert as one Baseline decision with default Current.
- Delete `infer_sid_match_block`, `find_insertion_index`, `apply_reverts_by_block_id`, all plural
  native/wasm/API methods, and their exports.

Gate:

- All 23 fixtures pass all-Baseline and all-Current byte identity.
- Random decision vectors over each fixture prove per-unit contribution cardinality (shared/pair
  exactly once; one-sided zero-or-once).
- CRLF baseline and LF current remain byte-exact at identity anchors.
- Unknown id returns/throws before any output; single revert of every changed unit equals
  `merge({id: Baseline}, Current)`.
- Run `rg "infer_sid_match_block|find_insertion_index|apply_reverts_by_block_id|revertDiffBlocks"`
  over `src`, `crates`, `README.md`, and `scripts`; it has no product-code/documentation hits.
  Generated packages are checked after regeneration in Step 7.

### 6. Hardened proptest suite (P1–P7, amended)

Add `proptest` as a dev dependency. Generate a structured chapter/document model first and render
valid USFM; do not mutate arbitrary byte offsets and discard cases with broad `prop_filter`s.

General strategies:

- Always include a valid `\\id` so native parse has a book; generate bounded chapters, verse
  singles/ranges, text atoms, balanced inline marker wraps, paragraph/heading changes, whitespace,
  and LF/CRLF separators.
- Derive Current from Baseline with typed edit scripts: text edit, insert/delete, reorder, boundary
  text migration, paragraph/heading change, whitespace-only, balanced marker wrap.
- Also generate fully independent baseline/current models and mixed EOL pairs.
- Generate decision vectors from the skeleton's returned unit ids, including partial maps and both
  defaults. Shrinking operates on the structured model/edit list so the defining shape survives.

Directed strategies are separate properties, not low-probability variants in one union:

1. adjacent deletion + two-block boundary migration;
2. boundary migration with each mixed Baseline/Current choice;
3. swap/reorder and three-item transposition;
4. duplicate SIDs on one/both sides and delete-first-occurrence;
5. bridge fuse-adjacency, merge-into-bridge, split, and overlap;
6. renumber typo (must remain Added+Deleted, never coalesced);
7. insertion at chapter start/end;
8. different-number back-to-back Added+Deleted (same-number replace is Modified);
9. fully independent streams and CRLF×LF.

Properties:

- **P1 partition:** side-specific block/slot reconstruction is exact and no block contributes twice.
- **P2 identities:** all-Baseline == B and all-Current == C byte-for-byte.
- **P3 totality/fixed point:** any valid decision vector returns; serialize→parse→serialize is a
  fixed point. Mixed-decision semantic verse warnings are out of scope.
- **P4 purity:** same inputs/vector twice, and again after another vector, return the same bytes.
- **P5 determinism:** complete skeleton equality (ids, unit order/data, slots, anchors,
  annotations) across two builds.
- **P6 directed shapes:** run P1–P4 plus exact chosen-side contribution/no-leakage witnesses for
  each dedicated strategy above.
- **P7 strict id:** inject a guaranteed-absent id and assert `UnknownUnitId` with no output.

Gate: each dedicated directed property executes independently under proptest. Run
`cargo test --workspace`; it is green. A generator is not considered coverage merely because its
enum contains a shape—the property must consume the shape-specific strategy.

### 7. Wasm/DTO/package release and consumer handoff

- Add Tsify/serde DTOs for skeleton, slots, anchors, units, statuses, sides, dup/coverage metadata,
  and merge request.
- Change `diffTokens`, `diffUsfm`, parsed methods, and by-chapter maps to the canonical skeleton
  return; add `mergeDiffBlocks`; make `revertDiffBlock` throw on strict errors.
- Remove flat diff DTOs/exports only after no wasm function references them. Do not leave two
  alignment/coalesce implementations.
- Regenerate both `pkg-bundler` and `pkg-web`; inspect `.d.ts` for exact unions, camelCase fields,
  `Record<string, MergeSide>`, removed options/plural revert, and new return types.
- Update `scripts/test-web-package.mjs` to exercise P2, mixed decisions, move two-slot identity, and
  the unknown-id throw. Run wasm goldens and review diffs; SID/block-id churn must be intentional.
- Run `npm run golden:wasm` first. If only expected contract changes fail, run
  `npm run golden:wasm:update`, inspect every changed golden, then rerun `npm run golden:wasm`.
- **No publication is part of this release** (Will, 2026-07-16): nothing is pushed to npm or
  crates.io. Version bumps are local-consumption bookkeeping only — Zephyr consumes the
  regenerated committed packages and its Tauri crate tracks the Rust API directly. Bump npm
  `0.0.8` → `0.0.9` and the workspace `0.1.1` → `0.2.0` so the breaking surface is visible to
  the consumer, and stop there.
- Export `normalizeTokenSids(tokens, bookCode)` from wasm — the canonical pass as a utility —
  plus a dual-run parity fixture (Rust pass vs Zephyr `mutAddSids` output on bridge/dup/intro
  streams) gating any consumer swap.
- Produce a Zephyr handoff listing required changes in `IUsfmOnionService`, onion types/adapters,
  `WebUsfmOnionService`, `TauriUsfmOnionService`, Tauri Rust DTO/status mapping and commands,
  `diffScope`, mocks/tests, removal of `defaultBuildSidBlocksOptions`, and the six
  `normalizeTokenSids`/`mutAddSids` call sites that later collapse onto the wasm export
  (tokenReplace, scriptureProjectToParsedFiles, usfmTokenStreamSerializedAdapter,
  maintainMetadata, rebuildParsedFileFromUsfm, parseRecoveredBookContents).

Gate commands:

```sh
cargo test --workspace
npm run test:wasm
npm run golden:wasm
npm run build:wasm
```

Final inspection gates:

- generated TS declarations expose the exact new model and no removed surface;
- package smoke tests prove both merge identities and strict error behavior;
- golden changes are reviewed, not blindly accepted;
- the prototype harness still reports 23/23 plus 21 pins;
- Zephyr migration is explicitly handed off and not falsely claimed complete.

## Breaking-change inventory

Onion repository:

- `Sid` layout/constructors/formatters and all exhaustive SID formatting call sites;
- `BuildSidBlocksOptions` and all builder/API/wasm signatures using it;
- flat `ChapterTokenDiff`/`SidBlockDiff` wasm return shapes and `DiffStatus` consumers;
- `ParsedUsfm.diff`, `diffByChapter`, top-level diff functions, token diff builders;
- single revert return/error behavior; plural revert deletion;
- `src/lib.rs`, README, playground, package smoke tests, wasm golden outputs;
- generated bundler/web JS, wasm, `.d.ts`, and package manifests;
- npm and workspace semver bumps (bookkeeping only — publication to npm/crates.io is explicitly
  out of scope for this release).

Known Zephyr fallout (separate repo/plan):

- generated onion type imports and `Diff` domain mapping;
- `IUsfmOnionService` plus web and Tauri implementations;
- Tauri Rust `BuildSidBlocksOptionsDto`, `map_diff_status`, diff/revert commands, command registry;
- `diffScope`, save/external-compare staging, mocks, and tests;
- `mutAddSids` retirement only after dual-run parity on bridge/duplicate/intro fixtures.

## Testing decisions (hardened)

- Fixture pins protect human narration and exact reference behavior.
- Properties protect the general projection invariants and future refactors.
- Native parsed-token and app-shaped FormatToken paths are both required.
- Tests assert serialized bytes and structural witnesses, not snapshots alone.
- Wasm smoke/golden tests protect the actual package contract; generated declarations are reviewed.

## Risks and stop conditions

- **SID representation drift:** any public surface emits a bridge start without its end. Stop and
  centralize formatting before continuing.
- **Duplicate normalization drift:** onion and Zephyr assign different `_dup_N` values. Do not
  retire `mutAddSids`; add a parity fixture and keep the consumer shim.
- **Two live diff algorithms:** wasm/native legacy and skeleton paths disagree. Stop and route all
  public diff/merge paths through the skeleton builder.
- **Unpinned new narration:** multiple bridge coverers or another ambiguous annotation appears.
  Add a prototype case/pin and get the decision before coding a tie-break.
- **Generator erosion:** shrinking removes the directed geometry. Move that property to a
  shape-preserving structured strategy.
- **Golden churn:** unrelated fields change. Revert the unrelated change; do not bless bulk output.

## Explicitly deferred

- Footnotes/cross-references as separate compare units.
- JSON patch/splice transfer format.
- Content-similarity pairing across verse numbers or fuzzy similarity within a key.
- Production hunk/row/chapter projection helpers and all move-rendering UI choices.
- Canonical-order LCS tie-break.
- A batched multi-chapter merge API; the core remains slice-generic, Zephyr batches per scope item.
- Zephyr removal of `mutAddSids`; only begin after new onion token DTOs pass a dual-run parity gate.

## Resolved former open questions

1. **Hunk projection:** Zephyr/app-side. Onion returns slots, both-side SIDs, and anchors; Rust
   directly pins the projection invariants.
2. **`mutAddSids` retirement:** not in this onion change. Ship range/dup/intro parity first, then
   remove it in a consumer migration with dual-run fixtures.
3. **Multi-chapter scope:** core works over any flat slice; Zephyr v1 invokes merge per existing
   scope item. No batched merge API now.
4. **UI verdict (Will, 2026-07-16):** moves render as the collocated box speaking the order
   panel's chip language ("Order was: start · 1 · 2 → Order is now: start · 2 · 1", one split
   positional decision). The production UI is implemented FRESH in Sefer, not ported from the
   prototype; the prototype's `projectRows`/order-neighborhood semantics are the reference.
   Adds no onion surface.

## Definition of done

- All seven step gates are green in order and recorded in the progress file.
- Rust matches all 23 reference cases, all 21 narration pins, and P1–P7 as amended.
- There is one canonical skeleton builder, one merge implementation, and one SID derivation
  algorithm (`derive_canonical_sids`).
- Fuzzy/stale fallbacks and sequential plural revert are absent.
- Native and wasm/package contracts are updated, versioned, and verified. Nothing publishes.
- Remaining consumer work is the named handoff below, not an implicit claim in this plan.

## TBD in Zephyr/Sefer (consumer plan — explicitly NOT this repo's work)

- Review-modal open sequence: normalize BOTH streams (working buffer + baseline) while taking
  the frozen snapshot; diff and merge those exact streams from modal state.
- Migrate the six `normalizeTokenSids`/`mutAddSids` call sites onto the wasm export after the
  dual-run parity fixture is green; then retire `mutAddSids`; then flip external token diff to
  always-derive (deleting the carried-sid interim branch in onion).
- Adapt `IUsfmOnionService`, web + Tauri adapters, Tauri Rust DTO/status mapping (`Moved` is a
  compile break in `map_diff_status`), diff/revert commands, `diffScope` staging, mocks/tests.
- Implement the review UI fresh in Sefer: moves as the collocated box with order-chip wording
  (prototype is the reference, not the codebase); shared-gutter chapter grid fed from the
  skeleton's row alignment; dup/relabel/bridge-coverage warning copy; WS/USFM filters.
- Decide Sefer-side snapshot-staleness UX (freeze/lock typing while the modal is open, abort on
  buffer change) — onion's P7 covers unknown ids only.
