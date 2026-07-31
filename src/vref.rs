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

/// Token count at or above which the map is built chapter-parallel. Below it the
/// fixed decomposition cost (partitioning, per-segment visitors, and the ordered
/// merge) outweighs the parallel gain, so the walk stays serial. Large books
/// clear it comfortably; a book below it projects in well under a millisecond
/// either way.
#[cfg(not(target_arch = "wasm32"))]
const PARALLEL_MIN_TOKENS: usize = 20_000;

pub fn tokens_to_vref_map_with_options(tokens: &[Token<'_>], options: VrefOptions) -> VrefMap {
    #[cfg(not(target_arch = "wasm32"))]
    if tokens.len() >= PARALLEL_MIN_TOKENS {
        return vref_map_partitioned(tokens, options);
    }
    let mut visitor = VrefVisitor::new(options);
    walk_tokens(tokens, &mut visitor);
    visitor.finish()
}

/// Partition the stream at `\c` and project each segment's verses through the
/// ordered-map seam, then merge the per-segment maps in segment order. Byte-
/// identical to the serial walk regardless of thread count: each segment walks
/// its absolute range via `walk_range` (real lookahead, whole-book close reasons
/// at the boundary), and the two pieces of state a whole-book walk carries across
/// a `\c` are reconciled after the fact rather than reset. The merge inserts in
/// segment order, so a reference repeated across chapters keeps the last-write
/// value serial lands on. The block verse-support flag (which a whole-book walk
/// carries across `\c`, never clearing it) is reconstructed from each segment's
/// [`SegmentSummary`]: a segment collects its leading verse text assuming the
/// flag is truthy, reports whether that assumption was load-bearing, and only the
/// rare segment whose carried flag is actually `false` is re-walked — real books
/// open a paragraph before any verse, so no re-walk happens. The corpus test
/// below also calls this directly to force the merge on small fixtures.
#[cfg(not(target_arch = "wasm32"))]
fn vref_map_partitioned(tokens: &[Token<'_>], options: VrefOptions) -> VrefMap {
    let segments = crate::walker::chapter_segments(tokens);
    let units: Vec<_> = segments
        .iter()
        .map(|segment| (segment.range.clone(), segment.boundary))
        .collect();

    let per_segment = crate::par::map_ordered(&units, |(range, boundary)| {
        let mut visitor = VrefVisitor::new(options);
        crate::walker::walk_range(tokens, range.clone(), *boundary, &mut visitor);
        visitor.finish_with_summary()
    });

    // Carry the block verse-support flag forward exactly as a whole-book walk
    // would (set on every block open, never reset across `\c`). A segment that
    // opened no block passes the flag through unchanged.
    let mut map = VrefMap::new();
    let mut carried: Option<bool> = None;
    for ((range, boundary), (segment_map, summary)) in units.iter().zip(per_segment) {
        let segment_map = if summary.seed_dependent && carried == Some(false) {
            // The segment collected leading verse text on the assumption the
            // carried flag was truthy, but it is actually `false`, so that text
            // must be dropped. Re-walk the segment with the true carried value.
            let mut visitor = VrefVisitor::new(options);
            visitor.current_block_supports_verse = Some(false);
            crate::walker::walk_range(tokens, range.clone(), *boundary, &mut visitor);
            visitor.finish()
        } else {
            segment_map
        };
        if summary.saw_block {
            carried = summary.last_block_supports_verse;
        }
        map.extend(segment_map);
    }
    map
}

/// What a chapter-parallel segment reports about the one visitor state a whole-
/// book walk carries across `\c` — the block verse-support flag — so
/// [`vref_map_partitioned`] can reconcile it without a second full scan.
#[cfg(not(target_arch = "wasm32"))]
struct SegmentSummary {
    /// A block scope opened in this segment (so it defines the flag for whatever
    /// follows it and for the next segment).
    saw_block: bool,
    /// The last block open's verse-support status; meaningful only when
    /// `saw_block`.
    last_block_supports_verse: Option<bool>,
    /// This segment collected verse text before opening any block of its own, so
    /// its output depends on the flag carried in from earlier chapters.
    seed_dependent: bool,
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

/// One verse's entry in a [`VrefIndex`], in the position the token stream put it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VrefEntry {
    pub sid: String,
    pub projection: VerseProjection,
}

/// Every verse's lossless projection, in **first-seen token order**, with a
/// by-SID lookup beside it. Same key set as [`VrefMap`]; the difference is
/// losslessness, the segment map, and order.
///
/// Order is part of the contract, not an implementation detail. A projection is
/// read against a document — an editor buffer, a file — and a document's verses
/// are in whatever order it actually puts them, including deliberately
/// out-of-order content: `\v 19` before `\v 2`, chapter 10 before chapter 2. A
/// sorted container at this boundary silently replaces that with an order the
/// document never had, and a consumer cannot recover the real one from the sorted
/// keys. So the entries are the authority and the lookup is the convenience;
/// nothing here sorts, and a consumer must never sort SIDs to recover sequence.
///
/// Serializes as its entries, in order, for the same reason.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct VrefIndex {
    entries: Vec<VrefEntry>,
    /// SID → position in `entries`. Skipped by serialization: it is derivable
    /// from the entries and would be a second, sorted view of the same data.
    #[serde(skip)]
    positions: BTreeMap<String, usize>,
}

impl VrefIndex {
    /// The verses in first-seen token order — the authoritative sequence.
    pub fn entries(&self) -> &[VrefEntry] {
        &self.entries
    }

    /// The SIDs in first-seen token order.
    pub fn sids(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|entry| entry.sid.as_str())
    }

    /// Iterates entries in first-seen token order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &VerseProjection)> {
        self.entries
            .iter()
            .map(|entry| (entry.sid.as_str(), &entry.projection))
    }

    pub fn get(&self, sid: &str) -> Option<&VerseProjection> {
        self.positions
            .get(sid)
            .and_then(|position| self.entries.get(*position))
            .map(|entry| &entry.projection)
    }

    pub fn contains_key(&self, sid: &str) -> bool {
        self.positions.contains_key(sid)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Records one verse's projection.
    ///
    /// A SID that has already been seen keeps its original position and takes the
    /// new projection — the same "one entry per SID, last write wins" the map this
    /// replaced had, with the position it now also has to answer for made
    /// explicit: where the verse first appeared in the stream.
    fn insert(&mut self, sid: String, projection: VerseProjection) {
        match self.positions.get(&sid) {
            Some(position) => self.entries[*position].projection = projection,
            None => {
                self.positions.insert(sid.clone(), self.entries.len());
                self.entries.push(VrefEntry { sid, projection });
            }
        }
    }
}

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
    // Persists across paragraph boundaries: once any Block has been
    // seen, this reflects the latest Block's supports-verse status, even
    // after that Block closes. Matches the pre-walker behaviour.
    current_block_supports_verse: Option<bool>,
    // Chapter-parallel bookkeeping for [`SegmentSummary`]; unused on the serial
    // path. `saw_block` — any Block opened here. `seed_dependent` — verse text was
    // collected before this segment's first Block, so the block flag carried in
    // from earlier chapters was load-bearing.
    #[cfg(not(target_arch = "wasm32"))]
    saw_block: bool,
    #[cfg(not(target_arch = "wasm32"))]
    seed_dependent: bool,
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

    #[cfg(not(target_arch = "wasm32"))]
    fn finish_with_summary(mut self) -> (VrefMap, SegmentSummary) {
        self.flush_current_verse();
        let summary = SegmentSummary {
            saw_block: self.saw_block,
            last_block_supports_verse: self.current_block_supports_verse,
            seed_dependent: self.seed_dependent,
        };
        (self.map, summary)
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
    }

    fn can_collect_text(&self, ctx: &WalkContext<'_, '_>) -> bool {
        self.current_ref.is_some()
            && !ctx.in_note()
            && self.current_block_supports_verse.unwrap_or(true)
    }

    /// Appends a content token's bytes, verbatim.
    ///
    /// Nothing is inserted, removed, or rewritten: a verse projection is the source
    /// bytes of its content tokens in order, so whatever separated two words in the
    /// document separates them here. Normalizing — trimming, collapsing runs,
    /// swapping a newline for a space — is a consumer's decision to make on top of a
    /// faithful projection, not something this walker can decide for every consumer.
    fn push_collected_text(&mut self, fragment: &str) {
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
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.saw_block = true;
            }
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
            #[cfg(not(target_arch = "wasm32"))]
            if !self.saw_block {
                self.seed_dependent = true;
            }
            self.push_collected_text(token.source);
        }
    }

    fn on_newline(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // A newline inside a verse is content, and its bytes are the separator the
        // source already spells. Discarding it and re-deriving "probably a space"
        // was how `gladness` and `so` came out as one word: the byte that kept them
        // apart was thrown away, and the replacement was conditional on
        // neighbouring whitespace that had also been thrown away.
        if self.can_collect_text(ctx) {
            self.current_text.push_str(token.source);
        }
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
        let text = token.source();
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
        if let Some(reference) = verse_ref_str(token.sid(), token.source()) {
            self.current_ref = Some(reference);
        }
    }

    fn on_text(&mut self, ctx: &WalkContext<'tokens, '_>, token: &'tokens T, _token_index: usize) {
        if self.can_collect_text(ctx) {
            self.push_token(token);
        }
    }

    /// A newline inside a verse is a content token like any other: it carries the
    /// byte that separates what surrounds it, and it has a real id and source span,
    /// so it becomes a segment like any other and the segments still tile the text
    /// completely. Dropping it was what made two words collide.
    fn on_newline(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        _token_index: usize,
    ) {
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

    /// A document's verses are in whatever order the document puts them, and a
    /// projection read against that document has to say so. This fixture is
    /// deliberately out of order in both dimensions a sorted container would
    /// silently "fix": verse 19 before verse 2 inside a chapter, and chapter 10
    /// before chapter 2. The emitted sequence must be the stream's, and a consumer
    /// must never have to sort — sorting is precisely what loses the answer.
    const OUT_OF_ORDER: &str = "\\id GEN\n\\c 29\n\\p\n\\v 19 nineteen\n\\v 2 two\n\\c 10\n\\p\n\\v 1 ten one\n\\c 2\n\\p\n\\v 1 two one\n";

    #[test]
    fn vref_index_emits_first_seen_token_order() {
        let index = usfm_to_vref_index(OUT_OF_ORDER);
        let order: Vec<&str> = index.sids().collect();
        assert_eq!(
            order,
            ["GEN 29:19", "GEN 29:2", "GEN 10:1", "GEN 2:1"],
            "the projection must report the document's own order"
        );
        // Every one of those pairs is a case where sorting would have differed:
        // lexicographic order puts "GEN 10:1" first and "GEN 29:19" before
        // "GEN 29:2"; numeric order puts chapter 2 first.
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_ne!(
            order, sorted,
            "the fixture must actually distinguish the two"
        );

        // Order is the authority, the lookup is the convenience — and they agree.
        for sid in &order {
            assert!(
                index.get(sid).is_some(),
                "{sid} must be reachable by lookup"
            );
        }
        assert_eq!(index.len(), order.len());

        // Post-serialization is where a sorted container betrays itself, since a
        // JSON object would emit its keys sorted. The serialized form is the
        // ordered entries.
        let json = serde_json::to_value(&index).expect("serializes");
        let serialized: Vec<&str> = json
            .as_array()
            .expect("an ordered array, not an object")
            .iter()
            .map(|entry| entry["sid"].as_str().expect("sid"))
            .collect();
        assert_eq!(serialized, order);
    }

    /// The container changed; the duplicate-SID answer did not. One entry per SID
    /// with the last projection written is what the map did, and the position that
    /// entry now also has to answer for is its first-seen one.
    #[test]
    fn a_repeated_sid_keeps_its_first_position_and_takes_the_last_projection() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 first\n\\v 2 second\n\\v 1 again\n";
        let index = usfm_to_vref_index(source);
        assert_eq!(index.sids().collect::<Vec<_>>(), ["GEN 1:1", "GEN 1:2"]);
        assert_eq!(index.len(), 2, "one entry per SID");
        assert_eq!(
            index.get("GEN 1:1").expect("present").text,
            "again\n",
            "the last write wins, exactly as it did before"
        );
        // And the key set still matches `to_vref`, whose own duplicate behavior is
        // unchanged.
        let map = usfm_to_vref_map(source);
        assert_eq!(
            index.sids().collect::<std::collections::BTreeSet<_>>(),
            map.keys().map(String::as_str).collect()
        );
    }

    // Test 1 — scope parity: the index collects exactly the verses `to_vref`
    // does (notes/headings excluded, whitespace-only verses dropped).
    #[test]
    fn vref_index_key_set_matches_to_vref() {
        for &src in VREF_INDEX_FIXTURES {
            let index = usfm_to_vref_index(src);
            let map = usfm_to_vref_map(src);
            let index_keys: std::collections::BTreeSet<_> =
                index.sids().map(str::to_owned).collect();
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
            for (sid, proj) in index.iter() {
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
            for (sid, proj) in index.iter() {
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
            for (sid, proj) in index.iter() {
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

        // Poetry join: every content byte between the two words survives — the
        // trailing space *and* the newline the source put there — while the `\q2`
        // delimiter itself contributes nothing.
        let src = "\\id ISA\n\\c 9\n\\p\n\\v 2 The people walked in  darkness \n\\q2 have seen a great light;\n";
        let proj = usfm_to_vref_index(src)
            .get("ISA 9:2")
            .cloned()
            .expect("verse present");
        assert!(proj.text.contains("darkness \nhave"));
        // The one thing that must never appear is a byte the source did not have.
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
            .get("GEN 1:1")
            .cloned()
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
            Some("In the beginning God created the heavens and the earth.\n")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("The earth was without form and void.\n")
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
            Some("First part.\nSecond part.")
        );
    }

    #[test]
    fn a_structural_break_keeps_its_own_separator_byte_and_leaks_no_delimiter() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\nGod created.");
        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning\nGod created.")
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
        assert_eq!(map.get("GEN 1:1").map(String::as_str), Some(" padded \n"));

        let trimmed = usfm_to_vref_map_with_options(src, VrefOptions { trim: true });
        assert_eq!(trimmed.get("GEN 1:1").map(String::as_str), Some("padded"));
    }

    #[test]
    fn root_level_verses_are_collected() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 In the beginning.\n\\v 2 And God said.");
        let map = tokens_to_vref_map(&parsed.tokens);

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.\n")
        );
        // No trailing newline on this one: the verse ends the file, so there is no
        // byte there to keep. Nothing is invented to make the two look alike.
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
            Some("The first two verses share text.\n"),
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
        assert_eq!(
            map.get("GEN 1:1,3").map(String::as_str),
            Some("Combined.\n"),
        );
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
            Some("Second verse — should still appear.\n"),
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
            Some("Second verse.\n"),
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod partition_tests {
    use super::{VrefOptions, VrefVisitor, vref_map_partitioned, vref_map_to_json_string};
    use crate::parse::parse;
    use crate::token::Token;
    use crate::walker::walk_tokens;
    use std::path::{Path, PathBuf};

    /// Guaranteed-serial projection, independent of the token-count threshold, so
    /// it stays a fixed baseline even as the routing in
    /// `tokens_to_vref_map_with_options` changes.
    fn serial(tokens: &[Token<'_>], options: VrefOptions) -> super::VrefMap {
        let mut visitor = VrefVisitor::new(options);
        walk_tokens(tokens, &mut visitor);
        visitor.finish()
    }

    /// Both trim settings — trim gates the flush, so it must survive segmentation.
    const OPTION_MATRIX: [VrefOptions; 2] =
        [VrefOptions { trim: false }, VrefOptions { trim: true }];

    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn partitioned_matches_serial_over_test_data() {
        assert_identical_over("testData");
    }

    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn partitioned_matches_serial_over_example_corpora() {
        assert_identical_over("example-corpora");
    }

    #[test]
    fn partitioned_matches_serial_on_boundary_shapes() {
        // Shapes the corpora may under-exercise: no `\c`; a bare verse opening a
        // chapter with no paragraph of its own, right after the previous chapter's
        // last block was a non-verse-supporting heading (the one carried-state
        // hazard the seed reproduces); a repeated reference across chapters and
        // within one chapter (last write must win); an open note before `\c`; a
        // trailing-whitespace verse (trim-sensitive); and a chapter opened with no
        // book/id at all.
        let cases = [
            "",
            "\\id GEN\n\\c 1\n\\v 1 no second chapter\n",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 aaa\n\\s trailing heading\n\\c 2\n\\v 1 bbb\n",
            "\\id GEN\n\\c 1\n\\s only a heading\n\\c 2\n\\v 1 bare verse after heading\n",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 first\n\\c 1\n\\p\n\\v 1 second\n",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 one\n\\v 1 two\n",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 text \\f + \\ft open note\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p\n\\v 1  padded \n\\c 2\n\\p\n\\v 1 next\n",
            "no id at all\n\\c 1\n\\p\n\\v 1 orphan chapter\n",
        ];
        for (index, source) in cases.iter().enumerate() {
            assert_identical(source, &format!("boundary case #{index}"));
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
        let parsed = parse(source);
        for options in OPTION_MATRIX {
            let want = serial(&parsed.tokens, options);
            // Force the partitioned merge even on small fixtures below the threshold.
            let got = vref_map_partitioned(&parsed.tokens, options);
            assert_eq!(got, want, "map differs for {label} (trim={})", options.trim);
            assert_eq!(
                vref_map_to_json_string(&got),
                vref_map_to_json_string(&want),
                "json differs for {label} (trim={})",
                options.trim
            );
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
