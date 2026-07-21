//! Per-operation benchmark matrix: String input vs. Token input.
//!
//! Each operation is benchmarked on a single representative book
//! (en_ulb / 43-LUK.usfm). For operations that accept either a USFM source
//! string or pre-parsed tokens, both forms are measured side-by-side so
//! the cost of re-parsing on the string path is visible.
//!
//! The op bodies themselves live in `common::run_named_op` — shared with
//! `examples/profile_ops.rs` so a samply flamegraph and this criterion
//! number are always measuring the same thing.
//!
//! Run: `cargo bench --bench operations`

mod common;

use common::{load_luke, load_psalms, run_named_op};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

/// Ingest ops on Psalms — the marker-dense book the `parse/serial` profile is
/// taken from. The main `operations` group runs Luke (smaller, less
/// marker-dense), so this lane exists so a marker-resolution before/after
/// benchmarks the workload the optimization actually targets.
const PSALMS_INGEST_OPS: &[&str] = &["lex/string", "parse/serial", "parse/string"];

fn benchmark_operations(c: &mut Criterion) {
    let book = load_luke();
    let fx = common::build_fixture(&book);

    let mut group = c.benchmark_group("operations");
    group.throughput(Throughput::Bytes(book.bytes as u64));

    for name in common::OP_NAMES {
        group.bench_function(*name, |b| b.iter(|| run_named_op(name, &fx)));
    }

    group.finish();

    let psalms = load_psalms();
    let psalms_fx = common::build_fixture(&psalms);

    let mut psalms_group = c.benchmark_group("operations-psalms");
    psalms_group.throughput(Throughput::Bytes(psalms.bytes as u64));

    for name in PSALMS_INGEST_OPS {
        psalms_group.bench_function(*name, |b| b.iter(|| run_named_op(name, &psalms_fx)));
    }

    psalms_group.finish();
}

criterion_group!(benches, benchmark_operations);
criterion_main!(benches);
