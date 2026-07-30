//! Resident lifecycle behavior: validation, atomicity, exact effects, chapter
//! runs, and the pull primitive.
//!
//! Every mutation and pull assertion runs in both ingest lanes — a parsed USFM
//! book (`BookInput::Usfm`, whose tokens carry parse-assigned positional ids and
//! remembered attribute positions) and a caller token push
//! (`BookInput::Tokens`, whose tokens are the ones the caller holds and whose
//! bytes are derived by re-emission). The two lanes take different code paths
//! through candidate construction, hashing, and serialization, and only the
//! second one is what an editor actually calls.

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusScope, IngestError, LineEnding, Scope, ScopeError, ScopedOutput, SourceHash,
    SourceKey,
};
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

const GEN_SOURCE: &str = "\\id GEN\n\\h Genesis\n\\c 1\n\\p\n\\v 1 In the beginning.\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n";
const EXO_SOURCE: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

fn braid() -> Braid {
    Braid::new(BraidConfig::new(LintOptions::scoped(LintScope::Book)))
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

/// The two ways tokens enter braid. Every lifecycle assertion runs through both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lane {
    Parsed,
    CallerTokens,
}

const LANES: [Lane; 2] = [Lane::Parsed, Lane::CallerTokens];

impl Lane {
    fn book(self, code: &str, source_key: &str, source: &str) -> BookInput {
        match self {
            Lane::Parsed => BookInput::Usfm {
                source_key: key(source_key),
                book: id(code),
                source: source.to_string(),
            },
            Lane::CallerTokens => BookInput::Tokens(BookTokensInput {
                source_key: key(source_key),
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
            lane.book("GEN", "01-GEN.usfm", GEN_SOURCE),
            lane.book("EXO", "02-EXO.usfm", EXO_SOURCE),
        ]))
        .expect("seed corpus");
    resident
}

fn single(output: ScopedOutput<String>) -> String {
    match output {
        ScopedOutput::Single(value) => value,
        ScopedOutput::All(_) => panic!("expected a single-scope projection"),
    }
}

/// Every observable fact a rejected mutation must leave untouched.
fn state_fingerprint(resident: &Braid) -> (u64, Vec<(String, BookId, SourceHash)>, Vec<BookId>) {
    (
        resident.expected_snapshot_id().0,
        resident
            .books()
            .into_iter()
            .map(|entry| {
                (
                    entry.source_key.as_str().to_string(),
                    entry.book,
                    entry.source_hash,
                )
            })
            .collect(),
        resident.dirty_books(),
    )
}

#[test]
fn both_lanes_agree_on_bytes_hashes_and_identity() {
    // The lanes reach the same resident state by different routes: one keeps the
    // supplied bytes, the other derives them from tokens. If they disagreed,
    // every downstream hash, snapshot id, and cache stamp would fork by lane.
    let parsed = seeded(Lane::Parsed);
    let pushed = seeded(Lane::CallerTokens);

    assert_eq!(
        single(parsed.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        GEN_SOURCE
    );
    assert_eq!(
        single(pushed.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        GEN_SOURCE
    );
    assert_eq!(parsed.expected_snapshot_id(), pushed.expected_snapshot_id());
    assert_eq!(
        parsed.books().first().unwrap().source_hash,
        pushed.books().first().unwrap().source_hash
    );
}

#[test]
fn corpus_order_and_chapter_runs_are_source_faithful() {
    for lane in LANES {
        let resident = seeded(lane);
        assert_eq!(
            resident
                .books()
                .into_iter()
                .map(|entry| entry.book)
                .collect::<Vec<_>>(),
            vec![id("GEN"), id("EXO")],
            "{lane:?}"
        );
        assert_eq!(
            resident.chapter_labels(id("GEN")).unwrap(),
            vec![
                ChapterLabel::FrontMatter,
                ChapterLabel::Number("1".into()),
                ChapterLabel::Number("2".into()),
            ],
            "{lane:?}"
        );
    }
}

#[test]
fn out_of_order_and_duplicate_chapter_runs_are_retained_in_order() {
    // Nothing sorts and nothing collapses: a book with `\c 2` before `\c 1` and
    // a reopened `\c 1` reports exactly that sequence.
    let source = "\\id GEN\n\\c 2\n\\p\n\\v 1 b\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 2 a2\n";
    for lane in LANES {
        let mut resident = braid();
        resident
            .replace_corpus(CorpusInput::new(vec![lane.book(
                "GEN",
                "01-GEN.usfm",
                source,
            )]))
            .unwrap();
        assert_eq!(
            resident.chapter_labels(id("GEN")).unwrap(),
            vec![
                ChapterLabel::FrontMatter,
                ChapterLabel::Number("2".into()),
                ChapterLabel::Number("1".into()),
                ChapterLabel::Number("1".into()),
            ],
            "{lane:?}"
        );
    }
}

#[test]
fn labels_are_verbatim_and_never_numerically_normalized() {
    let source = "\\id GEN\n\\c 01\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n";
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![Lane::Parsed.book(
            "GEN",
            "01-GEN.usfm",
            source,
        )]))
        .unwrap();
    // `01` and `1` are two distinct labels, so neither chapter op is ambiguous.
    assert_eq!(
        resident.chapter_labels(id("GEN")).unwrap(),
        vec![
            ChapterLabel::FrontMatter,
            ChapterLabel::Number("01".into()),
            ChapterLabel::Number("1".into()),
        ]
    );
}

#[test]
fn resubmitting_identical_content_is_a_no_op() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = state_fingerprint(&resident);

        let effect = resident
            .update_book(lane.book("GEN", "01-GEN.usfm", GEN_SOURCE))
            .unwrap();
        assert!(effect.changed.is_empty(), "{lane:?}");
        assert!(effect.removed.is_empty(), "{lane:?}");
        assert_eq!(effect.snapshot_id.0, before.0, "{lane:?}");

        let effect = resident
            .replace_corpus(CorpusInput::new(vec![
                lane.book("GEN", "01-GEN.usfm", GEN_SOURCE),
                lane.book("EXO", "02-EXO.usfm", EXO_SOURCE),
            ]))
            .unwrap();
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");

        // Re-submitting a chapter with its own current content is a no-op too,
        // even though it travels the whole splice-rebuild-rehash path.
        let effect = resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
                ChapterInput::Tokens(
                    resident
                        .to_tokens(Scope::chapter(id("GEN"), ChapterLabel::Number("1".into())))
                        .unwrap()
                        .remove(0)
                        .tokens,
                ),
            )
            .unwrap();
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn a_changed_chapter_reports_exactly_that_chapter() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = resident.expected_snapshot_id();
        let target = ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into()));

        let effect = resident
            .update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\c 2\n\\p\n\\v 1 Thus the heavens and the earth.\n")),
            )
            .unwrap();

        assert_eq!(
            effect.changed,
            vec![Scope::chapter(id("GEN"), ChapterLabel::Number("2".into()))],
            "{lane:?}"
        );
        assert!(effect.removed.is_empty(), "{lane:?}");
        assert_ne!(effect.snapshot_id, before, "{lane:?}");
        // EXO was not inspected and must not appear.
        assert_eq!(effect.changed.len(), 1, "{lane:?}");
        assert_eq!(
            single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
            "\\id GEN\n\\h Genesis\n\\c 1\n\\p\n\\v 1 In the beginning.\n\\c 2\n\\p\n\\v 1 Thus the heavens and the earth.\n",
            "{lane:?}"
        );
    }
}

#[test]
fn a_duplicate_chapter_book_widens_its_effect_to_the_whole_book() {
    // With two `\c 1` runs there is no unambiguous chapter address to hand a
    // consumer, so the effect reports the book. The *operation* still has to be
    // unambiguous — here the edit targets the unique `\c 2`.
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 2 b\n\\c 2\n\\p\n\\v 1 c\n";
    for lane in LANES {
        let mut resident = braid();
        resident
            .replace_corpus(CorpusInput::new(vec![lane.book(
                "GEN",
                "01-GEN.usfm",
                source,
            )]))
            .unwrap();

        let effect = resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into())),
                ChapterInput::Tokens(owned("\\c 2\n\\p\n\\v 1 c edited\n")),
            )
            .unwrap();
        assert_eq!(effect.changed, vec![Scope::book(id("GEN"))], "{lane:?}");
    }
}

#[test]
fn ambiguous_chapter_operations_error_atomically() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 2 b\n";
    for lane in LANES {
        let mut resident = braid();
        resident
            .replace_corpus(CorpusInput::new(vec![lane.book(
                "GEN",
                "01-GEN.usfm",
                source,
            )]))
            .unwrap();
        let before = state_fingerprint(&resident);
        let target = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));

        assert_eq!(
            resident.update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 edited\n"))
            ),
            Err(IngestError::AmbiguousChapter {
                target: target.clone(),
                matches: 2
            }),
            "{lane:?}"
        );
        assert_eq!(
            resident.remove_chapter(target.clone()),
            Err(ScopeError::AmbiguousChapter {
                target: target.clone(),
                matches: 2
            }),
            "{lane:?}"
        );
        assert_eq!(
            resident.to_tokens(Scope::chapter(id("GEN"), ChapterLabel::Number("1".into()))),
            Err(ScopeError::AmbiguousChapter { target, matches: 2 }),
            "{lane:?}"
        );
        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn a_missing_or_mismatched_chapter_replacement_is_rejected_atomically() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = state_fingerprint(&resident);

        let absent = ChapterTarget::new(id("GEN"), ChapterLabel::Number("9".into()));
        assert_eq!(
            resident.update_chapter(
                absent.clone(),
                ChapterInput::Tokens(owned("\\c 9\n\\p\n\\v 1 x\n"))
            ),
            Err(IngestError::ChapterNotFound(absent)),
            "{lane:?}"
        );

        // A chapter of a book that is not resident at all is equally not found.
        let absent_book = ChapterTarget::new(id("LEV"), ChapterLabel::Number("1".into()));
        assert_eq!(
            resident.update_chapter(absent_book.clone(), ChapterInput::Tokens(owned("\\c 1\n"))),
            Err(IngestError::ChapterNotFound(absent_book)),
            "{lane:?}"
        );

        // Replacing chapter 1 with chapter 3 is a structural change, which is
        // `update_book`'s job.
        let target = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
        assert_eq!(
            resident.update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\c 3\n\\p\n\\v 1 x\n"))
            ),
            Err(IngestError::ReplacementLabelMismatch {
                target: target.clone(),
                found: ChapterLabel::Number("3".into())
            }),
            "{lane:?}"
        );

        // Smuggling an extra chapter in behind a correct first run is rejected
        // for the same reason.
        assert_eq!(
            resident.update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 x\n\\c 5\n\\p\n\\v 1 y\n"))
            ),
            Err(IngestError::ReplacementLabelMismatch {
                target: target.clone(),
                found: ChapterLabel::Number("5".into())
            }),
            "{lane:?}"
        );

        // Two runs of the target label inside one replacement is ambiguous, not
        // a mismatch: the label matches, the count does not.
        assert_eq!(
            resident.update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 x\n\\c 1\n\\p\n\\v 2 y\n"))
            ),
            Err(IngestError::AmbiguousChapter {
                target: target.clone(),
                matches: 2
            }),
            "{lane:?}"
        );

        // Content with no chapter marker at all is front matter, not chapter 1.
        assert_eq!(
            resident.update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\p\n\\v 1 x\n"))
            ),
            Err(IngestError::ReplacementLabelMismatch {
                target,
                found: ChapterLabel::FrontMatter
            }),
            "{lane:?}"
        );

        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn front_matter_is_a_run_the_caller_can_replace_and_empty() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let target = ChapterTarget::new(id("GEN"), ChapterLabel::FrontMatter);

        let effect = resident
            .update_chapter(
                target.clone(),
                ChapterInput::Tokens(owned("\\id GEN\n\\h Beginnings\n")),
            )
            .unwrap();
        assert_eq!(
            effect.changed,
            vec![Scope::chapter(id("GEN"), ChapterLabel::FrontMatter)],
            "{lane:?}"
        );

        // An empty replacement clears front matter — the one run for which
        // "no content" is a legal shape.
        resident
            .update_chapter(target, ChapterInput::Tokens(Vec::new()))
            .unwrap();
        assert_eq!(
            single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
            "\\c 1\n\\p\n\\v 1 In the beginning.\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n",
            "{lane:?}"
        );
        assert_eq!(
            resident.chapter_labels(id("GEN")).unwrap(),
            vec![
                ChapterLabel::Number("1".into()),
                ChapterLabel::Number("2".into())
            ],
            "{lane:?}"
        );
    }
}

#[test]
fn duplicate_declared_books_and_source_keys_are_rejected_atomically() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = state_fingerprint(&resident);

        assert_eq!(
            resident.replace_corpus(CorpusInput::new(vec![
                lane.book("GEN", "a.usfm", GEN_SOURCE),
                lane.book("GEN", "b.usfm", GEN_SOURCE),
            ])),
            Err(IngestError::DuplicateBook {
                book: id("GEN"),
                sources: vec![key("a.usfm"), key("b.usfm")]
            }),
            "{lane:?}"
        );
        assert_eq!(
            resident.replace_corpus(CorpusInput::new(vec![
                lane.book("GEN", "same.usfm", GEN_SOURCE),
                lane.book("EXO", "same.usfm", EXO_SOURCE),
            ])),
            Err(IngestError::DuplicateSourceKey {
                source: key("same.usfm")
            }),
            "{lane:?}"
        );
        // Rebinding a key that another resident book already holds is refused
        // on the single-book path too.
        assert_eq!(
            resident.update_book(lane.book("EXO", "01-GEN.usfm", EXO_SOURCE)),
            Err(IngestError::DuplicateSourceKey {
                source: key("01-GEN.usfm")
            }),
            "{lane:?}"
        );

        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn a_duplicate_token_id_is_rejected_atomically() {
    // The caller-token lane is the one that can carry a collision: reconciliation
    // keys on (book, token id), so two tokens sharing an id would leave one
    // unaddressable.
    let mut resident = seeded(Lane::CallerTokens);
    let before = state_fingerprint(&resident);

    let mut tokens = owned(EXO_SOURCE);
    let duplicated = tokens[3].clone();
    tokens.push(duplicated.clone());

    assert_eq!(
        resident.update_book(BookInput::Tokens(BookTokensInput {
            source_key: key("02-EXO.usfm"),
            book: id("EXO"),
            tokens,
            line_ending: LineEnding::Lf,
        })),
        Err(IngestError::DuplicateTokenId {
            book: id("EXO"),
            id: duplicated.id().clone()
        })
    );
    assert_eq!(state_fingerprint(&resident), before);
}

#[test]
fn removal_and_clear_report_exactly_what_left() {
    for lane in LANES {
        let mut resident = seeded(lane);

        let effect = resident.remove_book(id("EXO"));
        assert_eq!(effect.removed, vec![id("EXO")], "{lane:?}");
        assert!(effect.changed.is_empty(), "{lane:?}");

        // Removing it again is a no-op: the requested end state already holds.
        let after = resident.expected_snapshot_id();
        let effect = resident.remove_book(id("EXO"));
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(effect.snapshot_id, after, "{lane:?}");

        let effect = resident.clear();
        assert_eq!(effect.removed, vec![id("GEN")], "{lane:?}");
        assert!(resident.books().is_empty(), "{lane:?}");

        let empty = resident.expected_snapshot_id();
        assert!(resident.clear().is_noop(), "{lane:?}");
        assert_eq!(resident.expected_snapshot_id(), empty, "{lane:?}");
    }
}

#[test]
fn removing_a_chapter_reports_the_whole_book() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let effect = resident
            .remove_chapter(ChapterTarget::new(
                id("GEN"),
                ChapterLabel::Number("1".into()),
            ))
            .unwrap();
        // Whole-book: the chapter address the caller used no longer exists.
        assert_eq!(effect.changed, vec![Scope::book(id("GEN"))], "{lane:?}");
        assert_eq!(
            single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
            "\\id GEN\n\\h Genesis\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n",
            "{lane:?}"
        );
        assert_eq!(
            resident.remove_chapter(ChapterTarget::new(
                id("LEV"),
                ChapterLabel::Number("1".into())
            )),
            Err(ScopeError::BookNotFound(id("LEV"))),
            "{lane:?}"
        );
    }
}

#[test]
fn snapshot_identity_is_content_derived_and_order_sensitive() {
    // Same books, same bytes, different order: the id is over the *ordered*
    // per-book hashes, so it differs. Nothing was rewritten, so `changed` is
    // empty — the two facts answer different questions.
    let mut forward = braid();
    forward
        .replace_corpus(CorpusInput::new(vec![
            Lane::Parsed.book("GEN", "01-GEN.usfm", GEN_SOURCE),
            Lane::Parsed.book("EXO", "02-EXO.usfm", EXO_SOURCE),
        ]))
        .unwrap();
    let before = forward.expected_snapshot_id();
    let effect = forward
        .replace_corpus(CorpusInput::new(vec![
            Lane::Parsed.book("EXO", "02-EXO.usfm", EXO_SOURCE),
            Lane::Parsed.book("GEN", "01-GEN.usfm", GEN_SOURCE),
        ]))
        .unwrap();
    assert!(effect.changed.is_empty());
    assert!(effect.removed.is_empty());
    assert_ne!(effect.snapshot_id, before);

    // A freshly built braid with the same ordered content reproduces the id
    // exactly — no counters, no timestamps, no session state.
    let mut rebuilt = braid();
    rebuilt
        .replace_corpus(CorpusInput::new(vec![
            Lane::Parsed.book("EXO", "02-EXO.usfm", EXO_SOURCE),
            Lane::Parsed.book("GEN", "01-GEN.usfm", GEN_SOURCE),
        ]))
        .unwrap();
    assert_eq!(
        rebuilt.expected_snapshot_id(),
        forward.expected_snapshot_id()
    );
}

#[test]
fn a_source_key_rebinding_changes_no_semantic_identity() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = resident.expected_snapshot_id();
        let dirty_before = resident.dirty_books();

        let effect = resident
            .update_book(lane.book("GEN", "moved/01-GEN.usfm", GEN_SOURCE))
            .unwrap();

        // A move rebinds metadata only: no tokens changed, so nothing needs
        // re-pulling, semantic identity holds, and caches stay valid.
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(effect.snapshot_id, before, "{lane:?}");
        assert_eq!(resident.dirty_books(), dirty_before, "{lane:?}");
        assert_eq!(
            resident.books().first().unwrap().source_key.as_str(),
            "moved/01-GEN.usfm",
            "{lane:?}"
        );
    }
}

#[test]
fn config_updates_leave_bytes_alone_and_stale_every_book() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = resident.expected_snapshot_id();

        let mut options = LintOptions::scoped(LintScope::Book);
        options.declared_book = Some(id("GEN"));
        let effect = resident.update_config(BraidConfig::new(options.clone()));

        // No tokens were rewritten and the id covers source bytes only.
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(effect.snapshot_id, before, "{lane:?}");
        assert_eq!(
            resident.dirty_books(),
            vec![id("GEN"), id("EXO")],
            "{lane:?}"
        );
        assert_eq!(resident.config().lint, options, "{lane:?}");
    }
}

#[test]
fn unchanged_books_keep_their_dirty_state_across_a_corpus_replacement() {
    // Cache preservation is the point: a corpus reseed that changes one book
    // must not stale the others. (Braid clears these stamps when lint runs;
    // until then every resident book is dirty by construction, so this asserts
    // the carry-forward path is wired, not the eventual clean state.)
    let lane = Lane::Parsed;
    let mut resident = seeded(lane);
    let effect = resident
        .replace_corpus(CorpusInput::new(vec![
            lane.book("GEN", "01-GEN.usfm", GEN_SOURCE),
            lane.book(
                "EXO",
                "02-EXO.usfm",
                "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names of the sons.\n",
            ),
        ]))
        .unwrap();
    assert_eq!(effect.changed, vec![Scope::book(id("EXO"))]);
    assert!(effect.removed.is_empty());
}

#[test]
fn to_tokens_normalizes_its_input() {
    for lane in LANES {
        let resident = seeded(lane);
        let chapter_one = Scope::chapter(id("GEN"), ChapterLabel::Number("1".into()));
        let chapter_two = Scope::chapter(id("GEN"), ChapterLabel::Number("2".into()));

        // Duplicates collapse.
        let pulled = resident
            .to_tokens(vec![chapter_one.clone(), chapter_one.clone()])
            .unwrap();
        assert_eq!(pulled.len(), 1, "{lane:?}");

        // A whole-book scope absorbs that book's chapter scopes, in either
        // arrival order — which is what makes concatenating several effects'
        // `changed` lists safe.
        for scopes in [
            vec![chapter_one.clone(), Scope::book(id("GEN"))],
            vec![Scope::book(id("GEN")), chapter_two.clone()],
        ] {
            let pulled = resident.to_tokens(scopes).unwrap();
            assert_eq!(pulled.len(), 1, "{lane:?}");
            assert_eq!(pulled[0].chapter, None, "{lane:?}");
        }

        // Two chapters of one book come back in run order regardless of request
        // order, and a second book follows in corpus order.
        let pulled = resident
            .to_tokens(vec![
                Scope::book(id("EXO")),
                chapter_two.clone(),
                chapter_one.clone(),
            ])
            .unwrap();
        assert_eq!(
            pulled
                .iter()
                .map(|scope| (scope.book, scope.chapter.clone()))
                .collect::<Vec<_>>(),
            vec![
                (id("GEN"), Some(ChapterLabel::Number("1".into()))),
                (id("GEN"), Some(ChapterLabel::Number("2".into()))),
                (id("EXO"), None),
            ],
            "{lane:?}"
        );
    }
}

#[test]
fn to_tokens_returns_current_truth_for_an_effects_scopes() {
    for lane in LANES {
        let mut resident = seeded(lane);
        let effect = resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into())),
                ChapterInput::Tokens(owned("\\c 2\n\\p\n\\v 1 Thus the heavens, edited.\n")),
            )
            .unwrap();

        // The one-expression form: pull straight from the effect.
        let pulled = resident.to_tokens(&effect).unwrap();
        assert_eq!(pulled.len(), 1, "{lane:?}");
        assert_eq!(
            pulled[0]
                .tokens
                .iter()
                .map(|token| token.source())
                .collect::<String>(),
            "\\c 2\n\\p\n\\v 1 Thus the heavens, edited.\n",
            "{lane:?}"
        );

        // Current truth, not state-at-effect-time: a later mutation is visible
        // through the same (still valid) effect scopes.
        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into())),
                ChapterInput::Tokens(owned("\\c 2\n\\p\n\\v 1 Later still.\n")),
            )
            .unwrap();
        let pulled = resident.to_tokens(&effect).unwrap();
        assert_eq!(
            pulled[0]
                .tokens
                .iter()
                .map(|token| token.source())
                .collect::<String>(),
            "\\c 2\n\\p\n\\v 1 Later still.\n",
            "{lane:?}"
        );
    }
}

#[test]
fn a_pull_of_an_unresolvable_scope_errors_rather_than_returning_a_partial() {
    let resident = seeded(Lane::Parsed);
    assert_eq!(
        resident.to_tokens(vec![Scope::book(id("GEN")), Scope::book(id("LEV"))]),
        Err(ScopeError::BookNotFound(id("LEV")))
    );
    assert_eq!(
        resident.to_tokens(Scope::chapter(id("GEN"), ChapterLabel::Number("9".into()))),
        Err(ScopeError::ChapterNotFound(ChapterTarget::new(
            id("GEN"),
            ChapterLabel::Number("9".into())
        )))
    );
    assert!(resident.to_tokens(Vec::new()).unwrap().is_empty());
}

#[test]
fn pulled_tokens_round_trip_through_the_token_ingest_lane() {
    // The editor loop: pull a book's tokens, push them back, get a no-op. This
    // is what proves the pull returns the resident stream itself rather than a
    // re-derived approximation.
    for lane in LANES {
        let mut resident = seeded(lane);
        let before = state_fingerprint(&resident);
        let tokens = resident
            .to_tokens(Scope::book(id("GEN")))
            .unwrap()
            .remove(0)
            .tokens;

        let effect = resident
            .update_book(BookInput::Tokens(BookTokensInput {
                source_key: key("01-GEN.usfm"),
                book: id("GEN"),
                tokens,
                line_ending: LineEnding::Lf,
            }))
            .unwrap();
        assert!(effect.is_noop(), "{lane:?}");
        assert_eq!(state_fingerprint(&resident), before, "{lane:?}");
    }
}

#[test]
fn a_books_line_ending_survives_a_token_push() {
    // The CRLF trap (epic §2.2#16): the editor pushes `\n` newline tokens into a
    // book whose file is CRLF. Without a per-book ending the book would silently
    // flip endings on first edit and its hash would never match what is saved.
    let crlf = GEN_SOURCE.replace('\n', "\r\n");
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
            source_key: key("01-GEN.usfm"),
            book: id("GEN"),
            source: crlf.clone(),
        }]))
        .unwrap();
    assert_eq!(
        resident.books().first().unwrap().line_ending,
        LineEnding::CrLf
    );
    assert_eq!(
        single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        crlf
    );

    resident
        .update_chapter(
            ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into())),
            // LF tokens, as any JS editor produces.
            ChapterInput::Tokens(owned("\\c 2\n\\p\n\\v 1 Edited.\n")),
        )
        .unwrap();
    let saved = single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap());
    assert_eq!(
        saved,
        "\\id GEN\r\n\\h Genesis\r\n\\c 1\r\n\\p\r\n\\v 1 In the beginning.\r\n\\c 2\r\n\\p\r\n\\v 1 Edited.\r\n"
    );
    assert!(!saved.contains("\n\n"));
    // The hash is over the bytes that would be written, so a CRLF book is not
    // permanently dirty against its own file: ingesting those exact bytes
    // reproduces the same hash.
    let mut reopened = braid();
    reopened
        .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
            source_key: key("01-GEN.usfm"),
            book: id("GEN"),
            source: saved.clone(),
        }]))
        .unwrap();
    assert_eq!(
        resident.books().first().unwrap().source_hash,
        reopened.books().first().unwrap().source_hash
    );
}

#[test]
fn a_declared_ending_applies_to_a_token_ingested_book() {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![BookInput::Tokens(BookTokensInput {
            source_key: key("01-GEN.usfm"),
            book: id("GEN"),
            tokens: owned(GEN_SOURCE),
            line_ending: LineEnding::CrLf,
        })]))
        .unwrap();
    assert_eq!(
        single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        GEN_SOURCE.replace('\n', "\r\n")
    );
}

#[test]
fn a_mixed_ending_file_is_preserved_verbatim_until_it_is_edited() {
    let mixed = "\\id GEN\r\n\\c 1\n\\p\r\n\\v 1 a\n";
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
            source_key: key("01-GEN.usfm"),
            book: id("GEN"),
            source: mixed.to_string(),
        }]))
        .unwrap();
    assert_eq!(
        single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        mixed,
        "ingest never normalizes"
    );

    resident
        .update_chapter(
            ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
            ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 b\n")),
        )
        .unwrap();
    // First edit re-emits the whole book with the ending it leads with.
    assert_eq!(
        single(resident.to_usfm(CorpusScope::Book(id("GEN"))).unwrap()),
        "\\id GEN\r\n\\c 1\r\n\\p\r\n\\v 1 b\r\n"
    );
}

#[test]
fn chapter_scoped_projections_report_the_run() {
    let resident = seeded(Lane::Parsed);
    assert_eq!(
        single(
            resident
                .to_usfm(CorpusScope::Chapter(ChapterTarget::new(
                    id("GEN"),
                    ChapterLabel::Number("1".into())
                )))
                .unwrap()
        ),
        "\\c 1\n\\p\n\\v 1 In the beginning.\n"
    );
    assert_eq!(
        single(
            resident
                .to_usfm(CorpusScope::Chapter(ChapterTarget::new(
                    id("GEN"),
                    ChapterLabel::FrontMatter
                )))
                .unwrap()
        ),
        "\\id GEN\n\\h Genesis\n"
    );
    match resident.to_usfm(CorpusScope::All).unwrap() {
        ScopedOutput::All(books) => {
            assert_eq!(
                books
                    .iter()
                    .map(|entry| (entry.book, entry.value.as_str()))
                    .collect::<Vec<_>>(),
                vec![(id("GEN"), GEN_SOURCE), (id("EXO"), EXO_SOURCE)]
            );
        }
        ScopedOutput::Single(_) => panic!("All scope must group"),
    }
}

/// The sid of the first token that carries a verse-level anchor.
fn verse_sid(tokens: &[OwnedToken]) -> Option<String> {
    tokens
        .iter()
        .find_map(|token| token.sid().filter(|sid| sid.contains(':')))
        .map(str::to_string)
}

#[test]
fn resident_tokens_keep_the_ids_and_sids_their_source_gave_them() {
    // Core is address-agnostic and honors caller ids/SIDs, and braid does not
    // re-address a stream it was handed: the editor lane keeps whatever the
    // caller holds, which is the sid braid handed it in the first place.
    let mut resident = seeded(Lane::Parsed);
    let tokens = resident
        .to_tokens(Scope::chapter(id("GEN"), ChapterLabel::Number("1".into())))
        .unwrap()
        .remove(0)
        .tokens;
    let sid = verse_sid(&tokens);
    resident
        .update_chapter(
            ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
            ChapterInput::Tokens(tokens),
        )
        .unwrap();
    let pulled = resident
        .to_tokens(Scope::chapter(id("GEN"), ChapterLabel::Number("1".into())))
        .unwrap()
        .remove(0)
        .tokens;
    assert_eq!(verse_sid(&pulled), sid);
}
