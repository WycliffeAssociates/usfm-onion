//! Publishing: braid's resident semantics turned into one packed
//! `corpus.bin` container, with a reuse cache that keeps a republication of
//! an unchanged corpus cheap.

use crate::{Braid, LintConfigFingerprint, LintEngineStamp, SourceHash, TokenIdentity};
use usfm_onion::token::BookId;
use usfm_onion_wire::corpus_codec::{
    CorpusSection, CorpusSectionInput, CorpusSectionTokens, EncodedCorpus, LintStamps,
    PublishedBook, encode_corpus,
};
use usfm_onion_wire::error::EncodeError;

/// One book's last published sections, with the facts that decide whether they
/// may be published again unchanged.
///
/// Three facts, because a section is derived from all three and the source hash
/// pins only one of them:
///
/// - the **source hash**, the bytes the section's spans are bound to;
/// - the **token identity**, which covers whatever the token stream carries that
///   its bytes do not — the exact set is `OwnedToken::hash_wire_identity`'s, not a
///   list restated here. The case it exists for: an editor re-pushing
///   byte-identical content under fresh ids changes every id in the section, and
///   every finding anchor and fix target that names one, while the bytes stay
///   identical; serving the old sections would hand back a publication addressing
///   tokens that no longer exist;
/// - the **stamps**, because a configuration change rewrites what a book's
///   findings *are* while leaving both of the above alone.
#[derive(Debug, Clone)]
struct CachedBook {
    source_hash: SourceHash,
    token_identity: TokenIdentity,
    stamps: LintStamps,
    published: PublishedBook,
}

/// What one publication produced, before it is turned into the public
/// [`PublishedCorpus`] shape (which also needs a fresh `lint()` read for
/// each book's source hash -- see [`Braid::publish`]).
#[derive(Debug)]
struct Publication {
    bytes: Vec<u8>,
    /// Per freshly encoded book, the exact source its sections are bound to.
    sources: Vec<(BookId, String)>,
    /// Books encoded this time. Whether the rest were reused is derived from
    /// this list at the [`PublishedCorpus`] boundary
    /// (`PublishedBookInfo::encoded`), rather than this struct separately
    /// tracking a `reused` list too -- one list, not two that must agree.
    encoded: Vec<BookId>,
}

/// A resident corpus's publication cache.
///
/// Holds only what wire produced and what decides its reuse; no source bytes, no
/// IO, and no knowledge of what is inside a section. A host that wants the cache
/// to outlive the process persists [`PublishedCorpus::bytes`] and reseeds through
/// [`Braid::restore_published_corpus`] — this type is an in-memory accelerator,
/// not a storage format.
///
/// Self-validating per publish: every [`Braid::publish`] re-derives reuse from
/// the resident corpus's own current state, so there are no external
/// invalidation hooks and a caller never has to remember to tell this cache
/// that anything moved.
#[derive(Debug, Default)]
pub struct PublicationCache {
    books: Vec<(BookId, CachedBook)>,
}

impl PublicationCache {
    /// Publishes the resident corpus as one packed container.
    ///
    /// Recomputes exactly the dirty books' lint (braid's own rule), re-encodes
    /// exactly the books whose bytes or stamps moved, and splices the rest from
    /// the last publication. A clean corpus that has already been published
    /// therefore encodes nothing at all.
    fn publish(&mut self, resident: &mut Braid) -> Result<Publication, EncodeError> {
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
        for book in &snapshot.books {
            let cached = self.books.iter().find(|(candidate, cached)| {
                *candidate == book.book
                    && cached.source_hash == book.source_hash
                    && cached.token_identity == book.token_identity
                    && cached.stamps == stamps
            });
            match cached {
                Some((_, cached)) => {
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

        // Every book publishes a finding section, so every publication carries
        // the stamps that license adopting them. An *empty* finding section is
        // evidence, not the absence of it: "lint ran over this book and found
        // nothing" is exactly what a clean project needs to restore, and without
        // stamps beside it a fully clean corpus would re-run every rule on reopen.
        // The distinction the format draws is no finding section at all (not
        // computed) versus a finding section with no rows (computed, clean).
        let EncodedCorpus {
            bytes,
            sources,
            books,
        } = encode_corpus(snapshot.id.0, Some(stamps), &sections)?;

        // Replaced wholesale from what the publication actually contains, so the
        // cache can never describe a book the last publication did not carry.
        self.books = books
            .into_iter()
            .map(|published| {
                let resident = snapshot
                    .books
                    .iter()
                    .find(|book| book.book == published.book)
                    .expect("every published book is a resident book");
                (
                    published.book,
                    CachedBook {
                        source_hash: resident.source_hash,
                        token_identity: resident.token_identity,
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
        })
    }
}

/// One book's own bookkeeping from a publish -- never the reuse-cache's
/// internal sections/bytes, which stay behind `PublicationCache`.
///
/// `source` is present exactly when `encoded` is `true`: a reused (spliced)
/// book's source did not change and wire never saw it this round, so the
/// caller is expected to already hold it from whichever earlier publish
/// first reported `encoded: true` for that book -- the same asymmetry
/// `EncodedCorpus::sources` documents natively.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PublishedBookInfo {
    pub book: String,
    pub source_hash: String,
    /// `true` when this book was freshly re-encoded this call; `false` when
    /// its previous publication's sections were spliced in unchanged.
    pub encoded: bool,
    pub source: Option<String>,
}

/// A packed corpus, ready to persist as `corpus.bin`, plus what the caller
/// needs to restore or re-publish it later.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PublishedCorpus {
    pub bytes: Vec<u8>,
    pub snapshot_id: String,
    /// One entry per resident book, in corpus order -- not only the freshly
    /// encoded ones, so a caller always has the complete bookkeeping set for
    /// what this publication now contains.
    pub books: Vec<PublishedBookInfo>,
}

/// Why a publish could not produce packed bytes.
///
/// Every variant is a pathological-input safety net (see
/// [`crate::dto::PackedEncodeError`]'s own doc comment) rather than something
/// a normal publish hits; surfaced as a typed refusal regardless, never a
/// panic.
// The intra-doc link above does not resolve from this crate -- there is no
// `crate::dto` here -- but the text is kept byte-for-byte identical to the
// wasm crate's own pre-extraction doc comment on purpose: tsify embeds this
// string verbatim into the generated .d.ts, and the npm surface must not
// change at all for this extraction. `PackedEncodeError` itself is
// `usfm_onion_wire::dto::PackedEncodeError`, referenced correctly below.
#[allow(rustdoc::broken_intra_doc_links)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "camelCase"))]
pub enum PublishError {
    Encode {
        error: usfm_onion_wire::dto::PackedEncodeError,
    },
}

impl Braid {
    /// Publishes the resident corpus as one packed `corpus.bin` container.
    ///
    /// Uses this handle's own [`PublicationCache`], so a repeat publish gets
    /// the cache's whole point -- splice-reuse of whatever did not change --
    /// automatically, with nothing for a caller to thread through. Dirty books
    /// are linted first (braid's own rule, via the `lint()` this runs
    /// internally); the reuse-cache's own sections and bytes never leave this
    /// method -- only the per-book bookkeeping in [`PublishedBookInfo`] does.
    pub fn publish(&mut self) -> Result<PublishedCorpus, PublishError> {
        // The cache is moved out for the duration of the publish and put
        // straight back. `PublicationCache::publish` needs `&mut` on both the
        // cache and the corpus, which as one `&mut self` would alias; taking
        // the field is how that is expressed without either splitting the
        // borrow unsafely or making the cache the caller's problem again.
        // `PublicationCache: Default` is an empty cache, so the window where
        // the field is empty is a window in which nothing reads it, and a
        // refusal restores it just as an success does.
        let mut cache = std::mem::take(&mut self.publication);
        let result = self.publish_with(&mut cache);
        self.publication = cache;
        result
    }

    fn publish_with(
        &mut self,
        cache: &mut PublicationCache,
    ) -> Result<PublishedCorpus, PublishError> {
        let publication = cache.publish(self).map_err(|error| PublishError::Encode {
            error: error.into(),
        })?;

        // A second `lint()` read, not a second lint *pass*: every book was just
        // made clean by the publish above, so this is a read of already-resident
        // state, needed only for the per-book source hash the bookkeeping DTO
        // reports (the internal `Publication` does not restate it).
        let snapshot = self.lint();
        let books = snapshot
            .books
            .iter()
            .map(|book| {
                let encoded = publication.encoded.contains(&book.book);
                let source = publication
                    .sources
                    .iter()
                    .find(|(candidate, _)| *candidate == book.book)
                    .map(|(_, source)| source.clone());
                PublishedBookInfo {
                    book: book.book.as_str().to_string(),
                    source_hash: format!("{:016x}", book.source_hash.0),
                    encoded,
                    source,
                }
            })
            .collect();

        Ok(PublishedCorpus {
            bytes: publication.bytes,
            snapshot_id: format!("{:016x}", snapshot.id.0),
            books,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BookInput, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget, CorpusInput, SourceKey,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion::parse::parse;
    use usfm_onion::token::OwnedToken;
    use usfm_onion_wire::corpus_codec::verify_corpus;

    fn resident() -> Braid {
        empty_resident()
    }

    fn empty_resident() -> Braid {
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

    /// Verifies a published corpus against the sources it says it is bound
    /// to, taking the unchanged ones from the caller's own copy — which is
    /// exactly what a host does: wire hands back sources only for what it
    /// encoded.
    fn verify(
        published: &PublishedCorpus,
        all: &[(BookId, &str)],
    ) -> usfm_onion_wire::corpus_codec::VerifiedCorpus {
        let sources: Vec<(BookId, &str)> = all.to_vec();
        verify_corpus(&published.bytes, &sources).expect("a publication verifies")
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
                let relabelled = crate::BookTokensInput {
                    source_key: SourceKey::new("EXO.usfm").unwrap(),
                    book: book("EXO"),
                    tokens,
                    line_ending: crate::LineEnding::Lf,
                };
                // Re-pushing the same tokens is a no-op; what matters here is the
                // lane, which the corpus gate below exercises with real edits.
                resident
                    .update_book(BookInput::Tokens(relabelled))
                    .expect("token push");
            }

            let published = resident.publish().expect("publishes");
            let encoded: Vec<&str> = published
                .books
                .iter()
                .filter(|book| book.encoded)
                .map(|book| book.book.as_str())
                .collect();
            assert_eq!(encoded, vec!["GEN", "EXO"]);

            let verified = verify(&published, &[(book("GEN"), GEN), (book("EXO"), EXO)]);
            assert_eq!(
                verified.snapshot_id,
                u64::from_str_radix(&published.snapshot_id, 16).unwrap()
            );
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
        let first = resident.publish().expect("first publication");
        assert_eq!(first.books.iter().filter(|book| book.encoded).count(), 2);

        // Publishing again with nothing changed encodes nothing.
        let unchanged = resident.publish().expect("republication");
        assert!(
            unchanged.books.iter().all(|book| !book.encoded),
            "a clean publish re-encodes nothing"
        );
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

        let second = resident.publish().expect("republication");
        let encoded: Vec<&str> = second
            .books
            .iter()
            .filter(|book| book.encoded)
            .map(|book| book.book.as_str())
            .collect();
        assert_eq!(encoded, vec!["GEN"], "one book re-encoded");
        let reused: Vec<&str> = second
            .books
            .iter()
            .filter(|book| !book.encoded)
            .map(|book| book.book.as_str())
            .collect();
        assert_eq!(reused, vec!["EXO"], "the other spliced");

        let edited_source = match resident
            .to_usfm(crate::CorpusScope::Book(book("GEN")))
            .expect("bytes")
        {
            crate::ScopedOutput::Single(source) => source,
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
        let first = resident.publish().expect("first publication");
        assert!(first.books.iter().all(|book| book.encoded));

        let mut options = LintOptions::scoped(LintScope::Book);
        options.allow_implicit_chapter_content_verse =
            !options.allow_implicit_chapter_content_verse;
        let effect = resident.update_config(BraidConfig::new(options));
        // No token moved, so identity and hydration are untouched.
        assert!(effect.is_noop());

        let second = resident.publish().expect("republication");
        assert!(
            second.books.iter().all(|book| book.encoded),
            "a stamp change must re-encode, not reuse"
        );
    }

    /// The reviewer's identity-only repro: byte-identical content re-pushed under
    /// different stable ids. The source hash cannot see it, so a cache keyed on
    /// the hash alone would serve the old sections — old ids, and finding anchors
    /// and fix targets naming tokens that no longer exist.
    #[test]
    fn an_identity_only_mutation_re_encodes_and_republishes_the_new_ids() {
        let mut resident = resident();
        resident
            .replace_corpus(CorpusInput::new(vec![usfm("GEN", GEN), usfm("EXO", EXO)]))
            .expect("two books");
        let hash_before = resident
            .books()
            .into_iter()
            .find(|entry| entry.book == book("GEN"))
            .expect("resident")
            .source_hash;
        let first = resident.publish().expect("first publication");
        assert!(first.books.iter().all(|book| book.encoded));
        let before = verify(&first, &[(book("GEN"), GEN), (book("EXO"), EXO)]);
        let old_anchor = before.books[0]
            .findings
            .iter()
            .find_map(|finding| finding.token_id.clone())
            .expect("GEN's findings anchor on tokens");

        // The same bytes, carrying the editor's own ids: relabelled through the
        // residency boundary, the only way a resident token gets a new id.
        let relabelled: Vec<OwnedToken> = parse(GEN)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect::<Vec<_>>()
            .iter()
            .enumerate()
            .map(|(index, token)| {
                let mut working = usfm_onion::format::FormatToken::from(token);
                working.id = Some(format!("editor-{index}"));
                OwnedToken::from_format_token(&working, Some(token)).expect("relabelled")
            })
            .collect();
        let effect = resident
            .update_book(BookInput::Tokens(crate::BookTokensInput {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book("GEN"),
                tokens: relabelled,
                line_ending: crate::LineEnding::Lf,
            }))
            .expect("an identity-only push");
        assert!(!effect.is_noop());
        let entry = resident
            .books()
            .into_iter()
            .find(|entry| entry.book == book("GEN"))
            .unwrap();
        assert_eq!(entry.source_hash, hash_before, "not one byte changed");

        let second = resident.publish().expect("republication");
        let encoded: Vec<&str> = second
            .books
            .iter()
            .filter(|book| book.encoded)
            .map(|book| book.book.as_str())
            .collect();
        assert_eq!(
            encoded,
            vec!["GEN"],
            "an identity-only change must re-encode"
        );

        let after = verify(&second, &[(book("GEN"), GEN), (book("EXO"), EXO)]);
        let new_anchor = after.books[0]
            .findings
            .iter()
            .find_map(|finding| finding.token_id.clone())
            .expect("the findings still anchor on tokens");
        assert_ne!(
            new_anchor, old_anchor,
            "the published anchors are the new ids"
        );
        assert!(
            new_anchor.starts_with("editor-"),
            "the anchor is the caller's own id, got {new_anchor}"
        );
        // Every fix target too, since a fix addresses its token by id.
        for finding in &after.books[0].findings {
            if let Some(fix) = &finding.fix {
                assert!(
                    fix.target_token_id().starts_with("editor-"),
                    "a published fix must target a token that exists"
                );
            }
        }
    }

    /// The subtler half of the identity rule: an attribute's own *spelling* is
    /// what the owned encoder searches for when it places attribute spans, so two
    /// streams that agree on every key, value, and default but spell an attribute
    /// differently encode to different bytes. A cache that could not see that
    /// would serve spans naming the wrong text.
    #[test]
    fn an_attribute_spelling_change_re_encodes_rather_than_serving_stale_bytes() {
        const ALIGNED: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\w gracious|lemma=\"grace\"\\w* b\n";
        let mut resident = resident();
        resident
            .replace_corpus(CorpusInput::new(vec![
                usfm("GEN", ALIGNED),
                usfm("EXO", EXO),
            ]))
            .expect("two books");
        let first = resident.publish().expect("first publication");
        assert!(first.books.iter().all(|book| book.encoded));

        // Re-push GEN with each attribute's recorded spelling narrowed by its
        // leading character while the verbatim list text is kept, so every
        // emitted byte stays identical: same key and value, same source bytes,
        // different per-attribute spellings.
        let respelled: Vec<OwnedToken> = parse(ALIGNED)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .map(|token| {
                if token.attribute_list().is_none() {
                    return token;
                }
                // The verbatim list is kept, so not one emitted byte moves; only
                // each attribute's own recorded spelling narrows, which is what
                // the encoder searches for when it places the attribute's span.
                let working = usfm_onion::format::FormatToken {
                    attributes: token
                        .attributes()
                        .iter()
                        .map(|attribute| usfm_onion::token::OwnedAttribute {
                            source: Box::from(&attribute.source[1..]),
                            key: attribute.key.clone(),
                            value: attribute.value.clone(),
                            is_default: attribute.is_default,
                            span: None,
                        })
                        .collect(),
                    ..usfm_onion::format::FormatToken::from(&token)
                };
                OwnedToken::from_format_token(&working, Some(&token)).expect("respelled")
            })
            .collect();
        let identity_before = resident
            .books()
            .into_iter()
            .find(|entry| entry.book == book("GEN"))
            .unwrap()
            .token_identity;
        resident
            .update_book(BookInput::Tokens(crate::BookTokensInput {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book("GEN"),
                tokens: respelled,
                line_ending: crate::LineEnding::Lf,
            }))
            .expect("a spelling-only push");
        let entry = resident
            .books()
            .into_iter()
            .find(|entry| entry.book == book("GEN"))
            .unwrap();
        assert_ne!(
            entry.token_identity, identity_before,
            "the attribute spelling moved, so the token identity must"
        );

        let second = resident.publish().expect("republication");
        let encoded: Vec<&str> = second
            .books
            .iter()
            .filter(|book| book.encoded)
            .map(|book| book.book.as_str())
            .collect();
        assert_eq!(
            encoded,
            vec!["GEN"],
            "a spelling-only change must re-encode"
        );

        let before = section_bytes(&first.bytes);
        let after = section_bytes(&second.bytes);
        let sections_of = |sections: &[(BookId, Vec<Vec<u8>>)], book: BookId| {
            sections
                .iter()
                .find(|(candidate, _)| *candidate == book)
                .map(|(_, sections)| sections.clone())
                .expect("published")
        };
        assert_ne!(
            sections_of(&after, book("GEN")),
            sections_of(&before, book("GEN")),
            "the served section must be the new one"
        );
        assert_eq!(
            sections_of(&after, book("EXO")),
            sections_of(&before, book("EXO")),
            "and the untouched book is still spliced"
        );
        let verified = verify(&second, &[(book("GEN"), ALIGNED), (book("EXO"), EXO)]);
        assert_eq!(verified.books.len(), 2);
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

        let first = resident.publish().expect("publishes");
        assert_eq!(first.books.iter().filter(|book| book.encoded).count(), 66);

        let sources: Vec<(BookId, &str)> = fixtures
            .iter()
            .map(|(book, _, source)| (*book, source.as_str()))
            .collect();
        let verified = verify(&first, &sources);
        assert_eq!(
            verified.snapshot_id,
            u64::from_str_radix(&first.snapshot_id, 16).unwrap()
        );
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

        let second = resident.publish().expect("republishes");
        let encoded: Vec<BookId> = second
            .books
            .iter()
            .filter(|book| book.encoded)
            .map(|book| BookId::from_str(&book.book).unwrap())
            .collect();
        assert_eq!(encoded, vec![target_book], "one book re-encoded");
        let reused_count = second.books.iter().filter(|book| !book.encoded).count();
        assert_eq!(reused_count, 65, "sixty-five spliced");

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
            .to_usfm(crate::CorpusScope::Book(target_book))
            .expect("bytes")
        {
            crate::ScopedOutput::Single(source) => source,
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
        assert_eq!(
            verified.snapshot_id,
            u64::from_str_radix(&second.snapshot_id, 16).unwrap()
        );
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
