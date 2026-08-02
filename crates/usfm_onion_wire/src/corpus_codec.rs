//! The public composition surface: a whole corpus's semantics in, one packed
//! container out, and back.
//!
//! This is the only place a caller can produce a complete publication, and it is
//! deliberately the *narrowest* entry point that can: everything below it
//! (sections, columns, dictionaries, the container writer) stays private, so
//! there is no second, lower-level wire API for a caller to assemble a container
//! out of by hand and no way to write bytes that skipped a check.
//!
//! # What each side owns
//!
//! A caller hands over semantics — tokens, findings, a snapshot id, the stamps
//! that describe how the findings were produced — and gets bytes. It never learns
//! an offset, a field id, or a width, and this module never learns what a
//! snapshot id *means*, when a book is dirty, or whether a fingerprint should be
//! trusted. The composing adapter above sits between them; that is the whole
//! reason it exists.
//!
//! # Reuse
//!
//! A publication may reuse a book's already-encoded sections instead of
//! re-encoding them ([`CorpusSection::Cached`]). A reused section is validated
//! before it is adopted — structure, both integrity checksums, and the facts the
//! caller claims about it — so "reuse" never means "trust". A cached section that
//! does not check out is a typed refusal of the whole publication, never a
//! container with one unverified book in it.

use usfm_onion::lint::LintResult;
use usfm_onion::token::{
    BookId, OwnedToken, ReconstructedSpans, Token, tokens_to_usfm_reconstruct_spanned,
};

use crate::container::{
    ContainerSection, SectionPayload, inspect_section, write_container_sections,
};
use crate::error::{EncodeError, LayoutRefusal};
use crate::finding_codec::{decode_finding_section, encode_findings_with};
use crate::finding_section::section_lint_stamps;
use crate::schema::SectionKind;

// Re-exported so the composition surface is one import: the stamps are part of
// what a publisher supplies, and a caller should not have to reach into the
// schema module to name them.
pub use crate::schema::LintStamps;
use crate::token_codec::{
    decode_token_section, encode_token_section_with_ids, owned_stable_ids, owned_to_borrowed,
};
use crate::verify::VerifiedBook;

/// How one book's tokens arrive.
///
/// The two arms are not a convenience: a parsed stream carries spans into a
/// source that already exists, while an owned stream is spanless by design, so
/// wire serializes it and derives the spans from that same emission pass. The
/// source an owned section is bound to is therefore one wire produced, not a file
/// on disk — which is why [`EncodedCorpus::sources`] hands it back.
#[derive(Debug)]
pub enum CorpusSectionTokens<'a> {
    /// Parsed tokens, which carry their own spans into `source`.
    Parsed {
        source: &'a str,
        tokens: &'a [Token<'a>],
    },
    /// Spanless owned tokens, as a resident corpus holds them.
    Owned { tokens: &'a [OwnedToken] },
}

/// One book's semantics to encode.
#[derive(Debug)]
pub struct CorpusSectionInput<'a> {
    pub book: BookId,
    pub tokens: CorpusSectionTokens<'a>,
    /// The book's lint contribution. `None` publishes tokens alone — a legal
    /// publication for a book whose findings are not wanted or not computed.
    pub findings: Option<&'a LintResult>,
}

/// One book's already-encoded sections, offered back for reuse.
///
/// The bytes are opaque to the caller and self-contained: a finished section
/// stores every offset relative to its own start and checksums only its own
/// bytes, so it can be placed in a later container verbatim. `book` and
/// `source_hash` are the caller's *claim* about what these bytes are, and they
/// are checked against what the bytes themselves say — a claim is what makes the
/// check meaningful rather than circular.
#[derive(Debug, Clone, Copy)]
pub struct CachedBookSections<'a> {
    pub book: BookId,
    pub source_hash: u64,
    pub sections: &'a [PublishedSection],
    pub bytes: &'a [u8],
}

/// Where one book of a publication comes from.
#[derive(Debug)]
pub enum CorpusSection<'a> {
    /// Encode this book now, from its semantics.
    Fresh(CorpusSectionInput<'a>),
    /// Reuse the bytes a previous publication produced for it.
    Cached(CachedBookSections<'a>),
}

impl CorpusSection<'_> {
    pub fn book(&self) -> BookId {
        match self {
            Self::Fresh(input) => input.book,
            Self::Cached(cached) => cached.book,
        }
    }
}

/// One encoded section's extent inside [`PublishedBook::bytes`].
///
/// A caller keeping a publication's per-book sidecar stores the bytes and these
/// extents together and hands both back; it never has to know what is in them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishedSection {
    pub kind: SectionKind,
    pub offset: usize,
    pub len: usize,
}

/// One book's reusable sidecar: its encoded sections and the facts that decide
/// whether they may be reused.
///
/// No source bytes and no IO: the sidecar describes what wire produced, and where
/// the caller keeps it is the caller's business.
#[derive(Debug, Clone)]
pub struct PublishedBook {
    pub book: BookId,
    pub source_hash: u64,
    pub bytes: Vec<u8>,
    pub sections: Vec<PublishedSection>,
}

impl PublishedBook {
    /// The sidecar as reuse input for the next publication.
    pub fn as_cached(&self) -> CachedBookSections<'_> {
        CachedBookSections {
            book: self.book,
            source_hash: self.source_hash,
            sections: &self.sections,
            bytes: &self.bytes,
        }
    }
}

/// A complete publication.
#[derive(Debug, Clone)]
pub struct EncodedCorpus {
    pub bytes: Vec<u8>,
    /// Per *freshly encoded* book, the exact source its spans and hash are bound
    /// to. For a parsed section this is the caller's own source; for an owned
    /// section it is the serialization wire produced, which — not any file on
    /// disk — is what the section is bound to. A reused section is absent by
    /// definition: its source did not change, and wire never saw it.
    pub sources: Vec<(BookId, String)>,
    /// Per book, the reusable sidecar. Present for reused books too: a
    /// publication's sidecar set is complete, so a caller can keep the newest one
    /// and discard what it had.
    pub books: Vec<PublishedBook>,
}

/// Encodes a whole corpus into one container.
///
/// `snapshot_id` is written verbatim and never recomputed — what it means is the
/// corpus side's business. `lint_stamps` describes how every published finding
/// was produced; supplying it is what lets a later reader adopt those findings as
/// a warm cache instead of merely reading them. Omitting it is the safe default,
/// and it is refused outright when no section carries findings at all, because
/// then it would describe nothing.
///
/// Book order is the caller's, preserved: the container is written token sections
/// in corpus order followed by their finding sections in the same order.
pub fn encode_corpus(
    snapshot_id: u64,
    lint_stamps: Option<LintStamps>,
    sections: &[CorpusSection<'_>],
) -> Result<EncodedCorpus, EncodeError> {
    let refuse = |book: BookId, reason| EncodeError::InvalidSectionLayout { book, reason };
    if let Some(index) = sections.iter().position(|section| {
        sections
            .iter()
            .filter(|other| other.book() == section.book())
            .count()
            > 1
    }) {
        return Err(refuse(
            sections[index].book(),
            LayoutRefusal::DuplicateSection {
                kind: SectionKind::Token,
            },
        ));
    }
    if lint_stamps.is_some()
        && !sections.iter().any(|section| match section {
            CorpusSection::Fresh(input) => input.findings.is_some(),
            CorpusSection::Cached(cached) => cached
                .sections
                .iter()
                .any(|section| section.kind == SectionKind::Finding),
        })
    {
        // Stamps that describe no findings are not a harmless extra: a reader
        // would take them as a licence covering a publication that has nothing
        // to licence.
        return Err(refuse(
            sections.first().map_or(BookId::UNKNOWN, |s| s.book()),
            LayoutRefusal::OrphanFindingSection,
        ));
    }

    // Owned sections serialize first, into a vector nothing borrows yet: the
    // borrowed tokens the encoders need point into these strings, so they cannot
    // be produced while the vector can still reallocate.
    let mut owned: Vec<Option<(String, Vec<ReconstructedSpans>)>> =
        Vec::with_capacity(sections.len());
    for section in sections {
        owned.push(match section {
            CorpusSection::Fresh(CorpusSectionInput {
                tokens: CorpusSectionTokens::Owned { tokens },
                ..
            }) => Some(tokens_to_usfm_reconstruct_spanned(tokens)),
            _ => None,
        });
    }

    // Everything is built before anything is written, so a refusal in the last
    // book cannot leave a half-composed container behind.
    let mut payloads: Vec<SectionPayload<'_>> = Vec::with_capacity(sections.len() * 2);
    let mut token_buffers = Vec::with_capacity(sections.len());
    let mut finding_buffers = Vec::with_capacity(sections.len());
    let mut sources: Vec<(BookId, String)> = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let CorpusSection::Fresh(input) = section else {
            continue;
        };
        let book = input.book;
        let (source, borrowed, stable_ids): (&str, Vec<Token<'_>>, Option<Vec<&str>>) =
            match (&input.tokens, &owned[index]) {
                (CorpusSectionTokens::Parsed { source, tokens }, _) => {
                    (source, tokens.to_vec(), None)
                }
                (CorpusSectionTokens::Owned { tokens }, Some((source, spans))) => {
                    let borrowed = owned_to_borrowed(book, tokens, spans, source)?;
                    let ids = owned_stable_ids(tokens, &borrowed);
                    (source.as_str(), borrowed, ids)
                }
                (CorpusSectionTokens::Owned { .. }, None) => {
                    unreachable!("every owned section serialized above")
                }
            };
        token_buffers.push((
            index,
            encode_token_section_with_ids(book, source, &borrowed, stable_ids.as_deref())?,
        ));
        if let Some(result) = input.findings {
            finding_buffers.push((
                index,
                encode_findings_with(
                    book,
                    source,
                    &borrowed,
                    stable_ids.as_deref(),
                    &result.issues,
                    lint_stamps,
                )?,
            ));
        }
        sources.push((book, source.to_string()));
    }

    // Payloads borrow the buffers, so they are built once both vectors are
    // complete and never pushed to again.
    for (_, buffers) in &token_buffers {
        payloads.push(buffers.payload());
    }
    for (_, buffers) in &finding_buffers {
        payloads.push(buffers.payload());
    }

    let mut container_sections: Vec<ContainerSection<'_>> = Vec::with_capacity(payloads.len());
    for payload in &payloads {
        container_sections.push(ContainerSection::Fresh(payload));
    }
    for section in sections {
        let CorpusSection::Cached(cached) = section else {
            continue;
        };
        for extent in cached.sections {
            let bytes = cached
                .bytes
                .get(extent.offset..extent.offset.saturating_add(extent.len))
                .ok_or_else(|| refuse(cached.book, LayoutRefusal::CachedSectionUnreadable))?;
            // The whole reuse contract in one call: the bytes must be a
            // structurally valid, checksum-intact section that says it is the book
            // and source the caller claims. Nothing is adopted on a caller's word.
            let section = inspect_section(bytes)
                .map_err(|_| refuse(cached.book, LayoutRefusal::CachedSectionUnreadable))?;
            let header = section.header;
            if header.book != cached.book
                || header.source_hash != cached.source_hash
                || header.kind != extent.kind
            {
                return Err(refuse(cached.book, LayoutRefusal::CachedSectionMismatch));
            }
            // Structure and binding are not enough: a finding section records the
            // stamps its findings were produced under, and splicing one into a
            // publication that claims different stamps would sign a statement
            // about those findings that is not true of them. Presence has to agree
            // too — an unstamped section in a stamped publication would silently
            // become adoptable, and a stamped one in an unstamped publication
            // would carry a licence the publication does not grant.
            if header.kind == SectionKind::Finding {
                let recorded = section_lint_stamps(&section)
                    .map_err(|_| refuse(cached.book, LayoutRefusal::CachedSectionUnreadable))?;
                if recorded != lint_stamps {
                    return Err(refuse(
                        cached.book,
                        LayoutRefusal::CachedSectionStampMismatch,
                    ));
                }
            }
            container_sections.push(ContainerSection::Encoded { header, bytes });
        }
    }

    let bytes = write_container_sections(snapshot_id, &container_sections)?;
    let books = published_books(&bytes)?;
    Ok(EncodedCorpus {
        bytes,
        sources,
        books,
    })
}

/// Splits a finished container into per-book reusable sidecars.
///
/// Read back off the container rather than accumulated while writing it, so a
/// sidecar can only ever describe bytes that are actually in the publication.
fn published_books(bytes: &[u8]) -> Result<Vec<PublishedBook>, EncodeError> {
    let container =
        crate::container::read_container(bytes).map_err(|_| EncodeError::InvalidSectionLayout {
            book: BookId::UNKNOWN,
            reason: LayoutRefusal::CachedSectionUnreadable,
        })?;
    let mut books: Vec<PublishedBook> = Vec::new();
    for entry in container.toc() {
        let start =
            usize::try_from(entry.offset).map_err(|_| EncodeError::InvalidSectionLayout {
                book: entry.book,
                reason: LayoutRefusal::CachedSectionUnreadable,
            })?;
        let len =
            usize::try_from(entry.byte_len).map_err(|_| EncodeError::InvalidSectionLayout {
                book: entry.book,
                reason: LayoutRefusal::CachedSectionUnreadable,
            })?;
        let slice = bytes
            .get(start..start + len)
            .ok_or(EncodeError::InvalidSectionLayout {
                book: entry.book,
                reason: LayoutRefusal::CachedSectionUnreadable,
            })?;
        let book = match books.iter_mut().find(|book| book.book == entry.book) {
            Some(book) => book,
            None => {
                books.push(PublishedBook {
                    book: entry.book,
                    source_hash: entry.source_hash,
                    bytes: Vec::new(),
                    sections: Vec::new(),
                });
                books.last_mut().expect("just pushed")
            }
        };
        book.sections.push(PublishedSection {
            kind: entry.kind,
            offset: book.bytes.len(),
            len,
        });
        book.bytes.extend_from_slice(slice);
    }
    Ok(books)
}

/// A whole verified publication.
#[derive(Debug, Clone)]
pub struct VerifiedCorpus {
    /// The id the publisher wrote, verbatim.
    pub snapshot_id: u64,
    /// The stamps every finding section agreed on, when they carried any.
    pub lint_stamps: Option<LintStamps>,
    /// One entry per book, in the container's own (corpus) order.
    pub books: Vec<VerifiedBook>,
}

/// Verifies a whole publication against the exact sources it was bound to.
///
/// The corpus-level twin of [`crate::verify::verify_book`], running the identical
/// per-book checks: every book in the container must have exactly one source
/// supplied, every supplied source must name a book the container has, and a
/// finding section must pair with its own book's token section. A missing or
/// extra source is refused rather than skipped — a partially verified corpus is
/// exactly the thing a caller would then treat as whole.
///
/// Findings that carry stamps must all carry the *same* stamps: they describe one
/// publication, and two books disagreeing about how their findings were produced
/// is a container no single cache decision can be made about.
pub fn verify_corpus(
    packed: &[u8],
    sources: &[(BookId, &str)],
) -> Result<VerifiedCorpus, crate::error::DecodeError> {
    use crate::error::DecodeError;

    let container = crate::container::read_container(packed)?;
    let toc = container.toc();
    let mut order: Vec<BookId> = Vec::new();
    for entry in toc {
        if entry.kind == SectionKind::Token {
            if order.contains(&entry.book) {
                return Err(DecodeError::InvalidToc);
            }
            order.push(entry.book);
        }
    }
    // The corpus is the set of books that have *tokens*, so a finding section for
    // a book with none is a section this walk would never open — and an unopened
    // section is one that was never verified, in a container a caller is about to
    // treat as whole. Same trust parity as the single-book verifier, which refuses
    // a finding section it cannot pair.
    for entry in toc {
        if entry.kind == SectionKind::Finding && !order.contains(&entry.book) {
            return Err(DecodeError::InvalidToc);
        }
    }
    if order.len() != sources.len() {
        return Err(DecodeError::InvalidToc);
    }

    let mut books = Vec::with_capacity(order.len());
    let mut stamps: Option<LintStamps> = None;
    let mut stamped = 0usize;
    for book in order {
        let source = sources
            .iter()
            .find(|(candidate, _)| *candidate == book)
            .map(|(_, source)| *source)
            .ok_or(DecodeError::InvalidToc)?;

        let token_index = toc
            .iter()
            .position(|entry| entry.kind == SectionKind::Token && entry.book == book)
            .ok_or(DecodeError::InvalidToc)?;
        let mut finding_index = None;
        for (index, entry) in toc.iter().enumerate() {
            if entry.kind == SectionKind::Finding && entry.book == book {
                if finding_index.is_some() {
                    return Err(DecodeError::InvalidToc);
                }
                finding_index = Some(index);
            }
        }

        let token_section = container
            .section(token_index)
            .ok_or(DecodeError::InvalidToc)??;
        let decoded_tokens = decode_token_section(&token_section, source)?;
        let (findings, lint_stamps) = match finding_index {
            Some(index) => {
                let section = container.section(index).ok_or(DecodeError::InvalidToc)??;
                decode_finding_section(&section, book, source, &decoded_tokens)?
            }
            None => (Vec::new(), None),
        };
        if let Some(found) = lint_stamps {
            stamped += 1;
            match stamps {
                None => stamps = Some(found),
                Some(existing) if existing == found => {}
                Some(_) => return Err(DecodeError::InvalidSection),
            }
        }

        books.push(crate::verify::verified_book(
            &token_section,
            book,
            container.header().snapshot_id,
            findings,
            lint_stamps,
        )?);
    }

    // All or none: a publication where only some finding sections are stamped
    // cannot be adopted as one cache, and silently adopting the stamped half is
    // the partial adoption this refuses.
    let finding_sections = toc
        .iter()
        .filter(|entry| entry.kind == SectionKind::Finding)
        .count();
    if stamped != 0 && stamped != finding_sections {
        return Err(crate::error::DecodeError::InvalidSection);
    }

    Ok(VerifiedCorpus {
        snapshot_id: container.header().snapshot_id,
        lint_stamps: stamps,
        books,
    })
}

/// Materializes every book's tokens as *resident* tokens, in the container's
/// own (corpus) order.
///
/// The corpus-grain twin of [`crate::verify::materialize_owned_tokens`],
/// needed for exactly the same reason that function exists beside
/// [`crate::verify::verify_book`]: [`verify_corpus`] hands back receipts and
/// findings but never tokens, because materializing a token is a cost only a
/// caller that actually wants one should pay. Repeats the same structural
/// checks `verify_corpus` does over the token/finding table of contents
/// (exactly one token section per book, sources supplied 1:1) rather than
/// trusting a caller to have verified first.
pub fn materialize_owned_tokens_corpus(
    packed: &[u8],
    sources: &[(BookId, &str)],
) -> Result<Vec<(BookId, Vec<OwnedToken>)>, crate::error::DecodeError> {
    use crate::error::DecodeError;

    let container = crate::container::read_container(packed)?;
    let toc = container.toc();
    let mut order: Vec<BookId> = Vec::new();
    for entry in toc {
        if entry.kind == SectionKind::Token {
            if order.contains(&entry.book) {
                return Err(DecodeError::InvalidToc);
            }
            order.push(entry.book);
        }
    }
    if order.len() != sources.len() {
        return Err(DecodeError::InvalidToc);
    }

    let mut books = Vec::with_capacity(order.len());
    for book in order {
        let source = sources
            .iter()
            .find(|(candidate, _)| *candidate == book)
            .map(|(_, source)| *source)
            .ok_or(DecodeError::InvalidToc)?;
        let token_index = toc
            .iter()
            .position(|entry| entry.kind == SectionKind::Token && entry.book == book)
            .ok_or(DecodeError::InvalidToc)?;
        let token_section = container
            .section(token_index)
            .ok_or(DecodeError::InvalidToc)??;
        let decoded = decode_token_section(&token_section, source)?;
        let tokens = decoded
            .tokens
            .iter()
            .enumerate()
            .map(|(index, token)| {
                let mut dto_token = crate::dto::Token::from(token);
                // The section's own opaque ids win when it carried them;
                // otherwise the positional label the decoder already
                // stamped is the id -- same rule
                // `materialize_owned_tokens` applies for one book.
                if let Some(ids) = decoded.stable_ids.as_ref() {
                    dto_token.id = ids
                        .get(index)
                        .map(|id| id.to_string())
                        .ok_or(DecodeError::InvalidSection)?;
                }
                crate::dto::owned_token_from_dto(&dto_token, index as u32)
                    .map_err(|_| DecodeError::InvalidSection)
            })
            .collect::<Result<Vec<_>, _>>()?;
        books.push((book, tokens));
    }
    Ok(books)
}

#[cfg(test)]
mod tests {
    use super::*;
    use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
    use usfm_onion::parse::parse;
    use usfm_onion::token::OwnedToken;

    const GEN: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n";
    const EXO: &str = "\\id exo Exodus\n\\c 1\n\\p\n\\v 1 These are the names.\n";

    fn book(code: &str) -> BookId {
        BookId::from_str(code).expect("test book code")
    }

    fn owned_tokens(source: &str) -> Vec<OwnedToken> {
        parse(source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect()
    }

    fn lint_of(source: &str) -> LintResult {
        lint_tokens(&parse(source).tokens, LintOptions::scoped(LintScope::Book))
    }

    fn stamps() -> LintStamps {
        LintStamps {
            config_fingerprint: 0xc0ff_ee00_u64,
            engine_stamp: 0x0e01_u64,
        }
    }

    /// The composition contract end to end, in the lane a resident corpus
    /// actually uses (owned, spanless tokens): several books in one container,
    /// each with findings, verified back per book against the sources wire says
    /// they are bound to.
    #[test]
    fn an_owned_corpus_publishes_and_verifies_book_by_book() {
        let gen_tokens = owned_tokens(GEN);
        let exo_tokens = owned_tokens(EXO);
        let gen_lint = lint_of(GEN);
        let exo_lint = lint_of(EXO);
        let sections = vec![
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &gen_tokens,
                },
                findings: Some(&gen_lint),
            }),
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("EXO"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &exo_tokens,
                },
                findings: Some(&exo_lint),
            }),
        ];

        let published = encode_corpus(0xfeed, Some(stamps()), &sections).expect("publishes");
        // The sources are wire's own serialization, which for a parse-origin
        // owned stream is the original file byte for byte.
        assert_eq!(published.sources.len(), 2);
        assert_eq!(published.sources[0], (book("GEN"), GEN.to_string()));
        assert_eq!(published.sources[1], (book("EXO"), EXO.to_string()));
        assert_eq!(published.books.len(), 2);

        let sources: Vec<(BookId, &str)> = published
            .sources
            .iter()
            .map(|(book, source)| (*book, source.as_str()))
            .collect();
        let verified = verify_corpus(&published.bytes, &sources).expect("verifies");
        assert_eq!(verified.snapshot_id, 0xfeed);
        assert_eq!(verified.lint_stamps, Some(stamps()));
        // Corpus order, not book-code order.
        assert_eq!(
            verified
                .books
                .iter()
                .map(|book| book.receipt.book.as_str())
                .collect::<Vec<_>>(),
            ["GEN", "EXO"]
        );
        for (verified, expected) in verified.books.iter().zip([&gen_lint, &exo_lint]) {
            assert_eq!(verified.findings.len(), expected.issues.len());
            assert_eq!(verified.lint_stamps, Some(stamps()));
        }
        // Determinism: the same semantics produce the same bytes.
        assert_eq!(
            encode_corpus(0xfeed, Some(stamps()), &sections)
                .unwrap()
                .bytes,
            published.bytes
        );
    }

    /// The corpus-grain token materializer matches what a per-book
    /// `materialize_owned_tokens` call on the *same bytes, sliced one book at
    /// a time* would produce -- same ids (positional or opaque, per the same
    /// rule), same token count, same corpus order.
    #[test]
    fn materialize_owned_tokens_corpus_matches_the_resident_tokens_book_by_book() {
        let gen_tokens = owned_tokens(GEN);
        let exo_tokens = owned_tokens(EXO);
        let sections = vec![
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &gen_tokens,
                },
                findings: None,
            }),
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("EXO"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &exo_tokens,
                },
                findings: None,
            }),
        ];
        let published = encode_corpus(0xfeed, None, &sections).expect("publishes");
        let sources: Vec<(BookId, &str)> = published
            .sources
            .iter()
            .map(|(book, source)| (*book, source.as_str()))
            .collect();

        let materialized =
            materialize_owned_tokens_corpus(&published.bytes, &sources).expect("materializes");
        assert_eq!(
            materialized
                .iter()
                .map(|(book, _)| *book)
                .collect::<Vec<_>>(),
            [book("GEN"), book("EXO")],
            "corpus order, not book-code order"
        );

        for ((corpus_book, corpus_tokens), original_tokens) in
            materialized.iter().zip([&gen_tokens, &exo_tokens])
        {
            assert_eq!(corpus_tokens.len(), original_tokens.len(), "{corpus_book}");
            for (materialized, original) in corpus_tokens.iter().zip(original_tokens) {
                assert_eq!(materialized.id(), original.id());
                assert_eq!(materialized.source(), original.source());
            }
        }
    }

    /// The parsed lane, which a cold-parse publisher uses, and the token-only
    /// publication: findings are optional and their absence is not a defect.
    #[test]
    fn a_parsed_token_only_corpus_publishes() {
        let parsed = parse(GEN);
        let sections = vec![CorpusSection::Fresh(CorpusSectionInput {
            book: book("GEN"),
            tokens: CorpusSectionTokens::Parsed {
                source: GEN,
                tokens: &parsed.tokens,
            },
            findings: None,
        })];
        let published = encode_corpus(7, None, &sections).expect("publishes");
        let verified = verify_corpus(&published.bytes, &[(book("GEN"), GEN)]).expect("verifies");
        assert_eq!(verified.snapshot_id, 7);
        assert_eq!(verified.lint_stamps, None);
        assert!(verified.books[0].findings.is_empty());

        // A one-book publication is exactly what the single-book verifier takes,
        // so the JS boundary (which verifies one book at a time) reads a
        // publication of one book without a second container shape.
        let single = crate::verify::verify_book(&published.bytes, GEN).expect("verifies");
        assert_eq!(single.receipt.book, verified.books[0].receipt.book);
        assert_eq!(single.receipt.token_count, parsed.tokens.len() as u32);
    }

    /// A one-book publication *with* stamps: the same shape, and the stamps come
    /// back through the single-book verifier too.
    #[test]
    fn a_single_book_publication_carries_its_stamps_through_verify_book() {
        let tokens = owned_tokens(GEN);
        let lint = lint_of(GEN);
        let sections = vec![CorpusSection::Fresh(CorpusSectionInput {
            book: book("GEN"),
            tokens: CorpusSectionTokens::Owned { tokens: &tokens },
            findings: Some(&lint),
        })];
        let published = encode_corpus(1, Some(stamps()), &sections).unwrap();
        let verified = crate::verify::verify_book(&published.bytes, GEN).expect("verifies");
        assert_eq!(verified.lint_stamps, Some(stamps()));
        assert_eq!(verified.findings.len(), lint.issues.len());
        // Every finding survives whole, fix included — the stamps ride alongside
        // the section, they do not displace anything in it.
        assert!(
            verified
                .findings
                .iter()
                .any(|finding| finding.fix.is_some())
        );
    }

    /// Stamps describe findings. Supplying them for a publication that has none
    /// would be a licence covering nothing, so it is refused rather than stored.
    #[test]
    fn stamps_without_findings_are_refused() {
        let parsed = parse(GEN);
        let sections = vec![CorpusSection::Fresh(CorpusSectionInput {
            book: book("GEN"),
            tokens: CorpusSectionTokens::Parsed {
                source: GEN,
                tokens: &parsed.tokens,
            },
            findings: None,
        })];
        assert!(matches!(
            encode_corpus(0, Some(stamps()), &sections),
            Err(EncodeError::InvalidSectionLayout { .. })
        ));
    }

    #[test]
    fn a_book_published_twice_is_refused() {
        let tokens = owned_tokens(GEN);
        let sections = vec![
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned { tokens: &tokens },
                findings: None,
            }),
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned { tokens: &tokens },
                findings: None,
            }),
        ];
        assert!(matches!(
            encode_corpus(0, None, &sections),
            Err(EncodeError::InvalidSectionLayout {
                reason: LayoutRefusal::DuplicateSection { .. },
                ..
            })
        ));
    }

    /// The reuse gate, and the strongest form of the proof available: the
    /// republication hands back only *bytes* for the untouched book — no tokens,
    /// no findings, nothing to re-encode from — so if the container comes out
    /// complete and verifies, reuse provably happened rather than being inferred
    /// from a counter.
    #[test]
    fn a_republication_reuses_an_untouched_books_sections_verbatim() {
        let gen_tokens = owned_tokens(GEN);
        let exo_tokens = owned_tokens(EXO);
        let gen_lint = lint_of(GEN);
        let exo_lint = lint_of(EXO);
        let first = encode_corpus(
            1,
            Some(stamps()),
            &[
                CorpusSection::Fresh(CorpusSectionInput {
                    book: book("GEN"),
                    tokens: CorpusSectionTokens::Owned {
                        tokens: &gen_tokens,
                    },
                    findings: Some(&gen_lint),
                }),
                CorpusSection::Fresh(CorpusSectionInput {
                    book: book("EXO"),
                    tokens: CorpusSectionTokens::Owned {
                        tokens: &exo_tokens,
                    },
                    findings: Some(&exo_lint),
                }),
            ],
        )
        .expect("first publication");

        // GEN is edited; EXO is untouched and comes back as bytes alone.
        let edited = GEN.replace("beginning.", "beginning, edited.");
        let edited_tokens = owned_tokens(&edited);
        let edited_lint = lint_of(&edited);
        let cached_exo = first.books[1].clone();
        assert_eq!(cached_exo.book, book("EXO"));
        let second = encode_corpus(
            2,
            Some(stamps()),
            &[
                CorpusSection::Fresh(CorpusSectionInput {
                    book: book("GEN"),
                    tokens: CorpusSectionTokens::Owned {
                        tokens: &edited_tokens,
                    },
                    findings: Some(&edited_lint),
                }),
                CorpusSection::Cached(cached_exo.as_cached()),
            ],
        )
        .expect("republication");

        // Only the edited book was encoded, so only it has a bound source.
        assert_eq!(second.sources.len(), 1);
        assert_eq!(second.sources[0].0, book("GEN"));
        // The reused book's sections are the first publication's bytes, exactly.
        let reused = second
            .books
            .iter()
            .find(|published| published.book == book("EXO"))
            .expect("the reused book is in the new sidecar set");
        assert_eq!(reused.bytes, cached_exo.bytes);
        assert_eq!(reused.sections, cached_exo.sections);
        assert_eq!(reused.source_hash, cached_exo.source_hash);
        // And the edited book's are not.
        let republished_gen = second
            .books
            .iter()
            .find(|published| published.book == book("GEN"))
            .unwrap();
        assert_ne!(republished_gen.bytes, first.books[0].bytes);

        // The whole publication still verifies, with the new snapshot id.
        let verified = verify_corpus(
            &second.bytes,
            &[(book("GEN"), edited.as_str()), (book("EXO"), EXO)],
        )
        .expect("verifies");
        assert_eq!(verified.snapshot_id, 2);
        assert_eq!(verified.lint_stamps, Some(stamps()));
        assert_eq!(
            verified.books[1].findings.len(),
            exo_lint.issues.len(),
            "the reused book's findings survive the splice"
        );
        assert_eq!(verified.books[0].findings.len(), edited_lint.issues.len());
    }

    /// Reuse never means trust. Every way a cached section can be wrong is a typed
    /// refusal of the whole publication — never a container carrying one section
    /// nobody checked.
    #[test]
    fn a_cached_section_that_does_not_check_out_is_refused() {
        let tokens = owned_tokens(GEN);
        let lint = lint_of(GEN);
        let first = encode_corpus(
            1,
            None,
            &[CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned { tokens: &tokens },
                findings: Some(&lint),
            })],
        )
        .unwrap();
        let cached = first.books[0].clone();

        let refuses = |cached: CachedBookSections<'_>| {
            matches!(
                encode_corpus(2, None, &[CorpusSection::Cached(cached)]),
                Err(EncodeError::InvalidSectionLayout { .. })
            )
        };

        // (a) a claim about a different book than the bytes describe.
        let mut wrong_book = cached.clone();
        wrong_book.book = book("EXO");
        assert!(refuses(wrong_book.as_cached()));

        // (b) a claim about a different source than the bytes are bound to —
        // exactly what a stale application cache looks like.
        let mut wrong_hash = cached.clone();
        wrong_hash.source_hash ^= 1;
        assert!(refuses(wrong_hash.as_cached()));

        // (c) a claim that a finding section is a token section.
        let mut wrong_kind = cached.clone();
        wrong_kind.sections[0].kind = SectionKind::Finding;
        assert!(refuses(wrong_kind.as_cached()));

        // (d) an extent that runs past the bytes it names.
        let mut past_the_end = cached.clone();
        past_the_end.sections[0].len += 1;
        assert!(refuses(past_the_end.as_cached()));

        // (e) a corrupted byte inside the section: its own checksum catches it,
        // which is the check that makes a splice as safe as an encode.
        let mut corrupt = cached.clone();
        let at = corrupt.sections[0].offset + corrupt.sections[0].len / 2;
        corrupt.bytes[at] ^= 0xff;
        assert!(refuses(corrupt.as_cached()));

        // (f) truncated bytes.
        let mut truncated = cached.clone();
        truncated.bytes.truncate(cached.bytes.len() / 2);
        assert!(refuses(truncated.as_cached()));

        // The untouched original still publishes, so none of the above poisoned
        // anything.
        assert!(encode_corpus(2, None, &[CorpusSection::Cached(cached.as_cached())]).is_ok());
    }

    /// A cached finding section records the stamps its findings were produced
    /// under. Splicing it into a publication that claims different ones — the
    /// case a caller hits by re-publishing after a config change while still
    /// holding yesterday's sidecar — would sign a claim that is not true of those
    /// findings, so it is refused. Presence has to agree in both directions too.
    #[test]
    fn a_cached_finding_section_must_match_the_publications_stamps() {
        let tokens = owned_tokens(GEN);
        let lint = lint_of(GEN);
        let section = |stamps: Option<LintStamps>| {
            encode_corpus(
                1,
                stamps,
                &[CorpusSection::Fresh(CorpusSectionInput {
                    book: book("GEN"),
                    tokens: CorpusSectionTokens::Owned { tokens: &tokens },
                    findings: Some(&lint),
                })],
            )
            .expect("publishes")
        };
        let stamped = section(Some(stamps()));
        let unstamped = section(None);
        let other = LintStamps {
            config_fingerprint: 99,
            engine_stamp: 98,
        };

        let republish = |cached: &PublishedBook, stamps: Option<LintStamps>| {
            encode_corpus(2, stamps, &[CorpusSection::Cached(cached.as_cached())])
        };
        // Different stamps.
        assert!(matches!(
            republish(&stamped.books[0], Some(other)),
            Err(EncodeError::InvalidSectionLayout {
                reason: LayoutRefusal::CachedSectionStampMismatch,
                ..
            })
        ));
        // Stamped section, unstamped publication: the licence would vanish.
        assert!(matches!(
            republish(&stamped.books[0], None),
            Err(EncodeError::InvalidSectionLayout {
                reason: LayoutRefusal::CachedSectionStampMismatch,
                ..
            })
        ));
        // Unstamped section, stamped publication: the licence would appear.
        assert!(matches!(
            republish(&unstamped.books[0], Some(stamps())),
            Err(EncodeError::InvalidSectionLayout {
                reason: LayoutRefusal::CachedSectionStampMismatch,
                ..
            })
        ));
        // Agreement in either direction publishes.
        assert!(republish(&stamped.books[0], Some(stamps())).is_ok());
        assert!(republish(&unstamped.books[0], None).is_ok());
    }

    /// Hand-assembles a container out of finished sections, bypassing
    /// [`encode_corpus`]'s own refusals — a stand-in for a producer this crate
    /// would refuse to be, so the *decoder*'s independent checks are exercised
    /// rather than only the encoder's.
    fn splice(publications: &[&EncodedCorpus]) -> Vec<u8> {
        let mut sections = Vec::new();
        let inspected: Vec<_> = publications
            .iter()
            .flat_map(|published| {
                published.books.iter().flat_map(|book| {
                    book.sections.iter().map(move |extent| {
                        let bytes = &book.bytes[extent.offset..extent.offset + extent.len];
                        (
                            crate::container::inspect_section(bytes).expect("valid"),
                            bytes,
                        )
                    })
                })
            })
            .collect();
        for (section, bytes) in &inspected {
            sections.push(crate::container::ContainerSection::Encoded {
                header: section.header,
                bytes,
            });
        }
        crate::container::write_container_sections(9, &sections).expect("writes")
    }

    /// One publication is one cache decision, and the decoder does not take the
    /// encoder's word for it: a container whose finding sections disagree about
    /// their stamps is refused rather than having one pair picked for it.
    #[test]
    fn a_container_whose_books_disagree_about_their_stamps_is_refused() {
        let gen_tokens = owned_tokens(GEN);
        let exo_tokens = owned_tokens(EXO);
        let gen_lint = lint_of(GEN);
        let exo_lint = lint_of(EXO);
        let mine = encode_corpus(
            1,
            Some(stamps()),
            &[CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &gen_tokens,
                },
                findings: Some(&gen_lint),
            })],
        )
        .unwrap();
        let theirs = |stamps: Option<LintStamps>| {
            encode_corpus(
                1,
                stamps,
                &[CorpusSection::Fresh(CorpusSectionInput {
                    book: book("EXO"),
                    tokens: CorpusSectionTokens::Owned {
                        tokens: &exo_tokens,
                    },
                    findings: Some(&exo_lint),
                })],
            )
            .unwrap()
        };

        // Two different stamp pairs.
        let disagreeing = theirs(Some(LintStamps {
            config_fingerprint: 1,
            engine_stamp: 2,
        }));
        let spliced = splice(&[&mine, &disagreeing]);
        assert_eq!(
            verify_corpus(&spliced, &[(book("GEN"), GEN), (book("EXO"), EXO)]).err(),
            Some(crate::error::DecodeError::InvalidSection)
        );

        // Stamped and unstamped: adopting the stamped half is the partial
        // adoption the batch contract forbids.
        let spliced = splice(&[&mine, &theirs(None)]);
        assert_eq!(
            verify_corpus(&spliced, &[(book("GEN"), GEN), (book("EXO"), EXO)]).err(),
            Some(crate::error::DecodeError::InvalidSection)
        );

        // The same splice with one agreed pair verifies, so it is the disagreement
        // being refused and not the splice itself.
        let spliced = splice(&[&mine, &theirs(Some(stamps()))]);
        assert!(verify_corpus(&spliced, &[(book("GEN"), GEN), (book("EXO"), EXO)]).is_ok());
    }

    /// A finding section whose book has no token section is a section the corpus
    /// walk would never open — and an unopened section is an unverified one, in a
    /// container the caller is about to treat as whole.
    #[test]
    fn a_finding_section_with_no_token_section_is_refused() {
        let tokens = owned_tokens(GEN);
        let lint = lint_of(GEN);
        let published = encode_corpus(
            1,
            None,
            &[CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned { tokens: &tokens },
                findings: Some(&lint),
            })],
        )
        .unwrap();
        let sidecar = &published.books[0];
        let finding = sidecar
            .sections
            .iter()
            .find(|section| section.kind == SectionKind::Finding)
            .expect("the publication has findings");
        let bytes = &sidecar.bytes[finding.offset..finding.offset + finding.len];
        let section = crate::container::inspect_section(bytes).expect("valid section");

        // A container of nothing but that finding section. The writer refuses to
        // build one, so it is assembled here the way a foreign producer could.
        let orphan_only = crate::container::write_container_sections(
            1,
            &[crate::container::ContainerSection::Encoded {
                header: section.header,
                bytes,
            }],
        );
        assert!(
            orphan_only.is_err(),
            "this crate's own writer refuses to orphan a finding section"
        );

        // With a *different* book's tokens beside it, the writer's pairing check
        // is satisfied per book and only the reader can catch the orphan.
        let exo_tokens = owned_tokens(EXO);
        let exo = encode_corpus(
            1,
            None,
            &[CorpusSection::Fresh(CorpusSectionInput {
                book: book("EXO"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &exo_tokens,
                },
                findings: None,
            })],
        )
        .unwrap();
        let exo_token_section = {
            let sidecar = &exo.books[0];
            let extent = sidecar.sections[0];
            let bytes = &sidecar.bytes[extent.offset..extent.offset + extent.len];
            (
                crate::container::inspect_section(bytes).expect("valid"),
                bytes,
            )
        };
        let crafted = crate::container::write_container_sections(
            1,
            &[
                crate::container::ContainerSection::Encoded {
                    header: exo_token_section.0.header,
                    bytes: exo_token_section.1,
                },
                crate::container::ContainerSection::Encoded {
                    header: section.header,
                    bytes,
                },
            ],
        );
        // Even the writer catches this one, because a finding section pairs by
        // book *and* source hash — which is the same rule the reader applies.
        assert!(crafted.is_err());

        // Which leaves the reader's own check to be exercised on bytes no writer
        // here can produce: the golden vector crafts exactly that container by
        // renaming the token section, orphaning the finding section beside it.
        let mut crafted = published.bytes.clone();
        crate::finding_goldens::orphan_finding_section(&mut crafted);
        assert_eq!(
            verify_corpus(&crafted, &[(book("EXO"), GEN)]).err(),
            Some(crate::error::DecodeError::InvalidToc),
            "the orphaned finding section must refuse the whole container"
        );
    }

    /// A corpus verifier that skipped a book, or accepted a source for a book the
    /// container does not have, would hand back something a caller then treats as
    /// the whole publication.
    #[test]
    fn verify_corpus_requires_exactly_one_source_per_book() {
        let gen_tokens = owned_tokens(GEN);
        let exo_tokens = owned_tokens(EXO);
        let sections = vec![
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("GEN"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &gen_tokens,
                },
                findings: None,
            }),
            CorpusSection::Fresh(CorpusSectionInput {
                book: book("EXO"),
                tokens: CorpusSectionTokens::Owned {
                    tokens: &exo_tokens,
                },
                findings: None,
            }),
        ];
        let published = encode_corpus(0, None, &sections).unwrap();

        // Missing one book's source.
        assert!(verify_corpus(&published.bytes, &[(book("GEN"), GEN)]).is_err());
        // An extra source naming a book the container does not carry.
        assert!(
            verify_corpus(
                &published.bytes,
                &[(book("GEN"), GEN), (book("EXO"), EXO), (book("LEV"), GEN)]
            )
            .is_err()
        );
        // Right count, wrong book: the count check passes and the lookup fails.
        assert!(
            verify_corpus(&published.bytes, &[(book("GEN"), GEN), (book("LEV"), EXO)]).is_err()
        );
        // The correct pairing, in the other order — sources are matched by book,
        // not by position.
        assert!(verify_corpus(&published.bytes, &[(book("EXO"), EXO), (book("GEN"), GEN)]).is_ok());
    }
}
