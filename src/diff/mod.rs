use crate::format::FormatToken;
use crate::parse::parse;
use crate::token::{Token, TokenKind};
use serde::Serialize;
use similar::{capture_diff_slices, Algorithm, ChangeTag};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

pub trait DiffableToken: Clone {
    fn sid(&self) -> Option<&str> {
        None
    }
    fn sid_string(&self) -> Option<String> {
        self.sid().map(ToOwned::to_owned)
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
}

impl<'a> DiffableToken for Token<'a> {
    fn sid_string(&self) -> Option<String> {
        self.sid.map(|sid| {
            if sid.verse == 0 {
                format!("{} {}:0", sid.book_code, sid.chapter)
            } else {
                format!("{} {}:{}", sid.book_code, sid.chapter, sid.verse)
            }
        })
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffTokenChange {
    Unchanged,
    Added,
    Deleted,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TokenAlignment {
    pub change: DiffTokenChange,
    pub counterpart_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffUndoSide {
    Original,
    Current,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SidBlockDiff {
    pub block_id: String,
    pub semantic_sid: String,
    pub status: DiffStatus,
    pub original: Option<SidBlock>,
    pub current: Option<SidBlock>,
    pub original_text: String,
    pub current_text: String,
    pub original_text_only: String,
    pub current_text_only: String,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
}

#[derive(Debug, Clone)]
struct NormalizedBlockText {
    text_only: String,
    full_without_whitespace: String,
    text_only_without_whitespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChapterTokenDiff<T> {
    pub block_id: String,
    pub semantic_sid: String,
    pub status: DiffStatus,
    pub original: Option<SidBlock>,
    pub current: Option<SidBlock>,
    pub original_text: String,
    pub current_text: String,
    pub original_text_only: String,
    pub current_text_only: String,
    pub is_whitespace_change: bool,
    pub is_usfm_structure_change: bool,
    pub original_tokens: Vec<T>,
    pub current_tokens: Vec<T>,
    pub original_alignment: Vec<TokenAlignment>,
    pub current_alignment: Vec<TokenAlignment>,
    pub undo_side: DiffUndoSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuildSidBlocksOptions {
    pub allow_empty_sid: bool,
}

impl Default for BuildSidBlocksOptions {
    fn default() -> Self {
        Self {
            allow_empty_sid: true,
        }
    }
}

pub type DiffsByChapterMap<TDiff> = BTreeMap<String, BTreeMap<u32, Vec<TDiff>>>;

pub fn build_sid_blocks<T: DiffableToken>(
    tokens: &[T],
    options: &BuildSidBlocksOptions,
) -> Vec<SidBlock> {
    let mut blocks = Vec::new();
    let mut occurrence_by_sid = BTreeMap::<String, usize>::new();
    let mut prev_block_id: Option<String> = None;
    let mut index = 0usize;

    while let Some(start) = next_block_start(tokens, index, options) {
        let current_sid = normalized_sid(&tokens[start]);
        let mut end_exclusive = start + 1;

        while end_exclusive < tokens.len()
            && token_included(&tokens[end_exclusive], options)
            && normalized_sid(&tokens[end_exclusive]) == current_sid
        {
            end_exclusive += 1;
        }

        let block_id = if let Some(first_token_id) = tokens[start].id_string() {
            format!("{current_sid}::{first_token_id}")
        } else {
            let next_occurrence = occurrence_by_sid.entry(current_sid.clone()).or_default();
            let block_id = format!("{current_sid}#{}", *next_occurrence);
            *next_occurrence += 1;
            block_id
        };

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
        index = end_exclusive;
    }

    blocks
}

pub fn diff_sid_blocks(original_blocks: &[SidBlock], current_blocks: &[SidBlock]) -> Vec<SidBlockDiff> {
    let original_seq = original_blocks
        .iter()
        .map(|block| block.block_id.clone())
        .collect::<Vec<_>>();
    let current_seq = current_blocks
        .iter()
        .map(|block| block.block_id.clone())
        .collect::<Vec<_>>();

    let original_by_id = original_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), block))
        .collect::<HashMap<_, _>>();
    let current_by_id = current_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), block))
        .collect::<HashMap<_, _>>();
    let original_text_by_id = original_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), normalize_block_text(&block.text_full)))
        .collect::<HashMap<_, _>>();
    let current_text_by_id = current_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), normalize_block_text(&block.text_full)))
        .collect::<HashMap<_, _>>();

    let mut out = Vec::new();

    for change in diff_id_sequences(&original_seq, &current_seq) {
        match change.tag {
            DiffStatus::Added => {
                for id in change.ids {
                    let Some(current) = current_by_id.get(id.as_str()) else {
                        continue;
                    };
                    let Some(current_text) = current_text_by_id.get(id.as_str()) else {
                        continue;
                    };
                    out.push(SidBlockDiff {
                        block_id: id,
                        semantic_sid: current.semantic_sid.clone(),
                        status: DiffStatus::Added,
                        original: None,
                        current: Some((*current).clone()),
                        original_text: String::new(),
                        current_text: current.text_full.clone(),
                        original_text_only: String::new(),
                        current_text_only: current_text.text_only.clone(),
                        is_whitespace_change: false,
                        is_usfm_structure_change: current_text.text_only.is_empty(),
                    });
                }
            }
            DiffStatus::Deleted => {
                for id in change.ids {
                    let Some(original) = original_by_id.get(id.as_str()) else {
                        continue;
                    };
                    let Some(original_text) = original_text_by_id.get(id.as_str()) else {
                        continue;
                    };
                    out.push(SidBlockDiff {
                        block_id: id,
                        semantic_sid: original.semantic_sid.clone(),
                        status: DiffStatus::Deleted,
                        original: Some((*original).clone()),
                        current: None,
                        original_text: original.text_full.clone(),
                        current_text: String::new(),
                        original_text_only: original_text.text_only.clone(),
                        current_text_only: String::new(),
                        is_whitespace_change: false,
                        is_usfm_structure_change: original_text.text_only.is_empty(),
                    });
                }
            }
            DiffStatus::Unchanged => {
                for id in change.ids {
                    let Some(original) = original_by_id.get(id.as_str()) else {
                        continue;
                    };
                    let Some(current) = current_by_id.get(id.as_str()) else {
                        continue;
                    };
                    let Some(original_text) = original_text_by_id.get(id.as_str()) else {
                        continue;
                    };
                    let Some(current_text) = current_text_by_id.get(id.as_str()) else {
                        continue;
                    };

                    if original.text_full == current.text_full {
                        out.push(SidBlockDiff {
                            block_id: id,
                            semantic_sid: current.semantic_sid.clone(),
                            status: DiffStatus::Unchanged,
                            original: Some((*original).clone()),
                            current: Some((*current).clone()),
                            original_text: original.text_full.clone(),
                            current_text: current.text_full.clone(),
                            original_text_only: original_text.text_only.clone(),
                            current_text_only: current_text.text_only.clone(),
                            is_whitespace_change: false,
                            is_usfm_structure_change: false,
                        });
                    } else {
                        out.push(build_modified_diff_with_normalized(
                            id,
                            original.semantic_sid.clone(),
                            Some((*original).clone()),
                            Some((*current).clone()),
                            original.text_full.clone(),
                            current.text_full.clone(),
                            original_text,
                            current_text,
                        ));
                    }
                }
            }
            DiffStatus::Modified => {}
        }
    }

    coalesce_delete_add_pairs(out)
}

pub fn diff_chapter_token_streams<T: DiffableToken>(
    baseline_tokens: &[T],
    current_tokens: &[T],
    options: &BuildSidBlocksOptions,
) -> Vec<ChapterTokenDiff<T>> {
    let baseline_blocks = build_sid_blocks(baseline_tokens, options);
    let current_blocks = build_sid_blocks(current_tokens, options);
    let diffs = diff_sid_blocks(&baseline_blocks, &current_blocks);

    diffs
        .into_iter()
        .map(|diff| build_chapter_token_diff(diff, baseline_tokens, current_tokens))
        .collect()
}

pub fn diff_usfm_sources<'a>(
    baseline_usfm: &'a str,
    current_usfm: &'a str,
    build_options: &BuildSidBlocksOptions,
) -> Vec<ChapterTokenDiff<Token<'a>>> {
    let baseline = parse(baseline_usfm);
    let current = parse(current_usfm);
    diff_chapter_token_streams(&baseline.tokens, &current.tokens, build_options)
}

pub fn diff_usfm_sources_by_chapter<'a>(
    baseline_usfm: &'a str,
    current_usfm: &'a str,
    build_options: &BuildSidBlocksOptions,
) -> DiffsByChapterMap<ChapterTokenDiff<Token<'a>>> {
    let baseline = parse(baseline_usfm);
    let current = parse(current_usfm);

    let baseline_groups = group_tokens_by_book_and_chapter(
        &baseline.tokens,
        baseline.analysis.book_code.unwrap_or("unknown"),
    );
    let current_groups = group_tokens_by_book_and_chapter(
        &current.tokens,
        current.analysis.book_code.unwrap_or("unknown"),
    );

    let mut out = DiffsByChapterMap::new();
    let mut all_books = BTreeSet::<String>::new();
    all_books.extend(baseline_groups.keys().cloned());
    all_books.extend(current_groups.keys().cloned());

    for book in all_books {
        let mut all_chapters = BTreeSet::<u32>::new();
        if let Some(chapters) = baseline_groups.get(&book) {
            all_chapters.extend(chapters.keys().copied());
        }
        if let Some(chapters) = current_groups.get(&book) {
            all_chapters.extend(chapters.keys().copied());
        }

        for chapter in all_chapters {
            let baseline_slice = baseline_groups
                .get(&book)
                .and_then(|chapters| chapters.get(&chapter))
                .cloned()
                .unwrap_or_default();
            let current_slice = current_groups
                .get(&book)
                .and_then(|chapters| chapters.get(&chapter))
                .cloned()
                .unwrap_or_default();

            let diffs = diff_chapter_token_streams(&baseline_slice, &current_slice, build_options);
            out = replace_chapter_diffs_in_map(&out, &book, chapter, diffs);
        }
    }

    out
}

fn build_chapter_token_diff<T: DiffableToken>(
    diff: SidBlockDiff,
    baseline_tokens: &[T],
    current_tokens: &[T],
) -> ChapterTokenDiff<T> {
    let original_tokens = diff
        .original
        .as_ref()
        .map(|block| baseline_tokens[block.start..block.end_exclusive].to_vec())
        .unwrap_or_default();
    let current_tokens_slice = diff
        .current
        .as_ref()
        .map(|block| current_tokens[block.start..block.end_exclusive].to_vec())
        .unwrap_or_default();
    let (original_alignment, current_alignment) =
        align_token_sequences(&original_tokens, &current_tokens_slice);

    ChapterTokenDiff {
        block_id: diff.block_id,
        semantic_sid: diff.semantic_sid,
        status: diff.status,
        original: diff.original,
        current: diff.current,
        original_text: diff.original_text,
        current_text: diff.current_text,
        original_text_only: diff.original_text_only,
        current_text_only: diff.current_text_only,
        is_whitespace_change: diff.is_whitespace_change,
        is_usfm_structure_change: diff.is_usfm_structure_change,
        original_tokens,
        current_tokens: current_tokens_slice,
        original_alignment,
        current_alignment,
        undo_side: diff_undo_side(diff.status),
    }
}

fn align_token_sequences<T: DiffableToken>(
    original_tokens: &[T],
    current_tokens: &[T],
) -> (Vec<TokenAlignment>, Vec<TokenAlignment>) {
    if original_tokens.is_empty() && current_tokens.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut original_alignment = vec![
        TokenAlignment {
            change: DiffTokenChange::Deleted,
            counterpart_index: None,
        };
        original_tokens.len()
    ];
    let mut current_alignment = vec![
        TokenAlignment {
            change: DiffTokenChange::Added,
            counterpart_index: None,
        };
        current_tokens.len()
    ];

    let original_keys = original_tokens
        .iter()
        .map(token_comparable_key)
        .collect::<Vec<_>>();
    let current_keys = current_tokens
        .iter()
        .map(token_comparable_key)
        .collect::<Vec<_>>();

    if original_keys == current_keys {
        for index in 0..original_tokens.len() {
            original_alignment[index] = TokenAlignment {
                change: DiffTokenChange::Unchanged,
                counterpart_index: Some(index),
            };
        }
        for index in 0..current_tokens.len() {
            current_alignment[index] = TokenAlignment {
                change: DiffTokenChange::Unchanged,
                counterpart_index: Some(index),
            };
        }
        return (original_alignment, current_alignment);
    }

    let seq = diff_sequences(&original_keys, &current_keys);

    let mut original_cursor = 0usize;
    let mut current_cursor = 0usize;
    let mut index = 0usize;

    while index < seq.len() {
        let Some(part) = seq.get(index) else {
            break;
        };
        let part_len = part.ids.len();

        match part.tag {
            DiffStatus::Unchanged => {
                for offset in 0..part_len {
                    let original_index = original_cursor + offset;
                    let current_index = current_cursor + offset;
                    if original_index >= original_tokens.len() || current_index >= current_tokens.len()
                    {
                        continue;
                    }
                    original_alignment[original_index] = TokenAlignment {
                        change: DiffTokenChange::Unchanged,
                        counterpart_index: Some(current_index),
                    };
                    current_alignment[current_index] = TokenAlignment {
                        change: DiffTokenChange::Unchanged,
                        counterpart_index: Some(original_index),
                    };
                }
                original_cursor += part_len;
                current_cursor += part_len;
            }
            DiffStatus::Deleted => {
                if let Some(next) = seq.get(index + 1) && next.tag == DiffStatus::Added {
                    align_removed_added_chunk(
                        original_tokens,
                        current_tokens,
                        original_cursor,
                        part_len,
                        current_cursor,
                        next.ids.len(),
                        &mut original_alignment,
                        &mut current_alignment,
                    );
                    original_cursor += part_len;
                    current_cursor += next.ids.len();
                    index += 1;
                } else {
                    for offset in 0..part_len {
                        let original_index = original_cursor + offset;
                        if original_index >= original_tokens.len() {
                            continue;
                        }
                        original_alignment[original_index] = TokenAlignment {
                            change: DiffTokenChange::Deleted,
                            counterpart_index: None,
                        };
                    }
                    original_cursor += part_len;
                }
            }
            DiffStatus::Added => {
                for offset in 0..part_len {
                    let current_index = current_cursor + offset;
                    if current_index >= current_tokens.len() {
                        continue;
                    }
                    current_alignment[current_index] = TokenAlignment {
                        change: DiffTokenChange::Added,
                        counterpart_index: None,
                    };
                }
                current_cursor += part_len;
            }
            DiffStatus::Modified => {}
        }

        index += 1;
    }

    (original_alignment, current_alignment)
}

#[allow(clippy::too_many_arguments)]
fn align_removed_added_chunk<T: DiffableToken>(
    original_tokens: &[T],
    current_tokens: &[T],
    original_start: usize,
    original_len: usize,
    current_start: usize,
    current_len: usize,
    original_alignment: &mut [TokenAlignment],
    current_alignment: &mut [TokenAlignment],
) {
    let original_slice =
        &original_tokens[original_start..original_start.saturating_add(original_len)];
    let current_slice = &current_tokens[current_start..current_start.saturating_add(current_len)];
    let original_shapes = original_slice.iter().map(token_shape_key).collect::<Vec<_>>();
    let current_shapes = current_slice.iter().map(token_shape_key).collect::<Vec<_>>();
    let seq = diff_sequences(&original_shapes, &current_shapes);

    let mut original_cursor = 0usize;
    let mut current_cursor = 0usize;

    for part in seq {
        let part_len = part.ids.len();
        match part.tag {
            DiffStatus::Unchanged => {
                for offset in 0..part_len {
                    let original_local = original_cursor + offset;
                    let current_local = current_cursor + offset;
                    let original_index = original_start + original_local;
                    let current_index = current_start + current_local;
                    let Some(original_token) = original_slice.get(original_local) else {
                        continue;
                    };
                    let Some(current_token) = current_slice.get(current_local) else {
                        continue;
                    };
                    let changed =
                        token_comparable_key(original_token) != token_comparable_key(current_token);
                    let change = if changed && can_pair_as_modified(original_token, current_token) {
                        DiffTokenChange::Modified
                    } else {
                        DiffTokenChange::Unchanged
                    };

                    if let Some(slot) = original_alignment.get_mut(original_index) {
                        *slot = TokenAlignment {
                            change,
                            counterpart_index: Some(current_index),
                        };
                    }
                    if let Some(slot) = current_alignment.get_mut(current_index) {
                        *slot = TokenAlignment {
                            change,
                            counterpart_index: Some(original_index),
                        };
                    }
                }
                original_cursor += part_len;
                current_cursor += part_len;
            }
            DiffStatus::Deleted => {
                for offset in 0..part_len {
                    let original_index = original_start + original_cursor + offset;
                    if let Some(slot) = original_alignment.get_mut(original_index) {
                        *slot = TokenAlignment {
                            change: DiffTokenChange::Deleted,
                            counterpart_index: None,
                        };
                    }
                }
                original_cursor += part_len;
            }
            DiffStatus::Added => {
                for offset in 0..part_len {
                    let current_index = current_start + current_cursor + offset;
                    if let Some(slot) = current_alignment.get_mut(current_index) {
                        *slot = TokenAlignment {
                            change: DiffTokenChange::Added,
                            counterpart_index: None,
                        };
                    }
                }
                current_cursor += part_len;
            }
            DiffStatus::Modified => {}
        }
    }
}

fn token_shape_key<T: DiffableToken>(token: &T) -> String {
    if token_is_linebreak(token) {
        return "linebreak".to_string();
    }
    format!(
        "{}|{}",
        token.kind_key().unwrap_or_default(),
        token.marker_key().unwrap_or_default()
    )
}

fn token_comparable_key<T: DiffableToken>(token: &T) -> String {
    if token_is_linebreak(token) {
        return "linebreak".to_string();
    }
    let kind = token.kind_key().unwrap_or_default();
    let marker = token.marker_key().unwrap_or_default();
    let comparable_text = match kind {
        "marker" | "endMarker" => String::new(),
        "number" => token.text().trim().to_string(),
        _ => token.text().to_string(),
    };
    format!("{kind}|{marker}|{comparable_text}")
}

fn can_pair_as_modified<T: DiffableToken>(original: &T, current: &T) -> bool {
    if token_is_linebreak(original) || token_is_linebreak(current) {
        return false;
    }

    let kind_match = match (original.kind_key(), current.kind_key()) {
        (Some(left), Some(right)) => left == right,
        (None, None) => true,
        _ => false,
    };
    if !kind_match {
        return false;
    }

    original.marker_key().unwrap_or_default() == current.marker_key().unwrap_or_default()
}

fn token_is_linebreak<T: DiffableToken>(token: &T) -> bool {
    matches!(token.kind_key(), Some("verticalWhitespace"))
}

fn diff_undo_side(status: DiffStatus) -> DiffUndoSide {
    match status {
        DiffStatus::Deleted => DiffUndoSide::Original,
        DiffStatus::Added | DiffStatus::Modified | DiffStatus::Unchanged => DiffUndoSide::Current,
    }
}

fn token_kind_key(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Newline => "verticalWhitespace",
        TokenKind::OptBreak => "optBreak",
        TokenKind::Marker => "marker",
        TokenKind::EndMarker => "endMarker",
        TokenKind::Milestone => "milestone",
        TokenKind::MilestoneEnd => "milestoneEnd",
        TokenKind::AttributeList => "attributes",
        TokenKind::BookCode => "bookCode",
        TokenKind::Number => "number",
        TokenKind::Text => "text",
    }
}

pub fn apply_revert_by_block_id<T: DiffableToken>(
    diff_block_id: &str,
    baseline_tokens: &[T],
    current_tokens: &[T],
    options: &BuildSidBlocksOptions,
) -> Vec<T> {
    let baseline_blocks = build_sid_blocks(baseline_tokens, options);
    let current_blocks = build_sid_blocks(current_tokens, options);

    let baseline_by_id = baseline_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), block))
        .collect::<HashMap<_, _>>();
    let current_by_id = current_blocks
        .iter()
        .map(|block| (block.block_id.as_str(), block))
        .collect::<HashMap<_, _>>();

    let baseline = baseline_by_id
        .get(diff_block_id)
        .copied()
        .or_else(|| infer_sid_match_block(diff_block_id, &baseline_blocks));
    let current = current_by_id
        .get(diff_block_id)
        .copied()
        .or_else(|| infer_sid_match_block(diff_block_id, &current_blocks));

    let mut next = current_tokens.to_vec();

    match (baseline, current) {
        (None, Some(current_block)) => {
            next.drain(current_block.start..current_block.end_exclusive);
            next
        }
        (Some(baseline_block), None) => {
            let baseline_slice = baseline_tokens[baseline_block.start..baseline_block.end_exclusive].to_vec();
            let insertion_index = find_insertion_index(baseline_block, &baseline_by_id, &current_by_id);
            next.splice(insertion_index..insertion_index, baseline_slice);
            next
        }
        (Some(baseline_block), Some(current_block)) => {
            let baseline_slice = baseline_tokens[baseline_block.start..baseline_block.end_exclusive].to_vec();
            next.splice(current_block.start..current_block.end_exclusive, baseline_slice);
            next
        }
        (None, None) => next,
    }
}

pub fn apply_reverts_by_block_id<T: DiffableToken>(
    diff_block_ids: &[String],
    baseline_tokens: &[T],
    current_tokens: &[T],
    options: &BuildSidBlocksOptions,
) -> Vec<T> {
    let mut next = current_tokens.to_vec();

    for diff_block_id in diff_block_ids {
        next = apply_revert_by_block_id(diff_block_id, baseline_tokens, &next, options);
    }

    next
}

pub fn replace_chapter_diffs_in_map<TDiff: Clone>(
    previous_map: &DiffsByChapterMap<TDiff>,
    book_code: &str,
    chapter_num: u32,
    chapter_diffs: Vec<TDiff>,
) -> DiffsByChapterMap<TDiff> {
    let mut next = previous_map.clone();
    let mut book = next.remove(book_code).unwrap_or_default();

    if chapter_diffs.is_empty() {
        book.remove(&chapter_num);
    } else {
        book.insert(chapter_num, chapter_diffs);
    }

    if !book.is_empty() {
        next.insert(book_code.to_string(), book);
    }

    next
}

pub fn replace_many_chapter_diffs_in_map<TDiff: Clone>(
    previous_map: &DiffsByChapterMap<TDiff>,
    chapter_diffs: &[(String, u32, Vec<TDiff>)],
) -> DiffsByChapterMap<TDiff> {
    let mut next = previous_map.clone();
    for (book_code, chapter_num, diffs) in chapter_diffs {
        next = replace_chapter_diffs_in_map(&next, book_code, *chapter_num, diffs.clone());
    }
    next
}

pub fn flatten_diff_map<TDiff: Clone>(diffs_by_chapter: &DiffsByChapterMap<TDiff>) -> Vec<TDiff> {
    let mut out = Vec::new();

    for chapters in diffs_by_chapter.values() {
        for diffs in chapters.values() {
            out.extend(diffs.iter().cloned());
        }
    }

    out
}

#[derive(Debug)]
struct SequenceChange {
    tag: DiffStatus,
    ids: Vec<String>,
}

fn token_included<T: DiffableToken>(token: &T, options: &BuildSidBlocksOptions) -> bool {
    options.allow_empty_sid || !normalized_sid(token).is_empty()
}

fn normalized_sid<T: DiffableToken>(token: &T) -> String {
    token.sid_string().unwrap_or_default()
}

fn next_block_start<T: DiffableToken>(
    tokens: &[T],
    start: usize,
    options: &BuildSidBlocksOptions,
) -> Option<usize> {
    (start..tokens.len()).find(|index| token_included(&tokens[*index], options))
}

fn diff_id_sequences(original_ids: &[String], current_ids: &[String]) -> Vec<SequenceChange> {
    diff_sequences(original_ids, current_ids)
}

fn diff_sequences<T>(original_keys: &[T], current_keys: &[T]) -> Vec<SequenceChange>
where
    T: Clone + Eq + Hash + Ord + AsRef<str>,
{
    if original_keys.is_empty() && current_keys.is_empty() {
        return Vec::new();
    }

    if original_keys == current_keys {
        return vec![SequenceChange {
            tag: DiffStatus::Unchanged,
            ids: original_keys
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect(),
        }];
    }

    if original_keys.is_empty() {
        return vec![SequenceChange {
            tag: DiffStatus::Added,
            ids: current_keys
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect(),
        }];
    }

    if current_keys.is_empty() {
        return vec![SequenceChange {
            tag: DiffStatus::Deleted,
            ids: original_keys
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect(),
        }];
    }

    let diff = capture_diff_slices(Algorithm::Myers, original_keys, current_keys);
    let mut out = Vec::<SequenceChange>::new();

    for op in diff {
        for (tag, slice) in op.iter_slices(original_keys, current_keys) {
            let tag = match tag {
                ChangeTag::Delete => DiffStatus::Deleted,
                ChangeTag::Insert => DiffStatus::Added,
                ChangeTag::Equal => DiffStatus::Unchanged,
            };
            let ids = slice
                .iter()
                .map(|value| value.as_ref().to_owned())
                .collect::<Vec<_>>();
            if ids.is_empty() {
                continue;
            }

            if let Some(last) = out.last_mut() && last.tag == tag {
                last.ids.extend(ids);
                continue;
            }

            out.push(SequenceChange { tag, ids });
        }
    }

    out
}

fn build_modified_diff(
    block_id: String,
    semantic_sid: String,
    original: Option<SidBlock>,
    current: Option<SidBlock>,
    original_text: String,
    current_text: String,
) -> SidBlockDiff {
    let original_normalized = normalize_block_text(&original_text);
    let current_normalized = normalize_block_text(&current_text);
    build_modified_diff_with_normalized(
        block_id,
        semantic_sid,
        original,
        current,
        original_text,
        current_text,
        &original_normalized,
        &current_normalized,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_modified_diff_with_normalized(
    block_id: String,
    semantic_sid: String,
    original: Option<SidBlock>,
    current: Option<SidBlock>,
    original_text: String,
    current_text: String,
    original_normalized: &NormalizedBlockText,
    current_normalized: &NormalizedBlockText,
) -> SidBlockDiff {
    let is_whitespace_change =
        original_normalized.full_without_whitespace == current_normalized.full_without_whitespace;
    let is_usfm_structure_change = !is_whitespace_change
        && original_normalized.text_only_without_whitespace
            == current_normalized.text_only_without_whitespace;
    let is_unchanged = original_text == current_text;

    SidBlockDiff {
        block_id,
        semantic_sid,
        status: if is_unchanged {
            DiffStatus::Unchanged
        } else {
            DiffStatus::Modified
        },
        original,
        current,
        original_text,
        current_text,
        original_text_only: original_normalized.text_only.clone(),
        current_text_only: current_normalized.text_only.clone(),
        is_whitespace_change: if is_unchanged { false } else { is_whitespace_change },
        is_usfm_structure_change: if is_unchanged { false } else { is_usfm_structure_change },
    }
}

fn normalize_block_text(value: &str) -> NormalizedBlockText {
    let text_only = strip_usfm_markers_for_display(value);
    NormalizedBlockText {
        text_only_without_whitespace: strip_all_whitespace(&text_only),
        full_without_whitespace: strip_all_whitespace(value),
        text_only,
    }
}

fn strip_all_whitespace(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn strip_usfm_markers_for_display(value: &str) -> String {
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

    collapse_horizontal_whitespace(&out).trim().to_string()
}

fn collapse_horizontal_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_space = false;

    for ch in value.chars() {
        if ch == ' ' || ch == '\t' {
            pending_space = true;
            continue;
        }

        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(ch);
    }

    out
}

fn coalesce_delete_add_pairs(diffs: Vec<SidBlockDiff>) -> Vec<SidBlockDiff> {
    let mut deleted_by_sid = HashMap::<String, Vec<usize>>::new();
    let mut added_by_sid = HashMap::<String, Vec<usize>>::new();

    for (index, diff) in diffs.iter().enumerate() {
        match diff.status {
            DiffStatus::Deleted => {
                deleted_by_sid
                    .entry(diff.semantic_sid.clone())
                    .or_default()
                    .push(index);
            }
            DiffStatus::Added => {
                added_by_sid
                    .entry(diff.semantic_sid.clone())
                    .or_default()
                    .push(index);
            }
            DiffStatus::Modified | DiffStatus::Unchanged => {}
        }
    }

    let mut replacements = HashMap::<usize, SidBlockDiff>::new();
    let mut skip = HashSet::<usize>::new();

    for (sid, deleted_indexes) in deleted_by_sid {
        let added_indexes = added_by_sid.get(&sid).cloned().unwrap_or_default();
        let pair_count = deleted_indexes.len().min(added_indexes.len());

        for pair in 0..pair_count {
            let deleted_index = deleted_indexes[pair];
            let added_index = added_indexes[pair];
            let Some(deleted) = diffs.get(deleted_index) else {
                continue;
            };
            let Some(added) = diffs.get(added_index) else {
                continue;
            };
            let (Some(original), Some(current)) = (deleted.original.clone(), added.current.clone())
            else {
                continue;
            };

            replacements.insert(
                deleted_index,
                build_modified_diff(
                    deleted.block_id.clone(),
                    deleted.semantic_sid.clone(),
                    Some(original),
                    Some(current),
                    deleted.original_text.clone(),
                    added.current_text.clone(),
                ),
            );
            skip.insert(added_index);
        }
    }

    let mut out = Vec::with_capacity(diffs.len());
    for (index, diff) in diffs.into_iter().enumerate() {
        if skip.contains(&index) {
            continue;
        }
        out.push(replacements.remove(&index).unwrap_or(diff));
    }
    out
}

fn infer_sid_match_block<'a>(diff_block_id: &str, blocks: &'a [SidBlock]) -> Option<&'a SidBlock> {
    let sid = extract_sid_from_block_id(diff_block_id)?;
    blocks.iter().find(|block| block.semantic_sid == sid)
}

fn extract_sid_from_block_id(block_id: &str) -> Option<&str> {
    if block_id.is_empty() {
        return None;
    }
    if let Some(index) = block_id.find("::") {
        return Some(&block_id[..index]);
    }
    if let Some(index) = block_id.rfind('#') {
        return Some(&block_id[..index]);
    }
    Some(block_id)
}

fn find_insertion_index(
    baseline_block: &SidBlock,
    baseline_by_id: &HashMap<&str, &SidBlock>,
    current_by_id: &HashMap<&str, &SidBlock>,
) -> usize {
    let mut anchor_id = baseline_block.prev_block_id.as_deref();
    while let Some(anchor) = anchor_id {
        if let Some(anchor_block) = current_by_id.get(anchor) {
            return anchor_block.end_exclusive;
        }
        anchor_id = baseline_by_id
            .get(anchor)
            .and_then(|block| block.prev_block_id.as_deref());
    }
    0
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
mod tests {
    use super::*;

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

    fn tk(
        sid: &str,
        id: Option<&str>,
        kind_key: Option<&str>,
        marker_key: Option<&str>,
        text: &str,
    ) -> TestToken {
        TestToken {
            sid: Some(sid.to_string()),
            text: text.to_string(),
            id: id.map(str::to_string),
            kind_key: kind_key.map(str::to_string),
            marker_key: marker_key.map(str::to_string),
        }
    }

    #[test]
    fn groups_contiguous_runs_of_same_sid() {
        let tokens = vec![
            t("GEN 1:1", "A", Some("1")),
            t("GEN 1:1", "B", Some("2")),
            t("GEN 1:2", "C", Some("3")),
            t("GEN 1:2", "D", Some("4")),
        ];

        let blocks = build_sid_blocks(&tokens, &BuildSidBlocksOptions::default());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].semantic_sid, "GEN 1:1");
        assert_eq!(blocks[0].text_full, "AB");
        assert_eq!(blocks[1].semantic_sid, "GEN 1:2");
        assert_eq!(blocks[1].text_full, "CD");
    }

    #[test]
    fn uses_first_token_id_for_stable_block_ids() {
        let tokens = vec![
            t("GEN 1:1", "A", Some("tok-aaa")),
            t("GEN 1:1", "B", Some("tok-bbb")),
        ];
        let blocks = build_sid_blocks(&tokens, &BuildSidBlocksOptions::default());
        assert_eq!(blocks[0].block_id, "GEN 1:1::tok-aaa");
    }

    #[test]
    fn coalesces_same_sid_id_drift_into_modified_and_can_revert() {
        let baseline = vec![t("ISA 33:9", "Alpha", Some("orig-id"))];
        let current = vec![t("ISA 33:9", "Beta", Some("new-id"))];

        let diffs =
            diff_chapter_token_streams(&baseline, &current, &BuildSidBlocksOptions::default());

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status, DiffStatus::Modified);
        assert_eq!(diffs[0].semantic_sid, "ISA 33:9");

        let reverted = apply_revert_by_block_id(
            &diffs[0].block_id,
            &baseline,
            &current,
            &BuildSidBlocksOptions::default(),
        );
        assert_eq!(reverted[0].text, "Alpha");
    }

    #[test]
    fn detects_whitespace_only_changes() {
        let baseline = vec![t("GEN 1:1", "Alpha  Beta", Some("a"))];
        let current = vec![t("GEN 1:1", "Alpha Beta", Some("a"))];
        let diff =
            diff_chapter_token_streams(&baseline, &current, &BuildSidBlocksOptions::default())
                .remove(0);

        assert_eq!(diff.status, DiffStatus::Modified);
        assert!(diff.is_whitespace_change);
        assert!(!diff.is_usfm_structure_change);
    }

    #[test]
    fn emits_token_alignment_metadata_for_marker_number_and_text() {
        let baseline = vec![
            tk("REV 19:4", Some("m-v"), Some("marker"), Some("v"), "\\v "),
            tk("REV 19:4", Some("n-v"), Some("number"), None, " 4"),
            tk("REV 19:4", Some("t-v"), Some("text"), None, " old"),
        ];
        let current = vec![
            tk("REV 19:4", Some("m-v"), Some("marker"), Some("v"), "\\v"),
            tk("REV 19:4", Some("n-v"), Some("number"), None, "4"),
            tk("REV 19:4", Some("t-v"), Some("text"), None, " new"),
        ];

        let diff =
            diff_chapter_token_streams(&baseline, &current, &BuildSidBlocksOptions::default())
                .remove(0);

        assert_eq!(diff.status, DiffStatus::Modified);
        assert_eq!(diff.undo_side, DiffUndoSide::Current);
        assert_eq!(
            diff.original_alignment
                .iter()
                .map(|entry| entry.change)
                .collect::<Vec<_>>(),
            vec![
                DiffTokenChange::Unchanged,
                DiffTokenChange::Unchanged,
                DiffTokenChange::Modified,
            ]
        );
        assert_eq!(
            diff.current_alignment
                .iter()
                .map(|entry| entry.change)
                .collect::<Vec<_>>(),
            vec![
                DiffTokenChange::Unchanged,
                DiffTokenChange::Unchanged,
                DiffTokenChange::Modified,
            ]
        );
    }

    #[test]
    fn current_parse_tokens_diffable_sid_and_id_shape_match_old_strings() {
        let parsed = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha\n");
        let verse_token = parsed
            .tokens
            .iter()
            .find(|token| token.kind() == TokenKind::Number && token.sid_string() == Some("GEN 1:1".to_string()))
            .expect("verse token should exist");

        assert_eq!(verse_token.sid_string().as_deref(), Some("GEN 1:1"));
        assert!(verse_token.id_string().unwrap().starts_with("GEN-"));
    }

    #[test]
    fn source_diff_by_chapter_groups_overlap_and_missing_chapters() {
        let baseline = "\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha\n\\c 2\n\\p\n\\v 1 Beta\n";
        let current = "\\id GEN\n\\c 1\n\\p\n\\v 1 Alpha edited\n\\c 3\n\\p\n\\v 1 Gamma\n";
        let diffs = diff_usfm_sources_by_chapter(
            baseline,
            current,
            &BuildSidBlocksOptions::default(),
        );

        let gen_chapters = diffs.get("GEN").unwrap();
        assert!(gen_chapters.contains_key(&1));
        assert!(gen_chapters.contains_key(&2));
        assert!(gen_chapters.contains_key(&3));
    }
}
