// use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
// use rayon::prelude::*;
// use std::fs;
// use std::hint::black_box;
// use std::path::{Path, PathBuf};

// use usfm_onion::DocumentFormat;
// use usfm_onion::ast::usfm_to_ast;
// use usfm_onion::convert::{HtmlOptions, usfm_to_html, usfm_to_usj, usfm_to_usx, usfm_to_vref};

// #[derive(Clone)]
// struct Corpus {
//     name: &'static str,
//     usfm_sources: Vec<String>,
//     total_usfm_bytes: usize,
// }

// #[derive(Clone, Copy)]
// struct CorpusSpec {
//     name: &'static str,
//     relative_path: &'static str,
// }

// #[derive(Clone, Copy)]
// enum Mode {
//     Serial,
//     Parallel,
// }

// #[derive(Clone, Copy)]
// struct Operation {
//     label: &'static str,
//     run: fn(&Corpus, Mode) -> usize,
// }

// const CORPORA: &[CorpusSpec] = &[
//     CorpusSpec {
//         name: "examples.bsb",
//         relative_path: "example-corpora/examples.bsb",
//     },
//     CorpusSpec {
//         name: "bdf_reg",
//         relative_path: "example-corpora/bdf_reg",
//     },
//     CorpusSpec {
//         name: "en_ult",
//         relative_path: "example-corpora/en_ult",
//     },
// ];

// const OPERATIONS: &[Operation] = &[
//     Operation {
//         label: "usfm_to_ast",
//         run: bench_ast_corpus,
//     },
//     Operation {
//         label: "usfm_to_usj",
//         run: bench_usj_corpus,
//     },
//     Operation {
//         label: "usfm_to_usx",
//         run: bench_usx_corpus,
//     },
//     Operation {
//         label: "usfm_to_html",
//         run: bench_html_corpus,
//     },
//     Operation {
//         label: "usfm_to_vref",
//         run: bench_vref_corpus,
//     },
// ];

// fn benchmark_document_tree_corpus(c: &mut Criterion) {
//     let corpora = CORPORA.iter().map(load_corpus).collect::<Vec<_>>();
//     for operation in OPERATIONS {
//         let mut group = c.benchmark_group(format!("ast_corpus/{}", operation.label));

//         for corpus in &corpora {
//             group.throughput(Throughput::Bytes(corpus.total_usfm_bytes as u64));

//             group.bench_with_input(
//                 BenchmarkId::new("serial", corpus.name),
//                 corpus,
//                 |b, corpus| {
//                     b.iter(|| black_box((operation.run)(corpus, Mode::Serial)));
//                 },
//             );

//             group.bench_with_input(
//                 BenchmarkId::new("parallel", corpus.name),
//                 corpus,
//                 |b, corpus| {
//                     b.iter(|| black_box((operation.run)(corpus, Mode::Parallel)));
//                 },
//             );
//         }

//         group.finish();
//     }
// }

// fn bench_ast_corpus(corpus: &Corpus, mode: Mode) -> usize {
//     map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
//         let tree = usfm_to_ast(source);
//         tree.content.len() + tree.tokens.len()
//     })
//     .into_iter()
//     .sum()
// }

// fn bench_usj_corpus(corpus: &Corpus, mode: Mode) -> usize {
//     map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
//         usfm_to_usj(source)
//             .expect("USFM -> USJ should succeed")
//             .content
//             .len()
//     })
//     .into_iter()
//     .sum()
// }

// fn bench_usx_corpus(corpus: &Corpus, mode: Mode) -> usize {
//     map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
//         usfm_to_usx(source)
//             .expect("USFM -> USX should succeed")
//             .len()
//     })
//     .into_iter()
//     .sum()
// }

// fn bench_html_corpus(corpus: &Corpus, mode: Mode) -> usize {
//     map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
//         usfm_to_html(source, HtmlOptions::default())
//             .expect("USFM -> HTML should succeed")
//             .len()
//     })
//     .into_iter()
//     .sum()
// }

// fn bench_vref_corpus(corpus: &Corpus, mode: Mode) -> usize {
//     map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
//         usfm_to_vref(source)
//             .expect("USFM -> VREF should succeed")
//             .len()
//     })
//     .into_iter()
//     .sum()
// }

// fn map_sources<T, F>(sources: &[String], mode: Mode, map: F) -> Vec<T>
// where
//     T: Send,
//     F: Fn(&str) -> T + Sync + Send,
// {
//     match mode {
//         Mode::Serial => sources.iter().map(|source| map(source.as_str())).collect(),
//         Mode::Parallel => sources
//             .par_iter()
//             .map(|source| map(source.as_str()))
//             .collect(),
//     }
// }

// fn load_corpus(spec: &CorpusSpec) -> Corpus {
//     let root = manifest_root().join(spec.relative_path);
//     let mut files = collect_usfm_files(&root);
//     files.sort();

//     let usfm_sources = files
//         .iter()
//         .map(|path| {
//             fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {}", path.display()))
//         })
//         .collect::<Vec<_>>();
//     let total_usfm_bytes = usfm_sources.iter().map(String::len).sum::<usize>();

//     Corpus {
//         name: spec.name,
//         usfm_sources,
//         total_usfm_bytes,
//     }
// }

// fn manifest_root() -> PathBuf {
//     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
// }

// fn collect_usfm_files(root: &Path) -> Vec<PathBuf> {
//     let mut out = Vec::new();
//     collect_usfm_files_recursive(root, &mut out);
//     out
// }

// fn collect_usfm_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
//     let entries =
//         fs::read_dir(root).unwrap_or_else(|_| panic!("failed to read {}", root.display()));
//     for entry in entries {
//         let entry = entry.expect("failed to read directory entry");
//         let path = entry.path();
//         if path.is_dir() {
//             collect_usfm_files_recursive(&path, out);
//         } else if matches!(DocumentFormat::from_path(&path), Some(DocumentFormat::Usfm)) {
//             out.push(path);
//         }
//     }
// }

// criterion_group!(benches, benchmark_document_tree_corpus);
// criterion_main!(benches);
