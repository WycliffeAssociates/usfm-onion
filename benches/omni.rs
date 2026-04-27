mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::cst::build_cst;
use usfm_onion::diff::{BuildSidBlocksOptions, diff_chapter_token_streams, diff_usfm_sources};
use usfm_onion::format::{FormatOptions, format_tokens, format_usfm, into_format_tokens};
use usfm_onion::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use usfm_onion::lexer::lex;
use usfm_onion::lint::{LintOptions, lint_tokens, lint_usfm};
use usfm_onion::parse::{parse, parse_lexemes};
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;

fn benchmark_omni(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut group = c.benchmark_group("omni/corpus");
    for case in &corpus_cases {
        let lexed = lex(case.source.as_str());
        let parsed = parse(case.source.as_str());
        let cst = build_cst(parsed.tokens.clone());
        let format_tokens_case = into_format_tokens(&parsed.tokens);

        group.throughput(Throughput::Bytes(case.total_bytes as u64));

        group.bench_with_input(
            BenchmarkId::new("lex", case_label(case)),
            case,
            |b, case| {
                b.iter(|| black_box(lex(case.source.as_str())));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse", case_label(case)),
            case,
            |b, case| {
                b.iter(|| black_box(parse_lexemes(case.source.as_str(), &lexed.tokens)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cst", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| black_box(build_cst(parsed.tokens.clone())));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cst_walk", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| {
                    for item in cst.iter_walk() {
                        black_box(item.token.kind());
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usj", case_label(case)),
            case,
            |b, case| {
                b.iter(|| {
                    black_box(usfm_to_usj(case.source.as_str()).expect("USJ export should succeed"))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("usx", case_label(case)),
            case,
            |b, case| {
                b.iter(|| {
                    black_box(usfm_to_usx(case.source.as_str()).expect("USX export should succeed"))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("lint_tokens", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| black_box(lint_tokens(&parsed.tokens, LintOptions::default())));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("lint_usfm", case_label(case)),
            case,
            |b, case| {
                b.iter(|| black_box(lint_usfm(case.source.as_str(), LintOptions::default())));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_tokens", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| {
                    let mut working = format_tokens_case.clone();
                    format_tokens(&mut working, FormatOptions::default());
                    black_box(working);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_usfm", case_label(case)),
            case,
            |b, case| {
                b.iter(|| black_box(format_usfm(case.source.as_str(), FormatOptions::default())));
            },
        );

        if case.name != "xl" {
            group.bench_with_input(
                BenchmarkId::new("html_tokens", case_label(case)),
                case,
                |b, _case| {
                    b.iter(|| black_box(tokens_to_html(&parsed.tokens, HtmlOptions::default())));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("html_usfm", case_label(case)),
                case,
                |b, case| {
                    b.iter(|| {
                        black_box(usfm_to_html(case.source.as_str(), HtmlOptions::default()))
                    });
                },
            );
        }

        group.bench_with_input(
            BenchmarkId::new("diff_tokens", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| {
                    black_box(diff_chapter_token_streams(
                        &parsed.tokens,
                        &parsed.tokens,
                        &BuildSidBlocksOptions::default(),
                    ))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("diff_usfm", case_label(case)),
            case,
            |b, case| {
                b.iter(|| {
                    black_box(diff_usfm_sources(
                        case.source.as_str(),
                        case.source.as_str(),
                        &BuildSidBlocksOptions::default(),
                    ))
                });
            },
        );
    }
    group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("omni/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let parsed_docs = batch
                .docs
                .iter()
                .map(|doc| parse(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("lint_tokens", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(lint_tokens(&parsed.tokens, LintOptions::default()));
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("format_usfm", batch_label(batch)),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(format_usfm(doc.source.as_str(), FormatOptions::default()));
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("html_usfm", batch_label(batch)),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(usfm_to_html(doc.source.as_str(), HtmlOptions::default()));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_omni);
criterion_main!(benches);
