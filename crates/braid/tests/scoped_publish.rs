//! `Braid::publish_scope`: per-book packed containers consumable verbatim as
//! `RestoreRecord`s, always encoded, never reading or invalidating the
//! handle's own `PublicationCache`.

use braid::{
    BookInput, Braid, BraidConfig, CorpusInput, CorpusScope, RestoreRecord, ScopeError,
    ScopedPublishError, SourceKey,
};
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::token::BookId;

const GEN: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\c 2\n\\p\n\\v 1 Thus.\n";
const EXO: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

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
        .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN), usfm("EXO", EXO)]))
        .expect("two books");
    resident
}

/// The scoped output must be feedable straight to `restore_corpus` (via
/// `restore_packed_books`) into a FRESH `Braid`, and the restored corpus must
/// agree with the original both in tokens and in findings — this is the
/// entire reason the container shape is `RestoreRecord`-compatible rather
/// than `PublishedCorpus`-shaped.
#[test]
fn a_scoped_publication_round_trips_through_restore_corpus_into_a_fresh_braid() {
    let mut resident = seeded();
    let expected = resident.lint();
    let expected_books: Vec<(BookId, usize, Vec<usfm_onion::lint::LintCode>)> = expected
        .books
        .iter()
        .map(|book| {
            (
                book.book,
                book.tokens.len(),
                book.result.issues.iter().map(|issue| issue.code).collect(),
            )
        })
        .collect();
    drop(expected);

    let scoped = resident.publish_scope(CorpusScope::All).expect("publishes");
    assert_eq!(scoped.books.len(), 2, "both books in scope");
    assert_eq!(
        scoped.snapshot_id,
        format!("{:016x}", resident.expected_snapshot_id().0),
        "snapshot_id must equal lint()'s corpus identity"
    );

    let records: Vec<RestoreRecord> = scoped
        .books
        .iter()
        .map(|book| RestoreRecord {
            path: format!("{}.usfm", book.book),
            packed: book.packed.clone(),
            source: book.source.clone().into_bytes(),
        })
        .collect();

    let mut fresh = braid();
    let report = fresh
        .restore_packed_books(&records)
        .expect("a scoped publication restores verbatim");
    assert_eq!(report.seeded.len(), 2, "{:?}", report.rejected);
    assert!(report.rejected.is_empty(), "{:?}", report.rejected);

    let restored = fresh.lint();
    let restored_books: Vec<(BookId, usize, Vec<usfm_onion::lint::LintCode>)> = restored
        .books
        .iter()
        .map(|book| {
            (
                book.book,
                book.tokens.len(),
                book.result.issues.iter().map(|issue| issue.code).collect(),
            )
        })
        .collect();
    assert_eq!(
        restored_books, expected_books,
        "token counts and finding codes must match, book for book"
    );
}

/// A clean, previously-published book must still yield fresh bytes and a
/// source from `publish_scope` -- there is no splice-reuse arm and no
/// `encoded: false` case, because the caller is by definition asking for
/// bytes it does not already hold.
#[test]
fn a_clean_previously_published_book_still_always_encodes() {
    let mut resident = seeded();
    // Publish once through the ordinary corpus-wide publish, which does
    // populate the reuse cache -- proving `publish_scope` does not consult
    // or depend on it.
    let first = resident.publish().expect("publishes");
    assert!(first.books.iter().all(|book| book.encoded));

    let scoped = resident
        .publish_scope(CorpusScope::Book(id("GEN")))
        .expect("publishes");
    assert_eq!(scoped.books.len(), 1);
    let book = &scoped.books[0];
    assert!(!book.packed.is_empty(), "always encodes real bytes");
    assert!(!book.source.is_empty(), "always carries source");
    assert_eq!(book.book, "GEN");
}

/// `snapshot_id` is the corpus identity from the same `lint()` read
/// `publish_scope` uses internally -- callers assert it against
/// `MutationEffect::snapshot_id` to detect races.
#[test]
fn snapshot_id_equals_lints_own_corpus_identity() {
    let mut resident = seeded();
    let expected = format!("{:016x}", resident.lint().id.0);
    let scoped = resident
        .publish_scope(CorpusScope::Book(id("EXO")))
        .expect("publishes");
    assert_eq!(scoped.snapshot_id, expected);
}

/// A chapter scope resolves to its book -- containers are book-grain.
#[test]
fn chapter_scope_resolves_to_its_book() {
    let mut resident = seeded();
    let scoped = resident
        .publish_scope(CorpusScope::Chapter(braid::ChapterTarget::new(
            id("GEN"),
            braid::ChapterLabel::Number("1".into()),
        )))
        .expect("publishes");
    assert_eq!(scoped.books.len(), 1);
    assert_eq!(scoped.books[0].book, "GEN");
}

/// Scope resolution errors reuse `ScopeError`, composed into
/// `ScopedPublishError::Scope`.
#[test]
fn an_unresolvable_scope_reports_the_typed_scope_error() {
    let mut resident = seeded();
    let error = resident
        .publish_scope(CorpusScope::Book(id("LEV")))
        .expect_err("LEV is not resident");
    assert_eq!(
        error,
        ScopedPublishError::Scope(ScopeError::BookNotFound(id("LEV")))
    );
}

/// `publish_scope` must not read or invalidate the handle's own
/// `PublicationCache`: a corpus-wide `publish()` right after it must still
/// see every book as reusable (unencoded), proving the scoped call left the
/// cache exactly as it was.
#[test]
fn publish_scope_does_not_touch_the_publication_cache() {
    let mut resident = seeded();
    let first = resident.publish().expect("publishes");
    assert!(first.books.iter().all(|book| book.encoded));

    // A scoped publish of one book -- if this touched the cache, the next
    // ordinary publish below would see it as invalidated.
    resident
        .publish_scope(CorpusScope::Book(id("GEN")))
        .expect("publishes");

    let second = resident.publish().expect("republishes");
    assert!(
        second.books.iter().all(|book| !book.encoded),
        "the ordinary publish cache must still consider every book reused"
    );
    assert_eq!(second.bytes, first.bytes);
}
