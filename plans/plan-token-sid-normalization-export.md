# Plan: standalone token SID normalization export

## Goal and definition of done

Ship a plain-JavaScript `usfm-onion-web/token-sids` package subpath in the
already-unreleased `v0.0.9`. It gives token-owning applications Onion's canonical
SID derivation without loading wasm. Done means the JS and live Rust implementations
produce identical SID vectors for the same structural token DTOs, the JS function is
pure, and existing token diff/merge/revert behavior is unchanged.

## Problem

Onion has two intentionally different ownership boundaries:

- Source-string APIs parse structure and derive canonical SIDs.
- Token APIs accept caller-owned token streams and trust their supplied `sid` values.

Changing token APIs to derive implicitly would make granular and synthetic token
streams impossible unless callers also supplied a whole chapter's structural context.
But applications ingesting complete JSON token streams still need an explicit way to
produce the same canonical address space Onion uses for source-based diffing. Importing
the wasm package for this one stateful pass is unnecessary, especially in Tauri.

## User stories

- As a token-store owner, I can normalize a complete structural token stream before
  freezing a compare snapshot.
- As a ZIP/JSON importer, I can repair missing or stale carried SIDs when marker and
  number structure is present.
- As a granular-token caller, I can keep intentional supplied SIDs because Onion's
  token diff/merge/revert functions do not reinterpret my fragment.
- As a Tauri consumer, I can use the canonical algorithm without adding wasm to the
  production application bundle.

## Settled design

### Ownership contract

- Source-string APIs own SID derivation.
- Token diff/merge/revert APIs trust caller-supplied SIDs.
- Normalization is explicit and occurs at the consumer's complete-snapshot boundary.
- Lexical editor metadata and Zephyr's `mutAddSids` are outside this change.

### Package API

```ts
import { normalizeTokenSids } from "usfm-onion-web/token-sids";

normalizeTokenSids(
  tokens: readonly Token[],
  bookCode: string,
): Token[];
```

The export is a standalone ESM file with a declaration file. It has no runtime import
of the wasm root or web entry points. `bookCode` is authoritative. The function:

- derives from `kind === "marker"`, `marker`, and the immediately following token's
  `numberInfo.start` / optional `numberInfo.end`;
- starts at `BOOK 0:0`, assigns chapter-open material to `BOOK C:0`, and uses
  `BOOK C:V[-END]` for verses;
- gives repeated identical verse/range bases `_dup_N`, with counters reset on a valid
  chapter transition;
- ignores embedded book-code tokens and every carried `sid`;
- returns a new array of shallow-cloned tokens, preserving every non-`sid` field and
  leaving the input array and objects untouched;
- is idempotent and does not change concatenated token source bytes.

The semantic equivalence is:

```text
normalizeTokenSids(tokens, bookCode).map(token => token.sid)
  == derive_canonical_sids(structurally_equivalent_tokens, bookCode)
```

It is not a promise that raw parser-carried SIDs already contain richer duplicate
suffixes. Consumers that need the canonical address space explicitly normalize.

## Implementation sequence and gates

### 1. Add the standalone module

Add `js/token-sids.js` and `js/token-sids.d.ts`. Port the current Rust
`derive_canonical_sids` state transition directly without adding validation or a new
abstraction layer.

Gate: a plain Node process imports `usfm-onion-web/token-sids` without wasm flags,
initialization, or resolution errors.

### 2. Publish the subpath through the root package

Add `js` to `package.json#files` and an explicit `./token-sids` export with `types` and
`default` conditions. Keep version `0.0.9`; this is additive work in an unreleased
package version.

Gate: test the real package self-reference, not a relative path into `js/`.

### 3. Add JS-to-Rust conformance coverage

Run identical DTO fixtures through the plain JS export and the live wasm
`normalizeTokenSids`, then compare exact SID vectors. Keep separate JS assertions for
unknown/irrelevant field preservation because wasm's typed `Token` boundary may discard
fields outside its DTO.

Required shapes:

- intro, chapter-open, singular verse, and bridge;
- duplicate singular verses and duplicate bridges;
- duplicate-counter reset at the next chapter;
- authoritative explicit book code despite an embedded different book code;
- malformed marker sequences and streams with no chapter/verse structure;
- missing, stale, and arbitrary carried SIDs;
- LF/CRLF source text and irrelevant token fields;
- idempotence, input immutability, non-`sid` preservation, and serialization stability.

Gate: targeted import and both bundler/web conformance runs pass.

### 4. Integrate with package verification

Run conformance after the existing package smoke for both wasm targets. Do not create a
new test framework. Document the subpath in the root README.

Gate:

```bash
npm run test:token-sids:import
npm run test:wasm
npm run golden:wasm
npm run golden:wasm:web
npm run check:wasm:web
cargo test --workspace
git diff --check
```

If dev wasm builds replace checked-in release artifacts, finish with
`npm run build:wasm` and inspect generated package declarations and smoke behavior.

## Non-goals

- No `bookCode` parameters on token diff/merge/revert APIs.
- No changes to Rust diff, merge, revert, parser SID storage, or native `Sid`.
- No Zephyr product-code or dependency edits.
- No Lexical `mutAddSids` rename, refactor, or retirement.
- No automatic normalization inside `IUsfmOnionService.diffTokens`.
- No version bump, tag, commit, or release publication.

## Consumer handoff

The later Zephyr Option C migration will read complete baseline/current chapter arrays
from `WorkingFilesStore`, normalize both through this subpath using the known book code,
freeze those exact arrays as the compare snapshot, and pass them unchanged to diff,
preview, and Apply. The dependency upgrade and new symmetric diff-review UI belong to
that coordinated Rust/web/UI plan, not this package slice.

## Release note

`v0.0.9` adds `usfm-onion-web/token-sids`, a wasm-free `normalizeTokenSids` export for
explicit canonicalization of caller-owned structural token streams. Token diff, merge,
and revert APIs continue to trust supplied SIDs.
