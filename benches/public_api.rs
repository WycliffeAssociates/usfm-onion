use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use usfm_onion::{
    convert::{
        HtmlOptions, convert_path, from_usj_str, from_usx_str, tokens_to_html, tokens_to_usj,
        tokens_to_usx, tokens_to_vref, usfm_to_html, usfm_to_usj, usfm_to_usx, usfm_to_vref,
    },
    diff::{BuildSidBlocksOptions, diff_content, diff_paths, diff_tokens},
    document_tree::{
        document_tree_to_tokens, read_usfm_to_document_tree, read_usj_to_document_tree,
        read_usx_to_document_tree, tokens_to_document_tree, usfm_to_document_tree,
        usj_to_document_tree, usx_to_document_tree,
    },
    format::{IntoTokensOptions, format_content, format_flat_tokens, format_path},
    lint::{LintOptions, TokenLintOptions, lint_content, lint_flat_tokens, lint_path},
    model::{DocumentFormat, Token, TokenKind},
    tokens::{
        TokenViewOptions, read_usfm_to_tokens, read_usj_to_tokens, read_usx_to_tokens,
        tokens_to_usfm, usfm_to_tokens, usj_to_tokens, usx_to_tokens,
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
    tokens: Vec<Token>,
    modified_tokens: Vec<Token>,
}

fn benchmark_public_api(c: &mut Criterion) {
    let short = load_case("2jn_short", "en_ulb/64-2JN.usfm");
    let medium = load_case("luk_medium", "en_ulb/43-LUK.usfm");
    let large = load_case("psa_large", "en_ulb/19-PSA.usfm");
    let cases = [short, medium, large];

    bench_intake_output_content(c, &cases);
    bench_intake_output_path(c, &cases);
    bench_lint_format_diff_tokens(c, &cases);
    bench_lint_format_diff_content(c, &cases);
}

fn bench_intake_output_content(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("public_api/intake_output_content");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| black_box(run_intake_output_content_surface(case)));
        });
    }
    group.finish();
}

fn bench_intake_output_path(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("public_api/intake_output_path");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| black_box(run_intake_output_path_surface(case)));
        });
    }
    group.finish();
}

fn bench_lint_format_diff_tokens(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("public_api/lint_format_diff_tokens");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| black_box(run_token_ops(case)));
        });
    }
    group.finish();
}

fn bench_lint_format_diff_content(c: &mut Criterion, cases: &[BenchCase]) {
    let mut group = c.benchmark_group("public_api/lint_format_diff_content");
    for case in cases {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("surface", case.label), case, |b, case| {
            b.iter(|| black_box(run_content_ops(case)));
        });
    }
    group.finish();
}

fn run_intake_output_content_surface(case: &BenchCase) -> usize {
    let mut score = 0usize;

    let usfm_tokens = usfm_to_tokens(case.usfm.as_str());
    let usx_tokens = usx_to_tokens(case.usx.as_str()).expect("USX content should tokenize");
    let usj_tokens = usj_to_tokens(case.usj.as_str()).expect("USJ content should tokenize");
    score += usfm_tokens.len() + usx_tokens.len() + usj_tokens.len();

    let usfm_tree = usfm_to_document_tree(case.usfm.as_str());
    let usx_tree = usx_to_document_tree(case.usx.as_str()).expect("USX content should project");
    let usj_tree = usj_to_document_tree(case.usj.as_str()).expect("USJ content should project");
    score += usfm_tree.content.len() + usx_tree.content.len() + usj_tree.content.len();

    let usfm_from_usx = from_usx_str(case.usx.as_str()).expect("USX should convert to USFM");
    let usfm_from_usj = from_usj_str(case.usj.as_str()).expect("USJ should convert to USFM");
    let usx_from_usfm = usfm_to_usx(case.usfm.as_str()).expect("USFM should convert to USX");
    let usj_from_usfm = usfm_to_usj(case.usfm.as_str()).expect("USFM should convert to USJ");
    let html_from_usfm =
        usfm_to_html(case.usfm.as_str(), HtmlOptions::default()).expect("USFM should render HTML");
    let vref_from_usfm = usfm_to_vref(case.usfm.as_str()).expect("USFM should project VREF");

    score += usfm_from_usx.len()
        + usfm_from_usj.len()
        + usx_from_usfm.len()
        + usj_from_usfm.content.len()
        + html_from_usfm.len()
        + vref_from_usfm.len();

    let usfm_from_tokens = tokens_to_usfm(case.tokens.as_slice());
    let tree_from_tokens = tokens_to_document_tree(case.tokens.as_slice());
    let tokens_from_tree =
        document_tree_to_tokens(&tree_from_tokens).expect("document tree should flatten");
    let usj_from_tokens = tokens_to_usj(case.tokens.as_slice()).expect("tokens should project USJ");
    let usx_from_tokens = tokens_to_usx(case.tokens.as_slice()).expect("tokens should project USX");
    let html_from_tokens = tokens_to_html(case.tokens.as_slice(), HtmlOptions::default())
        .expect("tokens should render HTML");
    let vref_from_tokens =
        tokens_to_vref(case.tokens.as_slice()).expect("tokens should project VREF");

    score += usfm_from_tokens.len()
        + tree_from_tokens.content.len()
        + tokens_from_tree.len()
        + usj_from_tokens.content.len()
        + usx_from_tokens.len()
        + html_from_tokens.len()
        + vref_from_tokens.len();

    score
}

fn run_intake_output_path_surface(case: &BenchCase) -> usize {
    let mut score = 0usize;

    let usfm_tokens =
        read_usfm_to_tokens(case.usfm_path.as_path()).expect("USFM path should tokenize");
    let usx_tokens = read_usx_to_tokens(case.usx_path.as_path()).expect("USX path should tokenize");
    let usj_tokens = read_usj_to_tokens(case.usj_path.as_path()).expect("USJ path should tokenize");
    score += usfm_tokens.len() + usx_tokens.len() + usj_tokens.len();

    let usfm_tree = read_usfm_to_document_tree(case.usfm_path.as_path())
        .expect("USFM path should project document tree");
    let usx_tree = read_usx_to_document_tree(case.usx_path.as_path())
        .expect("USX path should project document tree");
    let usj_tree = read_usj_to_document_tree(case.usj_path.as_path())
        .expect("USJ path should project document tree");
    score += usfm_tree.content.len() + usx_tree.content.len() + usj_tree.content.len();

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

    let tokens = usfm_to_tokens(&usfm);
    let usj = serde_json::to_string(&usfm_to_usj(&usfm).expect("USJ should serialize"))
        .expect("USJ JSON should serialize");
    let usx = usfm_to_usx(&usfm).expect("USX should serialize");

    let modified_usfm = mutate_usfm_source(&usfm, tokens.as_slice());
    let modified_tokens = usfm_to_tokens(&modified_usfm);

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

fn mutate_usfm_source(usfm: &str, tokens: &[Token]) -> String {
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
        if token.span.end <= usfm.chars().count()
            && token.span.start < token.span.end
            && let Some(replacement) = mutate_text_preserving_length(token.text.as_str())
        {
            let mut out = usfm.to_string();
            let start = char_offset_to_byte_index(usfm, token.span.start);
            let end = char_offset_to_byte_index(usfm, token.span.end);
            out.replace_range(start..end, replacement.as_str());
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

fn char_offset_to_byte_index(source: &str, char_offset: usize) -> usize {
    if char_offset == source.chars().count() {
        return source.len();
    }

    source
        .char_indices()
        .nth(char_offset)
        .map(|(index, _)| index)
        .unwrap_or(source.len())
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

criterion_group!(benches, benchmark_public_api);
criterion_main!(benches);
