mod common;

use common::standard_corpus_cases;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{lex, parse_lexemes};

fn benchmark_parse(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("parse/corpus");
    for case in &corpus_cases {
        let lexed = lex(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("parse", case.name), case, |b, case| {
            b.iter(|| black_box(parse_lexemes(case.source.as_str(), &lexed.tokens)));
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
