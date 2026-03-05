use std::fs;
use std::path::PathBuf;

use usfm3_v2::{DebugDumpOptions, debug_dump, parse};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut path: Option<PathBuf> = None;
    let mut options = DebugDumpOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--raw" => {
                options.include_raw = true;
                options.include_projected = false;
                options.include_recoveries = false;
                options.include_lint = false;
                options.include_document = false;
            }
            "--projected" => {
                options.include_raw = false;
                options.include_projected = true;
                options.include_recoveries = false;
                options.include_lint = false;
                options.include_document = false;
            }
            "--recoveries" => {
                options.include_raw = false;
                options.include_projected = false;
                options.include_recoveries = true;
                options.include_lint = false;
                options.include_document = false;
            }
            "--lint" => {
                options.include_raw = false;
                options.include_projected = false;
                options.include_recoveries = false;
                options.include_lint = true;
                options.include_document = false;
            }
            "--document" => {
                options.include_raw = false;
                options.include_projected = false;
                options.include_recoveries = false;
                options.include_lint = false;
                options.include_document = true;
            }
            "--all" => {
                options = DebugDumpOptions::default();
            }
            "--limit" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| panic!("--limit requires a numeric value"));
                options.limit = value
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("invalid --limit value: {value}"));
            }
            other if other.starts_with('-') => {
                panic!("unknown flag: {other}");
            }
            other => {
                path = Some(PathBuf::from(other));
            }
        }
    }

    let path = path.unwrap_or_else(|| {
        panic!(
            "usage: cargo run --bin inspect -- <path-to-usfm> [--all|--raw|--projected|--recoveries|--document] [--limit N]"
        )
    });

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let handle = parse(&source);
    println!("{}", debug_dump(&handle, options));
}
