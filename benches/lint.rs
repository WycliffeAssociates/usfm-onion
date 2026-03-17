mod common;

use common::{selected_corpus_batches, standard_corpus_cases};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use usfm_onion::{LintOptions, parse};

fn benchmark_lint(c: &mut Criterion) {
    let corpus_cases = standard_corpus_cases();

    let mut token_group = c.benchmark_group("lint/tokens");
    for case in &corpus_cases {
        let parsed = parse(case.source.as_str());
        token_group.throughput(Throughput::Bytes(case.source.len() as u64));
        token_group.bench_with_input(
            BenchmarkId::new("lint_tokens", case.name),
            case,
            |b, _case| {
                b.iter(|| {
                    black_box(usfm_onion::lint_tokens(
                        &parsed.tokens,
                        LintOptions::default(),
                    ))
                });
            },
        );
    }
    token_group.finish();

    let selected_batches = selected_corpus_batches();
    if !selected_batches.is_empty() {
        let mut whole_corpus_group = c.benchmark_group("lint/whole-corpora");
        for batch in &selected_batches {
            whole_corpus_group.throughput(Throughput::Bytes(batch.total_bytes as u64));

            let parsed_docs = batch
                .docs
                .iter()
                .map(|doc| parse(doc.source.as_str()))
                .collect::<Vec<_>>();

            whole_corpus_group.bench_with_input(
                BenchmarkId::new("lint_tokens", &batch.name),
                batch,
                |b, _batch| {
                    b.iter(|| {
                        for parsed in &parsed_docs {
                            black_box(usfm_onion::lint_tokens(
                                &parsed.tokens,
                                LintOptions::default(),
                            ));
                        }
                    });
                },
            );
        }
        whole_corpus_group.finish();
    }
}

criterion_group!(benches, benchmark_lint);
criterion_main!(benches);
