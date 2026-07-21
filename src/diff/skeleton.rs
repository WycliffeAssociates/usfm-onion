//! Merge-as-projection: a gapless Myers interleave of baseline/current sid
//! blocks, pair-loose coalesced moves occupying two slots bound to one
//! decision, and a pure per-slot merge.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use similar::{Algorithm, ChangeTag, capture_diff_slices};

use super::{
    DiffableToken, SidBlock, classify_text_diff, derive_canonical_sids,
    group_tokens_by_book_and_chapter, partition_by_sid,
};
use crate::parse::parse;
use crate::token::{BookId, Sid, Token};
use rustc_hash::{FxHashMap, FxHashSet};

/// Stable identity for a decision unit. Never a display SID used as an
/// implicit foreign key — slot references and decision-map keys use this.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct UnitId(String);

impl UnitId {
    /// Wraps a caller-supplied id (e.g. from a decisions map or a revert
    /// request). Construction never validates — an id with no matching unit
    /// in the skeleton is a supported input, rejected at merge time as
    /// [`MergeError::UnknownUnitId`], not here.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Which side of a decision a caller chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MergeSide {
    Baseline,
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SlotRole {
    Shared,
    BaselineOnly,
    CurrentOnly,
    PairBaseline,
    PairCurrent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Anchor {
    pub unit_id: UnitId,
    pub sid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Slot {
    pub unit_id: UnitId,
    pub role: SlotRole,
    pub after: Option<Anchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DecisionUnitKind {
    Shared,
    Added,
    Deleted,
    Coalesced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DecisionStatus {
    Unchanged,
    Modified,
    Added,
    Deleted,
    Moved,
}

/// How many blocks on each side share a unit's pairing key (dup neighborhood).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct DupContext {
    pub baseline_count: u32,
    pub current_count: u32,
}

impl DupContext {
    pub fn is_dup(&self) -> bool {
        self.baseline_count > 1 || self.current_count > 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CoveredSide {
    Baseline,
    Current,
}

/// Bridge-coverage narration for a one-sided unit whose verse number is
/// overlapped by a true multi-verse range on the opposite side of a
/// coalesced pair, in the same book/chapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoveredBy {
    pub unit_id: UnitId,
    pub sid: String,
    pub side: CoveredSide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionUnit<T> {
    pub id: UnitId,
    pub kind: DecisionUnitKind,
    pub status: DecisionStatus,
    pub baseline_sid: Option<String>,
    pub current_sid: Option<String>,
    pub baseline_tokens: Vec<T>,
    pub current_tokens: Vec<T>,
    pub displaced: bool,
    pub relabeled: bool,
    pub dup_context: DupContext,
    pub covered_by: Option<CoveredBy>,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffSkeleton<T> {
    pub slots: Vec<Slot>,
    pub units: Vec<DecisionUnit<T>>,
}

/// External/app-shaped token diff (interim calling convention: carried sids
/// trusted unchanged — see [`super::build_sid_blocks`]).
pub fn diff_skeleton<T: DiffableToken>(baseline: &[T], current: &[T]) -> DiffSkeleton<T> {
    let baseline_sids: Vec<String> = baseline
        .iter()
        .map(|token| token.sid_string().unwrap_or_default())
        .collect();
    let current_sids: Vec<String> = current
        .iter()
        .map(|token| token.sid_string().unwrap_or_default())
        .collect();
    build_skeleton(baseline, &baseline_sids, current, &current_sids)
}

/// Native calling convention for parsed `Token` streams: sids come from
/// [`derive_canonical_sids`], which never trusts a carried sid.
pub fn diff_skeleton_canonical<T: DiffableToken>(
    baseline: &[T],
    baseline_book: &str,
    current: &[T],
    current_book: &str,
) -> DiffSkeleton<T> {
    let baseline_sids = derive_canonical_sids(baseline, baseline_book);
    let current_sids = derive_canonical_sids(current, current_book);
    build_skeleton(baseline, &baseline_sids, current, &current_sids)
}

/// Native, source-level diff grouped by book and chapter — one canonical
/// skeleton per chapter, matching the by-chapter map shape of the retired
/// flat `diff_usfm_sources_by_chapter`. The core stays slice-generic; this is
/// the one place onion batches per book/chapter, not a second merge API.
pub fn diff_skeleton_by_chapter<'a>(
    baseline_usfm: &'a str,
    current_usfm: &'a str,
) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<Token<'a>>>> {
    let baseline = parse(baseline_usfm);
    let current = parse(current_usfm);

    let baseline_groups = group_tokens_by_book_and_chapter(
        &baseline.tokens,
        baseline.analysis.book_code.unwrap_or("unknown"),
    );
    let current_groups = group_tokens_by_book_and_chapter(
        &current.tokens,
        current.analysis.book_code.unwrap_or("unknown"),
    );

    diff_grouped_by_chapter(
        baseline_groups,
        current_groups,
        |book, baseline, current| diff_skeleton_canonical(baseline, book, current, book),
    )
}

/// External/app-shaped by-chapter diff (carried-sid convention — see
/// [`diff_skeleton`]): groups tokens by book/chapter first, then diffs each
/// chapter independently. This is the same batching shape as
/// [`diff_skeleton_by_chapter`] for the other calling convention — one
/// grouping algorithm, not a second one for app tokens.
pub fn diff_skeleton_by_chapter_from_tokens<T: DiffableToken + Sync>(
    baseline_tokens: &[T],
    current_tokens: &[T],
) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<T>>>
where
    DiffSkeleton<T>: Send,
{
    let baseline_groups = group_tokens_by_book_and_chapter(baseline_tokens, "unknown");
    let current_groups = group_tokens_by_book_and_chapter(current_tokens, "unknown");

    diff_grouped_by_chapter(
        baseline_groups,
        current_groups,
        |_book, baseline, current| diff_skeleton(baseline, current),
    )
}

/// Total token count (baseline + current, across every cell) at or above which
/// the per-chapter diffs run through the ordered-map seam. Below it the fixed
/// decomposition cost (building the work list and the ordered collect)
/// outweighs the parallel gain, so the cells diff serially. A real multi-
/// chapter book clears it comfortably; a small input diffs in well under a
/// millisecond either way.
const PARALLEL_MIN_TOKENS: usize = 20_000;

/// One per-chapter diff work unit: its book/chapter coordinates and the two
/// slices to diff. Slices are the same per-book/chapter clones the serial path
/// produced; nothing is shared between cells.
struct DiffCell<T> {
    book: String,
    chapter: u32,
    baseline: Vec<T>,
    current: Vec<T>,
}

/// Group-and-diff shared by both by-chapter entry points. Flattens the
/// per-(book, chapter) cells into a work list, diffs each cell independently
/// through the ordered-map seam (a Rayon pool natively, serial on wasm), and
/// rebuilds the nested map. Each cell's slices come only from its own
/// book/chapter, so the result is identical to diffing the cells one at a time
/// regardless of thread count or target. The two callers differ only in
/// `diff_cell`: the canonical path threads the book code through, the
/// carried-sid path ignores it.
fn diff_grouped_by_chapter<T, F>(
    baseline_groups: BTreeMap<String, BTreeMap<u32, Vec<T>>>,
    current_groups: BTreeMap<String, BTreeMap<u32, Vec<T>>>,
    diff_cell: F,
) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<T>>>
where
    T: DiffableToken + Sync,
    DiffSkeleton<T>: Send,
    F: Fn(&str, &[T], &[T]) -> DiffSkeleton<T> + Sync + Send,
{
    let mut all_books = BTreeSet::<String>::new();
    all_books.extend(baseline_groups.keys().cloned());
    all_books.extend(current_groups.keys().cloned());

    let mut cells: Vec<DiffCell<T>> = Vec::new();
    for book in all_books {
        let mut all_chapters = BTreeSet::<u32>::new();
        if let Some(chapters) = baseline_groups.get(&book) {
            all_chapters.extend(chapters.keys().copied());
        }
        if let Some(chapters) = current_groups.get(&book) {
            all_chapters.extend(chapters.keys().copied());
        }
        for chapter in all_chapters {
            let baseline = baseline_groups
                .get(&book)
                .and_then(|chapters| chapters.get(&chapter))
                .cloned()
                .unwrap_or_default();
            let current = current_groups
                .get(&book)
                .and_then(|chapters| chapters.get(&chapter))
                .cloned()
                .unwrap_or_default();
            cells.push(DiffCell {
                book: book.clone(),
                chapter,
                baseline,
                current,
            });
        }
    }

    let total: usize = cells
        .iter()
        .map(|cell| cell.baseline.len() + cell.current.len())
        .sum();
    let diffs: Vec<DiffSkeleton<T>> = if total >= PARALLEL_MIN_TOKENS {
        crate::par::map_ordered(&cells, |cell| {
            diff_cell(&cell.book, &cell.baseline, &cell.current)
        })
    } else {
        cells
            .iter()
            .map(|cell| diff_cell(&cell.book, &cell.baseline, &cell.current))
            .collect()
    };

    let mut out: BTreeMap<String, BTreeMap<u32, DiffSkeleton<T>>> = BTreeMap::new();
    for (cell, diff) in cells.into_iter().zip(diffs) {
        out.entry(cell.book).or_default().insert(cell.chapter, diff);
    }
    out
}

/// A staged unit id with no matching unit in the skeleton being merged. The
/// caller must abort and re-diff — there is no fuzzy/stale-id fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeError {
    UnknownUnitId(UnitId),
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUnitId(id) => write!(f, "unknown decision unit id: {id}"),
        }
    }
}

impl std::error::Error for MergeError {}

/// Pure projection: walk the skeleton once and emit each slot's chosen side.
/// Unknown ids in `decisions` are validated before any output is assembled —
/// contribution cardinality is exact (a `Shared` unit emits once; a
/// `Coalesced` pair emits exactly once for either choice via its two slots;
/// `Added`/`Deleted` emit zero or once). Output tokens are clones of the
/// inputs — never trimmed, normalized, or reserialized.
pub fn merge_skeleton<T: Clone>(
    skeleton: &DiffSkeleton<T>,
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) -> Result<Vec<T>, MergeError> {
    let units_by_id: FxHashMap<&UnitId, &DecisionUnit<T>> =
        skeleton.units.iter().map(|unit| (&unit.id, unit)).collect();

    for id in decisions.keys() {
        if !units_by_id.contains_key(id) {
            return Err(MergeError::UnknownUnitId(id.clone()));
        }
    }

    let mut out = Vec::new();
    for slot in &skeleton.slots {
        let unit = units_by_id[&slot.unit_id];
        let side = decisions
            .get(&slot.unit_id)
            .copied()
            .unwrap_or(default_side);
        match slot.role {
            SlotRole::Shared => {
                let tokens = if side == MergeSide::Baseline {
                    &unit.baseline_tokens
                } else {
                    &unit.current_tokens
                };
                out.extend(tokens.iter().cloned());
            }
            SlotRole::BaselineOnly | SlotRole::PairBaseline => {
                if side == MergeSide::Baseline {
                    out.extend(unit.baseline_tokens.iter().cloned());
                }
            }
            SlotRole::CurrentOnly | SlotRole::PairCurrent => {
                if side == MergeSide::Current {
                    out.extend(unit.current_tokens.iter().cloned());
                }
            }
        }
    }

    Ok(out)
}

/// Convenience wrapper: builds the skeleton once (external/`FormatToken`
/// calling convention — see [`diff_skeleton`]) and delegates to
/// [`merge_skeleton`].
pub fn merge_diff_blocks<T: DiffableToken>(
    baseline: &[T],
    current: &[T],
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) -> Result<Vec<T>, MergeError> {
    let skeleton = diff_skeleton(baseline, current);
    merge_skeleton(&skeleton, decisions, default_side)
}

/// Single revert is one-decision merge: `{X: Baseline}` with default
/// `Current`. An unknown id is an error — the caller must abort and re-diff,
/// never fall back to a fuzzy stale-id match.
pub fn revert_diff_block<T: DiffableToken>(
    diff_block_id: &str,
    baseline_tokens: &[T],
    current_tokens: &[T],
) -> Result<Vec<T>, MergeError> {
    let mut decisions = BTreeMap::new();
    decisions.insert(UnitId::new(diff_block_id), MergeSide::Baseline);
    merge_diff_blocks(
        baseline_tokens,
        current_tokens,
        &decisions,
        MergeSide::Current,
    )
}

/// Pairing key ("pair-loose"): book + chapter + verse-range-start, packed
/// into the same 8-byte `Copy` `Sid` the walker already uses. Strips the
/// range end, `_dup_N`, and `#occurrence` suffix, but never crosses a verse
/// number — same semantics as the previous `String`-based key, just parsed
/// once per block instead of re-parsed at every one of this function's
/// call sites, and hashed/compared as a fixed-size value instead of a
/// heap string.
///
/// Every real sid here comes from `derive_canonical_sids` or a trusted
/// carried sid, both of which always include a `book chapter:verse` colon
/// (chapter-open and intro blocks included, per `derive_canonical_sids`'s
/// own doc comment) — so the fallback below is defensive, not a normal
/// case: an unparsable sid degrades to sharing one bucket rather than
/// panicking.
fn pairing_key(sid: &str) -> Sid {
    let fallback = Sid::new(BookId::UNKNOWN, 0, 0);
    let Some((book, rest)) = sid.split_once(' ') else {
        return fallback;
    };
    let Some(book) = BookId::from_str(book) else {
        return fallback;
    };
    let Some((chapter, verse_part)) = rest.split_once(':') else {
        return fallback;
    };
    let Ok(chapter) = chapter.parse::<u16>() else {
        return fallback;
    };
    let verse_start: u16 = verse_part
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    Sid::new(book, chapter, verse_start)
}

/// Parse `(book, chapter, verse_start, verse_end)` out of a canonical sid
/// string, for bridge-coverage overlap checks. Returns `None` for a
/// malformed/intro sid.
fn sid_range(sid: &str) -> Option<(&str, u32, u32, u32)> {
    let (book, rest) = sid.split_once(' ')?;
    let (chapter_str, verse_part) = rest.split_once(':')?;
    let chapter: u32 = chapter_str.parse().ok()?;
    let core = verse_part.split('_').next().unwrap_or(verse_part);
    let (start_str, end_str) = core.split_once('-').unwrap_or((core, core));
    let start: u32 = start_str.parse().ok()?;
    let end: u32 = end_str.parse().ok()?;
    Some((book, chapter, start, end))
}

/// Matched `(baseline_index, current_index)` pairs from a Myers LCS over the
/// block-id sequences. Reuses `similar`'s Myers implementation — no second
/// LCS/DP implementation.
fn myers_pairs(baseline_ids: &[String], current_ids: &[String]) -> Vec<(usize, usize)> {
    if baseline_ids.is_empty() || current_ids.is_empty() {
        return Vec::new();
    }
    if baseline_ids == current_ids {
        return (0..baseline_ids.len())
            .map(|index| (index, index))
            .collect();
    }

    let diff = capture_diff_slices(Algorithm::Myers, baseline_ids, current_ids);
    let mut pairs = Vec::new();
    let mut baseline_cursor = 0usize;
    let mut current_cursor = 0usize;

    for op in diff {
        for (tag, slice) in op.iter_slices(baseline_ids, current_ids) {
            let len = slice.len();
            match tag {
                ChangeTag::Equal => {
                    for offset in 0..len {
                        pairs.push((baseline_cursor + offset, current_cursor + offset));
                    }
                    baseline_cursor += len;
                    current_cursor += len;
                }
                ChangeTag::Delete => baseline_cursor += len,
                ChangeTag::Insert => current_cursor += len,
            }
        }
    }

    pairs
}

enum Step {
    Shared(usize, usize),
    BaselineOnly(usize),
    CurrentOnly(usize),
}

struct UnitBuild<T> {
    want_id: String,
    kind: DecisionUnitKind,
    key: Sid,
    baseline_sid: Option<String>,
    current_sid: Option<String>,
    baseline_text: Option<String>,
    current_text: Option<String>,
    baseline_tokens: Vec<T>,
    current_tokens: Vec<T>,
}

fn unique_id(taken: &mut FxHashSet<String>, want: String) -> UnitId {
    if taken.insert(want.clone()) {
        return UnitId(want);
    }
    let mut suffix = 1u32;
    loop {
        let candidate = format!("{want}@{suffix}");
        if taken.insert(candidate.clone()) {
            return UnitId(candidate);
        }
        suffix += 1;
    }
}

fn build_skeleton<T: DiffableToken>(
    baseline_tokens: &[T],
    baseline_sids: &[String],
    current_tokens: &[T],
    current_sids: &[String],
) -> DiffSkeleton<T> {
    let baseline_blocks: Vec<SidBlock> =
        partition_by_sid(baseline_tokens, |index| baseline_sids[index].clone());
    let current_blocks: Vec<SidBlock> =
        partition_by_sid(current_tokens, |index| current_sids[index].clone());

    let baseline_block_ids: Vec<String> = baseline_blocks
        .iter()
        .map(|block| block.block_id.clone())
        .collect();
    let current_block_ids: Vec<String> = current_blocks
        .iter()
        .map(|block| block.block_id.clone())
        .collect();

    let pairs = myers_pairs(&baseline_block_ids, &current_block_ids);
    let shared_baseline: FxHashSet<usize> = pairs.iter().map(|&(b, _)| b).collect();
    let shared_current: FxHashSet<usize> = pairs.iter().map(|&(_, c)| c).collect();

    // Interleave steps: between LCS anchors, emit the baseline-only run, then
    // the current-only run, then the shared block — the supersequence order.
    let mut steps = Vec::new();
    let mut bi = 0usize;
    let mut ci = 0usize;
    for &(pb, pc) in &pairs {
        while bi < pb {
            steps.push(Step::BaselineOnly(bi));
            bi += 1;
        }
        while ci < pc {
            steps.push(Step::CurrentOnly(ci));
            ci += 1;
        }
        steps.push(Step::Shared(pb, pc));
        bi = pb + 1;
        ci = pc + 1;
    }
    while bi < baseline_blocks.len() {
        steps.push(Step::BaselineOnly(bi));
        bi += 1;
    }
    while ci < current_blocks.len() {
        steps.push(Step::CurrentOnly(ci));
        ci += 1;
    }

    let baseline_only: Vec<usize> = (0..baseline_blocks.len())
        .filter(|index| !shared_baseline.contains(index))
        .collect();
    let current_only: Vec<usize> = (0..current_blocks.len())
        .filter(|index| !shared_current.contains(index))
        .collect();

    // Parsed once per block (not re-parsed at every call site below) into
    // the same 8-byte `Copy` `Sid` the walker uses — see `pairing_key`.
    let baseline_keys: Vec<Sid> = baseline_blocks
        .iter()
        .map(|block| pairing_key(&block.semantic_sid))
        .collect();
    let current_keys: Vec<Sid> = current_blocks
        .iter()
        .map(|block| pairing_key(&block.semantic_sid))
        .collect();

    // Pair-loose coalesce: off-Myers blocks sharing a pairing key. Tier 1
    // (exact-text-first, stream order among ties), then tier 2 (positional
    // leftovers, first-with-first). Keys are visited in baseline stream
    // first-occurrence order for deterministic unit-creation order.
    let mut baseline_by_key: FxHashMap<Sid, Vec<usize>> = FxHashMap::default();
    let mut key_order: Vec<Sid> = Vec::new();
    for &index in &baseline_only {
        let key = baseline_keys[index];
        if !baseline_by_key.contains_key(&key) {
            key_order.push(key);
        }
        baseline_by_key.entry(key).or_default().push(index);
    }
    let mut current_by_key: FxHashMap<Sid, Vec<usize>> = FxHashMap::default();
    for &index in &current_only {
        let key = current_keys[index];
        current_by_key.entry(key).or_default().push(index);
    }

    let mut pair_for_baseline: FxHashMap<usize, usize> = FxHashMap::default();
    let mut pair_for_current: FxHashMap<usize, usize> = FxHashMap::default();
    let mut ordered_pairs: Vec<(usize, usize)> = Vec::new();

    for key in &key_order {
        let bis = baseline_by_key.get(key).cloned().unwrap_or_default();
        let cis = current_by_key.get(key).cloned().unwrap_or_default();
        let mut current_used: FxHashSet<usize> = FxHashSet::default();
        let mut baseline_left: Vec<usize> = Vec::new();

        for &bi in &bis {
            let matched = cis.iter().copied().find(|ci| {
                !current_used.contains(ci)
                    && baseline_blocks[bi].text_full == current_blocks[*ci].text_full
            });
            if let Some(ci) = matched {
                current_used.insert(ci);
                pair_for_baseline.insert(bi, ci);
                pair_for_current.insert(ci, bi);
                ordered_pairs.push((bi, ci));
            } else {
                baseline_left.push(bi);
            }
        }

        let current_left: Vec<usize> = cis
            .iter()
            .copied()
            .filter(|ci| !current_used.contains(ci))
            .collect();
        for (offset, &bi) in baseline_left.iter().enumerate() {
            if let Some(&ci) = current_left.get(offset) {
                pair_for_baseline.insert(bi, ci);
                pair_for_current.insert(ci, bi);
                ordered_pairs.push((bi, ci));
            }
        }
    }

    // Build units in the same order the prototype's addUnit calls happen:
    // shared (Myers order), coalesced (pairing insertion order), deleted
    // (baseline stream order), added (current stream order).
    let mut builds: Vec<UnitBuild<T>> = Vec::new();
    let mut unit_for_baseline_block: FxHashMap<usize, usize> = FxHashMap::default();
    let mut unit_for_current_block: FxHashMap<usize, usize> = FxHashMap::default();

    for &(pb, pc) in &pairs {
        let bb = &baseline_blocks[pb];
        let cb = &current_blocks[pc];
        let idx = builds.len();
        builds.push(UnitBuild {
            want_id: bb.block_id.clone(),
            kind: DecisionUnitKind::Shared,
            key: baseline_keys[pb],
            baseline_sid: Some(bb.semantic_sid.clone()),
            current_sid: Some(cb.semantic_sid.clone()),
            baseline_text: Some(bb.text_full.clone()),
            current_text: Some(cb.text_full.clone()),
            baseline_tokens: baseline_tokens[bb.start..bb.end_exclusive].to_vec(),
            current_tokens: current_tokens[cb.start..cb.end_exclusive].to_vec(),
        });
        unit_for_baseline_block.insert(pb, idx);
        unit_for_current_block.insert(pc, idx);
    }

    for &(bi, ci) in &ordered_pairs {
        let bb = &baseline_blocks[bi];
        let cb = &current_blocks[ci];
        let idx = builds.len();
        builds.push(UnitBuild {
            want_id: cb.block_id.clone(), // current-major decision key
            kind: DecisionUnitKind::Coalesced,
            key: baseline_keys[bi],
            baseline_sid: Some(bb.semantic_sid.clone()),
            current_sid: Some(cb.semantic_sid.clone()),
            baseline_text: Some(bb.text_full.clone()),
            current_text: Some(cb.text_full.clone()),
            baseline_tokens: baseline_tokens[bb.start..bb.end_exclusive].to_vec(),
            current_tokens: current_tokens[cb.start..cb.end_exclusive].to_vec(),
        });
        unit_for_baseline_block.insert(bi, idx);
        unit_for_current_block.insert(ci, idx);
    }

    for &bi in &baseline_only {
        if pair_for_baseline.contains_key(&bi) {
            continue;
        }
        let bb = &baseline_blocks[bi];
        let idx = builds.len();
        builds.push(UnitBuild {
            want_id: bb.block_id.clone(),
            kind: DecisionUnitKind::Deleted,
            key: baseline_keys[bi],
            baseline_sid: Some(bb.semantic_sid.clone()),
            current_sid: None,
            baseline_text: Some(bb.text_full.clone()),
            current_text: None,
            baseline_tokens: baseline_tokens[bb.start..bb.end_exclusive].to_vec(),
            current_tokens: Vec::new(),
        });
        unit_for_baseline_block.insert(bi, idx);
    }

    for &ci in &current_only {
        if pair_for_current.contains_key(&ci) {
            continue;
        }
        let cb = &current_blocks[ci];
        let idx = builds.len();
        builds.push(UnitBuild {
            want_id: cb.block_id.clone(),
            kind: DecisionUnitKind::Added,
            key: current_keys[ci],
            baseline_sid: None,
            current_sid: Some(cb.semantic_sid.clone()),
            baseline_text: None,
            current_text: Some(cb.text_full.clone()),
            baseline_tokens: Vec::new(),
            current_tokens: current_tokens[cb.start..cb.end_exclusive].to_vec(),
        });
        unit_for_current_block.insert(ci, idx);
    }

    // Deterministic unit ids: sid for the first contiguous... no, here it's
    // "first creation" collision-safety: `@1`, `@2`, ... in creation order.
    let mut taken_ids: FxHashSet<String> = FxHashSet::default();
    let mut unit_ids: Vec<UnitId> = Vec::with_capacity(builds.len());
    for build in &builds {
        unit_ids.push(unique_id(&mut taken_ids, build.want_id.clone()));
    }

    // dup_context: how many blocks on EACH side (shared and off-Myers alike)
    // share this unit's pairing key.
    let mut baseline_key_count: FxHashMap<Sid, u32> = FxHashMap::default();
    for &key in &baseline_keys {
        *baseline_key_count.entry(key).or_insert(0) += 1;
    }
    let mut current_key_count: FxHashMap<Sid, u32> = FxHashMap::default();
    for &key in &current_keys {
        *current_key_count.entry(key).or_insert(0) += 1;
    }

    let mut units: Vec<DecisionUnit<T>> = builds
        .into_iter()
        .zip(unit_ids.iter().cloned())
        .map(|(build, id)| {
            let status = match build.kind {
                DecisionUnitKind::Shared => {
                    if build.baseline_text == build.current_text {
                        DecisionStatus::Unchanged
                    } else {
                        DecisionStatus::Modified
                    }
                }
                DecisionUnitKind::Deleted => DecisionStatus::Deleted,
                DecisionUnitKind::Added => DecisionStatus::Added,
                DecisionUnitKind::Coalesced => {
                    if build.baseline_text == build.current_text {
                        // Finalized below once slot positions are known:
                        // Unchanged (same relational position) or Moved
                        // (displaced). Moved is the safe interim value.
                        DecisionStatus::Moved
                    } else {
                        DecisionStatus::Modified
                    }
                }
            };
            let byte_equal = build.baseline_text == build.current_text;
            let differ =
                build.baseline_text.is_some() && build.current_text.is_some() && !byte_equal;
            let (is_whitespace_change, is_usfm_structure_change) = if differ {
                classify_text_diff(
                    build.baseline_text.as_deref().unwrap_or(""),
                    build.current_text.as_deref().unwrap_or(""),
                )
            } else {
                (false, false)
            };
            let relabeled = matches!(build.kind, DecisionUnitKind::Coalesced)
                && byte_equal
                && build.baseline_sid != build.current_sid;
            let dup_context = DupContext {
                baseline_count: *baseline_key_count.get(&build.key).unwrap_or(&0),
                current_count: *current_key_count.get(&build.key).unwrap_or(&0),
            };

            DecisionUnit {
                id,
                kind: build.kind,
                status,
                baseline_sid: build.baseline_sid,
                current_sid: build.current_sid,
                baseline_tokens: build.baseline_tokens,
                current_tokens: build.current_tokens,
                displaced: false,
                relabeled,
                dup_context,
                covered_by: None,
                is_whitespace_change,
                is_usfm_structure_change,
            }
        })
        .collect();

    // Build the skeleton slot sequence by walking the interleaved steps.
    let mut slots: Vec<Slot> = Vec::with_capacity(steps.len());
    for step in &steps {
        let (unit_index, role) = match *step {
            Step::Shared(pb, _pc) => (unit_for_baseline_block[&pb], SlotRole::Shared),
            Step::BaselineOnly(bi) => {
                let unit_index = unit_for_baseline_block[&bi];
                let role = if matches!(units[unit_index].kind, DecisionUnitKind::Coalesced) {
                    SlotRole::PairBaseline
                } else {
                    SlotRole::BaselineOnly
                };
                (unit_index, role)
            }
            Step::CurrentOnly(ci) => {
                let unit_index = unit_for_current_block[&ci];
                let role = if matches!(units[unit_index].kind, DecisionUnitKind::Coalesced) {
                    SlotRole::PairCurrent
                } else {
                    SlotRole::CurrentOnly
                };
                (unit_index, role)
            }
        };
        slots.push(Slot {
            unit_id: unit_ids[unit_index].clone(),
            role,
            after: None,
        });
    }

    finalize_displacement_and_status(&mut units, &unit_ids, &slots);
    finalize_covered_by(&mut units, &unit_ids);
    finalize_anchors(&mut slots, &units, &unit_ids);

    DiffSkeleton { slots, units }
}

/// A coalesced pair is displaced iff its current slot is before its baseline
/// slot, or at least one shared slot lies strictly between its two slots.
/// One-sided Added/Deleted slots between the pair do not count. Finalizes
/// `Unchanged`/`Moved` for exact (byte-equal) coalesced pairs once slot
/// positions are known.
fn finalize_displacement_and_status<T>(
    units: &mut [DecisionUnit<T>],
    unit_ids: &[UnitId],
    slots: &[Slot],
) {
    let mut baseline_slot_index: FxHashMap<&UnitId, usize> = FxHashMap::default();
    let mut current_slot_index: FxHashMap<&UnitId, usize> = FxHashMap::default();
    for (index, slot) in slots.iter().enumerate() {
        match slot.role {
            SlotRole::PairBaseline => {
                baseline_slot_index.insert(&slot.unit_id, index);
            }
            SlotRole::PairCurrent => {
                current_slot_index.insert(&slot.unit_id, index);
            }
            _ => {}
        }
    }

    for (unit, id) in units.iter_mut().zip(unit_ids) {
        if !matches!(unit.kind, DecisionUnitKind::Coalesced) {
            continue;
        }
        let Some(&base_index) = baseline_slot_index.get(id) else {
            continue;
        };
        let Some(&cur_index) = current_slot_index.get(id) else {
            continue;
        };
        let lo = base_index.min(cur_index);
        let hi = base_index.max(cur_index);
        let shared_between = slots[lo + 1..hi]
            .iter()
            .any(|slot| matches!(slot.role, SlotRole::Shared));
        let displaced = cur_index < base_index || shared_between;
        unit.displaced = displaced;

        let byte_equal = unit.baseline_tokens_equal_current();
        if byte_equal {
            unit.status = if displaced {
                DecisionStatus::Moved
            } else {
                DecisionStatus::Unchanged
            };
        }
        // status stays Modified for a byte-different coalesced pair
        // regardless of displacement — displaced is still surfaced.
    }
}

impl<T> DecisionUnit<T> {
    /// A coalesced pair's byte-equality was already computed once from block
    /// text at build time and encoded via the interim `Moved` placeholder
    /// status (see `build_skeleton`); a `Modified`/`Unchanged`/`Moved` status
    /// at this point (before displacement finalization can flip
    /// Moved<->Unchanged) is exactly the byte-equal signal.
    fn baseline_tokens_equal_current(&self) -> bool {
        !matches!(self.status, DecisionStatus::Modified)
    }
}

fn finalize_covered_by<T>(units: &mut [DecisionUnit<T>], unit_ids: &[UnitId]) {
    let snapshots: Vec<(UnitId, DecisionUnitKind, Option<String>, Option<String>)> = units
        .iter()
        .zip(unit_ids)
        .map(|(unit, id)| {
            (
                id.clone(),
                unit.kind,
                unit.baseline_sid.clone(),
                unit.current_sid.clone(),
            )
        })
        .collect();

    for unit in units.iter_mut() {
        if !matches!(
            unit.kind,
            DecisionUnitKind::Deleted | DecisionUnitKind::Added
        ) {
            continue;
        }
        let own_sid = if matches!(unit.kind, DecisionUnitKind::Deleted) {
            unit.baseline_sid.as_deref()
        } else {
            unit.current_sid.as_deref()
        };
        let Some(own_sid) = own_sid else { continue };
        let Some((own_book, own_chapter, own_start, own_end)) = sid_range(own_sid) else {
            continue;
        };
        if own_start == 0 {
            continue;
        }
        // covered_by is only for a one-sided *verse* (singular, never a
        // bridge) overlapped by a true multi-verse range on the paired
        // unit's opposite side. A one-sided bridge is its own range event,
        // not something a neighboring bridge "covers".
        if own_start != own_end {
            continue;
        }

        for (candidate_id, kind, baseline_sid, current_sid) in &snapshots {
            if !matches!(kind, DecisionUnitKind::Coalesced) {
                continue;
            }
            let cover_sid = if matches!(unit.kind, DecisionUnitKind::Deleted) {
                current_sid.as_deref()
            } else {
                baseline_sid.as_deref()
            };
            let Some(cover_sid) = cover_sid else { continue };
            let Some((cover_book, cover_chapter, cover_start, cover_end)) = sid_range(cover_sid)
            else {
                continue;
            };
            if cover_end <= cover_start || cover_book != own_book || cover_chapter != own_chapter {
                continue;
            }
            if own_start <= cover_end && own_end >= cover_start {
                unit.covered_by = Some(CoveredBy {
                    unit_id: candidate_id.clone(),
                    sid: cover_sid.to_string(),
                    side: if matches!(unit.kind, DecisionUnitKind::Deleted) {
                        CoveredSide::Current
                    } else {
                        CoveredSide::Baseline
                    },
                });
                break;
            }
        }
    }
}

/// Every slot's `after` is the nearest preceding shared-or-paired anchor at
/// that slot's side/position. Added/Deleted slots never become anchors.
fn finalize_anchors<T>(slots: &mut [Slot], units: &[DecisionUnit<T>], unit_ids: &[UnitId]) {
    let mut unit_index_by_id: FxHashMap<&UnitId, usize> = FxHashMap::default();
    for (index, id) in unit_ids.iter().enumerate() {
        unit_index_by_id.insert(id, index);
    }

    let mut last_anchor: Option<Anchor> = None;
    for slot in slots.iter_mut() {
        slot.after = last_anchor.clone();
        let is_anchor_slot = matches!(
            slot.role,
            SlotRole::Shared | SlotRole::PairBaseline | SlotRole::PairCurrent
        );
        if !is_anchor_slot {
            continue;
        }
        let unit = &units[unit_index_by_id[&slot.unit_id]];
        let side_sid = match slot.role {
            SlotRole::PairBaseline => unit.baseline_sid.clone(),
            SlotRole::PairCurrent | SlotRole::Shared => unit.current_sid.clone(),
            _ => unreachable!(),
        };
        if let Some(sid) = side_sid {
            last_anchor = Some(Anchor {
                unit_id: slot.unit_id.clone(),
                sid,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use crate::token::Token;

    fn skeleton_for<'a>(baseline: &'a str, current: &'a str) -> DiffSkeleton<Token<'a>> {
        let baseline_parsed = parse(baseline);
        let current_parsed = parse(current);
        diff_skeleton_canonical(
            &baseline_parsed.tokens,
            "GEN",
            &current_parsed.tokens,
            "GEN",
        )
    }

    fn wrapped(body: &str) -> String {
        format!("\\id GEN\n{body}")
    }

    fn baseline_bearing(slot: &Slot) -> bool {
        matches!(
            slot.role,
            SlotRole::Shared | SlotRole::BaselineOnly | SlotRole::PairBaseline
        )
    }

    fn current_bearing(slot: &Slot) -> bool {
        matches!(
            slot.role,
            SlotRole::Shared | SlotRole::CurrentOnly | SlotRole::PairCurrent
        )
    }

    #[test]
    fn every_baseline_and_current_block_appears_in_exactly_one_bearing_slot() {
        let baseline = wrapped("\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n");
        let current = wrapped("\\c 1\n\\v 1 one\n\\v 3 three\n\\v 2 two\n");
        let skeleton = skeleton_for(&baseline, &current);

        let baseline_parsed = parse(&baseline);
        let current_parsed = parse(&current);
        let expected_baseline_blocks =
            super::super::build_sid_blocks_canonical(&baseline_parsed.tokens, "GEN").len();
        let expected_current_blocks =
            super::super::build_sid_blocks_canonical(&current_parsed.tokens, "GEN").len();

        assert_eq!(
            skeleton
                .slots
                .iter()
                .filter(|slot| baseline_bearing(slot))
                .count(),
            expected_baseline_blocks
        );
        assert_eq!(
            skeleton
                .slots
                .iter()
                .filter(|slot| current_bearing(slot))
                .count(),
            expected_current_blocks
        );
    }

    #[test]
    fn every_coalesced_unit_owns_exactly_one_pair_baseline_and_one_pair_current_slot() {
        // Case 10: pure reorder / swap — byte-identical, must surface as a
        // moved coalesced pair, not vanish.
        let baseline = wrapped("\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n");
        let current = wrapped("\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n");
        let skeleton = skeleton_for(&baseline, &current);

        let coalesced_ids: Vec<&UnitId> = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Coalesced))
            .map(|unit| &unit.id)
            .collect();
        assert_eq!(
            coalesced_ids.len(),
            1,
            "expected exactly one coalesced unit for a pure swap"
        );

        for id in coalesced_ids {
            let pair_base_count = skeleton
                .slots
                .iter()
                .filter(|slot| slot.unit_id == *id && matches!(slot.role, SlotRole::PairBaseline))
                .count();
            let pair_cur_count = skeleton
                .slots
                .iter()
                .filter(|slot| slot.unit_id == *id && matches!(slot.role, SlotRole::PairCurrent))
                .count();
            assert_eq!(pair_base_count, 1);
            assert_eq!(pair_cur_count, 1);
        }
    }

    #[test]
    fn case_13_pairs_exact_text_before_positional_leftovers() {
        // Duplicate: first occurrence deleted. Survivor 'c' must exact-pair
        // (byte-identical) rather than mis-pairing positionally with 'a'.
        let baseline = wrapped("\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n");
        let current = wrapped("\\c 1\n\\v 2 b\n\\v 1 c\n");
        let skeleton = skeleton_for(&baseline, &current);

        let coalesced: Vec<_> = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Coalesced))
            .collect();
        assert_eq!(
            coalesced.len(),
            1,
            "expected exactly one coalesced pair (the 'c' survivor)"
        );
        let block_text = |tokens: &[Token<'_>]| tokens.iter().map(|t| t.text()).collect::<String>();
        assert!(block_text(&coalesced[0].baseline_tokens).ends_with("c\n"));
        assert!(block_text(&coalesced[0].current_tokens).ends_with("c\n"));

        let deleted: Vec<_> = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Deleted))
            .collect();
        assert_eq!(
            deleted.len(),
            1,
            "expected exactly one deleted unit (the 'a' content)"
        );
        assert!(block_text(&deleted[0].baseline_tokens).ends_with("a\n"));
    }

    #[test]
    fn case_23_renumber_typo_never_coalesces_across_keys() {
        // Content-similarity across DIFFERENT verse numbers must never pair —
        // a renumber typo is delete+add, not a move.
        let baseline = wrapped("\\c 1\n\\v 1 a\n\\v 2 b\n\\v 10 j\n\\v 11 k\n");
        let current = wrapped("\\c 1\n\\v 1 a\n\\v 2 b\n\\v 10 j\n\\v 1 k\n");
        let skeleton = skeleton_for(&baseline, &current);

        let coalesced_count = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Coalesced))
            .count();
        assert_eq!(
            coalesced_count, 0,
            "renumbered content must never coalesce across keys"
        );

        let deleted: Vec<_> = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Deleted))
            .collect();
        let added: Vec<_> = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Added))
            .collect();
        assert_eq!(deleted.len(), 1);
        assert_eq!(added.len(), 1);
        let block_text = |tokens: &[Token<'_>]| tokens.iter().map(|t| t.text()).collect::<String>();
        assert!(block_text(&deleted[0].baseline_tokens).ends_with("k\n"));
        assert!(block_text(&added[0].current_tokens).ends_with("k\n"));
    }

    #[test]
    fn case_11_three_verses_one_displaced_has_exactly_one_moved_unit() {
        let baseline = wrapped("\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n");
        let current = wrapped("\\c 1\n\\v 1 one\n\\v 3 three\n\\v 2 two\n");
        let skeleton = skeleton_for(&baseline, &current);

        let coalesced_count = skeleton
            .units
            .iter()
            .filter(|unit| matches!(unit.kind, DecisionUnitKind::Coalesced))
            .count();
        assert_eq!(
            coalesced_count, 1,
            "LCS keeps two verses; only the odd one out coalesces"
        );
    }

    #[test]
    fn diff_skeleton_is_deterministic_across_repeated_builds() {
        let baseline = wrapped("\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n");
        let current = wrapped("\\c 1\n\\v 2 b\n\\v 1 c\n");

        let first = skeleton_for(&baseline, &current);
        let second = skeleton_for(&baseline, &current);

        let ids = |skeleton: &DiffSkeleton<Token<'_>>| {
            skeleton
                .units
                .iter()
                .map(|unit| unit.id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&first), ids(&second));

        let slot_shape = |skeleton: &DiffSkeleton<Token<'_>>| {
            skeleton
                .slots
                .iter()
                .map(|slot| (slot.unit_id.clone(), slot.role))
                .collect::<Vec<_>>()
        };
        assert_eq!(slot_shape(&first), slot_shape(&second));
    }
}

/// The parallelized by-chapter diffs must produce a map byte-identical to the
/// serial reference below over diverse, real diff pairs. A per-chapter diff has
/// no independent oracle, so this equality *is* the correctness gate for the
/// parallelization: the reference reproduces the pre-parallel nested-loop
/// exactly, and every cell must match it. Native-only — on wasm the seam is
/// already serial, so there is nothing to compare against.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod parallel_tests {
    use super::*;
    use crate::parse::parse;
    use crate::token::Token;
    use std::path::{Path, PathBuf};

    /// Serial reference for [`diff_skeleton_by_chapter`]: the original
    /// nested-loop grouping, kept as a fixed baseline independent of the
    /// parallel seam.
    fn serial_by_chapter_canonical<'a>(
        baseline_usfm: &'a str,
        current_usfm: &'a str,
    ) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<Token<'a>>>> {
        let baseline = parse(baseline_usfm);
        let current = parse(current_usfm);
        let baseline_groups = group_tokens_by_book_and_chapter(
            &baseline.tokens,
            baseline.analysis.book_code.unwrap_or("unknown"),
        );
        let current_groups = group_tokens_by_book_and_chapter(
            &current.tokens,
            current.analysis.book_code.unwrap_or("unknown"),
        );
        serial_grouped(baseline_groups, current_groups, |book, b, c| {
            diff_skeleton_canonical(b, book, c, book)
        })
    }

    /// Serial reference for [`diff_skeleton_by_chapter_from_tokens`].
    fn serial_by_chapter_from_tokens<'a>(
        baseline_tokens: &[Token<'a>],
        current_tokens: &[Token<'a>],
    ) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<Token<'a>>>> {
        let baseline_groups = group_tokens_by_book_and_chapter(baseline_tokens, "unknown");
        let current_groups = group_tokens_by_book_and_chapter(current_tokens, "unknown");
        serial_grouped(baseline_groups, current_groups, |_book, b, c| {
            diff_skeleton(b, c)
        })
    }

    /// The pre-parallel grouping loop, verbatim: build each book's chapter map
    /// serially and insert it. Deliberately does not call `diff_grouped_by_chapter`
    /// so it stays an independent oracle for the flatten/rebuild refactor.
    fn serial_grouped<T, F>(
        baseline_groups: BTreeMap<String, BTreeMap<u32, Vec<T>>>,
        current_groups: BTreeMap<String, BTreeMap<u32, Vec<T>>>,
        diff_cell: F,
    ) -> BTreeMap<String, BTreeMap<u32, DiffSkeleton<T>>>
    where
        T: DiffableToken,
        F: Fn(&str, &[T], &[T]) -> DiffSkeleton<T>,
    {
        let mut all_books = BTreeSet::<String>::new();
        all_books.extend(baseline_groups.keys().cloned());
        all_books.extend(current_groups.keys().cloned());

        let mut out = BTreeMap::new();
        for book in all_books {
            let mut all_chapters = BTreeSet::<u32>::new();
            if let Some(chapters) = baseline_groups.get(&book) {
                all_chapters.extend(chapters.keys().copied());
            }
            if let Some(chapters) = current_groups.get(&book) {
                all_chapters.extend(chapters.keys().copied());
            }
            let mut book_map = BTreeMap::new();
            for chapter in all_chapters {
                let baseline_slice = baseline_groups
                    .get(&book)
                    .and_then(|chapters| chapters.get(&chapter))
                    .cloned()
                    .unwrap_or_default();
                let current_slice = current_groups
                    .get(&book)
                    .and_then(|chapters| chapters.get(&chapter))
                    .cloned()
                    .unwrap_or_default();
                book_map.insert(chapter, diff_cell(&book, &baseline_slice, &current_slice));
            }
            out.insert(book, book_map);
        }
        out
    }

    /// Assert two by-chapter maps are structurally equal, naming the exact cell
    /// that diverges rather than dumping a whole skeleton on failure.
    fn assert_maps_equal(
        got: &BTreeMap<String, BTreeMap<u32, DiffSkeleton<Token<'_>>>>,
        want: &BTreeMap<String, BTreeMap<u32, DiffSkeleton<Token<'_>>>>,
        label: &str,
    ) {
        let got_books: Vec<&String> = got.keys().collect();
        let want_books: Vec<&String> = want.keys().collect();
        assert_eq!(got_books, want_books, "book set differs for {label}");
        for (book, got_chapters) in got {
            let want_chapters = &want[book];
            let got_ch: Vec<&u32> = got_chapters.keys().collect();
            let want_ch: Vec<&u32> = want_chapters.keys().collect();
            assert_eq!(
                got_ch, want_ch,
                "chapter set differs for {label} book {book}"
            );
            for (chapter, got_skeleton) in got_chapters {
                let want_skeleton = &want_chapters[chapter];
                assert!(
                    got_skeleton == want_skeleton,
                    "diff skeleton differs for {label} at {book} {chapter}"
                );
            }
        }
    }

    /// Every pair is asserted twice: the string entry point (canonical sids,
    /// parses internally) and the token entry point (carried sids, generic `T`).
    fn assert_pair_matches(baseline: &str, current: &str, label: &str) {
        assert_maps_equal(
            &diff_skeleton_by_chapter(baseline, current),
            &serial_by_chapter_canonical(baseline, current),
            &format!("{label} (canonical)"),
        );

        let baseline_parsed = parse(baseline);
        let current_parsed = parse(current);
        assert_maps_equal(
            &diff_skeleton_by_chapter_from_tokens(&baseline_parsed.tokens, &current_parsed.tokens),
            &serial_by_chapter_from_tokens(&baseline_parsed.tokens, &current_parsed.tokens),
            &format!("{label} (from_tokens)"),
        );
    }

    // Line-based mutators. They need not produce meaningful edits — the gate is
    // parallel == serial over whatever diff they induce — only a realistically
    // varied one: an edited verse, an inserted/deleted verse, and an
    // inserted/deleted whole chapter.

    fn edit_a_verse(src: &str) -> String {
        let mut out = String::with_capacity(src.len() + 16);
        let mut done = false;
        for line in src.split_inclusive('\n') {
            if !done && line.trim_start().starts_with("\\v ") {
                out.push_str(line.trim_end_matches('\n'));
                out.push_str(" edited-content");
                if line.ends_with('\n') {
                    out.push('\n');
                }
                done = true;
            } else {
                out.push_str(line);
            }
        }
        out
    }

    fn insert_a_verse(src: &str) -> String {
        let mut out = String::with_capacity(src.len() + 32);
        let mut done = false;
        for line in src.split_inclusive('\n') {
            if !done && line.trim_start().starts_with("\\v ") {
                out.push_str("\\v 249 inserted verse text\n");
                done = true;
            }
            out.push_str(line);
        }
        out
    }

    fn delete_a_verse(src: &str) -> String {
        let mut out = String::with_capacity(src.len());
        let mut done = false;
        for line in src.split_inclusive('\n') {
            if !done && line.trim_start().starts_with("\\v ") {
                done = true;
                continue;
            }
            out.push_str(line);
        }
        out
    }

    fn insert_a_chapter(src: &str) -> String {
        format!("{src}\n\\c 250\n\\p\n\\v 1 inserted chapter verse one\n\\v 2 and two\n")
    }

    /// Drop a whole chapter: the run from the second `\c ` line up to (not
    /// including) the third, or the first-to-second when there are only two.
    fn delete_a_chapter(src: &str) -> String {
        let chapter_line_indices: Vec<usize> = src
            .split_inclusive('\n')
            .enumerate()
            .filter(|(_, line)| line.trim_start().starts_with("\\c "))
            .map(|(index, _)| index)
            .collect();
        if chapter_line_indices.len() < 2 {
            return src.to_string();
        }
        let (drop_from, drop_to) = if chapter_line_indices.len() >= 3 {
            (chapter_line_indices[1], chapter_line_indices[2])
        } else {
            (chapter_line_indices[0], chapter_line_indices[1])
        };
        src.split_inclusive('\n')
            .enumerate()
            .filter(|(index, _)| *index < drop_from || *index >= drop_to)
            .map(|(_, line)| line)
            .collect()
    }

    fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_usfm(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("usfm") {
                out.push(path);
            }
        }
    }

    /// A spread across both corpora plus the large GEN book (263k tokens, well
    /// over `PARALLEL_MIN_TOKENS`), so the parallel branch is exercised, not
    /// just the small-input serial fallback.
    fn sample_sources() -> Vec<(String, String)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut paths = Vec::new();
        collect_usfm(&root.join("testData"), &mut paths);
        collect_usfm(&root.join("example-corpora"), &mut paths);
        paths.sort();
        assert!(!paths.is_empty(), "expected corpus fixtures");

        // Stride to keep the run bounded, then force-include the large GEN book.
        let stride = (paths.len() / 20).max(1);
        let mut selected: Vec<PathBuf> = paths.iter().step_by(stride).cloned().collect();
        let gen_book = root.join("example-corpora/en_ult/01-GEN.usfm");
        if gen_book.exists() && !selected.contains(&gen_book) {
            selected.push(gen_book);
        }

        selected
            .iter()
            .filter_map(|path| {
                std::fs::read_to_string(path)
                    .ok()
                    .map(|source| (path.to_string_lossy().into_owned(), source))
            })
            .collect()
    }

    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn parallel_by_chapter_matches_serial_over_diverse_pairs() {
        let sources = sample_sources();
        assert!(sources.len() >= 2, "need at least two fixtures to pair");

        for (label, source) in &sources {
            // Identity (all-equal) plus one mutation of every shape.
            assert_pair_matches(source, source, &format!("{label} self"));
            assert_pair_matches(
                source,
                &edit_a_verse(source),
                &format!("{label} edit-verse"),
            );
            assert_pair_matches(
                source,
                &insert_a_verse(source),
                &format!("{label} insert-verse"),
            );
            assert_pair_matches(
                source,
                &delete_a_verse(source),
                &format!("{label} delete-verse"),
            );
            assert_pair_matches(
                source,
                &insert_a_chapter(source),
                &format!("{label} insert-chapter"),
            );
            assert_pair_matches(
                source,
                &delete_a_chapter(source),
                &format!("{label} delete-chapter"),
            );
        }

        // Cross-book pairs (a file vs a different book): all-added/all-deleted
        // across two book keys.
        for window in sources.windows(2) {
            let (left_label, left) = &window[0];
            let (right_label, right) = &window[1];
            assert_pair_matches(left, right, &format!("{left_label} vs {right_label}"));
        }
    }
}
