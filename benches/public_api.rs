use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use usfm3_v2::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffStatus, FlatToken, FormatOptions,
    TokenLintOptions, TokenViewOptions, apply_revert_by_block_id, diff_chapter_token_streams,
    diff_usfm_sources, diff_usfm_sources_by_chapter, format_tokens, from_usj_value,
    from_usx_string, lint_tokens, parse, to_usj_value, to_usx_string, to_vref_json_string,
    tokens, write_exact,
};
#[cfg(feature = "rayon")]
use usfm3_v2::{diff_usfm_sources_by_chapter_parallel, diff_usfm_sources_parallel};

#[derive(Clone)]
struct SingleCase {
    label: &'static str,
    usfm: String,
    projected_tokens: Vec<FlatToken>,
    modified_usfm: String,
    modified_tokens: Vec<FlatToken>,
    modified_diff_block_id: String,
    usj: Value,
    usx: String,
}

struct CorpusCase {
    label: &'static str,
    files: Vec<SingleCase>,
    total_bytes: usize,
}

fn benchmark_public_api(c: &mut Criterion) {
    let tiny = load_single_case("tiny_fixture", "testData/basic/minimal/origin.usfm");
    let small = load_single_case("small_book", "en_ulb/65-3JN.usfm");
    let medium = load_single_case("medium_book", "en_ulb/67-REV.usfm");
    let large = load_single_case("large_book", "en_ulb/19-PSA.usfm");
    let corpus = load_en_ulb_corpus("en_ulb_corpus", "en_ulb");

    let sample_cases = [&tiny, &small, &medium, &large];

    bench_usfm_parse(c, &sample_cases, &corpus);
    bench_usfm_exact(c, &sample_cases, &corpus);
    bench_usfm_to_tokens(c, &sample_cases, &corpus);
    bench_usfm_to_lint(c, &sample_cases, &corpus);
    bench_usfm_to_diff(c, &sample_cases, &corpus);
    bench_usfm_to_diff_by_chapter(c, &sample_cases, &corpus);
    bench_usfm_to_usj(c, &sample_cases, &corpus);
    bench_usfm_to_usx(c, &sample_cases, &corpus);
    bench_usfm_to_vref(c, &sample_cases, &corpus);
    bench_tokens_lint(c, &sample_cases, &corpus);
    bench_tokens_diff(c, &sample_cases, &corpus);
    bench_tokens_revert(c, &sample_cases, &corpus);
    bench_tokens_format(c, &sample_cases, &corpus);
    bench_usj_to_usfm(c, &sample_cases, &corpus);
    bench_usx_to_usfm(c, &sample_cases, &corpus);
    #[cfg(feature = "rayon")]
    bench_parallel_diff(c, &corpus);
}

fn bench_usfm_parse(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_parse");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| parse(&case.usfm));
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = parse(&file.usfm);
            }
        });
    });
    group.finish();
}

fn bench_usfm_exact(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_exact_usfm");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let _ = write_exact(&handle);
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let _ = write_exact(&handle);
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_tokens(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_tokens");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let _ = tokens(&handle, TokenViewOptions::default());
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let _ = tokens(&handle, TokenViewOptions::default());
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_lint(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_lint");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let projected = tokens(&handle, TokenViewOptions::default());
                let _ = lint_tokens(&projected, TokenLintOptions::default());
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let projected = tokens(&handle, TokenViewOptions::default());
                let _ = lint_tokens(&projected, TokenLintOptions::default());
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_usj(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_usj");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let _ = to_usj_value(&handle);
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let _ = to_usj_value(&handle);
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_diff(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_diff");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = diff_usfm_sources(
                    &case.usfm,
                    &case.modified_usfm,
                    &TokenViewOptions::default(),
                    &BuildSidBlocksOptions::default(),
                );
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = diff_usfm_sources(
                    &file.usfm,
                    &file.modified_usfm,
                    &TokenViewOptions::default(),
                    &BuildSidBlocksOptions::default(),
                );
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_diff_by_chapter(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_diff_by_chapter");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = diff_usfm_sources_by_chapter(
                    &case.usfm,
                    &case.modified_usfm,
                    &TokenViewOptions::default(),
                    &BuildSidBlocksOptions::default(),
                );
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = diff_usfm_sources_by_chapter(
                    &file.usfm,
                    &file.modified_usfm,
                    &TokenViewOptions::default(),
                    &BuildSidBlocksOptions::default(),
                );
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_usx(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_usx");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let _ = to_usx_string(&handle).expect("USX should serialize");
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let _ = to_usx_string(&handle).expect("USX should serialize");
            }
        });
    });
    group.finish();
}

fn bench_usfm_to_vref(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usfm_to_vref");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let handle = parse(&case.usfm);
                let _ = to_vref_json_string(&handle);
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let handle = parse(&file.usfm);
                let _ = to_vref_json_string(&handle);
            }
        });
    });
    group.finish();
}

fn bench_tokens_lint(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/tokens_to_lint");
    for case in samples {
        let token_bytes = case.projected_tokens.iter().map(|token| token.text.len()).sum::<usize>();
        group.throughput(Throughput::Bytes(token_bytes as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = lint_tokens(&case.projected_tokens, TokenLintOptions::default());
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = lint_tokens(&file.projected_tokens, TokenLintOptions::default());
            }
        });
    });
    group.finish();
}

fn bench_tokens_diff(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/tokens_to_diff");
    for case in samples {
        let token_bytes = case.projected_tokens.iter().map(|token| token.text.len()).sum::<usize>();
        group.throughput(Throughput::Bytes(token_bytes as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = diff_chapter_token_streams(
                    &case.projected_tokens,
                    &case.modified_tokens,
                    &BuildSidBlocksOptions::default(),
                );
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = diff_chapter_token_streams(
                    &file.projected_tokens,
                    &file.modified_tokens,
                    &BuildSidBlocksOptions::default(),
                );
            }
        });
    });
    group.finish();
}

fn bench_tokens_revert(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/tokens_to_revert");
    for case in samples {
        let token_bytes = case.modified_tokens.iter().map(|token| token.text.len()).sum::<usize>();
        group.throughput(Throughput::Bytes(token_bytes as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = apply_revert_by_block_id(
                    &case.modified_diff_block_id,
                    &case.projected_tokens,
                    &case.modified_tokens,
                    &BuildSidBlocksOptions::default(),
                );
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = apply_revert_by_block_id(
                    &file.modified_diff_block_id,
                    &file.projected_tokens,
                    &file.modified_tokens,
                    &BuildSidBlocksOptions::default(),
                );
            }
        });
    });
    group.finish();
}

fn bench_tokens_format(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/tokens_to_format");
    for case in samples {
        let token_bytes = case.projected_tokens.iter().map(|token| token.text.len()).sum::<usize>();
        group.throughput(Throughput::Bytes(token_bytes as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = format_tokens(&case.projected_tokens, FormatOptions::default());
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = format_tokens(&file.projected_tokens, FormatOptions::default());
            }
        });
    });
    group.finish();
}

fn bench_usj_to_usfm(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usj_to_usfm");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usfm.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = from_usj_value(&case.usj).expect("USJ should import");
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = from_usj_value(&file.usj).expect("USJ should import");
            }
        });
    });
    group.finish();
}

fn bench_usx_to_usfm(c: &mut Criterion, samples: &[&SingleCase], corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api/usx_to_usfm");
    for case in samples {
        group.throughput(Throughput::Bytes(case.usx.len() as u64));
        group.bench_with_input(BenchmarkId::new("single", case.label), case, |b, case| {
            b.iter(|| {
                let _ = from_usx_string(&case.usx).expect("USX should import");
            });
        });
    }
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));
    group.bench_function(BenchmarkId::new("corpus", corpus.label), |b| {
        b.iter(|| {
            for file in &corpus.files {
                let _ = from_usx_string(&file.usx).expect("USX should import");
            }
        });
    });
    group.finish();
}

#[cfg(feature = "rayon")]
fn bench_parallel_diff(c: &mut Criterion, corpus: &CorpusCase) {
    let mut group = c.benchmark_group("public_api_parallel/usfm_diff");
    group.throughput(Throughput::Bytes(corpus.total_bytes as u64));

    group.bench_function(BenchmarkId::new("corpus_files_parallel", "usfm_to_diff"), |b| {
        b.iter(|| {
            corpus.files.par_iter().for_each(|file| {
                let _ = diff_usfm_sources(
                    &file.usfm,
                    &file.modified_usfm,
                    &TokenViewOptions::default(),
                    &BuildSidBlocksOptions::default(),
                );
            });
        });
    });

    group.bench_function(
        BenchmarkId::new("corpus_files_parallel", "usfm_to_diff_by_chapter"),
        |b| {
            b.iter(|| {
                corpus.files.par_iter().for_each(|file| {
                    let _ = diff_usfm_sources_by_chapter(
                        &file.usfm,
                        &file.modified_usfm,
                        &TokenViewOptions::default(),
                        &BuildSidBlocksOptions::default(),
                    );
                });
            });
        },
    );

    group.bench_function(
        BenchmarkId::new("corpus_internal_parallel", "usfm_to_diff"),
        |b| {
            b.iter(|| {
                for file in &corpus.files {
                    let _ = diff_usfm_sources_parallel(
                        &file.usfm,
                        &file.modified_usfm,
                        &TokenViewOptions::default(),
                        &BuildSidBlocksOptions::default(),
                    );
                }
            });
        },
    );

    group.bench_function(
        BenchmarkId::new("corpus_internal_parallel", "usfm_to_diff_by_chapter"),
        |b| {
            b.iter(|| {
                for file in &corpus.files {
                    let _ = diff_usfm_sources_by_chapter_parallel(
                        &file.usfm,
                        &file.modified_usfm,
                        &TokenViewOptions::default(),
                        &BuildSidBlocksOptions::default(),
                    );
                }
            });
        },
    );

    group.finish();
}

fn load_single_case(label: &'static str, relative_path: &str) -> SingleCase {
    let root = manifest_dir();
    let path = root.join(relative_path);
    let usfm = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let handle = parse(&usfm);
    let projected_tokens = tokens(&handle, TokenViewOptions::default());
    let modified_usfm = mutate_usfm_source(&usfm, &projected_tokens)
        .unwrap_or_else(|| panic!("failed to build modified benchmark case for {}", path.display()));
    let modified_handle = parse(&modified_usfm);
    let modified_tokens = tokens(&modified_handle, TokenViewOptions::default());
    let modified_diff_block_id = find_representative_modified_block(&projected_tokens, &modified_tokens)
        .unwrap_or_else(|| panic!("failed to find modified diff block for {}", path.display()));
    let usj = to_usj_value(&handle);
    let usx = to_usx_string(&handle)
        .unwrap_or_else(|error| panic!("failed to serialize USX for {}: {error}", path.display()));
    SingleCase {
        label,
        usfm,
        projected_tokens,
        modified_usfm,
        modified_tokens,
        modified_diff_block_id,
        usj,
        usx,
    }
}

fn load_en_ulb_corpus(label: &'static str, relative_dir: &str) -> CorpusCase {
    let root = manifest_dir().join(relative_dir);
    let mut paths = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
        .collect::<Vec<_>>();
    paths.sort();

    let files = paths
        .into_iter()
        .map(|path| {
            let usfm = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let handle = parse(&usfm);
            let projected_tokens = tokens(&handle, TokenViewOptions::default());
            let modified_usfm = mutate_usfm_source(&usfm, &projected_tokens).unwrap_or_else(|| {
                panic!("failed to build modified benchmark case for {}", path.display())
            });
            let modified_handle = parse(&modified_usfm);
            let modified_tokens = tokens(&modified_handle, TokenViewOptions::default());
            let modified_diff_block_id =
                find_representative_modified_block(&projected_tokens, &modified_tokens)
                    .unwrap_or_else(|| {
                        panic!("failed to find modified diff block for {}", path.display())
                    });
            let usj = to_usj_value(&handle);
            let usx = to_usx_string(&handle)
                .unwrap_or_else(|error| panic!("failed to serialize USX for {}: {error}", path.display()));
            SingleCase {
                label: Box::leak(path.file_name().unwrap().to_string_lossy().into_owned().into_boxed_str()),
                usfm,
                projected_tokens,
                modified_usfm,
                modified_tokens,
                modified_diff_block_id,
                usj,
                usx,
            }
        })
        .collect::<Vec<_>>();

    let total_bytes = files.iter().map(|file| file.usfm.len()).sum();
    CorpusCase {
        label,
        files,
        total_bytes,
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mutate_usfm_source(usfm: &str, projected_tokens: &[FlatToken]) -> Option<String> {
    let mut candidates = projected_tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.text.trim().is_empty() {
                return None;
            }
            if !matches!(
                token.kind,
                usfm3_v2::TokenKind::Text | usfm3_v2::TokenKind::BookCode
            ) {
                return None;
            }
            let replacement = mutate_text_preserving_length(&token.text)?;
            Some((index, replacement))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }
    let (chosen_index, replacement) = candidates.remove(candidates.len() / 2);
    let token = projected_tokens.get(chosen_index)?;
    let mut out = usfm.to_string();
    out.replace_range(token.span.clone(), &replacement);
    Some(out)
}

fn mutate_text_preserving_length(text: &str) -> Option<String> {
    let mut chars = text.chars().collect::<Vec<_>>();
    let index = chars.iter().position(|ch| ch.is_ascii_alphabetic())?;
    let replacement = match chars[index] {
        'a' => 'b',
        'A' => 'B',
        'z' => 'y',
        'Z' => 'Y',
        _ => 'z',
    };
    chars[index] = replacement;
    Some(chars.into_iter().collect())
}

fn find_representative_modified_block(
    baseline_tokens: &[FlatToken],
    modified_tokens: &[FlatToken],
) -> Option<String> {
    let diffs = diff_chapter_token_streams(
        baseline_tokens,
        modified_tokens,
        &BuildSidBlocksOptions::default(),
    );
    prefer_middle_modified_diff(&diffs)
        .map(|diff| diff.block_id.clone())
}

fn prefer_middle_modified_diff<T>(diffs: &[ChapterTokenDiff<T>]) -> Option<&ChapterTokenDiff<T>> {
    let modified = diffs
        .iter()
        .filter(|diff| diff.status == DiffStatus::Modified)
        .collect::<Vec<_>>();
    modified.get(modified.len() / 2).copied()
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = benchmark_public_api
);
criterion_main!(benches);
