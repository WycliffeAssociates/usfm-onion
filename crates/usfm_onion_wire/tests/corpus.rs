//! Corpus-scale wire gates. `#[ignore]`d by default: these walk every
//! `testData/**/*.usfm` fixture plus the `en_ult` and `en_ulb` example corpora,
//! which is far more work than a unit test should do on every `cargo test`. Run
//! with `cargo test -p usfm_onion_wire --test corpus -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::LintableToken;
use usfm_onion::lint::{LintCode, LintOptions, LintScope, lint_tokens};
use usfm_onion::markers::{MarkerKind, lookup_marker};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, UsfmToken};
use usfm_onion_wire::finding_codec::{decode_book, encode_book, issue_round_trips};

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

/// Evidence for the Phase B message-payload framing stop: a lint message
/// parameter whose value appears nowhere in the source and is not a catalog
/// marker, so no span or ordinal can encode it.
#[test]
fn message_params_can_carry_values_absent_from_the_source() {
    let source = "\\id php Philippians\n";
    let parsed = parse(source);
    let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.code == usfm_onion::lint::LintCode::BookCodeNotUppercase)
        .expect("lower-case book code is flagged");
    let uppercase = issue
        .message_params
        .get("uppercase")
        .expect("the remedy parameter");
    assert_eq!(uppercase, "PHP");
    // Neither a source slice nor a catalog marker: a span cannot name it and an
    // ordinal cannot either.
    assert!(!source.contains(uppercase.as_str()));
    assert_eq!(lookup_marker(uppercase).kind, MarkerKind::Unknown);
}

/// The six codes Gate 0D §2.1 found no *real* corpus fixture produces: three
/// reachable only from `lint_tokens(caller_tokens)` (never from
/// `lint_usfm(source)`, so no `.usfm` file can trigger them), and three that
/// are structurally reachable from source but happen not to occur in
/// `testData`/`en_ult`/`en_ulb` today. Hand-built fixtures for the first three
/// live in `usfm_onion_wire::finding_codec::tests`; this gate does not expect
/// any of the six to appear from a corpus walk.
const CORPUS_UNREACHABLE_CODES: [LintCode; 6] = [
    LintCode::MissingChapterNumber,
    LintCode::UnknownCloseMarker,
    LintCode::InvalidBookCode,
    LintCode::UnknownToken,
    LintCode::InvalidNumberRange,
    LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
];

/// Every `LintCode` the corpus conformance gate below expects to actually
/// fire at least once (the 26 codes not in [`CORPUS_UNREACHABLE_CODES`]).
fn corpus_producible_codes() -> Vec<LintCode> {
    [
        LintCode::MissingIdMarker,
        LintCode::DuplicateIdMarker,
        LintCode::IdMarkerNotAtFileStart,
        LintCode::EmptyParagraph,
        LintCode::MissingVerseNumber,
        LintCode::VerseIsEmpty,
        LintCode::UnknownMarker,
        LintCode::ContentBeforeFirstChapter,
        LintCode::VerseOutsideExplicitParagraph,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::MetadataOutsideTarget,
        LintCode::MarkerNotValidInContext,
        LintCode::MissingMilestoneSelfClose,
        LintCode::StrayCloseMarker,
        LintCode::MisnestedCloseMarker,
        LintCode::ImplicitlyClosedMarker,
        LintCode::UnclosedMarker,
        LintCode::DuplicateChapterNumber,
        LintCode::DuplicateVerseNumber,
        LintCode::MissingWhitespaceBeforeMarker,
        LintCode::MissingHorizontalWhitespaceAfterMarkerName,
        LintCode::MissingTagEndDelimiterAfterMarker,
        LintCode::MissingContentSpaceAfterCloseMarker,
        LintCode::VerseInSectionOrOtherParagraph,
        LintCode::ContentAfterBlankMarker,
        LintCode::BookCodeNotUppercase,
    ]
    .to_vec()
}

/// Gate 0D §2's per-`LintCode` semantic conformance gate: every finding the
/// full corpus (`testData` + `en_ult` + `en_ulb`) produces must round-trip
/// through [`encode_book`]/[`decode_book`] byte-for-byte-equivalent to
/// `LintIssue` (every field but `fix`, which §F.3 defers). Counts are
/// per-code, not just a total, so a code that silently stopped round-tripping
/// (while others kept the total non-zero) cannot hide.
#[test]
#[ignore = "walks the full corpus"]
fn corpus_findings_round_trip_per_lint_code() {
    let book = BookId::from_str("GEN").expect("test book code");
    let mut seen = std::collections::BTreeMap::<LintCode, usize>::new();
    let mut round_tripped = std::collections::BTreeMap::<LintCode, usize>::new();
    let mut total = 0usize;

    for path in corpus_paths() {
        let source = fs::read_to_string(&path).expect("fixture reads");
        let parsed = parse(&source);
        let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
        if result.issues.is_empty() {
            continue;
        }
        total += result.issues.len();

        let bytes = encode_book(book, &source, &parsed.tokens, &result.issues)
            .unwrap_or_else(|error| panic!("{}: encode failed: {error:?}", path.display()));
        let decoded = decode_book(&bytes, &source)
            .unwrap_or_else(|error| panic!("{}: decode failed: {error:?}", path.display()));

        let mut original = result.issues.clone();
        usfm_onion_wire::finding_codec::canonical_order_for_tokens(&mut original, &parsed.tokens);
        assert_eq!(
            original.len(),
            decoded.len(),
            "{}: finding count changed across the wire",
            path.display()
        );
        for (original, decoded) in original.iter().zip(&decoded) {
            *seen.entry(original.code).or_default() += 1;
            if issue_round_trips(original, decoded) {
                *round_tripped.entry(original.code).or_default() += 1;
            } else {
                panic!(
                    "{}: {:?} did not round-trip:\n  original: {original:?}\n  decoded:  {decoded:?}",
                    path.display(),
                    original.code
                );
            }
        }
    }

    // Printed so the ledger's evidence table can be regenerated from a run.
    for (code, count) in &seen {
        println!("{code:?}: {count} (round-tripped {})", round_tripped[code]);
    }
    println!("total findings={total}");

    assert!(total > 0, "corpus must exercise the finding codec");
    for code in corpus_producible_codes() {
        assert!(
            seen.get(&code).copied().unwrap_or(0) > 0,
            "expected {code:?} to appear at least once in the corpus"
        );
    }
    for code in CORPUS_UNREACHABLE_CODES {
        assert!(
            !corpus_producible_codes().contains(&code),
            "{code:?} must not double as both producible and unreachable"
        );
    }
}
