//! Snapshot-bound patches: one core `TokenFix`, resolved against the resident
//! snapshot and flattened into token operations.
//!
//! A **fix** is core's recipe (a target token id plus replacement templates). A
//! **patch row** is one flattened operation — an op, a position in the book's
//! token stream, and the template that operation places. A **patch** is the
//! contiguous run of rows for one fix. Byte offsets appear nowhere: a patch is
//! token operations, source bytes and spans are derived afterwards.
//!
//! Identity is `(snapshot, ordinal)`, so staleness falls out of comparing the
//! corpus's content-derived id — there is no separate revision counter to keep
//! in sync. Applying a patch is an ordinary mutation: it rewrites the book,
//! which changes that book's hash and the corpus id, which is exactly what makes
//! every other patch resolved against the old snapshot refuse to apply.

use usfm_onion::format::TokenTemplate;
use usfm_onion::lint::{MessageParams, TokenFix};
use usfm_onion::token::BookId;

use crate::state::{SnapshotId, SourceHash};

/// What one row does at its own position.
///
/// Discriminants are the frozen flat-edit ids the wire stores; they are stable
/// identifiers, not an application order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatchOp {
    /// Place the row's template immediately after the row's position. Several
    /// `Insert` rows at one position place in row order.
    Insert = 0,
    /// Replace the token at the row's position with the row's template.
    Replace = 1,
    /// Remove the token at the row's position.
    Delete = 2,
}

/// One token operation. `position` addresses the token stream of the snapshot
/// the owning patch is bound to, never the post-patch stream.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatchRow {
    pub op: PatchOp,
    pub position: u32,
    /// `None` exactly for [`PatchOp::Delete`], which places nothing.
    pub template: Option<TokenTemplate>,
}

/// A patch's snapshot-bound identity.
///
/// `ordinal` addresses the corpus-wide patch table of that snapshot, in corpus
/// order and then per-book canonical finding order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatchId {
    pub snapshot: SnapshotId,
    pub ordinal: u32,
}

/// One resolved fix, addressable and inspectable without applying it.
///
/// `source_hash` is the target book's hash at resolution time — the second half
/// of the staleness check, because a book can be rewritten and restored to a
/// different corpus that happens to hash the same overall.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Patch {
    pub id: PatchId,
    pub book: BookId,
    pub source_hash: SourceHash,
    /// The fix's own remedy code, which is *not* the finding's lint code.
    pub code: String,
    pub label: String,
    pub label_params: MessageParams,
    pub rows: Vec<PatchRow>,
}

/// A fix resolved against one book, before corpus-wide ordinals exist.
///
/// The originating `TokenFix` is kept because core owns fix semantics: braid
/// applies through `apply_token_fix_with_minter` rather than re-implementing
/// what the rows describe. The rows are the derived published form, produced
/// once here so the two can never disagree at application time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedFix {
    pub(crate) source_hash: SourceHash,
    pub(crate) rows: Vec<PatchRow>,
    pub(crate) fix: TokenFix,
}

impl ResolvedFix {
    /// Flattens one fix against the position its target token occupies.
    ///
    /// Every current corpus fix is a single-replacement `ReplaceToken`, so it
    /// flattens to exactly one row; the multi-template forms are expressed as a
    /// replace (or nothing) followed by inserts at the same position, which is
    /// how a run of rows stays addressable by one first/count pair.
    pub(crate) fn new(fix: &TokenFix, position: u32, source_hash: SourceHash) -> Self {
        let mut rows = Vec::new();
        match fix {
            TokenFix::ReplaceToken { replacements, .. } => {
                for (index, template) in replacements.iter().enumerate() {
                    rows.push(PatchRow {
                        op: if index == 0 {
                            PatchOp::Replace
                        } else {
                            PatchOp::Insert
                        },
                        position,
                        template: Some(template.clone()),
                    });
                }
            }
            TokenFix::DeleteToken { .. } => rows.push(PatchRow {
                op: PatchOp::Delete,
                position,
                template: None,
            }),
            TokenFix::InsertAfter { insert, .. } => {
                for template in insert {
                    rows.push(PatchRow {
                        op: PatchOp::Insert,
                        position,
                        template: Some(template.clone()),
                    });
                }
            }
        }
        Self {
            source_hash,
            rows,
            fix: fix.clone(),
        }
    }

    /// The public patch view, once the corpus assigns this fix its ordinal.
    pub(crate) fn published(&self, id: PatchId, book: BookId) -> Patch {
        let (code, label, label_params) = match &self.fix {
            TokenFix::ReplaceToken {
                code,
                label,
                label_params,
                ..
            }
            | TokenFix::DeleteToken {
                code,
                label,
                label_params,
                ..
            }
            | TokenFix::InsertAfter {
                code,
                label,
                label_params,
                ..
            } => (code, label, label_params),
        };
        Patch {
            id,
            book,
            source_hash: self.source_hash,
            code: code.clone(),
            label: label.clone(),
            label_params: label_params.clone(),
            rows: self.rows.clone(),
        }
    }
}
