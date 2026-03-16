mod common;

use common::standard_corpus_cases;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{build_cst, build_cst_roots, parse};

fn benchmark_cst(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("cst/corpus");
    for case in &corpus_cases {
        let parsed = parse(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("clone_tokens", case.name), case, |b, _case| {
            b.iter(|| black_box(parsed.tokens.clone()));
        });
        corpus_group.bench_with_input(BenchmarkId::new("build_roots", case.name), case, |b, _case| {
            b.iter(|| black_box(build_cst_roots(&parsed.tokens)));
        });
        corpus_group.bench_with_input(BenchmarkId::new("build_document", case.name), case, |b, _case| {
            b.iter(|| black_box(build_cst(parsed.tokens.clone())));
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_cst);
criterion_main!(benches);
