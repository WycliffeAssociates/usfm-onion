//! Resident lint: the complete semantic snapshot, recomputed one whole book at
//! a time and only for books that changed.
//!
//! There is exactly one recompute verb ([`crate::Braid::lint`]) and it is
//! explicit — no mutation lints implicitly, and findings are never part of a
//! [`crate::MutationEffect`]. What makes the cache safe is that dirtiness is
//! *derived* state (a stamp on each resident book), not a queue: a book is
//! recomputed when its content or the lint configuration changed, and reading
//! the snapshot twice runs no rules the second time.

use usfm_onion::lint::{LintResult, LintSummary};
use usfm_onion::token::{BookId, OwnedToken};

use crate::input::SourceKey;
use crate::state::{SnapshotId, SourceHash};

/// The complete resident lint snapshot, in corpus order.
///
/// Borrowed rather than owned: the tokens and results it names are the resident
/// ones, so publishing a snapshot copies no token streams. `id` is the corpus's
/// content-derived identity at the moment the snapshot was taken, which is what
/// binds a published finding (or a patch resolved against it) to the exact
/// corpus that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintSnapshot<'a> {
    pub id: SnapshotId,
    pub books: Vec<BookLintSnapshot<'a>>,
    pub summary: LintSummary,
}

/// One book's contribution to a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookLintSnapshot<'a> {
    pub source_key: &'a SourceKey,
    pub book: BookId,
    pub source_hash: SourceHash,
    pub tokens: &'a [OwnedToken],
    pub result: &'a LintResult,
}

/// Adds one book's counts into a corpus-wide summary.
///
/// The corpus summary is the sum of the per-book summaries core already
/// produced, never a second count over the findings: a re-count could disagree
/// with the per-book numbers it claims to total (suppressed findings, for one,
/// are counted but not carried).
pub(crate) fn accumulate(total: &mut LintSummary, book: &LintSummary) {
    for (category, count) in &book.by_category {
        *total.by_category.entry(*category).or_default() += count;
    }
    for (severity, count) in &book.by_severity {
        *total.by_severity.entry(*severity).or_default() += count;
    }
    for (issue_type, count) in &book.by_issue_type {
        *total.by_issue_type.entry(*issue_type).or_default() += count;
    }
    total.total_count += book.total_count;
    total.suppressed_count += book.suppressed_count;
}
