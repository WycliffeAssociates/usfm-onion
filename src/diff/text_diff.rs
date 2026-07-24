//! Intra-unit text diff (word/char runs) — presentation metadata layered on
//! top of an already-built [`DecisionUnit`]. Pure and additive: reads only
//! `baseline_tokens`/`current_tokens` on a unit, never the skeleton, slots,
//! ids, or decisions, and never mutates anything. See
//! `plans/approved/plan-intra-unit-text-diff-2026-07-20.md` for the design.

use serde::Serialize;
use similar::{ChangeTag, TextDiff};

use super::skeleton::{DecisionStatus, DecisionUnit, DiffSkeleton, diff_skeleton};
use super::DiffableToken;

/// Requested granularity for the intra-unit text diff. `None` computes
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum TextDiffMode {
    #[default]
    None,
    /// `similar::TextDiff::from_unicode_words` (UAX #29 word boundaries).
    Words,
    /// `similar::TextDiff::from_graphemes` (grapheme clusters).
    Chars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TextDiffRunKind {
    Unchanged,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextDiffRun {
    pub text: String,
    pub kind: TextDiffRunKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnitTextDiff {
    /// Kinds: `Unchanged` | `Removed`.
    pub baseline: Vec<TextDiffRun>,
    /// Kinds: `Unchanged` | `Added`.
    pub current: Vec<TextDiffRun>,
}

/// Pure. Reads only the unit's token slices; never mutates, never consults
/// the skeleton, slots, ids, or decisions. Returns `None` for
/// [`TextDiffMode::None`] and for `Unchanged`/`Moved` (byte-equal) units.
/// Deterministic.
pub fn unit_text_diff<T: DiffableToken>(
    unit: &DecisionUnit<T>,
    mode: TextDiffMode,
) -> Option<UnitTextDiff> {
    if mode == TextDiffMode::None {
        return None;
    }

    match unit.status {
        DecisionStatus::Unchanged | DecisionStatus::Moved => None,
        DecisionStatus::Added => {
            let text = unit_plain_text(&unit.current_tokens);
            let current = single_run(text, TextDiffRunKind::Added);
            Some(UnitTextDiff {
                baseline: Vec::new(),
                current,
            })
        }
        DecisionStatus::Deleted => {
            let text = unit_plain_text(&unit.baseline_tokens);
            let baseline = single_run(text, TextDiffRunKind::Removed);
            Some(UnitTextDiff {
                baseline,
                current: Vec::new(),
            })
        }
        DecisionStatus::Modified => {
            let baseline_text = unit_plain_text(&unit.baseline_tokens);
            let current_text = unit_plain_text(&unit.current_tokens);
            Some(split_runs(&baseline_text, &current_text, mode))
        }
    }
}

/// Native convenience sugar: one call that diffs and computes the per-unit
/// text runs, returned index-aligned with `skeleton.units`. This is how a
/// native caller avoids writing the loop — WITHOUT a `text_diff` field on
/// `DecisionUnit<T>`; the word diff rides *beside* the unit, so the core
/// type, the fixtures, and merge/revert stay byte-identical. Serial (no
/// rayon) — v1, per the plan's spike-first rule.
pub fn diff_skeleton_with_text<T: DiffableToken>(
    baseline: &[T],
    current: &[T],
    mode: TextDiffMode,
) -> (DiffSkeleton<T>, Vec<Option<UnitTextDiff>>) {
    let skeleton = diff_skeleton(baseline, current);
    let texts = skeleton
        .units
        .iter()
        .map(|unit| unit_text_diff(unit, mode))
        .collect();
    (skeleton, texts)
}

/// A single contiguous run for `Added`/`Deleted` units — matches print's
/// current `buildSides` shape. Empty plain text yields an empty vec rather
/// than a zero-length run.
fn single_run(text: String, kind: TextDiffRunKind) -> Vec<TextDiffRun> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![TextDiffRun { text, kind }]
    }
}

/// Concatenates the `.text()` of tokens whose `kind_key()` is one of the
/// content/whitespace-bearing kinds, in stream order. Markers, end markers,
/// milestones, numbers, and book codes are excluded — this is reader-visible
/// plain text only.
fn unit_plain_text<T: DiffableToken>(tokens: &[T]) -> String {
    tokens
        .iter()
        .filter(|token| {
            matches!(
                token.kind_key(),
                Some("text") | Some("verticalWhitespace") | Some("optBreak")
            )
        })
        .map(|token| token.text())
        .collect()
}

/// Runs `similar`'s Unicode-aware text diff over both plain texts, mapping
/// `Equal`→Unchanged (both arrays), `Delete`→Removed (baseline only), and
/// `Insert`→Added (current only), coalescing consecutive same-kind segments
/// into one run per side.
fn split_runs(baseline_text: &str, current_text: &str, mode: TextDiffMode) -> UnitTextDiff {
    let diff = match mode {
        TextDiffMode::Words => TextDiff::from_unicode_words(baseline_text, current_text),
        TextDiffMode::Chars => TextDiff::from_graphemes(baseline_text, current_text),
        TextDiffMode::None => unreachable!("caller returns early for TextDiffMode::None"),
    };

    let mut baseline_runs = Vec::new();
    let mut current_runs = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change.as_str().unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        match change.tag() {
            ChangeTag::Equal => {
                push_coalesced(&mut baseline_runs, text, TextDiffRunKind::Unchanged);
                push_coalesced(&mut current_runs, text, TextDiffRunKind::Unchanged);
            }
            ChangeTag::Delete => push_coalesced(&mut baseline_runs, text, TextDiffRunKind::Removed),
            ChangeTag::Insert => push_coalesced(&mut current_runs, text, TextDiffRunKind::Added),
        }
    }

    UnitTextDiff {
        baseline: baseline_runs,
        current: current_runs,
    }
}

fn push_coalesced(runs: &mut Vec<TextDiffRun>, text: &str, kind: TextDiffRunKind) {
    if let Some(last) = runs.last_mut()
        && last.kind == kind
    {
        last.text.push_str(text);
        return;
    }
    runs.push(TextDiffRun {
        text: text.to_string(),
        kind,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::skeleton::{DupContext, UnitId};
    use crate::parse::parse;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestToken {
        text: String,
        kind_key: Option<&'static str>,
    }

    impl DiffableToken for TestToken {
        fn text(&self) -> &str {
            &self.text
        }

        fn kind_key(&self) -> Option<&str> {
            self.kind_key
        }
    }

    fn tok(text: &str, kind_key: &'static str) -> TestToken {
        TestToken {
            text: text.to_string(),
            kind_key: Some(kind_key),
        }
    }

    fn unit(
        status: DecisionStatus,
        baseline_tokens: Vec<TestToken>,
        current_tokens: Vec<TestToken>,
    ) -> DecisionUnit<TestToken> {
        DecisionUnit {
            id: UnitId::new("u1"),
            kind: crate::diff::skeleton::DecisionUnitKind::Shared,
            status,
            baseline_sid: None,
            current_sid: None,
            baseline_tokens,
            current_tokens,
            displaced: false,
            relabeled: false,
            dup_context: DupContext::default(),
            covered_by: None,
            is_whitespace_change: false,
            is_usfm_structure_change: false,
        }
    }

    // --- plain-text extraction ---

    #[test]
    fn plain_text_excludes_markers_numbers_and_book_codes() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 Alpha\n");
        let text = unit_plain_text(&parsed.tokens);
        // The `\id`, `\c`, `\v` markers, the "GEN" book code, and the "1"
        // chapter/verse numbers are all excluded — only the newline
        // separators and the reader-visible "Alpha" text remain.
        assert_eq!(text, "\n\nAlpha\n");
        assert!(!text.contains("GEN"));
        assert!(!text.contains('1'));
    }

    #[test]
    fn plain_text_keeps_text_newline_and_opt_break() {
        let tokens = vec![
            tok("word", "text"),
            tok("\n", "verticalWhitespace"),
            tok("\u{2060}", "optBreak"),
            tok("more", "text"),
        ];
        assert_eq!(unit_plain_text(&tokens), "word\n\u{2060}more");
    }

    #[test]
    fn add_wrapped_text_is_not_glued_with_a_synthetic_space() {
        // `word\add ed\add*` — the marker/end-marker tokens are excluded, and
        // no space is inserted between "word" and "ed": the character-marker
        // wrap must read as "worded", not "word ed".
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 word\\add ed\\add*\n");
        let text = unit_plain_text(&parsed.tokens);
        assert!(
            text.contains("worded"),
            "expected \"worded\" glued with no space, got {text:?}"
        );
        assert!(!text.contains("word ed"));
    }

    // --- status gating ---

    #[test]
    fn unchanged_status_returns_none() {
        let u = unit(DecisionStatus::Unchanged, vec![tok("a", "text")], vec![tok("a", "text")]);
        assert_eq!(unit_text_diff(&u, TextDiffMode::Words), None);
    }

    #[test]
    fn moved_status_returns_none() {
        let u = unit(DecisionStatus::Moved, vec![tok("a", "text")], vec![tok("a", "text")]);
        assert_eq!(unit_text_diff(&u, TextDiffMode::Words), None);
    }

    #[test]
    fn none_mode_returns_none_regardless_of_status() {
        let u = unit(
            DecisionStatus::Modified,
            vec![tok("heaven", "text")],
            vec![tok("heavens", "text")],
        );
        assert_eq!(unit_text_diff(&u, TextDiffMode::None), None);
    }

    #[test]
    fn added_status_is_a_single_added_run() {
        let u = unit(DecisionStatus::Added, vec![], vec![tok("new text", "text")]);
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Added yields Some");
        assert_eq!(diff.baseline, vec![]);
        assert_eq!(
            diff.current,
            vec![TextDiffRun {
                text: "new text".to_string(),
                kind: TextDiffRunKind::Added
            }]
        );
    }

    #[test]
    fn added_status_with_empty_current_text_is_empty_current_vec() {
        let u = unit(DecisionStatus::Added, vec![], vec![tok("\\v", "marker")]);
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Added yields Some");
        assert_eq!(diff.baseline, vec![]);
        assert_eq!(diff.current, vec![]);
    }

    #[test]
    fn deleted_status_is_a_single_removed_run() {
        let u = unit(DecisionStatus::Deleted, vec![tok("old text", "text")], vec![]);
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Deleted yields Some");
        assert_eq!(
            diff.baseline,
            vec![TextDiffRun {
                text: "old text".to_string(),
                kind: TextDiffRunKind::Removed
            }]
        );
        assert_eq!(diff.current, vec![]);
    }

    #[test]
    fn modified_status_produces_word_runs() {
        let u = unit(
            DecisionStatus::Modified,
            vec![tok("heaven", "text")],
            vec![tok("heavens", "text")],
        );
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Modified yields Some");
        assert_eq!(
            diff.baseline,
            vec![TextDiffRun {
                text: "heaven".to_string(),
                kind: TextDiffRunKind::Removed
            }]
        );
        assert_eq!(
            diff.current,
            vec![TextDiffRun {
                text: "heavens".to_string(),
                kind: TextDiffRunKind::Added
            }]
        );
    }

    #[test]
    fn modified_status_produces_unchanged_runs_around_a_changed_word() {
        let u = unit(
            DecisionStatus::Modified,
            vec![tok("the quick fox", "text")],
            vec![tok("the slow fox", "text")],
        );
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Modified yields Some");
        // Only Unchanged/Removed on the baseline side, only Unchanged/Added
        // on the current side.
        assert!(diff
            .baseline
            .iter()
            .all(|r| matches!(r.kind, TextDiffRunKind::Unchanged | TextDiffRunKind::Removed)));
        assert!(diff
            .current
            .iter()
            .all(|r| matches!(r.kind, TextDiffRunKind::Unchanged | TextDiffRunKind::Added)));
        let baseline_text: String = diff.baseline.iter().map(|r| r.text.as_str()).collect();
        let current_text: String = diff.current.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(baseline_text, "the quick fox");
        assert_eq!(current_text, "the slow fox");
    }

    // --- reconstruction & alignment invariants ---

    #[test]
    fn reconstruction_matches_plain_text_on_both_sides() {
        let baseline_tokens = vec![tok("the the the", "text")];
        let current_tokens = vec![tok("the the", "text")];
        let u = unit(
            DecisionStatus::Modified,
            baseline_tokens.clone(),
            current_tokens.clone(),
        );
        for mode in [TextDiffMode::Words, TextDiffMode::Chars] {
            let diff = unit_text_diff(&u, mode).expect("Modified yields Some");
            let baseline_concat: String = diff.baseline.iter().map(|r| r.text.as_str()).collect();
            let current_concat: String = diff.current.iter().map(|r| r.text.as_str()).collect();
            assert_eq!(baseline_concat, unit_plain_text(&baseline_tokens));
            assert_eq!(current_concat, unit_plain_text(&current_tokens));
        }
    }

    #[test]
    fn unchanged_run_texts_match_across_both_arrays() {
        let u = unit(
            DecisionStatus::Modified,
            vec![tok("the quick fox", "text")],
            vec![tok("the slow fox", "text")],
        );
        let diff = unit_text_diff(&u, TextDiffMode::Words).expect("Modified yields Some");
        let baseline_unchanged: Vec<&str> = diff
            .baseline
            .iter()
            .filter(|r| r.kind == TextDiffRunKind::Unchanged)
            .map(|r| r.text.as_str())
            .collect();
        let current_unchanged: Vec<&str> = diff
            .current
            .iter()
            .filter(|r| r.kind == TextDiffRunKind::Unchanged)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(baseline_unchanged, current_unchanged);
    }

    #[test]
    fn is_idempotent_and_deterministic() {
        let u = unit(
            DecisionStatus::Modified,
            vec![tok("heaven and earth", "text")],
            vec![tok("heavens and earths", "text")],
        );
        let first = unit_text_diff(&u, TextDiffMode::Words);
        let second = unit_text_diff(&u, TextDiffMode::Words);
        assert_eq!(first, second);
    }

    // --- diff_skeleton_with_text convenience ---

    #[test]
    fn diff_skeleton_with_text_is_index_aligned_with_units() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n";
        let baseline_tokens = parse(baseline).tokens;
        let current_tokens = parse(current).tokens;
        let (skeleton, texts) =
            diff_skeleton_with_text(&baseline_tokens, &current_tokens, TextDiffMode::Words);
        assert_eq!(skeleton.units.len(), texts.len());
        let modified_index = skeleton
            .units
            .iter()
            .position(|u| u.status == DecisionStatus::Modified)
            .expect("one Modified unit");
        assert!(texts[modified_index].is_some());
    }
}
