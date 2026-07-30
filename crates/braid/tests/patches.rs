//! Snapshot-bound patch resolution, preview, and application.
//!
//! A patch is a resolved fix: the same recipe core's linter attached to a
//! finding, flattened into token operations against the resident snapshot and
//! addressable by an id that goes stale on its own when the corpus moves. Both
//! ingest lanes are exercised, because the parsed lane's tokens carry
//! parse-assigned ids while the caller lane's carry the editor's own — and a fix
//! addresses its target by id.

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterLabel, CorpusInput, CorpusScope,
    LineEnding, Patch, PatchError, PatchId, PatchOp, ScopedOutput, SourceKey,
};
use usfm_onion::lint::{LintCode, LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken, TokenKind};

/// One book, two whitespace findings that both carry a fix: `\p` jammed onto the
/// end of verse 1's text, and `\q1` jammed onto verse 2's.
const GEN_SOURCE: &str =
    "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth.\\q1\n\\v 3 Light.\n";

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

fn tokens_input(code: &str, source: &str) -> BookInput {
    BookInput::Tokens(BookTokensInput {
        source_key: key(&format!("{code}.usfm")),
        book: id(code),
        tokens: owned(source),
        line_ending: LineEnding::Lf,
    })
}

const LANES: [fn(&str, &str) -> BookInput; 2] = [usfm, tokens_input];

fn seeded(lane: fn(&str, &str) -> BookInput, books: Vec<(&str, &str)>) -> Braid {
    let mut resident = braid();
    resident
        .replace_corpus(CorpusInput::new(
            books
                .into_iter()
                .map(|(code, source)| lane(code, source))
                .collect(),
        ))
        .expect("distinct books");
    resident.lint();
    resident
}

fn source_of(resident: &Braid, book: &str) -> String {
    match resident.to_usfm(CorpusScope::Book(id(book))) {
        Ok(ScopedOutput::Single(source)) => source,
        other => panic!("expected one book's bytes, got {other:?}"),
    }
}

/// Every fix core attached becomes exactly one addressable patch, flattened as
/// the freeze specifies: one replace row, at the target token's own position,
/// carrying the replacement template.
#[test]
fn every_fix_resolves_to_one_addressable_patch() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        let snapshot = resident.lint();
        let fixes: Vec<_> = snapshot.books[0]
            .result
            .issues
            .iter()
            .filter(|issue| issue.fix.is_some())
            .collect();
        assert_eq!(fixes.len(), 2, "both jammed markers carry a fix");
        let positions: Vec<u32> = fixes.iter().map(|issue| issue.position).collect();
        let tokens = snapshot.books[0].tokens.to_vec();
        drop(snapshot);

        let patches = resident.patches();
        assert_eq!(patches.len(), 2);
        for (ordinal, patch) in patches.iter().enumerate() {
            assert_eq!(patch.id.ordinal, ordinal as u32);
            assert_eq!(patch.id.snapshot, resident.expected_snapshot_id());
            assert_eq!(patch.book, id("GEN"));
            // The census's finding: the fix's own code is a remedy name, never
            // the finding's lint code.
            assert!(patch.code.starts_with("insert-"));
            assert!(patch.label.starts_with("Insert"));
            assert!(patch.label_params.is_empty());

            assert_eq!(patch.rows.len(), 1);
            let row = &patch.rows[0];
            assert_eq!(row.op, PatchOp::Replace);
            assert_eq!(row.position, positions[ordinal]);
            let template = row.template.as_ref().expect("a replace places a template");
            assert_eq!(template.kind, TokenKind::Marker);
            // The remedy is the target token's own spelling with the missing
            // whitespace inserted, so the marker and anchor are unchanged.
            assert_eq!(
                template.marker.as_deref(),
                tokens[row.position as usize].marker_name()
            );
            assert_eq!(template.sid.as_deref(), tokens[row.position as usize].sid());
            assert!(template.text.contains('\n'));

            assert_eq!(resident.patch(patch.id).as_ref(), Ok(patch));
        }
    }
}

/// Applying a patch is an ordinary mutation: the bytes change exactly the way
/// the template said, the effect names the chapter the fix landed in, identity
/// moves, and the patched book is scheduled for recompute.
///
/// It also pins a real limitation of the three whitespace fixes core produces
/// today, found while building this: their remedy prepends the whitespace into
/// the *marker token's own source*, so the emitted bytes become correct while
/// the token stream keeps a marker token that no lexer would ever produce
/// (`"\n\\p"`). The rule that fired reads the *previous* token's trailing
/// character, so it still fires on the patched token stream even though a parse
/// of the patched bytes is clean. Braid applies core's recipe faithfully and
/// never reparses (reparsing would destroy the caller's token identity), so this
/// is core's fix shape to change, not braid's application to work around — see
/// the ledger entry for the recommended reshape.
#[test]
fn applying_a_patch_rewrites_the_bytes_the_template_named() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        let before = source_of(&resident, "GEN");
        let identity_before = resident.expected_snapshot_id();
        let patch = resident.patches().remove(0);

        let effect = resident
            .apply_patch(patch.id)
            .expect("a fresh patch applies");
        assert!(!effect.is_noop());
        assert_ne!(effect.snapshot_id, identity_before);
        assert_eq!(
            effect.changed,
            vec![braid::Scope::chapter(
                id("GEN"),
                ChapterLabel::Number("1".into())
            )],
            "the fix landed in chapter one"
        );
        assert_eq!(resident.dirty_books(), vec![id("GEN")]);

        let after = source_of(&resident, "GEN");
        assert_ne!(after, before);
        assert_eq!(after, before.replace("beginning.\\p", "beginning.\n\\p"));

        // The bytes are now what the rule asked for: a parse of them is clean
        // for this code at this position.
        let reparsed = usfm_onion::lint::lint_tokens(
            &parse(&after).tokens,
            LintOptions::scoped(LintScope::Book),
        );
        assert_eq!(
            reparsed
                .issues
                .iter()
                .filter(|issue| issue.code == LintCode::MissingWhitespaceBeforeMarker)
                .count(),
            1,
            "only the second jammed marker is left in the patched bytes"
        );

        // The token stream, however, still reports it: the remedy moved the
        // newline inside the marker token rather than in front of it.
        let snapshot = resident.lint();
        assert_eq!(
            snapshot.books[0]
                .result
                .issues
                .iter()
                .filter(|issue| issue.code == LintCode::MissingWhitespaceBeforeMarker)
                .count(),
            2,
            "core's whitespace remedy does not satisfy the rule on tokens"
        );
        drop(snapshot);
        assert_eq!(
            resident.patches().len(),
            2,
            "so both fixes are re-resolved against the new snapshot"
        );
    }
}

/// Preview and apply share one code path against one snapshot, so a preview is
/// exactly what apply would commit — and previewing mutates nothing.
#[test]
fn preview_shows_the_apply_without_performing_it() {
    let mut resident = seeded(LANES[0], vec![("GEN", GEN_SOURCE)]);
    let patch = resident.patches().remove(0);
    let identity = resident.expected_snapshot_id();
    let books = resident.books();

    let previewed = resident.preview_patch(patch.id).expect("preview");
    assert_eq!(resident.expected_snapshot_id(), identity);
    assert_eq!(resident.books(), books);
    assert!(resident.dirty_books().is_empty());

    resident.apply_patch(patch.id).expect("apply");
    let applied = resident
        .to_tokens(braid::Scope::book(id("GEN")))
        .expect("the patched book")
        .remove(0)
        .tokens;
    assert_eq!(previewed, applied);
}

/// The census found six real fixture cases where two findings propose different
/// whole-token replacements for the *same* token. Applying one must make the
/// other refuse rather than silently drop an edit — which is exactly what
/// snapshot-bound identity buys.
#[test]
fn a_sibling_patch_on_the_same_token_goes_stale() {
    // `\p` jammed onto the text *and* immediately followed by another marker:
    // one token, two codes, two different whole-token replacements. This is the
    // `testData/paratextTests/MarkersMissingSpace` shape the census found.
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 should have error\\p\\nd testing \\nd*\n";
    let mut resident = seeded(LANES[0], vec![("GEN", source)]);
    let patches = resident.patches();
    assert!(patches.len() >= 2);
    assert_eq!(
        patches[0].rows[0].position, patches[1].rows[0].position,
        "both fixes target the same token"
    );
    let (first, second) = (patches[0].id, patches[1].id);

    resident.apply_patch(first).expect("the first applies");
    assert_eq!(
        resident.apply_patch(second),
        Err(PatchError::StaleSnapshot {
            expected: resident.expected_snapshot_id(),
            found: second.snapshot,
        })
    );
    // And re-applying the one that already applied is equally refused.
    assert!(matches!(
        resident.apply_patch(first),
        Err(PatchError::StaleSnapshot { .. })
    ));
}

/// Every rejection path leaves resident state provably untouched, and none of
/// them can apply half a patch.
#[test]
fn rejected_applications_are_atomic() {
    let mut resident = seeded(
        LANES[0],
        vec![("GEN", GEN_SOURCE), ("EXO", "\\id EXO\n\\c 1\n")],
    );
    let identity = resident.expected_snapshot_id();
    let books = resident.books();
    let valid = resident.patches().remove(0);

    let unknown = PatchId {
        snapshot: identity,
        ordinal: 99,
    };
    assert_eq!(
        resident.apply_patch(unknown),
        Err(PatchError::UnknownPatch(unknown))
    );

    let stale = PatchId {
        snapshot: braid::SnapshotId(valid.id.snapshot.0 ^ 1),
        ordinal: valid.id.ordinal,
    };
    assert!(matches!(
        resident.apply_patch(stale),
        Err(PatchError::StaleSnapshot { .. })
    ));

    assert_eq!(resident.expected_snapshot_id(), identity);
    assert_eq!(resident.books(), books);
    assert!(resident.dirty_books().is_empty());
    // The valid one still applies afterwards: a rejection is not a poisoned
    // state.
    assert!(resident.apply_patch(valid.id).is_ok());
}

/// A book awaiting recompute publishes no patches. Its stored positions address
/// the token stream it held when its result was computed, and after an edit that
/// stream no longer exists — so nothing stale is ever addressable.
#[test]
fn a_dirty_book_publishes_no_patches() {
    let mut resident = seeded(LANES[0], vec![("GEN", GEN_SOURCE)]);
    let before = resident.patches();
    assert_eq!(before.len(), 2);

    resident
        .update_book(usfm(
            "GEN",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n",
        ))
        .expect("an edit");
    assert_eq!(resident.dirty_books(), vec![id("GEN")]);
    assert!(resident.patches().is_empty());
    assert!(matches!(
        resident.patch(before[0].id),
        Err(PatchError::StaleSnapshot { .. })
    ));

    resident.lint();
    assert!(!resident.patches().is_empty());
}

/// Ordinals address the corpus-wide table in corpus order, so a two-book corpus
/// numbers the second book's patches after the first book's.
#[test]
fn ordinals_run_across_the_corpus_in_corpus_order() {
    let jammed = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\\p\n";
    let resident = seeded(LANES[0], vec![("GEN", GEN_SOURCE), ("EXO", jammed)]);
    let patches: Vec<Patch> = resident.patches();
    let books: Vec<BookId> = patches.iter().map(|patch| patch.book).collect();
    assert_eq!(books, vec![id("GEN"), id("GEN"), id("EXO")]);
    assert_eq!(
        patches
            .iter()
            .map(|patch| patch.id.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    // Each patch carries the hash of the book it targets, which is the half of
    // the staleness rule that survives a corpus whose identity comes back.
    for patch in &patches {
        let entry = resident
            .books()
            .into_iter()
            .find(|entry| entry.book == patch.book)
            .expect("resident");
        assert_eq!(patch.source_hash, entry.source_hash);
    }
}
