use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use usfm3_v2::{
    FormatOptions, FormatProfile, TokenLintOptions, TokenViewOptions, format_tokens_profile,
    lint_tokens, parse, tokens,
};

#[derive(Debug)]
struct BenchRun {
    files: usize,
    bytes: usize,
    projected_tokens: usize,
    lint_issues: usize,
    parse_duration: Duration,
    project_duration: Duration,
    lint_duration: Duration,
    format_duration: Duration,
    format_profile: FormatProfile,
    total_duration: Duration,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut corpus_dir = PathBuf::from("en_ulb");
    let mut iterations = 5usize;
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

    let files = collect_usfm_files(&corpus_dir);
    if files.is_empty() {
        eprintln!("no .usfm files found under {}", corpus_dir.display());
        std::process::exit(1);
    }

    println!("corpus: {}", corpus_dir.display());
    println!("files: {}", files.len());
    println!("warmups: {warmups}");
    println!("iterations: {iterations}");

    for _ in 0..warmups {
        let _ = run_bench(&files);
    }

    let mut runs = Vec::new();
    for iteration in 0..iterations {
        let run = run_bench(&files);
        println!(
            "run {}: parse={} project={} lint={} format={} total={}",
            iteration + 1,
            format_duration(run.parse_duration),
            format_duration(run.project_duration),
            format_duration(run.lint_duration),
            format_duration(run.format_duration),
            format_duration(run.total_duration),
        );
        runs.push(run);
    }

    let summary = summarize(&runs);
    println!();
    println!("summary");
    println!("bytes: {}", summary.bytes);
    println!("projected_tokens: {}", summary.projected_tokens);
    println!("lint_issues: {}", summary.lint_issues);
    println!(
        "parse avg={} per_file={} throughput={}/s",
        format_duration(summary.parse_duration),
        format_duration(div_duration(summary.parse_duration, summary.files as u32)),
        format_bytes_per_sec(summary.bytes, summary.parse_duration),
    );
    println!(
        "project avg={} per_file={} throughput={}/s",
        format_duration(summary.project_duration),
        format_duration(div_duration(summary.project_duration, summary.files as u32)),
        format_bytes_per_sec(summary.bytes, summary.project_duration),
    );
    println!(
        "lint avg={} per_file={} throughput={}/s",
        format_duration(summary.lint_duration),
        format_duration(div_duration(summary.lint_duration, summary.files as u32)),
        format_bytes_per_sec(summary.bytes, summary.lint_duration),
    );
    println!(
        "format avg={} per_file={} throughput={}/s",
        format_duration(summary.format_duration),
        format_duration(div_duration(summary.format_duration, summary.files as u32)),
        format_bytes_per_sec(summary.bytes, summary.format_duration),
    );
    println!("format breakdown:");
    println!(
        "  normalize={} verse_cluster={} default_p={} linebreaks={} collapse_nl={} line_start={}",
        format_duration(summary.format_profile.normalize),
        format_duration(summary.format_profile.verse_normalize),
        format_duration(summary.format_profile.default_paragraphs),
        format_duration(summary.format_profile.structural_linebreaks),
        format_duration(summary.format_profile.collapse_linebreaks),
        format_duration(summary.format_profile.normalize_line_start),
    );
    println!(
        "total avg={} per_file={} throughput={}/s",
        format_duration(summary.total_duration),
        format_duration(div_duration(summary.total_duration, summary.files as u32)),
        format_bytes_per_sec(summary.bytes, summary.total_duration),
    );
}

fn run_bench(files: &[PathBuf]) -> BenchRun {
    let mut bytes = 0usize;
    let mut projected_tokens = 0usize;
    let mut lint_issues = 0usize;

    let total_start = Instant::now();
    let mut parse_duration = Duration::ZERO;
    let mut project_duration = Duration::ZERO;
    let mut lint_duration = Duration::ZERO;
    let mut format_duration_total = Duration::ZERO;
    let mut format_profile = FormatProfile::default();

    for file in files {
        let source = fs::read_to_string(file).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", file.display());
        });
        bytes += source.len();

        let parse_start = Instant::now();
        let handle = parse(&source);
        parse_duration += parse_start.elapsed();

        let project_start = Instant::now();
        let projected = tokens(&handle, TokenViewOptions::default());
        project_duration += project_start.elapsed();
        projected_tokens += projected.len();

        let lint_start = Instant::now();
        let issues = lint_tokens(&projected, TokenLintOptions::default());
        lint_duration += lint_start.elapsed();
        lint_issues += issues.len();

        let format_start = Instant::now();
        let (_formatted, profile) = format_tokens_profile(&projected, FormatOptions::default());
        format_duration_total += format_start.elapsed();
        add_profile(&mut format_profile, &profile);
    }

    BenchRun {
        files: files.len(),
        bytes,
        projected_tokens,
        lint_issues,
        parse_duration,
        project_duration,
        lint_duration,
        format_duration: format_duration_total,
        format_profile,
        total_duration: total_start.elapsed(),
    }
}

fn summarize(runs: &[BenchRun]) -> BenchRun {
    let files = runs.first().map(|run| run.files).unwrap_or(0);
    let bytes = runs.first().map(|run| run.bytes).unwrap_or(0);
    let projected_tokens = runs.first().map(|run| run.projected_tokens).unwrap_or(0);
    let lint_issues = runs.first().map(|run| run.lint_issues).unwrap_or(0);

    let run_count = runs.len().max(1) as u32;
    BenchRun {
        files,
        bytes,
        projected_tokens,
        lint_issues,
        parse_duration: div_duration(
            runs.iter().map(|run| run.parse_duration).sum(),
            run_count,
        ),
        project_duration: div_duration(
            runs.iter().map(|run| run.project_duration).sum(),
            run_count,
        ),
        lint_duration: div_duration(
            runs.iter().map(|run| run.lint_duration).sum(),
            run_count,
        ),
        format_duration: div_duration(
            runs.iter().map(|run| run.format_duration).sum(),
            run_count,
        ),
        format_profile: average_profile(runs, run_count),
        total_duration: div_duration(
            runs.iter().map(|run| run.total_duration).sum(),
            run_count,
        ),
    }
}

fn collect_usfm_files(root: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn add_profile(target: &mut FormatProfile, profile: &FormatProfile) {
    target.normalize += profile.normalize;
    target.verse_normalize += profile.verse_normalize;
    target.default_paragraphs += profile.default_paragraphs;
    target.structural_linebreaks += profile.structural_linebreaks;
    target.collapse_linebreaks += profile.collapse_linebreaks;
    target.normalize_line_start += profile.normalize_line_start;
    target.total += profile.total;
}

fn average_profile(runs: &[BenchRun], run_count: u32) -> FormatProfile {
    let mut total = FormatProfile::default();
    for run in runs {
        add_profile(&mut total, &run.format_profile);
    }
    FormatProfile {
        normalize: div_duration(total.normalize, run_count),
        verse_normalize: div_duration(total.verse_normalize, run_count),
        default_paragraphs: div_duration(total.default_paragraphs, run_count),
        structural_linebreaks: div_duration(total.structural_linebreaks, run_count),
        collapse_linebreaks: div_duration(total.collapse_linebreaks, run_count),
        normalize_line_start: div_duration(total.normalize_line_start, run_count),
        total: div_duration(total.total, run_count),
    }
}

fn div_duration(duration: Duration, divisor: u32) -> Duration {
    if divisor == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(duration.as_secs_f64() / divisor as f64)
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs_f64() >= 1.0 {
        format!("{:.3}s", duration.as_secs_f64())
    } else {
        format!("{:.2}ms", duration.as_secs_f64() * 1000.0)
    }
}

fn format_bytes_per_sec(bytes: usize, duration: Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 {
        return "inf B".to_string();
    }
    let bytes_per_sec = bytes as f64 / seconds;
    if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.2} MiB", bytes_per_sec / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.2} KiB", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B", bytes_per_sec)
    }
}
