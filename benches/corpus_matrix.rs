use rayon::prelude::*;
use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use usfm_onion::{
    convert::{
        HtmlOptions, convert_content, into_usj_lossless, into_usx_lossless, into_vref,
        usfm_content_to_html,
    },
    format::{FormatOptions, format_contents_with_options},
    lint::{LintOptions, lint_contents},
    model::{BatchExecutionOptions, DocumentFormat},
    parse::{IntoTokensOptions, into_tokens_from_contents, parse_content, parse_contents},
};

#[derive(Clone)]
struct Corpus {
    name: &'static str,
    usfm_sources: Vec<String>,
    usj_sources: Vec<String>,
    usx_sources: Vec<String>,
    file_count: usize,
    total_usfm_bytes: usize,
}

#[derive(Clone, Copy)]
struct CorpusSpec {
    name: &'static str,
    relative_path: &'static str,
}

#[derive(Clone, Copy)]
enum Mode {
    Serial,
    Parallel,
}

impl Mode {
    fn batch_options(self) -> BatchExecutionOptions {
        match self {
            Self::Serial => BatchExecutionOptions::sequential(),
            Self::Parallel => BatchExecutionOptions::parallel(),
        }
    }
}

#[derive(Clone, Copy)]
struct Operation {
    label: &'static str,
    run: fn(&Corpus, Mode) -> usize,
}

#[derive(Clone)]
struct OperationTiming {
    operation: &'static str,
    serial: Duration,
    parallel: Duration,
}

#[derive(Clone)]
struct CorpusTiming {
    corpus: Corpus,
    timings: Vec<OperationTiming>,
    iterations: usize,
}

const CORPORA: &[CorpusSpec] = &[
    CorpusSpec {
        name: "examples.bsb",
        relative_path: "example-corpora/examples.bsb",
    },
    CorpusSpec {
        name: "bdf_reg",
        relative_path: "example-corpora/bdf_reg",
    },
    CorpusSpec {
        name: "en_ult",
        relative_path: "example-corpora/en_ult",
    },
];

const OPERATIONS: &[Operation] = &[
    Operation {
        label: "parse usfm",
        run: bench_parse_usfm,
    },
    Operation {
        label: "project tokens",
        run: bench_into_tokens,
    },
    Operation {
        label: "lint usfm",
        run: bench_lint_usfm,
    },
    Operation {
        label: "format usfm",
        run: bench_format_usfm,
    },
    Operation {
        label: "usfm -> usj",
        run: bench_usfm_to_usj,
    },
    Operation {
        label: "usfm -> usj lossless",
        run: bench_usfm_to_usj_lossless,
    },
    Operation {
        label: "usfm -> usx",
        run: bench_usfm_to_usx,
    },
    Operation {
        label: "usfm -> usx lossless",
        run: bench_usfm_to_usx_lossless,
    },
    Operation {
        label: "usfm -> html",
        run: bench_usfm_to_html,
    },
    Operation {
        label: "usfm -> vref",
        run: bench_usfm_to_vref,
    },
    Operation {
        label: "usj -> usfm",
        run: bench_usj_to_usfm,
    },
    Operation {
        label: "usx -> usfm",
        run: bench_usx_to_usfm,
    },
];

fn main() {
    let config = Config::from_args(env::args().skip(1).collect());
    let corpora = CORPORA
        .iter()
        .filter(|spec| {
            config
                .corpus_filter
                .as_ref()
                .is_none_or(|filter| spec.name.contains(filter))
        })
        .map(load_corpus)
        .collect::<Vec<_>>();
    let timings = corpora
        .iter()
        .map(|corpus| run_corpus(corpus, &config))
        .collect::<Vec<_>>();

    if config.markdown {
        print!("{}", render_markdown(&timings));
    } else {
        print!("{}", render_text(&timings));
    }
}

struct Config {
    iterations: usize,
    warmup: usize,
    markdown: bool,
    corpus_filter: Option<String>,
    operation_filter: Option<String>,
}

impl Config {
    fn from_args(args: Vec<String>) -> Self {
        let mut iterations = 3usize;
        let mut warmup = 1usize;
        let mut markdown = false;
        let mut corpus_filter = None;
        let mut operation_filter = None;

        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--iterations" => {
                    if let Some(value) = args.get(index + 1) {
                        iterations = value.parse().expect("`--iterations` must be a number");
                        index += 1;
                    }
                }
                "--warmup" => {
                    if let Some(value) = args.get(index + 1) {
                        warmup = value.parse().expect("`--warmup` must be a number");
                        index += 1;
                    }
                }
                "--markdown" => markdown = true,
                "--corpus" => {
                    if let Some(value) = args.get(index + 1) {
                        corpus_filter = Some(value.clone());
                        index += 1;
                    }
                }
                "--operation" => {
                    if let Some(value) = args.get(index + 1) {
                        operation_filter = Some(value.clone());
                        index += 1;
                    }
                }
                "--help" | "-h" => {
                    println!(
                        "Usage: cargo bench --bench corpus_matrix --features rayon -- [--iterations N] [--warmup N] [--markdown] [--corpus NAME] [--operation TEXT]"
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
            index += 1;
        }

        Self {
            iterations,
            warmup,
            markdown,
            corpus_filter,
            operation_filter,
        }
    }
}

fn run_corpus(corpus: &Corpus, config: &Config) -> CorpusTiming {
    let timings = OPERATIONS
        .iter()
        .filter(|operation| {
            config
                .operation_filter
                .as_ref()
                .is_none_or(|filter| operation.label.contains(filter))
        })
        .map(|operation| {
            eprintln!("benchmarking {} / {}", corpus.name, operation.label);
            let serial = measure(
                operation.run,
                corpus,
                Mode::Serial,
                config.iterations,
                config.warmup,
            );
            let parallel = measure(
                operation.run,
                corpus,
                Mode::Parallel,
                config.iterations,
                config.warmup,
            );
            OperationTiming {
                operation: operation.label,
                serial,
                parallel,
            }
        })
        .collect::<Vec<_>>();

    CorpusTiming {
        corpus: corpus.clone(),
        timings,
        iterations: config.iterations,
    }
}

fn measure(
    op: fn(&Corpus, Mode) -> usize,
    corpus: &Corpus,
    mode: Mode,
    iterations: usize,
    warmup: usize,
) -> Duration {
    for _ in 0..warmup {
        black_box(op(corpus, mode));
    }

    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let checksum = black_box(op(corpus, mode));
        black_box(checksum);
        samples.push(start.elapsed());
    }
    median_duration(&mut samples)
}

fn median_duration(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn load_corpus(spec: &CorpusSpec) -> Corpus {
    let root = manifest_root().join(spec.relative_path);
    let mut files = collect_usfm_files(&root);
    files.sort();

    let usfm_sources = files
        .iter()
        .map(|path| {
            fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {}", path.display()))
        })
        .collect::<Vec<_>>();
    let total_usfm_bytes = usfm_sources.iter().map(String::len).sum::<usize>();

    let usj_sources = usfm_sources
        .iter()
        .map(|source| {
            convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usj)
                .expect("failed to precompute USJ corpus")
        })
        .collect::<Vec<_>>();

    let usx_sources = usfm_sources
        .iter()
        .map(|source| {
            convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usx)
                .expect("failed to precompute USX corpus")
        })
        .collect::<Vec<_>>();

    Corpus {
        name: spec.name,
        usfm_sources,
        usj_sources,
        usx_sources,
        file_count: total_file_count(&root),
        total_usfm_bytes,
    }
}

fn manifest_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_usfm_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_usfm_files_recursive(root, &mut out);
    out
}

fn collect_usfm_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(root).unwrap_or_else(|_| panic!("failed to read {}", root.display()));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_files_recursive(&path, out);
        } else if matches!(DocumentFormat::from_path(&path), Some(DocumentFormat::Usfm)) {
            out.push(path);
        }
    }
}

fn total_file_count(root: &Path) -> usize {
    let mut count = 0usize;
    let entries =
        fs::read_dir(root).unwrap_or_else(|_| panic!("failed to read {}", root.display()));
    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            count += total_file_count(&path);
        } else {
            count += 1;
        }
    }
    count
}

fn bench_parse_usfm(corpus: &Corpus, mode: Mode) -> usize {
    parse_contents(
        corpus.usfm_sources.as_slice(),
        DocumentFormat::Usfm,
        mode.batch_options(),
    )
    .into_iter()
    .map(|handle| handle.expect("parse corpus failed").source().len())
    .sum()
}

fn bench_into_tokens(corpus: &Corpus, mode: Mode) -> usize {
    into_tokens_from_contents(
        corpus.usfm_sources.as_slice(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
        mode.batch_options(),
    )
    .into_iter()
    .map(|tokens| tokens.expect("tokenize corpus failed").len())
    .sum()
}

fn bench_lint_usfm(corpus: &Corpus, mode: Mode) -> usize {
    lint_contents(
        corpus.usfm_sources.as_slice(),
        DocumentFormat::Usfm,
        LintOptions::default(),
        mode.batch_options(),
    )
    .into_iter()
    .map(|result| result.expect("lint corpus failed").len())
    .sum()
}

fn bench_format_usfm(corpus: &Corpus, mode: Mode) -> usize {
    format_contents_with_options(
        corpus.usfm_sources.as_slice(),
        DocumentFormat::Usfm,
        IntoTokensOptions::default(),
        FormatOptions::default(),
        mode.batch_options(),
    )
    .into_iter()
    .map(|result| result.expect("format corpus failed").tokens.len())
    .sum()
}

fn bench_usfm_to_usj(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usj)
            .expect("USFM -> USJ failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn bench_usfm_to_usj_lossless(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        let handle =
            parse_content(source, DocumentFormat::Usfm).expect("parse for lossless USJ failed");
        let document = into_usj_lossless(&handle);
        serde_json::to_vec(&document)
            .expect("serialize lossless USJ failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn bench_usfm_to_usx(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usx)
            .expect("USFM -> USX failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn bench_usfm_to_usx_lossless(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        let handle =
            parse_content(source, DocumentFormat::Usfm).expect("parse for lossless USX failed");
        into_usx_lossless(&handle)
            .expect("serialize lossless USX failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn bench_usfm_to_html(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        usfm_content_to_html(source, HtmlOptions::default()).len()
    })
    .into_iter()
    .sum()
}

fn bench_usfm_to_vref(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usfm_sources.as_slice(), mode, |source| {
        let handle = parse_content(source, DocumentFormat::Usfm).expect("parse for VREF failed");
        into_vref(&handle).len()
    })
    .into_iter()
    .sum()
}

fn bench_usj_to_usfm(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usj_sources.as_slice(), mode, |source| {
        convert_content(source, DocumentFormat::Usj, DocumentFormat::Usfm)
            .expect("USJ -> USFM failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn bench_usx_to_usfm(corpus: &Corpus, mode: Mode) -> usize {
    map_sources(corpus.usx_sources.as_slice(), mode, |source| {
        convert_content(source, DocumentFormat::Usx, DocumentFormat::Usfm)
            .expect("USX -> USFM failed")
            .len()
    })
    .into_iter()
    .sum()
}

fn map_sources<T, F>(sources: &[String], mode: Mode, map: F) -> Vec<T>
where
    T: Send,
    F: Fn(&str) -> T + Sync + Send,
{
    match mode {
        Mode::Serial => sources.iter().map(|source| map(source.as_str())).collect(),
        Mode::Parallel => sources
            .par_iter()
            .map(|source| map(source.as_str()))
            .collect(),
    }
}

fn render_text(results: &[CorpusTiming]) -> String {
    let mut out = String::new();
    out.push_str("Corpus timing matrix\n");
    out.push_str("===================\n\n");
    for result in results {
        out.push_str(&format!(
            "{}: {} USFM files, {} total files, {:.2} MiB\n",
            result.corpus.name,
            result.corpus.usfm_sources.len(),
            result.corpus.file_count,
            bytes_to_mib(result.corpus.total_usfm_bytes),
        ));
        for timing in &result.timings {
            out.push_str(&format!(
                "  {:<24} serial {:>8}  parallel {:>8}  speedup {:>5.2}x\n",
                timing.operation,
                format_duration(timing.serial),
                format_duration(timing.parallel),
                speedup(timing.serial, timing.parallel),
            ));
        }
        out.push('\n');
    }
    out
}

fn render_markdown(results: &[CorpusTiming]) -> String {
    let mut out = String::new();
    if let Some(first) = results.first() {
        out.push_str("<!-- corpus-bench:begin -->\n");
        out.push_str(&format!(
            "_Local release measurements, median wall-clock over {} runs, file-level parallelism via the `rayon` feature._\n\n",
            first.iterations
        ));
    }
    for result in results {
        out.push_str(&format!("### `{}`\n\n", result.corpus.name));
        out.push_str(&format!(
            "- USFM files: {}\n- Total files in corpus directory: {}\n- Total USFM source size: {:.2} MiB\n\n",
            result.corpus.usfm_sources.len(),
            result.corpus.file_count,
            bytes_to_mib(result.corpus.total_usfm_bytes),
        ));
        out.push_str("| Operation | Serial | Parallel | Speedup |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for timing in &result.timings {
            out.push_str(&format!(
                "| {} | {} | {} | {:.2}x |\n",
                timing.operation,
                format_duration(timing.serial),
                format_duration(timing.parallel),
                speedup(timing.serial, timing.parallel),
            ));
        }
        out.push('\n');
    }
    out.push_str("<!-- corpus-bench:end -->\n");
    out
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs_f64() >= 1.0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
    }
}

fn speedup(serial: Duration, parallel: Duration) -> f64 {
    let serial = serial.as_secs_f64();
    let parallel = parallel.as_secs_f64();
    if parallel == 0.0 {
        return 0.0;
    }
    serial / parallel
}

fn bytes_to_mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
