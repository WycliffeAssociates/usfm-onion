// use crate::internal::api::types::{
//     BatchExecutionOptions, DocumentError, DocumentFormat, IntoTokensOptions, ProjectUsfmOptions,
//     ProjectedUsfmDocument,
// };
// use crate::internal::lexer::lex;
// use crate::model::token::{SourceTokenText, Token, TokenViewOptions};
// use crate::parse::handle::{ParseHandle, tokens};

// pub fn push_whitespace(tokens: &[Token]) -> Vec<Token> {
//     tokens.to_vec()
// }

// pub fn into_tokens(handle: &ParseHandle, options: IntoTokensOptions) -> Vec<Token> {
//     let _ = options;
//     tokens(handle, TokenViewOptions::default())
// }

// pub fn into_usfm_from_tokens<T: SourceTokenText>(tokens: &[T]) -> String {
//     let capacity = tokens.iter().map(|token| token.source_text().len()).sum();
//     let mut out = String::with_capacity(capacity);
//     for token in tokens {
//         out.push_str(token.source_text());
//     }
//     out
// }

// pub fn project_document(
//     handle: &ParseHandle,
//     options: ProjectUsfmOptions,
// ) -> ProjectedUsfmDocument {
//     let tokens = into_tokens(handle, options.token_options);
//     let document_tree = super::convert::into_document_tree(handle);
//     let lint_issues = options
//         .lint_options
//         .map(|lint_options| super::ops::lint_document(handle, lint_options));

//     ProjectedUsfmDocument {
//         tokens,
//         document_tree,
//         lint_issues,
//     }
// }

// pub fn project_usfm(source: &str, options: ProjectUsfmOptions) -> ProjectedUsfmDocument {
//     let handle = crate::parse::parse(source);
//     project_document(&handle, options)
// }

// pub fn read_document(
//     path: impl AsRef<std::path::Path>,
//     _format: DocumentFormat,
// ) -> Result<String, DocumentError> {
//     Ok(std::fs::read_to_string(path)?)
// }

// pub fn parse_content(source: &str, format: DocumentFormat) -> Result<ParseHandle, DocumentError> {
//     let usfm = super::convert::decode_to_usfm(source, format)?;
//     Ok(crate::parse::parse(&usfm))
// }

// pub fn parse_usfm_content(source: &str) -> Result<ParseHandle, DocumentError> {
//     parse_content(source, DocumentFormat::Usfm)
// }

// pub fn parse_usj_content(source: &str) -> Result<ParseHandle, DocumentError> {
//     parse_content(source, DocumentFormat::Usj)
// }

// pub fn parse_usx_content(source: &str) -> Result<ParseHandle, DocumentError> {
//     parse_content(source, DocumentFormat::Usx)
// }

// pub fn parse_path(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
// ) -> Result<ParseHandle, DocumentError> {
//     let source = read_document(path, format)?;
//     parse_content(&source, format)
// }

// pub fn parse_usfm_path(path: impl AsRef<std::path::Path>) -> Result<ParseHandle, DocumentError> {
//     parse_path(path, DocumentFormat::Usfm)
// }

// pub fn parse_usj_path(path: impl AsRef<std::path::Path>) -> Result<ParseHandle, DocumentError> {
//     parse_path(path, DocumentFormat::Usj)
// }

// pub fn parse_usx_path(path: impl AsRef<std::path::Path>) -> Result<ParseHandle, DocumentError> {
//     parse_path(path, DocumentFormat::Usx)
// }

// pub fn parse_contents<S: AsRef<str> + Sync>(
//     sources: &[S],
//     format: DocumentFormat,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<ParseHandle, DocumentError>> {
//     super::map_with_batch(sources, batch_options, |source| {
//         parse_content(source.as_ref(), format)
//     })
// }

// pub fn parse_paths<P: AsRef<std::path::Path> + Sync>(
//     paths: &[P],
//     format: DocumentFormat,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<ParseHandle, DocumentError>> {
//     super::map_with_batch(paths, batch_options, |path| {
//         parse_path(path.as_ref(), format)
//     })
// }

// pub fn into_tokens_from_content(
//     source: &str,
//     format: DocumentFormat,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     let handle = parse_content(source, format)?;
//     Ok(into_tokens(&handle, options))
// }

// pub fn into_tokens_from_usfm_content(
//     source: &str,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_content(source, DocumentFormat::Usfm, options)
// }

// pub fn into_tokens_from_usj_content(
//     source: &str,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_content(source, DocumentFormat::Usj, options)
// }

// pub fn into_tokens_from_usx_content(
//     source: &str,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_content(source, DocumentFormat::Usx, options)
// }

// pub fn into_tokens_from_path(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     let handle = parse_path(path, format)?;
//     Ok(into_tokens(&handle, options))
// }

// pub fn into_tokens_from_usfm_path(
//     path: impl AsRef<std::path::Path>,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_path(path, DocumentFormat::Usfm, options)
// }

// pub fn into_tokens_from_usj_path(
//     path: impl AsRef<std::path::Path>,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_path(path, DocumentFormat::Usj, options)
// }

// pub fn into_tokens_from_usx_path(
//     path: impl AsRef<std::path::Path>,
//     options: IntoTokensOptions,
// ) -> Result<Vec<Token>, DocumentError> {
//     into_tokens_from_path(path, DocumentFormat::Usx, options)
// }

// pub fn into_tokens_from_contents<S: AsRef<str> + Sync>(
//     sources: &[S],
//     format: DocumentFormat,
//     options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<Vec<Token>, DocumentError>> {
//     super::map_with_batch(sources, batch_options, |source| {
//         into_tokens_from_content(source.as_ref(), format, options)
//     })
// }

// pub fn into_tokens_from_paths<P: AsRef<std::path::Path> + Sync>(
//     paths: &[P],
//     format: DocumentFormat,
//     options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Result<Vec<Token>, DocumentError>> {
//     super::map_with_batch(paths, batch_options, |path| {
//         into_tokens_from_path(path.as_ref(), format, options)
//     })
// }

// pub fn project_content(
//     source: &str,
//     format: DocumentFormat,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     let handle = parse_content(source, format)?;
//     Ok(project_document(&handle, options))
// }

// pub fn project_usfm_content(
//     source: &str,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_content(source, DocumentFormat::Usfm, options)
// }

// pub fn project_usj_content(
//     source: &str,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_content(source, DocumentFormat::Usj, options)
// }

// pub fn project_usx_content(
//     source: &str,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_content(source, DocumentFormat::Usx, options)
// }

// pub fn project_path(
//     path: impl AsRef<std::path::Path>,
//     format: DocumentFormat,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     let handle = parse_path(path, format)?;
//     Ok(project_document(&handle, options))
// }

// pub fn project_usfm_path(
//     path: impl AsRef<std::path::Path>,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_path(path, DocumentFormat::Usfm, options)
// }

// pub fn project_usj_path(
//     path: impl AsRef<std::path::Path>,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_path(path, DocumentFormat::Usj, options)
// }

// pub fn project_usx_path(
//     path: impl AsRef<std::path::Path>,
//     options: ProjectUsfmOptions,
// ) -> Result<ProjectedUsfmDocument, DocumentError> {
//     project_path(path, DocumentFormat::Usx, options)
// }

// pub fn lex_sources<S: AsRef<str> + Sync>(
//     sources: &[S],
//     batch_options: BatchExecutionOptions,
// ) -> Vec<crate::model::ScanResult> {
//     super::map_with_batch(sources, batch_options, |source| lex(source.as_ref()))
// }

// pub fn parse_sources<S: AsRef<str> + Sync>(
//     sources: &[S],
//     batch_options: BatchExecutionOptions,
// ) -> Vec<ParseHandle> {
//     super::map_with_batch(sources, batch_options, |source| {
//         crate::parse::parse(source.as_ref())
//     })
// }

// pub fn into_tokens_batch(
//     handles: &[ParseHandle],
//     options: IntoTokensOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<Vec<Token>> {
//     super::map_with_batch(handles, batch_options, |handle| {
//         into_tokens(handle, options)
//     })
// }

// pub fn project_usfm_batch<S: AsRef<str> + Sync>(
//     sources: &[S],
//     options: ProjectUsfmOptions,
//     batch_options: BatchExecutionOptions,
// ) -> Vec<ProjectedUsfmDocument> {
//     super::map_with_batch(sources, batch_options, |source| {
//         project_usfm(source.as_ref(), options.clone())
//     })
// }
