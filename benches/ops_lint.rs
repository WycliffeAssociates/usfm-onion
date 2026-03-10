use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use usfm_onion::{
    DocumentFormat,
    format::{FormatOptions, IntoTokensOptions, format_content, format_contents, format_tokens},
    lint::{LintOptions, TokenLintOptions, lint_content, lint_flat_tokens},
    parse::{parse, tokens},
    tokens::{BatchExecutionOptions, usfm_to_tokens},
};

#[derive(Clone)]
struct LintCase {
    label: &'static str,
    usfm: String,
    tokens: Vec<usfm_onion::Token>,
}

#[derive(Clone)]
struct CorpusCase {
    label: &'static str,
    usfm_sources: Vec<String>,
    total_usfm_bytes: usize,
}

fn benchmark_ops_lint(c: &mut Criterion) {
    let short = load_case("2jn_short", "example-corpora/en_ulb/64-2JN.usfm");
    let medium = load_case("luk_medium", "example-corpora/en_ulb/43-LUK.usfm");
    let large = load_case("psa_large", "example-corpora/en_ulb/19-PSA.usfm");
    let cases = [short, medium.clone(), large];
    let en_ulb = load_corpus("en_ulb", "example-corpora/en_ulb");

    bench_lint_content(c, &cases);
    bench_lint_flat_tokens(c, &cases);
    bench_format_content(c, &medium);
    bench_format_corpus(c, &en_ulb);
    bench_format_breakdown(c, &medium);
}

fn bench_lint_content(c: &mut Criterion, cases: &[LintCase]) {
    let mut group = c.benchmark_group("ops_lint/lint_content");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
            b.iter(|| {
                black_box(
                    lint_content(
                        black_box(case.usfm.as_str()),
                        DocumentFormat::Usfm,
                        LintOptions::default(),
                    )
                    .expect("lint_content should succeed"),
                )
            });
        });
    }
    group.finish();
}

fn bench_lint_flat_tokens(c: &mut Criterion, cases: &[LintCase]) {
    let mut group = c.benchmark_group("ops_lint/lint_flat_tokens");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
            b.iter(|| {
                black_box(lint_flat_tokens(
                    black_box(case.tokens.as_slice()),
                    TokenLintOptions::default(),
                ))
            });
        });
    }
    group.finish();
}

fn bench_format_content(c: &mut Criterion, case: &LintCase) {
    let mut group = c.benchmark_group("ops_lint/format_content");
    group.throughput(Throughput::Bytes(case.usfm.len() as u64));
    group.bench_with_input(BenchmarkId::new("usfm", case.label), case, |b, case| {
        b.iter(|| {
            black_box(
                format_content(
                    black_box(case.usfm.as_str()),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                )
                .expect("format_content should succeed"),
            )
        });
    });
    group.finish();
}

fn bench_format_corpus(c: &mut Criterion, corpus: &CorpusCase) {
    let mut group = c.benchmark_group("ops_lint/format_contents");
    group.throughput(Throughput::Bytes(corpus.total_usfm_bytes as u64));

    group.bench_with_input(
        BenchmarkId::new("serial", corpus.label),
        corpus,
        |b, corpus| {
            b.iter(|| {
                black_box(format_contents(
                    black_box(corpus.usfm_sources.as_slice()),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                    BatchExecutionOptions::sequential(),
                ))
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", corpus.label),
        corpus,
        |b, corpus| {
            b.iter(|| {
                black_box(format_contents(
                    black_box(corpus.usfm_sources.as_slice()),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                    BatchExecutionOptions::parallel(),
                ))
            });
        },
    );

    group.finish();
}

fn bench_format_breakdown(c: &mut Criterion, case: &LintCase) {
    let handle = parse(case.usfm.as_str());
    let projected = tokens(&handle, usfm_onion::tokens::TokenViewOptions::default());
    let mut group = c.benchmark_group("ops_lint/format_breakdown");
    group.throughput(Throughput::Bytes(case.usfm.len() as u64));

    group.bench_function(BenchmarkId::new("parse", case.label), |b| {
        b.iter(|| black_box(parse(black_box(case.usfm.as_str()))));
    });

    group.bench_function(BenchmarkId::new("project_tokens", case.label), |b| {
        b.iter(|| {
            black_box(tokens(
                black_box(&handle),
                usfm_onion::tokens::TokenViewOptions::default(),
            ))
        });
    });

    group.bench_function(BenchmarkId::new("format_tokens", case.label), |b| {
        b.iter(|| {
            black_box(format_tokens(
                black_box(projected.as_slice()),
                FormatOptions::default(),
            ))
        });
    });

    group.bench_function(BenchmarkId::new("format_flat_tokens", case.label), |b| {
        b.iter(|| {
            black_box(usfm_onion::format::format_flat_tokens(black_box(
                projected.as_slice(),
            )))
        });
    });

    group.bench_function(BenchmarkId::new("format_content", case.label), |b| {
        b.iter(|| {
            black_box(
                format_content(
                    black_box(case.usfm.as_str()),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                )
                .expect("format_content should succeed"),
            )
        });
    });

    group.finish();
}

fn load_case(label: &'static str, relative_path: &str) -> LintCase {
    let path = manifest_dir().join(relative_path);
    let usfm = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let tokens = usfm_to_tokens(&usfm);
    LintCase {
        label,
        usfm,
        tokens,
    }
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn load_corpus(label: &'static str, relative_path: &str) -> CorpusCase {
    let root = manifest_dir().join(relative_path);
    let mut files = collect_usfm_files(&root);
    files.sort();

    let usfm_sources = files
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect::<Vec<_>>();
    let total_usfm_bytes = usfm_sources.iter().map(String::len).sum();

    CorpusCase {
        label,
        usfm_sources,
        total_usfm_bytes,
    }
}

fn collect_usfm_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_usfm_files_recursive(root, &mut out);
    out
}

fn collect_usfm_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_files_recursive(&path, out);
        } else if matches!(DocumentFormat::from_path(&path), Some(DocumentFormat::Usfm)) {
            out.push(path);
        }
    }
}

criterion_group!(benches, benchmark_ops_lint);
criterion_main!(benches);
