//! Baseline state: "what was last saved" for one book, kept alongside its
//! current content so a caller can ask whether the two have diverged.
//!
//! A baseline is auxiliary per-book state, not a second corpus: it carries no
//! chapter/config/lint machinery of its own, only the exact bytes and tokens
//! needed to answer an equality question and to feed core's own diff. Setting
//! or clearing a baseline never touches current tokens, hashes, dirty stamps,
//! or the corpus snapshot id — only the book's own baseline slot changes.

use usfm_onion::token::{BookId, LineEnding, OwnedToken};

use crate::Braid;
use crate::corpus::{BookState, ChapterRun};
use crate::error::{IngestError, ScopeError};
use crate::input::CorpusScope;
use crate::state::{MutationEffect, Scope, SourceHash};

/// One book's declared baseline: the same facts a resident book's current
/// content carries, frozen at the moment `Braid::set_baseline` was called.
#[derive(Debug, Clone)]
pub(crate) struct BaselineState {
    pub(crate) source: String,
    pub(crate) hash: SourceHash,
    pub(crate) tokens: Vec<OwnedToken>,
    pub(crate) line_ending: LineEnding,
    pub(crate) runs: Vec<ChapterRun>,
}

impl BaselineState {
    /// Snapshots a validated candidate's content as a baseline.
    pub(crate) fn of(book: &BookState) -> Self {
        Self {
            source: book.source.clone(),
            hash: book.hash,
            tokens: book.tokens.clone(),
            line_ending: book.line_ending,
            runs: book.runs.clone(),
        }
    }
}

/// A resident scope that failed to resolve against a baseline, or that has no
/// baseline to resolve against at all.
///
/// `Scope` composes the ordinary read-path [`ScopeError`] (an absent book, an
/// absent/ambiguous chapter run) the same way current-content reads already
/// report it. `MissingBaseline` is the distinct baseline-specific failure: the
/// scope resolves fine against current content, but at least one requested
/// book has declared no baseline at all, so there is nothing to diff or
/// compare against — never synthesized as "equal" or "everything changed".
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BaselineError {
    Scope(ScopeError),
    MissingBaseline {
        books: Vec<BookId>,
    },
    /// [`Braid::revert_to_baseline`] supports only `Book`/`All` scopes — a
    /// single chapter run has no baseline slot of its own to revert against
    /// (baselines are whole-book), and reverting one run in isolation would
    /// have to reconstruct the surrounding book from a mix of current and
    /// baselined tokens, which is not what "revert" means. Deferred, not
    /// planned as a follow-up in this phase: the sanctioned workaround is
    /// `diff_baseline` (see what changed) followed by `update_chapter` with
    /// the baseline run's own tokens (revert just that run by hand).
    ChapterScopeUnsupported(crate::input::ChapterTarget),
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope(error) => write!(f, "{error}"),
            Self::MissingBaseline { books } => {
                write!(f, "no baseline declared for {} book(s)", books.len())
            }
            Self::ChapterScopeUnsupported(target) => write!(
                f,
                "revert_to_baseline does not support chapter scope ({target}); \
                 use diff_baseline plus update_chapter instead"
            ),
        }
    }
}

impl std::error::Error for BaselineError {}

impl From<ScopeError> for BaselineError {
    fn from(value: ScopeError) -> Self {
        Self::Scope(value)
    }
}

impl Braid {
    /// Whole-book replacement from each targeted book's own declared
    /// baseline, atomic across the scope.
    ///
    /// `Book`/`All` scopes only — see [`BaselineError::ChapterScopeUnsupported`]
    /// for why a chapter scope refuses instead of reverting one run in
    /// isolation.
    ///
    /// Atomicity: the whole scope is validated before anything mutates —
    /// every targeted book must be resident AND carry a baseline. Any missing
    /// baseline refuses with every offending book named at once
    /// (`BaselineError::MissingBaseline { books }`, listing all of them, not
    /// just the first), and resident state, its stamps, and the snapshot id
    /// are left byte-identical to before the call.
    ///
    /// A book whose current content already equals its baseline is a no-op:
    /// it is left untouched and does not appear in
    /// [`MutationEffect::changed`], not because reverting it would fail but
    /// because there is nothing to rewrite. An actually-reverted book is
    /// marked for lint recompute, the same as any other mutation; publication
    /// cache invalidation and snapshot id recomputation both run through the
    /// same internal effect-recording path every mutating verb shares. The
    /// baseline slot itself is never touched by this call: afterwards
    /// `is_dirty(scope)` is `false` and `diff_baseline` reports equality for
    /// every reverted book.
    pub fn revert_to_baseline(
        &mut self,
        scope: CorpusScope,
    ) -> Result<MutationEffect, BaselineError> {
        let indices: Vec<usize> = match scope {
            CorpusScope::Chapter(target) => {
                return Err(BaselineError::ChapterScopeUnsupported(target));
            }
            CorpusScope::Book(book) => {
                vec![self.index_of(book).ok_or(ScopeError::BookNotFound(book))?]
            }
            CorpusScope::All => (0..self.books.len()).collect(),
        };

        // Validate the whole scope before touching anything: every targeted
        // book must be resident (already established above) AND baselined.
        let missing: Vec<BookId> = indices
            .iter()
            .filter(|&&index| self.books[index].baseline.is_none())
            .map(|&index| self.books[index].book)
            .collect();
        if !missing.is_empty() {
            return Err(BaselineError::MissingBaseline { books: missing });
        }

        let mut changed = Vec::new();
        for index in indices {
            let resident = &self.books[index];
            let baseline = resident.baseline.as_ref().expect("checked missing above");
            // Full content identity, the same four facts `BookState::content_eq`
            // compares — token equality included, because two streams can
            // serialize to identical bytes while carrying different token
            // identities, and reverting exists precisely to reinstate the
            // baseline's ids. Hash-and-source alone would wrongly no-op that
            // case and leave the replacement ids resident.
            if resident.hash == baseline.hash
                && resident.source == baseline.source
                && resident.line_ending == baseline.line_ending
                && resident.tokens == baseline.tokens
            {
                // Already equal to its baseline: a no-op for this book.
                continue;
            }
            let reverted = resident.reverted_to_baseline(baseline);
            changed.push(Scope::book(reverted.book));
            self.books[index] = reverted;
        }

        Ok(self.effect(changed, Vec::new()))
    }

    /// Declares each in-scope book's CURRENT resident state as its baseline —
    /// the bulk counterpart to [`Self::set_baseline`], with no `BookInput`, no
    /// parse, and no validation, because the content is already resident and
    /// already validated: this is a pure snapshot of what braid already
    /// holds, taken directly from each targeted book's own resident state.
    ///
    /// Motivation: an editor's warm-restore path (reopen a saved corpus, then
    /// declare every book's just-restored content as its baseline) had no
    /// route to this fact except round-tripping every book back out through
    /// [`Self::set_baseline`]'s `BookInput::Usfm` arm — which re-parses the
    /// whole corpus purely to restate content braid already has resident,
    /// measured at 1.3-1.6s across a 66-book corpus. This verb is that same
    /// end state reached without a single re-parse.
    ///
    /// `Book`/`All` scopes only, deliberately symmetric with
    /// [`Self::revert_to_baseline`] rather than accepting the bare
    /// [`ScopeError`] an editor RFC originally proposed for this verb: a
    /// baseline is a whole-book slot (see this module's own doc comment), so
    /// the set and revert halves of that slot's lifecycle must agree on what
    /// scopes address it at all — a caller that could set a chapter-scoped
    /// baseline here but never revert to it would have built state with no
    /// way back through the pair's own verbs. A chapter scope therefore
    /// refuses via the same [`BaselineError::ChapterScopeUnsupported`]
    /// `revert_to_baseline` uses, not a bare `ScopeError`.
    ///
    /// Idempotent, and there is no `MissingBaseline` case: this verb's whole
    /// point is to create baselines, not to require them already exist. The
    /// returned effect is always the no-op shape — a baseline slot never
    /// participates in `changed`/`removed`/`reordered`, the same as
    /// [`Self::set_baseline`]/[`Self::clear_baseline`]. Afterwards
    /// `is_dirty(scope)` is `false` and `diff_baseline(scope)` reports
    /// equality for every book in scope; `All` on an empty corpus is a
    /// trivially successful no-op.
    pub fn set_baseline_to_current(
        &mut self,
        scope: CorpusScope,
    ) -> Result<MutationEffect, BaselineError> {
        let indices: Vec<usize> = match scope {
            CorpusScope::Chapter(target) => {
                return Err(BaselineError::ChapterScopeUnsupported(target));
            }
            CorpusScope::Book(book) => {
                vec![self.index_of(book).ok_or(ScopeError::BookNotFound(book))?]
            }
            CorpusScope::All => (0..self.books.len()).collect(),
        };

        for index in indices {
            self.books[index].baseline = Some(BaselineState::of(&self.books[index]));
        }
        Ok(self.effect(Vec::new(), Vec::new()))
    }
}

/// A baseline declaration that could not be installed.
///
/// Kept separate from [`crate::IngestError`] rather than adding a variant to
/// it: `IngestError` describes a rejected *candidate mutation* — every one of
/// its variants is about the content just supplied. "This book is not
/// currently resident" is not a fact about the candidate at all; it is a
/// precondition of `set_baseline` itself, which never introduces a book into
/// the corpus and never touches current content — only an existing book's
/// baseline slot. Folding it into `IngestError` would give every other
/// ingest verb a variant that names a failure mode they cannot produce.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SetBaselineError {
    /// Braid holds no current content for this book. A baseline is auxiliary
    /// state on an existing resident book, never a way to introduce one —
    /// callers must be able to rely on `set_baseline` leaving current
    /// content, hashes, and the corpus snapshot id untouched unconditionally,
    /// including when the named book is a typo or has since been removed.
    BookNotResident(BookId),
    /// The candidate itself failed the same validation any current-data
    /// ingest goes through (malformed input, a duplicate token id within it).
    Invalid(IngestError),
}

impl std::fmt::Display for SetBaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BookNotResident(book) => {
                write!(
                    f,
                    "book {book} has no current content to attach a baseline to"
                )
            }
            Self::Invalid(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SetBaselineError {}
