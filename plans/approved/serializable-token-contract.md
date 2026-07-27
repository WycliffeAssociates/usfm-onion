# Approved: `SerializableToken` — the minimal editor token contract + de-fork the serializer

Pulled out of the braid epic (was §4.3 / §17 #12) 2026-07-24 so the **editor can
start implementing the contract now, in parallel with braid**, and so the token→USFM
serializer stops being reimplemented in the wasm crate. Pre-braid; delegate-well shape.

## Why

Two problems, one fix:

1. **No written minimal contract.** We tell the editor "as long as you can hand back a
   Token, it's fine," but never spell out the floor. There are actually two contracts:
   - **String floor (always sufficient):** emit USFM *text* → `parse(str)` → full
     fidelity. Needs only "serialize your tree to a string." Attrs are inline in the
     string. This is the fallback whenever the token path is inconvenient.
   - **Token path (skip re-parse, the optimization):** hand back tokens that satisfy the
     operation's trait. `SerializableToken` is the *smallest* such trait — smaller than
     `LintableToken`/`DiffableToken` — and the natural first one the editor targets.
2. **The serializer is forked.** Core owns `tokens_to_usfm(&[Token<'_>])` (native). The
   wasm crate **reimplements** the same algorithm (`token_values_to_usfm` +
   `format_attribute_list` + `encode_attr_value` + `closer_shape`) because core's emitter
   takes the borrowed native token while JS hands back owned wire tokens. That's the
   behavior-level drift class the DTO SSOT work killed for data — still live for this algo.

## The contract

**`SerializableToken` (minimal): `kind`, `source`, `marker`, `attributes`, `attribute_list`.**
- `source() -> &str` — the **raw** bytes (marker text incl. backslash, text content, exact
  whitespace). As today, this EXCLUDES the `|...` attribute list — native `Token.source` is
  the marker+text only, and the attribute list is carried separately (see below). MUST stay
  raw for byte-losslessness — see the guardrail below.
- `kind() -> TokenKind` and `marker() -> Option<&str>` — drive `closer_shape` (where the
  attribute list attaches: char → before `\name*`; paragraph → before newline; milestone →
  its own). `marker` is derivable from `source`, but explicit avoids a re-strip.
- `attributes() -> &[AttributeItem]` — the structured list (`{key, value, is_default, …}`).
  Used for reading/semantics and for **authoring** new attributes. `is_default` drives the
  shorthand (`\w gracious|grace\w*` vs `|lemma="grace"`). **The caller never reconstructs a pipe.**
- `attribute_list() -> Option<&str>` — the **verbatim `|...` slice** (trivia). This is what
  makes a *structured* token losslessly serializable: native `MarkerAttrs.attribute_source`
  already holds it. When present the emitter emits it byte-for-byte (preserving exact
  inter-item whitespace, quote style, encoding); onion reconstructs from `attributes()` ONLY
  when it's absent (editor-authored content — canonical formatting is correct there).

### The lossless-structured shape (resolved 2026-07-24)

The whole point: a raw `source: string` floor would punt losslessness knowledge to the
editor. The resolution is **one structured shape that also retains the verbatim trivia** —
`attributes` (logical) + `attribute_list()` (verbatim `|...` slice). What it is NOT (see the
"two emitters" finding below) is *one emitter over all token types*: that regresses native
losslessness for malformed input, so native keeps span-drain and owned tokens get a second,
spanless emitter in core.

- The verbatim `|...` slice is dropped at the WIRE boundary today (wire `Token.source` =
  native `source`, which omits the attr list; the wire DTO carries decoded structured
  `attributes` but not `attribute_source`). That drop is why the owned-token emitter must
  reconstruct — and reconstruction is what normalizes.
- Fix: the wire `Token` DTO gains **`attributeSource?: string`**, populated from native
  `MarkerAttrs.attribute_source`. An editor that passes a parsed token through unchanged
  round-trips the trivia → byte-lossless. An editor that **edits** an attribute clears
  `attributeSource`; onion reconstructs canonically from the structured `attributes` it hands
  back. That is the one editor-facing rule: *touch an attribute → drop its verbatim; onion
  re-serializes from structure.* Editors that never touch attributes never see it.
- Value encoding follows the same split: the verbatim path needs no decode/encode (bytes are
  preserved); the reconstruct path applies `encode_attr_value` to the logical value the
  editor supplies. So decode/encode only runs for genuinely-authored values, never for
  round-tripped ones — the lossy-recovery cycle disappears for pass-through content.

### Attribute-mutation contract — granularity (confirmed 2026-07-24)

When the editor mutates attributes, the boundary is **whole-list, all-or-nothing**, and the
editor works **only in structured space** — it never splices the pipe string:

- **Remove a key:** hand back the `attributes` array without that item + `attribute_list()`
  → `None`. Onion reconstructs `|` + remaining items.
- **Edit a value (or key):** hand back the array with the new value + `attribute_list()`
  → `None`. Onion reconstructs.
- There is **no splice API**. The editor never emits a `|`, never escapes a value; onion owns
  the `|`, inter-item separators, `key="value"` vs. default-shorthand, and `"`/`\` escaping.
- **Granularity is per-token, not per-item:** touching *any* attribute on a marker drops the
  verbatim for the whole marker, so untouched sibling attributes on that same marker get
  canonically reformatted too. This is accepted (a little ugly, but fine — confirmed): the
  edited marker is a diff region anyway, canonical form is deterministic, and per-item
  preservation would force onion to own a separator-trivia model for marginal benefit (YAGNI
  until a real editor complains about diff noise).
- **Why this is contract-compliant:** the core rule is "never *silently* normalize on
  *ingest*" — parse-then-emit of *untouched* content is byte-identical. Edited content has no
  original bytes to preserve; producing canonical output there is formatting new bytes, not
  normalizing existing ones.

### Two emitters in core — native span-drain is NOT replaceable by closer-shape (finding 2026-07-24)

Attempting a single generic closer-shape emitter over *all* token types (Gate 1 spike)
regressed `cst_roundtrips_all_usfm_sources` on `testData/paratextTests/FigureNotClosed`: an
**unclosed** `\fig …|…` (parser-recovery fixture) has no closer, so closer-shape flushes its
pending attribute list at end-of-stream instead of at the source position. Native
**span-drain** reproduces the exact byte offset because it places by the attribute slice's
recorded span — robust for nested markers (attr after nested content) *and* unclosed markers
(attr before the implicit boundary) alike. A spanless emitter can't match that without
re-deriving the walker's implicit-scope-close semantics (heavy, error-prone).

`tokens_to_usfm` is lossless **by design including recovery** (CLAUDE.md), so regressing the
malformed case is a contract violation, not an acceptable golden shift. Resolution:

- **Native `Token` keeps its existing span-drain `tokens_to_usfm(&[Token])` UNCHANGED** — zero
  regression, byte-exact including recovery. It still `impl`s `SerializableToken` (contract
  reference; lets tests cross-check the generic emitter against span-drain on well-formed
  input), but does not route through the generic emitter.
- **Owned/spanless tokens get a SECOND core emitter** `tokens_to_usfm_reconstruct<T:
  SerializableToken>` (the ported closer-shape algorithm) that prefers `attribute_list()`
  verbatim and reconstructs from `attributes()` when absent. This is the emitter the wasm
  crate + editor + braid use.
- Both live in core sharing `closer_shape` / `format_attribute_list` / `encode_attr_value`, so
  the wasm **fork is still eliminated** (wasm deletes its copies, calls core). Two emitters
  isn't "averaging" — native and owned tokens carry genuinely different information (spans),
  so they get different, shared-helper implementations. Surface the conflict; don't blend.
- **Documented caveat:** owned-token serialization places a *malformed/unclosed* marker's
  attribute list at end-of-scope, not its exact source byte offset. Well-formed input is
  byte-identical through either emitter; this only affects recovery of malformed input on the
  spanless path — which is exactly today's wasm behavior, so it is not a regression.

`id` is **not** on `SerializableToken` — serialization doesn't need it. It lives on
`LintableToken`. The editor's concrete token impls whatever set of traits it wants; the
traits compose, so the editor's practical floor to be serialize- *and* lint-capable is
`{kind, source, marker, attributes} + id` — assembled from honest sibling traits, not one
polluted one.

### Trait hierarchy — siblings on a shared base, NOT a chain

`SerializableToken` must **not** be a supertrait of (or supertrait'd by) `LintableToken`:
lint doesn't need `attributes`, serialize doesn't need `structural`/`id`. Factor the shared
floor into a base and hang the two concerns off it as siblings:

```rust
trait UsfmToken            { fn kind(); fn marker(); fn text(); }   // shared floor (text == source)
trait SerializableToken: UsfmToken { fn attributes() -> &[AttributeItem]; }
trait WalkableToken:     UsfmToken { fn structural(); fn next_is_number(); }
trait LintableToken:     WalkableToken { fn id(); fn sid(); fn span(); fn number_info(); /* … */ }
```

Today `kind`/`marker`/`text`/`structural` all sit on `WalkableToken`; this work extracts
`{kind, marker, text}` into a `UsfmToken` base (implementation detail — could also leave
them duplicated, but the base is cleaner and duplication-free). A consumer impls
`UsfmToken` once + whichever branch(es) it needs; `LintableToken` transitively pulls in the
base. The editor implementing `LintableToken` for its nodes gets the base for free and adds
`SerializableToken`'s single `attributes()` to also serialize.

### `kind()` — the valid values (9)

`TokenKind`: `Newline`, `OptBreak`, `Marker`, `EndMarker`, `Milestone`, `MilestoneEnd`,
`BookCode`, `Number`, `Text`. Wire strings the Token DTO exposes: `newline`, `optBreak`,
`marker`, `endMarker`, `milestone`, `milestoneEnd`, `bookCode`, `number`, `text`.

**Decided (2026-07-24): one vocabulary = `"newline"`.** The diff's internal `kind_key`
currently maps `Newline → "verticalWhitespace"` while the token DTO's `kind` is `"newline"`.
Consolidate on **`"newline"`**. Coupled change — move in lockstep: `token_kind_key`
(`src/diff/mod.rs`) `Newline` arm → `"newline"`, AND `unit_plain_text`'s filter in
`text_diff.rs` (which matches `"verticalWhitespace"`) → `"newline"`, or the text-diff stops
filtering newlines. Verify: diff/text-diff fixtures + goldens byte-identical (kind_key isn't
in the golden'd surfaces, but confirm). Small, do it as part of this work on a clean tree.

### Ergonomics: default the re-derivable methods

So the editor's minimal impl is truly minimal, the re-derivable/optional methods should be
**defaulted** on the traits: `structural()` (currently non-defaulted on `WalkableToken`)
→ default `None` (onion re-derives from the marker); `sid`/`id`/`span`/`number_info`
(already defaulted). Then a Lexical node impls only `kind`/`marker`/`text` on `UsfmToken`,
adds `attributes()` for `SerializableToken`, and `id()` if it wants useful lint — everything
else it omits and onion re-derives or the dependent rule no-ops.

## Guardrail: `source` stays raw bytes — no discriminated union

A tempting alternative — a structured tagged union (`{marker:"add"}` instead of
`source:"\add "`) so the caller never sees a backslash — **breaks byte-losslessness**:
the raw source captures exact bytes/whitespace (`\add ` vs `\add\t` vs two spaces), and a
structured union forces onion to emit a canonical form → silent normalization → violates
the core "never silently normalize" contract. So the caller carries opaque `source`
strings (with backslashes) but never *interprets* them; a WYSIWYG node just stores its
source slice. Attribute syntax is the only thing the caller is spared, via `attributes` +
onion's re-merge.

## The refactor (de-fork)

Move the reconstruct emitter **into core** and make it generic over `SerializableToken`,
then delete the wasm reimplementation and have wasm call core. This is TWO emitters in core,
by necessity (see the "two emitters" finding above — span-drain and closer-shape are not
interchangeable):

- **Native span-drain emitter — UNCHANGED.** `fn tokens_to_usfm(tokens: &[Token<'_>]) ->
  String` stays exactly as it is today (byte-span drain, robust for nested + malformed). Do
  NOT rewrite it. Native Rust callers (`cst`, `parse`, `api`, wasm `toUsfm`) keep using it.
- **New generic reconstruct emitter for owned/spanless tokens.** `fn
  tokens_to_usfm_reconstruct<T: SerializableToken>(tokens: &[T]) -> String` — the closer-shape
  algorithm ported from the wasm fork. Per token: emit `source()`, place the attribute list
  at the `closer_shape` position — verbatim `attribute_list()` if `Some`, else
  `format_attribute_list(attributes())`. Owns `closer_shape` / `format_attribute_list` /
  `encode_attr_value` in core.
- `Token<'a>` impls `SerializableToken` (contract reference + parity tests;
  `attribute_list()` → `MarkerAttrs.attribute_source` slice). The wire/app token impls it
  (`attribute_list()` → its `attributeSource` field) and is what the reconstruct emitter runs
  over in production.
- Wire DTO: add `attributeSource: Option<String>` to `Token`, populated in
  `From<&NativeToken>` from `MarkerAttrs.attribute_source`.
- `usfm_onion_wasm`: delete `token_values_to_usfm` / `format_attribute_list` /
  `encode_attr_value` / the local `closer_shape`; `wasm_tokens_to_usfm` calls
  `tokens_to_usfm_reconstruct` over the wire tokens.

## Gates / verification (byte-lossless is the whole game)

0. **Native round-trip suite — DONE** (`6c22980`). It did not exist before; the only
   `round_trips_*` tests were in `usfm_onion_wasm` over the fork. Covers
   default-attribute-shorthand, embedded-quote, multiple-attrs, milestone-with-attributes,
   and non-canonical attribute-list whitespace. Frozen oracle for the span-drain emitter.
1. Add `SerializableToken` + `SerializableAttribute` + impl for native `Token`/`AttributeItem`
   (incl. `attribute_list()`). **Do NOT touch native `tokens_to_usfm`** — it stays span-drain
   (Gate-0 stays trivially green). Add the new generic `tokens_to_usfm_reconstruct<T:
   SerializableToken>` (closer-shape) + move `closer_shape`/`format_attribute_list`/
   `encode_attr_value` into core. Gate: full `cargo test -p usfm_onion` green (incl. Gate-0
   and `cst_roundtrips_all_usfm_sources`, which stays green *because native is unchanged*).
   Add a parity test: `tokens_to_usfm_reconstruct(&native_tokens) == tokens_to_usfm(&native_tokens)`
   for the well-formed Gate-0 corpus (proves the two emitters agree where they must).
2. Add `attributeSource` to the wire `Token` DTO + `From<&NativeToken>`; wire token impls
   `SerializableToken`; delete the wasm fork; `wasm_tokens_to_usfm` → `tokens_to_usfm_reconstruct`.
   Gate: `npm run golden:wasm` **byte-identical without `:update`**; `npm run test:wasm` green.
   `.d.ts` grows by one additive optional field (`attributeSource?`) — expected. Sanity: a
   round-trip through the wire (native → DTO → reconstruct emitter) now preserves non-canonical
   attribute whitespace it previously normalized — add one wire test pinning that improvement.
3. Doc: a trait-level doc comment enumerating the contract + the two-contract framing
   (string floor vs token path; `source` + `attributes` + verbatim `attribute_list` for
   lossless token→USFM; the touch-an-attribute-drops-verbatim editor rule; the malformed-input
   position caveat on the spanless path), so a Lexical-node author has a written contract.

Stop-and-report if `golden:wasm` diverges at Gate 2 — that would mean the reconstruct emitter
disagrees with the old wasm fork on some real (well-formed) case; surface the delta before
blessing anything.

## Editor adoption (parallel, not this repo)

Once `SerializableToken` exists, the editor implements it on its token type (it already
has `kind`/`source`/`attributes` structurally). That's the concrete "you can start now"
contract — the editor proves lossless round-trip through core while braid is built, and
picks up `LintableToken`/`DiffableToken` fields as it adopts those ops.

## Relationship to braid

This was epic §4.3 (reuse-preference: make core generic over a token trait) + §17 ledger
#12. Pulling it forward means braid's `usfm_onion_wasm` starts already thin (no serializer
fork), and the editor has a stable token contract to build against before braid lands.
Relates to [[project-braid-4-crate-architecture]], [[reference-wire-dto-single-source]].
