# Progress — serial parse marker-resolution cost

Append-only execution log for `plans/plan-parse-hot-path.md`.

## 2026-07-21 — plan sanity review

Status: plan revised; no product code changed.

Assumptions selected:

- Standard-depth plan, regular interview, hardened behavior-preservation gate.
- WS2 should first optimize the lex/parse path actually measured by
  `parse/serial`; lint/catalog compaction is a separate follow-up.
- Preserve public Rust lexeme/token shapes in WS2A.
- Use a typed derived projection before considering a packed `u64`.

Corrections to the prior draft:

- Both parser-token and lexer-lexeme presizing landed together in `8d4b848`;
  WS1b was stale.
- The parser contains seven mutually exclusive `structural_marker_info` call
  sites, not seven calls per marker occurrence.
- `benches/operations.rs` measures Luke, while the motivating serial profile is
  Psalms; a matching Psalms Criterion lane is required before WS2.
- `MARKER_SPECS`/`MARKER_WHITESPACE` are not the complete derivation contract;
  payload, default whitespace, aliases, families, structural behavior, and raw
  variant roles are also derived by code.
- `milestone_end`, nestedness, alias/numbered family role, and raw spelling are
  occurrence facts and cannot live in one canonical marker row.
- The live token/lexeme representations do not carry a dense numeric marker
  index. Claiming lexer-to-parser-to-lint field reads therefore assumed an
  unmade public API decision.
- The oracle proves serialized behavior, not downstream Rust source
  compatibility.

Approaches deferred:

- packed `u64` row until typed-row footprint/load cost is measured;
- perfect hash/direct ASCII table until one consolidated lookup remains hot;
- dense ID on final `TokenData` and lint context masks;
- lexer-owned delimiter whitespace until WS2 is measured.

Open decisions:

- Will to confirm the proposed 5% minimum win and 3% maximum regression gates.
- If WS2A leaves lookup hot, Will to choose the WS2B API strategy.
- Whether WS3 is desired as maintainability work without a performance win.

## 2026-07-21 — Will decisions + plan edits

- **API breakage: approved for this cycle.** A semver-breaking token/lexeme
  shape change is acceptable — `master` already carries a breaking diff change
  and the editor consumer will be updated alongside. WS2B chooses its mechanism
  (dense handle on token vs sidecar vec) on engineering merit, not semver fear;
  serialized output must still be byte-identical (oracle-gated). Changelog any
  source break so the editor updates in lockstep.
- Plan clarified on three points: (a) the motivation is the cross-phase 3×
  marker resolution (lexer metadata + parser structural + parser whitespace),
  not the within-parse arm count; (b) the lint context-validity `u32` bitmask is
  a real algorithmic win but belongs to a separate lint workstream, not WS2A's
  parse projection, and needs no packing; (c) WS2B breakage decision recorded.
- Verified against source this session: the seven `structural_marker_info` sites
  are indeed disjoint match arms (one fires per token); the public token types
  are **not** `#[non_exhaustive]`, so a field add is a real source break (hence
  the explicit approval above).

## 2026-07-21 — WS0 gate established (pinned baselines)

Everything below is pinned at the WS0 commit on `chapter-parallelism`. The
Sonnet agent implementing WS2A compares against these; do not regenerate.

**Oracle — green both thread configs (never `BLESS`):**

```sh
cargo test --test lint_oracle -- --ignored --exact lint_oracle_is_stable
RAYON_NUM_THREADS=1 cargo test --test lint_oracle -- --ignored --exact lint_oracle_is_stable
```

Both pass at this SHA. The single ignored test is `lint_oracle_is_stable`.

**Criterion baseline `ws2-pre`** (local artifact under `target/criterion/`;
release builds). Added an `operations-psalms` group (ingest ops on the
marker-dense profile target) alongside the existing Luke `operations` matrix.
Compare after WS2A with `--baseline ws2-pre`:

```sh
cargo bench --bench operations   -- --baseline ws2-pre
cargo bench --bench parallelism  -- --baseline ws2-pre
```

Release medians at pin:

| lane | median |
|---|---|
| `operations-psalms/parse/serial` (WS2 target) | 1210 µs |
| `operations-psalms/parse/string` (parallel) | 714 µs |
| `operations-psalms/lex/string` | 550 µs |
| `operations/parse/string` (Luke regression lane) | 300 µs |
| `operations/parse/serial` (Luke) | 300 µs |
| `parallelism/en_ulb/parse/serial` (corpus batch guard) | 10.2 ms |
| `parallelism/en_ulb/parse/rayon` (corpus batch guard) | 2.3 ms |

Retention gate targets the `operations-psalms/parse/serial` median (≥5% in all
three comparisons); Luke lanes + `parallelism/en_ulb/parse/*` are the ≤3%
regression guards.

**Hot-type sizes at pin** (`size_of`/`align_of`, release):

| type | size | align |
|---|---|---|
| `MarkerMetadata` | 24 | 8 |
| `MarkerToken` | 64 | 8 |
| `ScanToken` | 72 | 8 |
| `TokenData` | 96 | 8 |
| `Token` | 160 | 8 |

`Token` is 160 B (it embeds `TokenData` 96, which embeds `MarkerMetadata` 24 +
`StructuralMarkerInfo`). Context for WS2B/endgame: replacing the embedded
metadata/structural with a `u16` handle is the large-`memmove` prize, but is out
of WS2A scope. Do not add compile-time size asserts unless size becomes a
supported contract.

**Profile note:** the `parse/serial --book psalms` samply figures quoted earlier
(1.35 ms/iter etc.) are the `profiling` build — attribution only. Every quoted
speed number comes from the release Criterion medians above. Bucket percentages
were summed from **self**-time samples (each sample has exactly one self frame,
so summing self is a valid runtime allocation); do not sum inclusive/stack
percentages.

Next step: hand WS2A to the implementing agent against these pinned gates.

## 2026-07-21 — WS2A+WS2B landed and passed the gate (`ec117c8`)

WS2A (typed `ResolvedMarker` row, string-keyed) then WS2B (dense
`MarkerIndex(u16)` → static `ROWS`, resolve-once-in-lexer, drop the hashmap)
implemented by Sonnet agents, parent-verified, committed together as one
marker-resolution rework.

**Gate: PASSED.** Quiet Linux box (load ~0.05), release, WS2B vs the WS0 pin:

| lane | change | pin-vs-pin noise floor* |
|---|---|---|
| `operations-psalms/parse/serial` (**gate metric**) | **−10.7%** (p=0.00) | +2.4% |
| `operations/parse/serial` (Luke) | −10.2% | −0.2% |
| `parallelism/en_ulb/parse/serial` (corpus) | −9.4% | +0.2% |
| `parallelism/en_ulb/parse/rayon` (corpus) | −7.0% | +0.4% |

*The bench script's first pass (`--save-baseline pin`) reports change vs the
prior pin — i.e. identical code vs itself — so it doubles as a noise-floor
gauge. It showed the whole-corpus serial benches swing wide on identical code:
`format/string` ±14%, `chapter_grain/psalms/lint/serial` −10%, `lint/serial`
+9%, `lint/tokens` −8%. Read pass-2 changes against that floor.

- **WS2A alone was ~2–3%** (capped: it *introduced* a string-hashmap probe,
  `resolved_marker_for_canonical` ≈ 9.7% self-time). **WS2B removing that probe
  is what unlocked ~10%.** Before/after samply confirmed the probe went
  9.7% → 0.56% (array indexing).
- Bonus: usj/usx/cst/format-from-string improved 3–12% (they re-parse).
- **"Regressions" (`chapter_grain/psalms/lint/serial` +27%, `lint/tokens` +8%)
  are noise:** same benches swung −10%/−8% on identical code (pass 1),
  `lint_impl.rs` is untouched, `size_of::<Token>()` is unchanged (160 B — the
  index fit MarkerMetadata's padding), and the stabler
  `parallelism/en_ulb/lint/serial` was flat (+1.05%, p=0.45).
- Correctness: oracle byte-identical at 1 and full threads; full workspace +
  wasm green; `Token` 160 B unchanged. Semver: `MarkerMetadata` gained a
  private `index` field (external literal-construction break; approved).

Follow-ups queued (separate commits): (1) `FxHash` sweep — production code
still mixes default-hasher `HashMap`/`HashSet` (lint_impl, export_tree, markers,
diff/skeleton) with `FxHash*`; convert + add `clippy.toml`
`disallowed-types` guard; oracle-gate it (hash iteration order can touch
output). (2) WS3 — push delimiter-whitespace absorption into the lex phase,
riding the row's `absorbs_delimiter_whitespace` bit.

## 2026-07-21 — v0.0.9 final measure (released, quiet Linux box)

Full release (WS2 + box-attrs + FxHash) vs the WS0 pin (`727d4a3`), release
build, commit-based baseline (checkout pin → `--save-baseline pin` → checkout
release → `--baseline pin`). Second-pass (release-vs-pin) figures:

- **operations-psalms/parse/serial −16.6%** (headline; was −10.7% at WS2B — box
  attrs stacked ~6 more points via the smaller Token).
- operations-psalms/parse/string −20.3%; Luke parse/serial −13.5%, parse/string
  −12.6%; parallelism/en_ulb parse/serial −13.9%, parse/rayon −22.2%.
- Downstream lifted too (re-parse + denser tokens): cst −10%, lint/string
  −10.5%, chapter_grain/psalms/lint/serial −14.3%, format/string −5.6%, html
  −5.7%, diff/string −7.7%, usj/usx −5–6%, all parallelism ops −3 to −12%.

**KNOWN REGRESSION (shipped in 0.0.9): standalone `lex/string` +5.9% (Luke) /
+10.9% (Psalms).** Real (p=0.00, both books). Cause: WS2B added a second
lexer-side hash (`marker_index_by_canonical`) on top of `lookup_marker_metadata`
— one added lexer hash to remove two parser hashes. Net lex+parse is a big win
(hence parse/serial −16.6% despite this); only isolated lexing-without-parsing
pays it. **0.0.10 fix:** fold the index into `lookup_marker_metadata` (one lexer
lookup returning canonical + index), recovering lex while keeping the parse win.
Pairs with WS3 (deferred delimiter-whitespace-into-lex).

## 2026-07-21 — Item 1 (index fold) confirmed: lex regression recovered (`b5d03e9`)

`resolve_marker_metadata` now yields the MarkerIndex from its one existing
lookup per path (slow: index+spec from one probe via pointer-paired
exact_spec_index; fast: per-arm ordinal into a once-built [MarkerIndex; 48]).
Quiet box, Item 1 vs the v0.0.9 pin (`c341de5`):

- **operations-psalms/lex/string −11.5%**, operations/lex/string (Luke) −7.7%
  — erases the v0.0.9 standalone-lex regression (+10.9% / +5.9%).
- Parse improved further: Luke parse/serial −5.2%, psalms parse/serial −7.4%
  (lex is in the serial parse path). Combined with v0.0.9, psalms parse/serial
  is ~−22% vs the original WS0 baseline.
- Corpus parallel parse ~flat (already parallel-dominated); all non-marker ops
  no change. Oracle byte-identical both configs.

Alias collapse (fe/ef->f, ex->x) preserved and flagged for separate review
(fast path collapses; slow path does NOT — MARKER_SPECS has its own fe/ef/ex
rows — so the two disagree; see memory `project-marker-alias-collapse-unreviewed`).
