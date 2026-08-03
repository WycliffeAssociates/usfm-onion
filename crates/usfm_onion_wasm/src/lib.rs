//! WebAssembly bindings for `usfm_onion`.
//!
//! One module per responsibility: the boundary value types and their conversions
//! ([`dto`]), the stateless one-shot exports ([`stateless`]), and the resident
//! corpus handle ([`resident`]). Publish/restorePublishedCorpus's own
//! composition (the publication adapter) lives in `usfm_onion_host` now, not
//! here -- this crate only projects it across the wasm-bindgen boundary. This
//! root holds only what belongs to the crate as a whole — the hand-written
//! TypeScript section, and the re-exports that keep every public item
//! reachable at the crate root whichever module declares it.

use wasm_bindgen::prelude::*;

// Only what this root's own test module reaches for; everything else moved with
// the code that uses it.
#[cfg(test)]
use usfm_onion::diff::{
    TextDiffMode as NativeTextDiffMode, unit_text_diff as native_unit_text_diff,
};
#[cfg(test)]
use usfm_onion::lint::{
    LintCode as NativeLintCode, LintOptions as NativeLintOptions, LintScope as NativeLintScope,
    lint_tokens,
};
#[cfg(test)]
use usfm_onion::parse::parse as native_parse;
#[cfg(test)]
use usfm_onion::token::tokens_to_usfm_reconstruct;

pub use usfm_onion_wire::dto::{
    AttributeItem, BlockBehavior, ClosingBehavior, CoveredSide, DecisionStatus, DecisionUnitKind,
    DiffOptions, HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, InlineContext, LintCategory,
    LintCode, LintIssueType, LintSeverity, MarkerCategory, MarkerDefKind, MarkerFamily,
    MarkerFamilyRole, MarkerInfo, MarkerKind, MarkerMetadata, MarkerPayload, MergeSide, NoteFamily,
    NoteSubkind, NumberInfo, NumberRangeKind, PackedBookReceipt, PackedDecodeError,
    PackedMarkerDescriptor, ParagraphCategory, SlotRole, Span, SpecContext, StructuralMarkerInfo,
    StructuralScopeKind, TextDiffMode, TextDiffRun, TextDiffRunKind, Token, TokenKind,
    UnitTextDiff, format_sid, map_marker_info,
};

pub mod dto;
pub mod resident;
pub mod stateless;

/// The native-vs-wasm parity transcript generator. Test-only: it needs the
/// same `pub(crate)` DTO conversions `resident`'s wasm bindings call, so it
/// lives inside the crate rather than as an external integration test.
#[cfg(test)]
mod parity;

pub use crate::dto::*;
pub use crate::stateless::*;

// TODO: eventually move off of this ideally
#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
// JSON Value type and USJ document tree.
//
// Mirroring the native `UsjDocument` shape as a real Tsify type is
// follow-up work; until that lands, USJ is the one return type that
// is still emitted as a TS_TYPES hand-written declaration. Every
// other public shape is tsify-derived.

export type Value =
  | string
  | number
  | boolean
  | null
  | Value[]
  | { [key: string]: Value };

export type UsjDocument = {
  type: string;
  version: string;
  content: UsjNode[];
};

export type UsjNode = string | UsjElement;

export type UsjElement =
  | ({ type: "book"; marker: string; code: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "chapter"; marker: string; number: string; sid?: string } & Record<string, Value>)
  | ({ type: "verse"; marker: string; number: string; sid?: string } & Record<string, Value>)
  | ({ type: "para"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "char"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "ref"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "note"; marker: string; caller: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "ms"; marker: string } & Record<string, Value>)
  | ({ type: "figure"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "sidebar"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "periph"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table"; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table:row"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "table:cell"; marker: string; align?: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "unknown"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "unmatched"; marker: string; content?: UsjNode[] } & Record<string, Value>)
  | ({ type: "optbreak" } & Record<string, Value>);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(source: &str) -> String {
        let tokens = map_tokens(&native_parse(source).tokens);
        tokens_to_usfm_reconstruct(&tokens)
    }

    #[test]
    fn round_trips_character_marker_with_attributes() {
        // The canonical case the RFC was written to fix.
        let source = "\\w word|lemma=\"lemma\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_all_kitchen_sink_attribute_markers() {
        for source in [
            "\\w word|lemma=\"lemma\"\\w*",
            "\\rb gloss|gloss=\"gloss\"\\rb*",
            "\\wl word|index=\"x\"\\wl*",
            "\\jmp text|link-href=\"#x\"\\jmp*",
        ] {
            assert_eq!(round_trip(source), source, "round-trip failed for {source}");
        }
    }

    #[test]
    fn round_trips_milestone_with_attributes() {
        let source = "\\zaln-s |x-strong=\"H0430\"\\*word\\zaln-e\\*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_default_attribute_shorthand() {
        // `\w word|lemma\w*` — bare value, no key=. Round-trip preserves shorthand.
        let source = "\\w word|lemma\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_multiple_attributes() {
        let source = "\\w word|lemma=\"x\" strong=\"H0430\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn round_trips_non_canonical_attribute_whitespace_via_wire() {
        // Extra inter-item spacing (`lemma = "y"` instead of `lemma="y"`) is
        // trivia only a verbatim slice preserves. Before `attributeSource`
        // was added to the wire DTO, this normalized to canonical spacing
        // when it crossed native -> DTO -> wasm; now the verbatim
        // `attribute_source` rides along the wire and this round-trips
        // byte-identical.
        let source = "\\w x|lemma = \"y\"\\w*";
        assert_eq!(round_trip(source), source);
    }

    /// Parity contract for `normalizeTokenSids`, pinned on the onion side.
    ///
    /// The other half of this contract — comparing these exact values
    /// against a live Zephyr `mutAddSids(tokens, bookCode)` call — cannot
    /// run in this repo: `mutAddSids` is Zephyr application code, not a
    /// dependency of onion, and vendoring or importing it here would defeat
    /// the point of keeping the two implementations independent. This test
    /// is onion's half of the parity gate: the exact sid strings
    /// `normalizeTokenSids` must produce for bridge, duplicate, and intro
    /// streams, which Zephyr's consumer migration diffs its own
    /// `mutAddSids` output against before retiring it (see
    /// `plans/zephyr-handoff-merge-projection.md`). If this test ever
    /// changes, that is a signal the Zephyr-side comparison needs
    /// re-running, not just a golden update.
    #[test]
    fn normalize_token_sids_bridge_dup_intro_parity_contract() {
        let source = "\\id GEN\n\\h Genesis\n\\c 1\n\\v 1 a\n\\v 1-2 b\n\\v 1 c\n";
        let tokens = map_tokens(&native_parse(source).tokens);
        let normalized = wasm_normalize_token_sids(tokens.clone(), "GEN");

        assert_eq!(
            tokens.len(),
            normalized.len(),
            "normalizeTokenSids must preserve length/order"
        );
        for (before, after) in tokens.iter().zip(&normalized) {
            assert_eq!(before.id, after.id, "id must be untouched");
            assert_eq!(before.source, after.source, "source/text must be untouched");
            assert_eq!(
                format!("{:?}", before.kind),
                format!("{:?}", after.kind),
                "kind must be untouched"
            );
        }

        let sid_of = |text: &str| -> String {
            normalized
                .iter()
                .find(|t| t.source.trim() == text)
                .unwrap_or_else(|| panic!("no token with text {text:?}"))
                .sid
                .clone()
                .unwrap_or_else(|| panic!("token {text:?} has no sid"))
        };

        // intro: book code precedes any \c.
        assert_eq!(sid_of("Genesis"), "GEN 0:0");
        // single verse.
        assert_eq!(sid_of("a"), "GEN 1:1");
        // bridge: range end encoded.
        assert_eq!(sid_of("b"), "GEN 1:1-2");
        // duplicate: second occurrence of the same base sid gets _dup_1.
        assert_eq!(sid_of("c"), "GEN 1:1_dup_1");
    }

    #[test]
    fn round_trips_after_edit_inserting_text_in_marker_run() {
        // Edit scenario: parse, then splice a synthetic Text token between
        // the word content and the `\w*` closer. Span on the synthetic
        // token is None (consumer-fabricated). The structural emitter
        // must still drain the attribute slice before the closer.
        let source = "\\w hello|lemma=\"x\"\\w*";
        let mut tokens = map_tokens(&native_parse(source).tokens);
        let closer_idx = tokens
            .iter()
            .position(|t| matches!(t.kind, TokenKind::EndMarker))
            .expect("expected \\w* closer");
        tokens.insert(
            closer_idx,
            Token {
                id: "synthetic-1".into(),
                kind: TokenKind::Text,
                source: " world".into(),
                span: None,
                sid: None,
                marker: None,
                nested: None,
                marker_metadata: None,
                structural: None,
                number_info: None,
                book_code: None,
                book_code_valid: None,
                attributes: Vec::new(),
                attribute_source: None,
                attribute_offset: None,
            },
        );
        // Attribute slice still lands right before \w*, independent of
        // the inserted Text token between word content and closer.
        assert_eq!(
            tokens_to_usfm_reconstruct(&tokens),
            "\\w hello world|lemma=\"x\"\\w*"
        );
    }

    #[test]
    fn round_trips_embedded_double_quote_in_attribute_value_via_reconstruct() {
        // Source uses USFM 3.1 + the usfm-onion `\"` extension (D3). Decode-on-map
        // turns `\"` into `"` on the JS-facing value. Clearing `attributeSource`
        // simulates an editor having touched this attribute (the documented
        // rule: touch an attribute -> drop its verbatim, onion reconstructs
        // from structure) — the reconstruct path must re-escape the literal
        // quote for byte-identical output.
        let source = "\\w word|note=\"a\\\"b\"\\w*";
        let mut tokens = map_tokens(&native_parse(source).tokens);
        let attr_token = tokens
            .iter_mut()
            .find(|t| !t.attributes.is_empty())
            .expect("expected attribute-bearing token");
        assert_eq!(attr_token.attributes[0].value, "a\"b");
        attr_token.attribute_source = None;
        assert_eq!(tokens_to_usfm_reconstruct(&tokens), source);
    }

    #[test]
    fn format_tokens_preserves_a_structurally_edited_attribute() {
        // An editor that edits an attribute's structured value clears
        // `attributeSource` (the whole-list attr-edit contract), so the
        // structured edit is what must survive. Before this fix,
        // `token_value_to_format_token` only ever copied `attributeSource`
        // (never the structured list) and `map_format_token` unconditionally
        // returned an empty `attributes` vec, so `formatTokens` silently
        // reverted the edit.
        let source = "\\w word|note=\"a\"\\w*";
        let mut tokens = map_tokens(&native_parse(source).tokens);
        let attr_token = tokens
            .iter_mut()
            .find(|t| !t.attributes.is_empty())
            .expect("expected attribute-bearing token");
        let marker_span = attr_token.span.as_ref().map(|span| (span.start, span.end));
        let original_attribute_span = attr_token.attributes[0]
            .span
            .as_ref()
            .map(|span| (span.start, span.end))
            .expect("a parsed attribute must carry its own real span");
        assert_ne!(
            Some(original_attribute_span),
            marker_span,
            "fixture must actually put the attribute list at a different byte \
             range than the marker, or this test cannot catch span fabrication"
        );
        attr_token.attributes[0].value = "b".to_string();
        attr_token.attribute_source = None;

        let result = wasm_format_tokens(tokens, None);
        assert!(
            result.usfm.contains("note=\"b\""),
            "edited attribute must survive formatTokens: {:?}",
            result.usfm
        );
        let returned = result
            .tokens
            .iter()
            .find(|t| !t.attributes.is_empty())
            .expect("formatTokens must return the attribute-bearing token with its structured list intact");
        assert_eq!(returned.attributes[0].value, "b");
        assert!(returned.attribute_source.is_none());
        // The regression this guards: the attribute's own parsed span must
        // survive verbatim, never substituted with the owning marker's span.
        let returned_attribute_span = returned.attributes[0]
            .span
            .as_ref()
            .map(|span| (span.start, span.end));
        assert_eq!(
            returned_attribute_span,
            Some(original_attribute_span),
            "the attribute's own span must survive formatTokens, not be \
             replaced with the owning marker's span"
        );
    }

    #[test]
    fn empty_token_stream_emits_empty_string() {
        assert_eq!(tokens_to_usfm_reconstruct::<Token>(&[]), "");
    }

    #[test]
    fn round_trip_no_attributes_unchanged() {
        // Sanity: structural emit must not regress the no-attribute path.
        let source = "\\id GEN\n\\c 1\n\\v 1 In the beginning.\n";
        assert_eq!(round_trip(source), source);
    }

    #[test]
    fn lint_tokens_accepts_parsed_footnote_submarker_streams() {
        let source = "\\id GEN\n\\c 1\n\\pi1\n\\v 26 Then God said, “Let Us make man in Our image, after Our likeness, to rule over the fish of the sea and the birds of the air, over the livestock, and over all the earth itself\\f + \\fr 1:26 \\ft MT; Syriac \\fqa and over all the beasts of the earth\\f* and every creature that crawls upon it.”\n\\q1\n";
        let token_values = map_tokens(&native_parse(source).tokens);
        let adapter_tokens = parse_walk_tokens_from_values(token_values);
        let result = lint_tokens(
            &adapter_tokens,
            NativeLintOptions::scoped(NativeLintScope::Book),
        );

        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker),
            "unexpected stray-close issues: {:?}",
            result.issues
        );
    }

    // Note: a prior test fed `lint_tokens` a payload with `structural`,
    // `marker_metadata`, and `nested` stripped, and expected no stray-close
    // issues. After the walker migration in e1e6011 the lint visitor needs
    // `structural` to recover open scopes — with it missing, every closer
    // (\f*, \x*, \+xt*) appears stray. Callers must pass through the
    // canonical token shape produced by `parse()`; tearing it down before
    // round-tripping is no longer a supported contract.

    #[test]
    fn lint_tokens_accepts_parsed_nested_cross_reference_char_streams() {
        let source = "\\id GEN\n\\c 1\n\\v 1 In the beginning\\x + \\xo 1:1 \\xt cross ref \\+xt nested\\+xt* tail\\x*\n";
        let token_values = map_tokens(&native_parse(source).tokens);
        let adapter_tokens = parse_walk_tokens_from_values(token_values);
        let result = lint_tokens(
            &adapter_tokens,
            NativeLintOptions::scoped(NativeLintScope::Book),
        );

        assert!(
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker),
            "unexpected stray-close issues: {:?}",
            result.issues
        );
    }

    #[test]
    fn lint_token_batch_accepts_parsed_source_faithful_streams() {
        let sources = [
            "\\id GEN\n\\c 1\n\\pi1\n\\v 26 Then God said\\f + \\fr 1:26 \\ft MT; Syriac \\fqa and over all the beasts of the earth\\f*\n",
            "\\id GEN\n\\c 1\n\\v 1 In the beginning\\x + \\xo 1:1 \\xt cross ref \\+xt nested\\+xt* tail\\x*\n",
        ];
        let batches: Vec<Vec<WalkToken>> = sources
            .iter()
            .map(|source| map_tokens(&native_parse(source).tokens))
            .map(parse_walk_tokens_from_values)
            .collect();

        let results = batches
            .iter()
            .map(|tokens| lint_tokens(tokens, NativeLintOptions::scoped(NativeLintScope::Book)))
            .collect::<Vec<_>>();

        assert!(results.iter().all(|result| {
            !result
                .issues
                .iter()
                .any(|issue| issue.code == NativeLintCode::StrayCloseMarker)
        }));
    }

    #[test]
    fn lint_issue_types_and_mapping_include_message_params() {
        let code = usfm_onion::LintCode::DuplicateVerseNumber;
        let issue = usfm_onion::LintIssue {
            code,
            category: code.category(),
            severity: code.severity(),
            issue_type: code.issue_type(),
            template: code.template(),
            message: "expected verse 2 here, found 3".to_string(),
            message_params: std::collections::BTreeMap::from([
                ("expected".to_string(), "2".to_string()),
                ("found".to_string(), "3".to_string()),
            ]),
            span: None,
            related_span: None,
            token_id: None,
            related_token_id: None,
            sid: None,
            marker: Some("v".to_string()),
            fix: None,
            position: usfm_onion::lint::NO_TOKEN_POSITION,
            related_position: usfm_onion::lint::NO_TOKEN_POSITION,
        };

        let mapped = map_lint_issue(issue);
        assert_eq!(mapped.issue_type, LintIssueType::Content);
        assert_eq!(
            mapped.message_params.get("expected"),
            Some(&"2".to_string())
        );
        assert_eq!(mapped.message_params.get("found"), Some(&"3".to_string()));
    }

    /// The packed verify surface's findings now carry their fix, which is the
    /// wasm-facing half of the Phase C finding-and-patch gate: the whole chain
    /// (core lint -> wire patch table -> checked decode -> DTO mapping) has to
    /// hold for a consumer's fix button to have anything to press.
    #[test]
    fn verify_packed_book_returns_findings_with_their_fix() {
        // One `\p` jammed onto the text: `missing-whitespace-before-marker`
        // with a concrete remedy.
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n";
        let parsed = native_parse(source);
        let issues = usfm_onion::lint::lint_tokens(
            &parsed.tokens,
            usfm_onion::lint::LintOptions::scoped(usfm_onion::lint::LintScope::Book),
        )
        .issues;
        let packed = usfm_onion_wire::finding_codec::encode_book(
            usfm_onion::token::BookId::from_str("GEN").unwrap(),
            source,
            &parsed.tokens,
            &issues,
        )
        .expect("encodes");

        let outcome = wasm_verify_packed_book(&packed, source.as_bytes());
        let PackedBookOutcome::Verified { findings, .. } = outcome else {
            panic!("a container this crate just encoded must verify");
        };
        let fixed = findings
            .iter()
            .filter_map(|finding| finding.fix.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(fixed.len(), 1, "one finding carries a fix");
        let TokenFix::ReplaceToken {
            code,
            target_token_id,
            replacements,
            label_params,
            ..
        } = fixed[0]
        else {
            panic!("core's whitespace remedy is a ReplaceToken: {:?}", fixed[0]);
        };
        assert_eq!(code, "insert-whitespace-before-marker");
        assert!(label_params.is_empty());
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].text, "\n\\p");
        assert_eq!(replacements[0].marker.as_deref(), Some("p"));
        // The target is addressed by the same token id the DTO tokens carry, so
        // a consumer can find it in the array it already holds.
        assert!(
            map_tokens(&parsed.tokens)
                .iter()
                .any(|token| token.id.as_str() == target_token_id),
            "the fix's target names one of this book's tokens"
        );
    }

    // --- DiffOptions / textDiff wire threading (Gate 4) ---
    //
    // `DiffOptions`'s `text_diff` field is intentionally private (mirroring
    // the plan's sketch) — the wasm boundary only ever builds one by
    // deserializing the JS-facing wire shape, never by Rust field
    // construction. These tests do the same.

    fn diff_options(text_diff: &str) -> DiffOptions {
        serde_json::from_value(serde_json::json!({ "textDiff": text_diff }))
            .expect("DiffOptions must deserialize from its wire shape")
    }

    #[test]
    fn omitted_and_explicit_none_diff_options_produce_no_text_diff_and_match_byte_for_byte() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n\\v 2 old text\n\\v 3 same text\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n\\v 3 same text\n\\v 4 new text\n";

        let omitted = wasm_diff_usfm(baseline, current, None);
        let explicit_none = wasm_diff_usfm(baseline, current, Some(diff_options("none")));

        for skeleton in [&omitted, &explicit_none] {
            assert!(
                skeleton.units.iter().all(|u| u.text_diff.is_none()),
                "omitted/\"none\" DiffOptions must compute no text_diff on any unit"
            );
        }
        // Additive/back-compatible per the plan: omitting the option must be
        // byte-identical to passing an explicit "none" — and, since neither
        // computes a text_diff, both must be byte-identical to a caller that
        // never knew this feature existed.
        assert_eq!(
            serde_json::to_string(&omitted).unwrap(),
            serde_json::to_string(&explicit_none).unwrap()
        );
    }

    #[test]
    fn words_mode_populates_text_diff_on_changed_units_only() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n\\v 2 old text\n\\v 3 same text\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n\\v 3 same text\n\\v 4 new text\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));

        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit (v1 heaven -> heavens)");
        assert!(
            modified.text_diff.is_some(),
            "Modified must carry a text_diff"
        );

        let added = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Added))
            .expect("expected one Added unit (v4)");
        assert!(added.text_diff.is_some(), "Added must carry a text_diff");

        let deleted = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Deleted))
            .expect("expected one Deleted unit (v2)");
        assert!(
            deleted.text_diff.is_some(),
            "Deleted must carry a text_diff"
        );

        let unchanged = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Unchanged))
            .expect("expected one Unchanged unit (v3)");
        assert!(
            unchanged.text_diff.is_none(),
            "Unchanged (byte-equal) must not carry a text_diff"
        );
    }

    #[test]
    fn words_mode_leaves_a_pure_move_without_a_text_diff() {
        // Verses swapped, byte-identical text either side -> Moved.
        let baseline = "\\id GEN\n\\c 1\n\\v 1 First verse.\n\\v 2 Second verse.\n";
        let current = "\\id GEN\n\\c 1\n\\v 2 Second verse.\n\\v 1 First verse.\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));
        let moved = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Moved))
            .expect("expected one Moved unit");
        assert!(
            moved.text_diff.is_none(),
            "a pure move (byte-equal text) must not read as a content change"
        );
    }

    #[test]
    fn words_mode_current_and_baseline_runs_highlight_the_changed_word() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("words")));
        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit");
        let text_diff = modified
            .text_diff
            .as_ref()
            .expect("Modified unit must carry a text_diff under \"words\"");

        assert!(
            text_diff
                .current
                .iter()
                .any(|r| r.text == "heavens" && matches!(r.kind, TextDiffRunKind::Added)),
            "current runs must highlight the new word \"heavens\": {:?}",
            text_diff.current
        );
        assert!(
            text_diff
                .baseline
                .iter()
                .any(|r| r.text == "heaven" && matches!(r.kind, TextDiffRunKind::Removed)),
            "baseline runs must highlight the removed word \"heaven\": {:?}",
            text_diff.baseline
        );
    }

    #[test]
    fn chars_mode_is_also_threaded_through_diff_options() {
        let baseline = "\\id GEN\n\\c 1\n\\v 1 heaven\n";
        let current = "\\id GEN\n\\c 1\n\\v 1 heavens\n";

        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options("chars")));
        let modified = skeleton
            .units
            .iter()
            .find(|u| matches!(u.status, DecisionStatus::Modified))
            .expect("expected one Modified unit");
        let text_diff = modified
            .text_diff
            .as_ref()
            .expect("Modified unit must carry a text_diff under \"chars\"");
        assert!(
            text_diff
                .current
                .iter()
                .any(|r| r.text == "s" && matches!(r.kind, TextDiffRunKind::Added)),
            "grapheme-level diff must isolate the added \"s\": {:?}",
            text_diff.current
        );
    }

    /// The boundary's own statement of order, checked after serialization —
    /// which is the only place a sorted container's betrayal is visible. The
    /// fixture is out of order in both dimensions a sort would "fix": verse 19
    /// before verse 2, and chapter 10 before chapter 2.
    #[test]
    fn vref_index_states_the_documents_own_verse_order_across_the_boundary() {
        let source = "\\id GEN\n\\c 29\n\\p\n\\v 19 nineteen\n\\v 2 two\n\\c 10\n\\p\n\\v 1 ten one\n\\c 2\n\\p\n\\v 1 two one\n";
        let expected = ["GEN 29:19", "GEN 29:2", "GEN 10:1", "GEN 2:1"];

        for index in [
            crate::stateless::wasm_vref_index_usfm(source),
            // The token lane is the editor's: same projection, no reparse, and it
            // must state the same order.
            crate::stateless::wasm_vref_index_tokens(crate::dto::map_tokens(
                &native_parse(source).tokens,
            )),
        ] {
            let json = serde_json::to_value(&index).expect("serializes");
            let entries = json.as_array().expect("ordered pairs, not an object");
            let order: Vec<&str> = entries
                .iter()
                .map(|entry| entry[0].as_str().expect("sid"))
                .collect();
            assert_eq!(order, expected, "the boundary must report document order");
            // Sorting is what a keyed object would have done to it, and what a
            // consumer must never do to recover sequence.
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_ne!(order, sorted, "the fixture must distinguish the two");
            // Each pair carries its whole projection, so the sequence is the only
            // view a consumer needs.
            assert!(
                entries
                    .iter()
                    .all(|entry| entry[1]["text"].is_string() && entry[1]["segments"].is_array())
            );
        }
    }

    /// The resident handle projects the same verse index the stateless export does,
    /// including after an edit — the property that lets a consumer swap one for the
    /// other on a keystroke path without changing what it reads.
    #[test]
    fn the_resident_vref_index_matches_the_stateless_projection() {
        let source = "\\id PSA\n\\c 51\n\\q1\n\\v 8 Make me hear joy and gladness\n\\q2 so that the bones may rejoice.\n\\c 52\n\\p\n\\v 1 Why do you boast?\n";
        // Driven through braid's surface plus this crate's own conversions: a
        // `js_sys::Function` cannot be constructed on a non-wasm target, so the
        // handle's JS-facing constructor is exercised by the JS-side gates instead.
        let mut resident = braid::Braid::new(
            braid::BraidConfig::new(usfm_onion::lint::LintOptions::scoped(
                usfm_onion::lint::LintScope::Book,
            )),
            || "minted".to_string(),
        );
        resident
            .replace_corpus(braid::CorpusInput::new(vec![braid::BookInput::Usfm {
                source_key: braid::SourceKey::new("PSA.usfm").unwrap(),
                book: usfm_onion::token::BookId::from_str("PSA").unwrap(),
                source: source.to_string(),
            }]))
            .expect("one book");

        let resident_entries = match resident
            .vref_index(braid::CorpusScope::Book(
                usfm_onion::token::BookId::from_str("PSA").unwrap(),
            ))
            .expect("resident index")
        {
            braid::ScopedOutput::Single(entries) => entries,
            other => panic!("a book scope returns one value, got {other:?}"),
        };
        let stateless = crate::stateless::wasm_vref_index_usfm(source);
        let projected: Vec<(String, String)> = resident_entries
            .iter()
            .map(|entry| (entry.sid.clone(), entry.projection.text.clone()))
            .collect();
        let expected: Vec<(String, String)> = serde_json::to_value(&stateless)
            .expect("serializes")
            .as_array()
            .expect("ordered pairs")
            .iter()
            .map(|pair| {
                (
                    pair[0].as_str().expect("sid").to_string(),
                    pair[1]["text"].as_str().expect("text").to_string(),
                )
            })
            .collect();
        assert_eq!(projected, expected, "resident must equal stateless");
        // And the seam byte the projection preserves is actually in there, so this
        // is comparing real content and not two empty lists.
        assert!(projected[0].1.contains("gladness\nso that"));
    }

    // --- native <-> wasm parity fixture (Gate 5) ---
    //
    // The Gate 4 tests above prove the DTO path *populates* text_diff and pin
    // a few ASCII runs. This fixture proves the wasm DTO path is byte-identical
    // to the native `unit_text_diff` computation for the HARDENED multilingual
    // pins that `src/diff/text_diff_fixtures.rs` gates (apostrophe, Hebrew
    // niqqud, Arabic RTL, Thai, punctuation-adjacent). The wasm entry point
    // (`wasm_diff_usfm`) and the native reference (`native_diff_usfm` +
    // `native_unit_text_diff`) walk the SAME canonical-sid skeleton, so any
    // drift in `map_native_skeleton`, the `From<Native*>` conversions, or the
    // serde rename would surface here — and it's a plain cargo test, no pkg
    // build. This is the seam the fixture oracle can't reach: those run the
    // native fn directly; these run it through the wasm boundary's DTO path.

    /// The single Modified unit's `text_diff` as produced by the wasm DTO
    /// entry point.
    fn wasm_path_modified_text_diff(
        baseline: &str,
        current: &str,
        mode: &str,
    ) -> Option<UnitTextDiff> {
        let skeleton = wasm_diff_usfm(baseline, current, Some(diff_options(mode)));
        let modified = skeleton
            .units
            .into_iter()
            .filter(|u| matches!(u.status, DecisionStatus::Modified))
            .collect::<Vec<_>>();
        assert_eq!(
            modified.len(),
            1,
            "fixture must have exactly one Modified unit"
        );
        modified.into_iter().next().unwrap().text_diff
    }

    /// The same unit's `text_diff` computed natively (canonical-sid skeleton,
    /// exactly as the wasm path builds it) and converted into the wire DTO.
    fn native_path_modified_text_diff(
        baseline: &str,
        current: &str,
        mode: &str,
    ) -> Option<UnitTextDiff> {
        use usfm_onion::diff::DecisionStatus as NativeDecisionStatus;
        let native_mode = NativeTextDiffMode::from(diff_options(mode));
        let skeleton = native_diff_usfm(baseline, current);
        let modified = skeleton
            .units
            .iter()
            .filter(|u| matches!(u.status, NativeDecisionStatus::Modified))
            .collect::<Vec<_>>();
        assert_eq!(
            modified.len(),
            1,
            "fixture must have exactly one Modified unit"
        );
        native_unit_text_diff(modified[0], native_mode).map(Into::into)
    }

    fn assert_wasm_matches_native(baseline: &str, current: &str, mode: &str, label: &str) {
        let wasm = wasm_path_modified_text_diff(baseline, current, mode);
        let native = native_path_modified_text_diff(baseline, current, mode);
        // Non-trivial: two Nones would pass vacuously, but every pin here is a
        // Modified content change, so both sides MUST carry runs.
        assert!(
            wasm.is_some(),
            "{label}: wasm path produced no text_diff for a Modified unit"
        );
        assert_eq!(
            wasm, native,
            "{label}: wasm DTO text_diff must byte-match the native computation"
        );
    }

    #[test]
    fn parity_apostrophe_straight_vs_curly_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 They don't know.\n",
            "\\id GEN\n\\c 1\n\\v 1 They don\u{2019}t know.\n",
            "words",
            "apostrophe straight vs curly",
        );
    }

    #[test]
    fn parity_hebrew_combining_niqqud_chars() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05E8}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{05D0}\u{05D5}\u{05BC}\u{05E8}\n",
            "chars",
            "Hebrew combining niqqud",
        );
    }

    #[test]
    fn parity_arabic_rtl_word_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0644}\u{0647}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{0641}\u{064A} \u{0627}\u{0644}\u{0628}\u{062F}\u{0621} \u{062E}\u{0644}\u{0642} \u{0627}\u{0644}\u{0631}\u{0628}\n",
            "words",
            "Arabic RTL word edit",
        );
    }

    #[test]
    fn parity_thai_no_space_segment_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E42}\u{0E25}\u{0E01}\n",
            "\\id GEN\n\\c 1\n\\v 1 \u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}\u{0E1B}\u{0E23}\u{0E30}\u{0E40}\u{0E17}\u{0E28}\n",
            "words",
            "Thai no-space segment edit",
        );
    }

    #[test]
    fn parity_latin_punctuation_adjacent_edit_words() {
        assert_wasm_matches_native(
            "\\id GEN\n\\c 1\n\\v 1 Hello, world!\n",
            "\\id GEN\n\\c 1\n\\v 1 Hello, there!\n",
            "words",
            "Latin punctuation-adjacent edit",
        );
    }
}
