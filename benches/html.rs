mod common;

use common::{selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{HtmlOptions, parse, tokens_to_html, usfm_to_html};

fn benchmark_html(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases()
        .into_iter()
        .filter(|case| case.name != "xl")
        .collect::<Vec<_>>();

    let mut token_group = c.benchmark_group("html/tokens");
    for case in &corpus_cases {
        let parsed = parse(case.source.as_str());
        token_group.throughput(Throughput::Bytes(case.source.len() as u64));
        token_group.bench_with_input(
            BenchmarkId::new("tokens_to_html", case.name),
            case,
            |b, _case| {
                b.iter(|| black_box(tokens_to_html(&parsed.tokens, HtmlOptions::default())));
            },
        );
    }
    token_group.finish();

    let mut source_group = c.benchmark_group("html/source");
    for case in &corpus_cases {
        source_group.throughput(Throughput::Bytes(case.source.len() as u64));
        source_group.bench_with_input(BenchmarkId::new("usfm_to_html", case.name), case, |b, case| {
            b.iter(|| black_box(usfm_to_html(case.source.as_str(), HtmlOptions::default())));
        });
    }
    source_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("html/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let parsed_docs = batch
                .docs
                .iter()
                .map(|doc| parse(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("tokens_to_html", &batch.name),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(tokens_to_html(&parsed.tokens, HtmlOptions::default()));
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("usfm_to_html", &batch.name),
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

criterion_group!(benches, benchmark_html);
criterion_main!(benches);
