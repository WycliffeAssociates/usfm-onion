# Part 3/4 Lane Notes

Date: March 5, 2026

## Lane 1: Scanner/Glossary Compliance

Status: Implemented, tests green.

- `\r` now tokenizes as newline (`src/lexer.rs`).
- `//` now tokenizes as explicit optbreak token (`src/lexer.rs` + parser leaf propagation).
- Optbreak is now carried through parse and serializer paths (`src/parse.rs`, `src/syntax.rs`, `src/usj.rs`, `src/usx.rs`, `src/handle.rs`).
- Glossary text escape handling is now scanner-compliant for `\\|`, `\\/`, `\\~`, `\\\\`, `\\uXXXX`, `\\UXXXXXXXX` (`src/lexer.rs`).
- Scanner-focused tests added for carriage-return semantics, optbreak tokenization, text escapes, and invalid lexical/recovery code stability (`src/lexer.rs`, `tests/scanner.rs`).

## Lane 2: Token/API Naming + Legacy Naming Migration

Status: Implemented, build/tests green.

- Crate/package/test import naming migrated from legacy identifiers to `usfm_onion` (`Cargo.toml`, tests, bins).
- Bench imports migrated to `usfm_onion` (`benches/public_api.rs`).
- Lossless API naming migrated:
  - `to_usj_lossless_*` replaces legacy roundtrip naming.
  - `_lossless_roundtrip` metadata key replaces app-specific legacy key.
- Scanner token API renamed:
  - `RawTokenKind` -> `ScanTokenKind`
  - `LexToken` -> `ScanToken`
  - `LexResult` -> `ScanResult`
- Flat token API naming completed:
  - `TokenKind::HorizontalWhitespace` -> `TokenKind::Whitespace`
  - `TokenKind::VerticalWhitespace` -> `TokenKind::Newline`
  - Diff token-kind keys updated to `whitespace` / `newline` (`src/diff.rs`).

## Lane 3: USJ Compliance + Lossless Split

Status: Implemented with explicit equivalence rules, tests green.

- Strict structural comparison retained.
- Explicit equivalence rules are now narrow and declared in test code (`tests/usj.rs`):
  - object-boundary whitespace equivalence
  - reducible-whitespace string equivalence
  - `$.version` equivalence for expected `3.0` vs actual `3.1` in legacy fixture lane
- USJ test now runs without the prior ignored-fixture env gate.
- Recovery behavior improved for malformed chapter/verse arguments: parser now captures first-word fallback after `\c`/`\v` when strict numeric prefix parsing fails (`src/parse.rs`), preserving invalid-number spans for downstream serializers/lint.

## Lane 4: USX Compliance + Lossless Split

Status: Partially implemented, tests green with explicit exception inventory.

- Hidden legacy/compat filters removed.
- Replaced with explicit fixture exception list (`tests/usx.rs:is_explicit_exception_fixture`).
- USX parity gate is green for non-exception fixtures.
- Added explicit override to run exception fixtures in CI/dev: `USFM_ONION_USX_INCLUDE_EXCEPTIONS=1`.
- Burn-down updates:
  - `usfmjsTests_invalid` removed from explicit exceptions after escaped-backslash marker recovery (`src/lexer.rs`, `src/parse.rs`).
  - `usfmjsTests_greek_verse_objects` and `usfmjsTests_tit_1_12_alignment_zaln_not_start` now pass through targeted malformed-alignment demotion behavior (`src/parse.rs`, `src/usx.rs`).
  - `usfmjsTests_mat-4-6_oldformat`, `usfmjsTests_mat-4-6_whitespace_oldformat`, and `usfmjsTests_acts-1-20_aligned_oldformat` now pass after verse-bridge continuity fixes in malformed alignment flows (`src/usx.rs`).
  - TC compatibility controls are now explicit and available through options:
    - parse: `parse_with_options(ParseOptions { tc_compat: ... })` (`src/parse.rs`)
    - USX: `to_usx_string_with_options(UsxOptions { tc_compat: ... })` (`src/usx.rs`)

Explicit exception inventory currently includes:

- `PublishingVersesNotClosed`
- `57-TIT.greek.oldformat`
- `acts-1-20.aligned.crammed.oldformat`

Current exception-inclusive USX sweep count: 3 mismatches.

Remaining:

- Burn down explicit USX exceptions to reach full no-exception parity.
- Tighten each exception to a documented spec-cited rationale or remove via behavior fixes.

## Lane 5: Modularization + Clippy

Status: Started, not complete.

Completed:

- Removed one dead helper (`consume_until`) from lexer.
- Reduced clippy warning volume in touched modules by fixing API bound/collapsible loops/pattern trims and decode ownership issues (`src/api.rs`, `src/handle.rs`, `src/parse.rs`, `src/usj.rs`, `src/usx.rs`, `src/usx_to_usfm.rs`, `src/vref.rs`, `tests/common/mod.rs`, `tests/usx.rs`, `tests/usj_roundtrip.rs`, `src/bin/compare_usj.rs`).
- `cargo clippy --all-targets --all-features` is now clean for this branch state.

Remaining:

- Aggressive module decomposition for large files (`usj.rs`, `usx.rs`, related helpers).
