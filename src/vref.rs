use std::collections::BTreeMap;

use serde::Serialize;

use crate::lint_impl::LintableToken;
use crate::marker_defs::{StructuralScopeKind, marker_paragraph_supports_verse};
use crate::parse::parse;
use crate::token::{Sid, Span, Token};
use crate::walker::{ScopeFrame, Visitor, WalkContext, walk, walk_tokens};

pub type VrefMap = BTreeMap<String, String>;

/// Options for the lossy `to_vref` projection.
///
/// Separator insertion at structural breaks (see `push_collected_text`) is
/// unconditional — it is what makes the projection correct, not a mode. If a
/// consumer ever needs raw joins, expose it here as a new optional field
/// defaulting to today's behavior; that is purely additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VrefOptions {
    pub trim: bool,
}

pub fn usfm_to_vref_map(source: &str) -> VrefMap {
    usfm_to_vref_map_with_options(source, VrefOptions::default())
}

pub fn usfm_to_vref_map_with_options(source: &str, options: VrefOptions) -> VrefMap {
    let parsed = parse(source);
    tokens_to_vref_map_with_options(&parsed.tokens, options)
}

pub fn tokens_to_vref_map(tokens: &[Token<'_>]) -> VrefMap {
    tokens_to_vref_map_with_options(tokens, VrefOptions::default())
}

pub fn tokens_to_vref_map_with_options(tokens: &[Token<'_>], options: VrefOptions) -> VrefMap {
    let mut visitor = VrefVisitor::new(options);
    walk_tokens(tokens, &mut visitor);
    visitor.finish()
}

pub fn vref_map_to_json_string(map: &VrefMap) -> String {
    serde_json::to_string_pretty(map).expect("vref map should serialize")
}

/// UTF-16 code-unit offsets into a [`VerseProjection::text`]. Kept a
/// distinct type from [`Span`] (byte offsets into the original source) so
/// the unit is unmistakable at the call site and on the wire: JS/DOM
/// consumers index the projected text in UTF-16, native consumers index
/// the source in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Utf16Span {
    pub start: u32,
    pub end: u32,
}

/// One in-scope text token's contribution to a verse projection. Carries
/// both resolution anchors so a `text` range maps to either a raw-source
/// byte offset (`source_span`) or a node-based consumer's token
/// (`token_id`, which the editor mirrors as a DOM `data-id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Segment {
    pub token_id: String,
    /// Byte range into the original source — the raw-buffer anchor.
    pub source_span: Span,
    /// UTF-16 code-unit range into [`VerseProjection::text`].
    pub text_span: Utf16Span,
}

/// Lossless plain-text projection of one verse, plus the segment map back
/// to source / token coordinates. `text` is the verbatim concatenation of
/// the verse's in-scope text-token sources — no whitespace collapse, no
/// trim, markup excluded — so it matches a raw editor buffer byte-for-byte
/// and a content range resolves to exact glyphs on either anchor.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct VerseProjection {
    pub text: String,
    pub segments: Vec<Segment>,
}

/// `sid` → its lossless verse projection. Same key set as [`VrefMap`]; the
/// difference is losslessness plus the segment map.
pub type VrefIndex = BTreeMap<String, VerseProjection>;

pub fn usfm_to_vref_index(source: &str) -> VrefIndex {
    let parsed = parse(source);
    tokens_to_vref_index(&parsed.tokens)
}

/// Generic over the token representation so the same projection runs over
/// freshly-parsed native `Token`s (the parse path) **and** the editor's
/// rehydrated tokens (the live lint path) — no reparse, segments keyed by
/// each token's own id. Driven by the generic [`walk`], which resolves
/// chapter/verse opens from the slice exactly as `walk_tokens` does.
pub fn tokens_to_vref_index<T: LintableToken>(tokens: &[T]) -> VrefIndex {
    let mut visitor = IndexedVrefVisitor::default();
    walk(tokens, &mut visitor);
    visitor.finish()
}

#[derive(Debug, Default)]
struct VrefVisitor {
    options: VrefOptions,
    map: VrefMap,
    current_ref: Option<String>,
    current_text: String,
    pending_separator: bool,
    // Persists across paragraph boundaries: once any Block has been
    // seen, this reflects the latest Block's supports-verse status, even
    // after that Block closes. Matches the pre-walker behaviour.
    current_block_supports_verse: Option<bool>,
}

impl VrefVisitor {
    fn new(options: VrefOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    fn finish(mut self) -> VrefMap {
        self.flush_current_verse();
        self.map
    }

    fn flush_current_verse(&mut self) {
        let Some(reference) = self.current_ref.take() else {
            self.current_text.clear();
            return;
        };
        let output = if self.options.trim {
            self.current_text.trim()
        } else {
            self.current_text.as_str()
        };
        if !output.trim().is_empty() {
            self.map.insert(reference, output.to_string());
        }
        self.current_text.clear();
        self.pending_separator = false;
    }

    fn can_collect_text(&self, ctx: &WalkContext<'_, '_>) -> bool {
        self.current_ref.is_some()
            && !ctx.in_note()
            && self.current_block_supports_verse.unwrap_or(true)
    }

    fn push_collected_text(&mut self, fragment: &str) {
        if self.pending_separator
            && !self.current_text.is_empty()
            && !self
                .current_text
                .chars()
                .last()
                .is_some_and(char::is_whitespace)
            && !fragment.chars().next().is_some_and(char::is_whitespace)
        {
            self.current_text.push(' ');
        }
        self.pending_separator = false;
        self.current_text.push_str(fragment);
    }
}

impl<'tokens, 'src> Visitor<'tokens, Token<'src>> for VrefVisitor {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if frame.scope_kind == StructuralScopeKind::Block {
            self.current_block_supports_verse = Some(marker_paragraph_supports_verse(frame.marker));
            self.pending_separator = true;
        }
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        self.flush_current_verse();
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        self.flush_current_verse();
        if let Some(reference) = verse_ref(token.sid, token.source) {
            self.current_ref = Some(reference);
        }
    }

    fn on_text(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if self.can_collect_text(ctx) {
            self.push_collected_text(token.source);
        }
    }

    fn on_newline(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        self.pending_separator = true;
    }
}

/// Build the `BOOK CHAPTER:VERSE` sid for a verse number token.
///
/// `lexeme` is the verbatim verse-number form from source (e.g. `"1"`,
/// `"1-2"`, `"1,3"`, `"1-3,5"`). For a bridge or sequence, the lexeme is
/// preserved as-is in the sid — matching the USX/USJ normative form
/// (`"GEN 1:1-2"`) and the convention used by usfm-grammar, usfmtc, and
/// usfm3. Consumers that need per-verse lookup against a bridged source
/// should split the lexeme themselves.
fn verse_ref(sid: Option<Sid>, lexeme: &str) -> Option<String> {
    let sid = sid?;
    let chapter = sid.chapter;
    if chapter == 0 {
        return None;
    }

    let verse_part = lexeme.trim();
    if verse_part.is_empty() {
        return None;
    }

    Some(format!("{} {}:{}", sid.book, chapter, verse_part))
}

/// Build the verse-reference key from a generic token's formatted `sid()`
/// and its verse-number lexeme (`text()`). Mirrors [`verse_ref`] but works
/// off the `LintableToken` surface so the index runs over any token
/// representation. The verse part comes from the lexeme so bridges and
/// sequences (`1-2`, `1,3`) survive verbatim; the book/chapter prefix comes
/// from `sid`. `sid` is `"BOOK CHAPTER"` (verse 0) or `"BOOK CHAPTER:VERSE"`.
fn verse_ref_str(sid: Option<String>, lexeme: &str) -> Option<String> {
    let sid = sid?;
    let verse_part = lexeme.trim();
    if verse_part.is_empty() {
        return None;
    }
    let prefix = sid.split(':').next().unwrap_or(sid.as_str());
    // Mirror `verse_ref`'s chapter==0 guard (the chapter is the last
    // whitespace-separated component of the prefix).
    if prefix.rsplit(' ').next() == Some("0") {
        return None;
    }
    Some(format!("{prefix}:{verse_part}"))
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// Lossless sibling of [`VrefVisitor`]. Same scope gating (in a verse, not
/// in a note, in a verse-supporting paragraph), but appends each in-scope
/// text token's source verbatim — no boundary collapse, no trim — and
/// records a [`Segment`] mapping that run back to the token's byte span
/// and id. The raw text is what content analyzers operate on; the segments
/// resolve their findings back to source or DOM coordinates.
#[derive(Debug, Default)]
struct IndexedVrefVisitor {
    index: VrefIndex,
    current_ref: Option<String>,
    current: VerseProjection,
    current_utf16_len: u32,
    // Mirrors `VrefVisitor`: persists across paragraph boundaries.
    current_block_supports_verse: Option<bool>,
}

impl IndexedVrefVisitor {
    fn finish(mut self) -> VrefIndex {
        self.flush_current_verse();
        self.index
    }

    fn flush_current_verse(&mut self) {
        if let Some(reference) = self.current_ref.take() {
            // Match `VrefVisitor`: a whitespace-only verse produces no
            // entry, so the index key set stays identical to `to_vref`.
            if !self.current.text.trim().is_empty() {
                self.index
                    .insert(reference, std::mem::take(&mut self.current));
            }
        }
        self.current = VerseProjection::default();
        self.current_utf16_len = 0;
    }

    fn can_collect_text(&self, ctx: &WalkContext<'_, '_>) -> bool {
        self.current_ref.is_some()
            && !ctx.in_note()
            && self.current_block_supports_verse.unwrap_or(true)
    }

    fn push_token<T: LintableToken>(&mut self, token: &T) {
        let text = token.text();
        let len = utf16_len(text);
        self.current.segments.push(Segment {
            token_id: token.id().unwrap_or_default(),
            source_span: token.span().unwrap_or(Span::new(0, 0)),
            text_span: Utf16Span {
                start: self.current_utf16_len,
                end: self.current_utf16_len + len,
            },
        });
        self.current.text.push_str(text);
        self.current_utf16_len += len;
    }
}

impl<'tokens, T: LintableToken> Visitor<'tokens, T> for IndexedVrefVisitor {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        if frame.scope_kind == StructuralScopeKind::Block {
            self.current_block_supports_verse = Some(marker_paragraph_supports_verse(frame.marker));
        }
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        self.flush_current_verse();
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        _token_index: usize,
    ) {
        self.flush_current_verse();
        if let Some(reference) = verse_ref_str(token.sid(), token.text()) {
            self.current_ref = Some(reference);
        }
    }

    fn on_text(&mut self, ctx: &WalkContext<'tokens, '_>, token: &'tokens T, _token_index: usize) {
        if self.can_collect_text(ctx) {
            self.push_token(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Utf16Span, VrefOptions, tokens_to_vref_map, usfm_to_vref_index, usfm_to_vref_map,
        usfm_to_vref_map_with_options, vref_map_to_json_string,
    };
    use crate::parse::parse;

    /// Representative sources exercising the scope rules the index shares
    /// with `to_vref`: plain verses, stripped notes, skipped headings,
    /// verse-spanning paragraphs, and verse bridges.
    const VREF_INDEX_FIXTURES: &[&str] = &[
        "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n",
        "\\id GEN\n\\c 1\n\\p\n\\v 1 Text \\f + \\fr 1:1 \\ft note text \\f* rest.",
        "\\id GEN\n\\c 1\n\\s1 The Creation\n\\p\n\\v 1 In the beginning.",
        "\\id GEN\n\\c 1\n\\p\n\\v 1 First part.\n\\q1 Second part.",
        "\\id GEN\n\\c 1\n\\p\n\\v 1-2 The first two verses share text.\n\\v 3 Third.",
        // Poetry with trailing spaces before line breaks + inline char
        // marker: exercises delimiter/join whitespace adjacency.
        "\\id ISA\n\\c 9\n\\p\n\\v 2 The people who walked in darkness \n\\q2 have seen a \\nd great\\nd* light;\n\\q1 those who lived in the land, \n\\q2 the light has shone.\n",
    ];

    /// Read a UTF-16 sub-range back out of a projected verse text. Segment
    /// boundaries always fall on token boundaries (whole codepoints), so
    /// the slice never splits a surrogate pair.
    fn utf16_slice(text: &str, span: Utf16Span) -> String {
        let units: Vec<u16> = text.encode_utf16().collect();
        String::from_utf16(&units[span.start as usize..span.end as usize])
            .expect("segment boundaries fall on whole codepoints")
    }

    // Test 1 — scope parity: the index collects exactly the verses `to_vref`
    // does (notes/headings excluded, whitespace-only verses dropped).
    #[test]
    fn vref_index_key_set_matches_to_vref() {
        for &src in VREF_INDEX_FIXTURES {
            let index = usfm_to_vref_index(src);
            let map = usfm_to_vref_map(src);
            let index_keys: std::collections::BTreeSet<_> = index.keys().cloned().collect();
            let map_keys: std::collections::BTreeSet<_> = map.keys().cloned().collect();
            assert_eq!(index_keys, map_keys, "key sets diverge for: {src:?}");
        }
    }

    // Test 2 — text purity: the index is a content projection, not a
    // formatting pass. Structural separators for `to_vref` are inserted
    // by that visitor; the index stores only source-backed content bytes.
    #[test]
    fn vref_index_fragments_are_source_backed_content() {
        for &src in VREF_INDEX_FIXTURES {
            let index = usfm_to_vref_index(src);
            for (sid, proj) in &index {
                let mut rebuilt = String::new();
                for seg in &proj.segments {
                    rebuilt.push_str(&utf16_slice(&proj.text, seg.text_span));
                }
                assert_eq!(
                    rebuilt, proj.text,
                    "index text must be exactly the concatenated content segments for {sid} in {src:?}"
                );
            }
        }
    }

    // Test 3 — segment integrity: segments partition the text contiguously
    // in UTF-16, and each segment's byte/UTF-16 widths agree with its
    // fragment. This is the substrate the resolver binary-searches.
    #[test]
    fn vref_index_segments_partition_text_contiguously() {
        for &src in VREF_INDEX_FIXTURES {
            let index = usfm_to_vref_index(src);
            for (sid, proj) in &index {
                let mut cursor = 0u32;
                for seg in &proj.segments {
                    assert_eq!(
                        seg.text_span.start, cursor,
                        "non-contiguous text span in {sid}"
                    );
                    let frag = utf16_slice(&proj.text, seg.text_span);
                    assert_eq!(
                        seg.text_span.end - seg.text_span.start,
                        frag.encode_utf16().count() as u32,
                        "text_span width != fragment UTF-16 length in {sid}"
                    );
                    assert_eq!(
                        seg.source_span.end - seg.source_span.start,
                        frag.len() as u32,
                        "source_span byte width != fragment UTF-8 length in {sid}"
                    );
                    cursor = seg.text_span.end;
                }
                assert_eq!(
                    cursor,
                    proj.text.encode_utf16().count() as u32,
                    "segments do not cover full text in {sid}"
                );
            }
        }
    }

    // Test 4a — source round-trip (the universal anchor): each segment's
    // `source_span` reads back exactly the bytes the segment contributed to
    // the projected text. This is what a raw-buffer consumer relies on.
    #[test]
    fn vref_index_segment_source_spans_map_to_original_bytes() {
        for &src in VREF_INDEX_FIXTURES {
            let index = usfm_to_vref_index(src);
            for (sid, proj) in &index {
                for seg in &proj.segments {
                    let from_source =
                        &src[seg.source_span.start as usize..seg.source_span.end as usize];
                    let from_text = utf16_slice(&proj.text, seg.text_span);
                    assert_eq!(
                        from_source, from_text,
                        "source_span bytes != projected text for a segment in {sid} ({src:?})"
                    );
                }
            }
        }
    }

    // Marker delimiters are markup, so they do not appear in the lossless
    // content projection. Real content whitespace still maps back to the
    // exact source bytes through one segment.
    #[test]
    fn vref_index_marker_joins_are_content_pure() {
        let strictly_interior = |proj: &super::VerseProjection, a: u32, b: u32| {
            proj.segments
                .iter()
                .any(|s| s.text_span.start < a && b < s.text_span.end)
        };

        // Poetry join: trailing content space stays, `\q2` delimiter does not.
        let src = "\\id ISA\n\\c 9\n\\p\n\\v 2 The people walked in  darkness \n\\q2 have seen a great light;\n";
        let proj = usfm_to_vref_index(src)
            .remove("ISA 9:2")
            .expect("verse present");
        assert!(proj.text.contains("darkness have"));
        assert!(!proj.text.contains("darkness  have"));

        // Genuine in-content double space IS strictly interior — flaggable.
        let real = proj.text.find("in  darkness").expect("content run present") as u32 + 2;
        assert!(
            strictly_interior(&proj, real, real + 2),
            "content run must be strictly interior to one segment"
        );

        // Inline char marker: delimiter before `Lord` is markup and drops out.
        let src2 = "\\id GEN\n\\c 1\n\\p\n\\v 1 word \\nd Lord\\nd* more text.\n";
        let proj2 = usfm_to_vref_index(src2)
            .remove("GEN 1:1")
            .expect("verse present");
        assert!(proj2.text.contains("word Lord"));
        assert!(!proj2.text.contains("word  Lord"));
    }

    // Test 4b — the actual use case: a content finding (a double space)
    // resolves from a projected-text range to the exact source bytes. The
    // ASCII fixture makes UTF-16 and byte offsets coincide, so the intra-
    // segment math is identity here.
    #[test]
    fn vref_index_resolves_a_double_space_range_to_source() {
        let src = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the  beginning.\n";
        let index = usfm_to_vref_index(src);
        let proj = index.get("GEN 1:1").expect("verse present");
        // Lossless: the double space survives verbatim in the projection.
        let ws = proj
            .text
            .find("  ")
            .expect("double space preserved in projection") as u32;
        let (start, end) = (ws, ws + 2);
        let seg = proj
            .segments
            .iter()
            .find(|s| s.text_span.start <= start && end <= s.text_span.end)
            .expect("range lands within one segment");
        let intra = start - seg.text_span.start; // ASCII: UTF-16 == byte
        let src_start = (seg.source_span.start + intra) as usize;
        assert_eq!(&src[src_start..src_start + 2], "  ");
    }

    #[test]
    fn basic_vref_extracts_plain_verse_text() {
        let map = usfm_to_vref_map(
            "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n",
        );

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning God created the heavens and the earth.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("The earth was without form and void.")
        );
    }

    #[test]
    fn footnotes_are_stripped() {
        let map = usfm_to_vref_map(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 Text \\f + \\fr 1:1 \\ft note text \\f* rest.",
        );
        let verse = map.get("GEN 1:1").map(String::as_str).unwrap_or("");

        assert!(verse.contains("Text"));
        assert!(verse.contains("rest."));
        assert!(!verse.contains("note text"));
    }

    #[test]
    fn section_headings_are_skipped() {
        let map =
            usfm_to_vref_map("\\id GEN\n\\c 1\n\\s1 The Creation\n\\p\n\\v 1 In the beginning.");

        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
    }

    #[test]
    fn verse_spanning_paragraphs_is_concatenated() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 First part.\n\\q1 Second part.");

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("First part. Second part.")
        );
    }

    #[test]
    fn structural_break_inserts_separator_without_leaking_delimiters() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\nGod created.");
        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning God created.")
        );

        let malformed = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 word\\nd Lord\\nd* more.");
        assert_eq!(
            malformed.get("GEN 1:1").map(String::as_str),
            Some("wordLord more.")
        );
    }

    #[test]
    fn to_vref_preserves_edge_content_whitespace_unless_trim_is_requested() {
        let src = "\\id GEN\n\\c 1\n\\p\n\\v 1  padded \n";
        let map = usfm_to_vref_map(src);
        assert_eq!(map.get("GEN 1:1").map(String::as_str), Some(" padded "));

        let trimmed = usfm_to_vref_map_with_options(src, VrefOptions { trim: true });
        assert_eq!(trimmed.get("GEN 1:1").map(String::as_str), Some("padded"));
    }

    #[test]
    fn root_level_verses_are_collected() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 In the beginning.\n\\v 2 And God said.");
        let map = tokens_to_vref_map(&parsed.tokens);

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("And God said.")
        );
    }

    #[test]
    fn json_output_contains_refs_and_text() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
        let json = vref_map_to_json_string(&map);

        assert!(json.contains("\"GEN 1:1\""));
        assert!(json.contains("\"In the beginning.\""));
    }

    #[test]
    fn verse_bridge_produces_compound_sid() {
        // Matches USX/USJ normative form: \v 1-2 → sid="GEN 1:1-2".
        // usfm-grammar, usfmtc, and usfm3 all use the same compound key.
        let map = usfm_to_vref_map(
            "\\id GEN\n\\c 1\n\\p\n\\v 1-2 The first two verses share text.\n\\v 3 Third.",
        );
        assert_eq!(
            map.get("GEN 1:1-2").map(String::as_str),
            Some("The first two verses share text."),
        );
        assert_eq!(map.get("GEN 1:3").map(String::as_str), Some("Third."));
        assert!(
            !map.contains_key("GEN 1:1"),
            "bridge should not produce a separate GEN 1:1 entry; got keys: {:?}",
            map.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn verse_sequence_lexeme_is_preserved() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1,3 Combined.\n");
        assert_eq!(map.get("GEN 1:1,3").map(String::as_str), Some("Combined."),);
    }

    #[test]
    fn unclosed_footnote_does_not_swallow_subsequent_verses() {
        // Reproducer from the gitea-dashboard bug report
        // (2026-05-08-usfm-onion-unclosed-note-recovery): a single \f with
        // no \f* used to drop every subsequent verse and chapter from
        // to_vref() output.
        let src = "\\id GEN Sample\n\
                   \\c 1\n\
                   \\p\n\
                   \\v 1 First verse with an unclosed footnote.\\f + \\ft Note text never terminated.\n\
                   \\v 2 Second verse — should still appear.\n\
                   \\c 2\n\
                   \\p\n\
                   \\v 1 Chapter 2 should also still appear.\n";
        let map = usfm_to_vref_map(src);
        assert!(
            map.contains_key("GEN 1:1"),
            "v1 of ch1 missing: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(
            map.contains_key("GEN 1:2"),
            "v2 of ch1 dropped by unclosed-note bug: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(
            map.contains_key("GEN 2:1"),
            "ch2 dropped by unclosed-note bug: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("Second verse — should still appear."),
            "v2 text should not be polluted by note content",
        );
    }

    #[test]
    fn unclosed_footnote_with_nested_chars_recovers_at_next_paragraph() {
        // Mirrors the en_ulb Isaiah pattern: outer \f never closed, but
        // inner \fqa pairs are well-formed. The next \q1 paragraph marker
        // should close the outer note.
        let src = "\\id ISA\n\
                   \\c 33\n\
                   \\q2 Lebanon is ashamed and withers away;\\f + \\ft The word \\fqa mourns \\fqa* can be also be read as \\fqa dries up\\fqa*.\n\
                   \\q1 Sharon is like a desert plain;\n\
                   \\c 34\n\
                   \\p\n\
                   \\v 1 Chapter 34 should appear.\n";
        let map = usfm_to_vref_map(src);
        assert!(
            map.contains_key("ISA 34:1"),
            "ch34 dropped: {:?}",
            map.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn ulb_isaiah_full_corpus_all_chapters_present() {
        // Real-world reproducer from the bug report: en_ulb/23-ISA.usfm has
        // an unclosed \f at chapter 33 verse 9. Pre-fix, vref output stopped
        // at chapter 33 and dropped chapters 34-66 silently. Assert all
        // 66 chapters round-trip through to_vref.
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = repo_root.join("example-corpora/en_ulb/23-ISA.usfm");
        if !path.exists() {
            // Allow the test to no-op when the corpus isn't checked out.
            return;
        }
        let source = std::fs::read_to_string(&path).expect("isaiah should read");
        let map = usfm_to_vref_map(&source);

        let mut chapters_present = std::collections::BTreeSet::new();
        for key in map.keys() {
            // Key format: "ISA <chapter>:<verse-or-bridge>"
            if let Some(after_book) = key.strip_prefix("ISA ")
                && let Some((chapter_str, _)) = after_book.split_once(':')
                && let Ok(chapter) = chapter_str.parse::<u32>()
            {
                chapters_present.insert(chapter);
            }
        }

        assert_eq!(
            chapters_present.len(),
            66,
            "expected 66 chapters, got {} (last present: {:?})",
            chapters_present.len(),
            chapters_present.iter().next_back()
        );
        assert!(
            map.len() >= 1200,
            "expected ~1290 verse entries, got {}",
            map.len()
        );
    }

    #[test]
    fn unclosed_cross_reference_recovers_at_next_verse() {
        let src = "\\id GEN\n\
                   \\c 1\n\
                   \\p\n\
                   \\v 1 Text \\x - \\xt 1.1: ref text never closed.\n\
                   \\v 2 Second verse.\n";
        let map = usfm_to_vref_map(src);
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("Second verse."),
        );
    }
}
