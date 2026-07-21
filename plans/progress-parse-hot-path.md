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
