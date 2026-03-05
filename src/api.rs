use std::path::Path;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::diff::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffableFlatToken, DiffsByChapterMap,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
};
use crate::editor_tree_types::EditorTreeDocument;
use crate::format::{FormatOptions, FormattableFlatToken};
use crate::handle::{ParseHandle, tokens};
use crate::lint::{LintIssue, LintOptions, LintableFlatToken, TokenLintOptions, lint, lint_tokens};
use crate::token::{FlatToken, TokenViewOptions, WhitespacePolicy};
use crate::transform::{TokenFix, TokenTransformResult, apply_fixes, format_tokens_result};
use crate::usj::{to_editor_tree_document, to_usj_document};
use crate::usj_to_usfm::{UsjToUsfmError, from_usj_document};
use crate::usj_types::UsjDocument;
use crate::usx::{UsxError, to_usx_string};
use crate::usx_to_usfm::{UsxToUsfmError, from_usx_string};
use crate::vref::{VrefMap, to_vref_map};

pub fn read_usfm(path: impl AsRef<Path>) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntoTokensOptions {
    pub merge_horizontal_whitespace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchExecutionOptions {
    pub parallel: bool,
}

impl Default for BatchExecutionOptions {
    fn default() -> Self {
        Self { parallel: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectUsfmOptions {
    pub token_options: IntoTokensOptions,
    pub lint_options: Option<LintOptions>,
}

#[derive(Debug, Clone)]
pub struct ProjectedUsfmDocument {
    pub tokens: Vec<FlatToken>,
    pub editor_tree: EditorTreeDocument,
    pub lint_issues: Option<Vec<LintIssue>>,
}

pub fn push_whitespace(tokens: &[FlatToken]) -> Vec<FlatToken> {
    let mut merged = tokens.to_vec();
    merge_horizontal_whitespace(&mut merged);
    merged
}

pub fn into_tokens(handle: &ParseHandle, options: IntoTokensOptions) -> Vec<FlatToken> {
    tokens(
        handle,
        TokenViewOptions {
            whitespace_policy: if options.merge_horizontal_whitespace {
                WhitespacePolicy::MergeToVisible
            } else {
                WhitespacePolicy::Preserve
            },
        },
    )
}

pub fn project_document(
    handle: &ParseHandle,
    options: ProjectUsfmOptions,
) -> ProjectedUsfmDocument {
    let tokens = into_tokens(handle, options.token_options);
    let editor_tree = into_editor_tree(handle);
    let lint_issues = options.lint_options.map(|lint_options| lint_document(handle, lint_options));

    ProjectedUsfmDocument {
        tokens,
        editor_tree,
        lint_issues,
    }
}

pub fn project_usfm(source: &str, options: ProjectUsfmOptions) -> ProjectedUsfmDocument {
    let handle = crate::parse(source);
    project_document(&handle, options)
}

pub fn lex_sources<S: AsRef<str> + Sync>(
    sources: &[S],
    batch_options: BatchExecutionOptions,
) -> Vec<crate::LexResult> {
    map_with_batch(sources, batch_options, |source| crate::lex(source.as_ref()))
}

pub fn parse_sources<S: AsRef<str> + Sync>(
    sources: &[S],
    batch_options: BatchExecutionOptions,
) -> Vec<ParseHandle> {
    map_with_batch(sources, batch_options, |source| crate::parse(source.as_ref()))
}

pub fn into_tokens_batch(
    handles: &[ParseHandle],
    options: IntoTokensOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<Vec<FlatToken>> {
    map_with_batch(handles, batch_options, |handle| into_tokens(handle, options))
}

pub fn project_usfm_batch<S: AsRef<str> + Sync>(
    sources: &[S],
    options: ProjectUsfmOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<ProjectedUsfmDocument> {
    map_with_batch(sources, batch_options, |source| {
        project_usfm(source.as_ref(), options.clone())
    })
}

pub fn into_usj(handle: &ParseHandle) -> UsjDocument {
    to_usj_document(handle)
}

pub fn into_editor_tree(handle: &ParseHandle) -> EditorTreeDocument {
    to_editor_tree_document(handle)
}

pub fn into_usx(handle: &ParseHandle) -> Result<String, UsxError> {
    to_usx_string(handle)
}

pub fn into_vref(handle: &ParseHandle) -> VrefMap {
    to_vref_map(handle)
}

pub fn from_usj(value: &UsjDocument) -> Result<String, UsjToUsfmError> {
    from_usj_document(value)
}

pub fn from_usx(value: &str) -> Result<String, UsxToUsfmError> {
    from_usx_string(value)
}

pub fn lint_document(handle: &ParseHandle, options: LintOptions) -> Vec<LintIssue> {
    lint(handle, options)
}

pub fn lint_document_batch(
    handles: &[ParseHandle],
    options: LintOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<Vec<LintIssue>> {
    map_with_batch(handles, batch_options, |handle| {
        lint_document(handle, options.clone())
    })
}

pub fn lint_usfm_sources<S: AsRef<str> + Sync>(
    sources: &[S],
    options: LintOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<Vec<LintIssue>> {
    let handles = parse_sources(sources, batch_options);
    lint_document_batch(&handles, options, batch_options)
}

pub fn lint_flat_tokens<T: LintableFlatToken>(
    tokens: &[T],
    options: TokenLintOptions,
) -> Vec<LintIssue> {
    lint_tokens(tokens, options)
}

pub fn lint_flat_token_batches<T: LintableFlatToken + Sync>(
    token_batches: &[Vec<T>],
    options: TokenLintOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<Vec<LintIssue>> {
    map_with_batch(token_batches, batch_options, |tokens| {
        lint_flat_tokens(tokens.as_slice(), options.clone())
    })
}

pub fn format_flat_tokens<T: FormattableFlatToken>(
    tokens: &[T],
    options: FormatOptions,
) -> TokenTransformResult<T> {
    format_tokens_result(tokens, options)
}

pub fn format_flat_token_batches<T: FormattableFlatToken + Sync + Send>(
    token_batches: &[Vec<T>],
    options: FormatOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<TokenTransformResult<T>> {
    map_with_batch(token_batches, batch_options, |tokens| {
        format_flat_tokens(tokens.as_slice(), options)
    })
}

pub fn format_usfm_sources<S: AsRef<str> + Sync>(
    sources: &[S],
    token_options: IntoTokensOptions,
    format_options: FormatOptions,
    batch_options: BatchExecutionOptions,
) -> Vec<TokenTransformResult<FlatToken>> {
    let handles = parse_sources(sources, batch_options);
    let token_batches = into_tokens_batch(&handles, token_options, batch_options);
    format_flat_token_batches(&token_batches, format_options, batch_options)
}

pub fn apply_token_fixes<T: Clone>(
    tokens: &[T],
    fixes: &[TokenFix],
) -> TokenTransformResult<T>
where
    T: FormattableFlatToken,
{
    apply_fixes(tokens, fixes)
}

pub fn diff_tokens<T: DiffableFlatToken>(
    baseline_tokens: &[T],
    current_tokens: &[T],
    options: &BuildSidBlocksOptions,
) -> Vec<ChapterTokenDiff<T>> {
    diff_chapter_token_streams(baseline_tokens, current_tokens, options)
}

pub fn diff_usfm(
    baseline_usfm: &str,
    current_usfm: &str,
    token_view: &TokenViewOptions,
    build_options: &BuildSidBlocksOptions,
) -> Vec<ChapterTokenDiff<FlatToken>> {
    diff_usfm_sources(baseline_usfm, current_usfm, token_view, build_options)
}

pub fn diff_usfm_by_chapter(
    baseline_usfm: &str,
    current_usfm: &str,
    token_view: &TokenViewOptions,
    build_options: &BuildSidBlocksOptions,
) -> DiffsByChapterMap<ChapterTokenDiff<FlatToken>> {
    diff_usfm_sources_by_chapter(baseline_usfm, current_usfm, token_view, build_options)
}

fn merge_horizontal_whitespace(tokens: &mut Vec<FlatToken>) {
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != crate::TokenKind::HorizontalWhitespace {
            index += 1;
            continue;
        }

        let ws = tokens[index].clone();
        if let Some(next) = tokens.get_mut(index + 1) {
            if next.kind != crate::TokenKind::VerticalWhitespace {
                next.text = format!("{}{}", ws.text, next.text);
                next.span = ws.span.start..next.span.end;
                tokens.remove(index);
                continue;
            }
        }

        if index > 0 {
            let prev = &mut tokens[index - 1];
            prev.text.push_str(&ws.text);
            prev.span = prev.span.start..ws.span.end;
            tokens.remove(index);
            continue;
        }

        index += 1;
    }
}

#[cfg(feature = "rayon")]
fn map_with_batch<T, U, F>(items: &[T], batch_options: BatchExecutionOptions, map: F) -> Vec<U>
where
    T: Sync,
    U: Send,
    F: Fn(&T) -> U + Sync + Send,
{
    if batch_options.parallel {
        items.par_iter().map(&map).collect()
    } else {
        items.iter().map(map).collect()
    }
}

#[cfg(not(feature = "rayon"))]
fn map_with_batch<T, U, F>(items: &[T], _batch_options: BatchExecutionOptions, map: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    items.iter().map(map).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn into_tokens_preserves_horizontal_whitespace() {
        let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
        let projected = into_tokens(&handle, IntoTokensOptions::default());
        assert!(projected.iter().any(|token| token.kind == crate::TokenKind::HorizontalWhitespace));
    }

    #[test]
    fn into_tokens_can_merge_horizontal_whitespace() {
        let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
        let projected = into_tokens(
            &handle,
            IntoTokensOptions {
                merge_horizontal_whitespace: true,
            },
        );
        assert!(projected.iter().all(|token| token.kind != crate::TokenKind::HorizontalWhitespace));
    }

    #[test]
    fn into_usj_and_into_vref_are_composable() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let usj = into_usj(&handle);
        let vref = into_vref(&handle);

        assert_eq!(usj.doc_type, "USJ");
        assert_eq!(vref.get("GEN 1:1").map(String::as_str), Some("In the beginning"));
    }

    #[test]
    fn into_editor_tree_preserves_linebreak_nodes() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let json = serde_json::to_string(&serialized).expect("editor tree json");

        assert!(json.contains("\"type\":\"linebreak\""));
    }

    #[test]
    fn into_editor_tree_preserves_linebreak_between_chapter_and_paragraph() {
        let handle = parse("\\s5\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");

        assert!(content.iter().any(|node| {
            node.get("type").and_then(serde_json::Value::as_str) == Some("chapter")
        }));
        assert!(content.iter().any(|node| {
            node.get("type").and_then(serde_json::Value::as_str) == Some("linebreak")
        }));
    }

    #[test]
    fn into_editor_tree_preserves_space_after_verse_number() {
        let handle = parse("\\c 1\n\\p\n\\v 1 In the beginning\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let para = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
            .expect("paragraph node");
        let para_content = para
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("paragraph content");
        let verse_index = para_content
            .iter()
            .position(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
            .expect("verse node");
        let following_text = para_content
            .iter()
            .skip(verse_index + 1)
            .find_map(serde_json::Value::as_str)
            .expect("text after verse");

        assert!(following_text.starts_with(' '));
    }

    #[test]
    fn into_editor_tree_preserves_double_spaces_in_text_nodes() {
        let handle = parse("\\c 1\n\\p\n\\v 1 I will give  the inhabitants.\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let para = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
            .expect("paragraph node");
        let para_content = para
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("paragraph content");
        let verse_index = para_content
            .iter()
            .position(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
            .expect("verse node");
        let following_text = para_content
            .iter()
            .skip(verse_index + 1)
            .find_map(serde_json::Value::as_str)
            .expect("text after verse");

        assert!(
            following_text.contains("give  the"),
            "expected double space to be preserved in editor tree text"
        );
    }

    #[test]
    fn into_editor_tree_preserves_space_after_book_code() {
        let handle = parse("\\id GEN Unlocked Literal Bible\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let book = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("book"))
            .expect("book node");
        let book_content = book
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("book content");
        let first_text = book_content
            .iter()
            .find_map(serde_json::Value::as_str)
            .expect("first text child");

        assert!(first_text.starts_with(' '));
    }

    #[test]
    fn into_editor_tree_uses_zero_verse_sid_for_chapters() {
        let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let chapter = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("chapter"))
            .expect("chapter node");

        assert_eq!(chapter.get("sid").and_then(serde_json::Value::as_str), Some("GEN 1:0"));
    }

    #[test]
    fn into_editor_tree_preserves_explicit_note_char_closures() {
        let handle = parse(
            "\\c 1\n\\p\n\\v 26 text\\f + \\ft intro \\fqa quote\\fqa* \\f*\n",
        );
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let para = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
            .expect("paragraph node");
        let note = para
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("paragraph content")
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("note"))
            .expect("note node");
        let fqa = note
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("note content")
            .iter()
            .find(|node| node.get("marker").and_then(serde_json::Value::as_str) == Some("fqa"))
            .expect("fqa node");

        assert_eq!(fqa.get("closed").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            fqa.get("closeSuffix").and_then(serde_json::Value::as_str),
            Some(" ")
        );
    }

    #[test]
    fn into_editor_tree_preserves_exact_paragraph_marker_text() {
        let handle = parse("\\m(for fine linen is righteous)\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");
        let para = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
            .expect("paragraph node");

        assert_eq!(
            para.get("markerText").and_then(serde_json::Value::as_str),
            Some("\\m")
        );
    }

    #[test]
    fn into_editor_tree_preserves_marker_text_for_book_chapter_and_verse() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning.\n");
        let tree = into_editor_tree(&handle);
        let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
        let content = serialized
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("root content array");

        let book = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("book"))
            .expect("book node");
        assert_eq!(
            book.get("markerText").and_then(serde_json::Value::as_str),
            Some("\\id ")
        );

        let chapter = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("chapter"))
            .expect("chapter node");
        assert_eq!(
            chapter
                .get("markerText")
                .and_then(serde_json::Value::as_str),
            Some("\\c ")
        );

        let para = content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
            .expect("paragraph node");
        let para_content = para
            .get("content")
            .and_then(serde_json::Value::as_array)
            .expect("paragraph content");
        let verse = para_content
            .iter()
            .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
            .expect("verse node");
        assert_eq!(
            verse.get("markerText").and_then(serde_json::Value::as_str),
            Some("\\v ")
        );
    }

    #[test]
    fn project_usfm_can_return_tokens_tree_and_lint_from_one_parse() {
        let projection = project_usfm(
            "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n",
            ProjectUsfmOptions {
                token_options: IntoTokensOptions {
                    merge_horizontal_whitespace: true,
                },
                lint_options: Some(LintOptions::default()),
            },
        );

        assert_eq!(projection.editor_tree.doc_type, "USJ");
        assert!(!projection.tokens.is_empty());
        assert!(projection.lint_issues.is_some());
    }

    #[test]
    fn format_then_project_preserves_id_separator_and_book_sids() {
        let source = "\\id GEN Unlocked Literal Bible\n\\ide UTF-8\n\\h Genesis\n\\toc1 The Book of Genesis\n\\toc2 Genesis\n\\toc3 Gen\n\\mt Genesis\n\n\\s5\n\\c 1\n\\p\n\\v 1 In the beginning.\n";
        let handle = parse(source);
        let baseline = into_tokens(
            &handle,
            IntoTokensOptions {
                merge_horizontal_whitespace: true,
            },
        );
        let formatted = format_flat_tokens(&baseline, FormatOptions::default());
        let formatted_usfm = formatted
            .tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<String>();
        assert!(formatted_usfm.starts_with("\\id GEN Unlocked Literal Bible\n"));

        let projection = project_usfm(
            formatted_usfm.as_str(),
            ProjectUsfmOptions {
                token_options: IntoTokensOptions {
                    merge_horizontal_whitespace: true,
                },
                lint_options: None,
            },
        );
        let projected_usfm = projection
            .tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<String>();

        assert!(projected_usfm.starts_with("\\id GEN Unlocked Literal Bible\n"));
        assert!(
            projection
                .tokens
                .iter()
                .filter_map(|token| token.sid.as_deref())
                .any(|sid| sid.starts_with("GEN ")),
            "expected at least one GEN SID after format/project"
        );
        assert!(
            projection
                .tokens
                .iter()
                .filter_map(|token| token.sid.as_deref())
                .all(|sid| !sid.starts_with(' ')),
            "unexpected blank book-code SID emitted"
        );
    }

    #[test]
    fn push_whitespace_matches_flat_projection_policy() {
        let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
        let preserved = into_tokens(&handle, IntoTokensOptions::default());
        let merged = push_whitespace(&preserved);
        let flat = into_tokens(
            &handle,
            IntoTokensOptions {
                merge_horizontal_whitespace: true,
            },
        );

        let merged_shape = merged
            .iter()
            .map(|token| {
                (
                    token.kind.clone(),
                    token.span.clone(),
                    token.sid.clone(),
                    token.marker.clone(),
                    token.text.clone(),
                )
            })
            .collect::<Vec<_>>();
        let flat_shape = flat
            .iter()
            .map(|token| {
                (
                    token.kind.clone(),
                    token.span.clone(),
                    token.sid.clone(),
                    token.marker.clone(),
                    token.text.clone(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(merged_shape, flat_shape);
    }
}
