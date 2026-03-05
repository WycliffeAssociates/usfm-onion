use std::fs;
use std::path::PathBuf;

use usfm3_v2::{parse, to_usj_string_pretty};

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --bin print_usj -- <path-to-usfm>");

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let handle = parse(&source);
    let usj = to_usj_string_pretty(&handle).expect("failed to serialize usj");
    println!("{usj}");
}
