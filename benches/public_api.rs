use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use usfm_onion::{
    BuildSidBlocksOptions,
    convert::{
        convert_content, convert_path, into_usj, into_usj_from_tokens, into_usj_lossless,
        into_usj_lossless_from_tokens, into_usx, into_usx_from_tokens, into_usx_lossless,
        into_usx_lossless_from_tokens, into_vref, into_vref_from_tokens,
    },
    diff::{diff_content, diff_paths, diff_tokens},
    format::{format_content, format_contents, format_flat_tokens, format_path},
    lint::{
        LintOptions, TokenLintOptions, lint_content, lint_contents, lint_flat_tokens, lint_path,
    },
    model::{BatchExecutionOptions, DocumentFormat, FlatToken, TokenKind, TokenViewOptions},
    parse::{
        IntoTokensOptions, into_tokens, into_tokens_from_content, into_tokens_from_contents,
        into_tokens_from_path, into_usfm_from_tokens, parse, parse_content, parse_contents,
        parse_path,
    },
};

#[derive(Clone)]
struct BenchCase {
    label: &'static str,
    usfm: String,
    usj: String,
    usx: String,
    usfm_path: PathBuf,
    usj_path: PathBuf,
    usx_path: PathBuf,
    modified_usfm: String,
    modified_usfm_path: PathBuf,
    tokens: Vec<FlatToken>,
    modified_tokens: Vec<FlatToken>,
}

struct CorpusCase {
    sources: Vec<String>,
    total_bytes: usize,
}

fn benchmark_public_api(c: &mut Criterion) {
    let short = load_case("2jn_short", "en_ulb/64-2JN.usfm");
    let medium = load_case("luk_medium", "en_ulb/43-LUK.usfm");
    let large = load_case("psa_large", "en_ulb/19-PSA.usfm");
    let corpus = load_corpus("en_ulb");

    let cases = [short, medium, large];

    bench_intake_output_content(c, &cases);
    bench_intake_output_path(c, &cases);
    bench_lint_format_diff_tokens(c, &cases);
    bench_lint_format_diff_content(c, &cases);
    bench_corpus_parallel_file_level(c, &corpus);
}

fn bench_intake_output_content(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("quick_api/intake_output_content");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| {
                black_box(run_intake_output_content_surface(case));
            });
        });
    }
    group.finish();
}

fn bench_intake_output_path(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("quick_api/intake_output_path");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| {
                black_box(run_intake_output_path_surface(case));
            });
        });
    }
    group.finish();
}

fn bench_lint_format_diff_tokens(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("quick_api/lint_format_diff_tokens");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| {
                black_box(run_token_ops(case));
            });
        });
    }
    group.finish();
}

fn bench_lint_format_diff_content(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("quick_api/lint_format_diff_content");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| {
                black_box(run_content_ops(case));
            });
        });
    }
    group.finish();
}

fn bench_corpus_parallel_file_level(c: &mut Criterion, corpus: &CorpusCase) {
    let mut group = c.benchmark_group("quick_api/corpus_parallel_file_level");
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));

    group.bench_function(BenchmarkId::new("parse_contents", "serial"), |b| {
        b.iter(|| {
            black_box(parse_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                BatchExecutionOptions { parallel: false },
            ));
        });
    });

    group.bench_function(BenchmarkId::new("parse_contents", "parallel"), |b| {
        b.iter(|| {
            black_box(parse_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                BatchExecutionOptions { parallel: true },
            ));
        });
    });

    group.bench_function(
        BenchmarkId::new("into_tokens_from_contents", "serial"),
        |b| {
            b.iter(|| {
                black_box(into_tokens_from_contents(
                    corpus.sources.as_slice(),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                    BatchExecutionOptions { parallel: false },
                ));
            });
        },
    );

    group.bench_function(
        BenchmarkId::new("into_tokens_from_contents", "parallel"),
        |b| {
            b.iter(|| {
                black_box(into_tokens_from_contents(
                    corpus.sources.as_slice(),
                    DocumentFormat::Usfm,
                    IntoTokensOptions::default(),
                    BatchExecutionOptions { parallel: true },
                ));
            });
        },
    );

    group.bench_function(BenchmarkId::new("lint_contents", "serial"), |b| {
        b.iter(|| {
            black_box(lint_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                LintOptions::default(),
                BatchExecutionOptions { parallel: false },
            ));
        });
    });

    group.bench_function(BenchmarkId::new("lint_contents", "parallel"), |b| {
        b.iter(|| {
            black_box(lint_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                LintOptions::default(),
                BatchExecutionOptions { parallel: true },
            ));
        });
    });

    group.bench_function(BenchmarkId::new("format_contents", "serial"), |b| {
        b.iter(|| {
            black_box(format_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                IntoTokensOptions::default(),
                BatchExecutionOptions { parallel: false },
            ));
        });
    });

    group.bench_function(BenchmarkId::new("format_contents", "parallel"), |b| {
        b.iter(|| {
            black_box(format_contents(
                corpus.sources.as_slice(),
                DocumentFormat::Usfm,
                IntoTokensOptions::default(),
                BatchExecutionOptions { parallel: true },
            ));
        });
    });

    group.finish();
}

fn run_intake_output_content_surface(case: &BenchCase) -> usize {
    let mut score = 0usize;

    let usfm_tokens = into_tokens_from_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
    )
    .expect("USFM content should tokenize");
    let usx_tokens = into_tokens_from_content(
        case.usx.as_str(),
        DocumentFormat::Usx,
        IntoTokensOptions::default(),
    )
    .expect("USX content should tokenize");
    let usj_tokens = into_tokens_from_content(
        case.usj.as_str(),
        DocumentFormat::Usj,
        IntoTokensOptions::default(),
    )
    .expect("USJ content should tokenize");

    score += usfm_tokens.len() + usx_tokens.len() + usj_tokens.len();

    let usfm_from_usx =
        convert_content(case.usx.as_str(), DocumentFormat::Usx, DocumentFormat::Usfm)
            .expect("USX should convert to USFM");
    let usfm_from_usj =
        convert_content(case.usj.as_str(), DocumentFormat::Usj, DocumentFormat::Usfm)
            .expect("USJ should convert to USFM");
    let usx_from_usfm = convert_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        DocumentFormat::Usx,
    )
    .expect("USFM should convert to USX");
    let usj_from_usfm = convert_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        DocumentFormat::Usj,
    )
    .expect("USFM should convert to USJ");

    score += usfm_from_usx.len() + usfm_from_usj.len() + usx_from_usfm.len() + usj_from_usfm.len();

    let handle =
        parse_content(case.usfm.as_str(), DocumentFormat::Usfm).expect("USFM content should parse");
    let usj = into_usj(&handle);
    let usx = into_usx(&handle).expect("USX should serialize");
    let vref = into_vref(&handle);
    let usj_lossless = into_usj_lossless(&handle);
    let usx_lossless = into_usx_lossless(&handle).expect("lossless USX should serialize");

    score += usj.content.len();
    score += usx.len();
    score += vref.len();
    score += usj_lossless.content.len();
    score += usx_lossless.len();

    let usfm_from_tokens = into_usfm_from_tokens(case.tokens.as_slice());
    let usj_from_tokens = into_usj_from_tokens(case.tokens.as_slice());
    let usx_from_tokens =
        into_usx_from_tokens(case.tokens.as_slice()).expect("USX should serialize");
    let vref_from_tokens = into_vref_from_tokens(case.tokens.as_slice());
    let usj_lossless_from_tokens = into_usj_lossless_from_tokens(case.tokens.as_slice());
    let usx_lossless_from_tokens =
        into_usx_lossless_from_tokens(case.tokens.as_slice()).expect("USX should serialize");

    score += usfm_from_tokens.len();
    score += usj_from_tokens.content.len();
    score += usx_from_tokens.len();
    score += vref_from_tokens.len();
    score += usj_lossless_from_tokens.content.len();
    score += usx_lossless_from_tokens.len();

    score
}

fn run_intake_output_path_surface(case: &BenchCase) -> usize {
    let mut score = 0usize;

    let usfm_handle =
        parse_path(case.usfm_path.as_path(), DocumentFormat::Usfm).expect("USFM path should parse");
    let usx_handle =
        parse_path(case.usx_path.as_path(), DocumentFormat::Usx).expect("USX path should parse");
    let usj_handle =
        parse_path(case.usj_path.as_path(), DocumentFormat::Usj).expect("USJ path should parse");
    score += into_vref(&usfm_handle).len();
    score += into_vref(&usx_handle).len();
    score += into_vref(&usj_handle).len();

    let usfm_tokens = into_tokens_from_path(
        case.usfm_path.as_path(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
    )
    .expect("USFM path should tokenize");
    let usx_tokens = into_tokens_from_path(
        case.usx_path.as_path(),
        DocumentFormat::Usx,
        IntoTokensOptions::default(),
    )
    .expect("USX path should tokenize");
    let usj_tokens = into_tokens_from_path(
        case.usj_path.as_path(),
        DocumentFormat::Usj,
        IntoTokensOptions::default(),
    )
    .expect("USJ path should tokenize");
    score += usfm_tokens.len() + usx_tokens.len() + usj_tokens.len();

    let usfm_from_usx = convert_path(
        case.usx_path.as_path(),
        DocumentFormat::Usx,
        DocumentFormat::Usfm,
    )
    .expect("USX path should convert to USFM");
    let usfm_from_usj = convert_path(
        case.usj_path.as_path(),
        DocumentFormat::Usj,
        DocumentFormat::Usfm,
    )
    .expect("USJ path should convert to USFM");
    score += usfm_from_usx.len() + usfm_from_usj.len();

    let lint = lint_path(
        case.usfm_path.as_path(),
        DocumentFormat::Usfm,
        LintOptions::default(),
    )
    .expect("USFM path lint should succeed");
    score += lint.len();

    let format = format_path(
        case.usfm_path.as_path(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
    )
    .expect("USFM path format should succeed");
    score += format.tokens.len();

    let diffs = diff_paths(
        case.usfm_path.as_path(),
        DocumentFormat::Usfm,
        case.modified_usfm_path.as_path(),
        DocumentFormat::Usfm,
        &TokenViewOptions::default(),
        &BuildSidBlocksOptions::default(),
    )
    .expect("USFM path diff should succeed");
    score += diffs.len();

    score
}

fn run_token_ops(case: &BenchCase) -> usize {
    let lint = lint_flat_tokens(case.tokens.as_slice(), TokenLintOptions::default());
    let format = format_flat_tokens(case.tokens.as_slice());
    let diffs = diff_tokens(
        case.tokens.as_slice(),
        case.modified_tokens.as_slice(),
        &BuildSidBlocksOptions::default(),
    );

    lint.len() + format.tokens.len() + diffs.len()
}

fn run_content_ops(case: &BenchCase) -> usize {
    let lint_usfm = lint_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        LintOptions::default(),
    )
    .expect("USFM lint should succeed");
    let lint_usx = lint_content(
        case.usx.as_str(),
        DocumentFormat::Usx,
        LintOptions::default(),
    )
    .expect("USX lint should succeed");
    let lint_usj = lint_content(
        case.usj.as_str(),
        DocumentFormat::Usj,
        LintOptions::default(),
    )
    .expect("USJ lint should succeed");

    let format = format_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
    )
    .expect("USFM format should succeed");

    let diffs = diff_content(
        case.usfm.as_str(),
        DocumentFormat::Usfm,
        case.modified_usfm.as_str(),
        DocumentFormat::Usfm,
        &TokenViewOptions::default(),
        &BuildSidBlocksOptions::default(),
    )
    .expect("USFM diff should succeed");

    lint_usfm.len() + lint_usx.len() + lint_usj.len() + format.tokens.len() + diffs.len()
}

fn load_case(label: &'static str, relative_path: &str) -> BenchCase {
    let usfm_path = manifest_dir().join(relative_path);
    let usfm = fs::read_to_string(&usfm_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));

    let handle = parse(&usfm);
    let tokens = into_tokens(&handle, IntoTokensOptions::default());
    let usj = serde_json::to_string(&into_usj(&handle)).expect("USJ should serialize");
    let usx = into_usx(&handle).expect("USX should serialize");

    let modified_usfm = mutate_usfm_source(&usfm, tokens.as_slice());
    let modified_handle = parse(&modified_usfm);
    let modified_tokens = into_tokens(&modified_handle, IntoTokensOptions::default());

    let usj_path = write_temp_file(label, "json", usj.as_str());
    let usx_path = write_temp_file(label, "xml", usx.as_str());
    let modified_usfm_path = write_temp_file(&format!("{label}_modified"), "usfm", &modified_usfm);

    BenchCase {
        label,
        usfm,
        usj,
        usx,
        usfm_path,
        usj_path,
        usx_path,
        modified_usfm,
        modified_usfm_path,
        tokens,
        modified_tokens,
    }
}

fn load_corpus(relative_dir: &str) -> CorpusCase {
    let root = manifest_dir().join(relative_dir);
    let mut paths = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
        .collect::<Vec<_>>();
    paths.sort();

    let sources = paths
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect::<Vec<_>>();
    let total_bytes = sources.iter().map(String::len).sum();

    CorpusCase {
        sources,
        total_bytes,
    }
}

fn write_temp_file(stem: &str, ext: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("usfm_onion_public_api_bench");
    fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", dir.display()));
    let sanitized = stem.replace(['/', ' '], "_");
    let path = dir.join(format!("{sanitized}.{ext}"));
    fs::write(&path, content)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
    path
}

fn mutate_usfm_source(usfm: &str, tokens: &[FlatToken]) -> String {
    let mut candidates = tokens
        .iter()
        .filter(|token| matches!(token.kind, TokenKind::Text | TokenKind::BookCode))
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return format!("{usfm}\n");
    }

    let start = candidates.len() / 2;
    candidates.rotate_left(start);

    for token in candidates {
        if token.span.end <= usfm.len()
            && token.span.start < token.span.end
            && let Some(replacement) = mutate_text_preserving_length(token.text.as_str())
        {
            let mut out = usfm.to_string();
            out.replace_range(token.span.clone(), replacement.as_str());
            return out;
        }
    }

    format!("{usfm}\n")
}

fn mutate_text_preserving_length(text: &str) -> Option<String> {
    let mut chars = text.chars().collect::<Vec<_>>();
    let index = chars.iter().position(|ch| ch.is_ascii_alphabetic())?;
    chars[index] = match chars[index] {
        'a' => 'b',
        'A' => 'B',
        'z' => 'y',
        'Z' => 'Y',
        _ => 'z',
    };
    Some(chars.into_iter().collect())
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(400))
        .measurement_time(std::time::Duration::from_secs(1));
    targets = benchmark_public_api
);
criterion_main!(benches);
