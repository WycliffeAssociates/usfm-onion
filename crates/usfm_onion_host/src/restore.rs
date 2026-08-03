//! Restoring: a whole resident corpus seeded from one packed `corpus.bin`
//! container -- the corpus-grain counterpart to [`crate::publish_corpus`].

use braid::Braid;
use serde::{Deserialize, Serialize};
use usfm_onion::token::BookId;
use usfm_onion_wire::error::DecodeError;

/// One book's exact source, keyed the same way a corpus-grain restore or
/// re-publish needs to address it: by its resident book code *and* its own
/// source key (a packed container names the book but not the key a corpus
/// was originally addressed by).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct PublishedCorpusSource {
    pub book: String,
    pub source_key: String,
    pub source: Vec<u8>,
}

/// Why a warm restore was refused outright.
///
/// A refusal here is about the *call*: bytes that do not verify, or a corpus
/// that cannot be installed. A single book whose cached findings are not
/// adoptable is not a refusal — it seeds anyway and appears in
/// [`braid::RestoreReport::rejected`].
///
/// Deliberately native, not a wasm-facing DTO: `Ingest` carries
/// [`braid::IngestError`] verbatim (`BookId`, `ChapterTarget`, etc.) rather
/// than a String-projected mirror of it. braid must not grow a wire or
/// wasm-bindgen dependency, and the wasm crate already owns a `IngestError`
/// String-projection it converts *from* `braid::IngestError` for its own
/// verbs -- reusing that existing conversion is the "one definition, not a
/// second hand-copy" choice, so this error is not re-derived here just to be
/// tsify-compatible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreError {
    Decode(DecodeError),
    Ingest(braid::IngestError),
}

impl std::fmt::Display for RestoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(error) => write!(f, "{error}"),
            Self::Ingest(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RestoreError {}

/// Restores the whole resident corpus from one packed `corpus.bin`
/// container -- the corpus-grain counterpart to [`crate::publish_corpus`], as
/// a per-book `restoreCorpus`/`verify_book` pairing is to a single
/// publication.
///
/// `records` supplies each book's own source key and exact bound source (a
/// packed container names the book but never the key a corpus was
/// originally addressed by, and a freshly-encoded book's bound source is
/// wire's own serialization, not necessarily any file on disk -- see
/// [`crate::PublishedBookInfo::source`]). Verification is corpus-wide
/// (`verify_corpus`): every book must have exactly one source supplied, and
/// findings that carry stamps must all carry the *same* stamps, checked
/// atomically before anything installs.
///
/// Uses the witness path (`VerifiedCorpus::materialize_owned_tokens`) rather
/// than a second, independent `verify_corpus` pass: one full verification
/// per restore, not two -- the difference between single-digit milliseconds
/// and the better part of a second on an alignment-heavy, tens-of-books
/// corpus.
pub fn restore_published_corpus(
    braid: &mut Braid,
    packed: &[u8],
    records: &[PublishedCorpusSource],
) -> Result<braid::RestoreReport, RestoreError> {
    let mut owned_sources = Vec::with_capacity(records.len());
    for record in records {
        let source = std::str::from_utf8(&record.source)
            .map_err(|_| RestoreError::Decode(DecodeError::InvalidUtf8))?;
        let book = BookId::from_str(&record.book)
            .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
        owned_sources.push((book, source));
    }

    let verified = usfm_onion_wire::corpus_codec::verify_corpus(packed, &owned_sources)
        .map_err(RestoreError::Decode)?;
    let materialized = verified
        .materialize_owned_tokens(packed, &owned_sources)
        .map_err(RestoreError::Decode)?;

    // Any suppression configured on this resident's live config makes
    // `suppressed_count` unknowable from packed bytes alone (they carry no
    // such field), so priming is declined wholesale rather than claiming a
    // stale `0` -- the config a primed summary would be judged against is
    // always the *restoring* side's own live one, which restore adoption
    // already requires to match via the stamp check below.
    let summary_unknowable = !braid.config().lint.suppressed.is_empty();

    let mut books = Vec::with_capacity(verified.books.len());
    for verified_book in verified.books {
        let book = BookId::from_str(&verified_book.receipt.book)
            .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
        let record = records
            .iter()
            .find(|record| record.book == verified_book.receipt.book)
            .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
        let source = std::str::from_utf8(&record.source).unwrap_or_default();
        // An empty source key is a malformed `records` input -- not a decode
        // defect in `packed` itself, but there is no braid::IngestError this
        // maps to either (SourceKey::new only ever rejects emptiness, so
        // there is no valid SourceKey to build one from): treated the same
        // as this loop's other records/container correspondence failures.
        let source_key = braid::SourceKey::new(record.source_key.clone())
            .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
        let tokens = materialized
            .iter()
            .find(|(candidate, _)| *candidate == book)
            .map(|(_, tokens)| tokens.clone())
            .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
        books.push(braid::BookRestoreInput {
            source_key,
            book,
            source: source.to_string(),
            tokens,
            line_ending: usfm_onion::token::LineEnding::detect(source),
            lint: (!summary_unknowable)
                .then_some(())
                .and(verified_book.lint_stamps)
                .map(|_| braid::BookLintPrime {
                    book,
                    source_hash: braid::SourceHash(
                        u64::from_str_radix(&verified_book.receipt.source_hash, 16)
                            .unwrap_or_default(),
                    ),
                    result: usfm_onion::lint::LintResult {
                        summary: summarize_findings(&verified_book.findings),
                        issues: verified_book.findings,
                    },
                }),
        });
    }

    let stamps = verified
        .lint_stamps
        .unwrap_or(usfm_onion_wire::corpus_codec::LintStamps {
            config_fingerprint: 0,
            engine_stamp: 0,
        });

    braid
        .restore_corpus(braid::CorpusRestoreInput::new(
            braid::LintConfigFingerprint(stamps.config_fingerprint),
            braid::LintEngineStamp(stamps.engine_stamp),
            books,
        ))
        .map_err(RestoreError::Ingest)
}

/// Rebuilds a [`usfm_onion::lint::LintSummary`] from a restored finding list
/// -- the same fields `usfm_onion::lint`'s own (private) summarizer derives,
/// with one honest exception: `suppressed_count` is always `0` here, because
/// packed bytes carry the post-suppression `Vec<LintIssue>` and never a
/// separate summary section, so a suppressed finding is simply gone by the
/// time this runs. Callers that need an honest `suppressed_count` must
/// decline to prime from this summary when their own config suppresses
/// anything -- see [`restore_published_corpus`]'s `summary_unknowable` check.
fn summarize_findings(findings: &[usfm_onion::lint::LintIssue]) -> usfm_onion::lint::LintSummary {
    let mut by_category = std::collections::BTreeMap::new();
    let mut by_severity = std::collections::BTreeMap::new();
    let mut by_issue_type = std::collections::BTreeMap::new();
    for issue in findings {
        *by_category.entry(issue.category).or_insert(0) += 1;
        *by_severity.entry(issue.severity).or_insert(0) += 1;
        *by_issue_type.entry(issue.issue_type).or_insert(0) += 1;
    }
    usfm_onion::lint::LintSummary {
        by_category,
        by_severity,
        by_issue_type,
        total_count: findings.len(),
        suppressed_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PublicationCache, publish_corpus};
    use braid::{BookInput, BraidConfig, CorpusInput, SourceKey};
    use usfm_onion::lint::{LintOptions, LintScope};

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

    const GEN: &str = "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n";
    const EXO: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";

    /// The native round trip this whole extraction exists to prove: publish,
    /// verify, restore into a fresh `Braid` -- never touching the wasm
    /// crate, so a native capability regression can never hide behind
    /// wasm-only test coverage.
    #[test]
    fn publish_then_restore_reproduces_the_resident_state_natively() {
        let mut original = empty_resident();
        original
            .replace_corpus(CorpusInput::new(vec![
                BookInput::Usfm {
                    source_key: SourceKey::new("GEN.usfm").unwrap(),
                    book: book("GEN"),
                    source: GEN.to_string(),
                },
                BookInput::Usfm {
                    source_key: SourceKey::new("EXO.usfm").unwrap(),
                    book: book("EXO"),
                    source: EXO.to_string(),
                },
            ]))
            .expect("two books");

        // Copied out as owned values before `publish_corpus`'s mutable
        // borrow, since `lint()` returns a snapshot borrowing `original`
        // (the native return type -- no cloning of token streams for a
        // caller that just wants to read them).
        let (expected_id, expected_books, expected_total_count) = {
            let snapshot = original.lint();
            assert!(
                snapshot
                    .books
                    .iter()
                    .any(|book| !book.result.issues.is_empty()),
                "the fixture must carry a real finding"
            );
            let books: Vec<(
                BookId,
                Vec<(usfm_onion::lint::LintCode, Option<String>, String)>,
            )> = snapshot
                .books
                .iter()
                .map(|book| {
                    (
                        book.book,
                        book.result
                            .issues
                            .iter()
                            .map(|issue| (issue.code, issue.sid.clone(), issue.message.clone()))
                            .collect(),
                    )
                })
                .collect();
            (snapshot.id, books, snapshot.summary.total_count)
        };

        let mut cache = PublicationCache::default();
        let published = publish_corpus(&mut original, &mut cache).expect("publishes");
        assert!(published.books.iter().all(|book| book.encoded));

        let records: Vec<PublishedCorpusSource> = published
            .books
            .iter()
            .map(|book| PublishedCorpusSource {
                book: book.book.clone(),
                source_key: format!("{}.usfm", book.book),
                source: book
                    .source
                    .clone()
                    .expect("a freshly encoded book carries its bound source")
                    .into_bytes(),
            })
            .collect();

        let mut reopened = empty_resident();
        let report =
            restore_published_corpus(&mut reopened, &published.bytes, &records).expect("restores");
        assert_eq!(
            report.seeded.len(),
            2,
            "both books seed: {:?}",
            report.rejected
        );
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        let restored_snapshot = reopened.lint();
        assert_eq!(restored_snapshot.id, expected_id);
        assert_eq!(restored_snapshot.books.len(), expected_books.len());
        for (restored, (expected_book, expected_issues)) in
            restored_snapshot.books.iter().zip(&expected_books)
        {
            assert_eq!(restored.book, *expected_book);
            assert_eq!(
                restored.result.issues.len(),
                expected_issues.len(),
                "{}",
                restored.book
            );
            for (restored, (code, sid, message)) in
                restored.result.issues.iter().zip(expected_issues)
            {
                assert_eq!(restored.code, *code);
                assert_eq!(&restored.sid, sid);
                assert_eq!(&restored.message, message);
            }
        }
        assert_eq!(restored_snapshot.summary.total_count, expected_total_count);
    }

    /// A clean project must be able to restore its *negative* lint result:
    /// "lint ran and found nothing" is evidence, and without it reopening a
    /// clean corpus re-runs every rule. Ported from the wasm-only adapter's
    /// own test of this (which hand-built its `BookRestoreInput`s directly
    /// against `braid::Braid::restore_corpus`) to instead go through the
    /// real `publish_corpus`/`restore_published_corpus` composition -- the
    /// warm path a caller actually uses, not just the seed shape it produces.
    #[test]
    fn an_all_clean_corpus_restores_its_empty_findings_and_reopens_with_no_rule_work() {
        const CLEAN_GEN: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n";
        const CLEAN_EXO: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n";
        let mut original = empty_resident();
        original
            .replace_corpus(CorpusInput::new(vec![
                BookInput::Usfm {
                    source_key: SourceKey::new("GEN.usfm").unwrap(),
                    book: book("GEN"),
                    source: CLEAN_GEN.to_string(),
                },
                BookInput::Usfm {
                    source_key: SourceKey::new("EXO.usfm").unwrap(),
                    book: book("EXO"),
                    source: CLEAN_EXO.to_string(),
                },
            ]))
            .expect("two books");
        assert_eq!(
            original.lint().summary.total_count,
            0,
            "the fixture must actually be clean"
        );

        let mut cache = PublicationCache::default();
        let published = publish_corpus(&mut original, &mut cache).expect("publishes");
        assert!(
            published.books.iter().all(|book| book.encoded),
            "a first publish encodes every book"
        );

        let records: Vec<PublishedCorpusSource> = published
            .books
            .iter()
            .map(|book| PublishedCorpusSource {
                book: book.book.clone(),
                source_key: format!("{}.usfm", book.book),
                source: book.source.clone().unwrap().into_bytes(),
            })
            .collect();

        let mut reopened = empty_resident();
        let report =
            restore_published_corpus(&mut reopened, &published.bytes, &records).expect("restores");
        assert_eq!(report.seeded.len(), 2);
        assert!(
            report.rejected.is_empty(),
            "a clean book's cached result must be adoptable: {:?}",
            report.rejected
        );
        // The no-rule-work assertion: nothing is dirty, so the next `lint()`
        // runs no rules at all.
        assert!(
            reopened.books_awaiting_lint().is_empty(),
            "a restored clean corpus must not need recompute"
        );
        let snapshot = reopened.lint();
        assert_eq!(snapshot.summary.total_count, 0);
        assert!(
            snapshot
                .books
                .iter()
                .all(|book| book.result.issues.is_empty())
        );
    }

    /// A restoring config that suppresses a finding cannot know the packed
    /// bytes' `suppressed_count` (they never carried one) -- the restored
    /// summary must still come back honest via a full recompute, never a
    /// stale `0`.
    #[test]
    fn restore_declines_to_prime_a_summary_a_suppressing_config_cannot_recompute() {
        let mut options = LintOptions::scoped(LintScope::Book);
        options.suppressed = vec![usfm_onion::lint::LintSuppression {
            code: usfm_onion::lint::LintCode::DuplicateVerseNumber,
            sid: "GEN 1:1".to_string(),
        }];
        let mut next = 0u32;
        let mut original = Braid::new(BraidConfig::new(options.clone()), move || {
            next += 1;
            format!("minted-{next}")
        });
        original
            .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book("GEN"),
                source: GEN.to_string(),
            }]))
            .expect("one book");
        let expected_summary = original.lint().summary;
        assert!(expected_summary.suppressed_count >= 1);

        let mut cache = PublicationCache::default();
        let published = publish_corpus(&mut original, &mut cache).expect("publishes");
        let records: Vec<PublishedCorpusSource> = published
            .books
            .iter()
            .map(|book| PublishedCorpusSource {
                book: book.book.clone(),
                source_key: format!("{}.usfm", book.book),
                source: book.source.clone().unwrap().into_bytes(),
            })
            .collect();

        let mut next = 0u32;
        let mut reopened = Braid::new(BraidConfig::new(options), move || {
            next += 1;
            format!("minted-{next}")
        });
        let report =
            restore_published_corpus(&mut reopened, &published.bytes, &records).expect("restores");
        assert_eq!(report.seeded.len(), 1);
        assert!(report.rejected.is_empty());

        let restored_summary = reopened.lint().summary;
        assert_eq!(restored_summary, expected_summary);
    }
}
