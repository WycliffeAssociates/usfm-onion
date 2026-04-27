mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::cst::parse_cst;

fn benchmark_cst_walk(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("cst_walk/corpus");
    for case in &corpus_cases {
        let document = parse_cst(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        corpus_group.bench_with_input(
            BenchmarkId::new("iter_walk", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| {
                    let visited = document.iter_walk().fold(0usize, |acc, item| {
                        acc + item.depth + item.ancestor_token_indexes.len()
                    });
                    black_box(visited)
                });
            },
        );
    }
    corpus_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("cst_walk/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let docs = batch
                .docs
                .iter()
                .map(|doc| parse_cst(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("iter_walk", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for document in &docs {
                            let visited = document.iter_walk().fold(0usize, |acc, item| {
                                acc + item.depth + item.ancestor_token_indexes.len()
                            });
                            black_box(visited);
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_cst_walk);
criterion_main!(benches);
