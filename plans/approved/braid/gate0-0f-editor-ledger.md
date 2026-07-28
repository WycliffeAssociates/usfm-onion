# Gate 0F — editor contract alignment and ingest preconditions

Executes `handoff-gate0-0f.md` (§3.1 stage 0F of [`braid-epic.md`](./braid-epic.md), rescoped by
owner adjudication 2026-07-27). Evidence only. No production code, no commits, no tracked-file
edits in either repository.

Two jobs, in order:

1. **Alignment** — does braid-epic.md position braid to fulfill the editor's owner-verified
   adoption plan, as a *general* stateful USFM library (not coupled to editor app policy)?
2. **Preconditions** — the handful of facts about the *current* editor that braid's ingest
   invariants depend on regardless of what the editor migrates off of.

## 0. Provenance and replay

| Item | Value |
| --- | --- |
| `usfm_onion` base commit | `c22caa95a48902d91688919afe79dbde76bb3c6d` (branch `braid`), unchanged since 0A |
| Editor repo | `/Users/willkelly/Documents/Work/Code/scripture-editor-proto-2` |
| Editor HEAD | `ded10b698626bdf96dfe68919817d3ed206e817e`, branch `feat/stet`, committed 2026-07-21 |
| Editor's installed onion | `usfm-onion-web` 0.0.9 (`github:WycliffeAssociates/usfm-onion#v0.0.9`) — one minor behind this repo |
| Consumer intent document | `agent-tmp/plans/braid-editor-adoption/plan.md` (1053 lines, owner-verified) |
| Node | v24.4.1 · vitest 4.1.8 |

Probe files (all under the editor's gitignored `agent-tmp/`, never among tracked sources):

| File | sha256 |
| --- | --- |
| `agent-tmp/gate0f/vitest.probe.config.ts` | `eaf4b879763b159fe1fb835de2b8b1638c9c6689fa0113efe8a45a74c7bc2b13` |
| `agent-tmp/gate0f/preconditions.probe.ts` | `cce514024334b31f7cdeee0c9f42ee03aaea4e98d373265b230001a927623d9a` |
| `agent-tmp/gate0f/diagnose.probe.ts` | `4349367e3b3336c2df1c57725d38f8510caecd79e070cb424a1d03b7400c0f76` |

Commands (from the editor repo root):

```sh
npx vitest run --config agent-tmp/gate0f/vitest.probe.config.ts
npx vitest run --config agent-tmp/gate0f/vitest.probe.config.ts agent-tmp/gate0f/diagnose.probe.ts
```

The probe config re-exports the repo's real `vitest.config.ts` (same aliases, plugins, wasm
inlining, setup file) and overrides only `include`, so resolution matches the production test suite.
Raw JSON output: `agent-tmp/gate0f/out/{p1-p4-books,p1-p4-summary,p5-history-identity,p6-linebreak-stability,p7-crlf,diagnose}.json`.

Corpora scanned separately (never concatenated), rollup hashes by the Gate 0E method
(`find … | xargs shasum -a 256 | sort | shasum -a 256`):

| Corpus | Files | Rollup sha256 |
| --- | --- | --- |
| `tests/mockData/berean-standard-bible` | 66 `.usfm` (full Protestant canon) | `b9f2b8c4cbf2ea3a3acf57dfa5b35e1128d47192cac081f545ac5cb41cac7180` |
| `tests/mockData/llx_reg` | 27 `.usfm` (Lauan NT) | `245e1c2505b4bffa437d33870b10724fe137f6d872aad367f1d6af9e277c54cd` |
| `tests/mockData/synthetic` | 2 `.usfm` (kitchen-sink, common-errors) | `21c1abb795449e1fb9acc25b8a14b1de90915ca5322678b007782a02668c649c` |

Aggregate metadata only; no scripture text is reproduced in this ledger.

Reused rather than redone, per the handoff: the editor field-usage census in
[`gate0-0d-payload-ledger.md`](./gate0-0d-payload-ledger.md) §5 (reads `code`, `messageParams`,
`tokenId`, `message`, `sid`, `fix`, `severity`, `relatedTokenId`, `category`, `issueType`, `marker`;
zero reads of `template`, `span`, `relatedSpan`; identity already
`onion:${code}:${tokenId}:${relatedTokenId}#occurrence`).

---

## 1. Contract alignment matrix

Classifications: **F** fulfilled · **G** fulfilled via editor glue (braid supplies the general
primitive; the thin adapter is the intended design) · **P** app policy, correctly not braid's job ·
**GAP** intent braid should serve as a general library but the planned surface cannot ·
**CONFLICT** the two plans disagree on the same contract.

Citations: `E§` = editor adoption plan, `B§` = braid-epic.md.

| # | Contract the editor needs | Editor cite | Braid cite | Class |
| --- | --- | --- | --- | --- |
| A1 | `WorkingFilesStore` stays the UI authority for the live `Token[]`; braid is the semantic authority | E§3.1 | B§2.1#2, §5.4 | F |
| A2 | Braid owns parse, lint, serialization, baseline/diff, fix/format patching, marker facts, packed snapshots, cache validity | E§3.1 | B§5.3, §9 | F |
| A3 | App owns `SourceKey`, paths, IO timing, atomicity, cache location, Git, debounce, worker lifecycle, UI | E§3.1 | B§2.1#2; §9 "Braid does not read or write cache files" | P |
| A4 | Portable data keyed by `BookId`/`BookCode`; `SourceKey`/paths never enter packed bytes | E§3.1 | B§2.2#9, §5.2, §8.1 | F |
| A5 | Braid must not pre-decide Galley's lifecycle | E§3.1, §14#5 | B — silent | P |
| A6 | Braid's inputs are complete Tokens; vref/vref-map projection stays Galley-owned | E§3.1 | B§5.2 (`BookInput`); resident `to_vref` deliberately absent from §5.3 | P |
| S1 | Editor sends complete chapter / book / corpus, never a lint scope | E§3.2 | B§5.2, §5.3 | F |
| S2 | The live trigger is a no-argument `lint()` | E§3.2, story 20 | B§5.3 | F |
| S3 | v1 may recompute the whole dirty book; narrowing is internal and invisible | E§3.2 | B§2.2#1, §6.4 | F |
| S4 | Every successful `lint()` publishes a complete ordered corpus snapshot | E§3.2, story 21 | B§5.5 `LintSnapshot` | F |
| S5 | Clean `lint()` does no rule work and may reuse the current snapshot | E§3.2 | B§5.3, §12 | F |
| S6 | `update_chapter` addresses one existing run by exact label; ambiguity forces a whole-book publish | E§9.2 | B§5.2 (`ChapterLabel::Number(Box<str>)`, `AmbiguousChapter`) | F |
| S7 | Chapter mutations keyed by `SourceKey` in the host interface vs `BookId` in braid | E§9.1 | B§5.2 `ChapterTarget` | G |
| H1 | One `BraidHost` contract with a web-worker/wasm and a native-Rust implementation | E§3.3 | B§8.3 (braid is plain sync Rust; hosts are app adapters) | G |
| H2 | wasm braid resident **inside a Vite module worker** | E§3.3, §10 §0.5 | B — no packaging gate covers worker instantiation | GAP (small) |
| H3 | No hand-maintained Tauri DTOs: bind the native host to braid's own Rust types + the same generated TS | E§3.3, §7 "generated `Token` + errors" | B§2.2#13 gates serde behind a `wasm` feature that also pulls tsify/wasm-bindgen | GAP |
| H4 | Tauri returns packed bytes as `ArrayBuffer`, never a JSON object graph | E§3.3, Gate 2 | B§2.2#7 (wire packs), §4.1 — but no *native* Rust encode entrypoint is named anywhere | GAP |
| H5 | Native Rust is the planned desktop host and must pass the same contract suite at the editor's Gate 0 | E§10 §0.5–0.6, §14#7 | B§8.3 explicitly permits shipping wasm-in-webview first and deferring resident native IPC | CONFLICT |
| H6 | Function color: marker lookup / node transforms / caret stay synchronous; load, lint, serialize may be async | E§3.3 | B§8.2 (all methods synchronous) | F |
| H7 | Filesystem writes stay app-side; the host merely returns bytes | E§3.3 | B§5.3 `to_usfm`, §9 | F |
| **EOL** | **Per-book line ending owned by braid; `toUsfm` emits newline tokens using the stored book EOL** | **E§3.4 (declared an upstream release prerequisite)** | **B — no EOL contract exists; §9 hashes exact `to_usfm` bytes** | **CONFLICT** |
| **R1** | **`restoreCorpus(sources, packed)`: validate + seed resident tokens and lint cache without decoding tokens to JS and resubmitting them** | **E§3.5 (upstream release prerequisite)** | **B§5.3 `prime_lint_cache` restores lint only; residency needs `replace_corpus`; §8.2 `primeLintCache(input: LintPrimeInput)` takes a JS object** | **GAP** |
| R2 | Cache validity proven by braid, not the app: format/rules/catalog stamps, source length+hash, lint-config fingerprint, engine stamp, snapshot id | E§3.5, story 27 | B§2.2#10, §5.5, §9 | F |
| R3 | Invalid/missing/stale cache is an ordinary miss, never a project-open failure | E§2, §3.5, story 11 | B§5.3 `PrimeReport`/`PrimeRejection`, §5.6 `DecodeError` | F |
| R4 | One complete-corpus sidecar is the cache granularity | E§14#3 | B§7 container is whole-corpus | F |
| R5 | Packed tokens bind to caller-supplied exact source bytes | E§3.5 `RestoreBookSource.source` | B§2.2#3, §8.1 `materialize(sources, packed)` | F (see P3b) |
| N1 | `lint()` returns one packed complete snapshot as a transferable buffer | E§3.6 | B§8.2 `lint(): ApiResult<Uint8Array, LintError>` | F |
| N2 | Official `reconcileFindings(previous, packed, tokensByBook)` | E§3.6, §9.3 | B§8.1 — identical signature | F |
| N3 | Reconciliation is authoritative for add/remove/order; reuse an object only when every public field is equal | E§3.6 | B§8.1 | F |
| N4 | New findings inserted in canonical order without the app owning a comparator | E§3.6 | B§2.2#15 (position-keyed order; packed records stored in canonical order) | F |
| N5 | Finding identity excludes message text and fix payload | E§3.6 | B§8.1; 0D §5 confirms the editor's existing key matches | F |
| N6 | Stable generated lint-code union + locale-neutral structured params | E§3.6, Gate 4 | already shipped: `LintCode` is a 32-arm kebab string union and `messageParams: Record<string,string>` in `pkg-bundler/usfm_onion_web.d.ts:394,232` | F |
| N7 | App-owned wording; engine `message` never the UI fallback | E§3.6 | B keeps `message`/`template` as diagnostics; §7.6 message payload sidecar | P |
| N8 | Ignored codes, severity policy, active-book filtering, limits, overlays stay app selectors | E§3.6 | B — silent, correctly | P |
| N9 | Stale results rejected by snapshot/generation identity before publication | E§3.6, §9.3 | B§5.3 `expected_snapshot_id()`, §8.1 `snapshotMismatch`; the generation envelope is the host's | G |
| N10 | One thin Braid-Finding → editor-Finding projection at the publication edge | E§3.6, §9.3 | B§8.1 public `Finding` | G |
| D1 | Shadow-compare app dirty flags against braid `isDirty` during parity | E§3.7 | B§9, §5.3 `is_dirty(scope)` | F |
| D2 | Braid becomes the semantic dirty authority, including undo-back-to-baseline | E§3.7, story 9 | B§9 (exact byte equality against baseline) | F |
| D3 | Save flushes, captures, writes, then advances the baseline **only for successfully persisted books** | E§3.7, Gate 5 | B§5.3 `set_baseline(BookInput)` per book | F |
| D4 | Chapter-scope dirty against a book-scope baseline | E§3.7 (`chapter.dirty` is the current oracle) | B§9 defines dirty as book bytes vs baseline bytes; `is_dirty(Chapter)` behaviour when the baseline lacks or duplicates that label is unspecified | GAP (small) |
| D5 | `sourceTokens` retained for compare/revert/save-race | E§3.7#5, §5 | B — silent | P |
| M1 | One immutable core-owned marker catalog/query surface through the npm entrypoint | E§3.8 | B§2.1#9, §4.1 (wasm re-exports core registry) | F |
| M2 | Delete `initializeUsfmMarkerCatalog` and the mutable nullable app registry | E§3.8, Gate 7 | B — silent; wasm module init still precedes any query | G (see §2.8) |
| M3 | Marker semantics available synchronously to node transforms, paste, rendering, tests, workers | E§3.8, Gate 7 | B§8.2 sync methods; `js/token-sids.js` retained as the deliberate wasm-free exception (2026-07-27 adjudication 8.3) | G |
| M4 | Zephyr's `s5` acceptance is an app policy overlay (or an upstream compatibility option) | E§3.8, §9.6 | B — no compatibility option; the overlay is the editor's stated default | P |
| M5 | Registry answers "is this a marker", "takes children", paragraph category, payload kind, closing behaviour | E§3.8 | shipped `UsfmMarkerCatalog` + `MarkerCategory`/paragraph category in the generated `.d.ts` | F |
| V1 | Resident ops on the live corpus; stateless ops for symmetric external compare | E§3.9 | B§5.4 table | F |
| V2 | Braid diff must not turn the neutral left/right compare model into a dirty/current API | E§3.9 | B§5.3 (verbatim the same constraint) | F |
| V3 | Split `IUsfmOnionService` into `BraidSession` + `StatelessUsfmService` | E§3.9, §8.2 | B§5.4 supports the split; naming is the app's | P |
| V4 | Node transforms and paste remain synchronous editor code | E§3.9 | B§8.2 | P |
| B1 | Crash-backup bytes come from braid `toUsfm`; app keeps debounce/staleness/atomicity/classification | E§3.10, §10 §5.5 | B§5.3, §9 | F |
| B2 | Recovery folded into the loader before the first resident lint | E§3.10#1 | B — silent, correctly | P |
| B3 | Any future chapter-granular backup uses an ordered list, never a label-keyed map | E§3.10 | B§6.1 already forbids a single-range label map | P (consistent) |
| **X1** | **`applyPatch` hands back the new tokens for the store to commit; format preview shows what the patch will do** | **E§7 rows `applyPatch`/`prepareFormatPatch` ("commit returned tokens to store", "choose options and preview"), Gate 6** | **B§5.3 returns `MutationEffect`/`PatchHandle` only; braid exposes no resident token read and no patch inspection — the sole token exposure is `LintSnapshot.tokens` via `lint()`** | **GAP** |
| X2 | `WorkspaceTokenPublisher` emits complete ordered chapter/book/corpus deltas, preserving duplicate labels and disk order, never sorting numerically | E§9.2 | B§2.1#3, §5.2 | F |
| X3 | `BraidSnapshotPublisher` exposes the last valid packed buffer to the cache writer without decoding it | E§9.3 | B§8.1 (bytes are opaque to the app) | F |
| X4 | `WorkspaceWarmCache` is byte-only, app-owned, disposable | E§9.4 | B§9 | P |
| X5 | `BraidWorkspaceLoader` cold/warm flow | E§9.5 | depends on R1 | G (blocked on R1) |
| X6 | Resident `toUsx`/`toUsj`/`toHtml` for real product callers | E§7 | B§5.3, §8.2 (`toUsj` stays typed `any` per the 2026-07-27 adjudication 8.2) | F (documented exception) |
| X7 | Snapshot-bound fixes rejected when stale | E§7, Gate 6 | B§5.3 `PatchError::StaleSnapshot` | F |
| X8 | `SnapshotId` as semantic stale/result validation, mapped to the editor's own generation | E§7 | B§5.3 `expected_snapshot_id()`, §5.5 | G |
| X9 | Source hash / stamps / checksum are braid's cache validity, not the app's | E§7 | B§9 | F |
| X10 | Duplicate declared books must not be silently collapsed | E§9.2, story 16 | B§5.2 `DuplicateBook` refuses the whole corpus before `lint()` | F (see P4) |

**Tally — 63 rows:** **fulfilled 35** · **fulfilled via editor glue 8** (S7, H1, N9, N10, M2, M3,
X5, X8) · **app policy 12** (A3, A5, A6, N7, N8, D5, M4, V3, V4, B2, B3, X4) · **gap 6** (H2, H3,
H4, R1, D4, X1) · **conflict 2** (EOL contract, H5 native-host sequencing).

No row shows the adoption plan expecting braid to own app policy. Every P row is one the editor
itself already assigns to the app; the closest to a mis-assignment is M4, where the editor offers
"app policy overlay **or** upstream-recognized compatibility option" — braid should decline the
second half explicitly, since an app-specific marker acceptance is not a general USFM fact.

---

## 2. The eight named-sensitive contracts

### 2.1 EOL — CONFLICT (blocks the editor's own Gate 0)

`grep -n -i "line ending|lineending|crlf|eol" braid-epic.md` returns **one** hit, in an unrelated
bless-safety footgun (§15). braid-epic.md has **no EOL contract at all**, while the editor's §3.4
declares one and states "This is an upstream release prerequisite. Do not keep `tokensToUsfm` as a
permanent fallback," and its Gate 0 says "If either EOL or direct restore is absent, stop and
adjudicate upstream."

Why this is load-bearing rather than cosmetic — measured (probe P7, `out/p7-crlf.json`, a real
llx_reg book converted to CRLF in memory):

| Measurement | Result |
| --- | --- |
| `detectLineEnding` on parsed tokens | `CRLF` |
| every parsed newline token's `source` | `"\r\n"` |
| every newline token's `source` after one Lexical round trip | `"\n"` |
| `tokensToUsfm(currentTokens, eol)` reproduces the CRLF file | **true** |
| `tokensToUsfm(currentTokens, "\n")` reproduces it | false |
| naive `tokens.map(t => t.source).join("")` reproduces it | **false** |
| the raw `source`-join comparison `WorkingFilesStore.applyPatch` uses for `dirty` | **not equal** — an untouched CRLF chapter reads as dirty |

The editor is LF-internal by design and documents it
(`src/core/domain/usfm/usfmBytes.ts:20-25,45-52`; `src/app/domain/editor/utils/usfmTokenStreamSerializedAdapter.ts:276-281`
stamps `"\n"`; regression test `tests/unit/lineEndingPreservation.test.ts`). So for any CRLF book:

- braid `to_usfm` (source concatenation, per B§9 "token-to-USFM lossless") emits LF;
- `SourceHash` = xxhash3 over those bytes never matches the disk file → **warm cache can never
  validate**, `is_dirty` is permanently `true`, and every save rewrites the whole file;
- the editor cannot fix this on its side without keeping exactly the serializer B§16/E§5 delete.

The editor's shape (E§3.4): cold USFM input detects and stores the book's line ending; token-based
`BookInput` carries `lineEnding: "\n" | "\r\n"`; `update_chapter` inherits the resident book's
ending; `to_usfm` and USFM embedded in fixes/backups emit newline tokens using the stored ending;
all other token `source` and attribute trivia stay untouched. Carried forward from the memory note
`braid-eol-serialization`: core's `tokens_to_usfm_reconstruct` has no EOL override today.

**PROPOSED amendment A (owner adjudication).** Add an EOL contract to braid-epic: a per-book
`LineEnding` on `BookInput`/`BookTokensInput` (detected for `BookInput::Usfm`, declared for
`Tokens`), inherited by `ChapterInput`, applied by `to_usfm`/patch-embedded USFM, folded into the
book's `SourceHash`, and explicitly excluded from every other token field. Open sub-questions the
owner must settle: (a) does core gain an EOL-aware emitter or does braid post-process newline
tokens; (b) does `BookInput::Usfm` with mixed EOLs preserve bytes until first edit (the editor's
stated position) or refuse; (c) is EOL part of the packed container (it is derivable from the bound
source bytes, so it need not be).

### 2.2 Opaque warm-cache restore — GAP

The editor's §3.5 asks for `braid.restoreCorpus(sources, packed)` that validates format, checksum,
engine/rule/catalog stamps, book identity/order, source lengths/hashes, lint config, and snapshot
identity, then **seeds resident tokens and the lint cache directly**, with `materialize` remaining
an independent main-thread convenience. It explicitly names
"`materialize -> replaceCorpus(Token[]) -> primeLintCache`" as a fallback that must be measured and
adjudicated, "not silently enshrined."

braid-epic's planned surface is exactly that fallback, and the JS round trip is not incidental:

- residency can only be established by `replace_corpus`/`update_book`, i.e. `Usfm { source }`
  (re-parse) or `Tokens(Vec<OwnedToken>)` (JS objects sent back across the boundary) — B§5.2;
- `prime_lint_cache` restores only the lint contribution, and B§8.2 types it
  `primeLintCache(input: LintPrimeInput)`, so **the findings must also be decoded into JS objects
  first**. B§5.3's "the composing adapter decodes" identifies the wasm crate as the decoder, but no
  wasm entrypoint takes packed bytes.

Note the third path braid-epic already almost supports and the editor did not consider:
`replace_corpus(Usfm{source})` + `prime_lint_cache` skips the token round trip at the cost of one
cold parse per book while still skipping lint (the expensive half). Gate 0E measured parse-side
costs; this is the option worth measuring before adding API.

**PROPOSED amendment B.** Add to B§8.2 a packed-bytes restore entrypoint on the wasm `Braid` — e.g.
`restoreCorpus(sources: SourceCorpus, packed: Uint8Array): ApiResult<RestoreOutcome, RestoreError>`
— implemented in the composition root (wasm may call both wire and braid, so no new crate edge),
and a braid-side seeding entrypoint that accepts already-decoded semantic tokens+findings in one
call. Alternatively adjudicate the parse+prime path explicitly with measurements. Either way B§5.3
should state which of the three restore paths is the supported one, because the editor stops at its
Gate 0 otherwise.

### 2.3 Findings contract — fulfilled

Every clause aligns, and the 2026-07-27 order re-key improved the fit:

- signature match on `reconcileFindings(previous, packed, tokensByBook)` (B§8.1 = E§3.6/§9.3);
- identity tuple matches the editor's existing key (0D §5), and message/fix are excluded from
  identity but included in equality — B§8.1 says exactly this;
- B§2.2#15 keys canonical order on token position and stores packed findings in canonical order, so
  the JS side needs no comparator at all; the Rust-UTF-8 vs JS-UTF-16 divergence that Gate 0D found
  is gone;
- the generated `LintCode` union already exists as a closed 32-arm kebab string union
  (`pkg-bundler/usfm_onion_web.d.ts:394`) with `messageParams: Record<string, string>` — the
  editor's compile-exhaustive formatter requirement is satisfiable today, and F2's replacement of
  the hand-maintained `lint_code_variants()` with a wire-generated catalog keeps the runtime census
  honest.

One alignment note, not a gap: the editor assigns deterministic same-key **occurrence** itself after
a stable base-key sort (`normalizeFindings.ts:36-63`, 0D §5). B§8.1 also names occurrence as part
of identity. Both can hold — braid must simply guarantee that repeated identical keys arrive in a
deterministic order, which §2.2#15 now does. Worth one sentence in B§8.1 so the two occurrence
assignments cannot disagree.

### 2.4 Dirty and baseline — fulfilled, with one small gap

Aligned: byte-exact dirty (B§9 vs E§3.7), per-book baseline advance after successful writes,
missing baseline ⇒ dirty, `MissingBaseline` for diff, shadow-parity staging is app-side.

Gap D4: the editor's live oracle is `chapter.dirty`, and its migration explicitly keeps chapter
granularity in the UI (E§3.7#3). braid's `is_dirty(CorpusScope::Chapter(target))` exists, but §9
defines dirty only as *book* current bytes vs *book* baseline bytes. Unspecified: what
`is_dirty(Chapter)` means when the baseline has no run with that label (chapter added since the
baseline), or has more than one (duplicate labels — which §5.2 deliberately retains). Both cases are
reachable in the editor. **PROPOSED amendment C:** state the chapter-scope dirty rule — most
plausibly "compare the chapter run's exact bytes to the baseline run with the same label; a missing
or ambiguous baseline run is dirty (or a typed `ScopeError`)."

Also note, as EOL evidence rather than a separate finding, that the editor currently has two dirty
comparisons for this reason: the EOL-normalising `isChapterDirtyUsfm`
(`src/app/domain/project/saveAndRevertService.ts`, covered by `lineEndingPreservation.test.ts`) and
the raw `tokenSourcesEqual` source-join inside `WorkingFilesStore.applyPatch`
(`src/app/state/WorkingFilesStore.ts:356-375`). Probe P7 shows the raw one reports a CRLF chapter
dirty after an idempotent round trip.

### 2.5 Scope semantics — fulfilled

E§3.2's five clauses map one-to-one onto B§5.2/§5.3/§2.2#1/§6.4 (rows S1–S6). `CorpusScope` is
`All | Book | Chapter(ChapterTarget)` over resident data only; there is no `External(tokens)`
variant, matching the editor's insistence that a caller cannot accidentally run a cold operation on
the live corpus (E§3.9, B§5.4). The only translation the editor performs is `SourceKey → BookId`
(row S7), which its own §3.1 already claims as app-owned identity work.

### 2.6 Marker registry — fulfilled via glue, with one wording risk

B§2.1#9 and §4.1 keep registry knowledge core-owned and re-exported, never copied — exactly E§3.8.
The shipped `UsfmMarkerCatalog` plus `MarkerCategory`/paragraph-category types already answer the
five semantic questions E§3.8 lists.

The risk is a wording mismatch, not a capability one. E's Gate 7 says "All node/paste/transform
marker queries behave before app mount and in workers/tests" and "No `initializeUsfmMarkerCatalog`
call or nullable registry remains." A wasm-backed registry is available only after the module is
instantiated; on the `web` target that is an explicit async `init()`. So the achievable invariant is
"no *mutable app-side* registry and no app-owned initialization order", not "zero initialization."
The retained `js/token-sids.js` (adjudication 8.3) is the precedent that a wasm-free JS path is a
deliberate, conformance-tested exception rather than the default. **PROPOSED amendment D
(documentation):** one sentence in B§8.2 or Phase F stating that registry queries are synchronous
*after* module init, and that supplying marker facts without wasm initialization is not a braid
promise.

### 2.7 Resident versus stateless — fulfilled

B§5.4's table is a superset of E§3.9, including the clause the editor cared most about ("Braid diff
means resident-state operations with one side already resident… It must not turn the app's neutral
left/right compare model into a pre-baked dirty/current API") which B§5.3 states in the same terms.
`ScopedOutput<T>` as `T | ReadonlyMap<SourceKey, T>` (B§8.2) gives the editor the ordered grouping
it needs for multi-book save/serialize without a map that hides source order.

### 2.8 Dirty-buffer optimization boundary — app policy, consistent

E§3.10 keeps the whole-book backup, asks only for braid `toUsfm` bytes, and defers chapter-granular
artifacts behind measurement. braid-epic takes no position, which is correct. The one place the two
documents touch is representational and they agree: E§3.10 "must use an ordered list, never a map
keyed by `\c` label" ↔ B§6.1 "A map that stores only one range is forbidden because it would
collapse duplicates."

---

## 3. Additional gaps found while walking §7 and §9

### 3.1 X1 — no resident token read, no patch inspection (GAP)

The editor's capability matrix assigns itself, as the editor remainder for `applyPatch`, "commit
returned tokens to store", and for `prepareFormatPatch`, "choose options and preview"; Gate 6
requires "Preview and apply refer to the same frozen snapshot."

braid's surface (B§5.3) returns `PatchPreparation::Ready(PatchHandle)` — an opaque
`{snapshot, patch}` — and `apply_patch → MutationEffect`. There is **no** method that returns
resident tokens, and `TokenPatch`/`TokenEdit` are described as braid-internal patch-table storage.
The only token exposure in the whole plan is `LintSnapshot.books[].tokens` (B§5.5), reachable only
by calling `lint()`. So the editor's post-fix/post-format store update requires a full lint, and a
format *preview* has nothing to render.

This is a general-library gap, not an editor-policy one: any consumer that keeps its own view of the
token stream needs to read the resident tokens after a mutation it did not author. **PROPOSED
amendment E:** add a resident token read (`to_tokens(scope) -> ScopedOutput<Vec<OwnedToken>>`, the
same envelope as the other projections) and/or make `apply_patch` return the affected books'
tokens; and expose the prepared patch's edits (or a preview projection) so preview and apply share
one frozen snapshot. Note the packed container can already carry tokens, so the wasm form may be
packed bytes rather than objects.

### 3.2 H3/H4 — the native host cannot be built from the planned features (GAP)

E§3.3 forbids hand-maintained Tauri DTOs and requires the native host to bind braid's own Rust types
with the same generated TypeScript. Two obstacles in the current plan:

- **serde is coupled to wasm.** B§2.2#13 puts serde+tsify derives behind a single `wasm` feature. A
  native Tauri command host needs serde (Tauri IPC serializes to JSON) but must not pull tsify or
  wasm-bindgen into a native binary. **PROPOSED amendment F:** split braid's feature into `serde`
  (plain derives, native IPC hosts) and `wasm` (= `serde` + tsify + bindgen glue), and state that
  the tsify-generated `.d.ts` remains the single TypeScript contract for both transports because
  tsify derives from the same serde attributes. Mirror the same split on `usfm_onion_wire`.
- **no native codec entrypoint is named.** `grep` for `pack_`/`encode_`/`fn pack`/`Vec<u8>` in
  braid-epic returns nothing; §4.1 assigns the codec to wire and §8.2 shows only the wasm
  `lint(): Uint8Array`. A native host must call wire directly to produce the ArrayBuffer the editor
  requires. **PROPOSED amendment G:** name the public native wire encode/decode signatures in §5/§7
  and add the Tauri/native consumer to §8.3's boundary description.

### 3.3 H5 — native-host sequencing (CONFLICT)

B§8.3: "Phase F may adopt sync wasm in Tauri's webview first and defer resident native IPC."
E§10 §0.5–0.6 requires, at the editor's Gate 0, that "the native Rust crate behind a Tauri command
host pass the same corpus/contract suite" and that "Tauri packed results arrive as JavaScript
`ArrayBuffer`s"; E§14#7 makes wasm-in-Tauri a measured last resort. Semantically nothing conflicts —
braid is plain sync Rust and the host is app code — but the *release scope* does: whether braid v1
ships a native-host contract test/example or whether that proof is entirely editor-owned. Owner
adjudication; it also decides whether amendments F and G land in v1 or later.

### 3.4 H2 — module-worker packaging (small GAP)

The editor's primary web host is a Vite module worker. Nothing in braid-epic's Phase F package gate
covers importing/initialising the package inside a worker, and the existing smoke harness
(`scripts/test-web-package.mjs`) runs under Node on the main thread. **PROPOSED amendment H:** add
"the published package imports and initialises inside a module worker, and a packed `Uint8Array`
survives transfer" to the Phase F package gate.

---

## 4. Ingest preconditions (Task 2)

### 4.1 Book-wide unique stable token ids — **PRECONDITION NOT MET TODAY** (known cause, filed)

Scan: every `.usfm` file in all three corpora, through the real production functions —
`webUsfmOnionService.parseUsfm` → `normalizeTokenSids` → `groupFlatTokensByChapter` →
`tokensToLexical(regular)` → `lexicalToTokens(state, {bookCode})` (the exact call
`WorkingFilesStore.applyPatch` makes at `src/app/state/WorkingFilesStore.ts:327`) — collecting ids
across the whole book.

| Corpus | Books | Commit tokens | Books with duplicate ids | Distinct duplicated ids | Token instances carrying a duplicated id | Duplicates by kind | Ids missing | Ids empty |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| berean-standard-bible | 66 | 290,106 | **66 / 66** | 5,868 | 77,622 | `newline` 77,622 | 0 | 0 |
| llx_reg | 27 | 38,293 | **27 / 27** | 1,059 | 10,587 | `newline` 10,587 | 0 | 0 |
| synthetic | 2 | 1,060 | **2 / 2** | 54 | 228 | `newline` 228 | 0 | 0 |

**Every duplicate, in all 95 books, is a `newline` token with a `linebreak-N` id.** Non-newline
kinds contribute zero duplicates; zero ids are missing or empty. Worst single book: 279 distinct
duplicated ids. Total `linebreak-N`-shaped ids: 79,600 / 10,840 / 233 — i.e. most, but not all,
collide (the first occurrence per counter value in the book's first chapter is unique).

Mechanism, confirmed by source reading and by the numbers:

1. `lexicalToTokens` has exactly **one** synthesized-id site — `id: \`linebreak-${linebreakId++}\``
   at `usfmTokenStreamSerializedAdapter.ts:277`, with the counter reset per call, and the call is
   per chapter. Hence chapter-local ids repeating across every chapter of a book. Answering the
   handoff's question directly: **there is no other synthesized-id site in `lexicalToTokens()`.**
   The other `guidGenerator()`/`crypto.randomUUID()` sites in that file
   (`:351`, `:449`, `:538-542`) are in `tokensToLexical`/`prettifyTokenToLexicalNode` and mint ids
   for *new* Lexical nodes, which is correct behaviour, not a collision source.
2. The upstream cause is that `tokensToLexical` maps a `newline` token to a bare
   `{type: "linebreak", version: 1}` node (`:344-348`), **dropping the parsed token's id**. Cold-open
   `sourceTokens` do carry unique onion ids (`{BOOK}-{index}`); the ids are lost the first time a
   chapter is projected into Lexical and read back.
3. `materializeNodesForTokenization` can also insert brand-new structural linebreak nodes
   (`:241-258`) with no id, which then take counter ids from the same source.

This is the already-filed `../candidates/editor-persistent-linebreak-token-id.md` case, confirmed at
scale, and the adoption plan already owns the fix (E§10 Section 1 step 4: add `USFMLineBreakNode`,
preserve incoming newline ids, `guidGenerator()` for new ones). **No new synthesized-id site was
found.** Braid's B§2.2#5 invariant ("stable token id uniqueness is validated per book… synthesized
linebreak ids currently do not [satisfy it]. Phase F fixes those ids before adoption") is exactly
right, and `IngestError::DuplicateTokenId` would reject **every** book in every corpus until it
lands.

Persistence of ordinary ids across the lifecycle:

| Transition | Verdict | Evidence |
| --- | --- | --- |
| repeated tokenization of an unchanged state | stable | probe P6 `stableAcrossRepeatCalls: true` |
| undo / redo | **ids preserved exactly** | probe P5: 979 tokens → history canonical snapshot → tokens; `idsIdentical: true`, kinds and sources identical. History stores serialized flat nodes (`canonicalChapterState.ts:69-88`), and `USFMTextNode` keeps `id` in Lexical NodeState (`USFMTextNode.ts:86,103,113,154`), so ids ride the snapshot |
| ordinary edit | preserved for surviving nodes; new nodes get GUIDs | `lexicalToTokens` reads `node.id` verbatim (`:293`); new nodes mint `guidGenerator()`/`randomUUID()` |
| format / fix | preserved for surviving nodes | `useFormatMatching.tsx:200` re-tokenizes the same node tree |
| insert an earlier linebreak | **all later linebreak ids rebase** | probe P6: 210 → 211 newline tokens; `survivingIdsUnchanged: false`, `idsShiftedByOne: true` |
| save / reopen | **not stable across a cold reopen** | reopen re-parses, so ids are onion's positional `{BOOK}-{index}`; an insertion shifts every later id in the book. Harmless for braid (reconciliation is within a session and a reopen re-materializes everything) but it must not be mistaken for cross-session identity |

**Recorded blocker (not simulated).** A live interactive run — real keystrokes, real Lexical history
stack, real DOM, real save/reopen through OPFS/Tauri — is outside what this read-only probe can
perform. Repro steps for editor-side verification at E Gate 1 / B Phase F step 4: open a fully
backed project; note the ids of three `newline` tokens in chapter 2 via `lexicalToTokens`; type a
character in chapter 1, press Enter to create a linebreak, undo, redo; save, close, reopen; re-read
the same three ids after each step. Expected today: the chapter-2 newline ids are `linebreak-0..2`
throughout and equal to chapter 1's; expected after the `USFMLineBreakNode` fix: three distinct
GUIDs, unchanged across all six steps.

### 4.2 Complete chapter token runs — **PRECONDITION MET**

- `WorkingFilesStore.applyPatch` case `"chapter"` receives a whole shaped `lexicalState` for the
  chapter and replaces `currentTokens` wholesale with `lexicalToTokens(lexicalState, {bookCode})`
  (`src/app/state/WorkingFilesStore.ts:314-336`). There is no edit-fragment path into token state.
- The mirror feed reads that canonical stream directly: `tokenizeChapter` returns
  `{tokens: chapter.currentTokens, eol, dirty}`, and `patchesForCommit` emits one `pushChapter` per
  changed chapter ref, or `fullSync` for content-bearing project commits, or `deleteChapter` when
  the chapter is gone (`src/app/domain/editor/pipelines/mirrorPatchProducer.ts:44-140`). Seed is one
  `fullSync` of every book (`seedMirror`, same file `:146-160`).
- Grouping is order-faithful on real data, contrary to what the `Record<number, Token[]>` return type
  suggests: probe `diagnose` reports `groupingPreservesOrder: true` for every case examined, bucket
  keys strictly ascending in all 95 books, and bucket count = `\c` run count + 1 front-matter bucket
  in every corpus (1,255 = 1,189 + 66; 287 = 260 + 27; 14 = 12 + 2).
- Structural caveat, unexercised by any corpus: `groupFlatTokensByChapter` keys buckets by parsed
  integer (`flatTokensByChapter.ts:20-56`) and `ScriptureChapterState.chapterNumber` is a `number`,
  so duplicate `\c 3` runs would merge into one bucket, out-of-order chapters would be re-emitted in
  ascending order, and a non-integer label (`\c 3a`, `\c 003`) cannot be represented. Measured
  incidence: **0 books with duplicate chapter labels, 0 out-of-order, 0 non-integer labels** across
  all 95 files. braid's `ChapterLabel::Number(Box<str>)` and its retention of duplicate/reopened
  runs are strictly more expressive than the current editor model; E§9.2 already promises the target
  publisher will preserve them, which requires the working-state chapter identity to become a label
  rather than a number. Recorded as an editor-side prerequisite, not a braid gap.

### 4.3 Exact UTF-8 bytes — **MET at cold open from file bytes; NOT met through the editor's own serializer**

Three levels measured per book (all 95 files):

| Level | BSB | llx_reg | synthetic |
| --- | --- | --- | --- |
| flat `tokens.map(t => t.source).join("")` equals the file | 66/66 | **26/27** | **1/2** |
| cold-open `serializeChaptersToUsfm(sourceTokens)` equals the file | 66/66 | 26/27 | 1/2 |
| after one full Lexical round trip (`currentTokens`) | 66/66 | 26/27 | 1/2 |

The Lexical round trip adds **no** loss on any of the 95 books: every failure is already present at
the flat source-concatenation level. The two failures are `llx_reg/44-JHN.usfm` (first divergence at
byte 108,176, at a verse whose source contains a literal `|v 16 …` payload that onion parses as an
attribute list) and `synthetic/kitchen-sink.usfm` (byte 4,534, at `\rb ruby|gloss="gloss"\rb*`).
Cause: the editor's byte waist `tokensToUsfm` concatenates `token.source`
(`src/core/domain/usfm/usfmBytes.ts:53-58`), while onion parks USFM 3.1 attribute text in
`attributes`/`attributeSource`, not in `source`. onion's own `tokens_to_usfm` is byte-lossless
(Gate 0E: 425/425 files). **Braid taking over serialization fixes both cases**; this is precisely
E§4 goal 4 and E§5 "No editor-side … token-to-USFM fallback."

Two consequences for braid ingest:

- **P3a.** Exact bytes at cold open must come from the *file*, not from a token re-serialization.
  That is available: `Project.getBook(storageKey).contents` on web
  (`scriptureProjectToParsedFiles.ts:34-49`) and the file path on desktop.
- **P3b (worth an owner note).** The desktop cold-open path deliberately does **not** copy book
  contents into JS — `loadForApp` returns `{text: null, path}` and lets Rust read the files
  (`scriptureProjectToParsedFiles.ts:51-67`). But B§2.2#3 makes packed tokens an external-source
  sidecar, B§8.1 `materialize(sources, packed)` requires one source byte array **per book on the
  main thread**, and E§3.5's `RestoreBookSource.source: Uint8Array` assumes the same. So the warm
  path reintroduces on desktop exactly the whole-corpus byte copy into JS that the current loader
  avoids (≈5.15 MB for a single large book, per Gate 0E). It is not a contradiction — the app can
  read bytes it currently skips — but it is a real cost the editor's §3.10 load-sequencing
  measurements should attribute, and a reason B§12 should record `materialize` source-binding cost
  separately for the Tauri host.
- EOL: the only place normalization intervenes is the newline token's `source` after a Lexical round
  trip; the file's convention is recovered at parse (`detectLineEnding`) and re-applied at the
  serializer waist. See §2.1.

### 4.4 Unique declared books — **NOT GUARANTEED; app-side prerequisite**

The manifest convention that will supply `BookId` + `SourceKey`:

| Loader | `bookCode` (→ `BookId`) | `storageKey` / `path` (→ `SourceKey`) |
| --- | --- | --- |
| Resource Container | `args.identifier.toUpperCase()` from the RC manifest's project entry — `ResourceContainerProjectLoader.ts:39-54` | `storageKey = basename(path)`, `path = ${projectRootPath}/${relativePath}` |
| Scripture Burrito | book code extracted from the ingredient scope, validated against `/^(?:[1-3][A-Z]{2}|[A-Z]{3})$/`, uppercased — `ScriptureBurritoProjectLoader.ts:48-70,105-135` | same: `storageKey = basename(filePath)` |

`BookRef = {bookCode, title, fileName, storageKey, path}` (`src/core/persistence/ScriptureWorkspace.ts:32-38`);
the editable workspace reads `loadedProject.books[]` and keys everything by
`getBookSlug(entry.bookCode)` (`scriptureProjectToParsedFiles.ts:120-175`).

Can it declare duplicates today? **Yes.**

- Burrito `buildBookRefs` pushes one `BookRef` per qualifying ingredient with **no uniqueness
  check**; two ingredients whose scope names the same book produce two entries with the same
  `bookCode`.
- The RC loader shares `toBookRef` and enforces uniqueness only inside `addBook`, which
  replace-by-`bookCode` (`ResourceContainerProjectLoader.ts:398-405`) — the *loaded* manifest is
  never policed.
- `storageKey` is a **basename**, so two books under different subdirectories with the same filename
  collide, and `findBookEntryByStorageKey` (`:298-303`) would resolve ambiguously. That is braid's
  `IngestError::DuplicateSourceKey` as well as `DuplicateBook`.
- Both loaders also sort into canonical order (Burrito in `buildBookRefs`, the workspace via
  `sortUsfmFilesByCanonicalOrder`), so the "caller order" braid preserves is the editor's canonical
  order, not disk order. Compatible with B§2.1#3 (braid preserves whatever order the caller sends),
  but worth knowing when reading a packed TOC.

Measured: 0 duplicate declared codes in berean-standard-bible (66 distinct) and llx_reg (27
distinct); the `synthetic` directory declares `GEN` twice — an illustration that a bare directory of
files can collide, not a project.

Braid's failure mode is total: B§5.2 says a duplicate declared `BookId` "is reported by corpus
validation before resident `lint()` can run… packing is unavailable until the manifest collision is
fixed." So a single malformed manifest makes the whole project unopenable in the braid lane.
**Precondition prerequisite (editor-owned):** validate/deduplicate the manifest — unique `bookCode`
and unique `SourceKey` (use the relative path, not the basename) — before `replaceCorpus`. Worth one
line in braid's docs (B§13 npm README bullet) that the all-or-nothing refusal makes caller-side
manifest validation mandatory.

---

## 5. Target lifecycle transcripts (Phase F parity fixtures)

Written from the **adoption plan's target flows** (E§10 Sections 2–5), not from current glue, as the
handoff requires. `H` = the selected `BraidHost`; `pub` = `BraidSnapshotPublisher`; `wc` =
`WorkspaceWarmCache`. Each transcript is the assertable call sequence; braid-side names follow
B§5.3/§8.2.

### T1 — cold open (E§10 §2 steps 5, 7, 8)

```text
loader.loadExactUsfm()                        -> sources: [{sourceKey, bookId, bytes}]   (exact file bytes)
dirtyBufferRecovery.classify(sources)         -> effective working sources (app policy, before first lint)
wc.read("braid", workspaceKey)                -> null | bytes
H.replaceCorpus({books: [...BookInput::Usfm]}) -> ApiResult<MutationEffect::Changed, IngestError>
H.lint()                                       -> ApiResult<{snapshotId, packed}, LintError>
materialize(sources, packed)                   -> MaterializedSnapshot        (main thread, once)
pub.publish(snapshot)                          -> WorkingFilesStore seed + FindingsStore.commitBraidSnapshot (one transaction)
wc.put("braid", workspaceKey, packed)          -> best effort, after publication
```

Asserts: `MutationEffect::Changed`; complete snapshot covers every declared book in caller order;
`snapshotId` stable for identical inputs; duplicate/out-of-order chapters preserved; every token id
unique per book; `H.toUsfm(All)` reproduces the input bytes exactly (LF **and** CRLF books).

### T2 — warm open (E§10 §2 step 6, §9.5)

```text
loader.loadExactUsfm()                        -> sources
wc.read("braid", workspaceKey)                -> packed
H.restoreCorpus(sources, packed)              -> {kind:"restored", snapshotId} | {kind:"cache-miss", reason}
   restored: materialize(sources, packed) -> pub.publish(...)          (NO parse, NO lint)
   miss:     fall through to T1, then re-warm the cache
```

Asserts: restored path performs zero rule work and publishes findings byte-identical to T1's;
`snapshotId` equal to the cached one; every miss reason
(missing / truncated / corrupt / wrong format-version / wrong rules-version / wrong catalog stamp /
wrong lint config / wrong source hash / wrong source length) classifies as a miss and never as a
load failure. **Blocked on gap R1** — with the current surface this transcript can only be written
as `materialize → replaceCorpus(Token[]) → primeLintCache`, which is what E§3.5 refuses to enshrine
silently.

### T3 — live edit (E§10 §3 steps 2–4)

```text
Lexical commit -> WorkingFilesStore.applyPatch("chapter")  -> complete currentTokens
publisher.emit({kind:"updateChapter", bookCode, chapter:{label, tokens}, generation})
H.updateChapter({book, label:Number("3")}, ChapterInput::Tokens([...]))  -> MutationEffect
   ChapterNotFound | AmbiguousChapter | ReplacementLabelMismatch  -> republish the complete book
debounce -> H.lint()                       -> {snapshotId, packed}
pub: reject if snapshotId/generation is stale
     reconcileFindings(previous, packed, tokensByBook) -> identity-preserving findings
     FindingsStore.commitBraidSnapshot(...)
```

Asserts: unchanged findings keep object identity; fixed findings disappear; new findings land in
canonical order with no app comparator; `updateChapter` inherits the book's line ending (gap: §2.1);
a semantic no-op returns `Unchanged` and leaves dirty state and the prior publication intact; a
clean `lint()` does no rule work.

### T4 — undo to baseline, fix/format (E§10 §3, §6)

```text
undo -> store commit with the previous exact tokens -> H.updateChapter(...)
H.isDirty(Book(b))                    -> false       (undo back to saved content is clean)
H.prepareFormatPatch(scope, options)  -> Unchanged | Ready(handle)
   preview                            -> ??? gap X1: no edits/tokens exposed
H.applyPatch(handle)                  -> MutationEffect | PatchError::StaleSnapshot
   store update                       -> ??? gap X1: no returned tokens; requires a full lint()
H.lint() -> reconcile
```

### T5 — save, baseline, backup, post-save warming (E§10 §5)

```text
flushThroughGeneration(g)                  -- app barrier; braid is synchronous
H.expectedSnapshotId()                     -> id bound to the captured commit
H.toUsfm(All)                              -> ScopedOutput<String> keyed by SourceKey, exact bytes
app writes files                           -> per-book success/failure
for each successfully persisted book: H.setBaseline(BookInput::Usfm{captured bytes})
                                      H.isDirty(Book(b)) -> false
for each failed book:                 baseline unchanged, book still dirty
removed books:                        H.removeBook(b) + H.clearBaseline(b)
post-save (best effort):              wc.put("braid", key, packed bound to the PERSISTED bytes)
```

Asserts: untouched LF and CRLF corpora save byte-identically (gap: §2.1); an edit made during an
in-flight save stays dirty afterwards; partial multi-book failure advances only the written books in
disk hash, braid baseline, editor baseline, and cache eligibility; continued editing during warming
cannot cache unsaved state under the persisted bytes' hash; crash backup bytes equal explicit-save
bytes.

Cross-check against B§11.6's existing transcript: T1–T5 cover every line of it and add the warm-open
miss matrix (T2), the partial-write accounting (T5), and the EOL dimension. Recommend B§11.6 absorb
the additions.

---

## 6. Stop conditions and blockers

Per §3.1 0F, the declared stop condition is: the editor (as it will be) cannot supply unique declared
books, complete ordered sources, complete chapter token streams, or book-wide unique stable token
ids without a separately owned prerequisite.

| Precondition | Verdict |
| --- | --- |
| complete ordered sources | **met** — exact file bytes available on both platforms (desktop pays a new JS copy, §4.3 P3b) |
| complete chapter token streams | **met** — chapter commits and mirror patches always carry the whole run (§4.2) |
| book-wide unique stable token ids | **not met today**, cause is the single `linebreak-N` counter, already filed as `../candidates/editor-persistent-linebreak-token-id.md` and owned by E§10 Section 1 step 4. **No new synthesized-id site found**, so this is not a new stop condition |
| unique declared books | **not guaranteed** — neither container loader enforces unique `bookCode`, and `SourceKey` is a filename basename. Needs an editor-side manifest validation prerequisite before `replaceCorpus`; braid's refusal is all-or-nothing (§4.4) |

**Task 1 conflicts requiring owner adjudication before Phase A freezes discriminants:**

1. **EOL contract absent from braid-epic** (§2.1). Blocks the editor's own Gate 0 by its own terms,
   and makes every CRLF book permanently dirty and permanently cache-invalid.
2. **Native-host sequencing** (§3.3). Does braid v1 ship a native/Tauri host contract proof (and
   therefore amendments F and G), or is that entirely editor-owned?

**Gaps framed as PROPOSED amendments (not decided here):**

| # | Amendment | Cite |
| --- | --- | --- |
| A | Add a per-book `LineEnding` contract: on `BookInput`/`BookTokensInput`, inherited by `ChapterInput`, applied by `to_usfm` and patch-embedded USFM, folded into `SourceHash`, touching no other token field | §2.1 |
| B | Add a packed-bytes restore entrypoint (`Braid.restoreCorpus(sources, packed)` in wasm + a braid-side single-call seed), or explicitly adjudicate `replace_corpus(Usfm) + prime_lint_cache` with measurements | §2.2 |
| C | Specify chapter-scope `is_dirty`/baseline semantics when the baseline run is missing or ambiguous | §2.4 |
| D | Document that registry queries are synchronous *after* module init; braid does not promise marker facts without wasm initialization | §2.6 |
| E | Add a resident token read (`to_tokens(scope)`) and/or return tokens from `apply_patch`; expose prepared-patch content so preview and apply share one snapshot | §3.1 |
| F | Split braid's (and wire's) `wasm` feature into `serde` + `wasm` so a native IPC host gets derives without tsify/wasm-bindgen; state that the tsify `.d.ts` is the contract for both transports | §3.2 |
| G | Name the public native wire encode/decode entrypoints and add the native/Tauri consumer to §8.3 | §3.2 |
| H | Add "package imports and initialises inside a module worker; packed `Uint8Array` survives transfer" to the Phase F package gate | §3.4 |
| I | Documentation: `DuplicateBook`/`DuplicateSourceKey` refuse the entire corpus, so caller-side manifest validation is mandatory (npm README, B§13) | §4.4 |
| J | Absorb T2's cache-miss matrix, T5's partial-write accounting, and the EOL dimension into B§11.6's transcript | §5 |
| K | One sentence in B§8.1 that repeated identical finding keys arrive in a deterministic order, so braid's and the consumer's occurrence assignments cannot disagree | §2.3 |

Also recorded, not a gap: braid should decline the "upstream-recognized compatibility option" half of
E§3.8's `s5` sentence — an app-specific marker acceptance is not a general USFM fact (row M4).

## 7. Tree state

Both working trees end exactly as they started.

`usfm_onion` — `git status --short` before and after:

```text
 D plans/approved/braid-epic.md
 D plans/approved/progress-braid-epic.md
?? .claude/
?? bench-remote.sh
?? plans/approved/braid/
?? plans/approved/format-token-attribute-passthrough.md
```

`scripture-editor-proto-2` — `git status --short` before and after (all pre-existing, none touched by
this stage):

```text
 M package.json
 M product-docs/specs/project-import-and-management.md
 M public/stet/en.json
?? product-docs/specs/stet.md
```

Probe code and output live only in `agent-tmp/gate0f/`, which `.gitignore:46` excludes. No tracked
file was read-modified in either repository, no bless/update env var was set, and no
git reset/clean/checkout/stash was run anywhere.
