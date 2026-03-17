# usfm_onion

`usfm_onion` is a Rust-first USFM engine built around one canonical working model: flat tokens.

It currently provides:

- parsing and exact token round-trip
- source-faithful CST projection
- token-first lint, format, and diff
- semantic exports to USJ, USX, HTML, and VREF
- a typed Rust facade
- a typed `wasm-pack` wrapper in [`crates/usfm_onion_wasm`](/Users/willkelly/Documents/Work/Code/usfm_onion/crates/usfm_onion_wasm)
- a shared marker catalog for both Rust and wasm consumers

The design goal is

- parse once
- operate on tokens explicitly
- never silently normalize content on ingest

## Rust Quick Start

```rust
use usfm_onion::{FormatOptions, HtmlOptions, LintOptions, Usfm};

let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");

let parsed = doc.parse();
let issues = doc.lint(LintOptions::default());
let usj = doc.to_usj()?;
let usx = doc.to_usx()?;
let html = doc.to_html(HtmlOptions::default());
let formatted = doc.format(FormatOptions::default());

# Ok::<(), Box<dyn std::error::Error>>(())
```

If you already have tokens, use the token facade directly:

```rust
use usfm_onion::{FormatOptions, TokenStream, parse::parse};

let parsed = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
let mut stream = TokenStream::from_tokens(parsed.tokens);

let formatted_copy = stream.format(FormatOptions::default());
stream.format_mut(FormatOptions::default());

assert!(!formatted_copy.is_empty());
```

## Core Pieces

### `parse`

`parse::parse(source)` produces canonical flat tokens plus lightweight analysis.

Use this when you want the exact working representation for:

- lint
- format
- diff
- exact USFM reconstruction
- editor and wasm token flows

### `cst`

`cst::parse_cst(source)` builds a source-faithful tree over the canonical token stream.

Use this when you want:

- explicit structural nesting
- tree traversal without losing source fidelity
- a tree view that can always flatten back to canonical tokens

### `lint`

Lint is token-first and generic over the minimum lint token surface.

Main entrypoints:

```rust
use usfm_onion::lint::{lint_tokens, lint_usfm, LintOptions};
```

Machine-readable lint ids are exposed through `LintCode`.

### `format`

Formatting is explicit and opt-in.

- `format(...)` is pure
- `format_mut(...)` is explicitly mutating

Main entrypoints:

```rust
use usfm_onion::format::{format, format_mut, format_usfm, FormatOptions};
```

Machine-readable formatter rule ids are exposed through `FormatRule`.

### `diff`

Diff is token-first and SID-block based.

Main entrypoints:

```rust
use usfm_onion::diff::{
    diff_chapter_token_streams,
    diff_usfm_sources,
    diff_usfm_sources_by_chapter,
    BuildSidBlocksOptions,
};
```

### Semantic Exports

Available semantic output modules:

- `usj`
- `usx`
- `html`
- `vref`

Typical direct calls:

```rust
use usfm_onion::html::{HtmlOptions, usfm_to_html};
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::usfm_to_vref_map;
```

## Marker Catalog

The crate exposes a real marker metadata surface instead of only ad hoc helpers.

```rust
use usfm_onion::{marker_catalog, marker_info, is_known_marker};

let catalog = marker_catalog();
let p = marker_info("p");

assert!(catalog.contains("p"));
assert!(is_known_marker("p"));
assert_eq!(p.canonical.as_deref(), Some("p"));
```

Use this when downstream code needs to know:

- whether a marker is valid
- canonical marker identity
- marker category and kind
- note family and note subkind
- inline context
- allowed spec contexts
- default attributes and closing behavior

## WASM

The wasm wrapper is in [`crates/usfm_onion_wasm`](/Users/willkelly/Documents/Work/Code/usfm_onion/crates/usfm_onion_wasm).

It exposes:

- `parse(...)`
- `parseBatch(...)`
- typed `ParsedUsfm` and `ParsedUsfmBatch`
- token-direct `lintTokens`, `formatTokens`, `formatTokensMut`, `diffTokens`
- typed code exports such as `LintCode`, `FormatRule`, `MarkerInfo`, and `UsfmMarkerCatalog`

Build it with:

```bash
wasm-pack build crates/usfm_onion_wasm --target web
wasm-pack build crates/usfm_onion_wasm --target bundler
```

## Benchmarks

Focused benches live in [`benches/`](/Users/willkelly/Documents/Work/Code/usfm_onion/benches):

- `lexer`
- `parse`
- `cst`
- `cst_walk`
- `usj`
- `usx`
- `lint`
- `format`
- `diff`
- `html`
- `omni`

Useful commands:

```bash
cargo bench --bench omni
cargo bench --bench lint
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench format
USFM_BENCH_CORPORA=all cargo bench --bench omni
```

Use `omni` for a broad convenience sweep. Use the focused benches when you want cleaner Criterion reports for one subsystem.
