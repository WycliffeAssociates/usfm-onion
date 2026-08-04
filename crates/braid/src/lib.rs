//! Resident USFM corpus.
//!
//! `Braid` holds a complete ordered corpus of books, each with its exact source
//! bytes, its owned semantic tokens, its duplicate-preserving chapter runs, and
//! its derived stamps. It owns validation, the mutation lifecycle, and
//! current-truth token pulls. It does not own byte layouts (the wire crate's),
//! rule logic (core's), file IO, or scheduling (the application's).
//!
//! # The caller contract
//!
//! Mutate, then hydrate what changed:
//!
//! ```text
//! let effect = braid.update_chapter(target, replacement)?;
//! for scope in braid.to_tokens(&effect)? { /* reconcile scope.tokens */ }
//! ```
//!
//! Every mutating verb applies to resident state *before* it returns, so a read
//! after a mutation is always consistent, and a rejected mutation leaves state
//! provably untouched — same snapshot id, same per-book hashes. Effects are
//! plain values, not subscriptions: braid tracks no per-consumer cursor. A
//! consumer that defers hydration keeps its own pending scope list, and
//! [`Braid::to_tokens`] normalizes whatever it is handed, so naive accumulation
//! across several effects is always correct.
//!
//! Tokens enter braid in exactly one place: `replace_corpus`, `update_book`, and
//! `update_chapter` — the moments the caller knows something braid does not.
//! Source bytes, hashes, and chapter runs are always derived from tokens, never
//! the other way around.

mod baseline;
mod corpus;
mod error;
mod format_patch;
mod input;
mod lint;
mod patch;
mod publication;
mod restore;
mod stamps;
mod state;
mod vref;

use usfm_onion::diff::{DiffSkeleton, diff_skeleton};
use usfm_onion::format::{FormatOptions, FormatToken, format};
use usfm_onion::lint::{LintOptions, LintSummary, apply_token_fix};
use usfm_onion::token::{BookId, OwnedToken, tokens_to_usfm_reconstruct_with_eol};

use crate::baseline::BaselineState;
use crate::corpus::{BookState, chapter_runs};
use crate::format_patch::{PreparedFormatBook, PreparedFormatPatch};
use crate::lint::accumulate;
use crate::patch::ResolvedFix;

pub use crate::baseline::{BaselineError, SetBaselineError};
pub use crate::error::{FormatError, IngestError, PatchError, ScopeError};
pub use crate::format_patch::{FormatPatchError, FormatPatchId, PatchPreparation};
pub use crate::input::{
    BookInput, BookRestoreInput, BookTokensInput, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusRestoreInput, CorpusScope, LineEnding, ScopedOutput, SourceKey,
    SourceOutput,
};
pub use crate::lint::{BookLintSnapshot, LintSnapshot};
pub use crate::patch::{Patch, PatchId, PatchOp, PatchRow};
pub use crate::publication::{
    PublicationCache, PublishError, PublishedBookInfo, PublishedCorpus, ScopedPublication,
    ScopedPublishError, ScopedPublishedBook,
};
pub use crate::restore::{PublishedCorpusSource, RestoreError, RestoreRecord};
pub use crate::stamps::{LintConfigFingerprint, LintEngineStamp};
pub use crate::state::{
    BookEntry, BookLintPrime, LintPrimeInput, MutationEffect, PrimeRejectReason, PrimeRejection,
    PrimeReport, RestoreReport, Scope, ScopeSet, ScopeTokens, SnapshotId, SourceHash,
    TokenIdentity,
};
/// Re-exported so a caller can name a `vref_index` result's own types
/// without a direct `usfm_onion` dependency — the same courtesy every other
/// resident projection here extends over its own core return type.
pub use usfm_onion::vref::{Segment, Utf16Span, VerseProjection, VrefEntry};

/// Resident configuration. Lint options live here so every recompute uses one
/// declared configuration rather than a per-call argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BraidConfig {
    pub lint: LintOptions,
}

impl BraidConfig {
    pub fn new(lint: LintOptions) -> Self {
        Self { lint }
    }
}

/// The resident handle. Instance methods always operate on ingested resident
/// data; stateless operations over caller-supplied values stay on core's own
/// free functions.
pub struct Braid {
    config: BraidConfig,
    /// Ordered and unique by both declared book and source key. Caller order is
    /// preserved — nothing here sorts.
    books: Vec<BookState>,
    snapshot_id: SnapshotId,
    /// Prepared format patches awaiting `apply_format_patch`, keyed by their
    /// position in this vector (their `FormatPatchId::ordinal`). Cleared
    /// whenever a mutation actually changes the snapshot id, since every
    /// entry is bound to the snapshot it was prepared against and would only
    /// ever be found stale from that point on.
    format_patches: Vec<PreparedFormatPatch>,
    /// The application's own identity function, held for the life of the handle.
    ///
    /// Core never invents a token id, so every token a fix or format pass
    /// synthesizes gets one from here. The contract is deliberately thin — a
    /// function returning a `String` — because speed, spelling, and collision
    /// resistance are the application's trade, not braid's. Uniqueness is not
    /// assumed: it is enforced at the residency boundary by the same duplicate-id
    /// check every other ingest path goes through, so a colliding minter is a
    /// typed rejection rather than a corrupted book.
    minter: Box<dyn FnMut() -> String>,
    /// This handle's own packed-publication reuse cache.
    ///
    /// Handle-held rather than a per-call argument for the same reason the
    /// minter is: a repeat [`Braid::publish`] only gets the cache's whole
    /// point -- splice-reuse of every book whose bytes and stamps did not
    /// move -- if it is handed the same cache each time, and a caller
    /// threading one through by hand can always forget to. Self-validating,
    /// so nothing ever has to tell it that state moved: every publish
    /// re-derives reuse from the corpus's own current hashes and stamps.
    publication: PublicationCache,
}

impl std::fmt::Debug for Braid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Braid")
            .field("config", &self.config)
            .field("books", &self.books)
            .field("snapshot_id", &self.snapshot_id)
            .finish_non_exhaustive()
    }
}

impl Braid {
    /// Creates an empty handle bound to the application's id minter.
    ///
    /// The minter is handle-held rather than a per-call argument because every
    /// verb that can synthesize a token needs the same one, and an application
    /// that changed it mid-session would start issuing ids from a different
    /// space than the tokens already resident.
    pub fn new(config: BraidConfig, minter: impl FnMut() -> String + 'static) -> Self {
        Self {
            config,
            books: Vec::new(),
            snapshot_id: SnapshotId::of([]),
            format_patches: Vec::new(),
            minter: Box::new(minter),
            publication: PublicationCache::default(),
        }
    }

    pub fn config(&self) -> &BraidConfig {
        &self.config
    }

    /// The corpus's content-derived identity. With a content-derived id the
    /// "expected" id and the current one are the same value: any publication
    /// built from this state carries it, and any effective mutation replaces it.
    pub fn expected_snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }

    /// Resident books with their derived stamps, in corpus order.
    pub fn books(&self) -> Vec<BookEntry> {
        self.books
            .iter()
            .map(|book| BookEntry {
                source_key: book.source_key.clone(),
                book: book.book,
                source_hash: book.hash,
                token_identity: book.token_identity,
                line_ending: book.line_ending,
            })
            .collect()
    }

    /// One book's chapter-run labels in source order, duplicates included.
    ///
    /// This is the authoritative run order: a book with `\c 1`, `\c 3`, `\c 2`
    /// reports exactly that, and a reopened `\c 1` appears twice.
    pub fn chapter_labels(&self, book: BookId) -> Result<Vec<ChapterLabel>, ScopeError> {
        let state = self.book(book).ok_or(ScopeError::BookNotFound(book))?;
        Ok(state.runs.iter().map(|run| run.label.clone()).collect())
    }

    // ---- mutation ------------------------------------------------------

    /// Replaces the whole corpus with a validated candidate.
    ///
    /// Every candidate book is built, validated, and hashed before resident
    /// state is touched; on any rejection the corpus, its stamps, and its
    /// snapshot id are exactly as they were. Books whose content is unchanged
    /// keep their dirty state and (once lint exists) their cached results, even
    /// when their source key was rebound.
    pub fn replace_corpus(&mut self, corpus: CorpusInput) -> Result<MutationEffect, IngestError> {
        let mut candidates = Vec::with_capacity(corpus.books.len());
        for input in corpus.books {
            candidates.push(BookState::build(input)?);
        }
        validate_unique(&candidates)?;

        let removed: Vec<BookId> = self
            .books
            .iter()
            .filter(|resident| {
                !candidates
                    .iter()
                    .any(|candidate| candidate.book == resident.book)
            })
            .map(|resident| resident.book)
            .collect();

        let mut changed = Vec::new();
        for candidate in &mut candidates {
            match self.book(candidate.book) {
                Some(resident) if resident.content_eq(candidate) => {
                    candidate.inherit_cache(resident);
                }
                Some(resident) => {
                    // A content change is not how a baseline changes: carry
                    // the predecessor's forward rather than letting the fresh
                    // candidate's default `None` erase it. The vref cache
                    // carries forward too — see `crate::vref` for why that
                    // is always safe.
                    candidate.baseline = resident.baseline.clone();
                    candidate.vref_cache = resident.vref_cache.clone();
                    changed.push(Scope::book(candidate.book));
                }
                None => changed.push(Scope::book(candidate.book)),
            }
        }

        // The relative order of books present both before and after — additions
        // and removals alone are not a reorder, and are already observable via
        // `changed`/`removed`; only a genuine permutation of the survivors is.
        let new_order: Vec<BookId> = candidates.iter().map(|candidate| candidate.book).collect();
        let old_retained: Vec<BookId> = self
            .books
            .iter()
            .map(|book| book.book)
            .filter(|book| new_order.contains(book))
            .collect();
        let new_retained: Vec<BookId> = new_order
            .iter()
            .copied()
            .filter(|book| old_retained.contains(book))
            .collect();
        let reordered = (old_retained != new_retained).then(|| new_order.clone());

        self.books = candidates;
        Ok(self.effect_with_reorder(changed, removed, reordered))
    }

    /// Seeds the whole corpus from already-decoded state — the warm cold-open.
    ///
    /// Every book arrives as exact bytes plus the tokens they decode to, so no
    /// book is lexed or parsed. The pairing is what gets checked: a book whose
    /// tokens do not re-emit its own bytes is refused into the report rather than
    /// installed, because after a restore the first edit rewrites the whole book
    /// from its tokens — tokens that spell something else would take unrelated
    /// parts of the file with them.
    ///
    /// Corpus-level validation runs on the manifest as supplied, before anything
    /// else looks at a record: duplicate declared books or source keys refuse the
    /// entire call with resident state untouched. It has to be first, because a
    /// duplicate is a caller mistake about the *manifest* — checking it after
    /// per-record processing would only see the records that survived, so a
    /// manifest naming one book twice could seed one copy and report the other as
    /// a content rejection instead of refusing outright. Per-book refusals are
    /// data, and the caller re-ingests just those books.
    ///
    /// A book's optional `lint` field is validated against this corpus's own
    /// two stamps and this book's own post-seed hash before it is adopted — see
    /// [`RestoreReport`] for exactly what a rejection there does and does not
    /// invalidate. A book still seeds even when its cached lint is refused;
    /// only `SourceTokenMismatch` refuses the book itself.
    pub fn restore_corpus(
        &mut self,
        seed: CorpusRestoreInput,
    ) -> Result<RestoreReport, IngestError> {
        validate_unique_keys(seed.books.iter().map(|book| (book.book, &book.source_key)))?;

        let stamps_ok = seed.config_fingerprint == LintConfigFingerprint::of(&self.config.lint)
            && seed.engine_stamp == LintEngineStamp::current();
        let stamp_reason =
            if seed.config_fingerprint != LintConfigFingerprint::of(&self.config.lint) {
                PrimeRejectReason::ConfigFingerprintMismatch
            } else {
                PrimeRejectReason::EngineStampMismatch
            };

        let mut candidates = Vec::with_capacity(seed.books.len());
        let mut rejected = Vec::new();
        for book in seed.books {
            let expected_source = book.source;
            let lint = book.lint;
            let mut candidate = BookState::build(BookInput::Tokens(BookTokensInput {
                source_key: book.source_key,
                book: book.book,
                tokens: book.tokens,
                line_ending: book.line_ending,
            }))?;
            if candidate.source != expected_source {
                rejected.push(PrimeRejection {
                    book: book.book,
                    reason: PrimeRejectReason::SourceTokenMismatch,
                });
                continue;
            }
            if let Some(prime) = lint {
                match self.validate_prime(&candidate, &prime, stamps_ok, stamp_reason) {
                    Ok((result, patches)) => candidate.install_lint(result, patches),
                    Err(reason) => rejected.push(PrimeRejection {
                        book: book.book,
                        reason,
                    }),
                }
            }
            candidates.push(candidate);
        }

        let seeded = candidates.iter().map(|book| book.book).collect();
        self.books = candidates;
        self.snapshot_id = SnapshotId::of(self.books.iter().map(|book| book.hash));
        // Bypasses `effect`/`effect_with_reorder` (this returns a
        // `RestoreReport`, not a `MutationEffect`), so the token/residency
        // reseed this performs needs its own explicit invalidation of every
        // prepared format patch.
        self.format_patches.clear();
        Ok(RestoreReport { seeded, rejected })
    }

    /// Applies cached lint contributions to an already-resident corpus — the
    /// same validation `restore_corpus` runs per book, addressed by
    /// [`BookId`]/[`SourceHash`] instead of arriving alongside a fresh seed.
    /// Every accepted book's contribution replaces its current one atomically;
    /// a rejected book is left exactly as it was (dirty if it already was).
    pub fn prime_lint_cache(&mut self, input: LintPrimeInput) -> PrimeReport {
        let stamps_ok = input.config_fingerprint == LintConfigFingerprint::of(&self.config.lint)
            && input.engine_stamp == LintEngineStamp::current();
        let stamp_reason =
            if input.config_fingerprint != LintConfigFingerprint::of(&self.config.lint) {
                PrimeRejectReason::ConfigFingerprintMismatch
            } else {
                PrimeRejectReason::EngineStampMismatch
            };

        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for prime in input.books {
            let book = prime.book;
            let Some(index) = self.index_of(book) else {
                rejected.push(PrimeRejection {
                    book,
                    reason: PrimeRejectReason::BookNotResident,
                });
                continue;
            };
            match self.validate_prime(&self.books[index], &prime, stamps_ok, stamp_reason) {
                Ok((result, patches)) => {
                    self.books[index].install_lint(result, patches);
                    accepted.push(book);
                }
                Err(reason) => rejected.push(PrimeRejection { book, reason }),
            }
        }
        PrimeReport { accepted, rejected }
    }

    /// The validation both `restore_corpus` and `prime_lint_cache` run for one
    /// book's cached contribution: the batch's stamps (already compared once
    /// per call, not per book — a mismatch there is the same fact for every
    /// book in the batch), this book's own source hash, and finally that every
    /// fix in the result actually resolves against this book's own tokens.
    /// Ok returns the pieces the caller installs; it never installs them
    /// itself, so a corpus-level rejection this checks for cannot leave one
    /// book adopted and its sibling refused out of the same bad batch.
    fn validate_prime(
        &self,
        book: &BookState,
        prime: &BookLintPrime,
        stamps_ok: bool,
        stamp_reason: PrimeRejectReason,
    ) -> Result<(usfm_onion::lint::LintResult, Vec<ResolvedFix>), PrimeRejectReason> {
        if !stamps_ok {
            return Err(stamp_reason);
        }
        if prime.source_hash != book.hash {
            return Err(PrimeRejectReason::SourceHashMismatch);
        }
        let patches = book
            .try_resolve_cached_fixes(&prime.result)
            .ok_or(PrimeRejectReason::InvalidPatch)?;
        Ok((prime.result.clone(), patches))
    }

    /// Replaces one book, or appends it when it is not resident yet.
    ///
    /// Whole-book replacement is the structural escape hatch: chapter
    /// insertion, deletion, reordering, and duplicate resolution all go through
    /// here rather than through `update_chapter`.
    pub fn update_book(&mut self, replacement: BookInput) -> Result<MutationEffect, IngestError> {
        let mut candidate = BookState::build(replacement)?;
        if let Some(other) = self
            .books
            .iter()
            .find(|book| book.source_key == candidate.source_key && book.book != candidate.book)
        {
            return Err(IngestError::DuplicateSourceKey {
                source: other.source_key.clone(),
            });
        }

        match self.index_of(candidate.book) {
            Some(index) => {
                if self.books[index].content_eq(&candidate) {
                    // Content-identical: a no-op for hydration and for caches,
                    // even if the source key was rebound.
                    candidate.inherit_cache(&self.books[index]);
                    self.books[index] = candidate;
                    Ok(self.effect(Vec::new(), Vec::new()))
                } else {
                    // A content change is not how a baseline changes: carry
                    // the predecessor's forward rather than letting the fresh
                    // candidate's default `None` erase it. The vref cache
                    // carries forward too — safe regardless of how much of
                    // the book actually changed, since every entry is
                    // re-verified against its own run's current tokens on
                    // read (see `crate::vref`).
                    candidate.baseline = self.books[index].baseline.clone();
                    candidate.vref_cache = self.books[index].vref_cache.clone();
                    let book = candidate.book;
                    self.books[index] = candidate;
                    Ok(self.effect(vec![Scope::book(book)], Vec::new()))
                }
            }
            None => {
                let book = candidate.book;
                self.books.push(candidate);
                Ok(self.effect(vec![Scope::book(book)], Vec::new()))
            }
        }
    }

    /// Replaces exactly one existing chapter run with the caller's content.
    ///
    /// The replacement must be that same one run: zero matching resident runs
    /// is `ChapterNotFound`, several is `AmbiguousChapter`, and a replacement
    /// that is a different or additional chapter is `ReplacementLabelMismatch`.
    /// The book's stored line ending is inherited, so an editor pushing `\n`
    /// newline tokens into a CRLF book keeps saving CRLF.
    pub fn update_chapter(
        &mut self,
        target: ChapterTarget,
        replacement: ChapterInput,
    ) -> Result<MutationEffect, IngestError> {
        let (book_index, run_index) = self
            .resolve_chapter(&target)
            .map_err(|error| IngestError::from_scope(error, &target))?;

        let ChapterInput::Tokens(tokens) = replacement;
        validate_replacement_shape(&tokens, &target)?;

        let resident = &self.books[book_index];
        let run = resident.runs[run_index].clone();
        let mut spliced = Vec::with_capacity(resident.tokens.len() + tokens.len());
        spliced.extend_from_slice(&resident.tokens[..run.range.start]);
        spliced.extend(tokens);
        spliced.extend_from_slice(&resident.tokens[run.range.end..]);

        let candidate = resident.rebuilt(spliced)?;
        if resident.content_eq(&candidate) {
            return Ok(self.effect(Vec::new(), Vec::new()));
        }

        // A book with duplicate labels has no unambiguous chapter address to
        // hand a consumer, so its effects report the whole book instead of
        // inventing one.
        let widen = resident.has_duplicate_labels() || candidate.has_duplicate_labels();
        self.books[book_index] = candidate;
        let scope = if widen {
            Scope::book(target.book)
        } else {
            Scope::chapter(target.book, target.label)
        };
        Ok(self.effect(vec![scope], Vec::new()))
    }

    /// Removes a book. Removing an absent book is a no-op, not an error: the
    /// requested end state already holds.
    pub fn remove_book(&mut self, book: BookId) -> MutationEffect {
        match self.index_of(book) {
            Some(index) => {
                self.books.remove(index);
                self.effect(Vec::new(), vec![book])
            }
            None => self.effect(Vec::new(), Vec::new()),
        }
    }

    /// Removes one chapter run's tokens from its book.
    ///
    /// The effect is whole-book: the chapter address the caller used no longer
    /// exists, so there is nothing chapter-scoped left to hydrate.
    pub fn remove_chapter(&mut self, target: ChapterTarget) -> Result<MutationEffect, ScopeError> {
        let (book_index, run_index) = self.resolve_chapter(&target)?;
        let resident = &self.books[book_index];
        let run = resident.runs[run_index].clone();
        let mut tokens = resident.tokens[..run.range.start].to_vec();
        tokens.extend_from_slice(&resident.tokens[run.range.end..]);

        // Removing a whole run always rewrites the book; the only failure
        // `rebuilt` can raise is a duplicate id, which a removal cannot create.
        let candidate = resident
            .rebuilt(tokens)
            .expect("removing tokens cannot introduce a duplicate id");
        self.books[book_index] = candidate;
        Ok(self.effect(vec![Scope::book(target.book)], Vec::new()))
    }

    /// Drops every resident book. Clearing an empty braid is a no-op.
    pub fn clear(&mut self) -> MutationEffect {
        let removed: Vec<BookId> = self.books.iter().map(|book| book.book).collect();
        self.books.clear();
        self.effect(Vec::new(), removed)
    }

    /// Replaces the resident configuration.
    ///
    /// No tokens are rewritten, so `changed` is empty and the snapshot id —
    /// which covers source bytes only — is unchanged. What does change is
    /// staleness: every book is marked for recompute, because the config the
    /// cached results were produced under no longer applies.
    pub fn update_config(&mut self, config: BraidConfig) -> MutationEffect {
        if self.config != config {
            self.config = config;
            for book in &mut self.books {
                book.lint_dirty = true;
                // Dropped, not merely stamped stale: a cached result computed
                // under the old configuration is not an answer to any question
                // anyone can now ask, and the patches resolved from it are
                // addressable until they are gone.
                book.lint = None;
                book.patches.clear();
            }
        }
        self.effect(Vec::new(), Vec::new())
    }

    // ---- baseline --------------------------------------------------------

    /// Declares one book's baseline — "what was last saved" for that book,
    /// the reference point `is_dirty`/`diff_baseline` compare current content
    /// against.
    ///
    /// Never an ingest verb: it only ever touches an existing resident
    /// book's baseline slot, and always leaves current tokens, hashes,
    /// dirty stamps, and the corpus snapshot id exactly as they were —
    /// unconditionally, not just on the happy path. A caller must be able to
    /// rely on that regardless of whether the named book turns out to be a
    /// typo or one that has since been removed, so a book braid does not
    /// hold is refused rather than silently added; there is nowhere else a
    /// baseline could attach to. The candidate is still validated through
    /// the same input path as current data (`BookState::build`), so a
    /// malformed candidate is refused the same way, with every resident
    /// book and every baseline exactly as they were. The returned effect on
    /// success is always the no-op shape, since only the baseline slot
    /// changes.
    pub fn set_baseline(
        &mut self,
        replacement: BookInput,
    ) -> Result<MutationEffect, SetBaselineError> {
        let index = self
            .index_of(replacement.book())
            .ok_or(SetBaselineError::BookNotResident(replacement.book()))?;
        let candidate = BookState::build(replacement).map_err(SetBaselineError::Invalid)?;
        self.books[index].baseline = Some(BaselineState::of(&candidate));
        Ok(self.effect(Vec::new(), Vec::new()))
    }

    /// Drops one book's baseline. Absent is a no-op — the requested end state
    /// (no baseline) already holds. Current content is never touched, and the
    /// returned effect is always the no-op shape: a baseline never
    /// participates in `changed`/`removed`/`reordered`.
    pub fn clear_baseline(&mut self, book: BookId) -> MutationEffect {
        if let Some(index) = self.index_of(book) {
            self.books[index].baseline = None;
        }
        self.effect(Vec::new(), Vec::new())
    }

    // ---- lint ----------------------------------------------------------

    /// Recomputes every dirty book and returns the complete resident snapshot.
    ///
    /// The only recompute verb, and always explicit: no mutation lints
    /// implicitly and no effect carries findings. Exactly the dirty books run
    /// rules — a clean corpus runs none, and a one-chapter edit recomputes that
    /// one whole book. Recompute is whole-book because that is the grain every
    /// document-level rule needs; chapter-grain lint is deliberately absent.
    ///
    /// The returned snapshot borrows resident state, so publishing it copies no
    /// token streams.
    pub fn lint(&mut self) -> LintSnapshot<'_> {
        for book in &mut self.books {
            if book.lint_dirty || book.lint.is_none() {
                book.recompute_lint(&self.config.lint);
            }
        }

        let mut summary = LintSummary::default();
        let mut books = Vec::with_capacity(self.books.len());
        for book in &self.books {
            // Established by the loop above: every book was just recomputed or
            // already held a result.
            let result = book
                .lint
                .as_ref()
                .expect("every resident book holds a result after recompute");
            accumulate(&mut summary, &result.summary);
            books.push(BookLintSnapshot {
                source_key: &book.source_key,
                book: book.book,
                source_hash: book.hash,
                token_identity: book.token_identity,
                tokens: &book.tokens,
                result,
            });
        }
        LintSnapshot {
            id: self.snapshot_id,
            books,
            summary,
        }
    }

    // ---- patches -------------------------------------------------------

    /// Every patch of the current snapshot, in corpus order and then per-book
    /// canonical finding order — which is what assigns each one its ordinal.
    ///
    /// A book awaiting recompute contributes nothing: its stored positions
    /// address the token stream it held when its result was computed, so
    /// publishing them would hand out addresses into a stream that no longer
    /// exists. Call [`Self::lint`] first.
    pub fn patches(&self) -> Vec<Patch> {
        self.resolved()
            .map(|(ordinal, book, resolved)| {
                resolved.published(
                    PatchId {
                        snapshot: self.snapshot_id,
                        ordinal,
                    },
                    book.book,
                )
            })
            .collect()
    }

    /// One patch by id, refusing a stale or unknown one.
    pub fn patch(&self, id: PatchId) -> Result<Patch, PatchError> {
        let (book, resolved) = self.locate(id)?;
        Ok(resolved.published(id, self.books[book].book))
    }

    /// The token stream the patch would produce, without applying it.
    ///
    /// Preview and apply run the *same* pass over the same snapshot, so a preview
    /// cannot describe a result the apply would not produce. The book is
    /// [`Patch::book`] — one patch is one fix, and one fix targets one book.
    ///
    /// A preview is a projection, not a mutation, and it is never admitted to
    /// residency — so it mints nothing. An id is granted at admission, which is
    /// why the tokens here are the id-optional working type: every surviving
    /// token carries the resident id it already had, and a token the fix would
    /// synthesize carries `None` until an apply admits it. Previewing twice, or
    /// previewing and then applying, therefore leaves the application's own id
    /// space exactly as it was.
    pub fn preview_patch(&self, id: PatchId) -> Result<Vec<FormatToken>, PatchError> {
        let (book, resolved) = self.locate(id)?;
        Ok(self.applied_working(book, resolved))
    }

    /// Applies a patch as an ordinary mutation.
    ///
    /// Atomic in the same sense as every other mutating verb: the new token
    /// stream is built, id-checked, and re-serialized as a candidate before
    /// resident state is touched, so a rejection leaves the corpus, its hashes,
    /// and its snapshot id exactly as they were. The patched book is marked for
    /// recompute; nothing lints implicitly.
    pub fn apply_patch(&mut self, id: PatchId) -> Result<MutationEffect, PatchError> {
        let (book_index, resolved) = self.locate(id)?;
        let resolved = resolved.clone();
        let working = self.applied_working(book_index, &resolved);
        let tokens = self.admit(book_index, &resolved, working)?;

        let resident = &self.books[book_index];
        let candidate = resident
            .rebuilt(tokens)
            .map_err(PatchError::InvalidResult)?;
        if resident.content_eq(&candidate) {
            return Ok(self.effect(Vec::new(), Vec::new()));
        }

        // The scope is the run the patch landed in, unless this book's duplicate
        // labels make every chapter address in it ambiguous.
        let scope = resolved
            .rows
            .first()
            .filter(|_| !resident.has_duplicate_labels() && !candidate.has_duplicate_labels())
            .and_then(|row| {
                resident
                    .runs
                    .iter()
                    .find(|run| run.range.contains(&(row.position as usize)))
            })
            .map(|run| Scope::chapter(resident.book, run.label.clone()))
            .unwrap_or_else(|| Scope::book(resident.book));

        self.books[book_index] = candidate;
        Ok(self.effect(vec![scope], Vec::new()))
    }

    /// Runs core's `format` over a scope's current tokens and freezes the
    /// result as a snapshot-bound preparation, without mutating anything.
    ///
    /// `CorpusScope::Chapter` formats only that run's own tokens and splices
    /// the result back into the book, the same slice `update_chapter` cuts —
    /// format rules that need surrounding context (paragraph/verse spacing at
    /// the run's own edges) still see it, but nothing outside the run moves.
    /// `CorpusScope::All` may prepare more than one book at once; each book
    /// that `format` leaves byte-for-byte unchanged is left out of the
    /// preparation entirely. `Unchanged` means every targeted book was
    /// already exactly what `format` would produce.
    pub fn prepare_format_patch(
        &mut self,
        scope: CorpusScope,
        options: FormatOptions,
    ) -> Result<PatchPreparation, FormatError> {
        let targets = self.resolve_format_targets(&scope)?;
        let mut books = Vec::new();
        for (book_index, run_index) in targets {
            let book = &self.books[book_index];
            let current: Vec<FormatToken> = book.tokens.iter().map(FormatToken::from).collect();
            let (formatted, chapter) = match run_index {
                None => (format(&current, options), None),
                Some(run_index) => {
                    let run = &book.runs[run_index];
                    let mut whole = current[..run.range.start].to_vec();
                    whole.extend(format(&current[run.range.clone()], options));
                    whole.extend_from_slice(&current[run.range.end..]);
                    (whole, Some(run.label.clone()))
                }
            };
            if formatted != current {
                books.push(PreparedFormatBook {
                    book: book.book,
                    source_hash: book.hash,
                    chapter,
                    tokens: formatted,
                });
            }
        }

        if books.is_empty() {
            return Ok(PatchPreparation::Unchanged);
        }
        let id = FormatPatchId {
            snapshot: self.snapshot_id,
            ordinal: self.format_patches.len() as u32,
        };
        self.format_patches.push(PreparedFormatPatch { books });
        Ok(PatchPreparation::Ready(id))
    }

    /// Applies a prepared format patch as one atomic mutation.
    ///
    /// Every targeted book's candidate is built and id-checked before
    /// resident state is touched; if any one of them cannot become resident,
    /// none of them commit. Every book the preparation actually changes is
    /// marked for recompute; nothing lints implicitly.
    pub fn apply_format_patch(
        &mut self,
        id: FormatPatchId,
    ) -> Result<MutationEffect, FormatPatchError> {
        if id.snapshot != self.snapshot_id {
            return Err(FormatPatchError::StaleSnapshot {
                expected: self.snapshot_id,
                found: id.snapshot,
            });
        }
        let prepared = self
            .format_patches
            .get(id.ordinal as usize)
            .ok_or(FormatPatchError::UnknownPatch(id))?
            .clone();

        let mut candidates = Vec::with_capacity(prepared.books.len());
        for entry in &prepared.books {
            // Every mutation that can change tokens or residency clears the
            // whole preparation table (see `effect_with_reorder`), so a book
            // this preparation named should always still be resident. That
            // invalidation is a second, separate mechanism from the snapshot
            // check above, not implied by it — the byte-derived snapshot id
            // can stay the same across a book removed and a different one
            // added with identical bytes (declared `BookId` is outside
            // snapshot identity), so this is a real, independently reachable
            // failure mode, not defense-in-depth for an already-impossible
            // case. A typed rejection, never a panic.
            let Some(index) = self.index_of(entry.book) else {
                return Err(FormatPatchError::BookNotResident(entry.book));
            };
            if self.books[index].hash != entry.source_hash {
                return Err(FormatPatchError::StaleSnapshot {
                    expected: self.snapshot_id,
                    found: id.snapshot,
                });
            }

            let mut applied = entry.tokens.clone();
            for token in &mut applied {
                if token.id.is_none() {
                    token.id = Some((self.minter)());
                }
            }
            let tokens = self.admit_format(index, &applied)?;
            let candidate = self.books[index]
                .rebuilt(tokens)
                .map_err(FormatPatchError::InvalidResult)?;
            candidates.push((index, entry.chapter.clone(), candidate));
        }

        let mut changed = Vec::new();
        for (index, chapter, candidate) in &candidates {
            if !self.books[*index].content_eq(candidate) {
                let book = self.books[*index].book;
                changed.push(match chapter {
                    Some(label) => Scope::chapter(book, label.clone()),
                    None => Scope::book(book),
                });
            }
        }
        for (index, _, candidate) in candidates {
            self.books[index] = candidate;
        }
        Ok(self.effect(changed, Vec::new()))
    }

    // ---- reads ---------------------------------------------------------

    /// Current tokens for the requested scopes — the single hydration verb.
    ///
    /// Returns current truth, not state as of any earlier effect. The input is
    /// normalized first: duplicates collapse and a whole-book scope absorbs
    /// that book's chapter scopes, so concatenating several effects' `changed`
    /// lists is always correct. Output is corpus order, then run order.
    pub fn to_tokens<S: Into<ScopeSet>>(&self, scopes: S) -> Result<Vec<ScopeTokens>, ScopeError> {
        let requested = scopes.into();
        // Resolve everything before building any output, so a bad scope cannot
        // produce a partial pull.
        let mut plan: Vec<(usize, Option<Vec<usize>>)> = Vec::new();
        for scope in requested.as_slice() {
            let book_index = self
                .index_of(scope.book)
                .ok_or(ScopeError::BookNotFound(scope.book))?;
            let run_index = match &scope.chapter {
                None => None,
                Some(label) => Some(self.resolve_run(book_index, label)?),
            };
            match plan.iter_mut().find(|(index, _)| *index == book_index) {
                Some((_, runs)) => match (runs.as_mut(), run_index) {
                    // A whole-book request already covers every chapter of that
                    // book, in either arrival order.
                    (_, None) => *runs = None,
                    (None, Some(_)) => {}
                    (Some(existing), Some(run)) => {
                        if !existing.contains(&run) {
                            existing.push(run);
                        }
                    }
                },
                None => plan.push((book_index, run_index.map(|run| vec![run]))),
            }
        }

        plan.sort_by_key(|(index, _)| *index);
        let mut out = Vec::new();
        for (book_index, runs) in plan {
            let book = &self.books[book_index];
            match runs {
                None => out.push(ScopeTokens {
                    book: book.book,
                    chapter: None,
                    tokens: book.tokens.clone(),
                }),
                Some(mut runs) => {
                    runs.sort_unstable();
                    for run_index in runs {
                        let run = &book.runs[run_index];
                        out.push(ScopeTokens {
                            book: book.book,
                            chapter: Some(run.label.clone()),
                            tokens: book.tokens[run.range.clone()].to_vec(),
                        });
                    }
                }
            }
        }
        Ok(out)
    }

    /// The exact bytes a scope would be saved as.
    ///
    /// A book's bytes are authoritative: for USFM ingest they are the supplied
    /// bytes verbatim, and after any edit they are the token stream re-emitted
    /// with the book's line ending. A chapter scope re-emits that run, so for
    /// the one case where a book's stored bytes are not derivable — a
    /// never-edited mixed-ending file — a chapter projection reports the book's
    /// leading ending rather than the run's own mixture.
    pub fn to_usfm(&self, scope: CorpusScope) -> Result<ScopedOutput<String>, ScopeError> {
        match scope {
            CorpusScope::All => Ok(ScopedOutput::All(
                self.books
                    .iter()
                    .map(|book| SourceOutput {
                        source_key: book.source_key.clone(),
                        book: book.book,
                        value: book.source.clone(),
                    })
                    .collect(),
            )),
            CorpusScope::Book(book) => {
                let state = self.book(book).ok_or(ScopeError::BookNotFound(book))?;
                Ok(ScopedOutput::Single(state.source.clone()))
            }
            CorpusScope::Chapter(target) => {
                let (book_index, run_index) = self.resolve_chapter(&target)?;
                let book = &self.books[book_index];
                let run = &book.runs[run_index];
                Ok(ScopedOutput::Single(tokens_to_usfm_reconstruct_with_eol(
                    &book.tokens[run.range.clone()],
                    book.line_ending,
                )))
            }
        }
    }

    /// The scope's vref index — every in-scope verse's lossless projection,
    /// in stream order — content-identical to
    /// `usfm_onion::vref::tokens_to_vref_index` over the same tokens, the
    /// only difference being that a resident read may serve some or all of
    /// a book's chapter runs from cache instead of walking their tokens
    /// again. `&mut self`, the same explicit-recompute shape as `lint()`:
    /// this is where the cache is read, verified, and (for whatever it
    /// misses) updated — nothing here is computed as a side effect of a
    /// mutation.
    ///
    /// `CorpusScope::Chapter` reads and updates only the one run's own cache
    /// entry; a whole-book read (`Book`/`All`) additionally prunes every
    /// entry that does not belong to one of the book's current runs, so the
    /// cache never grows past the book's own run count on that path. See
    /// `crate::vref` for why a run's own `TokenIdentity` is a sound cache key
    /// for this specific projection.
    pub fn vref_index(
        &mut self,
        scope: CorpusScope,
    ) -> Result<ScopedOutput<Vec<VrefEntry>>, ScopeError> {
        match scope {
            CorpusScope::All => {
                let mut out = Vec::with_capacity(self.books.len());
                for index in 0..self.books.len() {
                    let entries = self.book_vref_entries(index);
                    let book = &self.books[index];
                    out.push(SourceOutput {
                        source_key: book.source_key.clone(),
                        book: book.book,
                        value: entries,
                    });
                }
                Ok(ScopedOutput::All(out))
            }
            CorpusScope::Book(book) => {
                let index = self.index_of(book).ok_or(ScopeError::BookNotFound(book))?;
                Ok(ScopedOutput::Single(self.book_vref_entries(index)))
            }
            CorpusScope::Chapter(target) => {
                let (book_index, run_index) = self.resolve_chapter(&target)?;
                Ok(ScopedOutput::Single(
                    self.run_vref_entries(book_index, run_index),
                ))
            }
        }
    }

    /// Every run's vref entries, concatenated in run order — the same order
    /// the token stream itself puts them in, since runs partition it without
    /// gaps or overlaps. Rebuilds the book's whole cache in the same pass:
    /// every run this visits either reuses a matching old entry (moved, not
    /// recomputed) or computes fresh, and nothing left over from a run that
    /// no longer exists in this shape survives past this call.
    fn book_vref_entries(&mut self, book_index: usize) -> Vec<VrefEntry> {
        let run_count = self.books[book_index].runs.len();
        let mut old_cache = std::mem::take(&mut self.books[book_index].vref_cache);
        let mut fresh_cache = Vec::with_capacity(run_count);
        let mut entries = Vec::new();
        // The one visitor state a whole-book walk carries across a `\c` and
        // never clears (see `crate::vref`) — threaded run to run in corpus
        // order, which is what makes each run's own incoming state actually
        // known rather than guessed.
        let mut carried_block_state = None;
        for run_index in 0..run_count {
            let range = self.books[book_index].runs[run_index].range.clone();
            let cached = vref::take_or_compute(
                &mut old_cache,
                &self.books[book_index].tokens[range],
                carried_block_state,
            );
            entries.extend(cached.entries.iter().cloned());
            carried_block_state = cached.outgoing_block_state;
            fresh_cache.push(cached);
        }
        self.books[book_index].vref_cache = fresh_cache;
        // A duplicate/reopened `\c` run can share a sid with an earlier run;
        // the whole-stream projection this must stay equivalent to folds
        // that through one shared index (first-seen position, last write
        // wins), so the per-run concatenation redoes exactly that fold once
        // here — see `vref::merge_by_sid`.
        vref::merge_by_sid(entries)
    }

    /// A single run's own entries — but correctness for *that* run still
    /// depends on the block-support state every earlier run in the book
    /// carries into it (see `crate::vref`), so this resolves runs `0..=
    /// run_index` in corpus order, threading each one's outgoing state into
    /// the next. Earlier runs' own entries are discarded once their
    /// outgoing state is read; only `run_index`'s are returned. Every run
    /// visited (including the earlier ones) still folds into the cache —
    /// without evicting a sibling entry, the same policy a lone
    /// chapter-scoped read already followed before this fix.
    fn run_vref_entries(&mut self, book_index: usize, run_index: usize) -> Vec<VrefEntry> {
        let mut carried_block_state = None;
        for earlier in 0..=run_index {
            let range = self.books[book_index].runs[earlier].range.clone();
            let cached = vref::run_entries(
                &self.books[book_index].vref_cache,
                &self.books[book_index].tokens[range],
                carried_block_state,
            );
            carried_block_state = cached.outgoing_block_state;
            if earlier == run_index {
                let entries = cached.entries.clone();
                if !self.books[book_index]
                    .vref_cache
                    .iter()
                    .any(|run| run.matches(cached.identity, cached.incoming_block_state))
                {
                    self.books[book_index].vref_cache.push(cached);
                }
                return entries;
            }
            if !self.books[book_index]
                .vref_cache
                .iter()
                .any(|run| run.matches(cached.identity, cached.incoming_block_state))
            {
                self.books[book_index].vref_cache.push(cached);
            }
        }
        unreachable!("the loop always returns at earlier == run_index")
    }

    /// Whether a scope's exact serialized bytes differ from its baseline.
    ///
    /// A book with no declared baseline is always dirty: there is no saved
    /// equality proof to compare against, so absence is never read as "clean"
    /// or synthesized into any other answer. `CorpusScope::All` is dirty if
    /// any resident book is. `CorpusScope::Chapter` compares only that run's
    /// own bytes against the baseline run sharing its label; a baseline with
    /// no run of that label is dirty for the same reason a missing book
    /// baseline is, and duplicate labels on either side — current or
    /// baseline — make the scope ambiguous, consistent with
    /// `update_chapter`/`to_usfm`.
    pub fn is_dirty(&self, scope: CorpusScope) -> Result<bool, ScopeError> {
        match scope {
            CorpusScope::All => Ok(self.books.iter().any(Self::book_is_dirty)),
            CorpusScope::Book(book) => {
                let state = self.book(book).ok_or(ScopeError::BookNotFound(book))?;
                Ok(Self::book_is_dirty(state))
            }
            CorpusScope::Chapter(target) => {
                let (book_index, run_index) = self.resolve_chapter(&target)?;
                let book = &self.books[book_index];
                let Some(baseline) = &book.baseline else {
                    return Ok(true);
                };
                let matches: Vec<usize> = baseline
                    .runs
                    .iter()
                    .enumerate()
                    .filter(|(_, run)| run.label == target.label)
                    .map(|(index, _)| index)
                    .collect();
                match matches.len() {
                    0 => Ok(true),
                    1 => {
                        let current_run = &book.runs[run_index];
                        let baseline_run = &baseline.runs[matches[0]];
                        let current_bytes = tokens_to_usfm_reconstruct_with_eol(
                            &book.tokens[current_run.range.clone()],
                            book.line_ending,
                        );
                        let baseline_bytes = tokens_to_usfm_reconstruct_with_eol(
                            &baseline.tokens[baseline_run.range.clone()],
                            baseline.line_ending,
                        );
                        Ok(current_bytes != baseline_bytes)
                    }
                    matches => Err(ScopeError::AmbiguousChapter { target, matches }),
                }
            }
        }
    }

    /// A scope's baseline diff, reusing core's own `diff_skeleton` — braid
    /// adds no diff model of its own. Errors typed `MissingBaseline` when any
    /// requested book has none declared, rather than synthesizing an
    /// "everything added" or "everything unchanged" skeleton for it.
    pub fn diff_baseline(
        &self,
        scope: CorpusScope,
    ) -> Result<ScopedOutput<DiffSkeleton<OwnedToken>>, BaselineError> {
        match scope {
            CorpusScope::All => {
                let missing: Vec<BookId> = self
                    .books
                    .iter()
                    .filter(|book| book.baseline.is_none())
                    .map(|book| book.book)
                    .collect();
                if !missing.is_empty() {
                    return Err(BaselineError::MissingBaseline { books: missing });
                }
                Ok(ScopedOutput::All(
                    self.books
                        .iter()
                        .map(|book| {
                            let baseline = book.baseline.as_ref().expect("checked missing above");
                            SourceOutput {
                                source_key: book.source_key.clone(),
                                book: book.book,
                                value: diff_skeleton(&baseline.tokens, &book.tokens),
                            }
                        })
                        .collect(),
                ))
            }
            CorpusScope::Book(book_id) => {
                let book = self
                    .book(book_id)
                    .ok_or(ScopeError::BookNotFound(book_id))?;
                let baseline = book
                    .baseline
                    .as_ref()
                    .ok_or(BaselineError::MissingBaseline {
                        books: vec![book_id],
                    })?;
                Ok(ScopedOutput::Single(diff_skeleton(
                    &baseline.tokens,
                    &book.tokens,
                )))
            }
            CorpusScope::Chapter(target) => {
                let (book_index, run_index) = self.resolve_chapter(&target)?;
                let book = &self.books[book_index];
                let baseline = book
                    .baseline
                    .as_ref()
                    .ok_or(BaselineError::MissingBaseline {
                        books: vec![book.book],
                    })?;
                let matches: Vec<usize> = baseline
                    .runs
                    .iter()
                    .enumerate()
                    .filter(|(_, run)| run.label == target.label)
                    .map(|(index, _)| index)
                    .collect();
                match matches.len() {
                    0 => Err(BaselineError::MissingBaseline {
                        books: vec![book.book],
                    }),
                    1 => {
                        let current_run = &book.runs[run_index];
                        let baseline_run = &baseline.runs[matches[0]];
                        Ok(ScopedOutput::Single(diff_skeleton(
                            &baseline.tokens[baseline_run.range.clone()],
                            &book.tokens[current_run.range.clone()],
                        )))
                    }
                    matches => Err(BaselineError::Scope(ScopeError::AmbiguousChapter {
                        target,
                        matches,
                    })),
                }
            }
        }
    }

    /// Books awaiting a lint recompute, in corpus order.
    ///
    /// This is the lint-cache axis only — whether `lint()` has rules left to
    /// run for a book — and says nothing about whether that book's saved
    /// bytes differ from its baseline. "Dirty" on its own is reserved for the
    /// baseline axis ([`Self::is_dirty`]); this name says which one it means.
    /// Derived from authoritative stamps rather than drained from a queue, so
    /// reading it twice is safe.
    pub fn books_awaiting_lint(&self) -> Vec<BookId> {
        self.books
            .iter()
            .filter(|book| book.lint_dirty)
            .map(|book| book.book)
            .collect()
    }

    // ---- internals -----------------------------------------------------

    /// The current patch table with its ordinals: corpus order, then each
    /// book's own canonical finding order.
    fn resolved(&self) -> impl Iterator<Item = (u32, &BookState, &ResolvedFix)> {
        self.books
            .iter()
            .filter(|book| !book.lint_dirty)
            .flat_map(|book| book.patches.iter().map(move |resolved| (book, resolved)))
            .enumerate()
            .map(|(ordinal, (book, resolved))| (ordinal as u32, book, resolved))
    }

    /// Resolves a patch id to its book and resolved fix, applying both halves of
    /// the staleness rule: the corpus's identity and the target book's own hash
    /// must be the ones the patch was resolved against. The hash is the half
    /// that matters after a mutation elsewhere in the corpus put the id's own
    /// snapshot back in play.
    fn locate(&self, id: PatchId) -> Result<(usize, &ResolvedFix), PatchError> {
        if id.snapshot != self.snapshot_id {
            return Err(PatchError::StaleSnapshot {
                expected: self.snapshot_id,
                found: id.snapshot,
            });
        }
        let (_, book, resolved) = self
            .resolved()
            .find(|(ordinal, ..)| *ordinal == id.ordinal)
            .ok_or(PatchError::UnknownPatch(id))?;
        if resolved.source_hash != book.hash {
            return Err(PatchError::StaleSnapshot {
                expected: self.snapshot_id,
                found: id.snapshot,
            });
        }
        let index = self
            .index_of(book.book)
            .expect("a located patch names a resident book");
        Ok((index, resolved))
    }

    /// The fix pass itself: one book's resident tokens converted to the working
    /// type, core's fix applied, nothing else.
    ///
    /// Deliberately pure — `&self`, no minter, no admission. Identity is granted
    /// when a token becomes resident, not when a pass invents it, so a token this
    /// produces may carry no id at all. That is what lets preview and apply share
    /// one pass: the projection is identical, and only the apply goes on to admit
    /// it. Core owns what the fix means; braid owns the conversions around it.
    fn applied_working(&self, book_index: usize, resolved: &ResolvedFix) -> Vec<FormatToken> {
        let book = &self.books[book_index];
        let working: Vec<FormatToken> = book.tokens.iter().map(FormatToken::from).collect();
        apply_token_fix(&working, &resolved.fix)
    }

    /// Admission: give every id-less token an id from the handle's minter, then
    /// convert the pass's output back to resident tokens.
    ///
    /// The single place the application's identity function is invoked, and the
    /// enforcement checkpoint behind it: the conversion demands an id, so the
    /// sweep is what makes an unaddressable resident token structurally
    /// unreachable rather than merely unlikely. Uniqueness is not assumed —
    /// `rebuilt` still rejects a collision.
    fn admit(
        &mut self,
        book_index: usize,
        resolved: &ResolvedFix,
        mut applied: Vec<FormatToken>,
    ) -> Result<Vec<OwnedToken>, PatchError> {
        for token in &mut applied {
            if token.id.is_none() {
                token.id = Some((self.minter)());
            }
        }

        let book = &self.books[book_index];
        let mut by_id: rustc_hash::FxHashMap<&str, &OwnedToken> =
            rustc_hash::FxHashMap::with_capacity_and_hasher(
                book.tokens.len(),
                rustc_hash::FxBuildHasher,
            );
        for token in &book.tokens {
            by_id.insert(token.id().as_str(), token);
        }
        // A token the pass synthesized descends from the fix's target, which is
        // the anchor core cloned its shape from.
        let target = by_id.get(resolved.fix.target_token_id()).copied();

        applied
            .iter()
            .map(|token| {
                let anchor = token
                    .id
                    .as_deref()
                    .and_then(|id| by_id.get(id).copied())
                    .or(target);
                OwnedToken::from_format_token(token, anchor)
                    .map_err(|error| PatchError::InvalidResult(IngestError::InvalidToken(error)))
            })
            .collect()
    }

    fn index_of(&self, book: BookId) -> Option<usize> {
        self.books.iter().position(|state| state.book == book)
    }

    fn book(&self, book: BookId) -> Option<&BookState> {
        self.books.iter().find(|state| state.book == book)
    }

    fn resolve_chapter(&self, target: &ChapterTarget) -> Result<(usize, usize), ScopeError> {
        let book_index = self
            .index_of(target.book)
            .ok_or(ScopeError::BookNotFound(target.book))?;
        let run_index = self.resolve_run(book_index, &target.label)?;
        Ok((book_index, run_index))
    }

    fn resolve_run(&self, book_index: usize, label: &ChapterLabel) -> Result<usize, ScopeError> {
        let book = &self.books[book_index];
        let matches = book.matching_runs(label);
        match matches.len() {
            0 => Err(ScopeError::ChapterNotFound(state::target(book.book, label))),
            1 => Ok(matches[0]),
            matches => Err(ScopeError::AmbiguousChapter {
                target: state::target(book.book, label),
                matches,
            }),
        }
    }

    /// A book is dirty exactly when it has no baseline, or its exact current
    /// bytes differ from its exact baseline bytes. The hash is only a cheap
    /// selector for the common case — a mismatch is conclusive, but a match
    /// still falls through to the exact byte comparison rather than being
    /// trusted as proof of equality on its own.
    fn book_is_dirty(book: &BookState) -> bool {
        match &book.baseline {
            None => true,
            Some(baseline) => book.hash != baseline.hash || book.source != baseline.source,
        }
    }

    /// Resolves a format scope to its target books and, for a chapter scope,
    /// the one run within that book to format — `None` means the whole book.
    fn resolve_format_targets(
        &self,
        scope: &CorpusScope,
    ) -> Result<Vec<(usize, Option<usize>)>, ScopeError> {
        match scope {
            CorpusScope::All => Ok((0..self.books.len()).map(|index| (index, None)).collect()),
            CorpusScope::Book(book) => {
                let index = self
                    .index_of(*book)
                    .ok_or(ScopeError::BookNotFound(*book))?;
                Ok(vec![(index, None)])
            }
            CorpusScope::Chapter(target) => {
                let (book_index, run_index) = self.resolve_chapter(target)?;
                Ok(vec![(book_index, Some(run_index))])
            }
        }
    }

    /// Converts a fully-minted format working stream back to resident tokens.
    ///
    /// Unlike [`Self::admit`], there is no single fix target to fall back on
    /// as an anchor: a whole-book/chapter format pass has no one token every
    /// synthesized token descends from. A survivor (its id already resident)
    /// anchors on itself. A synthesized token — one core inserted structurally
    /// (a default paragraph marker, a structural linebreak) — carries no id,
    /// but core still stamps it with the sid of the verse content around it,
    /// and every resident token sharing that sid carries the identical
    /// structured form, so any one of them resolves it just as well as the
    /// token the synthesized one happens to sit next to. Without this, every
    /// structurally-inserted marker/newline in a real book would refuse with
    /// `UnresolvableSid`, since a same-id anchor can never exist for a token
    /// that never had a resident id in the first place. A token that needs a
    /// payload fact neither its id nor its sid can anchor (a book code, a
    /// parsed number — core's format pass never synthesizes either) still
    /// converts with no anchor and is refused rather than guessed, the same
    /// "refuse, never invent" rule the residency checkpoint already enforces
    /// for a fix.
    fn admit_format(
        &self,
        book_index: usize,
        applied: &[FormatToken],
    ) -> Result<Vec<OwnedToken>, FormatPatchError> {
        let book = &self.books[book_index];
        let mut by_id: rustc_hash::FxHashMap<&str, &OwnedToken> =
            rustc_hash::FxHashMap::with_capacity_and_hasher(
                book.tokens.len(),
                rustc_hash::FxBuildHasher,
            );
        let mut by_sid: rustc_hash::FxHashMap<&str, &OwnedToken> = rustc_hash::FxHashMap::default();
        for token in &book.tokens {
            by_id.insert(token.id().as_str(), token);
            if let Some(sid) = token.sid() {
                by_sid.entry(sid).or_insert(token);
            }
        }

        applied
            .iter()
            .map(|token| {
                let anchor = token
                    .id
                    .as_deref()
                    .and_then(|id| by_id.get(id).copied())
                    .or_else(|| {
                        token
                            .sid
                            .as_deref()
                            .and_then(|sid| by_sid.get(sid).copied())
                    });
                OwnedToken::from_format_token(token, anchor).map_err(|error| {
                    FormatPatchError::InvalidResult(IngestError::InvalidToken(error))
                })
            })
            .collect()
    }

    /// Recomputes the content-derived snapshot id and packages the effect. The
    /// id falls out of installed state, which is why a no-op preserves it
    /// without any special case.
    fn effect(&mut self, changed: Vec<Scope>, removed: Vec<BookId>) -> MutationEffect {
        self.effect_with_reorder(changed, removed, None)
    }

    /// Same as [`Self::effect`], plus the corpus-order signal only
    /// [`Self::replace_corpus`] can produce: the full new book order, when the
    /// relative order of the books that persisted across the call changed,
    /// `None` otherwise. A pure reorder rewrites no tokens — so
    /// `changed`/`removed` stay empty — but it does change `snapshot_id`
    /// (order is part of the corpus's content-derived identity), and without
    /// this field that new order was otherwise unobservable through
    /// `to_tokens`.
    fn effect_with_reorder(
        &mut self,
        changed: Vec<Scope>,
        removed: Vec<BookId>,
        reordered: Option<Vec<BookId>>,
    ) -> MutationEffect {
        let new_id = SnapshotId::of(self.books.iter().map(|book| book.hash));
        // Every mutating verb that produces a `MutationEffect` runs through
        // here, so this is the one place to drop every prepared format
        // patch — unconditionally, not only when the byte-derived snapshot
        // id moves. Format consumes the token stream and which book holds
        // which position, not just source bytes: an identity-only token
        // push (same bytes, different stable ids) or a same-bytes book swap
        // under `replace_corpus` (declared `BookId` is outside snapshot
        // identity) both leave `new_id == self.snapshot_id` while silently
        // invalidating what a preparation actually recorded. Clearing here
        // also covers the ordinary snapshot-changed case, so there is no
        // separate conditional to keep in sync with it.
        self.format_patches.clear();
        self.snapshot_id = new_id;
        MutationEffect {
            snapshot_id: self.snapshot_id,
            changed,
            removed,
            reordered,
        }
    }
}

/// Corpus-level uniqueness. Both keys must be unique: the declared book keys
/// resident scope and wire identity, the source key binds it to where it came
/// from.
fn validate_unique(candidates: &[BookState]) -> Result<(), IngestError> {
    validate_unique_keys(
        candidates
            .iter()
            .map(|candidate| (candidate.book, &candidate.source_key)),
    )
}

/// The same rule over any manifest of `(declared book, source key)` pairs,
/// whatever it is a manifest *of*.
///
/// Split out because a seed has to be checked before its records are turned into
/// anything: a manifest with two records for one book is one caller mistake with
/// one answer — refuse the call — and a check that runs after per-record
/// processing can only see the records that survived it.
fn validate_unique_keys<'a>(
    manifest: impl Iterator<Item = (BookId, &'a SourceKey)> + Clone,
) -> Result<(), IngestError> {
    for (index, (book, source_key)) in manifest.clone().enumerate() {
        let duplicates: Vec<SourceKey> = manifest
            .clone()
            .filter(|(other, _)| *other == book)
            .map(|(_, key)| key.clone())
            .collect();
        if duplicates.len() > 1 {
            return Err(IngestError::DuplicateBook {
                book,
                sources: duplicates,
            });
        }
        if manifest
            .clone()
            .take(index)
            .any(|(_, other)| other == source_key)
        {
            return Err(IngestError::DuplicateSourceKey {
                source: source_key.clone(),
            });
        }
    }
    Ok(())
}

/// A chapter replacement must be exactly the targeted run.
///
/// An empty replacement counts as front matter, so clearing a book's front
/// matter is expressible while emptying a chapter is not — content removal is
/// `remove_chapter`'s job, and structural change is `update_book`'s.
fn validate_replacement_shape(
    tokens: &[OwnedToken],
    target: &ChapterTarget,
) -> Result<(), IngestError> {
    let runs = chapter_runs(tokens);
    let labels: Vec<ChapterLabel> = if runs.is_empty() {
        vec![ChapterLabel::FrontMatter]
    } else {
        runs.iter().map(|run| run.label.clone()).collect()
    };

    if let Some(found) = labels.iter().find(|label| *label != &target.label) {
        return Err(IngestError::ReplacementLabelMismatch {
            target: target.clone(),
            found: found.clone(),
        });
    }
    if labels.len() > 1 {
        return Err(IngestError::AmbiguousChapter {
            target: target.clone(),
            matches: labels.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use usfm_onion::format::TokenTemplate;
    use usfm_onion::lint::{LintOptions, LintScope, TokenFix};
    use usfm_onion::token::{BookId, TokenKind};

    use super::*;
    use crate::patch::ResolvedFix;

    /// Core's linter produces only single-replacement `ReplaceToken` fixes, so
    /// the synthesis half of the boundary — the minter, and the sweep behind it
    /// — has no public route to reach in this phase. These tests drive the
    /// internal seam directly rather than leave it unexercised until a later
    /// phase's formatter is the first thing to try it in anger.
    fn seeded(minter: impl FnMut() -> String + 'static) -> Braid {
        let mut resident = Braid::new(
            BraidConfig::new(LintOptions::scoped(LintScope::Book)),
            minter,
        );
        resident
            .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: BookId::from_str("GEN").unwrap(),
                source: "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\n".to_string(),
            }]))
            .expect("one book");
        resident
    }

    /// A fix that inserts a token: the new token is not in the resident stream,
    /// so it is minted an id by the handle's own function and rebuilt against
    /// the fix's target as its anchor.
    fn insert_after(target: &str, text: &str) -> ResolvedFix {
        ResolvedFix::new(
            &TokenFix::InsertAfter {
                code: "test-insert".to_string(),
                label: "TestInsert".to_string(),
                label_params: Default::default(),
                target_token_id: target.to_string(),
                insert: vec![TokenTemplate {
                    kind: TokenKind::Text,
                    text: text.to_string(),
                    marker: None,
                    sid: None,
                }],
            },
            0,
            SourceHash(0),
        )
        .expect("a one-template insertion edits something")
    }

    /// A fix that edits nothing is not representable as a patch: there is no run
    /// of rows to address, nothing to preview or apply, and the wire refuses to
    /// encode the same shape. No rule produces one; this pins the boundary rather
    /// than the producer.
    #[test]
    fn a_fix_that_edits_nothing_resolves_to_no_patch() {
        for fix in [
            TokenFix::ReplaceToken {
                code: "empty-replace".to_string(),
                label: "EmptyReplace".to_string(),
                label_params: Default::default(),
                target_token_id: "GEN-0".to_string(),
                replacements: Vec::new(),
            },
            TokenFix::InsertAfter {
                code: "empty-insert".to_string(),
                label: "EmptyInsert".to_string(),
                label_params: Default::default(),
                target_token_id: "GEN-0".to_string(),
                insert: Vec::new(),
            },
        ] {
            assert!(
                ResolvedFix::new(&fix, 0, SourceHash(0)).is_none(),
                "{fix:?} edits nothing"
            );
        }
        // A delete always edits something: the token it names.
        assert!(
            ResolvedFix::new(
                &TokenFix::DeleteToken {
                    code: "drop".to_string(),
                    label: "Drop".to_string(),
                    label_params: Default::default(),
                    target_token_id: "GEN-0".to_string(),
                },
                0,
                SourceHash(0),
            )
            .is_some()
        );
    }

    /// A minter that records how many ids it granted, so a test can assert on
    /// the *absence* of minting rather than only on its result.
    fn counting_minter() -> (
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
        impl FnMut() -> String,
    ) {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = std::sync::Arc::clone(&calls);
        (calls, move || {
            let next = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            format!("app-{next}")
        })
    }

    /// The comparable content of a working token: everything a fix decides,
    /// excluding the id — which the pass deliberately does not decide.
    fn working_shape(token: &FormatToken) -> (TokenKind, &str, Option<&str>, Option<&str>) {
        (
            token.kind,
            token.text.as_str(),
            token.marker.as_deref(),
            token.sid.as_deref(),
        )
    }

    fn resident_shape(token: &OwnedToken) -> (TokenKind, &str, Option<&str>, Option<&str>) {
        (
            token.kind(),
            token.source(),
            token.marker_name(),
            token.sid(),
        )
    }

    /// A synthesizing fix — the case the corpus cannot produce — proving the
    /// preview/apply split holds where it actually matters: the projection is the
    /// same, and only the apply spends an id.
    ///
    /// Minting is admission to residency, and a preview is never admitted. If a
    /// preview minted, every hover over a fix button would burn ids out of the
    /// application's own space, and two previews of one patch would disagree.
    #[test]
    fn preview_projects_a_synthesizing_fix_without_minting() {
        let (calls, minter) = counting_minter();
        let mut resident = seeded(minter);
        let target = resident.books[0].tokens[0].id().as_str().to_string();
        let resolved = insert_after(&target, "inserted");

        let first = resident.applied_working(0, &resolved);
        let second = resident.applied_working(0, &resolved);
        assert_eq!(first, second, "a preview is deterministic");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "a preview must not spend an id"
        );
        // The synthesized token is present and deliberately id-less.
        assert_eq!(first.len(), resident.books[0].tokens.len() + 1);
        assert_eq!(first[1].text, "inserted");
        assert_eq!(first[1].id, None);

        let admitted = resident
            .admit(0, &resolved, first.clone())
            .expect("admission");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            1,
            "admission is the one place an id is granted"
        );
        // Preview equals apply on content, token for token.
        assert_eq!(
            first.iter().map(working_shape).collect::<Vec<_>>(),
            admitted.iter().map(resident_shape).collect::<Vec<_>>()
        );
        assert_eq!(admitted[1].id().as_str(), "app-1");
        // And previewing after an admission is unchanged: nothing about the
        // resident book moved, so neither does its projection.
        assert_eq!(resident.applied_working(0, &resolved), second);
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn a_synthesized_token_is_minted_by_the_handles_own_function() {
        let mut next = 0u32;
        let mut resident = seeded(move || {
            next += 1;
            format!("app-{next}")
        });
        let target = resident.books[0].tokens[0].id().as_str().to_string();
        let resolved = insert_after(&target, "inserted");

        let working = resident.applied_working(0, &resolved);
        let tokens = resident.admit(0, &resolved, working).expect("applies");
        assert_eq!(tokens.len(), resident.books[0].tokens.len() + 1);
        assert_eq!(tokens[1].source(), "inserted");
        assert_eq!(
            tokens[1].id().as_str(),
            "app-1",
            "the application's function is the only source of a new id"
        );
        // Every other token came back byte-identical, anchored by its own id.
        assert_eq!(tokens[0], resident.books[0].tokens[0]);
        assert_eq!(&tokens[2..], &resident.books[0].tokens[1..]);
    }

    /// The minter contract is deliberately thin — any `() -> String` — so
    /// uniqueness is not assumed. A colliding one is caught by the same
    /// duplicate-id check every ingest path runs, as a typed rejection with the
    /// corpus untouched.
    #[test]
    fn a_colliding_minter_is_rejected_atomically() {
        let mut resident = seeded(|| "GEN-0".to_string());
        let target = resident.books[0].tokens[0].id().as_str().to_string();
        let resolved = insert_after(&target, "inserted");
        let before = resident.expected_snapshot_id();

        let working = resident.applied_working(0, &resolved);
        let tokens = resident.admit(0, &resolved, working).expect("rebuilds");
        let rejected = resident.books[0].rebuilt(tokens);
        assert!(matches!(
            rejected,
            Err(IngestError::DuplicateTokenId { .. })
        ));
        assert_eq!(resident.expected_snapshot_id(), before);
    }
}
