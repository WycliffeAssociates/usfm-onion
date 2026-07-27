# Plan: `@lang` attribute on `\tl` and `\wl` (USFM 3.1.2)

Status: open. Extracted from the now-completed `plan-marker-data-curation.md`
— every other item in that plan shipped; this is the one deliverable that did
not, because there was no attribute-allowlist mechanism to wire it into.

## Goal

USFM 3.1.2 adds an optional `@lang` attribute to the transliteration char
marker `\tl` and the wordlist char marker `\wl`
(e.g. `\tl |lang="grc"\tl*`). Recognize it as a valid, known attribute on
those two markers so the engine can:

1. expose it as a parsed attribute (already happens — see below), and
2. eventually validate attribute *names* against a per-marker allowlist, so an
   unknown attribute on a marker can be flagged.

Spec only lists `tl` and `wl` for `@lang` in 3.1.2 — audit confirmed no other
character markers gain it. Scope is exactly these two markers.

## Current state (verified 2026-06-26)

- `\tl` and `\wl` already parse and round-trip; attribute lists on any marker
  are lifted onto the owning marker token (`TokenData::Marker { attributes,
  attribute_source }` in `src/token.rs`) and re-emitted losslessly by
  `tokens_to_usfm`. So `\tl |lang="grc"\tl*` already preserves the attribute.
- `MarkerSpec` / `MarkerDef` carries only `default_attribute`
  (`src/marker_defs.rs:353`, `derive_default_attribute` at `:1249`) — the
  *shorthand* default-attribute key, not an allowlist of permitted attribute
  names.
- **There is no per-marker attribute allowlist and no attribute-name
  validation lint.** `grep` for attribute handling in `src/lint_impl.rs` finds
  only the `|`-as-tag-end-delimiter checks, nothing that validates attribute
  keys. So `@lang` is currently neither blessed nor rejected — it's simply
  passed through untyped.

This means the work is **net-new mechanism**, not "flip a flag." Decide whether
that mechanism is worth building yet.

## Decision to make first

Do we actually want attribute-name validation? Two options:

- **A — minimal / declarative-only.** Add an `allowed_attributes:
  &'static [&'static str]` field to `MarkerDef`, populate `tl`/`wl` with
  `["lang"]` (and `default_attribute` stays for the shorthand). No lint yet.
  Cheap, forward-compatible, but inert until a consumer reads it.
- **B — full validation.** A + a new lint code (e.g.
  `UnknownAttributeOnMarker`, `LintCategory` TBD) that fires when a marker
  carries an attribute whose key isn't in its allowlist (and isn't the
  default-attribute shorthand). This is the user-visible payoff but is the
  larger change — needs a default-attribute carve-out, a decision on severity,
  and a sweep of every marker's real allowlist (not just `tl`/`wl`) to avoid
  false positives on already-valid attributes like `\w …|strong="…"`.

Recommendation: **A now** (registers `@lang` declaratively, ~10 lines, no false
positives), and only do **B** when there's a consumer that needs attribute
validation — at which point the allowlist must be populated for *all*
attribute-bearing markers, not just these two.

## Implementation (option A)

1. Add `pub allowed_attributes: &'static [&'static str]` to `MarkerDef`
   (`src/marker_defs.rs`), defaulting to `&[]`.
2. Set `["lang"]` on the `tl` and `wl` rows in `src/marker_defs_data.rs`.
3. Surface it on the public `UsfmMarkerInfo` (`src/markers.rs`) if catalog
   consumers need it — mirror how `default_attribute` is exposed at `:139`/`:248`.
4. Test: assert `marker_def("tl")`/`marker_def("wl")` report `lang` as allowed;
   assert a marker without an allowlist reports empty. `cargo test -p usfm_onion`.

## Out of scope

- Validating attributes on any marker other than `tl`/`wl` (deferred to option
  B, which requires a full allowlist sweep).
- The WASM/Tsify surface for the new field — add only if a JS consumer needs it.
