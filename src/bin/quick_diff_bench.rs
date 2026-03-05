use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[cfg(feature = "rayon")]
use rayon::prelude::*;
use usfm3_v2::{
    BuildSidBlocksOptions, FlatToken, TokenKind, TokenViewOptions, diff_usfm_sources_by_chapter,
    parse, tokens,
};
#[cfg(feature = "rayon")]
use usfm3_v2::{diff_usfm_sources_by_chapter_parallel, diff_usfm_sources_parallel};

#[derive(Debug, Clone)]
struct Case {
    usfm: String,
    modified_usfm: String,
}

#[derive(Debug, Clone, Copy)]
struct BenchResult {
    files: usize,
    bytes: usize,
    parse_duration: Duration,
    diff_duration: Duration,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut corpus_dir = PathBuf::from("en_ulb");
    let mut iterations = 3usize;
    let mut warmups = 1usize;

    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                if let Some(value) = args.get(index + 1) {
                    corpus_dir = PathBuf::from(value);
                    index += 2;
                    continue;
                }
            }
            "--iterations" => {
                if let Some(value) = args.get(index + 1) {
                    if let Ok(parsed) = value.parse::<usize>() {
                        iterations = parsed.max(1);
                    }
                    index += 2;
                    continue;
                }
            }
            "--warmups" => {
                if let Some(value) = args.get(index + 1) {
                    if let Ok(parsed) = value.parse::<usize>() {
                        warmups = parsed;
                    }
                    index += 2;
                    continue;
                }
            }
            _ => {}
        }
        index += 1;
    }

    let cases = load_cases(&corpus_dir);
    if cases.is_empty() {
        eprintln!("no .usfm files found under {}", corpus_dir.display());
        std::process::exit(1);
    }

    let bytes = cases.iter().map(|case| case.usfm.len()).sum::<usize>();
    println!("corpus: {}", corpus_dir.display());
    println!("files: {}", cases.len());
    println!("bytes: {bytes}");
    println!("warmups: {warmups}");
    println!("iterations: {iterations}");
    println!();

    for _ in 0..warmups {
        let _ = run_single_threaded(&cases);
        #[cfg(feature = "rayon")]
        {
            let _ = run_parallel_over_files(&cases);
            let _ = run_parallel_internal(&cases);
        }
    }

    let single = collect_runs(iterations, || run_single_threaded(&cases));
    print_result("single_threaded", &single);

    #[cfg(feature = "rayon")]
    {
        let over_files = collect_runs(iterations, || run_parallel_over_files(&cases));
        print_result("rayon_files_parallel", &over_files);

        let internal = collect_runs(iterations, || run_parallel_internal(&cases));
        print_result("rayon_internal_parallel", &internal);
    }

    #[cfg(not(feature = "rayon"))]
    {
        println!("rayon: not enabled");
        println!("rerun with: cargo run --release --bin quick_diff_bench --features rayon -- --dir en_ulb");
    }
}

fn load_cases(root: &Path) -> Vec<Case> {
    let mut files = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
        .collect::<Vec<_>>();
    files.sort();

    files
        .into_iter()
        .map(|path| {
            let usfm = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let handle = parse(&usfm);
            let projected_tokens = tokens(&handle, TokenViewOptions::default());
            let modified_usfm = mutate_usfm_source(&usfm, &projected_tokens).unwrap_or_else(|| {
                panic!("failed to build modified benchmark case for {}", path.display())
            });
            Case {
                usfm,
                modified_usfm,
            }
        })
        .collect()
}

fn run_single_threaded(cases: &[Case]) -> BenchResult {
    let parse_start = Instant::now();
    for case in cases {
        let _ = parse(&case.usfm);
    }
    let parse_duration = parse_start.elapsed();

    let diff_start = Instant::now();
    for case in cases {
        let _ = diff_usfm_sources_by_chapter(
            &case.usfm,
            &case.modified_usfm,
            &TokenViewOptions::default(),
            &BuildSidBlocksOptions::default(),
        );
    }
    let diff_duration = diff_start.elapsed();

    BenchResult {
        files: cases.len(),
        bytes: cases.iter().map(|case| case.usfm.len()).sum(),
        parse_duration,
        diff_duration,
    }
}

#[cfg(feature = "rayon")]
fn run_parallel_over_files(cases: &[Case]) -> BenchResult {
    let parse_start = Instant::now();
    cases.par_iter().for_each(|case| {
        let _ = parse(&case.usfm);
    });
    let parse_duration = parse_start.elapsed();

    let diff_start = Instant::now();
    cases.par_iter().for_each(|case| {
        let _ = diff_usfm_sources_by_chapter(
            &case.usfm,
            &case.modified_usfm,
            &TokenViewOptions::default(),
            &BuildSidBlocksOptions::default(),
        );
    });
    let diff_duration = diff_start.elapsed();

    BenchResult {
        files: cases.len(),
        bytes: cases.iter().map(|case| case.usfm.len()).sum(),
        parse_duration,
        diff_duration,
    }
}

#[cfg(feature = "rayon")]
fn run_parallel_internal(cases: &[Case]) -> BenchResult {
    let parse_start = Instant::now();
    cases.par_iter().for_each(|case| {
        let _ = parse(&case.usfm);
    });
    let parse_duration = parse_start.elapsed();

    let diff_start = Instant::now();
    for case in cases {
        let _ = diff_usfm_sources_parallel(
            &case.usfm,
            &case.modified_usfm,
            &TokenViewOptions::default(),
            &BuildSidBlocksOptions::default(),
        );
        let _ = diff_usfm_sources_by_chapter_parallel(
            &case.usfm,
            &case.modified_usfm,
            &TokenViewOptions::default(),
            &BuildSidBlocksOptions::default(),
        );
    }
    let diff_duration = diff_start.elapsed();

    BenchResult {
        files: cases.len(),
        bytes: cases.iter().map(|case| case.usfm.len()).sum(),
        parse_duration,
        diff_duration,
    }
}

fn collect_runs<F>(iterations: usize, mut run: F) -> BenchResult
where
    F: FnMut() -> BenchResult,
{
    let mut runs = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        runs.push(run());
    }

    let first = runs[0];
    BenchResult {
        files: first.files,
        bytes: first.bytes,
        parse_duration: average_duration(runs.iter().map(|run| run.parse_duration)),
        diff_duration: average_duration(runs.iter().map(|run| run.diff_duration)),
    }
}

fn print_result(label: &str, result: &BenchResult) {
    println!("{label}");
    println!(
        "  parse: {} total | {} / file | {} / s",
        format_duration(result.parse_duration),
        format_duration(div_duration(result.parse_duration, result.files as u32)),
        format_bytes_per_sec(result.bytes, result.parse_duration),
    );
    println!(
        "  diff:  {} total | {} / file | {} / s",
        format_duration(result.diff_duration),
        format_duration(div_duration(result.diff_duration, result.files as u32)),
        format_bytes_per_sec(result.bytes, result.diff_duration),
    );
    println!();
}

fn average_duration<I>(durations: I) -> Duration
where
    I: Iterator<Item = Duration>,
{
    let values = durations.collect::<Vec<_>>();
    if values.is_empty() {
        return Duration::ZERO;
    }
    let count = values.len() as u32;
    let total: Duration = values.into_iter().sum();
    div_duration(total, count)
}

fn div_duration(duration: Duration, divisor: u32) -> Duration {
    if divisor == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(duration.as_secs_f64() / divisor as f64)
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() >= 1 {
        format!("{:.2}s", duration.as_secs_f64())
    } else if duration.as_millis() >= 1 {
        format!("{:.2}ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.2}µs", duration.as_secs_f64() * 1_000_000.0)
    }
}

fn format_bytes_per_sec(bytes: usize, duration: Duration) -> String {
    if duration.is_zero() {
        return "inf B".to_string();
    }
    let rate = bytes as f64 / duration.as_secs_f64();
    if rate >= 1024.0 * 1024.0 {
        format!("{:.2} MiB", rate / (1024.0 * 1024.0))
    } else if rate >= 1024.0 {
        format!("{:.2} KiB", rate / 1024.0)
    } else {
        format!("{rate:.2} B")
    }
}

fn mutate_usfm_source(usfm: &str, projected_tokens: &[FlatToken]) -> Option<String> {
    let mut candidates = projected_tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.text.trim().is_empty() {
                return None;
            }
            if !matches!(token.kind, TokenKind::Text | TokenKind::BookCode) {
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
    chars[index] = match chars[index] {
        'a' => 'b',
        'A' => 'B',
        'z' => 'y',
        'Z' => 'Y',
        _ => 'z',
    };
    Some(chars.into_iter().collect())
}
