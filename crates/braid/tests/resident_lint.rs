//! Resident lint and the complete semantic snapshot.
//!
//! The property every test here defends is that residency changes *nothing*
//! about what core lint says. Braid decides when rules run and caches what they
//! produced; it never filters, reorders, merges, or re-renders a finding. The
//! one deliberate difference is that resident tokens carry no byte spans (that
//! is what lets them outlive their source), so their findings carry no spans
//! either — canonical order keys on token position, which an owned stream has
//! exactly as much as a parsed one.

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, IngestError, LineEnding, LintConfigFingerprint, LintEngineStamp, SourceKey,
};
use usfm_onion::lint::{LintCode, LintIssue, LintOptions, LintScope, lint_tokens};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

/// Two whitespace-fix findings and a `\p`-less verse, so the corpus under test
/// has findings of several codes rather than one.
const GEN_SOURCE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth.\n";
const EXO_SOURCE: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

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

fn usfm(code: &str, source: &str) -> BookInput {
    BookInput::Usfm {
        source_key: key(&format!("{code}.usfm")),
        book: id(code),
        source: source.to_string(),
    }
}

/// The two stamps `braid()`'s own config actually matches — the restore
/// tests below aren't exercising lint-cache priming (that's Phase D's own
/// suite), just plain token/source seeding, so any restore seed needs a
/// pair that passes rather than a placeholder that would reject every book's
/// (here always-absent) `lint` field for the wrong reason.
fn matching_stamps() -> (LintConfigFingerprint, LintEngineStamp) {
    (
        LintConfigFingerprint::of(&LintOptions::scoped(LintScope::Book)),
        LintEngineStamp::current(),
    )
}

fn tokens_input(code: &str, source: &str) -> BookInput {
    BookInput::Tokens(BookTokensInput {
        source_key: key(&format!("{code}.usfm")),
        book: id(code),
        tokens: owned(source),
        line_ending: LineEnding::Lf,
    })
}

/// What core would say about the same tokens, with the same declared book.
fn stateless(book: &str, tokens: &[OwnedToken]) -> Vec<LintIssue> {
    let mut options = LintOptions::scoped(LintScope::Book);
    options.declared_book = Some(id(book));
    lint_tokens(tokens, options).issues
}

#[test]
fn resident_lint_equals_stateless_core_lint_in_content_and_order() {
    for lane in [usfm as fn(&str, &str) -> BookInput, tokens_input] {
        let mut resident = braid();
        resident
            .replace_corpus(CorpusInput::new(vec![
                lane("GEN", GEN_SOURCE),
                lane("EXO", EXO_SOURCE),
            ]))
            .expect("two distinct books");

        let snapshot = resident.lint();
        assert_eq!(snapshot.books.len(), 2);
        // Corpus order, not book-code order: the caller's order is the contract.
        assert_eq!(snapshot.books[0].book, id("GEN"));
        assert_eq!(snapshot.books[1].book, id("EXO"));
        assert!(!snapshot.books[0].result.issues.is_empty());

        for book in &snapshot.books {
            let expected = stateless(book.book.as_str(), book.tokens);
            assert_eq!(
                book.result.issues, expected,
                "resident lint diverged from core for {}",
                book.book
            );
        }

        // The corpus summary is the sum of the per-book ones, so a consumer
        // reading only the summary counts the same findings the books carry.
        let total: usize = snapshot
            .books
            .iter()
            .map(|book| book.result.summary.total_count)
            .sum();
        assert_eq!(snapshot.summary.total_count, total);
        assert_eq!(
            snapshot.summary.total_count,
            snapshot
                .books
                .iter()
                .map(|book| book.result.issues.len())
                .sum::<usize>()
        );
    }
}

/// The declared book reaches core's lint context, which is the whole reason the
/// C1 seam exists: the same bytes report `book-id-mismatch` when the caller
/// declared them as a different (valid) book, and report nothing when it did not.
#[test]
fn the_declared_book_reaches_core_and_fires_the_mismatch_rule() {
    let source = "\\id 2CO\n\\c 1\n\\p\n\\v 1 Paul.\n";

    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![usfm("1CO", source)]))
        .expect("one book");
    let snapshot = resident.lint();
    let mismatch = snapshot.books[0]
        .result
        .issues
        .iter()
        .find(|issue| issue.code == LintCode::BookIdMismatch)
        .expect("a source declared as 1CO whose own id says 2CO");
    assert_eq!(
        mismatch.message_params.get("expected").map(String::as_str),
        Some("1CO")
    );
    assert_eq!(
        mismatch.message_params.get("found").map(String::as_str),
        Some("2CO")
    );

    let mut honest = braid();
    honest
        .replace_corpus(CorpusInput::new(vec![usfm("2CO", source)]))
        .expect("one book");
    assert!(
        !honest.lint().books[0]
            .result
            .issues
            .iter()
            .any(|issue| issue.code == LintCode::BookIdMismatch)
    );
}

/// A no-op mutation must leave both the cache and the publication alone: the
/// snapshot id is identity over source bytes, and dirtiness is what decides
/// whether rules run. A dropped cache would show up as a dirty book, since that
/// is precisely the state a book is in when it needs recompute.
#[test]
fn no_ops_preserve_the_cache_and_the_snapshot_id() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN_SOURCE)]))
        .expect("one book");
    let findings = resident.lint().books[0].result.clone();
    let id_after_lint = resident.expected_snapshot_id();
    assert!(resident.books_awaiting_lint().is_empty());

    // Same content, new source key: a rebinding, not an edit.
    let rebound = resident
        .update_book(BookInput::Usfm {
            source_key: key("moved/GEN.usfm"),
            book: id("GEN"),
            source: GEN_SOURCE.to_string(),
        })
        .expect("a content-identical replacement");
    assert!(rebound.is_noop());
    assert_eq!(resident.expected_snapshot_id(), id_after_lint);
    assert!(
        resident.books_awaiting_lint().is_empty(),
        "a no-op must not schedule rule work"
    );
    assert_eq!(resident.lint().books[0].result, &findings);

    // The same push twice coalesces: the second one has nothing left to change.
    let target = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
    let chapter = resident
        .to_tokens(braid::Scope::chapter(
            id("GEN"),
            ChapterLabel::Number("1".into()),
        ))
        .expect("chapter one")
        .remove(0)
        .tokens;
    let first = resident
        .update_chapter(target.clone(), ChapterInput::Tokens(chapter.clone()))
        .expect("push");
    let second = resident
        .update_chapter(target, ChapterInput::Tokens(chapter))
        .expect("push again");
    assert!(first.is_noop() && second.is_noop());
    assert_eq!(resident.expected_snapshot_id(), id_after_lint);
    assert!(resident.books_awaiting_lint().is_empty());
}

/// A configuration change invalidates every cached result — and recompute is
/// driven by that derived stamp rather than a drained queue, so linting twice
/// after it produces the identical complete snapshot both times.
///
/// This is the observable form of "retry after a failed lint is safe": braid's
/// `lint` is infallible (core rule execution over resident tokens has no failure
/// mode), so what can be tested is the property that would make a retry safe if
/// it did — nothing is consumed, and a book with no cached result is recomputed
/// on the next call rather than published empty.
#[test]
fn a_config_change_invalidates_caches_and_recompute_is_repeatable() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            usfm("GEN", GEN_SOURCE),
            usfm("EXO", EXO_SOURCE),
        ]))
        .expect("two books");
    let first: Vec<usize> = resident
        .lint()
        .books
        .iter()
        .map(|book| book.result.issues.len())
        .collect();
    assert!(resident.books_awaiting_lint().is_empty());

    let mut options = LintOptions::scoped(LintScope::Book);
    options.allow_implicit_chapter_content_verse = !options.allow_implicit_chapter_content_verse;
    let effect = resident.update_config(BraidConfig::new(options));
    // No tokens were rewritten, so identity and hydration are untouched — only
    // staleness moved.
    assert!(effect.is_noop());
    assert_eq!(resident.books_awaiting_lint(), vec![id("GEN"), id("EXO")]);
    assert!(
        resident.patches().is_empty(),
        "patches resolved under the old configuration must stop being addressable"
    );

    let second: Vec<usize> = resident
        .lint()
        .books
        .iter()
        .map(|book| book.result.issues.len())
        .collect();
    let third: Vec<usize> = resident
        .lint()
        .books
        .iter()
        .map(|book| book.result.issues.len())
        .collect();
    assert_eq!(second, third, "recompute is repeatable");
    assert_eq!(first, second, "this option changes nothing for these books");
    assert!(resident.books_awaiting_lint().is_empty());
}

/// Exactly the dirty books recompute. A one-chapter edit makes its own book
/// dirty and leaves every other book's cached contribution alone.
#[test]
fn one_edit_dirties_one_book() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            usfm("GEN", GEN_SOURCE),
            usfm("EXO", EXO_SOURCE),
        ]))
        .expect("two books");
    let exo_before = resident.lint().books[1].result.clone();

    let tokens = owned("\\c 1\n\\p\n\\v 1 In the beginning, edited.\n");
    let effect = resident
        .update_chapter(
            ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
            ChapterInput::Tokens(tokens),
        )
        .expect("a real chapter edit");
    assert!(!effect.is_noop());
    assert_eq!(resident.books_awaiting_lint(), vec![id("GEN")]);

    let snapshot = resident.lint();
    assert_eq!(
        snapshot.books[1].result, &exo_before,
        "an untouched book keeps its exact cached contribution"
    );
}

#[test]
fn an_empty_corpus_has_a_valid_empty_snapshot() {
    let mut resident = braid();
    let snapshot = resident.lint();
    assert!(snapshot.books.is_empty());
    assert_eq!(snapshot.summary.total_count, 0);
    // The empty corpus's identity, which is a real value rather than a
    // sentinel: an empty braid publishes the same id every time.
    assert_eq!(snapshot.id, braid().expected_snapshot_id());
}

/// The warm cold-open: seeding from already-decoded state must land a corpus
/// indistinguishable from one that was parsed, without parsing anything.
#[test]
fn restore_seeds_a_corpus_that_matches_the_parsed_one() {
    let mut parsed = braid();
    parsed
        .replace_corpus(CorpusInput::new(vec![
            usfm("GEN", GEN_SOURCE),
            usfm("EXO", EXO_SOURCE),
        ]))
        .expect("two books");
    let expected_books = parsed.books();
    let expected_findings: Vec<Vec<usfm_onion::lint::LintIssue>> = parsed
        .lint()
        .books
        .iter()
        .map(|book| book.result.issues.clone())
        .collect();

    let mut restored = braid();
    let report = restored
        .restore_corpus(braid::CorpusRestoreInput::new(
            matching_stamps().0,
            matching_stamps().1,
            [("GEN", GEN_SOURCE), ("EXO", EXO_SOURCE)]
                .into_iter()
                .map(|(code, source)| braid::BookRestoreInput {
                    source_key: key(&format!("{code}.usfm")),
                    book: id(code),
                    source: source.to_string(),
                    tokens: owned(source),
                    line_ending: LineEnding::Lf,
                    lint: None,
                })
                .collect(),
        ))
        .expect("a well-formed seed");
    assert_eq!(report.seeded, vec![id("GEN"), id("EXO")]);
    assert!(report.rejected.is_empty());

    // Same identity, same hashes, same order, same findings.
    assert_eq!(restored.books(), expected_books);
    assert_eq!(
        restored.expected_snapshot_id(),
        parsed.expected_snapshot_id()
    );
    assert_eq!(restored.books_awaiting_lint(), vec![id("GEN"), id("EXO")]);
    let restored_findings: Vec<Vec<usfm_onion::lint::LintIssue>> = restored
        .lint()
        .books
        .iter()
        .map(|book| book.result.issues.clone())
        .collect();
    assert_eq!(restored_findings, expected_findings);
}

/// A seed whose tokens do not spell its own bytes is refused as data — that one
/// book falls back to ordinary ingest while the rest of the corpus stays warm.
#[test]
fn a_seed_whose_tokens_disagree_with_its_bytes_is_refused() {
    let mut restored = braid();
    let report = restored
        .restore_corpus(braid::CorpusRestoreInput::new(
            matching_stamps().0,
            matching_stamps().1,
            vec![
                braid::BookRestoreInput {
                    source_key: key("GEN.usfm"),
                    book: id("GEN"),
                    source: GEN_SOURCE.to_string(),
                    tokens: owned(GEN_SOURCE),
                    line_ending: LineEnding::Lf,
                    lint: None,
                },
                braid::BookRestoreInput {
                    source_key: key("EXO.usfm"),
                    book: id("EXO"),
                    // The bytes claim one thing, the tokens spell another.
                    source: EXO_SOURCE.to_string(),
                    tokens: owned("\\id EXO\n\\c 1\n\\p\n\\v 1 Something else entirely.\n"),
                    line_ending: LineEnding::Lf,
                    lint: None,
                },
            ],
        ))
        .expect("the corpus itself is well-formed");

    assert_eq!(report.seeded, vec![id("GEN")]);
    assert_eq!(
        report.rejected,
        vec![braid::PrimeRejection {
            book: id("EXO"),
            reason: braid::PrimeRejectReason::SourceTokenMismatch,
        }]
    );
    assert_eq!(
        restored.books().len(),
        1,
        "a refused book is not resident at all"
    );
}

/// A manifest naming one book twice is one caller mistake about the manifest, and
/// the answer is to refuse the whole call — even when one of the two records
/// would have been rejected for its own reasons. The reviewer's repro: a valid
/// GEN record and a mismatched GEN record must not seed one and report the other.
#[test]
fn a_duplicate_book_in_the_seed_manifest_refuses_the_whole_call() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![usfm("EXO", EXO_SOURCE)]))
        .expect("a book to be left untouched");
    let identity = resident.expected_snapshot_id();
    let books = resident.books();

    let seed = braid::CorpusRestoreInput::new(
        matching_stamps().0,
        matching_stamps().1,
        vec![
            braid::BookRestoreInput {
                source_key: key("GEN.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: LineEnding::Lf,
                lint: None,
            },
            braid::BookRestoreInput {
                source_key: key("copy/GEN.usfm"),
                book: id("GEN"),
                // Would be a content rejection on its own — which must not be how a
                // duplicate declaration gets reported.
                source: GEN_SOURCE.to_string(),
                tokens: owned(EXO_SOURCE),
                line_ending: LineEnding::Lf,
                lint: None,
            },
        ],
    );
    assert_eq!(
        resident.restore_corpus(seed),
        Err(IngestError::DuplicateBook {
            book: id("GEN"),
            sources: vec![key("GEN.usfm"), key("copy/GEN.usfm")],
        })
    );
    assert_eq!(resident.expected_snapshot_id(), identity);
    assert_eq!(resident.books(), books);
}

/// Same rule for the other key, and again with one of the two records carrying a
/// content mismatch: the source key is what binds a book to where it came from,
/// so two records claiming one binding is equally unanswerable.
#[test]
fn a_duplicate_source_key_in_the_seed_manifest_refuses_the_whole_call() {
    let mut resident = braid();
    let seed = braid::CorpusRestoreInput::new(
        matching_stamps().0,
        matching_stamps().1,
        vec![
            braid::BookRestoreInput {
                source_key: key("shared.usfm"),
                book: id("GEN"),
                source: GEN_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: LineEnding::Lf,
                lint: None,
            },
            braid::BookRestoreInput {
                source_key: key("shared.usfm"),
                book: id("EXO"),
                source: EXO_SOURCE.to_string(),
                tokens: owned(GEN_SOURCE),
                line_ending: LineEnding::Lf,
                lint: None,
            },
        ],
    );
    assert_eq!(
        resident.restore_corpus(seed),
        Err(IngestError::DuplicateSourceKey {
            source: key("shared.usfm"),
        })
    );
    assert!(resident.books().is_empty());
}
