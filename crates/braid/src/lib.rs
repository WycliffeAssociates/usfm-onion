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
mod state;

use usfm_onion::lint::LintOptions;
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken, tokens_to_usfm_reconstruct_with_eol};

use crate::corpus::{BookState, chapter_runs};

pub use crate::error::{IngestError, ScopeError};
pub use crate::input::{
    BookInput, BookTokensInput, ChapterInput, ChapterLabel, ChapterTarget, CorpusInput,
    CorpusScope, LineEnding, ScopedOutput, SourceKey, SourceOutput,
};
pub use crate::state::{
    BookEntry, MutationEffect, Scope, ScopeSet, ScopeTokens, SnapshotId, SourceHash,
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
#[derive(Debug, Clone)]
pub struct Braid {
    config: BraidConfig,
    /// Ordered and unique by both declared book and source key. Caller order is
    /// preserved — nothing here sorts.
    books: Vec<BookState>,
    snapshot_id: SnapshotId,
}

impl Braid {
    pub fn new(config: BraidConfig) -> Self {
        Self {
            config,
            books: Vec::new(),
            snapshot_id: SnapshotId::of([]),
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
                    candidate.lint_dirty = resident.lint_dirty;
                }
                _ => changed.push(Scope::book(candidate.book)),
            }
        }

        self.books = candidates;
        Ok(self.effect(changed, removed))
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
                    candidate.lint_dirty = self.books[index].lint_dirty;
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

        let tokens = match replacement {
            ChapterInput::Usfm { source } => parse(&source)
                .tokens
                .iter()
                .map(OwnedToken::from_parsed)
                .collect(),
            ChapterInput::Tokens(tokens) => tokens,
        };
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
            }
        }
        self.effect(Vec::new(), Vec::new())
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
        self.snapshot_id = SnapshotId::of(self.books.iter().map(|book| book.hash));
        MutationEffect {
            snapshot_id: self.snapshot_id,
            changed,
            removed,
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
