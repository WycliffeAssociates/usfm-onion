use std::fs;
use std::path::{Path, PathBuf};

use crate::cst::{CstDocument, parse_cst};
use crate::diff::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffsByChapterMap, DiffableToken,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
};
use crate::format::{FormatOptions, FormattableToken, FormatToken, format};
use crate::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use crate::lint::{LintOptions, LintResult, LintableToken, lint_tokens, lint_usfm};
use crate::parse::parse;
use crate::token::Token;
use crate::usj::{UsjDocument, UsjError, usfm_to_usj};
use crate::usx::{UsxError, usfm_to_usx, usj_to_usx};
use crate::vref::{VrefMap, usfm_to_vref_map};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Serial,
    Parallel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchItem<T> {
    pub path: Option<PathBuf>,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usfm {
    path: Option<PathBuf>,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsfmBatch {
    docs: Vec<Usfm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usj {
    path: Option<PathBuf>,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsjBatch {
    docs: Vec<Usj>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usx {
    path: Option<PathBuf>,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsxBatch {
    docs: Vec<Usx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream<T> {
    path: Option<PathBuf>,
    tokens: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenBatch<T> {
    streams: Vec<TokenStream<T>>,
}

pub trait SourceTokenText {
    fn source_text(&self) -> &str;
}

impl<'a> SourceTokenText for Token<'a> {
    fn source_text(&self) -> &str {
        self.source
    }
}

impl SourceTokenText for FormatToken {
    fn source_text(&self) -> &str {
        &self.text
    }
}

impl Usfm {
    pub fn from_str(source: &str) -> Self {
        Self {
            path: None,
            source: source.to_string(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        Ok(Self {
            path: Some(path),
            source,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn parse(&self) -> crate::token::ParseResult<'_> {
        parse(&self.source)
    }

    pub fn cst(&self) -> CstDocument<'_> {
        parse_cst(&self.source)
    }

    pub fn tokens(&self) -> Vec<Token<'_>> {
        self.parse().tokens
    }

    pub fn lint(&self, options: LintOptions) -> LintResult {
        lint_usfm(&self.source, options)
    }

    pub fn format(&self, options: FormatOptions) -> String {
        crate::format::format_usfm(&self.source, options)
    }

    pub fn to_usj(&self) -> Result<UsjDocument, UsjError> {
        usfm_to_usj(&self.source)
    }

    pub fn to_usx(&self) -> Result<String, UsxError> {
        usfm_to_usx(&self.source)
    }

    pub fn to_html(&self, options: HtmlOptions) -> String {
        usfm_to_html(&self.source, options)
    }

    pub fn to_vref(&self) -> VrefMap {
        usfm_to_vref_map(&self.source)
    }

    pub fn diff<'a>(&'a self, other: &'a Usfm) -> UsfmDiffBuilder<'a> {
        UsfmDiffBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }

    pub fn diff_by_chapter<'a>(&'a self, other: &'a Usfm) -> UsfmDiffByChapterBuilder<'a> {
        UsfmDiffByChapterBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }
}

impl UsfmBatch {
    pub fn from_strs<I, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            docs: sources
                .into_iter()
                .map(|source| Usfm::from_str(source.as_ref()))
                .collect(),
        }
    }

    pub fn from_paths<I, P>(paths: I) -> std::io::Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut docs = Vec::new();
        for path in paths {
            docs.push(Usfm::from_path(path)?);
        }
        Ok(Self { docs })
    }

    pub fn items(&self) -> &[Usfm] {
        &self.docs
    }

    pub fn lint(&self, options: LintOptions) -> UsfmBatchLintBuilder<'_> {
        UsfmBatchLintBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }

    pub fn format(&self, options: FormatOptions) -> UsfmBatchFormatBuilder<'_> {
        UsfmBatchFormatBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }

    pub fn to_usj(&self) -> UsfmBatchToUsjBuilder<'_> {
        UsfmBatchToUsjBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
    }

    pub fn to_usx(&self) -> UsfmBatchToUsxBuilder<'_> {
        UsfmBatchToUsxBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
    }

    pub fn to_html(&self, options: HtmlOptions) -> UsfmBatchToHtmlBuilder<'_> {
        UsfmBatchToHtmlBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }
}

impl Usj {
    pub fn from_str(source: &str) -> Self {
        Self {
            path: None,
            source: source.to_string(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        Ok(Self {
            path: Some(path),
            source,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn document(&self) -> Result<UsjDocument, UsjError> {
        Ok(serde_json::from_str(&self.source)?)
    }

    pub fn to_usx(&self) -> Result<String, UsxError> {
        let document = self.document()?;
        usj_to_usx(&document)
    }

    // TODO: restore facade-level Usj -> Usfm once reverse-import ergonomics are finalized.
}

impl UsjBatch {
    pub fn from_strs<I, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            docs: sources
                .into_iter()
                .map(|source| Usj::from_str(source.as_ref()))
                .collect(),
        }
    }

    pub fn from_paths<I, P>(paths: I) -> std::io::Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut docs = Vec::new();
        for path in paths {
            docs.push(Usj::from_path(path)?);
        }
        Ok(Self { docs })
    }

    pub fn items(&self) -> &[Usj] {
        &self.docs
    }

    pub fn to_usx(&self) -> UsjBatchToUsxBuilder<'_> {
        UsjBatchToUsxBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
    }
}

impl Usx {
    pub fn from_str(source: &str) -> Self {
        Self {
            path: None,
            source: source.to_string(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        Ok(Self {
            path: Some(path),
            source,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    // TODO: restore facade-level Usx -> Usfm and Usx -> Usj once reverse-import support is brought back.
}

impl UsxBatch {
    pub fn from_strs<I, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            docs: sources
                .into_iter()
                .map(|source| Usx::from_str(source.as_ref()))
                .collect(),
        }
    }

    pub fn from_paths<I, P>(paths: I) -> std::io::Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut docs = Vec::new();
        for path in paths {
            docs.push(Usx::from_path(path)?);
        }
        Ok(Self { docs })
    }

    pub fn items(&self) -> &[Usx] {
        &self.docs
    }
}

impl<T> TokenStream<T> {
    pub fn from_tokens(tokens: Vec<T>) -> Self {
        Self { path: None, tokens }
    }

    pub fn from_tokens_with_path(tokens: Vec<T>, path: impl AsRef<Path>) -> Self {
        Self {
            path: Some(path.as_ref().to_path_buf()),
            tokens,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn tokens(&self) -> &[T] {
        &self.tokens
    }

    pub fn into_tokens(self) -> Vec<T> {
        self.tokens
    }
}

impl<T: LintableToken> TokenStream<T> {
    pub fn lint(&self, options: LintOptions) -> LintResult {
        lint_tokens(&self.tokens, options)
    }
}

impl<T: FormattableToken + Clone> TokenStream<T> {
    pub fn format(&self, options: FormatOptions) -> Vec<T> {
        format(&self.tokens, options)
    }
}

impl<T: SourceTokenText> TokenStream<T> {
    pub fn to_usfm(&self) -> String {
        tokens_to_usfm_text(&self.tokens)
    }
}

impl<T: DiffableToken> TokenStream<T> {
    pub fn diff<'a>(&'a self, other: &'a TokenStream<T>) -> TokenDiffBuilder<'a, T> {
        TokenDiffBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }
}

impl<'a> TokenStream<Token<'a>> {
    pub fn to_html(&self, options: HtmlOptions) -> String {
        tokens_to_html(&self.tokens, options)
    }

    pub fn to_usj(&self) -> Result<UsjDocument, UsjError> {
        let usfm = self.to_usfm();
        usfm_to_usj(&usfm)
    }

    pub fn to_usx(&self) -> Result<String, UsxError> {
        let usfm = self.to_usfm();
        usfm_to_usx(&usfm)
    }

    pub fn to_vref(&self) -> VrefMap {
        let usfm = self.to_usfm();
        usfm_to_vref_map(&usfm)
    }
}

impl<T> TokenBatch<T> {
    pub fn from_token_streams(streams: Vec<TokenStream<T>>) -> Self {
        Self { streams }
    }

    pub fn items(&self) -> &[TokenStream<T>] {
        &self.streams
    }
}

impl<T: LintableToken> TokenBatch<T> {
    pub fn lint(&self, options: LintOptions) -> TokenBatchLintBuilder<'_, T> {
        TokenBatchLintBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }
}

impl<T: FormattableToken + Clone> TokenBatch<T> {
    pub fn format(&self, options: FormatOptions) -> TokenBatchFormatBuilder<'_, T> {
        TokenBatchFormatBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }
}

impl<T: SourceTokenText> TokenBatch<T> {
    pub fn to_usfm(&self) -> TokenBatchToUsfmBuilder<'_, T> {
        TokenBatchToUsfmBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
    }
}

impl<'a> TokenBatch<Token<'a>> {
    pub fn to_html(&self, options: HtmlOptions) -> TokenBatchToHtmlBuilder<'_, 'a> {
        TokenBatchToHtmlBuilder {
            batch: self,
            options,
            execution: ExecutionMode::Serial,
        }
    }
}

pub struct UsfmDiffBuilder<'a> {
    left: &'a Usfm,
    right: &'a Usfm,
    options: BuildSidBlocksOptions,
}

impl<'a> UsfmDiffBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn run(self) -> Vec<ChapterTokenDiff<Token<'a>>> {
        diff_usfm_sources(&self.left.source, &self.right.source, &self.options)
    }
}

pub struct UsfmDiffByChapterBuilder<'a> {
    left: &'a Usfm,
    right: &'a Usfm,
    options: BuildSidBlocksOptions,
}

impl<'a> UsfmDiffByChapterBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn run(self) -> DiffsByChapterMap<ChapterTokenDiff<Token<'a>>> {
        diff_usfm_sources_by_chapter(&self.left.source, &self.right.source, &self.options)
    }
}

pub struct TokenDiffBuilder<'a, T> {
    left: &'a TokenStream<T>,
    right: &'a TokenStream<T>,
    options: BuildSidBlocksOptions,
}

impl<'a, T: DiffableToken> TokenDiffBuilder<'a, T> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn run(self) -> Vec<ChapterTokenDiff<T>> {
        diff_chapter_token_streams(&self.left.tokens, &self.right.tokens, &self.options)
    }
}

pub struct UsfmBatchLintBuilder<'a> {
    batch: &'a UsfmBatch,
    options: LintOptions,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchLintBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<LintResult>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.lint(self.options.clone()),
        })
    }
}

pub struct UsfmBatchFormatBuilder<'a> {
    batch: &'a UsfmBatch,
    options: FormatOptions,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchFormatBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<String>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.format(self.options),
        })
    }
}

pub struct UsfmBatchToUsjBuilder<'a> {
    batch: &'a UsfmBatch,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchToUsjBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<UsjDocument, UsjError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usj(),
        })
    }
}

pub struct UsfmBatchToUsxBuilder<'a> {
    batch: &'a UsfmBatch,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchToUsxBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<String, UsxError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usx(),
        })
    }
}

pub struct UsfmBatchToHtmlBuilder<'a> {
    batch: &'a UsfmBatch,
    options: HtmlOptions,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchToHtmlBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<String>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_html(self.options),
        })
    }
}

pub struct UsjBatchToUsxBuilder<'a> {
    batch: &'a UsjBatch,
    execution: ExecutionMode,
}

impl<'a> UsjBatchToUsxBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<String, UsxError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usx(),
        })
    }
}

pub struct TokenBatchLintBuilder<'a, T> {
    batch: &'a TokenBatch<T>,
    options: LintOptions,
    execution: ExecutionMode,
}

impl<'a, T: LintableToken> TokenBatchLintBuilder<'a, T> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<LintResult>> {
        map_batch(&self.batch.streams, self.execution, |stream| BatchItem {
            path: stream.path.clone(),
            value: stream.lint(self.options.clone()),
        })
    }
}

pub struct TokenBatchFormatBuilder<'a, T> {
    batch: &'a TokenBatch<T>,
    options: FormatOptions,
    execution: ExecutionMode,
}

impl<'a, T: FormattableToken + Clone> TokenBatchFormatBuilder<'a, T> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Vec<T>>> {
        map_batch(&self.batch.streams, self.execution, |stream| BatchItem {
            path: stream.path.clone(),
            value: stream.format(self.options),
        })
    }
}

pub struct TokenBatchToUsfmBuilder<'a, T> {
    batch: &'a TokenBatch<T>,
    execution: ExecutionMode,
}

impl<'a, T: SourceTokenText> TokenBatchToUsfmBuilder<'a, T> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<String>> {
        map_batch(&self.batch.streams, self.execution, |stream| BatchItem {
            path: stream.path.clone(),
            value: stream.to_usfm(),
        })
    }
}

pub struct TokenBatchToHtmlBuilder<'a, 'token> {
    batch: &'a TokenBatch<Token<'token>>,
    options: HtmlOptions,
    execution: ExecutionMode,
}

impl<'a, 'token> TokenBatchToHtmlBuilder<'a, 'token> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<String>> {
        map_batch(&self.batch.streams, self.execution, |stream| BatchItem {
            path: stream.path.clone(),
            value: stream.to_html(self.options),
        })
    }
}

fn tokens_to_usfm_text<T: SourceTokenText>(tokens: &[T]) -> String {
    let capacity = tokens.iter().map(|token| token.source_text().len()).sum();
    let mut out = String::with_capacity(capacity);
    for token in tokens {
        out.push_str(token.source_text());
    }
    out
}

#[cfg(feature = "rayon")]
fn map_batch<T, U, F>(items: &[T], execution: ExecutionMode, map: F) -> Vec<U>
where
    T: Sync,
    U: Send,
    F: Fn(&T) -> U + Sync + Send,
{
    use rayon::prelude::*;

    match execution {
        ExecutionMode::Serial => items.iter().map(map).collect(),
        ExecutionMode::Parallel => items.par_iter().map(map).collect(),
    }
}

#[cfg(not(feature = "rayon"))]
fn map_batch<T, U, F>(items: &[T], _execution: ExecutionMode, map: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    items.iter().map(map).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usfm_from_str_and_path_work() {
        let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
        assert!(doc.path().is_none());
        assert_eq!(doc.source(), "\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
    }

    #[test]
    fn usfm_singular_methods_match_engines() {
        let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
        assert_eq!(doc.lint(LintOptions::default()), lint_usfm(doc.source(), LintOptions::default()));
        assert_eq!(doc.to_html(HtmlOptions::default()), usfm_to_html(doc.source(), HtmlOptions::default()));
        assert_eq!(
            doc.to_usj().expect("usj"),
            usfm_to_usj(doc.source()).expect("usj direct")
        );
        assert_eq!(
            doc.to_usx().expect("usx"),
            usfm_to_usx(doc.source()).expect("usx direct")
        );
    }

    #[test]
    fn token_stream_lint_matches_engine() {
        let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
        let tokens = doc.tokens();
        let stream = TokenStream::from_tokens(tokens.clone());
        assert_eq!(
            stream.lint(LintOptions::default()),
            lint_tokens(&tokens, LintOptions::default())
        );
    }

    #[test]
    fn usfm_batch_preserves_order() {
        let batch = UsfmBatch::from_strs([
            "\\id GEN\n\\c 1\n\\p\n\\v 1 One\n",
            "\\id EXO\n\\c 1\n\\p\n\\v 1 Two\n",
        ]);
        let output = batch
            .to_html(HtmlOptions::default())
            .with_execution(ExecutionMode::Serial)
            .run();
        assert_eq!(output.len(), 2);
        assert!(output[0].value.contains("GEN") || output[0].value.contains("One"));
        assert!(output[1].value.contains("EXO") || output[1].value.contains("Two"));
    }

    #[test]
    fn token_batch_matches_repeated_singular_calls() {
        let first_doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 One\n");
        let second_doc = Usfm::from_str("\\id EXO\n\\c 1\n\\p\n\\v 1 Two\n");
        let first = TokenStream::from_tokens(first_doc.tokens());
        let second = TokenStream::from_tokens(second_doc.tokens());
        let batch = TokenBatch::from_token_streams(vec![first.clone(), second.clone()]);

        let batch_output = batch.to_usfm().run();
        assert_eq!(batch_output[0].value, first.to_usfm());
        assert_eq!(batch_output[1].value, second.to_usfm());
    }

    #[cfg(not(feature = "rayon"))]
    #[test]
    fn parallel_execution_falls_back_to_serial_without_rayon() {
        let batch = UsfmBatch::from_strs(["\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n"]);
        let serial = batch
            .to_html(HtmlOptions::default())
            .with_execution(ExecutionMode::Serial)
            .run();
        let parallel = batch
            .to_html(HtmlOptions::default())
            .with_execution(ExecutionMode::Parallel)
            .run();
        assert_eq!(serial, parallel);
    }
}
