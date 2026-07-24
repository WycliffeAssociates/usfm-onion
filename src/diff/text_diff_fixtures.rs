//! Fixture oracle for `unit_text_diff` (Gate 2 of
//! `plans/approved/plan-intra-unit-text-diff-2026-07-20.md`). Pins exact
//! human-visible word/char-run behavior. Status/USFM/relabel/move coverage
//! reuses cases from the `skeleton_fixtures.rs` catalog (matched by
//! *behavior*, not case number); repeated-word alignment and the
//! multilingual pins are hand-built minimal `\id GEN`-parseable fixtures.
//! Every case runs through both native parsed `Token`s (the canonical sid
//! convention) and app-shaped `FormatToken`s with drifted ids (the
//! carried-sid convention), asserting identical runs — mirroring
//! `skeleton_fixtures.rs`'s two-shape exercise.

#[cfg(test)]
mod tests {
    use crate::diff::skeleton::{
        DecisionStatus, DecisionUnit, DecisionUnitKind, DiffSkeleton, diff_skeleton,
        diff_skeleton_canonical,
    };
    use crate::diff::{
        DiffableToken, TextDiffMode, TextDiffRun, TextDiffRunKind, UnitTextDiff,
        derive_canonical_sids, unit_text_diff,
    };
    use crate::format::FormatToken;
    use crate::parse::parse;
    use crate::token::Token;

    const BOOK: &str = "GEN";

    /// Mirrors `skeleton_fixtures.rs`'s helper of the same name: a
    /// normalized canonical sid plus a deliberately drifted per-shape token
    /// id, so the `FormatToken` shape exercises the carried-sid calling
    /// convention with different bookkeeping than the native parse.
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

    fn native_skeleton(baseline_src: &str, current_src: &str) -> DiffSkeleton<Token<'static>> {
        let baseline_src: &'static str = Box::leak(baseline_src.to_string().into_boxed_str());
        let current_src: &'static str = Box::leak(current_src.to_string().into_boxed_str());
        let baseline = parse(baseline_src);
        let current = parse(current_src);
        diff_skeleton_canonical(&baseline.tokens, BOOK, &current.tokens, BOOK)
    }

    fn format_token_skeleton(baseline_src: &str, current_src: &str) -> DiffSkeleton<FormatToken> {
        let baseline = parse(baseline_src);
        let current = parse(current_src);
        let baseline_tokens =
            to_format_tokens_with_normalized_sids_and_drifted_ids(&baseline.tokens, BOOK, "orig");
        let current_tokens =
            to_format_tokens_with_normalized_sids_and_drifted_ids(&current.tokens, BOOK, "new");
        diff_skeleton(&baseline_tokens, &current_tokens)
    }

    /// Selects the one unit matching `pred` (over the type-independent
    /// fields, so the same closure works for both `T`s) and returns its
    /// text diff. Panics if `pred` doesn't select exactly one unit — every
    /// fixture below is built so its predicate is unambiguous.
    fn text_diff_for<T: DiffableToken>(
        skeleton: &DiffSkeleton<T>,
        mode: TextDiffMode,
        pred: impl Fn(DecisionStatus, DecisionUnitKind, Option<&str>, Option<&str>) -> bool,
    ) -> Option<UnitTextDiff> {
        let matches: Vec<&DecisionUnit<T>> = skeleton
            .units
            .iter()
            .filter(|u| pred(u.status, u.kind, u.baseline_sid.as_deref(), u.current_sid.as_deref()))
            .collect();
        assert_eq!(matches.len(), 1, "predicate must select exactly one unit");
        unit_text_diff(matches[0], mode)
    }

    /// Runs one `(baseline_src, current_src)` pair through BOTH the native
    /// parsed-`Token` (canonical) and `FormatToken`-with-drifted-ids
    /// (carried-sid) calling conventions, asserts the two shapes produce
    /// byte-identical runs for the unit `pred` selects, and returns that
    /// (shape-agreed) diff for the caller to make its own pin assertions on.
    fn text_diff_agreeing_across_shapes(
        baseline_src: &str,
        current_src: &str,
        mode: TextDiffMode,
        pred: impl Fn(DecisionStatus, DecisionUnitKind, Option<&str>, Option<&str>) -> bool + Copy,
    ) -> Option<UnitTextDiff> {
        let native = native_skeleton(baseline_src, current_src);
        let native_diff = text_diff_for(&native, mode, pred);

        let formatted = format_token_skeleton(baseline_src, current_src);
        let format_diff = text_diff_for(&formatted, mode, pred);

        assert_eq!(
            native_diff, format_diff,
            "native Token and FormatToken shapes must produce identical text-diff runs"
        );
        native_diff
    }

    fn run(text: &str, kind: TextDiffRunKind) -> TextDiffRun {
        TextDiffRun {
            text: text.to_string(),
            kind,
        }
    }

    // ---- reused skeleton_fixtures.rs cases (mapped by behavior) ----

    #[test]
    fn case1_modified_word_change_is_added_removed_run_bounded_by_unchanged() {
        // skeleton_fixtures.rs case 1: "heaven" -> "heavens".
        let baseline =
            "\\id GEN\n\\c 1\n\\v 1 In the beginning God created the heaven and the earth.\n";
        let current =
            "\\id GEN\n\\c 1\n\\v 1 In the beginning God created the heavens and the earth.\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("In the beginning God created the ", TextDiffRunKind::Unchanged),
                run("heaven", TextDiffRunKind::Removed),
                run(" and the earth.\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("In the beginning God created the ", TextDiffRunKind::Unchanged),
                run("heavens", TextDiffRunKind::Added),
                run(" and the earth.\n", TextDiffRunKind::Unchanged),
            ]
        );
    }

    #[test]
    fn case7_whitespace_only_change_has_no_lexical_word_run_change() {
        // skeleton_fixtures.rs case 7: trailing whitespace added, no word change.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 one\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 one  \n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        // Only whitespace may appear in the Added/Removed runs — no run
        // introduces or removes a lexical (non-whitespace) word.
        let changed_text: String = diff
            .baseline
            .iter()
            .chain(diff.current.iter())
            .filter(|r| r.kind != TextDiffRunKind::Unchanged)
            .map(|r| r.text.as_str())
            .collect();
        assert!(
            changed_text.chars().all(|c| c.is_whitespace()),
            "expected only whitespace to change, got {changed_text:?}"
        );
        // And the word "one" itself is carried, unchanged, on both sides.
        let baseline_text: String = diff.baseline.iter().map(|r| r.text.as_str()).collect();
        let current_text: String = diff.current.iter().map(|r| r.text.as_str()).collect();
        assert!(baseline_text.contains("one"));
        assert!(current_text.contains("one"));
    }

    #[test]
    fn case8_usfm_structure_only_change_is_all_unchanged() {
        // skeleton_fixtures.rs case 8: chapter-open \p -> \m, same reader text.
        let baseline = "\\id GEN\n\\c 1\n\\p\n\\v 1 one\n\\s Section\n\\v 2 two\n";
        let current = "\\id GEN\n\\c 1\n\\m\n\\v 1 one\n\\v 2 two\n";
        let diff = text_diff_agreeing_across_shapes(
            baseline,
            current,
            TextDiffMode::Words,
            |_, _, baseline_sid, _| baseline_sid == Some("GEN 1:0"),
        )
        .expect("chapter-open unit is Modified (bytes differ) and yields Some");

        assert!(diff.baseline.iter().all(|r| r.kind == TextDiffRunKind::Unchanged));
        assert!(diff.current.iter().all(|r| r.kind == TextDiffRunKind::Unchanged));
        let baseline_text: String = diff.baseline.iter().map(|r| r.text.as_str()).collect();
        let current_text: String = diff.current.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            baseline_text, current_text,
            "structure-only change must not produce a false word change"
        );
    }

    #[test]
    fn case10_moved_byte_equal_unit_has_no_text_diff() {
        // skeleton_fixtures.rs case 10: two verses swap position, byte-equal.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n";
        let current = "\\id GEN\n\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Moved
        });
        assert_eq!(diff, None);
    }

    #[test]
    fn case13_relabeled_byte_equal_survivor_has_no_text_diff() {
        // skeleton_fixtures.rs case 13: duplicate-verse survivor relabels
        // sid only; text is byte-equal (status stays Unchanged, not Moved).
        let baseline = "\\id GEN\n\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n";
        let current = "\\id GEN\n\\c 1\n\\v 2 b\n\\v 1 c\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |_, kind, _, _| {
            kind == DecisionUnitKind::Coalesced
        });
        assert_eq!(diff, None);
    }

    #[test]
    fn case4_added_verse_is_a_single_added_run() {
        // skeleton_fixtures.rs case 4: verse 3 added.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 one\n\\v 2 two\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Added
        })
        .expect("Added unit yields Some");

        assert_eq!(diff.baseline, vec![]);
        assert_eq!(diff.current, vec![run("three\n", TextDiffRunKind::Added)]);
    }

    #[test]
    fn case5_deleted_verse_is_a_single_removed_run() {
        // skeleton_fixtures.rs case 5: verse 2 ("two") deleted.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 one\n\\v 2 two\n\\v 3 three\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 one\n\\v 3 three\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Deleted
        })
        .expect("Deleted unit yields Some");

        assert_eq!(diff.baseline, vec![run("two\n", TextDiffRunKind::Removed)]);
        assert_eq!(diff.current, vec![]);
    }

    #[test]
    fn case18_note_prose_edit_isolates_the_word_change_markers_excluded() {
        // skeleton_fixtures.rs case 18: footnote inner prose "a note" ->
        // "an edited note". The \f/\ft/\f* marker/end-marker tokens are
        // excluded from plain text by kind_key ("marker"/"endMarker"), so
        // they never appear in any run: no run below contains a `\`.
        //
        // Note (deviation from the plan's prose): the note's caller ("+")
        // has no dedicated token kind — the tokenizer has no concept of
        // "caller" at all (see src/token.rs's TokenKind and
        // src/marker_defs.rs's payload table, which only special-cases
        // `id`/`c`/`v`-family markers). "+" lexes as an ordinary Text
        // token, so per unit_plain_text's own filter (kind_key "text" is
        // included) it DOES participate in plain text and therefore in the
        // runs below — as literal unchanged "+" text — not "absent" the
        // way the plan narrative describes. This is verified byte-for-byte,
        // not assumed.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 Text\\f + \\ft a note\\f* more.\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 Text\\f + \\ft an edited note\\f* more.\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("Text+ ", TextDiffRunKind::Unchanged),
                run("a", TextDiffRunKind::Removed),
                run(" note more.\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("Text+ ", TextDiffRunKind::Unchanged),
                run("an edited", TextDiffRunKind::Added),
                run(" note more.\n", TextDiffRunKind::Unchanged),
            ]
        );
        for r in diff.baseline.iter().chain(diff.current.iter()) {
            assert!(
                !r.text.contains('\\'),
                "no run should carry a raw USFM marker: {r:?}"
            );
        }
    }

    // ---- hand-built: repeated words / ambiguous alignment ----

    #[test]
    fn repeated_words_ambiguous_alignment_has_one_deterministic_layout() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 the the the\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 the the\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        // Myers picks the earliest/leftmost matching subsequence: the first
        // two "the"s (plus the space between them) align as unchanged, and
        // the trailing " the" (space + third occurrence) is the removal —
        // not, say, the first "the", or a scattered set of single-word
        // removals. This is the one deterministic layout for this input.
        assert_eq!(
            diff.baseline,
            vec![
                run("the the", TextDiffRunKind::Unchanged),
                run(" the", TextDiffRunKind::Removed),
                run("\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(diff.current, vec![run("the the\n", TextDiffRunKind::Unchanged)]);

        // Determinism: re-running the whole pipeline produces the exact
        // same layout, not merely an equally-valid alternative alignment.
        let again = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        });
        assert_eq!(Some(diff), again);
    }

    // ---- multilingual pins (hardened, must-pass gates) ----

    #[test]
    fn apostrophe_straight_vs_curly_is_one_atomic_word_swap() {
        // UAX #29 classifies both the straight apostrophe (U+0027) and the
        // curly closing single quote (U+2019) as MidNumLet: an apostrophe
        // flanked by letters on both sides does not split the word. So
        // "don't" is ONE word token on each side — changing the quote style
        // is a whole-word swap, never a same-word "apostrophe-only"
        // sub-highlight. A naive (non-Unicode, ASCII-punctuation-splitting)
        // tokenizer would wrongly carve "don" / "'" / "t" into three tokens
        // and report only the punctuation as changed — that would be
        // linguistically wrong for a contraction.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 They don't know.\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 They don\u{2019}t know.\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("They ", TextDiffRunKind::Unchanged),
                run("don't", TextDiffRunKind::Removed),
                run(" know.\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("They ", TextDiffRunKind::Unchanged),
                run("don\u{2019}t", TextDiffRunKind::Added),
                run(" know.\n", TextDiffRunKind::Unchanged),
            ]
        );
    }

    #[test]
    fn hebrew_combining_niqqud_stays_one_grapheme_with_its_base_letter() {
        // A base letter plus a combining mark (here Hebrew vav U+05D5 plus
        // the dagesh/point U+05BC) is ONE grapheme cluster under
        // `from_graphemes`. The correct run boundary lands on the WHOLE
        // cluster: the bare base letter is removed and the base+mark
        // cluster is added as one atomic unit — never split into "base
        // letter unchanged" + "mark added", which would visually detach the
        // diacritic from its consonant (not how a Hebrew reader parses the
        // text).
        let baseline = "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05E8}\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05BC}\u{05E8}\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Chars, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("\u{05D0}", TextDiffRunKind::Unchanged),
                run("\u{05D5}", TextDiffRunKind::Removed),
                run("\u{05E8}\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("\u{05D0}", TextDiffRunKind::Unchanged),
                run("\u{05D5}\u{05BC}", TextDiffRunKind::Added),
                run("\u{05E8}\n", TextDiffRunKind::Unchanged),
            ]
        );
    }

    #[test]
    fn arabic_rtl_word_edit_is_bounded_by_unchanged_words() {
        // Unicode text is stored in logical (memory) order regardless of
        // display direction, so RTL Arabic word-splitting works exactly
        // like Latin: `from_unicode_words` finds the same word boundaries a
        // reader would, and the one changed word (roughly "God" ->
        // "Lord") is isolated between unchanged context words on both
        // sides — RTL display direction is a rendering concern, not a
        // segmentation one.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0644}\u{0647}\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0631}\u{0628}\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run(
                    "\u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} ",
                    TextDiffRunKind::Unchanged
                ),
                run("\u{0627}\u{0644}\u{0644}\u{0647}", TextDiffRunKind::Removed),
                run("\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run(
                    "\u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} ",
                    TextDiffRunKind::Unchanged
                ),
                run("\u{0627}\u{0644}\u{0631}\u{0628}", TextDiffRunKind::Added),
                run("\n", TextDiffRunKind::Unchanged),
            ]
        );
    }

    #[test]
    fn thai_no_space_segment_edit_lands_on_the_changed_word() {
        // Thai writes without inter-word spaces. Plain UAX #29 word
        // splitting has no dictionary, so on its own it falls back to
        // near-grapheme-sized segments across a run of Thai
        // consonants/vowel signs — but `unit_text_diff`'s run coalescing
        // (adjacent same-kind segments merge into one run) recovers a
        // single contiguous run per side for one contiguous edited
        // stretch: the "world" (\u{0E42}\u{0E25}\u{0E01}) -> "country"
        // (\u{0E1B}\u{0E23}\u{0E30}\u{0E40}\u{0E17}\u{0E28}) swap lands on
        // exactly one Removed/Added run bounded by the unchanged "hello"
        // prefix, not a scatter of single-character runs.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E42}\u{0E25}\u{0E01}\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E1B}\u{0E23}\u{0E30}\u{0E40}\u{0E17}\u{0E28}\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("\u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}", TextDiffRunKind::Unchanged),
                run("\u{0E42}\u{0E25}\u{0E01}", TextDiffRunKind::Removed),
                run("\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("\u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}", TextDiffRunKind::Unchanged),
                run(
                    "\u{0E1B}\u{0E23}\u{0E30}\u{0E40}\u{0E17}\u{0E28}",
                    TextDiffRunKind::Added
                ),
                run("\n", TextDiffRunKind::Unchanged),
            ]
        );
    }

    #[test]
    fn latin_punctuation_adjacent_edit_keeps_punctuation_out_of_the_word_run() {
        // The changed word sits directly against punctuation on both sides
        // (comma before, "!" immediately after, no space) — the word
        // boundary must not glue the punctuation into the Added/Removed
        // run.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 Hello, world!\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 Hello, there!\n";
        let diff = text_diff_agreeing_across_shapes(baseline, current, TextDiffMode::Words, |status, _, _, _| {
            status == DecisionStatus::Modified
        })
        .expect("Modified unit yields Some");

        assert_eq!(
            diff.baseline,
            vec![
                run("Hello, ", TextDiffRunKind::Unchanged),
                run("world", TextDiffRunKind::Removed),
                run("!\n", TextDiffRunKind::Unchanged),
            ]
        );
        assert_eq!(
            diff.current,
            vec![
                run("Hello, ", TextDiffRunKind::Unchanged),
                run("there", TextDiffRunKind::Added),
                run("!\n", TextDiffRunKind::Unchanged),
            ]
        );
    }
}
