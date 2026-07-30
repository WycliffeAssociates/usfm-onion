//! Content-derived identity, mutation effects, and the pull selector.

use usfm_onion::token::{BookId, OwnedToken};
use xxhash_rust::xxh3::Xxh3;

use crate::input::{ChapterLabel, ChapterTarget, LineEnding, SourceKey};

/// xxhash3-64 over one book's authoritative source bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceHash(pub u64);

impl SourceHash {
    pub(crate) fn of(source: &str) -> Self {
        let mut hasher = Xxh3::new();
        hasher.update(source.as_bytes());
        Self(hasher.digest())
    }
}

/// The resident corpus's content-derived identity: xxhash3-64 over the ordered
/// per-book source hashes.
///
/// Deterministic across processes and restores — the same corpus always yields
/// the same id, and no counter, timestamp, or session state participates. Book
/// *order* is part of it; source keys, config, and catalog stamps are not (they
/// are separate stamps, and cache validity is the tuple).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnapshotId(pub u64);

impl SnapshotId {
    pub(crate) fn of(hashes: impl IntoIterator<Item = SourceHash>) -> Self {
        let mut hasher = Xxh3::new();
        for hash in hashes {
            hasher.update(&hash.0.to_le_bytes());
        }
        Self(hasher.digest())
    }
}

/// What one mutation rewrote. `chapter: None` means the whole book.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scope {
    pub book: BookId,
    pub chapter: Option<ChapterLabel>,
}

impl Scope {
    pub fn book(book: BookId) -> Self {
        Self {
            book,
            chapter: None,
        }
    }

    pub fn chapter(book: BookId, label: ChapterLabel) -> Self {
        Self {
            book,
            chapter: Some(label),
        }
    }
}

/// The value every mutating verb returns, after it has already applied.
///
/// `changed` is exact — what was rewritten, not what was inspected — so an
/// empty `changed` is the no-op signal for hydration: nothing needs re-pulling.
/// Findings are absent by design; lint stays an explicit separate call. Plain
/// data, no handles.
///
/// `snapshot_id` is the corpus identity *after* the mutation. Three changes it
/// records that `changed` deliberately does not: a pure reorder, a
/// source-key rebinding, and (for the reorder case) the new order itself all
/// rewrite no tokens, so none of them appear in `changed`; reordering changes
/// the id (order is part of identity) while a rename does not (it is not
/// semantic content).
///
/// `reordered` is the reorder's own observable: `Some(full new book order)`
/// when the relative order of the books present both before and after the
/// mutation changed, `None` otherwise — including the ordinary no-op case.
/// Without it, a pure `[GEN, EXO] -> [EXO, GEN]` replace_corpus changed
/// `snapshot_id` (order is part of the corpus's content-derived identity)
/// while reporting an empty `changed`, so `is_noop` claimed nothing happened
/// and the new order was unobservable through `to_tokens`. Only
/// `replace_corpus` can produce a reorder today; every other verb always
/// passes `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MutationEffect {
    pub snapshot_id: SnapshotId,
    pub changed: Vec<Scope>,
    pub removed: Vec<BookId>,
    pub reordered: Option<Vec<BookId>>,
}

impl MutationEffect {
    /// Nothing was rewritten, nothing was removed, and the corpus order did
    /// not change.
    pub fn is_noop(&self) -> bool {
        self.changed.is_empty() && self.removed.is_empty() && self.reordered.is_none()
    }
}

/// A pull selector: what [`crate::Braid::to_tokens`] should hydrate.
///
/// Constructed by `Into`, so an effect can be pulled in one expression
/// (`braid.to_tokens(&effect)`) and so a caller accumulating pending scopes
/// across several effects can pass the concatenation directly — normalization
/// (dedupe, whole-book absorbing chapter scopes) happens inside the pull, which
/// is what makes naive accumulation correct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeSet(Vec<Scope>);

impl ScopeSet {
    pub fn as_slice(&self) -> &[Scope] {
        &self.0
    }
}

impl From<Scope> for ScopeSet {
    fn from(value: Scope) -> Self {
        Self(vec![value])
    }
}

impl From<Vec<Scope>> for ScopeSet {
    fn from(value: Vec<Scope>) -> Self {
        Self(value)
    }
}

impl From<&[Scope]> for ScopeSet {
    fn from(value: &[Scope]) -> Self {
        Self(value.to_vec())
    }
}

impl<const N: usize> From<[Scope; N]> for ScopeSet {
    fn from(value: [Scope; N]) -> Self {
        Self(value.to_vec())
    }
}

impl From<&MutationEffect> for ScopeSet {
    /// Only `changed` — a removed book has no tokens to pull, and the caller
    /// already learns about it from `removed`.
    fn from(value: &MutationEffect) -> Self {
        Self(value.changed.clone())
    }
}

impl From<MutationEffect> for ScopeSet {
    fn from(value: MutationEffect) -> Self {
        Self(value.changed)
    }
}

/// One pulled scope's current tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeTokens {
    pub book: BookId,
    pub chapter: Option<ChapterLabel>,
    pub tokens: Vec<OwnedToken>,
}

/// One resident book's identity and derived stamps, in corpus order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BookEntry {
    pub source_key: SourceKey,
    pub book: BookId,
    pub source_hash: SourceHash,
    pub line_ending: LineEnding,
}

/// A chapter scope that could not be resolved, kept alongside the address the
/// caller supplied so the error can name it.
pub(crate) fn target(book: BookId, label: &ChapterLabel) -> ChapterTarget {
    ChapterTarget {
        book,
        label: label.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{SnapshotId, SourceHash};

    /// The persisted fingerprint is xxhash3-64 over the exact source bytes, seed
    /// zero — the same algorithm and seed the wire container binds its sections
    /// with. Braid never depends on wire, so this golden value is the pin: it was
    /// produced by wire's own `source_hash`, and a drift on either side surfaces
    /// here or in wire's golden vectors rather than as a silently unmatchable
    /// warm cache.
    #[test]
    fn source_hash_matches_the_wire_fingerprint_for_the_same_bytes() {
        assert_eq!(
            SourceHash::of("\\id GEN\n\\c 1\n"),
            SourceHash(0xc261_d5e9_d488_ea4d)
        );
    }

    /// Identity is over the ordered per-book hashes, so it is sensitive to order
    /// and to content, and to nothing else.
    #[test]
    fn snapshot_identity_is_order_sensitive_and_deterministic() {
        let a = SourceHash(1);
        let b = SourceHash(2);
        assert_eq!(SnapshotId::of([a, b]), SnapshotId::of([a, b]));
        assert_ne!(SnapshotId::of([a, b]), SnapshotId::of([b, a]));
        assert_ne!(SnapshotId::of([a, b]), SnapshotId::of([a]));
        // An empty corpus still has a well-defined id.
        assert_eq!(SnapshotId::of([]), SnapshotId::of([]));
    }
}
