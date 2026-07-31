//! Resident `prepare_format_patch` / `apply_format_patch`.
//!
//! A prepared format patch proxies core's own `format` verbatim; braid's
//! contribution is scope resolution, freezing the result against a snapshot,
//! and atomic multi-book admission — nothing here is a second format engine.

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterLabel, ChapterTarget, CorpusInput,
    CorpusScope, FormatPatchError, IngestError, LineEnding, PatchPreparation, ScopedOutput,
    SourceKey,
};
use usfm_onion::format::FormatOptions;
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

/// A double space in verse 1's text that `collapse_whitespace_in_text`
/// collapses to one — a format change with no token insertion/removal, so it
/// never needs the minter.
const GEN_SOURCE: &str =
    "\\id GEN\n\\c 1\n\\p\n\\v 1 In  the beginning.\n\\c 2\n\\p\n\\v 1 Thus  the heavens.\n";
const GEN_FORMATTED: &str =
    "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n\\c 2\n\\p\n\\v 1 Thus the heavens.\n";
const EXO_SOURCE: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These  are the names.\n";
const EXO_FORMATTED: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";
/// Already exactly what `format` would produce.
const ALREADY_FORMATTED: &str = "\\id LEV\n\\c 1\n\\p\n\\v 1 And he called.\n";
/// A chapter intro with no `\p` before its first verse. Core's
/// `insert_default_paragraph_after_chapter_intro`/`insert_structural_linebreaks`
/// rules (both on by default) synthesize a paragraph marker and its newline —
/// the id-less path through admission that every fixture above avoids on
/// purpose.
const GEN_MISSING_PARAGRAPH: &str = "\\id GEN\n\\c 1\n\\v 1 In the beginning.\n";

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

fn seeded(lane: Lane, books: Vec<(&str, &str)>) -> Braid {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(
            books
                .into_iter()
                .map(|(code, source)| lane.book(code, source))
                .collect(),
        ))
        .expect("distinct books");
    resident
}

fn source_of(resident: &Braid, book: &str) -> String {
    match resident.to_usfm(CorpusScope::Book(id(book))) {
        Ok(ScopedOutput::Single(source)) => source,
        other => panic!("expected one book's bytes, got {other:?}"),
    }
}

#[test]
fn an_already_formatted_scope_prepares_unchanged() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("LEV", ALREADY_FORMATTED)]);
        let preparation = resident
            .prepare_format_patch(CorpusScope::Book(id("LEV")), FormatOptions::all_enabled())
            .unwrap();
        assert_eq!(preparation, PatchPreparation::Unchanged, "{lane:?}");
    }
}

#[test]
fn preparing_a_format_patch_does_not_mutate_anything() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        let before_snapshot = resident.expected_snapshot_id();
        let before_bytes = source_of(&resident, "GEN");
        let before_dirty = resident.books_awaiting_lint();

        let preparation = resident
            .prepare_format_patch(CorpusScope::Book(id("GEN")), FormatOptions::all_enabled())
            .unwrap();
        assert!(
            matches!(preparation, PatchPreparation::Ready(_)),
            "{lane:?}"
        );

        assert_eq!(resident.expected_snapshot_id(), before_snapshot, "{lane:?}");
        assert_eq!(source_of(&resident, "GEN"), before_bytes, "{lane:?}");
        assert_eq!(resident.books_awaiting_lint(), before_dirty, "{lane:?}");
    }
}

#[test]
fn applying_a_book_scoped_format_patch_rewrites_exactly_what_format_would() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        let before_snapshot = resident.expected_snapshot_id();

        let id_ = match resident
            .prepare_format_patch(CorpusScope::Book(id("GEN")), FormatOptions::all_enabled())
            .unwrap()
        {
            PatchPreparation::Ready(id) => id,
            PatchPreparation::Unchanged => panic!("expected a change"),
        };

        let effect = resident.apply_format_patch(id_).unwrap();
        assert!(!effect.is_noop(), "{lane:?}");
        assert_ne!(effect.snapshot_id.0, before_snapshot.0, "{lane:?}");
        assert_eq!(
            effect.changed,
            vec![braid::Scope::book(id("GEN"))],
            "{lane:?}"
        );
        assert_eq!(source_of(&resident, "GEN"), GEN_FORMATTED, "{lane:?}");
        assert_eq!(resident.books_awaiting_lint(), vec![id("GEN")], "{lane:?}");
    }
}

#[test]
fn chapter_scoped_format_touches_only_that_run() {
    let mut resident = seeded(Lane::Parsed, vec![("GEN", GEN_SOURCE)]);
    let chapter_1 = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));

    let id_ = match resident
        .prepare_format_patch(
            CorpusScope::Chapter(chapter_1.clone()),
            FormatOptions::all_enabled(),
        )
        .unwrap()
    {
        PatchPreparation::Ready(id) => id,
        PatchPreparation::Unchanged => panic!("expected chapter 1's double space to format"),
    };
    let effect = resident.apply_format_patch(id_).unwrap();
    // Chapter 1's own double space collapsed; chapter 2's did not move, so the
    // effect is chapter-scoped, not whole-book.
    assert_eq!(
        effect.changed,
        vec![braid::Scope::chapter(
            id("GEN"),
            ChapterLabel::Number("1".into())
        )]
    );
    let after = source_of(&resident, "GEN");
    assert!(after.contains("In the beginning."));
    assert!(after.contains("Thus  the heavens."), "chapter 2 untouched");
}

#[test]
fn all_scope_prepares_and_applies_every_book_that_changed_atomically() {
    let mut resident = seeded(
        Lane::Parsed,
        vec![
            ("GEN", GEN_SOURCE),
            ("EXO", EXO_SOURCE),
            ("LEV", ALREADY_FORMATTED),
        ],
    );

    let id_ = match resident
        .prepare_format_patch(CorpusScope::All, FormatOptions::all_enabled())
        .unwrap()
    {
        PatchPreparation::Ready(id) => id,
        PatchPreparation::Unchanged => panic!("GEN and EXO both need formatting"),
    };
    let effect = resident.apply_format_patch(id_).unwrap();

    // Only the two books that actually changed are reported, and both commit
    // together in one mutation.
    let mut changed = effect.changed;
    changed.sort_by_key(|scope| scope.book.to_string());
    assert_eq!(
        changed,
        vec![braid::Scope::book(id("EXO")), braid::Scope::book(id("GEN"))]
    );
    assert_eq!(source_of(&resident, "GEN"), GEN_FORMATTED);
    assert_eq!(source_of(&resident, "EXO"), EXO_FORMATTED);
    assert_eq!(
        source_of(&resident, "LEV"),
        ALREADY_FORMATTED,
        "LEV was already formatted"
    );
}

#[test]
fn a_stale_preparation_is_rejected_atomically() {
    let mut resident = seeded(Lane::Parsed, vec![("GEN", GEN_SOURCE)]);
    let id_ = match resident
        .prepare_format_patch(CorpusScope::Book(id("GEN")), FormatOptions::all_enabled())
        .unwrap()
    {
        PatchPreparation::Ready(id) => id,
        PatchPreparation::Unchanged => panic!("expected a change"),
    };

    // An unrelated mutation moves the corpus snapshot out from under the
    // preparation.
    resident
        .update_book(Lane::Parsed.book("GEN", GEN_FORMATTED))
        .unwrap();
    let before = resident.expected_snapshot_id();
    let before_bytes = source_of(&resident, "GEN");

    let result = resident.apply_format_patch(id_);
    assert!(matches!(
        result,
        Err(FormatPatchError::StaleSnapshot { .. })
    ));
    // Rejected atomically: nothing moved.
    assert_eq!(resident.expected_snapshot_id(), before);
    assert_eq!(source_of(&resident, "GEN"), before_bytes);
}

#[test]
fn an_unknown_ordinal_is_rejected() {
    let mut resident = seeded(Lane::Parsed, vec![("GEN", GEN_SOURCE)]);
    let snapshot = resident.expected_snapshot_id();
    let bogus = braid::FormatPatchId {
        snapshot,
        ordinal: 9999,
    };
    assert_eq!(
        resident.apply_format_patch(bogus),
        Err(FormatPatchError::UnknownPatch(bogus))
    );
}

#[test]
fn external_stateless_format_never_touches_resident_state() {
    // The stateless proxy operates purely on caller-supplied tokens and has no
    // resident receiver to mutate.
    let resident = seeded(Lane::Parsed, vec![("GEN", GEN_SOURCE)]);
    let before_snapshot = resident.expected_snapshot_id();
    let before_bytes = source_of(&resident, "GEN");

    let tokens = owned(GEN_SOURCE);
    let working: Vec<usfm_onion::format::FormatToken> = tokens
        .iter()
        .map(usfm_onion::format::FormatToken::from)
        .collect();
    let _ = usfm_onion::format::format(&working, FormatOptions::all_enabled());

    assert_eq!(resident.expected_snapshot_id(), before_snapshot);
    assert_eq!(source_of(&resident, "GEN"), before_bytes);
}

/// The apply-time mint sweep, exercised for real: unlike every fixture above
/// (chosen so formatting only edits existing tokens in place), this one
/// genuinely inserts tokens with no id at all. Every survivor keeps its own
/// id, every inserted token is minted a fresh one by the handle's own
/// function, and every id in the book is unique afterward.
#[test]
fn format_insertions_are_minted_unique_and_survivors_keep_their_ids() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_MISSING_PARAGRAPH)]);
        let before_ids: Vec<String> = resident
            .to_tokens(braid::Scope::book(id("GEN")))
            .unwrap()
            .remove(0)
            .tokens
            .iter()
            .map(|token| token.id().as_str().to_string())
            .collect();

        let id_ = match resident
            .prepare_format_patch(CorpusScope::Book(id("GEN")), FormatOptions::all_enabled())
            .unwrap()
        {
            PatchPreparation::Ready(id) => id,
            PatchPreparation::Unchanged => {
                panic!("{lane:?}: expected format to insert a paragraph marker")
            }
        };
        let effect = resident.apply_format_patch(id_).unwrap();
        assert!(!effect.is_noop(), "{lane:?}");

        let after_ids: Vec<String> = resident
            .to_tokens(braid::Scope::book(id("GEN")))
            .unwrap()
            .remove(0)
            .tokens
            .iter()
            .map(|token| token.id().as_str().to_string())
            .collect();
        assert!(
            after_ids.len() > before_ids.len(),
            "{lane:?}: format inserted at least one token"
        );

        // Every pre-existing token is still there under its own id.
        for original in &before_ids {
            assert!(
                after_ids.contains(original),
                "{lane:?}: {original} survived formatting"
            );
        }

        // Ids in the book are unique — the residency invariant, checked
        // directly rather than only inferred from a successful apply.
        let mut deduped = after_ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            after_ids.len(),
            "{lane:?}: every token id in the book is unique"
        );

        // Every id that is new relative to the pre-format book came from the
        // handle's own minter (this file's `braid()` prefixes every minted id
        // with "minted-"), not invented anywhere else.
        let minted: Vec<&String> = after_ids
            .iter()
            .filter(|candidate| !before_ids.contains(candidate))
            .collect();
        assert_eq!(minted.len(), after_ids.len() - before_ids.len(), "{lane:?}");
        for candidate in minted {
            assert!(
                candidate.starts_with("minted-"),
                "{lane:?}: {candidate} did not come from the handle's minter"
            );
        }
    }
}

/// A minter is BYO — braid enforces uniqueness at the residency boundary
/// rather than trusting it. A minter that hands back an id already present in
/// the book must make `apply_format_patch` reject atomically, not corrupt the
/// resident book with a duplicate id.
#[test]
fn a_hostile_minter_returning_a_colliding_id_is_rejected_atomically() {
    for lane in LANES {
        // Parsed ids for this fixture are positional (`GEN-0`, `GEN-1`, ...);
        // `GEN-6` is the book's `\v` marker in both lanes (the caller-tokens
        // lane is also seeded from `parse`), chosen deliberately over an
        // arbitrary existing id: it carries the same sid as the synthesized
        // paragraph marker/newline, so sid resolution succeeds and the
        // collision is caught where it is meant to be caught — the token-id
        // uniqueness check — rather than failing earlier for an unrelated
        // reason.
        let mut resident = Braid::new(BraidConfig::new(LintOptions::scoped(LintScope::Book)), {
            || "GEN-6".to_string()
        });
        resident
            .replace_corpus(CorpusInput::new(vec![
                lane.book("GEN", GEN_MISSING_PARAGRAPH),
            ]))
            .expect("one book");

        let id_ = match resident
            .prepare_format_patch(CorpusScope::Book(id("GEN")), FormatOptions::all_enabled())
            .unwrap()
        {
            PatchPreparation::Ready(id) => id,
            PatchPreparation::Unchanged => {
                panic!("{lane:?}: expected format to insert a paragraph marker")
            }
        };

        let before_snapshot = resident.expected_snapshot_id();
        let before_bytes = source_of(&resident, "GEN");
        let before_dirty = resident.books_awaiting_lint();

        let result = resident.apply_format_patch(id_);
        assert!(
            matches!(
                result,
                Err(FormatPatchError::InvalidResult(
                    IngestError::DuplicateTokenId { .. }
                ))
            ),
            "{lane:?}: got {result:?}"
        );

        // Rejected atomically: nothing about resident state moved.
        assert_eq!(resident.expected_snapshot_id(), before_snapshot, "{lane:?}");
        assert_eq!(source_of(&resident, "GEN"), before_bytes, "{lane:?}");
        assert_eq!(resident.books_awaiting_lint(), before_dirty, "{lane:?}");
    }
}
