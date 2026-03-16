use crate::marker_defs::{NoteSubkind, lookup_marker_def, marker_note_family};
use crate::markers::{MarkerKind, lookup_marker};
use crate::token::{Token, TokenData, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExportDocument<'a> {
    pub tokens: &'a [Token<'a>],
    pub children: Vec<ExportNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExportNode {
    Container(ExportContainerNode),
    Chapter {
        marker_index: usize,
        number_index: Option<usize>,
    },
    Verse {
        marker_index: usize,
        number_index: Option<usize>,
    },
    Milestone {
        marker_index: usize,
        attribute_index: Option<usize>,
        end_index: Option<usize>,
        closed: bool,
    },
    Leaf {
        token_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExportContainerNode {
    pub kind: ExportContainerKind,
    pub token_index: usize,
    pub close_index: Option<usize>,
    pub attribute_index: Option<usize>,
    pub children: Vec<ExportNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportContainerKind {
    Paragraph,
    Character,
    Note,
    Figure,
    Sidebar,
    Periph,
    TableRow,
    TableCell,
    Header,
    Meta,
    Unknown,
}

#[derive(Debug, Clone)]
struct OpenMarker {
    marker_name: String,
    kind: MarkerKind,
    token_index: usize,
    close_index: Option<usize>,
    attribute_index: Option<usize>,
    children: Vec<ExportNode>,
}

#[derive(Debug, Clone, Copy)]
struct PendingMilestone {
    token_index: usize,
    attribute_index: Option<usize>,
}

#[derive(Debug, Default)]
struct BuilderState {
    stack: Vec<OpenMarker>,
    root_children: Vec<ExportNode>,
    pending_chapter: Option<usize>,
    pending_verse: Option<usize>,
    pending_milestone: Option<PendingMilestone>,
    pending_empty_para_before_verse: bool,
}

impl BuilderState {
    fn append_node(&mut self, node: ExportNode) {
        if let Some(top) = self.stack.last_mut() {
            top.children.push(node);
        } else {
            self.root_children.push(node);
        }
    }

    fn append_leaf(&mut self, token_index: usize) {
        self.append_node(ExportNode::Leaf { token_index });
    }

    fn append_unmatched_marker(&mut self, token_index: usize) {
        self.append_node(ExportNode::Container(ExportContainerNode {
            kind: ExportContainerKind::Unknown,
            token_index,
            close_index: None,
            attribute_index: None,
            children: Vec::new(),
        }));
    }

    fn push_open(&mut self, token_index: usize, kind: MarkerKind) {
        self.stack.push(OpenMarker {
            marker_name: String::new(),
            kind,
            token_index,
            close_index: None,
            attribute_index: None,
            children: Vec::new(),
        });
    }

    fn pop_open(&mut self) -> Option<OpenMarker> {
        self.stack.pop()
    }

    fn append_finalized(&mut self, open: OpenMarker) {
        self.append_node(ExportNode::Container(finalize_open_marker(open)));
    }

    fn close_and_append_top(&mut self) {
        if let Some(open) = self.pop_open() {
            self.append_finalized(open);
        }
    }

    fn flush_pending_chapter(&mut self) {
        if let Some(marker_index) = self.pending_chapter.take() {
            self.append_node(ExportNode::Chapter {
                marker_index,
                number_index: None,
            });
        }
    }

    fn flush_pending_verse(&mut self) {
        if let Some(marker_index) = self.pending_verse.take() {
            self.append_node(ExportNode::Verse {
                marker_index,
                number_index: None,
            });
        }
    }

    fn flush_pending_numbers(&mut self) {
        self.flush_pending_chapter();
        self.flush_pending_verse();
    }

    fn close_pending_milestone(&mut self, closed: bool, end_index: Option<usize>) {
        if let Some(milestone) = self.pending_milestone.take() {
            self.append_node(ExportNode::Milestone {
                marker_index: milestone.token_index,
                attribute_index: milestone.attribute_index,
                end_index,
                closed,
            });
        }
    }
}

pub(crate) fn build_export_document<'a>(tokens: &'a [Token<'a>]) -> ExportDocument<'a> {
    let mut state = BuilderState::default();

    for (index, token) in tokens.iter().enumerate() {
        if state.pending_milestone.is_some()
            && !matches!(token.kind(), TokenKind::AttributeList | TokenKind::MilestoneEnd)
        {
            state.close_pending_milestone(false, None);
        }

        if (state.pending_chapter.is_some() || state.pending_verse.is_some())
            && token.kind() != TokenKind::Number
        {
            state.flush_pending_numbers();
        }

        match &token.data {
            TokenData::Marker { .. } => handle_open(index, token, &mut state, tokens),
            TokenData::EndMarker { name, .. } => close_matching_marker(name, index, &mut state),
            TokenData::Milestone { .. } => handle_open(index, token, &mut state, tokens),
            TokenData::MilestoneEnd => {
                if state.pending_milestone.is_some() {
                    state.close_pending_milestone(true, Some(index));
                } else {
                    state.append_unmatched_marker(index);
                }
            }
            TokenData::Text
            | TokenData::Newline
            | TokenData::OptBreak
            | TokenData::BookCode { .. } => {
                state.append_leaf(index);
            }
            TokenData::Number { .. } => handle_number(index, &mut state),
            TokenData::AttributeList { .. } => handle_attributes(index, &mut state),
        }
    }

    state.flush_pending_numbers();
    state.close_pending_milestone(false, None);
    finish_open_markers(&mut state);

    ExportDocument {
        tokens,
        children: state.root_children,
    }
}

fn handle_open(index: usize, token: &Token<'_>, state: &mut BuilderState, tokens: &[Token<'_>]) {
    let Some(marker) = token.marker_name() else {
        return;
    };

    if matches!(token.data, TokenData::Milestone { .. }) {
        state.pending_milestone = Some(PendingMilestone {
            token_index: index,
            attribute_index: None,
        });
        return;
    }

    let info = lookup_marker(marker);
    if info.kind != MarkerKind::Verse {
        state.pending_empty_para_before_verse = false;
    }

    match info.kind {
        MarkerKind::Chapter => {
            state.flush_pending_numbers();
            force_close_notes(state);
            close_paragraph(state);
            state.pending_chapter = Some(index);
        }
        MarkerKind::Verse => {
            if state.pending_empty_para_before_verse {
                state.push_open(index, MarkerKind::Paragraph);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = String::new();
                }
                state.pending_empty_para_before_verse = false;
            }
            state.flush_pending_verse();
            close_open_meta(state);
            state.pending_verse = Some(index);
        }
        MarkerKind::Paragraph | MarkerKind::Header => {
            force_close_notes(state);
            close_paragraph(state);
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::Meta => {
            if marker == "cat" && in_note_or_sidebar_context(&state.stack) {
                state.push_open(index, info.kind);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = marker.to_string();
                }
            } else if marker == "rem" && !in_note_context(&state.stack) && has_open_paragraph(&state.stack) {
                close_inline_above_paragraph(state);
                state.push_open(index, info.kind);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = marker.to_string();
                }
            } else {
                force_close_notes(state);
                close_paragraph(state);
                state.push_open(index, info.kind);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = marker.to_string();
                }
            }
        }
        MarkerKind::Note => {
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::Character => {
            let nested = matches!(token.data, TokenData::Marker { nested: true, .. });
            if in_note_context(&state.stack)
                && !nested
                && should_close_current_note_char(&state.stack, marker, info.valid_in_note, tokens)
                && marker != "fv"
            {
                close_character_in_note(state);
            }
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::Figure | MarkerKind::Periph => {
            if info.kind == MarkerKind::Periph {
                force_close_notes(state);
                close_paragraph(state);
            }
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::TableRow => {
            force_close_notes(state);
            close_paragraph(state);
            close_table_cell_in_row(state);
            close_table_row(state);
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::TableCell => {
            close_table_cell_in_row(state);
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::SidebarStart => {
            force_close_notes(state);
            close_paragraph(state);
            state.push_open(index, info.kind);
            if let Some(top) = state.stack.last_mut() {
                top.marker_name = marker.to_string();
            }
        }
        MarkerKind::SidebarEnd => close_sidebar(index, state),
        MarkerKind::MilestoneStart | MarkerKind::MilestoneEnd => {}
        MarkerKind::Unknown => {
            state.pending_empty_para_before_verse = false;
            if unknown_marker_starts_new_block(state, tokens) {
                force_close_notes(state);
                close_paragraph(state);
                state.push_open(index, MarkerKind::Paragraph);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = marker.to_string();
                }
            } else {
                state.push_open(index, info.kind);
                if let Some(top) = state.stack.last_mut() {
                    top.marker_name = marker.to_string();
                }
            }
        }
    }
}

fn handle_number(index: usize, state: &mut BuilderState) {
    if let Some(marker_index) = state.pending_chapter.take() {
        state.append_node(ExportNode::Chapter {
            marker_index,
            number_index: Some(index),
        });
        return;
    }

    if let Some(marker_index) = state.pending_verse.take() {
        state.append_node(ExportNode::Verse {
            marker_index,
            number_index: Some(index),
        });
        return;
    }

    state.append_leaf(index);
}

fn handle_attributes(index: usize, state: &mut BuilderState) {
    if let Some(milestone) = state.pending_milestone.as_mut() {
        milestone.attribute_index = Some(index);
        return;
    }

    if let Some(open) = state.stack.iter_mut().rev().find(|open| {
        matches!(
            open.kind,
            MarkerKind::Character | MarkerKind::Figure | MarkerKind::Periph
        )
    }) {
        open.attribute_index = Some(index);
        return;
    }

    state.append_leaf(index);
}

fn finalize_open_marker(open: OpenMarker) -> ExportContainerNode {
    ExportContainerNode {
        kind: container_kind_from_marker_kind(open.kind, open.token_index),
        token_index: open.token_index,
        close_index: open.close_index,
        attribute_index: open.attribute_index,
        children: open.children,
    }
}

fn container_kind_from_marker_kind(kind: MarkerKind, _token_index: usize) -> ExportContainerKind {
    match kind {
        MarkerKind::Paragraph => ExportContainerKind::Paragraph,
        MarkerKind::Character => ExportContainerKind::Character,
        MarkerKind::Note => ExportContainerKind::Note,
        MarkerKind::Figure => ExportContainerKind::Figure,
        MarkerKind::SidebarStart => ExportContainerKind::Sidebar,
        MarkerKind::Periph => ExportContainerKind::Periph,
        MarkerKind::TableRow => ExportContainerKind::TableRow,
        MarkerKind::TableCell => ExportContainerKind::TableCell,
        MarkerKind::Header => ExportContainerKind::Header,
        MarkerKind::Meta => ExportContainerKind::Meta,
        MarkerKind::Unknown => ExportContainerKind::Unknown,
        _ => ExportContainerKind::Unknown,
    }
}

fn close_matching_marker(close_marker: &str, token_index: usize, state: &mut BuilderState) {
    let is_note_close = matches!(close_marker, "f" | "fe" | "x" | "ef" | "ex");
    let match_idx = state.stack.iter().rposition(|open| {
        let open_name = marker_name_from_open(open);
        if is_note_close {
            open.kind == MarkerKind::Note && open_name == close_marker
        } else {
            open_name == close_marker
        }
    });

    match match_idx {
        Some(idx) => {
            while state.stack.len() > idx + 1 {
                let top = state.pop_open().expect("stack length checked");
                state.append_finalized(top);
            }
            if let Some(open) = state.stack.last_mut() {
                open.close_index = Some(token_index);
            }
            state.close_and_append_top();
        }
        None => state.append_unmatched_marker(token_index),
    }
}

fn marker_name_from_open(open: &OpenMarker) -> &str {
    open.marker_name.as_str()
}

fn close_paragraph(state: &mut BuilderState) {
    loop {
        match state.stack.last().map(|open| open.kind) {
            Some(MarkerKind::Character)
            | Some(MarkerKind::Unknown)
            | Some(MarkerKind::Figure)
            | Some(MarkerKind::TableCell) => {
                let open = state.pop_open().expect("stack last checked");
                state.append_finalized(open);
            }
            Some(MarkerKind::Paragraph)
            | Some(MarkerKind::Header)
            | Some(MarkerKind::Meta)
            | Some(MarkerKind::TableRow) => {
                state.close_and_append_top();
                break;
            }
            _ => break,
        }
    }
}

fn force_close_notes(state: &mut BuilderState) {
    loop {
        let note_idx = state
            .stack
            .iter()
            .rposition(|open| open.kind == MarkerKind::Note);
        match note_idx {
            Some(idx) => {
                while state.stack.len() > idx + 1 {
                    state.close_and_append_top();
                }
                let note = state.pop_open().expect("note index existed");
                state.append_finalized(note);
            }
            None => break,
        }
    }
}

fn close_sidebar(token_index: usize, state: &mut BuilderState) {
    let sidebar_idx = state
        .stack
        .iter()
        .rposition(|open| open.kind == MarkerKind::SidebarStart);
    match sidebar_idx {
        Some(idx) => {
            while state.stack.len() > idx + 1 {
                let top = state.pop_open().expect("stack length checked");
                state.append_finalized(top);
            }
            if let Some(open) = state.stack.last_mut() {
                open.close_index = Some(token_index);
            }
            state.close_and_append_top();
        }
        None => {
            close_paragraph(state);
            state.append_unmatched_marker(token_index);
        }
    }
}

fn close_table_cell_in_row(state: &mut BuilderState) {
    while state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::TableCell)
    {
        state.close_and_append_top();
    }
}

fn close_table_row(state: &mut BuilderState) {
    if state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::TableRow)
    {
        state.close_and_append_top();
    }
}

fn close_open_meta(state: &mut BuilderState) {
    while state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::Meta)
    {
        state.close_and_append_top();
    }
}

fn close_character_in_note(state: &mut BuilderState) {
    while let Some(MarkerKind::Character)
    | Some(MarkerKind::Unknown)
    | Some(MarkerKind::Figure)
    | Some(MarkerKind::TableCell) = state.stack.last().map(|open| open.kind)
    {
        state.close_and_append_top();
    }
}

fn in_note_context(stack: &[OpenMarker]) -> bool {
    stack.iter().rev().any(|open| open.kind == MarkerKind::Note)
}

fn in_note_or_sidebar_context(stack: &[OpenMarker]) -> bool {
    stack
        .iter()
        .rev()
        .any(|open| matches!(open.kind, MarkerKind::Note | MarkerKind::SidebarStart))
}

fn has_open_paragraph(stack: &[OpenMarker]) -> bool {
    stack
        .iter()
        .rev()
        .any(|open| matches!(open.kind, MarkerKind::Paragraph | MarkerKind::TableRow))
}

fn unknown_marker_starts_new_block(state: &BuilderState, tokens: &[Token<'_>]) -> bool {
    state
        .stack
        .iter()
        .rev()
        .find(|open| {
            matches!(
                open.kind,
                MarkerKind::Paragraph | MarkerKind::Header | MarkerKind::Meta
            )
        })
        .is_some_and(|open| {
            open.children
                .iter()
                .rev()
                .find_map(|node| match node {
                    ExportNode::Leaf { token_index } => match tokens[*token_index].kind() {
                        TokenKind::Newline => Some(true),
                        TokenKind::Text if tokens[*token_index].source.trim().is_empty() => None,
                        _ => Some(false),
                    },
                    _ => Some(false),
                })
                .unwrap_or(false)
        })
}

fn close_inline_above_paragraph(state: &mut BuilderState) {
    while let Some(MarkerKind::Character)
    | Some(MarkerKind::Figure)
    | Some(MarkerKind::Unknown)
    | Some(MarkerKind::TableCell) = state.stack.last().map(|open| open.kind)
    {
        state.close_and_append_top();
    }
}

fn is_same_note_family(stack: &[OpenMarker], incoming_marker: &str, tokens: &[Token<'_>]) -> bool {
    let note_family = stack
        .iter()
        .rev()
        .find(|open| open.kind == MarkerKind::Note)
        .and_then(|open| tokens[open.token_index].marker_name())
        .and_then(marker_note_family);
    let incoming_family = marker_note_family(incoming_marker);
    matches!((note_family, incoming_family), (Some(left), Some(right)) if left == right)
}

fn should_close_current_note_char(
    stack: &[OpenMarker],
    incoming_marker: &str,
    incoming_valid_in_note: bool,
    tokens: &[Token<'_>],
) -> bool {
    let Some(current_char) = stack
        .iter()
        .rev()
        .find(|open| open.kind == MarkerKind::Character)
    else {
        return false;
    };

    let Some(current_name) = tokens[current_char.token_index].marker_name() else {
        return false;
    };
    let current_info = lookup_marker(current_name);

    if incoming_valid_in_note && is_same_note_family(stack, incoming_marker, tokens) {
        return true;
    }

    if lookup_marker_def(current_name).and_then(|def| def.note_subkind)
        == Some(NoteSubkind::StructuralKeepsNestedCharsOpen)
    {
        return false;
    }

    current_info.valid_in_note && !matches!(incoming_marker, "ref" | "jmp")
}

fn finish_open_markers(state: &mut BuilderState) {
    while let Some(open) = state.pop_open() {
        state.append_finalized(open);
    }
}
