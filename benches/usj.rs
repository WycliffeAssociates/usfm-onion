mod common;

use common::standard_corpus_cases;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::usfm_to_usj;

fn benchmark_usj(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("usj/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("export", case.name), case, |b, case| {
            b.iter(|| black_box(usfm_to_usj(case.source.as_str()).expect("USJ export should succeed")));
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_usj);
criterion_main!(benches);
