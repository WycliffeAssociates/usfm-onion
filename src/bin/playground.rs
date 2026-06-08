#![allow(dead_code)]
// AGENT: USE THIS FILE TO TEST AND BENCHMARK CODE
//
// Usage:
//   cargo run --release --bin playground             // default fixture
//   cargo run --release --bin playground -- <path>   // file or dir of *.usfm
//   cargo run --release --bin playground -- --inline // SAMPLE_INLINE
//
// Toggle ops by commenting/uncommenting lines in `main`. Every op:
//   - prints wall time + docs/sec + MiB/sec
//   - when sources.len() == 1 AND iters == 1, also writes playgroundOut.*
//   - otherwise just runs (good for `samply record -- ...`)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use usfm_onion::{BuildSidBlocksOptions, FormatOptions, HtmlOptions, LintOptions, LintScope};

// ---- default fixtures ----------------------------------------------------

const PSA: &str = "example-corpora/examples.bsb/19PSABSB.usfm";
const JN1: &str = "example-corpora/examples.bsb/631JNBSB.usfm";
const GEN_ULB: &str = "example-corpora/en_ulb/01-GEN.usfm";
const GEN_BSB: &str = "example-corpora/examples.bsb/01GENBSB.usfm";

const DEFAULT_FIXTURE: &str = PSA;

const SAMPLE_INLINE: &str = r#"\c 6
\p
\v 1 He gone from dere back up inta he hometown, wit he disciples followin' him.
\v 2 An' wen da Sabbath come, he start teachin' in da synagogue.
\v 3 "Een dis da carpenter son, Mary boy, da brudda uh James an' Joses?"
"#;

// ---- main ----------------------------------------------------------------

fn main() {
    let sources = resolve_sources();
    eprintln!(
        "playground: loaded {} source(s), {} bytes total",
        sources.len(),
        sources.iter().map(|(_, s)| s.len()).sum::<usize>()
    );

    // Uncomment what you want to run. Single ops below; diff is special (two inputs).
    // run_parse(&sources, 1);
    // run_cst(&sources, 1);
    run_lint(&sources, 800);
    // run_usj(&sources, 1);
    // run_usx(&sources, 1);
    // run_html(&sources, 200);
    // run_format(&sources, 1);
    // run_vref(&sources, 1);

    // Sampling examples (no disk writes; black-boxed):
    // run_html(&sources, 200);
    // run_cst(&sources, 200);

    // Diff (uses its own defaults: GEN_ULB vs GEN_BSB unless overridden):
    // run_diff(GEN_ULB, GEN_BSB, 1);
    // run_diff(GEN_ULB, GEN_BSB, 20);
}

// ---- ops -----------------------------------------------------------------

fn run_parse(sources: &[(String, String)], iters: usize) {
    time_op("parse", sources, iters, |s| {
        let _ = usfm_onion::parse::parse(s);
    });
    // No canonical on-disk shape; nothing to dump.
}

fn run_cst(sources: &[(String, String)], iters: usize) {
    time_op("cst", sources, iters, |s| {
        let doc = usfm_onion::cst::parse_cst(s);
        std::hint::black_box(doc.tokens.len());
    });
    if let Some((label, source)) = single(sources, iters) {
        let doc = usfm_onion::cst::parse_cst(source);
        write_json("playgroundOut.json", &doc);
        eprintln!("wrote playgroundOut.json (cst of {label})");
    }
}

fn run_lint(sources: &[(String, String)], iters: usize) {
    time_op("lint", sources, iters, |s| {
        let _ = usfm_onion::lint::lint_usfm(s, LintOptions::scoped(LintScope::Book));
    });
    if let Some((label, source)) = single(sources, iters) {
        let result = usfm_onion::lint::lint_usfm(source, LintOptions::scoped(LintScope::Book));
        write_json("playgroundOut.json", &result);
        eprintln!("wrote playgroundOut.json (lint of {label})");
    }
}

fn run_usj(sources: &[(String, String)], iters: usize) {
    time_op("usj", sources, iters, |s| {
        let _ = usfm_onion::usj::usfm_to_usj(s);
    });
    if let Some((label, source)) = single(sources, iters) {
        let doc = usfm_onion::usj::usfm_to_usj(source).expect("USJ export should succeed");
        write_json("playgroundOut.json", &doc);
        eprintln!("wrote playgroundOut.json (usj of {label})");
    }
}

fn run_usx(sources: &[(String, String)], iters: usize) {
    time_op("usx", sources, iters, |s| {
        let _ = usfm_onion::usx::usfm_to_usx(s);
    });
    if let Some((label, source)) = single(sources, iters) {
        let xml = usfm_onion::usx::usfm_to_usx(source).expect("USX export should succeed");
        write_text("playgroundOut.xml", &xml);
        eprintln!("wrote playgroundOut.xml (usx of {label})");
    }
}

fn run_html(sources: &[(String, String)], iters: usize) {
    time_op("html", sources, iters, |s| {
        let _ = usfm_onion::html::usfm_to_html(s, HtmlOptions::default());
    });
    if let Some((label, source)) = single(sources, iters) {
        let html = usfm_onion::html::usfm_to_html(source, HtmlOptions::default());
        write_text("playgroundOut.html", &html);
        eprintln!("wrote playgroundOut.html (html of {label})");
    }
}

fn run_format(sources: &[(String, String)], iters: usize) {
    time_op("format", sources, iters, |s| {
        let _ = usfm_onion::format::format_usfm(s, FormatOptions::default());
    });
    if let Some((label, source)) = single(sources, iters) {
        let formatted = usfm_onion::format::format_usfm(source, FormatOptions::default());
        write_text("playgroundOut.usfm", &formatted);
        eprintln!("wrote playgroundOut.usfm (format of {label})");
    }
}

fn run_vref(sources: &[(String, String)], iters: usize) {
    time_op("vref", sources, iters, |s| {
        let _ = usfm_onion::vref::usfm_to_vref_map(s);
    });
    if let Some((label, source)) = single(sources, iters) {
        let map = usfm_onion::vref::usfm_to_vref_map(source);
        let json = usfm_onion::vref::vref_map_to_json_string(&map);
        write_text("playgroundOut.json", &json);
        eprintln!("wrote playgroundOut.json (vref of {label})");
    }
}

#[allow(dead_code)]
fn run_diff(baseline_path: &str, current_path: &str, iters: usize) {
    let baseline = read_source(Path::new(baseline_path));
    let current = read_source(Path::new(current_path));
    let bytes = baseline.len() + current.len();

    let started = Instant::now();
    for _ in 0..iters {
        let diffs = usfm_onion::diff::diff_usfm_sources(
            &baseline,
            &current,
            &BuildSidBlocksOptions::default(),
        );
        std::hint::black_box(diffs);
    }
    let elapsed = started.elapsed();
    print_timing("diff", iters, bytes * iters, elapsed);

    if iters == 1 {
        let diffs = usfm_onion::diff::diff_usfm_sources(
            &baseline,
            &current,
            &BuildSidBlocksOptions::default(),
        );
        write_json("playgroundOut.json", &diffs);
        eprintln!("wrote playgroundOut.json (diff of {baseline_path} vs {current_path})");
    }
}

// ---- timing core ---------------------------------------------------------

fn time_op<F>(label: &str, sources: &[(String, String)], iters: usize, mut f: F)
where
    F: FnMut(&str),
{
    let bytes: usize = sources.iter().map(|(_, s)| s.len()).sum();
    let started = Instant::now();
    for _ in 0..iters {
        for (_, source) in sources {
            f(source);
        }
    }
    let elapsed = started.elapsed();
    let docs = sources.len() * iters;
    print_timing(label, docs, bytes * iters, elapsed);
}

fn print_timing(label: &str, docs: usize, bytes: usize, elapsed: Duration) {
    let secs = elapsed.as_secs_f64();
    let docs_per_sec = if secs > 0.0 { docs as f64 / secs } else { 0.0 };
    let mib_per_sec = if secs > 0.0 {
        (bytes as f64 / (1024.0 * 1024.0)) / secs
    } else {
        0.0
    };
    println!(
        "{label:<8} docs={docs:<6} bytes={bytes:<10} elapsed={:>9.3?}  {docs_per_sec:>8.1} docs/s  {mib_per_sec:>6.2} MiB/s",
        elapsed
    );
}

// ---- source resolution ---------------------------------------------------

fn resolve_sources() -> Vec<(String, String)> {
    let mut args = std::env::args().skip(1);
    let mut path_arg: Option<String> = None;
    let mut inline = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--inline" => inline = true,
            "-h" | "--help" => {
                eprintln!(
                    "usage: playground [<path>] [--inline]\n  <path>: file or directory (recursive *.usfm)\n  --inline: use SAMPLE_INLINE instead of any file"
                );
                std::process::exit(0);
            }
            other if other.starts_with("--") => {
                eprintln!("unknown flag: {other}");
                std::process::exit(2);
            }
            other => path_arg = Some(other.to_string()),
        }
    }

    if inline {
        return vec![("<inline>".to_string(), SAMPLE_INLINE.to_string())];
    }

    let path = PathBuf::from(path_arg.unwrap_or_else(|| DEFAULT_FIXTURE.to_string()));
    if path.is_dir() {
        let paths = collect_usfm_paths(&path);
        if paths.is_empty() {
            eprintln!("no *.usfm files found under {}", path.display());
            std::process::exit(1);
        }
        paths
            .into_iter()
            .map(|p| (relative_display(&p), read_source(&p)))
            .collect()
    } else {
        vec![(relative_display(&path), read_source(&path))]
    }
}

fn single<'a>(sources: &'a [(String, String)], iters: usize) -> Option<(&'a str, &'a str)> {
    if iters == 1 && sources.len() == 1 {
        Some((sources[0].0.as_str(), sources[0].1.as_str()))
    } else {
        None
    }
}

// ---- io helpers ----------------------------------------------------------

fn collect_usfm_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    walk(root, &mut paths);
    paths.sort();
    paths
}

fn walk(root: &Path, paths: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            walk(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn relative_display(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .ok()
        .unwrap_or(path)
        .display()
        .to_string()
}

fn write_json<T: serde::Serialize>(path: &str, value: &T) {
    let output_path = Path::new(path);
    serde_json::to_writer_pretty(
        fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        value,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
}

fn write_text(path: &str, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|error| panic!("failed to write {path}: {error}"));
}
