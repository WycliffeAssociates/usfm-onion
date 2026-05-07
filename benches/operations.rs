//! Per-operation benchmark matrix: String input vs. Token input.
//!
//! Each operation is benchmarked on a single representative book
//! (en_ulb / 43-LUK.usfm). For operations that accept either a USFM source
//! string or pre-parsed tokens, both forms are measured side-by-side so
//! the cost of re-parsing on the string path is visible.
//!
//! Run: `cargo bench --bench operations`

mod common;

use common::{Book, load_luke};
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::cst::build_cst;
use usfm_onion::diff::{BuildSidBlocksOptions, diff_chapter_token_streams, diff_usfm_sources};
use usfm_onion::format::{FormatOptions, format_tokens, format_usfm, into_format_tokens};
use usfm_onion::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use usfm_onion::lexer::lex;
use usfm_onion::lint::{LintOptions, lint_tokens, lint_usfm};
use usfm_onion::parse::parse;
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;

fn benchmark_operations(c: &mut Criterion) {
    let book = load_luke();
    let parsed = parse(&book.source);
    let format_token_seed = into_format_tokens(&parsed.tokens);
    let lint_options = LintOptions::default();
    let format_options = FormatOptions::default();
    let html_options = HtmlOptions::default();
    let diff_options = BuildSidBlocksOptions::default();

    let mut group = c.benchmark_group("operations");
    group.throughput(Throughput::Bytes(book.bytes as u64));

    // Single-form operations (string-only or token-only).
    bench_string(&mut group, "lex/string", &book, |source| {
        black_box(lex(source));
    });
    bench_string(&mut group, "parse/string", &book, |source| {
        black_box(parse(source));
    });
    bench_string(&mut group, "usj/string", &book, |source| {
        black_box(usfm_to_usj(source).expect("USJ export"));
    });
    bench_string(&mut group, "usx/string", &book, |source| {
        black_box(usfm_to_usx(source).expect("USX export"));
    });

    group.bench_function("cst/tokens", |b| {
        b.iter(|| black_box(build_cst(parsed.tokens.clone())));
    });

    // Pairs: same operation, both input forms.
    bench_string(&mut group, "lint/string", &book, |source| {
        black_box(lint_usfm(source, lint_options.clone()));
    });
    {
        let opts = lint_options.clone();
        group.bench_function("lint/tokens", |b| {
            b.iter(|| black_box(lint_tokens(&parsed.tokens, opts.clone())));
        });
    }

    bench_string(&mut group, "format/string", &book, |source| {
        black_box(format_usfm(source, format_options));
    });
    group.bench_function("format/tokens", |b| {
        b.iter(|| {
            let mut working = format_token_seed.clone();
            format_tokens(&mut working, format_options);
            black_box(working);
        });
    });

    bench_string(&mut group, "html/string", &book, |source| {
        black_box(usfm_to_html(source, html_options));
    });
    group.bench_function("html/tokens", |b| {
        b.iter(|| black_box(tokens_to_html(&parsed.tokens, html_options)));
    });

    bench_string(&mut group, "diff/string", &book, |source| {
        black_box(diff_usfm_sources(source, source, &diff_options));
    });
    group.bench_function("diff/tokens", |b| {
        b.iter(|| {
            black_box(diff_chapter_token_streams(
                &parsed.tokens,
                &parsed.tokens,
                &diff_options,
            ))
        });
    });

    group.finish();
}

fn bench_string<M: criterion::measurement::Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, M>,
    name: &str,
    book: &Book,
    mut body: impl FnMut(&str),
) {
    group.bench_function(name, |b| {
        b.iter(|| body(&book.source));
    });
}

criterion_group!(benches, benchmark_operations);
criterion_main!(benches);
