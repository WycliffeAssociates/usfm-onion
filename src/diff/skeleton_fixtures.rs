//! Fixture oracle: the 23-case / 21-pin catalog from
//! `../scripture-editor-proto-2/agent-tmp/prototypes/merge-interleave/cases.js`,
//! ported verbatim. Each case without its own `\id` is wrapped with a shared
//! `\id GEN\n` prefix before `parse()`; case 19 keeps its existing id line.
//! Run through both native parsed `Token`s and app-shaped `FormatToken`s with
//! normalized sids and deliberately different token ids, so the same
//! narration holds regardless of which token shape a caller diffs.

#[cfg(test)]
mod tests {
    use crate::diff::skeleton::{
        CoveredSide, DecisionStatus, DecisionUnitKind, DiffSkeleton, MergeError, MergeSide, UnitId,
        diff_skeleton, diff_skeleton_canonical, merge_diff_blocks, merge_skeleton,
        revert_diff_block,
    };
    use crate::diff::{DiffableToken, derive_canonical_sids};
    use crate::format::FormatToken;
    use crate::parse::parse;
    use crate::token::Token;
    use std::collections::BTreeMap;

    struct Case {
        n: u32,
        baseline: &'static str,
        current: &'static str,
    }

    const BOOK: &str = "GEN";

    fn wrap(n: u32, body: &str) -> String {
        if n == 19 {
            body.to_string()
        } else {
            format!("\\id {BOOK}\n{body}")
        }
    }

    const CASES: &[Case] = &[
        Case {
            n: 1,
            baseline: "\\c 1\n\\v 1 In the beginning God created the heaven and the earth.\n",
            current: "\\c 1\n\\v 1 In the beginning God created the heavens and the earth.\n",
        },
        Case {
            n: 2,
            baseline: "\\c 1\n\\v 1 one\n\\v 3 three\n",
            current: "\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n",
        },
        Case {
            n: 3,
            baseline: "\\c 1\n\\v 2 two\n\\v 3 three\n",
            current: "\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n",
        },
        Case {
            n: 4,
            baseline: "\\c 1\n\\v 1 one\n\\v 2 two\n",
            current: "\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n",
        },
        Case {
            n: 5,
            baseline: "\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n",
            current: "\\c 1\n\\v 1 one\n\\v 3 three\n",
        },
        Case {
            n: 6,
            baseline: "\\c 1\n\\v 1 The old rendering of this verse.\n",
            current: "\\c 1\n\\v 1 A completely different rendering here.\n",
        },
        Case {
            n: 7,
            baseline: "\\c 1\n\\v 1 one\n",
            current: "\\c 1\n\\v 1 one  \n",
        },
        Case {
            n: 9,
            baseline: "\\c 1\n\\v 1 Alpha beta gamma.\n\\v 2 Delta epsilon.\n",
            current: "\\c 1\n\\v 1 Alpha beta.\n\\v 2 Gamma delta epsilon.\n",
        },
        Case {
            n: 10,
            baseline: "\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n",
            current: "\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n",
        },
        Case {
            n: 11,
            baseline: "\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n",
            current: "\\c 1\n\\v 1 one\n\\v 3 three\n\\v 2 two\n",
        },
        Case {
            n: 12,
            baseline: "\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n",
            current: "\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c edited\n",
        },
        Case {
            n: 13,
            baseline: "\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n",
            current: "\\c 1\n\\v 2 b\n\\v 1 c\n",
        },
        Case {
            n: 14,
            baseline: "\\c 1\n\\v 1 a\n\\v 1-2 b\n",
            current: "\\c 1\n\\v 1 a\n\\v 1-2 b edited\n",
        },
        Case {
            n: 15,
            baseline: "\\c 1\n\\v 1 a\n\\v 2 b\n",
            current: "\\c 1\n\\v 1-2 a b\n",
        },
        Case {
            n: 16,
            baseline: "\\c 1\n\\v 1-3 a\n",
            current: "\\c 1\n\\v 1-2 a\n\\v 3 b\n",
        },
        Case {
            n: 17,
            baseline: "\\c 1\n\\v 1-2 a\n\\v 2 b\n",
            current: "\\c 1\n\\v 1-2 a\n\\v 2 b changed\n",
        },
        Case {
            n: 8,
            baseline: "\\c 1\n\\p\n\\v 1 one\n\\s Section\n\\v 2 two\n",
            current: "\\c 1\n\\m\n\\v 1 one\n\\v 2 two\n",
        },
        Case {
            n: 18,
            baseline: "\\c 1\n\\v 1 Text\\f + \\ft a note\\f* more.\n",
            current: "\\c 1\n\\v 1 Text\\f + \\ft an edited note\\f* more.\n",
        },
        Case {
            n: 19,
            baseline: "\\id GEN\n\\h Genesis\n\\c 1\n\\v 1 one\n",
            current: "\\id GEN\n\\h The Book of Genesis\n\\c 1\n\\v 1 one\n",
        },
        Case {
            n: 20,
            baseline: "\\c 1\r\n\\v 1 one\r\n\\v 2 two\r\n",
            current: "\\c 1\n\\v 1 one\n\\v 2 two\n",
        },
        Case {
            n: 21,
            baseline: "\\c 5\n\\v 10 Something entirely.\n\\v 11 Unrelated content here.\n",
            current: "\\c 9\n\\v 1 Totally different text.\n\\v 2 Nothing shared at all.\n",
        },
        Case {
            n: 22,
            baseline: "\\c 1\n\\v 1 Alpha beta gamma.\n\\v 2 To be deleted.\n\\v 3 Delta epsilon.\n",
            current: "\\c 1\n\\v 1 Alpha beta.\n\\v 3 Gamma delta epsilon.\n",
        },
        Case {
            n: 23,
            baseline: "\\c 1\n\\v 1 a\n\\v 2 b\n\\v 10 j\n\\v 11 k\n",
            current: "\\c 1\n\\v 1 a\n\\v 2 b\n\\v 10 j\n\\v 1 k\n",
        },
    ];

    fn block_text<T: DiffableToken>(tokens: &[T]) -> String {
        tokens.iter().map(|t| t.text()).collect()
    }

    /// P1/P2-shaped sanity check: every baseline token contributes to exactly
    /// one unit's baseline side (Shared/Deleted/Coalesced), every current
    /// token to exactly one unit's current side (Shared/Added/Coalesced), and
    /// concatenating those in slot order reproduces the source byte-for-byte.
    fn assert_partition_reproduces_source<T: DiffableToken>(
        skeleton: &DiffSkeleton<T>,
        baseline_src: &str,
        current_src: &str,
    ) {
        use crate::diff::skeleton::SlotRole;

        let mut baseline_out = String::new();
        let mut current_out = String::new();
        let mut seen_baseline_units = rustc_hash::FxHashSet::default();
        let mut seen_current_units = rustc_hash::FxHashSet::default();

        for slot in &skeleton.slots {
            let unit = skeleton
                .units
                .iter()
                .find(|u| u.id == slot.unit_id)
                .expect("slot references a real unit");
            match slot.role {
                SlotRole::Shared => {
                    baseline_out.push_str(&block_text(&unit.baseline_tokens));
                    current_out.push_str(&block_text(&unit.current_tokens));
                    assert!(seen_baseline_units.insert(slot.unit_id.clone()));
                    assert!(seen_current_units.insert(slot.unit_id.clone()));
                }
                SlotRole::BaselineOnly => {
                    baseline_out.push_str(&block_text(&unit.baseline_tokens));
                    assert!(seen_baseline_units.insert(slot.unit_id.clone()));
                }
                SlotRole::CurrentOnly => {
                    current_out.push_str(&block_text(&unit.current_tokens));
                    assert!(seen_current_units.insert(slot.unit_id.clone()));
                }
                SlotRole::PairBaseline => {
                    baseline_out.push_str(&block_text(&unit.baseline_tokens));
                    assert!(seen_baseline_units.insert(slot.unit_id.clone()));
                }
                SlotRole::PairCurrent => {
                    current_out.push_str(&block_text(&unit.current_tokens));
                    assert!(seen_current_units.insert(slot.unit_id.clone()));
                }
            }
        }

        assert_eq!(baseline_out, baseline_src, "baseline reassembly mismatch");
        assert_eq!(current_out, current_src, "current reassembly mismatch");
    }

    #[test]
    fn all_23_cases_partition_and_reassemble_on_native_parsed_tokens() {
        for case in CASES {
            let baseline_src = wrap(case.n, case.baseline);
            let current_src = wrap(case.n, case.current);
            let baseline = parse(&baseline_src);
            let current = parse(&current_src);
            let skeleton = diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK);
            assert_partition_reproduces_source(&skeleton, &baseline_src, &current_src);
        }
    }

    fn to_format_tokens_with_normalized_sids_and_drifted_ids<'a>(
        tokens: &[Token<'a>],
        book_code: &str,
        id_prefix: &str,
    ) -> Vec<FormatToken> {
        let canonical = derive_canonical_sids(tokens, book_code);
        tokens
            .iter()
            .zip(canonical)
            .enumerate()
            .map(|(index, (token, sid))| {
                let mut owned = FormatToken::from(token);
                owned.sid = Some(sid);
                owned.id = Some(format!("{id_prefix}-{index}"));
                owned
            })
            .collect()
    }

    #[test]
    fn all_23_cases_partition_and_reassemble_on_format_tokens_with_id_drift() {
        for case in CASES {
            let baseline_src = wrap(case.n, case.baseline);
            let current_src = wrap(case.n, case.current);
            let baseline = parse(&baseline_src);
            let current = parse(&current_src);
            let baseline_tokens = to_format_tokens_with_normalized_sids_and_drifted_ids(
                &baseline.tokens,
                BOOK,
                "orig",
            );
            let current_tokens =
                to_format_tokens_with_normalized_sids_and_drifted_ids(&current.tokens, BOOK, "new");
            let skeleton = diff_skeleton(&baseline_tokens, &current_tokens);
            assert_partition_reproduces_source(&skeleton, &baseline_src, &current_src);
        }
    }

    fn skeleton_for(case_n: u32) -> (DiffSkeleton<Token<'static>>, String, String) {
        let case = CASES.iter().find(|c| c.n == case_n).expect("case exists");
        let baseline_src: &'static str = Box::leak(wrap(case.n, case.baseline).into_boxed_str());
        let current_src: &'static str = Box::leak(wrap(case.n, case.current).into_boxed_str());
        let baseline = parse(baseline_src);
        let current = parse(current_src);
        let skeleton = diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK);
        (skeleton, baseline_src.to_string(), current_src.to_string())
    }

    #[test]
    fn case_7_whitespace_only_change_is_flagged_and_not_usfm_structure() {
        let (skeleton, ..) = skeleton_for(7);
        let modified = skeleton
            .units
            .iter()
            .find(|u| u.status == DecisionStatus::Modified)
            .expect("one modified unit");
        assert!(modified.is_whitespace_change);
        assert!(!modified.is_usfm_structure_change);
    }

    #[test]
    fn case_8_paragraph_marker_change_is_usfm_structure_only() {
        let (skeleton, ..) = skeleton_for(8);
        // The chapter-open block (\p -> \m) is a structure-only change.
        let chapter_open = skeleton
            .units
            .iter()
            .find(|u| u.baseline_sid.as_deref() == Some("GEN 1:0"))
            .expect("chapter-open unit");
        assert!(chapter_open.is_usfm_structure_change);
        assert!(!chapter_open.is_whitespace_change);
        // The v1 heading-removal unit is a real content change: neither flag.
        let verse_1 = skeleton
            .units
            .iter()
            .find(|u| u.baseline_sid.as_deref() == Some("GEN 1:1"))
            .expect("verse 1 unit");
        assert!(!verse_1.is_whitespace_change);
        assert!(!verse_1.is_usfm_structure_change);
    }

    #[test]
    fn case_10_moved_unit_spans_exactly_two_linked_slots_in_document_order() {
        use crate::diff::skeleton::SlotRole;
        let (skeleton, ..) = skeleton_for(10);

        let moved = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .expect("one coalesced (moved) unit");
        assert_eq!(moved.status, DecisionStatus::Moved);

        // Baseline-side cells read in baseline document order (intro,
        // chapter-open, v1, v2); current-side cells read in current document
        // order (intro, chapter-open, v2, v1).
        let baseline_order: Vec<Option<&str>> = skeleton
            .slots
            .iter()
            .filter(|s| matches!(s.role, SlotRole::Shared | SlotRole::PairBaseline))
            .map(|s| {
                skeleton
                    .units
                    .iter()
                    .find(|u| u.id == s.unit_id)
                    .unwrap()
                    .baseline_sid
                    .as_deref()
            })
            .collect();
        assert_eq!(
            baseline_order,
            vec![
                Some("GEN 0:0"),
                Some("GEN 1:0"),
                Some("GEN 1:1"),
                Some("GEN 1:2")
            ]
        );

        let current_order: Vec<Option<&str>> = skeleton
            .slots
            .iter()
            .filter(|s| matches!(s.role, SlotRole::Shared | SlotRole::PairCurrent))
            .map(|s| {
                skeleton
                    .units
                    .iter()
                    .find(|u| u.id == s.unit_id)
                    .unwrap()
                    .current_sid
                    .as_deref()
            })
            .collect();
        assert_eq!(
            current_order,
            vec![
                Some("GEN 0:0"),
                Some("GEN 1:0"),
                Some("GEN 1:2"),
                Some("GEN 1:1")
            ]
        );

        // The moved unit spans exactly two slots (one decision, two ghosts).
        let moved_slot_count = skeleton
            .slots
            .iter()
            .filter(|s| s.unit_id == moved.id)
            .count();
        assert_eq!(moved_slot_count, 2);
    }

    #[test]
    fn case_13_full_narration() {
        let (skeleton, ..) = skeleton_for(13);

        let deleted = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Deleted))
            .expect("one deleted unit");
        // 13.1: deleted unit is the 'a'-side content.
        assert!(block_text(&deleted.baseline_tokens).ends_with("a\n"));

        let survivor = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .expect("survivor 'c' coalesces");
        // 13.2: survivor 'c' pairs as unchanged (sid relabel only).
        assert_eq!(survivor.status, DecisionStatus::Unchanged);
        // 13.14: relabeled flag (GEN 1:1_dup_1 -> GEN 1:1).
        assert!(survivor.relabeled);
        assert_ne!(survivor.baseline_sid, survivor.current_sid);

        // 13.15: deleted 'a' carries dup context (key group 2x in baseline, 1x in current).
        assert_eq!(deleted.dup_context.baseline_count, 2);
        assert_eq!(deleted.dup_context.current_count, 1);

        // 13.3: deleted 'a' anchored after GEN 1:0 (between chapter open and v2).
        let deleted_slot = skeleton
            .slots
            .iter()
            .find(|s| s.unit_id == deleted.id)
            .unwrap();
        let anchor = deleted_slot
            .after
            .as_ref()
            .expect("deleted unit has an anchor");
        assert_eq!(anchor.sid, "GEN 1:0");

        // 13.4: no two hunks fight over one anchor — among units a downstream
        // hunk projection would actually anchor (Deleted, or a displaced
        // Coalesced pair — mirroring the prototype's `needsAnchor`), taken at
        // each unit's first slot (mirroring `slot.emit`), no two distinct
        // units share the same anchor sid.
        let mut seen_units = rustc_hash::FxHashSet::default();
        let mut anchored_sids = Vec::new();
        for slot in &skeleton.slots {
            if !seen_units.insert(slot.unit_id.clone()) {
                continue;
            }
            let unit = skeleton
                .units
                .iter()
                .find(|u| u.id == slot.unit_id)
                .unwrap();
            let needs_anchor = matches!(unit.status, DecisionStatus::Deleted)
                || (matches!(unit.kind, DecisionUnitKind::Coalesced) && unit.displaced);
            if needs_anchor && let Some(anchor) = &slot.after {
                anchored_sids.push(anchor.sid.clone());
            }
        }
        let unique_count: rustc_hash::FxHashSet<_> = anchored_sids.iter().collect();
        assert_eq!(
            unique_count.len(),
            anchored_sids.len(),
            "two anchored hunks share one anchor"
        );
    }

    #[test]
    fn case_15_verses_merged_into_a_bridge() {
        let (skeleton, ..) = skeleton_for(15);

        let pair = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .expect("one coalesced pair (1 -> 1-2)");
        // 15.1: pair exposes both sids.
        assert_eq!(pair.baseline_sid.as_deref(), Some("GEN 1:1"));
        assert_eq!(pair.current_sid.as_deref(), Some("GEN 1:1-2"));

        let deleted_v2 = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Deleted))
            .expect("deleted GEN 1:2");
        assert_eq!(deleted_v2.baseline_sid.as_deref(), Some("GEN 1:2"));

        // 15.2 + 15.3: deleted v2 anchored after the pair's baseline-side sid
        // GEN 1:1 (not GEN 1:0 — the shared chapter open) — the pair's
        // baseline slot precedes the deletion in slot order.
        let deleted_slot_index = skeleton
            .slots
            .iter()
            .position(|s| s.unit_id == deleted_v2.id)
            .unwrap();
        let anchor = skeleton.slots[deleted_slot_index]
            .after
            .as_ref()
            .expect("deleted v2 has an anchor");
        assert_eq!(anchor.sid, "GEN 1:1");
        assert_eq!(&anchor.unit_id, &pair.id);

        use crate::diff::skeleton::SlotRole;
        let pair_baseline_slot_index = skeleton
            .slots
            .iter()
            .position(|s| s.unit_id == pair.id && matches!(s.role, SlotRole::PairBaseline))
            .expect("pair has a baseline slot");
        assert!(
            pair_baseline_slot_index < deleted_slot_index,
            "pair must precede the deletion in document order"
        );

        // 15.4 (pin 20): deleted GEN 1:2 is coveredBy the pair's current 1-2 bridge.
        let covered = deleted_v2
            .covered_by
            .as_ref()
            .expect("deleted v2 is covered");
        assert_eq!(covered.unit_id, pair.id);
        assert_eq!(covered.side, CoveredSide::Current);
        assert_eq!(covered.sid, "GEN 1:1-2");
    }

    #[test]
    fn case_16_bridge_split_added_verse_is_covered_by_baseline_bridge() {
        let (skeleton, ..) = skeleton_for(16);

        let pair = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .expect("one coalesced pair (1-3 -> 1-2)");
        assert_eq!(pair.baseline_sid.as_deref(), Some("GEN 1:1-3"));
        assert_eq!(pair.current_sid.as_deref(), Some("GEN 1:1-2"));

        let added_v3 = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Added))
            .expect("added GEN 1:3");
        assert_eq!(added_v3.current_sid.as_deref(), Some("GEN 1:3"));

        // pin 21: added GEN 1:3 was coveredBy the pair's baseline 1-3 bridge.
        let covered = added_v3.covered_by.as_ref().expect("added v3 is covered");
        assert_eq!(covered.unit_id, pair.id);
        assert_eq!(covered.side, CoveredSide::Baseline);
        assert_eq!(covered.sid, "GEN 1:1-3");
    }

    #[test]
    fn covered_by_never_annotates_a_one_sided_bridge_against_an_opposite_bridge() {
        // Regression: baseline has two off-Myers bridges sharing pairing key
        // "1:1" (\v 1-3 and \v 1-2); current has one bridge \v 1-4. Tier 1
        // exact-text pairs \v 1-3 with \v 1-4 (byte-identical body), leaving
        // \v 1-2 deleted. The deleted unit's OWN sid is itself a bridge
        // (1-2), not a singular verse, so covered_by must NOT annotate it
        // against the coalesced pair's current 1-4 bridge — covered_by is
        // only for a one-sided *verse*, never a one-sided bridge.
        let baseline_src = wrap(0, "\\c 1\n\\v 1-3 A\n\\v 1-2 B\n");
        let current_src = wrap(0, "\\c 1\n\\v 1-4 A\n");
        let baseline = parse(&baseline_src);
        let current = parse(&current_src);
        let skeleton = diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK);

        let pair = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .expect("one coalesced pair (1-3 -> 1-4, exact-text match)");
        assert_eq!(pair.baseline_sid.as_deref(), Some("GEN 1:1-3"));
        assert_eq!(pair.current_sid.as_deref(), Some("GEN 1:1-4"));

        let deleted = skeleton
            .units
            .iter()
            .find(|u| matches!(u.kind, DecisionUnitKind::Deleted))
            .expect("the unmatched 1-2 bridge is deleted");
        assert_eq!(deleted.baseline_sid.as_deref(), Some("GEN 1:1-2"));
        assert!(
            deleted.covered_by.is_none(),
            "a one-sided bridge must never be annotated covered_by, even when it overlaps an opposite bridge"
        );
    }

    #[test]
    fn case_23_renumber_typo_pins() {
        let (skeleton, ..) = skeleton_for(23);

        let deleted: Vec<_> = skeleton
            .units
            .iter()
            .filter(|u| matches!(u.kind, DecisionUnitKind::Deleted))
            .collect();
        let added: Vec<_> = skeleton
            .units
            .iter()
            .filter(|u| matches!(u.kind, DecisionUnitKind::Added))
            .collect();
        let coalesced_count = skeleton
            .units
            .iter()
            .filter(|u| matches!(u.kind, DecisionUnitKind::Coalesced))
            .count();

        // exactly one deleted unit: GEN 1:11 with the 'k' body.
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].baseline_sid.as_deref(), Some("GEN 1:11"));
        assert!(block_text(&deleted[0].baseline_tokens).ends_with("k\n"));

        // exactly one added unit: GEN 1:1_dup_1 with the same 'k' body.
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].current_sid.as_deref(), Some("GEN 1:1_dup_1"));
        assert!(block_text(&added[0].current_tokens).ends_with("k\n"));

        // NO paired/moved unit — content-similarity across keys is rejected.
        assert_eq!(coalesced_count, 0);
    }

    // ---- pure merge, strict revert ----------------------------------------

    #[test]
    fn all_23_cases_merge_all_baseline_and_all_current_are_byte_exact() {
        // Includes case 20 (CRLF baseline vs LF current): identity anchors
        // must stay byte-exact regardless of line-ending mix.
        for case in CASES {
            let baseline_src = wrap(case.n, case.baseline);
            let current_src = wrap(case.n, case.current);
            let baseline = parse(&baseline_src);
            let current = parse(&current_src);
            let skeleton = diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK);

            let all_baseline =
                merge_skeleton(&skeleton, &BTreeMap::new(), MergeSide::Baseline).unwrap();
            let all_current =
                merge_skeleton(&skeleton, &BTreeMap::new(), MergeSide::Current).unwrap();

            assert_eq!(
                block_text(&all_baseline),
                baseline_src,
                "case {}: all-Baseline merge must reproduce baseline exactly",
                case.n
            );
            assert_eq!(
                block_text(&all_current),
                current_src,
                "case {}: all-Current merge must reproduce current exactly",
                case.n
            );
        }
    }

    /// Deterministic LCG (no external rand dependency) — reproducible across
    /// runs, seeded per case so trials differ but are stable in CI.
    fn lcg_next(seed: &mut u32) -> u32 {
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        *seed
    }

    #[test]
    fn all_23_cases_random_decision_vectors_produce_the_expected_merged_output() {
        for case in CASES {
            let baseline_src = wrap(case.n, case.baseline);
            let current_src = wrap(case.n, case.current);
            let baseline = parse(&baseline_src);
            let current = parse(&current_src);
            let skeleton = diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK);

            let mut seed = 0x9e3779b9u32 ^ case.n.wrapping_mul(2_654_435_761);

            for _trial in 0..24 {
                let mut decisions = BTreeMap::new();
                for unit in &skeleton.units {
                    let r = lcg_next(&mut seed) % 5;
                    if r < 2 {
                        decisions.insert(unit.id.clone(), MergeSide::Baseline);
                    } else if r < 4 {
                        decisions.insert(unit.id.clone(), MergeSide::Current);
                    }
                    // else: left absent, exercises default_side.
                }
                let default_side = if lcg_next(&mut seed).is_multiple_of(2) {
                    MergeSide::Baseline
                } else {
                    MergeSide::Current
                };

                // Purity: same inputs/vector twice return the same bytes.
                let first = merge_skeleton(&skeleton, &decisions, default_side).unwrap();
                let second = merge_skeleton(&skeleton, &decisions, default_side).unwrap();
                assert_eq!(
                    block_text(&first),
                    block_text(&second),
                    "case {}: not idempotent",
                    case.n
                );

                // Chosen-side/no-leakage witness: compare the REAL merged
                // output against text independently assembled per slot role
                // (not a recount of merge_skeleton's own bookkeeping) — a
                // wrong-side emission, a duplicated slot, or a leaked
                // unchosen side would show up as a byte mismatch here.
                let expected = expected_merge_text(&skeleton, &decisions, default_side);
                assert_eq!(
                    block_text(&first),
                    expected,
                    "case {}: merge_skeleton output must equal the independently assembled chosen-side text",
                    case.n
                );
            }
        }
    }

    /// See `expected_merge_text` in `skeleton_proptest.rs` for the rationale:
    /// written independently of `merge_skeleton`'s own slot-walking code so a
    /// real implementation bug shows up as a byte mismatch, not merely as a
    /// self-consistent recount of the same bookkeeping.
    fn expected_merge_text<T: DiffableToken>(
        skeleton: &DiffSkeleton<T>,
        decisions: &BTreeMap<UnitId, MergeSide>,
        default_side: MergeSide,
    ) -> String {
        use crate::diff::skeleton::SlotRole;

        let units_by_id: rustc_hash::FxHashMap<&UnitId, &crate::diff::skeleton::DecisionUnit<T>> =
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

    #[test]
    fn unknown_unit_id_errors_before_any_output_is_assembled() {
        let (skeleton, ..) = skeleton_for(13);
        let mut decisions = BTreeMap::new();
        decisions.insert(UnitId::new("__no_such_unit__"), MergeSide::Current);

        let result = merge_skeleton(&skeleton, &decisions, MergeSide::Current);
        assert_eq!(
            result,
            Err(MergeError::UnknownUnitId(UnitId::new("__no_such_unit__")))
        );
    }

    #[test]
    fn merge_diff_blocks_also_rejects_an_unknown_id_before_producing_output() {
        let baseline_src = wrap(13, CASES.iter().find(|c| c.n == 13).unwrap().baseline);
        let current_src = wrap(13, CASES.iter().find(|c| c.n == 13).unwrap().current);
        let baseline = parse(&baseline_src);
        let current = parse(&current_src);

        let baseline_tokens =
            to_format_tokens_with_normalized_sids_and_drifted_ids(&baseline.tokens, BOOK, "orig");
        let current_tokens =
            to_format_tokens_with_normalized_sids_and_drifted_ids(&current.tokens, BOOK, "new");

        let mut decisions = BTreeMap::new();
        decisions.insert(UnitId::new("__no_such_unit__"), MergeSide::Baseline);
        let result = merge_diff_blocks(
            &baseline_tokens,
            &current_tokens,
            &decisions,
            MergeSide::Current,
        );
        assert!(matches!(result, Err(MergeError::UnknownUnitId(_))));
    }

    #[test]
    fn single_revert_of_every_changed_unit_equals_a_one_decision_merge() {
        // `revert_diff_block` uses the external/`FormatToken` calling
        // convention (plain `diff_skeleton`, carried sids) — exercise it on
        // the same id-drift fixture used for that convention elsewhere in
        // this file, not on raw native `Token`s (those go through the
        // canonical convention instead, a different skeleton entirely).
        for case in CASES {
            let baseline_src = wrap(case.n, case.baseline);
            let current_src = wrap(case.n, case.current);
            let baseline = parse(&baseline_src);
            let current = parse(&current_src);
            let baseline_tokens = to_format_tokens_with_normalized_sids_and_drifted_ids(
                &baseline.tokens,
                BOOK,
                "orig",
            );
            let current_tokens =
                to_format_tokens_with_normalized_sids_and_drifted_ids(&current.tokens, BOOK, "new");
            let skeleton = diff_skeleton(&baseline_tokens, &current_tokens);

            for unit in skeleton
                .units
                .iter()
                .filter(|u| u.status != DecisionStatus::Unchanged)
            {
                let reverted =
                    revert_diff_block(unit.id.as_str(), &baseline_tokens, &current_tokens).unwrap();

                let mut decisions = BTreeMap::new();
                decisions.insert(unit.id.clone(), MergeSide::Baseline);
                let direct_merge =
                    merge_skeleton(&skeleton, &decisions, MergeSide::Current).unwrap();

                assert_eq!(
                    block_text(&reverted),
                    block_text(&direct_merge),
                    "case {}: revert_diff_block({}) must equal merge({{id: Baseline}}, Current)",
                    case.n,
                    unit.id
                );
            }
        }
    }
}
