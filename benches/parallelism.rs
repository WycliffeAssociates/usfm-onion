//! Whole-corpus parallelism benchmark: serial vs. rayon over en_ulb.
//!
//! Demonstrates what a caller gets by parallelizing operations themselves
//! at the file level. The library is single-threaded by design — file IO
//! and parallel orchestration belong to the caller — and `rayon` lives in
//! `[dev-dependencies]` so this bench can show the comparison without any
//! production dependency on it.
//!
//! Each operation is run twice over the same corpus (~66 books):
//! once iterating with `.iter()`, once with `.par_iter()`. The numbers
//! tell a caller "if my workload looks like en_ulb, here's roughly what
//! a parallel host buys me."
//!
//! Run: `cargo bench --bench parallelism`

mod common;

use common::{Corpus, load_en_ulb};
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use rayon::prelude::*;
use usfm_onion::format::{FormatOptions, format_usfm};
use usfm_onion::html::{HtmlOptions, usfm_to_html};
use usfm_onion::lint::{LintOptions, lint_usfm};
use usfm_onion::parse::parse;
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;

fn benchmark_parallelism(c: &mut Criterion) {
    let corpus = load_en_ulb();
    let lint_options = LintOptions::default();
    let format_options = FormatOptions::default();
    let html_options = HtmlOptions::default();

    let mut group = c.benchmark_group("parallelism/en_ulb");
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));

    pair(&mut group, "parse", &corpus, |source| {
        black_box(parse(source));
    });
    pair(&mut group, "lint", &corpus, |source| {
        black_box(lint_usfm(source, lint_options.clone()));
    });
    pair(&mut group, "format", &corpus, |source| {
        black_box(format_usfm(source, format_options));
    });
    pair(&mut group, "usj", &corpus, |source| {
        if let Ok(doc) = usfm_to_usj(source) {
            black_box(doc);
        }
    });
    pair(&mut group, "usx", &corpus, |source| {
        if let Ok(xml) = usfm_to_usx(source) {
            black_box(xml);
        }
    });
    pair(&mut group, "html", &corpus, |source| {
        black_box(usfm_to_html(source, html_options));
    });

    group.finish();
}

/// Register a `serial` and `rayon` bench for the same per-book operation.
fn pair<M: criterion::measurement::Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, M>,
    op_name: &str,
    corpus: &Corpus,
    body: impl Fn(&str) + Sync + Send,
) {
    group.bench_function(format!("{op_name}/serial"), |b| {
        b.iter(|| {
            for source in &corpus.books {
                body(source);
            }
        });
    });
    group.bench_function(format!("{op_name}/rayon"), |b| {
        b.iter(|| {
            corpus.books.par_iter().for_each(|source| body(source));
        });
    });
}

criterion_group!(benches, benchmark_parallelism);
criterion_main!(benches);
