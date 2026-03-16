use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::fs;
use std::path::Path;

use usfm_onion::lexer::lex;

struct BenchCase {
    name: &'static str,
    source: String,
}

fn benchmark_lexer(c: &mut Criterion) {
    let corpus_cases = [
        load_corpus_case("short", "example-corpora/en_ulb/64-2JN.usfm"),
        load_corpus_case("medium", "example-corpora/en_ulb/43-LUK.usfm"),
        load_corpus_case("large", "example-corpora/en_ulb/19-PSA.usfm"),
        load_corpus_case("xl", "example-corpora/en_ult/19-PSA.usfm"),
    ];

    let mut corpus_group = c.benchmark_group("lexer/corpus");
    for case in &corpus_cases {
        corpus_group.throughput(Throughput::Bytes(case.source.len() as u64));
        corpus_group.bench_with_input(BenchmarkId::new("lex", case.name), case, |b, case| {
            b.iter(|| black_box(lex(case.source.as_str())));
        });
    }
    corpus_group.finish();
}

fn load_corpus_case(name: &'static str, relative_path: &str) -> BenchCase {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    BenchCase {
        name,
        source: fs::read_to_string(path).expect("benchmark corpus should read"),
    }
}

criterion_group!(benches, benchmark_lexer);
criterion_main!(benches);
