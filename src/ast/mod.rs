pub use crate::internal::api::{DocumentError, DocumentFormat, read_document};
pub use crate::model::document_tree::{
    DocumentTreeDocument as AstDocument, DocumentTreeElement as AstElement,
    DocumentTreeNode as AstNode,
};

use crate::cst::CstDocument;
use crate::model::token::Token;
use std::path::Path;

pub fn usfm_to_ast(source: &str) -> AstDocument {
    crate::document_tree::usfm_to_document_tree(source)
}

pub fn usj_to_ast(source: &str) -> Result<AstDocument, DocumentError> {
    crate::document_tree::usj_to_document_tree(source)
}

pub fn usx_to_ast(source: &str) -> Result<AstDocument, DocumentError> {
    crate::document_tree::usx_to_document_tree(source)
}

pub fn read_usfm_to_ast(path: impl AsRef<Path>) -> Result<AstDocument, DocumentError> {
    crate::document_tree::read_usfm_to_document_tree(path)
}

pub fn read_usj_to_ast(path: impl AsRef<Path>) -> Result<AstDocument, DocumentError> {
    crate::document_tree::read_usj_to_document_tree(path)
}

pub fn read_usx_to_ast(path: impl AsRef<Path>) -> Result<AstDocument, DocumentError> {
    crate::document_tree::read_usx_to_document_tree(path)
}

pub fn tokens_to_ast(tokens: &[Token]) -> AstDocument {
    crate::document_tree::tokens_to_document_tree(tokens)
}

pub fn cst_to_ast(document: &CstDocument) -> AstDocument {
    tokens_to_ast(document.tokens())
}

pub fn ast_to_tokens(document: &AstDocument) -> Result<Vec<Token>, DocumentError> {
    crate::document_tree::document_tree_to_tokens(document)
}

pub fn ast_to_usfm(document: &AstDocument) -> Result<String, DocumentError> {
    crate::document_tree::document_tree_to_usfm(document)
}
