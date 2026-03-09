use crate::internal::api::intake::into_usfm_from_tokens;
use crate::internal::api::types::{DocumentError, DocumentFormat};
use crate::internal::html::{
    HtmlOptions, into_html as render_html, usfm_content_to_html as render_usfm_html,
};
use crate::internal::usj::{to_editor_tree_document, to_usj_document, to_usj_lossless_document};
use crate::internal::usj_to_usfm::{UsjToUsfmError, from_usj_document, from_usj_value};
use crate::internal::usx::{UsxError, to_usx_lossless_string, to_usx_string};
use crate::internal::usx_to_usfm::{UsxToUsfmError, from_usx_string};
use crate::internal::vref::{VrefMap, to_vref_map};
use crate::model::editor_tree::EditorTreeDocument;
use crate::model::token::SourceTokenText;
use crate::model::usj::UsjDocument;
use crate::parse::handle::ParseHandle;

pub fn into_usj(handle: &ParseHandle) -> UsjDocument {
    to_usj_document(handle)
}

pub fn into_usj_lossless(handle: &ParseHandle) -> UsjDocument {
    to_usj_lossless_document(handle)
}

pub fn into_editor_tree(handle: &ParseHandle) -> EditorTreeDocument {
    to_editor_tree_document(handle)
}

pub fn into_usx(handle: &ParseHandle) -> Result<String, UsxError> {
    to_usx_string(handle)
}

pub fn into_usx_lossless(handle: &ParseHandle) -> Result<String, UsxError> {
    to_usx_lossless_string(handle)
}

pub fn into_vref(handle: &ParseHandle) -> VrefMap {
    to_vref_map(handle)
}

pub fn into_html(handle: &ParseHandle, options: HtmlOptions) -> String {
    render_html(handle, options)
}

pub fn into_usj_from_tokens<T: SourceTokenText>(tokens: &[T]) -> UsjDocument {
    let usfm = into_usfm_from_tokens(tokens);
    let handle = crate::parse::parse(&usfm);
    into_usj(&handle)
}

pub fn into_usj_lossless_from_tokens<T: SourceTokenText>(tokens: &[T]) -> UsjDocument {
    let usfm = into_usfm_from_tokens(tokens);
    let handle = crate::parse::parse(&usfm);
    into_usj_lossless(&handle)
}

pub fn into_usx_from_tokens<T: SourceTokenText>(tokens: &[T]) -> Result<String, UsxError> {
    let usfm = into_usfm_from_tokens(tokens);
    let handle = crate::parse::parse(&usfm);
    into_usx(&handle)
}

pub fn into_usx_lossless_from_tokens<T: SourceTokenText>(tokens: &[T]) -> Result<String, UsxError> {
    let usfm = into_usfm_from_tokens(tokens);
    let handle = crate::parse::parse(&usfm);
    into_usx_lossless(&handle)
}

pub fn into_vref_from_tokens<T: SourceTokenText>(tokens: &[T]) -> VrefMap {
    let usfm = into_usfm_from_tokens(tokens);
    let handle = crate::parse::parse(&usfm);
    into_vref(&handle)
}

pub fn from_usj(value: &UsjDocument) -> Result<String, UsjToUsfmError> {
    from_usj_document(value)
}

pub fn from_usx(value: &str) -> Result<String, UsxToUsfmError> {
    from_usx_string(value)
}

pub fn convert_content(
    source: &str,
    source_format: DocumentFormat,
    target_format: DocumentFormat,
) -> Result<String, DocumentError> {
    if source_format == target_format {
        return Ok(source.to_string());
    }

    let usfm = decode_to_usfm(source, source_format)?;
    match target_format {
        DocumentFormat::Usfm => Ok(usfm),
        DocumentFormat::Usj => {
            let handle = crate::parse::parse(&usfm);
            Ok(serde_json::to_string(&into_usj(&handle))?)
        }
        DocumentFormat::Usx => {
            let handle = crate::parse::parse(&usfm);
            Ok(into_usx(&handle)?)
        }
    }
}

pub fn usfm_content_to_usj(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usj)
}

pub fn usfm_content_to_usx(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usfm, DocumentFormat::Usx)
}

pub fn usfm_content_to_html(source: &str, options: HtmlOptions) -> String {
    render_usfm_html(source, options)
}

pub fn usj_content_to_usfm(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usj, DocumentFormat::Usfm)
}

pub fn usx_content_to_usfm(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usx, DocumentFormat::Usfm)
}

pub fn usj_content_to_usx(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usj, DocumentFormat::Usx)
}

pub fn usx_content_to_usj(source: &str) -> Result<String, DocumentError> {
    convert_content(source, DocumentFormat::Usx, DocumentFormat::Usj)
}

pub fn convert_path(
    path: impl AsRef<std::path::Path>,
    source_format: DocumentFormat,
    target_format: DocumentFormat,
) -> Result<String, DocumentError> {
    let source = super::intake::read_document(path, source_format)?;
    convert_content(&source, source_format, target_format)
}

pub fn usfm_path_to_usj(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usfm, DocumentFormat::Usj)
}

pub fn usfm_path_to_usx(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usfm, DocumentFormat::Usx)
}

pub fn usj_path_to_usfm(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usj, DocumentFormat::Usfm)
}

pub fn usx_path_to_usfm(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usx, DocumentFormat::Usfm)
}

pub fn usj_path_to_usx(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usj, DocumentFormat::Usx)
}

pub fn usx_path_to_usj(path: impl AsRef<std::path::Path>) -> Result<String, DocumentError> {
    convert_path(path, DocumentFormat::Usx, DocumentFormat::Usj)
}

pub(super) fn decode_to_usfm(
    source: &str,
    format: DocumentFormat,
) -> Result<String, DocumentError> {
    match format {
        DocumentFormat::Usfm => Ok(source.to_string()),
        DocumentFormat::Usj => {
            let value: serde_json::Value = serde_json::from_str(source)?;
            Ok(from_usj_value(&value)?)
        }
        DocumentFormat::Usx => Ok(from_usx(source)?),
    }
}
