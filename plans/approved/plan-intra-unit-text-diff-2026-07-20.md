# Plan: Onion-owned intra-unit text diff (word/char runs)

Status: design settled by interview (2026-07-20); implementation **not started** (planning only).
Repo scope: **usfm_onion only.** Zephyr/Sefer consumer work is out of scope end-to-end; the only
consumer deliverable is the adoption inventory at the bottom.
Dials: interview = ruthless (done, 2026-07-20); testing = **standard, with a hardened invariant set**
(native/wasm parity, none-vs-words non-interference, determinism, and Unicode pins are must-pass gates).

Builds directly on the merge-projection engine landed at `646db66` (see
`plans/plan-merge-projection.md`). This plan adds **presentation metadata only**; it does not touch
the decision grain, slots, unit ids, statuses, anchors, movement detection, or merge projection.

## Problem

Zephyr's Option C review consumes Onion's `DiffSkeleton` and, for a `Modified` unit, shows the
complete left/right verse text **without indicating which words changed**
(`chapterDiffViewModel.ts:tokensToReviewText` just concatenates token `.source`). Meanwhile the
print path (`buildPrintChangeSet.ts`) *does* highlight word-level changes, but with its own
independent `diffWordsWithSpace` call over its own plain-text extraction (`kind === "text"`, then
`.trim()`). Two consumers, two different word-diff implementations, two different notions of "plain
text" — cross-consumer semantic drift with no single source of truth.

## Solution

Onion becomes the single owner of the canonical intra-unit text diff. A new **pure** function
computes, for one already-built `DecisionUnit`, the word- or char-level run breakdown of its
baseline vs current **plain text**. Callers opt in via an extensible `textDiff: "none" | "words" |
"chars"` option (default `"none"`, the current behavior). The result is attached as an **optional
DTO field** (`DecisionUnit.textDiff`); it never becomes a decision unit, never carries a merge
decision, and is provably inert to skeleton construction and merge.

Word runs are **presentation metadata**. The verse/hunk `DecisionUnit` remains the only decision and
merge unit. Enabling word/char detail changes nothing about slots, ids, statuses, anchors,
movement, coverage, or merge output — it only adds a field some callers read.

## Settled decisions (interview, 2026-07-20)

1. **Architecture = separate pure function + optional DTO field** (not embedded on native
   `DecisionUnit<T>`; not computed inside `build_skeleton`). Rationale: merge/revert are inert *by
   construction* (there is no native field to read, and merge takes raw tokens / a skeleton whose
   unit type is unchanged); the core builder, all 23 fixtures, and P1–P7 proptests are literally
   unaffected because the type they build/compare does not change; word/char segmentation stays out
   of skeleton identity; `"none"` costs nothing. The only cost — a native Rust consumer calls a
   helper instead of reading a struct field — is negligible and keeps one source of truth.
2. **Mode is an extensible enum, not a boolean:** `none | words | chars`. Default `none`.
3. **Segmentation is Unicode-correct**, reusing the already-present `similar` crate with its
   `unicode` feature enabled:
   - `words` → `similar::TextDiff::from_unicode_words` (UAX #29 boundaries — correct across Latin,
     RTL, and no-space scripts like Thai/Khmer/CJK).
   - `chars` → `similar::TextDiff::from_graphemes` (grapheme clusters — a base letter plus a
     combining mark, e.g. Hebrew niqqud, stays one unit).
   Both native and wasm compile the same crate + feature, so segmentation is identical across
   targets. No new crate dependency; one pinned feature.
4. **Wire shape = pre-split baseline/current run arrays**, text-only:
   ```ts
   type TextDiffMode = "none" | "words" | "chars";
   type TextDiffRunKind = "unchanged" | "added" | "removed";
   type TextDiffRun = { text: string; kind: TextDiffRunKind };
   type UnitTextDiff = {
     baseline: TextDiffRun[]; // runs are "unchanged" | "removed"
     current:  TextDiffRun[]; // runs are "unchanged" | "added"
   };
   ```
   Zephyr renders each side directly and deletes both its `diffWordsWithSpace` call **and** its
   per-side filter logic — no alignment or filter code survives downstream to drift. Byte/UTF-16
   offsets and token-id provenance are **deferred** until a consumer actually needs them (see
   "Explicitly deferred").
5. **Plain text = reader-visible token sources only**, notes' inner prose **included**:
   - Include token kinds `text`, `verticalWhitespace` (newline), and `optBreak` — i.e. the
     content/whitespace-bearing kinds. This matches Zephyr's interactive `CONTENT_TOKEN_KINDS`
     exactly and preserves *real* inter-word whitespace, so no synthetic spacing is inserted and
     marker-wrapped text never glues wrong (`word\add ed\add*` → `"worded"`, not `"word ed"`).
   - Exclude `marker`, `endMarker`, `milestone`, `milestoneEnd`, `number` (verse/chapter numbers),
     and `bookCode`. Consequence: relabels, renumbers, and USFM-structure-only edits produce **no**
     word change (already carried by `relabeled` / `is_usfm_structure_change`).
   - Note/character-marker inner prose (e.g. `\ft a note`) flows in as reviewable text in v1;
     segregating notes into their own sub-stream is deferred.
6. **Which statuses get a `textDiff`** (confines work to changed units):
   - `Modified` (shared byte-diff, or coalesced byte-diff/displaced-and-modified) → real
     word/char diff over both sides' plain text.
   - `Added` → `baseline: []`, `current: [{ text: <all>, kind: "added" }]`.
   - `Deleted` → `baseline: [{ text: <all>, kind: "removed" }]`, `current: []`.
   - `Unchanged` and `Moved` (byte-equal) → `textDiff` absent (`None`). No false word changes on a
     text-identical move or a pure relabel.
   (Added/Deleted emit a single contiguous run per side — visually one highlight block, and it
   matches print's current `buildSides`. Segmenting them into per-word runs is unnecessary.)
7. **Option plumbing:** every wasm diff entry point gains an **optional** trailing `DiffOptions`
   argument (source-compatible; omitting it = `"none"` = today's behavior). No change to native
   `diff_skeleton*` signatures — the native word/char path is the standalone helper.

## Non-goals / invariants that must not move

- Do **not** change the verse/hunk decision grain; word runs never become decision units.
- Do **not** create nested decisions; do **not** make merge depend on word runs.
- Do **not** restore Zephyr's legacy flat alignment DTO.
- Do **not** introduce SID normalization into the text-diff API.
- Do **not** change move/coalescing semantics, statuses, anchors, `covered_by`, `dup_context`,
  `relabeled`, or the whitespace/USFM-structure flags.
- Do **not** modify the unrelated dirty files (`examples/grain_stats.rs`,
  `plans/plan-parallelism.md`, `plans/plan-sid-model-fidelity.md`, `tests/lint_oracle.rs`,
  `tests/lint_oracle_baseline.txt`).
- Do **not** publish to npm/crates.io. Version bump is optional bookkeeping only (see Gate 5).

## User stories

1. As a translation reviewer, I want changed words within a Modified verse highlighted in the
   interactive review, so that I can see *what* changed without re-reading the whole verse.
2. As a reviewer, I want the interactive review and the printed change report to highlight the
   *same* words the same way, so that the two views never disagree.
3. As a reviewer of a footnote edit, I want the changed note prose highlighted inside the verse's
   word diff, so that note-only edits are reviewable (v1 behavior).
4. As a reviewer of a verse that only moved (identical text) or was only relabeled, I want **no**
   word-change highlighting, so that a move/relabel isn't mistaken for a content edit.
5. As a reviewer of a USFM-structure-only edit (`\p`→`\m`, marker wrap toggled), I want **no** word
   change shown, so that structure churn doesn't read as prose churn.
6. As a reviewer working in Hebrew (combining marks), Arabic (RTL), or Thai (no inter-word spaces),
   I want segmentation that respects my script, so that highlights land on real word/char units.
7. As a caller who doesn't want the cost, I want `"none"` (the default) to compute nothing, so that
   existing diff performance is unchanged.
8. As a native (Tauri) Rust consumer, I want the same runs the browser gets from the same function,
   so that native and wasm never diverge.
9. As a Zephyr maintainer, I want to delete Zephyr's `diffWordsWithSpace` usage and per-side filter
   logic, so that alignment lives in exactly one place.

## Domain contract (native, `src/diff/`)

New module `src/diff/text_diff.rs`, re-exported from `src/diff/mod.rs`.

```rust
/// Requested granularity for the intra-unit text diff. `None` computes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum TextDiffMode {
    #[default]
    None,
    Words, // similar::TextDiff::from_unicode_words
    Chars, // similar::TextDiff::from_graphemes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TextDiffRunKind { Unchanged, Added, Removed }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextDiffRun { pub text: String, pub kind: TextDiffRunKind }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnitTextDiff {
    pub baseline: Vec<TextDiffRun>, // kinds: Unchanged | Removed
    pub current:  Vec<TextDiffRun>, // kinds: Unchanged | Added
}

/// Pure. Reads only the unit's token slices; never mutates, never consults the
/// skeleton, slots, ids, or decisions. Returns `None` for `TextDiffMode::None`
/// and for `Unchanged`/`Moved` (byte-equal) units. Deterministic.
pub fn unit_text_diff<T: DiffableToken>(
    unit: &DecisionUnit<T>,
    mode: TextDiffMode,
) -> Option<UnitTextDiff>;

/// Native convenience sugar (2026-07-24): one call that diffs and computes the
/// per-unit text runs, returned index-aligned with `skeleton.units`. This is
/// how a native caller avoids writing the loop — WITHOUT a `text_diff` field on
/// `DecisionUnit<T>`. The word diff rides *beside* the unit, so the core type,
/// the 23 fixtures, the P1–P7 proptests, and merge/revert stay byte-identical
/// (non-interference remains true by construction — decision #1). A struct
/// wrapper (`Vec<UnitWithText<T>>`) is an acceptable alternative to the tuple.
pub fn diff_skeleton_with_text<T: DiffableToken>(
    baseline: &[T],
    current: &[T],
    mode: TextDiffMode,
) -> (DiffSkeleton<T>, Vec<Option<UnitTextDiff>>);
```

**Do NOT add `text_diff` as a field on the native `DecisionUnit<T>`.** That is the one
thing that would forfeit the tautological non-interference (it changes the type the
builder/merge/fixtures construct and compare). The wire DTO `DecisionUnit` embeds the field
(additive on a separate type); the native side pairs alongside.

**Parallelism:** `unit_text_diff` per unit is independent, so `diff_skeleton_with_text`'s
loop MAY use `par_iter` — but native-only, behind the existing `rayon` cfg gate, and only if
a measurement justifies it. v1 is serial (the per-`Modified`-unit `similar` diff is unlikely
to beat rayon overhead on one book); revisit per the spike-first rule.

Internal helpers (private):

- `fn unit_plain_text<T: DiffableToken>(tokens: &[T]) -> String` — concatenates the `.text()` of
  tokens whose `kind_key()` ∈ {`"text"`, `"verticalWhitespace"`, `"optBreak"`}, in stream order.
  (These are the wire kind keys from `token_kind_key` in `src/diff/mod.rs`.)
- A `similar`-driven splitter that, given both plain-text strings and the mode, produces the two
  run vectors, coalescing consecutive same-kind segments into one run.

Status gating inside `unit_text_diff`:
- `Unchanged` / `Moved` → `None`.
- `Added` → `Some({ baseline: [], current: [one Added run of current plain text] })` (empty current
  plain text → `current: []`).
- `Deleted` → symmetric.
- `Modified` → run the `similar` diff over both plain texts; map `Equal`→Unchanged, `Delete`→Removed
  (baseline only), `Insert`→Added (current only). Unchanged segments appear in both arrays.

### Native reconstruction & alignment invariants (asserted by tests)

- `concat(text of unit.baseline runs) == unit_plain_text(baseline_tokens)`.
- `concat(text of unit.current runs) == unit_plain_text(current_tokens)`.
- `baseline` runs carry only `Unchanged`/`Removed`; `current` runs only `Unchanged`/`Added`.
- The ordered `Unchanged` run texts are identical between the two arrays (the common subsequence).
- Idempotent/deterministic: two calls on the same unit+mode yield equal `UnitTextDiff`.

## Wire contract (single-sourced in `usfm_onion_dto`, re-exported by wasm)

**SSOT update (supersedes the original 2026-07-20 text):** the new wire types
(`TextDiffMode`, `TextDiffRunKind`, `TextDiffRun`, `UnitTextDiff`, `DiffOptions`, and the
optional `textDiff` field on the DTO `DecisionUnit`) are defined **once in
`usfm_onion_dto`** — `#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]` + serde
`rename_all = "camelCase"`, with `From<Native…>` conversions — and `usfm_onion_wasm`
**re-exports** them (`pub use usfm_onion_dto::{…}`). Do NOT hand-declare them in the wasm
crate (that is the drift class the enum SSOT work eliminated; see the dto module-doc
checklist). What stays in wasm is only the boundary *glue*: `map_native_skeleton` /
`map_native_decision_unit` gaining the `TextDiffMode` arg and calling `unit_text_diff` per
unit (the wire-side `forEach`). tsify DTOs (camelCase), plus:

```rust
#[derive(... Tsify)] #[serde(rename_all = "camelCase")]
pub struct DiffOptions {
    #[serde(default)]
    text_diff: TextDiffMode, // "none" (default) | "words" | "chars"
}
```

`DecisionUnit` DTO grows one field:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
text_diff: Option<UnitTextDiff>,
```

Entry points gain an optional trailing arg (omission ⇒ `"none"`):

- `diffUsfm(left, right, options?)` — `wasm_diff_usfm`
- `diffUsfmByChapter(left, right, options?)` — `wasm_diff_usfm_by_chapter`
- `diffTokens(left, right, options?)` — `wasm_diff_tokens`
- `ParsedUsfm.diff(other, options?)`, `ParsedUsfm.diffByChapter(other, options?)`

Mapping seam: `map_native_skeleton` / `map_native_decision_unit` take a `TextDiffMode` and, per
unit, call `unit_text_diff(unit, mode)` to populate `text_diff`. This requires adding a
`T: DiffableToken` bound to those two map fns — satisfied by every instantiation (`NativeToken`,
`WalkToken`, `FormatToken`). No other wasm function changes; merge/revert paths are untouched.

## Modules / ownership by gate

| Gate | Files | What lands |
|---|---|---|
| 1 | `src/diff/text_diff.rs` (new), `src/diff/mod.rs` (add `pub mod`/re-export), `Cargo.toml` (`similar` `unicode` feature) | Native types, `unit_text_diff`, `diff_skeleton_with_text` convenience, plain-text + splitter helpers, unit tests |
| 2 | `src/diff/text_diff_fixtures.rs` (new, `#[cfg(test)]`) | Fixture oracle: status coverage, USFM/relabel/move suppression, multilingual pins |
| 3 | `src/diff/skeleton_proptest.rs` (extend) or new `text_diff_proptest.rs` | Reconstruction/alignment/determinism/non-interference properties |
| 4 | `crates/usfm_onion_dto/src/lib.rs` (new wire DTOs + `From<Native…>`); `crates/usfm_onion_wasm/src/lib.rs` (re-export, optional args, map-seam threading) | wire DTOs **single-sourced in dto** + re-exported by wasm; `DiffOptions` optional args; map-seam `forEach` |
| 5 | `pkg-bundler/*`, `pkg-web/*`, `scripts/test-web-package.mjs`, wasm goldens | Regenerate declarations/packages, smoke + golden updates |

## Implementation gates (execute in order; each starts by adding its failing tests)

Append surprises/deviations/verification to `plans/progress-intra-unit-text-diff.md`. Do not start a
gate while the previous gate is red.

### Gate 1 — native `unit_text_diff` + Unicode segmentation

- Enable `similar`'s `unicode` feature in `Cargo.toml` (verify `from_unicode_words` /
  `from_graphemes` are exposed at the pinned version; confirm the feature name).
- Add the module, types, `unit_text_diff`, `unit_plain_text`, and the splitter.
- Unit tests: plain-text extraction (markers/numbers/bookcodes excluded; text+newline+optbreak kept;
  `\add`-wrapped text not glued); Modified word runs; Added/Deleted single-run; Unchanged/Moved →
  None; reconstruction + common-subsequence invariants; idempotency.

Gate: `cargo test -p usfm_onion diff::text_diff` green; `cargo build` green (feature resolves).
Stop condition: if the `unicode` feature is unexpectedly heavy or unavailable, stop and report
before falling back to `from_words`/`from_chars` (that fallback needs Will's sign-off because it
changes the pinned multilingual behavior).

### Gate 2 — fixture oracle (reuse the 23-case catalog + multilingual pins)

Drive fixtures off the existing merge cases where possible (they already parse under `\id GEN`):
- **Modified stable runs:** case 1 (`heaven`→`heavens`) → current has one `added` run `"heavens"`
  bounded by unchanged runs; baseline has `"heaven"` as `removed`.
- **Whitespace-only:** case 7 → Modified, `is_whitespace_change` true; runs show only a
  whitespace-run change, no lexical word run changes. (Confirms WS stays distinguishable.)
- **USFM-structure-only:** case 8 chapter-open (`\p`→`\m`) → both plain texts equal → runs all
  `Unchanged`, no false change.
- **Moved (byte-equal):** case 10 → `textDiff` is `None`.
- **Relabeled (byte-equal):** case 13 survivor → `None`.
- **Added / Deleted:** cases 4 (added v3) / 5 (deleted v3) → single `added`/`removed` run.
- **Note prose edit:** case 18 (`a note`→`an edited note`) → the note prose participates; markers
  (`\f`, `\ft`, `\f*`) and the caller are absent from the runs.
- **Repeated words / ambiguous alignment:** a hand-built fixture (`"the the the"` → `"the the"`)
  asserting a single deterministic run layout.
- **Multilingual pins (dedicated fixtures):** apostrophe (`don't`→`don’t`), Hebrew base+niqqud
  (combining-mark edit under `chars`), Arabic RTL word edit, Thai no-space segment edit under
  `words`, Latin punctuation-adjacent edit. Each pins exact runs for its mode.
- Run the whole oracle through **both** native parsed `Token`s and app-shaped `FormatToken`s with
  drifted ids (mirror `skeleton_fixtures.rs`), asserting identical runs.

Gate: fixture suite green on both token shapes.

### Gate 3 — properties (non-interference, reconstruction, determinism)

Reuse the structured `edited_doc_strategy` from `skeleton_proptest.rs`.

- **Reconstruction (P-text-1):** for every unit and both modes, side-run concatenation equals that
  side's plain text; kinds are side-appropriate; unchanged texts match across arrays.
- **Non-interference (P-text-2):** the built `DiffSkeleton<T>` is bit-identical regardless of mode
  (mode never enters the builder). Assert the native skeleton equals itself, and that mapping the
  DTO with `None` vs `Words`/`Chars` differs **only** in `text_diff` — every other field
  (ids, slots, roles, anchors, statuses, `coveredBy`, `dupContext`, `relabeled`, `displaced`,
  flags, tokens) is byte-equal.
- **Merge unaffected (P-text-3):** for any decision vector, `merge_skeleton` output is identical
  whether or not `text_diff` was requested (trivial — merge takes tokens/skeleton, not runs — but
  pinned).
- **Determinism (P-text-4):** two `unit_text_diff` calls on the same unit+mode are equal.

Gate: `cargo test --workspace` green (including all pre-existing skeleton fixtures/proptests, which
must be untouched and passing).

### Gate 4 — wasm/DTO surface

- Add Tsify mirrors: `TextDiffMode`, `TextDiffRunKind`, `TextDiffRun`, `UnitTextDiff`, `DiffOptions`.
- Add `text_diff: Option<UnitTextDiff>` to the `DecisionUnit` DTO (skip-if-none).
- Thread `TextDiffMode` through `map_native_skeleton` / `map_native_decision_unit` (add
  `T: DiffableToken` bound); add the optional `options` arg to the five entry points.
- Rust wasm-side unit test (or `scripts/test-web-package.mjs` addition) asserting: `"none"` (and
  omitted) ⇒ no `textDiff`; `"words"`/`"chars"` ⇒ `textDiff` present on Modified/Added/Deleted and
  absent on Unchanged/Moved; a Modified verse's `current` runs render the expected highlighted text.

Gate: `cargo test --workspace` + `npm run test:wasm` green.

### Gate 5 — package regen, goldens, parity, handoff

- Regenerate `pkg-bundler` and `pkg-web`; inspect `.d.ts` for the exact `TextDiffMode` union,
  camelCase run/`UnitTextDiff` types, optional `DiffOptions` arg, and optional `textDiff?` field.
- `npm run golden:wasm`; if only the intended additive changes fail, `golden:wasm:update`, inspect
  every changed golden (all churn must be the new optional field / new inputs), rerun `golden:wasm`.
- **Native↔wasm parity fixture:** a dual-run test comparing native `unit_text_diff` runs against the
  DTO `textDiff` for the same inputs across the multilingual pins (mirrors the merge parity gate).
- **Perf note:** `"none"` does zero text work by construction; confirm the default path allocates
  nothing new (a criterion spot-check on a large book with `"none"` vs the pre-feature baseline is
  optional evidence, not a gate).
- **Version:** optional bookkeeping only. Bump the npm minor **only if** a consumer needs to feature-
  detect; otherwise do not choose a version (per handoff constraint). Nothing publishes.
- Produce the Zephyr adoption inventory (below).

Gate commands:
```sh
cargo test --workspace
npm run test:wasm
npm run golden:wasm
npm run build:wasm
```

## Compatibility & performance

- **Additive & backward-compatible.** New optional DTO field (skip-if-none) and optional trailing
  args. Callers that ignore both see byte-identical output to today. No native `diff_skeleton*`
  signature changes.
- **Cost is confined to changed units** and only when `words`/`chars` is requested: `unit_text_diff`
  returns early for `None` mode and for `Unchanged`/`Moved`. Whole-document diffs pay nothing for
  the unchanged majority.
- **Determinism across targets** is guaranteed by using one crate+feature (`similar` + `unicode`)
  compiled into both native and wasm.

## Testing decisions (standard + hardened invariant set)

- Fixture pins protect exact human-visible run behavior, including the multilingual cases (these are
  the load-bearing correctness pins and are treated as hardened).
- Properties protect the general reconstruction/alignment invariants and — critically —
  **non-interference**: that turning the feature on cannot perturb the skeleton, merge, or any
  existing field. With Option A this is cheap and near-tautological, which is exactly the point.
- Native parsed-token and app-shaped `FormatToken` paths are both required (mirrors the merge suite).
- Native↔wasm parity is asserted directly, not assumed.
- Existing `skeleton_fixtures.rs` / `skeleton_proptest.rs` must remain **unmodified** and green;
  any change to them is a signal the core was touched and should be reverted.

## Rollback boundary

The entire feature is additive and self-contained: one new native module + one pure function + one
optional DTO field + optional wasm args + new tests. Reverting the feature commit(s) restores exact
current behavior; nothing else depends on it. There is no data migration and no persisted state.

## Zephyr/Sefer adoption (consumer plan — explicitly NOT this repo's work)

- After regenerated packages land, `usfmOnionTypes.ts` picks up `UnitTextDiff`/`TextDiffRun`/
  `TextDiffMode`/`DiffOptions` automatically (it re-exports onion types). Add the aliases.
- Thread `{ textDiff: "words" }` (or `"chars"`) into the diff calls behind `diffScope` /
  `IUsfmOnionService` (web + Tauri).
- Interactive review (`DiffModalListView.tsx` / `chapterDiffViewModel.ts`): replace
  `tokensToReviewText` plain rendering with rendering `unit.textDiff.baseline` /
  `.current` runs (highlight `removed`/`added`); keep the `showUsfmMarkers` toggle by falling back
  to token-source rendering when markers are shown.
- Print (`buildPrintChangeSet.ts`): delete `diffWordsWithSpace` and `buildSides`' word-diff branch;
  map `unit.textDiff` runs → `PrintWordRun[]` directly (`unchanged|added|removed` → the existing
  marks). This removes the second implementation and its plain-text definition drift.
- Note the intentional print behavior shift: whitespace now participates as whitespace runs (Onion's
  plain text includes newline/optBreak), so trimming/spacing is a render choice, not an alignment
  input.

## Go / no-go recommendation

**Go**, as an additive follow-on to the merge-projection engine. The design is low-risk by
construction (Option A makes non-interference near-tautological), reuses existing machinery
(`similar` Myers, the `DiffableToken` surface, the `classify_text_diff` plain-text intuition, the
23-case fixture harness), and lands the whole thing behind a defaulted `"none"` option so nothing
changes until a caller opts in. Open follow-ups (offsets/token-id provenance, note sub-streams,
`from_words` fallback) are all deferred and none block v1.

## Explicitly deferred

- Byte/scalar/UTF-16 offsets and token-id provenance on runs (add when a consumer needs
  reconstruct-free DOM anchoring; the pre-split text-run shape is forward-compatible with adding an
  optional `span`/`tokenId` per run later).
- Segregating note/cross-reference text into its own sub-stream (v1 includes note prose inline).
- A `from_words`/`from_chars` (non-Unicode) fallback mode.
- Embedding `text_diff` on the native `DecisionUnit<T>` (rejected in favor of the pure helper).
- Any change to the decision grain, merge, or move/coalescing semantics.
```
