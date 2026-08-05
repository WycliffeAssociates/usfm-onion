//! Restoring: a whole resident corpus seeded from one packed `corpus.bin`
//! container -- the corpus-grain counterpart to [`Braid::publish`].

use crate::Braid;
use usfm_onion::token::BookId;
use usfm_onion_wire::error::DecodeError;

/// One book's exact source, keyed the same way a corpus-grain restore or
/// re-publish needs to address it: by its resident book code *and* its own
/// source key (a packed container names the book but not the key a corpus
/// was originally addressed by).
///
/// Native-only, deliberately not `wasm`/`tsify`-derived (v0.1.5, bytes-at-
/// boundary convention): a `source: Vec<u8>` field crossing wasm directly
/// would be a JS `number[]`, exactly the array-of-numbers shape the
/// convention exists to eliminate. The wasm crate builds this type as a
/// plain internal value -- sliced from its own single concatenated buffer
/// plus extent records -- immediately before calling
/// [`Braid::restore_published_corpus`], never exposing it as a JS-facing
/// type of its own.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PublishedCorpusSource {
    pub book: String,
    pub source_key: String,
    pub source: Vec<u8>,
}

/// One book's own packed container and the exact source it was bound to.
///
/// The per-book counterpart to [`PublishedCorpusSource`]: there, the packed
/// bytes are one corpus-wide container and each record supplies only a
/// source; here every record carries its own single-book container, which is
/// what a host that persists one file per book has to hand over.
#[derive(Debug, Clone)]
pub struct RestoreRecord {
    /// The caller's own binding for where the book came from -- normally a
    /// path. This becomes the book's [`crate::SourceKey`], which is why an
    /// empty one is refused ([`RestoreError::EmptySourceKey`]).
    pub path: String,
    pub packed: Vec<u8>,
    /// The exact bytes the container was bound to. Bytes rather than a string
    /// so a host can hand over what it read from disk without a round trip
    /// through a checked `String` it would only hand straight back.
    pub source: Vec<u8>,
}

/// Why a warm restore was refused outright.
///
/// A refusal here is about the *call*: bytes that do not verify, or a corpus
/// that cannot be installed. A single book whose cached findings are not
/// adoptable is not a refusal — it seeds anyway and appears in
/// [`crate::RestoreReport::rejected`].
///
/// Deliberately native, not a wasm-facing DTO: `Ingest` carries
/// [`crate::IngestError`] verbatim (`BookId`, `ChapterTarget`, etc.) rather
/// than a String-projected mirror of it. This error stays a native type:
/// default and serde-only builds of braid activate no wasm glue (tsify and
/// wasm-bindgen enter only behind the `wasm` feature), and the wasm crate
/// already owns a `IngestError`
/// String-projection it converts *from* `crate::IngestError` for its own
/// verbs -- reusing that existing conversion is the "one definition, not a
/// second hand-copy" choice, so this error is not re-derived here just to be
/// tsify-compatible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreError {
    Decode(DecodeError),
    Ingest(crate::IngestError),
    /// A record's own `source_key` is empty, so no `crate::SourceKey` can be
    /// built from it at all (`SourceKey::new` only ever rejects emptiness) --
    /// this cannot be expressed as `Ingest(crate::IngestError::DuplicateSourceKey)`,
    /// which requires an actual valid key to name. A distinct variant rather
    /// than force-fitting into `Decode`: the wasm crate's pre-extraction
    /// behavior classified this case as `{kind: "ingest", error: {kind:
    /// "duplicateSourceKey", source: ""}}`, and that classification is
    /// observable API a consumer may already depend on, so the wasm
    /// conversion reproduces it from this variant rather than this crate
    /// silently reclassifying it as a decode defect (which is what
    /// `RestoreError::Decode(DecodeError::InvalidSection)` would have been
    /// here -- arguably a more honest classification since the defect is in
    /// `records`, not `packed`, but not the one shipped, and changing it is
    /// a decision for a future breaking round, not this one).
    EmptySourceKey,
}

impl std::fmt::Display for RestoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(error) => write!(f, "{error}"),
            Self::Ingest(error) => write!(f, "{error}"),
            Self::EmptySourceKey => write!(f, "a restore record's source key is empty"),
        }
    }
}

impl std::error::Error for RestoreError {}

impl Braid {
    /// Seeds the whole resident corpus from per-book packed containers plus the
    /// sources they were bound to -- the warm cold-open for a host that persists
    /// one container per book, where [`Braid::restore_published_corpus`] is the same
    /// thing for a single corpus-wide publication.
    ///
    /// Composed here because this is the only layer allowed to know both halves:
    /// the bytes are verified and decoded by the wire codec, and the results are
    /// handed to the resident corpus, which never sees a packed byte itself.
    /// Verification is the full trust boundary -- structure, both checksums,
    /// exact source length and content hash, the catalog stamp, every
    /// discriminant and index -- so a container that does not check out is
    /// refused before anything is installed.
    ///
    /// A book whose cached findings cannot be adopted still seeds: residency and
    /// lint-priming are independent facts, so that book arrives with no lex or
    /// parse and is simply awaiting recompute.
    pub fn restore_packed_books(
        &mut self,
        records: &[RestoreRecord],
    ) -> Result<crate::RestoreReport, RestoreError> {
        let mut books = Vec::with_capacity(records.len());
        // Every record is verified individually (`verify_book`, below), but the
        // batch's stamps are what braid's own per-book adoption check compares
        // *every* book's cached lint against, so this loop also has to enforce
        // `verify_corpus`'s own invariant across records -- "findings that carry
        // stamps must all carry the same stamps" -- itself: nothing else here
        // ever compares one record's stamps against another's.
        let mut agreed_stamps: Option<usfm_onion_wire::corpus_codec::LintStamps> = None;
        // Restore adoption already requires stamp equality with this resident's
        // OWN config, so the restoring side's live `LintOptions` -- not anything
        // in the packed bytes -- is the config a primed summary would be judged
        // against. Packed bytes carry the post-suppression `Vec<LintIssue>` (each
        // one individually correct) but never a `suppressed_count`: with any
        // suppression configured that count is unknowable from the bytes alone,
        // so it must not be primed into the cache as if it were known. Findings
        // themselves are still valid and still seed; only the summary is
        // withheld, and the next `lint()` recomputes it honestly.
        let summary_unknowable = !self.config().lint.suppressed.is_empty();
        for record in records {
            let source = std::str::from_utf8(&record.source)
                .map_err(|_| RestoreError::Decode(DecodeError::InvalidUtf8))?;
            let verified = usfm_onion_wire::verify::verify_book(&record.packed, source)
                .map_err(RestoreError::Decode)?;
            if let Some(found) = verified.lint_stamps {
                match agreed_stamps {
                    None => agreed_stamps = Some(found),
                    Some(existing) if existing == found => {}
                    Some(_) => {
                        // Atomic: nothing has been installed into `braid` yet, so
                        // refusing here leaves resident state exactly as it was,
                        // the same guarantee every other rejected mutation gives.
                        return Err(RestoreError::Decode(DecodeError::InvalidSection));
                    }
                }
            }
            let tokens = usfm_onion_wire::verify::materialize_owned_tokens(&record.packed, source)
                .map_err(RestoreError::Decode)?;
            let book = BookId::from_str(&verified.receipt.book)
                .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
            // An empty source key: see `RestoreError::EmptySourceKey`'s own doc
            // comment for why this is its own variant rather than `Decode` or a
            // force-fit `Ingest(crate::IngestError::DuplicateSourceKey)`.
            let source_key =
                crate::SourceKey::new(record.path.clone()).ok_or(RestoreError::EmptySourceKey)?;
            books.push(crate::BookRestoreInput {
                source_key,
                book,
                source: source.to_string(),
                tokens,
                line_ending: usfm_onion::token::LineEnding::detect(source),
                // The findings the container carried are adoptable only if its own
                // stamps say what produced them; braid re-checks them against the
                // resident configuration before it trusts any of it.
                lint: (!summary_unknowable)
                    .then_some(())
                    .and(verified.lint_stamps)
                    .map(|_| crate::BookLintPrime {
                        book,
                        source_hash: crate::SourceHash(
                            u64::from_str_radix(&verified.receipt.source_hash, 16)
                                .unwrap_or_default(),
                        ),
                        result: usfm_onion::lint::LintResult {
                            summary: summarize_findings(&verified.findings),
                            issues: verified.findings,
                        },
                    }),
            });
        }

        // Whatever the batch agreed on above (or the all-zero placeholder when no
        // record carried any stamps at all, which never matches a real
        // config/engine fingerprint and so admits nothing) -- never a second,
        // independent re-verification of one arbitrarily chosen record, which is
        // what let a batch's real, already-checked agreement go unused while
        // re-deriving the exact same fact worse.
        let stamps = agreed_stamps.unwrap_or(usfm_onion_wire::corpus_codec::LintStamps {
            config_fingerprint: 0,
            engine_stamp: 0,
        });

        self.restore_corpus(crate::CorpusRestoreInput::new(
            crate::LintConfigFingerprint(stamps.config_fingerprint),
            crate::LintEngineStamp(stamps.engine_stamp),
            books,
        ))
        .map_err(RestoreError::Ingest)
    }

    /// Restores the whole resident corpus from one packed `corpus.bin`
    /// container -- the corpus-grain counterpart to [`Braid::publish`], as
    /// a per-book [`Braid::restore_packed_books`]/`verify_book` pairing is to a single
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
        &mut self,
        packed: &[u8],
        records: &[PublishedCorpusSource],
    ) -> Result<crate::RestoreReport, RestoreError> {
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
        let summary_unknowable = !self.config().lint.suppressed.is_empty();

        let mut books = Vec::with_capacity(verified.books.len());
        for verified_book in verified.books {
            let book = BookId::from_str(&verified_book.receipt.book)
                .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
            let record = records
                .iter()
                .find(|record| record.book == verified_book.receipt.book)
                .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
            let source = std::str::from_utf8(&record.source).unwrap_or_default();
            // An empty source key: see `RestoreError::EmptySourceKey`'s own doc
            // comment for why this is its own variant rather than `Decode` or a
            // force-fit `Ingest(crate::IngestError::DuplicateSourceKey)`.
            let source_key = crate::SourceKey::new(record.source_key.clone())
                .ok_or(RestoreError::EmptySourceKey)?;
            let tokens = materialized
                .iter()
                .find(|(candidate, _)| *candidate == book)
                .map(|(_, tokens)| tokens.clone())
                .ok_or(RestoreError::Decode(DecodeError::InvalidSection))?;
            books.push(crate::BookRestoreInput {
                source_key,
                book,
                source: source.to_string(),
                tokens,
                line_ending: usfm_onion::token::LineEnding::detect(source),
                lint: (!summary_unknowable)
                    .then_some(())
                    .and(verified_book.lint_stamps)
                    .map(|_| crate::BookLintPrime {
                        book,
                        source_hash: crate::SourceHash(
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

        self.restore_corpus(crate::CorpusRestoreInput::new(
            crate::LintConfigFingerprint(stamps.config_fingerprint),
            crate::LintEngineStamp(stamps.engine_stamp),
            books,
        ))
        .map_err(RestoreError::Ingest)
    }
}

/// Rebuilds a [`usfm_onion::lint::LintSummary`] from a restored finding list
/// -- the same fields `usfm_onion::lint`'s own (private) summarizer derives,
/// with one honest exception: `suppressed_count` is always `0` here, because
/// packed bytes carry the post-suppression `Vec<LintIssue>` and never a
/// separate summary section, so a suppressed finding is simply gone by the
/// time this runs. Callers that need an honest `suppressed_count` must
/// decline to prime from this summary when their own config suppresses
/// anything -- see [`Braid::restore_published_corpus`]'s `summary_unknowable` check.
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
    use crate::{BookInput, BraidConfig, CorpusInput, SourceKey};
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion_wire::corpus_codec::LintStamps;

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

    fn suppressing_options() -> LintOptions {
        let mut options = LintOptions::scoped(LintScope::Book);
        options.suppressed = vec![usfm_onion::lint::LintSuppression {
            code: usfm_onion::lint::LintCode::DuplicateVerseNumber,
            sid: "GEN 1:1".to_string(),
        }];
        options
    }

    fn suppressing_resident() -> Braid {
        let mut next = 0u32;
        Braid::new(BraidConfig::new(suppressing_options()), move || {
            next += 1;
            format!("minted-{next}")
        })
    }

    fn usfm_book(key: &str, code: &str, source: &str) -> BookInput {
        BookInput::Usfm {
            source_key: SourceKey::new(key).expect("a non-empty key"),
            book: book(code),
            source: source.to_string(),
        }
    }

    /// One book's own packed bytes and the exact source they are bound to,
    /// stamped with `stamps` -- a container carrying exactly one token section
    /// and one finding section, which is what `verify_book` and so
    /// [`Braid::restore_packed_books`]'s per-record shape expect, never a
    /// whole multi-book publication like [`Braid::publish`] produces.
    fn encode_one_book(
        resident: &mut Braid,
        target: BookId,
        stamps: LintStamps,
    ) -> (Vec<u8>, String) {
        let snapshot = resident.lint();
        let found = snapshot
            .books
            .iter()
            .find(|entry| entry.book == target)
            .expect("book is resident");
        // Shares the same per-book encode path `Braid::publish_scope` uses
        // (`crate::publication::encode_one_book_container`), rather than a
        // second, hand-rolled `encode_corpus` call here.
        crate::publication::encode_one_book_container(
            snapshot.id.0,
            target,
            found.tokens,
            found.result,
            stamps,
        )
        .expect("one book encodes")
    }

    fn current_stamps(resident: &Braid) -> LintStamps {
        LintStamps {
            config_fingerprint: crate::LintConfigFingerprint::of(&resident.config().lint).0,
            engine_stamp: crate::LintEngineStamp::current().0,
        }
    }

    fn record(path: &str, packed: Vec<u8>, source: &str) -> RestoreRecord {
        RestoreRecord {
            path: path.to_string(),
            packed,
            source: source.as_bytes().to_vec(),
        }
    }

    /// Field-by-field, via exhaustive destructuring (no `..`) of both sides: a
    /// plain per-field `assert_eq!` list is exactly the shape that let
    /// `suppressed_count` silently drop out of a prior version of this
    /// comparison. Destructuring means a new summary field fails to *compile*
    /// here until this helper accounts for it.
    fn assert_summaries_match(
        actual: &usfm_onion::lint::LintSummary,
        expected: &usfm_onion::lint::LintSummary,
    ) {
        let usfm_onion::lint::LintSummary {
            by_category: actual_by_category,
            by_severity: actual_by_severity,
            by_issue_type: actual_by_issue_type,
            total_count: actual_total_count,
            suppressed_count: actual_suppressed_count,
        } = actual;
        let usfm_onion::lint::LintSummary {
            by_category: expected_by_category,
            by_severity: expected_by_severity,
            by_issue_type: expected_by_issue_type,
            total_count: expected_total_count,
            suppressed_count: expected_suppressed_count,
        } = expected;
        assert_eq!(
            actual_total_count, expected_total_count,
            "total_count must match"
        );
        assert_eq!(
            actual_by_category, expected_by_category,
            "by_category must match"
        );
        assert_eq!(
            actual_by_severity, expected_by_severity,
            "by_severity must match"
        );
        assert_eq!(
            actual_by_issue_type, expected_by_issue_type,
            "by_issue_type must match"
        );
        assert_eq!(
            actual_suppressed_count, expected_suppressed_count,
            "suppressed_count must match -- packed bytes cannot carry it, so a \
             restore must either recompute it honestly or decline to prime a \
             summary at all, never claim a stale 0"
        );
    }

    /// A warm reopen must report the same summary a live publish-then-lint of
    /// the same content would -- never a zeroed placeholder with findings
    /// plainly present beside it.
    #[test]
    fn restore_packed_books_recomputes_the_summary_from_the_restored_findings() {
        let mut original = empty_resident();
        original
            .replace_corpus(CorpusInput::new(vec![usfm_book(
                "GEN.usfm",
                "GEN",
                "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n",
            )]))
            .expect("one book");
        let expected_summary = original.lint().summary.clone();
        assert!(
            expected_summary.total_count > 0,
            "the fixture must actually carry a warning"
        );

        let stamps = current_stamps(&original);
        let (packed, source) = encode_one_book(&mut original, book("GEN"), stamps);

        let mut reopened = empty_resident();
        let report = reopened
            .restore_packed_books(&[record("GEN.usfm", packed, &source)])
            .expect("a fresh, matching-stamp restore must succeed");
        assert_eq!(report.seeded, vec![book("GEN")]);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        assert_summaries_match(&reopened.lint().summary, &expected_summary);
    }

    /// Two records individually verify fine but carry different stamps (as if
    /// produced by two different rule-engine builds). The whole restore must
    /// refuse atomically -- never adopt the first record's stamps for the
    /// second's findings -- and leave the resident corpus exactly as it was
    /// before the call.
    #[test]
    fn restore_packed_books_refuses_the_whole_batch_when_records_disagree_on_stamps() {
        let mut source_a = empty_resident();
        source_a
            .replace_corpus(CorpusInput::new(vec![usfm_book(
                "GEN.usfm",
                "GEN",
                "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n",
            )]))
            .expect("one book");
        let stamps_a = current_stamps(&source_a);
        let (packed_gen, source_gen) = encode_one_book(&mut source_a, book("GEN"), stamps_a);

        let mut source_b = empty_resident();
        source_b
            .replace_corpus(CorpusInput::new(vec![usfm_book("EXO.usfm", "EXO", EXO)]))
            .expect("one book");
        // A stamp that cannot possibly match `stamps_a`: the real engine stamp
        // perturbed by one bit is still "some other build produced this",
        // which is all the test needs -- two individually-valid records that
        // disagree with each other.
        let stamps_b = LintStamps {
            config_fingerprint: stamps_a.config_fingerprint,
            engine_stamp: stamps_a.engine_stamp ^ 1,
        };
        let (packed_exo, source_exo) = encode_one_book(&mut source_b, book("EXO"), stamps_b);

        let mut reopened = empty_resident();
        let before_books: Vec<BookId> = reopened
            .books()
            .into_iter()
            .map(|entry| entry.book)
            .collect();
        let error = reopened
            .restore_packed_books(&[
                record("GEN.usfm", packed_gen, &source_gen),
                record("EXO.usfm", packed_exo, &source_exo),
            ])
            .expect_err("disagreeing stamps must refuse the whole batch");
        assert_eq!(
            error,
            RestoreError::Decode(DecodeError::InvalidSection),
            "the refusal must stay typed"
        );
        let after_books: Vec<BookId> = reopened
            .books()
            .into_iter()
            .map(|entry| entry.book)
            .collect();
        assert_eq!(
            after_books, before_books,
            "a refused restore must leave resident state exactly as it was"
        );
        assert!(before_books.is_empty(), "the fresh handle started empty");
    }

    /// Packed bytes carry the post-suppression `Vec<LintIssue>` but no
    /// `suppressed_count` at all, so a config with any suppression configured
    /// makes that count unknowable from the bytes alone. A restore must not
    /// prime a cached summary that quietly claims `0` for it -- it must
    /// decline to prime the cache for the affected book, let the book seed
    /// with no lint result, and let the next `lint()` recompute the whole
    /// thing (findings and summary alike) honestly.
    #[test]
    fn restore_packed_books_declines_to_prime_a_summary_a_suppressing_config_cannot_recompute() {
        let mut original = suppressing_resident();
        original
            .replace_corpus(CorpusInput::new(vec![usfm_book("GEN.usfm", "GEN", GEN)]))
            .expect("one book");
        let expected_summary = original.lint().summary.clone();
        assert!(
            expected_summary.suppressed_count >= 1,
            "the fixture must actually suppress a finding"
        );

        let stamps = current_stamps(&original);
        let (packed, source) = encode_one_book(&mut original, book("GEN"), stamps);

        let mut reopened = suppressing_resident();
        let report = reopened
            .restore_packed_books(&[record("GEN.usfm", packed, &source)])
            .expect("a fresh, matching-stamp restore must still seed the book");
        // The book still seeds -- residency and lint-priming are independent
        // facts -- and this is not a *rejection* either: priming was never
        // attempted for it, so there is nothing to report as refused.
        assert_eq!(report.seeded, vec![book("GEN")]);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        // The post-restore recompute must still return findings, and the
        // summary it recomputes -- including the suppressed count a cached
        // summary could never have supplied -- must match the original.
        let snapshot = reopened.lint();
        let restored_book = snapshot
            .books
            .iter()
            .find(|entry| entry.book == book("GEN"))
            .expect("GEN is resident");
        assert!(
            !restored_book.result.issues.is_empty(),
            "findings must still be returned after the honest recompute"
        );
        assert_summaries_match(&snapshot.summary, &expected_summary);
    }

    /// The native round trip the packed compositions exist to prove: publish,
    /// verify, restore into a fresh `Braid` -- never touching the wasm
    /// crate, so a native capability regression can never hide behind
    /// wasm-only test coverage.
    ///
    /// Supersedes a second, near-identical round-trip test that asserted the
    /// same publish-then-restore path: the two were merged rather than kept
    /// side by side, and this survivor carries the strict superset of their
    /// assertions -- per-book issue code/sid/message *and* the complete
    /// summary compared field-by-field, where the retired one checked only
    /// `total_count`.
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

        // Copied out as owned values before `publish`'s mutable
        // borrow, since `lint()` returns a snapshot borrowing `original`
        // (the native return type -- no cloning of token streams for a
        // caller that just wants to read them).
        let (expected_id, expected_books, expected_summary) = {
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
            (snapshot.id, books, snapshot.summary.clone())
        };

        let published = original.publish().expect("publishes");
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
        let report = reopened
            .restore_published_corpus(&published.bytes, &records)
            .expect("restores");
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
        assert_summaries_match(&restored_snapshot.summary, &expected_summary);
    }

    /// The editor's exact reported scenario, natively: a token-push book
    /// whose tokens carry `_dup_1` sids on a duplicate-verse region — the
    /// shape `js/token-sids.js`'s `normalizeTokenSids` produces (sticky sid
    /// stamped on every token, second `\v 14` region suffixed) — must seed,
    /// pull back verbatim, and survive a publish/restore round trip through
    /// the packed wire. Phase 1 (this release) makes exactly this
    /// representable; see `plans/approved/sid-occurrence-ordinals.md`.
    #[test]
    fn a_dup_suffixed_token_push_seeds_pulls_and_survives_publish_restore() {
        use usfm_onion::parse::parse;
        use usfm_onion::token::{OwnedToken, OwnedTokenParts};

        const DEU: &str = "\\id DEU\n\\c 16\n\\p\n\\v 14 First.\n\\v 14 Second.\n";

        // Rebuild every token whose parsed sid is the bare duplicate anchor
        // and whose own source text falls in the *second* occurrence (after
        // "Second.\n" begins) with the `_dup_1` spelling stamped on it --
        // exactly what an editor's own duplicate-detection pass does, since
        // `parse` itself stays bare in phase 1 (never mints occurrences).
        let parsed = parse(DEU);
        // The second `\v 14` marker's own byte offset -- restamping from
        // there (not from "Second." itself) catches that occurrence's number
        // and marker tokens too, not just its text.
        let second_region_starts = DEU.rfind("\\v 14").expect("fixture has a second \\v 14");
        let tokens: Vec<OwnedToken> = parsed
            .tokens
            .iter()
            .map(|token| {
                let owned = OwnedToken::from_parsed(token);
                let restamp = owned.sid() == Some("DEU 16:14")
                    && token.span.start as usize >= second_region_starts;
                if !restamp {
                    return owned;
                }
                OwnedToken::from_parts(OwnedTokenParts {
                    id: owned.id().as_str(),
                    kind: owned.kind(),
                    source: owned.source(),
                    sid: Some("DEU 16:14_dup_1"),
                    marker: owned.marker_name(),
                    nested: owned.nested(),
                    book_code: owned.book_code().map(|c| (&*c.code, c.is_valid)),
                    number: owned.number_info().cloned(),
                    attributes: owned.attributes(),
                    attribute_source: owned.attribute_list(),
                    attribute_offset: owned.attribute_offset(),
                })
                .expect("restamping only the sid keeps every other fact legal")
            })
            .collect();
        assert!(
            tokens.iter().any(|t| t.sid() == Some("DEU 16:14_dup_1")),
            "the fixture must actually carry the dup-suffixed spelling"
        );

        let mut original = empty_resident();
        original
            .update_book(BookInput::Tokens(crate::BookTokensInput {
                source_key: SourceKey::new("DEU.usfm").unwrap(),
                book: book("DEU"),
                tokens,
                line_ending: usfm_onion::token::LineEnding::Lf,
            }))
            .expect("a dup-suffixed sid is representable in phase 1");

        // Pulled tokens must carry the exact spelling verbatim.
        let pulled = original
            .to_tokens(crate::Scope::book(book("DEU")))
            .expect("DEU is resident")
            .remove(0)
            .tokens;
        assert!(
            pulled.iter().any(|t| t.sid() == Some("DEU 16:14_dup_1")),
            "to_tokens must return the dup-suffixed spelling verbatim"
        );
        assert!(pulled.iter().any(|t| t.sid() == Some("DEU 16:14")));

        // Publish + restore through the packed wire must preserve it.
        let published = original.publish().expect("publishes");
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
        let report = reopened
            .restore_published_corpus(&published.bytes, &records)
            .expect("restores");
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        let restored_pulled = reopened
            .to_tokens(crate::Scope::book(book("DEU")))
            .expect("DEU is resident after restore")
            .remove(0)
            .tokens;
        assert!(
            restored_pulled
                .iter()
                .any(|t| t.sid() == Some("DEU 16:14_dup_1")),
            "the packed round trip must preserve the occurrence, not coerce it back to the bare anchor"
        );
        assert!(restored_pulled.iter().any(|t| t.sid() == Some("DEU 16:14")));
    }

    /// A clean project must be able to restore its *negative* lint result:
    /// "lint ran and found nothing" is evidence, and without it reopening a
    /// clean corpus re-runs every rule. Ported from the wasm-only adapter's
    /// own test of this (which hand-built its `BookRestoreInput`s directly
    /// against `crate::Braid::restore_corpus`) to instead go through the
    /// real `publish`/`restore_published_corpus` composition -- the
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

        let published = original.publish().expect("publishes");
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
        let report = reopened
            .restore_published_corpus(&published.bytes, &records)
            .expect("restores");
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

        let published = original.publish().expect("publishes");
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
        let report = reopened
            .restore_published_corpus(&published.bytes, &records)
            .expect("restores");
        assert_eq!(report.seeded.len(), 1);
        assert!(report.rejected.is_empty());

        let restored_summary = reopened.lint().summary;
        assert_eq!(restored_summary, expected_summary);
    }

    /// Clean-room re-review P1: an empty `source_key` must classify as
    /// `RestoreError::EmptySourceKey` -- not silently reclassified as a
    /// decode defect (which is what falling through to one of this
    /// function's `RestoreError::Decode(DecodeError::InvalidSection)` sites
    /// would have produced). This is the fact the wasm crate's conversion
    /// depends on to reproduce its own pre-extraction classification
    /// (`{kind: "ingest", error: {kind: "duplicateSourceKey", source: ""}}`)
    /// byte-for-byte -- see `usfm_onion_wasm::resident`'s own match arm.
    #[test]
    fn an_empty_source_key_is_its_own_classification_not_a_decode_defect() {
        let mut original = empty_resident();
        original
            .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book("GEN"),
                source: GEN.to_string(),
            }]))
            .expect("one book");
        let published = original.publish().expect("publishes");

        let records = vec![PublishedCorpusSource {
            book: "GEN".to_string(),
            source_key: String::new(),
            source: published.books[0].source.clone().unwrap().into_bytes(),
        }];

        let mut reopened = empty_resident();
        let error = reopened
            .restore_published_corpus(&published.bytes, &records)
            .expect_err("an empty source key must refuse");
        assert_eq!(error, RestoreError::EmptySourceKey);
    }
}
