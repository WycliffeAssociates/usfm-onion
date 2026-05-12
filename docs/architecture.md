## Build, test, bench

```bash
cargo build
cargo test                              # full suite
cargo test <name>                       # filter by test name substring
cargo test --test <file>                # specific integration test file
cargo run --bin playground              # ad-hoc sampling harness (not a real CLI)
```

Benches use Criterion with a custom harness; only two harnesses are declared in `Cargo.toml`:

```bash
cargo bench --bench operations          # string-vs-tokens matrix on one book
cargo bench --bench parallelism         # serial vs. rayon over full en_ulb corpus
cargo run --release --example bench_report > BENCH_RESULTS.md
```

Note: `benches/README.md` and the top-level `README.md` reference per-subsystem benches (`lexer`, `parse`, `cst`, `lint`, etc.) — those are stale; the actual harnesses are `operations` and `parallelism`. `[profile.bench]` overrides the size-tuned `[profile.release]` for native speed.

The bench corpus selector is read from the env:

```bash
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench operations
USFM_BENCH_CORPORA=all cargo bench --bench parallelism
```

## WASM

The wasm crate is a separate workspace member at `crates/usfm_onion_wasm`. Build via npm scripts at the repo root:

```bash
npm run build:wasm                       # bundler + web targets, release
npm run build:wasm:bundler:dev           # dev build
npm run check:wasm:web                   # cargo check against wasm32 target
npm run test:wasm                        # runs scripts/test-web-package.mjs against both targets
```

`scripts/restore-wasm-package-layout.mjs` runs after each `wasm-pack build` to fix up `pkg-bundler/` and `pkg-web/` so the published npm package layout is consistent.

## Architecture

The pipeline is a layered onion. Each layer is a pure function over the previous layer's output.

1. **`lexer`** (`src/lexer.rs`) — `logos`-based scanner producing `Vec<Lexeme>` / `ScanToken`. Preserves byte spans and whitespace.
2. **`parse`** (`src/parse/mod.rs`) — `parse(source) -> ParseResult` walks lexemes and produces canonical flat `Token`s. Adds ids, sids, merges horizontal whitespace, coalesces multi-lexeme units (attribute lists especially) into single tokens. This is the working representation; tokens know how to round-trip back to USFM bytes exactly (`tokens_to_usfm`).
3. **`cst`** (`src/cst/mod.rs`) — `parse_cst(source) -> CstDocument` projects the flat tokens into a source-faithful tree. The tree adds nesting only; it never invents content and always flattens back to canonical tokens. Used when consumers want structural traversal without losing source fidelity.
4. **Operations over tokens** — `lint` (`src/lint_impl.rs`), `format` (`src/format/mod.rs`), `diff` (`src/diff/mod.rs`). All are generic over a minimum token-surface trait (`LintableToken`, `FormattableToken`, `DiffableToken`) so callers can pass either `&[Token]` (from parse) or `Vec<FormatToken>` (owned, suitable for wasm/JS). Lint and format rules are exposed via stable `LintCode` / `FormatRule` enums plus ICU-style message templates.
5. **Semantic exports** (`src/usj.rs`, `src/usx.rs`, `src/html.rs`, `src/vref.rs`) — Lossy projections. They share a common boundary-recovery contract for unclosed notes (`\f ... \f*`): when a note is left open, parsing stops at the first block-scope marker (chapter/paragraph/header/meta/periph/tablerow/sidebar) so subsequent content isn't accidentally nested inside the note. The shared boundary logic lives in `src/export_tree.rs`; vref/usj/usx route through it, html had its own copy that was unified in commit `8ed7439`.

### Public surface (facade)

`src/api.rs` exposes the user-facing types re-exported from `lib.rs`:

- `Usfm::from_str(source)` — owns a `String`, offers `.parse()`, `.cst()`, `.lint()`, `.format()`, `.to_usj()`, `.to_usx()`, `.to_html()`, `.to_vref()`.
- `ParsedUsfm` — pre-parsed, owned `Vec<FormatToken>` plus `OwnedParseAnalysis`. Use this when amortizing parse cost across multiple operations.
- `TokenStream<T>` — generic over the token kind so callers can format/lint/diff either borrowed `Token<'a>` or owned `FormatToken`.

### Marker catalog

`src/marker_defs.rs` + `src/marker_defs_data.rs` + `src/markers.rs` are the single source of truth for marker metadata (category, kind, note family, inline context, allowed spec contexts, default attributes, closing behavior, whitespace rules). Lookup via `marker_info(name)` / `marker_catalog()` / `is_known_marker(name)`. Lint rules and the structural marker classifier both go through this catalog — do not hardcode marker names in new code.

`src/whitespace.rs` and `src/marker_defs.rs::lookup_marker_whitespace` define structural whitespace requirements that the formatter and lint rules both consume.

## Test data

- `example-corpora/` — real-world USFM corpora used by benches (en_ulb is the canonical "whole Bible" corpus).
- `testData/` — focused fixture sets organized by category (basic, advanced, mandatory, paratextTests, usfmjsTests, samples-from-wild, special-cases, specExamples, biblica, markers.ext, introductions).
- `repos_to_compare/` — reference implementations to compare behavior against.

## Deferred work / planning docs

`docs/plan-*.md` capture roadmaps that have been agreed but not yet implemented. They're real (recently landed as roadmaps, not as code) — treat as design context, not as a description of current behavior. Current plans: marker-data curation, whitespace lint rules, walker architecture, CLI, wasm bindings (tsify migration + parity benches).
