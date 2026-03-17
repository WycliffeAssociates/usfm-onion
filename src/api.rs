use std::fs;
use std::path::{Path, PathBuf};

use crate::cst::{CstDocument, parse_cst};
use crate::diff::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffsByChapterMap, DiffableToken,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
};
use crate::format::{FormatOptions, FormatToken, FormattableToken, format, format_mut};
use crate::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use crate::lint::{LintOptions, LintResult, LintableToken, lint_tokens, lint_usfm};
use crate::parse::parse;
use crate::token::{ParseAnalysis, Sid, Token, TokenId};
use crate::usj::{UsjDocument, UsjError, from_usj_str, usfm_to_usj};
use crate::usx::{UsxError, from_usx_str, usfm_to_usx, usj_to_usx, usx_to_usj};
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
pub struct ParsedUsfm {
    path: Option<PathBuf>,
    source: String,
    tokens: Vec<FormatToken>,
    analysis: OwnedParseAnalysis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUsfmBatch {
    docs: Vec<ParsedUsfm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnedParseAnalysis {
    pub book_code: Option<String>,
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

    pub fn parse_owned(&self) -> ParsedUsfm {
        ParsedUsfm::from_usfm(self)
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

    pub fn parse(&self) -> UsfmBatchParseBuilder<'_> {
        UsfmBatchParseBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
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

    pub fn diff<'a>(&'a self, other: &'a UsfmBatch) -> UsfmBatchDiffBuilder<'a> {
        UsfmBatchDiffBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
            execution: ExecutionMode::Serial,
        }
    }

    pub fn diff_by_chapter<'a>(&'a self, other: &'a UsfmBatch) -> UsfmBatchDiffByChapterBuilder<'a> {
        UsfmBatchDiffByChapterBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
            execution: ExecutionMode::Serial,
        }
    }
}

impl ParsedUsfm {
    fn from_usfm(doc: &Usfm) -> Self {
        let parsed = parse(&doc.source);
        Self {
            path: doc.path.clone(),
            source: doc.source.clone(),
            tokens: parsed.tokens.iter().map(format_token_with_identity).collect(),
            analysis: OwnedParseAnalysis::from_borrowed(&parsed.analysis),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn analysis(&self) -> &OwnedParseAnalysis {
        &self.analysis
    }

    pub fn tokens(&self) -> &[FormatToken] {
        &self.tokens
    }

    pub fn into_tokens(self) -> Vec<FormatToken> {
        self.tokens
    }

    pub fn lint(&self, options: LintOptions) -> LintResult {
        lint_tokens(&self.tokens, options)
    }

    pub fn format(&self, options: FormatOptions) -> Vec<FormatToken> {
        format(&self.tokens, options)
    }

    pub fn format_mut(&mut self, options: FormatOptions) {
        format_mut(&mut self.tokens, options);
    }

    pub fn to_usfm(&self) -> String {
        self.source.clone()
    }

    pub fn to_html(&self, options: HtmlOptions) -> String {
        usfm_to_html(&self.source, options)
    }

    pub fn to_usj(&self) -> Result<UsjDocument, UsjError> {
        usfm_to_usj(&self.source)
    }

    pub fn to_usx(&self) -> Result<String, UsxError> {
        usfm_to_usx(&self.source)
    }

    pub fn to_vref(&self) -> VrefMap {
        usfm_to_vref_map(&self.source)
    }

    pub fn diff<'a>(&'a self, other: &'a ParsedUsfm) -> ParsedUsfmDiffBuilder<'a> {
        ParsedUsfmDiffBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }

    pub fn diff_by_chapter<'a>(&'a self, other: &'a ParsedUsfm) -> ParsedUsfmDiffByChapterBuilder<'a> {
        ParsedUsfmDiffByChapterBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }
}

impl ParsedUsfmBatch {
    pub fn items(&self) -> &[ParsedUsfm] {
        &self.docs
    }
}

impl OwnedParseAnalysis {
    fn from_borrowed(analysis: &ParseAnalysis<'_>) -> Self {
        Self {
            book_code: analysis.book_code.map(ToOwned::to_owned),
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

    pub fn to_usfm(&self) -> Result<String, UsjError> {
        from_usj_str(&self.source)
    }
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

    pub fn to_usfm(&self) -> UsjBatchToUsfmBuilder<'_> {
        UsjBatchToUsfmBuilder {
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

    pub fn to_usj(&self) -> Result<UsjDocument, UsxError> {
        usx_to_usj(&self.source)
    }

    pub fn to_usfm(&self) -> Result<String, UsxError> {
        from_usx_str(&self.source)
    }
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

    pub fn to_usj(&self) -> UsxBatchToUsjBuilder<'_> {
        UsxBatchToUsjBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
    }

    pub fn to_usfm(&self) -> UsxBatchToUsfmBuilder<'_> {
        UsxBatchToUsfmBuilder {
            batch: self,
            execution: ExecutionMode::Serial,
        }
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

impl<T: FormattableToken> TokenStream<T> {
    pub fn format_mut(&mut self, options: FormatOptions) {
        format_mut(&mut self.tokens, options);
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

impl<T: DiffableToken> TokenBatch<T> {
    pub fn diff<'a>(&'a self, other: &'a TokenBatch<T>) -> TokenBatchDiffBuilder<'a, T> {
        TokenBatchDiffBuilder {
            batch: self,
            other,
            options: BuildSidBlocksOptions::default(),
            execution: ExecutionMode::Serial,
        }
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

pub struct ParsedUsfmDiffBuilder<'a> {
    left: &'a ParsedUsfm,
    right: &'a ParsedUsfm,
    options: BuildSidBlocksOptions,
}

impl<'a> ParsedUsfmDiffBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn run(self) -> Vec<ChapterTokenDiff<FormatToken>> {
        diff_chapter_token_streams(&self.left.tokens, &self.right.tokens, &self.options)
    }
}

pub struct ParsedUsfmDiffByChapterBuilder<'a> {
    left: &'a ParsedUsfm,
    right: &'a ParsedUsfm,
    options: BuildSidBlocksOptions,
}

impl<'a> ParsedUsfmDiffByChapterBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn run(self) -> DiffsByChapterMap<ChapterTokenDiff<FormatToken>> {
        group_chapter_diffs(diff_chapter_token_streams(
            &self.left.tokens,
            &self.right.tokens,
            &self.options,
        ))
    }
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

pub struct UsfmBatchParseBuilder<'a> {
    batch: &'a UsfmBatch,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchParseBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> ParsedUsfmBatch {
        ParsedUsfmBatch {
            docs: map_batch(&self.batch.docs, self.execution, ParsedUsfm::from_usfm),
        }
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

pub struct UsfmBatchDiffBuilder<'a> {
    left: &'a UsfmBatch,
    right: &'a UsfmBatch,
    options: BuildSidBlocksOptions,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchDiffBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Vec<ChapterTokenDiff<FormatToken>>>> {
        assert_eq!(
            self.left.docs.len(),
            self.right.docs.len(),
            "UsfmBatch::diff requires equal batch lengths"
        );
        map_batch_pairs(&self.left.docs, &self.right.docs, self.execution, |(left, right)| BatchItem {
            path: left.path.clone().or_else(|| right.path.clone()),
            value: diff_chapter_token_streams(
                &ParsedUsfm::from_usfm(left).tokens,
                &ParsedUsfm::from_usfm(right).tokens,
                &self.options,
            ),
        })
    }
}

pub struct UsfmBatchDiffByChapterBuilder<'a> {
    left: &'a UsfmBatch,
    right: &'a UsfmBatch,
    options: BuildSidBlocksOptions,
    execution: ExecutionMode,
}

impl<'a> UsfmBatchDiffByChapterBuilder<'a> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<DiffsByChapterMap<ChapterTokenDiff<FormatToken>>>> {
        assert_eq!(
            self.left.docs.len(),
            self.right.docs.len(),
            "UsfmBatch::diff_by_chapter requires equal batch lengths"
        );
        map_batch_pairs(&self.left.docs, &self.right.docs, self.execution, |(left, right)| {
            let left = ParsedUsfm::from_usfm(left);
            let right = ParsedUsfm::from_usfm(right);
            BatchItem {
                path: left.path.clone().or_else(|| right.path.clone()),
                value: group_chapter_diffs(diff_chapter_token_streams(
                    &left.tokens,
                    &right.tokens,
                    &self.options,
                )),
            }
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

pub struct UsjBatchToUsfmBuilder<'a> {
    batch: &'a UsjBatch,
    execution: ExecutionMode,
}

impl<'a> UsjBatchToUsfmBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<String, UsjError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usfm(),
        })
    }
}

pub struct UsxBatchToUsjBuilder<'a> {
    batch: &'a UsxBatch,
    execution: ExecutionMode,
}

impl<'a> UsxBatchToUsjBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<UsjDocument, UsxError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usj(),
        })
    }
}

pub struct UsxBatchToUsfmBuilder<'a> {
    batch: &'a UsxBatch,
    execution: ExecutionMode,
}

impl<'a> UsxBatchToUsfmBuilder<'a> {
    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Result<String, UsxError>>> {
        map_batch(&self.batch.docs, self.execution, |doc| BatchItem {
            path: doc.path.clone(),
            value: doc.to_usfm(),
        })
    }
}

pub struct TokenBatchLintBuilder<'a, T> {
    batch: &'a TokenBatch<T>,
    options: LintOptions,
    execution: ExecutionMode,
}

impl<'a, T: LintableToken + Sync> TokenBatchLintBuilder<'a, T> {
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

impl<'a, T: FormattableToken + Clone + Sync + Send> TokenBatchFormatBuilder<'a, T> {
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

impl<'a, T: SourceTokenText + Sync> TokenBatchToUsfmBuilder<'a, T> {
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

pub struct TokenBatchDiffBuilder<'a, T> {
    batch: &'a TokenBatch<T>,
    other: &'a TokenBatch<T>,
    options: BuildSidBlocksOptions,
    execution: ExecutionMode,
}

impl<'a, T: DiffableToken + Sync + Send> TokenBatchDiffBuilder<'a, T> {
    pub fn with_options(mut self, options: BuildSidBlocksOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn run(self) -> Vec<BatchItem<Vec<ChapterTokenDiff<T>>>> {
        assert_eq!(
            self.batch.streams.len(),
            self.other.streams.len(),
            "TokenBatch::diff requires equal batch lengths"
        );
        map_batch_pairs(&self.batch.streams, &self.other.streams, self.execution, |(left, right)| {
            BatchItem {
                path: left.path.clone().or_else(|| right.path.clone()),
                value: diff_chapter_token_streams(&left.tokens, &right.tokens, &self.options),
            }
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

fn format_token_with_identity(token: &Token<'_>) -> FormatToken {
    let mut owned = FormatToken::from(token);
    owned.sid = token.sid.map(format_sid);
    owned.id = Some(format_token_id(token.id));
    owned
}

fn format_sid(sid: Sid<'_>) -> String {
    if sid.verse == 0 {
        format!("{} {}:0", sid.book_code, sid.chapter)
    } else {
        format!("{} {}:{}", sid.book_code, sid.chapter, sid.verse)
    }
}

fn format_token_id(id: TokenId<'_>) -> String {
    format!("{}-{}", id.book_code, id.index)
}

fn group_chapter_diffs(
    diffs: Vec<ChapterTokenDiff<FormatToken>>,
) -> DiffsByChapterMap<ChapterTokenDiff<FormatToken>> {
    let mut by_chapter = DiffsByChapterMap::default();
    for diff in diffs {
        let (chapter_sid, chapter_num) = chapter_key_from_semantic_sid(&diff.semantic_sid);
        by_chapter
            .entry(chapter_sid)
            .or_default()
            .entry(chapter_num)
            .or_default()
            .push(diff);
    }
    by_chapter
}

fn chapter_key_from_semantic_sid(semantic_sid: &str) -> (String, u32) {
    let mut parts = semantic_sid.split_whitespace();
    let book = parts.next().unwrap_or("").to_string();
    let chapter = parts
        .next()
        .and_then(|part| part.split(':').next())
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    (book, chapter)
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

#[cfg(feature = "rayon")]
fn map_batch_pairs<L, R, U, F>(left: &[L], right: &[R], execution: ExecutionMode, map: F) -> Vec<U>
where
    L: Sync,
    R: Sync,
    U: Send,
    F: Fn((&L, &R)) -> U + Sync + Send,
{
    use rayon::prelude::*;

    match execution {
        ExecutionMode::Serial => left.iter().zip(right.iter()).map(map).collect(),
        ExecutionMode::Parallel => left.par_iter().zip(right.par_iter()).map(map).collect(),
    }
}

#[cfg(not(feature = "rayon"))]
fn map_batch_pairs<L, R, U, F>(left: &[L], right: &[R], _execution: ExecutionMode, map: F) -> Vec<U>
where
    F: Fn((&L, &R)) -> U,
{
    left.iter().zip(right.iter()).map(map).collect()
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
    fn reverse_import_facades_work() {
        let usj = Usj::from_str("{\"type\":\"USJ\",\"version\":\"3.1\",\"content\":[{\"type\":\"book\",\"marker\":\"id\",\"code\":\"GEN\"},{\"type\":\"chapter\",\"marker\":\"c\",\"number\":\"1\"},{\"type\":\"para\",\"marker\":\"p\",\"content\":[{\"type\":\"verse\",\"marker\":\"v\",\"number\":\"1\"},\"Text\"]}]}");
        let usx = Usx::from_str("<usx version=\"3.0\"><book code=\"GEN\" style=\"id\"/><chapter number=\"1\" style=\"c\" sid=\"GEN 1\"/><para style=\"p\"><verse number=\"1\" style=\"v\" sid=\"GEN 1:1\"/>Text<verse eid=\"GEN 1:1\"/></para><chapter eid=\"GEN 1\"/></usx>");

        assert!(usj.to_usfm().expect("usj -> usfm").contains("\\v 1 Text"));
        assert_eq!(
            usx.to_usj().expect("usx -> usj"),
            usx_to_usj(usx.source()).expect("direct usx -> usj")
        );
        assert!(usx.to_usfm().expect("usx -> usfm").contains("\\v 1 Text"));
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
    fn parse_batch_and_batch_diff_preserve_order() {
        let left = UsfmBatch::from_strs([
            "\\id GEN\n\\c 1\n\\p\n\\v 1 One\n",
            "\\id EXO\n\\c 1\n\\p\n\\v 1 Two\n",
        ]);
        let right = UsfmBatch::from_strs([
            "\\id GEN\n\\c 1\n\\p\n\\v 1 Uno\n",
            "\\id EXO\n\\c 1\n\\p\n\\v 1 Dos\n",
        ]);

        let parsed = left.parse().with_execution(ExecutionMode::Serial).run();
        assert_eq!(parsed.items().len(), 2);
        assert_eq!(parsed.items()[0].analysis().book_code.as_deref(), Some("GEN"));
        assert_eq!(parsed.items()[1].analysis().book_code.as_deref(), Some("EXO"));

        let diffs = left
            .diff(&right)
            .with_execution(ExecutionMode::Serial)
            .run();
        assert_eq!(diffs.len(), 2);
        assert!(diffs.iter().all(|item| !item.value.is_empty()));
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

    #[test]
    fn token_batch_diff_matches_repeated_singular_calls() {
        let left_first_doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 One\n");
        let left_second_doc = Usfm::from_str("\\id EXO\n\\c 1\n\\p\n\\v 1 Two\n");
        let right_first_doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Uno\n");
        let right_second_doc = Usfm::from_str("\\id EXO\n\\c 1\n\\p\n\\v 1 Dos\n");

        let left_first = TokenStream::from_tokens(left_first_doc.tokens());
        let left_second = TokenStream::from_tokens(left_second_doc.tokens());
        let right_first = TokenStream::from_tokens(right_first_doc.tokens());
        let right_second = TokenStream::from_tokens(right_second_doc.tokens());

        let left = TokenBatch::from_token_streams(vec![left_first.clone(), left_second.clone()]);
        let right = TokenBatch::from_token_streams(vec![right_first.clone(), right_second.clone()]);

        let batch_output = left.diff(&right).with_execution(ExecutionMode::Serial).run();
        assert_eq!(
            batch_output[0].value,
            left_first.diff(&right_first).run()
        );
        assert_eq!(
            batch_output[1].value,
            left_second.diff(&right_second).run()
        );
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
