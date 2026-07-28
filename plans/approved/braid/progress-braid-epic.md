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
