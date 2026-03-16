// use crate::diff::{
//     BuildSidBlocksOptions, ChapterTokenDiff, DiffableToken, DiffsByChapterMap,
//     diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
// };
// use crate::format::{
//     BoxedTokenFormatPass, FormatOptions, FormattableToken, format_mut, format_mut_with_passes,
// };
// use crate::internal::api::intake::{
//     into_tokens, into_tokens_batch, parse_content, parse_path, parse_sources,
// };
// use crate::internal::api::types::{
//     BatchExecutionOptions, DocumentError, DocumentFormat, IntoTokensOptions,
// };
// use crate::internal::transform::{
//     TokenFix, TokenTransformResult, apply_fixes, format_tokens_result,
//     format_tokens_result_with_passes,
// };
// use crate::lint::{LintIssue, LintOptions, LintableToken, TokenLintOptions, lint_tokens};
// use crate::model::token::{Token, TokenViewOptions};
// use crate::parse::handle::{ParseHandle, tokens as project_tokens};

// pub fn lint_content(
//     source: &str,
//     format: DocumentFormat,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     let handle = parse_content(source, format)?;
//     Ok(lint_document(&handle, options))
// }

// pub fn lint_usfm_content(
//     source: &str,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_content(source, DocumentFormat::Usfm, options)
// }

// pub fn lint_usj_content(
//     source: &str,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_content(source, DocumentFormat::Usj, options)
// }

// pub fn lint_usx_content(
//     source: &str,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_content(source, DocumentFormat::Usx, options)
// }

// pub fn lint_contents<S: AsRef<str> + Sync>(
//     sources: &[S],
//     format: DocumentFormat,
//     options: LintOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<Vec<LintIssue>, DocumentError>> {
//     super::map_with_batch(sources, batch_options, |source| {
//         lint_content(source.as_ref(), format, options.clone())
//     })
// }

// pub fn lint_path(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     let handle = parse_path(path, format)?;
//     Ok(lint_document(&handle, options))
// }

// pub fn lint_usfm_path(
//     path: impl AsRef<std::path::Path>,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_path(path, DocumentFormat::Usfm, options)
// }

// pub fn lint_usj_path(
//     path: impl AsRef<std::path::Path>,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_path(path, DocumentFormat::Usj, options)
// }

// pub fn lint_usx_path(
//     path: impl AsRef<std::path::Path>,
//     options: LintOptions,
// ) -> Result<Vec<LintIssue>, DocumentError> {
//     lint_path(path, DocumentFormat::Usx, options)
// }

// pub fn lint_paths<P: AsRef<std::path::Path> + Sync>(
//     paths: &[P],
//     format: DocumentFormat,
//     options: LintOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<Vec<LintIssue>, DocumentError>> {
//     super::map_with_batch(paths, batch_options, |path| {
//         lint_path(path.as_ref(), format, options.clone())
//     })
// }

// pub fn format_content(
//     source: &str,
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     let handle = parse_content(source, format)?;
//     let token_batch = into_tokens(&handle, token_options);
//     Ok(format_flat_tokens(token_batch.as_slice()))
// }

// pub fn format_usfm_content(
//     source: &str,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content(source, DocumentFormat::Usfm, token_options)
// }

// pub fn format_usj_content(
//     source: &str,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content(source, DocumentFormat::Usj, token_options)
// }

// pub fn format_usx_content(
//     source: &str,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content(source, DocumentFormat::Usx, token_options)
// }

// pub fn format_content_with_options(
//     source: &str,
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     format_options: FormatOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     let handle = parse_content(source, format)?;
//     let token_batch = into_tokens(&handle, token_options);
//     Ok(format_flat_tokens_with_options(
//         token_batch.as_slice(),
//         format_options,
//     ))
// }

// pub fn format_content_with_passes(
//     source: &str,
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<Token>],
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     let handle = parse_content(source, format)?;
//     let token_batch = into_tokens(&handle, token_options);
//     Ok(format_flat_tokens_with_passes(
//         token_batch.as_slice(),
//         options,
//         passes,
//     ))
// }

// pub fn format_usfm_content_with_passes(
//     source: &str,
//     token_options: IntoTokensOptions,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<Token>],
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content_with_passes(source, DocumentFormat::Usfm, token_options, options, passes)
// }

// pub fn format_usj_content_with_passes(
//     source: &str,
//     token_options: IntoTokensOptions,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<Token>],
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content_with_passes(source, DocumentFormat::Usj, token_options, options, passes)
// }

// pub fn format_usx_content_with_passes(
//     source: &str,
//     token_options: IntoTokensOptions,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<Token>],
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_content_with_passes(source, DocumentFormat::Usx, token_options, options, passes)
// }

// pub fn format_contents<S: AsRef<str> + Sync>(
//     sources: &[S],
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<TokenTransformResult<Token>, DocumentError>> {
//     super::map_with_batch(sources, batch_options, |source| {
//         format_content(source.as_ref(), format, token_options)
//     })
// }

// pub fn format_contents_with_options<S: AsRef<str> + Sync>(
//     sources: &[S],
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     format_options: FormatOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<TokenTransformResult<Token>, DocumentError>> {
//     super::map_with_batch(sources, batch_options, |source| {
//         format_content_with_options(source.as_ref(), format, token_options, format_options)
//     })
// }

// pub fn format_path(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     let source = super::intake::read_document(path, format)?;
//     format_content(&source, format, token_options)
// }

// pub fn format_usfm_path(
//     path: impl AsRef<std::path::Path>,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_path(path, DocumentFormat::Usfm, token_options)
// }

// pub fn format_usj_path(
//     path: impl AsRef<std::path::Path>,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_path(path, DocumentFormat::Usj, token_options)
// }

// pub fn format_usx_path(
//     path: impl AsRef<std::path::Path>,
//     token_options: IntoTokensOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     format_path(path, DocumentFormat::Usx, token_options)
// }

// pub fn format_path_with_options(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     format_options: FormatOptions,
// ) -> Result<TokenTransformResult<Token>, DocumentError> {
//     let source = super::intake::read_document(path, format)?;
//     format_content_with_options(&source, format, token_options, format_options)
// }

// pub fn format_paths<P: AsRef<std::path::Path> + Sync>(
//     paths: &[P],
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<TokenTransformResult<Token>, DocumentError>> {
//     super::map_with_batch(paths, batch_options, |path| {
//         format_path(path.as_ref(), format, token_options)
//     })
// }

// pub fn format_paths_with_options<P: AsRef<std::path::Path> + Sync>(
//     paths: &[P],
//     format: DocumentFormat,
//     token_options: IntoTokensOptions,
//     format_options: FormatOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<TokenTransformResult<Token>, DocumentError>> {
//     super::map_with_batch(paths, batch_options, |path| {
//         format_path_with_options(path.as_ref(), format, token_options, format_options)
//     })
// }

// pub fn diff_content(
//     baseline_source: &str,
//     baseline_format: DocumentFormat,
//     current_source: &str,
//     current_format: DocumentFormat,
//     token_view: &TokenViewOptions,
//     build_options: &BuildSidBlocksOptions,
// ) -> Result<Vec<ChapterTokenDiff<Token>>, DocumentError> {
//     let baseline_usfm = super::convert::decode_to_usfm(baseline_source, baseline_format)?;
//     let current_usfm = super::convert::decode_to_usfm(current_source, current_format)?;
//     Ok(diff_usfm(
//         &baseline_usfm,
//         &current_usfm,
//         token_view,
//         build_options,
//     ))
// }

// pub fn diff_paths(
//     baseline_path: impl AsRef<std::path::Path>,
//     baseline_format: DocumentFormat,
//     current_path: impl AsRef<std::path::Path>,
//     current_format: DocumentFormat,
//     token_view: &TokenViewOptions,
//     build_options: &BuildSidBlocksOptions,
// ) -> Result<Vec<ChapterTokenDiff<Token>>, DocumentError> {
//     let baseline_source = super::intake::read_document(baseline_path, baseline_format)?;
//     let current_source = super::intake::read_document(current_path, current_format)?;
//     diff_content(
//         &baseline_source,
//         baseline_format,
//         &current_source,
//         current_format,
//         token_view,
//         build_options,
//     )
// }

// pub fn lint_document(handle: &ParseHandle, options: LintOptions) -> Vec<LintIssue> {
//     let projected = project_tokens(handle, options.token_view);
//     lint_tokens(&projected, options.token_rules)
// }

// pub fn lint_document_batch(
//     handles: &[ParseHandle],
//     options: LintOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Vec<LintIssue>> {
//     super::map_with_batch(handles, batch_options, |handle| {
//         lint_document(handle, options.clone())
//     })
// }

// pub fn lint_usfm_sources<S: AsRef<str> + Sync>(
//     sources: &[S],
//     options: LintOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Vec<LintIssue>> {
//     let handles = parse_sources(sources, batch_options);
//     lint_document_batch(&handles, options, batch_options)
// }

// pub fn lint_flat_tokens<T: LintableToken>(
//     tokens: &[T],
//     options: TokenLintOptions,
// ) -> Vec<LintIssue> {
//     lint_tokens(tokens, options)
// }

// pub fn lint_flat_token_batches<T: LintableToken + Sync>(
//     token_batches: &[Vec<T>],
//     options: TokenLintOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Vec<LintIssue>> {
//     super::map_with_batch(token_batches, batch_options, |tokens| {
//         lint_flat_tokens(tokens.as_slice(), options.clone())
//     })
// }

// pub fn format_flat_tokens<T: FormattableToken>(tokens: &[T]) -> TokenTransformResult<T> {
//     format_tokens_result(tokens, FormatOptions::default())
// }

// pub fn format_flat_tokens_with_options<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
// ) -> TokenTransformResult<T> {
//     format_tokens_result(tokens, options)
// }

// pub fn format_flat_tokens_with_passes<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<T>],
// ) -> TokenTransformResult<T> {
//     format_tokens_result_with_passes(tokens, options, passes)
// }

// pub fn format_flat_tokens_mut<T: FormattableToken>(tokens: &mut Vec<T>) {
//     format_mut(tokens);
// }

// pub fn format_flat_tokens_mut_with_passes<T: FormattableToken>(
//     tokens: &mut Vec<T>,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<T>],
// ) {
//     format_mut_with_passes(tokens, options, passes);
// }

// pub fn format_flat_token_batches<T: FormattableToken + Sync + Send>(
//     token_batches: &[Vec<T>],
//     batch_options: BatchExecutionOptions,
// ) -> Vec<TokenTransformResult<T>> {
//     super::map_with_batch(token_batches, batch_options, |tokens| {
//         format_flat_tokens(tokens.as_slice())
//     })
// }

// pub fn format_flat_token_batches_with_options<T: FormattableToken + Sync + Send>(
//     token_batches: &[Vec<T>],
//     options: FormatOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<TokenTransformResult<T>> {
//     super::map_with_batch(token_batches, batch_options, |tokens| {
//         format_flat_tokens_with_options(tokens.as_slice(), options)
//     })
// }

// pub fn format_usfm_sources<S: AsRef<str> + Sync>(
//     sources: &[S],
//     token_options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<TokenTransformResult<Token>> {
//     let handles = parse_sources(sources, batch_options);
//     let token_batches = into_tokens_batch(&handles, token_options, batch_options);
//     format_flat_token_batches(&token_batches, batch_options)
// }

// pub fn format_usfm_sources_with_options<S: AsRef<str> + Sync>(
//     sources: &[S],
//     token_options: IntoTokensOptions,
//     format_options: FormatOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<TokenTransformResult<Token>> {
//     let handles = parse_sources(sources, batch_options);
//     let token_batches = into_tokens_batch(&handles, token_options, batch_options);
//     format_flat_token_batches_with_options(&token_batches, format_options, batch_options)
// }

// pub fn apply_token_fixes<T: Clone + FormattableToken>(
//     tokens: &[T],
//     fixes: &[TokenFix],
// ) -> TokenTransformResult<T> {
//     apply_fixes(tokens, fixes)
// }

// pub fn diff_tokens<T: DiffableToken>(
//     baseline_tokens: &[T],
//     current_tokens: &[T],
//     options: &BuildSidBlocksOptions,
// ) -> Vec<ChapterTokenDiff<T>> {
//     diff_chapter_token_streams(baseline_tokens, current_tokens, options)
// }

// pub fn diff_usfm(
//     baseline_usfm: &str,
//     current_usfm: &str,
//     token_view: &TokenViewOptions,
//     build_options: &BuildSidBlocksOptions,
// ) -> Vec<ChapterTokenDiff<Token>> {
//     diff_usfm_sources(baseline_usfm, current_usfm, token_view, build_options)
// }

// pub fn diff_usfm_by_chapter(
//     baseline_usfm: &str,
//     current_usfm: &str,
//     token_view: &TokenViewOptions,
//     build_options: &BuildSidBlocksOptions,
// ) -> DiffsByChapterMap<ChapterTokenDiff<Token>> {
//     diff_usfm_sources_by_chapter(baseline_usfm, current_usfm, token_view, build_options)
// }
