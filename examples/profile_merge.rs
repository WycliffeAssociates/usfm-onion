//! Standalone profiling harness for the merge-projection engine (`diff` +
//! `merge`), meant to be wrapped by `samply record` for a native flamegraph.
//! Not a criterion bench — criterion's per-sample forking and harness
//! overhead make it a poor `samply` target; this just loops each scenario
//! directly so a short sampling window still collects enough stacks.
//!
//! Scenarios mirror realistic diff/merge review sizes:
//! - `small`: a handful of verses changed in one book (a translator
//!   reviewing a few edits).
//! - `medium`: dozens of verses changed across one book (a chapter-level
//!   revision pass).
//! - `large`: a whole corpus reformatted and diffed against the original,
//!   book by book (the "did reformatting change any content" workflow).
//!
//! Usage (`profiling` inherits `release`'s codegen plus debug symbols —
//! `release` itself has none, so samply can't resolve function names from
//! it):
//!   cargo build --profile profiling --example profile_merge
//!   samply record -- target/profiling/examples/profile_merge small 300
//!   samply record -- target/profiling/examples/profile_merge medium 100
//!   samply record -- target/profiling/examples/profile_merge large 5
//!   samply record -- target/profiling/examples/profile_merge all       # default iteration counts

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use usfm_onion::diff::{MergeSide, diff_skeleton_canonical, merge_skeleton};
use usfm_onion::format::{FormatOptions, format_usfm};
use usfm_onion::parse::parse;

const BOOK: &str = "LUK";

fn main() {
    let mut args = env::args().skip(1);
    let scenario = args.next().unwrap_or_else(|| "all".to_string());
    let iterations: Option<usize> = args.next().and_then(|s| s.parse().ok());

    match scenario.as_str() {
        "small" => run_book_scenario("small (3 verses changed)", 3, iterations.unwrap_or(300)),
        "medium" => run_book_scenario("medium (~50 verses changed)", 50, iterations.unwrap_or(100)),
        "large" => run_corpus_scenario(iterations.unwrap_or(5)),
        "all" => {
            run_book_scenario("small (3 verses changed)", 3, iterations.unwrap_or(300));
            run_book_scenario("medium (~50 verses changed)", 50, iterations.unwrap_or(100));
            run_corpus_scenario(iterations.unwrap_or(5));
        }
        other => {
            eprintln!("unknown scenario {other:?}; expected small|medium|large|all");
            std::process::exit(1);
        }
    }
}

fn run_book_scenario(label: &str, verses_changed: usize, iterations: usize) {
    let baseline = load_book("example-corpora/en_ulb/43-LUK.usfm");
    let current = edit_n_verses(&baseline, verses_changed);
    run_diff_merge_loop(label, &baseline, &current, iterations);
}

/// A whole-corpus "reformat then diff" pass: every book in en_ulb is
/// reformatted with the default rules and diffed/merged against its
/// original — the realistic "did reformatting change any content" check.
fn run_corpus_scenario(iterations: usize) {
    let books = load_corpus("example-corpora/en_ulb");
    let total_bytes: usize = books.iter().map(|s| s.len()).sum();
    println!(
        "=== large (whole corpus reformat-and-diff, {} books, {:.2} MiB) ===",
        books.len(),
        total_bytes as f64 / (1024.0 * 1024.0)
    );

    let started = Instant::now();
    for _ in 0..iterations {
        for baseline in &books {
            let current = format_usfm(baseline, FormatOptions::default());
            let _ = diff_and_merge_once(baseline, &current);
        }
    }
    let elapsed = started.elapsed();
    let per_iteration = elapsed / iterations as u32;
    let mib_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / per_iteration.as_secs_f64();
    println!(
        "  {iterations} pass(es) over the corpus in {elapsed:.2?} ({per_iteration:.2?}/pass, {mib_per_sec:.1} MiB/s)"
    );
}

fn run_diff_merge_loop(label: &str, baseline: &str, current: &str, iterations: usize) {
    println!(
        "=== {label}: baseline {} bytes, current {} bytes, {iterations} iterations ===",
        baseline.len(),
        current.len()
    );
    let started = Instant::now();
    for _ in 0..iterations {
        let _ = diff_and_merge_once(baseline, current);
    }
    let elapsed = started.elapsed();
    println!("  {iterations} iteration(s) in {elapsed:.2?} ({:.2?}/iteration)", elapsed / iterations as u32);
}

/// One realistic diff+merge round-trip: build the skeleton, then merge it
/// twice (accept-all-incoming and reject-all-incoming) — the two most
/// common real decisions, exercising both the Shared/Coalesced "always
/// emit" paths and the Added/Deleted "conditionally emit" paths.
fn diff_and_merge_once(baseline: &str, current: &str) -> usize {
    let baseline_tokens = parse(baseline);
    let current_tokens = parse(current);
    let skeleton = diff_skeleton_canonical(
        &baseline_tokens.tokens,
        BOOK,
        &current_tokens.tokens,
        BOOK,
    );

    let empty = BTreeMap::new();
    let accept_incoming = merge_skeleton(&skeleton, &empty, MergeSide::Current).unwrap();
    let reject_incoming = merge_skeleton(&skeleton, &empty, MergeSide::Baseline).unwrap();

    accept_incoming.len() + reject_incoming.len()
}

/// Insert a short marker word into the text of `count` verses, spread
/// roughly evenly across the book (stride computed from the total verse
/// count) so "3 changed" and "50 changed" land on genuinely different
/// verses rather than all clustering at the start.
fn edit_n_verses(source: &str, count: usize) -> String {
    let total_verses = source.matches("\\v ").count();
    let stride = (total_verses / count.max(1)).max(1);

    let mut out = String::with_capacity(source.len() + count * 8);
    let mut verse_index = 0usize;
    let mut edited = 0usize;
    let mut rest = source;

    while let Some(marker_pos) = rest.find("\\v ") {
        let (before, after_marker_start) = rest.split_at(marker_pos);
        out.push_str(before);
        verse_index += 1;

        // Copy the marker + verse number + the single delimiter space, then
        // decide whether this verse gets the edit.
        let after_marker = &after_marker_start[3..]; // past "\v "
        let number_end = after_marker
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(after_marker.len());
        let (number, tail) = after_marker.split_at(number_end);
        out.push_str("\\v ");
        out.push_str(number);

        let mut tail = tail;
        if let Some(space) = tail.strip_prefix(' ') {
            out.push(' ');
            tail = space;
        }

        if edited < count && verse_index.is_multiple_of(stride) {
            out.push_str("edited ");
            edited += 1;
        }

        rest = tail;
    }
    out.push_str(rest);
    out
}

fn load_book(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn load_corpus(relative_root: &str) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_root);
    let mut paths = Vec::new();
    collect_usfm_paths(&root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display())))
        .collect()
}

fn collect_usfm_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}
