mod common;

use common::{selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{FormatOptions, format_usfm, format_tokens, into_format_tokens, parse};

fn benchmark_format(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut token_group = c.benchmark_group("format/tokens");
    for case in &corpus_cases {
        let parsed = parse(case.source.as_str());
        let prepared = into_format_tokens(&parsed.tokens);
        token_group.throughput(Throughput::Bytes(case.source.len() as u64));
        token_group.bench_with_input(
            BenchmarkId::new("format_tokens", case.name),
            case,
            |b, _case| {
                b.iter(|| {
                    let mut working = prepared.clone();
                    format_tokens(&mut working, FormatOptions::default());
                    black_box(working)
                });
            },
        );
    }
    token_group.finish();

    let mut source_group = c.benchmark_group("format/source");
    for case in &corpus_cases {
        source_group.throughput(Throughput::Bytes(case.source.len() as u64));
        source_group.bench_with_input(BenchmarkId::new("format_usfm", case.name), case, |b, case| {
            b.iter(|| black_box(format_usfm(case.source.as_str(), FormatOptions::default())));
        });
    }
    source_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("format/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let prepared_docs = batch
                .docs
                .iter()
                .map(|doc| {
                    let parsed = parse(doc.source.as_str());
                    into_format_tokens(&parsed.tokens)
                })
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("format_tokens", &batch.name),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for prepared in &prepared_docs {
                            let mut working = prepared.clone();
                            format_tokens(&mut working, FormatOptions::default());
                            black_box(working);
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("format_usfm", &batch.name),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(format_usfm(doc.source.as_str(), FormatOptions::default()));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_format);
criterion_main!(benches);
