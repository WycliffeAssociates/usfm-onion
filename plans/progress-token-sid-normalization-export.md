# Progress: standalone token SID normalization export

Append-only execution log for `plan-token-sid-normalization-export.md`.

## 2026-07-17

- Confirmed the settled pivot: token diff/merge/revert continue trusting carried SIDs;
  this slice adds only an explicit normalization subpath.
- Read the live Rust `derive_canonical_sids` implementation and wasm wrapper. The pass
  reads marker/number structure, treats the explicit book code as authoritative, keeps
  duplicate counters per exact range base, and resets counters on valid chapter markers.
- Added standalone `js/token-sids.js` and matching declarations with no wasm import.
- Added the root `./token-sids` package export and included `js` in publish files.
- Added identical-DTO conformance coverage against live Rust through both wasm targets,
  plus purity, idempotence, stale-SID independence, field preservation, and byte-stability
  checks.
- First conformance run found that wasm's typed `Token` DTO drops an intentionally added
  unknown field. Corrected the test boundary: compare exact SID vectors for JS/Rust
  semantic parity, and separately prove the standalone JS function preserves all fields.
  No production behavior changed in response.
- Targeted results: `npm run test:token-sids:import` passed; direct bundler and web
  `scripts/test-token-sids.mjs` runs passed with 2 fixture streams each.
- The first `npm pack --dry-run` attempt was blocked before packing by a pre-existing
  ownership problem in `~/.npm/_cacache/tmp` (`EPERM`). Re-run uses an isolated temporary
  npm cache so package contents can still be verified without changing global state.
- `npm_config_cache=/tmp/usfm-onion-npm-cache npm pack --dry-run` passed. The 0.0.9
  tarball manifest contains `js/token-sids.js` and `js/token-sids.d.ts`; dry-run left no
  tarball behind.
- Full verification passed:
  - `npm run test:wasm`: bundler and web package smoke plus token-SID conformance green.
  - `npm run golden:wasm` and `npm run golden:wasm:web`: 7/7 fixtures green, unchanged.
  - `npm run check:wasm:web`: wasm32 compilation green.
  - `cargo test --workspace`: main library 188 passed, 0 failed, 2 ignored; integration,
    DTO, wasm (16/16), and doc-test suites green.
  - `npm run build:wasm`: release bundler and web artifacts restored successfully.
  - Release-artifact conformance passed again for bundler and web; standalone plain-Node
    package import passed without wasm flags or initialization.
  - `git diff --check`: clean.
- Final scope audit: this slice changed no Rust source or wasm API, package version,
  Zephyr file, tag, or commit. Dev wasm gates temporarily regenerated package artifacts
  and declarations; the required final `npm run build:wasm` restored the checked-in
  release forms. Final `git status --short` has no `pkg-bundler` or `pkg-web` changes.
- Follow-up review added a third, true no-structure fixture (no chapter or verse marker)
  and pins every resulting SID to `RUT 0:0`. `npm run test:token-sids:import`, bundler
  conformance (3 streams), web conformance (3 streams), and `git diff --check` all pass.
- Final steward verification re-ran the publish/import dry-run, both wasm package smoke and
  three-stream JS/Rust conformance suites, both 7-fixture wasm goldens, wasm32 compile check,
  and the full Rust workspace (188 passed, 2 ignored in the main crate; all integration,
  DTO, wasm, and doc-test suites green). A final release `npm run build:wasm` restored the
  checked-in release artifacts, and release bundler/web conformance plus `git diff --check`
  passed with no generated package changes left in the worktree.
- The implementation audit found no contract drift: `./token-sids` is a standalone ESM
  runtime export with no wasm dependency; its declaration deliberately reuses the published
  root `Token` type. The package tarball includes both JS and declaration files. The user has
  now explicitly authorized committing this previously uncommitted v0.0.9 slice; tagging and
  pushing remain outside this step.
