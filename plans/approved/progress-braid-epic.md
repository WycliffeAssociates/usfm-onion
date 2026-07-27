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
