# Benchmarks

Two criterion harnesses, each answering one question.

## `operations` — String vs. Tokens, per operation

Runs every supported operation against a single representative book
(`example-corpora/en_ulb/43-LUK.usfm`). For operations that accept either
a USFM source string or pre-parsed tokens, both forms are measured side
by side so the cost of re-parsing on the string path is visible.

```bash
cargo bench --bench operations
```

Bench IDs you'll see (under the `operations` group):

| Operation | Input forms |
|-----------|-------------|
| `lex`     | `string` |
| `parse`   | `string` |
| `cst`     | `tokens` |
| `usj`     | `string` |
| `usx`     | `string` |
| `lint`    | `string`, `tokens` |
| `format`  | `string`, `tokens` |
| `html`    | `string`, `tokens` |
| `diff`    | `string`, `tokens` |

The matrix tells you which operations recoup their parse cost on the
token path — useful for deciding whether to keep both surface forms on
each operation as the library evolves.

## `parallelism` — serial vs. rayon, on en_ulb

Loads the full English ULB corpus (~66 books) and runs each major
operation twice: once iterating with `.iter()`, once with `.par_iter()`.
Demonstrates what a host gets by parallelizing at the file level — the
core library is single-threaded by design, so any parallel speedup
belongs to the caller.

```bash
cargo bench --bench parallelism
```

Operations covered: `parse`, `lint`, `format`, `usj`, `usx`, `html`.
Each shows up as `<op>/serial` and `<op>/rayon` in the
`parallelism/en_ulb` group.

## Snapshotting results to `BENCH_RESULTS.md`

After running both benches, regenerate the human-readable summary at
the repo root:

```bash
cargo bench --bench operations
cargo bench --bench parallelism
cargo run --release --example bench_report > BENCH_RESULTS.md
```

The example reads `target/criterion/**/new/{benchmark,estimates}.json`
and emits markdown tables (operations matrix, serial-vs-rayon speedups).
Commit `BENCH_RESULTS.md` whenever you want to put a pin in current
performance — it diffs cleanly across runs and is the easiest way to
say "here's where we stand".

## Notes

- `rayon` is a `[dev-dependencies]` entry. The library itself does not
  depend on it; it is only here so the parallelism bench can demonstrate
  the comparison.
- Both benches use `criterion` with custom harness (no `#[bench]`
  attribute). Detailed HTML reports land under `target/criterion/`.
- `[profile.bench]` overrides the inherited `[profile.release]` (which
  is tuned for wasm-pack size, not native speed). If you change this,
  update the label in `examples/bench_report.rs`.
- If you want to sample a different book or corpus, edit
  `benches/common.rs` (`load_luke` / `load_en_ulb`).
