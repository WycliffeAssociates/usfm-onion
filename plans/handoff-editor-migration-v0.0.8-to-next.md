# Handoff — migrate `scripture-editor-proto-2` from usfm_onion v0.0.8 → upcoming release

**For:** an agent adapting the consumer (`scripture-editor-proto-2`) to the new
usfm_onion. **Scope:** the full v0.0.8 (`4e9de55`) → release delta (31 commits
on `chapter-parallelism`, which merges fast-forward onto master).

## How the editor consumes usfm_onion (BOTH surfaces)
- **JS/wasm + TS types:** `package.json` → `"usfm-onion-web": "file:../usfm_onion"`
  (the wasm-pack npm package). TS types are re-exported via
  `src/core/domain/usfm/usfmOnionTypes.ts` (`from "usfm-onion-web"`). It reads
  the serialized JSON — e.g. `matchFormattingByVerseAnchors.ts:203` reads
  `token.attributes`.
- **Native Rust:** the editor also has a `Cargo.toml` depending on the crate
  directly (Tauri/native path). So **Rust-API breaks matter here** even when the
  JSON is unchanged.

Two consequences: (1) JSON-preserving changes are invisible to the TS side but
can still break the native Rust side; (2) verify BOTH after migrating.

**Release not cut yet:** box-attributes and WS3 (below) are still in flight. Do
the JSON/TS + diff migration now against the landed pieces, but **finalize
against the tagged release and a freshly regenerated wasm pkg** (`.d.ts` may
change). Confirm the tag/version with Will (workspace is `0.2.0`; a 0.10.0 jump
was discussed).

---

## MUST-ADAPT changes

### 1. Diff API fully rewritten — merge-as-projection (the big one) — `646db66`
The entire diff surface was replaced. Old → new (both Rust exports and wasm):

| Removed (v0.0.8) | Replaced by |
|---|---|
| `diff_usfm_sources`, `diff_usfm_sources_by_chapter`, `diff_chapter_token_streams` | `diff_skeleton`, `diff_skeleton_by_chapter`, `diff_skeleton_canonical` |
| `ChapterTokenDiff`, `DiffStatus`, `DiffTokenChange`, `DiffUndoSide`, `SidBlockDiff`, `TokenAlignment`, `BuildSidBlocksOptions` | `DiffSkeleton`, `DecisionUnit`, `DecisionUnitKind`, `DecisionStatus`, `Slot`, `SlotRole`, `Anchor`, `CoveredBy`, `CoveredSide`, `UnitId`, `DupContext`, `MergeSide`, `MergeError` |
| `apply_revert_by_block_id`, `apply_reverts_by_block_id` | `revert_diff_block`, `merge_diff_blocks`, `merge_skeleton` |
| `DiffsByChapterMap` = `…Vec<ChapterTokenDiff>` | `DiffsByChapterMap` = `…DiffSkeleton` |

The model is now a diff **skeleton** of `Slot`s over a Myers interleave, with
`DecisionUnit`s carrying merge/revert decisions. The editor's `DiffModal/`
already imports `DecisionUnit`/`DiffSkeleton` in places
(`chapterDiffViewModel.ts`, `rowUsfmOverrides.ts`,
`CompareResultPreviewEditor.tsx`), so it may be **partially migrated** — audit
those and finish. Design intent is in usfm_onion `plans/plan-merge-projection.md`
(and the crate's `docs/`).

### 2. Standalone token SID normalization — `f70d4f0`
A standalone token-SID-normalization entry point was added. The editor has
`src/core/domain/usfm/tokenSidNormalization.ts` — check whether it should now
delegate to the library function instead of duplicating logic.

### 3. Lint findings are now in a canonical order — `2352e2e` (`feat(lint)!`)
Lint findings are emitted in a deterministic canonical order (this also backs
the new behavior oracle). If the editor sorts or de-dupes findings assuming the
old order, simplify to rely on the canonical order. (Scope-aware lint + dropped
consistency rules landed in 0.0.7, *before* this baseline — not part of this
delta.)

### 4. `MarkerMetadata` gained a private `index` field — `ec117c8` (`perf(parse)!`)
Native-Rust-only break: `MarkerMetadata` now has a private `index` field, so
external **struct-literal construction** or **exhaustive destructuring** of it
breaks. Reading its public fields (`canonical`, `kind`, `family`) is fine.
**JSON/TS unchanged** (`#[serde(skip)]`) — the wasm/TS side is unaffected.

### 5. (IN FLIGHT) Box the marker attributes — Token shrink
`TokenData::Marker`/`Milestone` are being changed: `attributes: Vec<…>` +
`attribute_source: Option<(Span,&str)>` fold into one boxed
`attrs: Option<Box<MarkerAttrs>>`. Native-Rust break for anyone matching/
constructing those variants or reading the two fields directly — **use the
stable `Token::attributes()` accessor** (`-> Option<&[AttributeItem]>`) instead.
**Serialized JSON is preserved** (same `attributes`/`attribute_source` keys), so
the TS side (`token.attributes`) is intended to keep working — but re-verify the
regenerated `.d.ts` once the release is cut.

---

## TRANSPARENT — no editor changes needed (perf/behavior-preserving)
All oracle-gated **byte-identical** across the real-world corpus at 1 and full
threads — output does not move, only speed:
- Chapter-parallel parse/lint/usj/usx/vref/html (`5bc8700`, `27b3e05`,
  `320c119`, `72e1a01`, `3460c5a`) — transparent; no API change.
- Marker-resolution rework internals, Vec presizing, memchr boundary scan,
  FxHash sweep (`ec117c8` internals, `8d4b848`, `0aeec8d`, `5f147fb`).
- (IN FLIGHT) WS3 — delimiter-whitespace absorption moved into the lexer;
  byte-identical, no API/JSON change.
- Note: `pub mod par` is now exported (parallelism seam) but it's an internal
  concurrency detail, not intended editor API.
- A `fix(lint)` (`c2d6ea7`, stale note-context on close) *corrects* lint output
  in a narrow unclosed-note case — if the editor has fixtures pinning the old
  (buggy) finding, update them.

---

## Suggested migration order + verification
1. **Diff (item 1)** — the bulk of the work; finish the `DiffSkeleton`/
   `DecisionUnit` migration in `DiffModal/` and the wasm diff calls.
2. Items 2–3 (token SID delegation, lint order assumptions) — small.
3. Items 4–5 (native Rust) — only if the editor's Rust/Tauri path constructs or
   destructures `MarkerMetadata` / `TokenData::Marker`; switch to accessors.
4. **Verify both surfaces:** rebuild the wasm pkg, `npm i` the `file:` dep,
   run the editor's typecheck + tests; and build the editor's native Rust path
   against the crate. Diff the regenerated `.d.ts` vs the old to catch any TS
   surface movement (should be limited to the diff types).

**Grep starting points in the editor:** `usfmOnionTypes.ts`,
`matchFormattingByVerseAnchors.ts` (reads `token.attributes`),
`tokenSidNormalization.ts`, `DiffModal/` (chapterDiffViewModel, rowUsfmOverrides,
CompareResultPreviewEditor), and any `diff_usfm_sources`/`ChapterTokenDiff`/
`DiffStatus` references (all removed).

## 0.0.10 additions (on top of the above)
- **`fe`/`ef`/`ex` canonical fix** (`48215ef`): `token.metadata.canonical` for
  `\fe` (endnote), `\ef` (extended footnote), `\ex` (extended cross-reference)
  now returns `"fe"`/`"ef"`/`"ex"` instead of the old collapsed `"f"`/`"f"`/`"x"`.
  Field *shape* is unchanged; only the value for these three markers differs.
  usj/usx/html output is unchanged (they always used the raw marker name). Only
  matters if the editor reads `metadata.canonical` to identify note type — and
  the new value is the correct one.
- **Index-fold** (`b5d03e9`) and any WS3: internal/non-breaking.
