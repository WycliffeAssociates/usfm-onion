//! One resident book: authoritative bytes, owned tokens, and the ordered
//! duplicate-preserving chapter runs derived from them.

use std::ops::Range;

use rustc_hash::{FxHashMap, FxHashSet};

use usfm_onion::lint::{LintOptions, LintResult, lint_tokens};
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, LineEnding, OwnedToken, tokens_to_usfm_reconstruct_with_eol};
use usfm_onion::walker::chapter_segments;

use crate::baseline::BaselineState;
use crate::error::IngestError;
use crate::input::{BookInput, ChapterLabel, SourceKey};
use crate::patch::ResolvedFix;
use crate::state::{SourceHash, TokenIdentity};
use crate::vref::CachedRun;

/// One contiguous chapter run: its label and its token range in the book.
///
/// A book may hold several runs with the same label — duplicate and reopened
/// `\c` numbers are retained in source order, never collapsed or sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChapterRun {
    pub(crate) label: ChapterLabel,
    pub(crate) range: Range<usize>,
}

/// A validated, fully derived book, built as a candidate before any resident
/// state is touched.
#[derive(Debug, Clone)]
pub(crate) struct BookState {
    pub(crate) source_key: SourceKey,
    pub(crate) book: BookId,
    /// The exact bytes this book would be saved as.
    pub(crate) source: String,
    pub(crate) hash: SourceHash,
    /// The token stream's own identity — see [`TokenIdentity`]. Derived here with
    /// the hash so every path that installs a book computes both or neither.
    pub(crate) token_identity: TokenIdentity,
    pub(crate) tokens: Vec<OwnedToken>,
    pub(crate) runs: Vec<ChapterRun>,
    pub(crate) line_ending: LineEnding,
    /// Set whenever this book's content or the lint config changed, cleared by
    /// a lint run. Derived from authoritative state rather than consumed from a
    /// queue, so retrying after a failure is safe.
    pub(crate) lint_dirty: bool,
    /// This book's last computed lint contribution. `None` until the first
    /// recompute; a dirty book's stale result is kept until a new one replaces
    /// it, so nothing is ever published half-recomputed.
    pub(crate) lint: Option<LintResult>,
    /// The fixes of [`Self::lint`], resolved against this book's own token
    /// stream. Resolved once at recompute time rather than on every read: the
    /// positions are only meaningful for the token stream they were resolved
    /// against, which is the stream this book held when its result was computed.
    pub(crate) patches: Vec<ResolvedFix>,
    /// "What was last saved" for this book, if anything has been declared —
    /// see [`BaselineState`]. Never written by anything but
    /// `Braid::set_baseline`/`clear_baseline`, and never inspected by an
    /// ordinary content mutation: a candidate replacing this book (edited,
    /// formatted, patched) always carries this field forward unchanged.
    pub(crate) baseline: Option<BaselineState>,
    /// Per-chapter-run vref-index cache. Content-addressed by each run's own
    /// `TokenIdentity` and re-verified against it on every read, so carrying
    /// this forward across a mutation is always safe: an entry that no
    /// longer matches any current run is simply never matched again, never
    /// applied. See `crate::vref` for why a run's own token identity is a
    /// sound key for this projection specifically.
    pub(crate) vref_cache: Vec<CachedRun>,
}

impl BookState {
    /// Builds a candidate book. Every failure here happens before the caller's
    /// resident state is touched.
    pub(crate) fn build(input: BookInput) -> Result<Self, IngestError> {
        let (source_key, book, source, tokens, line_ending) = match input {
            BookInput::Usfm {
                source_key,
                book,
                source,
            } => {
                let line_ending = LineEnding::detect(&source);
                let tokens = parse(&source)
                    .tokens
                    .iter()
                    .map(OwnedToken::from_parsed)
                    .collect();
                // The bytes stay exactly as supplied: a mixed-ending file is
                // preserved verbatim until an edit forces re-emission, at
                // which point the detected ending applies.
                (source_key, book, source, tokens, line_ending)
            }
            BookInput::Tokens(input) => {
                let source = tokens_to_usfm_reconstruct_with_eol(&input.tokens, input.line_ending);
                (
                    input.source_key,
                    input.book,
                    source,
                    input.tokens,
                    input.line_ending,
                )
            }
        };

        validate_token_ids(book, &tokens)?;
        Ok(Self {
            source_key,
            book,
            hash: SourceHash::of(&source),
            token_identity: TokenIdentity::of(&tokens),
            source,
            runs: chapter_runs(&tokens),
            tokens,
            line_ending,
            lint_dirty: true,
            lint: None,
            patches: Vec::new(),
            baseline: None,
            // Nothing to carry forward — this is either a brand new book or
            // a whole-book reseed with no predecessor in scope here.
            vref_cache: Vec::new(),
        })
    }

    /// Rebuilds bytes, hash, and runs from an already-validated token stream —
    /// the chapter-splice path, where the tokens are the mutation and
    /// everything else is derived from them.
    ///
    /// The predecessor's baseline and vref cache both carry forward as they
    /// were: a content mutation is not how a baseline changes, and every
    /// vref cache entry is re-verified against its own run's current tokens
    /// on read (see `crate::vref`), so carrying a now-partly-stale cache
    /// forward is always safe — a run whose content changed simply stops
    /// matching its old entry and gets recomputed there, not here.
    pub(crate) fn rebuilt(&self, tokens: Vec<OwnedToken>) -> Result<Self, IngestError> {
        validate_token_ids(self.book, &tokens)?;
        let source = tokens_to_usfm_reconstruct_with_eol(&tokens, self.line_ending);
        Ok(Self {
            source_key: self.source_key.clone(),
            book: self.book,
            hash: SourceHash::of(&source),
            token_identity: TokenIdentity::of(&tokens),
            source,
            runs: chapter_runs(&tokens),
            tokens,
            line_ending: self.line_ending,
            lint_dirty: true,
            lint: None,
            patches: Vec::new(),
            baseline: self.baseline.clone(),
            vref_cache: self.vref_cache.clone(),
        })
    }

    /// Adopts a content-identical predecessor's cached lint contribution.
    ///
    /// A no-op mutation must leave the caches exactly as they were — including
    /// after a source-key rebinding, which changes where a book came from but
    /// not what it says. Only ever called when [`Self::content_eq`] holds.
    pub(crate) fn inherit_cache(&mut self, resident: &Self) {
        debug_assert!(self.content_eq(resident));
        self.lint_dirty = resident.lint_dirty;
        self.lint = resident.lint.clone();
        self.patches = resident.patches.clone();
        self.baseline = resident.baseline.clone();
        self.vref_cache = resident.vref_cache.clone();
    }

    /// Recomputes this book's whole-book lint contribution and the patch table
    /// derived from it.
    ///
    /// The caller-declared [`BookId`] enters core's lint context here — this is
    /// the only place it can, since braid is the only thing that knows which
    /// book a source was declared as. A source whose own `\id` names a
    /// different valid book reports `book-id-mismatch` because of it.
    pub(crate) fn recompute_lint(&mut self, base: &LintOptions) {
        let mut options = base.clone();
        options.declared_book = Some(self.book);
        let result = lint_tokens(&self.tokens, options);
        let patches = self.resolve_fixes(&result);
        self.install_lint(result, patches);
    }

    /// Flattens every finding's fix against this book's token stream.
    ///
    /// Position resolution is by token id, the only address a `TokenFix`
    /// carries. A fix naming a token this book does not hold cannot happen from
    /// its own lint pass — core only attaches a fix to a token it just read, and
    /// resident ids are unique — and is skipped rather than guessed at. A fix that
    /// edits nothing is skipped for the same reason it is unrepresentable as a
    /// patch: there is nothing to address.
    fn resolve_fixes(&self, result: &LintResult) -> Vec<ResolvedFix> {
        let positions = self.positions_by_id();
        result
            .issues
            .iter()
            .filter_map(|issue| {
                let fix = issue.fix.as_ref()?;
                let position = positions.get(fix.target_token_id()).copied();
                debug_assert!(
                    position.is_some(),
                    "a fix from this book's own lint names one of its tokens"
                );
                let resolved = ResolvedFix::new(fix, position?, self.hash);
                debug_assert!(
                    resolved.is_some(),
                    "no rule produces a fix that edits nothing"
                );
                resolved
            })
            .collect()
    }

    /// Same flattening as [`Self::resolve_fixes`], for a lint result this book
    /// did not just compute — a restored or primed cache. Foreign data does not
    /// get the same trust: a matching source hash and matching stamps rule out
    /// most drift, but not all of it, so every fix must resolve to a real
    /// position and a non-empty patch or the whole contribution is refused
    /// (`None`) rather than silently adopting the findings with a partial or
    /// dropped patch table.
    pub(crate) fn try_resolve_cached_fixes(&self, result: &LintResult) -> Option<Vec<ResolvedFix>> {
        let positions = self.positions_by_id();
        result
            .issues
            .iter()
            .filter_map(|issue| issue.fix.as_ref())
            .map(|fix| {
                let position = *positions.get(fix.target_token_id())?;
                ResolvedFix::new(fix, position, self.hash)
            })
            .collect()
    }

    /// Installs a lint contribution — freshly computed or a validated cached
    /// one — as this book's current one. The shared tail of ordinary recompute
    /// and cache priming, so both paths leave the same three fields in the
    /// same consistent state.
    pub(crate) fn install_lint(&mut self, result: LintResult, patches: Vec<ResolvedFix>) {
        self.patches = patches;
        self.lint = Some(result);
        self.lint_dirty = false;
    }

    fn positions_by_id(&self) -> FxHashMap<&str, u32> {
        let mut positions =
            FxHashMap::with_capacity_and_hasher(self.tokens.len(), rustc_hash::FxBuildHasher);
        for (position, token) in self.tokens.iter().enumerate() {
            positions.insert(token.id().as_str(), position as u32);
        }
        positions
    }

    /// Exact content equality — the check that decides a no-op. Hash equality
    /// alone never decides it, and token equality is part of it because two
    /// streams can serialize to the same bytes while carrying different token
    /// identities.
    pub(crate) fn content_eq(&self, other: &Self) -> bool {
        self.hash == other.hash
            && self.source == other.source
            && self.line_ending == other.line_ending
            && self.tokens == other.tokens
    }

    /// True when any label appears on more than one run, which makes every
    /// chapter address in this book ambiguous to a consumer.
    pub(crate) fn has_duplicate_labels(&self) -> bool {
        let mut seen = FxHashSet::default();
        !self.runs.iter().all(|run| seen.insert(&run.label))
    }

    /// Indices of the runs carrying `label`, in source order.
    pub(crate) fn matching_runs(&self, label: &ChapterLabel) -> Vec<usize> {
        self.runs
            .iter()
            .enumerate()
            .filter(|(_, run)| &run.label == label)
            .map(|(index, _)| index)
            .collect()
    }

    /// Whole-book replacement from this book's own declared baseline —
    /// `Braid::revert_to_baseline`'s per-book step.
    ///
    /// Token identity is load-bearing here: every stored fact (source, hash,
    /// tokens with their ORIGINAL ids, line ending, runs) is reinstalled
    /// verbatim from `BaselineState`, never re-parsed or re-minted, because a
    /// revert's whole point is to reproduce exactly what was baselined —
    /// including the ids anything already anchored on. `token_identity` is
    /// still recomputed (it is a derived fact about the reinstalled tokens,
    /// not a stored one), the same as every other path that installs a token
    /// stream. The baseline slot itself is carried forward unchanged: a
    /// revert does not clear or move what it just reproduced.
    pub(crate) fn reverted_to_baseline(&self, baseline: &BaselineState) -> Self {
        Self {
            source_key: self.source_key.clone(),
            book: self.book,
            hash: baseline.hash,
            token_identity: TokenIdentity::of(&baseline.tokens),
            source: baseline.source.clone(),
            runs: baseline.runs.clone(),
            tokens: baseline.tokens.clone(),
            line_ending: baseline.line_ending,
            lint_dirty: true,
            lint: None,
            patches: Vec::new(),
            baseline: self.baseline.clone(),
            vref_cache: Vec::new(),
        }
    }
}

/// The ordered runs of a token stream, using core's own chapter segmentation so
/// braid's grain cannot drift from the linter's.
pub(crate) fn chapter_runs(tokens: &[OwnedToken]) -> Vec<ChapterRun> {
    chapter_segments(tokens)
        .into_iter()
        .map(|segment| ChapterRun {
            label: segment_label(tokens, &segment.range, segment.is_front),
            range: segment.range,
        })
        .collect()
}

/// A chapter segment's label is the verbatim number token core segmented on;
/// front matter has none.
fn segment_label(tokens: &[OwnedToken], range: &Range<usize>, is_front: bool) -> ChapterLabel {
    if is_front {
        return ChapterLabel::FrontMatter;
    }
    // Core opens a segment only at a `\c` whose next token is a number, so the
    // label token is always there.
    let label = tokens[range.start + 1].source();
    ChapterLabel::Number(Box::from(label))
}

fn validate_token_ids(book: BookId, tokens: &[OwnedToken]) -> Result<(), IngestError> {
    let mut seen = FxHashSet::with_capacity_and_hasher(tokens.len(), rustc_hash::FxBuildHasher);
    for token in tokens {
        if !seen.insert(token.id()) {
            return Err(IngestError::DuplicateTokenId {
                book,
                id: token.id().clone(),
            });
        }
    }
    Ok(())
}
