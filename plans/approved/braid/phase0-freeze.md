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
| 3 | `token_id_index` | `u32` | **optional** — present only when the section's `positional_ids` flag is clear (§3.2); absent for cold-parsed books |
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
| 4 | `marker_string_idx[N]:u32` | optional — present iff marker cannot be recovered unambiguously from the code + token |
| 5 | `patch_id[N]:u32` → snapshot-bound braid patch table | optional — present iff flag bit `fix` is used by any row |
| 6 | packed patch table (flat, sorted, non-overlapping insert/replace/delete edits incl. replacement token templates) | optional — present iff field 5 is present and non-empty |

Row counts: token section fields **13** (ids 0–12); finding section fields **7** (ids 0–6).

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
| 7 | `unknown-token` | `{ text: string, marker: string }` |
| 8 | `unknown-marker` | `{ marker: string }` |
| 9 | `unknown-close-marker` | `{ marker: string }` |
| 10 | `content-before-first-chapter` | `{ kind: "paragraph" \| "verse", marker: string }` |
| 12 | `note-submarker-outside-note` | `{ marker: string }` |
| 13 | `metadata-outside-target` | `{ marker: string, target: "chapter" \| "verse" }` |
| 14 | `marker-not-valid-in-context` | `{ marker: string, context: <16-arm select> }` |
| 15 | `missing-milestone-self-close` | `{ marker: string }` |
| 16 | `stray-close-marker` | `{ form: "milestone-end" \| "other", marker: string }` |
| 17 | `misnested-close-marker` | `{ has_expected: <select>, expected: string, marker: string }` |
| 18 | `implicitly-closed-marker` | `{ marker: string, closer: string }` |
| 19 | `unclosed-marker` | `{ kind: "note" \| "character" \| "other", marker: string, location: "at-eof" \| "at-boundary" }` |
| 20 | `duplicate-chapter-number` | `{ chapter: number }` |
| 21 | `duplicate-verse-number` | `{ verse: number, chapter: number }` |
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
| 4. Field ids | token section 13; finding section 7 |
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
