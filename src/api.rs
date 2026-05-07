use crate::cst::{CstDocument, parse_cst};
use crate::diff::{
    BuildSidBlocksOptions, ChapterTokenDiff, DiffableToken, DiffsByChapterMap,
    diff_chapter_token_streams, diff_usfm_sources, diff_usfm_sources_by_chapter,
};
use crate::format::{FormatOptions, FormatToken, FormattableToken, format, format_mut};
use crate::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use crate::lint::{
    LintOptions, LintResult, LintableToken, TokenFix, apply_token_fix, lint_tokens, lint_usfm,
};
use crate::parse::parse;
use crate::token::{ParseAnalysis, Sid, Token, TokenId};
use crate::usj::{UsjDocument, UsjError, from_usj_str, usfm_to_usj};
use crate::usx::{UsxError, from_usx_str, usfm_to_usx, usj_to_usx, usx_to_usj};
use crate::vref::{VrefMap, usfm_to_vref_map};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usfm {
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUsfm {
    source: String,
    tokens: Vec<FormatToken>,
    analysis: OwnedParseAnalysis,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnedParseAnalysis {
    pub book_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usj {
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usx {
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream<T> {
    tokens: Vec<T>,
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
            source: source.to_string(),
        }
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

    pub fn apply_token_fix(&self, fix: &TokenFix) -> Vec<FormatToken> {
        self.parse_owned().apply_token_fix(fix)
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

impl ParsedUsfm {
    fn from_usfm(doc: &Usfm) -> Self {
        let parsed = parse(&doc.source);
        Self {
            source: doc.source.clone(),
            tokens: parsed
                .tokens
                .iter()
                .map(format_token_with_identity)
                .collect(),
            analysis: OwnedParseAnalysis::from_borrowed(&parsed.analysis),
        }
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

    pub fn apply_token_fix(&self, fix: &TokenFix) -> Vec<FormatToken> {
        apply_token_fix(&self.tokens, fix)
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

    pub fn diff_by_chapter<'a>(
        &'a self,
        other: &'a ParsedUsfm,
    ) -> ParsedUsfmDiffByChapterBuilder<'a> {
        ParsedUsfmDiffByChapterBuilder {
            left: self,
            right: other,
            options: BuildSidBlocksOptions::default(),
        }
    }

    pub fn revert_diff_block(
        &self,
        current: &ParsedUsfm,
        diff_block_id: &str,
        options: BuildSidBlocksOptions,
    ) -> Vec<FormatToken> {
        crate::diff::apply_revert_by_block_id(
            diff_block_id,
            &self.tokens,
            &current.tokens,
            &options,
        )
    }

    pub fn revert_diff_blocks(
        &self,
        current: &ParsedUsfm,
        diff_block_ids: &[String],
        options: BuildSidBlocksOptions,
    ) -> Vec<FormatToken> {
        crate::diff::apply_reverts_by_block_id(
            diff_block_ids,
            &self.tokens,
            &current.tokens,
            &options,
        )
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
            source: source.to_string(),
        }
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

impl Usx {
    pub fn from_str(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
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

impl<T> TokenStream<T> {
    pub fn from_tokens(tokens: Vec<T>) -> Self {
        Self { tokens }
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

impl<T: FormattableToken> TokenStream<T> {
    pub fn apply_token_fix(&self, fix: &TokenFix) -> Vec<T> {
        apply_token_fix(&self.tokens, fix)
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

    pub fn revert_diff_block(
        &self,
        current: &TokenStream<T>,
        diff_block_id: &str,
        options: BuildSidBlocksOptions,
    ) -> Vec<T> {
        crate::diff::apply_revert_by_block_id(
            diff_block_id,
            &self.tokens,
            &current.tokens,
            &options,
        )
    }

    pub fn revert_diff_blocks(
        &self,
        current: &TokenStream<T>,
        diff_block_ids: &[String],
        options: BuildSidBlocksOptions,
    ) -> Vec<T> {
        crate::diff::apply_reverts_by_block_id(
            diff_block_ids,
            &self.tokens,
            &current.tokens,
            &options,
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LintCode;

    #[test]
    fn usfm_from_str_works() {
        let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
        assert_eq!(doc.source(), "\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
    }

    #[test]
    fn usfm_singular_methods_match_engines() {
        let doc = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Text\n");
        assert_eq!(
            doc.lint(LintOptions::default()),
            lint_usfm(doc.source(), LintOptions::default())
        );
        assert_eq!(
            doc.to_html(HtmlOptions::default()),
            usfm_to_html(doc.source(), HtmlOptions::default())
        );
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
        let usj = Usj::from_str(
            "{\"type\":\"USJ\",\"version\":\"3.1\",\"content\":[{\"type\":\"book\",\"marker\":\"id\",\"code\":\"GEN\"},{\"type\":\"chapter\",\"marker\":\"c\",\"number\":\"1\"},{\"type\":\"para\",\"marker\":\"p\",\"content\":[{\"type\":\"verse\",\"marker\":\"v\",\"number\":\"1\"},\"Text\"]}]}",
        );
        let usx = Usx::from_str(
            "<usx version=\"3.0\"><book code=\"GEN\" style=\"id\"/><chapter number=\"1\" style=\"c\" sid=\"GEN 1\"/><para style=\"p\"><verse number=\"1\" style=\"v\" sid=\"GEN 1:1\"/>Text<verse eid=\"GEN 1:1\"/></para><chapter eid=\"GEN 1\"/></usx>",
        );

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
    fn apply_token_fix_and_revert_diff_block_are_available() {
        let baseline = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha\n");
        let changed = Usfm::from_str("\\id GEN\n\\c 1\n\\p\n\\v 1Alpha\n");
        let malformed_tokens = vec![
            crate::FormatToken {
                kind: crate::TokenKind::Marker,
                text: "\\p".to_string(),
                marker: Some("p".to_string()),
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-0".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
            crate::FormatToken {
                kind: crate::TokenKind::Text,
                text: "Alpha".to_string(),
                marker: None,
                sid: Some("GEN 1:1".to_string()),
                id: Some("GEN-1".to_string()),
                span: None,
                structural: None,
                number_info: None,
                marker_profile: None,
            },
        ];
        let issue = TokenStream::from_tokens(malformed_tokens.clone())
            .lint(LintOptions::default())
            .issues
            .into_iter()
            .find(|issue| issue.code == LintCode::MissingSeparatorAfterMarker)
            .expect("expected missing-separator issue");
        let fix = issue.fix.expect("expected concrete token fix");
        let fixed = TokenStream::from_tokens(malformed_tokens).apply_token_fix(&fix);
        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed[1].text, " Alpha");

        let baseline_parsed = baseline.parse_owned();
        let changed_parsed = changed.parse_owned();
        let diffs = baseline_parsed.diff(&changed_parsed).run();
        let verse_diff = diffs
            .iter()
            .find(|diff| {
                diff.status == crate::DiffStatus::Modified && diff.semantic_sid.ends_with("1:1")
            })
            .expect("expected modified verse diff");
        let reverted = baseline_parsed.revert_diff_block(
            &changed_parsed,
            &verse_diff.block_id,
            BuildSidBlocksOptions::default(),
        );
        assert_eq!(reverted, baseline_parsed.tokens().to_vec());
    }
}
