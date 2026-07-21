> **ONION SIDE DONE (2026-07-21, commit f3689cf).** The token wire DTO is now
> single-sourced in `usfm_onion_dto` (Token + 6 sub-types + 3 dependency enums),
> tsify-gated, `span: Option`, with `From<usfm_onion::Token<'a>>`. Generated
> `.d.ts` is byte-identical; wasm public API unchanged; a no-`span` deser drift
> test is in the dto crate. **Editor follow-up (their repo):** import
> `usfm_onion_dto::Token`, delete `FlatTokenDto`/`map_flat_token_dto`, and audit
> `mirror.rs`'s required-`span` struct. Ships with usfm_onion 0.0.10.

# Handoff — single-source the token wire DTO (kill the Tauri-command DTO drift class)

**Date:** 2026-07-21
**For:** an agent working in `usfm_onion` (this repo).
**From:** the consumer, `scripture-editor-proto-2`.
**Status:** proposal / research handoff — not a locked plan. Reads on whether to
do this at all are in "The open decision" below.

## What happened (the motivating incident)

The editor consumes onion two ways: the **wasm/TS** surface (`usfm-onion-web`,
whose `.d.ts` the app re-exports via `src/core/domain/usfm/usfmOnionTypes.ts`)
and a **native Rust** surface (Tauri commands in
`src/tauri/rust/src/usfm_onion.rs` that depend on the `usfm_onion` +
`usfm_onion_dto` crates directly).

The app's `Token` is `Omit<OnionToken, "span">` — it deliberately drops `span`
(editor commit `4156f5db`), so the frontend serializes token command args
**without** `span`. The editor's native commands deserialize into a
**hand-written** `FlatTokenDto` (`src/tauri/rust/src/usfm_onion.rs`) whose `span`
was **required** → in the v0.0.9 desktop nightly, every native token command
(`usfm_onion_diff_tokens`, `usfm_onion_merge_diff_blocks`, the lint commands,
`usfm_onion_format_token_batches`, `usfm_onion_apply_token_fix`) failed at
runtime with `Got invalid args ...: missing field 'span'`. The Option C diff
review surfaced it first. **The wasm/web path was unaffected** because onion's
wasm-side token deserialization treats `span` as optional.

Editor-side band-aid already shipped (editor commit `1702e9a0`):
`#[serde(default)]` on `FlatTokenDto.span` + a regression test
(`usfm_onion::tests::diff_tokens_accepts_frontend_tokens_without_span`). That
stops the bleeding but does **not** address the root cause.

## The root cause (a recurring class)

`FlatTokenDto` is a **second, hand-maintained copy** of the token wire contract,
parallel to the wasm/TS type. Nothing forces the two to agree:
- the Tauri `invoke()` boundary is untyped end-to-end (only the return type is
  checked, not that the args match what Rust deserializes);
- the editor's typecheck / unit / e2e all exercise the **wasm/web** path — cargo
  *compiles* the native commands but never *invokes* them with real
  frontend-shaped JSON, so native-only drift is invisible until a desktop user
  hits it.

This is the **second** occurrence of the class. The first was the marker catalog
DTO drift (PascalCase string values diverged from the wasm `MarkerInfo`, broke
form mode) — which is *why* `usfm_onion_dto` exists today as "the single source
of the marker wire-contract strings." **Tokens are the leftover exception**:
they never got moved into the shared crate.

## Proposed direction (option 2 — the onion-side work)

Give the token type the same treatment markers already got: **one serde
definition, in `usfm_onion_dto`, that generates the TS type AND is what the
native commands deserialize into.** Then a field change (like dropping `span`)
moves the TS type and the native Rust expectation in lockstep, and a mismatch is
a compile error rather than a runtime crash.

Rough shape (you know this crate's real structure — adapt):

1. Define the canonical owned wire DTO in `usfm_onion_dto` (serde +
   `rename_all = "camelCase"`, `span: Option<..>` because *optional is the real
   contract* — encode it here once, not as an app-side `Omit<>`). Gate the
   `tsify`/wasm-bindgen derive behind the crate's existing `wasm` feature so the
   TS type is generated from it.
2. Have the **wasm bindings** (`pkg-bundler`/`pkg-web`) surface this DTO as the
   token type in their `.d.ts` (rather than whatever token type they emit today
   — confirm how that's currently generated; part of this may already be close).
3. Provide `From`/`Into` between the wire DTO and onion's internal borrowed
   `Token<'a>` (onion already has an owned `FormatToken` — this DTO may be able
   to reuse or replace it). Internal hot paths keep the borrowed type.
4. Tag a release carrying the new `usfm_onion_dto` + regenerated wasm.

## Editor-side follow-up (happens here, after onion ships — not your task)

For context so you know the downstream contract:
- `usfmOnionTypes.ts` re-exports the generated DTO and drops the `Omit<span>`.
- `src/tauri/rust/src/usfm_onion.rs` imports `usfm_onion_dto::<TokenDto>` as the
  command arg type and **deletes** `FlatTokenDto`, `map_flat_token_dto`, and the
  ~4 construction sites collapse into the DTO's `From`/`Into`.
- Re-verify BOTH surfaces (wasm typecheck + a native-command deser test with a
  frontend-shaped payload).

## The open decision (don't do the work until this is settled)

Option 2 is the *correct* end state but it's a **coordinated onion + editor
change** across two repos. There is a lighter, **editor-local** alternative that
needs zero onion work: **tauri-specta** (or `ts-rs`) to generate the TS
`invoke` wrappers from the editor's Rust command signatures, making the invoke
boundary type-checked at the call site. That closes the *runtime-crash* gap
without unifying the type families.

So: only invest in the onion-side DTO promotion if the team commits to
single-sourcing (which also removes the parallel-DTO maintenance for good). If
the team prefers tauri-specta, this handoff is moot — nothing to do in onion.
Confirm with Will before starting.

## Related cleanup (same class, if you do go the single-source route)
- Audit other hand-mirrored token DTOs: `src/tauri/rust/src/mirror.rs` has
  another token struct with a required `span` — latent instance of the same
  drift if it's ever fed frontend tokens.
- Sanity-check that the marker catalog path is genuinely single-sourced now (it
  was the first occurrence) and not just less likely to drift.

## Verification bar for the class
The gap survives because nothing exercises the native command seam with a real
frontend payload. Whatever approach is chosen, add a test that deserializes a
**frontend-shaped** token JSON (no `span`) through a native command — the editor
already has one (`diff_tokens_accepts_frontend_tokens_without_span`); shared JSON
fixtures across wasm + native keep both honest.
