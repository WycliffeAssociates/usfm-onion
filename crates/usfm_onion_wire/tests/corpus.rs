//! Corpus-scale wire gates. `#[ignore]`d by default: these walk every
//! `testData/**/*.usfm` fixture plus the `en_ult` and `en_ulb` example corpora,
//! which is far more work than a unit test should do on every `cargo test`. Run
//! with `cargo test -p usfm_onion_wire --test corpus -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::LintableToken;
use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
use usfm_onion::markers::{MarkerKind, lookup_marker};
use usfm_onion::parse::parse;
use usfm_onion::token::UsfmToken;

/// Repo root, reachable from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("crate sits two levels below the repo root")
}

fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usfm(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("usfm") {
            out.push(path);
        }
    }
}

/// The adversarial fixture corpus plus the two example scripture corpora the
/// packet names. Sorted so a failure names the same file on every run.
fn corpus_paths() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    collect_usfm(&root.join("testData"), &mut out);
    collect_usfm(&root.join("example-corpora/en_ult"), &mut out);
    collect_usfm(&root.join("example-corpora/en_ulb"), &mut out);
    out.sort();
    assert!(
        !out.is_empty(),
        "corpus paths must resolve from the crate dir"
    );
    out
}

/// How a finding's `marker` can be represented on the wire without a
/// finding-section string dictionary.
#[derive(Debug, PartialEq, Eq)]
enum MarkerRepresentation {
    /// Recoverable from the anchored token's own marker descriptor: zero bytes.
    AnchoredToken,
    /// A canonical catalog marker, nameable by a stamp-gated ordinal.
    CatalogOrdinal,
    /// Present verbatim in the bound source, nameable by a span.
    SourceSpan,
}

/// The three marker values core hard-codes rather than reading off the anchored
/// token. All three must stay catalog markers, or the frozen representation
/// loses its "catalog ordinal" arm for them and the encoder would have to fall
/// back to a span.
///
/// Cheap enough to run unignored: it reads the marker catalog and no corpus.
#[test]
fn hardcoded_finding_markers_are_catalog_markers() {
    // `missing-id-marker` anchors nothing and names "id"; the duplicate-chapter
    // rule names "c" while anchored to the number token; the verse rules name "v"
    // while anchored to the number token.
    for marker in ["id", "c", "v"] {
        assert_ne!(
            lookup_marker(marker).kind,
            MarkerKind::Unknown,
            "core hard-codes {marker:?} as a finding marker"
        );
    }
}

/// Drift guard for the frozen three-way finding-marker representation.
///
/// A finding-section string dictionary would only be needed for a `marker` value
/// that is simultaneously not the anchored token's own marker, not a catalog
/// marker, and not present in the source. This asserts over the whole corpus
/// that no such value exists, so the freeze's three-way encoding stays
/// sufficient. If a future rule invents one, this fails and names the code.
#[test]
#[ignore = "walks the full corpus"]
fn every_finding_marker_uses_a_frozen_representation() {
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    let mut findings = 0usize;
    let mut with_marker = 0usize;

    for path in corpus_paths() {
        let source = fs::read_to_string(&path).expect("fixture reads");
        let parsed = parse(&source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        findings += result.issues.len();

        // Anchored-token lookup is by id, so index once per file rather than
        // scanning the token list per finding.
        let by_id: std::collections::BTreeMap<String, usize> = parsed
            .tokens
            .iter()
            .enumerate()
            .filter_map(|(index, token)| token.id().map(|id| (id, index)))
            .collect();

        for issue in &result.issues {
            let Some(marker) = issue.marker.as_deref() else {
                continue;
            };
            with_marker += 1;

            // Representation 1: the anchored token's own marker. The wire row
            // already points at a descriptor carrying this name, so the finding
            // stores nothing.
            let anchored = issue
                .token_id
                .as_deref()
                .and_then(|id| by_id.get(id))
                .map(|index| &parsed.tokens[*index]);
            let representation = if anchored.and_then(|token| token.marker()) == Some(marker) {
                MarkerRepresentation::AnchoredToken
            } else if lookup_marker(marker).kind != MarkerKind::Unknown {
                MarkerRepresentation::CatalogOrdinal
            } else if source.contains(marker) {
                MarkerRepresentation::SourceSpan
            } else {
                panic!(
                    "no frozen representation covers marker {marker:?} on code {:?} in {}",
                    issue.code,
                    path.display()
                );
            };
            *seen
                .entry(format!("{:?}/{representation:?}", issue.code))
                .or_default() += 1;
        }
    }

    // Printed so the ledger's evidence table can be regenerated from a run.
    for (key, count) in &seen {
        println!("{key}: {count}");
    }
    println!("findings={findings} with_marker={with_marker}");
    assert!(with_marker > 0, "corpus must exercise the marker field");
}
