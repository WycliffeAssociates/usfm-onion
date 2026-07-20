//! Measurement only — no parallel code. Answers "is chapter-grain worth it?"
//! by quantifying the load-balance argument: how skewed are book token counts
//! vs. chapter token counts. Under a work-stealing scheduler with fewer cores
//! than tasks, the makespan floor is the single largest task — so the ratio
//! max/median at each grain (and max_book / max_chapter) predicts the tail-
//! latency win from splitting a book into chapters.
//!
//! Run: `cargo run --release --example grain_stats -- example-corpora/en_ulb`

use std::fs;
use std::path::Path;

use usfm_onion::parse::parse;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "example-corpora/en_ulb".to_string());
    let root = Path::new(&dir);

    let mut usfm_files: Vec<_> = fs::read_dir(root)
        .unwrap_or_else(|e| panic!("read {dir}: {e}"))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("usfm"))
        .collect();
    usfm_files.sort();

    let mut book_tokens: Vec<usize> = Vec::new();
    let mut chapter_tokens: Vec<usize> = Vec::new();

    for path in &usfm_files {
        let source = fs::read_to_string(path).expect("read book");
        let parsed = parse(&source);
        book_tokens.push(parsed.tokens.len());

        // Segment at `\c` markers. Tokens before the first `\c` are the front
        // segment; each `\c`..next-`\c` is one chapter. This is the exact split
        // a chapter-parallel pass would use (token stream, not source bytes).
        let mut current = 0usize;
        for token in &parsed.tokens {
            if token.marker_name() == Some("c") && current > 0 {
                chapter_tokens.push(current);
                current = 0;
            }
            current += 1;
        }
        if current > 0 {
            chapter_tokens.push(current);
        }
    }

    report("BOOK grain", &mut book_tokens);
    report("CHAPTER grain", &mut chapter_tokens);

    let max_book = *book_tokens.iter().max().unwrap_or(&0);
    let max_chapter = *chapter_tokens.iter().max().unwrap_or(&0);
    println!("\n=== headline ===");
    println!("corpus: {dir}  ({} books)", usfm_files.len());
    println!(
        "tasks available:  {} at book grain  ->  {} at chapter grain",
        book_tokens.len(),
        chapter_tokens.len()
    );
    println!(
        "makespan floor (largest single task):  {max_book} tok (book)  vs  {max_chapter} tok (chapter)"
    );
    if max_chapter > 0 {
        println!(
            "=> chapter-grain shrinks the straggler by ~{:.1}x",
            max_book as f64 / max_chapter as f64
        );
    }
}

fn report(label: &str, counts: &mut [usize]) {
    counts.sort_unstable();
    let n = counts.len();
    if n == 0 {
        println!("{label}: no units");
        return;
    }
    let total: usize = counts.iter().sum();
    let min = counts[0];
    let max = counts[n - 1];
    let median = counts[n / 2];
    let mean = total / n;
    println!("\n=== {label} ===");
    println!("units: {n}   total tokens: {total}");
    println!("min {min}   median {median}   mean {mean}   max {max}");
    println!(
        "skew  max/median = {:.1}x   max/mean = {:.1}x",
        max as f64 / median.max(1) as f64,
        max as f64 / mean.max(1) as f64
    );
}
