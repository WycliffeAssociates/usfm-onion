//! Gate 3 of `plans/approved/plan-intra-unit-text-diff-2026-07-20.md`:
//! property tests for `unit_text_diff` / `diff_skeleton_with_text`. Reuses
//! the structured USFM generator (`edited_doc_strategy`, bumped to
//! `pub(super)` in `skeleton_proptest.rs` for this reuse) so the same
//! move/coalesce/whitespace/USFM-structure-churn shapes that exercise the
//! skeleton also exercise the text-diff layer riding beside it.
//!
//! Four properties, per the plan's Gate 3:
//! - P-text-1 reconstruction: run-text concatenation equals plain text.
//! - P-text-2 non-interference: the skeleton is bit-identical regardless of
//!   `TextDiffMode` — mode never enters the builder.
//! - P-text-3 merge unaffected: `merge_skeleton` output does not depend on
//!   whether text runs were requested.
//! - P-text-4 determinism: two `unit_text_diff` calls on the same unit+mode
//!   agree.

#![cfg(test)]

use std::collections::BTreeMap;

use proptest::prelude::*;

use super::skeleton_proptest::edited_doc_strategy;
use crate::diff::DiffableToken;
use crate::diff::skeleton::{MergeSide, UnitId, diff_skeleton, merge_skeleton};
use crate::diff::text_diff::{
    TextDiffMode, TextDiffRunKind, diff_skeleton_with_text, unit_text_diff,
};
use crate::parse::parse;
use crate::token::Token;

const MODES: [TextDiffMode; 2] = [TextDiffMode::Words, TextDiffMode::Chars];

fn block_text<T: DiffableToken>(tokens: &[T]) -> String {
    tokens.iter().map(|t| t.text()).collect()
}

/// Independent witness for `unit_text_diff`'s internal (private, so not
/// reachable from this file) `unit_plain_text` helper: the same three
/// content/whitespace-bearing kind keys the plan's plain-text contract names
/// (`text`, `verticalWhitespace`, `optBreak`). Kept separate from the
/// production helper so a bug in the real filter shows up as a mismatch here,
/// not as a self-consistent recount of the same logic.
fn expected_plain_text<T: DiffableToken>(tokens: &[T]) -> String {
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// P-text-1: for every unit and both `Words`/`Chars` modes, each side's
    /// run-text concatenation equals that side's plain text; baseline runs
    /// carry only Unchanged/Removed, current only Unchanged/Added; and the
    /// concatenated Unchanged *content* is identical across the two arrays.
    ///
    /// Note: the earlier "ordered Unchanged run *texts* are identical" clause
    /// was relaxed to compare concatenated content, not per-run segmentation.
    /// An unbalanced edit (a one-sided Delete/Insert between two Equal spans)
    /// makes per-side coalescing merge the Equals on the untouched side but not
    /// the other, so the run *boundaries* legitimately differ — while the
    /// unchanged content is identical. The consumer renders each side
    /// independently (plan decision #4) and the DecisionUnit is the alignment
    /// unit, not the intra-unit runs, so run-boundary parity is not required.
    #[test]
    fn p_text_1_reconstruction_and_kind_gating((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let baseline_tokens = parse(&baseline_src).tokens;
        let current_tokens = parse(&current_src).tokens;
        let skeleton = diff_skeleton(&baseline_tokens, &current_tokens);

        for unit in &skeleton.units {
            for mode in MODES {
                let Some(diff) = unit_text_diff(unit, mode) else { continue };

                prop_assert!(
                    diff.baseline.iter().all(|r| matches!(
                        r.kind,
                        TextDiffRunKind::Unchanged | TextDiffRunKind::Removed
                    )),
                    "baseline runs must only carry Unchanged/Removed"
                );
                prop_assert!(
                    diff.current.iter().all(|r| matches!(
                        r.kind,
                        TextDiffRunKind::Unchanged | TextDiffRunKind::Added
                    )),
                    "current runs must only carry Unchanged/Added"
                );

                let baseline_concat: String =
                    diff.baseline.iter().map(|r| r.text.as_str()).collect();
                let current_concat: String =
                    diff.current.iter().map(|r| r.text.as_str()).collect();
                prop_assert_eq!(&baseline_concat, &expected_plain_text(&unit.baseline_tokens));
                prop_assert_eq!(&current_concat, &expected_plain_text(&unit.current_tokens));

                // Unchanged *content* identical across sides (segmentation may differ).
                let baseline_unchanged: String = diff
                    .baseline
                    .iter()
                    .filter(|r| r.kind == TextDiffRunKind::Unchanged)
                    .map(|r| r.text.as_str())
                    .collect();
                let current_unchanged: String = diff
                    .current
                    .iter()
                    .filter(|r| r.kind == TextDiffRunKind::Unchanged)
                    .map(|r| r.text.as_str())
                    .collect();
                prop_assert_eq!(baseline_unchanged, current_unchanged);
            }
        }
    }

    /// P-text-2: the built `DiffSkeleton<T>` is bit-identical regardless of
    /// mode — mode never enters the builder. `diff_skeleton(b, c)` must equal
    /// the skeleton half of `diff_skeleton_with_text(b, c, mode)` for every
    /// mode, including `None`.
    #[test]
    fn p_text_2_non_interference((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let baseline_tokens: Vec<Token> = parse(&baseline_src).tokens;
        let current_tokens: Vec<Token> = parse(&current_src).tokens;

        let plain_skeleton = diff_skeleton(&baseline_tokens, &current_tokens);

        for mode in [TextDiffMode::None, TextDiffMode::Words, TextDiffMode::Chars] {
            let (skeleton_with_text, _texts) =
                diff_skeleton_with_text(&baseline_tokens, &current_tokens, mode);
            prop_assert_eq!(
                &plain_skeleton,
                &skeleton_with_text,
                "requesting text runs (mode = {:?}) must not perturb the skeleton",
                mode
            );
        }
    }

    /// P-text-3: `merge_skeleton` output is identical whether or not text
    /// runs were requested alongside the skeleton — trivial, since merge
    /// takes the skeleton, not the runs, but pinned per the plan.
    #[test]
    fn p_text_3_merge_unaffected_by_text_request((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let baseline_tokens: Vec<Token> = parse(&baseline_src).tokens;
        let current_tokens: Vec<Token> = parse(&current_src).tokens;

        let (skeleton_without_text, _) =
            diff_skeleton_with_text(&baseline_tokens, &current_tokens, TextDiffMode::None);
        let (skeleton_with_text, _) =
            diff_skeleton_with_text(&baseline_tokens, &current_tokens, TextDiffMode::Words);

        let unit_ids: Vec<UnitId> = skeleton_without_text.units.iter().map(|u| u.id.clone()).collect();
        let alternating: BTreeMap<UnitId, MergeSide> = unit_ids
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, id)| (id.clone(), MergeSide::Baseline))
            .collect();

        for decisions in [BTreeMap::new(), alternating] {
            for default_side in [MergeSide::Baseline, MergeSide::Current] {
                let merged_without_text =
                    merge_skeleton(&skeleton_without_text, &decisions, default_side).unwrap();
                let merged_with_text =
                    merge_skeleton(&skeleton_with_text, &decisions, default_side).unwrap();
                prop_assert_eq!(
                    block_text(&merged_without_text),
                    block_text(&merged_with_text),
                    "merge output must not depend on whether text runs were requested"
                );
            }
        }
    }

    /// P-text-4: two `unit_text_diff` calls on the same unit + mode agree.
    #[test]
    fn p_text_4_determinism((baseline, current) in edited_doc_strategy()) {
        let baseline_src = baseline.render();
        let current_src = current.render();
        let baseline_tokens = parse(&baseline_src).tokens;
        let current_tokens = parse(&current_src).tokens;
        let skeleton = diff_skeleton(&baseline_tokens, &current_tokens);

        for unit in &skeleton.units {
            for mode in MODES {
                let first = unit_text_diff(unit, mode);
                let second = unit_text_diff(unit, mode);
                prop_assert_eq!(first, second);
            }
        }
    }
}
