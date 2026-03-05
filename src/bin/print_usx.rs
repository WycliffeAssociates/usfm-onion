use std::fs;
use std::path::PathBuf;

use usfm3_v2::{parse, to_usx_string};

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --bin print_usx -- <path-to-usfm>");

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let handle = parse(&source);
    let usx = to_usx_string(&handle).expect("failed to serialize usx");
    println!("{usx}");
}
