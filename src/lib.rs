mod api;
mod diff;
mod editor_tree_types;
mod format;
mod handle;
mod inspect;
mod lexer;
mod lint;
mod markers;
mod parse;
mod recovery;
mod syntax;
mod token;
mod transform;
mod usj;
mod usj_types;
mod usj_to_usfm;
mod usx;
mod usx_to_usfm;
mod vref;
mod write_exact;

pub use api::{
    BatchExecutionOptions, IntoTokensOptions, ProjectUsfmOptions, ProjectedUsfmDocument,
    apply_token_fixes, diff_tokens, diff_usfm, diff_usfm_by_chapter, format_flat_token_batches,
    format_flat_tokens, format_usfm_sources, from_usj, from_usx, into_editor_tree, into_tokens,
    into_tokens_batch, into_usj, into_usx, into_vref, lex_sources, lint_document,
    lint_document_batch, lint_flat_token_batches, lint_flat_tokens, lint_usfm_sources,
    parse_sources, project_document, project_usfm, project_usfm_batch, push_whitespace, read_usfm,
};
pub use diff::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffStatus, DiffTokenChange, DiffUndoSide,
    DiffableFlatToken, DiffsByChapterMap, SidBlock, SidBlockDiff, TokenAlignment,
    apply_revert_by_block_id, apply_reverts_by_block_id, build_sid_blocks, diff_chapter_token_streams,
    diff_sid_blocks, diff_usfm_sources, diff_usfm_sources_by_chapter, flatten_diff_map,
    replace_chapter_diffs_in_map, replace_many_chapter_diffs_in_map,
};
#[cfg(feature = "rayon")]
pub use diff::{diff_usfm_sources_by_chapter_parallel, diff_usfm_sources_parallel};
pub use handle::{ParseHandle, recoveries, tokens};
pub use format::{
    FormatOptions, FormatProfile, FormattableFlatToken, format_tokens, format_tokens_profile,
    prettify_tokens,
};
pub use inspect::{DebugDumpOptions, debug_dump};
pub use lexer::lex;
pub use lint::{
    LintCode, LintIssue, LintOptions, LintSuppression, LintableFlatToken, TokenLintOptions, lint,
    lint_tokens,
};
pub use parse::parse;
pub use recovery::{ParseRecovery, RecoveryCode, RecoveryPayload};
pub use token::{
    FlatToken, LexResult, LexToken, RawTokenKind, Span, TokenKind, TokenViewOptions,
    WhitespacePolicy,
};
pub use transform::{
    SkippedTokenTransform, TokenFix, TokenTemplate, TokenTransformChange, TokenTransformKind,
    TokenTransformResult, TokenTransformSkipReason, apply_fixes, format_tokens_result,
};
pub use usj::{
    to_usj_document, to_usj_roundtrip_document, to_usj_roundtrip_string,
    to_usj_roundtrip_string_pretty, to_usj_roundtrip_value, to_usj_string, to_usj_string_pretty,
    to_usj_value,
};
pub use usj_types::{UsjDocument, UsjElement, UsjNode, UsjRoundtrip};
pub use usj_to_usfm::{UsjToUsfmError, from_usj_document, from_usj_value};
pub use usx::{UsxError, to_usx_string};
pub use usx_to_usfm::{UsxToUsfmError, from_usx_string};
pub use vref::{VrefMap, to_vref_json_string, to_vref_map};
pub use write_exact::write_exact;
pub use editor_tree_types::{EditorTreeDocument, EditorTreeElement, EditorTreeNode};
