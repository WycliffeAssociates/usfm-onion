# Phase 0 step 3 — v1 freeze: stable discriminants, error names, public API ledger

Executes `./braid-epic.md` §10 Phase 0 step 3, per `./handoff-phase0-freeze.md`. Evidence/specification
work only — no production code, no commits, no bless/update env vars, no git reset/clean/checkout.
Base commit `c22caa9` (branch `braid`), collected 2026-07-27.

**Assignment rule (stated once, applies to every table below):** stable integers are assigned in
current-source declaration order — the order symbols appear in the cited `src/*.rs` file, or, for
not-yet-implemented wire-only concepts, the order the plan text (`braid-epic.md`) lists them —
starting at **0**, except where a value is explicitly reserved as a sentinel (noted per table). Once
assigned here, an integer is never reordered or reused; new variants are appended after the last
assigned value (tombstone-on-removal, never renumber).

---

## 1. `LintCode` → stable `u8`

Source: `src/lint_impl.rs` (32 variants, `pub enum LintCode`). Declaration order = assignment order,
starting at 0.

| `u8` | kebab code string | variant |
| ---: | --- | --- |
| 0 | `missing-id-marker` | `MissingIdMarker` |
| 1 | `duplicate-id-marker` | `DuplicateIdMarker` |
| 2 | `id-marker-not-at-file-start` | `IdMarkerNotAtFileStart` |
| 3 | `empty-paragraph` | `EmptyParagraph` |
| 4 | `missing-chapter-number` | `MissingChapterNumber` |
| 5 | `missing-verse-number` | `MissingVerseNumber` |
| 6 | `verse-is-empty` | `VerseIsEmpty` |
| 7 | `unknown-token` | `UnknownToken` |
| 8 | `unknown-marker` | `UnknownMarker` |
| 9 | `unknown-close-marker` | `UnknownCloseMarker` |
| 10 | `content-before-first-chapter` | `ContentBeforeFirstChapter` |
| 11 | `verse-outside-explicit-paragraph` | `VerseOutsideExplicitParagraph` |
| 12 | `note-submarker-outside-note` | `NoteSubmarkerOutsideNote` |
| 13 | `metadata-outside-target` | `MetadataOutsideTarget` |
| 14 | `marker-not-valid-in-context` | `MarkerNotValidInContext` |
| 15 | `missing-milestone-self-close` | `MissingMilestoneSelfClose` |
| 16 | `stray-close-marker` | `StrayCloseMarker` |
| 17 | `misnested-close-marker` | `MisnestedCloseMarker` |
| 18 | `implicitly-closed-marker` | `ImplicitlyClosedMarker` |
| 19 | `unclosed-marker` | `UnclosedMarker` |
| 20 | `duplicate-chapter-number` | `DuplicateChapterNumber` |
| 21 | `duplicate-verse-number` | `DuplicateVerseNumber` |
| 22 | `invalid-number-range` | `InvalidNumberRange` |
| 23 | `number-range-not-preceded-by-marker-expecting-number` | `NumberRangeNotPrecededByMarkerExpectingNumber` |
| 24 | `missing-whitespace-before-marker` | `MissingWhitespaceBeforeMarker` |
| 25 | `missing-horizontal-whitespace-after-marker-name` | `MissingHorizontalWhitespaceAfterMarkerName` |
| 26 | `missing-tag-end-delimiter-after-marker` | `MissingTagEndDelimiterAfterMarker` |
| 27 | `missing-content-space-after-close-marker` | `MissingContentSpaceAfterCloseMarker` |
| 28 | `verse-in-section-or-other-paragraph` | `VerseInSectionOrOtherParagraph` |
| 29 | `content-after-blank-marker` | `ContentAfterBlankMarker` |
| 30 | `invalid-book-code` | `InvalidBookCode` |
| 31 | `book-code-not-uppercase` | `BookCodeNotUppercase` |

**Policy:**

- Append-only. A removed rule leaves a tombstoned integer (never reused) and the code disappears
  from the active catalog; the wire `u8` is never renumbered to close the gap.
- The wire `u8` is **not** the sort key — canonical order sorts on the kebab string per §2.2#15
  (position first, code string second). The generated catalog must ship both the `u8` and the kebab
  string; a decoder that sorted by `u8` would silently reorder findings (Gate 0D §3.2).
- **Ceiling note (§7.7):** core's `EnabledCodes` bitmask is `u64` (`LintCode::bit() = 1u64 <<
  discriminant`, `src/lint_impl.rs:440`), capping `LintCode` at **64** variants — 32 in use, 32
  headroom. The wire `u8` (255 capacity) can never be the first ceiling to overflow; crossing 64
  requires widening core's mask (e.g. `u128`) before the wire width is ever a concern.

Row count: **32**.

---

## 2. `TokenKind` → stable `u8`, with legal-payload row

Source: `src/token.rs:237` (`TokenKind`, 9 variants) and `src/token.rs:280` (`TokenData<'a>`, same 9
variants, corrected per the Gate 0D token-variant matrix — three structurally distinct
"marker-like" shapes, not one class). Declaration order = assignment order, starting at 0.

| `u8` | `TokenKind` | span | SID | marker name | marker metadata | structural | `nested` | number | book code | attributes | attribute-source |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | `Newline` | required | sticky | — | — | — | — | — | — | — | — |
| 1 | `OptBreak` | required | sticky | — | — | — | — | — | — | — | — |
| 2 | `Marker` | required | sticky | **required** | **required** | **required** | **required** | — | — | optional | optional |
| 3 | `EndMarker` | required | sticky | **required** | **required** | **required** | **required** | — | — | **never** | **never** |
| 4 | `Milestone` | required | sticky | **required** | **required** | **required** | **never (no field)** | — | — | optional | optional |
| 5 | `MilestoneEnd` | required | sticky | — | — | — | — | — | — | — | — |
| 6 | `BookCode` | required | sticky | — | — | — | — | — | **required** | — | — |
| 7 | `Number` | required | sticky | — | — | — | — | **required** (`u32` start/end + kind) | — | — | — |
| 8 | `Text` | required | sticky | — | — | — | — | — | — | — | — |

"Sticky" = SID is inherited/propagated, not a per-variant payload field. "Never" = the variant has
no such field at all (not merely `None`) — an encoder/decoder must reject a payload combination that
puts attributes on an `EndMarker` row or `nested` on a `Milestone` row as `DecodeError::InvalidSection`
or equivalent, not silently accept it. This replaces the plan's original single "marker-like tokens
own marker metadata/attributes" class (§5.1), which the Gate 0D matrix found conflates three
distinct shapes.

`NumberRangeKind` (`src/token.rs:92`, used only inside the `Number` payload, not a discriminant of
its own row here — recorded for completeness since §7.5's fidelity bit depends on it):

| value | variant |
| ---: | --- |
| 0 | `Single` |
| 1 | `Range` |
| 2 | `Sequence` |
| 3 | `SequenceWithRange` |

Row count: **9** (`TokenKind`) + 4 (`NumberRangeKind`, recorded, not separately requested).

---

## 3. Section kinds, container flags, section flags, finding flag bits

### 3.1 TOC entry `kind` (§7.2) — token/finding section discriminant

Order given by the plan text ("token or finding stable discriminant"), starting at 0.

| value | section kind |
| ---: | --- |
| 0 | Token section |
| 1 | Finding section |

### 3.2 `positional_ids` section flag (§7.4)

**OWNER-DECISION (not resolved here — see §6 below):** the plan names one flag,
`positional_ids`, but the container defines flag fields at three levels — container header
`flags:u32` (offset 8), TOC entry `flags:u16` (offset 6, "kind-specific"), and section header
`flags:u8` (offset 9). §7.4 says only "the section's `positional_ids` flag," which reads as the
section-header `flags:u8`, but the TOC entry's flags are also documented "kind-specific" and could
carry it instead. Proposed default (not binding): section-header `flags:u8` bit 0 = `positional_ids`
(1 = ids are positional/omitted; 0 = explicit `token_id_index` column + dictionary present). Flagged
as OWNER-DECISION #5 below because the plan never states which flags field is authoritative.

### 3.3 Finding common-row `flags:u8` (§7.6, offset 14)

The plan lists the flag concepts in this order: "exact/anchor-only, no-anchor, range, related,
payload, fix, overflow." Mechanical assignment below reuses that order as bit position, low bit
first, consistent with the assignment rule applied everywhere else in this freeze (declaration/text
order → increasing integer, starting at 0):

| bit | name | meaning when set (1) |
| ---: | --- | --- |
| 0 | `fidelity` (exact vs anchor-only) | `1` = `AnchorOnly` (sequence/suffix/malformed/bridge>127 — derived from the number token's **source text** per §7.5/Gate 0D D2, never from `Sid`/`NumberRangeKind` alone); `0` = `Exact` |
| 1 | `no-anchor` | `1` = finding has **no SID at all** (e.g. `missing-id-marker`); required because `(chapter=0, verse=0)` is itself a legal front-matter SID (Gate 0D D3) and cannot double as "absent" |
| 2 | `range` | `1` = `range end` byte (offset 12) is meaningful (a verse bridge); `0` = single verse, byte is `0` |
| 3 | `related` | `1` = `related_token_idx[N]` + related-span sidecar present for this row |
| 4 | `payload` | `1` = `message_payload_idx[N]` sidecar present for this row |
| 5 | `fix` | `1` = `patch_id[N]` sidecar present for this row (`u32::MAX` sentinel still applies as the "no patch" value within the column even when this bit is 0, for column-uniformity; decoders must not depend on the bit alone) |
| 6 | `overflow` | `1` = `overflow_span[N]` sidecar (offset/len as `u32`) supersedes the row's `u16` offset/length for this finding |
| 7 | reserved | zero in v1; unknown-bit-set rejects per the general container contract |

Every current corpus finding has `length = 0` (whole-token span) and no overflow producer (Gate 0D
§2.3/§3.4), so bit 6 has zero real-corpus producers today; it is still specified because §7.6
requires the overflow path to exist, not merely be probable.

### 3.4 `token_idx` / `related_token_idx` sentinels

Per plan text, mechanical (not assigned here, restated for completeness): `token index = u32::MAX`
means anchor-only finding (§7.6); `patch_id[N] = u32::MAX` means no patch (§7.6); `sid_index =
0xffff` means no SID for that token (§7.4); `marker_descriptor_index = 0xffff` means no marker
(§7.4).

Row counts: section kind ids **2**; finding flag bits **8** (7 named + 1 reserved); sentinels **4**
(restated, not newly assigned).

---

## 4. Field ids for the §7.3 directory

The field-id space is **per section kind** (token section fields and finding section fields are
separate directories; the same `field_id` integer in a token section and a finding section names
unrelated things). Assignment order = the order §7.4/§7.6 list the fields in plan text, starting at
0. Unknown *required* field ids reject decode; unknown *optional* field ids are skipped (§7.3).

### 4.1 Token section fields

| `field_id` | name | element width | required/optional |
| ---: | --- | --- | --- |
| 0 | `kind` | `u8` | required |
| 1 | `span_start` | `u32` | required |
| 2 | `span_end` | `u32` | required |
| 3 | `token_id_index` | `u32` | **optional** — present only when the section's `positional_ids` flag is clear (§3.2); absent for cold-parsed books. Indexes field 9; every index must resolve to a **non-empty** string, because an empty id cannot be distinguished from a missing one and core's `StableTokenId` refuses to hold it |
| 4 | `sid_index` | `u16` | required (per-row column; `0xffff` sentinel encodes the per-token `None`) |
| 5 | `marker_descriptor_index` | `u16` | required (`0xffff` sentinel = no marker) |
| 6 | number records (sparse, keyed by `token_idx`: `start:u32`, `end:Option<u32>` via presence, `kind` discriminant) | `u32` fields | optional (sparse — present only for `Number`-kind rows) |
| 7 | book-code records (sparse, keyed by `token_idx`: code string index + `is_valid:bool`) | mixed | optional (sparse — present only for `BookCode`-kind rows) |
| 8 | attribute-list records (sparse, keyed by `token_idx`: attribute entries + verbatim attribute-source span) | mixed | optional (sparse — present only for `Marker`/`Milestone` rows carrying an attribute list) |
| 9 | token-id string dictionary (offsets + UTF-8 data) | mixed | **optional** — omitted together with field 3 when `positional_ids` is set |
| 10 | generic string dictionary (non-catalog strings: attribute keys/values, unknown-marker text, etc.) | mixed | required |
| 11 | marker descriptor dictionary (catalog-stamped index → recoverable metadata for unknown/known markers) | mixed | required |
| 12 | packed SID dictionary (the §7.5 eight-byte `PackedSid` records `sid_index` points into) | `8` | required (appended by the 2026-07-28 layout adjudication; count ≤ 65,535, and every non-sentinel `sid_index` must be below the count) |

### 4.2 Finding section fields

| `field_id` | name | required/optional |
| ---: | --- | --- |
| 0 | common row (fixed 16-byte record per §7.6, stored as one opaque array — not split into sub-fields at the directory level) | required |
| 1 | `related_token_idx[N]:u32` + related token-relative span | optional — present iff flag bit `related` is used by any row in the section |
| 2 | `overflow_span[N]:{offset:u32,len:u32}` | optional — present iff flag bit `overflow` is used by any row |
| 3 | `message_payload_idx[N]:u32` → typed per-code argument payloads | optional — present iff flag bit `payload` is used by any row |
| 4 | `marker_ref[N]` — tagged 8-byte marker reference (§M.3; formerly named `marker_string_idx`, redefined 2026-07-28 before implementation) | optional — present iff some row's marker cannot be recovered from the anchored token |
| 5 | `patch_id[N]:u32` → snapshot-bound braid patch table | optional — present iff flag bit `fix` is used by any row |
| 6 | packed patch table (flat, sorted, non-overlapping insert/replace/delete edits incl. replacement token templates) | optional — present iff field 5 is present and non-empty |
| 7 | finding-section string dictionary | optional — adopted by the §F.2 owner adjudication; present iff a payload-indexed field needs key/value strings |
| 8 | message payload table | optional — adopted by the §F.2 owner adjudication; present iff field 3 is present |

Row counts: token section fields **13** (ids 0–12); finding section fields **9** (ids 0–8). Fields 7–8 were adopted by the §F.2 owner adjudication after Phase B found message parameters have no byte storage.

---

## 5. `TokenFix`/patch and message-payload discriminants

### 5.1 `TokenFix` variants (source: `src/lint_impl.rs:491`, declaration order)

| value | variant | fields |
| ---: | --- | --- |
| 0 | `ReplaceToken` | `code, label, label_params, target_token_id, replacements: Vec<TokenTemplate>` |
| 1 | `DeleteToken` | `code, label, label_params, target_token_id` |
| 2 | `InsertAfter` | `code, label, label_params, target_token_id, insert: Vec<TokenTemplate>` |

Gate 0D §2.2: only variant 0 (`ReplaceToken`) has a real producer in core today (2 call sites, codes
24/25/26 — 0-indexed per table 1 above, i.e. `missing-whitespace-before-marker`,
`missing-horizontal-whitespace-after-marker-name`, `missing-tag-end-delimiter-after-marker`), always
with exactly one replacement and empty `label_params`. Variants 1/2 need hand-constructed values for
Phase B conformance tests (no lint rule or `.usfm` fixture produces them) — restated from Gate 0D
finding D4, not re-decided here.

### 5.2 Flat `TokenEdit` kinds (source: plan §5.3, `braid-epic.md:838`, declaration order — this is a
new braid-owned type, not yet in `src/`, so plan-text order is the "declaration order")

| value | variant | fields |
| ---: | --- | --- |
| 0 | `Insert` | `at: u32, tokens: Vec<OwnedToken>` |
| 1 | `Replace` | `range: Range<u32>, tokens: Vec<OwnedToken>` |
| 2 | `Delete` | `range: Range<u32>` |

`apply_patch` applies a book's edits highest-index-to-lowest so earlier positions never rebase
(§5.3) — this is an application-order rule, not a discriminant-order rule; the 0/1/2 values above are
stable identifiers only.

### 5.3 Per-code message-payload schema ids (from the 0D finding matrix, §2 of
`gate0-0d-payload-ledger.md`)

The `message_payload_idx[N]:u32` sidecar (table 4.2, field 3) indexes into a **per-code** typed
payload, keyed by the same `u8` rule code from table 1 — there is one payload shape per `LintCode`
that has non-empty `message_params`, not one flat global schema id space. Assignment: schema id =
the rule code's own `u8` (table 1); codes with zero message params (schema absent) need no payload
row at all and never populate field 3. From the Gate 0D matrix (`gate0-0d-payload-ledger.md` §2),
the codes with non-empty `message_params` and their param shapes:

| rule code (`u8`) | kebab code | message-param shape |
| ---: | --- | --- |
| 3 | `empty-paragraph` | `{ marker: string }` |
| 7 | `unknown-token` | `{ text: string }` — `marker` is the separate `LintIssue.marker` field, not a message parameter |
| 8 | `unknown-marker` | `{ marker: string }` |
| 9 | `unknown-close-marker` | `{ marker: string }` |
| 10 | `content-before-first-chapter` | `{ kind: "paragraph" \| "verse", marker: string }` |
| 12 | `note-submarker-outside-note` | `{ marker: string }` |
| 13 | `metadata-outside-target` | `{ marker: string, target: "chapter" \| "verse" }` |
| 14 | `marker-not-valid-in-context` | `{ marker: string, context: <20-value canonical SpecContext domain> }` |
| 15 | `missing-milestone-self-close` | `{ marker: string }` |
| 16 | `stray-close-marker` | `{ form: "milestone-end" }` \| `{ form: "named", marker: string }` — discriminated exact maps |
| 17 | `misnested-close-marker` | `{ has_expected: <select>, expected: string, marker: string }` |
| 18 | `implicitly-closed-marker` | `{ marker: string, closer: string }` |
| 19 | `unclosed-marker` | `{ kind: "note" \| "character" \| "other", marker: string, location: "at-eof" \| "at-boundary" }` |
| 20 | `duplicate-chapter-number` | `{ chapter: number, marker: string }` — the producer keeps the redundant marker parameter for public-map fidelity |
| 21 | `duplicate-verse-number` | `{ verse: number, chapter: number, marker: string }` — the producer keeps the redundant marker parameter for public-map fidelity |
| 22 | `invalid-number-range` | `{ found: string, verse: string, marker: string, context: string }` |
| 24 | `missing-whitespace-before-marker` | `{ marker: string }` |
| 25 | `missing-horizontal-whitespace-after-marker-name` | `{ marker: string }` |
| 26 | `missing-tag-end-delimiter-after-marker` | `{ marker: string }` |
| 27 | `missing-content-space-after-close-marker` | `{ marker: string }` |
| 28 | `verse-in-section-or-other-paragraph` | `{ category: "section" \| "other" }` |
| 29 | `content-after-blank-marker` | `{ marker: string }` |
| 30 | `invalid-book-code` | `{ code: string }` |
| 31 | `book-code-not-uppercase` | `{ code: string, uppercase: string }` — **load-bearing for remediation**: this code carries no `TokenFix`, so `uppercase` is the only encoded remedy (Gate 0D §2.2) |

Codes 0, 1, 2, 4, 5, 6, 11, 23 have empty `message_params` and never populate field 3.

Row counts: `TokenFix` **3**; `TokenEdit` **3**; per-code payload schemas **24** (of 32 codes).

---

## 6. Error-name freeze

Every variant below is listed in current-source (for `src/lint_impl.rs`-adjacent core errors,
n/a — none exist yet outside the plan) or plan-text declaration order (`braid-epic.md` §5.6, §7,
§8.1). TS tag strings follow the project's existing tagged-union convention (`{ kind: "..." }`,
camelCase, as already used for `SourceBindingError`/`TokenBindingError` in §8.1).

### 6.1 `IngestError` (plan §5.6)

| variant | TS tag (`kind`) |
| --- | --- |
| `DuplicateBook { book, sources }` | `duplicateBook` |
| `DuplicateSourceKey { source }` | `duplicateSourceKey` |
| `DuplicateTokenId { book, id }` | `duplicateTokenId` |
| `ChapterNotFound(ChapterTarget)` | `chapterNotFound` |
| `AmbiguousChapter { target, matches }` | `ambiguousChapter` |
| `ReplacementLabelMismatch { target, found }` | `replacementLabelMismatch` |
| `InvalidToken(TokenInputError)` | `invalidToken` |
| `Parse(ParseInputError)` | `parse` |

### 6.2 `DecodeError` (plan §5.6, §7)

| variant | TS tag (`kind`) |
| --- | --- |
| `Truncated` | `truncated` |
| `BadMagic` | `badMagic` |
| `UnsupportedVersion { found }` | `unsupportedVersion` |
| `UnsupportedFlags { found }` | `unsupportedFlags` |
| `InvalidToc` | `invalidToc` |
| `InvalidSection` | `invalidSection` |
| `InvalidUtf8` | `invalidUtf8` |
| `InvalidDiscriminant` | `invalidDiscriminant` |
| `OffsetOverflow` | `offsetOverflow` |
| `TooManySids { found }` | `tooManySids` |
| `ChecksumMismatch` | `checksumMismatch` |
| `CatalogMismatch` | `catalogMismatch` |
| `SourceLengthMismatch` | `sourceLengthMismatch` |
| `SourceHashMismatch` | `sourceHashMismatch` |

### 6.3 `PatchError` (plan §5.6)

| variant | TS tag (`kind`) |
| --- | --- |
| `StaleSnapshot { expected, found }` | `staleSnapshot` |
| `UnknownPatch(PatchId)` | `unknownPatch` |
| `InvalidEditOrder` | `invalidEditOrder` |
| `OverlappingEdits` | `overlappingEdits` |
| `OutOfBounds` | `outOfBounds` |
| `InvalidResult(IngestError)` | `invalidResult` |

### 6.4 `PrimeError` (plan §5.6)

| variant | TS tag (`kind`) |
| --- | --- |
| `DuplicateBook(BookId)` | `duplicateBook` |
| `InvalidFinding { book }` | `invalidFinding` |
| `InvalidPatch { book }` | `invalidPatch` |

### 6.5 `BaselineError` (plan §5.6)

| variant | TS tag (`kind`) |
| --- | --- |
| `Scope(ScopeError)` | `scope` |
| `MissingBaseline { books }` | `missingBaseline` |

### 6.6 `RestoreError` (plan §5.6, wasm composition union — not a braid/wire Rust enum, a generated
wasm-layer discriminated union over three other layers' errors)

| variant | TS tag (`kind`) | composes |
| --- | --- | --- |
| `decode` | `decode` | wire `DecodeError` |
| `sourceBinding` | `sourceBinding` | wasm `SourceBindingError` (§8.1) |
| `ingest` | `ingest` | braid `IngestError` |

Per-book stamp rejections inside a `restore_corpus`/`restoreCorpus` call are `RestoreReport.rejected`
**data**, never `RestoreError` variants (plan §5.6, restated).

### 6.7 `SourceBindingError` / `TokenBindingError` / `MaterializeError` / `ReconcileError`

Already fully specified with TS tags in plan §8.1 (`braid-epic.md:1462-1493`); restated here only for
completeness, not refrozen: `SourceBindingError` has 5 variants (`invalidBookKey`, `missingSource`,
`extraSource`, `sourceLengthMismatch`, `sourceHashMismatch`); `TokenBindingError` has 6
(`missingTokens`, `extraTokens`, `tokenCountMismatch`, `stableTokenIdMismatch`,
`sourceFingerprintMismatch`, `snapshotMismatch`); `MaterializeError` and `ReconcileError` are each a
2-arm `{decode | binding}` composition union.

### 6.8 OWNER-DECISION — undetermined variant sets (plan references the type, never lists variants)

These four types are used as `Result<_, X>` throughout §5.3/§8.2 but never defined. Proposed minimal
sets below, framed for owner sign-off — **not decided here**:

**`ScopeError`** — used by `remove_chapter`, `is_dirty`, `to_tokens`, `preview_patch` (no — that one
is `PatchError`), `to_usfm`. All of these resolve a `CorpusScope`/`ChapterTarget` against the
resident corpus without mutating it. Proposed:

| variant | TS tag | rationale |
| --- | --- | --- |
| `BookNotFound(BookId)` | `bookNotFound` | scope names a book not in the resident corpus |
| `ChapterNotFound(ChapterTarget)` | `chapterNotFound` | mirrors `IngestError`'s mutation-path variant, reused for the read path |
| `AmbiguousChapter { target, matches }` | `ambiguousChapter` | duplicate/reopened chapter labels (§9's chapter-scope-dirty rule explicitly reuses "the typed `AmbiguousChapter`-shaped `ScopeError`") |

**`LintError`** — `lint()`'s only documented failure mode is the atomicity guarantee itself ("lint
failure: mutations remain; no partial result committed"; §11.2 "injected failure commits none and
retry succeeds"). No real core rule-execution failure mode is named anywhere in the plan or Gate 0
evidence. Proposed minimal (deliberately thin, pending a real failure source):

| variant | TS tag | rationale |
| --- | --- | --- |
| `RuleExecutionFailed { book: BookId }` | `ruleExecutionFailed` | placeholder for a dirty-book's `lint_tokens` call panicking/erroring; needed only so the atomicity tests (§11.2) have something concrete to inject |

**`FormatError`** — `prepare_format_patch(scope, options)` resolves a scope, then calls core
`format`/`format_tokens`, which is not documented as fallible over well-formed resident tokens.
Proposed minimal:

| variant | TS tag | rationale |
| --- | --- | --- |
| `Scope(ScopeError)` | `scope` | scope resolution failure, composed the same way `BaselineError::Scope` composes |

**`ProjectionError`** — used by `to_usx`, `to_usj`, `to_html`. Core already has `UsjError` (§2.3 of
the 0C ledger: `src/usj/mod.rs`) and an implied `UsxError` (`src/usx.rs`); `to_html` has no core
error type in the 0C census (its functions are listed with no fallible signature). Proposed minimal:

| variant | TS tag | rationale |
| --- | --- | --- |
| `Scope(ScopeError)` | `scope` | scope resolution failure |
| `Usj(UsjError)` | `usj` | composes core's existing `UsjError` for `to_usj` |
| `Usx(UsxError)` | `usx` | composes core's existing `UsxError` (name inferred — 0C ledger row 2.3 lists `UsxError` as a public item at `src/usx.rs`) for `to_usx` |

**`EncodeError`** (plan §7, `encode_corpus`'s error type — also referenced but never enumerated;
raised here in addition to the four the handoff named, same gap pattern). §7.4/§7.5 name two known
refusal causes ("over-wide SIDs," "unknown payload") without giving variant names. Proposed minimal:

| variant | TS tag | rationale |
| --- | --- | --- |
| `TooManySids { book: BookId, found: u32 }` | `tooManySids` | encode-side mirror of `DecodeError::TooManySids` — §7.4's ">65,535 distinct SIDs" refusal must exist on the writer, not just the reader |
| `UnrepresentablePayload { book: BookId, code: u8 }` | `unrepresentablePayload` | catch-all for a finding payload/token combination the current schema version cannot encode (§7's "encoders refusing … return typed `EncodeError`, never truncate") |

Row counts: named error enums with fully-specified variants — `IngestError` **8**, `DecodeError`
**14**, `PatchError` **6**, `PrimeError` **3**, `BaselineError` **2**, `RestoreError` (composition)
**3**. OWNER-DECISION proposed variant counts: `ScopeError` **3**, `LintError` **1**, `FormatError`
**1**, `ProjectionError` **3**, `EncodeError` **2** — **5 OWNER-DECISION error types, 10 proposed
variants total**, none decided here.

---

## 7. Public API ledger (the breaking release)

This extends the Gate 0C census (`gate0-0c-api-ledger.md`) — it does **not** re-derive the 97
retained npm exports, 43 `usfm_onion_dto` items, or 423 core declarations already classified there.
Only NEW surface (braid lifecycle, wire codec, wasm `Braid` class) and the one crate DELETE are
listed. One row per export: name, owner crate, surface, status.

### 7.1 Retained (pointer to 0C, not repeated)

- **97 npm exports** (24 free functions + 2 classes + 71 Rust-derived types), **43 `usfm_onion_dto`
  items** (moving crates, declarations unchanged), **all 423 core declarations** — disposition
  **retain**, per `gate0-0c-api-ledger.md` §7. `WalkToken` + its 7 conversion helpers and
  `lint_code_variants()` are disposition **replace** (9 items), already counted in 0C, not repeated
  here.

### 7.2 Deleted

| name | owner crate | surface | status |
| --- | --- | --- | --- |
| `usfm_onion_dto` (the crate itself) | — | rust | **delete** — all 43 items move to `usfm_onion_wire::dto`; zero items are dropped (§2.2#2, 0C §10) |

### 7.3 NEW — `usfm_onion_wire` (§7)

| name | owner crate | surface | status |
| --- | --- | --- | --- |
| `encode_corpus(snapshot_id: u64, sections: &[CorpusSectionInput]) -> Result<Vec<u8>, EncodeError>` | wire | rust (native + wasm host) | new |
| `decode_borrowed<'wire,'source>(wire: &[u8], source: &str) -> Result<DecodedTokens, DecodeError>` | wire | rust (native + wasm host) | new |
| `decode_par` (native, non-wasm/rayon-gated only; Phase A step 5, conditional on spike evidence) | wire | rust (native only) | new |
| `CorpusSectionInput<'a>` | wire | rust | new |
| `DecodedTokens<'wire,'source>` | wire | rust | new |
| `EncodeError` (§6.8) | wire | rust + generated TS | new |
| `DecodeError` (§6.2) | wire | rust + generated TS | new |
| schema/discriminant constants (LintCode `u8` table, TokenKind `u8` table, section kind ids,
  field-id directory) | wire | rust + generated TS/JSON | new |
| `decodeView`, `decodeTokens`, `materialize`, `groupByBook`, `reconcileFindings` (§8.1) | js helper | js/ts | new |
| `DecodedContainer`, `SourceCorpus`, `MaterializedBook`, `MaterializedSnapshot`, `ApiResult<T,E>`,
  `SourceBindingError`, `TokenBindingError`, `MaterializeError`, `ReconcileError` (§8.1) | js helper | ts | new |
| `LineEnding` (`Lf | CrLf`, §2.2#16) | braid (feature-gated serde/tsify per §2.2#13) | rust + generated TS | new |

### 7.4 NEW — `braid` lifecycle surface (§5.2, §5.3, §5.5, §5.6)

| name | owner crate | surface | status |
| --- | --- | --- | --- |
| `Braid` (struct + all methods below) | braid | rust | new |
| `Braid::new`, `replace_corpus`, `restore_corpus`, `update_book`, `update_chapter`, `remove_book`,
  `remove_chapter`, `clear`, `update_config`, `prime_lint_cache`, `lint`, `prepare_format_patch`,
  `apply_patch`, `set_baseline`, `clear_baseline`, `diff_baseline`, `is_dirty`, `to_tokens`,
  `preview_patch`, `to_usfm`, `to_usx`, `to_usj`, `to_html`, `expected_snapshot_id` | braid | rust | new (23 methods) |
| `BraidConfig`, `MutationEffect`, `PatchId`, `PatchHandle`, `PatchPreparation`, `ScopedOutput<T>`,
  `SourceOutput<T>`, `UsjDocument` (braid-facing alias of core's), `TokenPatch`, `BookTokenPatch`,
  `TokenEdit` | braid | rust + generated TS (feature `serde`/`wasm`) | new |
| `BookInput`, `BookTokensInput`, `ChapterInput`, `ChapterLabel`, `ChapterTarget`, `CorpusInput`,
  `CorpusScope`, `SourceKey` | braid | rust + generated TS | new |
| `OwnedToken`, `StableTokenId`, `OwnedNumberInfo`, `OwnedBookCode` | **core** (§5.1 — not braid) | rust + generated TS (via wire DTO conversion) | new |
| `SnapshotId`, `LintSnapshot`, `BookLintSnapshot`, `SourceHash`, `LintPrimeInput`, `BookLintPrime`,
  `PrimeReport`, `PrimeRejection`, `PrimeRejectReason`, `CorpusRestoreInput`, `BookRestoreInput`,
  `RestoreReport` | braid | rust + generated TS | new |
| `IngestError`, `PatchError`, `PrimeError`, `BaselineError` (§6.1/6.3/6.4/6.5) | braid | rust + generated TS | new |
| `ScopeError`, `LintError`, `FormatError`, `ProjectionError` (§6.8 — OWNER-DECISION on variants,
  types themselves are new regardless) | braid | rust + generated TS | new |
| `lint_tokens`, `format_tokens`, `braid::diff_*` stateless proxies (§5.4) | braid (namespace fns) | rust + generated TS | new |

### 7.5 NEW — wasm `Braid` class and composition-root functions (§8.2, §8.3)

| name | owner crate | surface | status |
| --- | --- | --- | --- |
| `class Braid` (constructor + 23 methods mirroring §7.4, camelCase) | wasm | wasm/ts | new |
| `restoreCorpus(sources, packed)` | wasm | wasm/ts | new — composition root: decodes via wire, seeds via braid `restore_corpus`, braid itself stays wire-free |
| `RestoreError` (§6.6, generated composition union) | wasm | ts (generated, not a Rust enum) | new |
| native codec entrypoints exposed to the Tauri host under the `serde` feature (no wasm-bindgen) | braid + wire | rust (native) | new |

Row counts: retained (pointer only, not re-tallied) **97 npm + 43 dto + 423 core (9 of which flip to
replace, already counted in 0C)**; deleted **1 crate (0 items lost)**; NEW wire **~20 named
symbols/groups**; NEW braid **~45 named symbols/groups across 23 methods + ~30 types/errors**; NEW
wasm **4 named symbols/groups (class + 3)**. Exact per-symbol counts are intentionally left as
grouped rows here (mirroring the 0C ledger's own grouping-disclosure convention) rather than
exploded to one row per method/type, since every method/type name is already fully enumerated
verbatim in plan §5.3/§5.5/§5.6/§8.1/§8.2.

---

## 8. Stamp definitions

Three stamps are named in §5.5/§7.7/§9: the marker-catalog stamp, the lint-config fingerprint, and
the engine stamp. All three gate `prime_lint_cache`/`restore_corpus` acceptance (§9: "matching
lint-config fingerprint, deterministic rule/engine stamp") and the marker-catalog stamp additionally
gates whether packed marker-descriptor ordinals still mean the same thing (§7.7). **None of the
three has a fully specified input set in the plan — all three are OWNER-DECISION.**

### 8.1 Marker-catalog stamp (§7.7)

What's settled: it "invalidates sections whose marker ordinals no longer mean the same thing." What's
undetermined (OWNER-DECISION): the exact input set. Proposed minimal: a content hash (xxhash3, for
consistency with §2.1#8) over the ordered sequence of `(MarkerId, canonical name, family, kind)`
tuples from `src/marker_defs.rs`'s registry, so any addition/reorder/rename changes the stamp
regardless of crate version bump discipline. Open question: does core's crate semver alone suffice
(cheaper, but only correct if every registry change is guaranteed to bump it — not currently
enforced anywhere in the build)?

### 8.2 Lint-config fingerprint (§5.5 `LintConfigFingerprint`, §9)

What's settled: it must hash the **resolved** native `LintOptions`, not the wire tri-state form (0C
finding C7 — two equivalent configs must not fingerprint differently because one arrived as
`Option<bool>` and the other as native `bool`). What's undetermined (OWNER-DECISION): which
`LintOptions` fields participate. Proposed: `enabled_codes` (resolved to the effective `u64`
bitmask, not the wire `Option<Vec<LintCode>>`), `disabled_codes` (same resolution), `suppressed:
Vec<LintSuppression>` (order-independent hash), `allow_implicit_chapter_content_verse`. Proposed
**excluded**: `scope` — braid always lints at `LintScope::Book` (§6.4), so scope is not a
configuration axis for resident lint and including it would make every book's fingerprint
artificially path-dependent on which `LintScope` variant braid happens to construct internally.

### 8.3 Engine stamp (`LintEngineStamp`, §5.5, §9)

What's settled: it composes with the source hash and config fingerprint to gate cache acceptance,
and "a package version may participate in the stamp but is not the only proof unless the release
process guarantees every semantic change bumps it" (§5.3). What's undetermined (OWNER-DECISION): the
exact input. Proposed candidates, not chosen: (a) core crate semver alone (cheapest, but the plan's
own text doubts this is sufficient); (b) a content hash over the generated rule catalog (code
strings + params schemas from table 5.3) plus `format_version`/`rules_version` (§7.7) — catches
catalog changes even absent a version bump; (c) both, composed. Recommend (b) or (c) since (a) is
explicitly flagged by the plan as potentially insufficient on its own.

Row count: **3 stamp definitions, all 3 flagged OWNER-DECISION** on exact input composition.

---

## Summary — row counts per table

| table | row count |
| --- | ---: |
| 1. `LintCode` → `u8` | 32 |
| 2. `TokenKind` → `u8` (+ `NumberRangeKind` recorded) | 9 (+4) |
| 3. Section kinds / flags | section kind ids 2; finding flag bits 8; sentinels 4 (restated) |
| 4. Field ids | token section 13; finding section 9 |
| 5. Fix/patch/payload discriminants | `TokenFix` 3; `TokenEdit` 3; per-code payload schemas 24/32 |
| 6. Error freeze | fully-specified variants 36 across 6 enums/unions; **5 OWNER-DECISION error types, 10 proposed variants** |
| 7. API ledger | retained 97 npm + 43 dto + 423 core (pointer to 0C); deleted 1 crate/0 items; new ~20 wire + ~45 braid + 4 wasm symbol-groups |
| 8. Stamp definitions | 3 stamps, **all 3 OWNER-DECISION** on exact inputs |

## Complete OWNER-DECISION list (this document)

1. **`positional_ids` flag location** (§3.2) — which flags field (container/TOC-entry/section-header)
   carries it; proposed section-header `flags:u8` bit 0, not binding.
2. **`ScopeError` variant set** (§6.8) — proposed `BookNotFound`, `ChapterNotFound`,
   `AmbiguousChapter`.
3. **`LintError` variant set** (§6.8) — proposed single placeholder `RuleExecutionFailed { book }`;
   no real core failure mode is named anywhere in the plan or Gate 0 evidence.
4. **`FormatError` variant set** (§6.8) — proposed single `Scope(ScopeError)` composition.
5. **`ProjectionError` variant set** (§6.8) — proposed `Scope(ScopeError)`, `Usj(UsjError)`,
   `Usx(UsxError)`.
6. **`EncodeError` variant set** (§6.8, raised beyond the handoff's four named types) — proposed
   `TooManySids`, `UnrepresentablePayload`.
7. **Marker-catalog stamp input set** (§8.1) — proposed xxhash3 over ordered `(MarkerId, canonical,
   family, kind)` tuples; open question whether crate semver alone would suffice.
8. **Lint-config fingerprint input set** (§8.2) — proposed resolved `enabled_codes`/`disabled_codes`
   bitmasks + `suppressed` + `allow_implicit_chapter_content_verse`; `scope` proposed excluded.
9. **Engine stamp input set** (§8.3) — proposed catalog content hash + `format_version`/
   `rules_version`, composed with or instead of crate semver.

**9 OWNER-DECISION rows total.** None are decided in this document; all are framed for owner sign-off
before Phase A/B/C code depends on them.

## Assignment rules used (restated per the handoff's requirement)

- Every stable integer table (LintCode, TokenKind, section kind ids, field ids, TokenFix,
  TokenEdit) assigns in **current-source (or, for not-yet-implemented wire/braid types, plan-text)
  declaration order, starting at 0**, with no reserved-zero exception invoked anywhere in this
  freeze (nothing in the plan reserves integer 0 for any of these tables).
- Assignment is **append-only forever after this freeze**: a later addition gets the next unused
  integer; a removal tombstones its integer (documented as retired, never reused, never causes a
  renumber of neighbors).
- Bit-field tables (finding-row flags) assign bit 0 to the first-listed concept in the plan's own
  prose enumeration, ascending — the same "textual order → ascending integer" rule applied
  uniformly, not a separate convention invented for bits.

---

## Adjudication — 2026-07-27 (owner)

All nine OWNER-DECISION rows are decided; this appendix supersedes the "not decided here" framing
above:

1. `positional_ids` — **accepted**: section-header `flags:u8` bit 0.
2. `ScopeError` — **accepted as proposed** (`BookNotFound`, `ChapterNotFound`, `AmbiguousChapter`).
3. `LintError` — **accepted with rename**: the single placeholder variant is
   `EngineFailure { book: BookId }` (not `RuleExecutionFailed`), because the error means the engine
   failed to complete a run — never that the document linted badly (findings are success). Exists
   so §11.2's injected-failure atomicity tests have a typed channel; expected to be near-unreachable.
4. `FormatError` — **accepted as proposed** (`Scope(ScopeError)`).
5. `ProjectionError` — **accepted as proposed** (`Scope`, `Usj(UsjError)`, `Usx(UsxError)`).
6. `EncodeError` — **accepted as proposed** (`TooManySids`, `UnrepresentablePayload`).
7. Marker-catalog stamp — **accepted**: xxhash3 over the ordered registry tuples. The open
   question is answered NO: crate semver alone is insufficient because nothing enforces a bump per
   registry change.
8. Lint-config fingerprint — **accepted as proposed**, including the `scope` exclusion. Owner
   caveat recorded: whole-book lint may become chapter/verse compute with a whole-book second pass
   later; the exclusion still holds because computation grain is engine strategy (parity-gated),
   not user configuration — any strategy change that could alter results is the ENGINE stamp's job
   to invalidate, not the config fingerprint's.
9. Engine stamp — **accepted, option (c)**: catalog content hash + `format_version`/`rules_version`
   + crate semver, composed.

---

## Container-codec adjudication — 2026-07-27 (owner)

These rulings close five silences encountered while implementing the checked v1 container. They
are normative and append to the tables and API ledger above; they do not renumber an existing
wire value or error variant.

1. **Field-entry requiredness is carried on wire.** In each 16-byte field-directory entry,
   `flags:u8` bit 0 means `required`; all other v1 bits are reserved and reject when set. For a
   known field id, the bit must match that field's frozen required/optional classification. An
   unknown required field rejects. An unknown optional field is semantically skipped only after
   its id uniqueness, supported width, extent, alignment, in-section range, and non-overlap have
   all been validated.
2. **The supported fixed-width set is `{1, 2, 4, 8, 16}`.** `element_width = 0` means a
   variable/mixed record shape whose `byte_len` is authoritative and whose semantic codec owns
   internal record validation. The finding common-row field uses fixed width 16 and 16-byte
   alignment; it is not encoded as variable and is not split into directory-level sub-fields.
   Known fields with a frozen uniform width must declare that exact width.
3. **Structural encode refusal has a typed variant.** Append
   `EncodeError::InvalidSectionLayout { book: BookId, reason: LayoutRefusal }` after
   `UnrepresentablePayload`; the original two variants retain their positions. `LayoutRefusal` is
   an append-only typed payload with initial declaration-order variants `DuplicateSection`,
   `OrphanFindingSection`, `DuplicateField`, `FieldExtentMismatch`, `MissingRequiredField`,
   `PositionalIdConflict`, `SectionTooLarge`, `TooManyFields`, and `TooManySections`. The duplicate
   `(SectionKind, BookId)` refusal is the required duplicate-book/section write failure.
4. **Canonical writer order is grouped by section kind.** Emit all token sections in caller corpus
   order, followed by all finding sections in caller corpus order. Readers accept any otherwise
   valid non-overlapping TOC order; this ruling fixes deterministic writer bytes, not reader order.
5. **Decode checks map to the frozen errors as follows.** `Truncated` means validated arithmetic
   names bytes outside the enclosing buffer/section; `OffsetOverflow` means the offset/length
   arithmetic overflows or exceeds the host address range; `InvalidToc` covers TOC-level shape,
   alignment, overlap, duplicate-key, pairing, and legal-book-code rules; `InvalidSection` covers
   section-header/directory contradictions, missing or duplicate fields, fixed-width mismatch,
   and field alignment/overlap; `UnsupportedFlags` covers unknown set bits at every header/entry
   level. Corrupt input returns a typed error and must never panic.

Checksum omission is not an implicit disk-decoder mode: the normal persistent reader rejects a
zero container or section checksum. A zero checksum is accepted only through an explicitly named
unchecked/transient internal path, which still performs every structural validation and verifies
any checksum that is present.

---

## Adjudication — 2026-07-28 (owner): wire layout amendment and formatter id-minting

Accepted from the Phase A serial-encoding stop (spec promised four values with no allocated bytes):

1. Container header grows 32 → **48 bytes**: `snapshot_id: u64` at offset 32, 8 reserved zero
   bytes at 40.
2. Section header grows 48 → **64 bytes**: `source_len: u64` at offset 48 (the field
   `decode_borrowed` binds external bytes against), `catalog_stamp: u64` at offset 56 (the §7.7
   marker-catalog stamp).
3. Token field **12 = packed SID dictionary** appended: required, fixed element width 8, the
   §7.5 `PackedSid` records that `sid_index` points into.
4. Exact framing for string dictionaries, marker descriptors, and sparse number/book/attribute
   records must be frozen here (appendix or table amendment) BEFORE their columns are implemented.

Also accepted: **formatter id-minting context.** Core's formatter gains a caller-supplied minting
context for synthetic tokens (trait/closure yielding fresh `StableTokenId`s); core never invents
ids (address-agnostic, §2.1#7) and never uses randomness (std has none; determinism is a format
invariant). Braid supplies a deterministic minter (e.g. `{book}-p{patch}-{n}`), unique per book by
construction; ingest/apply validation remains the collision backstop.

Implementation note: the layout change must land as ONE commit updating this document's tables,
`schema.rs` constants, and the readers/writers together — no piecemeal drift against b08b9aa's
shipped 32/48-byte headers.

---

## Layout tables — 2026-07-28 amendment (normative)

Executes rows 1–3 of the 2026-07-28 adjudication above. These tables are the normative byte layout;
`braid-epic.md` §7.1/§7.3 are amended to match. Every field is little-endian. Offsets are fixed by
this version: a v1 producer that writes a different header length is rejected, not accommodated.

### L.1 Container header — 48 bytes (was 32)

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | magic | `[u8; 4]` | ASCII `uson` |
| 4 | format version | `u16` | `1`; any other value rejects |
| 6 | header length | `u16` | exactly `48` in v1 |
| 8 | flags | `u32` | none defined in v1; any set bit rejects |
| 12 | section count | `u32` | bounds-checked against the buffer before the TOC is allocated |
| 16 | TOC offset | `u64` | absolute, 16-byte aligned, at or after the header; `48` in canonical v1 output |
| 24 | integrity checksum | `u64` | xxhash3-64 over the whole container with these 8 bytes read as zero |
| 32 | snapshot id | `u64` | the `encode_corpus(snapshot_id, …)` argument, stored verbatim; wire never recomputes it |
| 40 | reserved | `[u8; 8]` | zero in v1; nonzero rejects, on the same "undefined bits reject" policy as flags |

### L.2 TOC entry — 32 bytes (unchanged)

Unchanged by this amendment; restated only so the three layout tables sit together. See
`braid-epic.md` §7.2.

### L.3 Section header — 64 bytes (was 48)

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | magic | `[u8; 4]` | ASCII `usos` |
| 4 | format version | `u16` | `1` |
| 6 | rules version | `u16` | rule-catalog version; exactly `0` for token sections |
| 8 | kind | `u8` | section-kind discriminant (§3.1); must equal the TOC entry's |
| 9 | flags | `u8` | kind-specific (§3.2); undefined-for-kind bits reject |
| 10 | book | `[u8; 3]` | canonical `BookId`; must equal the TOC entry's |
| 13 | reserved | `[u8; 3]` | zero in v1; nonzero rejects |
| 16 | record count | `u32` | tokens or findings in this section |
| 20 | directory count | `u16` | field entries following this header |
| 22 | directory entry size | `u16` | exactly `16` |
| 24 | source hash | `u64` | xxhash3-64 of the exact serialized book USFM; must equal the TOC entry's |
| 32 | section byte length | `u64` | header + directory + payloads; must equal the TOC entry's `byte_len` |
| 40 | integrity checksum | `u64` | xxhash3-64 over the section with these 8 bytes read as zero |
| 48 | source byte length | `u64` | exact length of the book's source bytes; `decode_borrowed` binds the caller's `&str` against this **and** the source hash before exposing any span |
| 56 | marker catalog stamp | `u64` | the §7.7 marker-catalog stamp; a mismatch means packed marker ordinals no longer mean the same thing and returns `DecodeError::CatalogMismatch` |

Both new fields are per **section**, not per container: two books in one container may legitimately
carry different source lengths, and a container assembled across a catalog change may carry sections
with different stamps, of which only the mismatched ones are rejected.

64 is a multiple of the 16-byte section alignment, so a section-relative field offset still has the
same alignment as its absolute offset — the property every field-payload alignment check relies on.

### L.4 Token field 12 — packed SID dictionary

| directory value | value |
| --- | --- |
| `field_id` | `12` |
| `element_width` | `8` (fixed; `count * 8 == byte_len` is enforced generically) |
| `flags` | `required` bit set |
| record shape | `braid-epic.md` §7.5: `book[3] | chapter:u16 | verse:u16 | delta_and_fidelity:u8` |
| count ceiling | `65,535` — the `u16` `sid_index` column minus its `0xffff` sentinel |
| index rule | every `sid_index[N]` is either `0xffff` (no SID for that token) or strictly less than `count`; anything else rejects |

The dictionary is required even when it is empty: a token section with no anchored token declares
field 12 with `count = 0`, so the presence of the field never has to be inferred from the index
column's contents.

Assignment order is **first use in token order**: an anchor's ordinal is fixed the first time a row
references it, so the bytes are a function of the token order alone and need no comparator over
hashed keys. Duplicate records are permitted by the format and avoided by the encoder, which interns
each distinct `(anchor, fidelity)` pair once — the fidelity bit is part of the record, so the same
anchor at two fidelities is two entries.

---

## Mixed-payload framing — 2026-07-28 (proposed, pending adjudication)

Executes row 4 of the 2026-07-28 adjudication: exact byte framing for the token-section payloads
whose directory ids and semantic names were frozen but whose byte shapes were not, so an independent
decoder could not be written. Rows marked **OWNER-DECISION** are genuine choices the plan does not
determine and are framed, not decided; everything else is a mechanical consequence of the plan text,
of `src/` reality, or of the container rules already frozen above.

General rules for every payload in this section:

- **No internal alignment guarantee.** These fields declare `element_width = 0`, so the container
  aligns them to one byte. Every multi-byte value inside them is read with an explicit
  little-endian load, never by casting a slice to a wider type, so alignment is not needed and is
  not promised. A future zero-copy column must declare a real fixed width to earn alignment.
- **Sub-record sizes are not constrained to `{1,2,4,8,16}`.** That set constrains a *directory*
  `element_width`, which these fields do not use. Records inside a variable payload may be any
  size; the size is fixed by this document and validated by the semantic codec.
- **Counts.** The directory entry's `count` means what each subsection below says it means, never
  "bytes". Any second count is *derived* from `byte_len` rather than stored, so the two can never
  disagree.
- **Ordering is canonical and ascending**, so a decoder reproduces order by reading order and an
  encoder needs no comparator over hashed keys.
- **Validation order** is always: checked arithmetic, then extent against `byte_len`, then
  per-record range/discriminant/index checks, then cross-record ordering. Nothing is allocated from
  a count before its bytes are proved present.
- **Error mapping** follows the 2026-07-27 container adjudication: shape/extent/index/ordering
  violations are `InvalidSection`; an out-of-table enum value is `InvalidDiscriminant`; bad string
  bytes are `InvalidUtf8`; a stamp mismatch is `CatalogMismatch`; set-but-undefined bits are
  `UnsupportedFlags`.

### D.1 UTF-8 string dictionary — token fields 9 and 10

One shape serves both the token-id dictionary (field 9) and the generic string dictionary
(field 10), and any later string dictionary.

| region | bytes | contents |
| --- | --- | --- |
| offsets | `4 * count` | `[u32; count]`, byte offset of each string within `data` |
| data | `byte_len - 4 * count` | concatenated UTF-8 bytes, no separators, no terminators |

- `count` (directory) = number of strings. `count == 0` requires `byte_len == 0`.
- String `i` is `data[offsets[i] .. offsets[i + 1]]`, where `offsets[count]` is implicitly
  `data_len`. Storing only the starts — not `count + 1` bounds — is what makes the empty dictionary
  zero bytes instead of four.
- `offsets[0]` must be `0`; offsets must be non-decreasing and at most `data_len`. Equal adjacent
  offsets mean an empty string, which is legal and required: the USFM 3.1 default-attribute
  shorthand stores an empty key.
- Each string is validated as UTF-8 individually rather than validating `data` once. A single
  whole-region check would accept offsets that split a multi-byte code point.
- An index into a dictionary is valid iff it is `< count`. There is no "absent" sentinel: a field
  that needs absence carries its own flag or omits its record, because index `0` is a legal string.
- Duplicate strings are permitted by the format and avoided by the encoder — dedupe is what makes
  the marker-name and attribute-key columns cheap — but a decoder must not assume uniqueness.

### D.2 Marker descriptor dictionary — token field 11

`marker_descriptor_index[N]:u16` (field 5) indexes this dictionary; `0xffff` means "row has no
marker", so the dictionary holds at most **65,535** entries and an encoder needing more refuses.

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | name index | `u32` | into the generic string dictionary (field 10); the marker name **as written**, without the leading backslash and with any `+` nesting prefix and numeric suffix intact |
| 4 | flags | `u8` | bit 0 = `nested`; other bits reserved and reject when set |
| 5 | reserved | `[u8; 3]` | zero |

Record size 8, declared as directory `element_width = 8`; `count` = number of distinct
descriptors.

**The wire stores no marker metadata fields at all.** `MarkerMetadata` and `StructuralMarkerInfo`
are pure functions of the marker name in `src/`: a decoder recovers them by calling
`usfm_onion::token::marker_metadata(name)` and then
`usfm_onion::marker_defs::structural_marker_info(name, metadata.kind)`. This is the §4.3 rule
(reuse core logic, never reimplement it across the boundary) applied to the descriptor, and it has
three consequences worth stating because each one is a cost the alternative design would have paid:

1. **No new stable integer tables are needed in v1** for `MarkerDefKind` (13 variants),
   `MarkerFamily` (7), `StructuralScopeKind` (13), `InlineContext` (4), or `SpecContext` (20). None
   of them ever reaches the wire, so none has to be frozen, renumber-protected, or kept in sync.
2. **`MarkerMetadata.canonical` is recoverable at all.** It is `Option<&'static str>` — a catalog
   pointer — so it *cannot* be rebuilt from arbitrary wire bytes. A name plus a matching catalog is
   the only construction that yields it. `MarkerMetadata.index` is likewise a build-unstable
   perf handle (`#[serde(skip)]` in core) and must never be encoded.
3. **The recovery is gated by the section header's `catalog_stamp`.** A stamp mismatch is
   `DecodeError::CatalogMismatch`, raised before any descriptor is resolved. The tradeoff is
   explicit: sections do not survive a catalog change, in exchange for never carrying a stale copy
   of metadata that disagrees with the engine reading it.

Unknown markers need no special case: `marker_metadata` returns all-`None` and
`structural_marker_info` returns `scope_kind: Unknown` for a name the catalog does not know, which
is exactly what the parser produced. The raw name survives in the string dictionary either way, so
"reproduce unknown markers" is satisfied without a per-descriptor unknown flag.

Canonical order: ascending `name_index`, then `nested` clear before set.

### D.3 Sparse number records — token field 6

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | token index | `u32` | row this record describes; `< record_count` |
| 4 | start | `u32` | first number |
| 8 | end | `u32` | range end; `0` when the `has_end` flag is clear |
| 12 | kind | `u8` | `NumberRangeKind` stable tag (§2) |
| 13 | flags | `u8` | bit 0 = `has_end`; other bits reserved and reject when set |
| 14 | reserved | `[u8; 2]` | zero |

Record size 16; `count` = number of `Number`-kind rows. `start`/`end` are `u32`, not `u16`, because
raw source numbers reach 999,999 in adversarial fixtures while core `Sid` saturates at 65,535 — the
token payload must not inherit the anchor's ceiling.

Validation: `token index` strictly ascending across records; every record's row must have kind tag
`Number`, and every `Number` row must have exactly one record — a sparse column that disagrees with
the kind column is `InvalidSection`, not a row silently missing its payload. `has_end` clear with a
nonzero `end` is rejected, so the absent case has exactly one encoding.

### D.4 Sparse book-code records — token field 7

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | token index | `u32` | `< record_count` |
| 4 | code index | `u32` | into the generic string dictionary; the code **as written**, which may be neither three characters nor uppercase |
| 8 | flags | `u8` | bit 0 = `is_valid`; other bits reserved and reject when set |
| 9 | reserved | `[u8; 7]` | zero |

Record size 16; `count` = number of `BookCode`-kind rows, one record each, strictly ascending.

`is_valid` is stored rather than recomputed on decode, unlike marker metadata, for two reasons:
core's `is_valid_book_code` is `pub(crate)` and not reachable from the wire crate, and the canonical
book list is **not** covered by the marker-catalog stamp — so recomputing would let a change to the
book list silently rewrite the meaning of an already-encoded token, which is the one thing the stamp
mechanism exists to prevent.

### D.5 Sparse attribute records — token field 8

Two ascending arrays in one payload. `count` (directory) = number of rows carrying an attribute
list. The attribute-entry count `M` is **derived**: `M = (byte_len - 24 * count) / 20`, and
`byte_len - 24 * count` must be a positive multiple of 20 (or zero when `count` is zero).

Row entries (24 bytes each, `count` of them):

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | token index | `u32` | `< record_count`; strictly ascending |
| 4 | first attribute | `u32` | index of this row's first attribute entry |
| 8 | attribute count | `u32` | attributes on this row; may be `0` (an empty list is distinct from no list) |
| 12 | list source offset | `u32` | byte offset into the bound book source of the verbatim whole-list attribute source; `0xffff_ffff` means **absent**, which is distinct from a present empty span (see the 2026-07-28 adjudication) |
| 16 | list source length | `u32` | length of that span; zero when the offset is the absent sentinel |
| 20 | flags | `u8` | no bit defined; reserved and rejects when set |
| 21 | reserved | `[u8; 3]` | zero |

Attribute entries (20 bytes each, `M` of them):

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | key index | `u32` | into the generic string dictionary; the **decoded** key, empty when `is_default` |
| 4 | value index | `u32` | into the generic string dictionary; the **decoded** value, with escapes resolved |
| 8 | source offset | `u32` | byte offset into the bound book source of this attribute's verbatim source |
| 12 | source length | `u32` | length of that span |
| 16 | flags | `u8` | bit 0 = `is_default`; other bits reserved and reject when set |
| 17 | reserved | `[u8; 3]` | zero |

Key and value are dictionary strings, not spans, because they are the *decoded* forms: an escaped
value differs byte-for-byte from its source. The verbatim source is kept as a span alongside them,
which is what makes the round trip lossless without storing the same text twice.

Validation: row `first attribute` values are contiguous and ascending, together covering exactly
`0..M` with no gap and no overlap — so the array partitions by row and a decoder needs no per-row
bounds search. Every source span is range-checked against the section header's `source_len` and must
fall on UTF-8 character boundaries of the bound source. `is_default` set requires the key string to
be empty, matching core's own contract for the shorthand form.

### D.6 OWNER-DECISION rows in this framing

1. **Where `nested` lives.** Proposed above: descriptor `flags` bit 0, so the dictionary keys on
   `(name, nested)`. Alternative: a new per-row token column (field id 13). Descriptor-side costs
   at most one extra dictionary entry per spelling and no per-token byte; column-side is
   semantically tidier because `nested` is a property of an occurrence, not of a marker. Note: for
   *parsed* tokens `nested` currently equals `name.starts_with('+')`, but that is a lexer property,
   not a format invariant — an `OwnedToken` a caller builds may set the two independently, so
   deriving it would be an inference rather than a decode. Not decided here.
2. **Attribute source: spans or dictionary strings.** Proposed above: spans, per §7.4's own wording
   ("verbatim attribute-source spans"), which also makes recovery zero-copy against the bound
   source. The cost: a synthetic or edited token whose attribute source is not a substring of the
   section's source cannot be encoded, and needs a typed refusal
   (`EncodeError::UnrepresentablePayload`). The alternative — string-dictionary indices — is always
   representable and four bytes smaller per attribute, but stores text the source already contains.
   Not decided here.
3. **Whether an absent attribute source is distinct from an empty one.** Core models it as
   `Option<Box<str>>`, and the framing above preserves the distinction with a presence flag. Confirm
   that the `Option` is a real semantic distinction rather than an artifact; if it is not, the flag
   bit and both span fields can be dropped from the row entry. Not decided here.
4. **The finding section's `marker_string_idx` has no dictionary to index.** Finding field 4 is a
   `u32` string index, but the frozen finding-section field table (§4.2) allocates no string
   dictionary, and a finding section is a separate section from its book's token section. Either a
   finding-section string dictionary is appended as finding field id 7, or the index is defined as
   pointing into the *token* section's field 10 for the same book — which makes a finding section
   undecodable on its own and couples the two sections' validity. Not decided here; unrelated to
   the token columns above, but it is the same class of gap and blocks the finding column work.
5. **Directory `element_width` for fields 6 and 7.** Both are now uniform records (16 bytes each),
   so they could declare fixed width 16 and let the container enforce `count * 16 == byte_len`
   generically, instead of `element_width = 0` with codec-side extent checks. Mechanical either
   way; it changes the frozen `FieldSpec` rows, so it is listed here rather than applied silently,
   and would land with the implementation commit. Field 8 stays variable, since its payload is two
   arrays of different record sizes.

---

## Adjudication — 2026-07-28 (owner): mixed-payload framing rows

Resolves the §D.6 rows. Rows 1, 2, 3, and 5 are decided; row 4's premise was rejected and replaced by
the evidence-backed freeze in §M below.

1. **`nested` location — ACCEPTED as proposed.** It is a flag on the marker descriptor record, so the
   dictionary keys on `(name, nested)`. It is not a per-token column.
2. **Attribute source — ACCEPTED as proposed:** a span into the bound source, per §7.4. An encoder
   that meets a token whose attribute source cannot bind to the section's source refuses with a
   typed `EncodeError`. That guards the formatter-synthetic edge and is expected to be unreachable
   through braid's serialize-then-encode path, where the source is generated from the tokens.
3. **Absent vs empty attribute source — ACCEPTED as distinct.** `None` and `Some("")` are different
   values and must round-trip as different values (lossless principle). Encoded as a **sentinel
   offset** (`0xffff_ffff`) rather than a presence flag bit; §D.5's row-entry table is amended above
   and its `flags` byte now defines no bit.
4. **Finding `marker_string_idx` — premise rejected, no string dictionary.** See §M.
5. **Fields 6 and 7 element width → 16 — ACCEPTED.** Freezing before implementing exists precisely so
   a not-yet-implemented row can be corrected. Applied to `FieldSpec`, so `count * 16 == byte_len` is
   now enforced by the generic container rather than by each codec.

**Extension applied under ruling 5's own argument:** token field **11** (marker descriptors) is also
a uniform array once §D.2 fixed its record at 8 bytes, so it likewise declares
`element_width = 8` instead of `0`. Only the string dictionaries (fields 9/10, an offset array
followed by character data) and the attribute records (field 8, two arrays of different record sizes)
remain genuinely mixed. Flagged rather than applied silently, on the same basis as the two rows the
owner named.

---

## M. Finding marker representation — 2026-07-28 (frozen, evidence-backed)

The §D.6 row 4 claim was that finding field 4 needs a string dictionary. That is **false**. A string
table would be needed only for a `LintIssue.marker` value that is simultaneously (a) not the anchored
token's own marker, (b) not a canonical catalog marker, and (c) not present in the bound source. No
such value exists.

### M.1 Producer census (static — `src/lint_impl.rs`)

`LintIssue.marker` is written in exactly **three** places, so the producer set is closed:

| site | how `marker` is set | producers | covered by |
| --- | --- | --- | --- |
| `issue()` `:2356` | `token.marker()` — the anchored token's own marker | every rule that does not override it | **anchored token** (zero bytes; the row's own descriptor already names it) |
| `simple_issue_with_marker()` `:2427` | caller-supplied `&str` | 5 call sites, below | **catalog ordinal** |
| `LintIssue { .. }` `:1437` | literal `"id"` | `missing-id-marker` (anchor-only) | **catalog ordinal** |

The five override call sites and what they pass:

| site | code | value | why it is a catalog marker |
| --- | --- | --- | --- |
| `:1518` | `duplicate-chapter-number` | literal `"c"` | catalog |
| `:1572` | `invalid-number-range` | literal `"v"` | catalog |
| `:1599` | `duplicate-verse-number` | literal `"v"` | catalog |
| `:1613` | `verse-is-empty` | literal `"v"` | catalog |
| `:1900` | `unknown-token` | a `[a-z0-9-]+` slice of the anchored token's **own source** | guarded by `lookup_marker(marker).kind != MarkerKind::Unknown` immediately above, so it is catalog by construction — and, being a source slice, a span would cover it too |

All four literals (`id`, `c`, `v`) are asserted to remain catalog markers by an unignored test, so a
future edit that swapped one for a non-catalog spelling fails loudly instead of silently needing a
string table.

### M.2 Corpus evidence

`cargo test -p usfm_onion_wire --test corpus -- --ignored`, over all 262 `testData/**/*.usfm`
fixtures plus `example-corpora/en_ult` and `en_ulb` (133 files): **62,948 findings, 62,945 carrying a
marker, zero requiring a representation outside the frozen three.**

| representation | findings | codes |
| --- | ---: | --- |
| anchored token | 31,928 | 21 codes, led by `unknown-marker` 13,953 and `marker-not-valid-in-context` 15,719 |
| catalog ordinal | 31,017 | `verse-is-empty` 30,947; `duplicate-verse-number` 57; `duplicate-chapter-number` 10; `missing-id-marker` 3 |
| source span | 0 | no producer today |
| fell through all three | **0** | — |

`unknown-token` and `invalid-number-range` have zero corpus occurrences — both are reachable only
through `lint_tokens(caller_tokens)`, per the 0D ledger §2.1 — so they are covered by the static
argument above rather than by a count. Note that `unknown-marker`'s 13,953 findings *are* non-catalog
marker names, and they cost nothing: they arrive through the anchored-token arm, whose name the token
row's own descriptor already carries.

### M.3 Frozen encoding — token field 4 is a tagged marker reference

Finding field id **4**, previously named `marker_string_idx`, is redefined (not renumbered) as
`marker_ref`: an optional per-row column, fixed `element_width = 8`, one record per finding row,
present only when some row needs a tag other than 0.

| offset | field | type | contract |
| ---: | --- | --- | --- |
| 0 | tag | `u8` | `0` = take the anchored token's marker; `1` = catalog ordinal; `2` = span into the bound source; `3` = explicitly absent. Any other value is `InvalidDiscriminant` |
| 1 | span length | `u8` | tag 2 only; zero otherwise. A marker name longer than 255 bytes refuses to encode rather than truncating |
| 2 | catalog ordinal | `u16` | tag 1 only; zero otherwise. Resolved through the marker catalog and therefore gated by the section header's `catalog_stamp` |
| 4 | span offset | `u32` | tag 2 only; zero otherwise. Range- and character-boundary-checked against `source_len` |

- Tag 0 is the common case and costs nothing beyond the byte itself; when *every* row is tag 0 the
  column is omitted entirely, which is what "present iff marker cannot be recovered unambiguously
  from the code + token" already meant.
- Tag 3 exists so "the finding has no marker while its anchored token does have one" stays
  encodable. Core cannot currently produce that combination — `issue()` copies the token's marker, so
  the two are absent together — but the distinction is one tag value and its absence would make the
  format lossy for a caller-supplied finding.
- Tag 2 has **no current producer**, exactly like the finding row's `overflow` flag bit. It is frozen
  anyway so that a future rule naming a non-catalog marker not on its anchored token is encodable
  without a format version bump.
- **No finding-section string dictionary is added** for the marker reference. Finding field id 7 stayed unassigned on that basis; §F.2 reopens it for message parameters, which are arbitrary strings and not covered by the marker evidence above.

---

## Adjudication — 2026-07-28 (owner): owned-token encoding and core exports

1. **Owned/live token encoding is serialize-then-encode with emitter-derived spans.** The generic
   `SerializableToken` reconstruct emitter gains a variant returning the serialized source **and**
   per-token spans as a by-product of the single emission pass it already makes — it is the only code
   that knows where a deferred attribute list lands. `OwnedToken` stays spanless as a type; spans are
   a transient encode-time artifact, so there is no mid-session span state and nothing to go stale.
   This is a cold path (once per snapshot, not per keystroke): clarity over cleverness.
2. **§7's encode API is respecified** to match. The original `CorpusSectionInput<'a>` promised a
   span-free `&[OwnedToken]` plus a `source` could be encoded directly; the token section's span
   columns are required and the concatenation of `token.source()` is not the source, so it could not.
   `braid-epic.md` §7 now models the two paths explicitly and returns the per-book source each
   section is bound to. Parsed-borrowed encoding is unchanged.
3. **`assign_ids` made public — accepted.** See the API ledger addition below.
4. **Field 11's fixed width 8 — accepted**, already applied.

### API ledger addition — new public core exports

| name | crate | surface | rationale |
| --- | --- | --- | --- |
| `usfm_onion::parse::assign_ids(&mut [Token])` | core | rust | The wire omits the token-id column for parsed books (Gate 0E measured the dictionary at 31–41% of section bytes, fully redundant for positional ids) and reproduces the ids by calling the same function parsing used. The alternatives were storing the column the layout deliberately omits, or reimplementing the rule in the wire crate — the §15 "adapter fork" footgun. |
| `usfm_onion::token::tokens_to_usfm_reconstruct_spanned(&[T]) -> (String, Vec<ReconstructedSpans>)` | core | rust | Row 1 above. One function; both entry points share one implementation so the spanned and plain emitters cannot drift, and the recorder is optional so the plain path pays nothing. |
| `usfm_onion::token::ReconstructedSpans` | core | rust | The return type of the above. Carries the token span and the attribute-list span separately, because neither is derivable from the other: a token that remembers where its list sat has it placed at that remembered distance; a positionless token falls back to its closer (or end-of-stream, if unclosed), which may be far from its marker. |
| `usfm_onion::token::OwnedToken::parsed_sid() -> Option<Sid>` | core | rust | The packed wire anchor is eight bytes built from the structured `Sid`. `OwnedToken` held it privately and exposed only the formatted spelling; re-parsing that string would fork core's formatting. |
| `usfm_onion::token::OwnedToken::attribute_offset() -> Option<BytePos>` | core | rust | Distance from the token's own end to its attribute list, per `attribute-position-fidelity.md`. Backs the placement an owned token remembers; `None` keeps the historical closer rule for positionless ingest. |
| `usfm_onion::token::SerializableToken::attribute_offset()` (defaulted) | core | rust | Lets the generic emitter read that placement from any token type. Defaulted to `None`, so no implementor outside core changes and every ingest path keeps its current behavior. |
| `usfm_onion::lint::LintCode::render_message(&MessageParams) -> String` | core | rust | Reuses the one core ICU compatibility renderer for a code's own frozen template. The wire decoder needs the same derived `LintIssue.message`; it receives no arbitrary-template renderer and therefore cannot fork catalog behavior. |

Seven new public core items total. Emitted bytes are unchanged for the parsed/borrowed path and for any
token with no remembered position; the reconstruct emitter's output changes only where it was
previously byte-shifted, which is the point of the fix.

### Emitter losslessness — resolved 2026-07-28

Originally recorded as a limit: the reconstruct emitter had one placement rule for an attribute list
("at the marker's closer"), so 19 of 395 corpus books serialized content-identically but
byte-shifted. **Fixed** by `plans/approved/attribute-position-fidelity.md`, adjudicated pre-Phase-F on
conceptual-correctness grounds: an owned token now remembers `attribute_offset`, the distance from its
own end to its list, and the emitter honours it. Byte identity now holds **395/395**, and the wire
crate's owned corpus gate asserts its divergence list is empty.

The format consequence still stands and is unchanged by the fix: an owned-token section is bound to
the **derived** source, not to any file on disk. For parse-origin tokens the two are now byte-identical,
but a section encoded from edited tokens describes what was serialized, so a caller persisting one must
treat the derived source as the authoritative pairing.

Unchanged either way: the span-based `tokens_to_usfm` was and is byte-lossless, and tokens with no
remembered position (wire DTO and editor ingest) keep the closer rule.

---

## Recorded implementation note — decoded stable ids (2026-07-28)

Core's `TokenId` is a structured positional label (`{book_code}, {index}`) and cannot hold an opaque
caller id. A decoded token section therefore returns explicit ids **alongside** the tokens rather than
inside them: `stable_ids` is `Some` exactly when the section's `positional_ids` flag is clear, holding
one validated non-empty id per row in row order. `Token::id` is filled with the positional label in
both cases; for an explicit-id section it is a derived convenience and the opaque id is the identity
that reconciliation keys on (§5.1's "reconciliation keys are `(BookId, StableTokenId)`, never
`token_idx`").

The encoder decides between the two forms by proof, not by a caller flag: it emits the id column and
dictionary only when some id differs from the positional form that stream's own `assign_ids` pass
produces. Emitting them otherwise would store a dictionary Gate 0E measured at 31–41% of section bytes
and 100% redundant.

---

## F. Finding-section framing — 2026-07-28 (STOP: two gaps, proposal pending adjudication)

Phase B's finding codec was scoped to encode the §7.6 common row and every record-aligned sidecar.
Four of the six frozen fields are fully determined and implementable as written. **Two are not**, and
both fail for the same reason the token section's mixed payloads did: the spec names a value the byte
tables allocate no storage for. Per the freeze-before-implement rule, they are proposed here rather
than guessed in code.

### F.1 Determined, no amendment needed

| field | status |
| --- | --- |
| 0 — common row | fully determined by §7.6 (16 bytes) and §3.3 (all eight flag bits) |
| 2 — `overflow_span` | fully determined: `{offset:u32, len:u32}`, 8 bytes, width 8 |
| 4 — `marker_ref` | fully determined by §M.3 (8-byte tagged record; tag 2 producerless by proof) |
| 1 — `related_token_idx` + related span | determined **up to one mechanical choice**: §7.6 says "`related_token_idx[N]:u32` plus related token-relative span" without giving the span's widths. Taken as `{token_idx:u32, offset:u16, len:u16}` = 8 bytes, width 8 — the related span is the same kind of value as the primary span in the common row and therefore takes the same widths, with the same `overflow` escape available if it ever needs more. Recorded as mechanical, not raised as a decision. |

### F.2 Finding message framing — 2026-07-28 owner adjudication

Field 3 is frozen as `message_payload_idx[N]:u32`, an index column of fixed width 4. What it indexes
has **no field id and no framing**: the finding field table ends at id 6, and §M closed the question of
a finding-section string dictionary with "none is added; finding field id 7 stays unassigned."

That conclusion was correct **for markers**, on marker evidence. It does not extend to message
parameters, which are `BTreeMap<String, String>` of arbitrary strings. Proven by counterexample rather
than argued — `message_params_can_carry_values_absent_from_the_source` in the wire crate's corpus
tests: linting `\id php Philippians` yields `book-code-not-uppercase` with
`message_params["uppercase"] == "PHP"`, a value that

- is **not** a substring of the source (the source says `php`), so no span can name it, and
- is **not** a catalog marker, so no stamp-gated ordinal can name it.

Gate 0D §2 already flagged this parameter as "load-bearing for remediation … the only encoded remedy"
for that code. 24 of 32 rule codes carry message parameters; without storage for them, those codes
cannot round-trip, which §7.6's own closing rule forbids ("Every current `LintIssue` must round-trip
semantically … Phase B amends the schema before coding; it does not drop the field").

**Owner decision:** add these two optional fields. Field 7 uses the checked §D.1
string-dictionary framing verbatim; field 8 stores generic key/value rows. This
does not create 24 physical payload structs: §5.3 remains the current-rules
validation contract for the one `MessageParams = BTreeMap<String, String>`
storage shape.

| `field_id` | name | width | required/optional |
| ---: | --- | --- | --- |
| 7 | finding-section string dictionary — the §D.1 framing verbatim (`[u32; count]` starts + concatenated UTF-8 data) | `0` (mixed) | optional — present iff any field that indexes it is present |
| 8 | message payload table | `0` (mixed) | optional — present iff field 3 is present |

Field 8 framing, mirroring §D.5's two-array shape so the second count is derived rather than stored:

- `count` (directory) = number of payload rows. `message_payload_idx[N]` indexes this array.
- Row entries, 8 bytes each: `{first_pair:u32, pair_count:u32}`. Contiguous and ascending, together
  covering exactly the pair array — the same partition rule field 8's token-section analogue uses.
- Pair entries, 8 bytes each: `{key_index:u32, value_index:u32}` into field 7.
- Pair order is the `BTreeMap`'s own key order, which is deterministic and is what makes the encoding
  reproducible without a comparator at read time.

Note this stores parameters as **key/value data**, not as 24 per-code typed structs. §5.3 assigns a
schema id per code and lists each code's param shape, which remains the *validation* contract the
checked decoder applies; it is not a reason to fork the storage 24 ways. A flat pair list round-trips
`MessageParams` byte-for-byte while its `BTreeMap` key order gives the writer canonical bytes.

**Census corrections recorded with this adjudication:** `unknown-token` is exactly
`{ text }`, because `lint_unknown_token_like` puts its marker in the separate
`LintIssue.marker` field. `duplicate-chapter-number` is `{ chapter, marker }`
and `duplicate-verse-number` is `{ verse, chapter, marker }`; their marker
parameters are public semantic data even though their current English templates
do not interpolate them. The §5.3 table and Gate 0D matrix are the source of
truth for all three corrections. `stray-close-marker` is a discriminated union:
the bare milestone-end producer emits exactly `{ form: "milestone-end" }`,
while a named close emits exactly `{ form: "named", marker }`. It is not an
optional-marker shape; checked decode selects and validates one exact branch.

`marker-not-valid-in-context.context` is the closed set of all 20 canonical
strings returned by `spec_context_name` for `SpecContext`: `scripture`,
`book-identification`, `book-headers`, `book-titles`, `book-introduction`,
`book-introduction-end-titles`, `book-chapter-label`, `chapter-content`,
`peripheral`, `peripheral-content`, `peripheral-division`, `chapter`, `verse`,
`section`, `para`, `list`, `table`, `sidebar`, `footnote`, and
`cross-reference`. The ICU template's 16 named branches plus `other` are
presentation logic, not a narrower semantic domain.

### F.3 GAP 2 — patch table framing (deferred, not proposed)

Field 6 is frozen only as prose: "a packed patch table of flat, sorted, non-overlapping
insert/replace/delete edits, including replacement token templates". No record shape, no ordering key,
no width. And its contents need more string storage than field 7 alone implies: `TokenFix` carries
`code: String`, `label: String`, `label_params: MessageParams`, `target_token_id: String`, and
`Vec<TokenTemplate>` where `TokenTemplate` is `{kind, text: String, marker: Option<String>,
sid: Option<String>}`.

Per this packet's own instruction, no framing is invented. **Findings encode without fix resolution**:
field 5 (`patch_id`) and field 6 are not emitted, and common-row flag bit 5 (`fix`) stays clear. This
is consistent with §5.3's division of labour — braid owns patch resolution and the patch table is
snapshot-bound to a braid `PatchId` that does not exist until braid does — so the natural time to
freeze this framing is the braid phase that produces it, not the wire phase that would only be storing
someone else's not-yet-defined identifiers.

Consequence to record: the `fix` payload of codes 24, 25, and 26 (`missing-whitespace-before-marker`,
`missing-horizontal-whitespace-after-marker-name`, `missing-tag-end-delimiter-after-marker` — the only
three with a producer, per Gate 0D §2.2) does not survive a wire round trip until that framing lands.
Their `LintIssue.fix` decodes as `None`. Every other field of those findings round-trips.

### F.4 What is blocked and what is not

The framing is now adjudicated. Phase B implements all six-plus-two fields in one
pass with the per-code conformance gate green from the start. Patch fields 5/6
remain deliberately deferred under §F.3; that interim omission does not weaken
the final Phase B all-fields gate.
