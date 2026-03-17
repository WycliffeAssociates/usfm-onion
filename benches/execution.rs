mod common;

use common::{batch_label, selected_corpus_batches};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{ExecutionMode, FormatOptions, LintOptions, UsfmBatch};

fn benchmark_execution(c: &mut Criterion) {
    let selected_batches = selected_corpus_batches();
    if selected_batches.is_empty() {
        return;
    }

    let mut group = c.benchmark_group("execution/api-whole-corpora");
    for batch in &selected_batches {
        let sources = batch
            .docs
            .iter()
            .map(|doc| doc.source.as_str())
            .collect::<Vec<_>>();
        let usfm_batch = UsfmBatch::from_strs(sources.iter().copied());

        group.throughput(Throughput::Bytes(batch.total_bytes as u64));

        group.bench_with_input(
            BenchmarkId::new("parse_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .parse()
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .parse()
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("lint_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .lint(LintOptions::default())
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("lint_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .lint(LintOptions::default())
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .format(FormatOptions::default())
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .format(FormatOptions::default())
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usj_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .to_usj()
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usj_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .to_usj()
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usx_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .to_usx()
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usx_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .to_usx()
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("diff_serial", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .diff(&usfm_batch)
                            .with_execution(ExecutionMode::Serial)
                            .run(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("diff_parallel", batch_label(batch)),
            batch,
            |b, _batch| {
                b.iter(|| {
                    black_box(
                        usfm_batch
                            .diff(&usfm_batch)
                            .with_execution(ExecutionMode::Parallel)
                            .run(),
                    )
                });
            },
        );

    }
    group.finish();
}

criterion_group!(benches, benchmark_execution);
criterion_main!(benches);
