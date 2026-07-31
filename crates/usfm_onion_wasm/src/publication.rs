//! The composing adapter: braid's semantics on one side, wire's bytes on the
//! other, and the reuse cache that keeps a republication cheap.
//!
//! This is the layer the two crates are deliberately kept from being: braid never
//! learns a byte layout and wire never learns what a dirty book or a snapshot id
//! means, so *something* has to know both, and this is it. It lives here because
//! this crate is the composition root — the one place allowed to depend on both.
//!
//! Nothing here is exported to JS. The public `Braid` class and
//! `restoreCorpus` are Phase F; what exists now is the Rust composition the
//! native host and that class will both call.

use braid::{Braid, LintConfigFingerprint, LintEngineStamp, SourceHash};
use usfm_onion::token::BookId;
use usfm_onion_wire::corpus_codec::{
    CorpusSection, CorpusSectionInput, CorpusSectionTokens, EncodedCorpus, LintStamps,
    PublishedBook, encode_corpus,
};
use usfm_onion_wire::error::EncodeError;

/// One book's last published sections, with the facts that decide whether they
/// may be published again unchanged.
///
/// The source hash alone is not enough: a configuration change rewrites what a
/// book's findings *are* while leaving its bytes alone, and the published section
/// carries those findings. Keying on the stamps as well is what stops a
/// republication from serving findings the current configuration would not
/// produce.
#[derive(Debug, Clone)]
struct CachedBook {
    source_hash: SourceHash,
    stamps: LintStamps,
    published: PublishedBook,
}

/// What one publication produced.
#[derive(Debug)]
pub(crate) struct Publication {
    pub(crate) bytes: Vec<u8>,
    /// Per freshly encoded book, the exact source its sections are bound to.
    pub(crate) sources: Vec<(BookId, String)>,
    /// Books encoded this time. The rest were reused — reported rather than
    /// inferred, because "did this republication actually reuse anything" is the
    /// question the whole cache exists to answer.
    pub(crate) encoded: Vec<BookId>,
    pub(crate) reused: Vec<BookId>,
}

/// A resident corpus's publication cache.
///
/// Holds only what wire produced and what decides its reuse; no source bytes, no
/// IO, and no knowledge of what is inside a section. A host that wants the cache
/// to outlive the process persists [`Publication::bytes`] and reseeds through
/// braid's own restore path — this type is an in-memory accelerator, not a
/// storage format.
#[derive(Debug, Default)]
pub(crate) struct PublicationCache {
    books: Vec<(BookId, CachedBook)>,
}

impl PublicationCache {
    /// Publishes the resident corpus as one packed container.
    ///
    /// Recomputes exactly the dirty books' lint (braid's own rule), re-encodes
    /// exactly the books whose bytes or stamps moved, and splices the rest from
    /// the last publication. A clean corpus that has already been published
    /// therefore encodes nothing at all.
    pub(crate) fn publish(&mut self, resident: &mut Braid) -> Result<Publication, EncodeError> {
        let stamps = LintStamps {
            config_fingerprint: LintConfigFingerprint::of(&resident.config().lint).0,
            engine_stamp: LintEngineStamp::current().0,
        };

        // One `lint()` call, borrowed for the whole composition: the snapshot's
        // tokens and results are the resident ones, so nothing is cloned to get
        // them into the encoder.
        let snapshot = resident.lint();
        let mut sections = Vec::with_capacity(snapshot.books.len());
        let mut encoded = Vec::new();
        let mut reused = Vec::new();
        for book in &snapshot.books {
            let cached = self.books.iter().find(|(candidate, cached)| {
                *candidate == book.book
                    && cached.source_hash == book.source_hash
                    && cached.stamps == stamps
            });
            match cached {
                Some((_, cached)) => {
                    reused.push(book.book);
                    sections.push(CorpusSection::Cached(cached.published.as_cached()));
                }
                None => {
                    encoded.push(book.book);
                    sections.push(CorpusSection::Fresh(CorpusSectionInput {
                        book: book.book,
                        tokens: CorpusSectionTokens::Owned {
                            tokens: book.tokens,
                        },
                        findings: Some(book.result),
                    }));
                }
            }
        }

        let has_findings = snapshot
            .books
            .iter()
            .any(|book| !book.result.issues.is_empty());
        let EncodedCorpus {
            bytes,
            sources,
            books,
        } = encode_corpus(
            snapshot.id.0,
            // Stamps describe findings; a corpus that produced none is published
            // without a licence for a cache that would hold nothing.
            has_findings.then_some(stamps),
            &sections,
        )?;

        // Replaced wholesale from what the publication actually contains, so the
        // cache can never describe a book the last publication did not carry.
        self.books = books
            .into_iter()
            .map(|published| {
                let source_hash = snapshot
                    .books
                    .iter()
                    .find(|book| book.book == published.book)
                    .map(|book| book.source_hash)
                    .expect("every published book is a resident book");
                (
                    published.book,
                    CachedBook {
                        source_hash,
                        stamps,
                        published,
                    },
                )
            })
            .collect();

        Ok(Publication {
            bytes,
            sources,
            encoded,
            reused,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use braid::{
        BookInput, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget, CorpusInput, SourceKey,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion::parse::parse;
    use usfm_onion::token::OwnedToken;
    use usfm_onion_wire::corpus_codec::verify_corpus;

    fn resident() -> Braid {
        let mut next = 0u32;
        Braid::new(
            BraidConfig::new(LintOptions::scoped(LintScope::Book)),
            move || {
                next += 1;
                format!("minted-{next}")
            },
        )
    }

    fn book(code: &str) -> BookId {
        BookId::from_str(code).expect("book code")
    }

    fn usfm(code: &str, source: &str) -> BookInput {
        BookInput::Usfm {
            source_key: SourceKey::new(format!("{code}.usfm")).unwrap(),
            book: book(code),
            source: source.to_string(),
        }
    }

    /// Verifies a publication against the sources it says it is bound to, taking
    /// the unchanged ones from the caller's own copy — which is exactly what a
    /// host does: wire hands back sources only for what it encoded.
    fn verify(
        publication: &Publication,
        all: &[(BookId, &str)],
    ) -> usfm_onion_wire::corpus_codec::VerifiedCorpus {
        let sources: Vec<(BookId, &str)> = all.to_vec();
        verify_corpus(&publication.bytes, &sources).expect("a publication verifies")
    }

    const GEN: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\c 2\n\\p\n\\v 1 Thus.\n";
    const EXO: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

    /// The publish → decode → compare gate at small scale, both dimensions of the
    /// standing id rule: the parsed lane's positional ids and a caller token
    /// push's own opaque ids must both survive publication.
    #[test]
    fn a_publication_decodes_back_to_the_native_snapshot() {
        for opaque_ids in [false, true] {
            let mut resident = resident();
            resident
                .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN), usfm("EXO", EXO)]))
                .expect("two books");
            if opaque_ids {
                // A caller token push: ids are the editor's, not positional.
                let tokens: Vec<OwnedToken> = parse(EXO)
                    .tokens
                    .iter()
                    .map(OwnedToken::from_parsed)
                    .collect();
                let relabelled = braid::BookTokensInput {
                    source_key: SourceKey::new("EXO.usfm").unwrap(),
                    book: book("EXO"),
                    tokens,
                    line_ending: braid::LineEnding::Lf,
                };
                // Re-pushing the same tokens is a no-op; what matters here is the
                // lane, which the corpus gate below exercises with real edits.
                resident
                    .update_book(BookInput::Tokens(relabelled))
                    .expect("token push");
            }

            let mut cache = PublicationCache::default();
            let publication = cache.publish(&mut resident).expect("publishes");
            assert_eq!(publication.encoded, vec![book("GEN"), book("EXO")]);
            assert!(publication.reused.is_empty());

            let verified = verify(&publication, &[(book("GEN"), GEN), (book("EXO"), EXO)]);
            assert_eq!(verified.snapshot_id, resident.expected_snapshot_id().0);
            assert_eq!(
                verified.lint_stamps,
                Some(LintStamps {
                    config_fingerprint: LintConfigFingerprint::of(&resident.config().lint).0,
                    engine_stamp: LintEngineStamp::current().0,
                })
            );

            let snapshot = resident.lint();
            assert_eq!(verified.books.len(), snapshot.books.len());
            for (decoded, native) in verified.books.iter().zip(&snapshot.books) {
                assert_eq!(decoded.receipt.book, native.book.as_str());
                assert_eq!(decoded.receipt.token_count as usize, native.tokens.len());
                assert_eq!(
                    decoded.receipt.source_hash,
                    format!("{:016x}", native.source_hash.0)
                );
                // Findings, in order, every field including the fix.
                assert_eq!(decoded.findings.len(), native.result.issues.len());
                for (decoded, native) in decoded.findings.iter().zip(&native.result.issues) {
                    assert_eq!(decoded.code, native.code);
                    assert_eq!(decoded.token_id, native.token_id);
                    assert_eq!(decoded.sid, native.sid);
                    assert_eq!(decoded.message, native.message);
                    assert_eq!(decoded.message_params, native.message_params);
                    assert_eq!(decoded.fix, native.fix);
                }
            }
        }
    }

    /// The reuse gate: a one-chapter edit re-encodes one book and splices the
    /// rest, and a second publication of a clean corpus encodes nothing at all.
    #[test]
    fn one_edit_re_encodes_one_book_and_reuses_the_other() {
        let mut resident = resident();
        resident
            .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN), usfm("EXO", EXO)]))
            .expect("two books");
        let mut cache = PublicationCache::default();
        let first = cache.publish(&mut resident).expect("first publication");
        assert_eq!(first.encoded.len(), 2);

        // Publishing again with nothing changed encodes nothing.
        let unchanged = cache.publish(&mut resident).expect("republication");
        assert!(
            unchanged.encoded.is_empty(),
            "a clean publish re-encodes nothing"
        );
        assert_eq!(unchanged.reused, vec![book("GEN"), book("EXO")]);
        assert_eq!(
            unchanged.bytes, first.bytes,
            "identical semantics, identical bytes"
        );

        // One chapter of one book changes.
        let edited_chapter: Vec<OwnedToken> = parse("\\c 2\n\\p\n\\v 1 Thus, edited.\n")
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect();
        let effect = resident
            .update_chapter(
                ChapterTarget::new(book("GEN"), ChapterLabel::Number("2".into())),
                ChapterInput::Tokens(edited_chapter),
            )
            .expect("a real edit");
        assert!(!effect.is_noop());

        let second = cache.publish(&mut resident).expect("republication");
        assert_eq!(second.encoded, vec![book("GEN")], "one book re-encoded");
        assert_eq!(second.reused, vec![book("EXO")], "the other spliced");
        // The edited book's source is the only one wire had to be handed.
        assert_eq!(second.sources.len(), 1);
        assert_eq!(second.sources[0].0, book("GEN"));

        let edited_source = match resident
            .to_usfm(braid::CorpusScope::Book(book("GEN")))
            .expect("bytes")
        {
            braid::ScopedOutput::Single(source) => source,
            other => panic!("expected one book, got {other:?}"),
        };
        let verified = verify(
            &second,
            &[(book("GEN"), edited_source.as_str()), (book("EXO"), EXO)],
        );
        assert_eq!(verified.snapshot_id, resident.expected_snapshot_id().0);
        assert_eq!(verified.books.len(), 2);
    }

    /// A configuration change rewrites what a book's findings are without touching
    /// its bytes, so the source hash alone would wrongly license reuse. The stamps
    /// are the other half of the key.
    #[test]
    fn a_config_change_invalidates_the_cache_even_though_no_byte_moved() {
        let mut resident = resident();
        resident
            .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN)]))
            .expect("one book");
        let mut cache = PublicationCache::default();
        let first = cache.publish(&mut resident).expect("first publication");
        assert_eq!(first.encoded, vec![book("GEN")]);

        let mut options = LintOptions::scoped(LintScope::Book);
        options.allow_implicit_chapter_content_verse =
            !options.allow_implicit_chapter_content_verse;
        let effect = resident.update_config(BraidConfig::new(options));
        // No token moved, so identity and hydration are untouched.
        assert!(effect.is_noop());
        assert_eq!(
            resident.expected_snapshot_id().0,
            first_snapshot_id(&first),
            "the corpus identity did not change"
        );

        let second = cache.publish(&mut resident).expect("republication");
        assert_eq!(
            second.encoded,
            vec![book("GEN")],
            "a stamp change must re-encode, not reuse"
        );
        assert!(second.reused.is_empty());
    }

    fn first_snapshot_id(publication: &Publication) -> u64 {
        u64::from_le_bytes(publication.bytes[32..40].try_into().expect("header slice"))
    }

    // ---- corpus scale -------------------------------------------------------

    fn corpus_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../example-corpora/en_ulb")
            .canonicalize()
            .expect("the example corpus resolves from the crate dir")
    }

    fn corpus_fixtures() -> Vec<(BookId, SourceKey, String)> {
        let mut paths: Vec<PathBuf> = fs::read_dir(corpus_root())
            .expect("corpus directory")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("usfm"))
            .collect();
        paths.sort();
        let fixtures: Vec<(BookId, SourceKey, String)> = paths
            .into_iter()
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?;
                let code = name.split('-').nth(1)?;
                Some((
                    BookId::from_str(code)?,
                    SourceKey::new(name)?,
                    fs::read_to_string(&path).ok()?,
                ))
            })
            .collect();
        assert_eq!(fixtures.len(), 66, "en_ulb is a 66-book corpus");
        fixtures
    }

    /// The Phase D gate at corpus scale: publish all 66 books, decode the
    /// publication back through the verify surface, and compare it against the
    /// native snapshot — tokens, findings including their fixes, stamps, and ids.
    /// Then edit one chapter and prove the republication re-encoded that book
    /// alone while every other book's bytes came back byte-identical.
    #[test]
    #[ignore = "corpus-scale"]
    fn the_whole_corpus_publishes_decodes_and_republishes_with_reuse() {
        let fixtures = corpus_fixtures();
        let mut resident = resident();
        resident
            .replace_corpus(CorpusInput::new(
                fixtures
                    .iter()
                    .map(|(book, source_key, source)| BookInput::Usfm {
                        source_key: source_key.clone(),
                        book: *book,
                        source: source.clone(),
                    })
                    .collect(),
            ))
            .expect("the whole corpus is resident");

        let mut cache = PublicationCache::default();
        let first = cache.publish(&mut resident).expect("publishes");
        assert_eq!(first.encoded.len(), 66);
        assert!(first.reused.is_empty());
        assert_eq!(first.sources.len(), 66);

        let sources: Vec<(BookId, &str)> = fixtures
            .iter()
            .map(|(book, _, source)| (*book, source.as_str()))
            .collect();
        let verified = verify(&first, &sources);
        assert_eq!(verified.snapshot_id, resident.expected_snapshot_id().0);
        assert_eq!(verified.books.len(), 66);
        assert_eq!(
            verified.lint_stamps,
            Some(LintStamps {
                config_fingerprint: LintConfigFingerprint::of(&resident.config().lint).0,
                engine_stamp: LintEngineStamp::current().0,
            })
        );

        let mut findings = 0usize;
        let mut fixes = 0usize;
        {
            let snapshot = resident.lint();
            for (decoded, native) in verified.books.iter().zip(&snapshot.books) {
                assert_eq!(decoded.receipt.book, native.book.as_str());
                assert_eq!(decoded.receipt.token_count as usize, native.tokens.len());
                assert_eq!(
                    decoded.receipt.source_hash,
                    format!("{:016x}", native.source_hash.0)
                );
                assert_eq!(decoded.lint_stamps, verified.lint_stamps);
                assert_eq!(
                    decoded.findings.len(),
                    native.result.issues.len(),
                    "{} finding count",
                    native.book
                );
                let book = native.book;
                for (decoded, native) in decoded.findings.iter().zip(&native.result.issues) {
                    assert_eq!(decoded.code, native.code, "{book}");
                    assert_eq!(decoded.token_id, native.token_id);
                    assert_eq!(decoded.related_token_id, native.related_token_id);
                    assert_eq!(decoded.sid, native.sid);
                    assert_eq!(decoded.marker, native.marker);
                    assert_eq!(decoded.message, native.message);
                    assert_eq!(decoded.message_params, native.message_params);
                    assert_eq!(decoded.fix, native.fix, "{book} fix");
                    findings += 1;
                    if decoded.fix.is_some() {
                        fixes += 1;
                    }
                }
            }
        }
        assert!(findings > 0, "the corpus must exercise the finding codec");
        // The census figure for this corpus, now proved end to end through
        // publication rather than only through the single-book codec.
        assert_eq!(fixes, 1, "en_ulb's one fix survives publication");

        // One chapter of one book changes; everything else must be spliced.
        let target_book = fixtures[0].0;
        let label = resident
            .chapter_labels(target_book)
            .expect("labels")
            .into_iter()
            .find_map(|label| match label {
                ChapterLabel::Number(number) => Some(ChapterLabel::Number(number)),
                ChapterLabel::FrontMatter => None,
            })
            .expect("a numbered chapter");
        let ChapterLabel::Number(number) = &label else {
            unreachable!("filtered above")
        };
        let replacement: Vec<OwnedToken> =
            parse(&format!("\\c {number}\n\\p\n\\v 1 Republished.\n"))
                .tokens
                .iter()
                .map(OwnedToken::from_parsed)
                .collect();
        resident
            .update_chapter(
                ChapterTarget::new(target_book, label),
                ChapterInput::Tokens(replacement),
            )
            .expect("a real edit");

        let second = cache.publish(&mut resident).expect("republishes");
        assert_eq!(second.encoded, vec![target_book], "one book re-encoded");
        assert_eq!(second.reused.len(), 65, "sixty-five spliced");
        assert_eq!(second.sources.len(), 1);

        // Byte-level proof of reuse: every untouched book's sections in the new
        // container are the first publication's bytes, section for section.
        let first_sections = section_bytes(&first.bytes);
        let second_sections = section_bytes(&second.bytes);
        assert_eq!(first_sections.len(), 66);
        assert_eq!(second_sections.len(), 66);
        for (book, sections) in &second_sections {
            let before = first_sections
                .iter()
                .find(|(candidate, _)| candidate == book)
                .map(|(_, sections)| sections)
                .expect("the same books are published both times");
            if *book == target_book {
                assert_ne!(sections, before, "the edited book must not be reused");
            } else {
                assert_eq!(
                    sections, before,
                    "{book} was spliced, so its bytes are unchanged"
                );
            }
        }

        // And the republication still decodes against the new truth.
        let edited_source = match resident
            .to_usfm(braid::CorpusScope::Book(target_book))
            .expect("bytes")
        {
            braid::ScopedOutput::Single(source) => source,
            other => panic!("expected one book, got {other:?}"),
        };
        let sources: Vec<(BookId, &str)> = fixtures
            .iter()
            .map(|(book, _, source)| {
                if *book == target_book {
                    (*book, edited_source.as_str())
                } else {
                    (*book, source.as_str())
                }
            })
            .collect();
        let verified = verify(&second, &sources);
        assert_eq!(verified.snapshot_id, resident.expected_snapshot_id().0);
        assert_eq!(verified.books.len(), 66);
    }

    /// Every book's sections, in TOC order, read out of a finished container —
    /// the comparison the reuse proof needs, without wire exposing its TOC. Kept
    /// grouped per book because a book has two sections, and a splice has to
    /// preserve both.
    fn section_bytes(container: &[u8]) -> Vec<(BookId, Vec<Vec<u8>>)> {
        // The container header's TOC offset and section count, then each entry's
        // book, offset, and length: the same four numbers a reader uses, read here
        // through the generated layout constants' own values.
        let section_count = u32::from_le_bytes(container[12..16].try_into().unwrap()) as usize;
        let toc_offset = u64::from_le_bytes(container[16..24].try_into().unwrap()) as usize;
        let mut out: Vec<(BookId, Vec<Vec<u8>>)> = Vec::with_capacity(section_count);
        for index in 0..section_count {
            let entry = toc_offset + index * 32;
            let book = BookId::from_str(
                std::str::from_utf8(&container[entry + 1..entry + 4]).expect("ascii book code"),
            )
            .expect("a valid book code");
            let offset =
                u64::from_le_bytes(container[entry + 8..entry + 16].try_into().unwrap()) as usize;
            let len =
                u64::from_le_bytes(container[entry + 16..entry + 24].try_into().unwrap()) as usize;
            let bytes = container[offset..offset + len].to_vec();
            match out.iter_mut().find(|(candidate, _)| *candidate == book) {
                Some((_, sections)) => sections.push(bytes),
                None => out.push((book, vec![bytes])),
            }
        }
        out
    }
}
