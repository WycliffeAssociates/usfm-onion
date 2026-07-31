//! The resident corpus handle: `Braid` as a JS class.
//!
//! Every verb here operates on ingested resident state and nothing else. The
//! stateless exports next door take their input from the caller; these take theirs
//! from the corpus the handle is holding. That separation is deliberate even where
//! both reach the same core function, because "operate on what I gave you" and
//! "operate on what you are holding" are different promises.
//!
//! Failures a caller can act on are values, not exceptions: every verb that can be
//! refused returns [`ApiResult`] carrying the same typed error the Rust crate
//! returns. Exceptions are reserved for programmer errors — a minter that is not
//! callable, a value that is not the shape its type claims.

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use braid::Braid as NativeBraid;

use crate::dto::{LintOptions, lint_options_into_native};

/// A verb's outcome: the value it produced, or the typed reason it was refused.
///
/// Tagged on a string rather than a boolean, matching the crate's existing packed
/// outcome type: a string tag is what makes this a real discriminated union in
/// TypeScript, so a consumer that checks `status` has the other field narrowed for
/// it rather than asserted by hand.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ApiResult<T, E> {
    Ok { value: T },
    Error { error: E },
}

impl<T, E> ApiResult<T, E> {
    fn of(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Ok { value },
            Err(error) => Self::Error { error },
        }
    }
}

/// The resident configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BraidConfig {
    pub lint: LintOptions,
}

impl BraidConfig {
    fn into_native(self) -> braid::BraidConfig {
        braid::BraidConfig::new(lint_options_into_native(self.lint))
    }
}

/// The resident corpus handle.
#[wasm_bindgen]
pub struct Braid {
    inner: NativeBraid,
}

#[wasm_bindgen]
impl Braid {
    /// Creates an empty handle bound to the application's own id minter.
    ///
    /// The minter is a JS callback returning a string, held for the life of the
    /// handle: core never invents a token id, so every token a fix or format pass
    /// synthesizes gets one from here. Speed, spelling, and collision resistance
    /// are the application's trade — uniqueness is not assumed but enforced at the
    /// residency boundary, where a collision is a typed rejection rather than a
    /// corrupted book.
    ///
    /// Throws only for a programmer error: a minter that throws, or one that
    /// returns something other than a string.
    #[wasm_bindgen(constructor)]
    pub fn new(config: BraidConfig, minter: js_sys::Function) -> Braid {
        let mint = move || {
            minter
                .call0(&JsValue::NULL)
                .expect("the id minter must not throw")
                .as_string()
                .expect("the id minter must return a string")
        };
        Braid {
            inner: NativeBraid::new(config.into_native(), mint),
        }
    }

    /// The corpus's content-derived identity, as a 16-digit hex string.
    ///
    /// Hex rather than a number because the value is 64 bits: a JS `number` cannot
    /// hold it without silently rounding, and a `bigint` does not survive every
    /// structured clone a worker boundary performs.
    #[wasm_bindgen(js_name = expectedSnapshotId)]
    pub fn expected_snapshot_id(&self) -> String {
        format!("{:016x}", self.inner.expected_snapshot_id().0)
    }
}

/// Proves the generic result type projects through tsify and wasm-bindgen before
/// the verbs depend on it. Removed once a real verb returns one.
#[allow(dead_code)]
fn _projects(result: Result<String, String>) -> ApiResult<String, String> {
    ApiResult::of(result)
}

// ---------------------------------------------------------------------------
// Inputs. Discriminated unions, not optional bags: a book arriving as bytes and
// a book arriving as tokens are different inputs with different obligations, and
// a shape that could be neither is one a caller can build by accident.
// ---------------------------------------------------------------------------

/// How a book's exact bytes end their lines when they have to be re-emitted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl From<LineEnding> for braid::LineEnding {
    fn from(value: LineEnding) -> Self {
        match value {
            LineEnding::Lf => Self::Lf,
            LineEnding::Crlf => Self::CrLf,
        }
    }
}

impl From<braid::LineEnding> for LineEnding {
    fn from(value: braid::LineEnding) -> Self {
        match value {
            braid::LineEnding::Lf => Self::Lf,
            braid::LineEnding::CrLf => Self::Crlf,
        }
    }
}

/// One book's worth of resident input.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BookInput {
    /// Cold load: exact USFM bytes, kept verbatim.
    Usfm {
        source_key: String,
        book: String,
        source: String,
    },
    /// Live push: the caller's own token array, which is the only moment the
    /// caller knows something the corpus does not.
    Tokens {
        source_key: String,
        book: String,
        tokens: Vec<crate::Token>,
        line_ending: LineEnding,
    },
}

/// A complete ordered corpus. Caller order is preserved exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CorpusInput {
    pub books: Vec<BookInput>,
}

/// A chapter run's label, exactly as the source spells it.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChapterLabel {
    FrontMatter,
    /// The label token verbatim: `1`, `01`, and `1a` are three distinct labels,
    /// because nothing here parses or normalizes one.
    Number {
        label: String,
    },
}

impl From<ChapterLabel> for braid::ChapterLabel {
    fn from(value: ChapterLabel) -> Self {
        match value {
            ChapterLabel::FrontMatter => Self::FrontMatter,
            ChapterLabel::Number { label } => Self::Number(label.into()),
        }
    }
}

impl From<&braid::ChapterLabel> for ChapterLabel {
    fn from(value: &braid::ChapterLabel) -> Self {
        match value {
            braid::ChapterLabel::FrontMatter => Self::FrontMatter,
            braid::ChapterLabel::Number(label) => Self::Number {
                label: label.to_string(),
            },
        }
    }
}

/// One chapter run's address.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTarget {
    pub book: String,
    pub label: ChapterLabel,
}

/// One chapter run's replacement content.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChapterInput {
    Tokens { tokens: Vec<crate::Token> },
}

/// A read or projection selector over resident data.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CorpusScope {
    All,
    Book { book: String },
    Chapter { target: ChapterTarget },
}

// ---------------------------------------------------------------------------
// Outputs.
// ---------------------------------------------------------------------------

/// What one mutation rewrote. `chapter` absent means the whole book.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub book: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterLabel>,
}

/// The value every mutating verb returns, after it has already applied.
///
/// `changed` is exact — what was rewritten, not what was inspected — so an empty
/// one means nothing needs re-pulling. Findings are absent by design: lint is an
/// explicit separate call.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MutationEffect {
    /// The corpus identity *after* the mutation, as 16 hex digits.
    pub snapshot_id: String,
    pub changed: Vec<Scope>,
    pub removed: Vec<String>,
    /// The full new book order, when the relative order of the books present both
    /// before and after actually changed. A pure reorder rewrites no tokens, so it
    /// appears here and nowhere else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reordered: Option<Vec<String>>,
}

impl From<braid::MutationEffect> for MutationEffect {
    fn from(effect: braid::MutationEffect) -> Self {
        Self {
            snapshot_id: format!("{:016x}", effect.snapshot_id.0),
            changed: effect.changed.iter().map(scope_out).collect(),
            removed: effect
                .removed
                .iter()
                .map(|book| book.as_str().to_string())
                .collect(),
            reordered: effect
                .reordered
                .as_ref()
                .map(|order| order.iter().map(|book| book.as_str().to_string()).collect()),
        }
    }
}

fn scope_out(scope: &braid::Scope) -> Scope {
    Scope {
        book: scope.book.as_str().to_string(),
        chapter: scope.chapter.as_ref().map(ChapterLabel::from),
    }
}

/// One resident book's identity and derived stamps, in corpus order.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BookEntry {
    pub source_key: String,
    pub book: String,
    /// 16 hex digits over the book's exact bytes.
    pub source_hash: String,
    /// 16 hex digits over everything the book's tokens carry that its bytes do
    /// not — the fact a consumer caching anything token-derived keys on.
    pub token_identity: String,
    pub line_ending: LineEnding,
}

/// One pulled scope's current tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ScopeTokens {
    pub book: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterLabel>,
    pub tokens: Vec<crate::Token>,
}

/// A projection over one scope, or over every resident book in corpus order.
///
/// Ordered pairs for the `all` case rather than an object keyed by source key:
/// corpus order is a contract, and an object's key enumeration is not.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScopedOutput<T> {
    Single { value: T },
    All { books: Vec<SourceOutput<T>> },
}

/// One book's value in an `all`-scoped projection.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SourceOutput<T> {
    pub source_key: String,
    pub book: String,
    pub value: T,
}

fn scoped_out<N, T>(value: braid::ScopedOutput<N>, map: impl Fn(N) -> T) -> ScopedOutput<T> {
    match value {
        braid::ScopedOutput::Single(value) => ScopedOutput::Single { value: map(value) },
        braid::ScopedOutput::All(books) => ScopedOutput::All {
            books: books
                .into_iter()
                .map(|entry| SourceOutput {
                    source_key: entry.source_key.as_str().to_string(),
                    book: entry.book.as_str().to_string(),
                    value: map(entry.value),
                })
                .collect(),
        },
    }
}

/// One book's lint contribution.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BookLintSnapshot {
    pub source_key: String,
    pub book: String,
    pub source_hash: String,
    pub token_identity: String,
    pub findings: Vec<crate::LintIssue>,
    pub summary: crate::LintSummary,
}

/// The complete resident lint snapshot, in corpus order.
///
/// Findings, not packed bytes: a finding's message is rendered by exactly one
/// renderer in one language, so findings cross this boundary already materialized.
/// Packed bytes are a separate question with a separate verb.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSnapshot {
    pub snapshot_id: String,
    pub books: Vec<BookLintSnapshot>,
    pub summary: crate::LintSummary,
}

// ---------------------------------------------------------------------------
// Typed refusals. Each carries the same information the Rust crate's error does,
// as a discriminated union a consumer can switch on.
// ---------------------------------------------------------------------------

/// A mutation refused before it touched resident state.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IngestError {
    DuplicateBook {
        book: String,
        sources: Vec<String>,
    },
    DuplicateSourceKey {
        source: String,
    },
    DuplicateTokenId {
        book: String,
        id: String,
    },
    ChapterNotFound {
        target: ChapterTarget,
    },
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
    ReplacementLabelMismatch {
        target: ChapterTarget,
        found: ChapterLabel,
    },
    /// A token the caller supplied cannot become resident. `message` is core's own
    /// verdict, which names the token and the fact that made it illegal.
    InvalidToken {
        message: String,
    },
}

fn target_out(target: &braid::ChapterTarget) -> ChapterTarget {
    ChapterTarget {
        book: target.book.as_str().to_string(),
        label: ChapterLabel::from(&target.label),
    }
}

impl From<braid::IngestError> for IngestError {
    fn from(error: braid::IngestError) -> Self {
        match error {
            braid::IngestError::DuplicateBook { book, sources } => Self::DuplicateBook {
                book: book.as_str().to_string(),
                sources: sources.iter().map(|key| key.as_str().to_string()).collect(),
            },
            braid::IngestError::DuplicateSourceKey { source } => Self::DuplicateSourceKey {
                source: source.as_str().to_string(),
            },
            braid::IngestError::DuplicateTokenId { book, id } => Self::DuplicateTokenId {
                book: book.as_str().to_string(),
                id: id.as_str().to_string(),
            },
            braid::IngestError::ChapterNotFound(target) => Self::ChapterNotFound {
                target: target_out(&target),
            },
            braid::IngestError::AmbiguousChapter { target, matches } => Self::AmbiguousChapter {
                target: target_out(&target),
                matches,
            },
            braid::IngestError::ReplacementLabelMismatch { target, found } => {
                Self::ReplacementLabelMismatch {
                    target: target_out(&target),
                    found: ChapterLabel::from(&found),
                }
            }
            ref error @ braid::IngestError::InvalidToken(_) => Self::InvalidToken {
                message: error.to_string(),
            },
        }
    }
}

/// A scope that does not resolve against the resident corpus.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScopeError {
    BookNotFound {
        book: String,
    },
    ChapterNotFound {
        target: ChapterTarget,
    },
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
}

impl From<braid::ScopeError> for ScopeError {
    fn from(error: braid::ScopeError) -> Self {
        match error {
            braid::ScopeError::BookNotFound(book) => Self::BookNotFound {
                book: book.as_str().to_string(),
            },
            braid::ScopeError::ChapterNotFound(target) => Self::ChapterNotFound {
                target: target_out(&target),
            },
            braid::ScopeError::AmbiguousChapter { target, matches } => Self::AmbiguousChapter {
                target: target_out(&target),
                matches,
            },
        }
    }
}
