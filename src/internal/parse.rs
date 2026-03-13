use crate::internal::lexer::lex;
use crate::internal::marker_defs::{
    MARKER_JMP, MARKER_REF, NoteSubkind, lookup_marker_def, lookup_marker_id, marker_note_family,
};
use crate::internal::markers::{MarkerKind, lookup_marker};
use crate::internal::recovery::{ParseRecovery, RecoveryCode};
use crate::internal::syntax::{ContainerKind, ContainerNode, Document, LeafKind, Node};
use crate::model::token::{ScanToken, ScanTokenKind, normalized_marker_name, number_prefix_len};
use crate::parse::handle::{ParseAnalysis, ParseHandle};

#[derive(Debug, Clone)]
struct OpenMarker {
    marker: String,
    kind: MarkerKind,
    span: std::ops::Range<usize>,
    close_span: Option<std::ops::Range<usize>>,
    children: Vec<Node>,
    special_span: Option<std::ops::Range<usize>>,
    attribute_spans: Vec<std::ops::Range<usize>>,
}

#[derive(Debug, Clone)]
struct PendingMilestone {
    marker: String,
    marker_span: std::ops::Range<usize>,
    attribute_spans: Vec<std::ops::Range<usize>>,
}

#[derive(Debug, Default)]
struct ParserState {
    stack: Vec<OpenMarker>,
    root_children: Vec<Node>,
    analysis: ParseAnalysis,
    pending_chapter: Option<std::ops::Range<usize>>,
    pending_verse: Option<std::ops::Range<usize>>,
    pending_milestone: Option<PendingMilestone>,
    pending_empty_para_before_verse: bool,
}

impl ParserState {
    fn append_node(&mut self, node: Node) {
        if let Some(top) = self.stack.last_mut() {
            top.children.push(node);
        } else {
            self.root_children.push(node);
        }
    }

    fn append_leaf(&mut self, kind: LeafKind, span: std::ops::Range<usize>) {
        self.append_node(Node::Leaf { kind, span });
    }

    fn append_unmatched_marker(&mut self, marker: &str, span: std::ops::Range<usize>) {
        self.append_node(Node::Container(ContainerNode {
            kind: ContainerKind::Unknown,
            marker: marker.to_string(),
            marker_span: span,
            close_span: None,
            special_span: None,
            attribute_spans: Vec::new(),
            children: Vec::new(),
        }));
    }

    fn push_open(&mut self, marker: &str, kind: MarkerKind, span: std::ops::Range<usize>) {
        self.stack.push(OpenMarker {
            marker: marker.to_string(),
            kind,
            span,
            close_span: None,
            children: Vec::new(),
            special_span: None,
            attribute_spans: Vec::new(),
        });
    }

    fn pop_open(&mut self) -> Option<OpenMarker> {
        self.stack.pop()
    }

    fn append_finalized(&mut self, open: OpenMarker) {
        self.append_node(finalize_open_marker(open));
    }

    fn close_and_append_top(&mut self) {
        if let Some(open) = self.pop_open() {
            self.append_finalized(open);
        }
    }

    fn flush_pending_chapter(&mut self) {
        if let Some(span) = self.pending_chapter.take() {
            self.analysis.recoveries.push(ParseRecovery::marker(
                RecoveryCode::MissingChapterNumber,
                span.clone(),
                "c",
            ));
            self.append_node(Node::Chapter {
                marker_span: span,
                number_span: None,
            });
        }
    }

    fn flush_pending_verse(&mut self) {
        if let Some(span) = self.pending_verse.take() {
            self.analysis.recoveries.push(ParseRecovery::marker(
                RecoveryCode::MissingVerseNumber,
                span.clone(),
                "v",
            ));
            self.append_node(Node::Verse {
                marker_span: span,
                number_span: None,
            });
        }
    }

    fn flush_pending_numbers(&mut self) {
        self.flush_pending_chapter();
        self.flush_pending_verse();
    }

    fn close_pending_milestone(
        &mut self,
        closed: bool,
        close_span: Option<std::ops::Range<usize>>,
    ) {
        if let Some(milestone) = self.pending_milestone.take() {
            if !closed {
                self.analysis.recoveries.push(ParseRecovery::marker(
                    RecoveryCode::MissingMilestoneSelfClose,
                    milestone.marker_span.clone(),
                    milestone.marker.clone(),
                ));
            }
            self.append_node(Node::Milestone {
                marker: milestone.marker,
                marker_span: milestone.marker_span,
                attribute_spans: milestone.attribute_spans,
                end_span: close_span,
                closed,
            });
        }
    }
}

pub fn parse(source: &str) -> ParseHandle {
    let lexed = lex(source);
    let analysis = collect_analysis(&lexed.tokens);
    ParseHandle::new(source.to_string(), lexed.tokens, analysis)
}

fn collect_analysis(tokens: &[ScanToken]) -> ParseAnalysis {
    let mut state = ParserState::default();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind == ScanTokenKind::Text
            && token.text == "\\"
            && state
                .stack
                .last()
                .is_some_and(|open| lookup_marker(open.marker.as_str()).kind == MarkerKind::Unknown)
            && matches!(
                tokens.get(index + 1),
                Some(next)
                    if matches!(
                        next.kind,
                        ScanTokenKind::Marker
                            | ScanTokenKind::NestedMarker
                            | ScanTokenKind::Milestone
                    )
            )
        {
            close_paragraph(&mut state, &token.span);
            state.pending_empty_para_before_verse = true;
            continue;
        }

        if state.pending_milestone.is_some()
            && !matches!(
                token.kind,
                ScanTokenKind::Whitespace
                    | ScanTokenKind::Newline
                    | ScanTokenKind::Attributes
                    | ScanTokenKind::MilestoneEnd
            )
            && !token_is_milestone_attribute_continuation(token)
        {
            state.close_pending_milestone(false, None);
        }

        if (state.pending_chapter.is_some() || state.pending_verse.is_some())
            && !matches!(
                token.kind,
                ScanTokenKind::Whitespace | ScanTokenKind::Newline | ScanTokenKind::Text
            )
        {
            state.flush_pending_numbers();
        }

        match token.kind {
            ScanTokenKind::Marker | ScanTokenKind::NestedMarker | ScanTokenKind::Milestone => {
                handle_open(index, token, &mut state);
            }
            ScanTokenKind::ClosingMarker | ScanTokenKind::NestedClosingMarker => {
                if let Some(marker) = token.marker_name() {
                    close_matching_marker(normalized_marker_name(marker), token, &mut state);
                }
            }
            ScanTokenKind::MilestoneEnd => {
                if state.pending_milestone.is_some() {
                    state.close_pending_milestone(true, Some(token.span.clone()));
                } else {
                    state.analysis.recoveries.push(ParseRecovery::close(
                        RecoveryCode::StrayCloseMarker,
                        token.span.clone(),
                        None,
                        String::new(),
                        "*".to_string(),
                    ));
                    state.append_unmatched_marker("*", token.span.clone());
                }
            }
            ScanTokenKind::Text => {
                if let Some(milestone) = state.pending_milestone.as_mut()
                    && token_is_milestone_attribute_continuation(token)
                {
                    milestone.attribute_spans.push(token.span.clone());
                } else {
                    handle_text(index, token, &mut state);
                }
            }
            ScanTokenKind::Whitespace => {
                if state.pending_milestone.is_none() {
                    state.append_leaf(LeafKind::Whitespace, token.span.clone());
                }
            }
            ScanTokenKind::Newline => {
                if state.pending_milestone.is_none() {
                    state.append_leaf(LeafKind::Newline, token.span.clone());
                }
            }
            ScanTokenKind::OptBreak => {
                if state.pending_milestone.is_none() {
                    state.append_leaf(LeafKind::OptBreak, token.span.clone());
                }
            }
            ScanTokenKind::Attributes => handle_attributes(token, &mut state),
        }
    }

    state.flush_pending_numbers();
    state.close_pending_milestone(false, None);
    finish_open_markers(&mut state);
    state.analysis.document = Document {
        children: state.root_children,
    };
    state.analysis
}

fn handle_open(index: usize, token: &ScanToken, state: &mut ParserState) {
    let Some(marker) = token.marker_name() else {
        return;
    };
    let marker = normalized_marker_name(marker);
    if token.kind == ScanTokenKind::Milestone {
        state.pending_milestone = Some(PendingMilestone {
            marker: marker.to_string(),
            marker_span: token.span.clone(),
            attribute_spans: Vec::new(),
        });
        let _ = index;
        return;
    }

    let info = lookup_marker(marker);
    if info.kind != MarkerKind::Verse {
        state.pending_empty_para_before_verse = false;
    }

    match info.kind {
        MarkerKind::Chapter => {
            state.flush_pending_numbers();
            force_close_notes(state, &token.span);
            close_paragraph(state, &token.span);
            state.pending_chapter = Some(token.span.clone());
        }
        MarkerKind::Verse => {
            if state.pending_empty_para_before_verse {
                state.push_open(
                    "",
                    MarkerKind::Paragraph,
                    token.span.start..token.span.start,
                );
                state.pending_empty_para_before_verse = false;
            }
            state.flush_pending_verse();
            close_open_meta(state);
            state.pending_verse = Some(token.span.clone());
        }
        MarkerKind::Paragraph | MarkerKind::Header => {
            force_close_notes(state, &token.span);
            close_paragraph(state, &token.span);
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::Meta => {
            if marker == "cat" && in_note_or_sidebar_context(&state.stack) {
                state.push_open(marker, info.kind, token.span.clone());
            } else if marker == "rem"
                && !in_note_context(&state.stack)
                && has_open_paragraph(&state.stack)
            {
                close_inline_above_paragraph(state);
                state.push_open(marker, info.kind, token.span.clone());
            } else {
                force_close_notes(state, &token.span);
                close_paragraph(state, &token.span);
                state.push_open(marker, info.kind, token.span.clone());
            }
        }
        MarkerKind::Note => {
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::Character => {
            if in_note_context(&state.stack)
                && token.kind != ScanTokenKind::NestedMarker
                && should_close_current_note_char(&state.stack, marker, info.valid_in_note)
                && marker != "fv"
            {
                close_character_in_note(state);
            }
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::Figure | MarkerKind::Periph => {
            if info.kind == MarkerKind::Periph {
                force_close_notes(state, &token.span);
                close_paragraph(state, &token.span);
            }
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::TableRow => {
            force_close_notes(state, &token.span);
            close_paragraph(state, &token.span);
            close_table_cell_in_row(state);
            close_table_row(state);
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::TableCell => {
            close_table_cell_in_row(state);
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::SidebarStart => {
            force_close_notes(state, &token.span);
            close_paragraph(state, &token.span);
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::SidebarEnd => {
            close_sidebar(state, &token.span);
        }
        MarkerKind::MilestoneStart | MarkerKind::MilestoneEnd => {
            state.push_open(marker, info.kind, token.span.clone());
        }
        MarkerKind::Unknown => {
            state.pending_empty_para_before_verse = false;
            if unknown_marker_starts_new_block(state) {
                force_close_notes(state, &token.span);
                close_paragraph(state, &token.span);
                state.push_open(marker, MarkerKind::Paragraph, token.span.clone());
            } else {
                state.push_open(marker, info.kind, token.span.clone());
            }
        }
    }

    if marker == "id" {
        state.analysis.book_code_token_index = None;
        state.analysis.book_code_prefix_len = None;
    }
    let _ = index;
}

fn handle_text(index: usize, token: &ScanToken, state: &mut ParserState) {
    if state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::Header && open.marker == "id")
        && let Some((book_code, prefix_len)) = extract_first_word(&token.text)
        && book_code.len() == 3
    {
        if let Some(top) = state.stack.last_mut() {
            top.special_span = Some(token.span.start..token.span.start + prefix_len);
        }
        if state.analysis.book_code.is_none() {
            state.analysis.book_code = Some(book_code.to_string());
            state.analysis.book_code_token_index = Some(index);
            state.analysis.book_code_prefix_len = Some(prefix_len);
        }
        if prefix_len < token.text.len() {
            state.append_leaf(
                LeafKind::Text,
                token.span.start + prefix_len..token.span.end,
            );
        }
        return;
    }

    if state.pending_chapter.is_some() {
        if let Some(prefix_len) = number_prefix_len(&token.text) {
            state.analysis.number_token_indexes.push(index);
            let marker_span = state.pending_chapter.take().expect("checked is_some");
            state.append_node(Node::Chapter {
                marker_span,
                number_span: Some(token.span.start..token.span.start + prefix_len),
            });
            if prefix_len < token.text.len() {
                state.append_leaf(
                    LeafKind::Text,
                    token.span.start + prefix_len..token.span.end,
                );
            }
            return;
        }
        if let Some((_, prefix_len)) = extract_first_word(&token.text) {
            state.analysis.number_token_indexes.push(index);
            let marker_span = state.pending_chapter.take().expect("checked is_some");
            state.append_node(Node::Chapter {
                marker_span,
                number_span: Some(token.span.start..token.span.start + prefix_len),
            });
            if prefix_len < token.text.len() {
                state.append_leaf(
                    LeafKind::Text,
                    token.span.start + prefix_len..token.span.end,
                );
            }
            return;
        }
        state.flush_pending_chapter();
    }

    if state.pending_verse.is_some() {
        if let Some(prefix_len) = number_prefix_len(&token.text) {
            state.analysis.number_token_indexes.push(index);
            let marker_span = state.pending_verse.take().expect("checked is_some");
            state.append_node(Node::Verse {
                marker_span,
                number_span: Some(token.span.start..token.span.start + prefix_len),
            });
            if prefix_len < token.text.len() {
                state.append_leaf(
                    LeafKind::Text,
                    token.span.start + prefix_len..token.span.end,
                );
            }
            return;
        }
        if let Some((_, prefix_len)) = extract_first_word(&token.text) {
            state.analysis.number_token_indexes.push(index);
            let marker_span = state.pending_verse.take().expect("checked is_some");
            state.append_node(Node::Verse {
                marker_span,
                number_span: Some(token.span.start..token.span.start + prefix_len),
            });
            if prefix_len < token.text.len() {
                state.append_leaf(
                    LeafKind::Text,
                    token.span.start + prefix_len..token.span.end,
                );
            }
            return;
        }
        state.flush_pending_verse();
    }

    if state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::Note && open.special_span.is_none())
        && let Some((_, prefix_len)) = extract_first_word(&token.text)
    {
        if let Some(top) = state.stack.last_mut() {
            top.special_span = Some(token.span.start..token.span.start + prefix_len);
        }
        if prefix_len < token.text.len() {
            state.append_leaf(
                LeafKind::Text,
                token.span.start + prefix_len..token.span.end,
            );
        }
        return;
    }

    state.append_leaf(LeafKind::Text, token.span.clone());
}

fn handle_attributes(token: &ScanToken, state: &mut ParserState) {
    if let Some(milestone) = state.pending_milestone.as_mut() {
        milestone.attribute_spans.push(token.span.clone());
        return;
    }

    if let Some(open) = state.stack.iter_mut().rev().find(|open| {
        matches!(
            open.kind,
            MarkerKind::Character | MarkerKind::Figure | MarkerKind::Periph
        )
    }) {
        open.attribute_spans.push(token.span.clone());
        return;
    }

    state.append_leaf(LeafKind::Attributes, token.span.clone());
}

fn finalize_open_marker(open: OpenMarker) -> Node {
    Node::Container(ContainerNode {
        kind: ContainerKind::from_marker_kind(open.kind, &open.marker),
        marker: open.marker,
        marker_span: open.span,
        close_span: open.close_span,
        special_span: open.special_span,
        attribute_spans: open.attribute_spans,
        children: open.children,
    })
}

fn close_matching_marker(close_marker: &str, token: &ScanToken, state: &mut ParserState) {
    let is_note_close = matches!(close_marker, "f" | "fe" | "x" | "ef" | "ex");
    let match_idx = state.stack.iter().rposition(|open| {
        if is_note_close {
            open.kind == MarkerKind::Note && open.marker == close_marker
        } else {
            open.marker == close_marker
        }
    });

    match match_idx {
        Some(idx) => {
            while state.stack.len() > idx + 1 {
                let top = state.pop_open().expect("stack length checked");
                if !(is_note_close
                    && matches!(
                        top.kind,
                        MarkerKind::Character | MarkerKind::Unknown | MarkerKind::TableCell
                    ))
                {
                    state.analysis.recoveries.push(ParseRecovery::close(
                        RecoveryCode::MisnestedCloseMarker,
                        token.span.clone(),
                        Some(top.span.clone()),
                        top.marker.clone(),
                        close_marker.to_string(),
                    ));
                }
                state.append_finalized(top);
            }
            if let Some(open) = state.stack.last_mut() {
                open.close_span = Some(token.span.clone());
            }
            state.close_and_append_top();
        }
        None => {
            state.analysis.recoveries.push(ParseRecovery::close(
                RecoveryCode::StrayCloseMarker,
                token.span.clone(),
                None,
                String::new(),
                close_marker.to_string(),
            ));
        }
    }
}

fn close_paragraph(state: &mut ParserState, trigger_span: &std::ops::Range<usize>) {
    loop {
        match state.stack.last().map(|open| open.kind) {
            Some(MarkerKind::Character)
            | Some(MarkerKind::Unknown)
            | Some(MarkerKind::Figure)
            | Some(MarkerKind::TableCell) => {
                let open = state.pop_open().expect("stack last checked");
                if !open.marker.starts_with('z') {
                    state.analysis.recoveries.push(ParseRecovery::close(
                        RecoveryCode::ImplicitlyClosedMarker,
                        trigger_span.clone(),
                        Some(open.span.clone()),
                        open.marker.clone(),
                        String::new(),
                    ));
                }
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

fn force_close_notes(state: &mut ParserState, trigger_span: &std::ops::Range<usize>) {
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
                state
                    .analysis
                    .recoveries
                    .push(ParseRecovery::marker_with_related(
                        RecoveryCode::UnclosedNote,
                        note.span.clone(),
                        Some(trigger_span.clone()),
                        note.marker.clone(),
                    ));
                state.append_finalized(note);
            }
            None => break,
        }
    }
}

fn close_sidebar(state: &mut ParserState, trigger_span: &std::ops::Range<usize>) {
    let sidebar_idx = state
        .stack
        .iter()
        .rposition(|open| open.kind == MarkerKind::SidebarStart);
    match sidebar_idx {
        Some(idx) => {
            while state.stack.len() > idx + 1 {
                let top = state.pop_open().expect("stack length checked");
                if !top.marker.starts_with('z') {
                    state.analysis.recoveries.push(ParseRecovery::close(
                        RecoveryCode::ImplicitlyClosedMarker,
                        trigger_span.clone(),
                        Some(top.span.clone()),
                        top.marker.clone(),
                        String::new(),
                    ));
                }
                state.append_finalized(top);
            }
            state.close_and_append_top();
        }
        None => {
            close_paragraph(state, trigger_span);
            state.analysis.recoveries.push(ParseRecovery::close(
                RecoveryCode::StrayCloseMarker,
                trigger_span.clone(),
                None,
                String::new(),
                "esbe".to_string(),
            ));
            state.append_node(Node::Container(ContainerNode {
                kind: ContainerKind::Unknown,
                marker: "esbe".to_string(),
                marker_span: trigger_span.clone(),
                close_span: None,
                special_span: None,
                attribute_spans: Vec::new(),
                children: Vec::new(),
            }));
        }
    }
}

fn close_table_cell_in_row(state: &mut ParserState) {
    while state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::TableCell)
    {
        state.close_and_append_top();
    }
}

fn close_table_row(state: &mut ParserState) {
    if state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::TableRow)
    {
        state.close_and_append_top();
    }
}

fn close_open_meta(state: &mut ParserState) {
    while state
        .stack
        .last()
        .is_some_and(|open| open.kind == MarkerKind::Meta)
    {
        state.close_and_append_top();
    }
}

fn close_character_in_note(state: &mut ParserState) {
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

fn unknown_marker_starts_new_block(state: &ParserState) -> bool {
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
                    Node::Leaf {
                        kind: LeafKind::Whitespace,
                        ..
                    } => None,
                    Node::Leaf {
                        kind: LeafKind::Newline,
                        ..
                    } => Some(true),
                    _ => Some(false),
                })
                .unwrap_or(false)
        })
}

fn close_inline_above_paragraph(state: &mut ParserState) {
    while let Some(MarkerKind::Character)
    | Some(MarkerKind::Figure)
    | Some(MarkerKind::Unknown)
    | Some(MarkerKind::TableCell) = state.stack.last().map(|open| open.kind)
    {
        state.close_and_append_top();
    }
}

fn is_same_note_family(stack: &[OpenMarker], incoming_marker: &str) -> bool {
    let note_family = stack
        .iter()
        .rev()
        .find(|open| open.kind == MarkerKind::Note)
        .and_then(|open| marker_note_family(&open.marker));
    let incoming_family = marker_note_family(incoming_marker);
    matches!((note_family, incoming_family), (Some(left), Some(right)) if left == right)
}

fn should_close_current_note_char(
    stack: &[OpenMarker],
    incoming_marker: &str,
    incoming_valid_in_note: bool,
) -> bool {
    let Some(current_char) = stack
        .iter()
        .rev()
        .find(|open| open.kind == MarkerKind::Character)
    else {
        return false;
    };

    let current_info = lookup_marker(current_char.marker.as_str());

    if incoming_valid_in_note && is_same_note_family(stack, incoming_marker) {
        return true;
    }

    if lookup_marker_def(current_char.marker.as_str()).and_then(|def| def.note_subkind)
        == Some(NoteSubkind::StructuralKeepsNestedCharsOpen)
    {
        return false;
    }

    current_info.valid_in_note
        && !matches!(
            lookup_marker_id(incoming_marker),
            Some(id) if id == MARKER_REF || id == MARKER_JMP
        )
}

fn token_is_milestone_attribute_continuation(token: &ScanToken) -> bool {
    token.kind == ScanTokenKind::Text && parses_as_attribute_continuation(token.text.as_str())
}

fn parses_as_attribute_continuation(text: &str) -> bool {
    let mut remaining = text.trim_start();
    if remaining.is_empty() || !remaining.contains('=') {
        return false;
    }

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            return true;
        }

        let Some(eq_pos) = remaining.find('=') else {
            return false;
        };
        let before_eq = &remaining[..eq_pos];
        if before_eq.is_empty() || before_eq.contains(' ') || before_eq.contains('"') {
            return false;
        }

        remaining = &remaining[eq_pos + 1..];
        if !remaining.starts_with('"') {
            return false;
        }
        remaining = &remaining[1..];

        let Some(end_quote) = find_unescaped_quote(remaining) else {
            return true;
        };
        remaining = &remaining[end_quote + 1..];
    }

    true
}

fn find_unescaped_quote(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    (0..bytes.len())
        .find(|&index| bytes[index] == b'"' && (index == 0 || bytes[index - 1] != b'\\'))
}

fn finish_open_markers(state: &mut ParserState) {
    while let Some(open) = state.pop_open() {
        match open.kind {
            MarkerKind::Note => state.analysis.recoveries.push(ParseRecovery::marker(
                RecoveryCode::UnclosedNote,
                open.span.clone(),
                open.marker.clone(),
            )),
            MarkerKind::Character
            | MarkerKind::Figure
            | MarkerKind::Unknown
            | MarkerKind::SidebarStart
            | MarkerKind::TableCell => {
                if !open.marker.starts_with('z') {
                    state.analysis.recoveries.push(ParseRecovery::marker(
                        RecoveryCode::UnclosedMarkerAtEof,
                        open.span.clone(),
                        open.marker.clone(),
                    ));
                }
            }
            _ => {}
        }
        state.append_finalized(open);
    }
}

fn extract_first_word(text: &str) -> Option<(&str, usize)> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let leading_ws = text.len() - trimmed.len();
    let end = trimmed
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
        .unwrap_or(trimmed.len());
    Some((&trimmed[..end], leading_ws + end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{RecoveryCode, handle::recoveries};

    #[test]
    fn parse_exposes_recoveries_without_diagnostics() {
        let handle = parse("\\id GEN\n\\c\n\\p\n\\v 1 text\\nd text");
        let recoveries = recoveries(&handle);
        assert!(
            recoveries
                .iter()
                .any(|recovery| recovery.code == RecoveryCode::MissingChapterNumber)
        );
        assert!(
            recoveries
                .iter()
                .any(|recovery| recovery.code == RecoveryCode::UnclosedMarkerAtEof)
        );
    }

    #[test]
    fn paragraph_boundary_force_closes_notes() {
        let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 text\\f + note\n\\p next");
        let recoveries = recoveries(&handle);
        assert!(
            recoveries
                .iter()
                .any(|recovery| recovery.code == RecoveryCode::UnclosedNote)
        );
    }

    #[test]
    fn paragraph_boundary_implicitly_closes_inline_markers() {
        let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 \\add text\n\\p next");
        let recoveries = recoveries(&handle);
        assert!(
            recoveries
                .iter()
                .any(|recovery| recovery.code == RecoveryCode::ImplicitlyClosedMarker)
        );
    }

    #[test]
    fn document_model_contains_structured_book_and_paragraph_nodes() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning");
        let children = &handle.document().children;

        assert!(matches!(
            children.first(),
            Some(Node::Container(ContainerNode {
                kind: ContainerKind::Book,
                ..
            }))
        ));
        assert!(children.iter().any(|node| matches!(
            node,
            Node::Container(ContainerNode {
                kind: ContainerKind::Paragraph,
                ..
            })
        )));
        assert!(
            children
                .iter()
                .any(|node| matches!(node, Node::Chapter { .. }))
        );
    }

    #[test]
    fn milestone_collects_multiline_attribute_continuations() {
        let handle = parse(
            "\\id GEN\n\\c 1\n\\p\n\\v 22 text \\qt1-s |sid=\"qt1_ACT_17:22\"\nwho=\"Paul\"\\*more",
        );

        let milestone = handle
            .document()
            .children
            .iter()
            .find_map(|node| match node {
                Node::Container(ContainerNode {
                    kind: ContainerKind::Paragraph,
                    children,
                    ..
                }) => children.iter().find_map(|child| match child {
                    Node::Milestone {
                        attribute_spans, ..
                    } => Some(attribute_spans),
                    _ => None,
                }),
                _ => None,
            })
            .expect("expected milestone node");

        assert_eq!(milestone.len(), 2);
    }

    #[test]
    fn extended_footnote_submarkers_are_siblings_not_nested() {
        let handle =
            parse("\\id MRK\n\\c 1\n\\p\n\\v 1 text\\ef - \\fr 1.1: \\fq quote \\ft tail\\ef*");

        let note_children = handle
            .document()
            .children
            .iter()
            .find_map(|node| match node {
                Node::Container(ContainerNode {
                    kind: ContainerKind::Paragraph,
                    children,
                    ..
                }) => children.iter().find_map(|child| match child {
                    Node::Container(ContainerNode {
                        kind: ContainerKind::Note,
                        children,
                        ..
                    }) => Some(children),
                    _ => None,
                }),
                _ => None,
            })
            .expect("expected note");

        let markers = note_children
            .iter()
            .filter_map(|child| match child {
                Node::Container(ContainerNode { marker, .. }) => Some(marker.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(markers, vec!["fr", "fq", "ft"]);
    }

    #[test]
    fn non_note_char_inside_cross_reference_becomes_sibling() {
        let handle = parse("\\id GEN\n\\c 1\n\\p \\x + \\xo 1.1 \\em stuff\\em*\\x*");

        let note_children = handle
            .document()
            .children
            .iter()
            .find_map(|node| match node {
                Node::Container(ContainerNode {
                    kind: ContainerKind::Paragraph,
                    children,
                    ..
                }) => children.iter().find_map(|child| match child {
                    Node::Container(ContainerNode {
                        kind: ContainerKind::Note,
                        children,
                        ..
                    }) => Some(children),
                    _ => None,
                }),
                _ => None,
            })
            .expect("expected note");

        let markers = note_children
            .iter()
            .filter_map(|child| match child {
                Node::Container(ContainerNode { marker, .. }) => Some(marker.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(markers, vec!["xo", "em"]);
    }

    #[test]
    fn nested_char_inside_note_char_stays_nested() {
        let handle =
            parse("\\id GEN\n\\c 1\n\\p \\x + \\xo 1.1 \\xt text \\+dc inner\\+dc* tail\\x*");

        let xt_children = handle
            .document()
            .children
            .iter()
            .find_map(|node| match node {
                Node::Container(ContainerNode {
                    kind: ContainerKind::Paragraph,
                    children,
                    ..
                }) => children.iter().find_map(|child| match child {
                    Node::Container(ContainerNode {
                        kind: ContainerKind::Note,
                        children,
                        ..
                    }) => children.iter().find_map(|note_child| match note_child {
                        Node::Container(ContainerNode {
                            marker, children, ..
                        }) if marker == "xt" => Some(children),
                        _ => None,
                    }),
                    _ => None,
                }),
                _ => None,
            })
            .expect("expected xt note child");

        assert!(xt_children.iter().any(|child| matches!(
            child,
            Node::Container(ContainerNode { marker, .. }) if marker == "dc"
        )));
    }

    #[test]
    fn unknown_marker_after_line_break_starts_new_block() {
        let handle = parse("\\id GEN\n\\c 1\n\\p \\v 1 Hi.\n\\ix text\n");
        let children = &handle.document().children;

        assert!(matches!(
            children.last(),
            Some(Node::Container(ContainerNode {
                kind: ContainerKind::Paragraph,
                marker,
                ..
            })) if marker == "ix"
        ));
    }

    #[test]
    fn second_id_marker_tracks_its_local_book_code_span() {
        let handle = parse("\\id GEN\n\\c 1\n\\id MAT\n");
        let books = handle
            .document()
            .children
            .iter()
            .filter_map(|node| match node {
                Node::Container(ContainerNode {
                    kind: ContainerKind::Book,
                    special_span,
                    ..
                }) => Some(special_span.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(books.len(), 2);
        assert_eq!(books[0], Some(4..7));
        assert_eq!(books[1], Some(17..20));
    }

    #[test]
    fn stray_sidebar_end_is_retained_as_unknown_node() {
        let handle = parse("\\id GEN\n\\c 1\n\\esbe\n");
        assert!(handle.document().children.iter().any(|node| matches!(
            node,
            Node::Container(ContainerNode {
                kind: ContainerKind::Unknown,
                marker,
                ..
            }) if marker == "esbe"
        )));
    }
}
