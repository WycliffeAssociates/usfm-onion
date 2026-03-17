mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::diff::{BuildSidBlocksOptions, diff_chapter_token_streams, diff_usfm_sources};
use usfm_onion::parse::parse;

fn benchmark_diff(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut token_group = c.benchmark_group("diff/tokens");
    for case in &corpus_cases {
        let baseline = parse(case.source.as_str());
        let current = parse(case.source.as_str());
        token_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        token_group.bench_with_input(
            BenchmarkId::new("diff_chapter_token_streams", case_label(case)),
            case,
            |b, _case| {
                b.iter(|| {
                    black_box(diff_chapter_token_streams(
                        &baseline.tokens,
                        &current.tokens,
                        &BuildSidBlocksOptions::default(),
                    ))
                });
            },
        );
    }
    token_group.finish();

    let mut source_group = c.benchmark_group("diff/source");
    for case in &corpus_cases {
        source_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        source_group.bench_with_input(
            BenchmarkId::new("diff_usfm_sources", case_label(case)),
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
    source_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("diff/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let parsed_docs = batch
                .docs
                .iter()
                .map(|doc| parse(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("diff_chapter_token_streams", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(diff_chapter_token_streams(
                                &parsed.tokens,
                                &parsed.tokens,
                                &BuildSidBlocksOptions::default(),
                            ));
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("diff_usfm_sources", batch_label(batch)),
                batch,
                |b, batch| {
                    b.iter(|| {
                        for doc in &batch.docs {
                            black_box(diff_usfm_sources(
                                doc.source.as_str(),
                                doc.source.as_str(),
                                &BuildSidBlocksOptions::default(),
                            ));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_diff);
criterion_main!(benches);
