# usfm_onion

Rust-first USFM parsing, linting, formatting, conversion, diffing, and round-trippable projection.

`usfm_onion` is meant to be a reusable engine crate, not an editor runtime. The public API is organized around first-class workflows:

- `parse`: parse, inspect, recover, and project
- `lint`: lint parsed content or flat token streams
- `format`: normalize flat token streams with built-in rules and optional custom passes
- `convert`: convert between USFM, USJ, USX, HTML, editor tree, and VREF views
- `diff`: semantic diffing over tokenized USFM content
- `model`: shared public types such as `DocumentFormat`, `FlatToken`, `UsjDocument`, and `VrefMap`

WebAssembly bindings still exist, but the crate is documented here from the native Rust perspective first. JS/browser packaging notes live in [`pkg-web/README.md`](pkg-web/README.md).

## Document Tree Typing

The wasm package returns document-tree values as runtime JSON objects.

Important current limitation:

- the generated TypeScript declarations do not yet expose a polished recursive discriminated union for the document tree
- tree-oriented functions therefore appear as `any` in the generated `.d.ts`

Practical guidance for downstream code:

- treat document-tree values as runtime data
- validate or narrow node shapes explicitly
- do not assume the package typings alone provide exhaustive compile-time coverage of every tree node shape

## What This Crate Optimizes For

- Native USFM fidelity first
- Token streams as a first-class public surface
- Explicit mutation boundaries
- Practical batch APIs for multi-file workflows
- Clear separation between reusable engine logic and downstream app policy

What that means in practice:

- Parsing does not silently rewrite your source.
- Formatting is an explicit mutation step.
- Linting can operate on either a parsed document or already-projected flat tokens.
- Conversions are explicit and named.
- Lossless interchange forms exist when you need to carry exact round-trip material forward.

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

## Read This API In One Minute

The main mental model is:

- `ParseHandle` is the rich parsed document
- `FlatToken` streams are the first-class linear working representation
- format/lint/diff work naturally on flat tokens
- conversion APIs can start from a `ParseHandle` or from flat tokens

Minimal example:

```rust
use usfm_onion::{
    DocumentFormat,
    convert,
    format,
    lint,
    parse,
};

let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning";

let handle = parse::parse_content(source, DocumentFormat::Usfm)?;
let tokens = parse::into_tokens(&handle, parse::IntoTokensOptions::default());

let issues = lint::lint_flat_tokens(&tokens, lint::TokenLintOptions::default());

let result = format::format_flat_tokens_with_options(
    &tokens,
    format::FormatOptions::default(),
);
let formatted_usfm = parse::into_usfm_from_tokens(&result.tokens);

let usj = convert::into_usj(&handle);
let usx = convert::into_usx(&handle)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Module Map

Use the crate like this:

```rust
use usfm_onion::{parse, lint, format, diff, convert, model};
```

The modules are intentionally capability-shaped:

- `parse`: parse source, inspect recoveries, lex, project, and materialize flat tokens
- `lint`: built-in structural/content lint rules over parsed documents or flat tokens
- `format`: built-in normalization rules plus custom token passes
- `diff`: semantic diffing and chapter/block helpers
- `convert`: USFM/USJ/USX/HTML/editor-tree/VREF conversion helpers
- `model`: public shared types and USJ walkers

`src/internal/` exists for implementation details. It is not the intended user-facing map.

## First-Class Citizens

These are the workflows the crate treats as primary:

| Concern | Primary entrypoints | Notes |
| --- | --- | --- |
| Parse | `parse::parse_content`, `parse::parse_path`, `parse::parse_usfm_content` | Produces a `ParseHandle` |
| Token projection | `parse::into_tokens`, `parse::into_tokens_from_content`, `parse::into_usfm_from_tokens` | Token streams are first-class |
| Lint | `lint::lint_document`, `lint::lint_content`, `lint::lint_flat_tokens`, `lint::lint_tokens` | Parsed and token-first both supported |
| Format | `format::format_content_with_options`, `format::format_flat_tokens_with_options`, `format::format_flat_tokens_with_passes` | Built-in rules plus custom passes |
| Diff | `diff::diff_content`, `diff::diff_tokens`, `diff::diff_usfm_by_chapter` | Token-semantic diffing |
| Convert | `convert::into_usj`, `convert::into_usx`, `convert::from_usj`, `convert::from_usx`, `convert::convert_content` | Explicit representation changes |

## Handle vs Tokens

`ParseHandle` and flat tokens are related but not interchangeable.

Use `ParseHandle` when you need:

- recoveries
- document structure
- chapter/book metadata
- conversion to USJ, USX, HTML, editor tree, or VREF from parsed content

Use flat tokens when you need:

- token-first linting
- formatting
- token-level transforms
- diffing
- token-to-USFM reconstruction

In code:

```rust
use usfm_onion::{DocumentFormat, parse};

let handle = parse::parse_content(source, DocumentFormat::Usfm)?;
let tokens = parse::into_tokens(&handle, parse::IntoTokensOptions::default());
let roundtrip_usfm = parse::into_usfm_from_tokens(&tokens);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The handle is the richer parse result. The tokens are a projected linear view derived from it.

## Intake -> Output Matrix

This is the current core conversion surface.

| Output | usfm intake | usx intake | usj intake | tokens intake |
| --- | --- | --- | --- | --- |
| `into_tokens` | direct via `parse::into_tokens_from_usfm_content/path` or `parse::into_tokens_from_content/path` | indirect via `usx -> usfm -> parse -> tokens` | indirect via `usj -> usfm -> parse -> tokens` | n/a |
| `into_usfm` | direct from original source or `parse::into_usfm_from_tokens` | direct via `convert::from_usx` or `convert::usx_content_to_usfm` | direct via `convert::from_usj` or `convert::usj_content_to_usfm` | direct via `parse::into_usfm_from_tokens` |
| `into_usj` | direct via `convert::into_usj` or `convert::usfm_content_to_usj` | indirect via `usx -> usfm -> parse -> into_usj` | direct parse + identity-ish decode path | indirect via `convert::into_usj_from_tokens` |
| `into_usx` | direct via `convert::into_usx` or `convert::usfm_content_to_usx` | direct parse + identity-ish decode path | indirect via `usj -> usfm -> parse -> into_usx` | indirect via `convert::into_usx_from_tokens` |
| `into_vref` | direct via `convert::into_vref` | indirect via `usx -> usfm -> parse -> into_vref` | indirect via `usj -> usfm -> parse -> into_vref` | indirect via `convert::into_vref_from_tokens` |

## Semantic Exports vs Round-Trip-Preserving Layers

### Regular USJ

`convert::into_usj` produces a semantic JSON document suitable for interchange and downstream structured processing.

It preserves the document structure and content, but it does not try to embed the entire original USFM source verbatim.

Use it when:

- you want a typed JSON document
- you want to inspect structure or feed another system
- exact source reproduction is not the payload itself

### Regular USX

`convert::into_usx` produces the XML serialization of the parsed document.

It is an interchange/rendering form, not an "embed every byte of the original USFM source" container.

Use it when:

- you want normal USX output
- you need XML interchange

### editorTree

`convert::into_editor_tree` is the structured editor-load projection.

Use it when:

- you want a non-mutating editor load model
- you want explicit linebreak and source-adjacent structure in a typed tree
- you want to open/project content without silently formatting or rewriting it

Loading `editorTree` is projection, not normalization. Walking it yourself does not apply fixes or formatting.

### Tokens and the internal document tree

If you care about round-trip preservation, the real fidelity layer is:

- the parsed `ParseHandle`
- projected flat tokens
- exact reconstruction via `parse::into_usfm_from_tokens`

That is the layer that preserves the information needed for faithful USFM reconstruction. USJ and USX are semantic exports, not the primary round-trip-preserving representation.

## Parsing

Common parse entrypoints:

- `parse::parse_content`
- `parse::parse_path`
- `parse::parse_usfm_content`
- `parse::parse_usj_content`
- `parse::parse_usx_content`
- `parse::lex`
- `parse::recoveries`
- `parse::debug_dump`

Example:

```rust
use usfm_onion::{DocumentFormat, parse};

let handle = parse::parse_content(source, DocumentFormat::Usfm)?;
let recoveries = parse::recoveries(&handle);
let dump = parse::debug_dump(&handle, parse::DebugDumpOptions::default());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`parse::IntoTokensOptions` controls projection behavior such as horizontal whitespace merging.

## Linting

There are two public lint styles:

- parsed-document linting via `lint::lint_document` and `lint::lint_content`
- token-first linting via `lint::lint_flat_tokens` and `lint::lint_tokens`

### Parsed-document linting

Use this when you want parse recoveries and projected token views folded into one flow:

```rust
use usfm_onion::{DocumentFormat, lint};

let issues = lint::lint_content(
    source,
    DocumentFormat::Usfm,
    lint::LintOptions::default(),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Token-first linting

Use this when you already have a flat token stream and want linting to stay token-native:

```rust
use usfm_onion::{lint, parse};

let tokens = parse::into_tokens(&handle, parse::IntoTokensOptions::default());
let issues = lint::lint_flat_tokens(&tokens, lint::TokenLintOptions::default());
```

### What You Can Ignore

The linter currently supports two control surfaces:

- `disabled_rules: Vec<LintCode>`
- `suppressions: Vec<LintSuppression>`

Example:

```rust
use usfm_onion::lint::{LintCode, LintSuppression, TokenLintOptions};

let options = TokenLintOptions {
    disabled_rules: vec![LintCode::MissingSeparatorAfterMarker],
    suppressions: vec![
        LintSuppression {
            code: LintCode::VerseOutsideExplicitParagraph,
            span: 120..128,
        }
    ],
    allow_implicit_chapter_content_verse: false,
};
```

Important limits:

- `disabled_rules` disables a built-in lint code everywhere for that run
- `suppressions` only suppress exact `(code, span)` matches
- suppressions are brittle across edits because spans move
- there is no public custom lint plugin/pass API yet
- there is no concept of "ignore this by SID instead of span" yet

If you need downstream project-specific lint policy, the current approach is:

- run built-in lint
- filter or post-process findings in your own crate

## Formatting

Formatting is rule-based.

Default behavior enables all built-in formatter rules:

```rust
use usfm_onion::format::FormatOptions;

let options = FormatOptions::default();
```

If you want only a narrow set of rules:

```rust
use usfm_onion::format::{FormatOptions, FormatRule};

let options = FormatOptions::only(&[
    FormatRule::CollapseWhitespaceInText,
    FormatRule::NormalizeSpacingAfterParagraphMarkers,
]);
```

If you want the default ruleset minus a few:

```rust
use usfm_onion::format::{FormatOptions, FormatRule};

let options = FormatOptions::excluding(&[
    FormatRule::RecoverMalformedMarkers,
    FormatRule::RemoveDuplicateVerseNumbers,
]);
```

### Built-In Rule Examples

Some of the more important built-in rules:

- `RecoverMalformedMarkers`
  - before: `text \zzbad`
  - after: `text \zz bad`
- `BridgeConsecutiveVerseMarkers`
  - before: `\v 1 ... \v 2 ... \v 3 ...`
  - after: `\v 1-3 ...`
- `RemoveBridgeVerseEnumerators`
  - before: `\v 1-3 1. James ... 2. Count it ... 3. because ...`
  - after: `\v 1-3 James ... Count it ... because ...`
- `MoveChapterLabelAfterChapterMarker`
  - before: `\cl Chapter 1 \c 1`
  - after: `\c 1 \cl Chapter 1`
- `InsertDefaultParagraphAfterChapterIntro`
  - before: chapter intro content reaches verse text with no paragraph marker
  - after: a default paragraph marker is inserted before verse-bearing content

### Custom Format Passes

The formatter does have a plugin-like extension point.

Built-in rules run first. Then any custom `TokenFormatPass` implementations run in order against the working token buffer.

Example:

```rust
use usfm_onion::format::{
    BoxedTokenFormatPass, FormatOptions, TokenFormatPass, format_flat_tokens_with_passes,
};
use usfm_onion::model::FlatToken;

struct ReplaceDoubleSpacePass;

impl TokenFormatPass<FlatToken> for ReplaceDoubleSpacePass {
    fn label(&self) -> &str {
        "replace-double-space"
    }

    fn apply(&self, tokens: &mut Vec<FlatToken>) {
        for token in tokens {
            if token.kind == usfm_onion::TokenKind::Text {
                token.text = token.text.replace("  ", " ");
            }
        }
    }
}

let passes: Vec<BoxedTokenFormatPass<FlatToken>> = vec![Box::new(ReplaceDoubleSpacePass)];
let result = format_flat_tokens_with_passes(
    &tokens,
    FormatOptions::default(),
    &passes,
);
```

What custom passes can do:

- inspect and mutate the token vector
- add, remove, or replace tokens
- implement downstream house style or cleanup policy

What custom passes do not get:

- direct parse tree access
- built-in incremental fix targeting by lint code
- a separate plugin registry system

If you need parse-tree-aware custom formatting, the usual pattern is:

- parse and project first
- inspect the `ParseHandle` yourself
- then run token formatting with custom passes

## Conversion

Main conversion entrypoints:

- `convert::convert_content`
- `convert::from_usj`
- `convert::from_usx`
- `convert::into_usj`
- `convert::into_usx`
- `convert::into_html`
- `convert::into_editor_tree`
- `convert::into_vref`

Examples:

```rust
use usfm_onion::{DocumentFormat, convert, parse};

let usfm = convert::from_usj(&usj_document)?;
let xml = convert::usfm_content_to_usx(source)?;

let handle = parse::parse_content(source, DocumentFormat::Usfm)?;
let html = convert::into_html(&handle, convert::HtmlOptions::default());
let vref = convert::into_vref(&handle);
# Ok::<(), Box<dyn std::error::Error>>(())
```

HTML is part of `convert` because it is an output representation, not a parser/linter concern.

## Diffing

Common diff entrypoints:

- `diff::diff_content`
- `diff::diff_tokens`
- `diff::diff_usfm_by_chapter`

Example:

```rust
use usfm_onion::{DocumentFormat, diff};
use usfm_onion::model::TokenViewOptions;
use usfm_onion::diff::BuildSidBlocksOptions;

let changes = diff::diff_content(
    baseline,
    DocumentFormat::Usfm,
    current,
    DocumentFormat::Usfm,
    &TokenViewOptions::default(),
    &BuildSidBlocksOptions::default(),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Batch APIs and Parallelism

Many content/path batch APIs accept `BatchExecutionOptions`.

Use:

```rust
use usfm_onion::model::BatchExecutionOptions;

let batch = BatchExecutionOptions::parallel();
```

This is intended for file-level parallelism. It is not an intra-file parser scheduler.

The CLI uses parallel batch execution where it makes sense for multi-path workflows.

## Corpus Timing Snapshot

The table below is meant to answer a practical question: "what does this feel like on a whole corpus, not a toy file?"

It is generated from [benches/corpus_matrix.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/benches/corpus_matrix.rs), using these corpora:

- `example-corpora/examples.bsb`: full Bible, typical unaligned corpus
- `example-corpora/bdf_reg`: smaller NT-only corpus
- `example-corpora/en_ult`: aligned corpus with heavier token/conversion load

Run it locally with:

```bash
cargo bench --bench corpus_matrix --features rayon -- --iterations 3 --markdown
```

<!-- corpus-bench:begin -->
_Local release measurements, median wall-clock over 3 runs, file-level parallelism via the `rayon` feature._

### `examples.bsb`

- USFM files: 66
- Total files in corpus directory: 74
- Total USFM source size: 4.51 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| parse usfm | 91.7ms | 30.6ms | 3.00x |
| project tokens | 155.4ms | 52.9ms | 2.94x |
| lint usfm | 771.4ms | 231.3ms | 3.33x |
| format usfm | 481.2ms | 120.0ms | 4.01x |
| usfm -> usj | 379.5ms | 115.0ms | 3.30x |
| usfm -> usx | 387.8ms | 93.5ms | 4.15x |
| usfm -> html | 486.8ms | 123.5ms | 3.94x |
| usfm -> vref | 100.0ms | 26.6ms | 3.76x |
| usj -> usfm | 58.1ms | 15.0ms | 3.87x |
| usx -> usfm | 60.6ms | 18.7ms | 3.23x |

### `bdf_reg`

- USFM files: 27
- Total files in corpus directory: 33
- Total USFM source size: 1.13 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| parse usfm | 11.6ms | 3.5ms | 3.36x |
| project tokens | 21.5ms | 6.4ms | 3.37x |
| lint usfm | 62.8ms | 15.4ms | 4.08x |
| format usfm | 49.2ms | 16.9ms | 2.92x |
| usfm -> usj | 49.3ms | 12.9ms | 3.82x |
| usfm -> usx | 51.0ms | 13.4ms | 3.81x |
| usfm -> html | 59.5ms | 22.4ms | 2.66x |
| usfm -> vref | 14.0ms | 3.3ms | 4.21x |
| usj -> usfm | 5.7ms | 2.2ms | 2.58x |
| usx -> usfm | 9.2ms | 2.8ms | 3.29x |

### `en_ult`

- USFM files: 67
- Total files in corpus directory: 78
- Total USFM source size: 98.43 MiB

| Operation | Serial | Parallel | Speedup |
| --- | ---: | ---: | ---: |
| parse usfm | 1.08s | 432.9ms | 2.49x |
| project tokens | 2.77s | 903.7ms | 3.06x |
| lint usfm | 8.24s | 1.52s | 5.44x |
| format usfm | 7.94s | 2.72s | 2.92x |
| usfm -> usj | 7.84s | 1.54s | 5.10x |
| usfm -> usx | 4.97s | 976.1ms | 5.09x |
| usfm -> html | 10.34s | 2.22s | 4.65x |
| usfm -> vref | 1.25s | 276.0ms | 4.54x |
| usj -> usfm | 1.63s | 424.9ms | 3.83x |
| usx -> usfm | 2.02s | 441.2ms | 4.57x |
<!-- corpus-bench:end -->

Notes:

- These are corpus-wide wall-clock timings, not single-file microbenchmarks.
- Parallel timings are file-level parallelism, not intra-file parser parallelism.
- Reverse conversion timings (`usj -> usfm`, `usx -> usfm`) are measured from precomputed corpora generated from the same USFM sources.
- Treat the table as a practical throughput snapshot, not a cross-machine guarantee.

## WASM Timing Snapshot

The following table uses the web-target WASM package in Node after module initialization. It is useful as a comparison point against the native numbers above, especially for downstream JS consumers.

Run it locally with:

```bash
node benches/wasm_corpus_matrix.mjs --iterations 3 --markdown
```

<!-- wasm-corpus-bench:begin -->
_Local release measurements of the web-target WASM package in Node, median wall-clock over 3 runs, after module initialization._

### `examples.bsb`

- USFM files: 66
- Total files in corpus directory: 74
- Total USFM source size: 4.51 MiB

| Operation | WASM |
| --- | ---: |
| parse usfm | 173.2ms |
| project tokens | 868.3ms |
| lint usfm | 1.04s |
| format usfm | 1.22s |
| usfm -> usj | 610.0ms |
| usfm -> usx | 590.3ms |
| usfm -> html | 722.5ms |
| usfm -> vref | 423.2ms |
| usj -> usfm | 101.6ms |
| usx -> usfm | 107.3ms |

### `bdf_reg`

- USFM files: 27
- Total files in corpus directory: 33
- Total USFM source size: 1.13 MiB

| Operation | WASM |
| --- | ---: |
| parse usfm | 25.8ms |
| project tokens | 108.4ms |
| lint usfm | 112.0ms |
| format usfm | 159.1ms |
| usfm -> usj | 84.0ms |
| usfm -> usx | 80.2ms |
| usfm -> html | 96.7ms |
| usfm -> vref | 69.2ms |
| usj -> usfm | 14.1ms |
| usx -> usfm | 46.2ms |

### `en_ult`

- USFM files: 67
- Total files in corpus directory: 78
- Total USFM source size: 98.43 MiB

| Operation | WASM |
| --- | ---: |
| parse usfm | 4.26s |
| project tokens | 23.23s |
| lint usfm | 15.52s |
| format usfm | 26.14s |
| usfm -> usj | 16.72s |
| usfm -> usx | 9.33s |
| usfm -> html | 14.84s |
| usfm -> vref | 8.14s |
| usj -> usfm | 3.72s |
| usx -> usfm | 2.83s |

<!-- wasm-corpus-bench:end -->

Takeaways:

- WASM is still usable for smaller corpora, especially for direct format-to-format conversion.
- Native Rust remains much faster for corpus-scale token work and formatting.
- Native parallel execution is in a different throughput class entirely on large aligned corpora like `en_ult`.

## CLI

The install surface is one binary:

```bash
usfm-onion --help
```

### Parse

```bash
usfm-onion parse example-corpora/en_ult/01-GEN.usfm
usfm-onion parse --json example-corpora/en_ult/01-GEN.usfm
```

### Lint

```bash
usfm-onion lint example-corpora/en_ult/01-GEN.usfm
usfm-onion lint --json example-corpora/en_ult/01-GEN.usfm
usfm-onion lint --from usj doc.json
```

### Format

Default behavior enables every built-in formatter rule:

```bash
usfm-onion format example-corpora/en_ult/01-GEN.usfm
```

Format in place:

```bash
usfm-onion format --in-place example-corpora/en_ult/01-GEN.usfm
```

Check mode:

```bash
usfm-onion format --check example-corpora/en_ult/01-GEN.usfm
```

Only specific rules:

```bash
usfm-onion format \
  --include collapse-whitespace-in-text,normalize-spacing-after-paragraph-markers \
  example-corpora/en_ult/01-GEN.usfm
```

Default rules except a few:

```bash
usfm-onion format \
  --exclude recover-malformed-markers,remove-duplicate-verse-numbers \
  example-corpora/en_ult/01-GEN.usfm
```

### Convert

```bash
usfm-onion convert --to usj example-corpora/en_ult/01-GEN.usfm
usfm-onion convert --to usx example-corpora/en_ult/01-GEN.usfm
usfm-onion convert --to html example-corpora/en_ult/01-GEN.usfm
usfm-onion convert --to editor-tree example-corpora/en_ult/01-GEN.usfm
usfm-onion convert --to vref example-corpora/en_ult/01-GEN.usfm
```

### Diff

```bash
usfm-onion diff old.usfm new.usfm
usfm-onion diff --by-chapter old.usfm new.usfm
usfm-onion diff --json old.usfm new.usfm
```

### Inspect

```bash
usfm-onion inspect --recoveries example-corpora/en_ult/01-GEN.usfm
usfm-onion inspect --projected --lint example-corpora/en_ult/01-GEN.usfm
```

## Public Shared Types

Frequently-used root re-exports:

- `DocumentFormat`
- `FlatToken`
- `TokenKind`
- `ParseHandle`
- `FormatOptions`
- `FormatRule`
- `LintOptions`
- `LintIssue`

And under `model`:

- `UsjDocument`
- `UsjNode`
- `UsjVisit`
- `walk_usj_document_depth_first`
- `walk_usj_node_depth_first`
- `EditorTreeDocument`
- `VrefMap`

## Repo Layout

- `src/lib.rs`: thin public facade
- `src/parse/`: public parse/projection API
- `src/lint/`: public lint API
- `src/format/`: public formatter API
- `src/diff/`: public diff API
- `src/convert/`: public conversion API
- `src/model/`: public shared types
- `src/internal/`: implementation details
- `src/bin/usfm-onion.rs`: CLI
- `examples/dev/`: old one-off development utilities kept out of the install surface

## Status

The README is intended to describe the current native Rust API surface. If README and code disagree, the code should be treated as authoritative and README should be updated.
