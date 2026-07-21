# Plan — reduce serial parse marker-resolution cost

Status: in progress on branch `chapter-parallelism`.

Plan depth: standard. Interview depth: regular. Testing tolerance: hardened for
behavior preservation, standard for performance evidence.

Progress log: `plans/progress-parse-hot-path.md`.

## Problem

Chapter-parallel parsing has a real single-large-book win, but lexing and the
merge tail remain serial. After presizing the lexer and parser vectors, a
rayon-free `parse/serial` profile on Psalms promoted repeated marker-catalog
work to the largest addressable parse bucket.

The current marker path resolves overlapping facts independently:

1. The lexer calls `lookup_marker_metadata` while constructing each marker
   lexeme.
2. The lexer separately calls the cheap `marker_payload` predicate.
3. The parser calls `structural_marker_info` once for the matching marker arm
   (there are seven mutually exclusive call sites, not seven calls per marker).
4. When following whitespace arrives, the parser calls
   `lookup_marker_whitespace` through `delimiter_absorption`.

Although only one parser arm fires per token, the *same* marker is still
resolved independently up to three times across the pipeline: the lexer's
`lookup_marker_metadata`, the parser's `structural_marker_info` (which probes
the spec map again internally via `lookup_spec_marker`), and the parser's
`lookup_marker_whitespace` through `delimiter_absorption`. Each repeats
normalization, string classification, and a map probe. That cross-phase
repetition — not any single call site — is what WS2A collapses.

The likely win is eliminating that repeated normalization, string
classification, and map probing. The profile does **not** yet prove that
bit-packing the result, using a perfect hash, or carrying a dense ID through
public token types is worth its additional complexity.

## Solution

First introduce one typed, derived marker projection and use it to resolve all
hot facts together within each phase. Keep existing public lexeme and token
shapes intact. Measure that simpler consolidation before considering a dense
numeric handle across the lexer/parser boundary.

If lookup remains material after consolidation, a second gated workstream may
carry a private dense marker index between phases. That workstream must first
choose an explicit Rust API-compatibility strategy; the behavior oracle cannot
detect source-level API breakage.

## User stories

1. As a caller parsing one large marker-dense book, I want lower serial ingest
   cost so chapter-parallel parsing spends less time with workers idle.
2. As a wasm caller, I want the same cheaper serial path without requiring
   rayon or changing output.
3. As a maintainer, I want marker facts derived from the existing catalog so a
   performance cache cannot silently become a second source of truth.
4. As a Rust API caller, I want this optimization not to change public token or
   lexeme construction unless that breaking change is reviewed separately.
5. As a reviewer, I want correctness and performance claims tied to explicit
   gates so complexity that benchmarks within noise is removed.

## Current evidence and measurement boundary

- `parse/serial` is the profiling lane: `lex` plus `parse_lexemes`, with rayon
  absent from the call stack. Samply percentages are attribution evidence only.
- Criterion is the timing authority. `benches/operations.rs` currently runs
  only Luke, while the motivating profile is Psalms. Add a Psalms
  `parse/serial` Criterion lane before changing marker resolution so the
  before/after benchmark measures the actual target workload.
- Keep Luke as the smaller/less marker-dense regression lane and the existing
  `parallelism/en_ulb/parse/{serial,rayon}` pair as the batch guard.
- Apple P/E-core scheduling made earlier single-thread ratios noisy. Compare
  saved before/after baselines under the same idle conditions, repeat the
  comparison three times, and report Criterion medians/intervals rather than a
  best run.
- Profiling builds identify where time moved. Release Criterion builds supply
  every quoted speed number.

### Proposed retention gate

Keep a workstream only when all are true:

- byte-identical oracle output at one and full worker counts;
- the targeted Samply bucket visibly shrinks or disappears;
- the Psalms `parse/serial` Criterion median improves by at least 5% in all
  three before/after comparisons; and
- Luke and whole-corpus parse show no repeatable regression above 3%.

The 5%/3% thresholds are recommended defaults, not a product requirement. If
Will wants a different complexity budget, decide it before WS2 begins and
record it in the progress log.

## Goals

- Remove redundant marker normalization/classification from serial parse.
- Preserve serialized token/CST/export/lint behavior exactly.
- Keep one hand-edited marker source of truth.
- Make each optimization independently revertible and benchmarkable.

## Non-goals

- Replacing Logos or parallelizing the lexer.
- Replacing the public string-backed `MarkerId`.
- Shrinking final `TokenData` in this plan.
- Optimizing lint/context lookup before an ID is intentionally carried on final
  tokens.
- Adding a generated or hand-maintained perfect-hash table.
- Reworking formatter whitespace semantics.

## Constraints and corrected assumptions

### The existing catalog is more than two tables

`MARKER_SPECS` and explicit `MARKER_WHITESPACE` rows are hand-curated inputs,
but several facts are derived by code: default whitespace by marker kind and
paragraph category, marker payload, family, block/closing behavior, note
context, aliases, numbered variants, table-cell variants, nested markers, and
milestone `-s`/`-e` forms. A row is valid only if it agrees with these existing
predicates; deriving from the two slices alone is insufficient.

### Canonical-row facts and occurrence facts are different

A canonical row may contain kind, structural info, payload, default whitespace
absorption, deprecation, and a context mask. It must not contain facts that can
vary for two raw spellings resolving to the same canonical marker:

- nested (`+` prefix);
- milestone start versus end;
- family role such as alias or numbered variant; or
- the raw marker spelling itself.

Those remain occurrence-level facts computed from the raw spelling.

### A packed `u64` is not the first milestone

The prior plan estimated a roughly 1.9 KiB packed table. A normal typed row
table is also small enough to be cache-resident, is easier to review, and does
not require relying on Rust enum discriminants. Start typed and record
`size_of::<MarkerRow>()` plus total table bytes. Pack only if a profile shows
row footprint or field loads remain material.

If packing is later justified, use explicit encode/decode functions and
explicit context-bit mappings. Do not cast enums with `as u32` unless their
numeric representation and ordering become an intentional tested contract.

### A dense index is private and distinct from `MarkerId`

If needed, introduce a private `MarkerIndex(u16)` with an explicit unknown
representation and a checked construction path. The existing public
`MarkerId(&'static str)` is a semantic/catalog API and is not repurposed.

### The oracle does not protect Rust source compatibility

`MarkerToken`, `ScanToken`, `ScanResult`, `TokenData`, and their fields are
public. Adding a private field, removing metadata, or adding a new public enum
field can break downstream struct construction or exhaustive matching even
when serialized output is identical. Any cross-phase dense-handle design must
either preserve those shapes or be approved as a separate breaking change.

## Workstream 0 — establish the gate

1. Add a Criterion lane for Psalms `parse/serial` using the exact shared
   `run_named_op` body used by `profile_ops`.
2. Save the pre-WS2 baseline for Psalms, Luke, and corpus parse.
3. Record current `size_of` values for `MarkerMetadata`, `MarkerToken`,
   `ScanToken`, `TokenData`, and `Token` in the progress log. Do not make these
   brittle compile-time assertions unless size itself becomes a supported
   contract.
4. Capture one symbolicated `parse/serial --book psalms` profile and record
   whether percentages are self-time or inclusive time. Avoid summing
   overlapping stack percentages.

Verification gate: the Criterion body and profiling body are identical, the
baseline commands/results are in the progress log, and no product code changed.

## Workstream 1 — vector presizing (landed in `8d4b848`)

- `parse_lexemes_seeded` uses `Vec::with_capacity(lexemes.len())`.
- `lex` uses the measured `source.len() / 6` heuristic.
- Together they removed the realloc/memmove ladder and improved the profiling
  build's Psalms serial wall time by roughly 35%.

The earlier draft incorrectly left lexer presizing as a future WS1b. Both
presizing changes are already landed. Do not repeat or reorder them around WS2.

Follow-up only if later evidence requires it: the parser capacity is an upper
bound and can over-allocate on `\\w`/`\\zaln`-dense data. Do not
`shrink_to_fit`; measure peak-memory impact separately before changing it.

## Workstream 2A — one typed, derived resolution per phase

### Projection

Add a private typed `MarkerRow`/`ResolvedMarker` API in `marker_defs` containing
only facts consumed by the current lex/parse hot path:

- canonical marker and `MarkerDefKind`;
- marker family/metadata;
- `StructuralMarkerInfo`;
- `MarkerPayload`;
- whether the opening form absorbs its following delimiter whitespace.

Build the projection from the existing canonical spec plus existing derivation
functions. An explicit whitespace row must override the category-derived
default exactly as `lookup_marker_whitespace` does today.

Do not add context masks, closing behavior, paragraph flags, or catalog-only
fields merely because they fit. They are outside the measured parse path.

Note for a future, separate lint-focused workstream (not WS2A): context
validity is the one place a bitmask is a genuine *algorithmic* win — a `u32`
bit-test on the typed row replacing lint's `marker_allows_context`
`&[SpecContext]` slice scan. It rides on a plain `u32` field of the typed row
and needs no bit-packing. Keep it out of the parse projection until lint
lookup is independently profiled and shown hot.

### Resolution

- Resolve/normalize a raw marker once within the lexer and use that result for
  both `MarkerMetadata` and pending payload.
- In the parser, replace separate structural and whitespace lookups with one
  resolution that supplies both facts. The seven `structural_marker_info` call
  sites are mutually exclusive match arms; consolidate their construction,
  but do not describe them as seven probes per occurrence.
- Keep unknown-marker behavior identical.
- Keep public `MarkerToken`, `ScanToken`, `ScanResult`, `TokenData`, and
  serialized fields unchanged in WS2A.
- Remove or narrow `fast_marker_metadata` only when the derived resolver covers
  its cases and parity tests prove that aliases such as `fe`/`ef` retain the
  current canonical metadata. Do not leave two competing hot-marker lists.

This phase may still perform one resolution in the lexer and one in the parser.
That is intentional: it captures the low-risk consolidation win without first
changing a public boundary.

### Drift/parity tests

Test behavior, not the chosen bit layout:

1. For every canonical `MARKER_SPECS` row, compare the projection with current
   metadata, structural, payload, and effective whitespace predicates.
2. Exercise normalization families explicitly: nested `+` markers, numbered
   paragraph variants, numbered/hyphenated table cells, milestone `-s`/`-e`,
   and the `esbe` alias.
3. Extract every distinct raw marker spelling in `testData`/example corpora and
   assert old/new resolution parity during the migration.
4. Include unknown and malformed marker names so the optimization does not
   accidentally turn them into known markers.

Verification gate:

- focused marker-definition and lexer/parser tests pass;
- full library/workspace tests pass;
- wasm target checks;
- oracle passes unchanged at one and full worker counts (never `BLESS=1`);
- retention gate passes; and
- a new serial profile confirms what displaced the marker-resolution bucket.

If the retention gate fails, revert WS2A rather than proceeding to a denser
representation.

## Workstream 2B — carry a dense handle across phases (conditional)

Attempt only if WS2A passes but marker resolution remains a material profile
bucket.

API-compatibility decision (Will, resolved): a semver-breaking token/lexeme
shape change **is acceptable this cycle** — `master` already carries a breaking
diff change and the editor consumer will be updated alongside. So WS2B is not
gated on preserving public shapes; choose the mechanism on engineering merit:

1. Carry a private dense handle (e.g. `MarkerIndex(u16)`) on the lexeme/token
   where it reads cleanest. Serialized output must stay byte-identical (the
   oracle still gates that); source-level breakage is allowed and must be noted
   in the changelog so the editor is updated in lockstep.
2. A parallel sidecar `Vec<MarkerIndex>` keyed by token position is the
   alternative — worth it only to avoid inflating the token enum's size/
   alignment (a `u16` field that bumps the enum can cost more bandwidth than the
   lookup it saves). This is a *memory-layout* tradeoff now, not a semver one.
3. Stop: accept one lexer and one parser resolution because the remaining cost
   does not justify any wider representation change.

Measure sidecar allocation/cache cost against the lookup it replaces before
choosing it. Do not claim lint can read `ROWS[id]` until final lintable tokens
actually carry a stable handle.

If implemented, measure the sizes from Workstream 0 again. A `u16` field that
increases the enclosing enum due to alignment may cost more memory bandwidth
than it saves in lookup time.

Verification gate: the same correctness gates as WS2A, explicit downstream API
compatibility evidence, and a second independent retention decision.

## Workstream 3 — lexer-owned delimiter whitespace (optional, separate)

This removes parser state; it is not assumed to be the main performance win.
Only start after WS2 is measured and only if simplifying `pending_ws` is itself
worth the behavior risk.

The lexer may absorb delimiter whitespace immediately after a resolved marker,
book code, or chapter/verse number, but must preserve:

- only the first whitespace character is absorbed;
- the remainder remains pending/separate exactly as today;
- number absorption occurs only when `cv_number == JustEmitted`;
- book-code absorption remains unconditional in that contextual position;
- non-absorbed pending whitespace extends the prior token span;
- adjacent text merging and empty-document behavior; and
- exact source spans at chunk boundaries.

Treat WS3 as its own commit and benchmark. If it simplifies code without a
timing win, review it on maintainability alone rather than laundering it as a
performance result.

## Verification commands

The implementing agent must record the exact commands supported by the current
toolchain in the progress log. The intended lanes are:

```sh
cargo test --workspace
cargo test --test lint_oracle -- --ignored
RAYON_NUM_THREADS=1 cargo test --test lint_oracle -- --ignored
cargo check --workspace --target wasm32-unknown-unknown
cargo bench --bench operations
cargo bench --bench parallelism
```

If the oracle test's actual ignored-test filter differs, discover it from the
test binary and update this plan/progress log rather than copying a stale
command. Never run `BLESS=1` for these representation-only changes.

## Sequencing and stop conditions

1. WS0 benchmark/profile baseline.
2. WS2A typed projection and per-phase consolidation.
3. Correctness gate, Criterion comparison, then serial re-profile.
4. Stop if WS2A fails the retention gate or removes the material bucket.
5. WS2B only if lookup remains material and the API strategy is explicitly
   chosen.
6. WS3 only as a separately justified simplification.

At every gate, prefer the smaller landed design. A `u64`, perfect hash, dense
ID on final tokens, or lexer-owned whitespace is not success unless its own
measurement pays for its own complexity.

## Risks and rollback

- **Derived-data drift:** parity tests compare observable predicates over base
  rows and normalization variants.
- **Alias/variant collapse:** keep occurrence facts separate and test `+`,
  numbered, table-cell, milestone, and alias forms.
- **Public Rust API break:** WS2A forbids public shape changes; WS2B may break
  source (approved this cycle — changelog it so the editor updates in lockstep)
  but must keep serialized output byte-identical.
- **Enum/table packing bugs:** typed rows first; packing is conditional and uses
  explicit encoders if ever justified.
- **Benchmark noise:** repeated Criterion comparisons and profile attribution,
  not single-run ratios.
- **Memory regression:** record enclosing type sizes and reject handle-carrying
  designs that inflate hot vectors without a compensating measured win.
- **Rollback:** each workstream is a separate commit and must leave the oracle
  green, so a failed retention gate is reverted without disturbing the landed
  chapter-parallel work.

## Decisions still requiring Will

1. Confirm or replace the proposed 5% win / 3% regression retention thresholds.
2. ~~If WS2A leaves lookup hot, approve a source-compatible or breaking
   dense-handle change.~~ **Resolved:** a semver-breaking token/lexeme change is
   acceptable this cycle (editor updated alongside; `master` already breaks via
   diff). WS2B chooses its mechanism on engineering merit; serialized output
   still must stay byte-identical.
3. Treat WS3 as optional maintainability work, or explicitly make removal of
   parser whitespace state a goal independent of performance.
