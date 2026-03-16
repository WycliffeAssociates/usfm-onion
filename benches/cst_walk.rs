mod common;

use common::standard_corpus_cases;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::parse_cst;

fn benchmark_cst_walk(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("cst_walk/corpus");
    for case in &corpus_cases {
        let document = parse_cst(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("iter_walk", case.name), case, |b, _case| {
            b.iter(|| {
                let visited = document
                    .iter_walk()
                    .fold(0usize, |acc, item| acc + item.depth + item.ancestor_token_indexes.len());
                black_box(visited)
            });
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_cst_walk);
criterion_main!(benches);
