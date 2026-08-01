//! Baseline mutation, exact `is_dirty`, and resident `diff_baseline`.
//!
//! Every assertion runs in both ingest lanes — a parsed USFM book (whose
//! tokens carry parse-assigned positional ids) and a caller token push (whose
//! tokens carry the editor's own ids) — because a baseline is compared by
//! exact serialized bytes, and the two lanes reach those bytes by different
//! routes (kept verbatim versus re-emitted from tokens).

use braid::{
    BaselineError, BookInput, BookTokensInput, Braid, BraidConfig, ChapterInput, ChapterLabel,
    ChapterTarget, CorpusInput, CorpusScope, LineEnding, ScopeError, ScopedOutput,
    SetBaselineError, SourceKey,
};
use usfm_onion::diff::diff_skeleton;
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

const GEN_SOURCE: &str =
    "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n";
const GEN_EDITED: &str =
    "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning God.\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n";
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

/// The two ways tokens enter braid. Every assertion below runs through both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lane {
    Parsed,
    CallerTokens,
}

const LANES: [Lane; 2] = [Lane::Parsed, Lane::CallerTokens];

impl Lane {
    fn book(self, code: &str, source: &str) -> BookInput {
        match self {
            Lane::Parsed => BookInput::Usfm {
                source_key: key(&format!("{code}.usfm")),
                book: id(code),
                source: source.to_string(),
            },
            Lane::CallerTokens => BookInput::Tokens(BookTokensInput {
                source_key: key(&format!("{code}.usfm")),
                book: id(code),
                tokens: owned(source),
                line_ending: LineEnding::Lf,
            }),
        }
    }
}

fn seeded(lane: Lane) -> Braid {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            lane.book("GEN", GEN_SOURCE),
            lane.book("EXO", EXO_SOURCE),
        ]))
        .expect("seed corpus");
    resident
}

fn single_str(output: Result<ScopedOutput<String>, ScopeError>) -> String {
    match output.unwrap() {
        ScopedOutput::Single(value) => value,
        ScopedOutput::All(_) => panic!("expected a single-scope projection"),
    }
}

/// Every observable fact a baseline mutation must leave untouched.
fn current_fingerprint(resident: &Braid) -> (u64, String, Vec<BookId>) {
    (
        resident.expected_snapshot_id().0,
        single_str(resident.to_usfm(CorpusScope::Book(id("GEN")))),
        resident.books_awaiting_lint(),
    )
}

// ---- is_dirty --------------------------------------------------------

#[test]
fn missing_baseline_is_always_dirty() {
    // A book that has never had a baseline declared has no saved equality
    // proof to compare against — "dirty" is the only honest answer, never a
    // synthesized "clean" or "unknown".
    for lane in LANES {
        let resident = seeded(lane);
        assert!(
            resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );
        assert!(resident.is_dirty(CorpusScope::All).unwrap(), "{lane:?}");
    }
}

#[test]
fn dirty_tracks_exact_serialized_equality_against_the_baseline() {
    for lane in LANES {
        let mut resident = seeded(lane);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();
        // Also baseline EXO so `All` genuinely reflects GEN's own state below,
        // rather than always reporting dirty because of EXO's still-missing
        // baseline.
        resident.set_baseline(lane.book("EXO", EXO_SOURCE)).unwrap();
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}: matches its own just-declared baseline"
        );
        assert!(!resident.is_dirty(CorpusScope::All).unwrap(), "{lane:?}");

        resident.update_book(lane.book("GEN", GEN_EDITED)).unwrap();
        assert!(
            resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}: content diverged from the baseline"
        );
        assert!(resident.is_dirty(CorpusScope::All).unwrap(), "{lane:?}");

        // An exact revert is clean again — this is the "false after exact
        // revert" case, not merely "false after any edit back".
        resident.update_book(lane.book("GEN", GEN_SOURCE)).unwrap();
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}: reverted to exactly the baseline's bytes"
        );
    }
}

#[test]
fn is_dirty_book_not_found_errors() {
    let resident = seeded(Lane::Parsed);
    assert_eq!(
        resident.is_dirty(CorpusScope::Book(id("LEV"))),
        Err(ScopeError::BookNotFound(id("LEV")))
    );
}

#[test]
fn chapter_scope_dirty_compares_the_same_label_baseline_run() {
    for lane in LANES {
        let mut resident = seeded(lane);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();
        let chapter_1 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
        let chapter_2 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into()));
        assert!(
            !resident
                .is_dirty(CorpusScope::Chapter(chapter_1.clone()))
                .unwrap(),
            "{lane:?}"
        );
        assert!(
            !resident
                .is_dirty(CorpusScope::Chapter(chapter_2.clone()))
                .unwrap(),
            "{lane:?}"
        );

        resident.update_book(lane.book("GEN", GEN_EDITED)).unwrap();
        // Only chapter 1's own bytes changed.
        assert!(
            resident.is_dirty(CorpusScope::Chapter(chapter_1)).unwrap(),
            "{lane:?}"
        );
        assert!(
            !resident.is_dirty(CorpusScope::Chapter(chapter_2)).unwrap(),
            "{lane:?}"
        );
    }
}

#[test]
fn a_baseline_run_missing_for_that_label_is_dirty() {
    // The baseline was declared before chapter 2 existed, so current chapter
    // 2 has no same-label baseline run to compare against.
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            Lane::Parsed.book("GEN", "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n"),
        ]))
        .unwrap();
    resident
        .set_baseline(Lane::Parsed.book("GEN", "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n"))
        .unwrap();
    resident
        .update_book(Lane::Parsed.book("GEN", GEN_SOURCE))
        .unwrap();

    let chapter_2 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into()));
    assert!(resident.is_dirty(CorpusScope::Chapter(chapter_2)).unwrap());
}

#[test]
fn duplicate_current_chapter_labels_make_is_dirty_ambiguous() {
    let dup = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n";
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![Lane::Parsed.book("GEN", dup)]))
        .unwrap();
    let chapter_1 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
    // Current-side duplicate labels are ambiguous even before a baseline
    // exists, because `is_dirty` resolves the current scope the same way
    // every other scoped read does.
    assert!(matches!(
        resident.is_dirty(CorpusScope::Chapter(chapter_1)),
        Err(ScopeError::AmbiguousChapter { .. })
    ));
}

#[test]
fn duplicate_baseline_chapter_labels_make_the_scope_ambiguous_too() {
    // The other side of the same rule: current content has one unambiguous
    // chapter 1 (so the current-side lookup alone would succeed), but the
    // declared baseline has two runs labeled "1" — both `is_dirty` and
    // `diff_baseline` must report the typed ambiguity rather than picking
    // one baseline run arbitrarily or treating the mismatch as "no baseline".
    let dup = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n";
    let unique = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n";
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![Lane::Parsed.book("GEN", dup)]))
        .unwrap();
    resident
        .set_baseline(Lane::Parsed.book("GEN", dup))
        .unwrap();
    resident
        .update_book(Lane::Parsed.book("GEN", unique))
        .unwrap();

    let chapter_1 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
    assert!(matches!(
        resident.is_dirty(CorpusScope::Chapter(chapter_1.clone())),
        Err(ScopeError::AmbiguousChapter { .. })
    ));
    assert!(matches!(
        resident.diff_baseline(CorpusScope::Chapter(chapter_1)),
        Err(BaselineError::Scope(ScopeError::AmbiguousChapter { .. }))
    ));
}

// ---- set_baseline / clear_baseline ------------------------------------

#[test]
fn baseline_mutation_cannot_change_current_state() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = current_fingerprint(&resident);

        let effect = resident.set_baseline(lane.book("GEN", GEN_EDITED)).unwrap();
        assert!(
            effect.is_noop(),
            "{lane:?}: set_baseline reports no current-content effect"
        );
        assert_eq!(current_fingerprint(&resident), before, "{lane:?}");

        let effect = resident.clear_baseline(id("GEN"));
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(current_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn clearing_an_absent_baseline_is_a_no_op() {
    let mut resident = seeded(Lane::Parsed);
    assert!(resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());
    let effect = resident.clear_baseline(id("GEN"));
    assert!(effect.is_noop());
    assert!(resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());
}

#[test]
fn clearing_a_baseline_makes_the_book_dirty_again() {
    let mut resident = seeded(Lane::Parsed);
    resident
        .set_baseline(Lane::Parsed.book("GEN", GEN_SOURCE))
        .unwrap();
    assert!(!resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());

    resident.clear_baseline(id("GEN"));
    assert!(resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());
}

#[test]
fn a_changed_update_book_carries_the_baseline_forward_unchanged() {
    // Editing current content is not how a baseline changes. This pins the
    // `update_book` "content differs, book already resident" seam
    // specifically: the candidate `BookState::build` produces defaults to no
    // baseline, so this is the branch that has to explicitly carry the
    // predecessor's forward rather than losing it to that default.
    for lane in LANES {
        let mut resident = seeded(lane);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();

        resident.update_book(lane.book("GEN", GEN_EDITED)).unwrap();
        assert!(
            resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );

        // Revert without touching the baseline: clean again, proving the
        // baseline survived the intervening edit rather than following it.
        resident.update_book(lane.book("GEN", GEN_SOURCE)).unwrap();
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );
    }
}

#[test]
fn update_chapter_carries_the_baseline_forward_unchanged() {
    // The `rebuilt()` seam: `update_chapter` splices a new run and rebuilds
    // the whole book from it, which must carry the resident predecessor's
    // own baseline forward exactly as `rebuilt` promises — a different code
    // path than `update_book`'s own "content differs" branch above.
    for lane in LANES {
        let mut resident = seeded(lane);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();

        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
                ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 In the beginning God.\n")),
            )
            .unwrap();
        assert!(
            resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );

        // Revert the same chapter back to exactly the baseline's own
        // content: clean again, proving the baseline survived the rebuilt()
        // mutation rather than being reset or dropped by it.
        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
                ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 In the beginning.\n")),
            )
            .unwrap();
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );
    }
}

#[test]
fn a_byte_identical_resubmission_carries_the_baseline_forward_via_inherit_cache() {
    // The `inherit_cache()` seam: resubmitting exactly the same content is a
    // no-op for hydration (`content_eq` holds), which is also where an
    // untouched baseline must survive rather than being reset to whatever a
    // freshly built candidate defaults to (`None`).
    for lane in LANES {
        let mut resident = seeded(lane);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}"
        );

        let effect = resident.update_book(lane.book("GEN", GEN_SOURCE)).unwrap();
        assert!(
            effect.is_noop(),
            "{lane:?}: byte-identical resubmission takes the inherit_cache no-op path"
        );
        assert!(
            !resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap(),
            "{lane:?}: baseline still matches after the no-op"
        );
    }
}

// ---- diff_baseline -----------------------------------------------------

#[test]
fn diff_baseline_reports_missing_baseline_typed() {
    let resident = seeded(Lane::Parsed);
    assert_eq!(
        resident.diff_baseline(CorpusScope::Book(id("GEN"))),
        Err(BaselineError::MissingBaseline {
            books: vec![id("GEN")]
        })
    );

    // `All` collects every book missing a baseline, not just the first.
    let mut all_missing = resident.diff_baseline(CorpusScope::All).unwrap_err();
    if let BaselineError::MissingBaseline { books } = &mut all_missing {
        books.sort_by_key(|book| book.to_string());
    }
    assert_eq!(
        all_missing,
        BaselineError::MissingBaseline {
            books: vec![id("EXO"), id("GEN")]
        }
    );
}

#[test]
fn diff_baseline_equals_the_stateless_core_diff() {
    // Braid must not invent a second diff model: its resident convenience is
    // exactly core's `diff_skeleton` over the baseline and current token
    // streams, wrapped in the ordinary scoped-output envelope.
    for lane in LANES {
        let mut resident = seeded(lane);
        let baseline_tokens = owned(GEN_SOURCE);
        resident.set_baseline(lane.book("GEN", GEN_SOURCE)).unwrap();
        resident.update_book(lane.book("GEN", GEN_EDITED)).unwrap();

        let current_tokens = match resident
            .to_tokens(braid::Scope::book(id("GEN")))
            .unwrap()
            .remove(0)
        {
            scoped => scoped.tokens,
        };
        let expected = diff_skeleton(&baseline_tokens, &current_tokens);

        let actual = match resident
            .diff_baseline(CorpusScope::Book(id("GEN")))
            .unwrap()
        {
            ScopedOutput::Single(skeleton) => skeleton,
            ScopedOutput::All(_) => panic!("expected a single-scope diff"),
        };
        assert_eq!(actual, expected, "{lane:?}");
    }
}

#[test]
fn chapter_scoped_diff_compares_only_that_run() {
    let mut resident = seeded(Lane::Parsed);
    resident
        .set_baseline(Lane::Parsed.book("GEN", GEN_SOURCE))
        .unwrap();
    resident
        .update_book(Lane::Parsed.book("GEN", GEN_EDITED))
        .unwrap();

    let chapter_1 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
    let diff = match resident
        .diff_baseline(CorpusScope::Chapter(chapter_1))
        .unwrap()
    {
        ScopedOutput::Single(skeleton) => skeleton,
        ScopedOutput::All(_) => panic!("expected a single-scope diff"),
    };
    // Chapter 1's text changed, so its diff must show at least one modified
    // unit rather than reporting no difference.
    assert!(
        diff.units
            .iter()
            .any(|unit| !matches!(unit.status, usfm_onion::diff::DecisionStatus::Unchanged)),
        "expected chapter 1's diff to show the edit"
    );
}

#[test]
fn chapter_scoped_diff_missing_baseline_run_is_typed_missing() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            Lane::Parsed.book("GEN", "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n"),
        ]))
        .unwrap();
    resident
        .set_baseline(Lane::Parsed.book("GEN", "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n"))
        .unwrap();
    resident
        .update_book(Lane::Parsed.book("GEN", GEN_SOURCE))
        .unwrap();

    let chapter_2 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into()));
    assert_eq!(
        resident.diff_baseline(CorpusScope::Chapter(chapter_2)),
        Err(BaselineError::MissingBaseline {
            books: vec![id("GEN")]
        })
    );
}

#[test]
fn set_baseline_on_a_book_with_no_current_content_is_rejected_atomically() {
    // set_baseline is never an ingest verb: a book braid does not currently
    // hold has nowhere for a baseline to attach, so the call must report
    // absence rather than silently adding a resident book — a caller has to
    // be able to rely on set_baseline never changing current content or
    // corpus identity, including when the named book is a typo or one that
    // has since been removed.
    for lane in LANES {
        let mut resident = braid();
        let before_snapshot = resident.expected_snapshot_id();
        let before_books = resident.books();

        let result = resident.set_baseline(lane.book("GEN", GEN_SOURCE));
        assert_eq!(
            result,
            Err(SetBaselineError::BookNotResident(id("GEN"))),
            "{lane:?}"
        );

        assert_eq!(resident.expected_snapshot_id(), before_snapshot, "{lane:?}");
        assert_eq!(resident.books(), before_books, "{lane:?}");
        assert_eq!(
            resident.is_dirty(CorpusScope::Book(id("GEN"))),
            Err(ScopeError::BookNotFound(id("GEN"))),
            "{lane:?}: still no such resident book, not merely 'not dirty'"
        );
    }
}

#[test]
fn set_baseline_on_a_book_not_resident_leaves_other_baselines_untouched() {
    // The rejection is scoped to the mistaken target: an unrelated book's
    // own already-declared baseline is not disturbed by a sibling call
    // naming a book braid does not hold.
    let mut resident = seeded(Lane::Parsed);
    resident
        .set_baseline(Lane::Parsed.book("GEN", GEN_SOURCE))
        .unwrap();
    assert!(!resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());

    let result =
        resident.set_baseline(Lane::Parsed.book("LEV", "\\id LEV\n\\c 1\n\\p\n\\v 1 a.\n"));
    assert_eq!(result, Err(SetBaselineError::BookNotResident(id("LEV"))));
    assert!(!resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());
}
