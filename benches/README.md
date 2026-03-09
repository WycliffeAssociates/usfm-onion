# Benchmarks

This folder has two benchmark layers:

- `public_api.rs`: smaller API-surface and representative single-document benchmarks using Criterion
- `corpus_matrix.rs`: whole-corpus throughput measurements intended for practical serial-vs-parallel comparisons

## Whole-Corpus Matrix

Run the corpus sweep with:

```bash
cargo bench --bench corpus_matrix --features rayon -- --iterations 3 --markdown
```

For a quicker pass:

```bash
target/release/deps/corpus_matrix-<hash> --iterations 1 --warmup 0 --markdown
```

The markdown output is designed to paste directly into the `Corpus Timing Snapshot` section of the root README.

Corpora used by the benchmark:

- `example-corpora/examples.bsb`
- `example-corpora/bdf_reg`
- `example-corpora/en_ult`
