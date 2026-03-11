pub use crate::internal::api::{
    BatchExecutionOptions, DocumentError, DocumentFormat, read_document,
};
pub use crate::model::token::{
    ScanResult, ScanToken, ScanTokenKind, SourceTokenText, Span, Token, TokenKind, TokenVariant,
    TokenViewOptions,
};

use crate::internal::api;
use std::path::Path;

pub fn usfm_to_tokens(source: &str) -> Vec<Token> {
    let handle = crate::internal::parse::parse(source);
    crate::parse::handle::tokens(&handle, TokenViewOptions::default())
}

pub fn usj_to_tokens(source: &str) -> Result<Vec<Token>, DocumentError> {
    api::into_tokens_from_content(
        source,
        DocumentFormat::Usj,
        api::IntoTokensOptions::default(),
    )
}

pub fn usx_to_tokens(source: &str) -> Result<Vec<Token>, DocumentError> {
    api::into_tokens_from_content(
        source,
        DocumentFormat::Usx,
        api::IntoTokensOptions::default(),
    )
}

pub fn read_usfm_to_tokens(path: impl AsRef<Path>) -> Result<Vec<Token>, DocumentError> {
    let source = read_document(path, DocumentFormat::Usfm)?;
    Ok(usfm_to_tokens(&source))
}

pub fn read_usj_to_tokens(path: impl AsRef<Path>) -> Result<Vec<Token>, DocumentError> {
    api::into_tokens_from_path(path, DocumentFormat::Usj, api::IntoTokensOptions::default())
}

pub fn read_usx_to_tokens(path: impl AsRef<Path>) -> Result<Vec<Token>, DocumentError> {
    api::into_tokens_from_path(path, DocumentFormat::Usx, api::IntoTokensOptions::default())
}

pub fn tokens_to_usfm<T: SourceTokenText>(tokens: &[T]) -> String {
    api::into_usfm_from_tokens(tokens)
}

pub fn classify_tokens(tokens: &[Token]) -> Vec<TokenVariant> {
    tokens.iter().map(Token::variant).collect()
}

pub fn usfm_to_token_variants(source: &str) -> Vec<TokenVariant> {
    classify_tokens(&usfm_to_tokens(source))
}
