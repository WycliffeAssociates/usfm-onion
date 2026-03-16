mod common;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::lexer::lex;
use common::standard_corpus_cases;

fn benchmark_lexer(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("lexer/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("lex", case.name), case, |b, case| {
            b.iter(|| black_box(lex(case.source.as_str())));
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_lexer);
criterion_main!(benches);
