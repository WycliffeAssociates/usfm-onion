//! Strict inputs and selectors.
//!
//! Every resident input names its book and its source binding explicitly. The
//! declared [`BookId`] is authoritative for corpus addressing; the source's own
//! `\id` token stays editable content and may temporarily disagree (that
//! disagreement is a lint finding, not an ingest failure).

use usfm_onion::token::{BookId, OwnedToken};

pub use usfm_onion::token::LineEnding;

use crate::stamps::{LintConfigFingerprint, LintEngineStamp};
use crate::state::BookLintPrime;

/// The caller's opaque binding for where a resident book came from — normally
/// a path.
///
/// Not written into any persisted artifact and not part of semantic identity: a
/// moved file rebinds metadata without invalidating hashes or caches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceKey(Box<str>);

impl SourceKey {
    /// Returns `None` for an empty key, which cannot bind anything.
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SourceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One book's worth of resident input.
#[derive(Debug, Clone)]
pub enum BookInput {
    /// Cold load: exact USFM bytes. The line ending is detected from them and
    /// the bytes are kept verbatim — mixed endings are never normalized on
    /// ingest.
    Usfm {
        source_key: SourceKey,
        book: BookId,
        source: String,
    },
    /// Live push: the caller's own token stream, which is the only moment the
    /// caller knows something braid does not.
    Tokens(BookTokensInput),
}

impl BookInput {
    pub fn book(&self) -> BookId {
        match self {
            Self::Usfm { book, .. } => *book,
            Self::Tokens(input) => input.book,
        }
    }

    pub fn source_key(&self) -> &SourceKey {
        match self {
            Self::Usfm { source_key, .. } => source_key,
            Self::Tokens(input) => &input.source_key,
        }
    }
}

/// Token ingest for a whole book. The line ending is declared rather than
/// detected: a token stream has no file to read it from.
#[derive(Debug, Clone)]
pub struct BookTokensInput {
    pub source_key: SourceKey,
    pub book: BookId,
    pub tokens: Vec<OwnedToken>,
    pub line_ending: LineEnding,
}

/// One chapter run's worth of replacement content. The book's stored line
/// ending is inherited; a chapter never declares its own.
///
/// Tokens-only by design (owner-ruled): a raw-USFM `Usfm { source: String }`
/// variant existed briefly and was removed. A bare chapter fragment has no
/// `\id`, so `parse` can address none of its tokens — no book code means no
/// sid at all, and its ids are positional to that one fragment's parse, so a
/// second raw-text splice into a different run of the same book collides on
/// `IngestError::DuplicateTokenId` the moment two fragments both start their
/// own token numbering from zero. No real caller needs it either: an editor
/// speaks tokens (it already has the caller-addressed stream in memory), and
/// raw USFM enters at book/corpus grain (`BookInput::Usfm`), where a real
/// `\id` makes addressing possible in the first place. `ChapterInput::Tokens`
/// carries the caller's own ids/sids and has neither problem.
///
/// If a raw-text chapter lane is ever wanted again, the design is: splice the
/// caller's bytes into the *book's* authoritative source at the run's byte
/// range, then reparse the whole book. Parsing in the book's own context is
/// what gives the new content a real `\id`-backed sid and ids, instead of
/// parsing the fragment in isolation and hoping to reconcile placeholder
/// identities afterward. Recorded here so a future need does not re-litigate
/// this from scratch.
#[derive(Debug, Clone)]
pub enum ChapterInput {
    Tokens(Vec<OwnedToken>),
}

/// A chapter run's label, exactly as the source spells it.
///
/// `Number` holds the label token verbatim rather than a parsed integer:
/// nothing here sorts numerically or normalizes, so `1`, `01`, and `1a` stay
/// three distinct labels. Front matter is a variant, never a magic string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChapterLabel {
    FrontMatter,
    Number(Box<str>),
}

impl std::fmt::Display for ChapterLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FrontMatter => f.write_str("front matter"),
            Self::Number(label) => write!(f, "chapter {label}"),
        }
    }
}

/// One chapter run's address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChapterTarget {
    pub book: BookId,
    pub label: ChapterLabel,
}

impl ChapterTarget {
    pub fn new(book: BookId, label: ChapterLabel) -> Self {
        Self { book, label }
    }
}

impl std::fmt::Display for ChapterTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of book {}", self.label, self.book)
    }
}

/// A complete ordered corpus. Caller order is preserved exactly — no canonical
/// or numeric reordering happens anywhere in braid.
#[derive(Debug, Clone, Default)]
pub struct CorpusInput {
    pub books: Vec<BookInput>,
}

impl CorpusInput {
    pub fn new(books: Vec<BookInput>) -> Self {
        Self { books }
    }
}

/// A complete corpus to seed from already-decoded state — the warm cold-open.
///
/// The point is that nothing here needs lexing or parsing: the caller (or the
/// adapter that decoded a packed container for it) already holds both the exact
/// bytes and the semantic tokens, so braid only has to check that they agree and
/// install them.
///
/// `config_fingerprint`/`engine_stamp` gate every book's *optional* cached
/// `lint` contribution as one batch: a mismatch here means nothing in this
/// seed's `lint` fields can be trusted, so every book that carries one is
/// rejected (its cached contribution only — the book itself still seeds; see
/// [`crate::RestoreReport`]) rather than checked one at a time against a value
/// already known to be wrong.
#[derive(Debug, Clone, Default)]
pub struct CorpusRestoreInput {
    pub config_fingerprint: LintConfigFingerprint,
    pub engine_stamp: LintEngineStamp,
    pub books: Vec<BookRestoreInput>,
}

impl CorpusRestoreInput {
    pub fn new(
        config_fingerprint: LintConfigFingerprint,
        engine_stamp: LintEngineStamp,
        books: Vec<BookRestoreInput>,
    ) -> Self {
        Self {
            config_fingerprint,
            engine_stamp,
            books,
        }
    }
}

/// One book's seed: its exact bytes *and* the tokens they decode to.
///
/// The two must agree, and braid checks it. A pairing it cannot verify is the
/// one thing a warm restore must never install: after a restore the first edit
/// re-emits the whole book from its tokens, so tokens that do not spell these
/// bytes would silently rewrite parts of the file nobody touched.
///
/// `lint` is this book's optional cached contribution, validated against this
/// same book's post-seed hash and the batch's two stamps before it is
/// adopted; a rejection leaves the book seeded (tokens installed, no lex or
/// parse) but dirty, exactly as if `lint` had been absent.
#[derive(Debug, Clone)]
pub struct BookRestoreInput {
    pub source_key: SourceKey,
    pub book: BookId,
    pub source: String,
    pub tokens: Vec<OwnedToken>,
    pub line_ending: LineEnding,
    pub lint: Option<BookLintPrime>,
}

/// A read/projection selector over resident data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CorpusScope {
    All,
    Book(BookId),
    Chapter(ChapterTarget),
}

/// A projection over one scope, or over every resident book in corpus order.
///
/// Ordered rather than a map, so the caller cannot lose corpus order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScopedOutput<T> {
    Single(T),
    All(Vec<SourceOutput<T>>),
}

/// One book's value in an `All`-scoped projection, with the metadata needed to
/// route it back to a file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceOutput<T> {
    pub source_key: SourceKey,
    pub book: BookId,
    pub value: T,
}
