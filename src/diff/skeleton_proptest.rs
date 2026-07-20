//! Hardened property suite for the merge-projection skeleton: a structured
//! chapter/document model rendered to valid USFM, typed edit scripts
//! deriving Current from Baseline, and P1-P7 properties. Shrinking operates
//! on the structured model/edit list (never on raw byte offsets), so a
//! shrunk failure keeps its defining shape.

#![cfg(test)]

use std::collections::BTreeMap;

use proptest::prelude::*;

use crate::diff::DiffableToken;
use crate::diff::skeleton::{
    DecisionStatus, DecisionUnit, DecisionUnitKind, DiffSkeleton, MergeError, MergeSide, SlotRole,
    UnitId, diff_skeleton_canonical, merge_skeleton,
};
use crate::parse::parse;
use crate::token::Token;

const WORDS: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
];
const HEADINGS: &[&str] = &["Section One", "Section Two", "Section Three"];
const BOOK: &str = "GEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Eol {
    Lf,
    CrLf,
}

impl Eol {
    fn as_str(self) -> &'static str {
        match self {
            Eol::Lf => "\n",
            Eol::CrLf => "\r\n",
        }
    }
}

#[derive(Debug, Clone)]
struct Verse {
    number: u32,
    end: Option<u32>,
    words: Vec<String>,
    wrapped: bool,
    trailing_extra_spaces: usize,
}

#[derive(Debug, Clone)]
struct Chapter {
    number: u32,
    open_marker: &'static str,
    heading: Option<String>,
    verses: Vec<Verse>,
}

#[derive(Debug, Clone)]
struct Doc {
    eol: Eol,
    chapters: Vec<Chapter>,
}

impl Doc {
    fn render(&self) -> String {
        let nl = self.eol.as_str();
        let mut out = format!("\\id {BOOK}{nl}");
        for chapter in &self.chapters {
            out.push_str(&format!("\\c {}{nl}", chapter.number));
            out.push_str(&format!("\\{}{nl}", chapter.open_marker));
            if let Some(heading) = &chapter.heading {
                out.push_str(&format!("\\s {heading}{nl}"));
            }
            for verse in &chapter.verses {
                let vref = match verse.end {
                    Some(end) => format!("{}-{}", verse.number, end),
                    None => verse.number.to_string(),
                };
                let text = verse.words.join(" ");
                let extra = " ".repeat(verse.trailing_extra_spaces);
                if verse.wrapped {
                    out.push_str(&format!("\\v {vref} \\add {text}\\add*{extra}{nl}"));
                } else {
                    out.push_str(&format!("\\v {vref} {text}{extra}{nl}"));
                }
            }
        }
        out
    }
}

fn word_strategy() -> impl Strategy<Value = String> {
    (0usize..WORDS.len()).prop_map(|i| WORDS[i].to_string())
}

fn eol_strategy() -> impl Strategy<Value = Eol> {
    prop_oneof![Just(Eol::Lf), Just(Eol::CrLf)]
}

/// `bridge_width` of 0 is a single verse; 1 or 2 is a bridge spanning that
/// many extra verse numbers.
/// `(bridge_width, word_indices, wrapped, trailing_extra_spaces)`.
type VerseShape = (u32, Vec<usize>, bool, usize);

fn verse_shape_strategy() -> impl Strategy<Value = VerseShape> {
    (
        0u32..=2,
        prop::collection::vec(0usize..WORDS.len(), 1..=3),
        prop::bool::ANY,
        0usize..=2,
    )
}

fn build_verses(shapes: Vec<VerseShape>) -> Vec<Verse> {
    let mut verses = Vec::new();
    let mut next = 1u32;
    for (bridge_width, word_indices, wrapped, extra_spaces) in shapes {
        let start = next;
        let end = if bridge_width > 0 {
            Some(start + bridge_width)
        } else {
            None
        };
        next = end.unwrap_or(start) + 1;
        verses.push(Verse {
            number: start,
            end,
            words: word_indices.iter().map(|&i| WORDS[i].to_string()).collect(),
            wrapped,
            trailing_extra_spaces: extra_spaces,
        });
    }
    verses
}

fn chapter_shape_strategy() -> impl Strategy<Value = (bool, Option<usize>, Vec<VerseShape>)> {
    (
        prop::bool::ANY,
        prop::option::of(0usize..HEADINGS.len()),
        prop::collection::vec(verse_shape_strategy(), 1..=5),
    )
}

/// General strategy: a bounded, structured document — always a valid `\id`,
/// 1-2 chapters, verse singles/ranges, text atoms, balanced `\add` wraps,
/// paragraph/heading variation, whitespace, and LF/CRLF.
fn doc_strategy() -> impl Strategy<Value = Doc> {
    (
        eol_strategy(),
        prop::collection::vec(chapter_shape_strategy(), 1..=2),
    )
        .prop_map(|(eol, chapter_shapes)| {
            let chapters = chapter_shapes
                .into_iter()
                .enumerate()
                .map(|(index, (use_p, heading_index, verse_shapes))| Chapter {
                    number: index as u32 + 1,
                    open_marker: if use_p { "p" } else { "m" },
                    heading: heading_index.map(|i| HEADINGS[i].to_string()),
                    verses: build_verses(verse_shapes),
                })
                .collect();
            Doc { eol, chapters }
        })
}

#[derive(Debug, Clone)]
enum Edit {
    AppendLetter {
        chapter: usize,
        verse: usize,
    },
    InsertVerse {
        chapter: usize,
        at: usize,
        word: String,
    },
    DeleteVerse {
        chapter: usize,
        at: usize,
    },
    Reorder {
        chapter: usize,
        from: usize,
        to: usize,
    },
    BoundaryMigration {
        chapter: usize,
        verse: usize,
    },
    ParagraphChange {
        chapter: usize,
    },
    WhitespaceOnly {
        chapter: usize,
        verse: usize,
    },
    MarkerWrapToggle {
        chapter: usize,
        verse: usize,
    },
}

fn edit_strategy() -> impl Strategy<Value = Edit> {
    prop_oneof![
        (0usize..2, 0usize..5).prop_map(|(chapter, verse)| Edit::AppendLetter { chapter, verse }),
        (0usize..2, 0usize..5, word_strategy()).prop_map(|(chapter, at, word)| Edit::InsertVerse {
            chapter,
            at,
            word
        }),
        (0usize..2, 0usize..5).prop_map(|(chapter, at)| Edit::DeleteVerse { chapter, at }),
        (0usize..2, 0usize..5, 0usize..5).prop_map(|(chapter, from, to)| Edit::Reorder {
            chapter,
            from,
            to
        }),
        (0usize..2, 0usize..5)
            .prop_map(|(chapter, verse)| Edit::BoundaryMigration { chapter, verse }),
        (0usize..2).prop_map(|chapter| Edit::ParagraphChange { chapter }),
        (0usize..2, 0usize..5).prop_map(|(chapter, verse)| Edit::WhitespaceOnly { chapter, verse }),
        (0usize..2, 0usize..5)
            .prop_map(|(chapter, verse)| Edit::MarkerWrapToggle { chapter, verse }),
    ]
}

fn apply_edit(doc: &mut Doc, edit: &Edit) {
    match edit {
        Edit::AppendLetter { chapter, verse } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && let Some(verse) = chapter.verses.get_mut(*verse)
                && let Some(first) = verse.words.first_mut()
            {
                first.push('Z');
            }
        }
        Edit::InsertVerse { chapter, at, word } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter) {
                let at = (*at).min(chapter.verses.len());
                let number = if at == 0 {
                    1
                } else {
                    chapter.verses[at - 1]
                        .end
                        .unwrap_or(chapter.verses[at - 1].number)
                        + 1
                };
                chapter.verses.insert(
                    at,
                    Verse {
                        number,
                        end: None,
                        words: vec![word.clone()],
                        wrapped: false,
                        trailing_extra_spaces: 0,
                    },
                );
                renumber_from(chapter, at + 1);
            }
        }
        Edit::DeleteVerse { chapter, at } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && *at < chapter.verses.len()
                && chapter.verses.len() > 1
            {
                chapter.verses.remove(*at);
            }
        }
        Edit::Reorder { chapter, from, to } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && !chapter.verses.is_empty()
            {
                let from = (*from).min(chapter.verses.len() - 1);
                let to = (*to).min(chapter.verses.len() - 1);
                if from != to {
                    let verse = chapter.verses.remove(from);
                    chapter.verses.insert(to, verse);
                }
            }
        }
        Edit::BoundaryMigration { chapter, verse } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && *verse + 1 < chapter.verses.len()
            {
                let moved = chapter.verses[*verse].words.pop();
                if let Some(word) = moved
                    && !chapter.verses[*verse].words.is_empty()
                {
                    chapter.verses[*verse + 1].words.insert(0, word);
                } else if let Some(word) = chapter.verses[*verse].words.pop() {
                    // never leave a verse with zero words
                    chapter.verses[*verse].words.push(word);
                }
            }
        }
        Edit::ParagraphChange { chapter } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter) {
                chapter.open_marker = if chapter.open_marker == "p" { "m" } else { "p" };
            }
        }
        Edit::WhitespaceOnly { chapter, verse } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && let Some(verse) = chapter.verses.get_mut(*verse)
            {
                verse.trailing_extra_spaces += 2;
            }
        }
        Edit::MarkerWrapToggle { chapter, verse } => {
            if let Some(chapter) = doc.chapters.get_mut(*chapter)
                && let Some(verse) = chapter.verses.get_mut(*verse)
            {
                verse.wrapped = !verse.wrapped;
            }
        }
    }
}

fn renumber_from(chapter: &mut Chapter, start_index: usize) {
    let mut next = if start_index == 0 {
        1
    } else {
        let prev = &chapter.verses[start_index - 1];
        prev.end.unwrap_or(prev.number) + 1
    };
    for verse in chapter.verses.iter_mut().skip(start_index) {
        let width = verse.end.map(|end| end - verse.number).unwrap_or(0);
        verse.number = next;
        verse.end = if width > 0 { Some(next + width) } else { None };
        next = verse.end.unwrap_or(verse.number) + 1;
    }
}

fn edited_doc_strategy() -> impl Strategy<Value = (Doc, Doc)> {
    (
        doc_strategy(),
        prop::collection::vec(edit_strategy(), 0..=3),
    )
        .prop_map(|(baseline, edits)| {
            let mut current = baseline.clone();
            for edit in &edits {
                apply_edit(&mut current, edit);
            }
            (baseline, current)
        })
}

fn skeleton_of<'a>(baseline: &'a str, current: &'a str) -> DiffSkeleton<Token<'a>> {
    let baseline_parsed = parse(baseline);
    let current_parsed = parse(current);
    diff_skeleton_canonical(&baseline_parsed.tokens, BOOK, &current_parsed.tokens, BOOK)
}

fn block_text<T: DiffableToken>(tokens: &[T]) -> String {
    tokens.iter().map(|t| t.text()).collect()
}

/// P1: side-specific block/slot reconstruction is exact and no block
/// contributes twice.
fn assert_p1_partition<T: DiffableToken>(
    skeleton: &DiffSkeleton<T>,
    baseline_src: &str,
    current_src: &str,
) {
    let mut baseline_out = String::new();
    let mut current_out = String::new();
    let mut seen_baseline = std::collections::HashSet::new();
    let mut seen_current = std::collections::HashSet::new();

    for slot in &skeleton.slots {
        let unit = skeleton
            .units
            .iter()
            .find(|u| u.id == slot.unit_id)
            .unwrap();
        match slot.role {
            SlotRole::Shared => {
                assert!(seen_baseline.insert(slot.unit_id.clone()));
                assert!(seen_current.insert(slot.unit_id.clone()));
                baseline_out.push_str(&block_text(&unit.baseline_tokens));
                current_out.push_str(&block_text(&unit.current_tokens));
            }
            SlotRole::BaselineOnly | SlotRole::PairBaseline => {
                assert!(seen_baseline.insert(slot.unit_id.clone()));
                baseline_out.push_str(&block_text(&unit.baseline_tokens));
            }
            SlotRole::CurrentOnly | SlotRole::PairCurrent => {
                assert!(seen_current.insert(slot.unit_id.clone()));
                current_out.push_str(&block_text(&unit.current_tokens));
            }
        }
    }

    assert_eq!(baseline_out, baseline_src);
    assert_eq!(current_out, current_src);
}

/// P2: all-Baseline == baseline and all-Current == current, byte-for-byte.
fn assert_p2_identities<T: DiffableToken>(
    skeleton: &DiffSkeleton<T>,
    baseline_src: &str,
    current_src: &str,
) {
    let all_baseline = merge_skeleton(skeleton, &BTreeMap::new(), MergeSide::Baseline).unwrap();
    let all_current = merge_skeleton(skeleton, &BTreeMap::new(), MergeSide::Current).unwrap();
    assert_eq!(block_text(&all_baseline), baseline_src);
    assert_eq!(block_text(&all_current), current_src);
}

/// Independently walks the skeleton's slots (the same contract
/// `merge_skeleton` implements, but written separately here so a bug in the
/// real implementation — wrong side, a duplicated slot, a leaked unchosen
/// side — shows up as a byte mismatch against real output, not just as a
/// self-consistent recount of the same bookkeeping the implementation used).
fn expected_merge_text<T: DiffableToken>(
    skeleton: &DiffSkeleton<T>,
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) -> String {
    let units_by_id: std::collections::HashMap<&UnitId, &DecisionUnit<T>> =
        skeleton.units.iter().map(|unit| (&unit.id, unit)).collect();
    let mut out = String::new();
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
                out.push_str(&block_text(tokens));
            }
            SlotRole::BaselineOnly | SlotRole::PairBaseline => {
                if side == MergeSide::Baseline {
                    out.push_str(&block_text(&unit.baseline_tokens));
                }
            }
            SlotRole::CurrentOnly | SlotRole::PairCurrent => {
                if side == MergeSide::Current {
                    out.push_str(&block_text(&unit.current_tokens));
                }
            }
        }
    }
    out
}

/// P6's chosen-side/no-leakage witness: actually calls `merge_skeleton` and
/// compares its real output against the independently-assembled expected
/// text above — byte-exact equality is what proves no unchosen-side token
/// leaked in and nothing double-contributed (a leak or a duplicate would
/// make the strings differ), not merely that the slot bookkeeping is
/// internally consistent with itself.
fn assert_merge_output_matches_expected<T: DiffableToken>(
    skeleton: &DiffSkeleton<T>,
    decisions: &BTreeMap<UnitId, MergeSide>,
    default_side: MergeSide,
) {
    let actual = merge_skeleton(skeleton, decisions, default_side).unwrap();
    let expected = expected_merge_text(skeleton, decisions, default_side);
    assert_eq!(
        block_text(&actual),
        expected,
        "real merge_skeleton output must equal the independently assembled chosen-side text"
    );
}

fn decisions_strategy(
    unit_ids: Vec<UnitId>,
) -> impl Strategy<Value = (BTreeMap<UnitId, MergeSide>, MergeSide)> {
    let per_unit = prop::collection::vec(0u8..3, unit_ids.len());
    (per_unit, prop::bool::ANY).prop_map(move |(choices, default_is_baseline)| {
        let mut decisions = BTreeMap::new();
        for (id, choice) in unit_ids.iter().zip(choices) {
            match choice {
                0 => {
                    decisions.insert(id.clone(), MergeSide::Baseline);
                }
                1 => {
                    decisions.insert(id.clone(), MergeSide::Current);
                }
                _ => {}
            }
        }
        let default_side = if default_is_baseline {
            MergeSide::Baseline
        } else {
            MergeSide::Current
        };
        (decisions, default_side)
    })
}

proptest! {
    // proptest's default (256) — measured at well under a second for this
    // whole module, so there is no cost reason to run fewer.
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn p1_p2_partition_and_identity((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let skeleton = skeleton_of(&baseline_src, &current_src);
        assert_p1_partition(&skeleton, &baseline_src, &current_src);
        assert_p2_identities(&skeleton, &baseline_src, &current_src);
    }

    #[test]
    fn p3_totality_and_reparse_fixed_point(
        (baseline_src, current_src, decisions, default_side) in edited_doc_strategy().prop_flat_map(|(b, c)| {
            let baseline_src = b.render();
            let current_src = c.render();
            let ids = skeleton_of(&baseline_src, &current_src)
                .units
                .iter()
                .map(|u| u.id.clone())
                .collect::<Vec<_>>();
            decisions_strategy(ids)
                .prop_map(move |(decisions, default_side)| {
                    (baseline_src.clone(), current_src.clone(), decisions, default_side)
                })
        })
    ) {
        let skeleton = skeleton_of(&baseline_src, &current_src);

        let merged = merge_skeleton(&skeleton, &decisions, default_side);
        prop_assert!(merged.is_ok(), "a valid decision vector must always return");
        let merged_text = block_text(&merged.unwrap());

        // serialize -> parse -> serialize is a fixed point.
        let reparsed = parse(&merged_text);
        let reserialized = crate::parse::into_usfm_from_tokens(&reparsed.tokens);
        prop_assert_eq!(reserialized, merged_text);
    }

    #[test]
    fn p4_purity_across_repeated_and_interleaved_calls(
        (baseline, current) in edited_doc_strategy(),
    ) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let skeleton = skeleton_of(&baseline_src, &current_src);

        let decisions_a: BTreeMap<UnitId, MergeSide> = BTreeMap::new();
        let decisions_b: BTreeMap<UnitId, MergeSide> = skeleton
            .units
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, u)| (u.id.clone(), MergeSide::Baseline))
            .collect();

        let first = merge_skeleton(&skeleton, &decisions_a, MergeSide::Current).unwrap();
        let _other = merge_skeleton(&skeleton, &decisions_b, MergeSide::Baseline).unwrap();
        let again = merge_skeleton(&skeleton, &decisions_a, MergeSide::Current).unwrap();

        prop_assert_eq!(block_text(&first), block_text(&again));
    }

    #[test]
    fn p5_skeleton_construction_is_fully_deterministic((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let first = skeleton_of(&baseline_src, &current_src);
        let second = skeleton_of(&baseline_src, &current_src);
        prop_assert!(first == second, "two builds from the same input must be fully equal");
    }

    #[test]
    fn p7_unknown_id_is_rejected_with_no_output((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let skeleton = skeleton_of(&baseline_src, &current_src);

        let guaranteed_absent = UnitId::new("__proptest_guaranteed_absent_unit__");
        let mut decisions = BTreeMap::new();
        decisions.insert(guaranteed_absent.clone(), MergeSide::Current);

        let result = merge_skeleton(&skeleton, &decisions, MergeSide::Current);
        prop_assert_eq!(result, Err(MergeError::UnknownUnitId(guaranteed_absent)));
    }
}

// ---------------------------------------------------------------------------
// P6: directed shapes. Each is its own property with a shape-specific
// generator — never a low-probability variant folded into the general
// strategy above. Each runs P1/P2/P4 plus the contribution/no-leakage
// witness for a handful of decision vectors.
// ---------------------------------------------------------------------------

fn wrap_id(body: String) -> String {
    format!("\\id {BOOK}\n{body}")
}

fn assert_directed_shape(
    baseline_body: String,
    current_body: String,
) -> DiffSkeleton<Token<'static>> {
    let baseline_src: &'static str = Box::leak(wrap_id(baseline_body).into_boxed_str());
    let current_src: &'static str = Box::leak(wrap_id(current_body).into_boxed_str());
    let skeleton = skeleton_of(baseline_src, current_src);
    assert_p1_partition(&skeleton, baseline_src, current_src);
    assert_p2_identities(&skeleton, baseline_src, current_src);

    // P4-shaped: a handful of decision vectors, each checked for idempotency
    // and real merged-output correctness.
    let unit_ids: Vec<UnitId> = skeleton.units.iter().map(|u| u.id.clone()).collect();
    for trial in 0..4u32 {
        let mut decisions = BTreeMap::new();
        for (index, id) in unit_ids.iter().enumerate() {
            match (trial as usize + index) % 3 {
                0 => {
                    decisions.insert(id.clone(), MergeSide::Baseline);
                }
                1 => {
                    decisions.insert(id.clone(), MergeSide::Current);
                }
                _ => {}
            }
        }
        let default_side = if trial % 2 == 0 {
            MergeSide::Baseline
        } else {
            MergeSide::Current
        };
        let first = merge_skeleton(&skeleton, &decisions, default_side).unwrap();
        let second = merge_skeleton(&skeleton, &decisions, default_side).unwrap();
        assert_eq!(block_text(&first), block_text(&second), "not idempotent");
        assert_merge_output_matches_expected(&skeleton, &decisions, default_side);
    }

    skeleton
}

proptest! {
    // 32, not the default 256: unlike the general suite above, every named
    // geometry in each directed property below now runs unconditionally on
    // every case (no shape is chosen at random and skipped) — so raising
    // the case count would only add more word-content variety on top of an
    // already-guaranteed-covered set of shapes, not more coverage. Runtime
    // for this whole module is well under a second either way.
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// 1. Adjacent deletion + two-block boundary migration: v2 is deleted
    /// while the last word of v1 migrates to the front of v3.
    #[test]
    fn directed_1_adjacent_deletion_and_boundary_migration(w1 in word_strategy(), w2 in word_strategy(), w3 in word_strategy(), w4 in word_strategy()) {
        let baseline = format!("\\c 1\n\\p\n\\v 1 {w1} {w2}\n\\v 2 {w3}\n\\v 3 {w4}\n");
        let current = format!("\\c 1\n\\p\n\\v 1 {w1}\n\\v 3 {w2} {w4}\n");
        assert_directed_shape(baseline, current);
    }

    /// 2. Boundary migration under every mixed Baseline/Current choice on the
    /// two touched units — the engine must total (never error/panic) even
    /// when a decision splits the migrated word across both sides.
    #[test]
    fn directed_2_boundary_migration_under_every_mixed_choice(w1 in word_strategy(), w2 in word_strategy(), w3 in word_strategy()) {
        let baseline_body = format!("\\c 1\n\\p\n\\v 1 {w1} {w2}\n\\v 2 {w3}\n");
        let current_body = format!("\\c 1\n\\p\n\\v 1 {w1}\n\\v 2 {w2} {w3}\n");
        // Runs the full P1/P2/P4 package (partition, identities, idempotency
        // + real-output verification) before this shape's own distinctive
        // check: every one of the 4 mixed Baseline/Current combinations on
        // the two touched units.
        let skeleton = assert_directed_shape(baseline_body, current_body);

        let v1_id = skeleton.units.iter().find(|u| u.baseline_sid.as_deref() == Some("GEN 1:1")).unwrap().id.clone();
        let v2_id = skeleton.units.iter().find(|u| u.baseline_sid.as_deref() == Some("GEN 1:2")).unwrap().id.clone();

        for &v1_side in &[MergeSide::Baseline, MergeSide::Current] {
            for &v2_side in &[MergeSide::Baseline, MergeSide::Current] {
                let mut decisions = BTreeMap::new();
                decisions.insert(v1_id.clone(), v1_side);
                decisions.insert(v2_id.clone(), v2_side);
                let result = merge_skeleton(&skeleton, &decisions, MergeSide::Current);
                prop_assert!(result.is_ok(), "mixed decision must total, never error");
                assert_merge_output_matches_expected(&skeleton, &decisions, MergeSide::Current);
            }
        }
    }

    /// 3. Swap/reorder and three-item transposition. Both named geometries
    /// run unconditionally every case (not one chosen at random by a bool) —
    /// a generator is not coverage of a shape unless it actually runs the
    /// shape-specific check every time.
    #[test]
    fn directed_3_reorder_and_transposition(w1 in word_strategy(), w2 in word_strategy(), w3 in word_strategy()) {
        let baseline = format!("\\c 1\n\\p\n\\v 1 {w1}\n\\v 2 {w2}\n\\v 3 {w3}\n");

        // swap: v2/v3 trade places, v1 stays put.
        let swap_current = format!("\\c 1\n\\p\n\\v 1 {w1}\n\\v 3 {w3}\n\\v 2 {w2}\n");
        assert_directed_shape(baseline.clone(), swap_current);

        // full three-item transposition.
        let transposition_current = format!("\\c 1\n\\p\n\\v 3 {w3}\n\\v 1 {w1}\n\\v 2 {w2}\n");
        assert_directed_shape(baseline, transposition_current);
    }

    /// 4. Duplicate sids on the baseline side; current deletes the first
    /// occurrence so the survivor renumbers from `_dup_1` to plain.
    #[test]
    fn directed_4_duplicate_sids_delete_first_occurrence(a in word_strategy(), b in word_strategy(), c in word_strategy()) {
        let baseline = format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 2 {b}\n\\v 1 {c}\n");
        let current = format!("\\c 1\n\\p\n\\v 2 {b}\n\\v 1 {c}\n");
        let skeleton = assert_directed_shape(baseline, current);

        let coalesced = skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Coalesced)).count();
        let deleted = skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Deleted)).count();
        prop_assert_eq!(coalesced, 1, "the survivor must coalesce (sid relabel only)");
        prop_assert_eq!(deleted, 1, "the first occurrence must be deleted");
    }

    /// 5. Bridge fuse-adjacency, merge-into-bridge, split, and overlap — all
    /// four named geometries run unconditionally every case (previously
    /// chosen at random by a `0u8..4` index, so a given run could miss one
    /// or more shapes entirely).
    #[test]
    fn directed_5_bridge_shapes(a in word_strategy(), b in word_strategy(), edit in word_strategy()) {
        // fuse-adjacency: \v 1 then \v 1-2 stay two blocks, never fused.
        assert_directed_shape(
            format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 1-2 {b}\n"),
            format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 1-2 {b} {edit}\n"),
        );

        // merge-into-bridge: v1 + v2 -> a 1-2 bridge.
        assert_directed_shape(
            format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 2 {b}\n"),
            format!("\\c 1\n\\p\n\\v 1-2 {a} {b}\n"),
        );

        // split: a 1-3 bridge -> 1-2 bridge + v3.
        assert_directed_shape(
            format!("\\c 1\n\\p\n\\v 1-3 {a}\n"),
            format!("\\c 1\n\\p\n\\v 1-2 {a}\n\\v 3 {edit}\n"),
        );

        // overlap: 1-2 bridge and v2 coexist; keys 1:1 vs 1:2 differ.
        assert_directed_shape(
            format!("\\c 1\n\\p\n\\v 1-2 {a}\n\\v 2 {b}\n"),
            format!("\\c 1\n\\p\n\\v 1-2 {a}\n\\v 2 {b} {edit}\n"),
        );
    }

    /// 6. Renumber typo: identical content under a different verse number
    /// must stay delete+add, never coalesce.
    #[test]
    fn directed_6_renumber_typo_never_coalesces(a in word_strategy(), b in word_strategy(), c in word_strategy(), moved in word_strategy()) {
        let baseline = format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 2 {b}\n\\v 10 {c}\n\\v 11 {moved}\n");
        let current = format!("\\c 1\n\\p\n\\v 1 {a}\n\\v 2 {b}\n\\v 10 {c}\n\\v 1 {moved}\n");
        let skeleton = assert_directed_shape(baseline, current);

        let coalesced = skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Coalesced)).count();
        prop_assert_eq!(coalesced, 0, "renumbered content must never coalesce across keys");
    }

    /// 7. Insertion at chapter start and end — both positions run
    /// unconditionally every case (previously chosen at random by a bool).
    #[test]
    fn directed_7_insertion_at_chapter_start_or_end(a in word_strategy(), b in word_strategy(), new_word in word_strategy()) {
        let baseline = format!("\\c 1\n\\p\n\\v 2 {a}\n\\v 3 {b}\n");

        let start_current = format!("\\c 1\n\\p\n\\v 1 {new_word}\n\\v 2 {a}\n\\v 3 {b}\n");
        let start_skeleton = assert_directed_shape(baseline.clone(), start_current);
        let start_added = start_skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Added)).count();
        prop_assert_eq!(start_added, 1);

        let end_current = format!("\\c 1\n\\p\n\\v 2 {a}\n\\v 3 {b}\n\\v 4 {new_word}\n");
        let end_skeleton = assert_directed_shape(baseline, end_current);
        let end_added = end_skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Added)).count();
        prop_assert_eq!(end_added, 1);
    }

    /// 8. Different-number back-to-back Added+Deleted must never coalesce;
    /// a same-number replacement must stay one Modified unit.
    #[test]
    fn directed_8_same_number_replace_vs_different_number_add_delete(x in word_strategy(), y in word_strategy()) {
        prop_assume!(x != y, "the replacement must actually change the text");
        let same_number_baseline = format!("\\c 1\n\\p\n\\v 5 {x}\n");
        let same_number_current = format!("\\c 1\n\\p\n\\v 5 {y}\n");
        let same_number_skeleton = assert_directed_shape(same_number_baseline, same_number_current);
        prop_assert_eq!(same_number_skeleton.units.len(), 3); // intro, chapter-open, v5
        let v5 = same_number_skeleton
            .units
            .iter()
            .find(|u| u.baseline_sid.as_deref() == Some("GEN 1:5"))
            .unwrap();
        prop_assert!(matches!(v5.kind, DecisionUnitKind::Shared));
        prop_assert_eq!(v5.status, DecisionStatus::Modified);

        let different_number_baseline = format!("\\c 1\n\\p\n\\v 5 {x}\n");
        let different_number_current = format!("\\c 1\n\\p\n\\v 7 {y}\n");
        let different_number_skeleton = assert_directed_shape(different_number_baseline, different_number_current);
        let coalesced = different_number_skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Coalesced)).count();
        let deleted = different_number_skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Deleted)).count();
        let added = different_number_skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Added)).count();
        prop_assert_eq!(coalesced, 0, "different verse numbers must never coalesce");
        prop_assert_eq!(deleted, 1);
        prop_assert_eq!(added, 1);
    }

    /// 9. Fully independent streams (no shared ancestry) with mixed EOL.
    #[test]
    fn directed_9_fully_independent_streams_with_crlf_and_lf(
        baseline_doc in doc_strategy(),
        current_doc in doc_strategy(),
        baseline_crlf in prop::bool::ANY,
    ) {
        let mut baseline_doc = baseline_doc;
        let mut current_doc = current_doc;
        baseline_doc.eol = if baseline_crlf { Eol::CrLf } else { Eol::Lf };
        current_doc.eol = if baseline_crlf { Eol::Lf } else { Eol::CrLf };
        // Force disjoint chapter numbers so there is genuinely no shared sid.
        for (index, chapter) in baseline_doc.chapters.iter_mut().enumerate() {
            chapter.number = index as u32 + 1;
        }
        for (index, chapter) in current_doc.chapters.iter_mut().enumerate() {
            chapter.number = index as u32 + 100;
        }

        let baseline_src = baseline_doc.render();
        let current_src = current_doc.render();
        let skeleton = skeleton_of(&baseline_src, &current_src);
        assert_p1_partition(&skeleton, &baseline_src, &current_src);
        assert_p2_identities(&skeleton, &baseline_src, &current_src);

        let coalesced = skeleton.units.iter().filter(|u| matches!(u.kind, DecisionUnitKind::Coalesced)).count();
        prop_assert_eq!(coalesced, 0, "no shared ancestry means nothing should coalesce");
    }
}
