//! Shared whitespace vocabulary used by lexer, parser, format, and lint.
//!
//! USFM 3.1 distinguishes several whitespace classes by where they're allowed
//! and how many characters are required (see `whitespace.md` in the repo
//! root for the spec definitions of `hs`/`HS`/`Hs`/`nl`/`NL`/`ws`/`WS`/`Ws`/
//! `anyws`/`allws`). Rather than scatter regex literals and ad-hoc `matches!`
//! checks across the codebase, this module centralizes:
//!
//! - Char-level predicates the lexer / format / lint all use to ask
//!   "is this character horizontal whitespace?" / "is this a newline?" /
//!   "is this any kind of reducible whitespace?"
//! - The structural-whitespace requirement enum used by per-marker rules
//!   (e.g. `\c` requires at least one horizontal whitespace after the
//!   marker name; paragraph markers require a newline-or-any-whitespace
//!   before the marker).
//! - Format profile preferences (single newline / single space) and the
//!   coarse category that profiles use to decide whether a marker should
//!   start its own line.
//!
//! Names here are intentionally verbose. The reader should not need to
//! look up `HS` vs `Hs` vs `WS` to understand a function call — the names
//! say what they mean.

use serde::{Deserialize, Serialize};

/// Specifies the structural-whitespace requirement at a particular position
/// relative to a marker (e.g. immediately before the open marker, immediately
/// after the marker name, before/after the closing `*` form).
///
/// Spec equivalents are noted in each variant's doc comment. See
/// `whitespace.md` in the repo root for the full spec definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructuralWhitespaceRequirement {
    /// At least one horizontal whitespace character (space or tab) is required.
    /// Spec: `HS`.
    AtLeastOneHorizontalWhitespace,
    /// Zero or more horizontal whitespace characters are allowed.
    /// Spec: `Hs`.
    OptionalHorizontalWhitespace,
    /// At least one whitespace character (horizontal whitespace or a newline)
    /// is required. Spec: `WS` (or `ws` when newline is permitted).
    AtLeastOneWhitespace,
    /// Zero or more whitespace characters are allowed.
    /// Spec: `Ws`.
    OptionalWhitespace,
    /// Exactly one newline (CR, LF, or CRLF) is required.
    /// Spec: `nl`.
    SingleNewline,
    /// At least one newline is required.
    /// Spec: `NL`.
    AtLeastOneNewline,
    /// Either a newline immediately precedes the marker, or any whitespace
    /// run precedes the marker. Used for paragraph-marker openings whose
    /// spec rule is `\n\\` or `${Ws}\\`.
    NewlineOrAnyWhitespaceBeforeMarker,
    /// "Tag end": whitespace OR end-of-input OR start-of-attributes. Used
    /// after open-marker names to delimit the marker from following text.
    /// Spec: `TAGEND`.
    TagEndDelimiter,
    /// No structural whitespace is required at this position.
    NotRequired,
}

/// Format-time preference for resolving whitespace at an ambiguous position.
/// Used by [`crate::marker_defs::MarkerWhitespace`] to tell the formatter
/// what to insert when normalizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatWhitespacePreference {
    /// Replace the whitespace at this position with a single newline.
    PreferSingleNewline,
    /// Replace the whitespace at this position with a single space.
    PreferSingleSpace,
    /// Remove all whitespace at this position.
    PreferRemoveAllWhitespace,
}

/// Coarse classification used by [`FormatTimings`](crate::format::FormatTimings)
/// to decide layout for a marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhitespaceFormatCategory {
    /// Block-starting markers: paragraph, chapter, verse, sidebar, table row.
    /// In code-editor profile these get their own line.
    Block,
    /// Inline markers (character markers and note containers like `\f`/`\x`).
    /// Never get their own line in either profile — they read most naturally
    /// alongside the verse text they sit in.
    Inline,
}

/// Returns true if `c` is a horizontal whitespace character — either an
/// ASCII space (`U+0020`) or a horizontal tab (`U+0009`). Matches USFM
/// spec class `hs`.
#[inline]
pub fn is_horizontal_whitespace_char(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

/// Returns true if `c` is part of a newline sequence (`\n`, `\r`, or the
/// CRLF pair when seen one char at a time). Matches USFM spec class `nl`
/// at the single-character granularity.
#[inline]
pub fn is_newline_char(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

/// Returns true if `c` is any reducible whitespace character — horizontal
/// whitespace or a newline character. Matches USFM spec class `anyws`.
#[inline]
pub fn is_any_whitespace_char(c: char) -> bool {
    is_horizontal_whitespace_char(c) || is_newline_char(c)
}

/// Returns true if `c` is one of the sentence-ending punctuation characters
/// that suppresses the intra-content whitespace collapse rule. The format
/// pipeline preserves a multi-space run when the character immediately
/// preceding it is one of these — protects the older typography convention
/// of two spaces after a period/question/exclamation/colon/semicolon
/// without needing per-document detection.
#[inline]
pub fn is_sentence_ending_punctuation_char(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | ':' | ';')
}

/// Returns true if `s` starts with at least one horizontal whitespace
/// character. Convenience wrapper around [`is_horizontal_whitespace_char`].
#[inline]
pub fn starts_with_at_least_one_horizontal_whitespace(s: &str) -> bool {
    s.chars().next().is_some_and(is_horizontal_whitespace_char)
}

/// Returns true if `s` starts with at least one newline character.
#[inline]
pub fn starts_with_at_least_one_newline(s: &str) -> bool {
    s.chars().next().is_some_and(is_newline_char)
}

/// Returns true if `s` ends with at least one horizontal whitespace character.
#[inline]
pub fn ends_with_at_least_one_horizontal_whitespace(s: &str) -> bool {
    s.chars()
        .next_back()
        .is_some_and(is_horizontal_whitespace_char)
}

/// Returns true if `s` ends with at least one newline character.
#[inline]
pub fn ends_with_at_least_one_newline(s: &str) -> bool {
    s.chars().next_back().is_some_and(is_newline_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_whitespace_recognizes_space_and_tab_only() {
        assert!(is_horizontal_whitespace_char(' '));
        assert!(is_horizontal_whitespace_char('\t'));
        assert!(!is_horizontal_whitespace_char('\n'));
        assert!(!is_horizontal_whitespace_char('\r'));
        assert!(!is_horizontal_whitespace_char('a'));
    }

    #[test]
    fn newline_recognizes_lf_and_cr_only() {
        assert!(is_newline_char('\n'));
        assert!(is_newline_char('\r'));
        assert!(!is_newline_char(' '));
        assert!(!is_newline_char('\t'));
    }

    #[test]
    fn any_whitespace_covers_horizontal_and_newline() {
        for c in [' ', '\t', '\n', '\r'] {
            assert!(is_any_whitespace_char(c), "expected whitespace: {c:?}");
        }
        assert!(!is_any_whitespace_char('a'));
        assert!(!is_any_whitespace_char('.'));
    }

    #[test]
    fn sentence_ending_punctuation_covers_protected_chars() {
        for c in ['.', '!', '?', ':', ';'] {
            assert!(
                is_sentence_ending_punctuation_char(c),
                "expected protected: {c:?}"
            );
        }
        assert!(!is_sentence_ending_punctuation_char(','));
        assert!(!is_sentence_ending_punctuation_char('-'));
    }
}
