//! Standalone profiling runner for `benches/operations.rs`'s per-op matrix,
//! meant to be wrapped by `samply record` for a native flamegraph. Not a
//! criterion bench — criterion's per-sample forking and harness overhead
//! make it a poor `samply` target; this just loops the op directly so a
//! short sampling window still collects enough stacks.
//!
//! Pulls in `benches/common.rs` directly (same fixture, same
//! `run_named_op` dispatcher `operations.rs` uses) so a flamegraph from
//! this binary and a number from `cargo bench --bench operations` are
//! always measuring the exact same code — there is no separate harness to
//! let drift in.
//!
//! Usage:
//!   cargo build --profile profiling --example profile_ops
//!   samply record -- target/profiling/examples/profile_ops diff/tokens 500
//!   samply record -- target/profiling/examples/profile_ops parse/string 2000
//!   samply record -- target/profiling/examples/profile_ops parse/serial 2000 --book psalms
//!   samply record -- target/profiling/examples/profile_ops lint/tokens 3000 --book bsb
//!   target/profiling/examples/profile_ops --list   # print valid op names
//!
//! `parse/serial` forces the serial ingest path (lex + parse_lexemes),
//! bypassing parse()'s chapter-parallel routing so a big book like
//! `--book psalms` profiles with no rayon in the stack — pure lib work.
//!
//! `--book <ulb|bsb|psalms>` (default `ulb`) selects the fixture: `ulb` is
//! en_ulb's Luke (491 `\s5` chunk markers — undocumented, so each fires a
//! lint issue; good for seeing what a real, recurring lint finding costs).
//! `bsb` is BSB's Luke (no `\s5`, but otherwise more USFM-varied — `\r`,
//! `\pc`, footnote/poetry markup en_ulb's Luke doesn't use; good for seeing
//! what inherent USFM complexity costs without that noise).

#[path = "../benches/common.rs"]
mod common;

use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut positional = Vec::new();
    let mut book_name = "ulb".to_string();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--book" {
            book_name = iter.next().unwrap_or_else(|| {
                eprintln!("--book requires a value (ulb|bsb)");
                std::process::exit(1);
            });
        } else {
            positional.push(arg);
        }
    }

    let mut positional = positional.into_iter();
    let op = positional.next().unwrap_or_else(|| {
        eprintln!(
            "usage: profile_ops <op> [iterations] [--book ulb|bsb]  (or: profile_ops --list)"
        );
        std::process::exit(1);
    });

    if op == "--list" {
        for name in common::OP_NAMES {
            println!("{name}");
        }
        return;
    }

    if !common::OP_NAMES.contains(&op.as_str()) {
        eprintln!("unknown op {op:?}; valid ops:");
        for name in common::OP_NAMES {
            eprintln!("  {name}");
        }
        std::process::exit(1);
    }

    let iterations: usize = positional
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let book = match book_name.as_str() {
        "ulb" => common::load_luke(),
        "bsb" => common::load_bsb_luke(),
        "psalms" => common::load_psalms(),
        other => {
            eprintln!("unknown --book {other:?}; expected ulb|bsb|psalms");
            std::process::exit(1);
        }
    };
    let fx = common::build_fixture(&book);

    println!(
        "=== {op} ({book_name}): {} bytes, {iterations} iterations ===",
        book.bytes
    );
    let started = Instant::now();
    for _ in 0..iterations {
        common::run_named_op(&op, &fx);
    }
    let elapsed = started.elapsed();
    let per_iteration = elapsed / iterations as u32;
    let mib_per_sec = (book.bytes as f64 / (1024.0 * 1024.0)) / per_iteration.as_secs_f64();
    println!(
        "  {iterations} iteration(s) in {elapsed:.2?} ({per_iteration:.2?}/iteration, {mib_per_sec:.1} MiB/s)"
    );
}
