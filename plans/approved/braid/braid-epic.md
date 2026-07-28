# Plan — `braid`: resident USFM state, packed transport, and the four-crate boundary

Date: 2026-07-23. Status: **discussing — owner-level architecture adjudicated; Gate 0 evidence
and contract verification are next.**

This revision turns the original design epic into an implementation-grade plan. It keeps the
owner's product model—token-first editing, complete snapshots, explicit lint, synchronous wasm,
ordered source-faithful storage—but corrects contracts that the live repositories disprove or
leave underspecified. In particular, v1 recomputes lint per dirty **book**, not per dirty chapter;
the packed token transport is a compiled sidecar bound to external UTF-8 USFM bytes; findings
retain their variable payloads; and reconciliation requires book-wide unique stable token ids.

Sibling and structural precedent:
`scripture-sous-chef/documentation/plans/2026-07-22-granularity-spine-plan.md`. Its `Galley` is
braid's content-space cousin. Complete-snapshot publication, explicit analysis, typed mutation
effects, failure-atomic publication, and JS reconciliation are adopted. Its chapter substrate and
boundary-replay machinery are **not** copied: onion's current whole-book rule families and its
much smaller compute cost make that complexity unjustified in braid v1.

## Document authority

This file is the sole normative design and execution queue for the braid epic. It supersedes the
earlier 313-line revision and the three discussing documents already folded into that revision.
The append-only companion `progress-braid-epic.md` records evidence, deviations, and gate results;
it may not silently redefine this plan.

Temporary phase task files may exist only if they:

- name the exact phase and step they execute;
- introduce no architectural decision;
- link here for every contract they rely on;
- stop and propose an amendment when implementation contradicts this plan; and
- are deleted or reduced to durable reference material at closeout.

Any parse, lint, format, diff, USJ, USX, HTML, CST, or vref oracle difference is a behavior change
requiring separate owner adjudication. Wire-layout changes bump the wire version; semantic lint
changes follow the rule-catalog policy. Neither is hidden in a refactor or golden update.

## 0. Problem, solution, and governing model

### Problem statement

The editor currently assembles a stateful USFM system out of application code:
`WorkspaceMirror`, `AnalyzeScope`, per-platform lint batching, chapter flattening, its own token
aliases, and its own Tauri DTO mapping. That duplicates onion's domain boundary and has already
allowed package/application drift to crash a desktop build. The costliest web path also turns
Rust tokens into millions of JavaScript objects through `serde_wasm_bindgen`, even though parse
itself is fast.

### Solution

`braid` is one resident, token-first USFM handle. The caller replaces a complete corpus or
updates complete books/chapters, explicitly asks it to lint, and receives a complete packed
snapshot. Rust owns USFM semantics and lifecycle; the wire crate owns bytes and boundary DTOs;
the wasm crate synchronously exports the wire-owned checked decode/materialization boundary; pure
JS only reconciles already-materialized semantic objects for identity reuse.

The runtime model is:

```text
complete ordered corpus
        ↓
validated token/USFM ingest → resident books + exact source bytes + hashes
        ↓ changed only
dirty-book set (mutations do not lint)
        ↓ explicit lint()
whole-book onion lint for each dirty book → cached native per-book results
        ↓
canonical complete-corpus result → packed token/finding container
        ↓ transferable ArrayBuffer
wasm wire decode/materialize → stable JS token/finding DTOs
        ↓
pure-JS reconcile of validated findings → stable objects for unchanged findings
```

A narrow mutation narrows recomputation. It never narrows the semantic scope of the returned
snapshot. `lint()` always describes the complete resident corpus.

### Golden loop

```text
save valid bytes
    → parse/lint once
    → write exact USFM bytes + packed token/finding sidecar to application-owned storage

open with identical bytes
    → read UTF-8 USFM bytes + packed sidecar
    → verify source length/hash and format/catalog stamp
    → reconstruct borrowed tokens without lex/parse

checkout/pull with changed books
    → reuse matching book sections
    → parse/lint only changed books

edit
    → update complete chapter tokens
    → explicit debounced lint
    → complete packed finding snapshot
    → reconcile by stable identity
```

Persistence policy is application-owned. The libraries encode, decode, validate, and expose
identity; they do not choose paths, perform IO, commit artifacts, or coordinate Git/LFS.

## 1. User stories

1. As an editor user, I want typing and lint refreshes to remain synchronous in the webview, so
   that async lifecycle plumbing does not leak through the editor.
2. As an editor user, I want unchanged findings to retain UI identity, so that lint refreshes do
   not repaint or reset interaction state.
3. As an editor user, I want a lint refresh to describe the whole project, so that a chapter edit
   cannot make findings from untouched books disappear.
4. As an editor user, I want malformed and duplicate USFM retained faithfully, so that opening a
   document never silently repairs or discards my source.
5. As an editor user, I want ambiguous chapter edits rejected clearly, so that the library never
   guesses which duplicate chapter I meant.
6. As an editor user, I want lossless token-to-USFM serialization, so that saving unchanged work
   preserves exact source bytes.
7. As an editor user, I want lint autofixes to remain available after packed transport, so that a
   performance optimization does not remove existing actions.
8. As an editor user, I want localized messages to retain their parameters, so that the app can
   render findings in the active locale rather than accepting baked English strings.
9. As an editor developer, I want one canonical token boundary, so that web and Tauri adapters
   cannot drift independently.
10. As an editor developer, I want mutation methods to report `Changed` or `Unchanged`, so that
    publication invalidation is explicit and no adapter rediscovers it heuristically.
11. As an editor developer, I want invalid inputs rejected atomically, so that a failed chapter
    update cannot partially mutate resident state.
12. As an editor developer, I want book-wide unique token ids validated once at ingest, so that
    finding identity is dependable internally.
13. As an editor developer, I want token indices treated as snapshot-local addresses, so that no
    code mistakes them for stable identities.
14. As an editor developer, I want source hashes based on serialized USFM bytes, so that dirty and
    cache decisions match what will actually be saved.
15. As an editor developer, I want hashes used as a fast selector rather than proof of equality,
    so that a collision cannot suppress a real mutation.
16. As a Rust consumer, I want semantic results without decoding the packed JS representation, so
    that native code does not pay for the wasm boundary.
17. As a Rust consumer, I want decoded tokens to borrow slices from the exact external USFM bytes,
    so that a cache load avoids both lex/parse and per-token string recreation.
18. As a JavaScript consumer, I want one official decoder and reconciler, so that applications do
    not hand-maintain byte offsets or identity rules.
19. As a JavaScript consumer, I want malformed buffers rejected before typed-array views escape,
    so that corrupt cache data cannot cause partial materialization.
20. As a maintainer, I want a one-way crate dependency graph, so that semantic, stateful, wire,
    and adapter responsibilities remain testable in isolation.
21. As a maintainer, I want stable append-only rule codes and explicit schema versions, so that
    old caches are rejected rather than misinterpreted.
22. As a maintainer, I want every fixed-width limit measured against the corpus before release, so
    that compact fields are evidence-backed rather than hopeful.
23. As a maintainer, I want the current behavior oracle green at every gated commit, so that
    architecture work does not smuggle semantic changes.
24. As a maintainer, I want spike code treated as evidence rather than authority, so that its
    assertions, panics, and private-index shortcuts do not become production contracts.
25. As a solo developer, I want phase boundaries that can pause indefinitely, so that each merged
    step leaves the repository coherent and releasable.
26. As a future raw-text consumer, I want today's token-relative addressing documented precisely,
    so that a later chapter-relative text address can be added without pretending it already
    exists.
27. As a future API designer, I want deferred type upgrades named but absent from v1, so that the
    current API does not accumulate speculative variants.

## 2. Decisions after second-opinion review

### 2.1 Retained owner decisions

1. Token space is primary. USFM strings are a cold-load and serialization boundary, not the live
   editor model.
2. Braid is a generic USFM-domain boundary; the editor remains the coordinator for network, file
   IO, scheduling, source keys, and cross-library sequencing.
3. The resident corpus is ordered and source-faithful. Nothing numerically sorts chapters or
   silently collapses duplicate chapter runs.
4. Mutations validate/mutate/hash/invalidate only. `lint()` is explicit and may coalesce several
   edits.
5. A complete snapshot plus JS reconciliation is the v1 diff effect. There is no delta/tombstone
   protocol.
6. Interactive webview calls are synchronous wasm. Threaded wasm, SAB, COOP/COEP, and workers are
   absent.
7. Core remains address-agnostic and honors caller token ids/SIDs.
8. xxhash3 is the persisted content fingerprint; FxHash remains internal-only.
9. Marker registry knowledge remains core-owned and is re-exported by adapters, never copied.
10. The public npm package remains one strongly typed entrypoint. Cargo features create build
    variants; JS bundler tree-shaking is not treated as wasm slimming.

### 2.2 Amended decisions

1. **Dirty-book lint in v1.** Current onion lint explicitly keeps structure, duplicate-chapter,
   and number/verse families on the whole stream. Therefore a changed chapter marks its book
   dirty and `lint()` reruns that whole book. Chapter-local lint computation is deferred until a
   separate plan classifies every rule and proves byte-identical parity. Reconcile still limits
   JS churn.
2. **`usfm_onion_dto` is absorbed into `usfm_onion_wire`.** “Grows into wire or a sibling” was
   not a boundary. There are exactly four Rust crates after migration, not four plus a DTO crate.
3. **Packed tokens are an external-source sidecar in v1.** The caller supplies the exact UTF-8
   USFM bytes plus the packed metadata. The token section binds to those bytes by source length
   and xxhash3, and every token/attribute offset indexes that supplied source. Embedding the same
   bytes for a one-read whole-corpus artifact is a separate candidate, not v1 storage overhead.
4. **Finding sidecars are normative.** Rule code plus token/anchor cannot reproduce current
   `message_params`, related anchors, markers, or `TokenFix`. Compact records carry common fields;
   typed sidecars carry the variable/rare payload.
5. **Stable token id uniqueness is validated per book.** The editor's ordinary Lexical node ids
   satisfy this; synthesized linebreak ids currently do not. Phase F fixes those ids before
   adoption. Duplicate ids are an ingest error, never a reconcile multiset heuristic.
6. **SID fidelity is a wire concern in v1.** Core `Sid` remains its current 8-byte canonical anchor
   with a full `u8` bridge delta. `PackedSid` uses the high bit for fidelity only in the packed
   representation. No core SID semantic change or oracle bless is bundled into the wire work.
7. **Semantic and representation APIs are separate.** `braid::Braid::lint()` returns a native
   semantic snapshot. `usfm_onion_wire` packs it; `usfm_onion_wasm::Braid::lint()` adapts the same
   operation to `Uint8Array`. Braid does not own codec tables.
8. **Core owns the canonical semantic `OwnedToken`.** Wire owns only boundary `TokenDto` and
   codecs. Braid and wire are sibling consumers of core; wasm is their composition root.
9. **Declared semantic identity and resident source binding are separate.** Each resident input
   supplies a unique canonical `BookId` from the caller's agreed manifest plus a unique opaque
   `SourceKey`, normally its path. The declared `BookId` keys corpus scope and the portable
   three-byte wire header; the source's current `\\id` remains editable content and may temporarily
   disagree. `SourceKey` is not written into wire or used for cache validity. Moving a file
   rebinds metadata without invalidating matching `BookId + source hash + semantic stamps` data.
10. **Validated cached lint may prime residency per book.** The composing adapter proves wire and
    catalog compatibility; Braid proves exact source hash, lint-config fingerprint, and a
    deterministic engine stamp. Package version alone is not the semantic cache key.
11. **Expected boundary failures are typed values.** Generated TypeScript uses tagged
    `ApiResult<T, E>` unions. Exceptions are reserved for programmer errors and violated internal
    invariants.
12. **The manifest book is authoritative; source `\\id` is lintable content.** Braid passes the
    declared `BookId` into a core-owned lint context. Invalid codes use the separately filed
    `InvalidBookCode` rule; a valid-but-wrong code uses `BookIdMismatch { expected, found }`.
    Neither condition prevents residency or packing under the declared three-byte header id.
13. **Braid serializes its own lifecycle types behind a feature.** (Adjudicated 2026-07-27 from the
    Gate 0C/0G ledgers.) The TypeScript mirrors of braid's lifecycle surface (`CorpusInput`,
    `MutationEffect`, `IngestError`, and the rest of §5.3/§5.6) cannot live in wire without a
    wire→braid reverse edge, and hand-written wasm mirrors would reinstate the DTO-drift disease.
    Therefore braid derives serialization **on its own types** behind features — split (per Gate
    0F, amendment F) into `serde` (plain derives; what a native Tauri IPC host needs, with no
    tsify/wasm-bindgen in the binary) and `wasm` (= `serde` plus tsify/wasm-bindgen glue), the
    same split mirrored on `usfm_onion_wire`; default `braid` stays dependency-pure and the wasm
    crate enables `wasm`. The tsify-generated declarations remain the single TypeScript contract
    for both transports because tsify derives from the same serde attributes. §4.1's "braid must not own JS values" is read narrowly: deriving a serialization for a
    type braid already owns is not owning a JS value — the prohibition targets byte layouts and DTO
    shapes distinct from braid's semantics.
14. **Exactly one wasm crate, hosting both surfaces.** The stateless one-shot exports and the
    stateful `Braid` class live in the same `usfm_onion_wasm` crate (§4.2 `stateless.rs` +
    `braid.rs`); `braid` itself carries no wasm-bindgen dependency. A separate `braid-wasm` crate
    is justified only to ship the stateful handle as an independent npm package, which no consumer
    needs today; it is not created in v1. This is the wasm analogue of decision 2.2#2 — no extra
    crate without a boundary that earns it.
15. **Canonical finding order keys on token position, not byte span.** (Adjudicated 2026-07-27 from
    Gate 0D finding D1.) `canonical_sort`'s primary key becomes the position of the finding's
    primary token in the book's token stream (anchor-only findings sort last), then the kebab-case
    code string, then related-token position. Position exists identically for parsed and
    caller-token ingest, so resident lint order equals stateless order without `OwnedToken`
    carrying spans. For parsed tokens every finding span is a whole token (Gate 0E), so position
    order equals today's span order and the re-key is expected oracle-neutral — verify against the
    lint oracle before adoption; any difference is a stop requiring separate adjudication.
    Token-id strings leave the sort path entirely, removing the Rust-UTF-8 versus JS-UTF-16
    comparator divergence; packed finding records are stored in canonical order, so Rust
    materialization and pure-JS reconciliation preserve order without any comparator.
    Chapter-grain lint (deferred) stays compatible:
    chapter runs are contiguous token ranges, so chapter-local positions rebase by run start, and
    positions remain snapshot-local addresses, never identity. Reference/canon-order presentation
    is a consumer concern — findings carry SIDs, and the library keeps one source-faithful order.
16. **Per-book line endings are a braid contract.** (Adjudicated 2026-07-27 from Gate 0F.) Each
    resident book has a `LineEnding` (`Lf | CrLf`): detected from the source for
    `BookInput::Usfm`, declared on `BookTokensInput` for token ingest, inherited by
    `ChapterInput`. `to_usfm` and patch/backup-embedded USFM emit newline tokens using the stored
    ending via an optional override on the core reconstruct emitter — core owns emission; braid
    never post-processes newline tokens itself. `SourceHash` is computed over the emitted bytes,
    so warm caches and dirty state match what is actually saved. Mixed-EOL `Usfm` input is
    preserved verbatim until first edit (never silently normalized). The packed container does not
    carry the ending — it is derivable from the bound source bytes. Without this contract every
    CRLF book has a permanently invalid warm cache and permanent dirty state (Gate 0F probe P7).
17. **Packed warm restore is the supported cold-open path.** (Adjudicated 2026-07-27 from Gate
    0F.) The golden loop promises "reconstruct borrowed tokens without lex/parse" on open; the
    surface must actually offer it. Braid gains an atomic `restore_corpus` that seeds resident
    books (source + decoded semantic tokens) and their lint contributions in one validated call,
    applying the same per-book stamps as `prime_lint_cache`; rejected books fall back to normal
    ingest+lint. The wasm crate — the composition root, which may call both wire and braid —
    exposes `restoreCorpus(sources, packed)` that decodes via wire and seeds braid, so braid stays
    wire-free. The re-parse-plus-`prime_lint_cache` path remains the documented fallback, not the
    silently enshrined default.
18. **The native resident host is a first-class v1 hosting path.** (Adjudicated 2026-07-27 from
    Gate 0F; supersedes the §8.3 deferral.) The editor's desktop process hosts a resident native
    `Braid` behind Tauri commands calling the same lifecycle methods and returning packed bytes.
    Braid and wire therefore ship the native surface: a plain `serde` feature independent of
    tsify/wasm-bindgen (see #13), public native codec entrypoints on wire (§7), and native
    round-trip contract tests in their own suites. The Tauri command host itself remains
    editor-owned app code; braid ships no Tauri dependency, and Phase F parity transcripts run
    against both the web wasm host and the native host.
19. **Rust is the only production packed decoder.** (Owner correction 2026-07-28.)
    `usfm_onion_wire` exclusively owns checked packed decode, checksum/source binding, and
    semantic token/finding materialization. The npm-facing `decodeTokens(packed, book, source)`
    and `materialize(sources, packed)` are wasm exports backed directly by that Rust wire path,
    returning ordinary DTOs/`MaterializedSnapshot` or the existing frozen typed errors. JS passes
    packed bytes and external source bytes into wasm; on a typed failure it falls back to normal
    Braid USFM ingest/parse. Native hosts call wire directly. Generated `./wire-schema` constants
    are conformance/debug artifacts only — never a second production parser or a JS XXH3
    implementation. Pure JS may reconcile already-validated semantic finding objects for identity
    reuse, but never accepts or parses packed bytes. The previously proposed npm `decodeView` raw
    buffer surface is narrowed away: `decode_view` remains a public native Rust representation API
    if Phase 0 retains that exact name, but is not a JS/npm parser.

## 3. Hard preconditions and Gate 0

Gate 0 produces evidence only. Do not add crates or change public types until every item is
recorded in `progress-braid-epic.md`.

### 3.1 Pre-spike exploration protocol

The order is normative. Complete 0A–0G before running or modifying a wire/performance spike.
Exploration answers what must be preserved and what the spike must prove; a benchmark run before
those contracts are frozen is not evidence for this epic.

Large generated inventories may live temporarily under `target/braid-gate0/`. Durable conclusions,
commands, input hashes, maxima, failures, and links to any retained evidence file are appended to
`progress-braid-epic.md`. Raw generated output is not normative and must not become a new hand-kept
schema source.

#### 0A — provenance and reproducibility

Run from the primary checkout and record exact output from:

```bash
git rev-parse HEAD
git status --short
git worktree list --porcelain
rustc -Vv
cargo -V
node --version
npm --version
wasm-pack --version
wasm-opt --version
cargo metadata --no-deps --format-version 1
```

Also record the commit and dirty state of `.claude/worktrees/agent-af68c779deab4e90a` and every
fixture/corpus path used later. Never clean, reset, update, or merge a preserved worktree merely to
make the replay convenient. If a required tool is absent, record that as an environment blocker;
do not silently substitute a different build profile or runner.

Deliverable: an environment ledger containing checkout commit, dirty paths classified as
pre-existing versus Gate 0 output, worktree commits, tool versions, workspace members/features,
and SHA-256 hashes of benchmark inputs. This is provenance only; xxhash remains the product
content fingerprint.

#### 0B — behavioral baseline before measurement

Run the supported paths without any bless/update environment variable:

```bash
cargo test --workspace
cargo test --test lint_oracle -- --ignored
npm run check:wasm:web
npm run test:wasm
npm run golden:wasm
npm run golden:wasm:web
npm run test:token-sids:import
```

`tests/lint_oracle.rs` is `#[ignore]`d by design (it stays an explicitly-invoked gate for
known-clean checkouts); it only exercises the oracle under `-- --ignored`, so the bare command is a
silent no-op. Never set `BLESS=1`. Likewise do not set `UPDATE_GOLDEN=1`. Record pass/fail, duration, and any platform-only
failure separately. A semantic/golden failure is a stop. An environment failure is still a stop
until reproduced or explicitly adjudicated; it is not permission to call the baseline green.

Deliverable: a baseline ledger naming every command and result, plus hashes of
`tests/lint_oracle_baseline.txt`, the generated wasm declarations, and representative golden
outputs. No production or snapshot file changes in this step.

#### 0C — public-contract and ownership census

Build one ledger with these columns:

```text
symbol/export | current owner/file | exact Rust/TS signature | serialized shape |
known consumers | target owner | migration action | compatibility disposition | proof
```

Inventory at minimum:

- core `Token`, `TokenData`, `TokenId`, `Sid`, spans, attributes, marker metadata, and every token
  trait used by lint/format/diff;
- `LintOptions`, `LintScope`, `LintResult`, `LintIssue`, `LintCode`, `TokenFix`, summaries, rule
  metadata, and canonical sorting;
- `DiffSkeleton<T>`, `DecisionUnit<T>`, `Slot`, `UnitId`, merge/revert functions, format result
  types, and USFM/USJ/USX/HTML/CST/vref outputs;
- every public declaration in `usfm_onion_dto`, `usfm_onion_wasm`, `pkg-bundler/*.d.ts`,
  `pkg-web/*.d.ts`, root `package.json` exports, and `js/token-sids.*`;
- serde/tsify/wasm-bindgen conversions that rename fields, erase lifetimes, clone source strings,
  translate enums, or turn Rust errors into exceptions.

Do not infer the npm surface solely from Rust source: compare both generated declaration trees and
actual package export maps. Do not infer downstream source compatibility from behavior-oracle
parity. Every current export must be marked retain, replace, or delete-in-breaking-release.

Deliverable: the API/ownership ledger and a proposed final Cargo edge list. Any type that would
force core to depend on wire/braid/wasm, or braid to depend on wire, is a stop and requires a plan
amendment.

#### 0D — semantic payload and ordering census

Before designing rows or sidecars, enumerate actual semantic variants:

1. Produce a token-variant matrix from source definitions plus adversarial fixtures. For every
   `TokenKind`/`TokenData` variant, record required/optional source span, SID, marker metadata,
   structural metadata, number data, book-code validity, attributes, and exact-USFM reconstruction
   requirements.
2. Produce a finding matrix with one row per `LintCode`. Record severity/category/issue type,
   primary and related anchors, token-relative spans, marker, message parameters, and every
   `TokenFix` variant. Cite at least one fixture/oracle line or mark the variant synthetically
   generated.
3. Capture canonical finding sort keys and tie behavior with duplicate-logical-identity cases.
   Determine where deterministic occurrence is assigned and prove a JS reconciler can reproduce
   that ordering without message text as identity.
4. Record how declared manifest `BookId` enters core lint context. Prove invalid source `\\id` and
   valid-but-wrong `\\id` remain resident, while no-context stateless lint retains current behavior.
5. Trace every finding field and fix used by the editor. A field absent from current fixtures is
   not presumed unused.

Deliverable: token and finding conformance ledgers. If any semantic field, localized message
argument, related address, attribute spelling, or fix cannot round-trip, stop before layout work.

#### 0E — corpus envelope and width study

Write or use a deterministic read-only scanner. Scan these sets separately:

- every `testData/**/*.usfm` adversarial fixture;
- wasm golden inputs;
- `example-corpora/en_ult` and `example-corpora/en_ulb` when present;
- at least one real editor corpus, if locally available and authorized, with only aggregate/path
  metadata recorded—never source text copied into the plan.

Keep per-book maxima distinct from whole-corpus totals; do not concatenate books and accidentally
prove a per-section width against the wrong grain. For every quantity, record count, p50/p95/max,
the book/fixture producing the max, proposed field width, reserved sentinel values, remaining
headroom, and specified overflow/version behavior. Measure at least:

- source bytes, token count, token/source-span endpoints, string-dictionary bytes/entries;
- unique token ids, SIDs, markers, book codes, and attributes;
- max attributes per token/book and max marker/attribute/token UTF-8 byte length;
- chapter, verse, bridge delta, exact packed-SID delta boundary 127/128, and unique SID count;
- finding count, rule-code count, token-relative offset/length, message payload bytes, related
  addresses, patch count, edit count, and replacement-token count;
- section count, section size, TOC size, complete container size, and total corpus source bytes;
- declared-book/source-`\\id` mismatches, invalid source book codes, duplicate declared book ids,
  duplicate source keys, and duplicate stable token ids.

Add synthetic boundary fixtures for every sentinel and overflow branch even when real corpora do
not reach it. “Observed maximum fits” is insufficient where no versioned overflow behavior exists.

Deliverable: the width ledger. A maximum at/over a proposed field, collision with a sentinel, or
unbounded value without a sidecar/format-version path is a stop.

#### 0F — editor consumer and lifecycle exploration

This is read-only exploration in `scripture-editor-proto-2`; no editor implementation belongs in
Gate 0. Record exact files/symbols for:

- the manifest/path convention that supplies declared `BookId` and resident `SourceKey`;
- full-corpus seed order and whether every working document can provide exact UTF-8 bytes;
- `lexicalToTokens()` and every synthetic token-id path, especially chapter-local `linebreak-N`;
- whether chapter updates always contain a complete chapter run rather than an edit fragment;
- token/finding fields actually read by lint decoration, result browsing, formatting, fixes,
  compare/diff, save, undo/redo, and Tauri/web adapters;
- current whole-book lint scheduling, stale-result suppression, frozen snapshots, and when token
  object identity is expected to survive;
- current cache/persistence inputs, including where Onion/package version, source hash, lint
  options, and findings are available for `prime_lint_cache`;
- the exact cold-open and warm-update call transcripts that will become Phase F parity fixtures.

Run a duplicate-id scanner over a real fully-backed project and report counts by book and token
kind. Separately exercise edit, undo/redo, format, save/reopen, and linebreak creation to distinguish
identity that is merely unique in one call from identity that persists across calls.

Deliverable: an editor contract ledger. Stop if the editor cannot supply unique declared books,
complete ordered sources, complete chapter token streams, or book-wide stable token ids without a
separately owned prerequisite.

#### 0G — dependency, feature, and generation feasibility

Using `cargo metadata`, manifests, generated declarations, and wasm build scripts, prove the target
DAG under both native and `wasm32-unknown-unknown`:

```text
            usfm_onion
             ↗      ↖
usfm_onion_wire      braid
             ↖      ↗
         usfm_onion_wasm
```

Record where canonical `OwnedToken`, `TokenDto`, generated rule/schema constants, and conversion
impls live. Verify core can own `OwnedToken` without acquiring serde/tsify/wasm-only dependencies
in its default semantic layer; feature-gated serialization must not create a reverse edge. List
every current consumer of `usfm_onion_dto` and the commit step that removes it. Verify bundler and
web builds consume one generated schema source and cannot silently ship different discriminants.

Deliverable: native/wasm dependency and feature matrices, plus the declaration-generation command
and drift check. A cycle, duplicated discriminant source, or target-specific semantic type is a
stop.

#### 0H — freeze the spike charter, then replay

Only after 0A–0G pass, write the spike charter before running timings. It must state:

- hypotheses: exact external-source token reconstruction without lex/parse; semantic equality;
  malformed-input rejection; object-materialization and transfer cost; size/heap benefit;
- non-hypotheses: chapter-local lint, threaded wasm, cache IO policy, and production API polish;
- exact input paths/hashes, build profile, target, warmup count, measured iterations, reported
  statistic, machine/runtime versions, and whether filesystem IO is inside or outside timing;
- correctness checks that execute before timers: decoded semantic tokens equal `parse(source)` in
  every public field; token-to-USFM is byte-identical; Rust serial and parallel decode agree where
  available; JS materialization agrees with Rust; wrong source and malformed buffers reject;
- independent timing phases: read, checksum/validation, decode-view construction, semantic object
  materialization, wasm parse, wasm-to-JS marshalling, structured clone/transfer, and peak heap;
- stop thresholds: any semantic mismatch, any unrepresented payload, any private marker-index
  dependency, or loss of the measured boundary advantage.

Replay `.claude/worktrees/agent-af68c779deab4e90a` unchanged first, using its documented commands:

```bash
cargo run --release --example wire_spike
node js/wire_decode.mjs <output-dir>
node js/wasm_vs_bin.mjs <output-dir>
node js/spike_b.mjs <output-dir> PSA
node js/spike_b.mjs <output-dir> GEN
```

The replay is historical evidence, not production design. In particular, its JS prototype embeds
a `SOURCE` section and uses private/raw catalog assumptions that v1 rejects. Record those deltas
explicitly. Do not copy spike code until a narrow follow-up proves the normative external-source,
three-byte declared-book header, checked-directory, and stable-catalog contracts with the same
semantic equivalence gate. Timing results from the old embedded-source format cannot by themselves
approve the new layout.

Deliverables: spike charter, unchanged-replay results, contract-delta list, and a yes/no decision
for each Phase A component: reuse, rewrite, or discard. Only then may Phase A add production code.

### 3.2 Gate 0 acceptance checklist

This is the compact completion checklist; it does not override the 0A–0H execution order above.

1. Record the execution base commit, dirty paths, preserved spike worktrees, toolchain versions,
   and current workspace members. Do not delete or clean user-owned worktrees.
2. Run the normal Rust suite, wasm package tests/goldens, and the lint oracle without `BLESS=1`
   or `UPDATE_GOLDEN=1`. A baseline failure is a stop clause.
3. Record the current public shapes of core `Token`, `TokenId`, `Sid`, `LintIssue`, `TokenFix`,
   `LintOptions`, `LintResult`, `FormattableToken`, `LintableToken`, and `DiffableToken`.
4. Record the current DTO/wasm duplicate types and every npm export. The migration must account
   for each exported type/function; silence is not permission to delete it.
5. After 0A–0G, replay the historical Phase A spike harness from `agent-af68c779deab4e90a`
   unchanged and record its commit, inputs, output sizes, and timings. The spike is not copied
   until its equivalence test passes on the execution base and its contract deltas are classified.
6. Scan every `testData/**/*.usfm` and chosen real corpora for: source byte length, token count,
   unique SID count, unique marker count, attribute count per token, attribute count per book,
   max token byte length, max token-relative finding offset/length, max chapter/verse/bridge,
   finding count per book, and rule-code count. Compare each maximum to every `u8/u16/u32` field.
7. Inventory every emitted `LintCode` and pin which `LintIssue` fields it uses: span,
   related-span, token id, related token id, SID, marker, message params, and fix variant. This
   becomes the sidecar conformance ledger. Any payload that cannot round-trip is a stop clause.
8. Verify the current canonical finding order and tie behavior. Packed decode must reproduce it;
   reconciliation must not define a new order.
9. In the editor, scan a real fully-backed project for duplicate token ids within each book.
   Confirm the known `linebreak-N` collision and identify the owning adapter change. Do not weaken
   the library invariant to accommodate synthesized unstable ids.
10. Verify that editor chapter updates supply complete chapter token streams and that GUID-backed
    node ids survive ordinary edits, undo/redo, formatting, and serialization/reload where
    identity reuse is expected.
11. Pin the exact fields the editor consumes from decoded tokens and findings. Current evidence
    includes token source/kind/id/SID/marker/number/attributes and finding code/category/severity/
    issue type/message params/SID/token ids/fix. Correct this plan if live consumers require more.
12. Extend and separately bless the filed invalid-book-code work with a core-owned optional
    declared-book lint context and `BookIdMismatch` rule. Prove stateless callers without context
    retain current behavior, while Braid reports a valid-but-wrong `\\id` against its token.
13. Prove one-way Cargo dependencies are possible with no feature cycle:

    ```text
                usfm_onion
                 ↗      ↖
    usfm_onion_wire      braid
                 ↖      ↗
             usfm_onion_wasm
    ```

    Core owns the canonical semantic `OwnedToken`. Wire and braid are sibling consumers of core;
    neither depends on the other. Wasm composes all three. Core never depends on wire/braid/wasm.
14. Treat the crate/npm version carrying Phase F as one deliberate breaking pre-1.0 release.
    Crate names, wire types, and generated declarations may change; review them as an explicit API
    diff. Do not retain deprecated duplicate DTO compatibility aliases.

The owner-level amendments in §2.2 are accepted. Gate 0 passes when every fixed-width field is
proven sufficient or has the overflow path specified below and all evidence/precondition checks
pass.

## 4. Crate and module boundaries

### 4.1 Final workspace

| Crate/module | Owns | Must not own |
| --- | --- | --- |
| `usfm_onion` | borrowed parse tokens; canonical semantic `OwnedToken`; USFM semantics; marker registry; lint/format/diff/vref/USJ/USX/HTML/CST; semantic rule catalog | resident state, JS DTOs, packed offsets, cache IO |
| `usfm_onion_wire` | boundary `TokenDto`; packed schema/version; checked packed token/finding decode; checksum/source binding; semantic materialization; serde/tsify DTOs; generated `wire-schema` conformance/debug data | semantic token ownership, lint execution, dirty state, lifecycle, cache IO, a JS decoder/hash implementation |
| `braid` | complete ordered corpus; validation; current/baseline state; hashes; dirty books; mutation/lint lifecycle; native complete snapshots; feature-gated serde/tsify derives over its own lifecycle types (§2.2#13) | byte layout, JS DTO shapes distinct from its semantics, IO, scheduling, duplicate rule logic |
| `usfm_onion_wasm` | wasm-bindgen classes/functions; JS input mapping; wasm exports for wire-owned materialization; `Uint8Array` return; core registry re-exports | copied DTO enums, second codec/parser/hash, different lifecycle semantics |
| generated `./wire-schema` + pure JS reconcile helper | generated schema conformance/debug constants; identity reuse over validated semantic findings | packed parsing, checksum/hash validation, source binding, token/finding materialization, lint rules, invalidation, persistence policy |

`usfm_onion_dto` is deleted only after all imports move to `usfm_onion_wire` and generated npm
declarations prove equivalent or intentionally changed. There is no compatibility crate left
behind.

### 4.2 Suggested source layout

```text
crates/usfm_onion_wire/src/
    lib.rs
    dto.rs                 serde/tsify boundary DTOs and core-token conversions
    schema.rs              magic, versions, field ids, stable discriminants
    token/{encode,decode}.rs
    finding/{encode,decode,payload}.rs
    container.rs           header, TOC, validation
    error.rs

crates/braid/src/
    lib.rs
    input.rs               CorpusInput/BookInput/ChapterInput validation
    corpus.rs              ordered books/runs and derived lookup metadata
    state.rs               clean/dirty lifecycle and mutation effects
    lint.rs                dirty-book orchestration and complete snapshot
    baseline.rs            baseline/diff/dirty/serialize operations
    error.rs

crates/usfm_onion_wasm/src/
    lib.rs
    braid.rs               wasm class only
    stateless.rs           existing one-shot adapters
    map.rs                 boundary conversions not owned by wire DTOs
```

Files should remain responsibility-focused; this is a boundary map, not permission to create an
abstraction for every filename.

### 4.3 Reusing core logic across the boundary

wasm-bindgen is the behavior bridge — it exports functions and structs-as-JS-classes; serde/tsify
ride on it to marshal DATA only. Tagging a type never carries an algorithm across the boundary. So
core LOGIC is reused, never reimplemented in an adapter, in this preference order:

1. wasm-bindgen-export the core function directly (owned/simple arguments).
2. Make the core function generic over a token trait (the existing `WalkableToken`/`LintableToken`
   pattern) so borrowed-native and owned-wire tokens flow through one implementation while core
   keeps borrowing on its hot path.
3. Only if a trait cannot express it, a thin adapter that reconstructs owned→borrowed and calls core.
4. Never reimplement the algorithm against wire types — that is the behavior-level form of the
   DTO-mirror drift (§15 "Algorithm fork").

Known existing violation to retire during adoption: `usfm_onion_wasm`'s `token_values_to_usfm`
(with `format_attribute_list` / `closer_shape` / `token_closes`) reimplements USFM emission because
core `tokens_to_usfm` takes borrowed `Token<'a>` while JS supplies owned wire tokens. The fix is
option 2 — a core `SerializableToken` trait the wire token implements, so wasm calls the one core
emitter. See §17 ledger.

## 5. Domain types and public Rust surface

The signatures here are normative at the semantic level. Exact derives and private storage may
change during implementation, but weakening a type into strings/booleans/optional bags requires
owner review.

### 5.1 Core-owned semantic token and wire DTO

```rust
pub struct StableTokenId(Box<str>);

pub struct OwnedToken {
    pub id: StableTokenId,
    pub kind: TokenKind,
    pub source: Box<str>,
    pub sid: Option<Box<str>>,
    pub marker: Option<Box<str>>,
    pub nested: bool,
    pub number: Option<OwnedNumberInfo>,
    pub book_code: Option<OwnedBookCode>,
    pub attributes: Box<[OwnedAttribute]>,
}

pub struct OwnedNumberInfo {
    pub start: u32,
    pub end: Option<u32>,
    pub kind: NumberRangeKind,
}

pub struct OwnedBookCode {
    pub code: Box<str>,
    pub is_valid: bool,
}
```

The production type may retain additional marker structural metadata if equivalence proves it is
required. It must not accept the current broad optional DTO and then trust illegal combinations.
`OwnedToken`, `StableTokenId`, and their semantic payload types live in core. Construction from a
parsed core token is core-owned; wire owns the DTO conversion:

```rust
impl OwnedToken {
    pub fn from_parsed(value: &Token<'_>) -> Self;
}

impl TryFrom<TokenDto> for OwnedToken {
    type Error = TokenInputError;

    fn try_from(value: TokenDto) -> Result<Self, Self::Error>;
}
```

`OwnedToken` implements the existing core token traits. Core algorithms remain generic and do not
learn about braid or wire. Token kind determines which payload is legal: number tokens require `number`;
book-code tokens require `book_code`; opening markers own marker metadata, `nested`, and may carry
attributes; end markers own marker metadata but never attributes; milestones own marker metadata
and attributes but no `nested` flag (per the Gate 0D token-variant matrix — "marker-like" conflated
three structurally different variants); other kinds reject those payloads. If a single Rust struct makes those invariants too loose, use a
private discriminated payload enum while retaining the current single tagged JS `Token` shape for
v1.

`StableTokenId` is nonempty UTF-8 and unique within one resident book. It is opaque to onion.
Cold USFM parse maps the current positional parser id to `{book}-{index}`. Live token ingest keeps
the caller id byte-for-byte. Reconciliation keys are `(BookId, StableTokenId)`, never `token_idx`.

### 5.2 Corpus inputs and selectors

```rust
pub enum BookInput {
    Usfm { source_key: SourceKey, book: BookId, source: String },
    Tokens(BookTokensInput),
}

pub struct BookTokensInput {
    pub source_key: SourceKey,
    pub book: BookId,
    pub tokens: Vec<OwnedToken>,
    pub line_ending: LineEnding,
}

pub enum LineEnding {
    Lf,
    CrLf,
}

pub struct SourceKey(Box<str>);

pub enum ChapterInput {
    Usfm { source: String },
    Tokens(Vec<OwnedToken>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ChapterLabel {
    FrontMatter,
    Number(Box<str>),
}

pub struct ChapterTarget {
    pub book: BookId,
    pub label: ChapterLabel,
}

pub struct CorpusInput {
    pub books: Vec<BookInput>,
}

pub enum CorpusScope {
    All,
    Book(BookId),
    Chapter(ChapterTarget),
}
```

`BookId` is core's existing `usfm_onion::token::BookId` (`[u8;3]`, 3-ASCII-alphanumeric shape
validation), which core adds to its root re-export list; canonical-66 membership is deliberately
NOT a type invariant — code validity remains a lint concern per §2.2#12 (adjudicated 2026-07-27).
Books are unique by both caller-declared `BookId` and resident `SourceKey` in v1 and remain in
caller order; Hebrew/Protestant order is preserved. The declared `BookId` is authoritative for
corpus addressing and wire identity. The parsed `\\id` token is source content: invalid or
mismatched content remains resident so the editor can display and repair it. Thus manifest books
`1CO` and `2CO` may temporarily both contain `\\id 2CO` without creating duplicate wire sections;
their declared header ids remain unique. Duplicate or reopened **chapter labels** are retained
within one declared book and linted.
`ChapterLabel::Number` holds the exact label token rather than a parsed integer; no numeric sort or
normalization occurs. `FrontMatter` is a variant, not the magic string `"intro"`.

Whole-book replacement is the structural escape hatch. `update_chapter` replaces exactly one
existing contiguous run whose label equals the target. Zero matches returns `ChapterNotFound`;
multiple matches/reopened runs return `AmbiguousChapter`; insertion, deletion, reorder, and
duplicate resolution use `update_book`.

`SourceKey` is deliberately absent from the packed container. It is an ephemeral resident binding
supplied again from the current manifest when reopening. A different application may bind the
same portable `BookId + source hash` sections to different source keys. A rename changes grouped
projection keys but neither semantic `SnapshotId` nor lint-cache validity.

A duplicate **declared** `BookId` is reported by corpus validation before resident `lint()` can
run, because there is no unambiguous book scope to install. `DuplicateBook { book, sources }` is
therefore the typed diagnostic the application may present in its problem UI; packing is
unavailable until the manifest collision is fixed. A duplicate or mismatched source `\\id` is
different: it does not make corpus addressing ambiguous and is reported as a normal core-owned
lint finding against the book-code token. Braid supplies the declared book as lint context; it
does not implement a duplicate rule table itself.

### 5.3 Braid lifecycle surface

```rust
pub struct BraidConfig {
    pub lint: LintOptions,
}

pub enum MutationEffect {
    Unchanged,
    Changed,
}

pub struct PatchId(u32);

pub struct PatchHandle {
    pub snapshot: SnapshotId,
    pub patch: PatchId,
}

pub enum PatchPreparation {
    Unchanged,
    Ready(PatchHandle),
}

pub enum ScopedOutput<T> {
    Single(T),
    All(Vec<SourceOutput<T>>),
}

pub struct SourceOutput<T> {
    pub source_key: SourceKey,
    pub book: BookId,
    pub value: T,
}

// USJ is illustrative: each supported resident projection uses this same
// Single-versus-All envelope rather than inventing a separate grouping rule.
pub struct UsjDocument { /* existing core USJ semantic output */ }

pub struct TokenPatch {
    pub books: Vec<BookTokenPatch>,
}

pub struct BookTokenPatch {
    pub book: BookId,
    pub edits: Vec<TokenEdit>,
}

pub enum TokenEdit {
    Insert { at: u32, tokens: Vec<OwnedToken> },
    Replace { range: Range<u32>, tokens: Vec<OwnedToken> },
    Delete { range: Range<u32> },
}

pub struct Braid { /* private */ }

impl Braid {
    pub fn new(config: BraidConfig) -> Self;

    pub fn replace_corpus(
        &mut self,
        corpus: CorpusInput,
    ) -> Result<MutationEffect, IngestError>;

    pub fn restore_corpus(
        &mut self,
        seed: CorpusRestoreInput,
    ) -> Result<RestoreReport, IngestError>;

    pub fn update_book(
        &mut self,
        replacement: BookInput,
    ) -> Result<MutationEffect, IngestError>;

    pub fn update_chapter(
        &mut self,
        target: ChapterTarget,
        replacement: ChapterInput,
    ) -> Result<MutationEffect, IngestError>;

    pub fn remove_book(&mut self, book: BookId) -> MutationEffect;

    pub fn remove_chapter(
        &mut self,
        target: ChapterTarget,
    ) -> Result<MutationEffect, ScopeError>;

    pub fn clear(&mut self) -> MutationEffect;

    pub fn update_config(&mut self, config: BraidConfig) -> MutationEffect;

    pub fn prime_lint_cache(
        &mut self,
        prime: LintPrimeInput,
    ) -> Result<PrimeReport, PrimeError>;

    pub fn lint(&mut self) -> Result<&LintSnapshot, LintError>;

    pub fn prepare_format_patch(
        &mut self,
        scope: CorpusScope,
        options: FormatOptions,
    ) -> Result<PatchPreparation, FormatError>;

    pub fn apply_patch(
        &mut self,
        handle: PatchHandle,
    ) -> Result<MutationEffect, PatchError>;

    pub fn set_baseline(
        &mut self,
        replacement: BookInput,
    ) -> Result<MutationEffect, IngestError>;

    pub fn clear_baseline(&mut self, book: BookId) -> MutationEffect;

    pub fn diff_baseline(
        &self,
        scope: CorpusScope,
    ) -> Result<ScopedOutput<DiffSkeleton<OwnedToken>>, BaselineError>;

    pub fn is_dirty(&self, scope: CorpusScope) -> Result<bool, ScopeError>;

    pub fn to_tokens(
        &self,
        scope: CorpusScope,
    ) -> Result<ScopedOutput<Vec<OwnedToken>>, ScopeError>;

    pub fn preview_patch(
        &self,
        handle: PatchHandle,
    ) -> Result<ScopedOutput<Vec<OwnedToken>>, PatchError>;

    pub fn to_usfm(&self, scope: CorpusScope) -> Result<ScopedOutput<String>, ScopeError>;

    pub fn to_usx(&self, scope: CorpusScope) -> Result<ScopedOutput<String>, ProjectionError>;

    pub fn to_usj(
        &self,
        scope: CorpusScope,
    ) -> Result<ScopedOutput<UsjDocument>, ProjectionError>;

    pub fn to_html(
        &self,
        scope: CorpusScope,
        options: HtmlOptions,
    ) -> Result<ScopedOutput<String>, ProjectionError>;

    pub fn expected_snapshot_id(&self) -> SnapshotId;
}
```

`lint()` is the only recompute verb. It returns the current semantic snapshot by reference; the
caller may then encode it. Calling `lint()` while clean returns the same logical snapshot with no
rule execution. Calling it on an empty braid returns a valid empty snapshot.

Braid resolves the core linter's current `TokenFix` recipe against the resident snapshot into a
flat `TokenPatch`, stores it in that snapshot's patch table, and publishes a typed
`PatchHandle { snapshot, patch }`. Resident formatting uses the same mechanism:
`prepare_format_patch(scope, options)` reads only the ingested corpus and returns either
`Unchanged` or a snapshot-bound patch handle; it never mutates implicitly. One handle may cover
several books selected by `CorpusScope::All`, while each book retains a flat edit vector.
`apply_patch` rejects a stale snapshot, verifies sorted non-overlapping ranges, applies each
book's edits from highest index to lowest so earlier positions do not rebase, commits every
selected book atomically, rebuilds source/hash/derived state, and marks affected books dirty
without implicitly linting. No syntax tree, nested token model, or new dependency is introduced.
Core continues to own rule/fix/format semantics; braid owns resident resolution, patch storage,
application, and lifecycle safety. `preview_patch` computes the post-patch token streams for the
handle's books against the same frozen snapshot without mutating anything, so preview and apply
share one snapshot and no consumer reimplements patch application. `to_tokens` is the resident
token projection in the standard `ScopedOutput` envelope — any consumer keeping its own view of
the token stream reads it after a mutation it did not author, without running `lint()`.

`restore_corpus` (§2.2#17) is the atomic warm cold-open: it seeds resident books (exact source
plus decoded semantic tokens) and, per book, an optional cached lint contribution validated with
the same stamps as `prime_lint_cache`. Books whose validation fails are rejected in the
`RestoreReport` and fall back to normal ingest and lint; accepted books require no lex, parse, or
rule execution. Braid still never decodes wire bytes — the composing adapter does (§8.2).

`prime_lint_cache` accepts semantic findings/patches decoded by the composing adapter; Braid does
not depend on wire. The adapter validates wire/catalog versions and checksums before constructing
`LintPrimeInput`; Braid independently accepts matching per-book entries only when source hash,
lint-config fingerprint, and deterministic rule/engine stamp agree. Rejected/missing books remain
dirty and are recomputed by the next `lint()`.
Accepted cached findings and their snapshot-bound patch table become the resident book's current
lint contribution. A package version may participate in the stamp but is not the only proof unless
the release process guarantees every semantic change bumps it.

`diff_baseline` is resident and errors when any requested book lacks a baseline. The exact
`diff_baseline` returns the current core `DiffSkeleton<T>` instantiated with canonical
`OwnedToken`, wrapped only in the same resident `ScopedOutput` envelope used by projections. It
does not invent a second Braid-specific diff model. Existing `UnitId`, `DecisionUnit`, `Slot`, and
merge/revert semantics remain core-owned. Arbitrary pairwise or N-ary inputs remain stateless.

There is no dedicated source-key rename method in v1. The uncommon path-move case is handled by
the next complete corpus seed/replacement; source-key-only changes preserve semantic hashes,
cached lint, and packed `SnapshotId` even though grouped projection metadata changes.

### 5.4 Resident versus stateless operations

The receiver answers where the data comes from. `Braid` is the resident handle type, while
lowercase `braid` is the Rust crate/module or imported package namespace. Instance methods always
operate on the ingested resident corpus; namespace/top-level/static proxies always operate on
caller-supplied external values. Do not add `CorpusScope::External(tokens)`, an external-token
sentinel, or an instance method whose behavior changes between resident mutation and stateless
transformation.

| Intent | Resident Braid API | External/stateless API |
| --- | --- | --- |
| lint | `braid.lint()` — complete resident snapshot | `lint_tokens(tokens, options)` |
| format | `braid.prepare_format_patch(scope, options)` — no implicit mutation | `format(tokens, options) -> Vec<T>` |
| apply result | `braid.apply_patch(handle)` — atomic resident mutation | caller owns returned external tokens |
| diff | `resident.diff_baseline(scope)` | pairwise/N-ary `braid::diff_*` or package-static proxies over arbitrary inputs |
| project/serialize | `braid.to_usfm(scope)` and other typed `to_*` resident projections | `tokens_to_usfm(tokens)` and existing stateless projections |
| dirty/baseline | resident only | not applicable |

Rust consumers may call core directly or use deliberately shallow `braid::lint_tokens`,
`braid::format_tokens`, and diff proxies when the crate provides the one-stop API. The npm package
may expose the equivalent as top-level namespace functions or static `Braid.*` functions; either
is stateless because there is no instance receiver. Pairwise or N-ary diff over arbitrary USFM or
tokens necessarily belongs on that stateless surface. The names and TypeScript signatures keep
the two modes distinct. A scope is always `All | Book(BookId) | Chapter(ChapterTarget)` over
resident data. Operations over `All` return an ordered grouped variant; book/chapter scopes return
a single variant. Rust uses ordered `Vec<SourceOutput<T>>` rather than a map that could obscure
source order. The JS facade materializes that ordered group as a `ReadonlyMap<SourceKey, T>`.

### 5.5 Snapshot types

```rust
pub struct SnapshotId(u64);

pub struct LintSnapshot {
    pub id: SnapshotId,
    pub books: Vec<BookLintSnapshot>,
    pub summary: LintSummary,
}

pub struct BookLintSnapshot {
    pub book: BookId,
    pub source_hash: SourceHash,
    pub tokens: Vec<OwnedToken>,
    pub result: LintResult,
}

pub struct SourceHash(u64);

pub struct LintPrimeInput {
    pub config_fingerprint: LintConfigFingerprint,
    pub engine_stamp: LintEngineStamp,
    pub books: Vec<BookLintPrime>,
}

pub struct BookLintPrime {
    pub book: BookId,
    pub source_hash: SourceHash,
    pub result: LintResult,
    pub patches: Vec<TokenPatch>,
}

pub struct PrimeReport {
    pub accepted: Vec<BookId>,
    pub rejected: Vec<PrimeRejection>,
}

pub struct PrimeRejection {
    pub book: BookId,
    pub reason: PrimeRejectReason,
}

pub enum PrimeRejectReason {
    BookNotResident,
    SourceHashMismatch,
    ConfigFingerprintMismatch,
    EngineStampMismatch,
    InvalidPatch,
}

pub struct CorpusRestoreInput {
    pub config_fingerprint: LintConfigFingerprint,
    pub engine_stamp: LintEngineStamp,
    pub books: Vec<BookRestoreInput>,
}

pub struct BookRestoreInput {
    pub source_key: SourceKey,
    pub book: BookId,
    pub source: String,
    pub tokens: Vec<OwnedToken>,
    pub lint: Option<BookLintPrime>,
}

pub struct RestoreReport {
    pub seeded: Vec<BookId>,
    pub rejected: Vec<PrimeRejection>,
}
```

Actual storage may avoid cloning tokens into each published view, but ordering and meaning are
fixed. `SnapshotId` folds format-independent semantic inputs: ordered `(book, source_hash)` and a
deterministic lint-config/rule-engine stamp. It is not a timestamp and is not the wire version.
Wire writes the id; wire never recomputes it.

### 5.6 Typed errors

Expected failures are explicit enums, not strings or panics:

```rust
pub enum IngestError {
    DuplicateBook { book: BookId, sources: Vec<SourceKey> },
    DuplicateSourceKey { source: SourceKey },
    DuplicateTokenId { book: BookId, id: StableTokenId },
    ChapterNotFound(ChapterTarget),
    AmbiguousChapter { target: ChapterTarget, matches: usize },
    ReplacementLabelMismatch { target: ChapterTarget, found: ChapterLabel },
    InvalidToken(TokenInputError),
    Parse(ParseInputError),
}

pub enum DecodeError {
    Truncated,
    BadMagic,
    UnsupportedVersion { found: u16 },
    UnsupportedFlags { found: u32 },
    InvalidToc,
    InvalidSection,
    InvalidUtf8,
    InvalidDiscriminant,
    OffsetOverflow,
    TooManySids { found: u32 },
    ChecksumMismatch,
    CatalogMismatch,
    SourceLengthMismatch,
    SourceHashMismatch,
}

pub enum PatchError {
    StaleSnapshot { expected: SnapshotId, found: SnapshotId },
    UnknownPatch(PatchId),
    InvalidEditOrder,
    OverlappingEdits,
    OutOfBounds,
    InvalidResult(IngestError),
}

pub enum PrimeError {
    DuplicateBook(BookId),
    InvalidFinding { book: BookId },
    InvalidPatch { book: BookId },
}

pub enum BaselineError {
    Scope(ScopeError),
    MissingBaseline { books: Vec<BookId> },
}
```

Variants may grow as implementation proves distinct recovery actions. Wasm maps expected errors
to generated discriminated error objects inside `ApiResult`; thrown `JsError` is reserved for a
programmer error or violated internal invariant. Neither surface exposes Rust debug dumps as API.

The wasm `RestoreError` (§8.2) is the generated composition union over
`{decode: DecodeError | sourceBinding: SourceBindingError | ingest: IngestError}` — the root
composes wire decode with braid seeding, so its failure classes are those layers' failures.
Per-book stamp rejections are data in `RestoreReport.rejected`, not errors.

## 6. Resident corpus and lifecycle invariants

### 6.1 Corpus shape

```text
Corpus (ordered unique books)
  BookState
    BookId
    current: exact source bytes + owned tokens + ordered ChapterRun[]
    baseline: optional exact source bytes + owned tokens
    current_hash / baseline_hash
    cached_lint: optional native result
    lint_stamp

  ChapterRun
    label: FrontMatter | exact chapter-number token
    token range in current book
```

Chapter lookup metadata may map a label to a small list of ranges. It is derived state, never the
authoritative corpus. A map that stores only one range is forbidden because it would collapse
duplicates.

### 6.2 Validation and atomicity

Every mutation performs all parsing, owned-token construction, structural validation, id
uniqueness checks, exact serialization, and candidate hash construction before touching `Braid`.
On failure, corpus, dirty state, cached results, snapshot id, and prior publication remain exactly
unchanged.

Hash equality selects an exact byte/token equality check. It never proves `Unchanged` alone.
`Unchanged` preserves the existing clean/dirty condition and cached publication exactly.

### 6.3 State table

| Event | Resident corpus | Cached lint | Publication |
| --- | --- | --- | --- |
| rejected mutation | unchanged | unchanged | valid as before |
| semantic no-op | unchanged | unchanged | valid as before |
| changed chapter | book replaced atomically | that book stale | prior complete snapshot stale |
| changed book | book replaced/inserted | that book stale | prior complete snapshot stale |
| validated restore (§2.2#17) | seeded books replaced atomically | accepted books primed; rejected books stale | prior complete snapshot stale |
| removed book | book removed | contribution removed | prior complete snapshot stale |
| changed config | corpus unchanged | affected/all books stale | prior complete snapshot stale |
| lint success | unchanged | dirty books replaced atomically | new complete semantic snapshot |
| lint failure | mutations remain | no partial result committed | no result claims new corpus |
| wire pack success | unchanged | unchanged | adapter may publish bytes/id |
| wire pack failure | unchanged | semantic snapshot current | adapter publishes no new bytes/id |

Dirty work is derived from authoritative stamps, not consumed from a destructive queue. Retrying
after failure is safe. Rust's `&mut Braid` serializes transitions; concurrent mutation, lint,
cancellation, and background workers are out of scope.

### 6.4 Lint recomputation

For each dirty book, call the existing whole-book `lint_tokens(..., LintScope::Book)` path. Keep
its canonical sort, suppressions, and summary semantics. Unchanged books reuse cached native
results. Assemble books in corpus order and fold a complete summary.

Do not call `LintScope::Chapter` and then manually synthesize whole-book families. Do not copy
private rule groupings from `lint_impl.rs` into braid. A later incremental-compute plan must first
expose a core-owned typed seam and prove parity over the full oracle.

## 7. Packed container specification

Little-endian. Decoders validate every multiplication, addition, alignment, range, UTF-8 slice,
count, and discriminant before constructing public views. Rust encoding **and decoding** are
authoritative; generated `wire-schema` constants and golden vectors are conformance/debug
artifacts, not a JS decoder or hash implementation.

The codec is public **native** API, not wasm plumbing (§2.2#18): a native host composes it exactly
as the wasm crate does. Normative at the semantic level (exact names frozen in Phase 0 step 3):

```rust
pub enum CorpusSectionTokens<'a> {
    /// Parsed tokens, which carry their own spans into `source`.
    Parsed { source: &'a str, tokens: &'a [Token<'a>] },
    /// Spanless owned tokens. Wire serializes them and derives spans from that
    /// same emission pass; the derived source is returned alongside the bytes,
    /// because it — not any file on disk — is what the section is bound to.
    Owned { tokens: &'a [OwnedToken] },
}

pub struct CorpusSectionInput<'a> {
    pub book: BookId,
    pub tokens: CorpusSectionTokens<'a>,
    pub findings: Option<&'a LintResult>,
}

pub fn encode_corpus(
    snapshot_id: u64,
    sections: &[CorpusSectionInput<'_>],
) -> Result<EncodedCorpus, EncodeError>;

/// The bytes plus, per book, the exact source the section's spans and hash are
/// bound to. For a `Parsed` section this is the caller's own `source`; for an
/// `Owned` section it is the serialization wire produced.
pub struct EncodedCorpus {
    pub bytes: Vec<u8>,
    pub sources: Vec<(BookId, String)>,
}

pub fn decode_borrowed<'wire, 'source>(
    wire: &'wire [u8],
    source: &'source str,
) -> Result<DecodedTokens<'wire, 'source>, DecodeError>;
```

Respecified by the 2026-07-28 owned-encoding adjudication in `./phase0-freeze.md`. The original
signature promised that a `&[OwnedToken]` plus a `source` could be encoded directly, which the
implementation showed is impossible: the token section's span columns are required, `OwnedToken` is
spanless by design, and the concatenation of `token.source()` is **not** the source — an attribute
list is emitted at a position of its own, not next to its marker. The owned path is therefore
explicitly serialize → `(source, spans)` → encode, using core's
`tokens_to_usfm_reconstruct_spanned`; the parsed-borrowed path keeps its spans and needs no
serialization. Owned tokens stay spanless as a type: spans are a transient encode-time artifact, so
there is no mid-session span state to go stale.

Serializing a parse-origin owned stream is byte-lossless — an owned token remembers the distance from
its own end to its attribute list (`./../attribute-position-fidelity.md`), so the derived source
equals the original file. An `Owned` section is nonetheless bound to the source wire *derived*, which
for edited tokens is a new document; the returned `sources` are therefore the authoritative pairing,
not any file on disk.

`snapshot_id` arrives as a plain `u64` — the composing adapter converts braid's `SnapshotId`;
wire writes the id and never recomputes it. Encoders refusing (over-wide SIDs §7.4, unknown
payload, a span that cannot bind to the section's source) return typed `EncodeError`, never
truncate.

### 7.1 Container header (48 bytes)

Amended from 32 to 48 bytes by the 2026-07-28 layout adjudication in `./phase0-freeze.md`, which
allocated bytes for `snapshot_id` (previously promised by the `encode_corpus` signature with no
field to hold it).

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | magic | `[u8; 4]` | ASCII `uson` |
| 4 | format version | `u16` | layout compatibility |
| 6 | header length | `u16` | exactly 48 in v1 |
| 8 | flags | `u32` | unknown set bits reject |
| 12 | section count | `u32` | bounds-checked before TOC allocation |
| 16 | TOC offset | `u64` | absolute; 48 in canonical v1 encoding |
| 24 | integrity checksum | `u64` | xxhash3-64 over canonical bytes with this field zero; zero means omitted only when the API explicitly requests unchecked transient output |
| 32 | snapshot id | `u64` | the `encode_corpus` argument, written verbatim and never recomputed |
| 40 | reserved | `[u8; 8]` = zero | nonzero rejects, like an unknown flag bit |

Top-level sections begin at 16-byte-aligned absolute offsets and may appear in any TOC order.
Canonical encoders emit ordered token sections followed by corresponding finding sections in
corpus order.

### 7.2 TOC entry (32 bytes)

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | kind | `u8` | token or finding stable discriminant |
| 1 | book | `[u8; 3]` | canonical `BookId` |
| 4 | section version | `u16` | v1 = 1 |
| 6 | flags | `u16` | kind-specific; unknown bits reject |
| 8 | absolute offset | `u64` | 16-byte aligned, non-overlapping |
| 16 | byte length | `u64` | includes section header/directory/payload |
| 24 | source hash | `u64` | exact serialized current-book USFM |

Exactly one token section exists per book. A finding section, when present, references the token
section with the same book and source hash. Duplicate books are already forbidden by corpus
validation, so no occurrence field is needed. The complete-corpus encoder refuses duplicate
`BookId` sections, and the decoder rejects a container whose TOC contains them as non-canonical.
This does not make either source file or a per-book sidecar unparsable; it rejects only the
ambiguous complete-corpus assembly. `SourceKey` is not a TOC field.

### 7.3 Section header and field directory

Every section begins with a 64-byte header. Amended from 48 to 64 bytes by the 2026-07-28 layout
adjudication in `./phase0-freeze.md`, which allocated bytes for the exact source length
`decode_borrowed` binds external bytes against and for the §7.7 marker-catalog stamp.

| offset | field | type |
| ---: | --- | --- |
| 0 | magic | `[u8; 4]` = `usos` |
| 4 | format version | `u16` |
| 6 | rules version | `u16` (`0` for token sections) |
| 8 | kind | `u8` |
| 9 | flags | `u8` |
| 10 | book | `[u8; 3]` |
| 13 | reserved | `[u8; 3]` = zero |
| 16 | record count | `u32` |
| 20 | directory count | `u16` |
| 22 | directory entry size | `u16` = 16 |
| 24 | source hash | `u64` |
| 32 | section byte length | `u64` |
| 40 | integrity checksum | `u64` or zero |
| 48 | source byte length | `u64` |
| 56 | marker catalog stamp | `u64` |

Each 16-byte field entry is `{field_id:u16, element_width:u8, flags:u8,
section_relative_offset:u32, byte_len:u32, count:u32}`. Required fields occur exactly once;
unknown required field ids reject; unknown optional fields may be skipped. Field payloads use the
alignment required by their element width. This directory—not handwritten arithmetic scattered
through decoders—is the only route to SoA columns.

### 7.4 Token section

The v1 decoder takes both artifacts explicitly:

```rust
pub fn decode_borrowed<'wire, 'source>(
    wire: &'wire [u8],
    source: &'source str,
) -> Result<DecodedTokens<'wire, 'source>, DecodeError>;
```

Before exposing tokens it verifies the supplied source byte length and xxhash3 against the section
header. A hash match does not replace bounds/UTF-8 validation. Required v1 fields are:

- `kind[N]:u8`;
- `span_start[N]:u32` and `span_end[N]:u32`, both into the caller-supplied source bytes;
- `token_id_index[N]:u32` into a UTF-8 string dictionary — present only when the section's
  `positional_ids` flag is clear. When set (cold parse: every id is `{book}-{index}`), the column
  and dictionary are omitted and the decoder synthesizes the ids from book + row; Gate 0E measured
  the dictionary at 31–41% of section bytes and 100% redundant for parsed books. Live token
  sections with opaque caller ids keep it;
- `sid_index[N]:u16` into the packed SID dictionary, with `0xffff = None`. A book with more than
  65,535 distinct SIDs refuses to encode with a typed error and decoders reject the sentinel
  collision via `DecodeError::TooManySids` — real scripture peaks at 2,612/book (Gate 0E), so this
  is a loud-refusal path for structurally legal but non-scriptural inputs, not a width increase;
- the packed SID dictionary itself (token field 12, fixed 8-byte §7.5 records) that
  `sid_index` indexes into; every non-sentinel index must be below its entry count;
- `marker_descriptor_index[N]:u16`, with `0xffff = None`;
- sparse number records keyed by `token_idx`, with `u32` number fields — raw source numbers reach
  999,999 in adversarial fixtures; core `Sid` saturates at 65,535 but the token payload must not;
- sparse book-code records keyed by `token_idx`;
- sparse attribute-list records keyed by `token_idx`, including verbatim attribute-source spans;
- string dictionary offsets/data for token ids and non-catalog strings;
- marker descriptors sufficient to reproduce unknown markers and all semantic metadata required
  by the core traits.

The production codec begins from the spike but does **not** inherit its `assert!`, unchecked
indexing, private raw marker index, duplicated enum-order arrays, or 10-byte SID record. Supplying
the exact source separately is normative. Stable schema discriminants are explicit. Marker
catalog ordinals are usable only with a deterministic catalog/schema stamp; unknown marker text
is recovered from validated source spans and required metadata remains in the section.

`token_idx` is a `u32` position within this book's token section. It is refreshed whenever the
book snapshot changes and never crosses section/container versions. It is suitable for lookup and
highlighting, not identity.

### 7.5 Packed SID

The wire dictionary stores exactly eight bytes per anchor:

```text
book[3] | chapter:u16 LE | verse:u16 LE | delta_and_fidelity:u8
```

The high bit is `AnchorOnly`; low seven bits hold bridge delta `0..=127`. `Exact` means a canonical
single verse or simple bridge with delta at most 127. Sequences, suffixes, malformed designators,
and bridges wider than 127 store the first canonical anchor with `AnchorOnly`. The exact token
source remains authoritative and lossless.

The fidelity bit is derived from the number token's **source text**, never from `Sid` or
`NumberRangeKind` alone: a sequence SID (`\v 1,3`) is byte-identical to a single-verse SID in core
`Sid`, and a suffixed verse (`\v 1a`) is identical to an unsuffixed one in both `Sid` and
`NumberRangeKind`, so an encoder written against the semantic payload alone would silently mark
them `Exact` (Gate 0D finding D2).

This is a byte codec, not a promise that `repr(Rust)` matches the bytes. Core `Sid` keeps its
current `u8` delta and semantics. Conversion is explicit and tested at the boundary.

### 7.6 Finding record and sidecars

The common row remains 16 bytes:

| offset | field | type | notes |
| ---: | --- | --- | --- |
| 0 | token index | `u32` | section-local; `u32::MAX` for anchor-only finding |
| 4 | offset in token | `u16` | byte offset; overflow sidecar when flagged |
| 6 | length | `u16` | `0` = whole token; overflow sidecar when flagged |
| 8 | chapter | `u16` | canonical anchor |
| 10 | verse | `u16` | canonical anchor |
| 12 | range end | `u8` | `0` none; fidelity in flags |
| 13 | rule code | `u8` | explicit stable append-only code |
| 14 | flags | `u8` | exact/anchor-only, no-anchor, range, related, payload, fix, overflow |
| 15 | reserved | `u8` | zero in v1 |

The `no-anchor` flag is required because `(chapter 0, verse 0)` is a **legal** chapter-scope SID
(book front matter, produced for `\id` and pre-`\c` tokens), so it cannot double as "finding has no
SID at all" (e.g. `missing-id-marker`; Gate 0D finding D3). Finding records are stored in canonical
order — position-keyed per §2.2#15 — so Rust materialization and the pure-JS reconciler preserve
canonical order by reading/object order, without any comparator.

Record-aligned optional columns use the finding row index, so the common row needs no tiny payload
pointer:

- `related_token_idx[N]:u32` plus related token-relative span where present;
- `overflow_span[N]:{offset:u32,len:u32}` for values that do not fit `u16`;
- `message_payload_idx[N]:u32` into typed per-code argument payloads;
- `marker_string_idx[N]:u32` when marker cannot be recovered unambiguously;
- `patch_id[N]:u32` into the snapshot-bound braid patch table; `u32::MAX` means no patch;
- a packed patch table of flat, sorted, non-overlapping insert/replace/delete edits, including
  replacement token templates needed to restore braid state from a persisted sidecar.

Severity, category, issue type, ICU template, and source-dependency metadata come from the
generated stable rule catalog. Rust wire materialization supplies the semantic DTO fields;
applications may render/localize from the template and message parameters. No JS rule list,
packed parser, or XXH3 implementation is handwritten.

Every current `LintIssue` must round-trip semantically. If Gate 0 finds a field that cannot be
derived or encoded by these sidecars, the Phase B/C full finding-and-patch gate amends the schema
before coding; it does not drop the field.

### 7.7 Stable rule and schema policy

- Rule codes are explicit fixed integers, append-only, never enum ordinals.
- Removed rules leave tombstones; codes are never reused.
- Gate 0 must prove the v1 `u8` capacity. Crossing 255 requires a format-version change, not
  truncation.
- Core's `EnabledCodes` `u64` bitmask caps `LintCode` at 64 variants (32 in use as of Gate 0E), so
  the wire `u8` can never be the first ceiling to overflow; crossing 64 requires widening core's
  mask (e.g. `u128`) before any wire concern arises.
- `format_version` changes for byte-layout interpretation.
- `rules_version` changes for catalog additions/removals/payload schema changes.
- A deterministic marker-catalog stamp invalidates sections whose marker ordinals no longer mean
  the same thing.
- Golden vectors cover every token kind, every sidecar kind, unknown markers, malformed buffers,
  and both byte orders' rejection/interpretation policy (v1 only emits little-endian).

## 8. Official JS and wasm surfaces

### 8.1 Wasm-backed npm materialization and pure-JS reconciliation

```ts
export type SourceCorpus = ReadonlyMap<BookCode, Uint8Array>;

export type MaterializedBook = Readonly<{
  book: BookCode;
  tokens: readonly Token[];
  findings: readonly Finding[];
}>;

export type MaterializedSnapshot = Readonly<{
  id: bigint;
  books: ReadonlyMap<BookCode, MaterializedBook>;
}>;

export type ApiResult<T, E> =
  | Readonly<{ ok: true; value: T }>
  | Readonly<{ ok: false; error: E }>;

export type SourceBindingError =
  | Readonly<{ kind: "invalidBookKey"; key: string }>
  | Readonly<{ kind: "missingSource"; book: BookCode }>
  | Readonly<{ kind: "extraSource"; book: BookCode }>
  | Readonly<{
      kind: "sourceLengthMismatch";
      book: BookCode;
      expected: number;
      found: number;
    }>
  | Readonly<{ kind: "sourceHashMismatch"; book: BookCode }>;

export type MaterializeError =
  | Readonly<{ kind: "decode"; error: DecodeError }>
  | Readonly<{ kind: "sourceBinding"; error: SourceBindingError }>;

/** wasm export backed directly by `usfm_onion_wire`. */
export function decodeTokens(
  packed: Uint8Array,
  book: BookCode,
  source: Uint8Array,
): ApiResult<DecodedTokenBook, MaterializeError>;
/** wasm export backed directly by `usfm_onion_wire`. */
export function materialize(
  sources: SourceCorpus,
  packed: Uint8Array,
): ApiResult<MaterializedSnapshot, MaterializeError>;
/** Pure JS: both arguments are already-validated semantic finding DTOs. */
export function reconcileFindings(
  previous: MaterializedFindings | undefined,
  next: MaterializedFindings,
): MaterializedFindings;
```

`decodeTokens` and `materialize` are synchronous wasm exports, backed directly by
`usfm_onion_wire`'s checked Rust decode. Rust alone validates headers, TOC/section ranges,
checksums, discriminants, exact source length/hash binding, and semantic token/finding
materialization; no independent production JS binary parser or JS XXH3 exists. The existing frozen
typed errors cross the same boundary as ordinary DTOs. A typed failure is the deliberate signal for
the application to use normal Braid USFM ingest/parse instead.

Native hosts call the wire crate directly. If the Phase 0 API ledger retains the promised
`decode_view` representation API, it remains a public native Rust view over checked bytes; it is
not an npm/raw-`Uint8Array` parser and is deliberately absent from this TypeScript surface.

`materialize` is the cold-open object API. Its canonical input is one source byte array per unique
`BookCode`; an ergonomic `Readonly<Partial<Record<BookCode, Uint8Array>>>` overload may normalize
to the same map after validating every key. The wasm wire path pairs keys directly with the
three-byte book ids in the packed TOC, verifies exact source length/hash, and creates plain JS
token/finding objects grouped by `BookCode` without lexing or parsing. Paths are not part of this
API or the packed format. The application retains its separate `SourceKey/path -> BookCode`
manifest association. On success it uses those regular semantic tokens/findings or seeds Braid
through `restoreCorpus`; on a typed failure it falls back to normal Braid USFM ingest/parse.

The generated `./wire-schema` JS constants exist for generated-declaration conformance, debugging,
and golden-vector inspection. They are never called by a production packed decoder, checksum/hash
validator, source binder, or materializer.

`reconcileFindings` is the pure-JS warm-update API. It receives only already-validated semantic
finding DTOs from wasm materialization or an equivalent native wire host; it neither accepts nor
parses packed bytes. It keys findings by book plus stable token/finding identity, reuses objects
only when every public field is equal, drops removed objects, inserts new objects in canonical
order, and never mutates caller-owned objects in place. The v1 behavior is immutable object reuse.

Finding identity is a deterministic tuple derived from rule code, primary stable token id or SID
anchor, token-relative range, related address, and deterministic same-key occurrence. Message
text and fix payload are excluded from identity but included in equality, matching the editor's
current finding model.

Malformed packed input returns one typed wasm `ApiResult` error before returning a partial result.
Decode errors describe packed corruption/version failures; source-binding errors name the affected
book and distinguish invalid/extra/missing input and source length/hash mismatch. No API accepts a
bare `ArrayBuffer` plus unchecked offsets; callers pass `Uint8Array`/views and Rust honors their
byte offset/length.

### 8.2 Wasm `Braid`

```ts
export class Braid {
  constructor(config: BraidConfig);
  replaceCorpus(input: CorpusInput): ApiResult<MutationEffect, IngestError>;
  restoreCorpus(
    sources: SourceCorpus,
    packed: Uint8Array,
  ): ApiResult<RestoreReport, RestoreError>;
  updateBook(input: BookInput): ApiResult<MutationEffect, IngestError>;
  updateChapter(
    target: ChapterTarget,
    input: ChapterInput,
  ): ApiResult<MutationEffect, IngestError>;
  removeBook(book: BookCode): MutationEffect;
  removeChapter(target: ChapterTarget): ApiResult<MutationEffect, ScopeError>;
  clear(): MutationEffect;
  updateConfig(config: BraidConfig): MutationEffect;
  primeLintCache(input: LintPrimeInput): ApiResult<PrimeReport, PrimeError>;
  lint(): ApiResult<Uint8Array, LintError>;
  prepareFormatPatch(
    scope: CorpusScope,
    options: FormatOptions,
  ): ApiResult<PatchPreparation, FormatError>;
  applyPatch(handle: PatchHandle): ApiResult<MutationEffect, PatchError>;
  setBaseline(input: BookInput): ApiResult<MutationEffect, IngestError>;
  clearBaseline(book: BookCode): MutationEffect;
  diffBaseline(
    scope: CorpusScope,
  ): ApiResult<ScopedOutput<DiffSkeleton<Token>>, BaselineError>;
  isDirty(scope: CorpusScope): ApiResult<boolean, ScopeError>;
  toTokens(scope: CorpusScope): ApiResult<ScopedOutput<readonly Token[]>, ScopeError>;
  previewPatch(handle: PatchHandle): ApiResult<ScopedOutput<readonly Token[]>, PatchError>;
  toUsfm(scope: CorpusScope): ApiResult<ScopedOutput<string>, ScopeError>;
  toUsx(scope: CorpusScope): ApiResult<ScopedOutput<string>, ProjectionError>;
  toUsj(scope: CorpusScope): ApiResult<ScopedOutput<UsjDocument>, ProjectionError>;
  toHtml(
    scope: CorpusScope,
    options: HtmlOptions,
  ): ApiResult<ScopedOutput<string>, ProjectionError>;
  expectedSnapshotId(): bigint;
}
```

Generated input types are discriminated unions (`{kind:"usfm",...}` / `{kind:"tokens",...}`),
not optional bags. Methods are synchronous. Expected domain/validation failures use a generated
tagged `ApiResult<T, E> = {ok:true,value:T} | {ok:false,error:E}` with discriminated typed error
enums. Exceptions are reserved for programmer errors and violated internal invariants. Returned
`Uint8Array` is a fresh owned wasm-to-JS view suitable for an
immediate copying/transfer policy documented by the package; tests prove it remains valid for the
documented lifetime.

Existing stateless exports remain through the migration release unless the explicit API ledger
marks one for deletion. Registry calls re-export core. Registry queries are synchronous **after**
module initialization (an explicit async `init()` on the `web` target); supplying marker facts
without wasm initialization is not a braid promise — the retained wasm-free `js/token-sids.js` is
a deliberate, conformance-tested exception, not the default. `restoreCorpus` is composed here in
the root: it decodes/validates packed bytes via wire, binds the supplied sources, and seeds braid
via `restore_corpus` (§2.2#17) — braid itself never sees wire bytes. The wasm crate defines no duplicate Token,
LintIssue, LintCode, or marker enums once wire DTO migration completes.

The top-level stateless exports accept caller-owned strings/tokens and never read or mutate a
`Braid` instance. The `Braid` methods accept resident scopes and never accept arbitrary external
tokens. This separation is deliberate even where both paths delegate to the same core function.
For generated TypeScript, `ScopedOutput<T>` is `T | ReadonlyMap<SourceKey, T>`: `Book` and
`Chapter` scopes return the value directly, while `All` returns an insertion-ordered map keyed by
the caller's current source binding. Packed lint/reconcile remains keyed by `BookCode`, because
portable wire data intentionally contains no `SourceKey`.

### 8.3 Native/Tauri boundary

The `braid` crate is synchronous Rust. A Tauri command/session may place it behind async IPC for
bulk native execution, but that is an application adapter, not a second braid lifecycle. The
native resident host is a **first-class v1 hosting path** (§2.2#18, adjudicated 2026-07-27,
superseding the earlier defer-native wording): the editor's desktop process hosts a resident
native `Braid` behind Tauri commands that call the same lifecycle methods and return packed bytes
as JavaScript `ArrayBuffer`s. Braid/wire ship what that host needs — the plain `serde` feature,
the public native codec (§7), and native round-trip contract tests — while the Tauri command host
itself remains editor-owned app code; braid ships no Tauri dependency. Phase F parity transcripts
run against both the web wasm host and the native host. No plan step assumes native Rayon
automatically predicts webview wasm performance.

## 9. Hashing, dirty state, baseline, and serialization

Each resident book carries its `LineEnding` (§2.2#16): detected for USFM ingest, declared for
token ingest, inherited by chapter updates. `to_usfm` and every patch/backup-embedded USFM emit
newline tokens with the stored ending through the core reconstruct emitter's optional `LineEnding`
override — braid never rewrites newline tokens itself. Mixed-EOL USFM input keeps its exact bytes
until first edit. The ending applies only to newline token emission; no other token source or
attribute trivia is touched.

`SourceHash` is xxhash3-64 over the exact bytes returned by lossless `to_usfm` for one book —
i.e. over the EOL-applied emitted bytes, so hashes, dirty state, and warm-cache validation match
what is actually saved.
Chapter hashes may accelerate candidate rebuilding but are private and must fold into/rebuild the
authoritative book bytes/hash on mutation. Hash equality is always confirmed by exact equality
before returning `Unchanged` or reusing persisted semantics.

Baseline state is per book and optional. `set_baseline` validates through the same input path as
current data. Dirty means exact current serialized bytes differ from exact baseline bytes. A
missing baseline makes `is_dirty` return `true`, because there is no saved equality proof;
`diff_baseline` returns typed `MissingBaseline` with every requested book lacking one, because
there is no meaningful comparison to synthesize.

Chapter-scope dirty (Gate 0F amendment C): `is_dirty(Chapter(target))` compares the chapter run's
exact bytes to the baseline run with the same label. A baseline run that is missing for that label
is dirty (consistent with missing book baselines); duplicate labels on either side make the scope
ambiguous and return the typed `AmbiguousChapter`-shaped `ScopeError`, consistent with
`update_chapter`.

Serialization is token-to-USFM lossless and never re-lexes token input. A changed mutation that
does not rebuild authoritative bytes/hash is a stop clause. Formatting remains an explicit core
operation; `to_usfm` does not format.

Application cache validation requires: supported format/rules/catalog versions, checksum when
present, matching ordered book/source hashes, matching lint-config fingerprint, deterministic
rule/engine stamp, and matching `SnapshotId` for semantic findings/patches. Validated semantic
entries may prime Braid per book; invalid/missing entries fall back to normal lint. Braid does not
read or write cache files and does not decode wire bytes itself. The current manifest supplies
resident `SourceKey`s independently, so a path move does not invalidate otherwise matching cache
content.

## 10. Execution phases and commit gates

Each numbered step is one reviewable commit unless a smaller split is needed. Run the normal test
suite and lint oracle at every commit. Run wasm/package gates whenever their surface changes.

### Phase 0 — evidence and contract freeze

1. Complete Gate 0 and append results.
2. Verify implementation-facing contracts reflect the accepted §2.2 adjudications; apply only
   evidence-driven fixed-width corrections through a recorded plan amendment.
3. Freeze v1 stable discriminants, error names, and the public API ledger.

Gate: all preconditions pass; no production behavior changed; the plan contains no unresolved
“or sibling”, “implementation detail”, or pseudocode ambiguity on a Phase A/B boundary.

### Phase A — `usfm_onion_wire` token codec

1. Create the crate and one-way dependencies; move DTO types without changing declarations.
2. Add schema constants, typed errors, checked reader/writer primitives, header/TOC/field
   directory validation, and malformed-input tests.
3. Promote token columns, dictionaries, and sidecars from the spike. Replace private raw
   marker-index coupling with the accepted catalog-stamp design; keep source bytes external.
4. Implement serial encode and validated borrowed decode over `(wire, exact_source)` with mandatory
   length/hash binding.
5. Add native `decode_par` only behind the existing non-wasm/Rayon configuration and only if the
   spike equivalence/performance evidence still supports it. Serial remains the semantic oracle.
6. Generate JS schema constants and token golden vectors from one Rust-owned schema source.

Gate: for every corpus, decoded tokens reproduce `parse(source).tokens` in all public semantic
fields and losslessly serialize to identical supplied source bytes; wrong source length/hash
rejects; serial/parallel decode agree; malformed buffers fail without panic; all existing oracles
remain byte-identical.

### Phase B — finding wire and wasm semantic materialization

1. Freeze stable `LintCode` numbers and generated catalog metadata.
2. Implement the 16-byte common record and record-aligned sidecars from the Gate 0 payload ledger.
3. Implement Rust semantic round-trip tests for finding fields which do not require a resident
   snapshot-bound patch table.
4. Export wasm `decodeTokens(packed, book, source)` and `materialize(sources, packed)` backed
   directly by wire's checked decode, source binding, and semantic materialization. Generate
   `./wire-schema` constants only as conformance/debug artifacts; do not add a JS decoder or JS
   XXH3.
5. Implement pure-JS `reconcileFindings(previous, next)` over already-validated semantic finding
   DTOs and deterministic finding identity. It never accepts packed bytes.
6. Keep `TokenFix` resolution and packed snapshot-bound patch records explicitly pending for
   Phase C, where `SnapshotId`, `PatchId`, residency, and patch storage exist.

Gate: wire and wasm produce equal materialized snapshots for the implemented no-patch finding
fields; unchanged JS objects are reused; one-of-N changes, insertions, deletions, reorder,
token-index rebase, same-key duplicates, and overflow spans behave deterministically; malformed
buffers return the same frozen typed classes through wasm. This is **not** the final all-fields
finding gate: Phase B cannot be called complete as a full finding/patch round-trip while fixes are
absent. The Phase C full finding-and-patch gate below must close before Phase D publication.

### Phase C — `braid` residency floor

1. Land the separately blessed core declared-book lint context and mismatch rule from Gate 0;
   keep no-context stateless lint behavior unchanged. In the same step, land the §2.2#15
   canonical-order re-key (token position replaces span as the primary sort key) with its
   oracle-neutrality verification; any oracle difference stops for separate adjudication.
2. Add the crate, strict input/selector/error types, and candidate validation.
3. Implement ordered unique declared books and duplicate-preserving chapter-run metadata.
4. Implement `replace_corpus`, `update_book`, `update_chapter`, removal, clear, config updates, and
   exact `MutationEffect` semantics.
5. Implement authoritative exact source bytes (EOL-applied per §2.2#16, including the core
   reconstruct emitter's optional `LineEnding` override), per-book hashes, snapshot identity, and
   dirty-book stamps.
6. Implement explicit whole-dirty-book `lint()` and complete native snapshots, passing each
   caller-declared `BookId` into core lint context.
7. Resolve every core `TokenFix` against the resident snapshot into Braid-owned flat patch-table
   entries; implement snapshot-bound patch lookup/application with reverse-ordered flat edits.
   The wasm composition adapter combines the resulting semantic findings/patches with the wire
   finding codec, without adding a braid→wire dependency.
8. Close the full finding-and-patch gate: every current finding field, every `TokenFix` variant,
   localized message arguments, patch id, and packed patch edit/template round-trip semantically
   through Rust wire and wasm materialization; stale/unknown/overlap/out-of-bounds patch paths are
   typed. This gate is required before Phase D; it is not retroactively claimed by Phase B.
9. Implement `to_tokens`, `preview_patch`, and the semantic `restore_corpus` seed path
   (§2.2#17; the public wasm `Braid.restoreCorpus` composition arrives in Phase F).

Gate: rejected mutations are atomic; no-ops preserve cache/publication; repeated mutations
coalesce; resident lint equals stateless whole-book core lint in content and order; duplicate and
out-of-order chapters are retained; ambiguous chapter operations error; duplicate token ids
error; retry after injected lint failure is safe; the full all-fields finding-and-patch gate above
has passed.

### Phase D — packed braid publication

1. Compose braid semantic snapshots with wire encoding without moving codec knowledge into braid.
2. Add token/finding section pairing, source-hash validation, `SnapshotId`, and complete container
   assembly.
3. Add per-book sidecar reuse helpers for application caches, without adding IO or embedding the
   external USFM bytes.

Gate: clean lint performs no core rule work; a one-chapter edit recomputes one whole book and
reuses untouched book sections/results; decoded complete publication equals the native snapshot;
corrupt/mismatched cached sections are rejected, not partially adopted. Phase D may not start
until Phase C has closed the full semantic finding/patch round-trip gate.

### Phase E — baseline, diff, dirty, and serialization

1. Implement baseline mutation and missing-baseline policy.
2. Implement exact `is_dirty` and scoped `to_usfm`.
3. Implement `prepare_format_patch(scope, options)` over resident `All | Book | Chapter`,
   producing one atomic snapshot-bound flat patch handle and proxying existing core format
   semantics without redesigning them.
4. Implement resident baseline diff as
   `ScopedOutput<DiffSkeleton<OwnedToken>>`, directly reusing current core diff/merge/revert
   semantics; arbitrary-token diff remains a top-level core/package function.
5. Keep diff object-shaped in v1. Any later packed diff representation is a separate schema and
   must not overload the findings record by coincidence.

Gate: save bytes are lossless; dirty matches exact serialized equality; baseline errors are
typed; existing diff/revert/merge tests and editor use cases remain equivalent.

### Phase F — wasm package and editor adoption

1. Split the wasm module by responsibility and expose the synchronous `Braid` class.
2. Move/delete DTO duplication only after generated declarations and all stateless exports pass
   the API ledger.
3. Ship the wasm-backed `decodeTokens`/`materialize` exports and pure-JS
   `reconcileFindings(previous, next)` for bundler and web builds, plus the composed
   `restoreCorpus(sources, packed)` on wasm `Braid` (§2.2#17). Do not ship a JS packed decoder,
   JS XXH3, or npm `decodeView` raw-buffer parser.
4. In the editor, make every token id book-wide unique, including linebreak/synthetic tokens; the
   persistent Lexical NodeState investigation is tracked separately in
   `../candidates/editor-persistent-linebreak-token-id.md`.
5. Seed braid from complete ordered working files; route complete chapter updates and explicit
   debounced lint through it.
6. Replace `WorkspaceMirror` onion state, `AnalyzeScope` lint batching, platform-specific onion
   DTOs, and app-owned aliases only after side-by-side parity passes.
7. Keep the editor as coordinator for sous-chef, scheduling, files, crash recovery, and network.
8. Remove old glue in one final narrow commit and regenerate declarations/packages.

Gate: the web wasm host and the native Tauri command host pass identical lifecycle transcripts
(§2.2#18); initial load/edit/undo/redo/format/fix/save/reopen findings match; no async coloring
enters the interactive caller; the published package imports and initialises inside a module
worker and a packed `Uint8Array` survives transfer; package import/golden tests pass; removed glue
has no remaining callers; wasm size and warm-path measurements are recorded without redefining
correctness — including `materialize` source-binding cost measured separately for the desktop
host, whose current loader avoids copying book bytes into JS (Gate 0F P3b).

## 11. Test inventory

### 11.1 Input and corpus tests

- ordered books retain caller order;
- duplicate book ids reject atomically with all colliding source keys;
- duplicate source keys reject atomically;
- unique declared `1CO`/`2CO` inputs may temporarily both contain `\\id 2CO`; both remain resident,
  receive distinct three-byte packed book ids, and the `1CO` source reports `BookIdMismatch`;
- invalid source `\\id` remains resident and reports `InvalidBookCode`;
- out-of-order chapter labels retain source order;
- duplicate and reopened chapter labels retain all runs;
- unique chapter update succeeds;
- missing/ambiguous chapter update/remove returns the exact typed error;
- chapter insertion/removal/reorder uses whole-book replacement;
- USFM expected-book mismatch rejects;
- malformed input does not partially mutate;
- duplicate stable token id rejects, including linebreak-shaped fixtures;
- identical replacement returns `Unchanged` after exact equality confirmation;
- hash collision injection cannot produce a false no-op.
- a complete reseed whose only change is `SourceKey` changes grouped projection metadata but
  preserves semantic hashes, cached lint, and packed snapshot identity.

### 11.2 Lifecycle tests

- empty braid lint publishes empty snapshot;
- mutation does not lint;
- several mutations coalesce to latest state;
- one chapter marks exactly one book dirty;
- config no-op preserves publication;
- changed config invalidates the correct scope (all books in v1 unless core proves narrower);
- lint success commits all dirty book results atomically;
- injected failure commits none and retry succeeds;
- clean lint reuses the semantic snapshot;
- removal drops cached findings from the complete answer;
- snapshot id changes iff semantic inputs/config/engine stamp change;
- validated restore seeds books and lint without lex/parse/rule execution; per-book stamp
  rejection falls back to normal ingest+lint on the next `lint()`; a failed restore is atomic.

### 11.3 Wire token tests

- every token kind and legal payload;
- unknown marker and invalid book code;
- attributes including default shorthand, escaped values, empty attr list, and verbatim source;
- token/string/SID dictionaries at zero, one, maximum measured, and limit boundaries;
- external source length/hash mismatch, including same-length wrong bytes;
- UTF-8 multi-byte spans and invalid boundary rejection;
- truncated/overlap/misaligned/overflow/duplicate/missing field errors;
- unknown flags/required field/discriminants reject;
- unknown optional field skips;
- checksum and source-hash mismatch;
- borrowed decode remains valid for the documented buffer lifetime;
- serial/parallel equality;
- native (no-wasm) round-trip contract: encode → decode → semantic equality, and serde-JSON
  round-trip of the braid lifecycle types under the plain `serde` feature (§2.2#18).

### 11.4 Finding and reconcile tests

- wasm `decodeTokens(packed, book, source)` and cold `materialize(sources, packed)` reconstruct
  tokens and findings without lex/parse, using Rust wire rather than a JS binary parser;
- materialization binds portable sections directly from unique `BookCode` keys and preserves
  canonical packed corpus order;
- another application with different paths but the same `BookCode -> bytes` inputs produces equal
  semantic objects;
- invalid book keys, missing or extra source books, duplicate/non-canonical packed book sections,
  and wrong source length/hash return typed errors without partial objects;
- every rule code and payload-ledger shape;
- the **Phase C full finding-and-patch gate** resolves all three current `TokenFix` variants to
  flat patch entries, then proves snapshot-bound patch id success, stale rejection, unknown id,
  reverse application, overlap, and out-of-bounds rejection before Phase D may start;
- resident format scope `All | Book | Chapter`, unchanged preparation, multi-book atomic apply,
  and no implicit mutation before apply;
- external stateless format/diff never reads or changes resident Braid state;
- message-param empty/single/multiple/non-ASCII values;
- primary and related anchors;
- whole-token, sub-token, and overflow spans;
- no-token SID-only finding;
- exact and anchor-only SID, bridge 127 and 128 boundary;
- canonical order and tie preservation;
- unchanged wasm materialization vs unchanged pure-JS reconcile; reconcile accepts only semantic
  findings and cannot parse packed bytes or validate a checksum/source hash;
- one-of-1000 change with object identity assertions;
- insertion/deletion/reorder and token-index rebase;
- duplicate logical identity deterministic occurrence;
- book removal/addition;
- malicious counts/offsets never allocate before bounds validation.

### 11.5 Baseline/diff/serialization tests

- tokens→USFM byte identity over all fixtures;
- dirty false after equal baseline, true after content change, false after exact revert;
- missing baseline is dirty; diff reports every requested missing book;
- scoped book/chapter serialization and ambiguity errors;
- resident diff equals stateless core diff;
- no formatting during serialization;
- baseline mutation cannot change current state;
- EOL contract (§2.2#16): CRLF USFM ingest round-trips byte-identically; token ingest with
  declared `CrLf` emits CRLF newline tokens and hashes the emitted bytes; chapter updates inherit
  the book's ending; mixed-EOL input preserves exact bytes until first edit; warm-cache validation
  succeeds for an untouched CRLF book;
- chapter-scope dirty: same-label baseline run comparison; missing baseline run is dirty;
  duplicate labels return the typed ambiguity error;
- `to_tokens`/`preview_patch`: preview equals post-apply tokens on the same snapshot; neither
  mutates; stale handles reject.

### 11.6 Oracles and consumer transcripts

At each phase run normal unit/property tests and the ignored lint oracle. Phase F also pins one
cross-platform transcript:

```text
seed corpus → initial lint → unchanged lint
update chapter → lint → reconcile
undo exact bytes → lint → reconcile
format/fix whole book → lint
add/remove book → lint
set baseline → edit → dirty/diff → exact revert → clean
serialize → reopen from packed cache → compare materialized snapshot
```

The transcript asserts semantic results, canonical order, stable identities, mutation effects,
dirty state, and which books recomputed. It does not assert private field layout.

## 12. Performance and size gates

Correctness gates first. Performance evidence is recorded after each relevant phase with the spike
harness and product-shaped editor path.

- Token encode/decode must retain a material advantage over wasm object materialization on GEN;
  otherwise stop before adoption and diagnose rather than ship complexity on historical numbers.
- Clean lint must do no core lint work and should be dominated by returning/packing policy.
- One-chapter edits may lint one whole book; measure before considering chapter compute.
- JS reconcile must show unchanged-object reuse and avoid whole-snapshot object churn.
- Record cold wasm initialization, wasm bytes, peak heap, encode, transfer, decode, reconcile, and
  end-to-end visible latency separately.
- Native Rayon is not evidence for webview wasm. Test bundler web, direct web, and the native
  Tauri host (§2.2#18) — measuring `materialize` source-binding cost separately on desktop.
- No hard performance threshold permits semantic or payload loss.

## 13. Documentation, comments, and generated artifacts

- Crate docs state ownership and “must not own” boundaries.
- Public types document identity, units, lifetime, error, ordering, and atomicity.
- Codec comments explain why fields/limits exist and cite the generating schema, never this plan.
- Generated files contain source/generator instructions and are checked for drift.
- npm README documents buffer lifetime, sync wasm behavior, wasm-backed `decodeTokens` and
  `materialize` versus pure-JS `reconcileFindings(previous, next)`, cache validation, stable
  token-id requirement, typed-error fallback to normal Braid USFM ingest/parse, and that there is
  no JS packed decoder/XXH3 or npm `decodeView` raw-buffer parser. It also documents that
  caller-side manifest validation (unique declared books and source keys) is mandatory, because
  `DuplicateBook` refusal is total by design (Gate 0F §4.4).
- Editor integration docs identify braid as USFM state and the editor as coordinator.
- No production comment references phases, Gate 0, or the plan.

## 14. Progress and execution mechanics

The append-only progress file records:

- current phase/step/status;
- execution commit and changed-file ownership;
- assumptions and owner adjudications;
- evidence and commands for each gate;
- fixed-width corpus maxima;
- public API/declaration diffs;
- deviations and the plan amendment that authorized them;
- discarded approaches and why;
- exact verification run/skipped;
- next step and stop conditions.

Implement in phase order. A later phase may not begin because an earlier piece “mostly works.”
Every commit leaves normal APIs and oracles coherent. Stop rather than blending incompatible
approaches.

## 15. Known footguns and stop clauses

- **Spike authority:** copying `wire_spike.rs` assertions/private raw marker indices as production.
- **Unbound source:** decoding offsets against external bytes whose length/hash was not verified.
- **Identity confusion:** keying reconciliation on positional `token_idx` or non-unique token ids.
- **Hidden duplicate collapse:** using `Map<BookId, Chapter>` or one range per chapter label.
- **False incremental semantics:** chapter-linting only the easy rules while dropping whole-book
  structure/number behavior.
- **Payload loss:** deriving messages/fixes in JS when required data was not encoded.
- **Schema by enum order:** serializing Rust discriminants or marker array position without a
  stable explicit table/stamp.
- **Hash as proof:** treating xxhash equality as semantic equality.
- **Partial commit:** consuming dirty flags before all dirty-book lint succeeds.
- **Adapter fork:** wasm/Tauri implementing a second lifecycle or codec.
- **Decoder fork:** JavaScript parsing packed bytes, computing XXH3, or treating generated
  `wire-schema` constants as a production decoder instead of entering wire through wasm/native
  Rust.
- **Algorithm fork:** reimplementing a core algorithm (e.g. USFM emission) against wire types
  instead of routing both token shapes through one core trait (§4.3).
- **Accidental formatting:** `to_usfm` changing bytes.
- **Unsafe bless:** `BLESS=1` in a CRLF-tainted or non-clean checkout.

Stop and return to the owner on:

1. any unadjudicated behavior-oracle difference;
2. any current finding payload that cannot round-trip;
3. any corpus maximum exceeding a field without the specified overflow/version path;
4. any need for a dependency edge opposite the DAG in §4;
5. any mutation that can change tokens without rebuilding authoritative source/hash;
6. any corpus representation unable to retain duplicate/out-of-order chapter runs;
7. any decoder path that panics or exposes partial views on malformed input;
8. any editor path unable to supply book-wide unique stable token ids;
9. any generated JS/Rust schema drift;
10. any performance result that removes the measured boundary advantage and therefore the reason
    for the packed layer.

## 16. Non-goals for this epic

- No delta/tombstone protocol.
- No silent deduplication, numeric chapter sorting, or ambiguous chapter guessing.
- No chapter-local lint compute, rule dependency graph, boundary replay, or incremental rule
  cache in v1.
- No chapter-local/global KeyIdx rebase.
- No absolute mutable book byte span as a finding identity.
- No raw-text/VS Code incremental addressing contract.
- No persistence IO, cache directory policy, Git/LFS automation, or network transfer.
- No embedded-source/one-read whole-corpus artifact in v1; see the candidate ledger.
- No threaded wasm, SAB, worker pool, COOP/COEP, or cancellation protocol.
- No async interactive wasm API.
- No independent production JS packed parser, JS XXH3, or npm `decodeView` raw-buffer parser;
  generated `wire-schema` data is conformance/debug-only.
- No cryptographic hash claim.
- No whole-Bible resident columnar storage unless a heap profile demonstrates need.
- No consistency/monotonicity rule reorganization.
- No formatter redesign or automatic formatting on save.
- No packed diff format in v1; resident diff returns core `DiffSkeleton<OwnedToken>`.
- No output-surface feature matrix beyond the separately approved Cargo build variants.

## 17. “Maybe eventually” type-shape ledger

These are named future seams, not v1 placeholders:

1. **Granular TS token union:** replace the single tagged optional-field `Token` with a generated
   discriminated union after editor adoption proves the migration cost worthwhile.
2. **Chapter-compute lint:** core-owned rule/substrate classification and parity proof; braid may
   then cache chapter products while keeping complete-book semantics. See
   `../candidates/chapter-grain-braid-lint.md`.
3. **Embedded-source corpus bundle:** optional one-read artifact containing the same external USFM
   bytes plus token/finding sidecars. Deferred at
   `../defferred/embedded-source-corpus-bundle.md`.
4. **Persistent editor linebreak ids:** investigate Lexical NodeState for non-custom linebreak
   nodes. See `../candidates/editor-persistent-linebreak-token-id.md`.
5. **Raw-text finding address:** chapter-relative source address plus SID fallback for consumers
   without stable token ids.
6. **Packed diff schema:** distinct stable record family only if later measurements justify
   replacing the existing object-shaped `DiffSkeleton` boundary.
7. **Consistency module:** onion marker/structural consistency vs sous-chef content consistency,
   with a separately adjudicated monotonicity boundary and `TokenPatch` fix model.
8. **Native resident Tauri session:** ~~only if product measurements justify the second hosting
   path~~ — **superseded 2026-07-27 by §2.2#18**: the native resident host is first-class in v1;
   the Tauri command host itself remains editor-owned app code.
9. **Selective output builds:** Cargo feature presets with explicit API matrices; never bundler
   tree-shaking claims.
10. **Resident columnar corpus:** only after heap evidence; it must not leak packed storage shapes
   into core semantics.
11. **Stronger cache integrity:** optional cryptographic digest for untrusted distribution, distinct
   from xxhash content identity.
12. **`SerializableToken` core trait:** unify USFM emission over borrowed-native and owned-wire
   tokens so `usfm_onion_wasm` deletes its `token_values_to_usfm` reimplementation and calls one
   core emitter. See §4.3 and §15 "Algorithm fork". **Pulled forward out of braid (2026-07-24)**
   as its own pre-braid plan — `../approved/serializable-token-contract.md` — so the editor has a
   concrete minimal token contract to implement now, and the serializer fork dies before braid.
13. **Typed `toUsj()`:** the four hand-written USJ TypeScript declarations
    (`Value`/`UsjDocument`/`UsjNode`/`UsjElement`, a `typescript_custom_section` string in the wasm
    crate) stay in wasm as the documented exception in v1; `toUsj()` remains declared `any`.
    Promoting them to real tsify types over core USJ output is a declaration change deferred until
    someone needs it. New `Braid.toUsj(scope)` declarations reuse this same union.
14. **Delete `js/token-sids.js`:** the deliberate wasm-free duplicate of `derive_canonical_sids`
    is retained in v1 (it is conformance-tested against the Rust export, which is its drift alarm).
    Re-examine at Phase F step 2 with caller evidence: delete in the breaking release only if the
    editor census shows zero remaining call sites.

## Relates to and prior art

- Phase A evidence worktree (do not delete):
  `.claude/worktrees/agent-af68c779deab4e90a`, especially `src/wire_spike.rs`,
  `examples/wire_spike.rs`, `js/{wire_decode,wasm_vs_bin,spike_b}.mjs`, and generated PSA/GEN
  fixtures.
- SIMD lexer worktree (wash; do not delete):
  `.claude/worktrees/agent-a2fd6de7683db6f87`.
- Current behavior oracle: `tests/lint_oracle.rs` and `tests/lint_oracle_baseline.txt`.
- Current boundary crates: `crates/usfm_onion_dto`, `crates/usfm_onion_wasm`.
- Current core seams: `src/token.rs`, `src/lint_impl.rs`, `src/format`, `src/diff`, `src/vref.rs`.
- Pre-braid prerequisite (pulled forward 2026-07-27 from Gate 0C finding F1):
  `../format-token-attribute-passthrough.md` — format/fix legs drop `attributes`/`attributeSource`
  via `FormatToken`; must land (generic pass-through shape) before Phase C/E's patch path, or §9's
  losslessness is violated on attribute-bearing books.
- Still-separate plans: candidates/deferred output work and consistency/monotonicity planning.
  (`lint-invalid-book-code` landed on branch `lint-book-code-rules` as `InvalidBookCode` +
  `BookCodeNotUppercase`, with the `\id` canonical set audited — XXA–XXG added.) (The
  `hygiene-caveats` survey is closed: the enum-mirror
  item shipped on branch `single-source-boundary-enums`; the `LintableToken` and `marker_defs`
  items were decided "leave" — the former folds into §4.3, the latter is captured in the
  `parse-perf-delimiter-absorption` memory.)
