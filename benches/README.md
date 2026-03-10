# Benchmarks

This folder has two benchmark layers:

- `public_api.rs`: smaller API-surface and representative single-document benchmarks using Criterion
- `ops_lint.rs`: focused Criterion benchmarks for the token-first lint ops path
- `conversions.rs`: focused Criterion benchmarks for individual conversion/projection paths
- `corpus_matrix.rs`: whole-corpus throughput measurements intended for practical serial-vs-parallel comparisons
- `wasm_corpus_matrix.mjs`: whole-corpus throughput measurements for the `pkg-web` WASM build in Node

## Whole-Corpus Matrix

Run the corpus sweep with:

```bash
cargo bench --bench corpus_matrix --features rayon -- --iterations 3 --markdown
```

If you want to build the benchmark target once and run the produced executable directly:

```bash
cargo build --release --features rayon --lib --bench corpus_matrix
target/release/deps/corpus_matrix-<hash> --iterations 3 --markdown
```

The markdown output is designed to paste directly into the `Corpus Timing Snapshot` section of the root README.

Corpora used by the benchmark:

- `example-corpora/examples.bsb`
- `example-corpora/bdf_reg`
- `example-corpora/en_ult`

## WASM Matrix

Build the web-target package and run the WASM sweep with:

```bash
wasm-pack build --manifest-path crates/usfm_onion_wasm/Cargo.toml --target web --release --out-dir ../../pkg-web --out-name usfm_onion_web
node benches/wasm_corpus_matrix.mjs --iterations 3 --markdown
```

The markdown output is designed to paste directly into the `WASM Timing Snapshot` section of the root README.
