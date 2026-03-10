use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use usfm_onion::DocumentFormat;
use usfm_onion::lint::{LintOptions, TokenLintOptions, lint_content, lint_flat_tokens};
use usfm_onion::tokens::usfm_to_tokens;

const DEFAULT_PATH: &str = "example-corpora/en_ulb/19-PSA.usfm";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args(std::env::args().skip(1));
    let source = fs::read_to_string(&config.path)?;
    let initial_tokens = usfm_to_tokens(&source);
    let initial_issues = lint_flat_tokens(initial_tokens.as_slice(), TokenLintOptions::default());

    println!("ops lint trace");
    println!("path: {}", config.path.display());
    println!("bytes: {}", source.len());
    println!("tokens: {}", initial_tokens.len());
    println!("issues: {}", initial_issues.len());
    println!("warmup: {}", config.warmup);
    println!("iterations: {}", config.iterations);
    println!();

    let read_file = benchmark(config.warmup, config.iterations, || {
        black_box(fs::read_to_string(&config.path).expect("fixture should read"))
    });
    let tokenize = benchmark(config.warmup, config.iterations, || {
        black_box(usfm_to_tokens(black_box(source.as_str())))
    });
    let lint_flat = benchmark(config.warmup, config.iterations, || {
        black_box(lint_flat_tokens(
            black_box(initial_tokens.as_slice()),
            TokenLintOptions::default(),
        ))
    });
    let lint_full = benchmark(config.warmup, config.iterations, || {
        black_box(
            lint_content(
                black_box(source.as_str()),
                DocumentFormat::Usfm,
                LintOptions::default(),
            )
            .expect("ops lint should succeed"),
        )
    });

    print_stat("read file", &read_file);
    print_stat("usfm_to_tokens", &tokenize);
    print_stat("lint_flat_tokens", &lint_flat);
    print_stat("ops::lint_content", &lint_full);

    let residual_avg = lint_full.avg.saturating_sub(lint_flat.avg);
    let residual_min = lint_full.min.saturating_sub(lint_flat.min);
    let residual_max = lint_full.max.saturating_sub(lint_flat.max);

    println!();
    println!("derived");
    println!(
        "{:<24} avg {:>10}  min {:>10}  max {:>10}",
        "full - flat lint",
        format_duration(residual_avg),
        format_duration(residual_min),
        format_duration(residual_max),
    );
    println!(
        "{:<24} avg {:>10}",
        "full / flat lint",
        format!("{:.2}x", ratio(lint_full.avg, lint_flat.avg)),
    );
    println!(
        "{:<24} avg {:>10}",
        "tokenize / flat lint",
        format!("{:.2}x", ratio(tokenize.avg, lint_flat.avg)),
    );

    Ok(())
}

#[derive(Debug)]
struct Config {
    path: PathBuf,
    iterations: usize,
    warmup: usize,
}

impl Config {
    fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        let mut path = PathBuf::from(DEFAULT_PATH);
        let mut iterations = 20usize;
        let mut warmup = 3usize;

        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--path" => {
                    if let Some(value) = args.get(index + 1) {
                        path = PathBuf::from(value);
                        index += 1;
                    }
                }
                "--iterations" => {
                    if let Some(value) = args.get(index + 1) {
                        iterations = value.parse().expect("--iterations must be numeric");
                        index += 1;
                    }
                }
                "--warmup" => {
                    if let Some(value) = args.get(index + 1) {
                        warmup = value.parse().expect("--warmup must be numeric");
                        index += 1;
                    }
                }
                "--help" | "-h" => {
                    print_help(&path);
                    std::process::exit(0);
                }
                raw if !raw.starts_with('-') && path == Path::new(DEFAULT_PATH) => {
                    path = PathBuf::from(raw);
                }
                _ => {}
            }
            index += 1;
        }

        Self {
            path,
            iterations,
            warmup,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    min: Duration,
    max: Duration,
    avg: Duration,
}

fn benchmark<T>(warmup: usize, iterations: usize, mut f: impl FnMut() -> T) -> Stats {
    for _ in 0..warmup {
        black_box(f());
    }

    let mut total = Duration::ZERO;
    let mut min = Duration::MAX;
    let mut max = Duration::ZERO;

    for _ in 0..iterations {
        let started = Instant::now();
        black_box(f());
        let elapsed = started.elapsed();
        total += elapsed;
        min = min.min(elapsed);
        max = max.max(elapsed);
    }

    let avg = duration_div(total, iterations as u32);
    Stats { min, max, avg }
}

fn duration_div(duration: Duration, divisor: u32) -> Duration {
    if divisor == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(duration.as_secs_f64() / divisor as f64)
}

fn ratio(lhs: Duration, rhs: Duration) -> f64 {
    let rhs_secs = rhs.as_secs_f64();
    if rhs_secs == 0.0 {
        return f64::INFINITY;
    }
    lhs.as_secs_f64() / rhs_secs
}

fn print_stat(label: &str, stats: &Stats) {
    println!(
        "{:<24} avg {:>10}  min {:>10}  max {:>10}",
        label,
        format_duration(stats.avg),
        format_duration(stats.min),
        format_duration(stats.max),
    );
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_secs_f64() * 1_000.0;
    if millis >= 1.0 {
        format!("{millis:.2} ms")
    } else {
        format!("{:.2} us", millis * 1_000.0)
    }
}

fn print_help(default_path: &Path) {
    println!(
        "Usage: cargo run --bin ops-lint-trace -- [PATH] [--path PATH] [--iterations N] [--warmup N]"
    );
    println!("Default path: {}", default_path.display());
}
