//! Baseline state: "what was last saved" for one book, kept alongside its
//! current content so a caller can ask whether the two have diverged.
//!
//! A baseline is auxiliary per-book state, not a second corpus: it carries no
//! chapter/config/lint machinery of its own, only the exact bytes and tokens
//! needed to answer an equality question and to feed core's own diff. Setting
//! or clearing a baseline never touches current tokens, hashes, dirty stamps,
//! or the corpus snapshot id — only the book's own baseline slot changes.

use usfm_onion::token::{LineEnding, OwnedToken};

use crate::corpus::{BookState, ChapterRun};
use crate::error::ScopeError;
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
    MissingBaseline {
        books: Vec<usfm_onion::token::BookId>,
    },
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
