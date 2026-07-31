//! Prepared format patches: a frozen working-token replacement for one or
//! more books, computed by proxying core's own `format` — no new diff or
//! patch-row machinery of its own.
//!
//! A prepared format patch is deliberately not shaped like [`crate::Patch`]'s
//! position-addressed rows. A fix's rows all name one position because they
//! flatten a single small, targeted `TokenFix`; a book- or chapter-wide format
//! pass can rewrite tokens throughout the run. Forcing that into
//! one-position-per-patch rows would mean either violating the fix patch
//! table's own shape or redesigning it — both out of scope here. Instead a
//! prepared format patch simply carries each targeted book's complete
//! post-format working-token stream, computed once against a frozen snapshot.
//! Applying it replaces that book's tokens wholesale, the same way
//! `update_book`/`update_chapter` already do.

use usfm_onion::format::FormatToken;
use usfm_onion::token::BookId;

use crate::error::IngestError;
use crate::input::ChapterLabel;
use crate::state::{SnapshotId, SourceHash};

/// One targeted book's prepared replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedFormatBook {
    pub(crate) book: BookId,
    /// This book's hash at prepare time — re-checked at apply time the same
    /// way a resolved fix's own `source_hash` is, so a book rewritten and then
    /// restored to a different corpus that happens to share the overall
    /// snapshot id still cannot silently apply a stale preparation.
    pub(crate) source_hash: SourceHash,
    /// `Some(label)` when this book was prepared under `CorpusScope::Chapter`
    /// — the only run this preparation could have touched, so the applied
    /// effect can report exactly that chapter instead of widening to the
    /// whole book. `None` for `Book`/`All`, where a format pass may touch
    /// tokens throughout the book.
    pub(crate) chapter: Option<ChapterLabel>,
    pub(crate) tokens: Vec<FormatToken>,
}

/// A prepared, snapshot-bound format patch. May span more than one book
/// (`CorpusScope::All`); applying it commits every targeted book atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedFormatPatch {
    pub(crate) books: Vec<PreparedFormatBook>,
}

/// A prepared format patch's identity.
///
/// `ordinal` addresses braid's own prepared-format table — a separate space
/// from [`crate::PatchId`]'s corpus-wide fix ordinal. The two tables hold
/// differently shaped things (one position-addressed row run per book versus
/// one whole replacement possibly spanning several books) and neither
/// addresses the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatPatchId {
    pub snapshot: SnapshotId,
    pub ordinal: u32,
}

/// What [`crate::Braid::prepare_format_patch`] returns: either the scope was
/// already exactly what `format` would produce (nothing to apply), or a
/// handle to a frozen preparation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatchPreparation {
    Unchanged,
    Ready(FormatPatchId),
}

/// A prepared format patch that could not be looked up or applied.
///
/// Every rejection happens before resident state is touched, and applying a
/// multi-book preparation is all-or-nothing: either every targeted book's
/// candidate builds and the corpus commits all of them, or none commit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatPatchError {
    /// The preparation was computed against a different corpus than the
    /// resident one, or one of its targeted books was rewritten since.
    StaleSnapshot {
        expected: SnapshotId,
        found: SnapshotId,
    },
    /// No such prepared format patch in this snapshot's table.
    UnknownPatch(FormatPatchId),
    /// Applying the preparation produced a token stream that cannot become
    /// resident.
    InvalidResult(IngestError),
}

impl std::fmt::Display for FormatPatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleSnapshot { expected, found } => write!(
                f,
                "format patch was prepared against snapshot {:016x}, resident is {:016x}",
                found.0, expected.0
            ),
            Self::UnknownPatch(id) => {
                write!(f, "no prepared format patch with ordinal {}", id.ordinal)
            }
            Self::InvalidResult(error) => {
                write!(f, "formatted book is not resident-valid: {error}")
            }
        }
    }
}

impl std::error::Error for FormatPatchError {}
