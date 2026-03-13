use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use usfm_onion::ast::usfm_to_ast;

#[derive(Clone)]
struct ConversionCase {
    label: &'static str,
    usfm: String,
}

fn benchmark_conversions(c: &mut Criterion) {
    let bsb_small = load_case(
        "bsb_small_phm",
        "example-corpora/examples.bsb/58PHMBSB.usfm",
    );
    let small = load_case("sm_2jn", "example-corpora/en_ult/64-2JN.usfm");
    let medium = load_case("md_luk", "example-corpora/en_ult/43-LUK.usfm");
    let large = load_case("lg_gen", "example-corpora/en_ult/01-GEN.usfm");
    let xl = load_case("xl_psa", "example-corpora/en_ult/19-PSA.usfm");
    let cases = [bsb_small, small, medium, large, xl];

    bench_usfm_to_ast(c, &cases);
}

fn bench_usfm_to_ast(c: &mut Criterion, cases: &[ConversionCase]) {
    let mut group = c.benchmark_group("conversions/usfm_to_ast");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
            b.iter(|| black_box(usfm_to_ast(black_box(case.usfm.as_str()))));
        });
    }
    group.finish();
}

fn load_case(label: &'static str, relative_path: &str) -> ConversionCase {
    let path = manifest_dir().join(relative_path);
    let usfm = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    ConversionCase { label, usfm }
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

criterion_group!(benches, benchmark_conversions);
criterion_main!(benches);
