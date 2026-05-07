//! Shared bench fixtures for the criterion harnesses in this directory.
//!
//! Two surfaces:
//! - [`load_luke`] — the single representative book used by `operations.rs`
//!   for the per-op String-vs-Tokens matrix. Picked because it's mid-sized,
//!   exercises chapters/verses/notes/cross-references, and runs fast enough
//!   for tight iteration on perf-sensitive changes.
//! - [`load_en_ulb`] — the full English ULB corpus (~66 books) used by
//!   `parallelism.rs` to compare serial vs `rayon` orchestration on a
//!   realistic whole-Bible workload.

use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct Book {
    pub label: String,
    pub source: String,
    pub bytes: usize,
}

#[allow(dead_code)]
pub struct Corpus {
    pub label: String,
    pub books: Vec<String>,
    pub total_bytes: usize,
}

#[allow(dead_code)]
pub fn load_luke() -> Book {
    load_book("example-corpora/en_ulb/43-LUK.usfm", "luke")
}

#[allow(dead_code)]
pub fn load_en_ulb() -> Corpus {
    load_corpus("en_ulb", "example-corpora/en_ulb")
}

fn load_book(relative_path: &str, label: &str) -> Book {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("benchmark book {} should read: {err}", path.display()));
    let bytes = source.len();
    Book {
        label: format!("{label} ({bytes} bytes)"),
        source,
        bytes,
    }
}

fn load_corpus(label: &str, relative_root: &str) -> Corpus {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_root);
    let mut paths = Vec::new();
    collect_usfm_paths(&root, &mut paths);
    paths.sort();

    let books = paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("benchmark corpus file {} should read: {err}", path.display())
            })
        })
        .collect::<Vec<_>>();

    let total_bytes = books.iter().map(|src| src.len()).sum::<usize>();

    Corpus {
        label: format!("{label} ({} books, {total_bytes} bytes)", books.len()),
        books,
        total_bytes,
    }
}

fn collect_usfm_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|err| panic!("read {}: {err}", root.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry under {}: {err}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}
