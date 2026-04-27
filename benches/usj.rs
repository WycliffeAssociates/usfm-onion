mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::usj::usfm_to_usj;

fn benchmark_usj(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("usj/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        corpus_group.bench_with_input(
            BenchmarkId::new("export", case_label(case)),
            case,
            |b, case| {
                b.iter(|| {
                    black_box(usfm_to_usj(case.source.as_str()).expect("USJ export should succeed"))
                });
            },
        );
    }
    corpus_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("usj/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));
            whole_corpus_group.bench_with_input(
                BenchmarkId::new("export", batch_label(batch)),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(
                                usfm_to_usj(doc.source.as_str())
                                    .expect("USJ export should succeed"),
                            );
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_usj);
criterion_main!(benches);
