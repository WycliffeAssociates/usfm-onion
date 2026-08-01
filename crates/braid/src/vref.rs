//! Resident vref-index caching: recompute only the chapter runs whose own
//! tokens changed; serve everything else from cache.
//!
//! [`usfm_onion::vref::tokens_to_vref_index`] (via its generic
//! `IndexedVrefVisitor` walk) reads, per token: its stable id (a segment's
//! anchor back to the token), kind, source text (a verse's projected text
//! and a verse-number token's own lexeme, which is what lets a bridge like
//! `"1,3"` survive into the sid), sid (the verse/chapter reference the
//! projection keys entries by), and marker name (paragraph-support scope
//! gating, read from `ScopeFrame::marker`). That is exactly what
//! [`usfm_onion::token::OwnedToken::hash_wire_identity`] already hashes with
//! no `..` — the same drift-proof projection this crate's own
//! [`TokenIdentity`] already uses for its own "did anything change" question
//! elsewhere. So a run's `TokenIdentity` over its own token slice is a sound
//! cache key for this projection too: anything the projection could possibly
//! read about a run's tokens is folded into that hash, and a run whose
//! tokens hash identically to a prior read is, by construction, a run the
//! projection would read identically again.
//!
//! This is deliberately not "invalidate the cache when mutation X happens" —
//! this epic has twice found that kind of predicate incomplete for something
//! the projection actually reads. Instead every read re-derives the current
//! identity of every run it visits and only ever reuses a cache entry whose
//! stored identity matches that fresh one; nothing is ever trusted without
//! being re-verified against the resident tokens it would be served in place
//! of. A stale entry left over from a run that no longer exists in this
//! shape is simply never matched again — never applied, never mixed into a
//! result — so there is no separate invalidation bookkeeping to keep in sync
//! with every mutation path in the first place. A book-code (`\id`) change
//! reaching this book's own sid keys is exactly one more source ordinary
//! source tokens (part of `hash_wire_identity`) or the token's own `sid`
//! field can change, so it is already covered without a separate case: any
//! ingest path that changes those tokens changes their hash, and a run's
//! own token slice is what is hashed, never a separate whole-book flag. A
//! reopened/duplicate `\c` shifting a later run's boundary is likewise
//! covered without a separate case — chapter runs are always recomputed
//! fresh from the current token stream (`crate::corpus::chapter_runs`), and
//! this module hashes whatever slice a run's *current* range names, never a
//! remembered offset.
//!
//! One run's own tokens are *not* the whole of what the projection reads,
//! though: a whole-book walk carries one more piece of state across a `\c`
//! boundary and never clears it — whether the most recently opened
//! paragraph-like block supports verse content (`usfm_onion::vref`'s
//! `IndexedVrefVisitor::current_block_supports_verse`, the same fact the
//! pre-existing chapter-parallel `vref_map_partitioned` path already has to
//! reconcile across segments). A book whose chapter 1 ends in a heading with
//! no verse-supporting paragraph, followed by a chapter 2 that opens
//! straight into a `\v` with no block of its own, projects **no** entry for
//! that verse in a whole-book walk — the incoming flag is `false` — but a
//! naive per-run cache walking chapter 2's own tokens in isolation would
//! seed the visitor at its own default (`None`, read as supporting verses)
//! and wrongly produce one. So a run's cache key is `(TokenIdentity,
//! incoming block state)`, not identity alone, and every read threads each
//! run's own *outgoing* state into the next run's *incoming* one — exactly
//! what `tokens_to_vref_index_seeded` exists to make possible without a
//! second walk of the whole book. Runs are always visited in corpus order
//! specifically so this state is genuinely known before a run is computed,
//! never guessed and reconciled after the fact the way the parallel path
//! has to.

use usfm_onion::token::OwnedToken;
use usfm_onion::vref::{VrefEntry, tokens_to_vref_index_seeded};

use crate::state::TokenIdentity;

/// One chapter run's cached vref entries, keyed by that run's own
/// `TokenIdentity` *and* the block-support state carried in from whatever
/// came before it — the same run's tokens can honestly project two
/// different results under two different incoming states, so either half
/// alone would be an unsound key.
#[derive(Debug, Clone)]
pub(crate) struct CachedRun {
    pub(crate) identity: TokenIdentity,
    pub(crate) incoming_block_state: Option<bool>,
    pub(crate) outgoing_block_state: Option<bool>,
    pub(crate) entries: Vec<VrefEntry>,
}

impl CachedRun {
    /// Computes fresh entries for `tokens`, seeded with the incoming
    /// block-support state — the projection itself, nothing braid adds on
    /// top beyond carrying that one fact in and back out.
    fn fresh(tokens: &[OwnedToken], incoming_block_state: Option<bool>) -> Self {
        #[cfg(test)]
        recompute_count::record();
        let (index, outgoing_block_state) =
            tokens_to_vref_index_seeded(tokens, incoming_block_state);
        Self {
            identity: TokenIdentity::of(tokens),
            incoming_block_state,
            outgoing_block_state,
            entries: index.entries().to_vec(),
        }
    }

    pub(crate) fn matches(
        &self,
        identity: TokenIdentity,
        incoming_block_state: Option<bool>,
    ) -> bool {
        self.identity == identity && self.incoming_block_state == incoming_block_state
    }
}

/// A thread-local count of real (non-cache-hit) projection runs, used only by
/// this crate's own tests to prove *which* runs recomputed without resorting
/// to a timing assertion. Thread-local rather than a shared global: `cargo
/// test` runs different tests concurrently on different threads, and each
/// test's own count must not see another test's recomputes.
#[cfg(test)]
pub(crate) mod recompute_count {
    use std::cell::Cell;

    thread_local! {
        static COUNT: Cell<usize> = const { Cell::new(0) };
    }

    pub(crate) fn record() {
        COUNT.with(|count| count.set(count.get() + 1));
    }

    pub(crate) fn reset() {
        COUNT.with(|count| count.set(0));
    }

    pub(crate) fn get() -> usize {
        COUNT.with(|count| count.get())
    }
}

#[cfg(test)]
mod tests {
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion::token::{BookId, OwnedToken};

    use crate::{
        BookInput, BraidConfig, ChapterInput, ChapterLabel, ChapterTarget, CorpusInput,
        CorpusScope, ScopedOutput, SourceKey,
    };

    use super::recompute_count;

    fn id(value: &str) -> BookId {
        BookId::from_str(value).expect("three-character code")
    }

    fn owned(source: &str) -> Vec<OwnedToken> {
        usfm_onion::parse::parse(source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect()
    }

    /// A chapter-replacement input whose tokens carry real sids: parses the
    /// *whole* edited book (so `\id` is in scope for sid derivation) and
    /// slices out just the target chapter's own run. A bare chapter
    /// fragment parsed on its own has no `\id` and so derives no sid at all
    /// (a documented, pre-existing limitation of parsing outside a book's
    /// context) — using one directly as `ChapterInput::Tokens` would silently
    /// produce sid-less replacement tokens and make every verse in the
    /// replaced run invisible to sid-keyed projections such as this one.
    fn chapter_replacement(whole_edited_source: &str, label: &str) -> Vec<OwnedToken> {
        let tokens = owned(whole_edited_source);
        let runs = crate::corpus::chapter_runs(&tokens);
        let run = runs
            .iter()
            .find(|run| matches!(&run.label, ChapterLabel::Number(n) if &**n == label))
            .unwrap_or_else(|| panic!("no chapter {label} in the edited source"));
        tokens[run.range.clone()].to_vec()
    }

    /// The proof this whole cache exists for: a one-chapter edit followed by
    /// a whole-book read recomputes exactly the touched run, not the whole
    /// book, not zero runs. Proven with a recompute counter (see
    /// `recompute_count`), never a timing assertion.
    #[test]
    fn a_whole_book_read_recomputes_only_the_dirty_chapter() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 2\n\\p\n\\v 1 b\n\\c 3\n\\p\n\\v 1 c\n";
        let mut resident = crate::Braid::new(
            BraidConfig::new(LintOptions::scoped(LintScope::Book)),
            || unreachable!("this fixture never synthesizes a token"),
        );
        resident
            .replace_corpus(CorpusInput::new(vec![BookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: id("GEN"),
                source: source.to_string(),
            }]))
            .unwrap();

        // First whole-book read: nothing cached yet, every run (front matter
        // plus chapters 1-3) must compute fresh.
        recompute_count::reset();
        let first = match resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap() {
            ScopedOutput::Single(entries) => entries,
            ScopedOutput::All(_) => panic!("expected a single-scope index"),
        };
        assert_eq!(
            first
                .iter()
                .map(|entry| entry.sid.as_str())
                .collect::<Vec<_>>(),
            ["GEN 1:1", "GEN 2:1", "GEN 3:1"]
        );
        assert!(
            recompute_count::get() >= 3,
            "the first read has nothing to reuse, so every run computes fresh"
        );

        // A whole-book re-read with nothing changed must recompute nothing.
        recompute_count::reset();
        let unchanged = match resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap() {
            ScopedOutput::Single(entries) => entries,
            ScopedOutput::All(_) => panic!("expected a single-scope index"),
        };
        assert_eq!(unchanged, first);
        assert_eq!(
            recompute_count::get(),
            0,
            "a clean re-read must be entirely cache hits"
        );

        // Edit exactly chapter 2, with tokens sidded against the whole edited
        // book so the projection can still key its entry.
        let edited_source =
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 2\n\\p\n\\v 1 b edited\n\\c 3\n\\p\n\\v 1 c\n";
        resident
            .update_chapter(
                ChapterTarget::new(id("GEN"), ChapterLabel::Number("2".into())),
                ChapterInput::Tokens(chapter_replacement(edited_source, "2")),
            )
            .unwrap();

        recompute_count::reset();
        let after_edit = match resident.vref_index(CorpusScope::Book(id("GEN"))).unwrap() {
            ScopedOutput::Single(entries) => entries,
            ScopedOutput::All(_) => panic!("expected a single-scope index"),
        };
        assert_eq!(
            recompute_count::get(),
            1,
            "only chapter 2's own run should have recomputed"
        );
        assert_eq!(after_edit.len(), 3);
        assert_ne!(after_edit[1], first[1], "chapter 2's own entry changed");
        // Chapter 1 and chapter 3's entries are untouched — the served-from-
        // cache proof at the content level, alongside the recompute count.
        assert_eq!(after_edit[0], first[0], "chapter 1 unaffected");
        assert_eq!(after_edit[2], first[2], "chapter 3 unaffected");
    }
}

/// Looks up or computes one run's entries against an existing cache, without
/// removing or replacing anything else in it — the chapter-scoped read path,
/// which only ever visits the one run it was asked for and so cannot claim
/// anything about whether its siblings are still current. The caller decides
/// whether/how to fold the returned `CachedRun` back into its cache.
///
/// `incoming_block_state` must be the true state carried in from every
/// earlier run in corpus order — never `None` as a stand-in for "unknown",
/// which is a real, distinct seed (no block has opened yet anywhere).
pub(crate) fn run_entries(
    cache: &[CachedRun],
    tokens: &[OwnedToken],
    incoming_block_state: Option<bool>,
) -> CachedRun {
    let identity = TokenIdentity::of(tokens);
    match cache
        .iter()
        .find(|run| run.matches(identity, incoming_block_state))
    {
        Some(hit) => hit.clone(),
        None => CachedRun::fresh(tokens, incoming_block_state),
    }
}

/// Looks up or moves one run's entries out of an old whole-book cache being
/// rebuilt — the whole-book read path. `old_cache` is drained via
/// `swap_remove` rather than searched non-destructively, because a
/// whole-book read visits every run exactly once, in corpus order, and the
/// caller collects every returned `CachedRun` into a brand new cache, so a
/// matched entry only ever needs to be found once.
pub(crate) fn take_or_compute(
    old_cache: &mut Vec<CachedRun>,
    tokens: &[OwnedToken],
    incoming_block_state: Option<bool>,
) -> CachedRun {
    let identity = TokenIdentity::of(tokens);
    match old_cache
        .iter()
        .position(|run| run.matches(identity, incoming_block_state))
    {
        Some(position) => old_cache.swap_remove(position),
        None => CachedRun::fresh(tokens, incoming_block_state),
    }
}

/// Reproduces [`usfm_onion::vref::VrefIndex::insert`]'s own dedup rule
/// across a whole book's concatenated per-run entries: a sid that has
/// already been seen keeps its first-seen *position* but takes the later
/// occurrence's *projection*.
///
/// Necessary specifically because a book may retain duplicate/reopened `\c`
/// runs sharing a chapter number, so the same sid can appear in two
/// different runs. The single-pass whole-stream projection this crate stays
/// equivalent to folds that case via one shared `VrefIndex`; this crate's
/// own per-run projection walks each run through a *separate* `VrefIndex`
/// (so a cache hit for one run cannot possibly see another run's entries),
/// so the same fold has to be redone once, here, over the concatenation —
/// otherwise a duplicate-labeled book would silently and wrongly report two
/// entries under one sid instead of the one the stateless projection would.
pub(crate) fn merge_by_sid(entries: Vec<VrefEntry>) -> Vec<VrefEntry> {
    let mut merged: Vec<VrefEntry> = Vec::with_capacity(entries.len());
    let mut positions: rustc_hash::FxHashMap<String, usize> =
        rustc_hash::FxHashMap::with_capacity_and_hasher(entries.len(), rustc_hash::FxBuildHasher);
    for entry in entries {
        match positions.get(&entry.sid) {
            Some(&position) => merged[position] = entry,
            None => {
                positions.insert(entry.sid.clone(), merged.len());
                merged.push(entry);
            }
        }
    }
    merged
}
