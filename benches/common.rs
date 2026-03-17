use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct BenchCase {
    #[allow(dead_code)]
    pub name: &'static str,
    pub label: String,
    pub source: String,
    #[allow(dead_code)]
    pub total_chars: usize,
    pub total_bytes: usize,
}

pub struct CorpusDoc {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub source: String,
}

pub struct CorpusBatch {
    #[allow(dead_code)]
    pub name: String,
    pub label: String,
    pub docs: Vec<CorpusDoc>,
    #[allow(dead_code)]
    pub total_chars: usize,
    pub total_bytes: usize,
}

#[allow(dead_code)]
pub fn load_corpus_case(name: &'static str, relative_path: &str) -> BenchCase {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let source = fs::read_to_string(path).expect("benchmark corpus should read");
    let total_chars = source.chars().count();
    let total_bytes = source.len();
    BenchCase {
        name,
        label: format!("{name} ({total_chars} chars)"),
        source,
        total_chars,
        total_bytes,
    }
}

#[allow(dead_code)]
pub fn standard_corpus_cases() -> [BenchCase; 4] {
    [
        load_corpus_case("short", "example-corpora/en_ulb/64-2JN.usfm"),
        load_corpus_case("medium", "example-corpora/en_ulb/43-LUK.usfm"),
        load_corpus_case("large", "example-corpora/en_ulb/19-PSA.usfm"),
        load_corpus_case("xl", "example-corpora/en_ult/19-PSA.usfm"),
    ]
}

pub fn selected_corpus_batches() -> Vec<CorpusBatch> {
    let requested = requested_corpora();
    let corpus_names = if requested.is_empty() {
        Vec::new()
    } else if requested.iter().any(|name| name == "all") {
        available_corpora()
    } else {
        requested
    };

    corpus_names
        .into_iter()
        .map(|name| load_corpus_batch(&name))
        .collect()
}

fn requested_corpora() -> Vec<String> {
    std::env::var("USFM_BENCH_CORPORA")
        .ok()
        .map(|raw| {
            raw.split([',', ' '])
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn available_corpora() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("example-corpora");
    let mut names = fs::read_dir(&root)
        .expect("example-corpora directory should read")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            path.is_dir()
                .then(|| path.file_name()?.to_str().map(ToOwned::to_owned))
                .flatten()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn load_corpus_batch(name: &str) -> CorpusBatch {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("example-corpora")
        .join(name);
    let mut paths = Vec::new();
    collect_usfm_paths(&root, &mut paths);
    paths.sort();

    let docs = paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("benchmark corpus file should read");
            CorpusDoc { path, source }
        })
        .collect::<Vec<_>>();

    let total_bytes = docs.iter().map(|doc| doc.source.len()).sum::<usize>();
    let total_chars = docs.iter().map(|doc| doc.source.chars().count()).sum::<usize>();

    CorpusBatch {
        name: name.to_string(),
        label: format!(
            "{name} ({} files, {total_chars} chars)",
            docs.len()
        ),
        docs,
        total_chars,
        total_bytes,
    }
}

#[allow(dead_code)]
pub fn case_label(case: &BenchCase) -> &str {
    &case.label
}

pub fn batch_label(batch: &CorpusBatch) -> &str {
    &batch.label
}

fn collect_usfm_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("benchmark corpus directory should read") {
        let entry = entry.expect("benchmark corpus entry should read");
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}
