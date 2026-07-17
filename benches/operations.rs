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

use common::{load_luke, run_named_op};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn benchmark_operations(c: &mut Criterion) {
    let book = load_luke();
    let fx = common::build_fixture(&book);

    let mut group = c.benchmark_group("operations");
    group.throughput(Throughput::Bytes(book.bytes as u64));

    for name in common::OP_NAMES {
        group.bench_function(*name, |b| b.iter(|| run_named_op(name, &fx)));
    }

    group.finish();
}

criterion_group!(benches, benchmark_operations);
criterion_main!(benches);
