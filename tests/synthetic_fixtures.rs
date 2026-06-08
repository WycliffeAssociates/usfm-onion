//! Synthetic fixtures live in `testData/synthetic/`. Two invariants:
//!
//! 1. `kitchen-sink.usfm` exercises (as much as practical of) the marker
//!    catalog and must round-trip byte-identical through
//!    `parse → tokens_to_usfm`. Downstream consumers can rely on this
//!    file as a reference for losslessness.
//! 2. `common-errors.usfm` triggers a documented set of lint codes —
//!    the linter must continue to flag each one.

use std::collections::BTreeSet;

use usfm_onion::lint::{LintCode, LintOptions, LintScope, lint_usfm};
use usfm_onion::parse::parse;
use usfm_onion::token::tokens_to_usfm;

const KITCHEN_SINK: &str = include_str!("../testData/synthetic/kitchen-sink.usfm");
const COMMON_ERRORS: &str = include_str!("../testData/synthetic/common-errors.usfm");

#[test]
fn kitchen_sink_round_trips_byte_identical() {
    let parsed = parse(KITCHEN_SINK);
    let reconstructed = tokens_to_usfm(&parsed.tokens);
    if reconstructed != KITCHEN_SINK {
        // Surface the first diverging byte to make debugging tractable.
        let common_len = reconstructed
            .as_bytes()
            .iter()
            .zip(KITCHEN_SINK.as_bytes())
            .take_while(|(a, b)| a == b)
            .count();
        let line = KITCHEN_SINK[..common_len].lines().count();
        let context_start = KITCHEN_SINK[..common_len]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let orig_tail = &KITCHEN_SINK[context_start..(common_len + 60).min(KITCHEN_SINK.len())];
        let new_tail = &reconstructed[context_start..(common_len + 60).min(reconstructed.len())];
        panic!(
            "kitchen-sink did not round-trip byte-identical.\n\
             Diverged at byte {common_len} (around line {line}).\n\
             Original:      {orig_tail:?}\n\
             Reconstructed: {new_tail:?}"
        );
    }
}

#[test]
fn common_errors_triggers_expected_lint_codes() {
    let result = lint_usfm(COMMON_ERRORS, LintOptions::scoped(LintScope::Book));
    let observed: BTreeSet<LintCode> = result.issues.iter().map(|i| i.code).collect();

    // The fixture is deliberately broken; each section triggers one code.
    //
    // A few codes only fire on hand-built token streams or extreme inputs
    // and are not exercised here — they have unit-test coverage in
    // `lint_impl.rs`:
    //   - missing-horizontal-whitespace-after-marker-name and
    //     missing-tag-end-delimiter-after-marker (canonical lexer
    //     normalizes `\c1` / `\pword` into a single Marker token)
    //   - invalid-number-range (only fires when the range value is
    //     wholly non-numeric, which the lexer typically rejects upstream)
    let expected = [
        LintCode::DuplicateIdMarker,
        LintCode::ContentBeforeFirstChapter,
        LintCode::VerseOutsideExplicitParagraph,
        LintCode::UnclosedMarker,
        LintCode::StrayCloseMarker,
        LintCode::UnknownMarker,
        LintCode::DuplicateVerseNumber,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::VerseInSectionOrOtherParagraph,
    ];

    let missing: Vec<LintCode> = expected.iter().copied().filter(|c| !observed.contains(c)).collect();
    assert!(
        missing.is_empty(),
        "common-errors.usfm failed to trigger: {missing:?}\nObserved: {observed:?}"
    );
}
