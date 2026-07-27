# Candidate: feature-gate the optional output surfaces (core + wasm)

Status: **candidate** — maybe one day; not deferred-decided, not pressing.

## Idea

Gate onion's optional surfaces behind Cargo features so a consumer who never touches a
given output doesn't pay for its code (or its deps) in their Rust binary *or* their wasm:

- `usx` (pulls **`quick_xml`** — the cleanest payoff), `usj`, `html`, `vref`, `diff`,
  possibly `lint`/`format`.
- `default = [all]` → the editor keeps everything; a minimal/embedded consumer opts out.
- wasm builds per feature set: `wasm-pack build -- --no-default-features --features lint`
  → the ungated `#[wasm_bindgen]` exports are the only roots, so `wasm-opt`/`gc-sections`
  strips the rest and the unused deps never link → a genuinely smaller wasm.

## Why it's a candidate, not a plan

- Real payoff only matters once there's a consumer that wants a slim build (today the editor
  uses everything). JS bundlers can't tree-shake wasm, so this is the *only* way to slim it.
- onion isn't feature-clean today; this is mechanical `#[cfg(feature = …)]` + module-gating
  work, doable incrementally (start with USX/`quick_xml`).
- Lighter than splitting into separate crates; revisit crate-splitting only if features get
  unwieldy or a surface needs true isolation/independent publish.

Relates: `../discussing/braid-stateful-handle.md` §6 #3 (one wasm package, one entrypoint;
slimming is a feature-gated build variant, never bundler tree-shaking).
