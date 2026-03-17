# Benchmarks

This folder now only contains active Criterion benches for the current engine surface.

Focused benches:

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

Convenience bench:

- `omni`

`omni` is the broad sweep. It does not replace the focused benches; it is just the fastest way to get one pass over the main operations.

## Corpus Selection

Whole-corpus runs use the `USFM_BENCH_CORPORA` environment variable.

Examples:

```bash
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench lint
USFM_BENCH_CORPORA="en_ulb en_ult" cargo bench --bench format
USFM_BENCH_CORPORA=all cargo bench --bench omni
```

## Typical Commands

Run one focused bench:

```bash
cargo bench --bench diff
```

Run the broad sweep:

```bash
cargo bench --bench omni
```

Run a whole-corpus convenience sweep:

```bash
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench omni
```
