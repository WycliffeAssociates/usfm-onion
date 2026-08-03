use crate::format::FormatToken;
use crate::token::{Sid, Token, TokenData, TokenKind};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::collections::BTreeMap;

/// A cheap, non-allocating partition key for `partition_by_sid`. Native
/// tokens carry a `Copy` compact [`Sid`]; app/wire tokens carry an already
/// allocated sid string we borrow. Both render to the *same* sid string a
/// per-token `sid_string()` would have produced — but only once per block, so
/// the hot boundary scan compares keys instead of allocating a `String` per
/// token. `Empty` mirrors the old `unwrap_or_default()` (`""`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SidKey<'a> {
    Compact(Sid),
    Text(&'a str),
    Empty,
}

impl SidKey<'_> {
    /// Materialize the sid string this key represents — byte-identical to the
    /// old per-token `DiffableToken::sid_string().unwrap_or_default()`.
    fn to_sid_string(&self) -> String {
        match self {
            SidKey::Compact(sid) => sid.to_string(),
            SidKey::Text(text) => (*text).to_owned(),
            SidKey::Empty => String::new(),
        }
    }
}

pub mod skeleton;
#[cfg(test)]
mod skeleton_fixtures;
#[cfg(test)]
mod skeleton_proptest;
pub mod text_diff;
#[cfg(test)]
mod text_diff_fixtures;
#[cfg(test)]
mod text_diff_proptest;
pub use skeleton::{
    Anchor, CoveredBy, CoveredSide, DecisionStatus, DecisionUnit, DecisionUnitKind, DiffSkeleton,
    DupContext, MergeError, MergeSide, Slot, SlotRole, UnitId, diff_skeleton,
    diff_skeleton_by_chapter, diff_skeleton_by_chapter_from_tokens, diff_skeleton_canonical,
    merge_diff_blocks, merge_skeleton, revert_diff_block,
};
pub use text_diff::{
    TextDiffMode, TextDiffRun, TextDiffRunKind, UnitTextDiff, diff_skeleton_with_text,
    unit_text_diff,
};

pub trait DiffableToken: Clone {
    fn sid(&self) -> Option<&str> {
        None
    }
    fn sid_string(&self) -> Option<String> {
        self.sid().map(ToOwned::to_owned)
    }
    /// Cheap, non-allocating partition key. Default borrows the token's carried
    /// sid string; native [`Token`] overrides it to carry the `Copy` compact
    /// [`Sid`] instead of formatting one per call. Must render (via
    /// `SidKey::to_sid_string`) to exactly what `sid_string()` returns.
    fn sid_key(&self) -> SidKey<'_> {
        match self.sid() {
            Some(text) => SidKey::Text(text),
            None => SidKey::Empty,
        }
    }
    fn text(&self) -> &str;
    fn id(&self) -> Option<&str> {
        None
    }
    fn id_string(&self) -> Option<String> {
        self.id().map(ToOwned::to_owned)
    }
    fn kind_key(&self) -> Option<&str> {
        None
    }
    fn marker_key(&self) -> Option<&str> {
        None
    }
    /// The `(start, end)` payload of a chapter/verse number token, if this
    /// token is one. Lets [`derive_canonical_sids`] read marker/number
    /// structure identically over native and app-shaped tokens.
    fn number_range(&self) -> Option<(u32, Option<u32>)> {
        None
    }
    /// The book code this token carries, if it is a book-code token.
    fn book_code(&self) -> Option<&str> {
        None
    }
}

impl<'a> DiffableToken for Token<'a> {
    fn sid_string(&self) -> Option<String> {
        self.sid.map(|sid| sid.to_string())
    }

    fn sid_key(&self) -> SidKey<'_> {
        // The compact Sid is `Copy` — key on it directly rather than formatting
        // a `String` per token (the diff's #1 hot spot). `to_sid_string()`
        // reproduces `sid_string()` exactly, once per block.
        match self.sid {
            Some(sid) => SidKey::Compact(sid),
            None => SidKey::Empty,
        }
    }

    fn text(&self) -> &str {
        self.source
    }

    fn id_string(&self) -> Option<String> {
        Some(format!("{}-{}", self.id.book_code, self.id.index))
    }

    fn kind_key(&self) -> Option<&str> {
        Some(token_kind_key(self.kind()))
    }

    fn marker_key(&self) -> Option<&str> {
        self.marker_name()
    }

    fn number_range(&self) -> Option<(u32, Option<u32>)> {
        match self.data {
            TokenData::Number { start, end, .. } => Some((start, end)),
            _ => None,
        }
    }

    fn book_code(&self) -> Option<&str> {
        match self.data {
            TokenData::BookCode { code, .. } => Some(code),
            _ => None,
        }
    }
}

impl DiffableToken for FormatToken {
    fn sid(&self) -> Option<&str> {
        self.sid.as_deref()
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn kind_key(&self) -> Option<&str> {
        Some(token_kind_key(self.kind))
    }

    fn marker_key(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    fn number_range(&self) -> Option<(u32, Option<u32>)> {
        self.number_info.map(|(start, end, _)| (start, end))
    }

    fn book_code(&self) -> Option<&str> {
        matches!(self.kind, TokenKind::BookCode).then(|| self.text.trim())
    }
}

/// The single stateful structural pass that derives canonical scripture-
/// reference strings for a token stream. Sticky sid assignment from
/// marker/number structure (`\c`, `\v`, range payloads); it never trusts a
/// token's own carried sid. `book_code` is authoritative — an embedded `\id`
/// is not consulted.
///
/// Duplicate verses get a per-chapter `_dup_N` positional suffix; the key
/// includes the range end, so `GEN 1:1` and `GEN 1:1-2` never share a
/// counter. Chapter-open and intro blocks are `BOOK CHAPTER:0` / `BOOK 0:0`.
///
/// A repeated `\c` label (the same chapter number opened more than once in
/// one stream) gets a per-book positional `_cdup_N` suffix on every sid it
/// produces, riding in the verse segment rather than the chapter segment so
/// the chapter number stays a bare, always-parseable integer for existing
/// consumers (e.g. `diff::skeleton::pairing_key`, which parses the chapter
/// segment strictly and only tolerates trailing `_`-delimited content after
/// the verse digits). Verse-duplicate counting resets for every chapter
/// occurrence, matching how it already resets on every `\c`.
pub fn derive_canonical_sids<T: DiffableToken>(tokens: &[T], book_code: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut chapter: u32 = 0;
    let mut chapter_suffix = String::new();
    let mut current = format!("{book_code} 0:0");
    let mut seen_this_chapter = FxHashMap::<String, u32>::default();
    let mut seen_chapters = FxHashMap::<u32, u32>::default();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind_key() == Some("marker") {
            match token.marker_key() {
                Some("c") => {
                    if let Some((start, _)) = tokens.get(index + 1).and_then(T::number_range) {
                        chapter = start;
                        let occurrence = seen_chapters.entry(chapter).or_insert(0);
                        chapter_suffix = if *occurrence > 0 {
                            format!("_cdup_{occurrence}")
                        } else {
                            String::new()
                        };
                        *occurrence += 1;
                        current = format!("{book_code} {chapter}:0{chapter_suffix}");
                        seen_this_chapter.clear();
                    }
                }
                Some("v") => {
                    if let Some((start, end)) = tokens.get(index + 1).and_then(T::number_range) {
                        let verse_end = end.unwrap_or(start);
                        let range_base = if verse_end != start {
                            format!("{book_code} {chapter}:{start}-{verse_end}{chapter_suffix}")
                        } else {
                            format!("{book_code} {chapter}:{start}{chapter_suffix}")
                        };
                        let occurrence = seen_this_chapter.entry(range_base.clone()).or_insert(0);
                        current = if *occurrence > 0 {
                            format!("{range_base}_dup_{occurrence}")
                        } else {
                            range_base
                        };
                        *occurrence += 1;
                    }
                }
                _ => {}
            }
        }
        out.push(current.clone());
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SidBlock {
    pub block_id: String,
    pub semantic_sid: String,
    pub start: usize,
    pub end_exclusive: usize,
    pub prev_block_id: Option<String>,
    pub text_full: String,
}

/// Gapless, disjoint partition into contiguous same-sid blocks: every token
/// belongs to exactly one block, including tokens whose sid is empty — the
/// partition is unconditionally strict, with no option to drop them.
/// `sid_at(i)` supplies the sid string for token `i`; block id is that sid string for its
/// first contiguous occurrence, then `#1`, `#2`, ... for a later
/// non-contiguous reuse of the exact same sid. Block id never depends on a
/// token's own id — id drift across baseline/current is a supported input.
fn partition_by_sid<T, K, KeyFn>(
    tokens: &[T],
    key_at: KeyFn,
    to_sid_string: impl Fn(&K) -> String,
) -> Vec<SidBlock>
where
    T: DiffableToken,
    K: PartialEq + Eq + std::hash::Hash + Clone,
    KeyFn: Fn(usize) -> K,
{
    // Upper bounds: at most one block/occurrence-entry per token. Pre-sizing
    // avoids the RawVec grow→malloc chain the profile flagged as the #2 cost.
    let mut blocks = Vec::with_capacity(tokens.len());
    let mut occurrence_by_sid =
        FxHashMap::<K, usize>::with_capacity_and_hasher(tokens.len(), Default::default());
    let mut prev_block_id: Option<String> = None;
    let mut start = 0usize;

    while start < tokens.len() {
        // Boundary scan compares cheap keys (Copy Sid / borrowed &str), not a
        // freshly allocated `String` per token.
        let current_key = key_at(start);
        let mut end_exclusive = start + 1;

        while end_exclusive < tokens.len() && key_at(end_exclusive) == current_key {
            end_exclusive += 1;
        }

        // Materialize the sid string once per block, not once per token.
        let current_sid = to_sid_string(&current_key);
        let occurrence = occurrence_by_sid.entry(current_key).or_insert(0);
        let block_id = if *occurrence == 0 {
            current_sid.clone()
        } else {
            format!("{current_sid}#{occurrence}")
        };
        *occurrence += 1;

        let text_full = tokens[start..end_exclusive]
            .iter()
            .map(|token| token.text())
            .collect::<String>();

        blocks.push(SidBlock {
            block_id: block_id.clone(),
            semantic_sid: current_sid,
            start,
            end_exclusive,
            prev_block_id: prev_block_id.clone(),
            text_full,
        });
        prev_block_id = Some(block_id);
        start = end_exclusive;
    }

    blocks
}

/// Partition using each token's own carried sid ([`DiffableToken::sid_string`]).
/// This is the interim calling convention for external/app-shaped token
/// streams (`FormatToken`): carried sids are trusted unchanged, verbatim.
pub fn build_sid_blocks<T: DiffableToken>(tokens: &[T]) -> Vec<SidBlock> {
    partition_by_sid(
        tokens,
        |index| tokens[index].sid_key(),
        SidKey::to_sid_string,
    )
}

/// Partition using [`derive_canonical_sids`] — the native calling convention
/// for parsed `Token` streams, which never trusts a carried sid.
pub fn build_sid_blocks_canonical<T: DiffableToken>(
    tokens: &[T],
    book_code: &str,
) -> Vec<SidBlock> {
    let canonical = derive_canonical_sids(tokens, book_code);
    partition_by_sid(
        tokens,
        |index| canonical[index].as_str(),
        |key: &&str| (*key).to_owned(),
    )
}

fn token_kind_key(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Newline => "verticalWhitespace",
        TokenKind::OptBreak => "optBreak",
        TokenKind::Marker => "marker",
        TokenKind::EndMarker => "endMarker",
        TokenKind::Milestone => "milestone",
        TokenKind::MilestoneEnd => "milestoneEnd",
        TokenKind::BookCode => "bookCode",
        TokenKind::Number => "number",
        TokenKind::Text => "text",
    }
}

/// Classify how `baseline` and `current` differ: as pure whitespace churn
/// (`ws`), or — if not — as pure USFM markup churn with the same
/// reader-visible text (`usfm`). `usfm` is only computed when `ws` is
/// false, and both checks compare via iterators rather than materializing
/// normalized copies — a whole-corpus reformat is overwhelmingly
/// whitespace-only, so this keeps the common case to a single short-
/// circuiting pass per side with no allocation at all.
fn classify_text_diff(baseline: &str, current: &str) -> (bool, bool) {
    let ws = whitespace_stripped_eq(baseline, current);
    let usfm =
        !ws && whitespace_stripped_eq(&strip_usfm_markers(baseline), &strip_usfm_markers(current));
    (ws, usfm)
}

fn whitespace_stripped_eq(a: &str, b: &str) -> bool {
    a.chars()
        .filter(|ch| !ch.is_whitespace())
        .eq(b.chars().filter(|ch| !ch.is_whitespace()))
}

/// Strip USFM markers, leaving reader-visible text. Whitespace is left
/// as-is: the sole caller, [`classify_text_diff`], strips it separately via
/// [`whitespace_stripped_eq`], so collapsing/trimming it here would just be
/// discarded work.
fn strip_usfm_markers(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() && chars[index + 1].is_ascii_alphabetic()
        {
            index += 2;
            while index < chars.len() && chars[index].is_ascii_alphanumeric() {
                index += 1;
            }
            if index < chars.len() && chars[index] == '*' {
                index += 1;
            }
            continue;
        }

        out.push(chars[index]);
        index += 1;
    }

    out
}

fn group_tokens_by_book_and_chapter<T: DiffableToken>(
    tokens: &[T],
    default_book_code: &str,
) -> BTreeMap<String, BTreeMap<u32, Vec<T>>> {
    let mut out = BTreeMap::<String, BTreeMap<u32, Vec<T>>>::new();

    for token in tokens {
        let sid = token.sid_string().unwrap_or_default();
        let book_code = sid_book_code(&sid).unwrap_or(default_book_code);
        let chapter = sid_chapter_num(&sid).unwrap_or(0);
        out.entry(book_code.to_string())
            .or_default()
            .entry(chapter)
            .or_default()
            .push(token.clone());
    }

    out
}

fn sid_book_code(sid: &str) -> Option<&str> {
    sid.split_once(' ').map(|(book_code, _)| book_code)
}

fn sid_chapter_num(sid: &str) -> Option<u32> {
    let (_, rest) = sid.split_once(' ')?;
    let (chapter, _) = rest.split_once(':')?;
    let digits = chapter
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

#[cfg(test)]
mod canonical_sid_tests {
    use super::derive_canonical_sids;
    use crate::parse::parse;

    #[test]
    fn intro_material_before_any_chapter_is_book_0_0() {
        let parsed = parse("\\id GEN\n\\h Genesis\n\\c 1\n\\v 1 one\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let h_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "Genesis")
            .expect("heading text token");
        assert_eq!(sids[h_index], "GEN 0:0");
    }

    #[test]
    fn chapter_open_block_is_chapter_colon_zero() {
        let parsed = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 one\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let p_index = parsed
            .tokens
            .iter()
            .position(|t| matches!(t.data, crate::token::TokenData::Marker { name: "p", .. }))
            .expect("\\p token");
        assert_eq!(sids[p_index], "GEN 1:0");
    }

    #[test]
    fn single_verse_sid_has_no_range_suffix() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 one\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let text_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "one")
            .expect("verse text token");
        assert_eq!(sids[text_index], "GEN 1:1");
    }

    #[test]
    fn bridge_verse_keeps_the_full_range_sid_for_every_following_token() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1-2 a b\n\\v 3 c\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let a_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "a b")
            .expect("bridge verse text token");
        assert_eq!(sids[a_index], "GEN 1:1-2");
    }

    #[test]
    fn duplicate_verse_gets_a_per_chapter_positional_dup_suffix() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 a\n\\v 2 b\n\\v 1 c\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let a_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "a")
            .unwrap();
        let c_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "c")
            .unwrap();
        assert_eq!(sids[a_index], "GEN 1:1");
        assert_eq!(sids[c_index], "GEN 1:1_dup_1");
    }

    #[test]
    fn single_and_range_sids_never_share_a_dup_counter() {
        // \v 1 and \v 1-2 must key separately: a repeated \v 1-2 gets
        // `_dup_1` without disturbing the plain \v 1 occurrence count.
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 a\n\\v 1-2 b\n\\v 1-2 c\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let a_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "a")
            .unwrap();
        let b_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "b")
            .unwrap();
        let c_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "c")
            .unwrap();
        assert_eq!(sids[a_index], "GEN 1:1");
        assert_eq!(sids[b_index], "GEN 1:1-2");
        assert_eq!(sids[c_index], "GEN 1:1-2_dup_1");
    }

    #[test]
    fn repeated_chapter_label_gets_a_positional_cdup_suffix() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 first\n\\c 1\n\\v 1 second\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let first_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "first")
            .unwrap();
        let second_index = parsed
            .tokens
            .iter()
            .position(|t| t.source.trim() == "second")
            .unwrap();
        assert_eq!(sids[first_index], "GEN 1:1");
        assert_eq!(sids[second_index], "GEN 1:1_cdup_1");
    }

    #[test]
    fn repeated_chapter_resets_verse_duplicate_counting() {
        // Both chapter occurrences repeat \v 1; the second occurrence's own
        // internal repeat must get `_dup_1` scoped to that occurrence, not a
        // running count carried over from the first chapter.
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 a\n\\v 1 b\n\\c 1\n\\v 1 c\n\\v 1 d\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let index_of = |text: &str| {
            parsed
                .tokens
                .iter()
                .position(|t| t.source.trim() == text)
                .unwrap()
        };
        assert_eq!(sids[index_of("a")], "GEN 1:1");
        assert_eq!(sids[index_of("b")], "GEN 1:1_dup_1");
        assert_eq!(sids[index_of("c")], "GEN 1:1_cdup_1");
        assert_eq!(sids[index_of("d")], "GEN 1:1_cdup_1_dup_1");
    }

    #[test]
    fn a_third_chapter_occurrence_gets_its_own_positional_suffix() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 a\n\\c 1\n\\v 1 b\n\\c 1\n\\v 1 c\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let index_of = |text: &str| {
            parsed
                .tokens
                .iter()
                .position(|t| t.source.trim() == text)
                .unwrap()
        };
        assert_eq!(sids[index_of("a")], "GEN 1:1");
        assert_eq!(sids[index_of("b")], "GEN 1:1_cdup_1");
        assert_eq!(sids[index_of("c")], "GEN 1:1_cdup_2");
    }

    #[test]
    fn chapter_open_pseudo_sid_also_carries_the_cdup_suffix() {
        let parsed = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n");
        let sids = derive_canonical_sids(&parsed.tokens, "GEN");
        let p_indices: Vec<_> = parsed
            .tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t.data, crate::token::TokenData::Marker { name: "p", .. }))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(sids[p_indices[0]], "GEN 1:0");
        assert_eq!(sids[p_indices[1]], "GEN 1:0_cdup_1");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestToken {
        sid: Option<String>,
        text: String,
        id: Option<String>,
        kind_key: Option<String>,
        marker_key: Option<String>,
    }

    impl DiffableToken for TestToken {
        fn sid(&self) -> Option<&str> {
            self.sid.as_deref()
        }

        fn text(&self) -> &str {
            &self.text
        }

        fn id(&self) -> Option<&str> {
            self.id.as_deref()
        }

        fn kind_key(&self) -> Option<&str> {
            self.kind_key.as_deref()
        }

        fn marker_key(&self) -> Option<&str> {
            self.marker_key.as_deref()
        }
    }

    fn t(sid: &str, text: &str, id: Option<&str>) -> TestToken {
        TestToken {
            sid: Some(sid.to_string()),
            text: text.to_string(),
            id: id.map(str::to_string),
            kind_key: None,
            marker_key: None,
        }
    }

    #[test]
    fn partition_is_gapless_including_leading_empty_sid_tokens() {
        // A leading newline before \id has no sid yet (state.current_sid is
        // None until the first BookCode/\id). The partition is unconditionally
        // strict — there is no option to drop empty-sid tokens.
        let source = "\n\\id GEN\n\\c 1\n\\v 1 one\n";
        let parsed = parse(source);
        let blocks = build_sid_blocks_canonical(&parsed.tokens, "GEN");
        let reassembled = blocks
            .iter()
            .map(|block| block.text_full.as_str())
            .collect::<String>();
        assert_eq!(reassembled, source);
    }

    #[test]
    fn format_token_partition_is_also_gapless() {
        let tokens = vec![
            TestToken {
                sid: None,
                text: "\n".to_string(),
                id: None,
                kind_key: None,
                marker_key: None,
            },
            t("GEN 1:1", "one", None),
        ];
        let blocks = build_sid_blocks(&tokens);
        let reassembled = blocks
            .iter()
            .map(|block| block.text_full.as_str())
            .collect::<String>();
        assert_eq!(reassembled, "\none");
    }

    #[test]
    fn canonical_block_ids_agree_across_independent_id_numbering() {
        // Two independently-parsed streams of the same structure produce the
        // same canonical block-id candidates even though nothing here reads
        // token ids — id drift between baseline/current is a supported input.
        let baseline = parse("\\id GEN\n\\c 1\n\\v 1 a\n\\v 2 b\n");
        let current = parse("\\id GEN\n\\c 1\n\\v 1 a edited\n\\v 2 b\n");
        let baseline_blocks = build_sid_blocks_canonical(&baseline.tokens, "GEN");
        let current_blocks = build_sid_blocks_canonical(&current.tokens, "GEN");
        let baseline_ids = baseline_blocks
            .iter()
            .map(|b| b.block_id.as_str())
            .collect::<Vec<_>>();
        let current_ids = current_blocks
            .iter()
            .map(|b| b.block_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(baseline_ids, current_ids);
    }

    #[test]
    fn groups_contiguous_runs_of_same_sid() {
        let tokens = vec![
            t("GEN 1:1", "A", Some("1")),
            t("GEN 1:1", "B", Some("2")),
            t("GEN 1:2", "C", Some("3")),
            t("GEN 1:2", "D", Some("4")),
        ];

        let blocks = build_sid_blocks(&tokens);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].semantic_sid, "GEN 1:1");
        assert_eq!(blocks[0].text_full, "AB");
        assert_eq!(blocks[1].semantic_sid, "GEN 1:2");
        assert_eq!(blocks[1].text_full, "CD");
    }

    #[test]
    fn block_ids_never_depend_on_token_ids() {
        // Block id is the sid string (plus a `#N` collision suffix for a later
        // non-contiguous reuse) — never the token's own id. Id drift across
        // baseline/current is a supported input, not a block-identity input.
        let tokens = vec![
            t("GEN 1:1", "A", Some("tok-aaa")),
            t("GEN 1:1", "B", Some("tok-bbb")),
        ];
        let blocks = build_sid_blocks(&tokens);
        assert_eq!(blocks[0].block_id, "GEN 1:1");
    }

    #[test]
    fn non_contiguous_reuse_of_the_same_sid_gets_a_collision_suffix() {
        let tokens = vec![
            t("GEN 1:1", "A", None),
            t("GEN 1:2", "B", None),
            t("GEN 1:1", "C", None),
        ];
        let blocks = build_sid_blocks(&tokens);
        assert_eq!(blocks[0].block_id, "GEN 1:1");
        assert_eq!(blocks[1].block_id, "GEN 1:2");
        assert_eq!(blocks[2].block_id, "GEN 1:1#1");
    }

    #[test]
    fn coalesces_same_sid_id_drift_into_modified_and_can_revert() {
        let baseline = vec![t("ISA 33:9", "Alpha", Some("orig-id"))];
        let current = vec![t("ISA 33:9", "Beta", Some("new-id"))];

        let skeleton = diff_skeleton(&baseline, &current);

        assert_eq!(skeleton.units.len(), 1);
        assert_eq!(skeleton.units[0].status, DecisionStatus::Modified);
        assert_eq!(skeleton.units[0].baseline_sid.as_deref(), Some("ISA 33:9"));

        let reverted =
            revert_diff_block(skeleton.units[0].id.as_str(), &baseline, &current).unwrap();
        assert_eq!(reverted[0].text, "Alpha");
    }

    #[test]
    fn detects_whitespace_only_changes() {
        let baseline = vec![t("GEN 1:1", "Alpha  Beta", Some("a"))];
        let current = vec![t("GEN 1:1", "Alpha Beta", Some("a"))];
        let skeleton = diff_skeleton(&baseline, &current);
        let unit = &skeleton.units[0];

        assert_eq!(unit.status, DecisionStatus::Modified);
        assert!(unit.is_whitespace_change);
        assert!(!unit.is_usfm_structure_change);
    }

    #[test]
    fn current_parse_tokens_diffable_sid_and_id_shape_match_old_strings() {
        let parsed = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha\n");
        let verse_token = parsed
            .tokens
            .iter()
            .find(|token| {
                token.kind() == TokenKind::Number
                    && token.sid_string() == Some("GEN 1:1".to_string())
            })
            .expect("verse token should exist");

        assert_eq!(verse_token.sid_string().as_deref(), Some("GEN 1:1"));
        assert!(verse_token.id_string().unwrap().starts_with("GEN-"));
    }

    #[test]
    fn source_diff_by_chapter_groups_overlap_and_missing_chapters() {
        let baseline = "\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha\n\\c 2\n\\p\n\\v 1 Beta\n";
        let current = "\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha edited\n\\c 3\n\\p\n\\v 1 Gamma\n";
        let diffs = diff_skeleton_by_chapter(baseline, current);

        let gen_chapters = diffs.get("GEN").unwrap();
        assert!(gen_chapters.contains_key(&1));
        assert!(gen_chapters.contains_key(&2));
        assert!(gen_chapters.contains_key(&3));
    }
}
