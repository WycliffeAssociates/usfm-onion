use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value as from_js_value, to_value as to_js_value};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use usfm_onion::{
    convert::{
        HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, HtmlOptions, convert_content,
        document_tree_to_html, document_tree_to_usj, document_tree_to_usx, document_tree_to_vref,
        from_usj, from_usx, into_document_tree, into_html, into_usj, into_usx, into_vref,
        tokens_to_html, tokens_to_usj, tokens_to_usx, tokens_to_vref, usfm_to_html, usfm_to_usj,
        usfm_to_usx, usfm_to_vref,
    },
    diff::{
        BuildSidBlocksOptions, ChapterTokenDiff, DiffStatus, DiffTokenChange, DiffUndoSide,
        DiffsByChapterMap, SidBlock, SidBlockDiff, TokenAlignment, apply_revert_by_block_id,
        apply_reverts_by_block_id, build_sid_blocks, diff_chapter_token_streams, diff_content,
        diff_sid_blocks, diff_tokens, diff_usfm, diff_usfm_by_chapter, diff_usfm_sources,
        diff_usfm_sources_by_chapter, flatten_diff_map, replace_chapter_diffs_in_map,
        replace_many_chapter_diffs_in_map,
    },
    format::{
        self, FormatOptions, SkippedTokenTransform, TokenFix, TokenTemplate, TokenTransformChange,
        TokenTransformKind, TokenTransformResult, TokenTransformSkipReason, apply_token_fixes,
        format_content_with_options, format_flat_token_batches,
        format_flat_token_batches_with_options, format_flat_tokens,
        format_flat_tokens_with_options,
    },
    lint::{
        self, BatchExecutionOptions, LintCode, LintIssue, LintOptions, LintSuppression,
        TokenLintOptions, lint_content, lint_document, lint_document_batch,
        lint_flat_token_batches, lint_flat_tokens,
    },
    model::{
        DocumentFormat, DocumentTreeDocument, ScanResult, ScanToken, ScanTokenKind, Span, Token,
        TokenKind, TokenVariant, TokenViewOptions, UsjDocument, VrefMap, WhitespacePolicy,
    },
    document_tree::{
        document_tree_to_tokens, tokens_to_document_tree, usfm_to_document_tree,
        usj_to_document_tree, usx_to_document_tree,
    },
    parse::{
        self, IntoTokensOptions, ParseHandle, ParseRecovery, ProjectUsfmOptions,
        ProjectedUsfmDocument, RecoveryCode, RecoveryPayload, into_tokens, into_tokens_batch,
        into_tokens_from_content, into_usfm_from_tokens, lex_sources, parse_content,
        parse_contents, parse_sources, project_content, project_document, project_usfm_batch,
        push_whitespace, recoveries,
    },
    tokens::{classify_tokens, usfm_to_token_variants},
};

#[wasm_bindgen(typescript_custom_section)]
const TS_JSON_VALUE_ALIAS: &str = r#"
export type Value =
  | string
  | number
  | boolean
  | null
  | Value[]
  | { [key: string]: Value };
"#;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebDocumentFormat {
    Usfm,
    Usj,
    Usx,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebWhitespacePolicy {
    Preserve,
    MergeToVisible,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebBatchExecutionOptions {
    #[serde(default = "default_parallel_true")]
    pub parallel: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoTokensOptions {
    #[serde(default)]
    pub merge_horizontal_whitespace: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenViewOptions {
    #[serde(default)]
    pub whitespace_policy: Option<WebWhitespacePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintSuppression {
    pub code: String,
    pub sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenLintOptions {
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub suppressions: Vec<WebLintSuppression>,
    #[serde(default)]
    pub allow_implicit_chapter_content_verse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintOptions {
    #[serde(default)]
    pub include_parse_recoveries: bool,
    #[serde(default)]
    pub token_view: Option<WebTokenViewOptions>,
    #[serde(default)]
    pub token_rules: Option<WebTokenLintOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectUsfmOptions {
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
    #[serde(default)]
    pub lint_options: Option<WebLintOptions>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebHtmlNoteMode {
    Extracted,
    Inline,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebHtmlCallerStyle {
    Numeric,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
    Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebHtmlCallerScope {
    DocumentSequential,
    VerseSequential,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebHtmlOptions {
    #[serde(default)]
    pub wrap_root: bool,
    #[serde(default = "default_prefer_native_true")]
    pub prefer_native_elements: bool,
    #[serde(default)]
    pub note_mode: Option<WebHtmlNoteMode>,
    #[serde(default)]
    pub caller_style: Option<WebHtmlCallerStyle>,
    #[serde(default)]
    pub caller_scope: Option<WebHtmlCallerScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebFormatOptions {
    #[serde(default)]
    pub recover_malformed_markers: bool,
    #[serde(default)]
    pub collapse_whitespace_in_text: bool,
    #[serde(default)]
    pub ensure_inline_separators: bool,
    #[serde(default)]
    pub remove_duplicate_verse_numbers: bool,
    #[serde(default)]
    pub normalize_spacing_after_paragraph_markers: bool,
    #[serde(default)]
    pub remove_unwanted_linebreaks: bool,
    #[serde(default)]
    pub bridge_consecutive_verse_markers: bool,
    #[serde(default)]
    pub remove_orphan_empty_verse_before_contentful_verse: bool,
    #[serde(default)]
    pub remove_bridge_verse_enumerators: bool,
    #[serde(default)]
    pub move_chapter_label_after_chapter_marker: bool,
    #[serde(default)]
    pub insert_default_paragraph_after_chapter_intro: bool,
    #[serde(default)]
    pub insert_structural_linebreaks: bool,
    #[serde(default)]
    pub collapse_consecutive_linebreaks: bool,
    #[serde(default)]
    pub normalize_marker_whitespace_at_line_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebBuildSidBlocksOptions {
    #[serde(default = "default_allow_empty_sid_true")]
    pub allow_empty_sid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebToken {
    pub id: String,
    pub kind: String,
    pub span: WebSpan,
    pub sid: Option<String>,
    pub marker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebTokenVariant {
    Newline {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
    OptBreak {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
    Marker {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    EndMarker {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    Milestone {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        marker: String,
        text: String,
    },
    MilestoneEnd {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        marker: Option<String>,
        text: String,
    },
    Attributes {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
    BookCode {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
    Number {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
    Text {
        id: String,
        span: WebSpan,
        sid: Option<String>,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebScanToken {
    pub kind: String,
    pub span: WebSpan,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebScanResult {
    pub tokens: Vec<WebScanToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebParseRecovery {
    pub code: String,
    pub span: WebSpan,
    pub related_span: Option<WebSpan>,
    pub payload: Option<WebRecoveryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebRecoveryPayload {
    Marker { marker: String },
    Close { open: String, close: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebParsedDocument {
    pub source_usfm: String,
    pub book_code: Option<String>,
    pub recoveries: Vec<WebParseRecovery>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintIssue {
    pub code: String,
    pub severity: String,
    pub marker: Option<String>,
    pub message: String,
    pub span: WebSpan,
    pub related_span: Option<WebSpan>,
    pub token_id: Option<String>,
    pub related_token_id: Option<String>,
    pub sid: Option<String>,
    pub fix: Option<WebTokenFix>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectedUsfmDocument {
    pub tokens: Vec<WebToken>,
    pub document_tree: DocumentTreeDocument,
    pub lint_issues: Option<Vec<WebLintIssue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenTemplate {
    pub kind: String,
    pub text: String,
    pub marker: Option<String>,
    pub sid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WebTokenFix {
    ReplaceToken {
        label: String,
        #[serde(rename = "targetTokenId")]
        target_token_id: String,
        replacements: Vec<WebTokenTemplate>,
    },
    InsertAfter {
        label: String,
        #[serde(rename = "targetTokenId")]
        target_token_id: String,
        insert: Vec<WebTokenTemplate>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenTransformChange {
    pub kind: String,
    pub label: String,
    pub target_token_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebSkippedTokenTransform {
    pub kind: String,
    pub label: String,
    pub target_token_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenTransformResult {
    pub tokens: Vec<WebToken>,
    pub applied_changes: Vec<WebTokenTransformChange>,
    pub skipped_changes: Vec<WebSkippedTokenTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebVrefEntry {
    pub reference: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenAlignment {
    pub change: String,
    pub counterpart_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebSidBlock {
    pub block_id: String,
    pub semantic_sid: String,
    pub start: usize,
    pub end_exclusive: usize,
    pub prev_block_id: Option<String>,
    pub text_full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebSidBlockDiff {
    pub block_id: String,
    pub semantic_sid: String,
    pub status: String,
    pub original: Option<WebSidBlock>,
    pub current: Option<WebSidBlock>,
    pub original_text: String,
    pub current_text: String,
    pub original_text_only: String,
    pub current_text_only: String,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebChapterTokenDiff {
    pub block_id: String,
    pub semantic_sid: String,
    pub status: String,
    pub original: Option<WebSidBlock>,
    pub current: Option<WebSidBlock>,
    pub original_text: String,
    pub current_text: String,
    pub original_text_only: String,
    pub current_text_only: String,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
    pub original_tokens: Vec<WebToken>,
    pub current_tokens: Vec<WebToken>,
    pub original_alignment: Vec<WebTokenAlignment>,
    pub current_alignment: Vec<WebTokenAlignment>,
    pub undo_side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebChapterDiffGroup {
    pub book: String,
    pub chapter: u32,
    pub diffs: Vec<WebChapterTokenDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokenBatch {
    pub tokens: Vec<WebToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintBatch {
    pub issues: Vec<WebLintIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebParseContentRequest {
    pub source: String,
    pub format: WebDocumentFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebParseContentsRequest {
    pub sources: Vec<String>,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLexSourcesRequest {
    pub sources: Vec<String>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoTokensRequest {
    pub document: WebParsedDocument,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoTokensFromContentRequest {
    pub source: String,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoTokensFromContentsRequest {
    pub sources: Vec<String>,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoTokensBatchRequest {
    pub documents: Vec<WebParsedDocument>,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectDocumentRequest {
    pub document: WebParsedDocument,
    #[serde(default)]
    pub options: Option<WebProjectUsfmOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectContentRequest {
    pub source: String,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub options: Option<WebProjectUsfmOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectContentsRequest {
    pub sources: Vec<String>,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub options: Option<WebProjectUsfmOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebIntoUsxRequest {
    pub document: WebParsedDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebContentRequest {
    pub source: String,
    pub source_format: WebDocumentFormat,
    pub target_format: WebDocumentFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintContentRequest {
    pub source: String,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub options: Option<WebLintOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintContentsRequest {
    pub sources: Vec<String>,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub options: Option<WebLintOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintDocumentRequest {
    pub document: WebParsedDocument,
    #[serde(default)]
    pub options: Option<WebLintOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintDocumentBatchRequest {
    pub documents: Vec<WebParsedDocument>,
    #[serde(default)]
    pub options: Option<WebLintOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintTokensRequest {
    pub tokens: Vec<WebToken>,
    #[serde(default)]
    pub options: Option<WebTokenLintOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintTokenBatchesRequest {
    pub token_batches: Vec<Vec<WebToken>>,
    #[serde(default)]
    pub options: Option<WebTokenLintOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebFormatContentRequest {
    pub source: String,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
    #[serde(default)]
    pub format_options: Option<WebFormatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebFormatContentsRequest {
    pub sources: Vec<String>,
    pub format: WebDocumentFormat,
    #[serde(default)]
    pub token_options: Option<WebIntoTokensOptions>,
    #[serde(default)]
    pub format_options: Option<WebFormatOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebFormatTokensRequest {
    pub tokens: Vec<WebToken>,
    #[serde(default)]
    pub format_options: Option<WebFormatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebFormatTokenBatchesRequest {
    pub token_batches: Vec<Vec<WebToken>>,
    #[serde(default)]
    pub format_options: Option<WebFormatOptions>,
    #[serde(default)]
    pub batch_options: Option<WebBatchExecutionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebApplyTokenFixesRequest {
    pub tokens: Vec<WebToken>,
    pub fixes: Vec<WebTokenFix>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebDiffContentRequest {
    pub baseline_source: String,
    pub baseline_format: WebDocumentFormat,
    pub current_source: String,
    pub current_format: WebDocumentFormat,
    #[serde(default)]
    pub token_view: Option<WebTokenViewOptions>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebDiffUsfmRequest {
    pub baseline_usfm: String,
    pub current_usfm: String,
    #[serde(default)]
    pub token_view: Option<WebTokenViewOptions>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebDiffTokensRequest {
    pub baseline_tokens: Vec<WebToken>,
    pub current_tokens: Vec<WebToken>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebBuildSidBlocksRequest {
    pub tokens: Vec<WebToken>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebDiffSidBlocksRequest {
    pub baseline_blocks: Vec<WebSidBlock>,
    pub current_blocks: Vec<WebSidBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebDiffChapterTokenStreamsRequest {
    pub baseline_tokens: Vec<WebToken>,
    pub current_tokens: Vec<WebToken>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebRevertDiffBlockRequest {
    pub block_id: String,
    pub baseline_tokens: Vec<WebToken>,
    pub current_tokens: Vec<WebToken>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebApplyRevertsByBlockIdRequest {
    pub diff_block_ids: Vec<String>,
    pub baseline_tokens: Vec<WebToken>,
    pub current_tokens: Vec<WebToken>,
    #[serde(default)]
    pub build_options: Option<WebBuildSidBlocksOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebReplaceChapterDiffsInMapRequest {
    pub groups: Vec<WebChapterDiffGroup>,
    pub book: String,
    pub chapter: u32,
    pub diffs: Vec<WebChapterTokenDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebChapterDiffReplacement {
    pub book: String,
    pub chapter: u32,
    pub diffs: Vec<WebChapterTokenDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebReplaceManyChapterDiffsInMapRequest {
    pub groups: Vec<WebChapterDiffGroup>,
    pub replacements: Vec<WebChapterDiffReplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebStringOpResult {
    pub value: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebParsedOpResult {
    pub value: Option<WebParsedDocument>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTokensOpResult {
    pub value: Option<Vec<WebToken>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebProjectedOpResult {
    pub value: Option<WebProjectedUsfmDocument>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebLintOpResult {
    pub value: Option<Vec<WebLintIssue>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WebTransformOpResult {
    pub value: Option<WebTokenTransformResult>,
    pub error: Option<String>,
}

#[wasm_bindgen(js_name = packageVersion)]
pub fn package_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen(js_name = parseContent)]
/// Parse raw content once and keep the returned document if you plan to project
/// multiple views from it.
pub fn wasm_parse_content(request: WebParseContentRequest) -> Result<WebParsedDocument, JsError> {
    let handle =
        parse_content(&request.source, document_format(request.format)).map_err(js_error)?;
    Ok(map_parse_handle(handle))
}

#[wasm_bindgen(js_name = parseContents)]
pub fn wasm_parse_contents(request: WebParseContentsRequest) -> Vec<WebParsedOpResult> {
    parse_contents(
        request.sources.as_slice(),
        document_format(request.format),
        batch_options(request.batch_options.clone()),
    )
    .into_iter()
    .map(|result| match result {
        Ok(handle) => WebParsedOpResult {
            value: Some(map_parse_handle(handle)),
            error: None,
        },
        Err(error) => WebParsedOpResult {
            value: None,
            error: Some(error.to_string()),
        },
    })
    .collect()
}

#[wasm_bindgen(js_name = parseSources)]
pub fn wasm_parse_sources(request: WebLexSourcesRequest) -> Vec<WebParsedDocument> {
    parse_sources(
        request.sources.as_slice(),
        batch_options(request.batch_options.clone()),
    )
    .into_iter()
    .map(map_parse_handle)
    .collect()
}

#[wasm_bindgen(js_name = lexSources)]
pub fn wasm_lex_sources(request: WebLexSourcesRequest) -> Vec<WebScanResult> {
    lex_sources(
        request.sources.as_slice(),
        batch_options(request.batch_options.clone()),
    )
    .into_iter()
    .map(map_scan_result)
    .collect()
}

#[wasm_bindgen(js_name = intoTokens)]
/// Project a previously parsed document into canonical flat tokens.
pub fn wasm_into_tokens(request: WebIntoTokensRequest) -> Vec<WebToken> {
    let handle = rehydrate_parse_handle(&request.document);
    into_tokens(&handle, into_tokens_options(request.token_options))
        .into_iter()
        .map(map_flat_token)
        .collect()
}

#[wasm_bindgen(js_name = intoTokensFromContent)]
/// Parse raw content and immediately project flat tokens.
///
/// Prefer `parseContent(...)` plus `intoTokens(...)` when you will also lint,
/// format, diff, or project other views from the same source.
pub fn wasm_into_tokens_from_content(
    request: WebIntoTokensFromContentRequest,
) -> Result<Vec<WebToken>, JsError> {
    into_tokens_from_content(
        &request.source,
        document_format(request.format),
        into_tokens_options(request.token_options),
    )
    .map(|tokens| tokens.into_iter().map(map_flat_token).collect())
    .map_err(js_error)
}

#[wasm_bindgen(js_name = intoTokensFromContents)]
pub fn wasm_into_tokens_from_contents(
    request: WebIntoTokensFromContentsRequest,
) -> Vec<WebTokensOpResult> {
    let options = into_tokens_options(request.token_options.clone());
    parse::into_tokens_from_contents(
        request.sources.as_slice(),
        document_format(request.format),
        options,
        batch_options(request.batch_options.clone()),
    )
    .into_iter()
    .map(|result| match result {
        Ok(tokens) => WebTokensOpResult {
            value: Some(tokens.into_iter().map(map_flat_token).collect()),
            error: None,
        },
        Err(error) => WebTokensOpResult {
            value: None,
            error: Some(error.to_string()),
        },
    })
    .collect()
}

#[wasm_bindgen(js_name = intoTokensBatch)]
pub fn wasm_into_tokens_batch(request: WebIntoTokensBatchRequest) -> Vec<WebTokenBatch> {
    let handles = request
        .documents
        .iter()
        .map(rehydrate_parse_handle)
        .collect::<Vec<_>>();
    into_tokens_batch(
        handles.as_slice(),
        into_tokens_options(request.token_options),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(|tokens| WebTokenBatch {
        tokens: tokens.into_iter().map(map_flat_token).collect(),
    })
    .collect()
}

#[wasm_bindgen(js_name = tokensToUsfm)]
pub fn wasm_tokens_to_usfm(tokens: Vec<WebToken>) -> String {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    into_usfm_from_tokens(native.as_slice())
}

#[wasm_bindgen(js_name = usfmToTokens)]
pub fn wasm_usfm_to_tokens(
    content: &str,
    token_options: Option<WebIntoTokensOptions>,
) -> Result<Vec<WebToken>, JsError> {
    into_tokens_from_content(content, DocumentFormat::Usfm, into_tokens_options(token_options))
        .map(|tokens| tokens.into_iter().map(map_flat_token).collect())
        .map_err(js_error)
}

#[wasm_bindgen(js_name = usjToTokens)]
pub fn wasm_usj_to_tokens(
    content: &str,
    token_options: Option<WebIntoTokensOptions>,
) -> Result<Vec<WebToken>, JsError> {
    into_tokens_from_content(content, DocumentFormat::Usj, into_tokens_options(token_options))
        .map(|tokens| tokens.into_iter().map(map_flat_token).collect())
        .map_err(js_error)
}

#[wasm_bindgen(js_name = usxToTokens)]
pub fn wasm_usx_to_tokens(
    content: &str,
    token_options: Option<WebIntoTokensOptions>,
) -> Result<Vec<WebToken>, JsError> {
    into_tokens_from_content(content, DocumentFormat::Usx, into_tokens_options(token_options))
        .map(|tokens| tokens.into_iter().map(map_flat_token).collect())
        .map_err(js_error)
}

#[wasm_bindgen(js_name = classifyTokens)]
pub fn wasm_classify_tokens(tokens: Vec<WebToken>) -> Vec<WebTokenVariant> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    classify_tokens(native.as_slice())
        .into_iter()
        .map(map_token_variant)
        .collect()
}

#[wasm_bindgen(js_name = usfmToTokenVariants)]
pub fn wasm_usfm_to_token_variants(content: &str) -> Vec<WebTokenVariant> {
    usfm_to_token_variants(content)
        .into_iter()
        .map(map_token_variant)
        .collect()
}

#[wasm_bindgen(js_name = pushWhitespace)]
pub fn wasm_push_whitespace(tokens: Vec<WebToken>) -> Vec<WebToken> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    push_whitespace(native.as_slice())
        .into_iter()
        .map(map_flat_token)
        .collect()
}

#[wasm_bindgen(js_name = projectDocument)]
pub fn wasm_project_document(request: WebProjectDocumentRequest) -> WebProjectedUsfmDocument {
    let handle = rehydrate_parse_handle(&request.document);
    map_projected_document(project_document(&handle, project_options(request.options)))
}

#[wasm_bindgen(js_name = projectContent)]
pub fn wasm_project_content(
    request: WebProjectContentRequest,
) -> Result<WebProjectedUsfmDocument, JsError> {
    project_content(
        &request.source,
        document_format(request.format),
        project_options(request.options),
    )
    .map(map_projected_document)
    .map_err(js_error)
}

#[wasm_bindgen(js_name = projectContents)]
pub fn wasm_project_contents(request: WebProjectContentsRequest) -> Vec<WebProjectedOpResult> {
    request
        .sources
        .iter()
        .map(|source| {
            project_content(
                source,
                document_format(request.format),
                project_options(request.options.clone()),
            )
            .map(map_projected_document)
        })
        .map(|result| match result {
            Ok(value) => WebProjectedOpResult {
                value: Some(value),
                error: None,
            },
            Err(error) => WebProjectedOpResult {
                value: None,
                error: Some(error.to_string()),
            },
        })
        .collect()
}

#[wasm_bindgen(js_name = projectUsfmBatch)]
pub fn wasm_project_usfm_batch(
    request: WebProjectContentsRequest,
) -> Vec<WebProjectedUsfmDocument> {
    project_usfm_batch(
        request.sources.as_slice(),
        project_options(request.options),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(map_projected_document)
    .collect()
}

#[wasm_bindgen(js_name = intoUsj)]
pub fn wasm_into_usj(document: WebParsedDocument) -> Result<JsValue, JsError> {
    let handle = rehydrate_parse_handle(&document);
    to_js_value(&into_usj(&handle)).map_err(js_error)
}

#[wasm_bindgen(js_name = intoDocumentTree)]
/// Project a parsed document into the canonical document tree.
pub fn wasm_into_document_tree(document: WebParsedDocument) -> Result<JsValue, JsError> {
    let handle = rehydrate_parse_handle(&document);
    to_js_value(&into_document_tree(&handle)).map_err(js_error)
}

#[wasm_bindgen(js_name = usfmToDocumentTree)]
pub fn wasm_usfm_to_document_tree(content: &str) -> Result<JsValue, JsError> {
    to_js_value(&usfm_to_document_tree(content)).map_err(js_error)
}

#[wasm_bindgen(js_name = usjToDocumentTree)]
pub fn wasm_usj_to_document_tree(content: &str) -> Result<JsValue, JsError> {
    usj_to_document_tree(content)
        .map_err(js_error)
        .and_then(|tree| to_js_value(&tree).map_err(js_error_from_serde))
}

#[wasm_bindgen(js_name = usxToDocumentTree)]
pub fn wasm_usx_to_document_tree(content: &str) -> Result<JsValue, JsError> {
    usx_to_document_tree(content)
        .map_err(js_error)
        .and_then(|tree| to_js_value(&tree).map_err(js_error_from_serde))
}

#[wasm_bindgen(js_name = tokensToDocumentTree)]
pub fn wasm_tokens_to_document_tree(tokens: Vec<WebToken>) -> Result<JsValue, JsError> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    to_js_value(&tokens_to_document_tree(native.as_slice())).map_err(js_error)
}

#[wasm_bindgen(js_name = documentTreeToTokens)]
pub fn wasm_document_tree_to_tokens(document: JsValue) -> Result<Vec<WebToken>, JsError> {
    let document: DocumentTreeDocument = from_js_value(document).map_err(js_error)?;
    document_tree_to_tokens(&document)
        .map(|tokens| tokens.into_iter().map(map_flat_token).collect())
        .map_err(js_error)
}

#[wasm_bindgen(js_name = intoHtml)]
pub fn wasm_into_html(document: WebParsedDocument, options: Option<WebHtmlOptions>) -> String {
    let handle = rehydrate_parse_handle(&document);
    into_html(&handle, html_options(options))
}

#[wasm_bindgen(js_name = intoUsx)]
pub fn wasm_into_usx(request: WebIntoUsxRequest) -> Result<String, JsError> {
    let handle = rehydrate_parse_handle(&request.document);
    into_usx(&handle).map_err(js_error)
}

#[wasm_bindgen(js_name = intoVref)]
pub fn wasm_into_vref(document: WebParsedDocument) -> Vec<WebVrefEntry> {
    let handle = rehydrate_parse_handle(&document);
    map_vref_map(into_vref(&handle))
}

#[wasm_bindgen(js_name = tokensToUsj)]
pub fn wasm_tokens_to_usj(tokens: Vec<WebToken>) -> Result<JsValue, JsError> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    tokens_to_usj(native.as_slice())
        .map_err(js_error)
        .and_then(|document| to_js_value(&document).map_err(js_error_from_serde))
}

#[wasm_bindgen(js_name = tokensToUsx)]
pub fn wasm_tokens_to_usx(tokens: Vec<WebToken>) -> Result<String, JsError> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    tokens_to_usx(native.as_slice()).map_err(js_error)
}

#[wasm_bindgen(js_name = tokensToHtml)]
pub fn wasm_tokens_to_html(
    tokens: Vec<WebToken>,
    options: Option<WebHtmlOptions>,
) -> Result<String, JsError> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    tokens_to_html(native.as_slice(), html_options(options)).map_err(js_error)
}

#[wasm_bindgen(js_name = tokensToVref)]
pub fn wasm_tokens_to_vref(tokens: Vec<WebToken>) -> Result<Vec<WebVrefEntry>, JsError> {
    let native = tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    tokens_to_vref(native.as_slice())
        .map(map_vref_map)
        .map_err(js_error)
}

#[wasm_bindgen(js_name = documentTreeToUsj)]
pub fn wasm_document_tree_to_usj(document: JsValue) -> Result<JsValue, JsError> {
    let document: DocumentTreeDocument = from_js_value(document).map_err(js_error)?;
    document_tree_to_usj(&document)
        .map_err(js_error)
        .and_then(|usj| to_js_value(&usj).map_err(js_error_from_serde))
}

#[wasm_bindgen(js_name = documentTreeToUsx)]
pub fn wasm_document_tree_to_usx(document: JsValue) -> Result<String, JsError> {
    let document: DocumentTreeDocument = from_js_value(document).map_err(js_error)?;
    document_tree_to_usx(&document).map_err(js_error)
}

#[wasm_bindgen(js_name = documentTreeToHtml)]
pub fn wasm_document_tree_to_html(
    document: JsValue,
    options: Option<WebHtmlOptions>,
) -> Result<String, JsError> {
    let document: DocumentTreeDocument = from_js_value(document).map_err(js_error)?;
    document_tree_to_html(&document, html_options(options)).map_err(js_error)
}

#[wasm_bindgen(js_name = documentTreeToVref)]
pub fn wasm_document_tree_to_vref(document: JsValue) -> Result<Vec<WebVrefEntry>, JsError> {
    let document: DocumentTreeDocument = from_js_value(document).map_err(js_error)?;
    document_tree_to_vref(&document)
        .map(map_vref_map)
        .map_err(js_error)
}

#[wasm_bindgen(js_name = fromUsj)]
pub fn wasm_from_usj(document: JsValue) -> Result<String, JsError> {
    let document: UsjDocument = from_js_value(document).map_err(js_error)?;
    from_usj(&document).map_err(js_error)
}

#[wasm_bindgen(js_name = fromUsx)]
pub fn wasm_from_usx(content: &str) -> Result<String, JsError> {
    from_usx(content).map_err(js_error)
}

#[wasm_bindgen(js_name = convertContent)]
pub fn wasm_convert_content(request: WebContentRequest) -> Result<String, JsError> {
    convert_content(
        &request.source,
        document_format(request.source_format),
        document_format(request.target_format),
    )
    .map_err(js_error)
}

#[wasm_bindgen(js_name = usfmToUsj)]
pub fn wasm_usfm_to_usj(content: &str) -> Result<JsValue, JsError> {
    usfm_to_usj(content)
        .map_err(js_error)
        .and_then(|document| to_js_value(&document).map_err(js_error_from_serde))
}

#[wasm_bindgen(js_name = usfmToUsx)]
pub fn wasm_usfm_to_usx(content: &str) -> Result<String, JsError> {
    usfm_to_usx(content).map_err(js_error)
}

#[wasm_bindgen(js_name = usfmToHtml)]
pub fn wasm_usfm_to_html(content: &str, options: Option<WebHtmlOptions>) -> Result<String, JsError> {
    usfm_to_html(content, html_options(options)).map_err(js_error)
}

#[wasm_bindgen(js_name = usfmToVref)]
pub fn wasm_usfm_to_vref(content: &str) -> Result<Vec<WebVrefEntry>, JsError> {
    usfm_to_vref(content).map(map_vref_map).map_err(js_error)
}

#[wasm_bindgen(js_name = usjToUsfm)]
pub fn wasm_usj_to_usfm(content: &str) -> Result<String, JsError> {
    wasm_convert_content(WebContentRequest {
        source: content.to_string(),
        source_format: WebDocumentFormat::Usj,
        target_format: WebDocumentFormat::Usfm,
    })
}

#[wasm_bindgen(js_name = usxToUsfm)]
pub fn wasm_usx_to_usfm(content: &str) -> Result<String, JsError> {
    wasm_convert_content(WebContentRequest {
        source: content.to_string(),
        source_format: WebDocumentFormat::Usx,
        target_format: WebDocumentFormat::Usfm,
    })
}

#[wasm_bindgen(js_name = lintContent)]
/// Parse content, project tokens, then lint.
///
/// If you already have canonical flat tokens, prefer `lintFlatTokens(...)`.
pub fn wasm_lint_content(request: WebLintContentRequest) -> Result<Vec<WebLintIssue>, JsError> {
    lint_content(
        &request.source,
        document_format(request.format),
        lint_options(request.options),
    )
    .map(|issues| issues.into_iter().map(map_lint_issue).collect())
    .map_err(js_error)
}

#[wasm_bindgen(js_name = lintContents)]
pub fn wasm_lint_contents(request: WebLintContentsRequest) -> Vec<WebLintOpResult> {
    lint::lint_contents(
        request.sources.as_slice(),
        document_format(request.format),
        lint_options(request.options.clone()),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(|result| match result {
        Ok(value) => WebLintOpResult {
            value: Some(value.into_iter().map(map_lint_issue).collect()),
            error: None,
        },
        Err(error) => WebLintOpResult {
            value: None,
            error: Some(error.to_string()),
        },
    })
    .collect()
}

#[wasm_bindgen(js_name = lintDocument)]
pub fn wasm_lint_document(request: WebLintDocumentRequest) -> Vec<WebLintIssue> {
    let handle = rehydrate_parse_handle(&request.document);
    lint_document(&handle, lint_options(request.options))
        .into_iter()
        .map(map_lint_issue)
        .collect()
}

#[wasm_bindgen(js_name = lintDocumentBatch)]
pub fn wasm_lint_document_batch(request: WebLintDocumentBatchRequest) -> Vec<WebLintBatch> {
    let handles = request
        .documents
        .iter()
        .map(rehydrate_parse_handle)
        .collect::<Vec<_>>();
    lint_document_batch(
        handles.as_slice(),
        lint_options(request.options),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(|issues| WebLintBatch {
        issues: issues.into_iter().map(map_lint_issue).collect(),
    })
    .collect()
}

#[wasm_bindgen(js_name = lintFlatTokens)]
/// Lint canonical flat tokens without reparsing source content.
pub fn wasm_lint_flat_tokens(request: WebLintTokensRequest) -> Vec<WebLintIssue> {
    let tokens = request
        .tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    lint_flat_tokens(tokens.as_slice(), token_lint_options(request.options))
        .into_iter()
        .map(map_lint_issue)
        .collect()
}

#[wasm_bindgen(js_name = lintTokenBatches)]
/// Lint batches of canonical flat token streams without reparsing source content.
pub fn wasm_lint_flat_token_batches(request: WebLintTokenBatchesRequest) -> Vec<WebLintBatch> {
    let batches = request
        .token_batches
        .into_iter()
        .map(|batch| {
            batch
                .into_iter()
                .map(from_web_flat_token)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    lint_flat_token_batches(
        batches.as_slice(),
        token_lint_options(request.options),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(|issues| WebLintBatch {
        issues: issues.into_iter().map(map_lint_issue).collect(),
    })
    .collect()
}

#[wasm_bindgen(js_name = formatContent)]
/// Parse content, project tokens, then run the formatter.
///
/// If you already have canonical flat tokens, prefer `formatFlatTokens(...)`.
pub fn wasm_format_content(
    request: WebFormatContentRequest,
) -> Result<WebTokenTransformResult, JsError> {
    format_content_with_options(
        &request.source,
        document_format(request.format),
        into_tokens_options(request.token_options),
        format_options(request.format_options),
    )
    .map(map_transform_result)
    .map_err(js_error)
}

#[wasm_bindgen(js_name = formatContents)]
pub fn wasm_format_contents(request: WebFormatContentsRequest) -> Vec<WebTransformOpResult> {
    format::format_contents_with_options(
        request.sources.as_slice(),
        document_format(request.format),
        into_tokens_options(request.token_options.clone()),
        format_options(request.format_options.clone()),
        batch_options(request.batch_options),
    )
    .into_iter()
    .map(|result| match result {
        Ok(value) => WebTransformOpResult {
            value: Some(map_transform_result(value)),
            error: None,
        },
        Err(error) => WebTransformOpResult {
            value: None,
            error: Some(error.to_string()),
        },
    })
    .collect()
}

#[wasm_bindgen(js_name = formatFlatTokens)]
/// Format canonical flat tokens without reparsing source content.
pub fn wasm_format_flat_tokens(request: WebFormatTokensRequest) -> WebTokenTransformResult {
    let tokens = request
        .tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let result = match request.format_options {
        Some(options) => {
            format_flat_tokens_with_options(tokens.as_slice(), format_options(Some(options)))
        }
        None => format_flat_tokens(tokens.as_slice()),
    };
    map_transform_result(result)
}

#[wasm_bindgen(js_name = formatTokenBatches)]
/// Format batches of canonical flat token streams without reparsing source content.
pub fn wasm_format_flat_token_batches(
    request: WebFormatTokenBatchesRequest,
) -> Vec<WebTokenTransformResult> {
    let batches = request
        .token_batches
        .into_iter()
        .map(|batch| {
            batch
                .into_iter()
                .map(from_web_flat_token)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let results = match request.format_options {
        Some(options) => format_flat_token_batches_with_options(
            batches.as_slice(),
            format_options(Some(options)),
            batch_options(request.batch_options),
        ),
        None => format_flat_token_batches(batches.as_slice(), batch_options(request.batch_options)),
    };
    results.into_iter().map(map_transform_result).collect()
}

#[wasm_bindgen(js_name = applyTokenFixes)]
pub fn wasm_apply_token_fixes(request: WebApplyTokenFixesRequest) -> WebTokenTransformResult {
    let tokens = request
        .tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let fixes = request
        .fixes
        .into_iter()
        .map(from_web_token_fix)
        .collect::<Vec<_>>();
    map_transform_result(apply_token_fixes(tokens.as_slice(), fixes.as_slice()))
}

#[wasm_bindgen(js_name = diffContent)]
/// Parse both sources, project canonical flat tokens, then diff.
///
/// If you already have canonical flat tokens, prefer `diffTokens(...)`.
pub fn wasm_diff_content(
    request: WebDiffContentRequest,
) -> Result<Vec<WebChapterTokenDiff>, JsError> {
    diff_content(
        &request.baseline_source,
        document_format(request.baseline_format),
        &request.current_source,
        document_format(request.current_format),
        &token_view_options(request.token_view),
        &build_sid_blocks_options(request.build_options),
    )
    .map(|diffs| diffs.into_iter().map(map_chapter_token_diff).collect())
    .map_err(js_error)
}

#[wasm_bindgen(js_name = diffTokens)]
/// Diff canonical flat token streams without reparsing source content.
pub fn wasm_diff_tokens(request: WebDiffTokensRequest) -> Vec<WebChapterTokenDiff> {
    let baseline = request
        .baseline_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let current = request
        .current_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    diff_tokens(
        baseline.as_slice(),
        current.as_slice(),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_chapter_token_diff)
    .collect()
}


#[wasm_bindgen(js_name = buildSidBlocks)]
pub fn wasm_build_sid_blocks(request: WebBuildSidBlocksRequest) -> Vec<WebSidBlock> {
    let tokens = request
        .tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    build_sid_blocks(
        tokens.as_slice(),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_sid_block)
    .collect()
}

#[wasm_bindgen(js_name = diffSidBlocks)]
pub fn wasm_diff_sid_blocks(request: WebDiffSidBlocksRequest) -> Vec<WebSidBlockDiff> {
    let baseline = request
        .baseline_blocks
        .into_iter()
        .map(from_web_sid_block)
        .collect::<Vec<_>>();
    let current = request
        .current_blocks
        .into_iter()
        .map(from_web_sid_block)
        .collect::<Vec<_>>();
    diff_sid_blocks(baseline.as_slice(), current.as_slice())
        .into_iter()
        .map(map_sid_block_diff)
        .collect()
}

#[wasm_bindgen(js_name = diffChapterTokenStreams)]
pub fn wasm_diff_chapter_token_streams(
    request: WebDiffChapterTokenStreamsRequest,
) -> Vec<WebChapterTokenDiff> {
    let baseline = request
        .baseline_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let current = request
        .current_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    diff_chapter_token_streams(
        baseline.as_slice(),
        current.as_slice(),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_chapter_token_diff)
    .collect()
}

#[wasm_bindgen(js_name = diffUsfm)]
pub fn wasm_diff_usfm(request: WebDiffUsfmRequest) -> Vec<WebChapterTokenDiff> {
    diff_usfm(
        &request.baseline_usfm,
        &request.current_usfm,
        &token_view_options(request.token_view),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_chapter_token_diff)
    .collect()
}

#[wasm_bindgen(js_name = diffUsfmByChapter)]
pub fn wasm_diff_usfm_by_chapter(request: WebDiffUsfmRequest) -> Vec<WebChapterDiffGroup> {
    let grouped = diff_usfm_by_chapter(
        &request.baseline_usfm,
        &request.current_usfm,
        &token_view_options(request.token_view),
        &build_sid_blocks_options(request.build_options),
    );
    map_diff_groups(grouped)
}

#[wasm_bindgen(js_name = diffUsfmSources)]
pub fn wasm_diff_usfm_sources(request: WebDiffUsfmRequest) -> Vec<WebChapterTokenDiff> {
    diff_usfm_sources(
        &request.baseline_usfm,
        &request.current_usfm,
        &token_view_options(request.token_view),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_chapter_token_diff)
    .collect()
}

#[wasm_bindgen(js_name = diffUsfmSourcesByChapter)]
pub fn wasm_diff_usfm_sources_by_chapter(request: WebDiffUsfmRequest) -> Vec<WebChapterDiffGroup> {
    map_diff_groups(diff_usfm_sources_by_chapter(
        &request.baseline_usfm,
        &request.current_usfm,
        &token_view_options(request.token_view),
        &build_sid_blocks_options(request.build_options),
    ))
}

#[wasm_bindgen(js_name = applyRevertByBlockId)]
pub fn wasm_apply_revert_by_block_id(request: WebRevertDiffBlockRequest) -> Vec<WebToken> {
    let baseline = request
        .baseline_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let current = request
        .current_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    apply_revert_by_block_id(
        &request.block_id,
        baseline.as_slice(),
        current.as_slice(),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_flat_token)
    .collect()
}

#[wasm_bindgen(js_name = revertDiffBlock)]
pub fn wasm_revert_diff_block(request: WebRevertDiffBlockRequest) -> Vec<WebToken> {
    wasm_apply_revert_by_block_id(request)
}

#[wasm_bindgen(js_name = applyRevertsByBlockId)]
pub fn wasm_apply_reverts_by_block_id(request: WebApplyRevertsByBlockIdRequest) -> Vec<WebToken> {
    let baseline = request
        .baseline_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    let current = request
        .current_tokens
        .into_iter()
        .map(from_web_flat_token)
        .collect::<Vec<_>>();
    apply_reverts_by_block_id(
        request.diff_block_ids.as_slice(),
        baseline.as_slice(),
        current.as_slice(),
        &build_sid_blocks_options(request.build_options),
    )
    .into_iter()
    .map(map_flat_token)
    .collect()
}

#[wasm_bindgen(js_name = revertDiffBlocks)]
pub fn wasm_revert_diff_blocks(request: WebApplyRevertsByBlockIdRequest) -> Vec<WebToken> {
    wasm_apply_reverts_by_block_id(request)
}

#[wasm_bindgen(js_name = replaceChapterDiffsInMap)]
pub fn wasm_replace_chapter_diffs_in_map(
    request: WebReplaceChapterDiffsInMapRequest,
) -> Vec<WebChapterDiffGroup> {
    let map = from_web_diff_groups(request.groups);
    map_diff_groups(replace_chapter_diffs_in_map(
        &map,
        &request.book,
        request.chapter,
        request
            .diffs
            .into_iter()
            .map(from_web_chapter_token_diff)
            .collect(),
    ))
}

#[wasm_bindgen(js_name = replaceManyChapterDiffsInMap)]
pub fn wasm_replace_many_chapter_diffs_in_map(
    request: WebReplaceManyChapterDiffsInMapRequest,
) -> Vec<WebChapterDiffGroup> {
    let map = from_web_diff_groups(request.groups);
    let replacements = request
        .replacements
        .into_iter()
        .map(|entry| {
            (
                entry.book,
                entry.chapter,
                entry
                    .diffs
                    .into_iter()
                    .map(from_web_chapter_token_diff)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    map_diff_groups(replace_many_chapter_diffs_in_map(
        &map,
        replacements.as_slice(),
    ))
}

#[wasm_bindgen(js_name = flattenDiffMap)]
pub fn wasm_flatten_diff_map(groups: Vec<WebChapterDiffGroup>) -> Vec<WebChapterTokenDiff> {
    let map = from_web_diff_groups(groups);
    flatten_diff_map(&map)
        .into_iter()
        .map(map_chapter_token_diff)
        .collect()
}

fn rehydrate_parse_handle(document: &WebParsedDocument) -> ParseHandle {
    parse::parse(&document.source_usfm)
}

fn map_parse_handle(handle: ParseHandle) -> WebParsedDocument {
    WebParsedDocument {
        source_usfm: handle.source().to_string(),
        book_code: handle.book_code().map(ToString::to_string),
        recoveries: recoveries(&handle)
            .iter()
            .cloned()
            .map(map_parse_recovery)
            .collect(),
    }
}

fn map_scan_result(result: ScanResult) -> WebScanResult {
    WebScanResult {
        tokens: result.tokens.into_iter().map(map_scan_token).collect(),
    }
}

fn map_scan_token(token: ScanToken) -> WebScanToken {
    WebScanToken {
        kind: scan_token_kind_name(&token.kind).to_string(),
        span: map_span(token.span),
        text: token.text,
    }
}

fn map_parse_recovery(recovery: ParseRecovery) -> WebParseRecovery {
    WebParseRecovery {
        code: recovery_code_name(&recovery.code).to_string(),
        span: map_span(recovery.span),
        related_span: recovery.related_span.map(map_span),
        payload: recovery.payload.map(map_recovery_payload),
    }
}

fn map_recovery_payload(payload: RecoveryPayload) -> WebRecoveryPayload {
    match payload {
        RecoveryPayload::Marker { marker } => WebRecoveryPayload::Marker { marker },
        RecoveryPayload::Close { open, close } => WebRecoveryPayload::Close { open, close },
    }
}

fn map_flat_token(token: Token) -> WebToken {
    WebToken {
        id: token.id,
        kind: token_kind_name(&token.kind).to_string(),
        span: map_span(token.span),
        sid: token.sid,
        marker: token.marker,
        text: token.text,
    }
}

fn map_token_variant(token: TokenVariant) -> WebTokenVariant {
    match token {
        TokenVariant::Newline { id, span, sid, text } => WebTokenVariant::Newline {
            id,
            span: map_span(span),
            sid,
            text,
        },
        TokenVariant::OptBreak { id, span, sid, text } => WebTokenVariant::OptBreak {
            id,
            span: map_span(span),
            sid,
            text,
        },
        TokenVariant::Marker {
            id,
            span,
            sid,
            marker,
            text,
        } => WebTokenVariant::Marker {
            id,
            span: map_span(span),
            sid,
            marker,
            text,
        },
        TokenVariant::EndMarker {
            id,
            span,
            sid,
            marker,
            text,
        } => WebTokenVariant::EndMarker {
            id,
            span: map_span(span),
            sid,
            marker,
            text,
        },
        TokenVariant::Milestone {
            id,
            span,
            sid,
            marker,
            text,
        } => WebTokenVariant::Milestone {
            id,
            span: map_span(span),
            sid,
            marker,
            text,
        },
        TokenVariant::MilestoneEnd {
            id,
            span,
            sid,
            marker,
            text,
        } => WebTokenVariant::MilestoneEnd {
            id,
            span: map_span(span),
            sid,
            marker,
            text,
        },
        TokenVariant::Attributes { id, span, sid, text } => WebTokenVariant::Attributes {
            id,
            span: map_span(span),
            sid,
            text,
        },
        TokenVariant::BookCode { id, span, sid, text } => WebTokenVariant::BookCode {
            id,
            span: map_span(span),
            sid,
            text,
        },
        TokenVariant::Number { id, span, sid, text } => WebTokenVariant::Number {
            id,
            span: map_span(span),
            sid,
            text,
        },
        TokenVariant::Text { id, span, sid, text } => WebTokenVariant::Text {
            id,
            span: map_span(span),
            sid,
            text,
        },
    }
}

fn from_web_flat_token(token: WebToken) -> Token {
    Token {
        id: token.id,
        kind: parse_token_kind(&token.kind),
        span: token.span.start..token.span.end,
        sid: token.sid,
        marker: token.marker,
        text: token.text,
    }
}

fn map_lint_issue(issue: LintIssue) -> WebLintIssue {
    WebLintIssue {
        code: issue.code.as_str().to_string(),
        severity: issue.severity.as_str().to_string(),
        marker: issue.marker,
        message: issue.message,
        span: map_span(issue.span),
        related_span: issue.related_span.map(map_span),
        token_id: issue.token_id,
        related_token_id: issue.related_token_id,
        sid: issue.sid,
        fix: issue.fix.map(map_token_fix),
    }
}

fn map_token_fix(fix: TokenFix) -> WebTokenFix {
    match fix {
        TokenFix::ReplaceToken {
            label,
            target_token_id,
            replacements,
        } => WebTokenFix::ReplaceToken {
            label,
            target_token_id,
            replacements: replacements.into_iter().map(map_token_template).collect(),
        },
        TokenFix::InsertAfter {
            label,
            target_token_id,
            insert,
        } => WebTokenFix::InsertAfter {
            label,
            target_token_id,
            insert: insert.into_iter().map(map_token_template).collect(),
        },
    }
}

fn map_token_template(template: TokenTemplate) -> WebTokenTemplate {
    WebTokenTemplate {
        kind: token_kind_name(&template.kind).to_string(),
        text: template.text,
        marker: template.marker,
        sid: template.sid,
    }
}

fn map_projected_document(document: ProjectedUsfmDocument) -> WebProjectedUsfmDocument {
    WebProjectedUsfmDocument {
        tokens: document.tokens.into_iter().map(map_flat_token).collect(),
        document_tree: document.document_tree,
        lint_issues: document
            .lint_issues
            .map(|issues| issues.into_iter().map(map_lint_issue).collect()),
    }
}

fn map_transform_result(result: TokenTransformResult<Token>) -> WebTokenTransformResult {
    WebTokenTransformResult {
        tokens: result.tokens.into_iter().map(map_flat_token).collect(),
        applied_changes: result
            .applied_changes
            .into_iter()
            .map(map_transform_change)
            .collect(),
        skipped_changes: result
            .skipped_changes
            .into_iter()
            .map(map_skipped_transform)
            .collect(),
    }
}

fn map_transform_change(change: TokenTransformChange) -> WebTokenTransformChange {
    WebTokenTransformChange {
        kind: transform_kind_name(&change.kind).to_string(),
        label: change.label,
        target_token_id: change.target_token_id,
    }
}

fn map_skipped_transform(skip: SkippedTokenTransform) -> WebSkippedTokenTransform {
    WebSkippedTokenTransform {
        kind: transform_kind_name(&skip.kind).to_string(),
        label: skip.label,
        target_token_id: skip.target_token_id,
        reason: transform_skip_reason_name(&skip.reason).to_string(),
    }
}

fn map_vref_map(map: VrefMap) -> Vec<WebVrefEntry> {
    map.into_iter()
        .map(|(reference, text)| WebVrefEntry { reference, text })
        .collect()
}

fn map_chapter_token_diff(diff: ChapterTokenDiff<Token>) -> WebChapterTokenDiff {
    WebChapterTokenDiff {
        block_id: diff.block_id,
        semantic_sid: diff.semantic_sid,
        status: diff_status_name(diff.status).to_string(),
        original: diff.original.map(map_sid_block),
        current: diff.current.map(map_sid_block),
        original_text: diff.original_text,
        current_text: diff.current_text,
        original_text_only: diff.original_text_only,
        current_text_only: diff.current_text_only,
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens: diff
            .original_tokens
            .into_iter()
            .map(map_flat_token)
            .collect(),
        current_tokens: diff
            .current_tokens
            .into_iter()
            .map(map_flat_token)
            .collect(),
        original_alignment: diff
            .original_alignment
            .into_iter()
            .map(map_token_alignment)
            .collect(),
        current_alignment: diff
            .current_alignment
            .into_iter()
            .map(map_token_alignment)
            .collect(),
        undo_side: diff_undo_side_name(diff.undo_side).to_string(),
    }
}

fn map_sid_block_diff(diff: SidBlockDiff) -> WebSidBlockDiff {
    WebSidBlockDiff {
        block_id: diff.block_id,
        semantic_sid: diff.semantic_sid,
        status: diff_status_name(diff.status).to_string(),
        original: diff.original.map(map_sid_block),
        current: diff.current.map(map_sid_block),
        original_text: diff.original_text,
        current_text: diff.current_text,
        original_text_only: diff.original_text_only,
        current_text_only: diff.current_text_only,
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
    }
}

fn map_sid_block(block: SidBlock) -> WebSidBlock {
    WebSidBlock {
        block_id: block.block_id,
        semantic_sid: block.semantic_sid,
        start: block.start,
        end_exclusive: block.end_exclusive,
        prev_block_id: block.prev_block_id,
        text_full: block.text_full,
    }
}

fn from_web_sid_block(block: WebSidBlock) -> SidBlock {
    SidBlock {
        block_id: block.block_id,
        semantic_sid: block.semantic_sid,
        start: block.start,
        end_exclusive: block.end_exclusive,
        prev_block_id: block.prev_block_id,
        text_full: block.text_full,
    }
}

fn map_token_alignment(alignment: TokenAlignment) -> WebTokenAlignment {
    WebTokenAlignment {
        change: diff_token_change_name(alignment.change).to_string(),
        counterpart_index: alignment.counterpart_index,
    }
}

fn map_diff_groups(groups: DiffsByChapterMap<ChapterTokenDiff<Token>>) -> Vec<WebChapterDiffGroup> {
    let mut out = Vec::new();
    for (book, chapters) in groups {
        for (chapter, diffs) in chapters {
            out.push(WebChapterDiffGroup {
                book: book.clone(),
                chapter,
                diffs: diffs.into_iter().map(map_chapter_token_diff).collect(),
            });
        }
    }
    out
}

fn from_web_diff_groups(
    groups: Vec<WebChapterDiffGroup>,
) -> DiffsByChapterMap<ChapterTokenDiff<Token>> {
    let mut out = DiffsByChapterMap::new();
    for group in groups {
        out.entry(group.book).or_default().insert(
            group.chapter,
            group
                .diffs
                .into_iter()
                .map(from_web_chapter_token_diff)
                .collect(),
        );
    }
    out
}

fn from_web_chapter_token_diff(diff: WebChapterTokenDiff) -> ChapterTokenDiff<Token> {
    ChapterTokenDiff {
        block_id: diff.block_id,
        semantic_sid: diff.semantic_sid,
        status: parse_diff_status(&diff.status),
        original: diff.original.map(from_web_sid_block),
        current: diff.current.map(from_web_sid_block),
        original_text: diff.original_text,
        current_text: diff.current_text,
        original_text_only: diff.original_text_only,
        current_text_only: diff.current_text_only,
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens: diff
            .original_tokens
            .into_iter()
            .map(from_web_flat_token)
            .collect(),
        current_tokens: diff
            .current_tokens
            .into_iter()
            .map(from_web_flat_token)
            .collect(),
        original_alignment: diff
            .original_alignment
            .into_iter()
            .map(from_web_token_alignment)
            .collect(),
        current_alignment: diff
            .current_alignment
            .into_iter()
            .map(from_web_token_alignment)
            .collect(),
        undo_side: parse_diff_undo_side(&diff.undo_side),
    }
}

fn from_web_token_alignment(alignment: WebTokenAlignment) -> TokenAlignment {
    TokenAlignment {
        change: parse_diff_token_change(&alignment.change),
        counterpart_index: alignment.counterpart_index,
    }
}

fn from_web_token_fix(fix: WebTokenFix) -> TokenFix {
    match fix {
        WebTokenFix::ReplaceToken {
            label,
            target_token_id,
            replacements,
        } => TokenFix::ReplaceToken {
            label,
            target_token_id,
            replacements: replacements
                .into_iter()
                .map(from_web_token_template)
                .collect(),
        },
        WebTokenFix::InsertAfter {
            label,
            target_token_id,
            insert,
        } => TokenFix::InsertAfter {
            label,
            target_token_id,
            insert: insert.into_iter().map(from_web_token_template).collect(),
        },
    }
}

fn from_web_token_template(template: WebTokenTemplate) -> TokenTemplate {
    TokenTemplate {
        kind: parse_token_kind(&template.kind),
        text: template.text,
        marker: template.marker,
        sid: template.sid,
    }
}

fn map_span(span: Span) -> WebSpan {
    WebSpan {
        start: span.start,
        end: span.end,
    }
}

fn document_format(format: WebDocumentFormat) -> DocumentFormat {
    match format {
        WebDocumentFormat::Usfm => DocumentFormat::Usfm,
        WebDocumentFormat::Usj => DocumentFormat::Usj,
        WebDocumentFormat::Usx => DocumentFormat::Usx,
    }
}

fn batch_options(options: Option<WebBatchExecutionOptions>) -> BatchExecutionOptions {
    let options = options.unwrap_or(WebBatchExecutionOptions {
        parallel: default_parallel_true(),
    });
    BatchExecutionOptions {
        parallel: options.parallel,
    }
}

fn into_tokens_options(options: Option<WebIntoTokensOptions>) -> IntoTokensOptions {
    let options = options.unwrap_or_default();
    IntoTokensOptions {
        merge_horizontal_whitespace: options.merge_horizontal_whitespace,
    }
}

fn token_view_options(options: Option<WebTokenViewOptions>) -> TokenViewOptions {
    let policy = options
        .and_then(|options| options.whitespace_policy)
        .unwrap_or(WebWhitespacePolicy::Preserve);
    TokenViewOptions {
        whitespace_policy: match policy {
            WebWhitespacePolicy::Preserve => WhitespacePolicy::MergeToVisible,
            WebWhitespacePolicy::MergeToVisible => WhitespacePolicy::MergeToVisible,
        },
    }
}

fn html_options(options: Option<WebHtmlOptions>) -> HtmlOptions {
    let options = options.unwrap_or_default();
    HtmlOptions {
        wrap_root: options.wrap_root,
        prefer_native_elements: options.prefer_native_elements,
        note_mode: match options.note_mode.unwrap_or(WebHtmlNoteMode::Extracted) {
            WebHtmlNoteMode::Extracted => HtmlNoteMode::Extracted,
            WebHtmlNoteMode::Inline => HtmlNoteMode::Inline,
        },
        caller_style: match options.caller_style.unwrap_or(WebHtmlCallerStyle::Numeric) {
            WebHtmlCallerStyle::Numeric => HtmlCallerStyle::Numeric,
            WebHtmlCallerStyle::AlphaLower => HtmlCallerStyle::AlphaLower,
            WebHtmlCallerStyle::AlphaUpper => HtmlCallerStyle::AlphaUpper,
            WebHtmlCallerStyle::RomanLower => HtmlCallerStyle::RomanLower,
            WebHtmlCallerStyle::RomanUpper => HtmlCallerStyle::RomanUpper,
            WebHtmlCallerStyle::Source => HtmlCallerStyle::Source,
        },
        caller_scope: match options
            .caller_scope
            .unwrap_or(WebHtmlCallerScope::VerseSequential)
        {
            WebHtmlCallerScope::DocumentSequential => HtmlCallerScope::DocumentSequential,
            WebHtmlCallerScope::VerseSequential => HtmlCallerScope::VerseSequential,
        },
    }
}

fn token_lint_options(options: Option<WebTokenLintOptions>) -> TokenLintOptions {
    let options = options.unwrap_or(WebTokenLintOptions {
        disabled_rules: Vec::new(),
        suppressions: Vec::new(),
        allow_implicit_chapter_content_verse: false,
    });
    TokenLintOptions {
        disabled_rules: options
            .disabled_rules
            .iter()
            .filter_map(|code| parse_lint_code(code))
            .collect(),
        suppressions: options
            .suppressions
            .into_iter()
            .filter_map(|suppression| {
                parse_lint_code(&suppression.code).map(|code| LintSuppression {
                    code,
                    sid: suppression.sid,
                })
            })
            .collect(),
        allow_implicit_chapter_content_verse: options.allow_implicit_chapter_content_verse,
    }
}

fn lint_options(options: Option<WebLintOptions>) -> LintOptions {
    let options = options.unwrap_or(WebLintOptions {
        include_parse_recoveries: false,
        token_view: None,
        token_rules: None,
    });
    LintOptions {
        include_parse_recoveries: options.include_parse_recoveries,
        token_view: token_view_options(options.token_view),
        token_rules: token_lint_options(options.token_rules),
    }
}

fn project_options(options: Option<WebProjectUsfmOptions>) -> ProjectUsfmOptions {
    let options = options.unwrap_or(WebProjectUsfmOptions {
        token_options: None,
        lint_options: None,
    });
    ProjectUsfmOptions {
        token_options: into_tokens_options(options.token_options),
        lint_options: options
            .lint_options
            .map(|options| lint_options(Some(options))),
    }
}

fn format_options(options: Option<WebFormatOptions>) -> FormatOptions {
    if let Some(options) = options {
        FormatOptions {
            recover_malformed_markers: options.recover_malformed_markers,
            collapse_whitespace_in_text: options.collapse_whitespace_in_text,
            ensure_inline_separators: options.ensure_inline_separators,
            remove_duplicate_verse_numbers: options.remove_duplicate_verse_numbers,
            normalize_spacing_after_paragraph_markers: options
                .normalize_spacing_after_paragraph_markers,
            remove_unwanted_linebreaks: options.remove_unwanted_linebreaks,
            bridge_consecutive_verse_markers: options.bridge_consecutive_verse_markers,
            remove_orphan_empty_verse_before_contentful_verse: options
                .remove_orphan_empty_verse_before_contentful_verse,
            remove_bridge_verse_enumerators: options.remove_bridge_verse_enumerators,
            move_chapter_label_after_chapter_marker: options
                .move_chapter_label_after_chapter_marker,
            insert_default_paragraph_after_chapter_intro: options
                .insert_default_paragraph_after_chapter_intro,
            insert_structural_linebreaks: options.insert_structural_linebreaks,
            collapse_consecutive_linebreaks: options.collapse_consecutive_linebreaks,
            normalize_marker_whitespace_at_line_start: options
                .normalize_marker_whitespace_at_line_start,
        }
    } else {
        FormatOptions::default()
    }
}

fn build_sid_blocks_options(options: Option<WebBuildSidBlocksOptions>) -> BuildSidBlocksOptions {
    let options = options.unwrap_or(WebBuildSidBlocksOptions {
        allow_empty_sid: default_allow_empty_sid_true(),
    });
    BuildSidBlocksOptions {
        allow_empty_sid: options.allow_empty_sid,
    }
}

fn token_kind_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Newline => "newline",
        TokenKind::OptBreak => "optbreak",
        TokenKind::Marker => "marker",
        TokenKind::EndMarker => "end-marker",
        TokenKind::Milestone => "milestone",
        TokenKind::MilestoneEnd => "milestone-end",
        TokenKind::Attributes => "attributes",
        TokenKind::BookCode => "book-code",
        TokenKind::Number => "number",
        TokenKind::Text => "text",
    }
}

fn parse_token_kind(kind: &str) -> TokenKind {
    match kind {
        "whitespace" => TokenKind::Text,
        "newline" => TokenKind::Newline,
        "optbreak" => TokenKind::OptBreak,
        "marker" => TokenKind::Marker,
        "end-marker" => TokenKind::EndMarker,
        "milestone" => TokenKind::Milestone,
        "milestone-end" => TokenKind::MilestoneEnd,
        "attributes" => TokenKind::Attributes,
        "book-code" => TokenKind::BookCode,
        "number" => TokenKind::Number,
        _ => TokenKind::Text,
    }
}

fn scan_token_kind_name(kind: &ScanTokenKind) -> &'static str {
    match kind {
        ScanTokenKind::Whitespace => "whitespace",
        ScanTokenKind::Newline => "newline",
        ScanTokenKind::OptBreak => "optbreak",
        ScanTokenKind::Marker => "marker",
        ScanTokenKind::NestedMarker => "nested-marker",
        ScanTokenKind::ClosingMarker => "closing-marker",
        ScanTokenKind::NestedClosingMarker => "nested-closing-marker",
        ScanTokenKind::Milestone => "milestone",
        ScanTokenKind::MilestoneEnd => "milestone-end",
        ScanTokenKind::Attributes => "attributes",
        ScanTokenKind::Text => "text",
    }
}

fn transform_kind_name(kind: &TokenTransformKind) -> &'static str {
    match kind {
        TokenTransformKind::Fix => "fix",
        TokenTransformKind::Format => "format",
        TokenTransformKind::CustomFormatPass => "custom-format-pass",
    }
}

fn transform_skip_reason_name(reason: &TokenTransformSkipReason) -> &'static str {
    match reason {
        TokenTransformSkipReason::TokenNotFound => "token-not-found",
        TokenTransformSkipReason::EmptyReplacement => "empty-replacement",
    }
}

fn recovery_code_name(code: &RecoveryCode) -> &'static str {
    match code {
        RecoveryCode::MissingChapterNumber => "missing-chapter-number",
        RecoveryCode::MissingVerseNumber => "missing-verse-number",
        RecoveryCode::MissingMilestoneSelfClose => "missing-milestone-self-close",
        RecoveryCode::ImplicitlyClosedMarker => "implicitly-closed-marker",
        RecoveryCode::StrayCloseMarker => "stray-close-marker",
        RecoveryCode::MisnestedCloseMarker => "misnested-close-marker",
        RecoveryCode::UnclosedNote => "unclosed-note",
        RecoveryCode::UnclosedMarkerAtEof => "unclosed-marker-at-eof",
    }
}

fn diff_status_name(status: DiffStatus) -> &'static str {
    match status {
        DiffStatus::Added => "added",
        DiffStatus::Deleted => "deleted",
        DiffStatus::Modified => "modified",
        DiffStatus::Unchanged => "unchanged",
    }
}

fn parse_diff_status(status: &str) -> DiffStatus {
    match status {
        "added" => DiffStatus::Added,
        "deleted" => DiffStatus::Deleted,
        "modified" => DiffStatus::Modified,
        _ => DiffStatus::Unchanged,
    }
}

fn diff_token_change_name(change: DiffTokenChange) -> &'static str {
    match change {
        DiffTokenChange::Unchanged => "unchanged",
        DiffTokenChange::Added => "added",
        DiffTokenChange::Deleted => "deleted",
        DiffTokenChange::Modified => "modified",
    }
}

fn parse_diff_token_change(change: &str) -> DiffTokenChange {
    match change {
        "added" => DiffTokenChange::Added,
        "deleted" => DiffTokenChange::Deleted,
        "modified" => DiffTokenChange::Modified,
        _ => DiffTokenChange::Unchanged,
    }
}

fn diff_undo_side_name(side: DiffUndoSide) -> &'static str {
    match side {
        DiffUndoSide::Original => "original",
        DiffUndoSide::Current => "current",
    }
}

fn parse_diff_undo_side(side: &str) -> DiffUndoSide {
    match side {
        "original" => DiffUndoSide::Original,
        _ => DiffUndoSide::Current,
    }
}

fn parse_lint_code(code: &str) -> Option<LintCode> {
    [
        LintCode::MissingSeparatorAfterMarker,
        LintCode::NumberRangeAfterChapterMarker,
        LintCode::VerseRangeExpectedAfterVerseMarker,
        LintCode::VerseContentNotEmpty,
        LintCode::UnknownToken,
        LintCode::CharNotClosed,
        LintCode::NoteNotClosed,
        LintCode::ParagraphBeforeFirstChapter,
        LintCode::VerseBeforeFirstChapter,
        LintCode::NoteSubmarkerOutsideNote,
        LintCode::DuplicateIdMarker,
        LintCode::IdMarkerNotAtFileStart,
        LintCode::ChapterMetadataOutsideChapter,
        LintCode::VerseMetadataOutsideVerse,
        LintCode::MissingChapterNumber,
        LintCode::MissingVerseNumber,
        LintCode::MissingMilestoneSelfClose,
        LintCode::ImplicitlyClosedMarker,
        LintCode::StrayCloseMarker,
        LintCode::MisnestedCloseMarker,
        LintCode::UnclosedNote,
        LintCode::UnclosedMarkerAtEof,
        LintCode::DuplicateChapterNumber,
        LintCode::ChapterExpectedIncreaseByOne,
        LintCode::DuplicateVerseNumber,
        LintCode::VerseExpectedIncreaseByOne,
        LintCode::InvalidNumberRange,
        LintCode::NumberRangeNotPrecededByMarkerExpectingNumber,
        LintCode::VerseTextFollowsVerseRange,
        LintCode::UnknownMarker,
        LintCode::UnknownCloseMarker,
        LintCode::InconsistentChapterLabel,
        LintCode::MarkerNotValidInContext,
        LintCode::VerseOutsideExplicitParagraph,
    ]
    .into_iter()
    .find(|candidate| candidate.as_str() == code)
}

fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

fn js_error_from_serde(error: serde_wasm_bindgen::Error) -> JsError {
    JsError::new(&error.to_string())
}

fn default_prefer_native_true() -> bool {
    true
}

fn default_parallel_true() -> bool {
    true
}

fn default_allow_empty_sid_true() -> bool {
    true
}
