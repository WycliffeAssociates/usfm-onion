# Candidate — trim `LintIssue` (and friends) memory width

Date: 2026-07-27. Status: candidate — parked during the braid epic; revisit after Phase B/C land
(the wire finding codec and braid residency change the cost picture anyway).

## Observation

Nightly `-Zprint-type-sizes` over the release lib (`lib_sizes.txt`, 2026-07-27, repo root at the
time; regenerate with `touch src/lib.rs && cargo +nightly rustc --release --lib -- -Zprint-type-sizes`):

| bytes | type |
| ---: | --- |
| 320 | `LintIssue` |
| 192 | `DecisionUnit<Token>` / `DecisionUnit<FormatToken>` |
| 152 | `UsjNode` / `UsjElement` |
| 136 | `FormatToken` |
| 128 | `TokenFix` / `FormatFix` |
| 120 | `Token<'_>` |

`LintIssue` is the bulk-moved type: en_ult produces ~46k findings (~14 MB as a `Vec<LintIssue>`),
and `canonical_sort` stable-sorts them **by value** (the dump shows
`AlignedStorage<LintIssue, 4096>` sort scratch buffers). A large share of the 320 B is inline
rare/optional payload — notably `Option<TokenFix>` (128 B) carried by every issue when only some
codes produce fixes, plus message-param storage.

## Candidate moves (unmeasured — hypotheses only)

- `Box<TokenFix>` (or `Option<Box<TokenFix>>`) — likely halves the row for the common no-fix case.
- Box or intern message params / related-anchor payload similarly.
- Same lens on `DecisionUnit`/`UsjNode` if diff/USJ heaps ever matter.
- `Token<'_>` at 120 B is the hot parse type: leave alone absent a measured win.

## Rules of engagement

Per the owner's spike-first policy (`feedback-spike-perf-hypotheses-first`): prove any win in a
throwaway spike with before/after peak-heap and lint-throughput numbers before touching
production types. Boxing changes `LintIssue`'s public shape only if fields go private/accessor —
check the wasm DTO conversion cost too (it clones anyway). Not part of the braid epic; do not fold
into a braid phase.
