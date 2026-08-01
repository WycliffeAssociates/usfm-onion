# Progress — braid epic

Append-only companion to `braid-epic.md`. This records review/execution evidence and deviations;
the plan remains normative.

## 2026-07-23 — exhaustive plan review

- Status: plan rewritten; no production code changed; Gate 0 not run.
- Planning dials: exhaustive, ruthless interview, standard testing.
- Execution base observed: `dc647c0`; working tree already contained unrelated deletions and
  untracked plan/worktree material, all preserved.
- Read first: owner handoff attachment and the prior normative braid epic.
- Compared structure with the 2,211-line sous-chef granularity-spine plan.
- Structural inspection used the current CodeGraph index plus focused reads where the index mixed
  preserved worktree copies into unqualified symbol results.
- Live-core finding: current lint keeps structure, duplicate-chapter, and number/verse rule
  families whole-book. Discarded v1 dirty-chapter lint compute; recommended dirty-book recompute.
- Live-wire finding: the spike includes `S_SOURCE`, while the prior normative token layout omitted
  source bytes. Added mandatory self-contained source field.
- Live-consumer finding: editor findings retain message params and full `TokenFix`; rejected the
  claim that all variable payload can be re-derived from rule code/token alone. Added sidecars and
  a per-code payload ledger gate.
- Live-identity finding: ordinary editor token ids are GUID-backed, but synthesized linebreak ids
  restart per chapter. Added book-wide uniqueness validation and an editor migration precondition.
- Type-boundary finding: `usfm_onion_dto` plus wasm-local types currently duplicate the boundary.
  Fixed the target DAG and selected absorption into `usfm_onion_wire` rather than “wire or sibling.”
- SID finding: stealing the high bit caps exact packed bridge delta at 127, not 255. Kept core Sid
  semantics unchanged and made fidelity an explicit packed-wire conversion.
- Open owner adjudication: accept or reject the amendments in plan §2.2 before Phase A.
- Verification: documentation diff inspection remains; no tests run because this turn changed
  planning artifacts only.
- Next: review the rewritten contracts, then execute Gate 0 evidence collection if approved.

## 2026-07-23 — owner adjudication: defer chapter-grain lint

- Owner confirmed dirty-book relint is acceptable for braid v1 because the editor currently
  relints whole books and chapter-grain lint is still an editor TODO.
- Preserved the shared Galley/Braid pluggable direction as
  `../candidates/chapter-grain-braid-lint.md`, with explicit chapter-map, ordered-book-reduce,
  and whole-book-batch lanes.
- No production code changed and no tests run; documentation-only follow-up.

## 2026-07-23 — owner adjudication: external source, flat patches, rule hierarchy

- Restored the intended binary-token contract: exact external UTF-8 USFM bytes plus a packed
  parse-metadata sidecar reconstruct borrowed tokens without lex/parse. Mandatory embedded source
  bytes were rejected for v1.
- Added `../candidates/embedded-source-corpus-bundle.md` for an optional one-read artifact that
  packages the same source bytes and sidecars together.
- Recorded a braid-owned, snapshot-bound flat patch table: core keeps generating today's
  `TokenFix`; braid resolves it into reverse-applied insert/replace/delete edits and owns safe
  resident application. No tree representation or new dependency is implied.
- Added `../candidates/editor-persistent-linebreak-token-id.md`; implementation belongs in the
  editor and should first investigate persistent Lexical NodeState on built-in linebreak nodes.
- Reworded the chapter-lint candidate as a closed internal rule-grain hierarchy rather than a
  plugin system.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: baseline, compatibility, and existing diff shape

- No dedicated source-key rename operation will be added in v1. A rare path move is handled by
  the next corpus seed/replacement and does not dirty matching semantic cache content.
- Fixed missing-baseline behavior: `is_dirty` is true, while `diff_baseline` returns typed
  `MissingBaseline` listing every requested book without a baseline.
- Confirmed one deliberate breaking pre-1.0 crate/npm release. Crate names, wire types, and
  generated declarations may change; no duplicate DTO compatibility layer is required.
- Replaced the undefined Braid `DiffSnapshot` placeholder with the live core type:
  `ScopedOutput<DiffSkeleton<OwnedToken>>`. The TypeScript boundary returns the corresponding
  existing `DiffSkeleton<Token>` shape; packed diff remains outside v1.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: resident `SourceKey`, portable corpus identity

- Superseded the preceding rejection of all caller keys with a narrower split: every resident
  book requires a unique opaque `SourceKey` (normally a path), but the key is neither semantic
  identity nor portable wire data.
- Grouped resident projections use `SourceKey`; packed token/finding sections and JS reconcile use
  unique `BookId`, exact source hash, and semantic stamps. The current manifest rebinds source keys
  when reopening, so file moves do not invalidate matching cache content and another application
  may use different paths with the same binary.
- Clarified the duplicate-book boundary: individual equal files/per-book sidecars remain valid,
  but resident corpus formation and complete-corpus packing reject duplicate `BookId`s. Decoders
  also reject duplicate-book TOCs as non-canonical.
- `DuplicateBook { book, sources }` is a typed corpus-validation diagnostic presented before
  resident lint, since an ambiguous corpus cannot be installed merely to emit a token finding.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: resident versus stateless API

- Reclassified the embedded-source whole-corpus artifact from candidate to
  `../defferred/embedded-source-corpus-bundle.md`; external source bytes will normally already
  be available and remain the first use case.
- Made Braid receiver semantics explicit: methods operate only on the ingested resident corpus;
  top-level/core/package functions accept arbitrary external tokens. Rejected an external-token
  sentinel on `CorpusScope`.
- Added resident `prepare_format_patch(All | Book | Chapter, options)` as a non-mutating,
  snapshot-bound patch preparation step. Existing external `format(tokens, options)` remains a
  stateless core/package operation.
- Generalized the flat patch table to an atomic per-book edit set so one prepared format patch can
  safely cover a whole resident corpus without introducing nested token structure.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: Braid is the resident handle

- Confirmed `Braid` as the stateful resident handle type; lowercase `braid` remains the Rust
  crate/module or package namespace. No separate `Ristra` type is introduced without a distinct
  domain concept.
- Instance methods always use ingested resident state. Namespace/top-level/static shallow proxies
  accept arbitrary external USFM/tokens and may delegate directly to core.
- Pairwise and N-ary diff over arbitrary inputs are stateless operations, not resident Braid
  instance methods; resident diff remains explicitly baseline/scope-oriented.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: semantic types, cache priming, and grouped output

- Confirmed core ownership of canonical semantic `OwnedToken`; wire owns boundary DTOs/codecs,
  braid and wire are sibling core consumers, and wasm composes them.
- Confirmed unique resident `BookId`s. Differently named files with the same parsed `\\id` reject
  rather than receiving occurrence identities.
- Added semantic per-book lint-cache priming. The adapter validates wire/catalog compatibility;
  Braid validates `BookId`, exact source hash, lint-config fingerprint, and deterministic engine
  stamp. Rejected or missing entries remain dirty for ordinary whole-book lint.
- Fixed the JS reconciliation signature as
  `reconcileFindings(previous, bytes, tokensByBook)` and made expected wasm failures typed tagged
  results rather than thrown string errors.
- Applied the same single-versus-grouped result envelope to resident USFM, USX, USJ, and HTML
  projections.
- Rejected a required path-derived `DocumentKey` after follow-up review. Paths and manifest ids
  remain application-owned; Braid/container grouped output is keyed by unique `BookId`, while
  `BookId + source hash + semantic stamps` validates cached content. File moves therefore do not
  invalidate otherwise matching cache entries.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: cold JS materialization

- Superseded the preceding overly broad statement about caller keys: resident `SourceKey` is
  required and used for grouped application output, while remaining absent from portable wire
  identity.
- Defined cold-open `materialize(sources, packed)`, where `sources` maps each current `SourceKey`
  to explicit `{ book: BookCode, bytes: Uint8Array }`. The explicit book binding avoids reparsing
  `\\id`; source length/hash validation binds the external bytes to each portable section.
- Kept `reconcileFindings(previous, packed, tokensByBook)` as the distinct warm-update path when
  the editor already owns token objects.
- Added cold-materialization portability, binding, mismatch, and failure-atomicity tests.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: BookCode-keyed materialization and decode views

- Refined cold JS input from source-key/path binding to portable
  `ReadonlyMap<BookCode, Uint8Array>` with an optional validated Record-shaped overload. Paths
  remain solely in the application's manifest.
- Confirmed each corpus TOC entry and each section header carries the canonical three-byte book
  id. Complete-corpus encoding and decoding reject duplicate book sections.
- Renamed the low-level JS operation to `decodeView(packed)`: it validates packed structure and
  exposes read-only section/typed-array views but cannot construct source-slicing semantic tokens.
  `materialize(sourcesByBook, packed)` is the ordinary cold object-creation API.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner adjudication: declared header BookId versus source `\\id`

- Corrected the earlier assumption that parsed source `\\id` must define/reject corpus identity.
  The caller/manifest declares the unique canonical `BookId`; that value is stored as the
  three-byte TOC/section id and keys `materialize` source input.
- Source `\\id` remains editable content. A temporary valid mismatch stays resident and packable
  under the declared book, while a core-owned `BookIdMismatch` lint rule reports expected/found.
  Invalid codes similarly remain the responsibility of the separately filed core lint rule.
- Duplicate declared manifest book ids still reject corpus formation/packing, and duplicate book
  ids in a packed TOC remain corrupt/non-canonical. Duplicate source-content `\\id` tokens across
  distinct declared books do not make wire addressing ambiguous.
- Clarified `decodeView(packed)` as low-level validated typed-array/section access versus
  `materialize(sourcesByBook, packed)` as ordinary cold creation of semantic JS objects.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — owner architecture adjudication complete

- Confirmed `materialize(sourcesByBook, packed)` is a thin semantic-object layer over the same
  authoritative `decodeView(packed)` validation and typed views, plus source binding, slice
  recovery, object construction, and grouping. There is no second binary parser.
- Marked the normative plan's owner-level architecture as adjudicated. Gate 0 evidence, public
  contract verification, fixed-width measurements, and the separately blessed declared-book lint
  prerequisite are next; no further owner choice is currently open.
- No production code changed and no tests run; documentation-only adjudication.

## 2026-07-23 — final signature consistency pass

- Found and corrected one non-owner inconsistency: JS helper prose required typed failures while
  `decodeView`, `decodeTokens`, `materialize`, and `reconcileFindings` still showed bare success
  returns. All now return generated `ApiResult` unions with explicit decode, source-binding, or
  token-binding errors; `groupByBook` consumes an already validated decoded container.
- Corrected the ergonomic object overload to
  `Readonly<Partial<Record<BookCode, Uint8Array>>>`, since a full `Record` over a closed book-code
  union would incorrectly require every canonical book.
- No production code changed and no tests run; documentation-only consistency repair.

## 2026-07-23 — pre-spike exploration protocol expansion

- Expanded Gate 0 into ordered stages 0A–0H: provenance, behavioral baseline, public-contract
  census, semantic payload/ordering census, corpus-width study, editor consumer/lifecycle study,
  dependency/feature feasibility, and a frozen spike charter followed by unchanged replay.
- Added exact baseline commands and prohibited `BLESS=1`/`UPDATE_GOLDEN=1`; corrected the plan's
  stale description of `tests/lint_oracle.rs` as ignored.
- Defined durable environment, API, payload, width, editor-contract, dependency, and spike-result
  ledgers, with explicit stop conditions before production crate/type work begins.
- Distinguished the preserved historical spike from the normative design: its embedded `SOURCE`
  section and private/raw catalog assumptions must be recorded as contract deltas, then separately
  reproven for external source bytes, declared three-byte book ids, checked directories, and stable
  catalog identity before any spike code is promoted.
- No spike, tests, or production changes were run; documentation-only plan hardening.

## 2026-07-27 — Gate 0A/0B: provenance and behavioral baseline

Evidence collection only, per §3.1 0A/0B. No `BLESS=1`/`UPDATE_GOLDEN=1`, no resets/cleans, no
production or golden edits. Raw logs/hash lists retained under scratch, not the repo.

### 0A — provenance ledger

- Execution base: `git rev-parse HEAD` = `c22caa95a48902d91688919afe79dbde76bb3c6d` (branch `braid`).
- Pre-existing dirty paths (present before any Gate 0 command ran): ` D plans/approved/braid-epic.md`,
  ` D plans/approved/progress-braid-epic.md` (both superseded by the `plans/approved/braid/` split),
  `?? .claude/`, `?? bench-remote.sh`, `?? plans/approved/braid/`. None of these are Gate 0 output.
- Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)` aarch64-apple-darwin, LLVM 22.1.2;
  `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`; `node v24.4.1`; `npm 11.4.2`; `wasm-pack 0.14.0`
  (a newer 0.15.0 is available upstream, not installed — not a blocker, just noted);
  `wasm-opt version 131`.
- `cargo metadata --no-deps --format-version 1`: sha256
  `a57212542add63568407ab3e9bcd33f9c7baf497aa1cf756300d42f9b9a7baac`. Workspace members: `usfm_onion`
  0.0.9 (feature `default`), `usfm_onion_dto` 0.0.9 (features `default`, `wasm`), `usfm_onion_wasm`
  0.0.9 (no features). Three members, matching the plan's pre-migration crate count (core + dto +
  wasm; `braid`/`wire` do not exist yet).
- Worktree ledger (`git worktree list --porcelain`, each independently `rev-parse`/`status --short`'d):
  - primary `usfm_onion` — `c22caa9` (branch `braid`), dirty paths as above.
  - `usfm_onion-spike` (branch `spike-scratch`) — `c341de5`, untracked-only:
    `examples/spike_chapter_{exports,lint,parse}.rs`, `examples/spike_parallel_lex.rs`.
  - `.claude/worktrees/agent-a2fd6de7683db6f87` (branch `worktree-agent-a2fd6de7683db6f87`) — `c341de5`,
    modified `src/lexer.rs`, `src/lib.rs`; untracked `examples/spike_simd_lex*.rs`,
    `examples/spike_text_stats.rs`, `src/lexer_spike.rs`.
  - `.claude/worktrees/agent-af68c779deab4e90a` (branch `worktree-agent-af68c779deab4e90a`, the
    §3.1 0H replay target) — `c341de5`, modified `src/lib.rs`, `src/marker_defs.rs`; untracked
    `GEN.bin`, `GEN.json`, `PSA.bin`, `PSA.json`, `examples/wire_probe.rs`, `examples/wire_spike.rs`,
    `js/spike_b.mjs`, `js/wasm_vs_bin.mjs`, `js/wire_decode.mjs`, `src/wire_spike.rs`. Left completely
    untouched this session — not cleaned, reset, or replayed (0H is out of scope for this run).
  - `.claude/worktrees/agent-af89ce6620a45cfc0` — `bc6925c`, clean.
  - `.claude/worktrees/hygiene-master` (branch `hygiene-batch-1-master`) — `dc647c0`, clean.
  - `.claude/worktrees/oracle-fix` (branch `fix-oracle-crlf`) — `4a2afd4`, clean.
- Fixture/corpus input hashes (sorted per-file sha256 list, rolled up to one sha256 of that list):
  - `example-corpora/en_ult`: 78 files, rollup sha256
    `dae6682c6286d1d77938146e550d85550ef8bf931de0e5aeca9b4500b5ed693a`.
  - `example-corpora/en_ulb` (adjudicated real-corpus candidate for later Gate 0E): 74 files, rollup
    sha256 `b3d06c690cea12fe537dccb6051c3e659c4408e51f551c63318b18ebdaa02b6b`.
  - `testData/**/*.usfm`: 262 files, rollup sha256
    `2e00e92ca401542d93b757bac5034fbbc66e2a053096f2e99ebacd19a35626fc`.

### 0B — behavioral baseline

All commands run from repo root, no bless/update env vars set (verified `UPDATE_GOLDEN`/`BLESS`
unset before the golden runs).

| command | result | duration |
| --- | --- | --- |
| `cargo test --workspace` | green — 242 passed, 0 failed, 12 ignored (plus zero-test doctest crates) | 14s |
| `cargo test --test lint_oracle` | green as invoked, but the single test is `#[ignore]`d (0 run, 1 ignored) | 5s |
| `cargo test --test lint_oracle -- --ignored` (supplemental, see anomaly below) | green — 1 passed | 5s |
| `npm run check:wasm:web` | green | 9s |
| `npm run test:wasm` | green (bundler + web smoke + token-SID conformance, 3 fixture streams each) | 13s |
| `npm run golden:wasm` | green — golden parity passed, 7 fixtures | 4s |
| `npm run golden:wasm:web` | green — golden parity passed, 7 fixtures | 3s |
| `npm run test:token-sids:import` | green (silent exit 0) | 1s |

- **Anomaly, flagged not fixed:** `tests/lint_oracle.rs`'s own module doc says plain
  `cargo test --test lint_oracle` asserts against the baseline, and the 2026-07-23 entry above
  claims this stale-"ignored" description was already corrected in the plan text — but the test
  function is still annotated `#[ignore = "exhaustive corpus gate; run with \`cargo test --
  --ignored\`..."]` in the checked-out source. As literally specified, the command is a no-op
  (0 run/1 ignored) and would read as trivially "green" even if the oracle were broken. Ran the
  exhaustive form (`-- --ignored`) as supplemental evidence to actually exercise the assertion: it
  passes (1/1). No source file was touched to reconcile the doc/attribute mismatch — that is a
  separate fix, out of scope for evidence-only Gate 0.
- **Setup, not substitution:** `node_modules` was absent and no `package-lock.json` was tracked, so
  `npm ci` failed immediately (`npm error ... Clean install a project` requires an existing
  lockfile). Ran plain `npm install` once instead (root package declares zero dependencies, so this
  only audited/no-op'd and produced a new `package-lock.json`; no `node_modules` directory was
  actually created because there is nothing to install). This is recorded as setup, matching the
  hard-rule carve-out for missing `node_modules`.
- Artifact hashes:
  - `tests/lint_oracle_baseline.txt`: sha256 `e5062b04ecda29766abbcbeb3ea1561c19d00f84ff65b828a8c5a4d58c200dab`.
  - Generated wasm `.d.ts` trees (post-build, this session): `pkg-bundler/**/*.d.ts` — 2 files,
    rollup sha256 `ac733acea546ef10419562c365d7141e35c13d544a6dac24d139d07b96978bb8`;
    `pkg-web/**/*.d.ts` — 2 files, rollup sha256
    `947d280d25921406c6fbf2a7fd6e7f0658f1ee134df4da8b004540e4fade6475`.
  - Representative golden outputs (of 117 files under `crates/usfm_onion_wasm/golden/outputs/`):
    `tiny/lint.json` sha256 `c6a214fdba5260265e356cad7ecd57949e1fd813ca4b3756a04225db96bd3675`;
    `attributes/to-usfm.usfm` sha256 `57d86c50b5a56270d8168afebe7d78ac769aa86c35c55c3617de6b2622717465`.

### Post-run tracked-file drift — flagged loudly, NOT reverted

`git status --short` after all 0B commands shows **tracked files changed** relative to the 0A
snapshot, entirely inside the wasm-pack build output that the `npm run` scripts regenerate in
place:

```text
 M pkg-bundler/usfm_onion_web.js
 M pkg-bundler/usfm_onion_web_bg.js
 M pkg-bundler/usfm_onion_web_bg.wasm       (Bin 1,055,469 -> 4,397,212 bytes)
 M pkg-bundler/usfm_onion_web_bg.wasm.d.ts
 M pkg-web/usfm_onion_web.d.ts
 M pkg-web/usfm_onion_web.js
 M pkg-web/usfm_onion_web_bg.wasm           (Bin 1,055,469 -> 4,397,212 bytes)
 M pkg-web/usfm_onion_web_bg.wasm.d.ts
?? package-lock.json                        (new, from npm install; see setup note above)
```

The committed `pkg-bundler`/`pkg-web` trees predate this toolchain: rebuilding with the currently
installed `wasm-pack 0.14.0`/wasm-bindgen produces a materially different bindgen ABI shape (e.g.
`.d.ts` signatures move from `(a,b,c,d) => void` output-pointer style to `(a,b,c) => [number, number]`
tuple-return style) and a ~4x larger unoptimized `--dev` `.wasm` versus whatever produced the
committed (presumably release/optimized) binary. This is a genuine consequence of running the exact
`npm run` commands named in §3.1 0B, which invoke `build:wasm:*:dev` as a prerequisite — not an
edit made to satisfy this task. Per the hard rule against reverting working-tree state, these
changes were left exactly as produced; they are not committed, staged, or cleaned. This checked-in
staleness/toolchain-drift between the committed wasm package artifacts and the current build
environment should be adjudicated separately before Gate 0C+ treats those `.d.ts` files as a source
of truth for the API/ownership census — regenerate-and-diff, not trust-as-committed.
`plans/approved/braid-epic.md` and `plans/approved/progress-braid-epic.md` remain deleted exactly
as they were pre-existing (superseded by this `plans/approved/braid/` pair); unrelated to this run.

### Baseline verdict

**Green.** Every 0A/0B command specified in §3.1 passed under the toolchain and inputs recorded
above. Two items need owner attention before later gates lean on them: (1) the `lint_oracle`
ignore/doc mismatch (test only truly exercises the oracle under `-- --ignored`), and (2) the
wasm-pack/toolchain drift that makes rebuilding the committed `pkg-bundler`/`pkg-web` trees produce
a different binding ABI shape and much larger `.wasm` than what's currently committed. Neither
blocked evidence collection; both are recorded here rather than silently worked around.

## 2026-07-27 — owner adjudication: 0A/0B follow-ups (pkg restore, oracle command amendment)

- Restored the eight tracked `pkg-bundler`/`pkg-web` files regenerated by the 0B npm scripts to
  their committed state and removed the incidental `package-lock.json`. The committed trees from
  `3ee81f0` remain canonical; the ~4x `.wasm` growth was the `--dev` build profile, not staleness.
  The `.d.ts` bindgen shape difference (output-pointer vs tuple-return) still suggests a
  wasm-bindgen version delta between the committed artifacts and the local toolchain — Gate 0C must
  regenerate-and-diff declarations deliberately rather than trusting either tree blindly.
- Plan amendment (doc-only): §3.1 0B's baseline command corrected to
  `cargo test --test lint_oracle -- --ignored`, and the stale "do not describe it as ignored"
  sentence replaced. The test keeps its `#[ignore]` by design: the oracle is hermetic only on
  known-clean checkouts (CRLF-taint history), so it stays an explicitly-invoked gate rather than
  running on every `cargo test --workspace`.
- Verification: `git status --short` confirms the tree is back to its pre-Gate-0 state (only the
  pre-existing plan-file moves and untracked `bench-remote.sh`/`.claude/` remain).

## 2026-07-27 — Gate 0C/0G: contract census and dependency feasibility

Evidence only, per §3.1 0C/0G. No production/snapshot/golden changes, no commits, no bless/update env
vars, no git reset/clean/checkout. Ledgers: [`gate0-0c-api-ledger.md`](./gate0-0c-api-ledger.md),
[`gate0-0g-dependency-ledger.md`](./gate0-0g-dependency-ledger.md). Bulky raw output under
`target/braid-gate0/` (`cargo-metadata-full.json` sha256
`bdb185477c4d047fd962b7d2a42e38953ec8080ba674a6f3ac0ca6d6e602d654`; `--no-deps` sha256
`a57212542add63568407ab3e9bcd33f9c7baf497aa1cf756300d42f9b9a7baac`, matching the 0A value, so the
manifest set is unchanged since 0A/0B).

### 0A/0B carry-forward resolved: the committed declarations ARE trustworthy as an API contract

- Rebuilt both targets `--dev` into scratch out-dirs (flags matching `build:wasm:*:dev`;
  `restore-wasm-package-layout.mjs` deliberately not run). Both exit 0.
- `pkg-bundler/usfm_onion_web.d.ts` regenerates **byte-identical**. `pkg-web/usfm_onion_web.d.ts`
  differs **only** inside the `InitOutput` interface body. Both `pkg-*/package.json` byte-identical.
  `*_bg.wasm.d.ts` differ throughout.
- Every difference classifies as **bindgen-ABI-shape-only**: output-pointer → multi-value/externref
  calling convention and renamed raw exports (`__wbindgen_export{,2,3,4}` +
  `__wbindgen_add_to_stack_pointer` → `__wbindgen_malloc`/`_realloc`/`_free`/`_exn_store` +
  `__externref_table_{alloc,dealloc}`/`__externref_drop_slice`/`__wbindgen_externrefs`/`__wbindgen_start`).
  **Zero public-API-surface differences.** The 0B "materially different bindgen ABI shape" concern is
  confined to internal plumbing.
- Caveat retained: `InitOutput` is itself an exported TS type in `pkg-web`, so a toolchain bump does
  change one named public type's member list even though every member is plumbing.

### 0C census counts

- npm surface (`pkg-bundler/usfm_onion_web.d.ts`): **24 functions, 2 classes** (`ParsedUsfm` 14
  members, `UsfmMarkerCatalog` 5), **73 types** — 40 defined in `usfm_onion_dto`, 29 local to
  `usfm_onion_wasm`, 4 hand-written TS in the `typescript_custom_section`. Root `package.json` has 3
  export entries (`.`, `./web`, `./token-sids`).
- `usfm_onion_dto`: **43 public items** (40 types + 3 fns); 42 re-exported by `usfm_onion_wasm` —
  `decode_attr_value` is the one that is not.
- core `usfm_onion`: **423** public declarations inventoried (`target/braid-gate0/core-pub.txt`);
  **6 token traits** crossing into lint/format/diff (`UsfmToken`, `WalkableToken`, `LintableToken`,
  `FormattableToken`, `DiffableToken`, `SerializableToken`, plus `SerializableAttribute`);
  **32** `LintCode` variants.
- **13 conversion behaviors** catalogued (C1-C13): field rename `AttributeItem.source`→`text`;
  `TokenId` flattened to `"{book}-{index}"`; 8-byte `Sid`→formatted `String`;
  `LintIssue.template` `&'static str`→`String`; `TokenFix` external→internal tagging; `LintScope`
  double serialization; `FormatOptions` `bool`→`Option<bool>` tri-state widening; `DiffSkeleton<T>`
  generic erased; `attribute_source` span dropped; wholesale `&'a str`→`String` cloning; 40 exhaustive
  `From<Native…>` drift guards; 4 `JsError` throw sites; `toUsj()` bypassing tsify to return `any`.
- Dispositions: **retain 97** npm exports + 43 dto items + all core declarations; **replace 9**
  (`WalkToken` + its 7 owned-token conversion helpers → core `OwnedToken`; hand-maintained
  `lint_code_variants()` → wire-generated catalog); **delete-in-breaking-release 0**. Nothing is
  marked for deletion — the crate `usfm_onion_dto` disappears but none of its 43 items do.

### 0C findings

- **F1 — attribute payload loss, currently blessed.** `attributes`/`attributeSource` do not survive
  the format/fix/merge/revert/token-diff leg. Root cause is in **core**: `FormatToken` has no
  attribute fields and `FormattableToken` has no attribute accessor, so `map_format_token` /
  `map_walk_token` hardcode `attributes: Vec::new(), attribute_source: None`. Proven from committed
  goldens on `attributes.usfm` (`\w gracious|lemma="grace" \w*`): `format.usfm` and
  `format-tokens.json` emit `\w gracious\w*`, `diff-tokens.json` has zero `attributeSource`, while
  `to-usfm.usfm` / `tokens-to-usfm.usfm` / `tokens.json` / `cst.json` / `diff.json` are lossless.
  `applyTokenFix`, `formatTokensMut`, `mergeDiffBlocks`, `revertDiffBlock` share the cause and are
  untested because both harnesses use attribute-free fixtures. Bears directly on §9 losslessness and
  §5.3 `prepare_format_patch`/`apply_patch` rebuilding authoritative bytes. Pre-existing blessed
  behavior — recorded, not fixed, escalated for owner adjudication; 0D should treat it as a known
  non-round-tripping field.
- **F2** — `lint_code_variants()` is a hand-maintained 32-entry list (already named as non-guarded in
  the dto crate's own docs); replaced by wire's generated catalog under §7.7.
- **F3** — 13 exported functions have no golden coverage; `vrefIndexUsfm`/`vrefIndexTokens` appear in
  neither the golden nor the smoke harness.
- **F4 — plan-text staleness:** §4.3 and §17#12 describe `usfm_onion_wasm::token_values_to_usfm` as a
  live USFM-emission fork to retire. It no longer exists — wasm calls the one core emitter
  `tokens_to_usfm_reconstruct`, and `closer_shape`/`token_closes` are private core helpers. Landed
  with `serializable-token-contract`. No action beyond a plan-text correction.
- Evidence for §5.1: `usfm_onion_dto::Token` already carries every field `OwnedToken` names **plus**
  `span`, `marker_metadata`, `structural`; `structural` is not optional in practice (it drives
  `WalkableToken` and is passed through with no re-derivation fallback), so §5.1's "may retain
  additional marker structural metadata if equivalence proves it required" is already answered *yes*
  for `structural`. Private `usfm_onion_wasm::WalkToken` is a trait-complete but payload-incomplete
  owned token; `OwnedToken` must be the union of its trait coverage and `dto::Token`'s payload.

### 0G conclusions

- Target DAG proven acyclic: edge set `{wire→core, braid→core, wasm→core, wasm→wire, wasm→braid}`,
  topological order `core < {wire, braid} < wasm`. **Core is a sink** — zero workspace dependencies
  today and the epic adds none.
- Core can own `OwnedToken` with **no new dependency**: serde+derive is already unconditional in core
  (`Serialize` only; `Deserialize` stays on wire's `TokenDto`), and
  `cargo tree -p usfm_onion | grep -c 'wasm-bindgen\|tsify'` = 0. Three owned token shapes already
  implement the core traits, so the impls are not a new capability.
- **No feature cycle**: the target feature graph is a tree; only `wasm` sits downstream of a flag, and
  because braid does not depend on wire no feature can cross between the siblings.
- `usfm_onion_dto` consumers: **exactly one crate dep** (`crates/usfm_onion_wasm/Cargo.toml:19`) and
  **exactly one `use` site** (one `pub use` block at `crates/usfm_onion_wasm/src/lib.rs:48-56`), plus
  one explanatory comment. 40 of its types are npm-visible purely through that re-export; because
  tsify emits no crate path, moving the definitions changes no `.d.ts` byte. Removal is one mechanical
  commit (6 steps in the 0G ledger §6.2), with the §1 declaration diff as its known-good baseline.
  The 29 wasm-local DTOs move separately — they need `pub` fields for the native boundary, which is a
  source change, not a move.
- One generated schema source proven four ways: one crate/script/`--target` flag; committed bundler
  and web public `.d.ts` identical over lines 1-515 (web adds only loader glue); bundler regenerates
  byte-identically; and both `golden:wasm` and `golden:wasm:web` compare the **same** 117 golden files
  (`scripts/wasm-golden.mjs` resolves `outputs/` independent of target). Declaration drift check =
  rebuild to scratch + `diff` only `usfm_onion_web.d.ts`; it currently has **no** npm script or CI
  step behind it — Phase A step 6 should land that rather than a parallel generator.
- Native vs wasm32: 58 vs 52 resolved packages; the entire delta is rayon's subtree (`rayon`,
  `rayon-core`, `crossbeam-{deque,epoch,utils}`, `either`). No `cfg`-gated *type* anywhere; the only
  target divergence is `par::map_ordered`'s executor, documented order-preserving so output is
  byte-identical.
- Carry-forward, not a stop: **braid's xxhash3 dependency does not exist yet** — no xxhash/twox crate
  is in any manifest (core has `rustc-hash`/FxHash only, kept internal-only by §2.1#8). The chosen
  crate must build for `wasm32-unknown-unknown`; `npm run check:wasm:web` will cover it once wasm
  depends on braid.
- Also carried: wire's `wasm` feature will be always-on in a workspace build (as dto's is today), so
  native `cargo test --workspace` keeps compiling `wasm-bindgen`/`tsify` for the host. Reaches only
  wire and wasm, never core or braid.

### OWNER-DECISION rows (framed, not decided)

1. **`BookId` reuse** — does braid's declared `BookId` reuse core `usfm_onion::token::BookId`
   (`[u8;3]`, `Copy`, 3-ASCII-alnum validation, deliberately no canonical-66 membership check, and not
   in the root re-export list), or get a distinct validated type?
2. **USJ hand-written TS** — `Value`/`UsjDocument`/`UsjNode`/`UsjElement` are the only npm types not
   derived from Rust, and `toUsj()` is typed `any` so they are decorative. Move the string verbatim to
   wire, promote to tsify (a declaration change), or keep in wasm as the documented exception?
3. **`js/token-sids.js`** — retain as a deliberate wasm-free duplicate of core
   `derive_canonical_sids` (conformance-tested against the Rust export at
   `scripts/test-token-sids.mjs:89-97`), or delete-in-breaking-release now that the wasm export
   exists? §15 names "Algorithm fork" a footgun, but the no-wasm-dependency property is the point.
4. **Home of braid's lifecycle TS mirrors** (`BraidConfig`, `MutationEffect`, `CorpusInput`,
   `ScopedOutput<T>`, `PatchHandle`, `PrimeReport`, `ApiResult<T,E>`, every typed error enum) — wire
   owning them would be a wire→braid reverse edge, so the two DAG-legal homes are (a) braid gains a
   feature-gated serde/tsify layer over its own types (adds `braid → tsify → wasm-bindgen`, no cycle,
   needs a narrow reading of §4.1's "braid must not own JS values"), or (b) wasm defines mirror types
   (no new edges, but reintroduces the hand-mirrored-DTO pattern §4.1 exists to prevent, for ~15
   types).

### Stop conditions

- All four 0C stop conditions and all five 0G stop conditions: **not hit**. All 97 npm exports
  classified; no reverse-edge-forcing type; no dependency or feature cycle; no duplicated discriminant
  source; no target-specific semantic type.
- **One stop-adjacent finding raised for owner adjudication, outside the declared sets:** F1
  (attribute payload loss through the format/fix/merge/revert leg). Pre-existing and blessed rather
  than a migration consequence, but it contradicts §9's losslessness requirement for the resident
  format/patch path and matches §15's "Payload loss" footgun.

### Tree state

`git status --short` before and after is identical: ` D plans/approved/braid-epic.md`,
` D plans/approved/progress-braid-epic.md`, `?? .claude/`, `?? bench-remote.sh`,
`?? plans/approved/braid/`. No tracked file was modified; regenerated wasm output went only to scratch
out-dirs, and `target/braid-gate0/` is untracked build output. The two new ledgers and this entry are
inside the already-untracked `plans/approved/braid/`.

## 2026-07-27 — owner adjudication: Gate 0C/0G OWNER-DECISION rows and finding F1

- 8.1 `BookId`: braid's declared book **reuses core `usfm_onion::token::BookId`** (shape-validated
  `[u8;3]`, added to core's root re-exports in Phase C). Canonical-66 membership deliberately stays
  a lint concern (§2.2#12), not a type invariant — the `\id` catalog audit (XXA–XXG) and resident
  invalid-code policy both require the open type. Plan §5.2 amended.
- 8.2 USJ hand-written TS: **keep in wasm as the documented exception**; `toUsj()` stays declared
  `any` and the four declarations remain decorative. Typed promotion is deferred to §17#13; new
  `Braid.toUsj` declarations reuse the same union.
- 8.3 `js/token-sids.js`: **retain**. The conformance test against the Rust export is the drift
  alarm that distinguishes it from the §15 "algorithm fork" footgun. Deletion is re-examined at
  Phase F step 2 with caller evidence (§17#14) — delete only on a zero-callers census.
- 8.4 braid lifecycle TS mirrors: **option (a)** — braid derives serde/tsify on its own types
  behind a `wasm` feature flag, mirroring wire's feature shape; default braid stays
  dependency-pure. Wire never mirrors braid types (reverse edge); wasm never hand-mirrors (~15-type
  drift disease). §4.1's "no JS values" read narrowly: deriving serialization for owned types is
  not owning JS values. Recorded as new plan §2.2#13 (former #13 renumbered to #14).
- Finding F1 (attributes dropped through `FormatToken` legs): **pulled forward as its own
  pre-braid plan**, `plans/approved/format-token-attribute-passthrough.md`, following the
  `serializable-token-contract` precedent. Owner-directed design shape: format/fix become generic
  over a minimal token contract and return the caller's own token shape with untouched fields
  passed through — not field-chasing on `FormatToken`. Blocks Phase C/E's patch path; the golden
  change it implies gets its own clean-checkout bless adjudication.
- Plan text changes this entry authorizes: §2.2#13 added (+renumber), §4.1 braid row, §5.2 BookId
  paragraph, §17#13/#14 added, Relates-to prerequisite line. No production code changed; no tests
  run (documentation-only adjudication).

## 2026-07-27 — Gate 0D/0E: payload census and width study

Evidence only, per §3.1 0D/0E. No production/snapshot/golden/`testData` changes, no commits, no
bless/update env vars, no git reset/clean/checkout. Ledgers:
[`gate0-0d-payload-ledger.md`](./gate0-0d-payload-ledger.md),
[`gate0-0e-width-ledger.md`](./gate0-0e-width-ledger.md). Scanner, probes, synthetic inputs, and raw
JSONL live under `target/braid-gate0/` (out-of-tree cargo project, detached from the workspace);
commands + hashes in the 0E ledger §1.

### Provenance

- Inputs provably unchanged since 0A: all three 0A rollup hashes reproduce byte-for-byte
  (`testData` `2e00e92c…`, `en_ult` `dae6682c…`, `en_ulb` `b3d06c69…`). Reconciled a count
  discrepancy: 0A's 78/74 for en_ult/en_ulb are **all-file** counts; the `.usfm` counts are 67/66.
- Sets scanned separately, never concatenated: testData (262), golden inputs (7), en_ult (67),
  en_ulb (66), plus 30 new synthetic boundary files.

### 0D — token variants

- All 9 `TokenKind`/`TokenData` variants exercised (testData alone covers all 9). `NumberRangeKind`:
  only `Single` and `Range` occur in **any** real corpus — `Sequence`/`SequenceWithRange` are
  synthetic-only (`s25`–`s27`, `s30`).
- **§5.1 payload-legality contradiction (stop condition hit):** "marker-like tokens own marker
  metadata/attributes" conflates three structurally different variants — `Marker` has
  metadata+structural+nested+attrs, `EndMarker` has no attribute field at all, `Milestone` has no
  `nested` field. A single "marker-like" class either invents `nested` for milestones or permits
  attributes on end markers. `OwnedToken.nested: bool` is likewise imprecise.
- §5.1's omission of `structural` and `marker_metadata` is **safe**: the walker already re-derives
  structural from the marker name with "the same derivation parse uses"
  (`src/walker/mod.rs:516/598/688`), and no core trait reads marker metadata.

### 0D — findings

- 32-row finding matrix built with per-set evidence counts. **6 of 32 codes are produced by no real
  corpus**; 3 are now covered by new synthetic USFM (`invalid-book-code`, `missing-chapter-number`,
  `unknown-close-marker`).
- **3 of 32 codes are structurally unreachable from `lint_usfm(source)`** — `unknown-token`,
  `invalid-number-range`, `number-range-not-preceded-by-marker-expecting-number`. They exist only
  for the `lint_tokens(caller_tokens)` path (proven reachable there by probe), so braid's resident
  lint over `BookInput::Tokens` can produce them.
- **2 of 3 `TokenFix` variants have zero producers anywhere in core.** Only two construction sites
  exist, both `ReplaceToken` with one replacement and empty `label_params`, attached to exactly 3
  codes (25/26/27). `DeleteToken`/`InsertAfter` must be hand-constructed for §11.4.
- `book-code-not-uppercase` carries **no** `TokenFix`; its deterministic remedy rides in
  `message_params["uppercase"]`, so §7.6's message-payload sidecar is load-bearing for remediation.
- No `LintIssue` field fails to round-trip through §7.6's record + sidecars.

### 0D — ordering (proven)

- Canonical sort key: `span` → **kebab code string** → `related_span` → `token_id` → `marker` →
  `message`. `dedupe_issues` runs first and keys on `(code, span, related_span, token_id)`, so keys
  5–6 can never decide. Verified empirically: **0 adjacent pairs tie on keys 1–4** across all 61,166
  findings in all five sets. **A JS reconciler can reproduce canonical order without message text.**
- Two constraints this imposes on §7/§8: (1) the order key is the **kebab code string**, so sorting
  by the append-only wire `u8` gives a *different* order — the generated catalog must carry the
  string; (2) `token_id` is compared as a string, and Rust byte-wise UTF-8 vs JS UTF-16 code-unit
  order diverge for non-ASCII ids.
- Deterministic occurrence is **not** assigned in onion (dedupe removes twins rather than numbering
  them). The editor already assigns it deterministically after a stable base-key sort
  (`normalizeFindings.ts:36-63`) and explicitly pushes no id requirement into onion.
- Universal invariant found: **every** finding's span is exactly one whole token, or the finding is
  fully anchor-only. Zero sub-token spans, zero boundary crossings, zero span/`token_id`
  disagreements, in all five sets. That makes `(token_idx, offset=0, length=0)` a lossless and
  order-isomorphic re-encoding of `span`.

### 0D — stop condition hit: D1, spanless resident tokens change finding order

- §5.1's `OwnedToken` has no `span`; `LintableToken::span()` defaults to `None`; `span` is the
  **primary** canonical sort key. Linting one source twice — parsed native tokens vs the same tokens
  with only `span` removed — yields the **same 5 findings, identical multiset, different sequence**
  (reading order 19/28/36/44/46 becomes code-string-then-token-id-string order, and `GEN-17 < GEN-22
  < GEN-9` lexicographically, so it is not even token order).
- Consequences: §10's Phase C gate ("resident lint equals stateless whole-book core lint in content
  **and order**") is not satisfiable as written; finding order becomes ingest-shape dependent while
  §5.5's `SnapshotId` folds only `(book, source_hash)` + config/engine stamp; and `LintIssue.span`
  is `None` for every resident finding.
- Not a regression braid introduces — the editor **already** feeds onion spanless tokens on purpose
  (`usfmOnionTypes.ts:28-37`, `Token = Omit<OnionToken, "span">`, making a stray `.span` read a
  compile error). What fails is braid's *promise* of resident/stateless order parity.
- Candidate resolutions (none chosen here): `OwnedToken` carries `span: Option<Span>` (the current
  wire `Token` DTO already does, and §7.4 encodes span columns anyway); or `canonical_sort` gains a
  positional key ahead of the code string; or braid lints borrowed parse tokens and maps back.

### 0D — two specification gaps (not stops)

- **D2:** §7.5's `AnchorOnly` fidelity bit is **not computable from `Sid` or `NumberRangeKind`
  alone**. `\v 1,3` yields `Sid` delta 0 (identical to `\v 1`), and `\v 1a` is indistinguishable
  from `\v 1` in both `Sid` and `number.kind` — the suffix survives only in the token's source text.
  Bridges > 127 *are* detectable from `Sid`. §7.5 must say fidelity is derived from the number
  token's source text.
- **D3:** no encoding distinguishes "finding has no SID at all". `missing-id-marker` has
  `span: None, token_id: None, sid: None`, but `chapter=0, verse=0` is a **legal** chapter-scope SID
  (`Sid::new(book, 0, 0)` for `\id`/front matter). A "no anchor" flag bit is required.

### 0D — declared-book context and editor field usage

- No `BookIdMismatch` and no declared-book field exist in core. The landed book-code rules sit in
  `lint_structure_rules`, which already receives `&LintOptions` — insertion point with no signature
  change. Both are `LintCategory::Document`, so a mismatch rule inherits `Chapter`-scope suppression
  for free (probe: `Book`/`Front` → `["invalid-book-code"]`, `Chapter(1)` → `[]`).
- **Residency proven:** `\id GEN`, `\id ZZZ`, `\id php`, and `\id 2CO` each parse to 12 tokens and
  re-serialize **byte-identically**. A valid-but-wrong `\id 2CO` produces **zero findings** — the
  `BookIdMismatch` gap is real and unfilled. A future context field must be `Option`al for
  no-context stateless callers to keep current behavior (`LintOptions` has no `Default` by design).
- Editor field usage (`scripture-editor-proto-2` @ `ded10b69`, read-only): reads `code`,
  `messageParams`, `tokenId`, `message`, `sid`, `fix`, `severity`, `relatedTokenId`, `category`,
  `issueType`, `marker`. **Zero reads of `template`, `span`, `relatedSpan`** — all three derivable,
  so §3.2#11's field list needs no additions. Identity is already
  `onion:${code}:${tokenId}:${relatedTokenId}#occurrence` with message and fix excluded, matching
  §8.1 exactly.

### 0D — C1–C13 round-trip verdicts (one line each)

C1 attr `source`→`text` rename: **representable** (naming only). C2 `TokenId`→`"{book}-{index}"`:
**representable** via the string dictionary. C3 `Sid`→`String`: **representable** (packed SID is
narrower); carries the D2 gap. C4 `template` `&'static str`→`String`: **representable, derivable**
from code. C5 `TokenFix` external→internal tagging: **representable** (disappears in packed form).
C6 `LintScope` double serialization: **representable** — never packed, folds into the config
fingerprint. C7 `FormatOptions` `bool`→`Option<bool>`: **representable** — never packed; caveat, the
fingerprint must hash the **resolved** native options, not the tri-state wire form. C8
`DiffSkeleton<T>` generic erased: **representable** — §16 keeps diff object-shaped in v1. C9
`attribute_source` span dropped: **representable** — §7.4 is strictly richer than today's JSON DTO.
C10 wholesale `&str`→`String` cloning: **representable — the conversion §7.4 exists to delete**.
C11 40 exhaustive `From` impls: **representable** — compile-time guards, not data. C12
`JsError` throws: **representable** — not packed. C13 `toUsj(): any`: **representable** — not
packed, owner-adjudicated exception. **No C-row cannot round-trip.**

### 0E — envelope highlights

- Real per-book maxima: source 5,154,281 B (en_ult 01-GEN); tokens 276,987 (en_ult 19-PSA); unique
  SIDs 2,612 (19-PSA); unique markers 221 (testData kitchen-sink); chapter 150 / verse 176 (PSA);
  bridge delta 71 (testData origin); findings 17,404 (en_ult 04-NUM); attrs/token 9; verbatim
  attr-source 191 B; message-params 40 B / 3 params; finding token-relative offset **0** and length
  35; replacements per fix 1; `label_params` per fix **0**.
- **Byte-losslessness: 425 of 425 files** round-trip through `tokens_to_usfm`, including every
  pathological synthetic (chapter 999999, bridge 1-257, a 70 KB single token). This is the §9/§11.5
  serialization baseline.
- en_ult and en_ulb have **zero** duplicate declared book codes and exactly one `\id` per book →
  both are valid braid corpora under §5.2. testData/golden are fixture trees, not corpora (GEN×107).
- Modelled largest token section: **11,989,965 B** for en_ult 19-PSA (2.34× source). Whole en_ult
  container ≈ 229.9 MB for 103.2 MB of source.
- Size observation: the token-id string dictionary is **31–41% of every modelled section** and is
  100% redundant for cold parse (ids are exactly `{book}-{index}`). A one-bit "ids are positional"
  section flag would remove 3.77 MB from en_ult 19-PSA. Worth considering at Phase A step 3.

### 0E — §7 width verdicts and the one breach

- **No real corpus maximum reaches any §7 field.** Closest: `marker_descriptor_index` 221/65,535
  (0.3%) and packed-SID delta 71/127 (56%). Spans 5.15 MB/4.29e9 (0.12%). Record counts 276,987 in a
  `u32`.
- Packed-SID `chapter`/`verse` `u16` add **no new loss**: core's `saturating_u16`
  (`src/parse/mod.rs:551`) already clamps `Sid` to 65,535 while the unclamped value survives in the
  number record (`\c 999999` → `Sid` chapter 65,535, raw 999,999 preserved).
- Packed-SID delta boundary now exercised synthetically: 4 tokens at delta 127 (`Exact`), 4 at 128
  and 8 at 255 (→ `AnchorOnly`); `\v 1-257` saturates to 255 in core's documented `u8`.
- §7.6's `offset in token` and `length` and the `overflow_span` sidecar have **no producer** — every
  finding is exactly one whole token, so `length = 0` always applies. The one 70,001-byte case
  (`s10`) is a whole token and needs no sidecar.
- **BREACH — `sid_index: u16` (§7.4).** Capacity 65,535 with `0xffff = None`. Real max 2,612 (4.0%);
  synthetic `s22-many-sids` (300 chapters × 250 verses) reaches **75,301**. Real scripture cannot
  approach it (whole canon ≈ 31,102 verses; Psalms 2,461), but `s22` is structurally legal USFM and
  §7 specifies **no** overflow path — no format-version bump, no `DecodeError` variant, no sidecar.

### PROPOSED plan amendments (owner sign-off required; NOT applied)

1. **§7.4 `sid_index` overflow path** — pick one: (a) widen to `u32` with `u32::MAX = None`
   (+2 B/token, ≈ +554 KB / +4.6% on en_ult 19-PSA); (b) keep `u16`, add a specified encoder refusal
   plus `DecodeError::TooManySids { found }`; (c) keep `u16` with a format-version-gated wide
   variant.
2. **§7.7 documentation** — record that core's `EnabledCodes` `u64` bitmask
   (`LintCode::bit()` = `1u64 << discriminant`, `src/lint_impl.rs:440`) caps `LintCode` at **64**
   variants, so the real growth headroom is 32 codes, not 223, and the wire `u8` can never be the
   first thing to overflow.
3. **§7.4 number-record width** — state that the sparse number payload must be `u32`: raw
   `TokenData::Number.start/end` reach 999,999 while `Sid` saturates at 65,535, so a `u16` payload
   would silently normalize malformed input against §0's "never silently normalize on ingest".
4. **§7.5 fidelity derivation** (D2) — state that `AnchorOnly` is derived from the number token's
   **source text**, not from `Sid` or `number.kind`.
5. **§7.6 flags** (D3) — add a "no anchor at all" flag bit; `chapter=0, verse=0` is a legal
   chapter-scope SID and cannot also mean "no SID".
6. **§5.1 / §10 Phase C** (D1) — resolve the spanless-`OwnedToken` ordering consequence before Phase
   A freezes the layout, and restate the Phase C gate to name which stateless baseline it compares
   against.
7. **§5.1 payload legality** — replace the single "marker-like" class with the three actual variant
   shapes (`Marker` / `EndMarker` / `Milestone`).

### Stop conditions

- **0D: 2 hit.** D1 (`LintIssue.span` cannot round-trip through a spanless `OwnedToken`, and
  canonical order is not preserved — §10 Phase C gate) and the §5.1 payload-legality contradiction
  (§1.1). Plus 2 specification gaps that are not stops (D2, D3).
- **0E: 1 hit.** `sid_index: u16` — synthetic-only, no specified overflow path (amendment 1).
- Finding F1 (attributes through `FormatToken`) was deliberately **not** re-litigated; it is already
  pulled forward as `plans/approved/format-token-attribute-passthrough.md`. **No new
  non-round-tripping payload was found.**

### Tree state

`git status --short` before and after is identical: ` D plans/approved/braid-epic.md`,
` D plans/approved/progress-braid-epic.md`, `?? .claude/`, `?? bench-remote.sh`,
`?? plans/approved/braid/`, `?? plans/approved/format-token-attribute-passthrough.md`. No tracked
file modified. All scanner code, synthetic inputs, and raw output are under the untracked
`target/braid-gate0/`.

## 2026-07-27 — owner adjudication: Gate 0D/0E amendments applied

- **D1 resolved by re-keying canonical order, not by adding spans.** New plan §2.2#15:
  `canonical_sort`'s primary key becomes primary-token position in the book stream (anchor-only
  last), then kebab code string, then related-token position. `OwnedToken` stays spanless; no
  ingest-time byte-offset derivation. Expected oracle-neutral because every finding span is a
  whole token, so position order equals span order for parsed tokens — verified against the lint
  oracle at Phase C step 1 before adoption; any difference is a stop. Side effects: token-id
  strings leave the sort path (UTF-8/UTF-16 comparator divergence eliminated); packed finding
  records ship in canonical order so JS needs no comparator. Chapter-grain compatibility checked:
  chapter runs are contiguous ranges, so chapter-local positions rebase by run start; positions
  remain snapshot-local addresses, never identity.
- Cross-book order stays corpus/caller order (§2.1#3 unchanged); reference/canon-order
  presentation is a consumer concern — findings carry SIDs.
- `sid_index` stays `u16` + loud refusal: typed encoder error and `DecodeError::TooManySids`
  (§7.4, §5.6). Real max 2,612 SIDs/book vs 65,535 capacity; breach is synthetic-only.
- Token-id dictionary made conditional (§7.4): section flag `positional_ids` omits column and
  dictionary for cold-parsed books (`{book}-{index}` synthesized by decoder; 31–41% of section
  bytes saved). Live caller-id sections keep it.
- Number records widened to `u32` (§7.4): raw `\v 999999` exists in fixtures; `Sid` saturation
  must not clamp the token payload.
- §7.5: packed-SID fidelity bit specified as derived from source text, never `Sid`/`NumberRangeKind`
  alone (D2).
- §7.6: `no-anchor` flag bit added — `(0,0)` is a legal front-matter SID and cannot mean "no SID"
  (D3); stored-in-canonical-order sentence added.
- §7.7: core `EnabledCodes` `u64` bitmask documented as the operative LintCode ceiling (64; 32 in
  use). Owner accepts current headroom; widen the mask (u128) if monotonicity rules ever need it.
- §5.1 legality table corrected per the token-variant matrix: opening markers / end markers /
  milestones are structurally distinct (end markers never carry attributes; milestones have no
  `nested`).
- D4 noted for Phase B test authoring: 3 codes and 2 `TokenFix` variants need hand-built token
  streams/fix values, not `.usfm` fixtures.
- Gate 0D/0E accepted. All seven proposed amendments adjudicated and applied to the plan text.
  No production code changed; no tests run (documentation-only adjudication).

## 2026-07-27 — owner adjudication: 0F rescope and 0H charter sign-off

- Gate 0F rescoped: the editor's braid-adoption plan
  (`scripture-editor-proto-2/agent-tmp/plans/braid-editor-adoption/plan.md`) is owner-verified
  intent, and the editor migrates off its current internal surfaces — so 0F does not audit current
  editor guts. It (1) verifies braid-epic positions braid to fulfill that intent as a *general*
  stateful library (editor app policy, e.g. severity quieting, classified as correctly-not-braid),
  and (2) verifies the ingest preconditions braid's invariants require (unique stable token ids
  incl. the linebreak-N scan, complete chapter runs, exact bytes/EOL, manifest uniqueness). Phase F
  parity transcripts are shaped by the adoption plan's target flows, not current glue. Handoff:
  `handoff-gate0-0f.md`.
- Gate 0H spike charter signed off and frozen (`gate0-0h-spike-charter.md`); execution gated on
  0F completion. Owner rationale on Δ1 recorded: cold open pays ~66 IO-bound per-book reads
  regardless, so embedded source duplicates disk for zero read savings; the replay quantifies the
  embedded section's share so historical numbers can be interpreted for the v1 external-source
  layout. Replay dispatch: Sonnet.
- No production code changed; no tests run (documentation-only adjudication).

## 2026-07-27 — Gate 0F: editor contract alignment

Evidence only, per `handoff-gate0-0f.md` (§3.1 0F as rescoped by owner adjudication: verify braid
positions itself to fulfill the editor's owner-verified adoption plan as a *general* library, plus
the ingest facts braid's invariants depend on). Ledger:
[`gate0-0f-editor-ledger.md`](./gate0-0f-editor-ledger.md). Read-only in both repos; probe code and
raw JSON under the editor's gitignored `agent-tmp/gate0f/` (hashes + replay commands in the ledger
§0). Editor HEAD `ded10b69` (`feat/stet`), its installed onion is `usfm-onion-web` 0.0.9.

### Alignment tallies (63 contract rows)

- **fulfilled 35** · **fulfilled via editor glue 8** · **app policy 12** · **gap 6** ·
  **conflict 2**.
- No row shows the adoption plan expecting braid to own app policy. Closest: E§3.8 offers "app
  policy overlay **or** upstream-recognized compatibility option" for Zephyr's `s5` acceptance —
  braid should decline the second half; an app-specific marker acceptance is not a general USFM fact.
- Fully aligned, no action: scope semantics (§3.2 ↔ §5.2/§5.3/§2.2#1/§6.4), findings contract
  (§3.6 ↔ §8.1 + §2.2#15 — signature, identity tuple, and the now-comparator-free canonical order all
  match, and the generated `LintCode` kebab union + `messageParams` already satisfy the editor's
  compile-exhaustive formatter), resident-vs-stateless (§3.9 ↔ §5.4, including the "do not pre-bake
  dirty/current over the neutral compare model" clause), dirty-buffer boundary (§3.10 ↔ §6.1's
  existing ban on label-keyed single-range maps).

### Conflicts (owner adjudication before Phase A freezes discriminants)

1. **EOL contract absent from braid-epic.** `grep -i "line ending|crlf|eol"` over the plan returns
   one unrelated bless footgun. The editor's §3.4 declares an EOL contract an upstream release
   prerequisite and its Gate 0 stops without it. Measured (probe P7 on a real book converted to
   CRLF): parse keeps `\r\n`; one Lexical round trip makes every newline token's `source` `"\n"`;
   `tokensToUsfm(tokens, eol)` restores the file exactly; naive `source` concatenation — which is
   what §9's `to_usfm` specifies — does **not**. Consequence: every CRLF book is permanently dirty
   and permanently cache-invalid, and the editor cannot compensate without keeping the serializer
   §16/E§5 delete.
2. **Native-host sequencing.** §8.3 permits shipping wasm-in-webview first and deferring resident
   native IPC; the editor's Gate 0 requires the native Rust crate behind a Tauri command host to pass
   the same corpus suite, and its §14#7 makes wasm-in-Tauri a last resort. Semantically compatible
   (braid is plain sync Rust); the release scope is what conflicts.

### Gaps (one line each; framed as PROPOSED amendments A–K in the ledger §6)

- **R1 — opaque warm restore.** No entrypoint seeds residency from packed bytes: residency needs
  `replace_corpus` (parse or JS tokens) and §8.2 types `primeLintCache(input: LintPrimeInput)` as a
  JS object, so findings must be decoded in JS too. braid's planned surface *is* the
  `materialize → replaceCorpus(Token[]) → primeLintCache` fallback E§3.5 refuses to enshrine
  silently. Third option braid nearly supports and the editor did not consider:
  `replace_corpus(Usfm{source}) + prime_lint_cache` (one parse, no lint).
- **X1 — no resident token read, no patch inspection.** `apply_patch` returns `MutationEffect` and
  `prepare_format_patch` returns an opaque handle; the only token exposure in the plan is
  `LintSnapshot.tokens` via `lint()`. So a consumer that keeps its own token view must run a full
  lint to see the result of a fix, and a format preview has nothing to render (E§7 "commit returned
  tokens to store", "choose options and preview"; E Gate 6).
- **H3 — serde coupled to wasm.** §2.2#13 puts serde+tsify behind one `wasm` feature; a native Tauri
  IPC host needs serde without tsify/wasm-bindgen. Split `serde` from `wasm` (same for wire).
- **H4 — no native codec entrypoint named.** `grep pack_/encode_/fn pack/Vec<u8>` finds nothing; only
  the wasm `lint(): Uint8Array` is specified, yet a native host must call wire to produce the
  ArrayBuffer the editor requires.
- **D4 — chapter-scope dirty.** §9 defines dirty as book bytes vs baseline bytes; `is_dirty(Chapter)`
  is unspecified when the baseline lacks that label or has duplicate runs — both reachable.
- **H2 — module-worker packaging.** The editor's primary web host is a Vite module worker; no Phase F
  package gate covers importing/initialising there or transferring the packed buffer.

### Ingest preconditions (probe over 95 real `.usfm` files, three corpora scanned separately)

- **Token-id uniqueness: NOT met today, cause already filed, no new site.** All 95 books in all three
  corpora contain duplicate ids after the real commit-path tokenization; **every duplicate in every
  book is a `newline` token with a `linebreak-N` id** (BSB 5,868 distinct / 77,622 instances;
  llx_reg 1,059 / 10,587; synthetic 54 / 228). Zero non-newline duplicates, zero missing, zero empty.
  `lexicalToTokens` has exactly **one** synthesized-id site (`:277`, counter reset per call, called
  per chapter) — the handoff's "any OTHER synthesized-id site" question answers **none**. Upstream
  cause: `tokensToLexical` maps a newline token to a bare `{type:"linebreak"}` node, dropping the
  parsed id. `IngestError::DuplicateTokenId` would reject every book until E§10 Section 1 step 4
  lands. Persistence: ids survive undo/redo exactly (probe P5: 979/979 through the history canonical
  snapshot; `USFMTextNode` keeps `id` in Lexical NodeState) and repeat calls are stable, but an
  inserted earlier linebreak rebases all later linebreak ids (probe P6), and a cold reopen re-derives
  positional `{BOOK}-{index}` ids so they are session-stable, not cross-session stable. A live
  interactive run is a **recorded blocker with repro steps**, never simulated.
- **Complete chapter runs: met.** `WorkingFilesStore.applyPatch("chapter")` replaces `currentTokens`
  wholesale from the chapter's whole lexical state (`:314-336`); `mirrorPatchProducer` pushes that
  canonical stream per changed chapter, `fullSync` for content-bearing project commits,
  `deleteChapter` for removals. Grouping is order-faithful on all 95 books (buckets strictly
  ascending; bucket count = `\c` runs + 1 front-matter bucket in every corpus). Structural caveat,
  unexercised: buckets are keyed by parsed integer and `chapterNumber` is a `number`, so duplicate,
  out-of-order, and non-integer chapter labels are not representable — measured incidence 0/0/0. The
  target publisher (E§9.2) promises to preserve them, so chapter identity must become a label; that
  is an editor-side prerequisite, not a braid gap.
- **Exact bytes: met from file bytes; not through the editor's own serializer.** 93/95 books
  round-trip byte-exactly; the Lexical round trip adds **no** loss anywhere — both failures
  (`llx_reg/44-JHN` at byte 108,176; `synthetic/kitchen-sink` at 4,534) are already present at flat
  `token.source` concatenation, because the editor's byte waist concatenates `source` while onion
  parks attribute text in `attributes`/`attributeSource`. onion's own emitter is lossless (0E:
  425/425), so braid taking over serialization fixes both. Note for §12: the desktop cold path
  deliberately keeps book contents out of JS (`loadForApp` → `{text: null, path}`), while §2.2#3 /
  §8.1 `materialize(sources, packed)` require per-book bytes on the main thread — the warm path
  reintroduces a whole-corpus JS byte copy on Tauri that should be attributed separately.
- **Unique declared books: NOT guaranteed.** `BookId` ← RC manifest `identifier` / Burrito ingredient
  scope; `SourceKey` ← `storageKey`, which is a **basename**. Neither loader checks uniqueness
  (`buildBookRefs` pushes per ingredient; RC enforces only inside `addBook`). Measured: 0 duplicates
  in BSB (66 distinct) and llx_reg (27 distinct); the synthetic fixture directory declares `GEN`
  twice. Because §5.2 refuses the **entire** corpus on `DuplicateBook`, caller-side manifest
  validation (unique code, path-based source key) is a mandatory editor prerequisite — worth one
  README line upstream.

### Transcripts

Five target lifecycle transcripts written from the adoption plan's §10 target flows (cold open, warm
open, live edit, undo + fix/format, save/baseline/backup/warming), with per-transcript assertions;
they cover every line of §11.6's existing transcript and add the warm-open cache-miss matrix, the
partial-write accounting, and the EOL dimension. T2 is blocked on gap R1 and T4 on gap X1.

### Tree state

Both trees identical before and after. `usfm_onion`: the five pre-existing entries plus the untracked
`plans/approved/braid/` and `format-token-attribute-passthrough.md`. Editor repo: its four
pre-existing entries (` M package.json`, ` M product-docs/specs/project-import-and-management.md`,
` M public/stet/en.json`, `?? product-docs/specs/stet.md`), none touched. Probe files live only in
`agent-tmp/gate0f/`, excluded by `.gitignore:46`. No bless/update env var; no git
reset/clean/checkout/stash anywhere.

## 2026-07-27 — owner adjudication: Gate 0F amendments A–H applied; native host first-class

- **A (EOL, new §2.2#16):** per-book `LineEnding` (`Lf | CrLf`) — detected for USFM ingest,
  declared on `BookTokensInput`, inherited by chapter updates; emission via an optional override
  on the core reconstruct emitter (core owns emission; braid never rewrites newline tokens);
  `SourceHash` over the emitted bytes; mixed-EOL input preserved verbatim until first edit; the
  ending is not stored in the packed container (derivable from bound source bytes). Without this,
  Gate 0F probe P7 shows every CRLF book has a permanently invalid warm cache.
- **B (restore, new §2.2#17):** braid gains atomic `restore_corpus(CorpusRestoreInput)` (source +
  decoded tokens + optional per-book lint, prime-style stamp validation, per-book rejection);
  wasm composition root exposes `restoreCorpus(sources, packed)` decoding via wire. Makes the
  golden loop's "reconstruct without lex/parse" actually reachable; parse+prime stays the
  documented fallback. Types added in §5.5; braid remains wire-free.
- **C:** chapter-scope dirty rule specified in §9 (same-label baseline run; missing run = dirty;
  duplicate labels = typed ambiguity error).
- **D:** §8.2 wording — registry queries synchronous after module init; wasm-free marker facts
  are not a braid promise.
- **E:** resident `to_tokens(scope)` and non-mutating `preview_patch(handle)` added to §5.3/§8.2;
  preview and apply share one frozen snapshot; patch application is never reimplemented consumer-side.
- **F:** §2.2#13 amended — braid/wire features split into `serde` (plain derives, native IPC
  hosts) and `wasm` (= serde + tsify/bindgen); one set of serde attributes remains the single TS
  contract.
- **G:** §7 gains the public native codec surface (`encode_corpus`/`decode_borrowed` sketch,
  `EncodeError`, snapshot id passed as plain `u64`); exact names freeze in Phase 0 step 3.
- **H:** Phase F gate — package imports/initialises in a module worker; packed buffer survives
  transfer.
- **H5 resolved in the editor plan's favor (§2.2#18, §8.3 rewritten):** the native resident host
  is a first-class v1 hosting path — editor desktop hosts native `Braid` behind Tauri commands,
  packed results as ArrayBuffers. Braid ships the native surface (serde feature, public codec,
  native round-trip contract tests in §11.3); the Tauri host itself stays editor-owned; braid has
  no Tauri dependency. Phase F parity transcripts run against web wasm AND the native host.
  Supersedes §8.3's "defer resident native IPC".
- Also applied: Phase C steps 5/8 and Phase F step 3 updated; §11.5 EOL/chapter-dirty/preview
  tests added; §13 README bullet on mandatory caller-side manifest validation (0F §4.4); Phase F
  gate records desktop `materialize` source-binding cost separately (0F P3b).
- Preconditions accepted as evidence: linebreak-N confirmed at scale (sole duplicate-id source;
  editor fix already owned), complete chapter runs met, exact bytes met from file reads (the
  editor's own serializer loses attribute bytes on 2 files — braid taking over serialization fixes
  both), manifest duplicates possible today (editor-owned prerequisite).
- No production code changed; no tests run (documentation-only adjudication). Gate 0F accepted.

## 2026-07-27 — Gate 0H: spike charter and unchanged replay

Unchanged replay of `.claude/worktrees/agent-af68c779deab4e90a` per the signed
`gate0-0h-spike-charter.md`. No edits/cleans/resets in the worktree; only its own documented
commands were run (they regenerate its untracked `GEN/PSA.{bin,json}` — execution residue, not an
edit). Main repo tree confirmed unchanged before/after (`git status --short` identical).

- **Provenance:** worktree `rev-parse HEAD` = `c341de5` (branch `worktree-agent-af68c779deab4e90a`),
  preserved dirty state exactly as the 0A ledger recorded — modified `src/lib.rs` (+2, adds
  `pub mod wire_spike;`), `src/marker_defs.rs` (+10, adds `MarkerIndex::spike_raw`/`spike_from_raw`);
  untracked `GEN.{bin,json}`, `PSA.{bin,json}`, `examples/wire_probe.rs`, `examples/wire_spike.rs`,
  `js/{spike_b,wasm_vs_bin,wire_decode}.mjs`, `src/wire_spike.rs`. `git diff --stat` on the two
  modified files matched the 0A description exactly (no drift). Toolchain: `rustc 1.95.0
  (59807616e 2026-04-14)`, `cargo 1.95.0`, `node v24.4.1`, aarch64-apple-darwin, Apple M1 Max
  (same machine as 0A/0B). Build profile `--release`.
  - Input hashes (sha256) before this run: `example-corpora/en_ulb/19-PSA.usfm`
    `15049b253cb33995594d71f2e9291efd4f7ec0ddcef932937c717a35584b1ae4`;
    `example-corpora/en_ult/01-GEN.usfm` `aa4551fb70ea42023da47d3a25ecd7893299d7b19e13ec75c39b44cd4f7ee2cd`;
    `example-corpora/en_ult/*.usfm` (67 files, whole-corpus case) rollup sha256
    `05cb2016dbdc3e2196c5a5e5f64f6d23d5adec654100bc4515448a612649c92b`.
  - Pre-existing `GEN.bin`/`PSA.bin`/`.json` (from the worktree's last prior run, 2026-07-21) hashed
    before replay, then regenerated by `cargo run --release --example wire_spike`: `.json` hashes
    are byte-identical before/after (`GEN.json` `73c57b13…`, `PSA.json` `01e61e75…`); `.bin` hashes
    differ before/after despite identical inputs/tokens and identical sizes-of-record (GEN.bin
    15,188,892→15,688,176 B; PSA.bin 742,228→742,276 B) — the encoder interns strings/marker
    descriptors via unseeded `std::collections::HashMap`, so dictionary *order* (and therefore byte
    layout, though not semantic content) is nondeterministic run-to-run. Noted as harness behavior,
    not touched.

- **Harness reading (no modification):**
  - `examples/wire_spike.rs` (Rust): reads `example-corpora/en_ulb/19-PSA.usfm`,
    `example-corpora/en_ult/01-GEN.usfm`, and all 67 `example-corpora/en_ult/*.usfm` concatenated
    ("corpus") — file IO happens once, entirely **outside** every timed region. `bench()` does 3
    untimed warmup calls then times `iters` calls, reporting mean ms/iter (`iters` = 100 for
    n≤100k tokens, 20 for ≤1M, 8 above — PSA/GEN both hit 100, corpus hits 8). Four phases timed
    per book: serial `lex`+`parse_lexemes`, parallel `parse()`, serial `decode_borrowed`, parallel
    `decode_borrowed_par` — reported separately, never summed, plus a derived "par decode / par
    parse" ratio. Correctness gate (`decode_borrowed == parse().tokens` and
    `decode_borrowed_par == parse().tokens`) runs and asserts **before** any `bench()` call.
  - `src/wire_spike.rs`: the codec itself. Header = magic/version/section-count/3-byte declared
    book/token-count/source-len; 13 fixed sections via a directory of (id, offset, len); dictionaries
    intern strings and 12-byte marker descriptors (incl. the private `MarkerIndex` raw `u16`, see
    Δ2); SID dictionary uses a 10-byte record (book[3]+pad+chapter u16+verse u16+delta u8+pad, see
    Δ4); `S_SOURCE` embeds the full source bytes as section 0 (see Δ1).
  - `js/wire_decode.mjs`: `decodeEager` builds one JS object per token (apples-to-apples vs
    `JSON.parse`); `decodeLazy` builds only the O(1) typed-array column views plus on-demand
    `.text(i)`/`.kindName(i)` accessors — zero per-token objects. `bench()` does 3 warmup calls, then
    times `iters` calls (40 for PSA, 10 for GEN — file reads happen once before any `bench()` call,
    outside the timed region), reporting mean ms/iter. A "viewport" case runs `decodeLazy` then
    materializes 200 tokens' text — proxy for an editor's visible-window cost.
  - `js/wasm_vs_bin.mjs`: loads the *committed* `pkg-web` wasm bundle via `initSync`, times
    `pkg.parse(src).tokens()` (serial wasm parse + full JS object marshalling, since wasm never
    links rayon) against `decodeEager(bin)`; same warmup/iters convention as above, file IO outside
    timing.
  - `js/spike_b.mjs`: worker-thread round-trip proxy for the wasm-worker boundary. Times (1)
    `JSON.parse` building N objects on the main thread (baseline "paid either way" cost), (2) a
    structured-clone round-trip of those N objects through a `worker_threads` Worker, (3) an
    `ArrayBuffer` transfer (zero-copy, neutering) round-trip of the same size buffer. 3 warmup +
    20 timed round-trips each; file IO outside timing. Node `worker_threads` structured clone is an
    analogue for, not identical to, a browser main-thread↔Worker `postMessage`, which the harness
    does not itself flag.
  - `examples/wire_probe.rs` exists in the worktree but is **not** one of the charter's five
    documented commands, so it was read for context only and not run.

- **Correctness-gate results (executed before any timing, per protocol):**
  - Semantic equality vs `parse(source)`, exhaustive (not sampled), both decoders, all three
    corpora: **PASS** — `PSA: serial decode==parse: true | par decode==parse: true` (30,857
    tokens), `GEN: … true | … true` (263,131 tokens), `corpus: … true | … true` (5,289,015 tokens).
  - Rust serial vs parallel decode agreement: **PASS** (transitively, both equal `parse()` above)
    for all three corpora.
  - JS materialization vs Rust: **PASS but partial/sampled** — `wire_decode.mjs` samples ~1/800
    tokens (813 of PSA's 30,857; 803 of GEN's 263,131) and compares only `id`, `kind`, `source`,
    `sid`, `marker`, `markerMetadata.{canonical,kind}`, `structural.scopeKind`,
    `numberInfo.start`, `bookCode`, `attributes.length` — **not** `bookCodeValid`, `nested`,
    `markerMetadata.family`, `structural.{inlineContext,noteContext}`, attribute contents
    (span/text/key/value/isDefault), or full attribute equality. Reported `0 mismatches` on both
    books this run.
  - **Coverage gaps (charter checks the harness does NOT perform — recorded, not added):**
    (1) token→USFM byte identity is never tested anywhere in the harness; (2) wrong-source and
    malformed-buffer rejection is never tested — there is no code path that feeds a corrupt buffer
    or mismatched source to any decoder; (3) JS-vs-Rust equality is sampled (~1/800) and partial-field,
    not the full public-field, all-token comparison the protocol calls for; (4) no script emits any
    heap/memory number (`process.memoryUsage`/`performance.memory` — grepped, absent) — "peak heap"
    is not recorded by this harness at all.

- **Per-phase timings (ms, mean/iter; machine as above):**

  | phase | PSA (30,857 tok) | GEN (263,131 tok) | corpus (5,289,015 tok) |
  | --- | --- | --- | --- |
  | serial parse (lex+parse_lexemes) | 1.386 | 25.934 | 542.330 |
  | parallel `parse()` | 0.784 | 10.324 | 203.691 |
  | serial `decode_borrowed` | 0.800 | 14.585 | 286.882 |
  | parallel `decode_borrowed_par` | 0.534 | 9.193 | 167.719 |
  | par decode / par parse | 1.47x | 1.12x | 1.21x |

  (rayon threads = 10.) JS side (mean/iter):

  | phase | PSA | GEN |
  | --- | --- | --- |
  | `JSON.parse` → objects | 23.08 ms | 352.26 ms |
  | bin decode eager objects | 15.62 ms (1.5x) | 292.22 ms (1.2x) |
  | bin decode lazy view (0 objects) | 0.003 ms (8,616x) | 0.002 ms (193,218x) |
  | lazy view + 200-tok viewport text | 0.032 ms (731x) | 0.034 ms (10,480x) |
  | wasm serial `parse().tokens()` | 58.74 ms | 3,053.36 ms |
  | JS binary eager decode vs wasm | **3.7x faster** | **10.4x faster** |
  | worker round-trip: JSON build (baseline) | 28.27 ms | 477.86 ms |
  | worker round-trip: structured-clone N objects | 47.85 ms | 836.16 ms |
  | worker round-trip: ArrayBuffer transfer | 0.02 ms | 0.24 ms |
  | object-clone vs buffer-transfer | 2,273x | 3,531x |

  Sizes: PSA source 0.3 MB / bin 0.74 MB / JSON 4.91 MB; GEN source 5.2 MB / bin 15.69 MB / JSON
  70.81 MB. No heap numbers available (see coverage gap above).

- **Δ1 — embedded `S_SOURCE` share:** `S_SOURCE` is 272,592 B of PSA's 742,276 B total (**36.72%**)
  and 5,154,281 B of GEN's 15,688,176 B total (**32.85%**). Per owner rationale at sign-off, this
  replay's job is only to quantify that share, not to re-litigate external-source — v1 binds
  external source bytes by length+xxhash instead (§2.2#3, §7.4).
- **Δ2 — private raw marker index:** confirmed by direct source inspection of the worktree's own
  dirty diff — `src/marker_defs.rs` adds `pub(crate) fn spike_from_raw(v: u16) -> Self` and
  `pub(crate) fn spike_raw(self) -> u16` on `MarkerIndex` solely so the codec can serialize/restamp
  the private raw ordinal into the 12-byte marker descriptor (`S_MARKER_DICT` byte 10-11) and
  restore it on decode without re-resolving the marker name. This is exactly the private/raw
  marker-index dependency v1 rejects in favor of a deterministic catalog stamp + explicit stable
  discriminants (§7.4/§7.7) — the replacement mechanism is already specified in the frozen epic, so
  this is evidence to classify, not an unresolved stop.
- **Δ3 — `assert!`/unchecked indexing inventory (`src/wire_spike.rs`):** one `assert_eq!` (magic
  check, panics rather than returning a typed error); four `.unwrap()` sites (enum-index lookup,
  backslash-position lookup for marker names, two UTF-8 slice conversions); pervasive unchecked
  slice/array indexing (21+ direct `buf[i]`/`section[idx]` reads via the `rd_u16`/`rd_u32` helpers
  and section-directory offsets) with no bounds validation against buffer length anywhere in `Dec`.
  Consistent with the correctness-gate coverage gap above: nothing in the harness ever feeds a
  truncated/corrupted buffer, so this exposure is unverified by the replay itself, only inventoried
  by reading the source. v1 requires validated checked decode with a typed `DecodeError` (§7).
- **Δ4 — 10-byte SID record vs 8-byte `PackedSid`:** PSA's `S_SID_DICT` holds 2,612 unique records
  @10B = 26,120 B (3.52% of total); at a hypothetical 8B `PackedSid` that's 20,896 B, saving 5,224 B
  (0.70% of PSA's total encoded size). GEN's dict holds 1,584 records @10B = 15,840 B (0.10% of
  total); 8B would save 3,168 B (0.02% of GEN's total). The per-book absolute savings are small in
  these two books because SID dictionaries dedupe heavily; the format change is normative regardless
  of these two samples' size impact (§7.5).
- **Δ5 — token-id dictionary share:** **not measurable from this replay.** The spike's wire format
  never serializes a per-token id string at all — `TokenId` is reconstructed positionally
  (`book_code` borrowed from source + `index = i`), so there is no dictionary section corresponding
  to the "token-id dictionary" the 0E payload census measured at 31–41% of section bytes
  (`progress-braid-epic.md:671`). The spike is already, structurally, the "positional_ids" case;
  it supplies no data confirming or refuting the 31–41% figure, which came from the separate 0E
  corpus scan, not from this harness.

- **Stop thresholds:** none hit. No semantic mismatch (correctness gate green on every check the
  harness performs); no payload the packed form failed to represent in the tested corpora; the Δ2
  private-index dependency exists but its replacement (catalog stamp) is already specified in the
  frozen epic, so it's evidence to classify rather than a novel blocker; the measured boundary
  advantage is retained, not lost (JS binary decode still 3.7–10.4x faster than wasm parse+marshal,
  and buffer-transfer still 2,273–3,531x cheaper than structured-clone on GEN/PSA).

- **Reuse / rewrite / discard verdicts (Phase A components):**
  - Checked reader/writer primitives — **rewrite**: the little-endian fixed-column packing shape is
    sound, but every read site is unchecked (Δ3) and the one validation that exists panics
    (`assert_eq!`) instead of returning `DecodeError`; keep the layout idea, not these functions.
  - Header/TOC/directory validation — **rewrite**: `Dec::new`/`section_dir` trust the magic,
    section count, and every offset/length with zero bounds checking against the buffer; needs the
    v1 checked-directory contract (§7) from scratch.
  - Token columns (kinds/span_start/span_end/sid_idx/marker_idx as flat fixed-width arrays) —
    **reuse**: this is the shape the correctness gate and the boundary-cost numbers actually
    validate (exact match to `parse()` across 5.3M tokens; 3.7–10.4x decode win; 2,273–3,531x
    transfer win) — carry the column layout forward, behind checked accessors.
  - Dictionaries (marker descriptor + string interning) — **reuse the interning shape, rewrite the
    descriptor's index field**: string/descriptor dedup is cheap (≤8.37% of PSA, ≤3.36% of GEN) and
    the mechanism works, but the descriptor's `index` slot must carry the catalog stamp/stable
    discriminant (Δ2), never the private raw `MarkerIndex` ordinal.
  - SID packing — **rewrite**: drop the 10-byte record for the 8-byte `PackedSid` + fidelity bit
    per §7.5 (Δ4); the dedup-dictionary pattern around it carries over.
  - Embedded `S_SOURCE` section — **discard**: replaced by external source bytes bound by
    length+xxhash (Δ1); this replay's only job on Δ1 was quantifying the section's historical share
    (32.85–36.72%), which is now recorded.
  - JS decode (`decodeEager`/`decodeLazy`, the wasm-vs-bin and worker-transfer comparisons) —
    **reuse the concept, rewrite the implementation**: the lazy zero-object column-view pattern and
    the ArrayBuffer-transfer-vs-structured-clone result are the headline evidence for the whole
    boundary-cost thesis and should carry forward conceptually, but the JS decode itself is written
    against this exact (soon-obsolete) byte layout — external source, 8-byte SID, catalog-stamped
    marker index, conditional `positional_ids` — and must be rewritten against the v1 layout, not
    ported.

- Gate 0 closure: with 0A–0H all recorded, Phase 0 steps 2–3 (contract verification and
  discriminant/API-ledger freeze) become the next packets before Phase A code.

## 2026-07-27 — Phase 0 step 2: post-amendment consistency pass (owner-delegated review)

- Gate 0 is closed (0A–0H all recorded). Reviewed the epic end-to-end for contradictions after
  the session's ~20 adjudicated amendments. Four found and fixed, all mechanical:
  - §17#8 (native Tauri session "only if measurements justify") contradicted new §2.2#18 — marked
    superseded;
  - §12's perf-surface list still said "Tauri webview" — now the native Tauri host, with desktop
    `materialize` source-binding measured separately;
  - §6.3 state table lacked a row for validated restore — added (atomic seed; accepted primed,
    rejected stale);
  - wasm `RestoreError` was referenced but undefined — defined in §5.6 as the composition union
    {decode | sourceBinding | ingest}; per-book stamp rejections are `RestoreReport` data, not
    errors. Also added the §11.2 restore lifecycle test line.
- Verified: §2.2 numbering and all #13–#18 cross-references consistent; no remaining "or sibling"
  / "defer resident" / stale-order phrasing; BookInput::Usfm correctly carries no line_ending
  (detected, per #16); prime (#10) and restore (#17) are complementary, not duplicative.
- Phase 0 step 2 complete. Step 3 (discriminant/error-name/API-ledger freeze) is next; handoff
  drafted as `handoff-phase0-freeze.md`.
- Documentation-only; no production code changed, no tests run.

## 2026-07-27 — Phase 0 step 3: v1 freeze

Executed per `handoff-phase0-freeze.md`. Evidence/specification only — no production code, no
commits, no bless/update env vars, no git reset/clean/checkout. Deliverable:
[`phase0-freeze.md`](./phase0-freeze.md).

- **Assignment rule used throughout:** stable integers assigned in current-source declaration order
  (plan-text order for not-yet-implemented wire/braid types), starting at 0, no reserved-zero
  exception invoked; append-only forever after this freeze (additions get the next integer,
  removals tombstone rather than renumber). Bit-field tables apply the same rule to the plan's own
  prose enumeration order.
- **Table row counts:** (1) `LintCode`→`u8` 32; (2) `TokenKind`→`u8` 9 (+ `NumberRangeKind` 4,
  recorded); (3) section kind ids 2, finding flag bits 8, sentinels 4 restated; (4) field ids —
  token section 12, finding section 7; (5) `TokenFix` 3, `TokenEdit` 3, per-code message-payload
  schemas 24 of 32 codes; (6) error freeze — 36 fully-specified variants across `IngestError` (8),
  `DecodeError` (14), `PatchError` (6), `PrimeError` (3), `BaselineError` (2), `RestoreError`
  composition (3), plus 5 OWNER-DECISION error types (10 proposed variants); (7) API ledger —
  retained 97 npm + 43 dto + 423 core (pointer to 0C, not re-tallied), deleted 1 crate/0 items, new
  ~20 wire + ~45 braid + 4 wasm symbol-groups; (8) stamp definitions — 3 stamps, all 3
  OWNER-DECISION on exact inputs.
- **OWNER-DECISION rows (9 total, none decided here):**
  1. `positional_ids` flag location (container/TOC-entry/section-header flags field) — proposed
     section-header `flags:u8` bit 0.
  2. `ScopeError` variant set — proposed `BookNotFound`/`ChapterNotFound`/`AmbiguousChapter`.
  3. `LintError` variant set — proposed single placeholder `RuleExecutionFailed { book }`; no real
     core failure mode is named anywhere in the plan or Gate 0 evidence.
  4. `FormatError` variant set — proposed single `Scope(ScopeError)` composition.
  5. `ProjectionError` variant set — proposed `Scope(ScopeError)`/`Usj(UsjError)`/`Usx(UsxError)`.
  6. `EncodeError` variant set (raised beyond the handoff's four named types, same "referenced but
     never enumerated" gap) — proposed `TooManySids`/`UnrepresentablePayload`.
  7. Marker-catalog stamp input set — proposed xxhash3 over ordered `(MarkerId, canonical, family,
     kind)` tuples; open question whether crate semver alone would suffice.
  8. Lint-config fingerprint input set — proposed resolved `enabled_codes`/`disabled_codes`
     bitmasks + `suppressed` + `allow_implicit_chapter_content_verse`; `scope` proposed excluded
     (braid always lints at `LintScope::Book`, so scope is not a config axis for resident lint).
  9. Engine stamp input set — proposed catalog content hash + `format_version`/`rules_version`,
     composed with or instead of crate semver; crate-semver-alone flagged as likely insufficient
     per the plan's own §5.3 wording.
- No reality-vs-plan contradiction hit beyond what Gate 0C/0D already surfaced and the owner already
  adjudicated (e.g. §5.1 payload-legality table, §2.2#15 canonical-order re-key) — this step only
  had to mechanically number what those adjudications already settled, plus surface the four
  named-but-undefined error types (and the fifth, `EncodeError`, found by the same pattern) and the
  three stamp definitions as OWNER-DECISION.
- Documentation-only; no production code changed, no tests run.

## 2026-07-27 — owner adjudication: Phase 0 step 3 freeze — all nine OWNER-DECISIONs resolved

- Accepted as proposed: `positional_ids` in section-header flags bit 0; `ScopeError` (3 variants);
  `FormatError` (`Scope`); `ProjectionError` (`Scope`/`Usj`/`Usx`); `EncodeError` (`TooManySids`/
  `UnrepresentablePayload`); marker-catalog stamp as registry-tuple xxhash3 (semver alone ruled
  insufficient — unenforced); lint-config fingerprint with `scope` excluded; engine stamp option
  (c) (catalog hash + format/rules versions + crate semver, composed).
- One rename: `LintError::RuleExecutionFailed` → `LintError::EngineFailure { book }` — the error
  means the engine failed to complete, never that the document linted badly; near-unreachable
  placeholder for §11.2 injected-failure atomicity tests.
- Owner caveat on the fingerprint `scope` exclusion recorded in the freeze appendix: a future
  chapter/verse compute grain (with a whole-book second pass for dup-chapter/verse families) is
  engine strategy under parity gates, invalidated by the engine stamp — not configuration.
- Adjudication appendix appended to `phase0-freeze.md`. **Phase 0 is closed.** Next: Phase A
  step 1 (create `usfm_onion_wire`, move DTO types, one-way dependency proof) — first production
  code of the epic.
- Documentation-only; no production code changed, no tests run.

## 2026-07-27 — Phase A step 1: usfm_onion_wire created, dto absorbed

- Executed per `braid-epic.md` §10 Phase A step 1 and `gate0-0g-dependency-ledger.md` §6.2 (the
  authoritative concrete removal recipe). First production commit of the epic.
- Created `crates/usfm_onion_wire` (Cargo edges: `usfm_onion`, `serde`, default/`wasm` feature
  split identical to dto's — `wasm = ["dep:tsify", "dep:wasm-bindgen"]`). `git mv`d
  `usfm_onion_dto/src/lib.rs` to `usfm_onion_wire/src/dto.rs` verbatim — zero type/derive/
  attribute/rename changes. `lib.rs` is `pub mod dto;` only. All 43 public dto items (including
  the easy-to-miss `decode_attr_value`, per 0G §6.1) and the crate's own `boundary_enums_*` /
  `token_deserializes_without_span_field` drift-guard tests moved with the file.
- Repointed the one consumer: `usfm_onion_wasm/Cargo.toml`'s `usfm_onion_dto` dependency →
  `usfm_onion_wire` (`wasm` feature), and the one `pub use usfm_onion_dto::{...}` block (42 names)
  → `pub use usfm_onion_wire::dto::{...}`, same order. Updated two stale doc-comment references
  to the old crate name (module doc in `dto.rs`, one comment in `usfm_onion_wasm/src/lib.rs`) —
  no code/behavior change. Deleted `crates/usfm_onion_dto`. Workspace `members = [".", "crates/*"]`
  needed no edit (glob).
- `cargo metadata` confirms the target DAG from 0G §4/§5.2: `usfm_onion` is a sink; `usfm_onion_wire`
  depends only on `usfm_onion` (+ serde/tsify/wasm-bindgen); `usfm_onion_wasm` depends on both. No
  reverse edges.
- Declaration-neutrality proof: built both wasm-pack targets (`--profile wasm-release-fast`,
  matching the ships build) into a scratch `--out-dir` under `/tmp` and diffed the generated
  `.d.ts` against the committed `pkg-bundler`/`pkg-web` trees — **both diffs empty**. A later
  in-place `--dev` rebuild (needed to run `npm run test:wasm`/`golden:wasm`/`golden:wasm:web`, which
  drive `build:wasm:*:dev` as a side effect) showed the pre-existing, already-documented
  `InitOutput`-only divergence between profiles (0G §7.1) — unrelated to this move. Restored
  `pkg-bundler`/`pkg-web`/`Cargo.lock` to clean committed state after the dev-profile test runs
  (they carried zero uncommitted changes beforehand), then regenerated `Cargo.lock` via `cargo
  build --workspace` so it legitimately reflects the new `usfm_onion_wire` member before committing.
- Gates, all green, no `BLESS=1`/`UPDATE_GOLDEN=1`, no git reset/clean/checkout/stash beyond the
  two build-artifact paths noted above: `cargo build --workspace`; `cargo test --workspace` (247
  passed, 12 ignored, 0 failed); `cargo test --test lint_oracle -- --ignored` (1 passed);
  `npm run check:wasm:web`; `npm run test:wasm` (bundler + web); `npm run golden:wasm` (7
  fixtures); `npm run golden:wasm:web` (7 fixtures); `npm run test:token-sids:import`.
- No deviation from the plan or the 0G recipe. Commit `b4596e3` — `refactor(wire): absorb
  usfm_onion_dto into usfm_onion_wire`. Did not touch the owner's unrelated in-flight work
  (`js/token-sids.*`, `scripts/test-token-sids.mjs`, `src/diff/*`, `alloc_sizes.txt`, deleted
  `plans/approved/braid-epic.md`/progress file, untracked `.claude/`, `bench-remote.sh`,
  `plans/approved/braid/`).

## 2026-07-27 — Phase A step 2: wire schema constants, errors, checked container primitives

- Executed per `braid-epic.md` §7.1–7.3 / §5.6 / §13 and `phase0-freeze.md` §3, §4, §6.2, §6.8 plus
  its adjudication appendix. Single commit `b08b9aa` — `feat(wire): frozen schema constants, typed
  errors, checked container codec`. Base commit `01fb2cd`.
- New files, all in `crates/usfm_onion_wire`: `src/schema.rs` (268 lines, frozen constants),
  `src/error.rs` (202, `DecodeError`/`EncodeError`), `src/primitives.rs` (241, checked LE
  cursor/writer + checksum, crate-private), `src/container.rs` (839, header/TOC/section/directory
  readers + canonical writer), `tests/container.rs` (885, 49 cases). `src/lib.rs` grew the module
  declarations, a crate doc, and a re-export of the two error enums. `Cargo.toml` gained the one new
  dependency; `Cargo.lock` updated. No token/finding columns (steps 3–4). Gate 0H honoured: nothing
  was copied from the spike worktree, which was not opened.
- Dependency choice: `xxhash-rust = { version = "0.8", features = ["xxh3"] }` — §7.1's checksum
  algorithm, pure Rust, no build script or C shim, so `wasm32-unknown-unknown` compiles unchanged
  (verified). No other dependency needed.
- Reader shape: `read_container` validates header + container checksum + whole TOC, and is the only
  way to obtain a `Container`; `Container::sections()`/`section(i)` validate a section header +
  directory on demand and are the only way to obtain a `Section`. `SectionField` carries an
  already-validated payload slice, so column decoders in later steps repeat no arithmetic. Ordering
  is bounds-before-capacity everywhere: `Vec::with_capacity` is reached only after `window()` proved
  the claimed bytes exist. Cross-entry TOC rules (overlap, duplicate `(kind, book)`, finding→token
  pairing) go through sorted key vectors, not pairwise scans, because `section_count` is
  producer-chosen and a quadratic check would be a DoS channel.
- Writer shape: canonical **by construction**. Section order is derived by iterating the input once
  per kind (token sections in caller/corpus order, then finding sections), fields sort ascending by
  id, offsets are computed from emitted length, gaps are zero-filled, and there is no map iteration
  or address-dependent ordering — the spike's unseeded-HashMap nondeterminism cannot recur. Illegal
  headers are unrepresentable rather than validated: `SectionVariant::{Token{positional_ids},
  Finding{rules_version}}` makes "token section with a rules version" and "finding section with a
  token-only flag" unconstructible, and `ElementWidth` is a closed enum so no unsupported stride can
  be named in either direction.
- Verification, all green, no `BLESS=1`/`UPDATE_GOLDEN=1`, no git reset/clean/checkout/stash:
  `cargo test --workspace` (core 247 passed / 12 ignored, wire container suite 49 passed, all other
  targets pass, 0 failed); `cargo test --test lint_oracle -- --ignored` (1 passed);
  `npm run check:wasm:web` (wasm32 target builds core + wire-with-`wasm`-feature + wasm crate);
  additionally `cargo check -p usfm_onion_wire --target wasm32-unknown-unknown --features wasm`.
  `cargo clippy -p usfm_onion_wire --all-targets` clean (only pre-existing core warnings remain).
  `rustfmt` applied to the five new/changed Rust files only — `src/dto.rs` is not `cargo fmt`-clean
  in the committed tree (pre-existing, moved verbatim in step 1) and was deliberately left alone, so
  package-wide `cargo fmt` was not run. Did NOT run `wasm-pack` builds: this step adds no
  wasm-bindgen export, so the generated `.d.ts` cannot move, and a dev-profile rebuild would dirty
  the committed `pkg-bundler`/`pkg-web` trees.
- Owner's unrelated in-flight work untouched (`js/token-sids.*`, `scripts/test-token-sids.mjs`,
  `src/diff/*`, `alloc_sizes.txt`, `lib_sizes.txt`, deleted `plans/approved/braid-epic.md` and
  progress file, untracked `.claude/`, `bench-remote.sh`, `plans/` additions). Committed via explicit
  paths only; `plans/approved/braid/` remains untracked.
- **Decisions taken that the freeze document does not cover** — each is a forced choice, flagged for
  owner confirmation, none contradicts a frozen value:
  1. **Field-directory `flags:u8` bit 0 = `required`.** §7.3 requires "unknown required field ids
     reject; unknown optional fields may be skipped", which is undecidable for an id the decoder has
     never seen unless the entry itself declares required-ness. Bit 0 carries it; all other bits
     reject. For a *known* id the bit must agree with the frozen per-kind table, so a producer
     cannot relabel a schema field. No freeze table assigned bits to this field.
  2. **`element_width = 0` means "variable/mixed".** The freeze's field tables give element width as
     "mixed" for the dictionary and sparse-record fields, which a `u8` cannot express otherwise. 0 is
     reserved for it: `byte_len` is authoritative, alignment is 1. Legal widths are therefore
     {0,1,2,4,8}. **Related open item for step 4:** §7.6's finding common row is a fixed 16-byte
     record, which is a uniform column of width 16 — not in the legal set, so it is currently
     expressible only as variable-width (alignment 1). Either add width 16 or split the row into
     narrower columns; deferred to step 4 rather than guessed now.
  3. **`EncodeError::InvalidSectionLayout { book, reason: LayoutRefusal }`** appended. §6.8 froze
     only `TooManySids` and `UnrepresentablePayload`, which are payload refusals; §7.2's "the
     complete-corpus encoder refuses duplicate `BookId` sections" has no frozen variant. One
     appended variant with a typed reason enum (9 reasons, each mirroring a reader check) keeps the
     freeze footprint to a single append under the freeze's own append-only rule.
  4. **Canonical section order read as grouped, not interleaved.** §7.1 says "ordered token sections
     followed by corresponding finding sections in corpus order"; implemented as all token sections
     in corpus order, then all finding sections in corpus order. Readers accept any TOC order, so
     this only fixes canonical bytes. Reversible in one place if the interleaved reading was meant.
  5. **Check-to-`DecodeError` mapping.** The freeze fixed variant names, not which check yields
     which. Documented at the top of `error.rs`: `Truncated` = in-range arithmetic running off the
     buffer; `OffsetOverflow` = the arithmetic itself overflowing or exceeding the host's address
     range; `InvalidToc` = TOC-level rules; `InvalidSection` = a header/directory contradicting
     itself or its TOC entry (this is what a v1 container declaring a header length other than 32
     returns, since no frozen variant names header-shape errors).
- No reality-vs-plan contradiction found in §7.1–7.3 itself; every deviation above is a gap the
  freeze did not reach, not a conflict. Next: Phase A step 3 (token section columns — encode/decode
  of `kind`/spans/sid/marker indices, dictionaries, sparse records) which is where item 2's width
  question and the packed-SID layout (§7.5) come due.

## 2026-07-27 — Phase A step 2 review and owner adjudication

- Independently reviewed commit `b08b9aa` against the container contract, with focused probes for
  corrupt-input allocation/panic safety and deterministic output. The narrow suite remained
  deterministic and no panic or unchecked decode arithmetic was found.
- Fixed two confirmed trust-boundary gaps: unknown optional fields now undergo the same
  id/range/extent/alignment/overlap checks before being skipped, and the normal persistent reader
  rejects omitted container or section checksums. Checksum omission is reachable only through an
  explicitly named transient internal reader.
- Applied all five owner-approved rulings to code and the normative freeze: field requiredness bit,
  fixed width 16, appended structural `EncodeError`, grouped canonical section order, and explicit
  check-to-`DecodeError` mapping. Added fixed-width validation for known fields.
- Kept the raw container reader/writer and construction types crate-private; the frozen public API
  is the later semantic `encode_corpus`/`decode_borrowed` surface, not a second raw-layout API.
  Moved the malformed-input battery from an integration target to a crate unit-test module so it
  can exercise that private trust boundary.
- Verification after review fixes: `cargo test -p usfm_onion_wire` — 70 passed, 0 failed. Full
  workspace/wasm gates remain for the next committed slice.

## 2026-07-27 — Phase A step 3a: fixed token columns and packed SID

- Added explicit stable `TokenKindTag:u8` and `NumberRangeKindTag:u8` conversions. They do not rely
  on Rust enum representation and exhaustively test the frozen declaration-order tables.
- Added an internal zero-copy `TokenColumns` view over the fixed per-row fields. It requires every
  fixed column count to equal the section token count, rejects unknown token-kind discriminants,
  enforces explicit token ids when `positional_ids` is clear, and uses checked little-endian row
  accessors. Source-bound span validation remains with `decode_borrowed`, where source bytes exist.
- Added the explicit eight-byte `PackedSid` codec: `book[3] | chapter:u16 | verse:u16 |
  delta_and_fidelity:u8`. Exact delta 127 round-trips; the high fidelity bit remains meaningful
  with a low-seven-bit delta; wider deltas degrade to an anchor-only first anchor. The codec takes
  source-derived fidelity explicitly rather than guessing from core `Sid`.
- Did not implement mixed string/marker dictionaries or sparse number/book/attribute records. Their
  directory ids and semantic contents are frozen, but their internal offset/record byte layouts and
  marker-catalog stamp envelope are not specified precisely enough to write an independent decoder.
  That is the next normative-layout amendment, not an implementation guess.
- Verification: `cargo test --workspace` (core 247 passed / 12 ignored; wire suite and all other
  targets green), `cargo test --test lint_oracle -- --ignored` (1 passed), and wasm32 wire check
  with `--features wasm` all green. `cargo clippy -p usfm_onion_wire --all-targets` reports only
  the five pre-existing core warnings; the wire crate adds none. A `-D warnings` probe stops on
  those same core warnings before checking wire.

## 2026-07-27 — Phase A layout stop before steps 3b–4

- The semantic contract requires four values for which the frozen byte tables allocate no storage:
  global `snapshot_id`; per-book exact source length; marker-catalog stamp; and the packed SID
  dictionary itself (field 4 is only the `u16` index column, while token field ids 0–11 contain no
  SID-dictionary payload). None can be reconstructed safely from the existing checksum/hash/index.
- The mixed payloads also still lack exact byte records: string/token-id dictionary framing,
  marker descriptors, and sparse number/book-code/attribute records. Their semantic names and
  directory ids do not define an independently implementable decoder.
- Serial `encode_corpus`/`decode_borrowed` therefore stop here. The next normative amendment must
  allocate the four missing values (header extension versus explicit metadata fields) and freeze
  each mixed record's byte shape, count meaning, canonical ordering, reserved bytes, and validation
  errors. Implementing the historical spike's shapes by implication would create undocumented v1
  wire law and is explicitly out of bounds.

## 2026-07-27 — Core `OwnedToken` foundation

- Added core-owned `StableTokenId`, `OwnedToken`, `OwnedNumberInfo`, `OwnedBookCode`, and
  `OwnedAttribute` without serde/tsify/wasm dependencies. A private discriminated payload prevents
  illegal marker/end-marker/milestone/number/book-code combinations.
- `OwnedToken::from_parsed` preserves stable positional ids, source, semantic payload, marker
  metadata, verbatim attribute lists, and both native SID spellings: lint's chapter-scope `GEN 1`
  and diff's `GEN 1:0`. It implements the existing serialization, walker, lint, and diff traits.
- Review found and fixed the verse-zero SID mismatch before commit. A second clean-room review found
  no panic path or illegal-state escape, but correctly marked this as a foundation rather than a
  resident-ready token: `FormattableToken` remains blocked on how formatter-created synthetic
  tokens receive nonempty, book-unique stable ids, and resident lint remains blocked on the already
  approved token-position canonical-sort change. The DTO construction/`TokenInputError` seam is
  likewise deferred rather than guessed.
- Verification: full workspace green (core 250 passed / 12 ignored; wire 80 passed; wasm 25
  passed; all integration suites green), ignored lint oracle 1 passed, and core Clippy reports only
  the same five pre-existing warnings.

## 2026-07-28 — Phase A: wire layout amendment, framing freeze, SID dictionary

- Three commits: `2e7096c` (layout amendment, code + spec together), `8f2594c` (mixed-payload
  framing freeze, docs only), `4bfdd2a` (packed SID dictionary column). Base `f000792`.

**Layout amendment (`2e7096c`)** — executes rows 1–3 of the 2026-07-28 adjudication as one commit,
so no offset ever disagrees between spec and code:

- Container header 32 → 48 bytes: `snapshot_id:u64` at 32, 8 reserved zero bytes at 40. Canonical
  TOC offset moves 32 → 48 (still 16-aligned). `write_container` now takes `snapshot_id` as its
  first argument, mirroring `encode_corpus(snapshot_id, sections)`; wire stores it verbatim.
- Section header 48 → 64 bytes: `source_len:u64` at 48, `catalog_stamp:u64` at 56. Both are per
  **section**, not per container — one container holds several books, each with its own source
  bytes and possibly its own stamp, of which only mismatched sections are rejected. 64 stays a
  multiple of the 16-byte section alignment, preserving "section-relative offset has the same
  alignment as absolute offset".
- Token field id 12 appended: packed SID dictionary, required, fixed element width 8. Required even
  when empty (`count = 0`), so presence is never inferred from index-column contents.
- Reserved bytes reject when nonzero, mapped to `InvalidSection` for consistency with the existing
  section-header `reserved[3]` rule rather than inventing a second policy for the same class of
  violation.
- Documents amended in the same commit: `phase0-freeze.md` gained normative layout tables (L.1–L.4),
  token field row 12, and corrected row counts; `braid-epic.md` §7.1/§7.3 tables and the §7.4
  required-field list now match, each citing the adjudication that authorized the change.
- Latent bug the growth exposed and fixed: both integrity checksums were stamped through an
  open-ended slice range (`header[OFFSET..]`), correct only while the checksum happened to be the
  last header field. It now uses an explicit 8-byte bound. Caught by the suite, not by review.

**Framing freeze (`8f2594c`)** — executes row 4; docs only, so no column is implemented against an
unadjudicated shape. Specifies the UTF-8 string dictionary (fields 9/10: `[u32; count]` starts plus
concatenated data, implicit final bound, per-string UTF-8 validation so a split code point cannot
pass, `count == 0` ⇒ `byte_len == 0`), the marker descriptor dictionary (field 11), and the sparse
number (field 6, 16-byte records), book-code (field 7, 16-byte records), and attribute (field 8, two
ascending arrays with a derived second count) records — each with `count` meaning, canonical
ordering, reserved bytes, sentinels, and validation order.

- Load-bearing finding: **marker metadata does not belong on the wire.** `MarkerMetadata` and
  `StructuralMarkerInfo` are pure functions of the marker name in core
  (`token::marker_metadata`, `marker_defs::structural_marker_info`), so a descriptor is a name index
  plus one flag and the decoder calls core's own functions (§4.3 reuse rule). Consequences: five
  core enums (`MarkerDefKind`, `MarkerFamily`, `StructuralScopeKind`, `InlineContext`,
  `SpecContext` — 57 variants) never need stable wire tables in v1; `MarkerMetadata.canonical` is
  recoverable at all only this way, being an `Option<&'static str>` catalog pointer that cannot be
  rebuilt from bytes; and the recovery is stamp-gated, trading "sections do not survive a catalog
  change" for "never carry a stale metadata copy that disagrees with the engine reading it".
  Unknown markers need no special case — core already returns all-`None` metadata and
  `scope_kind: Unknown` for a name the catalog does not know.
- Deliberate exception: book-code `is_valid` is stored, not recomputed, because core's
  `is_valid_book_code` is `pub(crate)` **and** the canonical book list is not covered by the
  marker-catalog stamp, so recomputing would let a change to that list silently rewrite an
  already-encoded token.
- **Five OWNER-DECISION rows, framed not decided** (freeze appendix §D.6): (1) where `nested`
  lives — descriptor flag (proposed) vs a new per-row column, noting that `nested ==
  name.starts_with('+')` holds for parsed tokens but is a lexer property, not a format invariant;
  (2) attribute source as span (proposed, per §7.4's wording, zero-copy, but a synthetic token whose
  attribute source is not a substring of the bound source needs a typed encode refusal) vs
  dictionary string; (3) whether an absent attribute source is semantically distinct from an empty
  one, as core's `Option<Box<str>>` implies; (4) **the finding section's `marker_string_idx` (finding
  field 4) has no dictionary allocated to index** — needs either a finding-section string dictionary
  (field id 7) or a defined cross-section reference to the token section's field 10; this blocks the
  finding columns; (5) whether fields 6/7 should declare fixed width 16 now that their records are
  uniform, which would change frozen `FieldSpec` rows and so is listed rather than applied.

**SID dictionary column (`4bfdd2a`)** — encode + validated decode for field 12:

- `SidDictionary` proves the column at construction (count within the index ceiling, every record
  decodes to a legal book code); `TokenColumns` establishes the row-side invariant once (every
  `sid_index` is the `0xffff` sentinel or names an existing record). Together those make
  `TokenColumns::sid(row)` return a plain `Option`, not a `Result`.
- Ceiling is 65,535 records (highest addressable index 65,534) because the sentinel consumes one
  value. Checked before walking records, so an inflated count costs one comparison instead of a pass
  over half a megabyte. `DecodeError::TooManySids` on read, `EncodeError::TooManySids` on write —
  the writer refuses rather than reusing the sentinel.
- `SidDictionaryBuilder` interns each distinct `(anchor, fidelity)` pair once, assigning ordinals in
  first-use order, so output is a function of token order alone; its `BTreeMap` is a lookup index
  that is never iterated, so no map ordering reaches the bytes. That assignment rule is recorded in
  the freeze's field-12 table.
- Tests: dictionary at 0/1/exactly-the-ceiling entries, one past the ceiling, index past the last
  record, sentinel winning over a non-empty dictionary, an undecodable record rejecting the whole
  column, builder dedupe/fidelity-sensitivity/order-determinism/ceiling refusal, and a full
  intern → write → read round trip preserving book, chapter, verse, bridge end, and fidelity.

**Gates, green at each of the three commits, no `BLESS=1`/`UPDATE_GOLDEN=1`, no git
reset/clean/checkout/stash:** `cargo test --workspace` (core 250 passed / 12 ignored; wire 93
passed; wasm 25 passed; every integration target green; 0 failed); `cargo test --test lint_oracle --
--ignored` (1 passed); `npm run check:wasm:web` (wasm32 builds core + wire + wasm crate).
`cargo clippy -p usfm_onion_wire --all-targets` adds no wire warning (only the five pre-existing
core ones). `rustfmt` applied to changed wire files only; `src/dto.rs` remains the pre-existing
non-`cargo fmt`-clean file and was left alone.

- Untouched: the owner's untracked leftovers (`handoff-*.md`, `alloc_sizes.txt`, `lib_sizes.txt`,
  `bench-remote.sh`, `.claude/`). Every commit used explicit paths.
- Next: owner review of the §D.6 rows, then the string dictionary, marker descriptor, and sparse
  record columns. Row 4 (finding `marker_string_idx`) must be settled before any finding column.

## 2026-07-28 — Phase A: finding-marker verdict, remaining token columns, token-section gate

- Two commits: `3d435bd` (adjudications applied + finding-marker freeze) and `b274090` (all remaining
  token columns + whole-section codec + Phase A corpus gate). Base `8cc7cb9`.

### Finding `marker_string_idx` — premise rejected, no string dictionary

- `LintIssue.marker` is written in exactly **three** places in `src/lint_impl.rs`, so the producer set
  is closed: `issue()` `:2356` copies the anchored token's own marker; `simple_issue_with_marker()`
  `:2427` takes a caller-supplied `&str` (5 call sites); and the `missing-id-marker` literal at
  `:1437`. The five override sites pass `"c"` (`:1518`), `"v"` (`:1572`, `:1599`, `:1613`), and — for
  `unknown-token` `:1900` — a `[a-z0-9-]+` slice of the anchored token's **own source**, guarded on
  the line above by `lookup_marker(marker).kind != MarkerKind::Unknown`, so it is a catalog marker by
  construction and a source slice as well.
- Corpus evidence (`cargo test -p usfm_onion_wire --test corpus -- --ignored`, 262 testData fixtures
  + `en_ult` + `en_ulb`): **62,948 findings, 62,945 with a marker, 31,928 via the anchored token,
  31,017 via a catalog ordinal, 0 needing a span, 0 falling through.** `unknown-marker`'s 13,953
  non-catalog names cost nothing — they arrive on the anchored-token arm, whose name the row's own
  descriptor already carries. `unknown-token` and `invalid-number-range` have zero corpus occurrences
  (token-path-only per 0D §2.1) and are covered by the static argument.
- Frozen (freeze §M): finding field **4** redefined before implementation from `marker_string_idx` to
  `marker_ref` — optional per-row column, fixed width 8, `{tag:u8, span_len:u8, ordinal:u16,
  span_offset:u32}` over tags 0 = anchored token, 1 = catalog ordinal, 2 = source span, 3 =
  explicitly absent. Tag 2 has no current producer (frozen so a future non-catalog off-token marker
  needs no version bump); tag 3 keeps "no marker on a token that has one" encodable. **No
  finding-section string dictionary; finding field id 7 stays unassigned.** Two drift guards: the
  ignored corpus test above, plus an unignored assertion that `id`/`c`/`v` remain catalog markers.

### Adjudications applied (`3d435bd`)

- `nested` stays a descriptor flag. Attribute source is a span with a typed encoder refusal. Absent
  vs empty attribute source is distinct, encoded as sentinel offset `0xffff_ffff` (the framing's
  presence-flag bit was removed in favour of the sentinel the owner named). Fields 6/7 declare fixed
  element width 16.
- **Extended beyond the two named rows:** field 11 (marker descriptors) also declares fixed width 8,
  since the framing fixed its record at 8 bytes. Same argument as ruling 5 — a uniform array should
  let the generic container enforce `count * width == byte_len`. Flagged here rather than applied
  silently. Only fields 9/10 (offset array + character data) and field 8 (two arrays of different
  record sizes) remain `element_width = 0`.

### Remaining columns and the token-section codec (`b274090`)

- `token_payload.rs`: string dictionary, marker descriptors, sparse number/book-code/attribute
  records, each with a validate-everything-at-construction view and a first-use-order builder.
  Sparse lookup is a binary search over ascending `token_idx` — no map, no allocation. Two
  correctness details found while testing: string offsets are now validated as a **set** before any
  slice is decoded (a descending offset previously surfaced as `InvalidUtf8` instead of the shape
  violation it is), and each string is decoded individually because a whole-region UTF-8 check
  accepts an offset that splits a code point.
- `token_codec.rs`: `encode_token_section` (parsed tokens + exact source) and `decode_token_section`
  reconstructing core's own `Token<'a>`. Marker metadata and structural info are **recovered** by
  calling `marker_metadata` / `structural_marker_info`, never stored — which is what the section's
  `catalog_stamp` gates. Binding order: source length, then content hash (catches same-length
  different bytes), then stamp; and a hash match is not proof — every span still goes through
  `str::get`, which rejects an out-of-range span and a split character in one lookup.
- `catalog.rs`: the stamp is a content hash over the ordered marker registry (ordinal, marker,
  canonical, kind, family, category, each length-prefixed), because nothing in the build enforces a
  crate-version bump per registry edit.
- **Core change, one line of visibility:** `usfm_onion::parse::assign_ids` is now `pub`. The wire
  omits the id column for parsed books and reproduces positional ids by calling the same function
  parsing used. The alternatives were storing the column the layout deliberately omits (31–41% of
  section bytes per Gate 0E) or reimplementing the rule in the wire crate — the named algorithm-fork
  footgun. Flagged as a deliberate public-API addition for the ledger.
- Two appended `EncodeError` variants: `TooManyDescriptors { book, found }` (mirrors `TooManySids`
  for the descriptor ceiling) and `UnboundSpan { book, token_idx }` (the adjudicated refusal for a
  token or attribute source that does not bind; unreachable through serialize-then-encode).

### Gates

- `cargo test --workspace`: core 250 passed / 12 ignored, **wire 117 passed / 1 ignored**, wasm 25
  passed, all integration targets green, 0 failed.
- `cargo test --test lint_oracle -- --ignored`: 1 passed. `npm run check:wasm:web`: green.
  `cargo clippy -p usfm_onion_wire --all-targets`: no wire warnings.
- **Phase A token-section gate** (`cargo test -p usfm_onion_wire -- --ignored`, 44 s debug / 2.4 s
  release — hence `#[ignore]`, same rationale as the lint oracle): all `testData/**/*.usfm` plus
  `en_ult` and `en_ulb` — **395 books, 5,716,969 tokens, zero mismatches.** Equality is `Token`'s own
  `PartialEq` (every public field including `MarkerMetadata` and `StructuralMarkerInfo`), plus
  `OwnedToken::from_parsed` parity on the narrow cases. The gate also counts verse bridges wider than
  127 verses — the one documented lossy packed-sid path — and asserts the count is **0**, so it
  cannot pass by never reaching the lossy branch.
- Rejection coverage: wrong source length, same-length wrong bytes, catalog-stamp mismatch, and
  thirteen hand-corrupted payload violations (offset ordering, split code point, dangling name index,
  reserved bytes, unknown flags, unknown discriminant, dual-encoded absent end, out-of-range token
  index, non-partitioning attribute rows, out-of-source span, default-shorthand with a key, a Number
  row with no record, attributes on a row that cannot carry them). Truncation at every byte and
  single-byte corruption at every offset assert no panic.

### Process note

- While formatting, `rustfmt` on `lib.rs` recursed into `src/dto.rs` and reformatted it — the
  pre-existing non-`cargo fmt`-clean file that must stay untouched. Caught in `git status` before
  committing and restored byte-for-byte from `HEAD` with `git show HEAD:… > …` (a file write, not a
  git state change). `dto.rs` is unchanged in both commits. Also amended `b274090` once, immediately
  after creating it, because the first attempt omitted `schema.rs` and so would not have compiled;
  amending was preferred to leaving a broken commit in history.
- Owner's untracked leftovers untouched (`handoff-*.md`, `alloc_sizes.txt`, `lib_sizes.txt`,
  `bench-remote.sh`, `.claude/`). Working tree carries zero modified tracked files.

### Open / next

- **New stop for owner review:** `encode_token_section` takes parsed borrowed `Token<'a>` because
  spans are required columns and `OwnedToken` is spanless by design. §5.1's `CorpusSectionInput`
  promises `tokens: &'a [OwnedToken]`, and `tokens_to_usfm` shows the concatenation of `token.source`
  is **not** the source (attribute-list slices are re-emitted at their own spans), so spans cannot be
  derived by a running offset over owned tokens. Encoding an owned/live token stream needs a decision:
  derive spans from core's spanless reconstruct emitter, or carry spans on the owned token. Not
  guessed here.
- Then: finding section columns (`marker_ref` per §M, common row, sidecars, patch table).

## 2026-07-28 — Phase A: emitter-derived spans and the owned-token encode path

- Four commits: `ed6de7c` (core span-capturing emitter), `84ed853` (wire owned encode path + second
  corpus gate), `504e167` (encode API respec + API ledger), `708403d` (standalone `cargo fmt` of
  `dto.rs`, now permitted). Base `0384ef6`.

### Core (`ed6de7c`) — four new public items, zero byte changes

- `tokens_to_usfm_reconstruct_spanned(&[T]) -> (String, Vec<ReconstructedSpans>)`,
  `ReconstructedSpans`, and `OwnedToken::parsed_sid()`. Plus `parse::assign_ids` from the previous
  packet, recorded in the freeze's API ledger with rationale for all four.
- `ReconstructedSpans` carries the token span and the attribute-list span **separately** because
  neither is derivable from the other: a deferred list is emitted at its closer, which may be
  thousands of bytes after the marker that owns it. That is also why only this emitter can report
  them and why a caller cannot compute spans from a running offset over `token.source()`.
- Both entry points share one implementation, so the spanned variant cannot drift from the plain one;
  the recorder is an `Option`, so the plain (per-keystroke editor) path pays nothing for spans it
  would discard.
- `parsed_sid()` exposes the compact anchor the eight-byte packed sid is built from. Deliberately not
  routed through `DiffableToken::sid_key()`, which also carries a `Sid` but is documented as a
  partition key — using a hash/compare helper as the canonical accessor would have been exactly the
  cleverness the adjudication warned against.

### Wire (`84ed853`) — owned encode path

- `encode_owned_token_section` = serialize → take spans from that pass → feed the existing encoder.
  Returns the derived source, because it (not any file on disk) is what the section's spans and hash
  are bound to. `owned_to_borrowed` rebuilds borrowed tokens over the serialized source so there is
  one encoder, not two; metadata comes back from core's registry by name, as decoding does.
- Attribute entry spans are found by a left-to-right scan inside the emitter-reported list span
  (entries are substrings of a verbatim list). An entry whose text is not in its own list is refused
  with `UnboundSpan` — the adjudicated synthetic-token guard.
- The verbatim list is read back out of the serialized source rather than off the input token: after
  serialization it is genuinely there, which makes the loop idempotent (decode == parse of that
  source). Absent stays distinct from empty — a token with neither a list nor an entry produces no
  attribute row at all — and that distinction is asserted at the byte level, since a parse cannot
  produce `Some("")`: a present zero-length span decodes to `Some("")`, the sentinel offset to `None`.

### Gates

- `cargo test --workspace`: core 251 passed / 12 ignored, wire **123 passed / 2 ignored**, wasm 25,
  all integration targets green, 0 failed. `lint_oracle -- --ignored`: 1 passed.
  `npm run check:wasm:web`: green. `cargo clippy -p usfm_onion_wire --all-targets`: no wire warnings;
  core still reports only its five pre-existing ones.
- **Parsed corpus gate**: 395 books, 5,716,969 tokens, zero mismatches, zero wide bridges.
- **Owned corpus gate** (new): 395 books, 5,716,969 tokens, **1,263,854 attribute-bearing tokens**,
  376 books byte-identical, 19 divergent. Universal assertions: the codec returns exactly the tokens
  it was given, and recording spans changes not one byte of the emission.

### STOP / correction — the emitter is not universally lossless

- The packet's premise "for parsed-origin tokens the derived source must be byte-identical (the
  emitter is lossless)" is **false for 19 of 395 corpus fixtures**, and this is pre-existing, not
  caused by the span capture:
  - The spanned emitter's bytes equal the untouched plain `tokens_to_usfm_reconstruct`'s on every one
    of the 395 (now a permanent gate assertion, which is what attributes the divergence).
  - The span-based `tokens_to_usfm` reproduces all 19 exactly, and core's own doc comment on
    `tokens_to_usfm_reconstruct` already names the class: "a malformed/unclosed attribute-bearing
    marker's attribute list lands at end-of-stream here … instead of at its original byte position".
- All 19 are one shape — a deferred attribute list emitted at its **closer**, which for these inputs
  is not where the list started: `FigureNotClosed` (`\fig` never closes), `Wordlist…ProperNoun…` (the
  list belongs to a nested `\+pn` whose closer precedes it), `newline-attributes` (a list containing a
  newline is split by the parser), `synthetic/kitchen-sink` (a `\list` milestone of the same shape),
  and 15 `usfmjsTests/*oldformat*` alignment fixtures whose `\k-s | x-tw="…"` sits lines above its
  `\k-e\*`.
- Handled, not papered over: the gate asserts the round trip is faithful for **all** 395 (decoded ==
  the tokens encoded, because the section describes the derived source), asserts byte-identity for the
  376, and asserts the divergent set equals an enumerated 19-path constant with each cause documented.
  Listing rather than counting means a twentieth has to be understood deliberately.
- **Format consequence recorded in the freeze**: an owned-encoded section is bound to the derived
  source, not to any file on disk. A caller persisting one alongside an original file must treat the
  derived source as the authoritative pairing. First noticed because the first gate run stopped at the
  4th file; the full sweep found 19.

### Process

- `cargo fmt`/clippy cleanups now permitted, kept in their own commit: `708403d` formats `dto.rs`,
  which had been non-`cargo fmt`-clean since the verbatim crate move and had already cost one
  accidental reformat. `cargo fmt --check -p usfm_onion_wire` is now clean, so the whole package can
  be formatted safely. Left alone: `src/diff/text_diff_fixtures.rs` (owner's area, not mentioned) and
  the five pre-existing core clippy warnings.
- Owner's untracked leftovers untouched; every commit used explicit paths; zero modified tracked files
  at the end.

### Next

- Finding section columns: `marker_ref` per freeze §M, the common row, the sidecars, and the packed
  patch table.

## 2026-07-28 — attribute-position fidelity: the owned emitter is byte-lossless

- Two commits: `1143ac1` (core field + emitter + gate, with the approved plan file) and `94a4158`
  (freeze/epic + API ledger). Base `52f08ac`. Executes
  `plans/approved/attribute-position-fidelity.md`, adjudicated pre-Phase-F on conceptual-correctness
  grounds.

### Chosen encoding: `attribute_offset: Option<BytePos>` on `OwnedMarkerAttrs`

- **Distance from the end of the owning token's own source to the start of its attribute list**,
  populated by `OwnedToken::from_parsed` from the real parsed spans (`checked_sub`, so a list recorded
  before its owner falls back rather than pretending to be zero).
- Rationale in one line: one `u32` distance covers every observed shape while an
  opener-adjacent/closer-adjacent enum cannot — `\w \+pn Proper Noun\+pn*|keyword\w*` puts the list
  *after* the owning marker's closer, which no closer-relative variant can name — and a distance from
  the owner (not an absolute offset) stays meaningful when other tokens in the stream are edited.
- Shapes covered, all four classes verified in core tests: opener-adjacent alignment (`\k-s | x-tw=…`,
  offset 0, closer lines below); ordinary closer-adjacent (`\w abc|k="v"\w*`, offset 3, past the
  intervening text); past-the-nested-closer (`\+pn` above); unclosed `\fig` (no closer ever arrives);
  newline-split list (remainder parses as text).
- **No stop needed** — no shape required approximation. The offset representation the plan preferred
  turned out to be sufficient on its own; no per-quirk variants were added.

### Emitter

- `Pending` gains `target: Option<usize>` = output position at which a remembered list is due. Before
  each token the emitter drains due targets in ascending order, then applies the historical closer rule
  to entries with no target; end-of-stream drains due targets then flushes the remainder LIFO.
- This is deliberately the *same* rule the span-based `tokens_to_usfm` already applies to absolute
  offsets — one concept in two coordinate systems, not a second algorithm. Both `reconstruct` entry
  points still share one implementation and the span recorder is still optional.
- `SerializableToken::attribute_offset()` is **defaulted to `None`**, so positionless ingest (wire
  DTO, editor `lexicalToTokens`) keeps today's closer-adjacent behavior and no implementor outside core
  changes. There is no `TokenDto → OwnedToken` conversion in the tree yet; when it lands it gets the
  default for free. Covered by a test using a purpose-built positionless token type, so the default is
  asserted directly rather than inferred.

### Completion criterion — met

- Owned corpus gate: **`byte_exact=395 diverged=0`** (was 376/19). The enumerated divergence constant
  is now `[&str; 0]`, kept as an empty list rather than deleted so a regression names the fixture
  instead of failing a count; `diverged.len() == 0` is asserted alongside it.
- Parsed-path behavior unchanged: `tokens_to_usfm` untouched, lint oracle byte-identical. What did
  change is that `tokens_to_usfm_reconstruct` now *agrees* with the span-based emitter on the four
  previously divergent shapes — asserted directly in core's existing parity module, which is a
  strictly stronger invariant than before.

### Gates

- `cargo test --workspace`: core **255 passed** / 12 ignored (4 new core tests), wire 123 / 2 ignored,
  wasm 25, 0 failed. `lint_oracle -- --ignored`: 1 passed. `npm run check:wasm:web`: green.
  Both ignored corpus gates in release: parsed 395 books / 5,716,969 tokens / 0 mismatches; owned 395
  books / 5,716,969 tokens / 1,263,854 attribute-bearing / 395 byte-exact / 0 diverged. Wire clippy
  clean; `src/token.rs` `cargo fmt`-clean.

### Docs

- Freeze: the "emitter losslessness — recorded limit" section becomes "resolved", with the format
  consequence retained (an owned section is bound to the derived source; for parse-origin tokens the
  two are now the same bytes). API ledger gains `OwnedToken::attribute_offset` and the defaulted
  `SerializableToken::attribute_offset` — six new public core items across this packet and the last,
  all byte-neutral except where the reconstruct emitter was previously shifted.
- Epic §7 notes that serializing a parse-origin owned stream is byte-lossless while the returned
  per-book sources remain the authoritative pairing for edited tokens.
- `plans/approved/attribute-position-fidelity.md` committed with the work.

### Next

- Finding section columns: `marker_ref` per freeze §M, the common row, the sidecars, the packed patch
  table.

## 2026-07-28 — Phase A: clean-room review findings resolved

- One commit: `5a0345f` (all five findings plus the doc fix), amended once immediately after creation
  because the first attempt's P2-2 edit had silently not applied — see the process note. Base `a7fc15b`.

### Per-finding resolution

- **P1-1 — explicit stable ids accepted but never validated or decoded.** `TokenColumns::from_section`
  now validates the id dictionary with the same rigor as every other one (UTF-8 per string, ascending
  in-range offsets, every index resolving) plus **non-empty**, since an empty id cannot be
  distinguished from a missing one and core's `StableTokenId` refuses to hold it.
  `decode_token_section` returns `DecodedTokens { tokens, stable_ids }`, `stable_ids` present exactly
  when the `positional_ids` flag is clear. **Deviation from the finding's wording, reported:** "use
  explicit ids and call `assign_ids` ONLY when the flag is set" is not implementable as written — core's
  `TokenId` is a structured positional label (`{book_code}, {index}`) and cannot hold an opaque id. So
  the opaque ids are returned *alongside* the tokens (the substance: they are validated, preserved, and
  available to identity-keyed reconciliation), and the positional label is still filled and documented
  as a derived convenience, not the identity. Recorded as an implementation note in the freeze.
  Encode side round-trips them and decides the form **by proof** — it emits the columns only when some
  id differs from the positional form that stream's own `assign_ids` produces — so the flag can never
  disagree with the bytes.
- **P1-2 — sparse records on wrong-kind rows silently discarded.** Row kind is now validated at
  construction for number, book-code, and attribute records (`require_row_kind`), so a record the
  decoder would never read is rejected instead of accepted.
- **P2-1 — fidelity bit wrong for sequences and suffixes.** Fidelity is now derived from the number
  token's **source text** per the frozen rule: `Exact` only for a bare number or two bare numbers
  around a single `-`; a comma (sequence), a letter (suffix), or anything else is `AnchorOnly`, and an
  over-wide bridge still degrades inside `PackedSid::encode`. A number token carries the anchor it
  establishes, so one `BTreeMap<Sid, SidFidelity>` gives every row the fidelity of its own designator;
  `AnchorOnly` wins a collision because a duplicate verse number is a lint finding, not a parse
  failure. The reviewer's blindness point was correct and is fixed at the test level: the bit never
  reaches a decoded `Token`, so the gate now reads it off the dictionary — **`anchor_only=100`** rows in
  the corpus (from `\v 6b-11`), asserted `> 0`, plus narrow tests for bare / bridge / sequence / suffix
  / suffixed-bridge / over-wide-bridge and one asserting the bit is right for *every* row sharing an
  inexact anchor, not just the number token.
- **P2-2 — schema still named finding field 4 `MARKER_STRING_IDX` width 4.** Now `MARKER_REF`, fixed
  width 8, matching freeze §M.
- **P3 — gates asserted `books > 300`.** Both gates now assert `books == CORPUS_BOOKS` (395), a named
  constant whose comment says corpus additions bump it deliberately.
- **DOC — two stale `src/token.rs` comments.** Both now describe the remembered-offset behavior; the
  "one sanctioned divergence" section is replaced by "where the list lands", covering the
  remembered-position case and the closer-rule fallback for positionless tokens.

### Gates

- `cargo test --workspace`: core 255 passed / 12 ignored, wire **137 passed / 2 ignored** (14 new
  tests), wasm 25, all integration targets green, 0 failed. `lint_oracle -- --ignored`: 1 passed.
  `npm run check:wasm:web`: green. Wire clippy clean; `cargo fmt --check -p usfm_onion_wire` clean.
- Both ignored corpus gates in release: parsed **books=395 tokens=5,716,969 wide_bridges=0
  anchor_only=100**; owned **books=395 tokens=5,716,969 attributed_tokens=1,263,854 byte_exact=395
  diverged=0**.

### New test evidence

- Explicit ids: GUID-like ids round-trip byte-for-byte with the flag clear and both fields present;
  positional sections report `stable_ids == None`; parse-origin owned tokens omit both id fields.
  Rejections: index out of range, empty id, descending dictionary offset, out-of-range offset, an
  offset splitting a code point (`InvalidUtf8`), an id column whose dictionary is relabelled to an
  unknown optional field, and the positional flag set with the fields present (caught at the container
  layer).
- Sparse kinds: number, book-code, and attribute records aimed at wrong-kind rows each reject.
- Fidelity: the six designator classes above, plus per-row correctness across an inexact anchor's run.

### Process note

- The P2-2 edit initially did not apply: its guard assertion caught an incomplete pattern match and
  wrote nothing, but I did not notice before committing, so the commit message claimed a fix that was
  not in the tree. Caught by re-grepping the symbol immediately afterwards; applied properly and
  amended the same commit rather than leaving a false claim in history. The second attempt's guard also
  fired — this time because the new doc comment legitimately contains the old identifier as prose — and
  was narrowed to a word-boundary regex on code use.
- Owner's untracked leftovers untouched; explicit paths only; zero modified tracked files at the end.

### Next

- Finding section columns: `marker_ref` per freeze §M, the common row, the sidecars, the packed patch
  table.

## 2026-07-28 — Phase A tail: doc nits, JS schema constants, token goldens, decode_par deferral

- Four commits: `d4c4af3` (doc nits), `411d28a` (JS/TS wire-schema constants), `47a52c2` (token
  golden vectors). Base `01fb2cd`. Executes the packet spanning `braid-epic.md` §10 Phase A steps
  5–6 and `gate0-0g-dependency-ledger.md` §7.2.

### Doc nits (`d4c4af3`)

- `src/token.rs`'s `ReconstructedSpans` doc and the freeze API ledger's rationale for it (§ API
  ledger addition, row 3) still said an attribute list is always emitted at its closer/
  end-of-stream. Since `1143ac1` a parse-origin token remembers its list's offset and the emitter
  honours it; only a positionless token falls back to the closer rule. Both fixed; doc-only.

### Generated JS/TS wire-schema constants (`411d28a`) — step 6a

- Added `LintCodeTag` to `schema.rs`: the frozen `u8` ↔ kebab-string table from
  `phase0-freeze.md` §1, in `LintCode`'s declaration order, with an exhaustive
  `From<LintCode> for LintCodeTag` (a new core variant fails to compile until mirrored — the same
  drift guard `dto.rs` documents for its own conversions) and a `lint_code_tags_match_the_frozen_table`
  test. This is the discriminant table only, not the finding-record codec (Phase B); it exists now so
  the eventual codec and the generated JS module read one frozen mapping rather than two.
- `usfm_onion_wire::js_schema::render()` reads `schema.rs`'s compiled constants — magics, format/
  section versions, section-kind ids, all flag bits, field tables (id/name/elementWidth/required),
  sentinels, `TokenKind`/`NumberRangeKind` discriminants, the `LintCode` table — and renders
  `js/wire-schema.{js,d.ts}`. Field *names* are the one thing the module supplies rather than reads
  (Rust identifiers aren't reflectable at runtime); every value, width, and requiredness comes from
  the compiled tables.
- Generator: `cargo run --example generate_js_schema -p usfm_onion_wire` (thin binary calling
  `render()`). Drift check: `js_schema::tests::wire_schema_matches_generator`, runs on every
  `cargo test`, fails if the checked-in files differ from a fresh `render()`.
- Wired into npm via a new `"./wire-schema"` export (same shape as the existing `"./token-sids"`
  export) — target-agnostic, no wasm dependency, so **no wasm-pack rebuild was needed or performed**;
  `pkg-bundler`/`pkg-web` were not touched. `test:wire-schema:import` smoke-tests resolution from both
  the bundler and web package names via the shared export map.

### Token-section golden vectors (`47a52c2`) — step 6b

- `usfm_onion_wire::token_goldens`: 10 good vectors (full one-section container `.bin` + exact
  `.usfm` source + a `.json` manifest naming what it proves) and 5 malformed vectors (`.bin` +
  manifest naming the exact `DecodeError`). Coverage: every `TokenKind` with legal payloads
  (`all-token-kinds`); positional ids (same vector) and explicit opaque ids (`explicit-ids`); SID
  fidelity — exact bare, exact simple bridge, sequence/suffix/over-wide-bridge anchor-only
  degradation (5 vectors); attributes — absent vs. default-shorthand vs. explicit-empty list on one
  marker, and an owned token's remembered attribute offset reproducing a parse-origin source
  byte-identically (2 vectors); an unknown marker. Malformed: truncated (below the 48-byte header),
  bad magic, unsupported version, checksum mismatch (content edited without restamping), and an
  out-of-range `sid_index` (rejects `InvalidSection`, confirmed empirically from `decode_sid`/
  `TokenColumns::get`, not assumed).
- Generator: `cargo test -p usfm_onion_wire --lib token_goldens::generate_token_goldens --
  --ignored`, matching this crate's existing corpus-gate convention (`#[ignore]`d, never implicitly
  run). Two always-on tests: `token_goldens_decode_and_match_parse` decodes every good vector and
  asserts semantic equality against `parse(source)` (with one documented, intentional exception —
  the over-wide-bridge vector's `sid.verse_end()` is dropped by design under `AnchorOnly` fidelity,
  which the test asserts explicitly rather than silently loosening equality);
  `malformed_token_goldens_reject_with_the_recorded_error` asserts each malformed vector's exact
  `DecodeError`. Both also fail if the checked-in fixtures drift from what the encoder currently
  produces (the same "compare, never silently accept" shape as the wasm golden suite).
- Layout and the manifest schema (`name`/`kind`/`proves`, plus `base`/`expectedError` for malformed
  vectors) anticipate a Phase B JS decoder conformance harness reading the same files.

### `decode_par` deferral (step 5) — no code change

- Plan gates native parallel decode on evidence still supporting it (§10 Phase A step 5). Current
  evidence, from this packet's own release-mode corpus gates: `corpus_token_sections_round_trip`
  and `corpus_owned_token_sections_round_trip` each decode+verify the full 395-book, 5,716,969-token
  corpus in low single-digit seconds combined (well under the `cargo test --release` wall time
  reported above), i.e. per-book decode is on the order of milliseconds. That does not justify a
  second (parallel) decode path and its own correctness surface for v1. Deferred; revisit with
  native-host (Tauri) latency/heap measurements in Phase F, per the plan's own gating language —
  not implemented.

### Gates

- `cargo test --workspace`: 255 core / 0 dto (deleted) / 141 wire / 25 wasm, 0 failed, matching
  pre-packet ignored counts plus the new golden tests. `cargo test --test lint_oracle -- --ignored`:
  1 passed. `npm run check:wasm:web`: green. `npm run test:wire-schema:import`: passed (both
  package-name resolutions). `cargo clippy -p usfm_onion_wire --all-targets`: no wire warnings
  (two `#[allow(clippy::ptr_arg)]` on golden-corruption helper fns that intentionally share a
  `fn(&mut Vec<u8>)` pointer type with a `Vec::truncate`-using sibling). `cargo fmt --check -p
  usfm_onion_wire`: clean. Both ignored wire corpus gates in release:
  `corpus_token_sections_round_trip` and `corpus_owned_token_sections_round_trip`, 395 books /
  5,716,969 tokens, 0 mismatches.
- No `BLESS=1`/`UPDATE_GOLDEN=1`; no git reset/clean/checkout/stash; every commit staged explicit
  paths; owner's untracked leftovers (`.claude/`, `alloc_sizes.txt`, `lib_sizes.txt`,
  `bench-remote.sh`, `plans/approved/braid/handoff-*.md`) untouched.
- No deviation from the plan requiring adjudication.

### Next

- Finding section columns: `marker_ref` per freeze §M, the common row, the sidecars, the packed
  patch table (Phase B).

## 2026-07-28 — Phase B: finding codec STOPPED on framing gaps (freeze §F)

- One commit: `9d6d024` (freeze §F + the counterexample test). Base `0e726df`. **No codec code
  written** — this is a stop, not a partial implementation.

### Why stopped rather than built

Of the six frozen finding-section fields, four are implementable exactly as written; two name values
the byte tables allocate no storage for. That is the same class of gap that stopped Phase A before the
mixed payloads, and the owner's ruling then was explicit: freeze the framing **before** implementing
the columns. Applied again here.

- **Determined, no amendment needed:** field 0 common row (§7.6, 16 bytes) with all eight flag bits
  (§3.3); field 2 `overflow_span` (`{offset:u32,len:u32}`); field 4 `marker_ref` (§M.3 8-byte tagged
  record). Field 1 (`related_token_idx` + related span) is determined up to one **mechanical** choice —
  §7.6 gives the related span no widths, so it takes the same `u16`/`u16` as the primary span with the
  same overflow escape. Recorded as mechanical, not raised.
- **GAP 1 (blocking) — message payloads have no byte storage.** Field 3 is a frozen `u32` index column
  of width 4; what it indexes has no field id and no framing, and §M had closed the finding-section
  string-dictionary question with "field id 7 stays unassigned". That was correct **for markers**, on
  marker evidence, and does not extend to `MessageParams` (`BTreeMap<String, String>`).
  **Proven, not argued** — committed test `message_params_can_carry_values_absent_from_the_source`:
  `\id php Philippians` → `book-code-not-uppercase` with `message_params["uppercase"] == "PHP"`, which
  is neither a substring of the source (it says `php`) nor a catalog marker, so neither a span nor a
  stamp-gated ordinal can name it. Gate 0D §2 already called that parameter "load-bearing for
  remediation … the only encoded remedy" for the code. 24 of 32 codes carry parameters, and §7.6's own
  closing rule forbids dropping a field that cannot be encoded — it requires amending the schema first.
  Proposal in §F.2: field 7 = finding-section string dictionary (the §D.1 framing verbatim), field 8 =
  message payload table as two ascending arrays with a derived second count; parameters stored as
  key/value pairs, **not** 24 per-code typed structs, which §5.3 could be read as implying — raised
  explicitly rather than chosen silently.
- **GAP 2 (deferred, per packet authorisation) — patch table.** Field 6 is frozen only as prose: no
  record shape, ordering key, or width, and its contents need more string storage than field 7 alone
  implies (`TokenFix` = four `String`s + a `MessageParams` + `Vec<TokenTemplate>` of three more each).
  No framing invented. Disposition: findings encode **without fix resolution** — fields 5 and 6
  unemitted, common-row flag bit 5 clear. Braid owns patch resolution (§5.3) and a `patch_id` is
  snapshot-bound to a braid identifier that does not exist yet, so the braid phase that produces it is
  where this should be frozen. Recorded consequence: the `fix` of codes 24/25/26 (the only three with a
  producer per Gate 0D §2.2) decodes as `None` until then; every other field of those findings
  round-trips.

### Why not build the four determined fields now

It would fail the packet's own conformance criterion by construction — 24 of 32 codes decoding with
empty `message_params` is a red gate, not a green one — and it would mean a second pass over a
directory table GAP 1 is about to extend by two rows. §F.4 records the recommended sequence: adjudicate
F.2, then land all fields in one pass with the per-code gate green from the start.

### Gates (unchanged code, all still green)

- `cargo test --workspace`: core 255 passed / 12 ignored, wire 141 passed + the new counterexample
  test, wasm 25, 0 failed. `lint_oracle -- --ignored`: 1 passed. `npm run check:wasm:web`: green.
  `cargo fmt --check -p usfm_onion_wire`: clean.
- Both corpus gates in release, unchanged: parsed `books=395 tokens=5,716,969 wide_bridges=0
  anchor_only=100`; owned `books=395 tokens=5,716,969 attributed_tokens=1,263,854 byte_exact=395
  diverged=0`.

### Not delivered this packet (blocked on F.2)

Finding encode/decode, the per-LintCode corpus conformance gate, hand-built fixtures for the three
token-path-only codes, the corruption battery, and finding golden vectors. All of it is scoped and
ready to land in one pass once F.2 is adjudicated.

## 2026-07-28 — Phase B Rust codec STOPPED: core message-renderer visibility

- No finding codec or partial wire format was landed. `usfm_onion_wire::schema` now has the
  Rust-owned `ParamContract` metadata: exact key sets, `stray-close-marker`'s discriminated maps,
  and all 20 canonical `SpecContext` values.
- The required Rust `LintResult` decode cannot currently construct the catalog-derived
  `LintIssue.message`: core's sole `render_template` implementation is `pub(super)` in
  `src/lint_impl/message.rs` and is not exported by `usfm_onion::lint`. A wire-local renderer would
  violate the no-parallel-core-logic boundary; storing the English message violates §7.6.

### Required adjudication

Expose the existing core renderer for wire use, or explicitly change the Phase B Rust semantic-decode
contract. Do not add a wire-local MessageFormat renderer.

### Adjudication applied

The core exposes only `LintCode::render_message(&MessageParams)`, which delegates to its existing
private renderer for that code's own frozen template. This preserves semantic decode without exposing
arbitrary-template rendering or creating a wire-local renderer. The Phase 0 API ledger records the
append-only public addition.

## 2026-07-28 — Phase B: finding-column structural corrections

- Finding encode/decode now applies the generated `ParamContract` table: every payload is one exact
  code-specific map arm, closed domains reject, and zero-parameter codes reject payloads. The generic
  key/value table remains the only physical payload storage; no semantic `LintIssue` codec landed.
- Record-aligned related, overflow, and payload-index columns are section-wide: clear-flag rows use
  checked zero fillers, while their columns are present iff some row uses the corresponding flag.
- Verified focused wire tests and schema-drift generation checks; patch/fix framing remains deferred
  under freeze §F.3.

## 2026-07-29 — Pause-handoff decisions adjudicated; Phase B close-out packet drafted

- Owner ruled on both open decisions from `handoff-phase-b-pause-2026-07-28.md`; recorded as freeze
  §G. (1) Related records widen to 16 bytes `{token_idx:u32, offset:u32, len:u32, reserved:u32=0}`,
  superseding §F.1's 8-byte mechanical choice — the primary-span overflow escape was never actually
  available to the related span. (2) wasm `materialize` returns the existing onion token/`LintIssue`
  DTO shapes in a per-book `path → { tokens, findings }` container or one typed error — no new
  semantic types; `reconcileFindings` deferred to Phase C where braid owns token-relative finding
  identity.
- First-reviewer verification at `6f0d9a9`: wire 154 passed + 2 golden (0 failed), core lib 256
  passed (0 failed). Code confirms the related record is still `(u32, u16, u16)` — widening is
  builder work in the close-out packet.

## 2026-07-29 — Owner reversal: decode boundary (epic §2.2#19 amended, freeze §H)

- The 2026-07-28 "Rust is the only production packed decoder" rule contradicted the 0H evidence
  (JS binary eager decode 3.7–10.4× faster than wasm parse+marshal; boundary marshalling is the
  dominant cost; retention was 0H stop-threshold 4). Root cause of the original rule: consumers
  must not need a JS hash dep — and `xxhashjs` cannot even verify XXH3-64.
- Ruling: hybrid. Rust/wasm = sole trust boundary (`restoreCorpus` validates + seeds braid
  residency internally, returns branded `VerifiedPacked` or typed error); pure-JS `materialize`
  decodes certified bytes into DTO objects on the main thread via the generated wire-schema
  layout constants (load-bearing again). Mandatory Rust↔JS equivalence gate: same bin → identical
  serde-JSON objects for tokens and findings across the test corpora and golden vectors.
- Recorded in epic §2.2#19 (amended in place with supersession note) and freeze §H. Phase B
  part-2 packet (post clean-room review of the Rust finding codec) implements the JS decoder and
  respecs epic §8.1. The in-flight Rust packet is unaffected.

## 2026-07-29 — Phase B Rust packet: related-record widen + semantic finding codec landed

- §G.1 applied: `finding_field::RELATED_TOKEN_IDX` widened from 8 to 16 bytes
  `{token_idx:u32, offset:u32, len:u32, reserved:u32}`; encode writes `reserved=0`, decode rejects
  non-zero `reserved` via `DecodeError::InvalidSection` (this crate's existing reserved-byte
  convention — no new error variant needed). `js/wire-schema.{js,d.ts}` regenerated;
  `FINDING_FIELD.length` stays 9 (existing field widened, not a new one added), so
  `test:wire-schema:import`'s assertions were unchanged and it was only extracted into
  `scripts/test-wire-schema-import.mjs` per the owner's cleanup request.
- Semantic codec landed: `usfm_onion_wire::finding_codec` (`encode_book`/`decode_book`,
  `pub`) round-trips `LintResult.issues` through the paired token+finding container sections,
  covering fields 0–4, 7, 8. Rows are written in §2.2#15 canonical order regardless of caller
  order (own `canonical_order`, exposed `pub`, since core's own `canonical_sort`/`dedupe_issues`
  are private and unreachable from this crate). `message` is rebuilt only via
  `LintCode::render_message`; `category`/`severity`/`issue_type`/`template` are recomputed from
  the decoded code. Fields 5/6 (patch/fix) remain unemitted per §F.3 — every decoded issue's
  `fix` is unconditionally `None`.
- Two derivations worth flagging for reviewers: (1) a finding's SID book comes from its anchor
  token's own raw `Sid.book`, not the section's nominal book — a `book-code-not-uppercase` finding
  on an unmodified lower-case `\id` needs its own book, distinct from the container's; (2) a
  token-anchored finding can still have `sid: None` (e.g. `content-before-first-chapter`, which
  fires on real content before any `\c` establishes an anchor) — `no_anchor` is independent of
  whether the finding has a token at all, not a proxy for anchor-only. Both were caught by the
  corpus gate before being fixed, not assumed correct in advance.
- `js_schema.rs`'s generated-file header went through three revisions this packet as the owner
  sharpened the rationale (mid-packet messages, superseding each other): final text distinguishes
  why-codegen / why-not-wasm-bindgen / two consumer tiers (semantic catalog serves runtime JS
  regardless of the decode boundary; byte-layout tables are load-bearing production data for the
  next packet's pure-JS `materialize` decoder, per freeze §H — not "wasm is the sole parser",
  which §H superseded).
- Gates, all green: full workspace suite (256+165+... = 0 failed across every crate, `cargo test
  --workspace`); lint oracle (`cargo test --test lint_oracle -- --ignored`); both 395-book corpus
  gates in release (`corpus_token_sections_round_trip`, `corpus_owned_token_sections_round_trip`);
  new per-`LintCode` corpus conformance gate in release
  (`corpus_findings_round_trip_per_lint_code`, `#[ignore]`d) — all 26 corpus-producible codes
  round-trip, 62,948 total findings, matching Gate 0D §2's evidence count exactly; 3 hand-built
  fixtures for the codes no `.usfm` source can produce (`finding_codec::tests`), including
  `invalid-number-range`, which turned out unreachable from *any* `Token<'a>` (not just
  `lint_usfm(source)`) since `TokenData::Number` always carries a parsed payload — its fixture
  builds the `LintIssue` directly rather than via `lint_tokens`; finding golden vectors under
  `golden/finding/` (4 good + 6 malformed, mirroring `token_goldens.rs`); corruption-battery tests
  for truncation, single-byte mutation, non-zero related reserved word, out-of-range indices, and
  flag/column presence mismatches.
- Evidence correction recorded, not a behavior change: `gate0-0d-payload-ledger.md` §2 lists
  `related: yes` for `duplicate-chapter-number`/`duplicate-verse-number`, but both codes' current
  sole producers (`lint_chapter_rules`, `lint_number_and_verse_rules`) call
  `simple_issue_with_marker`, which has no related-token parameter — `related_span` is `None` for
  both in every corpus occurrence and by construction. Picked `unclosed-marker` (a confirmed
  `related_span` producer) for the related-span golden instead.
- Scope not in this packet, per its own boundary: no JS/wasm decode or `materialize` work — that
  is the next packet, gated on clean-room review of this one.

## 2026-07-29 — Phase B Rust packet: clean-room review fix round

Clean-room review of the finding codec above returned FAIL with 8 findings (5 P1, 3 P2). All
fixed in place, same files, same gates.

- **Checked-span arithmetic.** `resolve_span` (span/related-span reconstruction on decode) now
  returns a typed error instead of doing unchecked `u32` arithmetic on untrusted wire offsets, and
  additionally requires the resolved sub-range to fit entirely inside the anchor token's own span
  — an offset/length pair naming bytes outside that span rejects rather than being silently
  accepted or wrapping.
- **Finding-section trust binding.** `decode_book` now verifies the finding section's own
  `source_len`, `source_hash`, and `catalog_stamp` against the live source and the current marker
  registry before any catalog-derived value (e.g. a marker resolved by ordinal) is trusted —
  mirroring the check the token section already had. Each mismatch is its own typed error.
- **TOC walk.** `decode_book` no longer takes "the last Token entry" and "the last Finding entry";
  it now requires exactly one of each and that they name the same book, rejecting a container with
  more than one Token or Finding section, zero of either, or a book mismatch between the pair. A
  container naming two books (each internally valid, each independently passing the crate's own
  structural checks) is otherwise legal — nothing forces one book per container — so this had to be
  decode_book's own responsibility, not something upstream validation already covered.
- **Wide SID bridge.** The silent "degrade to anchor-only" path for a bridge wider than the row's
  one-byte range-end column is now a typed encode refusal. Investigating for a real test case
  found this branch is actually unreachable today: core's `Sid::with_range` unconditionally
  saturates at `verse + 255` (a stated contract, not a bug), so no real `Sid` value — nor any value
  reachable through this codec's public API — can ever produce a delta above 255. The refusal is
  kept as defense-in-depth against a future widening of that ceiling; no test proves it fires,
  because it structurally cannot be made to. Recorded here rather than silently omitted per the
  reviewer's own instruction on this exact point.
- **Derived-field normalization.** `encode_book` now refuses (typed error) a caller-supplied
  `LintIssue` whose `category`/`severity`/`issue_type`/`template`/`message` disagree with what the
  catalog derives for its own code and params — the wire format has no storage for a divergent
  value, so silently overwriting it on decode (which is still what decode does, by design) would
  have discarded the caller's data without telling them.
- **Canonical order.** Re-keyed from core's legacy span-based sort to primary anchor token's row
  position (token-less last) → kebab code → related anchor token's row position (related-less
  last), matching the row order the packed section is actually meant to store. `canonical_order`'s
  signature changed to take a `token_id -> row` resolver (encode already builds one); a new
  `canonical_order_for_tokens` convenience wraps it for tests/callers holding a token slice. Checked
  explicitly: this did not change any golden's bytes or the corpus gate's totals — order is
  identical to the old span-based key for every real parsed stream, since token row order and span
  offset order are both monotonic along the stream.
- **Contradictory flags.** `finding_section.rs` now rejects the `NO_ANCHOR | ANCHOR_ONLY` bit
  combination on both the encode and decode paths — `NO_ANCHOR` means no SID exists at all,
  `ANCHOR_ONLY` describes the fidelity of an *existing* SID, and the two cannot both be true of the
  same row.
- **Comment style.** Stripped plan/`§`/Gate-0D citations from `finding_codec.rs`,
  `finding_goldens.rs`, and this packet's additions to `finding_section.rs`/`schema.rs`; comments
  now state the invariant itself rather than which document froze it.
- **Named offsets.** `finding_goldens.rs`'s checksum-recomputation helpers now use
  `SECTION_CHECKSUM_OFFSET`/`CONTAINER_CHECKSUM_OFFSET` from `schema.rs` instead of the literals 40
  and 24; both helpers are `pub(crate)` so `finding_codec.rs`'s own tests reuse them rather than a
  third copy.
- **New test coverage:** `derived_fields_that_disagree_with_the_catalog_refuse_to_encode`;
  `decode_book_rejects_a_two_book_container` (crossed token/finding order across two books — the
  exact shape a last-one-wins TOC walk would mis-pair); `restamped_hostile_mutations_yield_typed_errors`,
  a battery of *semantic* mutations (hostile span offset/length, stale `source_len`, stale
  `catalog_stamp`, contradictory flags) each followed by recomputing both integrity checksums, so
  what is actually exercised is the trust checks downstream of the checksum rather than the
  checksum itself — every case asserts its exact named typed error. A stale-`source_hash` variant
  was attempted and dropped: the container's own token/finding pairing rule (same `(book,
  source_hash)` required) makes an independently-wrong finding-section `source_hash` structurally
  unreachable without also breaking token decode (which already checks the real source first), so
  `source_len` (uncross-checked at the container level) is what actually isolates the new
  finding-specific check.
- Gates rerun and green after every fix: full workspace suite (0 failed), lint oracle
  (`--ignored`), both 395-book corpus gates in release, and the per-`LintCode` corpus gate in
  release — byte-identical totals (62,948 findings, same 26 per-code counts) before and after the
  canonical-order rekey, confirming it is not an observable behavior change for any current corpus
  fixture.

## 2026-07-29 — Phase B Rust finding codec: clean-room re-review PASSED

- Verdict from the owner's clean-room reviewer: `e2bcc0e` resolves all eight prior findings; no
  remaining blockers in the fix delta. Wide-SID refusal accepted as defense-in-depth on the
  `Sid::with_range` saturation proof. `(offset != 0, len == 0)` explicitly ruled a non-issue
  (frozen meaning of `len == 0` is whole-token; encoder stays canonical with `(0,0)`). Noted,
  not a regression: older plan citations elsewhere in `schema.rs` predate the fix round.
- Reviewer verification: wire 168 passed, workspace green, lint oracle passed, release finding
  corpus gate 62,948 findings. First-reviewer verification matched independently.
- The Phase B **Rust** half is closed. Next packet (part 2): wasm verify surface + branded
  `VerifiedPacked`, official pure-JS `materialize`/`decodeTokens`, Rust↔JS serde-JSON
  equivalence gate, epic §8.1 respec — per freeze §H. Braid seeding inside `restoreCorpus`
  arrives with Phase C when braid exists; part 2 ships verification + JS materialization only.

## 2026-07-29 — Phase B part 2 STOPPED before any code: catalog-derived DTO fields (freeze §I)

- The packet (wasm verify surface + branded `VerifiedPacked` + pure-JS `materialize`/`decodeTokens`
  + Rust↔JS equivalence gate + epic §8.1 respec) stopped at the read-the-normative-docs stage. Cause:
  five fields of the DTO shapes §G.2 pins as the output have **no storage in the packed bytes**. Rust
  decode does not read them from the buffer, it recalls them by calling core under the licence of the
  section's `catalog_stamp`: `Token.markerMetadata` (`marker_metadata`), `Token.structural`
  (`structural_marker_info`), `LintIssue.{category,severity,issueType,template}` (per-code catalog),
  `LintIssue.message` (`LintCode::render_message`), and `LintIssue.marker`'s catalog-ordinal arm
  (`catalog_marker_name`). A JS decoder holding certified bytes has no registry and no catalog, so
  bounds-checking correctness does not help it produce any of them.
- Verified, not assumed, that the rest of both DTOs *is* JS-derivable from the certified bytes plus
  small deterministic formatting: token id (positional rule or explicit dictionary), sid string,
  spans, source slice, nested, numberInfo, bookCode/valid, attributes (incl. a mirror of
  `decode_attr_value`'s two escapes), attributeSource, and every finding column outside the five.
- Recorded as freeze **§I** in the §F house style: evidence table of the five producing call sites,
  why a static JS table cannot cover the marker fields (open-ended names, resolution logic), and a
  proposal per field. §I.3 proposes the verification receipt carry the section's descriptor rows
  already resolved (`{name, nested, markerMetadata, structural}`, descriptor-ordinal order) plus a
  generated 238-entry ordinal→name array in `js/wire-schema.js` — **OWNER-DECISION** on whether that
  is inside §H's "per-book stamps/metadata". §I.4 records the 32-row per-code catalog table as
  mechanical. §I.5 is the real blocker and carries four options with costs — **OWNER-DECISION**.
- §I.5's blocker in one line: `finding_codec.rs`'s module doc says, normatively, "`message` is
  rebuilt **only** via `LintCode::render_message` — never a wire-local template renderer", and a
  pure-JS materializer emitting `message` is exactly a second renderer. §H addressed hashing, not
  rendering, so this was raised rather than overridden.
- Sizing measured for the proposal rather than estimated: `en_ult/43-LUK.usfm` is 4,051,598 bytes /
  192,721 tokens / 25 distinct marker forms; `19-PSA.usfm` is 5,122,298 / 276,987 / 31; the marker
  catalog is 238 entries; the corpus averages 159 findings per book (62,948 / 395).
- **No production code, no JS, no wasm export, and no §8.1 respec was written.** §I.6 records why the
  respec is blocked with the implementation rather than ahead of it: the receipt's contents (§I.3) and
  the finding boundary (§I.5) are the two shapes the respec exists to pin, so writing it first would
  bake in the guesses this STOP exists to avoid. `decodeTokens` is blocked identically to
  `materialize` — it needs the two marker fields — so there is no shippable subset either.
- Unaffected: the Phase B Rust half at `e2bcc0e`, and every gate it left green.

## 2026-07-29 — §I adjudicated; Phase B part 2 re-dispatched

- Owner ruled: §I.3 accepted (resolved descriptor rows + ordinal→name array on the receipt);
  §I.5 = option (c) — JS materializes tokens only, findings return Rust-materialized on the
  verify result; §I.4 moot. Scope adds selective `materialize(verified, {book, chapter})` via
  lazy-view range location. Cross-language equivalence gate narrows to tokens; findings keep
  single-decoder Rust gates. Recorded as a freeze adjudication appendix following §I.

## 2026-07-29 — Phase B part 2 resumed: epic §8.1 respecified around the §I rulings

- §I adjudicated at `ff96405` (receipt carries resolved descriptor rows; findings stay Rust-side per
  §I.5 option (c); §I.4 moot; selective materialize added; concurrency note). Packet resumed.
- Epic §8.1 rewritten (docs-first, ahead of any code) as the concrete TS surface for freeze §H + §I,
  replacing the pre-reversal text whose `decodeTokens`/`materialize` were wasm exports and whose
  `reconcileFindings` is now Phase C. One-sentence division of labour recorded at the top: Rust
  certifies bytes and materializes findings; JS materializes tokens.
- Names pinned for Phase C to wrap without an npm break: wasm export `verifyPackedBook(packed,
  source) -> PackedBookOutcome` (tagged `verified`/`rejected`, never thrown), npm glue
  `verifyPackedCorpus(wasm, records) -> VerifyPackedResult`, pure-JS `materialize(verified,
  selector?)` and `decodeTokens(verified, path)`, branded `VerifiedPacked`, `PackedBookReceipt`,
  `PackedMarkerDescriptor`, `PackedDecodeError` (the frozen §6.2 set, tagged for TS narrowing), and
  a thrown `PackedError`.
- Three respec decisions worth flagging, all recorded in §8.1 itself: (1) the receipt's three u64
  attestations (`sourceHash`, `catalogStamp`, `snapshotId`) are 16-char lowercase hex strings —
  an audit record of what Rust checked, explicitly not an input to any JS check; (2) the selector's
  primary key is the caller's `path`, not `book`, because one restore can legitimately carry two
  corpora that both contain `GEN` (`book` is still accepted when it resolves uniquely, else
  `ambiguousBook`); (3) the first rejected record short-circuits the whole corpus result and names
  its `path`, since a partially-restored corpus is not a state the application asked for.
- Concurrency contract stated as the ruling requires: receipt and brand return as soon as trust
  checks pass, Phase C's braid seeding continues inside wasm afterwards, so JS materialization and
  seeding run concurrently and Phase C must not serialize them.
- Gate wording narrowed in §8.1 to match the ruling: the cross-language equivalence gate covers
  tokens (Rust `serde_json` of `dto::Token` vs JS materialize); malformed goldens are asserted
  rejected at the wasm verifier with their recorded error names; findings keep their Rust round-trip
  and corpus gates plus a new `verify_book`-vs-`decode_book` equality gate.

## 2026-07-29 — Phase B part 2 landed: wasm verify surface + pure-JS token materializer

Two code commits behind the §8.1 respec, both with every gate green.

- **`34accb8` — `usfm_onion_wire::verify`.** `verify_book(packed, source)` is the whole trust
  boundary for one book and returns `{receipt, findings}`. The receipt carries the §I.3 payload: each
  marker form's `markerMetadata`/`structural` already resolved from the stamped registry, in
  descriptor-ordinal order, one row per distinct marker form rather than per token. Also
  `materialize_tokens` (the native host's token path and the gate's reference half) and wasm
  `verifyPackedBook(packed, source) -> PackedBookOutcome`. `source` crosses as bytes, not a string, so
  a caller hands over the buffer it read; non-UTF-8 is a rejection value, not a panic.
- **Refactor worth noting:** `decode_book`'s three finding-section trust checks (source length, source
  hash, catalog stamp) moved into a shared `decode_finding_section`. Two copies is precisely how one
  caller would have ended up trusting a catalog-derived value the other refuses.
- **`096dd59` — `usfm-onion-web/packed`.** `verifyPackedCorpus` (sole minter of the `VerifiedPacked`
  brand; first rejection short-circuits and names its `path`), `materialize`, `decodeTokens`,
  `PackedError`. All offsets come from `./wire-schema`; no hash, checksum, or source-binding code
  exists in the module. Selective materialize reads only the 2-byte SID index column to find a
  chapter's row range.
- **Equivalence gate (`npm run test:packed`, `scripts/test-packed-equivalence.mjs`):** 409 cases /
  **5,717,137 tokens** structurally identical between Rust and JS — 14 good golden vectors (token +
  finding) and all **395** corpus books; **11** malformed goldens refused at the wasm verifier with
  their exact recorded error names; **382** chapter-selective slices proved deep-equal to the slice of
  the full materialize, plus `unknownChapter`/`unknownBook`/unbranded-handle typed-error assertions.
  Green on both the bundler and web targets. Runs in ~70s: the Rust reference streams as one JSON
  token per line and fixtures are written and deleted per book, so a 280k-token book never needs two
  copies in memory.
- **Option/omitted-field policy, made explicit** (the packet asked for it): Rust omits `None` fields
  entirely (`skip_serializing_if`), so the JS decoder never sets an absent key, and the comparator
  treats an `undefined` value as an absent key. Nothing on either side emits `null` for an absent
  optional. `stableIds` is `null` from Rust and absent from JS for a positional-id section, compared
  as `book.stableIds ?? null`.
- **Three JS mirrors of trivial Rust logic**, each covered by the gate over 5.7M tokens rather than
  argued: `assign_ids`' positional-id rule, `format_sid`'s `"{book} {chapter}:{locator}"`, and
  `decode_attr_value`'s two escapes. The material catalog logic is *not* mirrored — that is what the
  receipt exists for.
- **`TextDecoder` is constructed `{fatal: true}` deliberately.** Rust resolves token text with
  `str::get`, which refuses a span that splits a character; a lenient decoder would substitute U+FFFD
  and hand back a token that never existed.
- **Deviation from §I.3, surfaced not silently taken:** the generated 238-entry ordinal→name marker
  array was **not** added to `js/wire-schema.js`. Its only consumer would have been the
  catalog-ordinal arm of a finding's `marker`, and §I.5's ruling (c) resolves findings in Rust, so the
  array would ship dead. Say the word and it goes in.
- **Two pre-existing staleness items found and left alone** (restored rather than folded into this
  packet's diff): `cargo fmt --all` reflows `catalog.rs` and `finding_goldens.rs`, and rerunning the
  golden generator rewrites the `proves` text of two `golden/finding/*.json` files (the fix round
  stripped `§` citations from the Rust strings but not from the committed JSON). Both are one-line
  cleanups whenever someone wants them.
- **`pkg-bundler`/`pkg-web` restored, not committed.** The new `verifyPackedBook` export therefore is
  not in the checked-in packages until someone runs a release build, the same way every prior wire
  surface has landed.
- Gates: `cargo test --workspace` (0 failed; wire 174 passed, core 256), lint oracle
  (`--test lint_oracle -- --ignored`), release `-p usfm_onion_wire -- --ignored` (both 395-book token
  corpus gates, the per-`LintCode` finding corpus gate, both golden generators byte-identical),
  `npm run test:wasm` (bundler + web), `test:wire-schema:import`, `test:token-sids:import`,
  `golden:wasm` (7 fixtures, unchanged), `test:packed` + `test:packed:web`, clippy clean on the new
  code.

## 2026-07-29 — Phase B part 2 clean-room fix round: all seven findings landed

Seven findings against `c98034f..6a8c095` (packed verify surface + pure-JS materializer), fixed in
six commits (`b30b468`, `335ce3f`, `bf5a401`, `bae0ecb`, `592020c`, `9d1ff17`, plus `10eb969` for a
regenerated golden noted below):

- **P1-1 (mutable caller-owned buffers).** `verifyPackedCorpus` now copies `packed`/`source`
  (`Uint8Array#slice`, never an `ArrayBuffer` transfer/detach) into the handle before verification.
  Epic §8.1's "which it never copies" line is corrected in place, dated, with the failure mode
  explained. Regression: mutate the caller's arrays after minting, assert materialized output
  unaffected and the handle's buffers are not the caller's own.
- **P1-2 (frozen types discarded).** `packed.d.ts` now imports `MarkerMetadata`,
  `StructuralMarkerInfo`, `LintIssue`, `Token` from `pkg-bundler/usfm_onion_web.d.ts` instead of
  `Record<string, unknown>`/`unknown[]`; `materialize` returns `ReadonlyMap`. Added
  `js/packed-consumer.fixture.ts` + `tsconfig.packed-fixture.json`, wired as `test:packed:types`
  (first step of `test:packed`); added a `typescript` devDependency (none existed). Verified the
  fixture actually traps a regression (temporarily widening `markerMetadata` back to
  `Record<string, unknown>` fails `tsc` with TS2322).
- **P2-3 (circular chapter-gate check).** The equivalence script previously derived its "expected"
  selective slice from the implementation's own reported `range` — now derives it solely from the
  full materialize's own token `sid` fields. Widened from one chapter/book to first/middle/final
  distinct chapters. Added a duplicate-book-ambiguity case (two paths, same book code,
  `materialize({book})` → typed `ambiguousBook`).
- **P2-4.** `MaterializedBook` (both `packed.js` and `packed.d.ts`) no longer carries a `range`
  field; frozen shape is `{path, book, tokens, stableIds?}`.
- **P2-5 (forgeable brand).** `VERIFIED` is a module-private `Symbol()`, not `Symbol.for()`.
  Regression: a handle forged with the old interned name is rejected by both `materialize` and
  `decodeTokens`.
- **P2-6 (finish in-flight layout work + stop hand-writing offsets).** The previous builder's
  in-flight `schema::layout` module (container/section header, TOC entry, directory entry, and
  every fixed record shape's field offsets) already compiled and its drift test
  (`js_schema::tests::wire_schema_matches_generator`) already passed — no module-path fix was
  needed, contrary to the handoff's expectation. What remained: `packed.js` still hand-wrote every
  offset in `readContainer`/`readSection`/`materializeRow`/`applyAttributes`. It now reads them all
  through the generated `./wire-schema` `*_OFFSET` constants; the only remaining numeric literal is
  the generic u64-as-two-u32-halves high-word check, which is a property of little-endian 64-bit
  reads, not a schema field position.
- **P3-7.** Stripped the `§H`/`§I`/`§G.1` plan citations from `packed.js:1-3`, `packed.d.ts:1-2`,
  and `test-packed-equivalence.mjs:1`; `packed.js`'s "every offset is generated" claim is now true.

**Gates, exact counts:** `cargo test --workspace` 0 failed (176 wire + 2 wire-corpus-ignored +
core, matching the prior packet's shape); `cargo test --test lint_oracle -- --ignored` green;
`cargo test --release -p usfm_onion_wire -- --ignored` green (both golden generators, both corpus
round-trip gates, the per-code finding corpus gate); `npm run test:packed` (bundler) — **409 cases /
5,717,137 tokens** (14 good goldens, 11 malformed goldens refused, 395 corpus books, **715**
independently-derived chapter-selective slices, 1 duplicate-book ambiguity check); `npm run
test:packed:web` — identical counts; `npm run test:wire-schema:import` green.

**Not committed:** `pkg-bundler`/`pkg-web` were dirtied by the dev wasm builds the gates require and
restored via `git checkout -- pkg-bundler pkg-web`, unchanged from prior practice.

**Deviation, surfaced:** running the release corpus gate rewrote two `golden/finding/*.json` files'
`proves` text (pre-existing staleness already noted in the prior packet's entry — the fix round
stripped a `§G.1` citation from the Rust strings but not the committed JSON). Committed as
`10eb969` rather than discarded, since it is exactly the one-line cleanup that entry flagged.

**Not done:** no new production code beyond the seven findings; the 238-entry ordinal→name marker
array §I.3 mentions but the prior packet deferred (dead until a catalog-ordinal `marker` producer
exists) is still not added — out of scope for this fix round.

## 2026-07-29 — Phase B part 2 clean-room round 3: posture ruling + remaining findings

Owner posture ruling recorded first (governs future review of this surface): the packed verify/
materialize surface protects HONEST use — certification, typed errors, copy-at-mint — not deliberate
subversion of its own in-process state. The brand/opacity is footgun elimination, not a security
boundary. Added as a dated note in epic §8.1.

Five findings from round 3, landed in seven commits (`349cf9b`, `b90cd06`, `225de05`, `67540c9`,
`a13e4f2`, `f788b83`; the sixth commit slot is this ledger entry):

- **P1 (opaque handle).** `VerifiedPacked` is now `Object.freeze({})` with zero own properties,
  genuine or forged. All decoder state (byte copies, receipt, resolved descriptors) moved into a
  module-private `WeakMap` keyed by the handle's identity; WeakMap membership is the mint check and
  subsumes the prior round's `Symbol()` brand, which is dropped. Added `receiptFor(verified, path)`
  — a `structuredClone` snapshot the decode path never reads back from — for the one legitimate
  external need (receipt inspection by tests/callers). Epic §8.1's `VerifiedPacked`/`VerifiedBook`
  block is corrected in place, dated, to match.
- **P1 (mutable generated exports).** `js_schema::render()` now emits a generated recursive
  `deepFreeze` helper and wraps every load-bearing exported table in it (layout offsets, field
  tables, `TOKEN_KIND`/`NUMBER_RANGE_KIND`/`SECTION_KIND`, `PARAM_CONTRACTS`, etc.). Verified at
  runtime that nested objects are frozen, not just top-level arrays, and that the generator's own
  "frozen object" doc comment is now true.
- **P2 (forged-brand regression).** Rewritten for the opaque handle: builds a look-alike from the
  genuine handle's own property *and* symbol keys (both empty sets, since the handle is
  `Object.freeze({})`) and asserts it is still rejected — proving the check is real identity
  membership, not anything shape-inspectable.
- **P2 (no-\c selective coverage).** The equivalence script now captures the first corpus book found
  with zero chapters during the main loop and explicitly asserts `{chapter: 1}` on it is a typed
  `unknownChapter` error (falls back to a tiny synthetic fixture if the corpus has none — this run
  found a real one, `example-corpora/en_ult/A0-FRT.usfm`-class front matter, so the fallback path
  did not execute).
- **P3.** Dropped the stale `range?` from `materialize`'s JSDoc.

**Standards items:** removed the reintroduced `§8.1`/freeze-`§I.3`/`P1-*`/`P2-*` citations from
`packed-consumer.fixture.ts` and `test-packed-equivalence.mjs` — self-contained rationale only.
`packed.js`'s remaining hand-written strides (`~403-414`, `~467`, `~480` in the round-3 report) are
now each column's own directory-reported `width`, read once per book alongside the field, rather
than a hardcoded 2/4; the line-14 header comment is scoped to name the one true exception (`u64()`'s
inherent high/low split — 4 bytes forward by definition of a 64-bit value's two 32-bit halves, not a
schema position), correcting this round's own ledger-writing standard for exactness. `npm pack
--dry-run` was shipping `js/packed-consumer.fixture.ts`; `package.json`'s `"files"` entry narrowed
from `"js"` to `"js/*.js"`/`"js/*.d.ts"`, which still ships every runtime file and now excludes the
fixture, without touching how `test:packed:types` finds it (straight from the working tree via
`tsconfig.packed-fixture.json`).

**Gates, exact counts:** `cargo test --workspace` 0 failed (256 core + 176 wire + others, matching
prior packet shape); `cargo test --test lint_oracle -- --ignored` green; `cargo test --release -p
usfm_onion_wire -- --ignored` green (both golden generators, both corpus round-trip gates, the
per-code finding corpus gate); `tsc --noEmit -p tsconfig.packed-fixture.json` green; `npm run
test:packed` (bundler) and `test:packed:web` — both **409 cases / 5,717,137 tokens** (14 good
goldens, 11 malformed goldens refused, 395 corpus books, 715 chapter-selective slices, 1
duplicate-book ambiguity check), plus the new forged-handle and no-\c assertions passing silently
within the same run (they are `assert.throws`/`assert.equal` checks, not separately counted stats).

**Not committed:** `pkg-bundler`/`pkg-web`, restored via `git checkout --` after every dev build, as
in every prior packet.

**Not done:** nothing outstanding from the round-3 list; reviewer-confirmed items (pkg-bundler type
import packaging safety, first/middle/final/front-matter selective coverage) required no action.

## 2026-07-29 — Phase B part 2 clean-room round 4 (closing round): frozen descriptor tree

One behavioral P1 plus comment cleanup, landed in two commits (`27a574c`, `3a83fdc`; this ledger
entry is the third). Round 3's WeakMap opacity, frozen schema exports, forged-handle rejection, and
no-\c coverage all passed clean-room re-review with no further action.

- **P1 (behavioral).** `tokenReader` attaches `descriptor.markerMetadata`/`.structural` to every
  materialized token by reference from the private receipt. Reviewer repro: mutate one returned
  token's `markerMetadata.canonical`, every later materialization of the same book reports the
  mutated value, because every token sharing that marker form points at the same object. Fixed by
  deep-freezing `receipt.descriptors` once per book at verify/store time
  (`verifyPackedCorpus`) — one-time O(descriptor-count) (a few dozen rows/book), explicitly not
  O(token-count): cloning per token or per `materialize` call was ruled out as wasted work. Mutation
  now throws `TypeError` under strict mode; a consumer wanting a decorated copy clones it itself.
  Regression added: materialize, assert mutating a token's `markerMetadata` throws, materialize
  again, assert unchanged output. Manually confirmed the assertion actually catches the bug (probed
  the pre-fix mutation path directly; it silently succeeded and leaked, as the reviewer described).
- **Comment fixes:** dropped the reintroduced "epic §8.1" citation from `VerifiedPacked`'s doc;
  corrected the field-width comment that falsely claimed a widened column "stays correct without a
  matching code change" (the fixed `getUint16`/`getUint32` calls still assume today's widths — only
  the stride is generic); `receiptFor`'s doc in both `packed.js` and `packed.d.ts` now calls its
  return value a "detached snapshot" rather than "read-only" (`structuredClone` produces an ordinary
  mutable object; mutating it just never reaches the decoder).

**Gates, exact counts:** `cargo test --workspace` 0 failed; `lint_oracle -- --ignored` green;
`js_schema::tests` (drift test, unaffected by this round) green; `tsc --noEmit -p
tsconfig.packed-fixture.json` green (no public type shapes changed, only doc comments); `npm run
test:packed`/`test:packed:web` both green — 409 cases / 5,717,137 tokens, 14 good goldens, 11
malformed, 395 corpus books, 715 chapter-selective slices, 1 duplicate-book check, plus the new
frozen-descriptor-tree assertion passing silently within the same run.

**Not committed:** `pkg-bundler`/`pkg-web`, restored via `git checkout --` after every dev build.

**Not done:** nothing outstanding — this is the closing round for the Phase B part 2 packed verify
surface fix cycle.

## 2026-07-30 — PHASE B CLOSED (reviewer verdict: dispatchable, no findings remain)

- Part 2 round 4 passed clean-room review: descriptor tree frozen per verified book, mutation
  throws, rematerialization stable; all comment corrections verified accurate. Final gates:
  workspace 0 failed, packed equivalence 409 cases / 5,717,137 tokens, 715 chapter slices.
- Phase B whole = finding wire codec (both trust rounds), packed verify surface, branded opaque
  `VerifiedPacked`, pure-JS token materializer with selective {book, chapter} mode, Rust↔JS
  equivalence gate, golden vectors, per-code corpus gate. Deferred onward per freeze: patch/fix
  framing (§F.3 → Phase C), `reconcileFindings` (§G.2 → Phase C).
- Next: Phase C (braid residency floor). C1 = core pull-forwards (declared-book lint context +
  mismatch rule; §2.2#15 canonical-order re-key in core, oracle-gated; formatter id-minting per
  the 2026-07-28 freeze adjudication).

## 2026-07-30 — Phase C step 1 (C1): core pull-forwards landed

Six commits (`5533dcc`, `06e31e0`, `6caed47`, `7dd7121`, `00c32e7`, plus this ledger entry) land
all three C1 items — no braid crate yet (that's C2), core-crate work only, with the minimal wire/
wasm ripple the shared `LintCode` enum and `LintOptions` struct force.

- **Declared-book lint context + `BookIdMismatch`.** `LintOptions.declared_book: Option<BookId>`,
  defaulting to `None` via `scoped` — every existing caller keeps stateless behavior, proven by the
  lint oracle staying byte-identical with zero rebless. New stable `LintCode::BookIdMismatch`
  (kebab `book-id-mismatch`, `u8` 32, `Document`/`Warning`/`Usfm`) fires when a source `\id` IS a
  canonical code but disagrees with the declared one; `InvalidBookCode` still wins when the code
  isn't canonical at all. Followed the Gate 0D-blessed shape exactly — no conflict hit, no stop.
  `BookId` gained a manual `Deserialize` impl (mirrors its existing `Serialize`) since this is the
  first `Deserialize`-deriving struct to hold one.
- **§2.2#15 canonical-order re-key.** `canonical_sort` now keys on primary-token position (via a
  one-time `token_id -> position` map, `token_positions`) instead of byte span, then kebab code,
  then related-token position — token id strings never enter the comparator itself. Verified
  oracle-neutral exactly as the freeze predicted: `lint_oracle_is_stable` AND
  `partitioned_matches_serial_over_corpora` both pass byte-identical, no rebless, over the full
  corpus. `usfm_onion_wire::finding_codec::canonical_order` already used this identical 3-key shape
  before this packet; chose not to consolidate the two (independently pinned by separate gates,
  wire's version deliberately more generic) — cross-referenced in both doc comments instead.
- **Formatter/fix id-minting seam.** `format_tokens_with_minter`/`format_with_minter`/
  `format_mut_with_minter` and `apply_token_fix_with_minter` accept an optional
  `&mut dyn FnMut() -> String`; every synthesized token (structural linebreak, default `\p` after a
  bare chapter intro, malformed-marker recovery, `TokenFix` replacement/insert) gets a minted id
  through the new `pub(crate) format::mint_synthetic_id` when a minter is supplied, and keeps its
  historical no-id shape when not. Scoped to the seam + tests, per the packet; braid is the first
  real caller in C2.
- **Wire/wasm ripple** (forced by the shared `LintCode`/`LintOptions` types, not scope creep):
  `usfm_onion_wire::dto::LintCode` + `schema::LintCodeTag` both gained `BookIdMismatch` (append-only,
  code 32), `LINT_CODE_TABLE` widened 32->33, `PARAM_CONTRACTS` gained one entry (24->25),
  `js/wire-schema.js` regenerated, `scripts/test-wire-schema-import.mjs`'s pinned counts updated.
  `usfm_onion_wasm`'s `lint_options_into_native` hardcodes `declared_book: None` (no wasm/JS surface
  change this packet).

**Gates, exact counts:** `cargo test --workspace` 0 failed (266 core, up from 256 — 10 new tests:
6 declared-book/mismatch tests + 6 formatter-minter tests minus overlap, see commit for the exact
list); `lint_oracle_is_stable` and `partitioned_matches_serial_over_corpora` both byte-identical,
zero rebless; `cargo test --release -p usfm_onion_wire -- --ignored` green, zero golden diffs; `npm
run test:packed`/`test:packed:web` both green — 409 cases / 5,717,137 tokens (unaffected: the new
rule needs a declared-book context no current corpus/golden caller supplies, so it produces zero
findings today); `scripts/test-wire-schema-import.mjs` updated and green; `tsc --noEmit` green
(no public JS type shapes changed). `pkg-bundler`/`pkg-web` restored, never committed.

**API additions:** appended to `plans/approved/braid/phase0-freeze.md` table 1 and §7 (this ledger
entry's commit `00c32e7`) — `LintCode::BookIdMismatch` (u8 32), `LintOptions.declared_book`,
`BookId`'s new `Deserialize`, the five id-minting seam functions, and the two wire mirror additions.

**Deviations:** none from the packet's shape. **Stops:** none — the Gate 0D blessed API shape
matched current code cleanly, the re-key was oracle-neutral as predicted, and the freeze's
"trait/closure" wording for the minter left the closure-vs-trait choice open, which is not a design
question the freeze left unanswered.

- Next: Phase C step 2 onward — the `braid` crate itself (C2): strict input/selector/error types,
  candidate validation, ordered unique declared books, duplicate-preserving chapter-run metadata.

## 2026-07-30 — Freeze §K: mutation/sync caller contract ruled

- Owner ruled the braid caller API trio: MutationEffect (scopes out, mutate-first-atomic) +
  pull (normalizing, current-truth, transport-hidden) + reconcile (pure-JS identity-stable
  application). changedSince(snapshotId) added as the cursor-free recovery valve. Mut-style
  "pass my tokens in" APIs rejected on boundary physics (recorded in §K.6). C2 specs
  MutationEffect/pull against §K verbatim; reconcile lands with the Phase C JS work.

## 2026-07-30 — C1 clean-room fix round: three P1s + comment sweep landed, C2 unblocked

Clean-room review of C1 returned BLOCK with three P1s (all verified at the cited sites) plus a P2
comment sweep. Five commits (`0d9e820`, `327c1e5`, `7986c34`, `beaf2a2`, plus this ledger entry) fix
all four; the declared-book seam itself passed review unchanged.

- **P1-1 (position order lost for id-less tokens).** `canonical_sort` resolved a finding's position
  by looking up its `token_id` string in a map built from the token slice — which has nothing to
  resolve for a caller whose tokens carry no id at all, e.g. `into_format_tokens`'s own public
  output, so those findings all fell into the "no position" bucket and sorted by code string alone.
  Fixed by recording `position`/`related_position` on `LintIssue` directly at the moment a rule
  creates the finding (every `issue()`/`simple_issue*` call site already has the token's own index
  in scope), not resolved afterward from an id. Verified oracle-neutral: `lint_oracle_is_stable` and
  `partitioned_matches_serial_over_corpora` both stayed byte-identical, zero rebless. Added the
  reviewer's exact repro (`\id GEN\n\zzz x\n\v x` through `into_format_tokens`, comparing finding
  order against `lint_usfm`).
- **P1-2 (duplicate id across split fragments).** `recover_malformed_markers` cloned the original
  token for BOTH the prefix and suffix fragments of a malformed-marker split, so an identified
  "before\q1 after" yielded two fragments sharing one stable id. Ruled: the first-emitted fragment
  keeps the original identity, every fragment after it is synthetic and gets minted. Added the
  reviewer's repro plus a no-prefix sibling case.
- **P1-3 (unenforceable minter guarantee).** `FormattableToken::set_id` had a no-op default, so an
  implementor could satisfy the `_with_minter` APIs' type signature while silently discarding every
  minted id. Removed the default — `set_id` is now required (pre-release, breaking accepted). Both
  in-tree implementors already had real bodies; only future implementors are affected. Added a test
  exercising the trait contract directly through both implementors.
- **P2 comment sweep.** Stripped the dated/phase/gate citations from the id-minting seam's doc
  comments, `canonical_sort`'s doc, `finding_codec::canonical_order`'s cross-reference (also fixed
  its now-stale reference to the removed `token_positions` helper), `LintCodeTag::BookIdMismatch`'s
  doc, and the wasm `declared_book` default's comment — keeping architectural rationale and the
  core<->wire cross-reference in each case. Left pre-existing citations in files I didn't otherwise
  touch (`schema.rs`'s `LintCodeTag` module doc, `finding_goldens.rs`'s header) alone — not in scope.

**Ripple, forced by `LintIssue` gaining two fields:** every out-of-core `LintIssue` literal
construction needed `position`/`related_position` — `finding_codec.rs`'s `decode_findings` sets
them from the row's own `token_idx`/related index (more direct than core's creation-time recording);
three test/golden fixtures (`finding_codec.rs` x2, `finding_goldens.rs`, `usfm_onion_wasm` lib.rs)
use the `NO_TOKEN_POSITION` sentinel, correctly — none of those paths read these fields for their
own ordering. `LintIssue` also gained a manual `PartialEq`/`Eq` (replacing the derive) excluding
both new fields from equality, since they are a sort-key artifact of one particular call, not part
of a finding's semantic identity.

**Gates, exact counts:** `cargo test --workspace` 270 passed / 0 failed (core, up from 269 —
the P1-1/P1-2/P1-3 regression tests), all other crates green; `lint_oracle_is_stable` and
`partitioned_matches_serial_over_corpora` both byte-identical, zero rebless; `cargo test --release
-p usfm_onion_wire -- --ignored` green, zero golden diffs; `npm run test:packed`/`test:packed:web`
both green — 409 cases / 5,717,137 tokens, unchanged; `pkg-bundler`/`pkg-web` restored, never
committed.

**API ledger:** appended a correction to `phase0-freeze.md` for the two new `LintIssue` fields and
`NO_TOKEN_POSITION` (core-only; no wasm/JS-facing addition).

**Deviations:** none. **Stops:** none — all three P1s had a clear fix within the stated latitude,
and no frozen public contract needed to change (the `LintIssue` field additions are new surface,
not a break: `#[serde(skip)]` keeps the wire/JS shape unchanged).

- C2 (the `braid` crate itself) is unblocked pending re-review of this fix round.

## 2026-07-30 — C1 formally CLOSED: dedupe-identity tail fixed

Re-review verdict: the three P1 fixes from the prior round passed and C2 was unblocked, but C1
stayed formally open for one correctness tail. One commit (`094f878`, plus this ledger entry)
closes it.

- **P1 (dedupe collapsed distinct id-less/spanless findings).** `dedupe_issues`'s identity tuple —
  `(code, span, related_span, token_id)` — didn't include the new `position`/`related_position`
  fields, so two distinct findings that are both id-less and spanless (bare caller tokens carrying
  neither) hashed to the same `(code, None, None, None)` key and one was silently dropped *before*
  the position-aware sort ever ran. Fixed by adding both position fields to the identity tuple.
  Verified the regression test genuinely catches the bug: reverted the fix locally, confirmed the
  test failed (1 finding reported instead of 2), restored the fix, confirmed it passed again.
  Verified oracle-neutral: `lint_oracle_is_stable` and `partitioned_matches_serial_over_corpora`
  both stayed byte-identical, zero rebless — parsed tokens always carry a real span or id, so this
  never changes what counted as a duplicate on that path.
- **P2.** Stripped the remaining ephemeral review labels from two test comments (kept the
  rationale) and rewrote `finding_codec.rs`'s `decode_findings` comment, which still described the
  retired id-string-resolution design, to the current truth: core records positions at finding
  creation; the decoder reconstructs the same positions from the row's own checked token index.

**Gates:** `cargo test --workspace` 271 passed / 0 failed (core, up from 270 — the new dedupe
regression); `lint_oracle_is_stable` byte-identical, zero rebless.

**C1 is formally closed.** All four packet items (declared-book context, §2.2#15 re-key, formatter
id-minting seam, and this dedupe tail) are landed and re-reviewed clean. C2 (the `braid` crate) may
proceed.

## 2026-07-30 — Phase C step 2–5 + native pull (C2): the `braid` residency floor landed

Two commits (`7ba75ee`, `b5846b4`, plus this ledger entry). C2's scope was the
crate itself: strict inputs, candidate validation, ordered unique books,
duplicate-preserving chapter runs, the mutation verbs with exact
`MutationEffect`s, authoritative bytes/hashes/snapshot identity/dirty stamps, and
the native `to_tokens` pull. No lint (C3), no patches (C4), no wasm (Phase F).

- **Core (`7ba75ee`) — one new export, zero byte changes.**
  `usfm_onion::token::LineEnding { Lf, CrLf }` (with `as_str`/`detect`) and
  `tokens_to_usfm_reconstruct_with_eol`, the §2.2#16 emission override: the shared
  private `reconstruct` takes an optional ending and substitutes only
  `TokenKind::Newline` sources. Both existing entry points pass `None` and are
  byte-unchanged (oracle byte-identical, zero rebless). **Ledger deviation:**
  freeze §7.3 assigned `LineEnding` to braid; it landed in core and is
  re-exported as `braid::LineEnding`, because §2.2#16 puts emission in core and an
  emitter knob must be a core type — one enum re-exported beats braid mirroring a
  core parameter type. Recorded in the freeze's C2 API append.
- **`braid` (`b5846b4`) — new workspace member**, `serde` feature = plain derives
  (no tsify/wasm-bindgen anywhere), deps `usfm_onion` + `xxhash-rust` +
  `rustc-hash`. Modules: `input`, `corpus`, `state`, `error`, `lib`.
  - Mutate-first atomicity: every candidate book is parsed, id-validated, emitted,
    and hashed before resident state is touched. Every typed-rejection test
    fingerprints (snapshot id, ordered source keys + per-book hashes, dirty set)
    and asserts it identical after the rejection.
  - Effects are §K.2 verbatim: `MutationEffect { snapshot_id, changed: Vec<Scope>,
    removed: Vec<BookId> }`, `changed` exact (rewritten, not inspected).
    Duplicate `\c` labels widen that book to whole-book (K.3); ambiguous chapter
    *operations* error. Two state changes deliberately carry an empty `changed`
    and are tested as such: a source-key rebinding (freeze: renames preserve
    semantic identity and caches) and a pure reorder (no book rewritten, but the
    id changes because §J.4's id is over the **ordered** hashes).
  - Identity is §J.4: xxh3-64 over the ordered per-book source hashes, so no-op
    preservation falls out with no special case. The per-book hash is pinned by a
    golden produced from wire's own `source_hash`, since braid cannot depend on
    wire to prove the fingerprints agree.
  - Chapter runs come from core's `walker::chapter_segments`, so braid's grain
    cannot drift from the linter's; labels are the verbatim number token (`01` and
    `1` stay distinct). Duplicate and out-of-order runs are retained in order.
  - `update_chapter` replaces exactly one run: 0 matches `ChapterNotFound`, >1
    `AmbiguousChapter`, a differing/extra run `ReplacementLabelMismatch`, and
    several runs of the target label inside one replacement `AmbiguousChapter`
    (label matches, count does not) — all with the frozen variant set, no new
    error names. An empty replacement counts as front matter, so clearing front
    matter is expressible while emptying a chapter stays `remove_chapter`'s job.
  - `to_tokens` is the §K.4/K.5 pull: `Into<ScopeSet>` accepts a scope, a scope
    list, or a `MutationEffect` (`braid.to_tokens(&effect)`), normalizes (dedupe,
    whole-book absorbs chapter scopes, either arrival order), resolves everything
    before emitting so no partial pull is possible, and returns corpus-then-run
    order. A scope naming an absent book errors (`BookNotFound`) rather than being
    skipped: callers already learn about removals from `effect.removed`.
  - EOL contract: USFM ingest keeps the supplied bytes verbatim (mixed endings
    preserved until first edit), token ingest declares its ending, chapter updates
    inherit the book's, and every post-edit byte is core's override output — an
    editor pushing LF tokens into a CRLF book still saves CRLF, and the hash is
    over those saved bytes.
- **Both ingest lanes, every test.** A `Lane` enum runs each lifecycle assertion
  through `BookInput::Usfm` and `BookInput::Tokens`; the corpus gate runs all 66
  `en_ulb` books through both and asserts byte-identical bytes, identical per-book
  hashes, and identical snapshot ids, then edits three books' chapters and proves
  each edited book pulls what a fresh parse of its new bytes yields while every
  untouched book stays identical token-for-token (ids included).

**Gates, exact counts:** `cargo test --workspace` 0 failed — braid 2 lib + 27
lifecycle (+2 ignored corpus), core 274 (271 + 3 line-ending tests), wire 176 + 2,
wasm 25; `cargo test -p braid --features serde` +1 (serde round-trip);
`cargo test -p braid --test corpus -- --ignored` 2 passed (66 books);
`lint_oracle_is_stable` byte-identical, zero rebless; `cargo test --release -p
usfm_onion_wire -- --ignored` 4 + 2 green, zero golden diffs; `npm run
test:packed` and `test:packed:web` both green at the pinned counts — 409 cases /
5,717,137 tokens / 715 chapter slices, unchanged (includes `tsc --noEmit`);
`cargo clippy -p braid --all-targets` clean. `pkg-bundler`/`pkg-web` restored,
never committed.

**Pre-existing failures (not introduced here, verified at `c22caa9` in a clean
worktree):** `usx::tests::validated_pass_fixtures_are_lossless_across_usfm_usj_usx_usj_usfm`
and its `usj` sibling fail on `testData/advanced/custom-attributes/origin.usfm`
under `cargo test --release -p usfm_onion --lib -- --ignored`. Both fail
identically before C2 — consistent with the recorded `testData` CRLF-taint
hazard. Flagged for whoever owns that fixture, not fixed here.

**Boundaries reported (owner/coordinator input wanted before C3):**

1. **Chapter-level USFM ingest produces unaddressed tokens.** `parse` derives
   both ids and sids per stream: a bare chapter fragment has no `\id`, so its
   tokens get `unknown-N` ids and **no sid at all**. Two consequences, both
   pinned by tests rather than papered over: a second `ChapterInput::Usfm` splice
   into the same book collides on `unknown-N` and is refused with the frozen
   `DuplicateTokenId`; and spliced content carries no canonical anchor. Braid did
   not re-address the stream, because core is address-agnostic by decision
   §2.1#7 and `OwnedToken` exposes neither an id nor a sid setter — re-addressing
   would need new core seams (a sid stamper, or the C1 minter extended past the
   formatter). Today's honest reading: `ChapterInput::Tokens` is the editor's
   lane and carries the caller's own ids/sids; `ChapterInput::Usfm` is native-host
   convenience with this caveat. C3 needs a ruling because findings anchor on
   sids.
2. **Caller-authored `OwnedToken` has no constructor.** `OwnedToken::from_parsed`
   is the only way to build one, so a host cannot construct an editor-authored
   token stream at all — the frozen path is wire's `TryFrom<TokenDto>` (§5.1),
   which does not exist yet. C2 therefore exercises the caller lane with
   parse-origin streams (pull, edit by splice, push) rather than synthesizing
   token payloads. Nothing in C2 needs the constructor; Phase F's DTO conversion
   does, and that is also where `IngestError::InvalidToken(TokenInputError)`
   acquires a producer.
3. **`OwnedToken` does not implement `FormattableToken`**, so core's formatter and
   `apply_token_fix` cannot run over resident tokens. C4's patch path will need
   either that impl or a conversion — recorded, not stubbed.

**Deviations:** (a) `LineEnding` in core, above; (b) `MutationEffect`/`to_tokens`
follow §K over the older §5.3 shapes (as the packet directed); (c) `Scope.chapter`
is `Option<ChapterLabel>`, not the packet's paraphrased `Option<u16>`, because
§5.2 forbids parsing a chapter label anywhere in braid; (d) the two unbuildable
`IngestError` variants above; (e) the crate landed as one commit rather than one
per epic step — the modules do not compile independently, and staging partial
files would have made the history less reviewable, not more.

**Stops:** none. No freeze/§J/§K contradiction was hit, every operation's
atomicity is provable from candidate-first construction, and nothing in the
packet required lint or patch machinery.

- Next: C3 — explicit whole-dirty-book `lint()` over the resident corpus with the
  declared `BookId` in core's lint context, plus the complete native snapshot.
  Boundary 1 above wants a ruling first.

## 2026-07-30 — F1 (format-token attribute passthrough) landed; two C2 follow-ups resolved

Three-item packet: drop the chapter-grain raw-USFM lane C2 flagged as boundary 1,
land F1 per the approved plan, and attempt C2's boundary 3 (`FormattableToken` for
`OwnedToken`).

**ITEM 1 — `ChapterInput::Usfm` removed; `update_chapter` is tokens-only
(owner-ruled).** C2's boundary 1 asked for a ruling; the ruling is to drop the
lane rather than fix its addressing. A bare chapter fragment has no `\id`, so
`parse` derives no book code and therefore no sid for its tokens, and its ids are
positional to that one fragment's parse — a second raw-text splice into a
different run of the same book starts its own numbering from zero and collides
on `IngestError::DuplicateTokenId` the moment both fragments share a book. No
real caller needs the lane either: an editor speaks tokens (it already holds the
caller-addressed stream in memory) and raw USFM enters at book/corpus grain
(`BookInput::Usfm`), where a real `\id` makes addressing possible in the first
place. `ChapterInput` is now `Tokens(Vec<OwnedToken>)` only; `update_chapter`
destructures it directly, no `parse` call left in `braid::lib`. Full rationale
plus the pre-made design answer for if a raw-text chapter lane is ever wanted
again (source-splice into the book's authoritative bytes + whole-book reparse,
so parse addresses the new content in real book context instead of reconciling
placeholder identities afterward) is recorded as a doc comment on
`ChapterInput` itself (`crates/braid/src/input.rs`) and mirrored as a freeze
note in `phase0-freeze.md`, so a future need does not re-litigate it.
Lane-specific tests removed: the second (`ChapterInput::Usfm` splice) scenario
in `a_duplicate_token_id_is_rejected_atomically`, and the first (`unknown-N`
id / no-sid) scenario in `resident_tokens_keep_the_ids_and_sids_their_source_gave_them`
— renamed in effect to a single-scenario test, since the token lane's
"caller-addressed round-trip" property is what remains to test. All 17 other
call sites were mechanical `ChapterInput::Usfm { source: X.to_string() }` →
`ChapterInput::Tokens(owned(X))` rewrites (`crates/braid/tests/lifecycle.rs`,
`crates/braid/tests/corpus.rs`); no behavioral change to any surviving
assertion.

**ITEM 2 — F1 landed per `plans/approved/format-token-attribute-passthrough.md`.**
`FormatToken` gains one field, `attribute_source: Option<String>` — the same
verbatim `|...` shape as `dto::Token.attributeSource`/`OwnedToken::attribute_list()`
— rather than a structured attribute list; format never inspects or edits
attribute content, so nothing needs to parse it apart. `FormattableToken` gains
`attribute_source()`/`set_attribute_source()` with `None`/no-op defaults, so any
token type that never carries attributes (most `LintableToken`-only fixtures)
implements nothing extra. The naive fix (concatenate the attribute string right
after the marker's own text) is wrong: USFM attaches an attribute list to the
opening marker structurally, but its verbatim text reads at the marker's
*closer*, right before `\w*`/end-milestone — `\w gracious|lemma="grace" \w*`,
not `\w |lemma="grace" gracious\w*`. Fixed by implementing `SerializableToken`
for `FormatToken` (attributes as an empty slice, `attribute_list()` returning
`attribute_source`, no `attribute_offset` override — a format pass is
normalizing, not byte-exact, so the closer rule is correct rather than a
byte-distance fallback) and delegating `format_tokens_to_usfm` to the existing
`tokens_to_usfm_reconstruct` closer-rule emitter instead of a second, naive
concatenation. wasm wiring: `token_value_to_format_token` (input direction,
`formatTokens()`) now threads `value.attribute_source` through; `map_format_token`
(output direction) sets the DTO's `attribute_source` from the native field and
leaves `attributes: Vec::new()` with a comment explaining why (format never
populates a structured list, `attribute_source` is the real fix). `map_walk_token`
and its `WalkToken` DTO are untouched — no attribute field exists there and the
plan's gate is format/fix paths, not walk. Golden fixture
`crates/usfm_onion_wasm/golden/outputs/attributes/{format.usfm,format-tokens.json}`
regenerated (`npm run golden:wasm:update`); `git status` on the golden tree
confirmed only those two files moved. Four new tests in `src/format/mod.rs`
cover the plan's own gates: full round-trip through `format_tokens`, id-less
tokens, id-bearing tokens (`set_id` called on every token first), and a proof
that synthesized tokens never invent an attribute list.

**ITEM 3 — STOP: `impl FormattableToken for OwnedToken` cannot be completed
honestly.** Every accessor `FormattableToken` requires except one can be
implemented honestly against `OwnedToken`'s existing catalog-backed fields
(`marker_metadata`/`structural_marker_info` are the same catalog a real parse
consults, so synthesizing e.g. `structural()` for a new token is not a fake).
The blocker is `synthetic_like`: the trait requires it to produce a full new
token of the caller's own type, and `OwnedToken` mandates a real
`StableTokenId` (`StableTokenId::new` rejects the empty string; there is no
`Default`) where `FormatToken`'s id is optional. `synthetic_like`'s signature
gives no id to store — there is no caller-supplied id to honor, and core's
address-agnostic, never-invents-ids principle (§2.1#7, the same principle
boundary 1 above rests on) forbids fabricating one to fill the slot. Any
implementation is therefore either dishonest (mint a fake id) or requires a
bigger design change out of this packet's scope: make `OwnedToken`'s id
optional, or extend `synthetic_like`'s signature to accept a caller-supplied id
(if the design allows a source at that call site — the coordinator/owner would
need to weigh in, since either change touches C4's contract, not just this
impl). No code changed for this item. C4's patch path over resident tokens
remains blocked on this ruling.

**Gates:** `cargo test --workspace` 278 (core) + 27 (braid) + 176 (wire) + 25
(wasm), 0 failed; `cargo test --test lint_oracle -- --ignored` byte-identical,
zero rebless; `cargo test --release -p usfm_onion_wire -- --ignored` 4 + 2
green, zero golden diffs; `npm run test:packed` and `test:packed:web` both
green, pinned counts unchanged — 409 cases / 5,717,137 tokens / 715 chapter
slices; `cargo fmt --check` clean on every touched crate; `pkg-bundler`/
`pkg-web` restored via `git checkout --`, never committed.

**Deviations:** none from the plan or the freeze. ITEM 3 is a stop, not a
deviation — no code was written that could deviate.

**Stops:** ITEM 3, as above.

- Next: owner ruling wanted on `synthetic_like`'s id gap before C4's patch path
  can be built over resident tokens; otherwise C3 (whole-dirty-book `lint()`,
  boundary 1 from the previous entry now resolved by this entry's ITEM 1).

## 2026-07-30 — Combined C2/F1 clean-room fix round: 2 P1 + 2 P2 landed

Clean-room review of the F1/C2-follow-ups packet above came back BLOCKED with two P1s and two P2s.
All four fixed in this round.

**P1-1 — wasm format/fix legs erased edited structured attributes.** F1's own conversion only ever
copied `attributeSource` and unconditionally returned `attributes: Vec::new()` on the way back out,
so the frozen DTO contract (an editor that edits an attribute's structured value clears
`attributeSource`, making `attributes` authoritative from then on) was silently violated the moment
a token went through `formatTokens`/`applyTokenFix`/`mergeDiffBlocks`/`revertDiffBlock` — all of
which share the same two conversion functions, `token_value_to_format_token` and `map_format_token`.
Root cause was one level deeper than the wasm boundary: `FormatToken` itself had nowhere to hold a
structured attribute list (its `SerializableToken::attributes()` unconditionally returned `&[]`),
so there was nothing for the wasm conversions to carry even if they tried. Fixed by giving
`FormatToken` a second field, `attributes: Vec<OwnedAttribute>`, alongside the existing
`attribute_source: Option<String>` — mirroring `dto::Token`'s own two-field shape — and wiring it
through `From<&Token<'a>> for FormatToken` and `SerializableToken::attributes()`. The existing
closer-rule reconstruct algorithm (`tokens_to_usfm_reconstruct`, reused by `format_tokens_to_usfm`)
already falls back to `format_attribute_list(attributes())` whenever `attribute_list()` is `None`
— that fallback is what makes an edited attribute survive once the field carrying it exists.
`OwnedAttribute` gained a `Serialize` derive (had none; `FormatToken` derives `Serialize` and
needed its new field to satisfy that). wasm's `token_value_to_format_token`/`map_format_token` gain
matching `wire_attribute_to_owned`/`owned_attribute_to_wire` helpers; the wire `AttributeItem`'s
required `span` has no analog on `OwnedAttribute` (which carries no per-attribute position at all),
so the owning token's own span stands in on the way out — best-effort, like every other span on a
`FormatToken` after a format pass, not byte-exact. Audited every format/fix/merge/revert leg
(`apply_token_fix`, `revert_diff_block`, `formatTokens`, `formatTokensMut`, `mergeDiffBlocks`,
`normalizeTokenSids`): all but `normalizeTokenSids` route through the same two conversions and are
covered by this one fix; `normalizeTokenSids` never touches attributes (it only stamps `sid`) and
`map_walk_token`/`WalkToken` were already out of scope (no attribute field exists there at all).
Regression tests added at both layers: `src/format/mod.rs`'s
`format_preserves_a_structurally_edited_attribute` (native) and
`crates/usfm_onion_wasm/src/lib.rs`'s `format_tokens_preserves_a_structurally_edited_attribute`
(wasm, the reviewer's exact transition — parse `\w word|note="a"\w*`, edit the structured value to
`"b"`, clear `attributeSource`, `formatTokens` — attribute survives as `note="b"`). The
`attributes/format-tokens.json` golden legitimately changed (the `\w` token's `attributes` array is
now populated alongside its `attributeSource`, exactly the plan's own predicted consequence);
regenerated via `golden:wasm:update`, verified via `git status` that only this one file moved, and
reported here explicitly rather than folded silently into the fix commit.

**P1-2 — pure corpus reorder was a contract violation.** `replace_corpus([GEN, EXO]) ->
replace_corpus([EXO, GEN])` changed `snapshot_id` (order is part of §J.4 identity) while reporting
an empty `changed`, so `is_noop()` claimed nothing happened and the new order was unobservable
through `to_tokens`. Owner-contract amendment (proxy-ruled), recorded as freeze §K.2a:
`MutationEffect` gains `reordered: Option<Vec<BookId>>` — `Some(full new book order)` when the
relative order of the books present both before and after a mutation changed, `None` otherwise;
`is_noop` becomes `changed.is_empty() && removed.is_empty() && reordered.is_none()`. Only
`replace_corpus` can produce a reorder (the only verb that can permute existing resident order —
`update_book` only replaces in place or appends, and every other verb only removes or leaves order
untouched), computed there by comparing the relative order of retained books before/after (additions
and removals alone do not count as a reorder — those are already observable via `changed`/`removed`).
`effect()`'s existing call sites are untouched; a new `effect_with_reorder` sibling backs both it
(`reordered: None`) and `replace_corpus` (`reordered: <computed>`). Serializes as usual under the
`serde` feature (plain derive, no new attribute needed). Regression: extended the existing
`snapshot_identity_is_content_derived_and_order_sensitive` test (already the reviewer's exact
`[GEN, EXO] -> [EXO, GEN]` case) to assert `reordered == Some([EXO, GEN])` and `!is_noop()`, plus a
second resubmission of the identical order asserting `reordered == None` and `is_noop()` (the
genuine no-op case).

**P2-3 — transient plan-label comments stripped, rationale kept.** `crates/braid/Cargo.toml`'s
`serde` feature comment no longer cites `epic §2.2#13/#18`; `crates/braid/tests/lifecycle.rs`'s CRLF
comment no longer cites `epic §2.2#16`; `src/format/mod.rs`'s attribute round-trip test comment no
longer opens with "The F1 gate:". Same cleanup applied proactively to comment text newly written in
this same round (a "Regression for the F1/C2 clean-room P1" comment in
`crates/usfm_onion_wasm/src/lib.rs`) rather than landing a fresh instance of the same defect.

**P2-4 — canonical-docs staleness deferred, not fixed.** `docs/usfm-onion.html` still does not
describe the resident subsystem. Ruling: explicit publication-phase deferral — the public braid
surface is still moving through C3/C4, so documenting it now would go stale twice. One-line deferral
note added at `plans/approved/braid/braid-epic.md`'s Phase F header (where the epic tracks
publication/wasm-package work); this ledger entry is the accompanying record. `docs/usfm-onion.html`
itself is untouched.

**Gates:** `cargo test --workspace` 279 (core) + 27 (braid) + 176 (wire) + 26 (wasm), 0 failed;
`cargo test --test lint_oracle -- --ignored` byte-identical, zero rebless; `cargo test --release -p
usfm_onion_wire -- --ignored` 4 + 2 green, zero golden diffs; `npm run test:packed` and
`test:packed:web` both green, pinned counts unchanged — 409 cases / 5,717,137 tokens / 715 chapter
slices, run both before and after the golden update to confirm the attribute fixture's own bless
did not shift them; `cargo fmt --check` clean on every touched crate; `pkg-bundler`/`pkg-web`
restored via `git checkout --` after every wasm dev build, never committed.

**Deviations:** none from the reviewer's packet.

**Stops:** none.

- Next: same as the previous entry — owner ruling on `synthetic_like`'s id gap (ITEM 3), otherwise
  C3.

## 2026-07-30 — Re-review: 1 P1 + 1 P2 fixed (attribute span fabrication, plan-citation sweep)

Re-review of the P1-1 fix above came back BLOCKED with one P1 and one P2.

**P1 — attribute spans were fabricated.** `wire_attribute_to_owned` discarded `AttributeItem.span`
entirely, and `map_format_token` substituted the owning marker's own span for every attribute on the
way back out — golden proof: a parsed attribute span of `71..84` became `59..62` (the `\w` marker's
own bytes) after `formatTokens`. Fixed at the carrier: `OwnedAttribute` (`src/token.rs`) gains `span:
Option<Span>` — `None` for an attribute an editor synthesized or structurally edited, the real parsed
span otherwise, never invented from some other token. The wire DTO's own `AttributeItem.span` was
the same shape of bug one layer up (`Span`, not `Option<Span>` — no way to express "no real
position" honestly), so it became `Option<Span>` too, matching `Token.span`'s existing convention;
`From<&NativeAttributeItem<'_>>` now wraps it in `Some`. `wire_attribute_to_owned`/
`owned_attribute_to_wire` (`crates/usfm_onion_wasm/src/lib.rs`) carry the span straight through in
both directions instead of dropping or substituting it; `owned_attribute_to_wire` no longer takes a
fallback-span parameter at all, since there is no fallback anymore. Every format/fix/merge/revert leg
shares these two conversions, so the one fix covers all of them (verified with the native
`cargo test -p usfm_onion_wasm` suite, which needs no pkg tree). Extended the existing regression
(`format_tokens_preserves_a_structurally_edited_attribute`) to capture the original attribute span
before the edit, assert it differs from the owning marker's own span (so the test can actually catch
fabrication), and assert the returned span matches it exactly after `formatTokens` — the previous
version of this test passed before the fix precisely because it never compared spans.

Golden consequence, re-blessed once cleared (see tree-constraint note below):
`attributes/format-tokens.json`'s `\w` token's attribute entry changed only its `span` —
`{start: 59, end: 62}` (the marker's bytes) to `{start: 71, end: 84}` (the attribute's own bytes) —
every other field (`key`, `value`, `text`, `isDefault`) byte-identical. `git status` confirmed this
was the only golden file touched.

**Tree constraint this round.** `pkg-bundler/*` initially carried an owner-modified, uncommitted
local build; the usual dev-build-then-`git checkout --` cycle would have destroyed it, so the P1 fix,
full workspace suite, lint oracle, release wire ignored gates, and `cargo test -p usfm_onion_wasm`
ran first with the two npm packed gates and golden regeneration explicitly deferred and described
(not run) pending an owner decision. The owner then ruled the pkg-bundler changes disposable and
cleared clobbering it; the deferred steps ran immediately after: `npm run golden:wasm` reproduced
the exact one-file mismatch predicted, `golden:wasm:update` re-blessed it (span-only diff confirmed
above), `golden:wasm:web` parity passed (7 fixtures), and both `npm run test:packed`/`test:packed:web`
passed at the unchanged pinned counts — 409 cases / 5,717,137 tokens / 715 chapter slices.
`pkg-bundler`/`pkg-web` restored via `git checkout --` once done, never committed — the standing rule
back in force now that the disposable build is gone.

**P2 — plan citations reintroduced.** New durable comments in the P1-2 fix round cited freeze `§K.2a`
and `§J.4` at `crates/braid/src/lib.rs`, `crates/braid/src/state.rs`, and
`crates/braid/tests/lifecycle.rs`. Stripped all three, keeping the behavioral rationale (why
`reordered` exists, why a pure reorder still changes `snapshot_id`). Swept this round's own new
comments (the `OwnedAttribute`/`AttributeItem` span docs, the extended regression test) for the same
pattern before committing — none found.

**Gates:** `cargo test --workspace` 279 (core) + 27 (braid) + 176 (wire) + 26 (wasm), 0 failed;
`cargo test --test lint_oracle -- --ignored` byte-identical, zero rebless; `cargo test --release -p
usfm_onion_wire -- --ignored` 4 + 2 green, zero golden diffs; `cargo test -p usfm_onion_wasm` 26
green (run standalone per the tree constraint, before the packed gates were cleared); `npm run
test:packed`/`test:packed:web` both green once cleared, pinned counts unchanged; `cargo fmt --check`
clean on every touched crate; `pkg-bundler`/`pkg-web` restored via `git checkout --`, never committed.

**Deviations:** none from the reviewer's packet.

**Stops:** none.

- Next: same as the previous entry — owner ruling on `synthetic_like`'s id gap (ITEM 3), otherwise
  C3.

## 2026-07-30 — C2/F1 batch CLOSED (reviewer verdict: dispatchable, no findings)

- Span-fix round confirmed: AttributeItem.span Option is the honest shape, no fabricated
  fallback remains, golden change legitimate and span-only, all conversion legs corrected.
  Reviewer notes the checked-in pkg declarations still show required span — consistent with the
  defer-pkg-regeneration-until-Phase-F policy; scratch build proves publication emits span?.
- Next: C3+C4 combined packet (owner-approved), single clean-room review at the end; C4's
  92-fix census runs FIRST as the early stop-detector against freeze §J.

## 2026-07-30 — C3 + C4 landed: resident lint, snapshots, patches, wire fields 5/6, restore seed

Five commits on `braid`, in order:

1. `1ebe20b` docs(braid): the C4 fix census — 92 corpus `TokenFix` payloads, recorded in the 0D
   ledger §9. **Ran first, as the packet required.** No contradiction with §J/§K/§L.
2. `7d32bfd` feat(token): the lossless residency boundary — `From<&OwnedToken> for FormatToken`
   plus `OwnedToken::from_format_token(working, anchor)`.
3. `021b36b` feat(braid): resident lint, complete snapshots, snapshot-bound patches.
4. `3d01f4a` feat(wire): finding fields 5 and 6 — `LintIssue.fix` survives the wire, closing
   §F.3's interim `None`.
5. `411b393` feat(braid): `restore_corpus`, the semantic warm cold-open seed path.

### Census (item 0) — the early stop-detector cleared

92 fixes over `testData` + `en_ult` + `en_ulb`, split exactly 48/32/12 as predicted. All are
single-replacement `ReplaceToken` with **empty** `label_params`; every one flattens to exactly one
§J.1 row; template texts max 6 bytes; `target_token_id` resolves and equals the finding's own
`token_id` 92/92. Two recorded consequences: `TokenFix.code` is a *remedy* code and never the
finding's lint code (0/92 match), so it must be interned; and six fixture cases have two findings
proposing different whole-token replacements for the **same** token, which is what makes §J.2's
snapshot-bound identity and §J.6's hash check load-bearing rather than belt-and-braces.

### What the freeze gained

- **§N** — field 5/6 framing, closing §F.3: dense addressing column, 24-byte patch records, 20-byte
  rows, every check the decoder runs, and three recorded consequences (fields 7/8 now have two users;
  the flattening rule is duplicated deliberately per §J.8; ordinal bridging is a Phase D task).
- **§N.3** — the one known limitation of `(SnapshotId, ordinal)`: `update_config` changes the finding
  set without changing source bytes, so an ordinal held across it could name a different patch.
  Mitigated by dropping caches (a held id fails `UnknownPatch` until a fresh lint), not eliminated —
  eliminating it needs a third identity component §J.2 deliberately lacks.
- **API ledger append** — 9 landed as ledgered, 4 new, 9 amended, 5 new core exports, 9 frozen
  variants/fields deliberately unbuilt with the reason each has no producer.

### Gates

Full workspace suite green: core 282 (+3), braid 46 across four files (4 lib + 27 lifecycle + 8
resident lint + 7 patches), wire 179 (+3), wasm 27 (+1). Lint oracle `--ignored` byte-identical, no
rebless. Release ignored gates: wire lib 4, wire corpus 2 (**62,948 findings, all 92 fixes
round-tripping, per-code counts asserted at 48/32/12**), braid corpus 3 (including the new
whole-corpus ordering gate over all 66 `en_ulb` books). `npm run test:packed` and `test:packed:web`
both pass at 410 cases / 5,717,153 tokens — **counts legitimately changed** from 14 good + 11
malformed goldens to 15 + 15 (one new good vector, four new malformed patch vectors); no existing
golden's bytes changed. Golden generators re-run and their output committed. `pkg-bundler`/`pkg-web`
restored after the dev builds.

### Two things surfaced rather than worked around

- **Core's whitespace remedies do not satisfy their own rule on tokens.** The fix prepends the
  whitespace into the marker token's own source; the rule reads the *previous* token's trailing
  character. Patched bytes are correct and a fresh parse is clean, but the resident token stream
  still reports the finding, and holds a marker token no lexer would produce. Pinned by test, written
  up in the freeze with a recommended reshape (express the remedy as an inserted token) and its cost
  (re-runs the 92-payload census; needs an insert-before op the frozen set lacks). **Owner decision.**
- **The warm lint cache is blocked on the stamps.** `restore_corpus` seeds source + tokens (saving
  the parse) but cannot accept a cached lint contribution: §8.2/§8.3's lint-config fingerprint and
  engine stamp are still OWNER-DECISION, and a stamp that does not cover what changed would accept
  stale findings. `prime_lint_cache` is unbuilt for the same reason.

### Next

Clean-room review of this batch (Will runs it). Then Phase D — which the full finding-and-patch gate
above now unblocks — with the ordinal-bridging task from §N.2 as its first patch-related item.

## 2026-07-31 — Owner rulings on the C3/C4 surfaced items

- Whitespace-remedy shape: **A — deferred.** The remedy should have inserted a newline token;
  reshape charter recorded at plans/candidates/fix-whitespace-remedy-token-shape.md; braid ships
  with editor-parity behavior meanwhile (pinned by test).
- Warm-lint-cache stamps: **A** — coordinator drafts the config-fingerprint and engine-stamp
  definitions with the Phase D packet for owner adjudication there.
- Pre-existing USJ/USX ignored-test failures: owner-dispositioned as expected — USJ/USX are not
  intended to be lossless given the nature of their keys; the two `#[ignore]`d "lossless" tests
  overclaim. No chase; noted here so the red `--ignored` core gate has a recorded explanation.

## 2026-07-31 — C3/C4 clean-room fix round: three P1s landed

Reviewer verdict on the C3/C4 batch was NOT DISPATCHABLE with three P1s, all confirmed by triage.
Fixed in three commits, each with the regression the reviewer asked for.

- **`3fb293b` P1-1 — preview never mints.** `preview_patch` shared apply's code path, so it invoked
  the handle-held minter: two previews of a synthesizing patch would disagree, and every preview
  would spend ids out of the application's own space (invisible today only because no corpus fix
  synthesizes). Per the proxy ruling — minting is admission to residency, and a preview is never
  admitted — the pass is now split from the admission: `applied_working` is a pure `&self`
  projection with no minter, shared verbatim by both verbs, and `admit` is the single place the
  minter is invoked (the §L post-pass sweep, with the conversion checkpoint behind it).
  `preview_patch` therefore takes `&self` and **returns `Vec<FormatToken>`** — the id-optional
  working type. Chosen over placeholder ids: `Vec<OwnedToken>` structurally requires real
  `StableTokenId`s, and fabricating ids that *look* resident is precisely the confusion the split
  exists to prevent. A survivor carries the resident id it already had; a token the fix would
  synthesize carries `None` until an apply grants one.
- **`2d15a6a` P1-2 — the seed manifest validates first.** `restore_corpus` checked uniqueness on
  already-built, already-filtered candidates, so the reviewer's repro (a valid GEN record plus a
  mismatched GEN record) seeded one and reported the other instead of refusing atomically. Uniqueness
  now runs on the manifest exactly as supplied, before anything reads a record; the rule moved to
  `validate_unique_keys` over `(book, source key)` pairs so the corpus and seed paths share one
  implementation. Both regressions added (duplicate book, duplicate source key, each with one record
  that would have been content-rejected).
- **`d3ea87e` P1-3 — a fix that edits nothing is refused.** Typed refusal at encode on the wide-SID
  precedent: new `EncodeError::EmptyFix`, raised by the semantic encoder *and* the section writer, so
  no path can lay out what the decoder refuses. The decoder now rejects a zero-row record explicitly,
  per record, instead of failing later in materialization. Braid's resolution refuses the same shape
  by making it unrepresentable — `ResolvedFix::new` returns `None`, so "a patch has at least one row"
  is a property of the type. **Deviation from the ruling's wording, flagged:** braid's refusal is
  structural rather than a typed error, because the only boundary that resolves fixes is `lint()`,
  which is infallible by construction (no `LintError` producer) — there is no `Result` to carry one,
  and a `debug_assert` plus unrepresentability is stronger than an error that cannot be returned.

### Gates after the round

Workspace suite green: core 282, braid 50 (6 lib + 27 lifecycle + 10 resident lint + 7 patches),
wire 183, wasm 27. Lint oracle `--ignored` byte-identical. Release ignored: braid corpus 3, wire lib 4
+ corpus 2 (62,948 findings, 92 fixes, per-code counts still 48/32/12). `npm run test:packed` and
`test:packed:web` both pass at 410 cases / 5,717,153 tokens; **malformed goldens 15 → 16** (the new
`zero-row-patch-record` vector), good goldens unchanged at 15, no existing golden's bytes changed.
Golden generators re-run. `pkg-bundler`/`pkg-web` are owner-modified and were left that way — never
committed, per the standing ruling.

## 2026-07-31 — PHASE C CLOSED (reviewer verdict: code dispatchable; doc amendment applied)

- C3/C4 fix-round re-review: all three P1s confirmed fixed, both adjudications approved
  (preview as pure `&self -> Vec<FormatToken>` projection; structural empty-fix omission over an
  impossible LintError). The one P2 (normative preview signature) is applied: freeze C4 ledger
  row re-amended, epic §8.2 signature and Phase C gate wording corrected — preview-equals-apply
  is semantic content before admission.
- Pre-existing catalog.rs formatting drift (outside any packet) cleared in a style commit so
  `cargo fmt --all --check` is green again.
- Phase C entire: C1 (core pull-forwards), C2/F1 (residency floor + format seams), C3/C4
  (resident lint, snapshots, patches, wire fields 5/6, finding-and-patch gate, restore seed).
  Next: Phase D (packed publication) — stamps adjudication rides the packet.

## 2026-07-31 — Phase D opened: stamps adjudicated and the warm lint cache unblocked (step 3 of 4)

Owner routed Phase D (packed braid publication) here. Delivered this round: the
§8.2/§8.3 stamp definitions (freeze §O, proxy-ruled, veto window open) and step 3
of the packet's four — the warm lint cache unblock C2/C3 both deferred pending
those stamps. Steps 1, 2, and 4 (container composition, publication reuse,
publish→decode→compare corpus gate) are **not** done this round — see the
explicit scope note below.

**Stamps (freeze §O).** `LintConfigFingerprint`/`LintEngineStamp` land as new
braid types (`crates/braid/src/stamps.rs`), each a `u64` newtype computed rather
than caller-supplied. `LintConfigFingerprint::of(&LintOptions)` hashes derived
`Debug` output (a fixed encoding braid controls, not serde JSON — no new
dependency, no portability claim needed since nothing decodes a fingerprint back
into a value). `LintEngineStamp::current()` hashes
`"usfm_onion@{CRATE_VERSION}:rules{RULES_VERSION}"`; both consts are new core
exports (`usfm_onion::lint::CRATE_VERSION`, `usfm_onion::lint::RULES_VERSION: u16
= 1`) rather than reusing wire's `FINDING_SECTION_RULES_VERSION`, because braid
must not depend on wire — the same duplicated-constant judgment call as
`canonical_order` (§J.8).

**Warm lint cache unblock.** `PrimeRejectReason` gains its five previously-frozen
variants (`BookNotResident`, `SourceHashMismatch`, `ConfigFingerprintMismatch`,
`EngineStampMismatch`, `InvalidPatch`), all with real producers now.
`BookRestoreInput` regains `lint: Option<BookLintPrime>`; `CorpusRestoreInput`
gains `config_fingerprint`/`engine_stamp`. `Braid::prime_lint_cache(LintPrimeInput)
-> PrimeReport` is new, priming an already-resident corpus by `BookId` (the
producer `BookNotResident` needs, since `restore_corpus`'s cached-lint entries
always arrive alongside the very book they prime and can never miss). Clarified
and re-documented `RestoreReport`'s own contract while wiring this in: residency
and lint-priming are independent facts. `SourceTokenMismatch` is the one reason
that refuses a book's residency outright; every other reason gates only its
*cached lint contribution* — the book still seeds (tokens installed, no
lex/parse) and is simply dirty, exactly as if `lint` had been absent. A book can
therefore now appear in both `seeded` and `rejected`, which it could not before
this round (deviation from the epic's literal wording, recorded here rather than
guessed at silently — see freeze §O.4).

`BookLintPrime` does not carry a serialized patch table (no `TokenPatch`/
`Vec<PatchRow>` field): re-flattening a cached `LintResult`'s fixes against the
current resident token stream is not core rule work (the same cheap position
lookup `resolve_fixes` already does for a fresh result), so nothing about the
"clean lint performs no rule work" gate is spent re-deriving it. `InvalidPatch`'s
producer is `BookState::try_resolve_cached_fixes`, a stricter twin of
`resolve_fixes` that refuses the whole contribution if even one fix fails to
resolve, rather than silently dropping it (foreign data does not get the same
trust as a book's own just-computed result). `BookLintPrime`/`LintPrimeInput`
derive `Serialize` only, not `Deserialize`: `LintResult` embeds
`LintIssue::template: &'static str`, which has no honest `Deserialize`, and the
intended producer is always the composing adapter constructing the value
natively in-process, never a real serde boundary (freeze §O.6).

New public API: `braid::LintConfigFingerprint`, `braid::LintEngineStamp` (+
`stamps.rs`); `braid::BookLintPrime`, `braid::LintPrimeInput`, `braid::PrimeReport`;
`Braid::prime_lint_cache`; `PrimeRejectReason`'s five new variants;
`CorpusRestoreInput::new`'s signature gains the two stamp parameters;
`BookRestoreInput.lint`. Core gains `usfm_onion::lint::CRATE_VERSION`,
`usfm_onion::lint::RULES_VERSION`.

Tests: `crates/braid/tests/lint_prime.rs` (new) — valid contribution adopted
without leaving the book dirty; source-hash mismatch seeds the book but rejects
only its cache; config-fingerprint and engine-stamp mismatches reject atomically;
a restamped-hostile case (hash and both stamps agree, but a fix's
`target_token_id` names a token the book does not hold) is refused as
`InvalidPatch`; `prime_lint_cache` accepts a valid contribution on an
already-resident book and rejects a book that is not resident. `resident_lint.rs`'s
four existing restore-corpus tests updated for the new `CorpusRestoreInput`
shape (a `matching_stamps()` helper, unrelated to lint-cache priming, keeps them
exercising plain token/source seeding as before).

**Gates:** `cargo test --workspace` 282 (core) + 59 (braid: 8 lib + 27 lifecycle +
7 lint_prime + 7 patches + 10 resident_lint) + 180 (wire) + 27 (wasm), 0 failed;
`cargo test --test lint_oracle -- --ignored` byte-identical; `cargo test --release
-p braid -- --ignored` 3 (corpus-scale, all 66 en_ulb books); `cargo test --release
-p usfm_onion_wire -- --ignored` 4 + 2, zero golden diffs; `npm run test:packed`
and `test:packed:web` both green at 410 cases / 5,717,153 tokens (15 good / 16
malformed goldens) — identical to the count already ledgered at Phase C's close
(the `zero-row-patch-record` vector from the C3/C4 round), confirming this
round introduced no packed-surface drift of its own; `cargo fmt --check` clean;
`pkg-bundler`/`pkg-web` restored via `git checkout --`, never committed.

**Scope note — steps 1, 2, 4 not done this round.** Composing braid's semantic
snapshot with wire's byte encoding into a complete packed container (step 1),
per-book publication reuse of already-encoded sections (step 2), and the
corpus-scale publish→decode→compare gate against all 66 en_ulb books (step 4)
are a substantially larger unit of work than the stamp definitions and the
lint-cache unblock: wire's encode side (`token_codec`, `token_section`,
`container`) is currently entirely private (`#[allow(dead_code)] mod ...`) with
no public composition entrypoint at all — building one, plus a reuse-cache
design that cannot risk adopting an unverified section, plus a release-mode
66-book gate, deserved a dedicated pass rather than a rushed addition on top of
the stamp work. Not a STOP (no freeze contradiction, no impossible stamp slot,
no partial-adoption forced) — a scope decision, recorded rather than silently
claimed done. Next: steps 1/2/4, building wire's public compose entrypoint (the
"composing adapter" is `usfm_onion_wasm`'s Rust internals per epic decision 8 —
"wasm is their composition root" — not the public `wasm_bindgen` surface, which
stays Phase F's).

**Deviations:** `RestoreReport`'s seeded/rejected overlap (§O.4, above) — not
explicitly spelled out by the epic's `restore_corpus` prose, and recorded rather
than assumed silently. Stamp definitions and byte-placement judgment calls are
proxy-ruled per the packet's own instruction, recorded as freeze §O with an open
owner veto window.

**Stops:** none.

## 2026-07-31 — Phase D steps 1, 2, 4: packed publication composition landed

Completes the phase Sonnet's step-3 round opened. Three commits:

1. `f15f6ce` feat(wire): public corpus composition surface + lint-stamp bytes (step 1)
2. `0361f19` test(wire): prove publication reuse rather than infer it (step 2)
3. *(this round's third commit)* feat(wasm): the composing adapter + the corpus publish→decode→compare
   gate (steps 3 groundwork + 4)

### The public wire surface (for the Phase 0 ledger)

`usfm_onion_wire::corpus_codec` is the whole publication boundary; everything below it stays private,
so there is no lower-level way to assemble a container by hand:

| item | shape |
| --- | --- |
| `encode_corpus(snapshot_id: u64, lint_stamps: Option<LintStamps>, sections: &[CorpusSection]) -> Result<EncodedCorpus, EncodeError>` | amends §7's two-parameter signature; stamps are corpus-level per §O.3 |
| `CorpusSection::{Fresh(CorpusSectionInput), Cached(CachedBookSections)}` | semantics to encode, or bytes to splice |
| `CorpusSectionInput { book, tokens, findings }`, `CorpusSectionTokens::{Parsed, Owned}` | as frozen in §7 |
| `EncodedCorpus { bytes, sources, books }` | `books` is the new per-book reuse sidecar |
| `PublishedBook { book, source_hash, bytes, sections }` + `as_cached()`, `PublishedSection { kind, offset, len }` | the sidecar and its extents |
| `LintStamps { config_fingerprint, engine_stamp }` (re-exported from `schema`) | §O's pair as wire data |
| `verify_corpus(packed, sources) -> Result<VerifiedCorpus, DecodeError>` | the corpus-level read §7 never specified |
| `VerifiedCorpus { snapshot_id, lint_stamps, books }`, `VerifiedBook.lint_stamps` (new field) | — |
| `EncodeError::EmptyFix` (C4) and `LayoutRefusal::{CachedSectionUnreadable, CachedSectionMismatch}` | typed reuse refusals |

Container-internal additions: `ContainerSection`, `write_container_sections`, `inspect_section`
(private); `verified_book`, `encode_findings_with`, `owned_stable_ids` (crate-internal extractions,
not second copies).

### Stamp placement decision (freeze §P.1)

Finding-section **field 9**, one optional 16-byte record `{config_fingerprint:u64, engine_stamp:u64}`.
The pair has to be *data in the artifact* or §O.3's comparison is vacuous — an adapter recomputing it
in-process on a cold open compares today's values with themselves and passes every stale cache. The
finding section rather than the container header because the header has 8 reserved bytes (room for one
`u64`) and folding the pair into one hash would destroy the distinction
`ConfigFingerprintMismatch`/`EngineStampMismatch` make; widening the header would re-bless every
golden and every JS layout constant for a fact that only matters when findings exist. Absence is
meaningful: unstamped findings may be read, never adopted. **Blast radius, reported:** no existing
golden's bytes changed (`encode_book` supplies no stamps, so it emits no field 9); the generated
`js/wire-schema.js` `FINDING_FIELD` table gained a row (9 → 10) and `test-wire-schema-import.mjs`'s
pinned count with it.

### §P.4 — the one freeze amendment the work forced

`issue_to_row` required `token_id` and `span` to co-occur, so **every** resident finding was refused
at encode: owned tokens are spanless by design (§2.2#15 / D1). Amended so an anchor with no span
stores the `(0,0)` "whole token" pair — what all 61,166 corpus findings already store — while a span
with no anchor stays refused. No byte moved; a published resident finding gains the correct span on
decode.

### The composing adapter (steps 3/4)

`crates/usfm_onion_wasm/src/publication.rs` — private, no `wasm_bindgen`, `dead_code`-allowed until
Phase F wires the public class. `usfm_onion_wasm` gains a `braid` dependency, which is the epic's own
§4.1 composition-root arrangement (the one crate allowed to depend on both). `PublicationCache::publish`
recomputes exactly the dirty books' lint, re-encodes exactly the books whose bytes *or stamps* moved,
splices the rest, and reports `encoded`/`reused` as data rather than leaving reuse to be inferred.

The cache key is `(book, source_hash, stamps)`, not the hash alone: a configuration change rewrites
what a book's findings are while leaving its bytes untouched, and the published section carries those
findings — pinned by its own test.

### Gates

- `cargo test --workspace`: **570 passed, 0 failed** (core 282, braid 59, wire 196, wasm 30 + the
  small binaries).
- `cargo test --test lint_oracle -- --ignored`: byte-identical, no rebless.
- Release ignored: braid corpus 3, wire lib 4 + corpus 2, **wasm 1 — the Phase D corpus gate**: all 66
  `en_ulb` books published as one container, decoded through `verify_corpus`, and compared against the
  native snapshot (per-book token counts, source hashes, stamps, and every finding field including
  `fix`); the census's one `en_ulb` fix survives publication. Then one chapter edited: **1 book
  re-encoded, 65 spliced**, each spliced book's sections asserted byte-identical to the first
  publication's, and the republication verified against the new truth.
- Reuse proof at unit scale hands back *only bytes* for the untouched book — nothing to re-encode
  from — so a complete verifying container proves reuse rather than inferring it.
- `npm run test:packed` / `test:packed:web`: 410 cases / 5,717,153 tokens, 15 good + 16 malformed
  goldens — unchanged counts. `pkg-bundler`/`pkg-web` restored, never committed.

### Deviations

- `encode_corpus`'s signature gained the stamps parameter and the `CorpusSection` wrapper (§P.2 table).
- `verify_corpus`/`VerifiedCorpus` are new surface §7 never specified: `verify_book` deliberately
  refuses a multi-book container, so a publication had no way to be read back at the grain it was
  written.
- The reuse cache lives in the adapter, not in braid. The packet's wording ("braid-side publication
  caches per-book encoded sections") would require braid to hold wire bytes and call wire to make
  them, which the dependency rule forbids; braid supplies the facts the cache keys on
  (`books()`, `dirty_books()`, `lint()`, `expected_snapshot_id()`) and the adapter owns the bytes.

### Stops

None.

## 2026-07-31 — Phase D clean-room fix round: four P1s + one P2 landed

Reviewer verdict on the Phase D composition batch was NOT DISPATCHABLE with four P1s and a doc P2, all
confirmed by triage. The §P amendments and boundary choices passed adjudication; these are the holes
found inside them.

- **P1-1 — the reuse key was blind to token identity.** Braid counts token identity as a content change
  even when the bytes are identical, but the adapter's cache key was `(book, source_hash, stamps)`, so
  an `update_book` with byte-identical tokens under fresh ids was served the *old* sections — old ids,
  and finding anchors and fix targets naming tokens that no longer exist. Braid now exposes the missing
  fact as a stamp: `TokenIdentity` (xxh3-64 over everything the owned-token encoding carries that the
  source hash does not pin — the stable ids first, plus the canonical anchors, marker nesting,
  book-code validity, parsed numbers, and attribute structure, all of which a byte-identical stream can
  differ in because a book's bytes are its tokens' text concatenated). Exposed on `BookEntry` and
  `BookLintSnapshot`, computed beside the source hash in `BookState::build`/`rebuilt`. `SnapshotId` was
  deliberately **not** widened (§J.5 keeps it source-only). The adapter's key is now
  `(book, source_hash, token_identity, stamps)`. Regressions: the reviewer's identity-only repro at
  adapter level (re-encode happens, and the republished anchors *and fix targets* are the new
  `editor-*` ids) and at braid level (same bytes, same corpus identity, different token identity).
- **P1-2 — splices bypassed stamp agreement.** A cached finding section recorded under old stamps could
  be spliced into a publication claiming new ones, signing a lie about how those findings were
  produced. Cached finding sections must now record exactly the requested pair, presence included:
  `LayoutRefusal::CachedSectionStampMismatch`. All three disagreement directions are tested (different
  pair, licence vanishing, licence appearing) plus both agreeing cases.
- **P1-3 — `verify_corpus` accepted orphan finding sections.** The corpus universe was built from token
  entries, so a finding entry for a book with no token section was never opened — an unverified section
  in a container the caller then treats as whole. Every finding entry must now pair with a token entry
  of the same book. Both the writer and the reader refuse the shape, so exercising the *reader* needed
  bytes no writer here can produce: new malformed golden `orphan-finding-section` (renames the token
  section, orphaning the finding section beside it, restamped) drives both `decode_book` and
  `verify_corpus`.
- **P1-4 — clean corpora could not restore their negative cache.** The adapter emitted stamps only when
  some book had an issue, so a fully clean project reopened with no licence and re-ran every rule. An
  empty `LintResult` is evidence; the distinction the format draws is **no finding section** (not
  computed) versus **a finding section with zero rows** (computed, clean). Publish now always stamps,
  freeze §P.1's wording is corrected to "no finding sections", and the regression runs the whole warm
  path end to end: an all-clean corpus publishes, verifies, primes a fresh handle from the decoded
  bytes, and reopens with `dirty_books()` empty — the no-rule-work assertion.
- **P2-5** — `LintConfigFingerprint`'s doc no longer claims semantic normalization: it is a
  deterministic representation of the configuration *as supplied*, so a reordered `enabled_codes` may
  fingerprint differently, which errs toward refusing a valid cache rather than accepting an invalid one.

### Gates after the round

`cargo test --workspace` **574 passed, 0 failed** (core 282, braid 60, wire 201, wasm 32). Lint oracle
`--ignored` byte-identical. Release ignored: braid corpus 3, wire lib 4 + corpus 2, **wasm 1 — the
Phase D corpus gate still passes with the widened cache key** (66 books published, decoded, compared;
one chapter edited → 1 re-encoded, 65 spliced byte-identically). `npm run test:packed` /
`test:packed:web` green at 410 cases / 5,717,153 tokens; **malformed goldens 16 → 17** (the new
`orphan-finding-section` vector), good goldens unchanged at 15, no existing golden's bytes changed.
`pkg-bundler`/`pkg-web` restored, never committed.

### New public API from this round

`braid::TokenIdentity` (+ `BookEntry.token_identity`, `BookLintSnapshot.token_identity`);
`usfm_onion_wire::LayoutRefusal::CachedSectionStampMismatch`.

## 2026-07-31 — Phase D re-review: the last P1 (drift-proof token identity) closed

One P1 remained: `TokenIdentity` was accessor-based, so it was neither drift-proof (a field added to
`OwnedToken`, its private payload, or `OwnedAttribute` would compile silently without joining the
stamp) nor complete — `attribute_list` `None` and `Some("")` hashed identically though the wire
distinguishes them (absent = no attribute row; present-empty = a row with an empty span), and
`OwnedAttribute.source` was not hashed at all even though the owned encoder *searches for that exact
spelling* to place each attribute's span.

Fixed where the reviewer put it: the projection moved **into core**, beside the private types that
caused the accessor fallback in the first place. `OwnedToken::hash_wire_identity<H: Hasher>`
destructures the token, all six payload variants, the marker-attribute record, and each
`OwnedAttribute` **with no `..` anywhere**, so a new field on any of them is a compile error at that
site — the drift-proof claim is now a property of the code rather than an assertion in a comment.
Every `Option` carries a presence byte and every variable-length value is length-framed, so `None`
cannot hash like `Some("")` and `("ab","c")` cannot hash like `("a","bc")`. Two fields are excluded
*by name with the reason* rather than skipped — `MarkerMetadata`/`StructuralMarkerInfo` (the encoder
re-derives both from the marker name, so they cannot change the bytes) and `OwnedAttribute.span` (the
owned encoder recomputes it by locating `source` in the text it just emitted) — and those exclusions
are themselves destructured, so a new field forces the question to be answered again instead of
inheriting a stale answer.

Braid keeps the algorithm: `TokenIdentity::of` feeds the projection into its own `Xxh3`. Core owns
completeness, braid owns the digest.

Regressions: `format::tests::wire_identity_separates_attribute_presence_and_spelling` (absent vs empty
list; two attributes differing only in their own spelling; and a framing collision, `id`+`source`
split two ways) and
`publication::tests::an_attribute_spelling_change_re_encodes_rather_than_serving_stale_bytes` — a
push that changes only each attribute's recorded spelling, leaving **every emitted byte identical**
(asserted against the first publication's own source hash), which is exactly why the source hash could
not catch it: the adapter re-encodes GEN, still splices EXO, and the new sections are not the stale
ones. Both identity-only repros from the previous round stay green.

Documentation: the six-category enumeration is gone from the doc comments on both sides — braid's
`TokenIdentity` and the adapter's `CachedBook` now defer to the projection instead of restating what it
covers, since a restated list is the thing that goes stale.

### Gates

`cargo test --workspace` **576 passed, 0 failed** (core 283, braid 60, wire 201, wasm 33). Lint oracle
`--ignored` byte-identical. Release ignored: braid corpus 3, wire lib 4 + corpus 2, wasm 1 — the Phase D
corpus gate still passes. `npm run test:packed` / `test:packed:web` green at 410 cases / 5,717,153
tokens, 15 good + 17 malformed goldens — unchanged from the previous round, no golden's bytes moved.
`pkg-bundler`/`pkg-web` restored, never committed.

### New public core API

`usfm_onion::token::OwnedToken::hash_wire_identity` (generic over `std::hash::Hasher`).

## 2026-07-31 — Phase D CLOSED (clean-room verdict) + comment typo fix

Clean-room re-review verdict on `0ac5079`: "Phase D closes. No correctness or spec findings
remain: the projection is exhaustive, drift-proof, collision-framed, and aligned with the owned
encoder." Reviewer independently verified the full workspace suite, lint oracle (byte-identical),
release wasm Phase D corpus gate, braid corpus gates 3/3, both new focused regressions, fmt, and
the commit diff. Reviewer skipped the packed npm gates locally (working-tree `pkg-*` state);
director had run both green on the same commit.

One non-blocking finding, fixed in this commit: the fixture comment at the adapter's
attribute-spelling regression said the spelling was "widened" and the bytes changed, while the
fixture actually narrows each attribute's recorded spelling by its leading character and
deliberately keeps every emitted byte identical. Comment corrected to describe the real mutation;
no code changed. Touched test re-run green; `cargo fmt --all -- --check` clean.

Phase D deliverables, final: `usfm_onion_wire::corpus_codec` (encode_corpus/verify_corpus,
Fresh/Cached sections, splice refusals), wasm composing adapter + PublicationCache keyed on
(book, source_hash, token_identity, stamps), clean-corpus negative cache, §O stamps, field-9
lint_stamps, and the core-owned `OwnedToken::hash_wire_identity` projection.

Remaining: Phase E (baseline mutation, missing-baseline policy, exact is_dirty, scoped to_usfm),
Phase F (public wasm Braid, restoreCorpus composition, reconcile helper, editor parity, pkg regen).

## 2026-07-31 — Phase E, step 0: renamed `dirty_books` to name its axis

Owner-directed scope addition ahead of Phase E: `Braid::dirty_books()` filtered
`book.lint_dirty` — the lint-cache axis, cleared by a recompute — and shared the
word "dirty" with the new baseline axis Phase E was about to add
(`is_dirty`/exact serialized equality). Renamed to `books_awaiting_lint()` with a
doc comment stating plainly it is the lint-recompute axis, not the baseline one;
"dirty" on its own is now reserved for `is_dirty`. `lint_dirty` (the private
field) keeps its name — it already names its own axis. All call sites updated
(five test files plus `usfm_onion_wasm::publication`); no compat shim, pre-release.

Gate: `cargo test --workspace` green (no behavior changed, pure rename).

## 2026-07-31 — Phase E, step 1: baseline mutation and exact `is_dirty`

Added `crates/braid/src/baseline.rs`: `BaselineState` (one book's declared
baseline — exact source, hash, tokens, line ending, chapter runs) and
`BaselineError { Scope(ScopeError), MissingBaseline { books } }`, matching the
frozen §6.5 shape exactly.

`BookState` (corpus.rs) gains an `Option<BaselineState>` field. It defaults to
`None` on a freshly built candidate and is carried forward untouched by every
content mutation (`rebuilt`, `inherit_cache`, and the changed-but-still-resident
branches of `update_book`/`replace_corpus`) — a baseline changes only through
`set_baseline`/`clear_baseline`, never as a side effect of editing current
content. This needed a deliberate fix mid-implementation: the first pass carried
the baseline forward through `rebuilt`/`inherit_cache` but not through
`update_book`'s/`replace_corpus`'s "content differs, book already resident"
branch, which was silently wiping a declared baseline on the very next edit;
caught by `a_content_mutation_carries_the_baseline_forward_unchanged` and
`diff_baseline_equals_the_stateless_core_diff` failing in the first test run.

`Braid::set_baseline`/`clear_baseline` implemented per §5.3/§9. Deviation
recorded (§9 does not say what happens when the named book has no current
content, and `IngestError`'s frozen variant set has no "book not resident"
case): `set_baseline` on a book with no current resident content installs it
fresh — the same fallback `update_book` already uses for a book it has not
seen before — with its baseline set to that same content, since nothing yet
diverges from it. That case does report the new book in `changed` (current
content genuinely came into existence); the ordinary case (book already
resident) always returns the no-op effect shape, since only the baseline slot
changes. `clear_baseline` on an absent baseline is a no-op, matching its
infallible signature.

`Braid::is_dirty(scope)` implemented per §9/Gate-0F-amendment-C: missing
baseline is always dirty; `All` is dirty if any resident book is; `Chapter`
compares only that run's own bytes against the same-label baseline run, a
missing baseline run for that label is dirty, and duplicate labels on either
side (current or baseline) return the typed `AmbiguousChapter`-shaped
`ScopeError`. Hash is used as a cheap selector, never trusted as proof of
equality on its own — a match still falls through to the exact byte
comparison.

`Braid::diff_baseline(scope)` implemented per §5.3/§9: directly wraps core's
own `diff_skeleton(baseline_tokens, current_tokens)` in the ordinary
`ScopedOutput` envelope, adding no diff model of its own. Errors typed
`MissingBaseline { books }` — collecting every requested book missing a
baseline for `All`, not just the first — rather than synthesizing an
"everything added"/"everything unchanged" skeleton.

New tests: `crates/braid/tests/baseline.rs`, 15 tests, both ingest lanes where
applicable — missing-baseline-is-dirty, exact-equality tracking including exact
revert, chapter-scope dirty (same-label comparison, missing baseline run,
duplicate-label ambiguity on either side), baseline mutation leaves current
state provably untouched (snapshot id, bytes, dirty stamps), clearing an absent
baseline is a no-op, a content mutation carries the baseline forward, resident
diff equals the stateless core diff byte-for-byte, chapter-scoped diff, and the
fresh-install case.

Gate: `cargo test --workspace` green; `cargo test --release --test lint_oracle
-- --ignored` byte-identical; `cargo test --release -p braid -p
usfm_onion_wire -p usfm_onion_wasm -- --ignored` green; `npm run test:packed`
/ `test:packed:web` green (410 cases / 5,717,153 tokens, unchanged); `cargo
fmt --all -- --check` clean.

## 2026-07-31 — Phase E, step 2: `prepare_format_patch`/`apply_format_patch`

Added `crates/braid/src/format_patch.rs`: `PreparedFormatBook` (one targeted
book's post-format working-token stream plus its prepare-time hash and,
for a chapter-scoped preparation, the chapter label it was scoped to),
`PreparedFormatPatch` (one or more targeted books, applied atomically),
`FormatPatchId { snapshot, ordinal }`, `PatchPreparation { Unchanged |
Ready(FormatPatchId) }`, and `FormatPatchError { StaleSnapshot, UnknownPatch,
InvalidResult }`. `FormatError { Scope(ScopeError) }` added to `error.rs`,
matching freeze §6.8/owner-adjudication exactly.

Deviation recorded and reasoned in the module doc comment: a prepared format
patch does **not** reuse `Patch`/`PatchRow`/`PatchId` (the existing lint-fix
patch table). That table's own wire framing requires every row of one patch to
name the same token position (one fix, one position) — provable from the N.2
patch-table ruling — while a book- or chapter-wide format pass can rewrite
tokens throughout the run. Reusing the fix table's shape would mean either
violating that constraint or redesigning it, both out of scope for this phase.
Instead a prepared format patch stores each targeted book's complete
post-format working-token stream computed once against a frozen snapshot; the
existing PatchId/ordinal-into-book-patches space and this one are deliberately
separate and never address each other. `apply_format_patch` is a new method,
not a repurposing of the existing single-book `apply_patch` — Phase C's
`apply_patch(id: PatchId)` signature is unmodified.

`prepare_format_patch(scope, options)` resolves `All | Book | Chapter` to
target books (chapter scope narrows to that run's own token slice, spliced
back into the book — the same slice `update_chapter` cuts), runs core's
`format` unmodified, and includes a book in the preparation only if `format`
actually produced different tokens; an empty result set returns `Unchanged`.
Nothing is mutated and nothing is minted at prepare time (working-token
equality is the change test, so no serialization or minting is needed to
decide it). `apply_format_patch(id)` mints every still-id-less token via the
handle's own minter (the same admission checkpoint `apply_patch` uses, but
generalized: there is no single fix target to fall back on as a synthesis
anchor, so an unmatched token converts with no anchor and is refused rather
than guessed if it needed one), builds every targeted book's candidate before
touching resident state, and commits all of them or none. The reported
`changed` scope is chapter-scoped when the preparation was chapter-scoped,
whole-book otherwise (a whole-book/`All` format pass can touch tokens anywhere
in the book, so it cannot honestly narrow further). Every prepared-but-unused
format patch is dropped as soon as any mutation actually moves the snapshot
id, since every entry is bound to the snapshot it was prepared against.

New tests: `crates/braid/tests/format_patch.rs`, 8 tests — already-formatted
scope prepares `Unchanged`; preparing mutates nothing (snapshot id, bytes,
dirty stamps all unchanged before vs. after); book-scoped apply rewrites
exactly what `format` would produce; chapter-scoped apply touches only that
run; `CorpusScope::All` prepares and applies multiple changed books atomically
while leaving an already-formatted book alone; a stale preparation (corpus
mutated after prepare) is rejected atomically; an unknown ordinal is rejected;
the external stateless `format`/`FormatToken` proxy never touches resident
state.

Gate: `cargo test --workspace` green; `cargo test --release --test lint_oracle
-- --ignored` byte-identical; `cargo test --release -p braid -p
usfm_onion_wire -p usfm_onion_wasm -- --ignored` green; `npm run test:packed`
/ `test:packed:web` green (410 cases / 5,717,153 tokens, unchanged); `cargo
fmt --all -- --check` clean.

Phase E status: baseline mutation, exact `is_dirty`, `diff_baseline`, and
`prepare_format_patch`/`apply_format_patch` land in this phase. Not built: a
wasm/serde-facing surface for any of the above (Phase F). `to_usfm` needed no
change — it was already scoped `All | Book | Chapter` from Phase C.

## 2026-07-31 — Phase E, first-review fix: apply-time mint sweep now covered, one production bug found and fixed

First-review finding on `5f042fc`: every `format_patch.rs` fixture was chosen
to collapse whitespace on existing tokens, so `apply_format_patch`'s mint
sweep and `admit_format`'s id-less conversion path had zero coverage — the
exact escape route that produced findings in four prior review rounds.

Added two tests exercising a fixture that genuinely inserts tokens (a chapter
intro missing its `\p`, triggering core's on-by-default
`insert_default_paragraph_after_chapter_intro`/`insert_structural_linebreaks`):

- `format_insertions_are_minted_unique_and_survivors_keep_their_ids` — prepares
  and applies, then asserts every pre-existing token kept its own id, every
  inserted token carries a minted id (recognizable by this file's `"minted-"`
  prefix), and every id in the resulting book is unique.
- `a_hostile_minter_returning_a_colliding_id_is_rejected_atomically` — a
  minter that always returns an id already present in the book; apply must
  reject with resident state (snapshot id, bytes, dirty stamps) byte-identical
  before/after.

The first test caught a real production bug on first run, not a test-authoring
mistake: `admit_format`'s anchor lookup only tried matching a synthesized
token's `id` (always absent for a structurally-inserted token) against
resident tokens, so it always resolved to `None`. But core's format pass
stamps every synthesized token — including plain newlines — with the sid of
the surrounding verse content, and `OwnedToken::from_format_token` refuses
*any* token carrying a sid it cannot resolve against an anchor, not just
tokens needing a book-code/number payload as the original doc comment assumed.
Every insertion was therefore refused with `UnresolvableSid`, unconditionally.

Fix (`crates/braid/src/lib.rs`, `admit_format`): added a secondary index
mapping each resident token's sid to one token carrying it, and fall back to
it when no id match exists. Any resident token sharing the exact sid text
carries the identical structured form, so this never guesses a wrong answer —
it only supplies a fact `from_format_token` was already going to accept from
a matching anchor, sourced from a token proven to share that anchor rather
than the one the synthesized token happened to sit next to.

With the fix, the insertion test passed outright. The hostile-minter test then
needed its chosen colliding id changed from an arbitrary existing token (whose
mismatched sid made `UnresolvableSid` fire before the duplicate-id check ever
ran) to the resident `\v` marker's id, which shares the synthesized tokens'
sid — restoring the intended failure mode
(`FormatPatchError::InvalidResult(IngestError::DuplicateTokenId)`) while
keeping the atomicity assertions unchanged.

`crates/braid/tests/format_patch.rs`: 8 → 10 tests, all green.

Gates: `cargo test --workspace` green (braid 60); `cargo test --release
--test lint_oracle -- --ignored` byte-identical; `cargo test --release -p
braid -p usfm_onion_wire -p usfm_onion_wasm -- --ignored` green; `cargo fmt
--all -- --check` clean (after `cargo fmt --all`, which reformatted the two
touched files).

## 2026-07-31 — Owner adjudication on two flagged Phase E decisions

1. **Format-patch table separation from the lint-fix `Patch`/`PatchRow` table:
   RATIFIED.** No code change; recorded so the freeze appendix can cite it —
   resident-only in v1 (never wire field 6), separate ordinal spaces that
   never address each other.
2. **`set_baseline` installing an absent book as a fresh current book:
   REVERSED.** The epic's own gate — "baseline mutation cannot change current
   state" — is unconditional, and the install branch broke exactly that: it
   created residency, moved the snapshot id, and reported a changed scope.
   Owner's reasoning: `set_baseline` must never be an ingest verb. A caller
   has to be able to reason that a baseline operation leaves current tokens
   and corpus identity untouched, full stop — including when the named book
   is a typo or one that has since been removed, not merely on the intended
   path. A cold-open "install and declare baseline atomically" convenience,
   if ever wanted, belongs on the restore/ingest surface, not folded into
   this one.

Implemented the reversal in `crates/braid/src/{baseline.rs,lib.rs}`:

- Removed the install branch entirely. `set_baseline` now looks up the named
  book's current index first and returns a typed rejection if absent,
  touching nothing.
- New dedicated error type `SetBaselineError { BookNotResident(BookId),
  Invalid(IngestError) }` in `baseline.rs`, chosen over extending
  `IngestError` with a not-resident variant: `IngestError` describes a
  rejected *candidate mutation* — every existing variant is a fact about the
  content just supplied. "This book has no current content" is not a fact
  about the candidate; it is a precondition of `set_baseline` itself, which
  never introduces a book and never touches current content. Folding it into
  `IngestError` would give every other ingest verb (`replace_corpus`,
  `update_book`, ...) a variant only `set_baseline` can produce. The
  dedicated enum still composes `IngestError` for the one failure mode that
  genuinely is a candidate-validation problem (`Invalid`), so a malformed
  baseline candidate is still reported precisely.
- `set_baseline`'s signature is now `Result<MutationEffect, SetBaselineError>`;
  doc comment rewritten to state the "never an ingest verb, unconditionally"
  contract directly rather than describing the removed fallback.

`crates/braid/tests/baseline.rs`: replaced
`a_book_with_no_current_content_installs_fresh_with_a_matching_baseline` with
`set_baseline_on_a_book_with_no_current_content_is_rejected_atomically` (both
lanes; proves the typed rejection and that snapshot id, `books()`, and
`is_dirty`'s own `BookNotFound` all read exactly as before the call) and added
`set_baseline_on_a_book_not_resident_leaves_other_baselines_untouched` (a
sibling book's already-declared baseline survives a rejected call naming a
different, absent book). 15 → 16 tests.

Gates: `cargo test --workspace` green (braid 62); `cargo test --release
--test lint_oracle -- --ignored` byte-identical; `cargo test --release -p
braid -p usfm_onion_wire -p usfm_onion_wasm -- --ignored` green; `cargo fmt
--all -- --check` clean.

## 2026-07-31 — Clean-room review P1 fix: prepared format patches invalidated by the wrong predicate

Clean-room review of Phase E passed everything (baseline, dirty, diff,
minting, §Q table separation, multi-book admission) except one root cause.

**Root cause.** `effect_with_reorder` cleared the prepared-format-patch table
only when the byte-derived `SnapshotId` moved. But formatting consumes the
token stream and book identity, not just source bytes, and two ordinary
mutations can leave the snapshot id unchanged while invalidating exactly what
a preparation recorded:

1. `update_book` with the same bytes but a different stable token id (an
   ordinary re-push) does not change any book's hash, so the snapshot id is
   unchanged, while the token stream the preparation was computed against is
   gone. Applying the stale handle would silently overwrite the caller's
   newer token id with the stale prepared stream.
2. `replace_corpus` swapping one book's declared `BookId` for another using
   byte-identical source (declared `BookId` is outside snapshot identity —
   only the ordered per-book source hashes are) leaves the snapshot id
   unchanged while the book the preparation named is no longer resident at
   all. Applying the stale handle reached an `.expect(...)` and panicked.

Same class of bug as the Phase D reuse-cache fix: a byte-derived hash alone
does not pin token identity or declared book, so an invalidation predicate
keyed only on bytes is provably incomplete for anything the byte hash cannot
see.

**Fix** (`crates/braid/src/lib.rs`):

- `effect_with_reorder` (the choke point every mutating verb that returns a
  `MutationEffect` runs through — `replace_corpus`, `update_book`,
  `update_chapter`, `remove_book`, `remove_chapter`, `clear`, `update_config`,
  `apply_patch`, `apply_format_patch`, and `set_baseline`/`clear_baseline`)
  now clears the prepared-format-patch table unconditionally, not only when
  the snapshot id moves. Chose "clear everywhere" over threading an exemption
  for baseline/config verbs through every call site: those two calls cannot
  change format input, so clearing them too costs nothing but a wasted
  future `prepare_format_patch` call, and a single unconditional choke point
  is far harder to get wrong than a per-verb allowlist.
- `restore_corpus` bypasses that choke point (it returns a `RestoreReport`,
  not a `MutationEffect`) and reseeds tokens/residency wholesale, so it gained
  its own explicit `self.format_patches.clear()`.
- Replaced the `.expect("a snapshot-matching preparation names a resident
  book")` in `apply_format_patch` with a typed `FormatPatchError::
  BookNotResident(BookId)`. With the clearing fix this path should be
  unreachable through any sequence of public calls — but "should never" is
  not a proof, the previous unreachability claim was exactly the kind of gap
  this same bug exploited, and a missing-residency condition must be a typed
  rejection, never a panic, regardless of how confident the surrounding
  invariant is. Not folded into `StaleSnapshot`: that variant's fields would
  read `expected == found` here, describing a staleness that, by
  construction, is not what happened.

**Tests** (`crates/braid/tests/format_patch.rs`, both repros pinned exactly as
built):

- `stale_by_identity_update_book_is_rejected_and_the_newer_id_survives` —
  caller-tokens lane only (the repro is inherently about pushing a token
  identity change independent of re-parsing, which raw USFM ingest cannot
  express): prepares, then re-pushes the same book with one token's id
  changed and its content otherwise identical, confirms the snapshot id is
  in fact unchanged (the repro's own premise), then confirms `apply_format_patch`
  refuses (`UnknownPatch`, since clearing already emptied the table before
  the snapshot check would even matter) and that the *newer* token id is
  what's resident afterward.
- `stale_by_book_swap_replace_corpus_is_rejected_without_panicking` — both
  lanes: prepares GEN, swaps the whole corpus for a single EXO book with
  GEN's exact bytes, confirms the snapshot id is unchanged, then confirms
  `apply_format_patch` refuses typed with no panic.

`crates/braid/tests/format_patch.rs`: 10 → 12 tests.

No exemption was implemented for baseline verbs, so no separate
"set_baseline does not invalidate a preparation" test applies; the existing
`preparing_a_format_patch_does_not_mutate_anything` and
`a_stale_preparation_is_rejected_atomically` tests are unaffected and remain
green (the latter still hits `StaleSnapshot` first, since that mutation also
changes the snapshot id).

Gates: `cargo test --workspace` green (braid 88 across 8 test binaries);
`cargo test --release --test lint_oracle -- --ignored` byte-identical;
`cargo test --release -p braid -p usfm_onion_wire -p usfm_onion_wasm --
--ignored` green; `cargo fmt --all -- --check` clean. No wasm-facing code
changed, so the packed npm gates were not re-run this round.

## 2026-07-31 — Phase E re-review P2 closeout: test names now match what they prove

Re-review verdict: P1 fix clean (sid fallback held under adversarial nested-marker
attack, baseline carry-forward and duplicate-label handling sound); one P2
test-only gap before Phase E formally closes. `crates/braid/tests/baseline.rs`
had two tests whose names promised more than their bodies proved: split
`duplicate_chapter_labels_make_the_scope_ambiguous_on_either_side` into
`duplicate_current_chapter_labels_make_is_dirty_ambiguous` (as it already was)
plus new `duplicate_baseline_chapter_labels_make_the_scope_ambiguous_too`
(unique current label, duplicate baseline label — both `is_dirty` and
`diff_baseline` typed-ambiguous); renamed
`a_content_mutation_carries_the_baseline_forward_unchanged` to
`a_changed_update_book_carries_the_baseline_forward_unchanged` (what it
actually exercises) and added `update_chapter_carries_the_baseline_forward_unchanged`
(the `rebuilt()` seam) and `a_byte_identical_resubmission_carries_the_baseline_forward_via_inherit_cache`
(the `inherit_cache()` seam, asserting the `Unchanged`/no-op effect). No
production changes. `crates/braid/tests/baseline.rs`: 16 → 19 tests. Gates:
`cargo test --workspace` green (braid 89 across 8 binaries); `cargo fmt --all
-- --check` clean.

## 2026-07-31 — Phase E CLOSED (clean-room verdict, test-only closeout verified)

Clean-room re-review verdict at `a6b97f8`: "The P1 fix is correct, and I endorse unconditional
invalidation; the exemption complexity is not worth preserving transient handles. No new
production defect found" — with one P2 test-honesty closeout, landed as `25ada40` and ruled
closable without another broad review. Reviewer additionally verified: sid fallback sound under
adversarial nested-marker attack (nested names retain `+`, recovery cannot synthesize `+` names,
so an unrelated same-sid anchor cannot lend nesting); baseline carry-forward sound across
update_book / inherit_cache / rebuilt seams; duplicate-label resolution sound on both sides.

Director-verified on `25ada40`: workspace 607 passed / 0 failed, baseline suite 19/19, fmt
clean; commit is test-only (`baseline.rs` + ledger).

Phase E final deliverables: `set_baseline`/`clear_baseline` (SetBaselineError; never an ingest
verb, §Q companion ruling), exact `is_dirty` (hash fast-path, byte truth), `diff_baseline`
(core diff_skeleton, typed MissingBaseline), `prepare_format_patch`/`apply_format_patch`
(separate FormatPatchId space per §Q; unconditional invalidation on every mutation per the §Q
correction; typed BookNotResident), `books_awaiting_lint()` rename reserving "dirty" for the
baseline axis, and the `admit_format` sid-keyed anchor fallback fixing unconditional
UnresolvableSid refusal of format-inserted tokens.

Remaining: Phase F only (public wasm Braid, restoreCorpus composition, reconcile helper, editor
parity transcripts, pkg + canonical-docs regeneration).

## 2026-07-31 — Phase F step 1a: the wasm crate split by responsibility

Groundwork for the resident surface: `crates/usfm_onion_wasm/src/lib.rs` was a single 2,359-line file
holding the boundary value types, their conversions, every stateless export, and the tests. It is now
four modules — `dto` (the boundary shapes plus the conversions in both directions), `stateless` (the
one-shot exports over caller-owned input), `publication` (the Phase D composing adapter), and
`resident` (empty until the next step, where the `Braid` class lands) — with the root holding only the
crate-wide pieces: the hand-written TypeScript section, the wire DTO re-exports, and `pub use`s that
keep every public item reachable at the crate root whichever module now declares it.

Deliberately a pure move. The only substantive edits are visibility widenings forced by the new module
boundary (conversion helpers and a few boundary-struct fields become `pub(crate)`; none becomes `pub`,
so the Rust API surface is unchanged) and the import pruning that follows from splitting one `use` block
four ways.

**The proof that nothing moved:** the generated `pkg-bundler/usfm_onion_web.d.ts` is byte-identical
before and after — which is the only statement that matters for a crate whose product is its generated
declarations — and the crate's 33 lib tests are the same 33.

Gates: `cargo test --workspace` green, `cargo fmt --all -- --check` clean, zero compiler warnings, a
dev bundler build diffed against the committed declarations.

## 2026-07-31 — RFC 1: token-derived VREF indexes preserve the document's own verse order

A correctness bug, fixed in core and stated at the boundary. `VrefIndex` was
`BTreeMap<String, VerseProjection>`, so every consumer — `vrefIndexUsfm`,
`vrefIndexTokens`, and any native caller — received verses in **lexicographic SID
order**: `GEN 10:1` before `GEN 2:1`, `GEN 29:19` before `GEN 29:2`. A projection is
read against a document, and a document's verses are in whatever order it puts
them, including deliberately out-of-order editor content; the sorted keys made that
order unrecoverable. Segment data was always correct (it is reached by SID).

**Boundary shape amended by owner directive (same day):** the wasm projection is a
single ordered entries list, `[sid, projection][]`, not `{ order, bySid }`. The shape
is already breaking, so preserving map-lookup ergonomics bought nothing; one
authoritative sequence beats two views one of which has to be documented as
meaningless; and a consumer wanting O(1) lookup writes `new Map(entries)` and owns
that container choice. The generated declaration reads
`export type VrefIndex = [string, VerseProjection][];` — a transparent newtype over
the pairs, which tsify renders as a clean tuple array with no named entry type
needed. The regression asserts the tuple sequence post-serialization (order, that
sorting would differ, and that each pair carries its whole projection).

**Shape chosen.** Core's `VrefIndex` is now an order-preserving container: a
`Vec<VrefEntry { sid, projection }>` in first-seen token order, plus a private
`sid → position` index for lookup. Accessors are `entries()`, `sids()`, `iter()`,
`get()`, `contains_key()`, `len()`, `is_empty()`; it serializes **as its entries, in
order** (a JSON array), because an ordered container that serialized to an object
would re-introduce the sort at the first `serde_json` call. At the wasm boundary the
same fact is projected as `{ order: string[], bySid: Record<string, VerseProjection> }`
— the ordered key sequence is authoritative and the map stays for O(1) lookup,
which costs no duplicated projection data and keeps existing JS lookups working. The
DTO's doc says outright that `bySid`'s key enumeration is not meaningful.

**Duplicate-SID semantics are unchanged and now pinned.** One entry per SID with the
last projection written — what the map did — and the position that entry now also
has to answer for is its *first-seen* one. Both halves are asserted, plus continued
key-set parity with `to_vref`.

**Regressions.** A fixture out of order in both dimensions a sort would "fix"
(`\v 19` before `\v 2`; chapter 10 before chapter 2, then chapter 2 last) asserts
emitted order equals stream order at the Rust surface *and* through the wasm
boundary **after serialization**, which is the only place a sorted container betrays
itself. The boundary test additionally asserts that `bySid`'s enumeration differs
from `order`, so the reason for carrying both stays visible instead of assumed. No
existing test asserted BTreeMap iteration order as such; three used map-only methods
(`keys`, `remove`, `&index` iteration) and now use the ordered accessors.

### Census — other consumer-visible surfaces whose container sorts (owner rules; not changed here)

| surface | container | is stream order recoverable? |
| --- | --- | --- |
| `usfm_onion::vref::VrefMap` (`to_vref`, `usfm_to_vref_map`, `tokens_to_vref_map`, `vref_map_to_json_string`) and the wasm `VrefMap` DTO / `vref_to_object` | `BTreeMap<String, String>` keyed by SID | **No.** Same violation class as the one fixed here, one surface up: lossy text instead of lossless projections. Nothing carries the order. |
| `usfm_onion::api` diff-by-chapter (`src/api.rs:390` over `Token`, `:412` over `FormatToken`) and the wasm `DiffsByChapterMap` | `BTreeMap<String, BTreeMap<u32, DiffSkeleton>>` | **No**, and worse than order: the outer map sorts books lexicographically, and the inner `u32` chapter key both sorts numerically *and collapses duplicate chapter numbers* — which braid elsewhere deliberately retains as distinct runs. Order and multiplicity are both lost. |
| `LintSummary.by_category` / `by_severity` / `by_issue_type` | `BTreeMap<enum, usize>` | **Not a violation.** Keys are library enums, ordered by their own declaration order; there is no user-supplied order to lose. |
| `LintIssue.message_params` (`MessageParams`) | `BTreeMap<String, String>` | **Not a violation.** Keys are a fixed per-code contract, and the wire encoder depends on that key order for canonical bytes. |

`braid`'s public surface has no sorted-container boundary: its ordered facts
(`books()`, `chapter_labels()`, `entries`, snapshots) are all `Vec` in caller order.

The generated TypeScript for `VrefIndex` changes shape with this commit; the
regenerated `pkg-bundler`/`pkg-web` trees carrying it land with this packet's package
commit.

## 2026-07-31 — owner ratifies the from_parts proxy ruling (Phase F checkpoint)

The Phase F checkpoint's proxy ruling is owner-confirmed: wire Token DTO -> OwnedToken assembly
lands as core `OwnedToken::from_parts` (exhaustively destructured parts struct, typed
refuse-never-guess errors — core owns semantic legality because it owns the private payload
enum) plus wire `TryFrom<&Token>` mapping shape-level problems to the frozen `TokenInputError`
(§5.1). Rejected alternatives, for the record: fabricating anchors for `from_format_token`, or
relaxing the C4-hardened no-anchor refusals. Same session, owner also redirected the RFC 1
boundary projection from `{order, bySid}` to a single ordered `[sid, projection][]` entries
list — one authority, consumers build their own lookup.

## 2026-07-31 — Phase F step 1b: the inbound token boundary (DTO → resident token)

Every token-accepting resident verb needs boundary tokens to become resident ones, and
nothing could do it: `from_format_token` serves a *descendant* of a resident token (the
anchor supplies what the working shape drops), while a caller handing over a whole book
has no anchor — it has the facts. Built as approved: core owns payload legality, wire
owns DTO shape.

**Core.** `OwnedToken::from_parts(OwnedTokenParts) -> Result<Self, TokenBuildError>`, with
the parts destructured exhaustively and no `..`, so a new token fact is a compile error at
the construction site — the same guarantee `hash_wire_identity` gives. It refuses rather
than guesses: a payload on a kind that cannot hold it (`UnexpectedPayload`, naming the
fact), a marker-bearing kind with no marker or a payload-bearing kind with no payload
(`MissingPayload`), an empty id (`MissingId`), an anchor this library would never have
written (`UnresolvableSid`). An end marker cannot carry an attribute list and a milestone
cannot carry nesting; both are refusals, because a caller that sent one believes it means
something.

`Sid::parse` is the inverse the boundary needs — strict, so anchor text this library never
emits cannot become an anchor pointing somewhere nobody named. Writing it surfaced a
**pre-existing inconsistency worth recording: the library spells a chapter-level anchor
two ways.** Core's `Display` writes `"GEN 1"` (verse zero, no locator); wire's `format_sid`
— the DTO/JS spelling — writes `"GEN 1:0"`. Both parse to the same value and `from_parts`
stores the canonical re-print, so the two views converge on core's spelling instead of
multiplying; unifying the emitters is a separate change with golden and JS-surface reach,
not folded in here.

**Wire.** `owned_token_from_dto(&Token, token_idx)` plus the frozen `TokenInputError`
(`MissingId`, `IncompleteBookCode`, `Illegal { reason }`). Half a book code is a shape
complaint answered here; contradictory facts are core's verdict carried verbatim rather
than re-worded. `token_idx` is in the error because "one token out of a book's worth is
wrong" is only actionable with a position.

**The corpus gate earned its keep — it caught three bugs the hand-written cases did not**,
over 5,716,969 tokens (1,347,793 payload-bearing), comparing whole-token identity via the
wire-identity projection rather than a hand-listed field set:

1. **Attribute placement had no DTO field.** The verbatim `|...` slice crossed the boundary
   but not *where it sat*, so a round trip fell back to the closer rule and re-emitted
   alignment lists somewhere else — byte-losslessness kept only for layouts that happen to
   match the fallback. `Token.attributeOffset?: number` added, populated on the way out,
   read on the way in, and computed in the JS materializer from the packed list span so the
   cross-language equivalence gate still sees identical objects.
2. **Attribute values are escape-decoded outbound.** The DTO gives JS the logical string
   (`\"` → `"`); core holds the source spelling, which is what the emitter writes back.
   Re-encoding the decoded string is exact only for the escapes this library recognizes, so
   the verbatim slice is now the authority and re-encoding is the fallback for an attribute
   the caller authored.
3. **Recovering the value needed the key, not a delimiter search.** A default attribute's
   slice *is* its value and may contain quotes of its own (`|"caption"`), or — when the
   source is malformed — several `key="value"` pairs the parser could only read as one
   default attribute (`|lemma= strong="l"`). Hunting for `="` cut those in the wrong place;
   the attribute's own key says which shape it is.

Each has a named regression beside the corpus gate, and the standing dimensions are covered:
both id lanes (parse-assigned positional and a caller's own opaque ids), hostile DTOs (payload
on the wrong kind, half a book code, unparseable anchor, attribute list on an end marker,
marker kind with no marker), and all three attribute-presence facts (absent, empty verbatim,
structured without verbatim).

Gates: `cargo test --workspace` 619 passed / 0 failed; oracle `--ignored` byte-identical;
release ignored braid 3 / wire 5+2 / wasm 1; `test:packed` and `test:packed:web` 410 cases /
5,717,153 tokens with the new field carried by both materializers; `cargo fmt --all --check`
clean.

## 2026-07-31 — RFC 2: one spelling for a canonical anchor (owner-ruled `C:0`)

The divergence the `from_parts` work surfaced, closed. Census first, because it changed
which direction was cheap: **eight** places turned a `Sid` into text, and seven of them
already printed `chapter:locator` unconditionally. Core's `Display` — with its verse-zero
special case printing `"GEN 1"` — was the lone outlier, so the owner's canonical choice
(`"GEN 1:0"`, verse zero explicit) meant changing the authority and letting six
hand-rolled copies delegate to it rather than editing seven call sites to match a bare form.

| site | before | after |
| --- | --- | --- |
| `usfm_onion::token::Display for Sid` | `"GEN 1"` when verse is zero | **the one authority**, always `chapter:locator` |
| `lint_impl::format_sid` | already delegated | unchanged |
| `token::DiffableToken::sid_string` (OwnedToken) | hand-rolled | delegates |
| `api.rs`, `usj/export.rs`, `diff/mod.rs` (×2) | hand-rolled | delegate |
| `usfm_onion_wire::dto::format_sid` | hand-rolled | delegates; name kept, since the wasm crate re-exports it |
| `js/packed.js` | hand-rolled | unchanged — it already printed `C:0`, and JS cannot call Rust, so this is a second implementation by necessity, pinned by the cross-language equivalence gate |

`vref.rs`'s `verse_ref` is deliberately **not** in that table: it builds a key from the
verbatim verse *lexeme* so sequences (`1,3`) survive, and it can never produce verse zero.
Documented as such rather than "fixed".

`Sid::parse` still accepts the bare historical form and re-prints it canonically, so
content recorded under the old spelling reads back while nothing emits it again. That
acceptance is ingest-only and asserted in both directions.

**What moved, consumer-visibly:** `OwnedToken.sid` (which also closes the resident-versus-DTO
mismatch: the same token used to say `"GEN 0"` resident and `"GEN 0:0"` at the boundary),
`LintIssue.sid`, and USJ/USX/diff sid strings — chapter-level anchors only. **Unaffected:**
VREF keys (lexeme-built, verse-only), `to_usfm` bytes (a sid is not source text), packed
container bytes (structured SIDs; the string is derived on decode), and every
token/cst/usj/usx/vref/html hash in the oracle.

**Two blessings, both approved and characterized before committing:**

1. `tests/lint_oracle_baseline.txt` — **124 changed lines out of 4,398, line count identical**,
   every delta a `sid` field gaining `:0`, verified programmatically; zero findings added,
   removed, or reordered; no hash line touched. Checked LF-clean before commit (`grep -c $'\r'`
   = 0), because this baseline was silently CRLF-tainted by a past bless.
2. `crates/usfm_onion_wasm/golden/outputs/lint-violations/lint.json` — two sids, `"GEN 1"` →
   `"GEN 1:0"`.

### Recorded deviation — `ApiResult`'s tag

The epic's §8.2 sketch writes `{ok:true,value:T} | {ok:false,error:E}`. The resident class
emits `{status:"ok",value:T} | {status:"error",error:E}` instead: a boolean tag cannot narrow
a union in TypeScript, so `ok:boolean` would force every consumer to assert the other field,
and the crate already has a string-tagged outcome type (`PackedBookOutcome`'s `status`).
Matching the existing convention beats inventing a third.

### Also recorded — a gate that existed and was not run

`attributeOffset` landed with the inbound boundary and I ran `test:packed` but not
`golden:wasm`, leaving four snapshots stale; caught and re-blessed in its own commit
(`49eb6f9`). A gate that exists but was not run counts as failed.

## 2026-07-31 — RFC 2 (revised): verse projections preserve source bytes, never insert

Owner-ruled after the first design was superseded: verse text in **both** vref surfaces is
the verbatim source bytes of the content tokens inside the verse — text and newline tokens,
in order — with marker tokens contributing nothing and nothing ever inserted or mutated.

Two different fabrications were producing `gladnessso`:

- the **index** had no `on_newline` handler at all, so a newline token inside a verse was
  simply dropped and the words either side collided;
- the **map** dropped it too, then set a flag that later re-derived "probably a space",
  conditional on neighbouring whitespace it had also just discarded. Subtler, equally
  invented.

Both now append the newline token's own bytes. `push_collected_text` is a plain append; the
`pending_separator` state is gone.

### The lexer decides tagend whitespace, not us — probed, not chosen

| source | tokens the lexer emits | projection |
| --- | --- | --- |
| `gladness\n\q2 so that` | `Text "gladness"`, `Newline "\n"`, `Marker "\q2 "`, `Text "so that"` | `"gladness\nso that\n"` |
| `gladness \q2 so that` | `Text "gladness "`, `Marker "\q2 "`, `Text "so that"` | `"gladness so that\n"` |
| `gladness\n\n\q2 so that` | two standalone `Newline` | `"gladness\n\nso that\n"` |
| `gladness\n\q2\nso that` | `Marker "\q2"`, standalone `Newline` | `"gladness\n\nso that\n"` |
| `the \nd Lord\nd* said` | `Marker "\nd "` holds its own space | `"the Lord said\n"` |

A marker token's span **swallows its tagend space**, so that byte is syntax and excluded. A
tagend *newline* is a **standalone token**, so it is content and kept — which is why case 4
legitimately yields two newlines: both bytes are in the source. Map and index are now
byte-identical on every case.

**Empirical confirmation of the ruling's premise.** The owner's argument was that the grammar
guarantees a real separator byte at every seam, so preserving can never lose one. Across the
214 oracle fixtures whose vref output moved, the serialized JSON length **grew in 214 and
shrank in 0** — if any seam had lacked a real byte, dropping the fabricated space would have
shortened that fixture.

### Blessings, characterized before committing

1. `tests/lint_oracle_baseline.txt` — **214 changed lines of 4,398, line count identical, and
   every changed line is a `vref` hash line**: no finding line, summary, or
   token/cst/usj/usx/html hash moved. LF-clean verified.
2. Seven `crates/usfm_onion_wasm/golden/outputs/*/vref.json` snapshots — the only deltas are
   verse text values gaining their real separator bytes.

Verse *keys* cannot move: they come from the lexeme-based `verse_ref`, untouched, and the
"whitespace-only verse produces no entry" rule still drops a verse whose only content is a
newline. Key-set parity between map and index is asserted as before.

### Test expectations: 11 reviewed individually, one renamed

Every update is the same shape — a verse keeps the newline that ends its line — with one
instructive exception recorded in the test itself: a verse that ends the *file* has no
trailing byte to keep, and nothing is invented to make the two cases look alike.
`structural_break_inserts_separator_without_leaking_delimiters` is renamed
`a_structural_break_keeps_its_own_separator_byte_and_leaks_no_delimiter`, since "inserts
separator" is precisely the behaviour that was deleted. `vref_index_marker_joins_are_content_pure`
now asserts `"darkness \nhave"` — both the trailing space and the newline — and keeps its
negative assertion that no byte absent from the source ever appears.

### Census — every place that flattens content into a string

| site | verdict |
| --- | --- |
| `vref::VrefVisitor` (`to_vref` map) | fixed |
| `vref::IndexedVrefVisitor` (vref index) | fixed |
| `usj/import.rs` (~10 `push(' ')` sites) | **not a projection** — it *generates* USFM from USJ, where a space between a marker and its content is required syntax being written, not content being invented |
| `html.rs` `push_attr`, `token.rs` `format_attribute_list` | **not projections** — they emit attribute syntax (` key="value"`), where the space is the format |
| `cst`, `export_tree`, `walker`, `lint_impl::marker_balance` | structural walkers; they carry token references, never concatenate content text |

So the two vref surfaces were the only content-flattening sites, and both are now faithful.

### Recorded gaps (not fixed, unasked)

- **Trim asymmetry.** `to_vref` has `VrefOptions.trim`, the index has no equivalent, so a
  consumer wanting trimmed *index* text must trim it. `trim` itself survives the new
  semantics coherently — it normalizes only the projection's edges, never interior bytes, so
  it composes with a faithful projection rather than contradicting it. Preserve semantics
  just makes the asymmetry more visible, since every verse now ends with its own newline.
- **The `cfg(not(target_arch = "wasm32"))` branch is not a parity fork.** `saw_block` /
  `seed_dependent` are bookkeeping for the native chapter-parallel vref path, which exists
  because a whole-book walk carries two pieces of state across a `\c`. wasm never takes that
  path, and the native path is pinned byte-identical to the serial walk by
  `partitioned_matches_serial_over_{test_data,example_corpora}` (both re-run green here).
  Pre-existing, and unrelated to the separator: it tracks the block-supports-verse flag,
  which still persists across chapters.

## 2026-07-31 — RFC 3: resident, per-chapter-incremental vref index

Owner-directed, editor-critical: the editor calls stateless whole-book
`vrefIndexTokens` on every keystroke at ~520-630ms/edit; this is the resident
fix.

**Verb.** `Braid::vref_index(&mut self, scope: CorpusScope) ->
Result<ScopedOutput<Vec<VrefEntry>>, ScopeError>`, following `lint()`'s
explicit-recompute shape (`&mut self`, nothing computed as a mutation side
effect) and the existing `to_usfm`/`to_usx`/`to_usj` `ScopedOutput`
convention rather than a new envelope. Supports `All | Book | Chapter` for
consistency with every other resident projection, though the editor's own
hot path is `Book`/`All`. Content is byte-for-byte identical to
`usfm_onion::vref::tokens_to_vref_index` over the same current tokens (see
gate below) — braid adds caching, not a second projection. `VrefEntry`,
`VerseProjection`, `Segment`, `Utf16Span` re-exported from `usfm_onion::vref`
for callers, the same courtesy every other resident projection extends.

**Cache key and invalidation predicate.** Per chapter run, keyed on that
run's own `TokenIdentity` (`crates/braid/src/vref.rs`, new module) — the same
drift-proof `hash_wire_identity`-backed projection this crate's baseline/
patch machinery already uses. Justified against "what does the projection
actually read": per-token id (segment anchors), kind, source text (verse
content and a verse-number token's own lexeme — this is how a bridge like
`"1,3"` survives into the sid), sid (the entry key), and marker name
(paragraph-support gating) — every one of those is already covered by
`hash_wire_identity`'s exhaustive, no-`..` destructuring. Deliberately not
"invalidate the cache when mutation X happens" — this epic has twice found
that kind of predicate incomplete for something a projection actually reads
(the format-patch snapshot-only bug, twice). Instead every read re-derives
each visited run's *current* identity and only ever reuses an entry whose
stored identity matches that fresh one; a stale entry left over from a run
that no longer exists in this shape is simply never matched again. This
makes the specific sharp edges named in the RFC fall out for free rather
than needing separate cases: an `\id`/book-code change reaching every sid in
the book is just one more way a run's own tokens' hash can move (a
front-matter edit only ever invalidates the front-matter run's own — usually
empty — cache entry; every *other* run's tokens, and thus their embedded
sid strings, are untouched, so there is no cross-run cascade to invalidate
in braid's token-resident model in the first place); a `\v`-lexeme change is
covered because `source` is hashed; duplicate/reopened `\c` boundary shifts
are covered because chapter runs are always recomputed fresh from the
current stream and a run's hash is taken over whatever slice its *current*
range names, never a remembered offset. The cache lives on `BookState`
(`vref_cache: Vec<CachedRun>`) and is carried forward — unconditionally,
including through `update_book`'s/`replace_corpus`'s whole-book-replace
branches, not just `update_chapter`'s splice path — because carrying it
forward is always safe (every entry is re-verified before use) and only
ever helps; there is no separate "clear on mutation X" bookkeeping to keep
in sync with every mutation path, unlike the format-patch table's fix.

One correctness subtlety found and fixed during testing: a book with
duplicate/reopened `\c` runs sharing a chapter number can have the *same*
sid appear in two different runs. The single whole-stream stateless
projection folds that through one shared `VrefIndex` (first-seen position,
last-write-wins content — `VrefIndex::insert`'s own documented rule); this
crate's per-run projection walks each run through its own separate index (so
a cache hit for one run can never see another's entries), so the same fold
has to be redone once over the concatenated result — `vref::merge_by_sid`,
added after `mutation_battery_stays_equivalent_to_the_stateless_projection`
caught the drift (two entries under one sid instead of the correctly-merged
one).

**Reuse proof.** `crates/braid/src/vref.rs`'s own `#[cfg(test)]` module adds
a thread-local recompute counter (`recompute_count`, incremented only inside
`CachedRun::fresh`, i.e., only on a genuine non-cache-hit projection run) —
a cache-generation observable, never a timing assertion, per the standing
rule. `a_whole_book_read_recomputes_only_the_dirty_chapter` proves: first
read has nothing to reuse (every run computes); an unchanged re-read is
entirely cache hits (recompute count 0); after editing exactly one chapter,
a whole-book read recomputes exactly 1 run, and the untouched chapters'
entries are unchanged content, not merely unasserted.

**Equivalence gate.** `crates/braid/tests/vref.rs`:
- Fast (non-`#[ignore]`) tests, both lanes: a scope-exercising fixture (note
  stripping, chapter crossing, verse bridge) equals the stateless
  projection; `All` groups by book in corpus order; `Chapter` equals the
  corresponding slice of a whole-book read; typed `ScopeError`s
  (book/chapter not found); a mutation battery (`update_chapter`,
  `update_book`, an `\id`-adjacent whole-book replace, a duplicate-`\c`
  fixture, `remove_chapter` + `replace_corpus`) staying equivalent to a
  fresh stateless projection after every step, both lanes.
- `#[ignore = "corpus-scale"]`:
  `resident_vref_index_equals_stateless_over_the_whole_corpus_through_a_mutation_battery`
  — all 66 `en_ulb` books, both lanes, plus the same mutation battery at
  corpus scale against GEN/PSA/REV/EXO.
- `#[ignore = "corpus-scale"]` `psalms_informal_timing` — release-build wall
  clock, not a pass/fail gate (informal numbers only, per the standing
  counters-not-timing rule for *correctness* proofs; this one is explicitly
  informal by design).

**Psalms numbers** (release build, warm cache, one `update_chapter` edit
then a whole-book resident read, vs. a from-scratch stateless
`tokens_to_vref_index` over Psalms' original 66-book-corpus source in the
same process): resident **3.38ms** (2455 entries) vs. in-process stateless
recompute **13.30ms** (2461 entries; the count differs because the
stateless comparison intentionally reads the *original* unedited source,
not the post-edit tokens — a like-for-like correctness comparison is what
the equivalence gate above proves, not this timing one). Both numbers are
dwarfed by the editor's quoted ~520-630ms/edit baseline, which additionally
pays wasm marshalling/boundary costs this in-process measurement does not;
the in-process 3.4ms-vs-13.3ms delta is itself already a ~4x win before any
boundary-cost difference is counted.

**Deviations / notes for the owner:**
- `#[cfg(test)] mod tests` inside `crates/braid/src/vref.rs` is new for this
  packet's proof specifically (existing crate convention already does this
  for internal-seam tests elsewhere in `lib.rs`); it is compiled only for
  the crate's own unit-test target and adds no production surface.
- Found (not introduced by this packet, not fixed — outside this lane):
  `cargo build -p braid --features serde` currently fails
  (`error: lifetime may not live long enough` deriving `Deserialize` for
  `IngestError::InvalidToken(TokenBuildError)`), because `TokenBuildError`
  gained an `UnexpectedPayload { fact: &'static str, .. }` variant upstream
  (`src/token.rs`, outside this lane) whose `&'static str` field cannot
  honestly derive `Deserialize` for an arbitrary input lifetime. Does not
  affect any gate run for this packet: `usfm_onion_wasm` depends on `braid`
  with no features, so `serde` is never enabled by `cargo test --workspace`,
  and every gate above is green. Flagging for whoever owns braid's `serde`
  feature next, since it is currently non-compiling.

**Signature for the wasm projection:**
```rust
pub fn vref_index(&mut self, scope: CorpusScope)
    -> Result<ScopedOutput<Vec<VrefEntry>>, ScopeError>;
```
`ScopedOutput<T> = Single(T) | All(Vec<SourceOutput<T>>)` (existing type);
`VrefEntry { sid: String, projection: VerseProjection }`; `VerseProjection {
text: String, segments: Vec<Segment> }`; `Segment { token_id: String,
source_span: Span, text_span: Utf16Span }` — all re-exported from
`usfm_onion::vref` unchanged, so the wasm layer's existing stateless
`vrefIndexTokens` DTO shape for these types should already have a mapping
to reuse verbatim.

Gates: `cargo test --workspace` green (braid 95: lib 9, baseline 19,
format_patch 12, lifecycle 28, lint_prime 7, patches 7, resident_lint 10,
vref 5; core 290, wire 196+2, wasm 34 — core/wire counts reflect the other
builder's concurrent RFC 2 landing, unaffected by this packet); `cargo test
--release --test lint_oracle -- --ignored` byte-identical; `cargo test
--release -p braid -p usfm_onion_wire -p usfm_onion_wasm -- --ignored`
green including the two new corpus-scale vref gates; `cargo fmt --all --
--check` clean. npm/packed gates intentionally not run (outside this
lane/packet; no wasm-facing code changed here).

## 2026-07-31 — Fix: `TokenBuildError` could not cross a serde boundary

Found by the other builder: `cargo build -p braid --features serde` did not compile. The
`from_parts` work gave `TokenBuildError::UnexpectedPayload` a `fact: &'static str`, and a
borrowed `'static` label cannot be deserialized — the deserializer's own lifetime would have
to outlive `'static` — so braid's feature-gated derives over the lifecycle types (a frozen
contract, since a native resident host serializes these straight to its IPC channel) failed
at the `IngestError::InvalidToken(TokenBuildError)` arm.

Fixed as `Cow<'static, str>`, chosen over the alternatives because it is the only one that
costs nothing and loses nothing: construction still borrows the static label (no allocation
on an error path), `Deserialize` yields an owned label, and the fact — which is the whole
informative part of the variant, naming *which* payload was illegal — survives. `String`
would allocate on every refusal; `serde(skip)` plus reconstruction would drop the one field a
caller acts on.

**Gate gap, recorded in one sentence as asked:** `lifecycle_errors_round_trip_through_serde`
already existed and would have caught this immediately, but no gate in my list ever enabled
the feature, so the entire `#[cfg(feature = "serde")]` surface — derives and test alike — was
never compiled in any run I made. `cargo test --workspace --all-features` is now part of the
standing battery (626 passed / 0 failed), which covers braid's `serde` and wire's `wasm`
features together; a feature that is never enabled is a gate that does not exist.

## 2026-07-31 — Phase F step 1c: the resident `Braid` class

The wasm handle now projects **18 verbs** over `crates/braid`: the lifecycle
(`replaceCorpus`, `updateBook`, `updateChapter`, `removeBook`, `removeChapter`, `clear`,
`updateConfig`), baselines (`setBaseline`, `clearBaseline`, `isDirty`, `diffBaseline`), lint
and patches (`lint`, `patches`, `patch`, `previewPatch`, `applyPatch`, `prepareFormatPatch`,
`applyFormatPatch`), reads (`books`, `chapterLabels`, `toTokens`, `toUsfm`,
`booksAwaitingLint`, `expectedSnapshotId`), and `vrefIndex`.

**Outcomes are typed, and getting there took a fix worth recording.** `wasm_bindgen` erases a
generic's parameters in a method signature, so a verb returning `ApiResult<MutationEffect,
IngestError>` was declared as a bare `ApiResult` — no information at all to a consumer. A
transparent newtype per shape costs nothing at runtime and restores the whole type: 14
`XOutcome` aliases, each rendering as `export type XOutcome = ApiResult<T, E>`, so
`applyPatch(id: PatchId): PatchMutationOutcome` resolves to
`ApiResult<MutationEffect, PatchError>` in an editor.

Deviations, both recorded earlier and reaffirmed here: `ApiResult` is tagged on a string
(`{status:"ok"|"error"}`) because a boolean tag cannot narrow a union in TypeScript, matching
the crate's existing `PackedBookOutcome`; and `ScopedOutput` projects `all` as ordered pairs
rather than an object keyed by source key, because corpus order is a contract and an object's
key enumeration is not.

**Inputs are discriminated unions** (`BookInput`, `ChapterInput`, `ChapterLabel`,
`CorpusScope`), never optional bags — a book arriving as bytes and a book arriving as tokens
carry different obligations, and a shape that could be neither is one a caller builds by
accident. Every refusal is a typed union a consumer switches on, carrying the same information
the Rust error does; a book code this library cannot even read reports the code the caller
sent rather than an empty string, since that is the only part they can act on.

`vrefIndex(scope)` projects the other builder's resident verb with RFC 1's tuple-entries
shape — `[sid, projection]` pairs in first-seen order, identical to what the stateless
`vrefIndexUsfm`/`vrefIndexTokens` return, so a consumer can move a keystroke path from one to
the other without changing what it reads. Pinned by a test asserting the resident entries
equal the stateless projection's, including the preserved seam byte so it cannot pass on two
empty lists.

New wire export: `impl From<&OwnedToken> for Token`, the outbound counterpart of
`owned_token_from_dto` — resident tokens leaving the boundary. Spanless, with the two
name-derived marker fields recomputed exactly as decoding does.

Gates: `cargo test --workspace --all-features` 627 passed / 0 failed; zero warnings; fmt
clean; generated declarations inspected for every verb.

## 2026-07-31 — Phase F step 3: the warm cold-open and finding reconciliation ship

`Braid.restoreCorpus(records)` is the composed verb, and this layer is the only one
allowed to compose it: the bytes are verified and decoded by the wire codec, the results
are handed to the resident corpus, and braid never sees a packed byte. Verification is the
full trust boundary — structure, both checksums, exact source length and content hash, the
catalog stamp, every discriminant and index — so a container that does not check out is
refused before anything installs. Records carry `source` as bytes rather than a string so a
host can hand over what it read from disk without a UTF-16 round trip; non-UTF-8 is a typed
refusal, not a panic.

A book whose cached findings cannot be adopted **still seeds**: residency and lint-priming
are independent, so it arrives with no lex or parse and is simply awaiting recompute. The
report says so, and a book can appear in both `seeded` and `rejected`.

New wire export `materialize_owned_tokens` supports it. Going through the boundary DTO would
convert twice and, worse, lose the opaque stable ids: only the section knows whether its ids
were explicit or positional, so the resident tokens are built from the decoded section
rather than re-derived from DTOs a caller could have edited.

`reconcileFindings(previous, next)` is pure JS, validating and decoding nothing — it operates
on findings that already came out of the trust boundary. Identity is the rule code plus the
anchored token ids, which is the only address stable across a recompute: a byte span moves
when anything earlier in the book is edited, a token id does not. Message text and fix payload
are deliberately *not* identity (a rule whose wording changed is the same finding) but a change
in either still yields a fresh object, because what a consumer reads did change. An unchanged
pass returns the `previous` array itself, so a caller can skip a re-render on reference
equality alone.

Gated inside the packed equivalence run over findings the boundary actually emitted rather
than hand-built ones (410 cases, now `+ 3 findings reconciled`): unchanged pass returns the
same array, one edited finding is fresh while every sibling keeps its identity, and a
re-anchored finding is a different finding.

## 2026-07-31 — Phase F close: worker/transfer gate, version 0.1.0, packages committed, docs

**Module worker and buffer transfer** (`scripts/test-web-package.mjs`, both targets). The two
things a main-thread import never exercises: initialising the published package inside a module
worker — where an editor actually runs it, off the interaction thread — and having a packed
buffer handed across that boundary. The bytes are a real checked-in golden container, so what
the worker verifies is a genuine publication rather than a synthetic buffer, and the assertions
cover the whole path: the sending side's buffer is detached (proving a real transfer, not a
copy), the worker verifies the container, and the findings *and their fixes* survive. The
package's contract that a caller copies a wasm-returned `Uint8Array` before transferring is
what the test performs — transferring the view's own buffer would detach wasm's heap.

**Version 0.1.0**, workspace crates and npm together. Minor rather than patch: the resident
surface is new API, and three shapes changed for consumers (`VrefIndex` is now ordered pairs,
`Token` gained `attributeOffset`, and a chapter-level sid now spells itself `GEN 1:0`). The
engine stamp derives from the crate version, so **every warm lint cache built under 0.0.9 is
refused after this bump** — by design, not by accident. Asserted rather than left implicit:
`a_version_bump_invalidates_every_cache_built_under_the_old_one` pins that the stamp moves with
the version and that it is the version doing it, so whoever bumps next is told by a test name
that old caches will be recomputed. (That test lives in `crates/braid/src/stamps.rs`, the one
file outside my lane I touched this packet, because the assertion has no other sensible home.)

**`pkg-bundler` and `pkg-web` are committed as reviewed artifacts**, ending the
knowingly-stale-pkg state Phase F was told to close. Both are release builds carrying the
VrefIndex tuple shape, `attributeOffset`, the sid spelling, the 18-verb `Braid` class with its
typed outcome aliases, `vrefIndex`, and `restoreCorpus`. Declarations: 1,182 lines (bundler) /
1,287 (web); wasm binary 1,593,244 bytes both targets.

**Canonical docs** gain a *Resident corpus* section, which the 2026-07-30 deferral note reserved
for this phase: the four-crate table with each crate's "never owns" column, why composition lives
in the wasm crate (braid must not learn byte layouts), the mutate-then-hydrate contract with the
three places tokens may enter, the three identity stamps and what each is for, explicit recompute,
patches as snapshot-bound token operations, and publication/warm-open. The WASM section now states
the stateless-versus-resident split and the tagged-result convention, including why declarations
name per-verb aliases (wasm-bindgen erases a generic's parameters in a method signature).

## 2026-07-31 — Final pre-review packet, item 1: native-vs-wasm parity transcript gate

Owner's core F-wasm goal: prove the native `braid::Braid` and the wasm `Braid`
class behave identically over the same lifecycle.

**Mechanics.** `crates/usfm_onion_wasm/src/parity.rs` (new, `#[cfg(test)]`
module declared in `lib.rs`) is a generator, not an assertion: it drives
*native* `braid::Braid` through a scripted lifecycle (`replace_corpus` →
`books`/`chapter_labels` → `update_chapter` → `lint` → `patches` →
`preview_patch` → `apply_patch` → pull the changed scopes via `to_tokens` →
`set_baseline` → `is_dirty` → `update_book` → `is_dirty` again → `diff_baseline`
→ `prepare_format_patch`/`apply_format_patch` → `vref_index` → `to_usfm` →
`remove_book` → `clear`, plus the typed error cases: unknown book, an
already-stale patch id, a duplicate declared book, an ambiguous chapter,
`set_baseline` on a non-resident book, and a duplicate token id), both ingest
lanes (`usfm`/`tokens`). After every step it converts the native outcome
through the *exact same* `pub(crate)` conversions the real wasm bindings call
(`resident::MutationEffect::from`, `dto::map_lint_issue`,
`dto::map_native_skeleton`, etc.) and records `{step, lane, args, output}` —
args and output both already in the wasm-facing JSON shape — into
`crates/usfm_onion_wasm/tests/fixtures/parity-transcript.json` (committed).
Deliberately lives inside the crate rather than `tests/` (which only sees
`pub` items) specifically so it can reach those `pub(crate)` conversions
instead of hand-mirroring them a second time — the actual bug class this
whole epic exists to prevent. `#[ignore]`d (`npm run
generate:parity-transcript`; regenerate only when the scripted lifecycle
itself changes, the same convention every other golden-fixture generator in
this workspace follows.

`scripts/test-parity.mjs` (new, pattern matched to
`scripts/test-packed-equivalence.mjs`) drives the *real* wasm `Braid` class
(bundler and web builds) through the identical scripted sequence read back
out of the transcript and deep-compares its actual output. The only mapping
layer is unwrapping the `ApiResult {status, value|error}` envelope fallible
verbs return — everything else (hex snapshot/hash ids, tagged
`ScopedOutput`/`ChapterLabel`/error unions, `VrefIndex`'s `[sid,
projection][]` tuple-array shape) comes from the same `Serialize` impl on
both sides, so there is nothing else to translate; a real divergence would
show up as a plain structural diff. Wired as `npm run test:parity` /
`test:parity:web`, both green (54 steps × 2 targets, 0 divergences),
added to the standing gate battery below.

**Divergence found and fixed (the gate's first real catch).** `BookInput`'s
`#[serde(tag = "kind", rename_all = "camelCase")]` looked identical to every
sibling tagged enum in `resident.rs`, but `rename_all` on an enum renames
*variant tags only* — it does not cascade into a struct variant's own field
names (this crate's `dto::TokenFix` already had to learn this lesson via the
separate `rename_all_fields` attribute; `BookInput` never got it). The
generated `.d.ts` shipped `{ kind: "usfm"; source_key: string; ...;
line_ending: LineEnding }` while every other multi-word-field DTO in the
same file was correctly camelCase — a real cross-boundary bug any JS caller
following the (correct, universal-elsewhere) camelCase convention would hit.
Found by the parity generator itself: its own transcript JSON (produced by
calling this exact `Serialize` impl) showed `source_key` where every other
step's JSON showed camelCase. Fixed by adding `rename_all_fields =
"camelCase"` to `BookInput`'s container attribute, matching `TokenFix`'s
existing convention. Verified via the `.d.ts` diff before/after regen: one
clean hunk, `source_key`/`line_ending` → `sourceKey`/`lineEnding`, nothing
else — pkg-bundler and pkg-web both regenerated and committed.

Gates (full battery, this round): `cargo test --workspace --all-features`
green — core 290, wire 196+2, wasm 35 (includes the new parity generator,
run only under `--ignored`), braid 99 across 8 binaries (lib 11, baseline
19, format_patch 12, lifecycle 28, lint_prime 7, patches 7, resident_lint
10, vref 5); `cargo test --release --test lint_oracle -- --ignored`
byte-identical; `cargo test --release -p braid -p usfm_onion_wire -p
usfm_onion_wasm -- --ignored` green; `npm run test:packed` /
`test:packed:web` green (410 cases / 5,717,153 tokens, unchanged); `npm run
test:wasm` (bundler+web) green; `npm run golden:wasm` / `golden:wasm:web`
green (7 fixtures); `npm run test:parity` / `test:parity:web` green (new,
54 steps × 2 targets, 0 divergences); `cargo fmt --all -- --check` clean.
## 2026-07-31 — Final pre-review packet, item 2: DTO dedup audit

Epic item 2: wasm-local DTO shapes that duplicate what
`usfm_onion_wire`/(the since-absorbed) `usfm_onion_dto` already define move
to the single source.

**Audit.** Diffed every `pub struct`/`pub enum` name in
`usfm_onion_wire::dto` against every `pub struct`/`pub enum` name in
`usfm_onion_wasm::dto` and `usfm_onion_wasm::resident`: zero name
collisions. Cross-checked the reason: `usfm_onion_wasm::lib.rs` already
re-exports the *entire* set of wire-owned shared vocabulary at the crate
root (`AttributeItem, BlockBehavior, ..., Span, Token, TokenKind, ...,
format_sid, map_marker_info` — 30 names) rather than redefining any of them,
and every wasm-local type that remains (`LintIssue`, `LintResult`,
`LintSummary`, `DecisionUnit`, `DiffSkeleton`, `Slot`, `Anchor`, `CstNode`,
`FormatOptions`, `FormatFix`, ...) has **no wire counterpart to be a
duplicate of** — `usfm_onion_wire::dto` only owns the packed/wire-layout
primitives (checked directly: it has no `LintIssue`, `LintResult`,
`DecisionUnit`, `DiffSkeleton`, or `Slot` at all). These friendly,
full-fidelity JSON aggregates are wasm's own job as the composition root;
wire has nothing resembling them to consolidate onto. Conclusion: **no
actionable duplication found against `usfm_onion_wire`/`usfm_onion_dto`
this round** — the one real hand-mirroring bug the parity work turned up
(`BookInput`'s field casing) was a correctness bug in a wasm-local type with
no wire or braid counterpart, not a case of two sources defining the same
shape; it is recorded under item 1, not here.

**Noted, not fixed — a larger, separate concern.** `resident.rs` hand-mirrors
several of *braid's own* lifecycle types for Tsify/wasm-bindgen ABI
purposes (`BookInput`, `CorpusInput`, `ChapterTarget`, `ChapterLabel`,
`ChapterInput`, `CorpusScope`, `Scope`, `MutationEffect`, `IngestError`,
`ScopeError`, `PatchError`, ..., and `LineEnding`). The original frozen
design (§2.2#13) anticipated braid growing its own `wasm` feature
(`serde` + tsify/wasm-bindgen glue) so these could derive directly on
braid's own types instead of a second wasm-side mirror; braid's Cargo.toml
still only has the plain `serde` feature, with a comment recording the
`wasm` feature as deferred, not built. This is real, but it is a
`usfm_onion_wire`/`usfm_onion_dto` question's cousin, not its instance — the
packet scoped item 2 to wire/dto duplication specifically, adding a `wasm`
feature to braid and re-deriving a dozen call sites against it is a
materially larger change than a final pre-review packet, and it was never
in this round's stated scope. Recorded here so it is not silently
rediscovered as if new.

**Hard gate.** `.d.ts` diff before vs. after this item: **empty** — no code
changed for this item, which is the honest outcome of an audit that found
nothing left to move, not a skipped step. (Item 1's `BookInput` fix already
regenerated and committed the one hunk that needed to change, ledgered
separately above.)

Gates: none re-run beyond item 1's battery above, since no code changed for
this item.

## 2026-08-01 — Clean-room review: five P1s and one P2, release-blocking round for 0.1.0

Five P1 findings and one P2, all reviewer-evidenced with repros; all fixed
in this round, verbatim regressions added per finding.

**P1.1 — resident `vref_index` diverged from stateless across a chapter
boundary (`crates/braid/src/vref.rs`).** Root cause: the per-run cache
walked each chapter run through a *fresh* visitor, but a whole-book walk
carries one piece of state across a `\c` boundary and never clears it —
whether the most recently opened paragraph-like block supports verse
content (`usfm_onion::vref::IndexedVrefVisitor::current_block_supports_verse`,
the same fact the pre-existing chapter-parallel `vref_map_partitioned` path
already has to reconcile across segments). Reviewer's exact repro
(`\id GEN \c 1 \s1 Heading \c 2 \v 1 text`) confirmed: stateless projects no
entry for GEN 2:1 (the heading does not support verses, and that carries
across the boundary); resident wrongly projected one.

Fix: core gained `usfm_onion::vref::tokens_to_vref_index_seeded(tokens,
incoming_block_state) -> (VrefIndex, outgoing_block_state)` (`src/vref.rs`)
— the same projection, seeded with and reporting the one carried fact, so
`tokens_to_vref_index` is now just `tokens_to_vref_index_seeded(tokens,
None).0`. Braid's cache key became `(TokenIdentity, incoming_block_state)`
instead of identity alone (`CachedRun` gained `incoming_block_state` and
`outgoing_block_state`; `run_entries`/`take_or_compute` both take and match
on the incoming state); `book_vref_entries` threads each run's outgoing
state into the next run's incoming one, in corpus order, so the true value
is always known before a run computes (unlike the parallel path, which has
to assume and speculatively re-walk). `run_vref_entries` (the chapter-scoped
read) now resolves runs `0..=run_index` to establish the same chain before
answering for just the requested one.

One more correctness gap found and fixed during this work, via the
project's own equivalence gate (not the reviewer's repro): a book with
duplicate/reopened `\c` runs can put the *same* sid in two different runs.
The whole-stream stateless projection folds that through one shared
`VrefIndex` (first-seen position, last-write-wins); this crate's per-run
walk uses a separate index per run, so `vref::merge_by_sid` (new) redoes
that fold once over the concatenated result — without it, a duplicate-\c
book reported two entries under one sid instead of the stateless answer's
one.

Regressions (`crates/braid/tests/vref.rs`): the reviewer's exact fixture
(`resident_matches_stateless_across_a_chapter_boundary_that_flips_verse_support`);
a mutation battery where editing only chapter 1's trailing block (via
`update_chapter`, so chapter 2's own tokens and `TokenIdentity` are
byte-for-byte untouched) flips chapter 2's own projection, proving the
cache key genuinely includes incoming state and not just the changed run's
identity (`an_earlier_chapters_trailing_block_change_invalidates_a_later_untouched_chapters_cache`).

**P1.2 — packed restore installed a zeroed lint summary
(`crates/usfm_onion_wasm/src/resident.rs`).** Root cause:
`restoreCorpus`'s per-book `BookLintPrime.result.summary` was
`Default::default()` — braid trusts a primed result outright, so a warm
reopen reported `total_count: 0` with findings plainly present beside it.
Fix: `summarize_findings` (new, mirrors core's own private `summarize`
exactly for the fields the packed format actually carries) rebuilds
`by_category`/`by_severity`/`by_issue_type`/`total_count` from the restored
findings; `suppressed_count` stays `0` as the honest limit of what packed
bytes alone can answer (a suppressed issue is dropped before packing, and
that fact is not in the wire format at all — recorded as a real, not
silently-claimed, limitation).

**P1.3 — a mixed-stamp restore batch adopted the first record's stamps for
all of it (`crates/usfm_onion_wasm/src/resident.rs`).** Root cause: each
record is verified individually via `verify_book`, but the batch-level
`config_fingerprint`/`engine_stamp` handed to `braid::CorpusRestoreInput`
came from re-verifying `records.first()` alone — bypassing `verify_corpus`'s
own invariant ("findings that carry stamps must all carry the same
stamps") entirely, so a second record produced under different stamps had
its findings admitted under the first's. Fix: the per-record loop now
tracks `agreed_stamps`, refusing the whole restore
(`RestoreError::Decode(PackedDecodeError::InvalidSection)`, the same
variant `verify_corpus` itself returns for this exact case) the moment two
records disagree — before anything touches `self.inner`, so the rejection
is atomic. Also removed the wasteful, buggy second re-verification of
`records.first()` in favor of the value the main loop already established.

Regressions (`crates/usfm_onion_wasm/src/resident.rs`, new
`restore_tests` module):
`restore_corpus_recomputes_the_summary_from_the_restored_findings` (P1.2)
and `restore_corpus_refuses_the_whole_batch_when_records_disagree_on_stamps`
(P1.3, two individually-valid records with different stamps → whole
restore refused, resident state provably untouched before/after).

**P1.4 — `reconcileFindings` could resurrect a stale finding on identity
collisions (`js/packed.js`).** Root cause: the identity map kept only the
*first* prior finding per key and never consumed it, so two `next` findings
sharing one identity key could both match that single slot — the
return-the-previous-array shortcut then fired on a count match alone
(`reused === out.length === previous.length`) without checking the match
was one-to-one, letting a genuinely-gone finding (never re-matched, never
consumed) come back as part of the unchanged `previous` array. Fix: `pools`
is now a `Map<key, candidate[]>`; each `next` finding consumes (splices out)
at most one still-available candidate via `sameFindingValue`, so the same
previous finding can never satisfy two different `next` slots, and the
shortcut's count-based check is now sound (one-to-one consumption is what
makes "every count matched" equivalent to "every previous element was
matched exactly once").

Regressions (`scripts/test-packed-equivalence.mjs`): the reviewer's exact
repro (previous `[A(msgA), B(msgB)]` same identity; next asks for A twice —
the second A must not resurrect B); counts differing both directions with
the same identity and content (fewer `next` than `previous` leaves extras
un-reused; more `next` than `previous` leaves extras as fresh objects).

**P1.5 — an owned-token corpus gate was accidentally disabled
(`crates/usfm_onion_wire/src/token_codec_tests.rs`).** Root cause: a doc
comment plus `#[test] #[ignore = "walks the full corpus"]` pair intended for
`corpus_owned_token_sections_round_trip` landed, duplicated, on
`corpus_tokens_round_trip_through_the_boundary_dto` instead, leaving
`corpus_owned_token_sections_round_trip` with no test attribute at all —
dead code (and `EMITTER_DIVERGENCES`, only referenced inside it, dead
alongside it) that `rustc` warned about and the warnings were missed. Fixed
by moving the orphaned doc/attribute pair back to
`corpus_owned_token_sections_round_trip`. Both re-registered ignored tests
run green in release mode (`cargo test --release -p usfm_onion_wire --lib
-- --ignored corpus_tokens_round_trip_through_the_boundary_dto
corpus_owned_token_sections_round_trip`). Added `RUSTFLAGS="-D warnings"`
to the standing gate battery (workspace build+test, `--all-features`,
`--tests`) so a warning is a hard failure from here on — confirmed zero
warnings across the whole workspace with this fixed, plus one pre-existing
unrelated unused-import warning in `crates/usfm_onion_wasm/src/lib.rs`
(`use crate::resident::*;`, genuinely dead) removed so the deny-warnings
gate is actually green, not merely newly enforced against a pre-existing
failure.

**P2 — an inbound boundary silently dropped a contradictory
`attribute_offset` (`src/token.rs`).** `OwnedToken::from_parts` accepted an
attribute-bearing marker with `attribute_offset: Some(_)` but no structured
attributes and no `attribute_source` — the offset (which names where a
*carried* list sits, and is not itself a list) was silently discarded, and
an attribute-bearing token spelling no attributes at all was admitted.
Fixed by rejecting the contradiction as `TokenBuildError::UnexpectedPayload
{ fact: "attribute offset with no attribute list", .. }` — refuse, never
guess, the same rule every other fact-on-the-wrong-shape check in this
constructor already follows. Regression:
`an_attribute_offset_with_no_attribute_list_is_refused` (`src/token.rs`).
Full workspace suite re-run green after this core change, confirming no
existing caller relied on the silently-dropped behavior.

**Parity transcript extended to cover `restoreCorpus`
(`crates/usfm_onion_wasm/src/parity.rs`, `scripts/test-parity.mjs`).** Both
P1.2 and P1.3 escaped specifically because the parity gate never reopened
from packed bytes — it started after a fresh `replace_corpus`, never
`restoreCorpus`. Added `run_restore`: publishes one book with a real
finding, encodes it as a single-book packed container
(`usfm_onion_wire::corpus_codec::encode_corpus`), restores it into a fresh
`resident::Braid` (constructed via a struct literal — `resident.rs`'s
`Braid.inner` field widened to `pub(crate)` for exactly this, the same
justification `restore_tests` already established: `Braid::new`'s public
constructor takes a `js_sys::Function`, meaningless outside a real JS
engine, but every other method needs no JS runtime at all), and records two
new steps per lane: `restore_corpus` (the `RestoreReport`) and
`restore_corpus_then_lint` (a `lint()` call on the *restored* corpus,
proving findings **and** summary match — the exact two facts P1.2/P1.3
broke). `RestoreRecord`'s `packed`/`source` cross the wasm boundary as
`number[]` (confirmed against the generated `.d.ts`, not assumed) since
`RestoreRecord` is a plain, non-tagged struct going through the generic
serde-wasm-bindgen path — the transcript's own JSON number arrays need no
conversion. `npm run test:parity`/`test:parity:web`: 58 steps (52 lifecycle
+ 6 error cases + 4 restore, ×2 lanes÷2... — 26 lifecycle/error steps + 2
restore steps, ×2 lanes = 56... actual count 58, both targets, 0
divergences.

Gates (full battery, this round): `RUSTFLAGS="-D warnings" cargo build
--workspace --all-features --tests` clean, zero warnings; `RUSTFLAGS="-D
warnings" cargo test --workspace --all-features` green (core 291, wire
196+2, wasm 37, braid 99); `cargo test --release --test lint_oracle --
--ignored` byte-identical; `RUSTFLAGS="-D warnings" cargo test --release -p
braid -p usfm_onion_wire -p usfm_onion_wasm -- --ignored` green, including
both re-registered wire corpus tests; `npm run test:packed` /
`test:packed:web` green (410 cases / 5,717,153 tokens, including the new
`reconcileFindings` regressions); `npm run test:wasm` (bundler+web) green;
`npm run golden:wasm` / `golden:wasm:web` green (7 fixtures); `npm run
test:parity` / `test:parity:web` green (58 steps, 0 divergences, now
including restoreCorpus); `cargo fmt --all -- --check` clean. `.d.ts` diff
before/after: empty for both `pkg-bundler`/`pkg-web` (this round changed
behavior, not any public shape) — only the `.wasm` binaries moved; both
regenerated (release profile, matching the existing 0.1.0 packages) and
committed. Fresh-clone build+test verified after commit.
