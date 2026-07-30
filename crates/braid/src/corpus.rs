//! One resident book: authoritative bytes, owned tokens, and the ordered
//! duplicate-preserving chapter runs derived from them.

use std::ops::Range;

use rustc_hash::FxHashSet;

use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, LineEnding, OwnedToken, tokens_to_usfm_reconstruct_with_eol};
use usfm_onion::walker::chapter_segments;

use crate::error::IngestError;
use crate::input::{BookInput, ChapterLabel, SourceKey};
use crate::state::SourceHash;

/// One contiguous chapter run: its label and its token range in the book.
///
/// A book may hold several runs with the same label — duplicate and reopened
/// `\c` numbers are retained in source order, never collapsed or sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChapterRun {
    pub(crate) label: ChapterLabel,
    pub(crate) range: Range<usize>,
}

/// A validated, fully derived book, built as a candidate before any resident
/// state is touched.
#[derive(Debug, Clone)]
pub(crate) struct BookState {
    pub(crate) source_key: SourceKey,
    pub(crate) book: BookId,
    /// The exact bytes this book would be saved as.
    pub(crate) source: String,
    pub(crate) hash: SourceHash,
    pub(crate) tokens: Vec<OwnedToken>,
    pub(crate) runs: Vec<ChapterRun>,
    pub(crate) line_ending: LineEnding,
    /// Set whenever this book's content or the lint config changed, cleared by
    /// a lint run. Derived from authoritative state rather than consumed from a
    /// queue, so retrying after a failure is safe.
    pub(crate) lint_dirty: bool,
}

impl BookState {
    /// Builds a candidate book. Every failure here happens before the caller's
    /// resident state is touched.
    pub(crate) fn build(input: BookInput) -> Result<Self, IngestError> {
        let (source_key, book, source, tokens, line_ending) = match input {
            BookInput::Usfm {
                source_key,
                book,
                source,
            } => {
                let line_ending = LineEnding::detect(&source);
                let tokens = parse(&source)
                    .tokens
                    .iter()
                    .map(OwnedToken::from_parsed)
                    .collect();
                // The bytes stay exactly as supplied: a mixed-ending file is
                // preserved verbatim until an edit forces re-emission, at
                // which point the detected ending applies.
                (source_key, book, source, tokens, line_ending)
            }
            BookInput::Tokens(input) => {
                let source = tokens_to_usfm_reconstruct_with_eol(&input.tokens, input.line_ending);
                (
                    input.source_key,
                    input.book,
                    source,
                    input.tokens,
                    input.line_ending,
                )
            }
        };

        validate_token_ids(book, &tokens)?;
        Ok(Self {
            source_key,
            book,
            hash: SourceHash::of(&source),
            source,
            runs: chapter_runs(&tokens),
            tokens,
            line_ending,
            lint_dirty: true,
        })
    }

    /// Rebuilds bytes, hash, and runs from an already-validated token stream —
    /// the chapter-splice path, where the tokens are the mutation and
    /// everything else is derived from them.
    pub(crate) fn rebuilt(&self, tokens: Vec<OwnedToken>) -> Result<Self, IngestError> {
        validate_token_ids(self.book, &tokens)?;
        let source = tokens_to_usfm_reconstruct_with_eol(&tokens, self.line_ending);
        Ok(Self {
            source_key: self.source_key.clone(),
            book: self.book,
            hash: SourceHash::of(&source),
            source,
            runs: chapter_runs(&tokens),
            tokens,
            line_ending: self.line_ending,
            lint_dirty: true,
        })
    }

    /// Exact content equality — the check that decides a no-op. Hash equality
    /// alone never decides it, and token equality is part of it because two
    /// streams can serialize to the same bytes while carrying different token
    /// identities.
    pub(crate) fn content_eq(&self, other: &Self) -> bool {
        self.hash == other.hash
            && self.source == other.source
            && self.line_ending == other.line_ending
            && self.tokens == other.tokens
    }

    /// True when any label appears on more than one run, which makes every
    /// chapter address in this book ambiguous to a consumer.
    pub(crate) fn has_duplicate_labels(&self) -> bool {
        let mut seen = FxHashSet::default();
        !self.runs.iter().all(|run| seen.insert(&run.label))
    }

    /// Indices of the runs carrying `label`, in source order.
    pub(crate) fn matching_runs(&self, label: &ChapterLabel) -> Vec<usize> {
        self.runs
            .iter()
            .enumerate()
            .filter(|(_, run)| &run.label == label)
            .map(|(index, _)| index)
            .collect()
    }
}

/// The ordered runs of a token stream, using core's own chapter segmentation so
/// braid's grain cannot drift from the linter's.
pub(crate) fn chapter_runs(tokens: &[OwnedToken]) -> Vec<ChapterRun> {
    chapter_segments(tokens)
        .into_iter()
        .map(|segment| ChapterRun {
            label: segment_label(tokens, &segment.range, segment.is_front),
            range: segment.range,
        })
        .collect()
}

/// A chapter segment's label is the verbatim number token core segmented on;
/// front matter has none.
fn segment_label(tokens: &[OwnedToken], range: &Range<usize>, is_front: bool) -> ChapterLabel {
    if is_front {
        return ChapterLabel::FrontMatter;
    }
    // Core opens a segment only at a `\c` whose next token is a number, so the
    // label token is always there.
    let label = tokens[range.start + 1].source();
    ChapterLabel::Number(Box::from(label))
}

fn validate_token_ids(book: BookId, tokens: &[OwnedToken]) -> Result<(), IngestError> {
    let mut seen = FxHashSet::with_capacity_and_hasher(tokens.len(), rustc_hash::FxBuildHasher);
    for token in tokens {
        if !seen.insert(token.id()) {
            return Err(IngestError::DuplicateTokenId {
                book,
                id: token.id().clone(),
            });
        }
    }
    Ok(())
}
