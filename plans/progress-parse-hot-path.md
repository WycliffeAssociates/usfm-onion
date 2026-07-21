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

Next step:

- Add and record the WS0 Psalms Criterion baseline, type sizes, and clean
  symbolicated profile before editing marker resolution.
