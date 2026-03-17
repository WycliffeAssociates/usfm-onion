mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::lexer::lex;

fn benchmark_lexer(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("lexer/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        corpus_group.bench_with_input(BenchmarkId::new("lex", case_label(case)), case, |b, case| {
            b.iter(|| black_box(lex(case.source.as_str())));
        });
    }
    corpus_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("lexer/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));
            whole_corpus_group.bench_with_input(
                BenchmarkId::new("lex", batch_label(batch)),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(lex(doc.source.as_str()));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_lexer);
criterion_main!(benches);
