mod common;

use common::{batch_label, case_label, selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::cst::{build_cst, build_cst_roots};
use usfm_onion::parse::parse;

fn benchmark_cst(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut corpus_group = c.benchmark_group("cst/corpus");
    for case in &corpus_cases {
        let parsed = parse(case.source.as_str());
        corpus_group.throughput(Throughput::Bytes(case.total_bytes as u64));
        corpus_group.bench_with_input(BenchmarkId::new("clone_tokens", case_label(case)), case, |b, _case| {
            b.iter(|| black_box(parsed.tokens.clone()));
        });
        corpus_group.bench_with_input(BenchmarkId::new("build_roots", case_label(case)), case, |b, _case| {
            b.iter(|| black_box(build_cst_roots(&parsed.tokens)));
        });
        corpus_group.bench_with_input(BenchmarkId::new("build_document", case_label(case)), case, |b, _case| {
            b.iter(|| black_box(build_cst(parsed.tokens.clone())));
        });
    }
    corpus_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("cst/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let parsed_docs = batch
                .docs
                .iter()
                .map(|doc| parse(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("clone_tokens", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(parsed.tokens.clone());
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("build_roots", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(build_cst_roots(&parsed.tokens));
                        }
                    });
                },
            );

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("build_document", batch_label(batch)),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(build_cst(parsed.tokens.clone()));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_cst);
criterion_main!(benches);
