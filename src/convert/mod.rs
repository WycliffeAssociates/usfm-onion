pub use crate::internal::api::{DocumentError, DocumentFormat, convert_content, convert_path};
pub use crate::internal::html::{HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, HtmlOptions};
pub use crate::internal::usj_to_usfm::UsjToUsfmError;
pub use crate::internal::usx::UsxError;
pub use crate::internal::usx_to_usfm::UsxToUsfmError;
pub use crate::internal::vref::VrefMap;
pub use crate::model::usj::UsjDocument;

use crate::ast::AstDocument;
use crate::tokens::{Token, usfm_to_tokens};

pub fn from_usj_str(source: &str) -> Result<String, DocumentError> {
    crate::internal::api::usj_content_to_usfm(source)
}

pub fn from_usj(document: &UsjDocument) -> Result<String, UsjToUsfmError> {
    crate::internal::api::from_usj(document)
}

pub fn from_usx_str(source: &str) -> Result<String, DocumentError> {
    crate::internal::api::usx_content_to_usfm(source)
}

pub fn from_usx(source: &str) -> Result<String, UsxToUsfmError> {
    crate::internal::api::from_usx(source)
}

pub fn ast_to_usj(document: &AstDocument) -> Result<UsjDocument, DocumentError> {
    Ok(crate::internal::usj::document_tree_to_usj_document(
        document,
    ))
}

pub fn ast_to_usx(document: &AstDocument) -> Result<String, DocumentError> {
    Ok(crate::internal::usx::document_tree_to_usx_string(document)?)
}

pub fn ast_to_html(document: &AstDocument, options: HtmlOptions) -> Result<String, DocumentError> {
    Ok(crate::internal::html::render_document_tree(
        document, options,
    ))
}

pub fn ast_to_vref(document: &AstDocument) -> Result<VrefMap, DocumentError> {
    Ok(crate::internal::vref::document_tree_to_vref_map(document))
}

pub fn into_ast(handle: &crate::parse::ParseHandle) -> AstDocument {
    crate::internal::api::into_document_tree(handle)
}

pub fn into_usj(handle: &crate::parse::ParseHandle) -> UsjDocument {
    crate::internal::api::into_usj(handle)
}

pub fn into_usx(handle: &crate::parse::ParseHandle) -> Result<String, UsxError> {
    crate::internal::api::into_usx(handle)
}

pub fn into_html(handle: &crate::parse::ParseHandle, options: HtmlOptions) -> String {
    crate::internal::api::into_html(handle, options)
}

pub fn into_vref(handle: &crate::parse::ParseHandle) -> VrefMap {
    crate::internal::api::into_vref(handle)
}

pub fn into_usj_from_tokens<T: crate::model::SourceTokenText>(tokens: &[T]) -> UsjDocument {
    crate::internal::api::into_usj_from_tokens(tokens)
}

pub fn into_usx_from_tokens<T: crate::model::SourceTokenText>(
    tokens: &[T],
) -> Result<String, UsxError> {
    crate::internal::api::into_usx_from_tokens(tokens)
}

pub fn into_vref_from_tokens<T: crate::model::SourceTokenText>(tokens: &[T]) -> VrefMap {
    crate::internal::api::into_vref_from_tokens(tokens)
}

pub fn usfm_to_usj(source: &str) -> Result<UsjDocument, DocumentError> {
    let ast = crate::ast::usfm_to_ast(source);
    ast_to_usj(&ast)
}

pub fn usfm_to_usx(source: &str) -> Result<String, DocumentError> {
    let ast = crate::ast::usfm_to_ast(source);
    ast_to_usx(&ast)
}

pub fn usfm_to_html(source: &str, options: HtmlOptions) -> Result<String, DocumentError> {
    let ast = crate::ast::usfm_to_ast(source);
    ast_to_html(&ast, options)
}

pub fn usfm_content_to_html(source: &str, options: HtmlOptions) -> String {
    crate::internal::api::usfm_content_to_html(source, options)
}

pub fn usfm_to_vref(source: &str) -> Result<VrefMap, DocumentError> {
    let ast = crate::ast::usfm_to_ast(source);
    ast_to_vref(&ast)
}

pub fn usj_to_usx(source: &str) -> Result<String, DocumentError> {
    let ast = crate::ast::usj_to_ast(source)?;
    ast_to_usx(&ast)
}

pub fn usx_to_usj(source: &str) -> Result<UsjDocument, DocumentError> {
    let ast = crate::ast::usx_to_ast(source)?;
    ast_to_usj(&ast)
}

pub fn tokens_to_usj(tokens: &[Token]) -> Result<UsjDocument, DocumentError> {
    let ast = crate::ast::tokens_to_ast(tokens);
    ast_to_usj(&ast)
}

pub fn tokens_to_usx(tokens: &[Token]) -> Result<String, DocumentError> {
    let ast = crate::ast::tokens_to_ast(tokens);
    ast_to_usx(&ast)
}

pub fn tokens_to_html(tokens: &[Token], options: HtmlOptions) -> Result<String, DocumentError> {
    let ast = crate::ast::tokens_to_ast(tokens);
    ast_to_html(&ast, options)
}

pub fn tokens_to_vref(tokens: &[Token]) -> Result<VrefMap, DocumentError> {
    let ast = crate::ast::tokens_to_ast(tokens);
    ast_to_vref(&ast)
}

pub fn usj_to_tokens(source: &str) -> Result<Vec<Token>, DocumentError> {
    let usfm = from_usj_str(source)?;
    Ok(usfm_to_tokens(&usfm))
}

pub fn usx_to_tokens(source: &str) -> Result<Vec<Token>, DocumentError> {
    let usfm = from_usx_str(source)?;
    Ok(usfm_to_tokens(&usfm))
}
