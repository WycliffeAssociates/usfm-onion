//! Chapter-parallel parse: split the source at `\c` chapter boundaries, lex and
//! parse each slice through the order-preserving [`crate::par::map_ordered`] seam
//! (a Rayon pool natively, serial on wasm), then stitch the slices back into a
//! single token stream that is byte-for-byte identical to serial [`super::parse`].
//!
//! Why this is safe to split:
//! - A line-start `\c` followed by a chapter number is a clean lexeme boundary
//!   (the byte before it is a newline, which resets the lexer's attribute-run
//!   state), and a numbered `\c` fully re-establishes chapter/verse state, so a
//!   slice depends on nothing before it except the current book.
//! - The book is threaded in as a seed (chapter slices carry no `\id`), taken
//!   from the last `\id` preceding each slice so misplaced/multi-book input keeps
//!   its serial behavior.
//! - Byte spans are slice-local, so each is rebased by its slice offset; token
//!   ids restart per slice, so they are reassigned once over the merged stream.

use crate::lexer::lex;
use crate::token::{ParseAnalysis, ParseResult, Span, Token, TokenData};

use super::{assign_ids, parse_lexemes_seeded};

/// Byte length at or above which `super::parse` routes to the partitioned path.
/// Below it the fixed decomposition cost (per-chapter lex + allocation and the
/// serial stitch) outweighs the parallel gain, so parsing stays serial. Byte
/// length is a cheap, conservative proxy — heavy books clear it comfortably, and a
/// book left below it parses in well under a millisecond either way.
pub(crate) const PARALLEL_MIN_BYTES: usize = 256 * 1024;

/// Partition at `\c` and parse chapters through the ordered-map seam. Byte-identical
/// to serial `super::parse` regardless of thread count or target. `super::parse`
/// routes here for large native inputs; the corpus test below also calls it directly
/// to exercise the merge on small adversarial fixtures.
pub(crate) fn parse_partitioned(source: &str) -> ParseResult<'_> {
    let chunks = source_chunks(source);

    // Each slice does its own span rebasing (its byte offset is known) and flags
    // whether an `\id XXX` fired, so the serial stitch below is a bulk move plus a
    // single id pass — the only inherently sequential work.
    let results = crate::par::map_ordered(&chunks, |chunk| {
        let lexed = lex(chunk.text);
        let mut result = parse_lexemes_seeded(chunk.text, &lexed.tokens, chunk.seed);
        let offset = chunk.start as u32;
        for token in &mut result.tokens {
            rebase_token(token, offset);
        }
        let has_marker_id = has_adjacent_id_bookcode(&result.tokens);
        (result, has_marker_id)
    });

    // Reserve the whole stream up front: growing a multi-megabyte token Vec by
    // repeated reallocation costs more than the parallel parse it follows.
    let total: usize = results.iter().map(|(result, _)| result.tokens.len()).sum();
    let mut tokens = Vec::with_capacity(total);
    let mut last_marker_id: Option<&str> = None;
    let mut first_standalone: Option<&str> = None;

    for (result, has_marker_id) in results {
        // Reconstruct serial `analysis.book_code`: the last `\id XXX` wins, and a
        // standalone book code (an `\id` separated from its code by a newline)
        // only fills in when no `\id XXX` was ever seen.
        if let Some(code) = result.analysis.book_code {
            if has_marker_id {
                last_marker_id = Some(code);
            } else if first_standalone.is_none() {
                first_standalone = Some(code);
            }
        }
        tokens.extend(result.tokens);
    }

    assign_ids(&mut tokens);

    ParseResult {
        tokens,
        analysis: ParseAnalysis {
            book_code: last_marker_id.or(first_standalone),
        },
    }
}

/// One parse work unit: a source slice, its byte offset in the whole source, and
/// the book code in effect entering it (`None` for the front slice, which carries
/// its own `\id`).
struct Chunk<'a> {
    start: usize,
    text: &'a str,
    seed: Option<&'a str>,
}

/// Partition `source` into the pre-first-chapter front slice plus one slice per
/// chapter, seeding each chapter slice with the last preceding `\id` book code.
fn source_chunks(source: &str) -> Vec<Chunk<'_>> {
    let (offsets, seeds) = scan_boundaries(source);

    // Book in effect entering a slice = the last `\id` that begins before it.
    let seed_for = |start: usize| -> Option<&str> {
        seeds
            .iter()
            .rev()
            .find(|(offset, _)| *offset < start)
            .map(|(_, code)| *code)
    };

    let mut chunks = Vec::new();
    let first = offsets.first().copied().unwrap_or(source.len());
    if first > 0 {
        chunks.push(Chunk {
            start: 0,
            text: &source[0..first],
            seed: None,
        });
    }
    for (index, &start) in offsets.iter().enumerate() {
        let end = offsets.get(index + 1).copied().unwrap_or(source.len());
        chunks.push(Chunk {
            start,
            text: &source[start..end],
            seed: seed_for(start),
        });
    }
    chunks
}

/// One byte pass collecting both split points and book seeds:
///
/// - Chapter offsets — a line-start `\c` whose name is exactly `c` (terminated by
///   horizontal whitespace) followed, on the same line, by a digit. This matches
///   the walker's "`c` marker immediately followed by a `Number`" boundary: an
///   indented `\c`, a numberless `\c`, a number on the next line, or an extended
///   name (`\cl`, `\c1`) never opens a chapter here. Requiring a line start is
///   conservative — it only merges slices that could have split, never mis-splits
///   — so every returned offset is a genuine boundary.
/// - Book seeds — `(\id offset, three-char code)`. Mirrors the lexer: the marker
///   name must be exactly `id`, its book-code payload survives intervening
///   whitespace and newlines, and the code is the first three bytes of the
///   following run only when that whole run is ASCII (>= 3 chars) with an
///   alphanumeric prefix (the `consume_book_code` rule). A run opening with a
///   marker/pipe/slash carries no code.
fn scan_boundaries(source: &str) -> (Vec<usize>, Vec<(usize, &str)>) {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut offsets = Vec::new();
    let mut seeds = Vec::new();
    let mut i = 0;
    while i < len {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }
        match (bytes.get(i + 1), bytes.get(i + 2)) {
            (Some(&b'c'), Some(b' ' | b'\t')) if i == 0 || bytes[i - 1] == b'\n' => {
                let mut j = i + 2;
                while j < len && matches!(bytes[j], b' ' | b'\t') {
                    j += 1;
                }
                if bytes.get(j).is_some_and(u8::is_ascii_digit) {
                    offsets.push(i);
                }
            }
            (Some(&b'i'), Some(&b'd')) => {
                let after = i + 3;
                let extends_name = matches!(bytes.get(after), Some(&c) if c.is_ascii_lowercase() || c.is_ascii_digit());
                let numbered_range = bytes.get(after) == Some(&b'-')
                    && bytes.get(after + 1).is_some_and(u8::is_ascii_digit);
                if !extends_name && !numbered_range {
                    let mut j = after;
                    while j < len && matches!(bytes[j], b' ' | b'\t' | b'\r' | b'\n') {
                        j += 1;
                    }
                    if j < len && !matches!(bytes[j], b'\\' | b'|' | b'/') {
                        let mut k = j;
                        while k < len && !matches!(bytes[k], b'\\' | b'\r' | b'\n' | b'|' | b'/') {
                            k += 1;
                        }
                        let run = &bytes[j..k];
                        if run.len() >= 3
                            && run.is_ascii()
                            && run[..3].iter().all(u8::is_ascii_alphanumeric)
                        {
                            seeds.push((i, &source[j..j + 3]));
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    (offsets, seeds)
}

/// Shift every byte span in a slice-local token to whole-source coordinates by
/// its slice offset: the token span plus the spans of any attached attribute
/// list. Borrowed `&str` values (`token.source`, attribute text) need no change —
/// a slice is a subslice of the source, so they already point at the right bytes
/// with the source's lifetime; only the numeric spans move.
fn rebase_token(token: &mut Token<'_>, offset: u32) {
    let shift = |span: Span| Span::new(span.start + offset, span.end + offset);
    token.span = shift(token.span);
    match &mut token.data {
        TokenData::Marker {
            attributes,
            attribute_source,
            ..
        }
        | TokenData::Milestone {
            attributes,
            attribute_source,
            ..
        } => {
            for attribute in attributes.iter_mut() {
                attribute.span = shift(attribute.span);
            }
            if let Some((span, _)) = attribute_source {
                *span = shift(*span);
            }
        }
        _ => {}
    }
}

/// Whether an `\id XXX` (marker-id path) fired in this slice: a `Marker { "id" }`
/// token immediately followed by a `BookCode` token. A standalone book code (an
/// `\id` separated from its code by a newline) has a `Newline` token between them.
fn has_adjacent_id_bookcode(tokens: &[Token<'_>]) -> bool {
    tokens.windows(2).any(|pair| {
        matches!(pair[0].data, TokenData::Marker { name: "id", .. })
            && matches!(pair[1].data, TokenData::BookCode { .. })
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::super::{parse, parse_lexemes};
    use super::parse_partitioned;
    use crate::lexer::lex;
    use crate::token::ParseResult;

    /// Guaranteed-serial baseline (never routes to the partitioned path), so the
    /// comparisons below hold even once `parse` itself parallelizes large inputs.
    fn serial(source: &str) -> ParseResult<'_> {
        let lexed = lex(source);
        parse_lexemes(source, &lexed.tokens)
    }

    #[test]
    fn matches_serial_over_test_data() {
        assert_identical_over("testData");
    }

    #[test]
    fn matches_serial_over_example_corpora() {
        assert_identical_over("example-corpora");
    }

    #[test]
    fn targeted_boundary_shapes_match_serial() {
        // Shapes the corpora may under-exercise: numberless `\c`, indented `\c`, an
        // extended-name near-miss (`\cl`, `\c1`), a chapter number on the next line,
        // a `\c` at byte 0, multiple `\id`, and a misplaced mid-book `\id`.
        let cases = [
            "",
            "\\c 1\n\\v 1 verse-at-byte-zero\n",
            "\\id GEN\n\\c 1\n\\v 1 a\n\\c 2\n\\v 1 b\n\\c 3\n\\v 1 c\n",
            "\\id GEN\n\\c\n\\p no chapter number\n\\c 2\n\\v 1 real\n",
            "\\id GEN\n\\cl Psalm\n\\c 1\n\\v 1 a\n",
            "\\id GEN\n\\c1 glued-name-not-a-chapter\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c \n5\n\\p number on next line\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n  \\c 2\n\\v 1 indented-c\n",
            "\\id GEN\n\\id MAT\n\\c 1\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\id MAT\\p \\v 2 rest\n",
            "\\id GEN\n\\c 1\n\\id MAT\n\\c 2\n\\v 1 after-mid-book-id\n",
            "no id at all\n\\c 1\n\\v 1 a\n",
        ];
        for (index, source) in cases.iter().enumerate() {
            assert_identical(source, &format!("targeted case #{index}"));
        }
    }

    fn assert_identical_over(dir: &str) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        let mut paths = Vec::new();
        collect_usfm(&root, &mut paths);
        paths.sort();
        assert!(!paths.is_empty(), "expected {dir}/**/*.usfm fixtures");
        for path in paths {
            let source = std::fs::read_to_string(&path).expect("read fixture");
            assert_identical(&source, &path.to_string_lossy());
        }
    }

    fn assert_identical(source: &str, label: &str) {
        let want = serial(source);
        // Force the partitioned merge even on the small adversarial fixtures.
        assert_matches(
            &parse_partitioned(source),
            &want,
            &format!("{label} (partitioned)"),
        );
        // The transparent entry point (serial for small, partitioned for large) must
        // also match the serial baseline.
        assert_matches(&parse(source), &want, &format!("{label} (parse)"));
    }

    fn assert_matches(actual: &ParseResult<'_>, want: &ParseResult<'_>, label: &str) {
        assert_eq!(
            actual.analysis, want.analysis,
            "analysis differs for {label}"
        );
        assert_eq!(
            actual.tokens.len(),
            want.tokens.len(),
            "token count differs for {label}"
        );
        for (index, (got, exp)) in actual.tokens.iter().zip(&want.tokens).enumerate() {
            assert_eq!(got, exp, "token #{index} differs for {label}");
        }
    }

    fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_usfm(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("usfm") {
                out.push(path);
            }
        }
    }
}
