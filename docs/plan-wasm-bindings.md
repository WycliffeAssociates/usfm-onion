# Plan: wasm crate cleanup via tsify + parity benches

Status: **landed** in commits 0788a42..(latest). Phases 1–9 executed
in sequence, each verified by the golden parity harness. Six drift
bugs surfaced and were fixed along the way (see commit history for
details). `BENCH_RESULTS_WASM.md` is generated from
`benches/wasm/run.mjs`.

The remaining follow-ups (open from this work):

1. **`ParsedUsfm` lazy-parse caching.** Every method on `ParsedUsfm`
   re-parses the source. The wasm bench made this visible:
   `parse/string` and `tokens/marshal-out` cost almost the same
   (~11.8ms each), meaning the per-call FFI marshalling dominates
   over the underlying parse. A `ouroboros`/`self_cell` wrapper that
   caches `ParseResult` would speed up every source-in op.
2. **Mirror `UsjDocument` as a real Tsify type.** It's the last
   hand-written declaration in `TS_TYPES`. ~15 variants, mostly
   small. Straightforward but not trivial.
3. **Fold `AdapterToken` onto `Token` directly** if we can find a
   shape that doesn't force per-method conversion on hot lint/diff
   paths. Not blocking anything; trade-off discussed below in the
   "Phase 8" section.

Original status preserved below for context:

> Design locked; implementation is a single multi-step refactor of
> `crates/usfm_onion_wasm/src/lib.rs` plus a new `benches/wasm.rs`
> harness (or its node-driven equivalent). The walker refactor is
> already in; the marker-data work is in; the library API
> contractions from `b6094d3` (drop rayon, drop batch types, drop
> `PathBuf` constructors) are in. The wasm surface has nothing left
> to wait on.

## Why this exists

`crates/usfm_onion_wasm/src/lib.rs` is 2,613 lines of which:

- ~484 lines (lines 48–531) are a hand-written `TS_TYPES` block
  declared via `#[wasm_bindgen(typescript_custom_section)]`.
- ~340 lines (lines 533–871) are a parallel `*Value` type system
  (`TokenValue`, `LintIssueValue`, `ChapterTokenDiffValue`, …) with
  manual `#[derive(Serialize, Deserialize)]` and `rename_all =
  "camelCase"` annotations.
- ~270 lines (lines 1573–1928) are hand-written `map_*` /
  `*_to_value` converters that translate every native field into
  the parallel value type.
- ~330 lines (lines 2047–2491) are stringify pairs:
  `lint_code_str` / `parse_lint_code`, `token_kind_str` /
  `parse_token_kind`, `scope_kind_str` / `parse_scope_kind`, …
  twelve enum-to-string round-trip pairs that the TS_TYPES block
  separately declares as string-literal unions.
- ~60 lines (lines 1307–1402) are `parse_*_options` adapters that
  walk an `Option<bool>` field-by-field to populate the native
  options struct.

That's roughly **1,500 lines of pure schema bookkeeping**. Every
field on every public type appears in *four* places: the native
type in `usfm_onion::*`, the FFI value type, the TS_TYPES block,
and at least one mapping function. The only thing that goes the
other direction is the test on line 2585, which asserts the
generated `.d.ts` *contains* certain strings — a fragile shape that
doesn't catch type-level drift, only the absence of specific
substrings.

This is the same drift class the walker plan and the marker-data
plan were designed to kill: one source of truth replaced by N
hand-written restatements. The refactor we just shipped on the
Rust side (walker + marker-data + slim public surface) means *the
schema mirror is the largest remaining drift surface in the repo*.

Two concrete drifts already exist in master:

1. **`scripts/test-web-package.mjs` is calling functions that don't
   exist.** That script (line 22 onward) calls `pkg.parseContent({
   source, format })`, `pkg.intoHtml(parsed, …)`, `pkg.intoUsj`,
   `pkg.intoUsx`, `pkg.intoAst`, `pkg.usfmToTokens`,
   `pkg.classifyTokens`, `pkg.tokensToAst`, `pkg.astToTokens`,
   `pkg.lintContent`, `pkg.lintFlatTokens`, `pkg.formatContent`,
   `pkg.formatFlatTokens`, `pkg.diffUsfm`, `pkg.diffTokens`,
   `pkg.revertDiffBlock`. None of those names appear in
   `lib.rs`'s current exports. The current exports are `parse`,
   `lintUsfm`, `lintTokens`, `formatUsfm`, `formatTokens`,
   `tokensToUsfm`, `tokensToHtml`, `diffUsfm`, `diffUsfmByChapter`,
   `diffTokens`, plus the `ParsedUsfm` / `UsfmMarkerCatalog`
   classes. The test script is from an older surface and was never
   updated; `npm run test:wasm` is almost certainly failing or
   silently skipped. Confirm during step 1.
2. **The lint surface inside `lib.rs` exists in five locations.**
   `LintCode` is enumerated in `lint_code_variants()` (line 2054),
   stringified in `lint_code_str()` (line 2094), restated in the
   `TS_TYPES` union (line 167), parsed in `parse_lint_code()`
   (line 2047), and tested for membership in `lib.rs:2585`. The
   walker plan added `VerseOutsideExplicitParagraph` to all five
   without ceremony, but the next addition is one more place to
   forget.

The goal of this plan is: **one Rust definition per public type,
TypeScript and JS-facing field shapes derived automatically,
fidelity guaranteed by a round-trip parity test rather than by
human discipline.** And alongside that, a wasm bench harness that
mirrors `benches/operations.rs` so any perf regression on the wasm
target is visible.

## Why this is easier than it would have been

The library shed enough surface in the last fortnight that the
remaining wasm bindings are a small, settled set:

- **Batch APIs dropped** (`b6094d3`). The wasm crate no longer
  needs to mirror `lint_usfm_batch`, `format_usfm_batch`, or any
  rayon-driven entry — those don't exist on the native side
  anymore.
- **`PathBuf` constructors dropped** (`b6094d3`). The wasm side
  was already string-only; the parity it has to maintain shrank.
- **Attributes lifted onto the owning marker token** (`f1cfda1`).
  The FFI Token shape no longer carries a separate
  `AttributeList` token kind; `TokenValue.attributes` on the
  marker is the only path. One fewer variant.
- **Walker plan retired `export_tree`'s public footprint**
  (commits `3175186`, `6124592`). USJ/USX/HTML are still
  source-in only from JS, which means none of the walker's
  intermediate types (`WalkContext`, `ScopeFrame`, event payloads)
  ever cross the FFI boundary. Zero new wasm types from that
  refactor.
- **Marker data spec landed** (`f1cfda1`, `aa3baab`, `53e416d`).
  The `MarkerInfo` shape is now stable enough that deriving its TS
  form once and forgetting it is realistic; six months ago the
  spec was still mutating.

So tsify isn't just "less code" — it's "less code to write *and*
the target type system has stopped moving under us." That's the
right window.

## Target architecture

```
                ┌─────────────────────────────────┐
                │ usfm_onion (library)            │
                │   Token<'a>, FormatToken,       │
                │   LintResult, ChapterTokenDiff, │
                │   MarkerInfo, …                 │
                │   — borrowed types, no JS deps  │
                └─────────────────────────────────┘
                              │
                              │ owned, FFI-shaped re-exports
                              ▼
┌──────────────────────────────────────────────────────────────┐
│ usfm_onion_wasm                                              │
│                                                              │
│   #[derive(Tsify, Serialize, Deserialize)]                   │
│   #[tsify(into_wasm_abi, from_wasm_abi)]                     │
│   #[serde(rename_all = "camelCase")]                         │
│   pub struct Token { id, kind: TokenKind, … }                │
│   pub enum TokenKind { Newline, OptBreak, Marker, … }        │
│   pub enum LintCode { … }                                    │
│   pub struct LintIssue { … fix: Option<TokenFix> }           │
│   pub enum TokenFix { ReplaceToken { … }, … }                │
│   pub struct ChapterTokenDiff { … }                          │
│   pub struct MarkerInfo { … }                                │
│                                                              │
│   impl From<native::Token<'_>> for Token { … }               │
│   impl From<&Token> for native::FormatToken { … }            │
│   // (or TryFrom where validation is needed)                 │
│                                                              │
│   #[wasm_bindgen]                                            │
│   pub fn lint_tokens(tokens: Vec<Token>,                     │
│                      options: Option<LintOptions>)           │
│       -> LintResult { … }                                    │
└──────────────────────────────────────────────────────────────┘
                              │
                              │  wasm-pack
                              ▼
              ┌────────────────────────────────────┐
              │ pkg-bundler/, pkg-web/             │
              │   usfm_onion_web.d.ts (generated)  │
              │   usfm_onion_web.js                │
              │   usfm_onion_web_bg.wasm           │
              └────────────────────────────────────┘
```

Three layers, each with one job:

1. **Library types** stay untouched. They use lifetimes
   (`Token<'a>`, `AttributeItem<'a>`), they implement
   `Serialize` already for native JSON debugging, and they are
   *not* `Tsify`. Adding `Tsify` to the library would force
   wasm-bindgen as a transitive dep on the native crate — bad.
2. **Wasm FFI types** are owned mirrors with `derive(Tsify,
   Serialize, Deserialize)`. They are the canonical schema for the
   JS boundary. They live only in the wasm crate.
3. **Conversions** are `From` / `TryFrom` impls between (1) and
   (2). One direction per pair. The whole `*_to_value` /
   `parse_*` / `*_str` zoo collapses into these.

Crucially, the FFI types are *not* generated from the native
types — they're hand-written but **derive-only**. No
`Serialize`/`Deserialize` bodies, no field-by-field map functions.
The hand-written part is the field list and the conversion impl,
each of which appears exactly once.

## Tsify specifics

`tsify` 0.5.6 is already in `Cargo.toml` (line 17) but unused.
Move it from dependency-of-record to dependency-in-use.

### Derive incantation

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub kind: TokenKind,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    // …
}
```

- `into_wasm_abi` + `from_wasm_abi` let the type appear directly
  in `#[wasm_bindgen]` function signatures as `Vec<Token>` etc.,
  *without* the current `JsValue` + `serde_wasm_bindgen::from_value`
  ceremony.
- `rename_all = "camelCase"` matches what the hand-written TS_TYPES
  expects today, so the JS-facing shape is unchanged.
- `skip_serializing_if = "Option::is_none"` preserves the
  "absent" vs "null" distinction the hand-rolled value types
  already make — important because tests like
  `lint_tokens_accepts_reduced_public_token_payload_without_structural_fields`
  (line 2519) rely on consumers passing partial token payloads.

### Tagged-enum shape for `TokenFix`

The current TS_TYPES has `TokenFix` as a discriminated union with
`type: "replaceToken" | "deleteToken" | "insertAfter"`. Tsify
honors `#[serde(tag = "type", rename_all = "camelCase")]` on
enums, which produces exactly that shape. The hand-written
`TokenFixValue` enum (line 685) already uses this serde
incantation; the change is to drop the duplicated value type and
add `Tsify` to a single canonical FFI enum.

### Stringly-typed enums become real enums

The twelve `*_str` / `parse_*` pairs (lint code, lint category,
lint severity, lint issue type, token kind, number kind, scope
kind, inline context, spec context, marker category, marker kind,
diff status, …) all become:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum TokenKind {
    Newline,
    OptBreak,
    Marker,
    EndMarker,
    Milestone,
    MilestoneEnd,
    BookCode,
    Number,
    Text,
}

impl From<native::TokenKind> for TokenKind { … }
impl From<TokenKind> for native::TokenKind { … }
```

Tsify emits the TS as `type TokenKind = "newline" | "optBreak" |
…` — identical to the current hand-written union (line 51).

The `match` arms in the `From` impls are exhaustive over the
native enum, so adding a variant on the native side breaks
compilation — *that's the drift-prevention we want*, and it costs
nothing per addition.

### What does NOT use tsify

- **`ParsedUsfm` and `UsfmMarkerCatalog`** stay as
  `#[wasm_bindgen]` classes (opaque JS-side handles holding Rust
  state). They have hand-written `.d.ts` declarations today
  (`TS_TYPES` lines 486–508). After the refactor, `#[wasm_bindgen]`
  generates their `.d.ts` directly — no custom-section block
  needed. Methods that *return* tsified types pick up the right
  return type for free.
- **`VrefMap`** is `BTreeMap<String, String>` on the Rust side and
  `Record<string, string>` on the TS side. Serde+tsify handle
  this as a `js_sys::Object` ABI without manual mapping.
- **`DiffsByChapterMap`** is nested
  `BTreeMap<String, BTreeMap<u32, …>>`. Same treatment; emerges as
  `Record<string, Record<number, ChapterTokenDiff[]>>`.

### What about `UsjDocument` (the Value tree)?

USJ today returns a `serde_json::Value` mapped to `JsValue`. The
TS_TYPES block has a hand-written `UsjElement` union (line 433)
that drifts from `src/usj.rs` independently. Two options:

1. Keep the loose typing. Return `JsValue` for `to_usj()`; declare
   the TS type as `unknown` or `Record<string, Value>`. Loses
   discriminated-union typing on the JS side.
2. Mirror the USJ document shape as Tsify-derived types
   (`UsjDocument`, `UsjNode`, `UsjElement` as `#[serde(tag =
   "type", rename_all = "camelCase")]`). Adds a Rust-side type but
   gains end-to-end type safety.

Pick (2). The USJ shape is small (~15 element variants) and
stable. Adding a Rust mirror is one-time work; preserving the
discriminated-union UX on the JS side matters more for downstream
editors than the bundle-size cost.

## File layout after the refactor

```
crates/usfm_onion_wasm/src/
├── lib.rs                       — entry, only #[wasm_bindgen] fns
├── ffi/
│   ├── mod.rs                   — pub use re-exports
│   ├── token.rs                 — Token, TokenKind, AttributeItem, Span, …
│   ├── cst.rs                   — CstNode, CstDocument
│   ├── lint.rs                  — LintCode, LintIssue, LintResult, TokenFix, …
│   ├── format.rs                — FormatRule, FormatOptions, FormatResult
│   ├── html.rs                  — HtmlOptions, HtmlNoteMode, …
│   ├── diff.rs                  — DiffStatus, ChapterTokenDiff, SidBlock, …
│   ├── marker.rs                — MarkerInfo, MarkerCategory, MarkerKind, …
│   └── usj.rs                   — UsjDocument, UsjElement, …
├── convert.rs                   — From impls between native and ffi types
└── adapter.rs                   — LintableToken/DiffableToken impls for FFI Token
```

Target size for `lib.rs`: ~250 lines (the `#[wasm_bindgen]`
function bodies, each calling `into()` on inputs and `.into()` on
outputs). The 2,613 → ~250 collapse is the headline; the rest of
the LOC moves into typed modules.

## Migration phases

Each phase is **independently mergeable** and keeps the JS surface
identical (no consumer changes between phases). Verification is the
golden-output parity test described under "Fidelity" below.

### Phase 1 — Test-script repair

**Before touching any Rust**, fix `scripts/test-web-package.mjs`
to match the current actual `lib.rs` surface. Without this, the
`npm run test:wasm` smoke test is meaningless and any subsequent
phase's "tests still pass" claim is hollow.

- Read the current `lib.rs` exports.
- Rewrite the script to exercise `parse()`, `ParsedUsfm.toUsj()`,
  `.toHtml()`, `.toUsx()`, `lintUsfm()`, `formatUsfm()`,
  `diffUsfm()`, plus the standalone `tokensToUsfm` etc.
- Add an assertion that the generated `.d.ts` parses successfully
  with `tsc --noEmit` against a tiny consumer fixture — catches
  whole-file TS breakage, unlike the current `includes("…")`
  substring test (line 2585).

This phase changes no Rust code. It is preparation; everything
else assumes `npm run test:wasm` is green at start.

### Phase 2 — Golden-output parity harness

Land a test that captures the current JS-side output of every
public function on a fixed input corpus, serializes to canonical
JSON, and stores the result under `crates/usfm_onion_wasm/golden/`.

```
crates/usfm_onion_wasm/golden/
├── inputs/
│   ├── tiny.usfm
│   ├── footnote-heavy.usfm
│   ├── alignment.usfm        # acts_1_11.aligned, the deep-nesting case
│   └── full-luke.usfm
└── outputs/
    ├── tokens__tiny.json
    ├── lint__tiny.json
    ├── lint__footnote-heavy.json
    ├── usj__tiny.json
    ├── html__tiny.html
    ├── usx__tiny.xml
    ├── vref__tiny.json
    ├── diff__tiny-vs-edit.json
    ├── format__tiny.usfm
    ├── markerCatalog.json
    └── …
```

The harness is a node script: load the wasm pkg, call each
operation on each input, serialize, compare to the on-disk
golden. First run with `UPDATE_GOLDEN=1` to bake them. After that,
**any** schema or value change anywhere in the wasm crate is a CI
failure.

This is the load-bearing artifact of the whole plan. Without it,
"the refactor is byte-for-byte equivalent" is a promise we cannot
verify; with it, every subsequent phase is mechanical because the
oracle never changes.

The native side has nothing equivalent — `cargo test` covers
behavior but doesn't pin the wire format. Worth considering a
parallel native goldens harness as a follow-up, but **out of scope
for this plan**.

### Phase 3 — Migrate the stringly-typed enums

Touch only the small enum pairs first to validate the pattern:
`TokenKind`, `NumberRangeKind`, `LintSeverity`, `LintCategory`,
`LintIssueType`, `DiffStatus`, `DiffTokenChange`, `DiffUndoSide`,
`HtmlNoteMode`, `HtmlCallerStyle`, `HtmlCallerScope`.

For each:
1. Define the FFI enum in `ffi/<module>.rs` with the derive
   incantation above.
2. Write `From<native> for ffi` and `From<ffi> for native`. Reject
   the existing `parse_*(value: &str) -> Result<…>` shape — with
   real enum round-trips, the only failure mode is an absent
   variant, which is a compile error now.
3. Replace one stringify pair at a time inside the existing value
   types (`TokenValue.kind`, `LintIssueValue.severity`, …). The
   value types still exist after this phase; only the field types
   change from `String` to the new FFI enum.
4. Run golden parity. If a JSON string changes, the renaming
   doesn't match — fix the `serde(rename)` and re-run.

After phase 3: ~330 lines of stringify pairs deleted; ~150 added.
Net −180.

### Phase 4 — Migrate the struct value types

Replace `TokenValue` → `Token`, `LintIssueValue` → `LintIssue`,
`ChapterTokenDiffValue` → `ChapterTokenDiff`, etc. Each replacement
deletes one `*Value` struct, one `map_*` function, and (if it had
one) one `parse_*` function.

Order within this phase, smallest-blast-radius first:

1. **`Span`** — leaf type, used everywhere.
2. **`AttributeItem`** — used only inside `Token`.
3. **`NumberInfo`, `StructuralMarkerInfo`, `MarkerMetadata`** —
   used inside `Token`.
4. **`Token`** — the big one. Used by tokens(), cst, format,
   diff, lint, fix-apply.
5. **`CstNode`, `CstDocument`** — Token-dependent.
6. **`TokenFix`, `TokenTemplate`** — tagged-union enum + leaf.
7. **`LintSuppression`, `LintOptions`, `LintIssue`,
   `LintSummary`, `LintResult`** — bottom-up.
8. **`FormatOptions`, `FormatResult`**.
9. **`HtmlOptions`, `BuildSidBlocksOptions`**.
10. **`SidBlock`, `TokenAlignment`, `ChapterTokenDiff`**.
11. **`MarkerInfo`** (plus `MarkerCategory`, `MarkerKind`,
    `MarkerFamily`, `MarkerFamilyRole`, `NoteFamily`,
    `NoteSubkind`, `BlockBehavior`, `ClosingBehavior`,
    `SpecContext`, `InlineContext`, `StructuralScopeKind`).
12. **`UsjDocument`, `UsjElement`, `UsjNode`** — moves USJ off
    `JsValue`/`serde_json::Value`.

Each substep ends with golden parity green.

### Phase 5 — Migrate `#[wasm_bindgen]` function signatures

Replace `JsValue` parameters and returns with the FFI types
directly. The `from_js_or_default` / `to_js_value` ceremony
disappears:

```rust
// before
#[wasm_bindgen(skip_typescript, js_name = lintTokens)]
pub fn wasm_lint_tokens(tokens: JsValue, options: Option<JsValue>)
    -> Result<JsValue, JsError> {
    let tokens = parse_adapter_tokens(tokens)?;
    let options = parse_lint_options(options)?;
    to_js_value(&map_lint_result(lint_tokens(&tokens, options)))
}

// after
#[wasm_bindgen(js_name = lintTokens)]
pub fn lint_tokens(tokens: Vec<Token>, options: Option<LintOptions>)
    -> LintResult {
    let native_opts = options.map(Into::into).unwrap_or_default();
    let adapter: Vec<AdapterToken> = tokens.into_iter().map(Into::into).collect();
    native::lint_tokens(&adapter, native_opts).into()
}
```

The `skip_typescript` annotation goes away because tsify now
provides the type. Function names stay identical (`#[wasm_bindgen(js_name = …)]`)
so the JS-side surface is unchanged.

### Phase 6 — Delete the TS_TYPES block

Once every binding returns/accepts tsified types, the entire
`typescript_custom_section` const (lines 48–531) is dead code.
Delete it. The generated `.d.ts` is now fully derived from Rust.

Run golden parity one final time, then compare the on-disk
`.d.ts` before and after — the diff should be reorderings and
maybe whitespace, not type-shape changes. Worth committing the
before/after `.d.ts` as part of the PR description so reviewers
can audit the schema diff at a glance.

### Phase 7 — Delete the value types and converters

`SpanValue`, `TokenValue`, `LintIssueValue`, `ChapterTokenDiffValue`,
… are now orphaned. So are `map_token`, `map_lint_issue`,
`token_value_to_format_token`, `parse_token_kind`, `parse_lint_code`,
`lint_code_str`, … the whole zoo. Delete in one mechanical sweep;
the compiler tells you when you've missed one.

### Phase 8 — Adapter token cleanup

`AdapterToken` (line 873) is the bridge from FFI-side tokens to
the native `LintableToken` and `DiffableToken` traits. After
phase 4 the FFI `Token` type owns enough data to implement those
traits directly, removing the adapter step. Move the trait impls
onto `Token` itself; delete `AdapterToken`,
`parse_adapter_tokens`, `parse_adapter_tokens_from_values`,
`token_value_to_adapter`, `map_adapter_token`, and
`map_adapter_diffs`.

The walker plan's "internal `WalkableToken` trait" question
(`plan-walker-architecture.md` line 112) is unaffected by this
change — `WalkableToken` is library-internal and doesn't cross
the FFI boundary.

## Fidelity — the only thing that matters

Throughout phases 3–8, the golden parity harness (phase 2) is the
acceptance criterion. The refactor is correct iff:

1. Every existing golden output byte-matches after the refactor.
2. The generated `.d.ts` describes a TS surface that compiles
   against the test consumer fixture without changes.
3. `npm run test:wasm` passes both bundler and web targets.
4. Every native test still passes (the wasm crate's `#[cfg(test)]`
   block at line 2497 is updated as types move, but the assertions
   stay).

If any of these breaks, the phase is incorrect — not "close
enough." This is the same fidelity contract that the walker plan
held the CST migration to (`cst_roundtrips_all_usfm_sources`); the
golden harness plays the role of that test for the wasm wire.

### Subtleties to test explicitly

- **`Option::None` vs missing field.** Today's value types use
  `skip_serializing_if = "Option::is_none"` so a missing optional
  serializes as field-absent (not `null`). Tsify must reproduce
  this — both for output (consumers checking `if (issue.fix)`
  rather than `if (issue.fix !== null)`) and for input
  (consumers passing partial tokens, per the test at line 2519).
- **`Vec::is_empty` skip.** `TokenValue.attributes` skips when
  empty (line 603). Same requirement on the new `Token` type.
- **Tagged-union string variants.** `TokenFix` has `"replaceToken"`,
  not `"replace_token"`. Verify the `rename_all` cascades into
  variant names, not just field names — Tsify's behavior matches
  serde's here, but the parity test confirms it.
- **`BTreeMap` ordering.** Iteration order of `message_params`,
  `by_category`, `by_severity`, `by_issue_type` must stay
  alphabetic. `BTreeMap` guarantees this on the Rust side;
  `serde_wasm_bindgen` preserves insertion order on the JS object.
  Tsify uses the same backend, so this is preserved — but assert it
  in the harness (sort-stability test) rather than trusting it.
- **String-keyed numeric maps.** `DiffsByChapterMap` is
  `BTreeMap<String, BTreeMap<u32, …>>`. The current code emits the
  inner key as a JSON number; JS receives a `Record<number, …>`
  whose runtime keys are strings (JS objects don't have number
  keys). Tsify generates `Record<number, …>` which matches today's
  TS_TYPES. Verify in the harness.
- **`UsjDocument` lossless round-trip.** USJ is an export-only
  format on the native side — there's no `usj_to_usfm` — so the
  parity test for USJ only needs equality of generation, not
  round-trip.

## Equivalent wasm benchmarks

Goal: every operation benched in `benches/operations.rs` (native)
gets a wasm counterpart with the same throughput-per-byte
measurement, so wasm-side perf regressions are visible at parity
with native ones.

### Why this is non-trivial

Criterion (used by native benches) doesn't compile to
`wasm32-unknown-unknown`. The standard playbook is:

1. **`wasm-bindgen-bench`**: marketed for this; less mature than
   criterion.
2. **`criterion` with `cfg(target_arch = "wasm32")` shim**: stub
   `Instant` via `js_sys::Date::now()`; works but is rough.
3. **Node-driven harness**: load the wasm pkg in node, time each
   call with `performance.now()`, aggregate. Most flexible,
   matches the test-web-package.mjs harness's environment.

**Pick (3).** It mirrors the way real consumers will use the
package, doesn't require a Rust-side bench harness, and reuses the
phase-2 fixture corpus. The bench output is a markdown table that
parallels `BENCH_RESULTS.md`.

### Harness shape

```
benches/wasm/
├── run.mjs                    # node entry; reads corpus from env
├── corpus.mjs                 # shared with test-web-package.mjs
└── report.mjs                 # markdown formatter
```

Top-level scripts:

```bash
npm run bench:wasm                    # bundler target, default corpus
USFM_BENCH_CORPORA=examples.bsb npm run bench:wasm
USFM_BENCH_CORPORA=all npm run bench:wasm:web
```

The corpus selector matches the existing native bench env
(`benches/README.md` line 24), so a single env var drives both.

### Operations to bench

The native `operations` group benches (string-in vs token-in
where applicable):

| Native bench (`benches/operations.rs`) | Wasm equivalent          |
| -------------------------------------- | ------------------------ |
| `lex/string`                           | *(not exposed to JS)*    |
| `parse/string`                         | `parse(source)`          |
| `cst/tokens`                           | `parsedUsfm.cst()`       |
| `usj/string`                           | `parsedUsfm.toUsj()`     |
| `usx/string`                           | `parsedUsfm.toUsx()`     |
| `html/string`                          | `parsedUsfm.toHtml()`    |
| `html/tokens`                          | `tokensToHtml(tokens)`   |
| `lint/string`                          | `lintUsfm(source)`       |
| `lint/tokens`                          | `lintTokens(tokens)`     |
| `format/string`                        | `formatUsfm(source)`     |
| `format/tokens`                        | `formatTokens(tokens)`   |
| `diff/string`                          | `diffUsfm(a, b)`         |
| `diff/tokens`                          | `diffTokens(a, b)`       |
| `vref/string`                          | `parsedUsfm.toVref()`    |

`lex` is library-internal and not exposed to JS; the wasm bench
matrix doesn't need it. Everything else has a 1:1 mapping.

### Throughput accounting

Native benches use `criterion::Throughput::Bytes(book.bytes as
u64)`. The wasm harness mirrors this: for each operation, divide
elapsed time by source byte length, report MB/s. Same units, same
fixtures, side-by-side comparable.

The token-in path needs a one-time pre-parse before measurement
(matching native, which uses `parsed.tokens.clone()` inside the
bench loop's iter). Track that pre-parse as a separate setup
column in the report — it's the JS↔WASM marshalling cost, which
*is* what we'd want to measure for "how much does the FFI
boundary cost relative to the operation itself."

### What the report shows

```
| operation        | native (release) | wasm (bundler) | wasm/native |
| ---------------- | ---------------- | -------------- | ----------- |
| parse/string     | 312 MB/s         | 87 MB/s        | 0.28x       |
| lint/tokens      | 410 MB/s         | 240 MB/s       | 0.58x       |
| usj/string       | 168 MB/s         | 51 MB/s        | 0.30x       |
| diff/tokens      | …                | …              | …           |
```

Tracking the ratio over time is what catches regressions. A
single bad commit that adds an unnecessary copy on the FFI
boundary shows up as wasm/native dropping 2x for one operation.
The native column is sourced from the existing `BENCH_RESULTS.md`
snapshot.

### Optional: marshalling-only micro-bench

In addition to whole-operation benches, add three micro-benches
that measure FFI overhead in isolation:

- **Token list round-trip**: `parse(source).tokens()` →
  `tokensToUsfm(tokens)`, measure the per-byte cost of the JS↔WASM
  hop with the operation itself being a near-no-op.
- **Lint result return**: lint a fixture; measure time to
  receive the `LintResult` in JS for issue counts of N=10, 100,
  1000.
- **Diff result return**: same, for `ChapterTokenDiff[]`.

These tell us whether tsify's `into_wasm_abi` is materially
slower than the current `serde_wasm_bindgen::to_value` path. If
it is, that's a finding worth knowing before phase 7 — there's
an escape hatch (`#[tsify(into_wasm_abi)]` can be omitted for
expensive types in favor of explicit `js_sys::Object` building),
but only if we know which types are expensive.

## Intersection with other plans

- **`plan-walker-architecture.md`**: the walker is library-internal.
  None of its types (`WalkContext`, `ScopeFrame`,
  `LeaveReason`, walker events) cross the FFI boundary. Step 6 of
  the walker plan (lint visitor migration) is independent of this
  one. The walker plan explicitly says (line 423): "No new
  token-in WASM bindings. The existing surface is preserved."
  This plan respects that contract: the JS surface is unchanged,
  only its implementation collapses.
- **`plan-marker-data-curation.md`**: `MarkerInfo` is one of the
  types this plan FFI-mirrors. If the marker-data cleanup changes
  `MarkerInfo`'s shape — adding/removing fields, splitting an
  enum — the tsify mirror updates in one place. Currently it
  updates in three (FFI value type, TS_TYPES, mapping function).
  The plan-walker plan and plan-marker-data plan both benefit
  from this refactor landing first or alongside.
- **`plan-whitespace-lint-rules.md`**: the six new lint codes
  land via the `LintCode` enum on the native side. Today, each
  new variant requires touching `lint_code_variants()`,
  `lint_code_str()`, `parse_lint_code()`, the `LintCode` TS
  union, and tests. After this plan: one variant on the native
  enum + one variant on the FFI enum (with a single `From` arm).
  Halves the per-rule wasm-side overhead for the whitespace
  rules.
- **`plan-cli.md`**: the CLI doesn't consume wasm. No
  intersection.

## Out of scope

- **Replacing `wasm-bindgen` with `wit-bindgen` / Component
  Model**. Different ecosystem, much larger lift, and the
  wasm-bindgen → Component Model transition path is not yet
  stable enough for a library that wants to publish to npm today.
  Revisit when Component Model has first-class npm consumer
  support.
- **Replacing `serde-wasm-bindgen` with manual `js_sys::Object`
  builders**. Could yield bundle-size wins; loses the schema
  derivation that's the whole point. If marshalling-only
  micro-benches (above) show specific hot types where it
  matters, fix those individually; don't do a blanket migration.
- **Top-level JS API ergonomics improvements** (e.g. fluent
  builder, async iteration over diffs). The current shape is
  fine; changing it is a separate design question. This plan
  preserves the JS surface byte-for-byte.
- **Bundle-size budget enforcement in CI**. Worth doing, separately.
  Not part of the schema-derivation refactor.
- **Source maps for wasm**. Independent concern.
- **Tree-shaking unused exports.** wasm-pack doesn't tree-shake
  Rust exports today; either every binding ships or none do.
  Component Model fixes this; until then, no.

## Open questions to resolve during implementation

- **Does `Tsify` reproduce `serde-wasm-bindgen`'s exact
  semantics for `BTreeMap<u32, …>`?** Spec says yes
  (`Record<number, …>`), but specifically the *string-keyed
  number* semantics on the JS side need a parity check. If
  Tsify changes this to `Map<number, …>` (newer TS proposal),
  consumers downstream break. Verify in phase 2.
- **Should `LintOptions.disabled_codes` accept a serde
  default of `[]` vs require explicit `[]`?** Today
  `from_js_or_default` returns `LintOptionsValue::default()`
  on undefined/null, which means JS callers can pass
  `undefined` and it works. Tsify's `from_wasm_abi` requires
  the value or `Option<…>`. If we want `lintTokens(tokens)`
  with no second arg to keep working, the binding signature
  must be `options: Option<LintOptions>`. Confirm and codify.
- **Where does the `ParsedUsfm` lazy-parse boundary live after
  the refactor?** Today `ParsedUsfm` holds the source string and
  re-parses on every method call (lines 956, 961, 972, 1017,
  …). That's wasteful — six native re-parses for one JS call to
  `parsed.tokens()` then `parsed.cst()` then `parsed.lint()`.
  Should `ParsedUsfm` cache the `ParseResult` internally? The
  lifetime issue (parse result borrows from source) means the
  cache has to be wrapped in `ouroboros` / `self_cell` or held
  as an owned `ParseAnalysis` post-extraction. Worth answering,
  not necessarily fixing in this plan — but call it out as a
  perf follow-up the wasm bench would surface.
- **Should the FFI `Token` type implement `LintableToken` and
  `DiffableToken` directly, or should the adapter pattern stay
  for clarity?** Direct impls are smaller. The adapter exists
  today because the FFI `TokenValue` is a serde struct and
  can't carry the native borrowed types. The new FFI `Token`
  has the same constraint (owned strings, no lifetimes), so the
  trait impls just delegate to `self.kind`, `self.text`, etc. —
  no adapter needed. Confirm via phase 8.
- **Does `tsify` honor `#[serde(flatten)]`?** The native
  `Token<'a>` uses `#[serde(flatten)]` to flatten `TokenData`
  into the token shape. If the FFI mirror keeps the same flat
  shape (currently it does — `TokenValue` has fields like
  `marker`, `nested`, etc. inlined), then flatten isn't
  needed. If the FFI mirror were ever restructured as
  `{ id, span, data: TokenData }`, this question would matter.
  For this plan: keep the flat shape, sidestep the question.

## Estimated scope

- `lib.rs`: 2,613 lines → ~250 lines.
- New `ffi/*.rs` modules: ~600 lines total (one struct
  declaration + one `From` impl per type).
- New `convert.rs`: ~250 lines (the rest of the conversions).
- New `benches/wasm/*.mjs`: ~300 lines.
- New `crates/usfm_onion_wasm/golden/`: ~20 input fixtures, ~50
  golden output files (binary blobs, not counted as code).
- `scripts/test-web-package.mjs` rewrite: ~150 lines.

Net code change: **−2,613 in lib.rs, +1,100 elsewhere**. The wasm
crate shrinks by ~1,500 lines and gains a real fidelity contract
plus a perf-tracking harness.

The schema-mirroring drift class — TS_TYPES vs value types vs
mapping functions vs native types — is eliminated. New types added
to either the native crate or the wasm crate fail compilation
until both sides match, instead of failing silently at runtime in
a downstream consumer's editor.
