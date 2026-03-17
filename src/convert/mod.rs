pub use crate::usj::{UsjDocument, UsjError};
pub use crate::usx::UsxError;
pub use crate::html::HtmlOptions;

pub fn usfm_to_usj(source: &str) -> Result<UsjDocument, UsjError> {
    crate::usj::usfm_to_usj(source)
}

pub fn from_usj(document: &UsjDocument) -> Result<String, UsjError> {
    crate::usj::from_usj(document)
}

pub fn from_usj_str(source: &str) -> Result<String, UsjError> {
    crate::usj::from_usj_str(source)
}

pub fn usfm_to_usx(source: &str) -> Result<String, UsxError> {
    crate::usx::usfm_to_usx(source)
}

pub fn usj_to_usx(document: &UsjDocument) -> Result<String, UsxError> {
    crate::usx::usj_to_usx(document)
}

pub fn usx_to_usj(source: &str) -> Result<UsjDocument, UsxError> {
    crate::usx::usx_to_usj(source)
}

pub fn from_usx_str(source: &str) -> Result<String, UsxError> {
    crate::usx::from_usx_str(source)
}

pub fn usfm_to_html(source: &str, options: HtmlOptions) -> String {
    crate::html::usfm_to_html(source, options)
}
