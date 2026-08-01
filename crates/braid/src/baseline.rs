//! Baseline state: "what was last saved" for one book, kept alongside its
//! current content so a caller can ask whether the two have diverged.
//!
//! A baseline is auxiliary per-book state, not a second corpus: it carries no
//! chapter/config/lint machinery of its own, only the exact bytes and tokens
//! needed to answer an equality question and to feed core's own diff. Setting
//! or clearing a baseline never touches current tokens, hashes, dirty stamps,
//! or the corpus snapshot id — only the book's own baseline slot changes.

use usfm_onion::token::{BookId, LineEnding, OwnedToken};

use crate::corpus::{BookState, ChapterRun};
use crate::error::{IngestError, ScopeError};
use crate::state::SourceHash;

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
    MissingBaseline { books: Vec<BookId> },
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope(error) => write!(f, "{error}"),
            Self::MissingBaseline { books } => {
                write!(f, "no baseline declared for {} book(s)", books.len())
            }
        }
    }
}

impl std::error::Error for BaselineError {}

impl From<ScopeError> for BaselineError {
    fn from(value: ScopeError) -> Self {
        Self::Scope(value)
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
