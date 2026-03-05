# USFM Onion Current State

As of: March 5, 2026

## Public API Surface

Library-first surface is exported through:

- `src/lib.rs` (`pub use` exports for parse, token projection, lint, format, diff/revert, conversion, inspect)
- `src/api.rs` (high-level workflow functions)
- `src/parse.rs` + `src/write_exact.rs` (native parse + exact write)
- `src/lint.rs`, `src/format.rs`, `src/diff.rs`, `src/usj.rs`, `src/usj_to_usfm.rs`, `src/usx.rs`, `src/usx_to_usfm.rs`, `src/vref.rs`

Brief binary tools available in `src/bin`:

- `bench.rs`
- `quick_diff_bench.rs`
- `inspect.rs`
- `compare_usj.rs`
- `print_usj.rs`
- `print_usx.rs`

## Principle-to-Evidence Matrix

Status rule used:

- `Implemented` = API + observed behavior + test evidence.
- `Partial` = API + behavior exists, but evidence/coverage is incomplete for principle scope.
- `Missing` = no proven implementation for the principle.

| Principle (from `vision.md`) | Status | Evidence (path + function/test names) | Notes |
| --- | --- | --- | --- |
| Engine-first and consumer-agnostic | Partial | `src/lib.rs` `pub use` surface; `src/api.rs` `project_usfm`, `parse_sources`; `tests/scanner.rs` `scanner_covers_and_reconstructs_all_origin_usfm_fixtures` | Core engine APIs are present and reusable. Principle-level proof of broad consumer neutrality is implicit, not directly tested. |
| Correctness first, then maintainability, then performance | Partial | `tests/roundtrip.rs` `all_origin_usfm_fixtures_roundtrip_exactly`; `tests/usj_roundtrip.rs` `usfm_usj_usfm_matches_origin_disk_strings`; `tests/usj.rs` `usj_matches_origin_json_fixtures` | Current behavior emphasizes correctness checks. Maintainability/performance ordering is a governance priority, not directly enforceable by code/tests alone. |
| Native USFM fidelity is non-negotiable by default | Implemented | `src/parse.rs` `parse`; `src/write_exact.rs` `write_exact`; `src/handle.rs` test `whitespace_projection_does_not_mutate_canonical_source`; `tests/roundtrip.rs` `all_origin_usfm_fixtures_roundtrip_exactly` | Proven exact roundtrip on native USFM fixtures and non-mutating projection path. |
| Flat token streams are first-class | Implemented | `src/lib.rs` exports `FlatToken`, `tokens`, `into_tokens`; `src/api.rs` `into_tokens`, `lint_flat_tokens`, `format_flat_tokens`, `diff_tokens`; `src/api.rs` tests `into_tokens_preserves_horizontal_whitespace`, `push_whitespace_matches_flat_projection_policy` | Token-first surface is public and exercised in tests. |
| Mutations are strict opt-in | Implemented | `src/api.rs` `apply_token_fixes`, `format_flat_tokens`, `from_usj`, `from_usx`; `src/handle.rs` test `whitespace_projection_does_not_mutate_canonical_source`; `tests/roundtrip.rs` `all_origin_usfm_fixtures_roundtrip_exactly` | Mutation paths are explicit function calls; parse/projection baseline is proven non-mutating. |
| Spec USJ/USX are interchange; fidelity may require a distinct internal representation | Partial | `src/api.rs` `into_usj`, `into_usx`, `from_usj`, `from_usx`; `src/usj.rs` `to_usj_roundtrip_value`, `to_usj_roundtrip_document`; `tests/usj_roundtrip.rs` `usfm_usj_usfm_matches_origin_disk_strings`; `tests/usx_roundtrip.rs` `usx_roundtrip_matches_origin_for_targeted_fixture` | Interchange and roundtrip helpers exist. Formal wording boundary versus official spec semantics is not finalized in this pass. |
| Static analysis is a directional extension, not current core scope | Missing | `src/lint.rs` `lint`, `lint_tokens` (current lint scope only) | Current implementation covers linting, not broader static-analysis feature set. |
| Do not become a UI/editor monolith | Partial | `src/lib.rs` exports are engine-centric; `src/api.rs` focuses on parse/lint/format/diff/conversion; `tests/scanner.rs` and `tests/roundtrip.rs` validate engine behavior | Repo is engine-first today, but some editor-oriented data projection exists (`into_editor_tree`), so strict boundary is not yet codified as an explicit contract. |

## Test Evidence Summary

- Verified locally in this pass: `cargo test -q` completed successfully.
- Current observed outcomes:
  - Unit tests: passing
  - Integration tests: passing (`roundtrip`, `scanner`, `usj`, `usj_roundtrip`, `usx`, `usx_roundtrip`)
- Warnings observed:
  - Unused helper functions in `tests/common/mod.rs` (`collect_origin_usfm_json_pairs`, `collect_origin_usfm_xml_pairs`) under some test target combinations.

## Known Gaps

- Principle-level governance (priority ordering and anti-monolith boundary) is documented but only partially enforceable via current tests.
- Static analysis beyond lint is not yet present as a concrete API/feature lane.
- USJ/USX interchange boundary language is currently practical and implementation-led; full official-doc boundary wording is deferred.
- Repo and package technical naming still includes legacy identifiers in code/package metadata; docs now use `usfm_onion` terminology.

## Open Questions + Next Action

1. **Formal boundary wording for Lossless USJ Tree vs spec USJ**
   - Next action: compare current `src/usj.rs` roundtrip behavior and shape against official docs, then finalize terminology and guarantees in both docs.

2. **Technical rename workstream to eliminate remaining legacy identifiers**
   - Next action: inventory all remaining legacy name references in code/package metadata and define a safe rename plan (crate/package/module/docs/tests) as a separate change set.
