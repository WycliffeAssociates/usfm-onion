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

mod corpus;
mod error;
mod input;
mod lint;
mod patch;
mod state;

use usfm_onion::format::FormatToken;
use usfm_onion::lint::{LintOptions, LintSummary, apply_token_fix};
use usfm_onion::token::{BookId, OwnedToken, tokens_to_usfm_reconstruct_with_eol};

use crate::corpus::{BookState, chapter_runs};
use crate::lint::accumulate;
use crate::patch::ResolvedFix;

pub use crate::error::{IngestError, PatchError, ScopeError};
pub use crate::input::{
    BookInput, BookRestoreInput, BookTokensInput, ChapterInput, ChapterLabel, ChapterTarget,
    CorpusInput, CorpusRestoreInput, CorpusScope, LineEnding, ScopedOutput, SourceKey,
    SourceOutput,
};
pub use crate::lint::{BookLintSnapshot, LintSnapshot};
pub use crate::patch::{Patch, PatchId, PatchOp, PatchRow};
pub use crate::state::{
    BookEntry, MutationEffect, PrimeRejectReason, PrimeRejection, RestoreReport, Scope, ScopeSet,
    ScopeTokens, SnapshotId, SourceHash,
};

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
            minter: Box::new(minter),
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
                _ => changed.push(Scope::book(candidate.book)),
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
    /// Corpus-level validation is the same as any other seed: duplicate declared
    /// books or source keys refuse the entire call with resident state untouched.
    /// Per-book refusals are data, and the caller re-ingests just those books.
    ///
    /// Seeded books are dirty: this restores the parse, not the findings. Braid
    /// never decodes wire bytes — the composing adapter does that and hands the
    /// results here.
    pub fn restore_corpus(
        &mut self,
        seed: CorpusRestoreInput,
    ) -> Result<RestoreReport, IngestError> {
        let mut candidates = Vec::with_capacity(seed.books.len());
        let mut rejected = Vec::new();
        for book in seed.books {
            let expected = book.source;
            let candidate = BookState::build(BookInput::Tokens(BookTokensInput {
                source_key: book.source_key,
                book: book.book,
                tokens: book.tokens,
                line_ending: book.line_ending,
            }))?;
            if candidate.source == expected {
                candidates.push(candidate);
            } else {
                rejected.push(PrimeRejection {
                    book: book.book,
                    reason: PrimeRejectReason::SourceTokenMismatch,
                });
            }
        }
        validate_unique(&candidates)?;

        let seeded = candidates.iter().map(|book| book.book).collect();
        self.books = candidates;
        self.snapshot_id = SnapshotId::of(self.books.iter().map(|book| book.hash));
        Ok(RestoreReport { seeded, rejected })
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

    /// Books awaiting recompute, in corpus order. Derived from authoritative
    /// stamps rather than drained from a queue, so reading it twice is safe.
    pub fn dirty_books(&self) -> Vec<BookId> {
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
        self.snapshot_id = SnapshotId::of(self.books.iter().map(|book| book.hash));
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
    for (index, candidate) in candidates.iter().enumerate() {
        let duplicates: Vec<SourceKey> = candidates
            .iter()
            .filter(|other| other.book == candidate.book)
            .map(|other| other.source_key.clone())
            .collect();
        if duplicates.len() > 1 {
            return Err(IngestError::DuplicateBook {
                book: candidate.book,
                sources: duplicates,
            });
        }
        if candidates[..index]
            .iter()
            .any(|other| other.source_key == candidate.source_key)
        {
            return Err(IngestError::DuplicateSourceKey {
                source: candidate.source_key.clone(),
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
