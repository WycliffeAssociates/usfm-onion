//! Warm lint-cache priming: the two stamps that license a caller's cached lint
//! contribution to replace a rule pass, and the typed, atomic rejections when
//! they — or the cached result itself — do not check out.
//!
//! `resident_lint.rs` already proves residency and stateless lint agree and
//! that `restore_corpus` seeds a warm corpus without lex/parse; this file
//! proves the layer on top: that a cached *lint* contribution is only ever
//! adopted when compared, never trusted, and that any one mismatch rejects
//! that one book's cache cleanly rather than partially.

use braid::{
    BookLintPrime, BookRestoreInput, Braid, BraidConfig, CorpusInput, CorpusRestoreInput,
    LintConfigFingerprint, LintEngineStamp, LintPrimeInput, PrimeRejectReason, PrimeRejection,
    SourceKey,
};
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

const GEN_SOURCE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth.\n";

fn braid() -> Braid {
    Braid::new(BraidConfig::new(LintOptions::scoped(LintScope::Book)), {
        let mut next = 0u32;
        move || {
            next += 1;
            format!("minted-{next}")
        }
    })
}

fn key(value: &str) -> SourceKey {
    SourceKey::new(value).expect("non-empty key")
}

fn id(value: &str) -> BookId {
    BookId::from_str(value).expect("three-character code")
}

fn owned(source: &str) -> Vec<OwnedToken> {
    parse(source)
        .tokens
        .iter()
        .map(OwnedToken::from_parsed)
        .collect()
}

fn matching_stamps() -> (LintConfigFingerprint, LintEngineStamp) {
    (
        LintConfigFingerprint::of(&LintOptions::scoped(LintScope::Book)),
        LintEngineStamp::current(),
    )
}

/// A resident, already-linted GEN book to harvest a real cached contribution
/// from — real ids (parsed, not synthetic), real fixes, real source hash.
fn linted_gen() -> Braid {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![braid::BookInput::Usfm {
            source_key: key("GEN.usfm"),
            book: id("GEN"),
            source: GEN_SOURCE.to_string(),
        }]))
        .expect("one book");
    resident.lint();
    resident
}

fn book_prime(resident: &mut Braid) -> BookLintPrime {
    let snapshot = resident.books();
    let book = snapshot[0].book;
    let source_hash = snapshot[0].source_hash;
    // `resident.lint()` above already computed and cached the result this
    // harvests — `patches()`/`lint()` read it back without recomputing.
    let result = resident.lint().books[0].result.clone();
    BookLintPrime {
        book,
        source_hash,
        result,
    }
}

#[test]
fn a_valid_cached_contribution_is_adopted_without_recompute() {
    let mut source_of_truth = linted_gen();
    let prime = book_prime(&mut source_of_truth);
    let expected_issue_codes: Vec<_> = prime.result.issues.iter().map(|i| i.code).collect();
    assert!(
        !expected_issue_codes.is_empty(),
        "fixture must actually produce findings, or this test proves nothing"
    );

    let (config_fingerprint, engine_stamp) = matching_stamps();
    let mut restored = braid();
    let report = restored
        .restore_corpus(CorpusRestoreInput::new(
            config_fingerprint,
            engine_stamp,
            vec![BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: braid::LineEnding::Lf,
                lint: Some(prime),
            }],
        ))
        .expect("well-formed seed");

    assert_eq!(report.seeded, vec![id("GEN")]);
    assert!(
        report.rejected.is_empty(),
        "a valid cached contribution must not appear in rejected: {:?}",
        report.rejected
    );
    assert!(
        restored.books_awaiting_lint().is_empty(),
        "an adopted cache must not leave the book dirty"
    );
    let adopted_codes: Vec<_> = restored.lint().books[0]
        .result
        .issues
        .iter()
        .map(|i| i.code)
        .collect();
    assert_eq!(adopted_codes, expected_issue_codes);
}

#[test]
fn a_source_hash_mismatch_seeds_the_book_but_rejects_its_cache() {
    let mut source_of_truth = linted_gen();
    let mut prime = book_prime(&mut source_of_truth);
    prime.source_hash = braid::SourceHash(prime.source_hash.0 ^ 1);

    let (config_fingerprint, engine_stamp) = matching_stamps();
    let mut restored = braid();
    let report = restored
        .restore_corpus(CorpusRestoreInput::new(
            config_fingerprint,
            engine_stamp,
            vec![BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: braid::LineEnding::Lf,
                lint: Some(prime),
            }],
        ))
        .expect("well-formed seed");

    // The book itself still seeds — only its cached lint is untrusted.
    assert_eq!(report.seeded, vec![id("GEN")]);
    assert_eq!(
        report.rejected,
        vec![PrimeRejection {
            book: id("GEN"),
            reason: PrimeRejectReason::SourceHashMismatch,
        }]
    );
    assert_eq!(restored.books_awaiting_lint(), vec![id("GEN")]);
}

#[test]
fn a_config_fingerprint_mismatch_rejects_every_books_cache_atomically() {
    let mut source_of_truth = linted_gen();
    let prime = book_prime(&mut source_of_truth);

    let wrong_fingerprint = LintConfigFingerprint(matching_stamps().0.0 ^ 1);
    let mut restored = braid();
    let report = restored
        .restore_corpus(CorpusRestoreInput::new(
            wrong_fingerprint,
            matching_stamps().1,
            vec![BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: braid::LineEnding::Lf,
                lint: Some(prime),
            }],
        ))
        .expect("well-formed seed");

    assert_eq!(report.seeded, vec![id("GEN")]);
    assert_eq!(
        report.rejected,
        vec![PrimeRejection {
            book: id("GEN"),
            reason: PrimeRejectReason::ConfigFingerprintMismatch,
        }]
    );
}

#[test]
fn an_engine_stamp_mismatch_rejects_the_cache() {
    let mut source_of_truth = linted_gen();
    let prime = book_prime(&mut source_of_truth);

    let wrong_engine = LintEngineStamp(matching_stamps().1.0 ^ 1);
    let mut restored = braid();
    let report = restored
        .restore_corpus(CorpusRestoreInput::new(
            matching_stamps().0,
            wrong_engine,
            vec![BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: braid::LineEnding::Lf,
                lint: Some(prime),
            }],
        ))
        .expect("well-formed seed");

    assert_eq!(
        report.rejected,
        vec![PrimeRejection {
            book: id("GEN"),
            reason: PrimeRejectReason::EngineStampMismatch,
        }]
    );
}

/// A restamped-hostile case: source hash and both stamps all agree, but the
/// cached result names a fix for a token id this book's stream does not
/// hold — a forged or corrupted cache that passed every cheap check. The
/// whole contribution is refused rather than adopting the findings with a
/// dropped or partial patch table.
#[test]
fn a_fix_that_does_not_resolve_against_this_books_tokens_is_refused_as_invalid_patch() {
    let mut source_of_truth = linted_gen();
    let mut prime = book_prime(&mut source_of_truth);
    let hostile_issue = prime
        .result
        .issues
        .iter_mut()
        .find(|issue| issue.fix.is_some())
        .expect("fixture must carry at least one fix, or this test proves nothing");
    match hostile_issue.fix.as_mut().unwrap() {
        usfm_onion::lint::TokenFix::ReplaceToken {
            target_token_id, ..
        }
        | usfm_onion::lint::TokenFix::DeleteToken {
            target_token_id, ..
        }
        | usfm_onion::lint::TokenFix::InsertAfter {
            target_token_id, ..
        } => *target_token_id = "does-not-exist".to_string(),
    }

    let (config_fingerprint, engine_stamp) = matching_stamps();
    let mut restored = braid();
    let report = restored
        .restore_corpus(CorpusRestoreInput::new(
            config_fingerprint,
            engine_stamp,
            vec![BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: braid::LineEnding::Lf,
                lint: Some(prime),
            }],
        ))
        .expect("well-formed seed");

    assert_eq!(report.seeded, vec![id("GEN")]);
    assert_eq!(
        report.rejected,
        vec![PrimeRejection {
            book: id("GEN"),
            reason: PrimeRejectReason::InvalidPatch,
        }]
    );
    assert_eq!(restored.books_awaiting_lint(), vec![id("GEN")]);
}

#[test]
fn prime_lint_cache_accepts_a_valid_contribution_on_an_already_resident_book() {
    let mut source_of_truth = linted_gen();
    let prime = book_prime(&mut source_of_truth);

    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![braid::BookInput::Usfm {
            source_key: key("GEN.usfm"),
            book: id("GEN"),
            source: GEN_SOURCE.to_string(),
        }]))
        .expect("one book");
    assert_eq!(resident.books_awaiting_lint(), vec![id("GEN")]);

    let (config_fingerprint, engine_stamp) = matching_stamps();
    let report = resident.prime_lint_cache(LintPrimeInput {
        config_fingerprint,
        engine_stamp,
        books: vec![prime],
    });

    assert_eq!(report.accepted, vec![id("GEN")]);
    assert!(report.rejected.is_empty());
    assert!(resident.books_awaiting_lint().is_empty());
}

#[test]
fn prime_lint_cache_rejects_a_book_that_is_not_resident() {
    let mut source_of_truth = linted_gen();
    let prime = book_prime(&mut source_of_truth);

    let mut empty = braid();
    let (config_fingerprint, engine_stamp) = matching_stamps();
    let report = empty.prime_lint_cache(LintPrimeInput {
        config_fingerprint,
        engine_stamp,
        books: vec![prime],
    });

    assert!(report.accepted.is_empty());
    assert_eq!(
        report.rejected,
        vec![PrimeRejection {
            book: id("GEN"),
            reason: PrimeRejectReason::BookNotResident,
        }]
    );
}
