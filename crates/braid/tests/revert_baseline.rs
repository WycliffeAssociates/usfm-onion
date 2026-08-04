//! `Braid::revert_to_baseline`: whole-book replacement from `BaselineState`,
//! atomic across the scope, with load-bearing token identity.

use braid::{
    BaselineError, BookInput, BookTokensInput, Braid, BraidConfig, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusScope, LineEnding, SourceKey,
};
use usfm_onion::format::FormatToken;
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

fn id(value: &str) -> BookId {
    BookId::from_str(value).expect("three-character code")
}

fn key(value: &str) -> SourceKey {
    SourceKey::new(value).expect("non-empty key")
}

fn usfm(code: &str, source: &str) -> BookInput {
    BookInput::Usfm {
        source_key: key(&format!("{code}.usfm")),
        book: id(code),
        source: source.to_string(),
    }
}

fn seeded() -> Braid {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(vec![
            usfm("GEN", GEN_SOURCE),
            usfm("EXO", EXO_SOURCE),
        ]))
        .expect("two books");
    resident
}

/// Mixed scope, one unbaselined book: the whole call must refuse, listing
/// every offender, and leave resident state -- snapshot id and every token --
/// byte-identical to before the call.
#[test]
fn revert_atomicity_mixed_scope_one_unbaselined_book_refuses_listing_all_offenders() {
    let mut resident = seeded();
    // Only GEN gets a baseline; EXO never does.
    resident.set_baseline(usfm("GEN", GEN_SOURCE)).unwrap();
    resident.update_book(usfm("GEN", GEN_EDITED)).unwrap();

    let before_snapshot = resident.expected_snapshot_id();
    let before_gen_token =
        resident.to_tokens(braid::Scope::book(id("GEN"))).unwrap()[0].tokens[0].clone();

    let error = resident
        .revert_to_baseline(CorpusScope::All)
        .expect_err("EXO has no baseline");
    assert_eq!(
        error,
        BaselineError::MissingBaseline {
            books: vec![id("EXO")]
        }
    );

    assert_eq!(resident.expected_snapshot_id(), before_snapshot);
    let after_gen_token =
        resident.to_tokens(braid::Scope::book(id("GEN"))).unwrap()[0].tokens[0].clone();
    assert_eq!(
        after_gen_token, before_gen_token,
        "GEN must be untouched even though it alone had a baseline"
    );
}

/// Every book missing a baseline is named, not just the first.
#[test]
fn every_offending_book_is_listed_not_just_the_first() {
    let mut resident = seeded();
    // Neither book has a baseline.
    let mut error = resident
        .revert_to_baseline(CorpusScope::All)
        .expect_err("neither book has a baseline");
    if let BaselineError::MissingBaseline { books } = &mut error {
        books.sort_by_key(|book| book.to_string());
    }
    assert_eq!(
        error,
        BaselineError::MissingBaseline {
            books: vec![id("EXO"), id("GEN")]
        }
    );
}

/// Relabels a parsed token stream's ids with the given prefix, keeping every
/// other fact (kind, text, marker, sid) identical -- an editor's own opaque
/// ids, the same relabelling `publication.rs`'s own identity tests use.
fn relabelled(source: &str, prefix: &str) -> Vec<OwnedToken> {
    parse(source)
        .tokens
        .iter()
        .map(OwnedToken::from_parsed)
        .collect::<Vec<_>>()
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let mut working = FormatToken::from(token);
            working.id = Some(format!("{prefix}-{index}"));
            OwnedToken::from_format_token(&working, Some(token)).expect("relabelled")
        })
        .collect()
}

fn token_push(code: &str, tokens: Vec<OwnedToken>) -> BookInput {
    BookInput::Tokens(BookTokensInput {
        source_key: key(&format!("{code}.usfm")),
        book: id(code),
        tokens,
        line_ending: LineEnding::Lf,
    })
}

/// Reverted tokens carry their ORIGINAL stable ids, not merely equal content
/// -- the baseline's own tokens are reinstalled verbatim, never re-parsed or
/// re-minted. Content-identical positional re-parses would coincidentally
/// re-derive the same ids, so this drives the token-push lane with the
/// editor's own distinctly-prefixed opaque ids to actually distinguish
/// "same ids" from "merely equal content".
#[test]
fn revert_identity_reinstalls_the_baselines_own_token_ids() {
    let mut resident = seeded();
    resident
        .update_book(token_push("GEN", relabelled(GEN_SOURCE, "orig")))
        .unwrap();
    resident
        .set_baseline(token_push("GEN", relabelled(GEN_SOURCE, "orig")))
        .unwrap();
    let baseline_ids: Vec<String> = resident.to_tokens(braid::Scope::book(id("GEN"))).unwrap()[0]
        .tokens
        .iter()
        .map(|token| token.id().as_str().to_string())
        .collect();
    assert!(baseline_ids.iter().all(|id| id.starts_with("orig-")));

    resident
        .update_book(token_push("GEN", relabelled(GEN_EDITED, "edited")))
        .unwrap();
    let edited_ids: Vec<String> = resident.to_tokens(braid::Scope::book(id("GEN"))).unwrap()[0]
        .tokens
        .iter()
        .map(|token| token.id().as_str().to_string())
        .collect();
    assert_ne!(edited_ids, baseline_ids);

    resident
        .revert_to_baseline(CorpusScope::Book(id("GEN")))
        .expect("GEN has a baseline");
    let reverted_ids: Vec<String> = resident.to_tokens(braid::Scope::book(id("GEN"))).unwrap()[0]
        .tokens
        .iter()
        .map(|token| token.id().as_str().to_string())
        .collect();
    assert_eq!(
        reverted_ids, baseline_ids,
        "revert must reinstall the baseline's own original ids, not merely equal content"
    );
}

/// A book whose current content already equals its baseline is a no-op:
/// absent from `changed`, not an error.
#[test]
fn a_clean_book_is_a_no_op_absent_from_changed() {
    let mut resident = seeded();
    resident.set_baseline(usfm("GEN", GEN_SOURCE)).unwrap();
    resident.set_baseline(usfm("EXO", EXO_SOURCE)).unwrap();
    assert!(!resident.is_dirty(CorpusScope::All).unwrap());

    let before_snapshot = resident.expected_snapshot_id();
    let effect = resident
        .revert_to_baseline(CorpusScope::All)
        .expect("both books are baselined and already clean");
    assert!(
        effect.changed.is_empty(),
        "a clean book must not appear in changed"
    );
    assert_eq!(effect.snapshot_id, before_snapshot);
}

/// The mixed case: one book changed, one clean -- only the changed one
/// appears in `changed`, and the clean one's tokens are untouched.
#[test]
fn only_the_actually_changed_book_appears_in_changed() {
    let mut resident = seeded();
    resident.set_baseline(usfm("GEN", GEN_SOURCE)).unwrap();
    resident.set_baseline(usfm("EXO", EXO_SOURCE)).unwrap();
    resident.update_book(usfm("GEN", GEN_EDITED)).unwrap();

    let effect = resident
        .revert_to_baseline(CorpusScope::All)
        .expect("both books are baselined");
    assert_eq!(effect.changed, vec![braid::Scope::book(id("GEN"))]);
    assert!(!resident.is_dirty(CorpusScope::All).unwrap());
}

/// After a successful revert, `is_dirty` is false and `diff_baseline`
/// reports equality -- the baseline slot itself is untouched, not cleared.
#[test]
fn after_revert_is_dirty_is_false_and_diff_baseline_reports_equality() {
    let mut resident = seeded();
    resident.set_baseline(usfm("GEN", GEN_SOURCE)).unwrap();
    resident.update_book(usfm("GEN", GEN_EDITED)).unwrap();
    assert!(resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());

    resident
        .revert_to_baseline(CorpusScope::Book(id("GEN")))
        .expect("GEN has a baseline");
    assert!(!resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());

    let diff = match resident
        .diff_baseline(CorpusScope::Book(id("GEN")))
        .unwrap()
    {
        braid::ScopedOutput::Single(skeleton) => skeleton,
        braid::ScopedOutput::All(_) => panic!("expected a single-scope diff"),
    };
    assert!(
        diff.units
            .iter()
            .all(|unit| matches!(unit.status, usfm_onion::diff::DecisionStatus::Unchanged)),
        "a reverted book must diff as fully unchanged against its own baseline"
    );
}

/// Chapter-scope revert is not implemented in this phase and refuses with a
/// clear, typed error rather than reverting the whole book or one run in
/// isolation.
#[test]
fn chapter_scope_revert_refuses() {
    let mut resident = seeded();
    resident.set_baseline(usfm("GEN", GEN_SOURCE)).unwrap();
    resident.update_book(usfm("GEN", GEN_EDITED)).unwrap();

    let target = ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into()));
    let error = resident
        .revert_to_baseline(CorpusScope::Chapter(target.clone()))
        .expect_err("chapter scope is unsupported");
    assert_eq!(error, BaselineError::ChapterScopeUnsupported(target));
    // The refusal must leave resident state untouched.
    assert!(resident.is_dirty(CorpusScope::Book(id("GEN"))).unwrap());
}

/// A book named by the scope that is simply not resident refuses via the
/// ordinary scope-lookup path, composed as `BaselineError::Scope`.
#[test]
fn a_book_not_resident_refuses_via_scope_error() {
    let mut resident = seeded();
    let error = resident
        .revert_to_baseline(CorpusScope::Book(id("LEV")))
        .expect_err("LEV is not resident");
    assert_eq!(
        error,
        BaselineError::Scope(braid::ScopeError::BookNotFound(id("LEV")))
    );
}
