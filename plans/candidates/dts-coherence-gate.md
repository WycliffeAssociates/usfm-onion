# Candidate — generated-d.ts coherence gate

Date: 2026-08-04. Status: candidate — owner-approved for the pile after three
editor-reported escapes in one day, all of the same class. Cheap (one script +
one package.json entry), high leverage.

## The escape class

v0.1.5 shipped three defects no existing gate could see, all found by the
editor implementer, not by the battery:

1. **Stale hand-copy vs generated** — `js/packed.d.ts`'s hand-declared
   `PublishedCorpusOutcome` (the structural type of its `wasm` module
   parameter) missed the `invalidExtent` arm the bytes round added to the
   generated type. Fixed `204a05c`.
2. **Declaration vs runtime** — `PublishedCorpus.bytes` /
   `ScopedPublication.packed`/`sources` delivered `Uint8Array` at runtime but
   declared `number[]` (tsify's TS generation doesn't follow `serde_bytes`).
   Fixed `29b1a29` via `#[tsify(type = "Uint8Array")]`.
3. **Declaration vs declaration** — braid's native `ScopedPublishError` kept
   its Tsify derive after the wasm crate declared its own boundary mirror of
   the same name; both emitted into the same `.d.ts` as conflicting
   `export type`s (the native one with the wrong, un-projected Scope shape).
   Fixed `1c41e93`.

Common cause: the battery exercises runtime values and typechecks the packed
fixture, but **nothing reads the generated `.d.ts` files as an artifact** —
`tsc --noEmit` on the fixture only parses declarations for symbols it
actually imports.

## The gate

One script (`scripts/check-dts-coherence.mjs`, both targets), asserting:

1. **No duplicate top-level declarations**: parse each generated
   `usfm_onion_web.d.ts`; every `export type` / `export interface` /
   `export class` / `export function` name declared at most once
   (overload groups excepted for functions).
2. **Hand-copy conformance**: the tsc fixture types the `./packed` lane's
   `wasm` parameter as `typeof import("../pkg-bundler/usfm_onion_web")`
   (or a structural-assignability check between `js/packed.d.ts`'s declared
   module parameter and the generated module type), so any future drift
   between the hand copy and the generated surface fails `test:packed:types`
   instead of failing in the editor.
3. **Bytes-convention tripwire**: no field declaration of type `number[]`
   in either generated d.ts (doc-comment prose exempt). Any legitimate future
   exception is an explicit allowlist entry with a written reason —
   allowlist starts empty.
4. (Optional, same spirit) tsc-parse both generated d.ts files standalone
   (`tsc --noEmit` over a stub importing `*` from each) so a syntactically
   duplicate/conflicting declaration is a hard error even if rule 1's parser
   misses a form.

Wire into the standing battery next to `test:packed:types` and run in the
same final-order sweep (after release build, before commit).

## Non-goals

- Not a semantic diff of d.ts against Rust types (tsify owns that); this only
  checks the emitted artifact's internal coherence and the one hand-written
  copy we deliberately keep (`js/packed.d.ts`).
- No new build machinery — plain Node script + tsc invocations that already
  exist in the repo.
