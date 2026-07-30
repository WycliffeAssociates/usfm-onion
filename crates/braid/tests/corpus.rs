//! Corpus-scale residency gate over the `en_ulb` example corpus (66 books).
//!
//! `#[ignore]`d by default: it parses, hashes, and re-emits a whole Bible, which
//! is far more work than a unit test should do on every `cargo test`. Run with
//! `cargo test -p braid --test corpus -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusScope, LineEnding, Scope, ScopedOutput, SourceKey,
};
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken, UsfmToken};

/// One corpus book: the declared id from its filename, its source key, and its
/// exact bytes.
struct Fixture {
    book: BookId,
    source_key: SourceKey,
    source: String,
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../example-corpora/en_ulb")
        .canonicalize()
        .expect("crate sits two levels below the repo root")
}

/// The declared book id comes from the filename (`01-GEN.usfm`), which is what a
/// real manifest supplies — deliberately not from the file's own `\id`, since
/// that is editable content that may disagree.
fn fixtures() -> Vec<Fixture> {
    let mut paths: Vec<PathBuf> = fs::read_dir(corpus_root())
        .expect("corpus directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
        .collect();
    paths.sort();

    let fixtures: Vec<Fixture> = paths
        .into_iter()
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?;
            let code = name.split('-').nth(1)?;
            Some(Fixture {
                book: BookId::from_str(code)?,
                source_key: SourceKey::new(name)?,
                source: fs::read_to_string(&path).ok()?,
            })
        })
        .collect();
    assert_eq!(fixtures.len(), 66, "en_ulb is a 66-book corpus");
    fixtures
}

fn braid() -> Braid {
    Braid::new(BraidConfig::new(LintOptions::scoped(LintScope::Book)))
}

fn parsed_corpus(fixtures: &[Fixture]) -> CorpusInput {
    CorpusInput::new(
        fixtures
            .iter()
            .map(|fixture| BookInput::Usfm {
                source_key: fixture.source_key.clone(),
                book: fixture.book,
                source: fixture.source.clone(),
            })
            .collect(),
    )
}

fn owned(source: &str) -> Vec<OwnedToken> {
    parse(source)
        .tokens
        .iter()
        .map(OwnedToken::from_parsed)
        .collect()
}

/// A token's content, excluding the two things a stream's origin decides rather
/// than its content: the stable id (positional to whichever parse produced it)
/// and the sid (derived at parse time from the book code in that same stream, so
/// a chapter fragment parsed on its own has none). Both are addresses, not
/// content — a consumer needing canonical anchors derives them from the resident
/// stream with `usfm_onion::diff::derive_canonical_sids`.
fn semantics(tokens: &[OwnedToken]) -> Vec<(usfm_onion::token::TokenKind, &str, Option<&str>)> {
    tokens
        .iter()
        .map(|token| (UsfmToken::kind(token), token.source(), token.marker_name()))
        .collect()
}

fn all_sources(resident: &Braid) -> Vec<(BookId, String)> {
    match resident.to_usfm(CorpusScope::All).expect("all scope") {
        ScopedOutput::All(books) => books
            .into_iter()
            .map(|entry| (entry.book, entry.value))
            .collect(),
        ScopedOutput::Single(_) => panic!("All scope must group"),
    }
}

#[test]
#[ignore = "corpus-scale"]
fn a_whole_corpus_is_resident_byte_identically_in_both_ingest_lanes() {
    let fixtures = fixtures();

    let mut parsed = braid();
    let effect = parsed.replace_corpus(parsed_corpus(&fixtures)).unwrap();
    assert_eq!(effect.changed.len(), 66, "every book is new");
    assert!(effect.removed.is_empty());

    // Authoritative bytes are the supplied bytes, in caller order.
    assert_eq!(
        all_sources(&parsed),
        fixtures
            .iter()
            .map(|fixture| (fixture.book, fixture.source.clone()))
            .collect::<Vec<_>>()
    );

    // The token-push lane derives the same bytes from tokens plus the declared
    // ending, so both lanes agree on every book's hash and on corpus identity.
    let mut pushed = braid();
    pushed
        .replace_corpus(CorpusInput::new(
            fixtures
                .iter()
                .map(|fixture| {
                    BookInput::Tokens(BookTokensInput {
                        source_key: fixture.source_key.clone(),
                        book: fixture.book,
                        tokens: owned(&fixture.source),
                        line_ending: LineEnding::detect(&fixture.source),
                    })
                })
                .collect(),
        ))
        .unwrap();
    assert_eq!(all_sources(&pushed), all_sources(&parsed));
    assert_eq!(pushed.books(), parsed.books());
    assert_eq!(pushed.expected_snapshot_id(), parsed.expected_snapshot_id());

    // Content-derived identity: reseeding the same ordered corpus is a no-op and
    // a fresh braid reproduces the id.
    let before = parsed.expected_snapshot_id();
    assert!(
        parsed
            .replace_corpus(parsed_corpus(&fixtures))
            .unwrap()
            .is_noop()
    );
    assert_eq!(parsed.expected_snapshot_id(), before);
    let mut again = braid();
    again.replace_corpus(parsed_corpus(&fixtures)).unwrap();
    assert_eq!(again.expected_snapshot_id(), before);
}

#[test]
#[ignore = "corpus-scale"]
fn mutated_books_pull_what_a_fresh_parse_of_their_new_bytes_would_yield() {
    let fixtures = fixtures();
    let mut resident = braid();
    resident.replace_corpus(parsed_corpus(&fixtures)).unwrap();

    // Three books, one chapter edit each (one splice per book: a second
    // separately parsed fragment would collide on positional token ids).
    let edited: Vec<BookId> = ["GEN", "PSA", "REV"]
        .into_iter()
        .map(|code| BookId::from_str(code).unwrap())
        .collect();

    for book in &edited {
        let label = match resident
            .chapter_labels(*book)
            .unwrap()
            .into_iter()
            .find(|label| matches!(label, ChapterLabel::Number(_)))
        {
            Some(label) => label,
            None => panic!("{book} has no numbered chapter"),
        };
        let target = ChapterTarget::new(*book, label.clone());
        let current = match resident
            .to_usfm(CorpusScope::Chapter(target.clone()))
            .unwrap()
        {
            ScopedOutput::Single(value) => value,
            ScopedOutput::All(_) => unreachable!(),
        };
        let effect = resident
            .update_chapter(
                target,
                ChapterInput::Usfm {
                    source: format!("{current}\\p\n\\v 99 An appended verse.\n"),
                },
            )
            .unwrap();
        assert_eq!(
            effect.changed,
            vec![Scope::chapter(*book, label)],
            "one chapter, exactly"
        );
    }

    for (book, source) in all_sources(&resident) {
        let pulled = resident
            .to_tokens(Scope::book(book))
            .unwrap()
            .remove(0)
            .tokens;
        let reparsed = owned(&source);

        if edited.contains(&book) {
            // An edited book's stream is the splice, so its ids are the caller's
            // and its semantics are what re-parsing its new bytes yields.
            assert_eq!(semantics(&pulled), semantics(&reparsed), "{book}");
        } else {
            // An untouched book is exactly a parse of its own bytes, ids
            // included.
            assert_eq!(pulled, reparsed, "{book}");
        }
    }

    // Only the three edited books moved; every other hash is untouched.
    let mut pristine = braid();
    pristine.replace_corpus(parsed_corpus(&fixtures)).unwrap();
    let changed: Vec<BookId> = resident
        .books()
        .into_iter()
        .zip(pristine.books())
        .filter(|(now, before)| now.source_hash != before.source_hash)
        .map(|(now, _)| now.book)
        .collect();
    assert_eq!(changed, edited);
    assert_ne!(
        resident.expected_snapshot_id(),
        pristine.expected_snapshot_id()
    );
}
