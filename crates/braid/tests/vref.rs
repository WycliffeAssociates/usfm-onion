//! Resident `vref_index`: content-identical to the stateless whole-book
//! projection, with per-chapter-run caching underneath.
//!
//! Every non-corpus-scale assertion runs in both ingest lanes — a parsed
//! USFM book (whose tokens carry parse-assigned positional ids) and a caller
//! token push (whose tokens carry the editor's own ids) — because the
//! projection reads each token's own id (segment anchors) and sid (entry
//! keys), and the two lanes populate those differently.

use braid::{
    BookInput, BookTokensInput, Braid, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusScope, LineEnding, ScopeError, ScopedOutput, SourceKey, VrefEntry,
};
use usfm_onion::lint::{LintOptions, LintScope};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};
use usfm_onion::vref::tokens_to_vref_index;

/// Exercises the scope rules the projection cares about: plain verses,
/// a stripped note, a verse-spanning paragraph break, a verse bridge, and
/// (deliberately) out-of-order chapters/verses, so entry order is a real
/// assertion rather than incidentally already sorted.
const GEN_SOURCE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n\\c 2\n\\p\n\\v 1 Thus the heavens and the earth were finished.\\f + \\fr 2:1 \\ft a note \\f* and all their host.\n";
const EXO_SOURCE: &str =
    "\\id EXO\n\\c 1\n\\p\n\\v 1-2 These are the names of the sons of Israel.\n";

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

fn single(output: ScopedOutput<Vec<VrefEntry>>) -> Vec<VrefEntry> {
    match output {
        ScopedOutput::Single(entries) => entries,
        ScopedOutput::All(_) => panic!("expected a single-scope index"),
    }
}

/// The resident book's *current* tokens, via the public pull primitive —
/// what a fresh stateless projection is compared against, so this test never
/// silently re-derives its own idea of "the tokens" out of band.
fn current_tokens(resident: &Braid, book: BookId) -> Vec<OwnedToken> {
    resident
        .to_tokens(braid::Scope::book(book))
        .unwrap()
        .remove(0)
        .tokens
}

fn assert_matches_stateless(resident: &mut Braid, book: BookId) {
    let resident_entries = single(resident.vref_index(CorpusScope::Book(book)).unwrap());
    let stateless_entries = tokens_to_vref_index(&current_tokens(resident, book))
        .entries()
        .to_vec();
    assert_eq!(resident_entries, stateless_entries, "{book}");
}

#[test]
fn resident_matches_stateless_over_a_scope_exercising_fixture() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        assert_matches_stateless(&mut resident, id("GEN"));

        // Order is part of the contract, not incidental: this fixture crosses
        // a chapter boundary and includes a note, so a sorted or
        // note-swallowing bug would show up as an order/count mismatch, not
        // just a content one.
        let entries = single(resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap());
        assert_eq!(
            entries.iter().map(|e| e.sid.as_str()).collect::<Vec<_>>(),
            ["GEN 1:1", "GEN 1:2", "GEN 2:1"],
            "{lane:?}"
        );
        assert!(
            entries[2].projection.text.contains("and all their host"),
            "{lane:?}: note content must be stripped, not the tokens around it"
        );
        assert!(
            !entries[2].projection.text.contains("a note"),
            "{lane:?}: the note's own text must not leak into the verse"
        );
    }
}

#[test]
fn all_scope_groups_every_book_in_corpus_order() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE), ("EXO", EXO_SOURCE)]);
        match resident.vref_index(CorpusScope::All).unwrap() {
            ScopedOutput::All(books) => {
                assert_eq!(books.len(), 2, "{lane:?}");
                assert_eq!(books[0].book, id("GEN"), "{lane:?}");
                assert_eq!(books[1].book, id("EXO"), "{lane:?}");
                assert_eq!(
                    books[1]
                        .value
                        .iter()
                        .map(|e| e.sid.as_str())
                        .collect::<Vec<_>>(),
                    ["EXO 1:1-2"],
                    "{lane:?}: a bridge lexeme survives verbatim into the sid"
                );
            }
            ScopedOutput::Single(_) => panic!("expected a grouped projection"),
        }
    }
}

#[test]
fn chapter_scope_matches_the_corresponding_slice_of_the_whole_book_read() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE)]);
        let whole = single(resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap());
        let chapter_2 = single(
            resident
                .vref_index(CorpusScope::Chapter(ChapterTarget::new(
                    id("GEN"),
                    ChapterLabel::Number("2".into()),
                )))
                .unwrap(),
        );
        assert_eq!(chapter_2, vec![whole[2].clone()], "{lane:?}");
    }
}

#[test]
fn vref_index_reports_typed_scope_errors() {
    let mut resident = seeded(Lane::Parsed, vec![("GEN", GEN_SOURCE)]);
    assert_eq!(
        resident.vref_index(CorpusScope::Book(id("LEV"))),
        Err(ScopeError::BookNotFound(id("LEV")))
    );
    let absent = ChapterTarget::new(id("GEN"), ChapterLabel::Number("9".into()));
    assert_eq!(
        resident.vref_index(CorpusScope::Chapter(absent.clone())),
        Err(ScopeError::ChapterNotFound(absent))
    );
}

/// The invalidation-predicate gate, run through the actual mutation surface
/// rather than the cache internals directly: every kind of mutation the RFC
/// called out by name still leaves the resident index content-identical to a
/// fresh stateless projection over whatever the current tokens actually are.
#[test]
fn mutation_battery_stays_equivalent_to_the_stateless_projection() {
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", GEN_SOURCE), ("EXO", EXO_SOURCE)]);
        assert_matches_stateless(&mut resident, id("GEN"));

        // update_chapter: touch exactly one run.
        let edited_whole = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning God made everything.\n\\v 2 The earth was without form and void.\n\\c 2\n\\p\n\\v 1 Thus the heavens and the earth were finished.\\f + \\fr 2:1 \\ft a note \\f* and all their host.\n";
        let edited_tokens = owned(edited_whole);
        let runs = braid_chapter_runs_for_test(&edited_tokens);
        let chapter_1_range = runs
            .iter()
            .find(|(label, _)| label == "1")
            .map(|(_, range)| range.clone())
            .unwrap();
        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
                ChapterInput::Tokens(edited_tokens[chapter_1_range].to_vec()),
            )
            .unwrap();
        assert_matches_stateless(&mut resident, id("GEN"));

        // update_book: whole-book replace, including an \id-adjacent front
        // matter change.
        resident.update_book(lane.book("GEN", GEN_SOURCE)).unwrap();
        assert_matches_stateless(&mut resident, id("GEN"));

        // Duplicate/reopened `\c`: chapter-run boundaries shift under a
        // relabeled/duplicated chapter.
        let dup_source = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n";
        resident.update_book(lane.book("GEN", dup_source)).unwrap();
        assert_matches_stateless(&mut resident, id("GEN"));

        // remove_chapter + replace_corpus: residency itself changes shape.
        resident.update_book(lane.book("GEN", GEN_SOURCE)).unwrap();
        resident
            .remove_chapter(ChapterTarget::new(
                id("GEN"),
                ChapterLabel::Number("2".into()),
            ))
            .unwrap();
        assert_matches_stateless(&mut resident, id("GEN"));

        resident
            .replace_corpus(CorpusInput::new(vec![lane.book("GEN", GEN_SOURCE)]))
            .unwrap();
        assert_matches_stateless(&mut resident, id("GEN"));
    }
}

/// Test-local mirror of `crate::corpus::chapter_runs`'s label/range pairing
/// (that function is crate-private to braid) — just enough to slice a
/// correctly-sidded replacement chapter out of a whole re-parsed book, the
/// same way a real caller with its own token model already carries correct
/// sids rather than parsing an id-less fragment in isolation.
fn braid_chapter_runs_for_test(tokens: &[OwnedToken]) -> Vec<(String, std::ops::Range<usize>)> {
    usfm_onion::walker::chapter_segments(tokens)
        .into_iter()
        .filter(|segment| !segment.is_front)
        .map(|segment| {
            let label = tokens[segment.range.start + 1].source().to_string();
            (label, segment.range)
        })
        .collect()
}

/// A chapter-replacement input whose tokens carry real sids: parses the
/// *whole* edited book (so `\id` is in scope for sid derivation) and slices
/// out just the target chapter's own run, using `braid_chapter_runs_for_test`
/// above.
fn chapter_replacement(whole_edited_source: &str, label: &str) -> Vec<OwnedToken> {
    let tokens = owned(whole_edited_source);
    let (_, range) = braid_chapter_runs_for_test(&tokens)
        .into_iter()
        .find(|(found, _)| found == label)
        .unwrap_or_else(|| panic!("no chapter {label} in the edited source"));
    tokens[range].to_vec()
}

/// Clean-room review P1.1's exact repro: a chapter ending in a heading with
/// no verse-supporting paragraph, followed by a chapter opening straight
/// into a verse with no block of its own. The stateless whole-book
/// projection carries the "does not support verse" state across the `\c`
/// boundary and drops the second chapter's verse entirely; the resident
/// per-run cache must reproduce exactly that, not silently include it by
/// seeding every run as if nothing came before it.
#[test]
fn resident_matches_stateless_across_a_chapter_boundary_that_flips_verse_support() {
    const HEADING_THEN_BARE_VERSE: &str = "\\id GEN\n\\c 1\n\\s1 Heading\n\\c 2\n\\v 1 text\n";
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", HEADING_THEN_BARE_VERSE)]);
        let entries = single(resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap());
        assert_eq!(
            entries,
            Vec::<VrefEntry>::new(),
            "{lane:?}: the heading in chapter 1 does not support verse content, and that \
             carries across the \\c boundary into chapter 2's bare \\v — no entry, not \
             GEN 2:1"
        );
        assert_matches_stateless(&mut resident, id("GEN"));
    }
}

/// The mutation-battery half of P1.1: an *earlier* chapter's trailing block
/// changes from verse-supporting to not (or back), which flips a *later*,
/// completely untouched chapter's own projection — proving the cache key
/// genuinely includes the incoming state rather than only the changed
/// run's own token identity (which would incorrectly keep serving the
/// later chapter's stale cached entry).
#[test]
fn an_earlier_chapters_trailing_block_change_invalidates_a_later_untouched_chapters_cache() {
    const HEADING_THEN_BARE_VERSE: &str = "\\id GEN\n\\c 1\n\\s1 Heading\n\\c 2\n\\v 1 text\n";
    const PARAGRAPH_THEN_BARE_VERSE: &str = "\\id GEN\n\\c 1\n\\p\n\\c 2\n\\v 1 text\n";
    for lane in LANES {
        let mut resident = seeded(lane, vec![("GEN", HEADING_THEN_BARE_VERSE)]);
        // Warm the cache: chapter 2 has no entry.
        let before = single(resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap());
        assert!(before.is_empty(), "{lane:?}");

        // Replace *only* chapter 1 with a verse-supporting paragraph instead
        // of a heading, via `update_chapter` — chapter 2's own tokens are
        // untouched by construction (same objects, same ids, same
        // `TokenIdentity`), so this isolates the incoming-state half of the
        // cache key: if a stale entry were served by identity alone,
        // chapter 2 would still wrongly report no entry.
        let chapter_1 = chapter_replacement(PARAGRAPH_THEN_BARE_VERSE, "1");
        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("1".into())),
                ChapterInput::Tokens(chapter_1),
            )
            .unwrap();

        let after = single(resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap());
        assert_eq!(
            after.iter().map(|e| e.sid.as_str()).collect::<Vec<_>>(),
            ["GEN 2:1"],
            "{lane:?}: chapter 1 now supports verse content, so chapter 2's bare \\v is \
             in scope — a stale cache keyed on chapter 2's own identity alone would still \
             report no entry"
        );
        assert_matches_stateless(&mut resident, id("GEN"));
    }
}

// ---- corpus-scale gates (`cargo test -p braid --test vref -- --ignored`) -

mod corpus_scale {
    use std::fs;
    use std::path::{Path, PathBuf};

    use braid::{
        BookInput, BookTokensInput, Braid, BraidConfig, ChapterLabel, ChapterTarget, CorpusInput,
        CorpusScope, LineEnding, Scope, ScopedOutput, SourceKey,
    };
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion::parse::parse;
    use usfm_onion::token::{BookId, OwnedToken};
    use usfm_onion::vref::tokens_to_vref_index;

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
        Braid::new(BraidConfig::new(LintOptions::scoped(LintScope::Book)), {
            let mut next = 0u32;
            move || {
                next += 1;
                format!("minted-{next}")
            }
        })
    }

    fn owned(source: &str) -> Vec<OwnedToken> {
        parse(source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect()
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

    fn tokens_corpus(fixtures: &[Fixture]) -> CorpusInput {
        CorpusInput::new(
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
        )
    }

    fn assert_equals_stateless(resident: &mut Braid, book: BookId) {
        let resident_entries = match resident.vref_index(CorpusScope::Book(book)).unwrap() {
            ScopedOutput::Single(entries) => entries,
            ScopedOutput::All(_) => panic!("expected a single-scope index"),
        };
        let current = resident
            .to_tokens(Scope::book(book))
            .unwrap()
            .remove(0)
            .tokens;
        let stateless = tokens_to_vref_index(&current).entries().to_vec();
        assert_eq!(resident_entries, stateless, "{book}");
    }

    /// The equivalence gate at corpus scale, both lanes, including after a
    /// mutation battery covering every sharp edge the RFC named: a chapter
    /// edit (`update_chapter`), a whole-book replace (`update_book`), an
    /// `\id`-adjacent whole-book replace, a duplicate-`\c` fixture, and
    /// removal/replacement of residency itself.
    #[test]
    #[ignore = "corpus-scale"]
    fn resident_vref_index_equals_stateless_over_the_whole_corpus_through_a_mutation_battery() {
        let fixtures = fixtures();

        for corpus in [parsed_corpus(&fixtures), tokens_corpus(&fixtures)] {
            let mut resident = braid();
            resident.replace_corpus(corpus).unwrap();
            for fixture in &fixtures {
                assert_equals_stateless(&mut resident, fixture.book);
            }
        }

        // The mutation battery runs once, on the parsed lane, against a
        // handful of representative books.
        let mut resident = braid();
        resident.replace_corpus(parsed_corpus(&fixtures)).unwrap();

        for code in ["GEN", "PSA", "REV"] {
            let book = BookId::from_str(code).unwrap();
            let label = resident
                .chapter_labels(book)
                .unwrap()
                .into_iter()
                .find(|label| matches!(label, ChapterLabel::Number(_)))
                .unwrap_or_else(|| panic!("{code} has no numbered chapter"));
            let target = ChapterTarget::new(book, label);
            let current = match resident
                .to_usfm(CorpusScope::Chapter(target.clone()))
                .unwrap()
            {
                ScopedOutput::Single(value) => value,
                ScopedOutput::All(_) => unreachable!(),
            };
            resident
                .update_chapter(
                    target,
                    braid::ChapterInput::Tokens(owned(&format!(
                        "{current}\\p\n\\v 99 An appended verse for the equivalence gate.\n"
                    ))),
                )
                .unwrap();
            assert_equals_stateless(&mut resident, book);
        }

        // Whole-book replace, including an \id-adjacent one (front matter is
        // part of the replacement).
        let gen_fixture = fixtures
            .iter()
            .find(|f| f.book == BookId::from_str("GEN").unwrap())
            .unwrap();
        resident
            .update_book(BookInput::Usfm {
                source_key: gen_fixture.source_key.clone(),
                book: gen_fixture.book,
                source: gen_fixture.source.clone(),
            })
            .unwrap();
        assert_equals_stateless(&mut resident, gen_fixture.book);

        // Duplicate/reopened `\c`.
        let dup_book = BookId::from_str("EXO").unwrap();
        let dup_source =
            "\\id EXO\n\\c 1\n\\p\n\\v 1 first opening.\n\\c 1\n\\p\n\\v 1 reopened.\n";
        resident
            .update_book(BookInput::Usfm {
                source_key: SourceKey::new("02-EXO.usfm").unwrap(),
                book: dup_book,
                source: dup_source.to_string(),
            })
            .unwrap();
        assert_equals_stateless(&mut resident, dup_book);

        // Removal, then a fresh replacement corpus.
        resident.remove_book(dup_book);
        resident.replace_corpus(parsed_corpus(&fixtures)).unwrap();
        for fixture in &fixtures {
            assert_equals_stateless(&mut resident, fixture.book);
        }
    }

    /// Informal wall-clock numbers for the ledger, release build only: one
    /// chapter edit followed by a whole-book resident read, against a
    /// stateless whole-book recompute from scratch — the editor's own
    /// baseline call. Not a pass/fail timing gate (the standing rule is
    /// counters/observables for correctness, not timing assertions); this
    /// exists to print numbers, not to enforce them.
    #[test]
    #[ignore = "corpus-scale"]
    fn psalms_informal_timing() {
        let fixtures = fixtures();
        let psa = fixtures
            .iter()
            .find(|f| f.book == BookId::from_str("PSA").unwrap())
            .expect("en_ulb carries Psalms");

        let mut resident = braid();
        resident
            .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
                source_key: psa.source_key.clone(),
                book: psa.book,
                source: psa.source.clone(),
            }]))
            .unwrap();
        // Warm the cache once, matching steady-state editor usage (the very
        // first read after cold-open is not the number being quoted here).
        resident.vref_index(CorpusScope::Book(psa.book)).unwrap();

        let label = resident
            .chapter_labels(psa.book)
            .unwrap()
            .into_iter()
            .find(|label| matches!(label, ChapterLabel::Number(_)))
            .expect("Psalms has a numbered chapter");
        let target = ChapterTarget::new(psa.book, label);
        let current = match resident
            .to_usfm(CorpusScope::Chapter(target.clone()))
            .unwrap()
        {
            ScopedOutput::Single(value) => value,
            ScopedOutput::All(_) => unreachable!(),
        };
        resident
            .update_chapter(
                target,
                braid::ChapterInput::Tokens(owned(&format!("{current}\\p\n\\v 99 Edited.\n"))),
            )
            .unwrap();

        let start = std::time::Instant::now();
        let resident_entries = match resident.vref_index(CorpusScope::Book(psa.book)).unwrap() {
            ScopedOutput::Single(entries) => entries,
            ScopedOutput::All(_) => unreachable!(),
        };
        let resident_elapsed = start.elapsed();

        let start = std::time::Instant::now();
        let stateless_entries = tokens_to_vref_index(&owned(&psa.source)).entries().to_vec();
        let stateless_elapsed = start.elapsed();

        eprintln!(
            "Psalms one-chapter-edit resident vref_index: {resident_elapsed:?} ({} entries); \
             stateless whole-book recompute: {stateless_elapsed:?} ({} entries); \
             editor's stateless baseline quoted at ~520-630ms per edit",
            resident_entries.len(),
            stateless_entries.len(),
        );
    }
}
