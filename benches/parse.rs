mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::lexer::lex;
use usfm_onion::parse::parse_lexemes;

fn benchmark_parse(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("parse/corpus");
    for case in &corpus_cases {
        let lexed = lex(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        corpus_group.bench_with_input(
            BenchmarkId::new("parse", case_label(case)),
            case,
            |b, case| {
                b.iter(|| black_box(parse_lexemes(case.source.as_str(), &lexed.tokens)));
            },
        );
    }
    corpus_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("parse/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));
            let lexed_docs = batch
                .docs
                .iter()
                .map(|doc| (doc.source.clone(), lex(doc.source.as_str())))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("parse", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for (source, lexed) in &lexed_docs {
                            black_box(parse_lexemes(source.as_str(), &lexed.tokens));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
