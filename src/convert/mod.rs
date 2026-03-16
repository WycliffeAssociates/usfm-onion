pub use crate::usj::{UsjDocument, UsjError};

pub fn usfm_to_usj(source: &str) -> Result<UsjDocument, UsjError> {
    crate::usj::usfm_to_usj(source)
}

pub fn from_usj(document: &UsjDocument) -> Result<String, UsjError> {
    crate::usj::from_usj(document)
}

pub fn from_usj_str(source: &str) -> Result<String, UsjError> {
    crate::usj::from_usj_str(source)
}
