use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use usfm_onion::convert::into_document_tree;
use usfm_onion::document_tree::usfm_to_document_tree;
use usfm_onion::parse::{parse, tokens};
use usfm_onion::tokens::TokenViewOptions;

#[derive(Clone)]
struct Case {
    label: &'static str,
    usfm: String,
}

fn benchmark_document_tree_breakdown(c: &mut Criterion) {
    let medium = load_case("md_luk", "example-corpora/en_ult/43-LUK.usfm");
    let xl = load_case("xl_psa", "example-corpora/en_ult/19-PSA.usfm");
    let cases = [medium, xl];

    bench_parse(c, &cases);
    bench_project_tokens(c, &cases);
    bench_document_tree_from_handle(c, &cases);
    bench_usfm_to_document_tree(c, &cases);
}

fn bench_parse(c: &mut Criterion, cases: &[Case]) {
    let mut group = c.benchmark_group("document_tree_breakdown/parse");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
            b.iter(|| black_box(parse(black_box(case.usfm.as_str()))));
        });
    }
    group.finish();
}

fn bench_project_tokens(c: &mut Criterion, cases: &[Case]) {
    let mut group = c.benchmark_group("document_tree_breakdown/tokens");
    for case in cases {
        let handle = parse(case.usfm.as_str());
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parsed_handle", case.label),
            case,
            |b, _| {
                b.iter(|| black_box(tokens(black_box(&handle), TokenViewOptions::default())));
            },
        );
    }
    group.finish();
}

fn bench_usfm_to_document_tree(c: &mut Criterion, cases: &[Case]) {
    let mut group = c.benchmark_group("document_tree_breakdown/full_tree");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
            b.iter(|| black_box(usfm_to_document_tree(black_box(case.usfm.as_str()))));
        });
    }
    group.finish();
}

fn bench_document_tree_from_handle(c: &mut Criterion, cases: &[Case]) {
    let mut group = c.benchmark_group("document_tree_breakdown/tree_from_handle");
    for case in cases {
        let handle = parse(case.usfm.as_str());
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parsed_handle", case.label),
            case,
            |b, _| {
                b.iter(|| black_box(into_document_tree(black_box(&handle))));
            },
        );
    }
    group.finish();
}

fn load_case(label: &'static str, relative_path: &str) -> Case {
    let path = manifest_dir().join(relative_path);
    let usfm = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    Case { label, usfm }
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

criterion_group!(benches, benchmark_document_tree_breakdown);
criterion_main!(benches);
