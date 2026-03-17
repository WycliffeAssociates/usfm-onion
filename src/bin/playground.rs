// // AGENT: USE THIS FILE TO TEST AND BENCHMARK CODE

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() {
    let _bsb_corpus = "example-corpora/examples.bsb";
    let _en_ulb = "example-corpora/en_ulb";
    let _en_ult = "example-corpora/en_ult";
    // profile();
    // dump_usj();
    // dump_usx();
    // dump_vref();
    // dump_lint();
    // dump_format();
    // dump_diff();
    // dump_html();
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| {
    //     usfm_onion::format_usfm(source, usfm_onion::FormatOptions::default())
    // });
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| {
    //     usfm_onion::usfm_to_html(source, usfm_onion::HtmlOptions::default())
    // });
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| usfm_onion::parse(source));
    // 1. Load the actual USFM data into memory first
    // let corpus_path = Path::new(_bsb_corpus);
    // let entries = load_corpus(corpus_path)
    //     .into_iter()
    //     .map(|(path, source)| CorpusEntry {
    //         path: relative_display(&path),
    //         value: source, // We store the source string here to lint it later
    //     })
    //     .collect::<Vec<_>>();

    // println!("Loaded {} files. Starting profile...", entries.len());

    // 2. Fix the profile closure
    // let started = Instant::now();
    // profile(
    //     || {
    //         // entries.iter().map(...) is lazy! We use for_each to actually run it.
    //         entries.iter().for_each(|entry| {
    //             // Pass the content (entry.value), not the directory path
    //             // let _ = usfm_onion::lint_usfm(&entry.value, usfm_onion::LintOptions::default());
    //             let _ = usfm_onion::format_usfm(&entry.value, usfm_onion::FormatOptions::default());
    //         });
    //     },
    //     20,
    // );
    // let elapsed = started.elapsed();
    // println!("took {:?} time for {} iters", elapsed, 20);

    // println!("Profile complete.");

    dif_book_genesis();
}

#[allow(dead_code)]
fn profile_cst() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let mut total = 0usize;

    for _ in 0..200 {
        let doc = usfm_onion::parse_cst(&source);
        total += doc.tokens.len();
        std::hint::black_box(&doc);
    }

    println!("{total}");
}
#[allow(dead_code)]
fn profile<F: Fn()>(f: F, iters: usize) {
    for _ in 0..iters {
        std::hint::black_box(f());
    }
}

#[allow(dead_code)]
fn dump_usj() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let document = usfm_onion::usfm_to_usj(&source).expect("USJ export should succeed");

    let output_path = std::path::Path::new("playgroundOut.json");
    serde_json::to_writer_pretty(
        std::fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        &document,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!(
        "{}",
        serde_json::to_string_pretty(&document).expect("USJ should serialize")
    );
}

#[allow(dead_code)]
fn dump_usx() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let xml = usfm_onion::usfm_to_usx(&source).expect("USX export should succeed");

    let output_path = std::path::Path::new("playgroundOut.xml");
    std::fs::write(output_path, &xml)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{xml}");
}

#[allow(dead_code)]
fn dump_vref() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/631JNBSB.usfm").unwrap();
    let map = usfm_onion::usfm_to_vref_map(&source);
    let json = usfm_onion::vref_map_to_json_string(&map);

    let output_path = std::path::Path::new("playgroundOut.json");
    std::fs::write(output_path, &json)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{json}");
}

#[allow(dead_code)]
fn dump_lint() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let result = usfm_onion::lint_usfm(&source, usfm_onion::LintOptions::default());
    let json = serde_json::to_string_pretty(&result).expect("lint result should serialize");

    let output_path = std::path::Path::new("playgroundOut.json");
    std::fs::write(output_path, &json)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{json}");
}

#[allow(dead_code)]
fn dump_format() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let formatted = usfm_onion::format_usfm(&source, usfm_onion::FormatOptions::default());

    let output_path = std::path::Path::new("playgroundOut.usfm");
    std::fs::write(output_path, &formatted)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{formatted}");
}

#[allow(dead_code)]
fn dif_book_genesis() {
    let ulb_path = "example-corpora/en_ulb/01-GEN.usfm";
    let bsb_path = "example-corpora/examples.bsb/01GENBSB.usfm";

    // 1. Load the actual USFM data into memory first
    let ulb_source = fs::read_to_string(ulb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", ulb_path));
    let bsb_source = fs::read_to_string(bsb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", bsb_path));

    println!("Loaded files. Starting profile...");

    let iters = 20;
    let started = Instant::now();

    // 2. Profile the diffing operation
    profile(
        || {
            let _diffs = usfm_onion::diff_usfm_sources(
                &ulb_source,
                &bsb_source,
                &usfm_onion::BuildSidBlocksOptions::default(),
            );
        },
        iters,
    );

    let elapsed = started.elapsed();
    println!("took {:?} time for {} iters", elapsed, iters);
    println!("Profile complete.");
}

#[allow(dead_code)]
fn dump_diff() {
    let baseline = std::fs::read_to_string("example-corpora/examples.bsb/01GENBSB.usfm").unwrap();
    let current = baseline.replace(
        "God saw that the light was good",
        "God saw the light was good",
    );
    let diffs = usfm_onion::diff_usfm_sources(
        &baseline,
        &current,
        &usfm_onion::BuildSidBlocksOptions::default(),
    );

    let output_path = std::path::Path::new("playgroundOut.json");
    serde_json::to_writer_pretty(
        std::fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        &diffs,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!(
        "{}",
        serde_json::to_string_pretty(&diffs).expect("diff result should serialize")
    );
}

#[allow(dead_code)]
fn dump_html() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let html = usfm_onion::usfm_to_html(&source, usfm_onion::HtmlOptions::default());

    let output_path = std::path::Path::new("playgroundOut.html");
    std::fs::write(output_path, &html)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{html}");
}

#[derive(serde::Serialize)]
struct CorpusEntry<T> {
    path: String,
    value: T,
}

#[allow(dead_code)]
fn dump_file<T, F>(path: &str, f: F)
where
    T: serde::Serialize,
    F: Fn(&str) -> T,
{
    let path = Path::new(path);
    let source = read_source(path);
    let value = f(&source);
    write_json("playgroundOut.json", &value);
    println!("wrote {} to playgroundOut.json", path.display());
}

#[allow(dead_code)]
fn dump_corpus<T, F>(root: &str, f: F)
where
    T: serde::Serialize,
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let entries = load_corpus(root)
        .into_iter()
        .map(|(path, source)| CorpusEntry {
            path: relative_display(&path),
            value: f(&source),
        })
        .collect::<Vec<_>>();
    write_json("playgroundOut.json", &entries);
    println!(
        "wrote {} corpus entries from {} to playgroundOut.json",
        entries.len(),
        root.display()
    );
}

#[allow(dead_code)]
fn time_file<T, F>(label: &str, path: &str, f: F)
where
    F: Fn(&str) -> T,
{
    let path = Path::new(path);
    let source = read_source(path);
    let started = Instant::now();
    let value = f(&source);
    let elapsed = started.elapsed();
    std::hint::black_box(value);

    print_timing(label, path, 1, source.len(), elapsed, None);
}

#[allow(dead_code)]
fn time_corpus<T, F>(label: &str, root: &str, iters: usize, f: F)
where
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let corpus = load_corpus(root);
    let bytes = corpus.iter().map(|(_, source)| source.len()).sum::<usize>();
    let started = Instant::now();
    run_corpus_iters(&corpus, iters, &f);
    let elapsed = started.elapsed();
    print_timing(
        label,
        root,
        corpus.len() * iters,
        bytes * iters,
        elapsed,
        None,
    );
}

#[allow(dead_code)]
fn profile_corpus<T, F>(label: &str, root: &str, iters: usize, f: F)
where
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let corpus = load_corpus(root);
    let bytes = corpus.iter().map(|(_, source)| source.len()).sum::<usize>();
    let started = Instant::now();
    run_corpus_iters(&corpus, iters, &f);
    let elapsed = started.elapsed();
    print_timing(
        label,
        root,
        corpus.len() * iters,
        bytes * iters,
        elapsed,
        Some(iters),
    );
}

#[allow(dead_code)]
fn time_parse_cst_file(path: &str) {
    let path = Path::new(path);
    let source = read_source(path);
    let started = Instant::now();
    let document = usfm_onion::parse_cst(&source);
    let elapsed = started.elapsed();

    println!(
        "parse_cst {} -> {} tokens in {:.2?}",
        path.display(),
        document.tokens.len(),
        elapsed
    );
}

fn load_corpus(root: &Path) -> Vec<(PathBuf, String)> {
    collect_usfm_paths(root)
        .into_iter()
        .map(|path| {
            let source = read_source(&path);
            (path, source)
        })
        .collect()
}

fn collect_usfm_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_usfm_paths_into(root, &mut paths);
    paths.sort();
    paths
}

fn collect_usfm_paths_into(root: &Path, paths: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths_into(&path, paths);
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

fn run_corpus_iters<T, F>(corpus: &[(PathBuf, String)], iters: usize, f: &F)
where
    F: Fn(&str) -> T,
{
    for _ in 0..iters {
        for (_, source) in corpus {
            let value = f(source);
            std::hint::black_box(value);
        }
    }
}

fn print_timing(
    label: &str,
    root: &Path,
    docs: usize,
    bytes: usize,
    elapsed: Duration,
    iters: Option<usize>,
) {
    let millis = elapsed.as_secs_f64() * 1000.0;
    let docs_per_sec = if elapsed.is_zero() {
        0.0
    } else {
        docs as f64 / elapsed.as_secs_f64()
    };
    let mib_per_sec = if elapsed.is_zero() {
        0.0
    } else {
        (bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
    };

    let mut summary = BTreeMap::new();
    summary.insert("label", label.to_string());
    summary.insert("root", root.display().to_string());
    summary.insert("docs", docs.to_string());
    summary.insert("bytes", bytes.to_string());
    if let Some(iters) = iters {
        summary.insert("iters", iters.to_string());
    }
    summary.insert("elapsed_ms", format!("{millis:.3}"));
    summary.insert("docs_per_sec", format!("{docs_per_sec:.2}"));
    summary.insert("mib_per_sec", format!("{mib_per_sec:.2}"));

    println!(
        "{}",
        serde_json::to_string_pretty(&summary).expect("timing summary should serialize")
    );
}
// fn main() {
//     let path =
//         Path::new(env!("CARGO_MANIFEST_DIR")).join("example-corpora/examples.bsb/642JNBSB.usfm");
//     let source = fs::read_to_string(&path)
//         .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
//     let document = cst::parse_usfm(&source);

//     let output_path = Path::new("playgroundOut.json");
//     serde_json::to_writer(
//         fs::File::create(output_path)
//             .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
//         &document,
//     )
//     .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

//     println!(
//         "{}",
//         serde_json::to_string_pretty(&document).expect("CST should serialize")
//     );
// }
