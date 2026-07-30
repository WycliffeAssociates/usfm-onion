//! Typed lifecycle failures.
//!
//! Every rejection names the invariant it violated, and a rejected mutation
//! leaves resident state untouched — the caller can retry or repair without
//! first re-reading everything. `Display` text is for logs; callers match on
//! variants.

use usfm_onion::token::{BookId, StableTokenId, TokenBuildError};

use crate::input::{ChapterLabel, ChapterTarget, SourceKey};
use crate::patch::PatchId;
use crate::state::SnapshotId;

/// Core's [`StableTokenId`] as a plain string.
///
/// Braid serializes the core type it embeds rather than core growing derives for
/// a boundary it does not own — the same division the wire DTOs follow. The
/// non-empty invariant is re-checked on the way in, so a hostile payload cannot
/// deserialize an unaddressable id.
#[cfg(feature = "serde")]
mod stable_token_id {
    use serde::{Deserialize, Deserializer, Serializer};
    use usfm_onion::token::StableTokenId;

    pub(super) fn serialize<S: Serializer>(
        value: &StableTokenId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.as_str())
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<StableTokenId, D::Error> {
        let raw = String::deserialize(deserializer)?;
        StableTokenId::new(raw)
            .ok_or_else(|| serde::de::Error::custom("stable token id must not be empty"))
    }
}

/// A mutation refused before it touched resident state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IngestError {
    /// Two resident inputs declare the same book. There is no unambiguous
    /// scope to install, so the whole candidate corpus is refused.
    DuplicateBook {
        book: BookId,
        sources: Vec<SourceKey>,
    },
    /// Two resident inputs bind the same source key.
    DuplicateSourceKey { source: SourceKey },
    /// Token identity must be unique within one book — reconciliation keys on
    /// it, so a collision would make one of the two tokens unaddressable.
    DuplicateTokenId {
        book: BookId,
        #[cfg_attr(feature = "serde", serde(with = "stable_token_id"))]
        id: StableTokenId,
    },
    /// No run in the book carries this label — including the case where the
    /// book itself is not resident, since a chapter of an absent book is
    /// equally not found.
    ChapterNotFound(ChapterTarget),
    /// Duplicate/reopened runs carry this label, so the operation has no single
    /// target. Insertion, deletion, reorder, and duplicate resolution go
    /// through `update_book`.
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
    /// The replacement is not exactly this chapter: its first differing run
    /// carries `found`.
    ReplacementLabelMismatch {
        target: ChapterTarget,
        found: ChapterLabel,
    },
    /// A token produced by a format or fix pass cannot become resident: the
    /// working shape it came back in is missing something a resident token must
    /// have, and core refuses to invent it. Frozen as `InvalidToken`; the payload
    /// is core's own boundary error, because the producer here is core's
    /// working-token checkpoint rather than a DTO conversion.
    InvalidToken(TokenBuildError),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateBook { book, sources } => {
                write!(f, "book {book} is declared by {} inputs", sources.len())
            }
            Self::DuplicateSourceKey { source } => {
                write!(f, "source key {} is bound twice", source.as_str())
            }
            Self::DuplicateTokenId { book, id } => {
                write!(f, "book {book} carries token id {id} more than once")
            }
            Self::ChapterNotFound(target) => write!(f, "{target} is not resident"),
            Self::AmbiguousChapter { target, matches } => {
                write!(f, "{target} matches {matches} runs")
            }
            Self::ReplacementLabelMismatch { target, found } => {
                write!(f, "replacement for {target} carries the run {found}")
            }
            Self::InvalidToken(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for IngestError {}

/// A patch that could not be looked up or applied.
///
/// Patch application is never partial: every rejection below happens before
/// resident state is touched, and a patch that applies commits its whole book at
/// once.
///
/// Three of the frozen variants have no producer yet and are deliberately not
/// built: `InvalidEditOrder`, `OverlappingEdits`, and `OutOfBounds` describe a
/// patch table braid did not resolve itself. Every table in this phase comes from
/// braid's own resolution against its own token stream, so those three cannot
/// arise; they get producers when a patch table can arrive from outside (a
/// restored lint cache, a decoded container), and inventing them now would be
/// unreachable code claiming a check that never runs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatchError {
    /// The patch was resolved against a different corpus than the resident one —
    /// either the corpus identity moved, or the target book was rewritten since.
    /// Both halves matter: identity alone would accept a patch whose own book
    /// changed and changed back within a corpus that hashes the same.
    StaleSnapshot {
        expected: SnapshotId,
        found: SnapshotId,
    },
    /// The current snapshot's patch table has no such ordinal. A book awaiting
    /// recompute contributes no patches, so this is also what a patch of a
    /// not-yet-relinted book reports.
    UnknownPatch(PatchId),
    /// Applying the patch produced a token stream that cannot become resident.
    InvalidResult(IngestError),
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleSnapshot { expected, found } => write!(
                f,
                "patch was resolved against snapshot {:016x}, resident is {:016x}",
                found.0, expected.0
            ),
            Self::UnknownPatch(id) => {
                write!(f, "no patch with ordinal {} in this snapshot", id.ordinal)
            }
            Self::InvalidResult(error) => write!(f, "patched book is not resident-valid: {error}"),
        }
    }
}

impl std::error::Error for PatchError {}

/// A scope that does not resolve against the resident corpus.
///
/// Read paths (`to_tokens`, `to_usfm`) and non-ingesting mutations
/// (`remove_chapter`) share it: resolving an address is the same job whether
/// or not the caller then writes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScopeError {
    BookNotFound(BookId),
    ChapterNotFound(ChapterTarget),
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BookNotFound(book) => write!(f, "book {book} is not resident"),
            Self::ChapterNotFound(target) => write!(f, "{target} is not resident"),
            Self::AmbiguousChapter { target, matches } => {
                write!(f, "{target} matches {matches} runs")
            }
        }
    }
}

impl std::error::Error for ScopeError {}

impl IngestError {
    /// Chapter ingest resolves its target through the same lookup the read
    /// paths use, so the two error sets overlap on those variants. `target`
    /// carries the caller's own address, which is why an absent book collapses
    /// into `ChapterNotFound` without inventing a label.
    pub(crate) fn from_scope(value: ScopeError, target: &ChapterTarget) -> Self {
        match value {
            ScopeError::BookNotFound(_) | ScopeError::ChapterNotFound(_) => {
                Self::ChapterNotFound(target.clone())
            }
            ScopeError::AmbiguousChapter { target, matches } => {
                Self::AmbiguousChapter { target, matches }
            }
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use usfm_onion::token::{BookId, StableTokenId};

    use super::{IngestError, ScopeError};
    use crate::input::{ChapterLabel, ChapterTarget};

    /// A native (Tauri) host serializes these straight to its IPC channel, so
    /// the derives have to cover every payload braid embeds — including the two
    /// core types it does not own.
    #[test]
    fn lifecycle_errors_round_trip_through_serde() {
        let book = BookId::from_str("GEN").unwrap();
        let errors = vec![
            IngestError::DuplicateTokenId {
                book,
                id: StableTokenId::new("GEN-4").unwrap(),
            },
            IngestError::ReplacementLabelMismatch {
                target: ChapterTarget::new(book, ChapterLabel::Number("1".into())),
                found: ChapterLabel::FrontMatter,
            },
        ];
        for error in errors {
            let json = serde_json::to_string(&error).unwrap();
            assert_eq!(serde_json::from_str::<IngestError>(&json).unwrap(), error);
        }

        let scope = ScopeError::AmbiguousChapter {
            target: ChapterTarget::new(book, ChapterLabel::Number("2".into())),
            matches: 2,
        };
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(serde_json::from_str::<ScopeError>(&json).unwrap(), scope);

        // The non-empty invariant survives the boundary rather than being
        // re-derivable only from a well-behaved sender.
        assert!(
            serde_json::from_str::<IngestError>(r#"{"DuplicateTokenId":{"book":"GEN","id":""}}"#)
                .is_err()
        );
    }
}
