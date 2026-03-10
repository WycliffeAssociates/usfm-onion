use crate::internal::usj_to_usfm::UsjToUsfmError;
use crate::internal::usx::UsxError;
use crate::internal::usx_to_usfm::UsxToUsfmError;
use crate::lint::{LintIssue, LintOptions};
use crate::model::document_tree::DocumentTreeDocument;
use crate::model::token::Token;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

pub fn read_usfm(path: impl AsRef<Path>) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Usfm,
    Usj,
    Usx,
}

impl DocumentFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usfm => "usfm",
            Self::Usj => "usj",
            Self::Usx => "usx",
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path.as_ref().extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "sfm" | "usfm" => Some(Self::Usfm),
            "json" | "usj" => Some(Self::Usj),
            "xml" | "usx" => Some(Self::Usx),
            _ => None,
        }
    }
}

impl std::fmt::Display for DocumentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DocumentFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "usfm" | "sfm" => Ok(Self::Usfm),
            "usj" | "json" => Ok(Self::Usj),
            "usx" | "xml" => Ok(Self::Usx),
            other => Err(format!("unsupported document format: {other}")),
        }
    }
}

#[derive(Debug)]
pub enum DocumentError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Usj(UsjToUsfmError),
    Usx(UsxToUsfmError),
    UsxSerialize(UsxError),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Usj(error) => write!(f, "usj conversion error: {error}"),
            Self::Usx(error) => write!(f, "usx conversion error: {error}"),
            Self::UsxSerialize(error) => write!(f, "usx serialization error: {error}"),
        }
    }
}

impl std::error::Error for DocumentError {}

impl From<std::io::Error> for DocumentError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for DocumentError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<UsjToUsfmError> for DocumentError {
    fn from(value: UsjToUsfmError) -> Self {
        Self::Usj(value)
    }
}

impl From<UsxToUsfmError> for DocumentError {
    fn from(value: UsxToUsfmError) -> Self {
        Self::Usx(value)
    }
}

impl From<UsxError> for DocumentError {
    fn from(value: UsxError) -> Self {
        Self::UsxSerialize(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntoTokensOptions {
    pub merge_horizontal_whitespace: bool,
}

impl IntoTokensOptions {
    pub const fn preserve() -> Self {
        Self {
            merge_horizontal_whitespace: false,
        }
    }

    pub const fn merge_horizontal_whitespace() -> Self {
        Self {
            merge_horizontal_whitespace: true,
        }
    }

    pub const fn with_merge_horizontal_whitespace(mut self, merge: bool) -> Self {
        self.merge_horizontal_whitespace = merge;
        self
    }
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

impl BatchExecutionOptions {
    pub const fn parallel() -> Self {
        Self { parallel: true }
    }

    pub const fn sequential() -> Self {
        Self { parallel: false }
    }

    pub const fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectUsfmOptions {
    pub token_options: IntoTokensOptions,
    pub lint_options: Option<LintOptions>,
}

impl ProjectUsfmOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_token_options(mut self, token_options: IntoTokensOptions) -> Self {
        self.token_options = token_options;
        self
    }

    pub fn with_lint_options(mut self, lint_options: LintOptions) -> Self {
        self.lint_options = Some(lint_options);
        self
    }

    pub fn without_lint(mut self) -> Self {
        self.lint_options = None;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ProjectedUsfmDocument {
    pub tokens: Vec<Token>,
    pub document_tree: DocumentTreeDocument,
    pub lint_issues: Option<Vec<LintIssue>>,
}
