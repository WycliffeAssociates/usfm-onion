mod common;

use common::standard_corpus_cases;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::usx::usfm_to_usx;

fn benchmark_usx(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("usx/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("export", case.name), case, |b, case| {
            b.iter(|| black_box(usfm_to_usx(case.source.as_str()).expect("USX export should succeed")));
        });
    }
    corpus_group.finish();
}

criterion_group!(benches, benchmark_usx);
criterion_main!(benches);
