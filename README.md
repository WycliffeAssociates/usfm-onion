# usfm_onion

Rust-first USFM parsing, token operations, structural projection, semantic export, and corpus tooling.

`usfm_onion` is organized around four ideas:

- `tokens`: the canonical flat working representation
- `document_tree`: the canonical structural interchange format
- semantic exports: `USJ`, `USX`, `HTML`, and `VREF`
- explicit mutation only: lint, format, diff, and fix application never happen implicitly on open

The crate is meant to be a reusable engine crate. It is not an editor runtime, and it does not silently normalize content when you ingest it.

WebAssembly bindings still exist, but they now live in the separate wrapper crate at [`crates/usfm_onion_wasm/Cargo.toml`](/Users/willkelly/Documents/Work/Code/usfm_onion/crates/usfm_onion_wasm/Cargo.toml). This README documents the native Rust API first.

## Install

As a library:

```bash
cargo add usfm_onion
```

As a CLI from this repo:

```bash
cargo install --path . --bin usfm-onion
```

During development:

```bash
cargo run --bin usfm-onion -- --help
```

## Public Modules

Use the crate like this:

```rust
use usfm_onion::{convert, diff, document_tree, format, lint, tokens};
```

Top-level public modules:

- `tokens`: tokenization, token reconstruction, and token/file intake helpers
- `document_tree`: structural tree projection and tree/token conversion
- `lint`: token-first linting plus content/path batch helpers
- `format`: token-first formatting plus content/path batch helpers
- `diff`: token-based diffing and revert helpers
- `convert`: semantic export and cross-format conversion helpers
- `model`: shared public types such as `DocumentFormat`, `Token`, and `UsjDocument`

`src/internal/` exists for implementation details only.

## The Core Model

### `tokens`

`Token` is the canonical flat representation for operations.

Use tokens for:

- lint
- format
- diff
- fixes
- token-stream transforms
- exact USFM reconstruction

The main entrypoints are:

```rust
use usfm_onion::tokens;

let tokens = tokens::usfm_to_tokens(source);
let usfm = tokens::tokens_to_usfm(&tokens);
```

Token guarantees:

- newline is explicit
- token spans are char offsets
- token ids are stable only within one invocation
- horizontal whitespace is preserved in token text
- tokenization is designed to accept malformed content rather than hard-fail on it

### `document_tree`

`DocumentTreeDocument` is the structural interchange format.

It follows USFM nesting semantics for things like:

- book
- chapter
- verse
- para
- char
- note
- milestone
- figure
- sidebar
- periph
- table / row / cell
- ref
- unknown / unmatched

Unlike the older scalar-text shape, text is now also an element in the discriminated union:

```rust
use usfm_onion::DocumentTreeElement;

let text = DocumentTreeElement::Text {
    value: "In the beginning".to_string(),
};
```

The key property is:

- `document_tree` is structured
- opening/projecting into it does not mutate content
- it can flatten back to tokens

In plain language, `document_tree` is the thing you use when you want:

- a real tree you can traverse and edit structurally
- the current closest thing to a structured editor tree for USFM
- one canonical intermediate form before projecting to `USJ`, `USX`, `HTML`, or `VREF`

Current implementation note:

- `DocumentTreeDocument.content` now has its own reconstruction path back to USFM / tokens
- the normal USFM -> `document_tree` projection path no longer populates a backing `tokens` vector
- exact round-trip from tree content alone is still incomplete across the full fixture corpus, so this area is still under active refactor

Why that still differs from `USJ` or `USX`:

- `document_tree.content` keeps more editor-oriented structural distinctions than semantic exports do
- `USJ` and `USX` are semantic formats, not exact source reconstruction formats

Examples of source details `document_tree` preserves for USFM-originated content:

- explicit linebreak nodes
- exact marker text for things like book / chapter / verse markers
- spaces after marker numbers or book codes
- repeated spaces inside text nodes
- explicit unmatched / unknown nodes
- explicit note / char closure distinctions that matter for exact token reconstruction

What it does not currently expose as first-class tree metadata:

- parse recoveries are not stored as a separate `recoveries` field on `DocumentTreeDocument`
- if recovery details matter to your workflow, keep the `ParseHandle` / parser output around as well

So the practical split is:

- use `document_tree` when you want a structured editor/interchange tree
- use `USJ` / `USX` when you want semantic interchange formats
- use `tokens` when you want the lowest-level exact working form for lint / format / diff / fix application

Intended future direction:

- `document_tree.content` should eventually be sufficient for faithful reconstruction on its own
- at that point `document.tokens` should no longer be required as a parallel backing store

Typical use:

```rust
use usfm_onion::{document_tree, tokens};

let tokens = tokens::usfm_to_tokens(source);
let tree = document_tree::tokens_to_document_tree(&tokens);
let roundtrip_tokens = document_tree::document_tree_to_tokens(&tree)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Semantic exports

`USJ`, `USX`, `HTML`, and `VREF` are narrower projections built from the canonical pipeline:

- input -> tokens -> document_tree -> output

Use them when you want semantic interchange or rendering, not when you want the highest-fidelity working form.

Another plain-language way to say it:

- `document_tree` is an editor/interchange tree with lossless USFM backing tokens
- `USJ` and `USX` are semantic export formats

That means `USJ` / `USX` may preserve the meaning and structure of a passage while dropping distinctions that only matter if you are trying to reconstruct the original token stream exactly.

## Round-Trip Semantics

This crate makes a hard distinction between structural fidelity and semantic export.

### Exact USFM round-trip

For USFM-originated content, the exact round-trip-preserving layer is:

- `tokens`
- `document_tree`
- `tokens::tokens_to_usfm`

`document_tree` is included in that list as an architectural target, but the tree-content-only reconstruction path is not yet exact across the full fixture corpus.

That is the path to use if exact original USFM matters.

### Semantic USJ / USX round-trip

`USJ` and `USX` are semantic forms.

That means:

- `USFM -> USJ -> USFM` is semantically meaningful, not an attempt to preserve every original byte of source formatting
- `USFM -> USX -> USFM` is likewise semantic
- `USJ` and `USX` input are treated as semantic input, not byte-faithful source containers

So the guarantee is:

- USFM-originated `tokens` / `document_tree` round-trip exactly to USFM
- USJ / USX round-trip semantically, not byte-for-byte to original JSON/XML formatting

Put differently:

- current `document_tree` answers: "give me a structured tree, and attempt reconstruction from tree content alone"
- `USJ` / `USX` answer: "give me a portable semantic document format"

## Quick Start

### USFM -> tokens -> lint / format / diff

```rust
use usfm_onion::{
    diff::{self, BuildSidBlocksOptions},
    format,
    lint,
    tokens,
};

let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning";

let tokens = tokens::usfm_to_tokens(source);

let issues = lint::lint_flat_tokens(&tokens, lint::TokenLintOptions::default());

let formatted = format::format_flat_tokens(&tokens);
let formatted_usfm = tokens::tokens_to_usfm(&formatted.tokens);

let diff = diff::diff_tokens(
    &tokens,
    &tokens::usfm_to_tokens(&formatted_usfm),
    &BuildSidBlocksOptions::default(),
);
```

If you already have a chapter flattened back into canonical tokens, stay on the
token APIs:

- `lint::lint_flat_tokens`
- `format::format_flat_tokens`
- `diff::diff_tokens`

The `*_content` helpers are convenience wrappers that parse and project first.

### USFM -> document_tree -> semantic outputs

```rust
use usfm_onion::{convert, document_tree};

let tree = document_tree::usfm_to_document_tree(source);
let usj = convert::document_tree_to_usj(&tree)?;
let usx = convert::document_tree_to_usx(&tree)?;
let html = convert::document_tree_to_html(&tree, convert::HtmlOptions::default())?;
let vref = convert::document_tree_to_vref(&tree)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### USJ / USX intake

```rust
use usfm_onion::{document_tree, tokens};

let usj_tokens = tokens::usj_to_tokens(usj_json)?;
let usx_tree = document_tree::usx_to_document_tree(usx_xml)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Input / Output Matrix

| Input | Tokens | Document tree | USFM | USJ | USX | HTML | VREF |
| --- | --- | --- | --- | --- | --- | --- | --- |
| USFM | `tokens::usfm_to_tokens` | `document_tree::usfm_to_document_tree` | original or `tokens::tokens_to_usfm` | `convert::usfm_to_usj` | `convert::usfm_to_usx` | `convert::usfm_to_html` | `convert::usfm_to_vref` |
| USJ | `tokens::usj_to_tokens` | `document_tree::usj_to_document_tree` | `convert::from_usj_str` | semantic input | `convert::usj_to_usx` | via tree/tokens | via tree/tokens |
| USX | `tokens::usx_to_tokens` | `document_tree::usx_to_document_tree` | `convert::from_usx_str` | `convert::usx_to_usj` | semantic input | via tree/tokens | via tree/tokens |
| tokens | already there | `document_tree::tokens_to_document_tree` | `tokens::tokens_to_usfm` | `convert::tokens_to_usj` | `convert::tokens_to_usx` | `convert::tokens_to_html` | `convert::tokens_to_vref` |
| document tree | `document_tree::document_tree_to_tokens` | already there | through tokens | `convert::document_tree_to_usj` | `convert::document_tree_to_usx` | `convert::document_tree_to_html` | `convert::document_tree_to_vref` |

## Lint

Lint is token-first.

The core low-level API is:

```rust
use usfm_onion::{lint, tokens};

let tokens = tokens::usfm_to_tokens(source);
let issues = lint::lint_flat_tokens(&tokens, lint::TokenLintOptions::default());
```

You can also lint raw content or paths:

- `lint::lint_content`
- `lint::lint_path`
- `lint::lint_contents`
- `lint::lint_paths`

### Suppressions

Suppressions are currently exact `(code, sid)` matches:

```rust
use usfm_onion::lint::{LintCode, LintSuppression, TokenLintOptions};

let options = TokenLintOptions {
    suppressions: vec![LintSuppression {
        code: LintCode::VerseOutsideExplicitParagraph,
        sid: "GEN 1:1".to_string(),
    }],
    ..TokenLintOptions::default()
};
```

What you can do today:

- disable built-in rules with `disabled_rules`
- suppress exact `(code, sid)` findings
- run token lint directly or via content/path helpers

What you cannot do today:

- register custom lint passes through a public plugin API
- suppress by moving spans
- assume token ids remain stable across separate invocations

### Fixes

`LintIssue` may carry a serializable token-stream fix:

- anchored to the current invocation’s token ids
- intended to be handed back to the same token stream

Use:

- `format::apply_token_fixes`
- `format::apply_fixes`

depending on whether you are applying one or many edits through the format/transform layer.

## Format

Format is also token-first.

The low-level entrypoints are:

- `format::format_flat_tokens`
- `format::format_flat_tokens_with_options`
- `format::format_flat_tokens_with_passes`
- `format::format_flat_tokens_mut`
- `format::format_flat_tokens_mut_with_passes`

The default is “all built-in rules enabled.”

Use `FormatOptions` to selectively include or exclude rules.

### Custom passes

The formatter exposes a public pass API. You can supply your own token pass functions through:

- `format::format_flat_tokens_with_passes`
- `format::format_content_with_passes`

Relevant public types:

- `format::TokenFormatPass`
- `format::BoxedTokenFormatPass`
- `format::TokenTransformResult`
- `format::TokenFix`
- `format::TokenTemplate`

This is the extension story today:

- built-in rules for general normalization
- custom token passes for repo- or workflow-specific rewrite policy

## Diff

Diff is token-based.

Core entrypoints:

- `diff::diff_tokens`
- `diff::diff_content`
- `diff::diff_paths`
- `diff::diff_usfm_by_chapter`

Revert helpers and SID block helpers live in the `diff` module as well.

## WebAssembly / JS

The npm package exports both bundler and browser builds:

```ts
import init, {
  parseContent,
  intoTokens,
  intoDocumentTree,
  lintFlatTokens,
  formatFlatTokens,
  diffFlatTokens,
} from "usfm-onion-web";
```

Token-first JS/WASM usage mirrors the Rust API:

```ts
const parsed = parseContent({
  source,
  format: "usfm",
});

const tokens = intoTokens({ document: parsed });

const issues = lintFlatTokens({ tokens });
const formatted = formatFlatTokens({ tokens });
const diffs = diffFlatTokens({
  baselineTokens: tokens,
  currentTokens: formatted.tokens,
});

const tree = intoDocumentTree(parsed);
```

Use the content helpers only when you do not already have tokens:

- `lintContent`
- `formatContent`
- `diffContent`

Compatibility notes:

- `intoEditorTree` still exists in the wasm package as a compatibility alias, but `intoDocumentTree` is the preferred name.
- `lintTokens` / `formatTokens` / `diffTokens` still exist, and `lintFlatTokens` / `formatFlatTokens` / `diffFlatTokens` are clearer aliases for the same flat-token operations.

## CLI

The CLI mirrors the same operation-oriented model:

- `parse`-style hidden parser internals are not the public story anymore
- commands are about converting, linting, formatting, diffing, and inspection

Check:

```bash
cargo run --bin usfm-onion -- --help
```

## Benchmarks

Bench utilities live in [benches/README.md](benches/README.md).

The practical corpus sweep is:

```bash
cargo bench --bench corpus_matrix --features rayon -- --iterations 3 --markdown
```

The smaller Criterion API benchmark is:

```bash
cargo bench --bench public_api
```

## Corpus Timing Snapshot

This table should be refreshed from the current native benchmark after major API or pipeline changes.

<!-- corpus-bench:begin -->
_Local release measurements, median wall-clock over 3 runs, file-level parallelism via the `rayon` feature._

### `examples.bsb`

- USFM files: 66
- Total files in corpus directory: 74
- Total USFM source size: 4.51 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| usfm -> document_tree | 821.4ms | 193.6ms | 4.24x |
| usfm -> tokens | 382.6ms | 92.8ms | 4.12x |
| lint usfm | 3.38s | 637.9ms | 5.30x |
| format usfm | 930.2ms | 189.0ms | 4.92x |
| usfm -> usj | 730.1ms | 162.6ms | 4.49x |
| usfm -> usx | 893.8ms | 167.6ms | 5.33x |
| usfm -> html | 1.86s | 410.4ms | 4.52x |
| usfm -> vref | 1.18s | 312.3ms | 3.79x |
| usj -> usfm | 125.5ms | 48.2ms | 2.61x |
| usx -> usfm | 259.0ms | 31.4ms | 8.24x |

### `bdf_reg`

- USFM files: 27
- Total files in corpus directory: 33
- Total USFM source size: 1.13 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| usfm -> document_tree | 123.0ms | 43.4ms | 2.84x |
| usfm -> tokens | 114.2ms | 26.2ms | 4.36x |
| lint usfm | 581.3ms | 71.3ms | 8.16x |
| format usfm | 393.6ms | 55.7ms | 7.07x |
| usfm -> usj | 117.1ms | 29.7ms | 3.95x |
| usfm -> usx | 127.9ms | 66.0ms | 1.94x |
| usfm -> html | 486.2ms | 106.2ms | 4.58x |
| usfm -> vref | 156.3ms | 40.3ms | 3.88x |
| usj -> usfm | 9.2ms | 2.7ms | 3.42x |
| usx -> usfm | 12.8ms | 3.6ms | 3.57x |

### `en_ult`

- USFM files: 67
- Total files in corpus directory: 78
- Total USFM source size: 98.43 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| usfm -> document_tree | 19.32s | 4.88s | 3.96x |
| usfm -> tokens | 10.65s | 1.32s | 8.04x |
| lint usfm | 36.17s | 7.06s | 5.13x |
| format usfm | 13.16s | 3.74s | 3.52x |
| usfm -> usj | 15.76s | 3.54s | 4.46x |
| usfm -> usx | 13.14s | 2.39s | 5.49x |
| usfm -> html | 40.05s | 8.99s | 4.46x |
| usfm -> vref | 22.34s | 4.99s | 4.48x |
| usj -> usfm | 2.42s | 543.9ms | 4.44x |
| usx -> usfm | 2.81s | 603.5ms | 4.66x |
<!-- corpus-bench:end -->
