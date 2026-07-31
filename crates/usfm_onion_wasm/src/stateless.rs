//! The stateless exports: one-shot functions over caller-owned strings and tokens.
//!
//! Everything here takes its input from the caller and hands the result straight
//! back, reading and mutating no resident state — which is exactly what separates
//! it from the resident handle. Where both surfaces reach the same core function
//! they still stay separate, because "operate on what I gave you" and "operate on
//! what you are holding" are different promises even when the arithmetic matches.

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::dto::*;
use usfm_onion::cst::parse_cst;
use usfm_onion::diff::{
    DiffSkeleton as NativeDiffSkeleton, UnitId as NativeUnitId,
    derive_canonical_sids as native_derive_canonical_sids, diff_skeleton as native_diff_skeleton,
    diff_skeleton_by_chapter as native_diff_skeleton_by_chapter,
    diff_skeleton_canonical as native_diff_skeleton_canonical,
    merge_diff_blocks as native_merge_diff_blocks, revert_diff_block as native_revert_diff_block,
};
use usfm_onion::format::{
    FormatRule as NativeFormatRule, FormatToken as NativeFormatToken,
    format_tokens as native_format_tokens, format_tokens_to_usfm, format_usfm,
};
use usfm_onion::html::usfm_to_html;
use usfm_onion::lint::{apply_token_fix, lint_tokens, lint_usfm};
use usfm_onion::markers::{is_known_marker, marker_catalog, marker_info};
use usfm_onion::parse::parse as native_parse;
use usfm_onion::token::{Token as NativeToken, tokens_to_usfm, tokens_to_usfm_reconstruct};
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::{tokens_to_vref_index, usfm_to_vref_index, usfm_to_vref_map_with_options};
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
use usfm_onion_wire::verify::verify_book as verify_packed_book;

#[wasm_bindgen]
pub struct ParsedUsfm {
    source: String,
}

#[wasm_bindgen]
pub struct UsfmMarkerCatalog;

#[wasm_bindgen]
impl ParsedUsfm {
    fn new(source: String) -> Self {
        Self { source }
    }

    pub fn tokens(&self) -> Vec<Token> {
        let parsed = native_parse(&self.source);
        map_tokens(&parsed.tokens)
    }

    pub fn cst(&self) -> CstDocument {
        let cst = parse_cst(&self.source);
        map_cst_document(&cst)
    }

    pub fn lint(&self, options: LintOptions) -> LintResult {
        map_lint_result(lint_usfm(&self.source, lint_options_into_native(options)))
    }

    #[wasm_bindgen(js_name = applyTokenFix)]
    pub fn apply_token_fix(&self, fix: TokenFix) -> Vec<Token> {
        let parsed = native_parse(&self.source);
        let native_tokens = parsed
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let result = apply_token_fix(&native_tokens, &token_fix_into_native(fix));
        result.iter().map(map_format_token).collect()
    }

    #[wasm_bindgen(js_name = revertDiffBlock)]
    pub fn revert_diff_block(
        &self,
        current: &ParsedUsfm,
        block_id: &str,
    ) -> Result<Vec<Token>, JsError> {
        let baseline = native_parse(&self.source);
        let current = native_parse(&current.source);
        let baseline = baseline
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let current = current
            .tokens
            .iter()
            .map(format_token_with_identity)
            .collect::<Vec<_>>();
        let reverted = native_revert_diff_block(block_id, &baseline, &current).map_err(js_error)?;
        Ok(reverted.iter().map(map_format_token).collect())
    }

    pub fn format(&self, options: Option<FormatOptions>) -> String {
        format_usfm(&self.source, format_options_into_native(options))
    }

    #[wasm_bindgen(js_name = toUsfm)]
    pub fn to_usfm(&self) -> String {
        let parsed = native_parse(&self.source);
        tokens_to_usfm(&parsed.tokens)
    }

    #[wasm_bindgen(js_name = toUsj)]
    pub fn to_usj(&self) -> Result<JsValue, JsError> {
        let document = usfm_to_usj(&self.source).map_err(js_error)?;
        to_js_value(&document)
    }

    #[wasm_bindgen(js_name = toUsx)]
    pub fn to_usx(&self) -> Result<String, JsError> {
        usfm_to_usx(&self.source).map_err(js_error)
    }

    #[wasm_bindgen(js_name = toHtml)]
    pub fn to_html(&self, options: Option<HtmlOptions>) -> String {
        usfm_to_html(&self.source, html_options_into_native(options))
    }

    #[wasm_bindgen(js_name = toVref)]
    pub fn to_vref(&self, options: Option<VrefOptions>) -> VrefMap {
        let options = vref_options_into_native(options);
        VrefMap(vref_to_object(usfm_to_vref_map_with_options(
            &self.source,
            options,
        )))
    }

    #[wasm_bindgen(js_name = vrefIndex)]
    pub fn vref_index(&self) -> VrefIndex {
        map_vref_index(usfm_to_vref_index(&self.source))
    }

    pub fn diff(&self, other: &ParsedUsfm, options: Option<DiffOptions>) -> DiffSkeleton {
        map_native_skeleton(
            &native_diff_usfm(&self.source, &other.source),
            map_token,
            diff_options_into_native(options),
        )
    }

    #[wasm_bindgen(js_name = diffByChapter)]
    pub fn diff_by_chapter(
        &self,
        other: &ParsedUsfm,
        options: Option<DiffOptions>,
    ) -> DiffsByChapterMap {
        DiffsByChapterMap(map_diffs_by_chapter(
            &native_diff_skeleton_by_chapter(&self.source, &other.source),
            map_token,
            diff_options_into_native(options),
        ))
    }
}

#[wasm_bindgen]
impl UsfmMarkerCatalog {
    fn new() -> Self {
        Self
    }

    pub fn all(&self) -> Vec<MarkerInfo> {
        marker_catalog()
            .all()
            .iter()
            .cloned()
            .map(map_marker_info)
            .collect()
    }

    pub fn get(&self, marker: &str) -> Option<MarkerInfo> {
        marker_catalog().get(marker).cloned().map(map_marker_info)
    }

    pub fn contains(&self, marker: &str) -> bool {
        marker_catalog().contains(marker)
    }
}

#[wasm_bindgen(js_name = parse)]
pub fn wasm_parse(source: &str) -> ParsedUsfm {
    ParsedUsfm::new(source.to_string())
}

/// What [`wasm_verify_packed_book`] reports for one record.
///
/// A rejection is a *value*, not a thrown exception: it is an expected outcome
/// (the caller falls back to normal USFM ingest) and it carries the frozen
/// `DecodeError` variant rather than a message a consumer would have to parse.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PackedBookOutcome {
    Verified {
        receipt: PackedBookReceipt,
        findings: Vec<LintIssue>,
    },
    Rejected {
        error: PackedDecodeError,
    },
}

/// The packed trust boundary: verifies one book's container against its exact
/// source and returns the receipt plus that book's findings.
///
/// This runs the whole Rust boundary — container/section structure, both
/// integrity checksums, exact source length and XXH3 content hash, the
/// marker-catalog stamp, every discriminant, index range, and reserved byte.
/// Nothing but tokens is left for the caller to materialize, and no token
/// object crosses this boundary. Findings are materialized here so
/// `LintIssue.message` keeps a single renderer (core's), in a single language.
///
/// `source` is bytes rather than a string so the caller can hand over the same
/// buffer it read from disk without a UTF-16 round trip; non-UTF-8 source is a
/// rejection, not a panic.
#[wasm_bindgen(js_name = verifyPackedBook)]
pub fn wasm_verify_packed_book(packed: &[u8], source: &[u8]) -> PackedBookOutcome {
    let Ok(text) = std::str::from_utf8(source) else {
        return PackedBookOutcome::Rejected {
            error: PackedDecodeError::InvalidUtf8,
        };
    };
    match verify_packed_book(packed, text) {
        Ok(verified) => PackedBookOutcome::Verified {
            receipt: verified.receipt,
            findings: verified.findings.into_iter().map(map_lint_issue).collect(),
        },
        Err(error) => PackedBookOutcome::Rejected {
            error: error.into(),
        },
    }
}

#[wasm_bindgen(js_name = lintUsfm)]
pub fn wasm_lint_usfm(source: &str, options: LintOptions) -> LintResult {
    map_lint_result(lint_usfm(source, lint_options_into_native(options)))
}

#[wasm_bindgen(js_name = vrefIndexUsfm)]
pub fn wasm_vref_index_usfm(source: &str) -> VrefIndex {
    map_vref_index(usfm_to_vref_index(source))
}

/// Build the vref index from an existing token stream (the editor's live
/// path) — same rehydration as `lintTokens`, no reparse. Segment ids match
/// the tokens passed in, so they line up with the editor's DOM `data-id`s.
#[wasm_bindgen(js_name = vrefIndexTokens)]
pub fn wasm_vref_index_tokens(tokens: Vec<Token>) -> VrefIndex {
    let tokens = parse_walk_tokens_from_values(tokens);
    map_vref_index(tokens_to_vref_index(&tokens))
}

#[wasm_bindgen(js_name = lintTokens)]
pub fn wasm_lint_tokens(tokens: Vec<Token>, options: LintOptions) -> LintResult {
    let tokens = parse_walk_tokens_from_values(tokens);
    map_lint_result(lint_tokens(&tokens, lint_options_into_native(options)))
}

#[wasm_bindgen(js_name = applyTokenFix)]
pub fn wasm_apply_token_fix(tokens: Vec<Token>, fix: TokenFix) -> Vec<Token> {
    let native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let result = apply_token_fix(&native_tokens, &token_fix_into_native(fix));
    result.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = formatUsfm)]
pub fn wasm_format_usfm(source: &str, options: Option<FormatOptions>) -> String {
    format_usfm(source, format_options_into_native(options))
}

#[wasm_bindgen(js_name = formatTokens)]
pub fn wasm_format_tokens(tokens: Vec<Token>, options: Option<FormatOptions>) -> FormatResult {
    let mut native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, format_options_into_native(options));
    FormatResult {
        tokens: native_tokens.iter().map(map_format_token).collect(),
        usfm: format_tokens_to_usfm(&native_tokens),
    }
}

#[wasm_bindgen(js_name = formatTokensMut)]
pub fn wasm_format_tokens_mut(tokens: Vec<Token>, options: Option<FormatOptions>) -> Vec<Token> {
    let mut native_tokens = tokens
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    native_format_tokens(&mut native_tokens, format_options_into_native(options));
    native_tokens.iter().map(map_format_token).collect()
}

#[wasm_bindgen(js_name = tokensToUsfm)]
pub fn wasm_tokens_to_usfm(tokens: Vec<Token>) -> String {
    tokens_to_usfm_reconstruct(&tokens)
}

#[wasm_bindgen(js_name = tokensToHtml)]
pub fn wasm_tokens_to_html(tokens: Vec<Token>, options: Option<HtmlOptions>) -> String {
    let usfm = tokens_to_usfm_reconstruct(&tokens);
    usfm_to_html(&usfm, html_options_into_native(options))
}

pub(crate) fn native_diff_usfm<'a>(
    baseline_usfm: &'a str,
    current_usfm: &'a str,
) -> NativeDiffSkeleton<NativeToken<'a>> {
    let baseline = native_parse(baseline_usfm);
    let current = native_parse(current_usfm);
    let baseline_book = baseline.analysis.book_code.unwrap_or("unknown");
    let current_book = current.analysis.book_code.unwrap_or("unknown");
    native_diff_skeleton_canonical(
        &baseline.tokens,
        baseline_book,
        &current.tokens,
        current_book,
    )
}

#[wasm_bindgen(js_name = diffUsfm)]
pub fn wasm_diff_usfm(left: &str, right: &str, options: Option<DiffOptions>) -> DiffSkeleton {
    map_native_skeleton(
        &native_diff_usfm(left, right),
        map_token,
        diff_options_into_native(options),
    )
}

#[wasm_bindgen(js_name = diffUsfmByChapter)]
pub fn wasm_diff_usfm_by_chapter(
    left: &str,
    right: &str,
    options: Option<DiffOptions>,
) -> DiffsByChapterMap {
    DiffsByChapterMap(map_diffs_by_chapter(
        &native_diff_skeleton_by_chapter(left, right),
        map_token,
        diff_options_into_native(options),
    ))
}

#[wasm_bindgen(js_name = diffTokens)]
pub fn wasm_diff_tokens(
    left: Vec<Token>,
    right: Vec<Token>,
    options: Option<DiffOptions>,
) -> DiffSkeleton {
    let left = parse_walk_tokens_from_values(left);
    let right = parse_walk_tokens_from_values(right);
    map_native_skeleton(
        &native_diff_skeleton(&left, &right),
        map_walk_token,
        diff_options_into_native(options),
    )
}

#[wasm_bindgen(js_name = mergeDiffBlocks)]
pub fn wasm_merge_diff_blocks(
    baseline: Vec<Token>,
    current: Vec<Token>,
    request: MergeRequest,
) -> Result<Vec<Token>, JsError> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let decisions = request
        .decisions
        .into_iter()
        .map(|(id, side)| (NativeUnitId::new(id), side.into()))
        .collect();
    let merged =
        native_merge_diff_blocks(&baseline, &current, &decisions, request.default_side.into())
            .map_err(js_error)?;
    Ok(merged.iter().map(map_format_token).collect())
}

#[wasm_bindgen(js_name = revertDiffBlock)]
pub fn wasm_revert_diff_block(
    baseline: Vec<Token>,
    current: Vec<Token>,
    block_id: &str,
) -> Result<Vec<Token>, JsError> {
    let baseline = baseline
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let current = current
        .into_iter()
        .map(token_value_to_format_token)
        .collect::<Vec<_>>();
    let reverted = native_revert_diff_block(block_id, &baseline, &current).map_err(js_error)?;
    Ok(reverted.iter().map(map_format_token).collect())
}

#[wasm_bindgen(js_name = normalizeTokenSids)]
pub fn wasm_normalize_token_sids(tokens: Vec<Token>, book_code: &str) -> Vec<Token> {
    let native_tokens: Vec<NativeFormatToken> = tokens
        .iter()
        .cloned()
        .map(token_value_to_format_token)
        .collect();
    let canonical = native_derive_canonical_sids(&native_tokens, book_code);
    tokens
        .into_iter()
        .zip(canonical)
        .map(|(mut token, sid)| {
            token.sid = Some(sid);
            token
        })
        .collect()
}

#[wasm_bindgen(js_name = markerCatalog)]
pub fn wasm_marker_catalog() -> UsfmMarkerCatalog {
    UsfmMarkerCatalog::new()
}

#[wasm_bindgen(js_name = markerInfo)]
pub fn wasm_marker_info(marker: &str) -> MarkerInfo {
    map_marker_info(marker_info(marker))
}

#[wasm_bindgen(js_name = isKnownMarker)]
pub fn wasm_is_known_marker(marker: &str) -> bool {
    is_known_marker(marker)
}

#[wasm_bindgen(js_name = lintCodes)]
pub fn wasm_lint_codes() -> Vec<LintCode> {
    lint_code_variants()
        .into_iter()
        .map(LintCode::from)
        .collect()
}

#[wasm_bindgen(js_name = lintCodeMeta)]
pub fn wasm_lint_code_meta() -> Vec<LintCodeMeta> {
    lint_code_variants()
        .into_iter()
        .map(|code| LintCodeMeta {
            code: code.into(),
            category: code.category().into(),
            severity: code.severity().into(),
            issue_type: code.issue_type().into(),
        })
        .collect()
}

#[wasm_bindgen(js_name = formatRules)]
pub fn wasm_format_rules() -> Vec<String> {
    NativeFormatRule::ALL
        .iter()
        .map(|rule| rule.code().to_string())
        .collect()
}

#[wasm_bindgen(js_name = formatRuleMeta)]
pub fn wasm_format_rule_meta() -> Vec<FormatRuleMeta> {
    NativeFormatRule::ALL
        .iter()
        .map(|rule| FormatRuleMeta {
            code: rule.code().to_string(),
            label_key: rule.label_key().to_string(),
        })
        .collect()
}
