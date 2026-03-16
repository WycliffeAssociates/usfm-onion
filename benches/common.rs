use std::fs;
use std::path::Path;

pub struct BenchCase {
    pub name: &'static str,
    pub source: String,
}

pub fn load_corpus_case(name: &'static str, relative_path: &str) -> BenchCase {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    BenchCase {
        name,
        source: fs::read_to_string(path).expect("benchmark corpus should read"),
    }
}

pub fn standard_corpus_cases() -> [BenchCase; 4] {
    [
        load_corpus_case("short", "example-corpora/en_ulb/64-2JN.usfm"),
        load_corpus_case("medium", "example-corpora/en_ulb/43-LUK.usfm"),
        load_corpus_case("large", "example-corpora/en_ulb/19-PSA.usfm"),
        load_corpus_case("xl", "example-corpora/en_ult/19-PSA.usfm"),
    ]
}
