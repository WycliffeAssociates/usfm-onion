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

/// xxhash3-64 over everything one book's token stream carries that its source
/// bytes do not pin.
///
/// A book's bytes are the concatenation of its tokens' own text, so two different
/// token streams can serialize identically while differing in every fact a
/// serializer never writes: the stable ids, the canonical anchors, marker
/// nesting, book-code validity, parsed number payloads, attribute structure. An
/// editor re-pushing byte-identical content under fresh ids is the ordinary case
/// — braid already treats it as a real change (`content_eq` compares tokens, not
/// just hashes), and this is the same fact as a stamp, so a consumer holding
/// something derived from the *tokens* (a packed section, a cached anything) can
/// tell staleness without re-reading the whole stream.
///
/// Deliberately not part of [`SnapshotId`]: corpus identity covers source bytes
/// only, on purpose, so that publishing or restoring the same bytes lands the
/// same id. This is a per-book fact a consumer keys its own caches on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenIdentity(pub u64);

impl TokenIdentity {
    /// Hashes every field the owned-token representation holds — including the
    /// source text, so the stamp is self-sufficient rather than only meaningful
    /// beside a source hash.
    ///
    /// Field-by-field rather than through a derived `Hash`: core's `OwnedToken`
    /// keeps its payload private and derives no `Hash`, and spelling the fields
    /// out here is also what makes an addition to that payload a visible decision
    /// on this side rather than a silent change of meaning.
    pub(crate) fn of(tokens: &[OwnedToken]) -> Self {
        let mut hasher = Xxh3::new();
        for token in tokens {
            hasher.update(token.id().as_str().as_bytes());
            hasher.update(&[0]);
            hasher.update(&[token.kind() as u8]);
            hasher.update(token.source().as_bytes());
            hasher.update(&[0]);
            if let Some(sid) = token.sid() {
                hasher.update(sid.as_bytes());
            }
            hasher.update(&[0]);
            if let Some(sid) = token.parsed_sid() {
                hasher.update(sid.book.as_str().as_bytes());
                hasher.update(&sid.chapter.to_le_bytes());
                hasher.update(&sid.verse.to_le_bytes());
                hasher.update(&sid.verse_end().to_le_bytes());
            }
            hasher.update(&[0]);
            if let Some(marker) = token.marker_name() {
                hasher.update(marker.as_bytes());
            }
            hasher.update(&[0, u8::from(token.nested())]);
            if let Some(code) = token.book_code() {
                hasher.update(code.code.as_bytes());
                hasher.update(&[u8::from(code.is_valid)]);
            }
            hasher.update(&[0]);
            if let Some(number) = token.number_info() {
                hasher.update(&number.start.to_le_bytes());
                hasher.update(&number.end.unwrap_or(u32::MAX).to_le_bytes());
                hasher.update(&[number.kind as u8]);
            }
            hasher.update(&[0]);
            if let Some(list) = token.attribute_list() {
                hasher.update(list.as_bytes());
            }
            hasher.update(&[0]);
            if let Some(offset) = token.attribute_offset() {
                hasher.update(&offset.to_le_bytes());
            }
            hasher.update(&[0]);
            for attribute in token.attributes() {
                hasher.update(attribute.key.as_bytes());
                hasher.update(&[0]);
                hasher.update(attribute.value.as_bytes());
                hasher.update(&[0, u8::from(attribute.is_default)]);
            }
            hasher.update(&[0]);
        }
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
    /// Everything this book's tokens carry that its bytes do not pin — the fact a
    /// consumer caching anything token-derived has to key on alongside the hash.
    pub token_identity: TokenIdentity,
    pub line_ending: LineEnding,
}

/// What one [`crate::Braid::restore_corpus`] call installed, and what it refused.
///
/// Residency and lint-priming are two independent facts, and a book can carry
/// both: `SourceTokenMismatch` refuses the book's residency entirely (its
/// tokens do not spell its own bytes, so there is nothing safe to seed — the
/// caller re-ingests it from scratch). Every other reason instead gates the
/// book's *cached lint contribution* only — the book still seeds (`seeded`
/// includes it) with no lex/parse, just without a warm lint result, and stays
/// dirty until the next ordinary `lint()`. A book therefore appears in
/// `rejected` alone (residency refused) or in both `seeded` and `rejected`
/// (residency accepted, cached lint refused), never in `rejected` for a
/// residency reason while also appearing in `seeded`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RestoreReport {
    pub seeded: Vec<BookId>,
    pub rejected: Vec<PrimeRejection>,
}

/// One book a warm seed or cache prime would not accept.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrimeRejection {
    pub book: BookId,
    pub reason: PrimeRejectReason,
}

/// Why a book — or its cached lint contribution — was refused.
///
/// `SourceTokenMismatch` is a residency refusal (see [`RestoreReport`]); every
/// other variant gates a cached lint contribution only and leaves the book's
/// residency untouched, whether it arrived through `restore_corpus` or
/// `prime_lint_cache`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimeRejectReason {
    /// The supplied tokens do not spell the supplied bytes.
    SourceTokenMismatch,
    /// [`crate::Braid::prime_lint_cache`] named a book that is not currently
    /// resident. (Unreachable from `restore_corpus`, whose cached-lint entries
    /// are always embedded alongside the very book they prime.)
    BookNotResident,
    /// The cached contribution's own recorded source hash does not match this
    /// book's actual hash — it was computed against different bytes.
    SourceHashMismatch,
    /// The batch's `config_fingerprint` does not match the lint configuration
    /// this corpus actually runs with.
    ConfigFingerprintMismatch,
    /// The batch's `engine_stamp` does not match the rule engine this build
    /// actually runs.
    EngineStampMismatch,
    /// The source hash and both stamps agreed, but at least one finding's fix
    /// could not be resolved against this book's own token stream — a
    /// structural inconsistency a matching hash does not rule out, so the
    /// whole cached contribution is refused rather than adopting the findings
    /// with an incomplete patch table.
    InvalidPatch,
}

/// One book's cached lint contribution, presented for validation.
///
/// `book`/`source_hash` are carried on this type rather than inferred from
/// where it is embedded, because [`crate::Braid::prime_lint_cache`] addresses
/// resident books by this pair directly (its `books` are a flat list, not
/// nested inside a per-book restore record).
///
/// `Serialize` only, deliberately: `usfm_onion::lint::LintResult` embeds
/// `LintIssue::template: &'static str`, which has no honest `Deserialize` —
/// there is no way to manufacture a `'static` reference from arbitrary input
/// bytes without a lookup back through `LintCode::template`, and core itself
/// has never derived one for exactly this reason (`LintResult`/`LintIssue`
/// are `Serialize`-only today). The intended producer of a `BookLintPrime` is
/// always the composing adapter decoding wire bytes natively in the same Rust
/// process as the `Braid` it seeds — never a value crossing a real serde
/// boundary — so this asymmetry costs nothing today; it would need a manual
/// `Deserialize` impl (reconstructing `template` from `code`) the day
/// something other than that in-process caller needs one.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BookLintPrime {
    pub book: BookId,
    pub source_hash: SourceHash,
    pub result: usfm_onion::lint::LintResult,
}

/// [`crate::Braid::prime_lint_cache`]'s input: one batch of cached
/// contributions, all produced under the same lint configuration and rule
/// engine.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintPrimeInput {
    pub config_fingerprint: crate::stamps::LintConfigFingerprint,
    pub engine_stamp: crate::stamps::LintEngineStamp,
    pub books: Vec<BookLintPrime>,
}

/// What one [`crate::Braid::prime_lint_cache`] call accepted and refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrimeReport {
    pub accepted: Vec<BookId>,
    pub rejected: Vec<PrimeRejection>,
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
