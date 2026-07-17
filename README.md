# usfm_onion

`usfm_onion` is a Rust-first USFM engine built around one canonical working model: flat tokens.

It currently provides:

- parsing and exact token round-trip
- source-faithful CST projection
- token-first lint, format, and diff
- semantic exports to USJ, USX, HTML, and VREF
- a typed Rust facade
- a typed `wasm-pack` wrapper in [`crates/usfm_onion_wasm`](./crates/usfm_onion_wasm)
- a shared marker catalog for both Rust and wasm consumers

The design goal is

- parse once
- operate on tokens explicitly
- never silently normalize content on ingest

## Documentation

The engine overview, architecture notes, walker design, and performance snapshots live in **[`docs/usfm-onion.html`](./docs/usfm-onion.html)** — open it in any browser. That document is the canonical reference; this README is a quick orientation only.

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

Diff is projection-based: a `DiffSkeleton` interleaves baseline/current sid
blocks, with a coalesced move occupying two linked slots bound to one
decision. See `diff::skeleton` for the full model (`Slot`, `Anchor`,
`DecisionUnit`, `DecisionStatus`, `DecisionUnitKind`).

Main entrypoints:

```rust
use usfm_onion::diff::{
    diff_skeleton,           // a skeleton over two token slices
    diff_skeleton_canonical, // native Token convention: never trusts a carried sid
    diff_skeleton_by_chapter, // one skeleton per book/chapter, from raw USFM source
    merge_diff_blocks,       // pure projection merge from staged decisions
    revert_diff_block,       // revert a single unit; errors on an unknown id
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

The wasm wrapper is in [`crates/usfm_onion_wasm`](./crates/usfm_onion_wasm). All public types are `tsify`-derived — TypeScript declarations come straight from Rust.

The exposed surface is string-in only at construction; token-in entry points exist for the repeated editor operations (lint, format, diff):

- `parse(source)` → `ParsedUsfm`
- `ParsedUsfm.tokens()`, `.lint()`, `.format()`, `.diff()`, `.toUsj()`, `.toUsx()`, `.toHtml()`, `.toVref()`
- top-level `lintTokens`, `formatTokens`, `formatTokensMut`, `diffTokens` for the token-in fast path
- typed exports: `LintCode`, `FormatRule`, `MarkerInfo`, `UsfmMarkerCatalog`

Applications that own complete structural token streams can normalize their SIDs without
loading wasm:

```ts
import { normalizeTokenSids } from "usfm-onion-web/token-sids";

const normalized = normalizeTokenSids(tokens, "GEN");
```

This explicit utility derives from chapter/verse marker structure and leaves its input
untouched. Token diff, merge, and revert APIs continue to trust the SIDs supplied by their
caller, which keeps granular and synthetic token streams usable.

Build it with the root npm scripts:

```bash
npm run build:wasm                       # bundler + web targets, release
npm run build:wasm:bundler:dev           # dev build
npm run check:wasm:web                   # cargo check against wasm32 target
npm run test:wasm                        # scripts/test-web-package.mjs against both targets
```

## Benchmarks

Two criterion harnesses live in [`benches/`](./benches):

- `operations` — string-vs-tokens matrix on a single book (Luke by default)
- `parallelism` — serial vs `rayon` over the full en_ulb corpus

```bash
cargo bench --bench operations
cargo bench --bench parallelism
cargo run --release --example bench_report > BENCH_RESULTS.md
```

Different corpora:

```bash
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench operations
USFM_BENCH_CORPORA=all cargo bench --bench parallelism
```

Snapshots live at [`BENCH_RESULTS.md`](./BENCH_RESULTS.md) (native) and [`BENCH_RESULTS_WASM.md`](./BENCH_RESULTS_WASM.md) (browser).
