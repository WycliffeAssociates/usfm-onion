# usfm_onion

Rust-first USFM parsing, token operations, structural projection, semantic export, and corpus tooling.

`usfm_onion` is organized around four ideas:

- `tokens`: the canonical flat working representation
- `cst`: the canonical source-faithful tree
- `ast`: the canonical semantic tree and export tree
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
use usfm_onion::{ast, convert, cst, diff, format, lint, tokens};
```

Top-level public modules:

- `tokens`: tokenization, token reconstruction, and token/file intake helpers
- `cst`: source-faithful tree projection and tree/token conversion
- `ast`: semantic tree projection and tree/token conversion
- `lint`: token-first linting plus content/path batch helpers
- `format`: token-first formatting plus content/path batch helpers
- `diff`: token-based diffing and revert helpers
- `convert`: semantic export and cross-format conversion helpers
- `model`: shared public types such as `DocumentFormat`, `Token`, and `UsjDocument`

`src/internal/` exists for implementation details only.

## The Core Model

### Pipeline

The native USFM pipeline is:

- lex: raw source -> scan tokens
- parse: scan tokens -> internal syntax/analysis state
- project: internal parse state -> public `tokens`, `cst`, `ast`, or semantic exports

So lex and parse are two distinct steps. Public `tokens::usfm_to_tokens(...)` is a convenience that parses first and then projects canonical public tokens.

For USFM input, the public mental model should be:

- `USFM -> tokens -> CST -> AST -> semantic exports`

For semantic inputs such as `USJ` and `USX`, the important thing is different:

- `USJ/USX -> AST` is the primary semantic intake path
- `USJ/USX -> tokens` and `USJ/USX -> CST` are generated projections, not source-faithful reconstructions of the original JSON/XML bytes

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
- exact USFM round-trip is guaranteed at the token layer

If you want an exhaustively matchable token API instead of `TokenKind + fields`, use `TokenVariant`:

```rust
use usfm_onion::{TokenVariant, tokens};

for token in tokens::usfm_to_token_variants(source) {
    match token {
        TokenVariant::Marker { marker, text, .. } => {
            let _ = (marker, text);
        }
        TokenVariant::EndMarker { marker, .. } => {
            let _ = marker;
        }
        TokenVariant::Text { text, .. } => {
            let _ = text;
        }
        TokenVariant::Newline { .. }
        | TokenVariant::OptBreak { .. }
        | TokenVariant::Milestone { .. }
        | TokenVariant::MilestoneEnd { .. }
        | TokenVariant::Attributes { .. }
        | TokenVariant::BookCode { .. }
        | TokenVariant::Number { .. } => {}
    }
}
```

### `cst`

`CstDocument` is the source-faithful tree.

Use the CST when you want:

- tree traversal without giving up exact source coverage
- a structured representation that still carries canonical tokens
- a source-faithful view of containers, leaves, and boundaries
- the closest public representation to the parser’s internal structure

The key property is:

- `tokens` own the exact flat source text
- `cst` owns structure and references canonical tokens
- `cst::cst_to_tokens(...)` gives you the canonical tokens back exactly

Typical use:

```rust
use usfm_onion::{cst, tokens};

let cst = cst::parse_usfm(source);
let tokens = cst::cst_to_tokens(&cst);
let usfm = tokens::tokens_to_usfm(&tokens);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Why reach for the CST instead of tokens:

- you want a tree, not a flat stream
- you still care about source-faithful structure
- you may want to inspect note / char / paragraph nesting without collapsing back to semantic export shapes

Why reach for tokens instead of the CST:

- lint, format, diff, and fixes are currently token-first
- downstream workflows are often simpler if everything flattens back to one canonical stream
- token streams are the easiest place to reason about explicit edits and exact round-trip

So today the practical rule is:

- use `tokens` for operations
- use `cst` for source-faithful tree traversal
- flatten back to tokens before running operations

That choice is intentional. A future version could accept CST input for more of
these operations because the CST can always return canonical tokens, but the
current public API stays token-first because it is simpler for downstream
consumers to reason about one flat working stream than to require a tree.

### `ast`

`AstDocument` is the semantic tree format.

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
use usfm_onion::AstElement;

let text = AstElement::Text {
    value: "In the beginning".to_string(),
};
```

The key property is:

- `ast` is structured
- opening/projecting into it does not mutate content
- it can be projected back into tokens / USFM

In plain language, `ast` is the thing you use when you want:

- a real tree you can traverse semantically
- a normalized tree before projecting to USJ / USX / HTML / VREF
- one canonical intermediate form before projecting to `USJ`, `USX`, `HTML`, or `VREF`

What `ast` is not:

- it is not the exact-lossless source of truth
- it should not be treated as the byte-faithful round-trip layer for arbitrary USFM

Why it still differs from `USJ` or `USX`:

- `ast.content` keeps more structural distinctions than semantic exports do
- `USJ` and `USX` are semantic formats, not exact source reconstruction formats

Examples of source details `ast` may preserve:

- explicit linebreak nodes
- exact marker text for things like book / chapter / verse markers
- spaces after marker numbers or book codes
- repeated spaces inside text nodes
- explicit unmatched / unknown nodes
- explicit note / char closure distinctions that matter for exact token reconstruction

What it does not currently expose as first-class tree metadata:

- parse recoveries are not stored as a separate `recoveries` field on `AstDocument`
- if recovery details matter to your workflow, use the internal parser-facing APIs rather than relying on the public AST alone

So the practical split is:

- use `ast` when you want a structured semantic tree
- use `cst` when you want a structured source-faithful tree
- use `USJ` / `USX` when you want semantic interchange formats
- use `tokens` when you want the exact working form for lint / format / diff / fix application

Typical use:

```rust
use usfm_onion::{ast, cst};

let cst = cst::parse_usfm(source);
let tree = ast::cst_to_ast(&cst);
# let roundtrip_tokens = ast::ast_to_tokens(&tree)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Why reach for the AST instead of the CST:

- you want semantic normalization, not source-faithful trivia
- you want one tree for `USJ`, `USX`, `HTML`, and `VREF`
- you do not want each exporter to reinvent nesting separately

Why reach for the CST instead of the AST:

- you still care about exact source-oriented structure
- you want to preserve a clearer connection to the original token stream
- you are building tooling where trivia and boundary placement still matter

### Semantic exports

`USJ`, `USX`, `HTML`, and `VREF` are narrower projections built from the canonical pipeline:

- input -> tokens -> cst -> ast -> output

Use them when you want semantic interchange or rendering, not when you want the highest-fidelity working form.

Another plain-language way to say it:

- `cst` is the source-faithful tree
- `ast` is the semantic structural tree
- `USJ` and `USX` are semantic export formats
- `tokens` are the lossless source-faithful layer

That means `USJ` / `USX` may preserve the meaning and structure of a passage while dropping distinctions that only matter if you are trying to reconstruct the original token stream exactly.

## Round-Trip Semantics

This crate makes a hard distinction between structural fidelity and semantic export.

### Exact USFM round-trip

For USFM-originated content, the exact round-trip-preserving layer is:

- `tokens`
- `tokens::tokens_to_usfm`
- optionally `cst`, because the CST carries the canonical tokens it was built from

If exact original USFM matters, keep tokens.

### Semantic USJ / USX round-trip

`USJ` and `USX` are semantic forms.

That means:

- `USFM -> USJ -> USFM` is semantically meaningful, not an attempt to preserve every original byte of source formatting
- `USFM -> USX -> USFM` is likewise semantic
- `USJ` and `USX` input are treated as semantic input, not byte-faithful source containers

So the guarantee is:

- USFM-originated `tokens` round-trip exactly to USFM
- USJ / USX round-trip semantically, not byte-for-byte to original JSON/XML formatting

Put differently:

- current `ast` answers: "give me a structured tree, and attempt semantic export from tree content"
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

### USFM -> CST -> AST -> semantic outputs

```rust
use usfm_onion::{ast, convert, cst};

let cst = cst::parse_usfm(source);
let ast = ast::cst_to_ast(&cst);
let usj = convert::ast_to_usj(&ast)?;
let usx = convert::ast_to_usx(&ast)?;
let html = convert::ast_to_html(&ast, convert::HtmlOptions::default())?;
let vref = convert::ast_to_vref(&ast)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### USJ / USX intake

```rust
use usfm_onion::{ast, tokens};

let usj_tokens = tokens::usj_to_tokens(usj_json)?;
let usx_tree = ast::usx_to_ast(usx_xml)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Input / Output Matrix

| Input  | Tokens                   | CST                      | AST                  | USFM                                 | USJ                      | USX                      | HTML                      | VREF                      |
| ------ | ------------------------ | ------------------------ | -------------------- | ------------------------------------ | ------------------------ | ------------------------ | ------------------------- | ------------------------- |
| USFM   | `tokens::usfm_to_tokens` | `cst::parse_usfm`        | `ast::usfm_to_ast`   | original or `tokens::tokens_to_usfm` | `convert::usfm_to_usj`   | `convert::usfm_to_usx`   | `convert::usfm_to_html`   | `convert::usfm_to_vref`   |
| USJ    | `tokens::usj_to_tokens`  | `cst::usj_to_cst`        | `ast::usj_to_ast`    | `convert::from_usj_str`              | semantic input           | `convert::usj_to_usx`    | via ast/tokens            | via ast/tokens            |
| USX    | `tokens::usx_to_tokens`  | `cst::usx_to_cst`        | `ast::usx_to_ast`    | `convert::from_usx_str`              | `convert::usx_to_usj`    | semantic input           | via ast/tokens            | via ast/tokens            |
| tokens | already there            | `cst::tokens_to_cst`     | `ast::tokens_to_ast` | `tokens::tokens_to_usfm`             | `convert::tokens_to_usj` | `convert::tokens_to_usx` | `convert::tokens_to_html` | `convert::tokens_to_vref` |
| CST    | `cst::cst_to_tokens`     | already there            | `ast::cst_to_ast`    | through tokens                       | through AST              | through AST              | through AST               | through AST               |
| AST    | `ast::ast_to_tokens`     | generated through tokens | already there        | through tokens                       | `convert::ast_to_usj`    | `convert::ast_to_usx`    | `convert::ast_to_html`    | `convert::ast_to_vref`    |

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
  usfmToTokens,
  intoAst,
  lintFlatTokens,
  formatFlatTokens,
  diffTokens,
} from "usfm-onion-web";
```

Token-first JS/WASM usage mirrors the Rust API:

```ts
const parsed = parseContent({
  source,
  format: "usfm",
});

const tokens = intoTokens({ document: parsed });
const sameTokens = usfmToTokens(source);

const issues = lintFlatTokens({ tokens });
const formatted = formatFlatTokens({ tokens });
const diffs = diffTokens({
  baselineTokens: tokens,
  currentTokens: formatted.tokens,
});

const tree = intoAst(parsed);
```

Use the content helpers only when you do not already have tokens:

- `lintContent`
- `formatContent`
- `diffContent`

Current wasm `ast` contract:

- tree values are real runtime JSON objects
- the generated `.d.ts` does not currently expose a polished recursive TypeScript union for that tree
- downstream code should narrow/validate tree nodes explicitly instead of assuming compile-time exhaustiveness from the package typings

Canonical JS names now follow the Rust modules more closely:

- `usfmToTokens` / `usjToTokens` / `usxToTokens`
- `tokensToUsfm`
- `usfmToAst` / `tokensToAst` / `astToTokens`
- `tokensToUsj` / `tokensToUsx` / `tokensToHtml` / `tokensToVref`
- `lintFlatTokens`, `formatFlatTokens`, `diffTokens`

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

| Operation      |  Serial | Parallel | Speedup |
| -------------- | ------: | -------: | ------: |
| usfm -> cst    | 329.7ms |   54.2ms |   6.08x |
| usfm -> ast    | 333.1ms |  133.8ms |   2.49x |
| usfm -> tokens | 403.0ms |   50.1ms |   8.05x |
| lint usfm      | 544.3ms |   94.6ms |   5.76x |
| format usfm    | 534.9ms |   92.2ms |   5.80x |
| usfm -> usj    | 545.0ms |  110.4ms |   4.94x |
| usfm -> usx    | 727.1ms |  152.2ms |   4.78x |
| usfm -> html   | 461.1ms |  109.8ms |   4.20x |
| usfm -> vref   | 349.1ms |   61.6ms |   5.67x |
| usj -> usfm    |  78.0ms |   20.5ms |   3.80x |
| usx -> usfm    |  82.0ms |   15.7ms |   5.22x |

### `bdf_reg`

- USFM files: 27
- Total files in corpus directory: 33
- Total USFM source size: 1.13 MiB

| Operation      |  Serial | Parallel | Speedup |
| -------------- | ------: | -------: | ------: |
| usfm -> cst    |  38.4ms |    7.5ms |   5.12x |
| usfm -> ast    |  47.2ms |    9.2ms |   5.14x |
| usfm -> tokens |  36.8ms |    9.4ms |   3.93x |
| lint usfm      |  64.6ms |   13.8ms |   4.69x |
| format usfm    |  72.1ms |   12.9ms |   5.58x |
| usfm -> usj    |  82.0ms |   16.5ms |   4.97x |
| usfm -> usx    | 114.1ms |   22.1ms |   5.17x |
| usfm -> html   |  66.7ms |   11.6ms |   5.77x |
| usfm -> vref   |  50.5ms |   10.1ms |   5.00x |
| usj -> usfm    |   9.3ms |    2.5ms |   3.75x |
| usx -> usfm    |  11.7ms |    2.9ms |   4.10x |

### `en_ult`

- USFM files: 67
- Total files in corpus directory: 78
- Total USFM source size: 98.43 MiB

| Operation      | Serial | Parallel | Speedup |
| -------------- | -----: | -------: | ------: |
| usfm -> cst    |  3.71s |  643.1ms |   5.77x |
| usfm -> ast    |  6.70s |    1.18s |   5.67x |
| usfm -> tokens |  3.23s |  547.9ms |   5.90x |
| lint usfm      |  8.07s |    1.22s |   6.61x |
| format usfm    |  7.56s |    1.35s |   5.60x |
| usfm -> usj    | 10.73s |    1.97s |   5.45x |
| usfm -> usx    |  7.67s |    1.21s |   6.35x |
| usfm -> html   | 11.11s |    1.92s |   5.79x |
| usfm -> vref   |  8.30s |    1.28s |   6.50x |
| usj -> usfm    |  2.98s |  474.8ms |   6.27x |
| usx -> usfm    |  3.08s |  509.6ms |   6.04x |

<!-- corpus-bench:end -->