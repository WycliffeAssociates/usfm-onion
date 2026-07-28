# Gate 0D — semantic payload and ordering census

Executes `./braid-epic.md` §3.1 0D. Evidence only; no production code, snapshots, goldens, or
`testData/` changes. Base commit `c22caa9` (branch `braid`), collected 2026-07-27.

Evidence tools (all under `target/braid-gate0/`, out-of-tree, detached from the workspace):
`scanner/src/main.rs` (0E scanner, also used here for coverage counts),
`scanner/src/bin/probe.rs`, `scanner/src/bin/probe2.rs`. Commands, hashes, and per-file records are
in the [0E width ledger](./gate0-0e-width-ledger.md) §1.

---

## 1. Token-variant matrix

One row per `TokenKind`/`TokenData<'a>` variant (`src/token.rs:237-331`). "required" = the variant
always carries it; "—" = the variant structurally cannot carry it (there is no field).

| `TokenKind` | `TokenData` variant | source span | SID | marker name | marker metadata | structural | nested | number | book code | attributes | attribute_source | exact-USFM reconstruction needs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `Newline` | `Newline` | required | sticky (inherited) | — | — | — | — | — | — | — | — | `source` verbatim |
| `OptBreak` | `OptBreak` | required | sticky | — | — | — | — | — | — | — | — | `source` verbatim |
| `Marker` | `Marker{name,metadata,structural,nested,attrs}` | required | sticky | **required** | **required** | **required** | **required** | — | — | optional (`attrs`) | optional (`attrs.attribute_source`) | `source` + attribute-list placement by closer shape |
| `EndMarker` | `EndMarker{name,metadata,structural,nested}` | required | sticky | **required** | **required** | **required** | **required** | — | — | **—** | **—** | `source`; triggers the pending-attribute drain |
| `Milestone` | `Milestone{name,metadata,structural,attrs}` | required | sticky | **required** | **required** | **required** | **—** | — | — | optional | optional | `source` + attribute list before its own close |
| `MilestoneEnd` | `MilestoneEnd` | required | sticky | — | — | — | — | — | — | — | — | `source` verbatim |
| `BookCode` | `BookCode{code,is_valid}` | required | sticky | — | — | — | — | — | **required** (`code`+`is_valid`) | — | — | `source` verbatim |
| `Number` | `Number{start,end,kind}` | required | sticky | — | — | — | — | **required** (`start:u32`, `end:Option<u32>`, `kind`) | — | — | — | `source` verbatim — suffixes/sequences live **only** here |
| `Text` | `Text` | required | sticky | — | — | — | — | — | — | — | — | `source` verbatim |

Corpus coverage of the nine variants (counts are whole-set token totals; per-book figures in the 0E
ledger):

| variant | testData | golden | en_ult | en_ulb | synthetic |
| --- | ---: | ---: | ---: | ---: | ---: |
| `text` | 50,360 | 32 | 932,024 | 49,359 | 75,025 |
| `marker` | 49,696 | 55 | 866,134 | 79,638 | 75,699 |
| `newline` | 34,501 | 40 | 807,477 | 92,208 | 75,694 |
| `number` | 18,968 | 31 | 32,292 | 32,291 | 75,344 |
| `endMarker` | 17,111 | 4 | 793,454 | 997 | 3 |
| `milestone` | 1,455 | 0 | 922,704 | 0 | 0 |
| `milestoneEnd` | 1,114 | 0 | 934,787 | 0 | 0 |
| `bookCode` | 255 | 7 | 67 | 66 | 24 |
| `optBreak` | 2 | 0 | 9 | 0 | 0 |

All nine are exercised (testData alone covers all nine). Sub-variant coverage: `nested` markers in
23 testData files; USFM 3.1 default-attribute shorthand (`is_default`) in 29 testData files;
milestones with attribute lists throughout `en_ult` (word alignment).

`NumberRangeKind` coverage — real corpora exercise only two of four:

| kind | source form | testData | en_ult | en_ulb | synthetic | resulting core `Sid` |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `Single` | `\v 1` | 18,935 | 32,292 | 32,291 | 75,346 | exact single verse |
| `Range` | `\v 1-2` | 33 | 0 | 0 | 6 | `verse` + `verse_end_delta` |
| `Sequence` | `\v 1,3` | **0** | **0** | **0** | 2 (`s25`, `s30`) | `verse = 1`, **delta 0** |
| `SequenceWithRange` | `\v 1,3-5` / `\v 1-3,7` | **0** | **0** | **0** | 2 (`s26`, `s27`) | `verse = 1`, delta = last end − start |

Verse **suffixes** (`\v 1a`, `\v 1a-2b`) parse as `Single`/`Range` with the suffix consumed by
`consume_number_suffix` and preserved **only in the token `source`** — the semantic payload is
indistinguishable from an unsuffixed verse (`s28`, `s29`).

### 1.1 Verdict against §5.1's payload-legality rules

§5.1 states: "number tokens require `number`; book-code tokens require `book_code`; marker-like
tokens own marker metadata/attributes; other kinds reject those payloads."

| §5.1 claim | verdict | evidence |
| --- | --- | --- |
| number tokens require `number` | **correct** | `TokenData::Number` always carries `start`/`end`/`kind` |
| book-code tokens require `book_code` | **correct** | `TokenData::BookCode{code, is_valid}` — both always present |
| "marker-like tokens own marker metadata/attributes" | **wrong as one class** | three distinct shapes: `Marker` owns metadata+structural+nested+attrs; `EndMarker` owns metadata+structural+nested and **cannot** own attributes; `Milestone` owns metadata+structural+attrs and has **no `nested` field**. A single "marker-like" class either invents `nested` for milestones or permits attributes on end markers. |
| other kinds reject those payloads | **correct** | the six remaining variants are fieldless or single-payload |
| `OwnedToken.nested: bool` (non-optional) | **imprecise** | forces a value for `Milestone`, which has no such concept in core, and for the six payload-free kinds |
| `OwnedToken` omits `structural` | **safe** | the walker re-derives it: `token.structural().unwrap_or_else(\|\| derive_structural_from_marker(marker))` at `src/walker/mod.rs:516`, `:598`, `:688`, using "the same derivation parse uses" |
| `OwnedToken` omits `marker_metadata` | **safe** | no core trait reads it (`LintableToken`/`WalkableToken`/`DiffableToken`/`FormattableToken` have no accessor); wire can recompute it from the marker name via `marker_defs::lookup_marker_metadata` |
| `OwnedToken` omits `span` | **NOT safe — see §6, finding D1** | not re-derivable, and it changes canonical finding order and drops `LintIssue.span` |

---

## 2. Finding matrix — one row per `LintCode`

32 codes. Columns: category / severity / issue type (from `LintCode::category()`, `severity()`,
`issue_type()`, `src/lint_impl.rs:349-431`); message params from the ICU template
(`LintCode::template()`, `:272-347`); `fix` = whether any core site attaches a `TokenFix`;
`anchor` = what the finding is anchored to; `evidence` = the scanned set that produced it, with the
per-set count, or the synthetic/probe that did.

Anchor legend: **token** = `span` = exactly the anchor token's span and `token_id` = that token
(the universal case, §3); **anchor-only** = `span: None`, `token_id: None`.

| # | code | cat | sev | type | message params | related | fix | anchor | evidence (count) |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `missing-id-marker` | document | error | usfm | — | no | no | **anchor-only** (`marker: Some("id")`) | testData 3; probe F |
| 2 | `duplicate-id-marker` | document | error | usfm | — | yes | no | token | testData 2 |
| 3 | `id-marker-not-at-file-start` | document | error | usfm | — | no | no | token | testData 2 |
| 4 | `empty-paragraph` | structure | **warning** | usfm | `marker` | no | no | token | en_ulb 609, en_ult 42, testData 37 |
| 5 | `missing-chapter-number` | structure | error | content | — | no | no | token | **synthetic `s12`** (0 in all real sets) |
| 6 | `missing-verse-number` | structure | error | content | — | no | no | token | testData 2 |
| 7 | `verse-is-empty` | structure | error | content | — | no | no | token | en_ult 30,077; testData 870 |
| 8 | `unknown-token` | structure | error | usfm | `text` — `marker` is the separate `LintIssue.marker` field | no | no | token | **probe2 A2 only — token path only** (0 everywhere) |
| 9 | `unknown-marker` | structure | error | usfm | `marker` | no | no | token | en_ulb 13,636; testData 317 |
| 10 | `unknown-close-marker` | structure | error | usfm | `marker` | no | no | token | **synthetic `s13`** (0 in all real sets) |
| 11 | `content-before-first-chapter` | document | error | usfm | `kind` (select paragraph/verse), `marker` | no | no | token | testData 17 |
| 12 | `verse-outside-explicit-paragraph` | context | error | usfm | — | no | no | token | en_ult 614, testData 3, golden 1 |
| 13 | `note-submarker-outside-note` | context | error | usfm | `marker` | no | no | token | testData 20 |
| 14 | `metadata-outside-target` | context | error | usfm | `marker`, `target` (select chapter/verse) | no | no | token | testData 7 |
| 15 | `marker-not-valid-in-context` | context | error | usfm | `marker`, `context` (20-value canonical `SpecContext` domain; template `other` is presentation only) | no | no | token | en_ult 15,643; testData 76 |
| 16 | `missing-milestone-self-close` | structure | error | usfm | `marker` | no | no | token | testData 354 |
| 17 | `stray-close-marker` | structure | error | usfm | exact union: `{ form: "milestone-end" }` \| `{ form: "named", marker }` | no | no | token | testData 37; synthetic `s13` |
| 18 | `misnested-close-marker` | structure | error | usfm | `has_expected` (select), `expected`, `marker` | yes | no | token | testData 2 |
| 19 | `implicitly-closed-marker` | structure | error | usfm | `marker`, `closer` | yes | no | token | testData 2 |
| 20 | `unclosed-marker` | structure | error | usfm | `kind` (note/character/other), `marker`, `location` (at-eof/at-boundary) | yes | no | token | testData 16, en_ulb 2, synthetic `s14` |
| 21 | `duplicate-chapter-number` | numbering | error | content | `chapter` (number), `marker` — producer preserves the public redundant marker param | yes | no | token | testData 10 |
| 22 | `duplicate-verse-number` | numbering | error | content | `verse`, `chapter` (number), `marker` — producer preserves the public redundant marker param | yes | no | token | testData 57 |
| 23 | `invalid-number-range` | numbering | error | content | `found`, `verse`, `marker`, `context` | no | no | token | **probe A only — token path only** (0 everywhere) |
| 24 | `number-range-not-preceded-by-marker-expecting-number` | numbering | error | content | — | no | no | token | **probe A only — token path only** (0 everywhere) |
| 25 | `missing-whitespace-before-marker` | structure | error | usfm | `marker` | no | **YES** `ReplaceToken` | token | testData 42, en_ult 6 |
| 26 | `missing-horizontal-whitespace-after-marker-name` | structure | error | usfm | `marker` | no | **YES** `ReplaceToken` | token | testData 19, en_ult 13, synthetic `s12` |
| 27 | `missing-tag-end-delimiter-after-marker` | structure | error | usfm | `marker` | no | **YES** `ReplaceToken` | token | testData 11, golden 1, en_ulb 1 |
| 28 | `missing-content-space-after-close-marker` | structure | **warning** | usfm | `marker` | no | no | token | testData 391, golden 1 |
| 29 | `verse-in-section-or-other-paragraph` | context | error | usfm | `category` (select section/other) | no | no | token | testData 2, en_ult 3 |
| 30 | `content-after-blank-marker` | structure | error | usfm | `marker` | no | no | token | testData 1, en_ulb 1, synthetic `s10` |
| 31 | `invalid-book-code` | document | **warning** | usfm | `code` | no | no | token | **synthetic `s11`** (0 in all real sets) |
| 32 | `book-code-not-uppercase` | document | **warning** | usfm | `code`, `uppercase` | no | no | token | testData 1, synthetic `s24` |

### 2.1 Coverage gaps found

- **6 of 32 codes are produced by no scanned real corpus.** Three are now covered by synthetic
  USFM added under `target/braid-gate0/synthetic/`: `missing-chapter-number` (`s12`),
  `unknown-close-marker` (`s13`), `invalid-book-code` (`s11`).
- **3 of 32 codes are structurally unreachable from `lint_usfm(source)`** and exist only for the
  `lint_tokens(caller_tokens)` path — i.e. they are editor-token-shape guards. Braid's resident
  lint over `BookInput::Tokens` **can** produce them, so they are not dead:
  - `unknown-token` (`src/lint_impl.rs:1158`) requires a single `TokenKind::Text` token whose source
    is `\` + a *known* marker name + a character that is not `[a-z0-9-]`. The lexer always splits
    that into Marker+Text, so no source reaches it. Probe2 A2 confirms `\pWord`, `\p.x`, `\péx`
    fire, while `\pword`, `\p1x`, `\p-x` do not (the suffix is absorbed into the marker name).
  - `invalid-number-range` (`:1573`) requires a `Number` token whose text does not parse as a range
    and which carries no `number_info`. The lexer only mints `Number` tokens it has already parsed.
  - `number-range-not-preceded-by-marker-expecting-number` (`:1932`, `:1944`) requires a `Number`
    token whose previous significant token is not `\c/\ca/\cp/\v/\va/\vp`. `marker_payload` returns
    `numberRange` only for those six markers, so a parsed `Number` always has a valid predecessor.

### 2.2 `TokenFix` producer census — two of three variants have no producer

Core has exactly **two** `TokenFix` construction sites, both `ReplaceToken` with exactly one
replacement and an **empty** `label_params`: `prepend_ws_fix` (`src/lint_impl.rs:2385`) and
`append_ws_fix` (`:2402`). They are attached at three call sites (`:1717`, `:1792`, `:1820`) for
codes 25/26/27 only, and only when `!prefix.is_empty() && token.id().is_some()`.

| variant | core producers | corpus occurrences | consumer |
| --- | ---: | --- | --- |
| `ReplaceToken` | 2 | testData 72, en_ult 19, en_ulb 1, golden 1, synthetic 2 | `apply_token_fix` `:1048` |
| `DeleteToken` | **0** | **0 everywhere** | `apply_token_fix` `:1056` |
| `InsertAfter` | **0** | **0 everywhere** | `apply_token_fix` `:1059` |

Consequence for §11.4 ("all three current `TokenFix` variants resolved to flat patch entries"): two
variants must be exercised with **hand-constructed** `TokenFix` values, because no lint rule and no
USFM input can produce them. Recorded, not changed.

Related: `book-code-not-uppercase` carries **no** `TokenFix` despite its deterministic remedy — the
uppercase form rides in `message_params["uppercase"]` (`src/lint_impl.rs:1232-1238`; the test at
`:3144` is named `..._with_fix` but asserts only the message params). So the message-params sidecar
(§7.6 `message_payload_idx`) is **load-bearing for remediation**, not merely cosmetic.

### 2.3 Verdict against §7.6's record + sidecars

| `LintIssue` field | §7.6 carrier | verdict |
| --- | --- | --- |
| `code` | `rule code: u8` in the common row | fits — 32 of 255 used (but see 0E: core's own ceiling is 64) |
| `category`, `severity`, `issue_type`, `template` | generated stable rule catalog, keyed by code | derivable — all four are pure functions of `code` (`src/lint_impl.rs:349-431`, `:272`) |
| `message` | recreated by the JS helper from template + params | derivable — `message == render_template(template, &message_params)` by construction (`:456-459`) |
| `message_params` | `message_payload_idx[N]:u32` → typed per-code payload | fits; max 3 params, max 40 payload bytes (0E) |
| `span` | `token index:u32` + `offset in token:u16` + `length:u16` | fits — **every** finding span is exactly one whole token (§3), so `length = 0` ("whole token") always applies; `offset` is always 0 |
| `related_span` | `related_token_idx[N]:u32` + related token-relative span | fits; max 262 related findings in one book |
| `token_id` | resolved from `token_idx` against the token section | fits — `token_id` and `span` are always consistent (§3) |
| `related_token_id` | resolved from `related_token_idx` | fits — `related_span` and `related_token_id` co-occur exactly (0E: equal counts in every set) |
| `sid` | `chapter:u16` + `verse:u16` + `range end:u8` + fidelity flag | fits, **with one gap — see D3** (no encoding distinguishes "no SID at all") |
| `marker` | `marker_string_idx[N]:u32` when not recoverable | fits |
| `fix` | `patch_id[N]:u32` → braid patch table | fits; max 1 replacement token per fix, `label_params` always empty in practice |

No `LintIssue` field fails to round-trip through the §7.6 layout.

---

## 3. Canonical ordering proof

### 3.1 The sort key

`canonical_sort` (`src/lint_impl.rs:2445-2461`) sorts by, in order:

1. `span_key(span)` = `(0, start, end)` if `Some`, else `(1, u32::MAX, u32::MAX)` — spanless last;
2. **`code.code()` — the kebab-case string**, not the enum discriminant;
3. `span_key(related_span)`;
4. `token_id` (`Option<String>`);
5. `marker` (`Option<String>`);
6. `message` (`String`).

`dedupe_issues` (`:2463-2478`) runs **before** the sort and keys identity on
`(code, span, related_span, token_id)`.

### 3.2 Message text is never needed to reproduce the order

Because dedupe removes every pair equal on `(code, span, related_span, token_id)`, no two surviving
findings can tie on sort keys 1–4, so keys 5 (`marker`) and 6 (`message`) can never decide the
comparison. Verified empirically rather than argued: the scanner counts adjacent pairs in canonical
order that tie on keys 1–4.

| set | findings | adjacent pairs tying on (span, code, related_span, token_id) | pairs also tying on marker |
| --- | ---: | ---: | ---: |
| testData | 2,301 | **0** | 0 |
| golden-inputs | 3 | **0** | 0 |
| en_ult | 46,398 | **0** | 0 |
| en_ulb | 14,249 | **0** | 0 |
| synthetic | 15 | **0** | 0 |

**Proven: a JS reconciler can reproduce canonical order from `(span, rule code string, related
span, token id)` alone.** Two constraints follow, both of which the §7 layout must respect:

- **The order key is the kebab-case code *string*, not the wire `u8`.** §7.7 requires the `u8` rule
  code to be "explicit fixed integers, append-only, never enum ordinals". An append-only integer
  sequence is not alphabetical, so **sorting by the wire `u8` produces a different order than
  core**. The generated rule catalog must therefore carry each code's kebab string, and the
  reconciler must sort on it. (Today's 32 codes: enum order is not alphabetical either — e.g.
  `missing-id-marker` is discriminant 0 but sorts after `id-marker-not-at-file-start`.)
- **`token_id` is compared as a string.** Rust `String: Ord` is byte-wise UTF-8. JS `<` and
  `Array.prototype.sort` default are UTF-16 code-unit order; `localeCompare` is neither. For ASCII
  ids all three agree; for non-ASCII stable token ids UTF-8 byte order and UTF-16 code-unit order
  **diverge**. The editor already uses `localeCompare` for its own base-key sort
  (`scripture-editor-proto-2/src/app/domain/editor/annotations/normalizeFindings.ts:50`), which is
  safe there only because it sorts for occurrence assignment, not to reproduce onion's order.

### 3.3 Where deterministic occurrence is assigned

Not in onion. Core has no occurrence concept: `dedupe_issues` *removes* duplicate logical
identities rather than numbering them, so an occurrence index cannot arise from core output.

Occurrence is assigned by the **consumer**, and the editor already does it deterministically
(`normalizeFindings.ts:36-63`):

- base key = `onion:${code}:${tokenId ?? ""}:${relatedTokenId ?? ""}`;
- indices are sorted by `baseKey.localeCompare(...)` with the input index as final tiebreak;
- occurrence counters are assigned in that sorted pass, then findings are returned **in the
  caller's input order** (i.e. onion's canonical order is preserved for rendering);
- the file states the rationale: "Twins are interchangeable, so which twin gets which suffix is
  unobservable; only determinism matters", and "NO id requirement is pushed into onion or sous".

Duplicate-logical-identity cases therefore cannot reach a consumer from a single core lint pass —
core deduped them. They can only arise from *merging* two producers (onion + sous + local lint),
which is the editor's problem and already solved. §8.1's "deterministic same-key occurrence" is
satisfiable, and §8.1's claim that message text and fix payload are excluded from identity matches
the editor's existing recipe exactly.

### 3.4 Span/token consistency — the invariant that makes §7.6 addressing work

Measured over every finding in all five sets (61,166 findings total):

| property | testData | golden | en_ult | en_ulb | synthetic |
| --- | ---: | ---: | ---: | ---: | ---: |
| findings whose span lies outside every token | 0 | 0 | 0 | 0 | 0 |
| findings whose span crosses a token boundary | 0 | 0 | 0 | 0 | 0 |
| findings whose `token_id` disagrees with the token containing its span | 0 | 0 | 0 | 0 | 0 |
| findings with `span` but no `token_id` | 0 | 0 | 0 | 0 | 0 |
| findings with `token_id` but no `span` | 0 | 0 | 0 | 0 | 0 |
| findings with span == exactly one whole token | 2,298 | 3 | 46,398 | 14,249 | 15 |
| findings with a **sub-token** span | **0** | **0** | **0** | **0** | **0** |
| spanless (anchor-only) findings | 1 | 0 | 0 | 0 | 0 |

So today, for parsed tokens: a finding is either exactly one whole token, or fully anchor-only.
`(token_idx, offset=0, length=0/"whole token")` is a lossless and **order-isomorphic** re-encoding
of `span` — token order equals source order, so sorting by `token_idx` reproduces sorting by
`span.start`. That is what lets the packed record drop the byte span entirely.

---

## 4. Declared-book lint context (0D item 4)

### 4.1 What exists today

- `LintOptions` (`src/lint_impl.rs:549`) has **no** book field. There is no declared-book context
  and no `BookIdMismatch` rule anywhere in core (`grep -rn "BookIdMismatch\|declared_book" src/` →
  no match).
- The two landed rules from branch `lint-book-code-rules` are emitted in `lint_structure_rules`
  (`:1224-1247`), which already receives `&LintOptions` — the natural insertion point for a
  declared `BookId`, with no signature change.
- Both are `LintCategory::Document` (`:355`), so they are gated by
  `EnabledCodes::has` → `run_document_rules` → `LintScope::runs_document_rules()`
  (`:540`, `:782`, `:790`): they run for `Front` and `Book`, never for `Chapter`.
- `is_valid_book_code` (`src/lexer.rs:519`) is the canonical set, including the audited XXA–XXG and
  `FRT`.

### 4.2 Residency proof (probe C)

Each source parsed, then `tokens_to_usfm` compared byte-for-byte, then linted at `LintScope::Book`:

| source | tokens | `analysis.book_code` | byte-lossless | findings |
| --- | ---: | --- | --- | --- |
| `\id GEN` (valid canonical) | 12 | `Some("GEN")` | **true** | — |
| `\id ZZZ` (invalid) | 12 | `Some("ZZZ")` | **true** | `invalid-book-code` |
| `\id php` (miscased) | 12 | `Some("php")` | **true** | `book-code-not-uppercase` |
| `\id 2CO` (valid but wrong vs a declared `1CO`) | 12 | `Some("2CO")` | **true** | **none** |

**Both required residency properties hold**: an invalid `\id` and a valid-but-wrong `\id` parse to a
full token stream that re-serializes byte-identically, so nothing prevents residency. `BookId` shape
validation is irrelevant here — the `\id` payload is a `TokenData::BookCode{code, is_valid}` string,
not a `BookId`.

**The gap is confirmed and unfilled**: a valid-but-wrong `\id` produces **no finding at all**. That
is exactly what §2.2#12's `BookIdMismatch { expected, found }` is for, and nothing in core can
detect it today because core is never told the expected book.

### 4.3 No-context stateless behavior (probe D)

| scope | findings on `\id ZZZ\n\c 1\n\p\n\v 1 body\n` |
| --- | --- |
| `Book` | `["invalid-book-code"]` |
| `Front` | `["invalid-book-code"]` |
| `Chapter(1)` | `[]` |

A future declared-book context must be **optional** (`Option<BookId>`, absent by default) for
"no-context stateless lint retains current behavior" to hold — `LintOptions` has no `Default` impl
(deliberately, `:545`), so every existing caller constructs it explicitly and would keep the
`None`. A `BookIdMismatch` rule in `LintCategory::Document` inherits the `Chapter`-scope suppression
for free, matching §2.2#12's requirement that a chapter-grain caller not get book-head behavior.

Evidence only; no rule, field, or bless is proposed here.

---

## 5. Editor field usage (light-touch, read-only)

Repo: `/Users/willkelly/Documents/Work/Code/scripture-editor-proto-2` at `ded10b69`, branch
`feat/stet`. Full lifecycle exploration is Gate 0F; this is field usage only.

`LintIssue` field reads across `src/**/*.{ts,tsx}`:

| field | read? | where |
| --- | --- | --- |
| `code` | **yes** (13) | `normalizeFindings.ts:73,77`; `usfmOnionLocalization.ts` |
| `messageParams` | **yes** (12) | `formatFindingMessage.ts`, `usfmOnionLocalization.ts` |
| `tokenId` | **yes** (7) | `normalizeFindings.ts:73,82,87` — **the identity anchor** |
| `message` | **yes** (6) | localization fallback |
| `sid` | **yes** (4) | `normalizeFindings.ts:83,107` — chapter bucketing via `parseSid` |
| `fix` | **yes** (4) | `decorateFinding.tsx:76`, `lintFix.ts:141,143,181` |
| `severity` | **yes** (3) | `normalizeFindings.ts:78` |
| `relatedTokenId` | **yes** (3) | `normalizeFindings.ts:73,87` — part of the anchor key |
| `category` | **yes** (3) | derived, not passed through (`issueType === "content" ? "content" : "structure"`) |
| `issueType` | **yes** (1) | `normalizeFindings.ts:79` |
| `marker` | **yes** (2) | localization |
| `template` | **no** (0) | — |
| `span` | **no** (0) | — |
| `relatedSpan` | **no** (0) | — |

`TokenFix` fields read: `labelParams` (`usfmOnionLocalization.ts:27,204`), `targetTokenId`,
`replacements` — declared in the editor's own alias at
`src/core/domain/usfm/usfmOnionTypes.ts:160-176`.

**The editor already omits `span` from its Token type, deliberately:**

```ts
// src/core/domain/usfm/usfmOnionTypes.ts:28-37
/**
 * `span` is omitted from the app's Token. Onion emits it in book-relative
 * coordinates that don't line up with the app's chapter-scoped token streams,
 * nothing in the app needs it, and onion's own diff/revert functions ignore
 * incoming span — so tokens round-trip without it. Omitting it (rather than
 * leaving it unread) makes a stray `.span` read a compile error; ...
 */
export type Token = Omit<OnionToken, "span">;
```

Per the handoff's "a field absent from current fixtures is NOT presumed unused": `template`,
`span`, and `relatedSpan` are the three `LintIssue` fields with **zero** editor reads. `template` and
`span` are derivable (§2.3) so nothing is lost; `relatedSpan` is derivable from
`related_token_idx`. This matches §3.2#11's field list and adds no new required field.

---

## 6. Findings

### D1 — spanless resident tokens change canonical finding order (stop condition)

§5.1's `OwnedToken` has no `span`. `LintableToken::span()` defaults to `None`
(`src/lint_impl.rs:29`), and `span` is the **primary** canonical sort key. Probe2 B2 lints one
source two ways — the parsed native tokens, and the same tokens converted to an owned spanless shape
with everything else preserved (id, sid, marker, structural, number) — so `span` is the only
difference:

```
spanned  (native Token<'a>):                    spanless (owned, span = None):
  verse-is-empty                   GEN-9          missing-whitespace-before-marker  GEN-13
  missing-whitespace-before-marker GEN-13         unknown-marker                    GEN-23
  verse-is-empty                   GEN-17         verse-is-empty                    GEN-17
  verse-is-empty                   GEN-22         verse-is-empty                    GEN-22
  unknown-marker                   GEN-23         verse-is-empty                    GEN-9
```

Same 5 findings, **identical multiset, different sequence**. Reading order (byte offsets 19, 28, 36,
44, 46) becomes code-string-then-token-id-string order — and `GEN-17 < GEN-22 < GEN-9`
lexicographically, so the spanless order is not even token order.

Three consequences:

1. **§10's Phase C gate is not satisfiable as written.** It requires "resident lint equals stateless
   whole-book core lint in content **and order**". Resident lint over spanless `OwnedToken`s equals
   stateless lint over the *same spanless tokens*, but **not** over `parse(source).tokens`.
2. **Order becomes ingest-shape-dependent.** A book seeded via `BookInput::Usfm` is parsed (spans
   available); a book seeded via `BookInput::Tokens` never had spans. If `OwnedToken` drops span,
   both collapse to the spanless order — but if braid lints borrowed parse tokens for one path and
   owned tokens for the other, the same book yields two different orders. §5.5 folds only
   `(book, source_hash)` and a config/engine stamp into `SnapshotId`, so two differently-ordered
   snapshots would share an id, and §11.6's "serialize → reopen from packed cache → compare
   materialized snapshot" transcript would compare a cold-open order against a warm-update order.
3. **`LintIssue.span` is `None` for every resident finding.** The wasm `LintIssue` DTO exposes
   `span`; the editor happens not to read it (§5), but §7.6's `offset in token` / `length` /
   `overflow_span` fields lose their only source of truth.

Not a regression braid introduces — the live editor already feeds onion spanless tokens
(§5), so today's editor lint results already carry this order. But braid promises order parity
between resident and stateless lint, and that promise is what fails.

Candidate resolutions, **not decided here**: `OwnedToken` carries `span: Option<Span>` (the current
wire `Token` DTO already does, and it is the field the plan's own §7.4 `span_start`/`span_end`
columns encode); or `canonical_sort` gains a positional key ahead of the code string; or braid lints
the borrowed parse tokens and maps findings back. Each is a plan amendment.

### D2 — packed-SID fidelity is not computable from the semantic payload alone

§7.5 requires `AnchorOnly` for "sequences, suffixes, malformed designators, and bridges wider than
127". Measured against what an encoder can see:

| source | `TokenData::Number` | core `Sid` | fidelity computable from… |
| --- | --- | --- | --- |
| `\v 1-129` | `{1, Some(129), Range}` | verse 1, delta 128 | **`Sid` alone** (delta > 127) ✓ |
| `\v 1,3` | `{1, None, Sequence}` | verse 1, **delta 0** | **not `Sid`** — needs `NumberRangeKind` |
| `\v 1a` | `{1, None, Single}` | verse 1, delta 0 | **neither** — needs the token's `source` text |
| `\v 1a-2b` | `{1, Some(2), Range}` | verse 1, delta 1 | **neither** — needs `source` |

A `Sequence` SID is byte-identical to a `Single` SID, and a suffixed verse is identical to an
unsuffixed one in both `Sid` and `NumberRangeKind`. So §7.5's fidelity bit must be derived from the
number token's **source text** (available: `OwnedToken.source`, and the span columns for borrowed
decode), not from `Sid` or `number.kind`. Implementable, but §7.5 does not say so, and an encoder
written against `Sid` alone would silently mark suffixed and sequence verses `Exact`. Specification
gap, not a stop.

### D3 — no encoding distinguishes "finding has no SID at all"

`missing-id-marker` is anchor-only with `span: None`, `token_id: None`, **and `sid: None`** (probe
F). §7.6's common row has `chapter:u16` + `verse:u16` described as "canonical anchor" with a
fidelity bit in flags, and `token index = u32::MAX` for anchor-only — but `chapter = 0, verse = 0` is
a **legal** chapter-scope SID (`Sid::new(book, 0, 0)`, produced for `\id` and pre-`\c` tokens,
`src/parse/mod.rs:43`). So `(0, 0)` cannot mean both "no SID" and "book front matter". A "no anchor"
flag bit is needed. One real occurrence (testData `origin.usfm`) and one synthetic; low frequency but
unambiguously required for round-trip.

### D4 — 3 codes and 2 fix variants have no source-reachable producer

Restated from §2.1/§2.2 because it changes what Phase B's conformance tests can be built from:
`unknown-token`, `invalid-number-range`, and
`number-range-not-preceded-by-marker-expecting-number` require hand-built token streams;
`TokenFix::DeleteToken` and `TokenFix::InsertAfter` require hand-built fix values. Five of §11.4's
required cases cannot be driven from a `.usfm` fixture.

---

## 7. C1–C13 round-trip verdicts (0C carry-forward)

Verdict vocabulary: **representable** = carried by the §7 layout as specified; **needs sidecar** =
requires one of §7.6's record-aligned columns or §7.4's sparse records; **spec gap** = carryable but
§7 does not say how; **cannot round-trip** = stop.

| # | conversion (0C ledger §6) | verdict | basis |
| ---: | --- | --- | --- |
| C1 | `AttributeItem.source` → `text` rename | **representable** | pure naming; §7.4's attribute-list records carry key/value/is_default plus the verbatim attribute-source span. The rename is a wire-name choice with no data loss; wire owns both sides after absorption. |
| C2 | `TokenId{book_code, index}` → `"{book}-{index}"` string | **representable** | §7.4 `token_id_index[N]:u32` into a UTF-8 string dictionary carries any id. §5.1 keeps the `{book}-{index}` mapping for cold parse and byte-preserves caller ids. Note 0E: for cold parse the dictionary is 100% derivable and is 31–41% of a section. |
| C3 | 8-byte `Sid` → formatted `String` | **representable** | §7.5's 8-byte `PackedSid` is a *narrower* encoding than the string, and `Sid → string` is a pure function (`format_sid`). Fidelity flag: **see D2 (spec gap)**. |
| C4 | `LintIssue.template: &'static str` → `String` | **representable, derivable** | `template()` is a `const fn` of `code` (`src/lint_impl.rs:272`); §7.6 puts it in the generated catalog. Zero editor reads (§5). |
| C5 | `TokenFix` external → internal tagging | **representable** | §7.6's patch table stores flat insert/replace/delete edits; the tagging style is a JSON concern that disappears entirely in the packed form. |
| C6 | `LintScope` double serialization | **representable** | not packed at all — `LintScope` is lint *input*, folded into §5.5's `LintConfigFingerprint`, never a wire record. |
| C7 | `FormatOptions` 15 `bool` → 15 `Option<bool>` tri-state | **representable** | not packed. It is lint/format *input*; §5.5 folds config into the fingerprint. The tri-state widening is a boundary ergonomics choice with a defined collapse (`apply_opt`). Flagged only because a fingerprint must hash the **resolved** native options, not the tri-state wire form, or two equivalent configs would produce different fingerprints. |
| C8 | `DiffSkeleton<T>`/`DecisionUnit<T>` generic erased; `UnitId` → `String` | **representable** | §16 explicitly keeps diff object-shaped in v1 ("No packed diff format in v1"), and §5.3 returns `DiffSkeleton<OwnedToken>`. Nothing crosses the packed layout. |
| C9 | `attribute_source: Option<(Span, &str)>` → `Option<String>` (span dropped) | **representable** | §7.4 requires "sparse attribute-list records keyed by `token_idx`, **including verbatim attribute-source spans**" — the packed form is strictly richer than the current JSON DTO, which drops the span. Max verbatim slice 191 bytes real / 4 KB synthetic. |
| C10 | every `&'a str` → `String` (wholesale source cloning) | **representable — this is the conversion §7.4 exists to delete** | `decode_borrowed(wire, source)` re-borrows from caller-supplied bytes. 0E: token-source column would be pure `(u32, u32)` spans; the real cost measured is the token-id dictionary, not the sources. |
| C11 | 40 exhaustive `From<Native…>` enum impls | **representable** | compile-time drift guards, not data. They must survive the wire move; §7.7's stable-discriminant table replaces enum ordinals on the wire. |
| C12 | `Result<_, JsError>` → thrown exception (4 sites) | **representable** | not packed. §2.2#11 replaces this with tagged `ApiResult` for new APIs; the four pre-existing throws (`toUsj`, `toUsx`, `mergeDiffBlocks`, `revertDiffBlock`) are unchanged by this epic. |
| C13 | `to_js_value` → `toUsj(): any` | **representable** | not packed; USJ is a lossy projection (§0) with no wire section. Owner adjudicated 2026-07-27 to keep the hand-written TS as the documented exception. |

**No C-row cannot round-trip.** C3 carries the D2 spec gap; C7 carries a fingerprint caveat. The one
payload that genuinely cannot round-trip today is `attributes`/`attributeSource` through the
`FormatToken` legs — 0C finding F1, already pulled forward as
`plans/approved/format-token-attribute-passthrough.md` and deliberately not re-litigated here.

---

## 8. Stop conditions

| handoff stop condition | status |
| --- | --- |
| any semantic field, localized message argument, related address, attribute spelling, or fix payload that cannot round-trip through §7 + sidecars | **one hit — D1**: `LintIssue.span` cannot round-trip through a spanless `OwnedToken`, and canonical finding **order** is not preserved, which §10's Phase C gate requires. No other field, message argument, related address, attribute spelling, or fix payload fails. |
| any corpus maximum at/over a proposed field width, or colliding with a sentinel, without the specified overflow path | see the [0E ledger](./gate0-0e-width-ledger.md) §4 — one breach (`sid_index: u16`), synthetic-only. |
| any unbounded value with no sidecar/format-version path | 0E §4 — none in real corpora; the SID dictionary is the unbounded one. |
| any place §5.1's payload-legality table contradicts actual token variants | **one hit — §1.1**: "marker-like tokens own marker metadata/attributes" conflates three structurally different variants (`EndMarker` cannot carry attributes; `Milestone` has no `nested`). `structural` and `marker_metadata` omissions are **safe** (re-derivable); `span`'s omission is D1. |

Two specification gaps that are not stops: **D2** (fidelity bit needs the number token's source
text) and **D3** (no "finding has no SID" encoding).
