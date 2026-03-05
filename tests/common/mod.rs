use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_origin_usfm_fixtures(root: &Path) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();
    visit(root, &mut fixtures);
    fixtures.sort();
    fixtures
}

pub fn collect_origin_usfm_json_pairs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    for usfm in collect_origin_usfm_fixtures(root) {
        let Some(parent) = usfm.parent() else {
            continue;
        };
        let json = parent.join("origin.json");
        if json.is_file() {
            pairs.push((usfm, json));
        }
    }
    pairs
}

pub fn collect_origin_usfm_xml_pairs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    for usfm in collect_origin_usfm_fixtures(root) {
        let Some(parent) = usfm.parent() else {
            continue;
        };
        let xml = parent.join("origin.xml");
        if xml.is_file() {
            pairs.push((usfm, xml));
        }
    }
    pairs
}

pub fn fixture_slug(root: &Path, fixture: &Path) -> String {
    fixture
        .strip_prefix(root)
        .unwrap_or(fixture)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "_")
        .replace('.', "_")
}

pub fn verbose_enabled() -> bool {
    matches!(
        std::env::var("USFM3_V2_VERBOSE").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

pub fn log_fixture(prefix: &str, fixture: &Path) {
    if verbose_enabled() {
        println!("{prefix}: {}", fixture.display());
    }
}

fn visit(root: &Path, fixtures: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, fixtures);
            continue;
        }
        if path.file_name().is_some_and(|name| name == "origin.usfm") {
            fixtures.push(path);
        }
    }
}
