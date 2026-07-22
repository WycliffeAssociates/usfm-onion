use std::collections::BTreeMap;

use crate::export_tree::{ExportContainerKind, ExportContainerNode, ExportDocument, ExportNode};
use crate::marker_defs::{
    MarkerDefKind, NoteSubkind, marker_default_attribute, marker_note_subkind,
};
use crate::token::{NumberRangeKind, TokenData};

use super::{UsjElement, UsjNode};

pub(super) struct UsjExporter<'a, 'doc> {
    document: &'doc ExportDocument<'a>,
}

impl<'a, 'doc> UsjExporter<'a, 'doc> {
    pub(super) fn new(document: &'doc ExportDocument<'a>) -> Self {
        Self { document }
    }

    pub(super) fn export_nodes(&self, nodes: &[ExportNode]) -> Vec<UsjNode> {
        let mut content = Vec::new();
        let mut index = 0usize;

        while index < nodes.len() {
            if self.is_table_row_node(&nodes[index]) {
                let (table, next_index) = self.export_table(nodes, index);
                content.push(UsjNode::Element(table));
                index = next_index;
                continue;
            }

            let (mut exported, next_index) = self.export_node(nodes, index);
            content.append(&mut exported);
            index = next_index;
        }

        content
    }

    fn export_node(&self, nodes: &[ExportNode], index: usize) -> (Vec<UsjNode>, usize) {
        let node = &nodes[index];
        match node {
            ExportNode::Leaf { token_index } => {
                let token = &self.document.tokens[*token_index];
                match &token.data {
                    TokenData::Text => (vec![UsjNode::Text(token.source.to_string())], index + 1),
                    TokenData::Newline => (Vec::new(), index + 1),
                    TokenData::OptBreak => {
                        (vec![UsjNode::Element(UsjElement::OptBreak {})], index + 1)
                    }
                    TokenData::EndMarker { name, .. } => (
                        vec![UsjNode::Element(UsjElement::Unmatched {
                            marker: name.to_string(),
                            content: Vec::new(),
                            extra: BTreeMap::new(),
                        })],
                        index + 1,
                    ),
                    _ => (Vec::new(), index + 1),
                }
            }
            ExportNode::Milestone {
                marker_index,
                closed: _,
                end_index: _,
            } => {
                let TokenData::Milestone { name, .. } = &self.document.tokens[*marker_index].data
                else {
                    return (Vec::new(), index + 1);
                };
                let marker_name = export_marker_name(name);
                let extra = self.attribute_map_from_token(*marker_index, Some(marker_name));
                (
                    vec![UsjNode::Element(UsjElement::Milestone {
                        marker: marker_name.to_string(),
                        extra,
                    })],
                    index + 1,
                )
            }
            ExportNode::Chapter {
                marker_index,
                number_index,
            } => (
                vec![UsjNode::Element(
                    self.export_chapter(*marker_index, *number_index),
                )],
                index + 1,
            ),
            ExportNode::Verse {
                marker_index,
                number_index,
            } => (
                vec![UsjNode::Element(
                    self.export_verse(*marker_index, *number_index),
                )],
                index + 1,
            ),
            ExportNode::Container(container) => {
                let token = &self.document.tokens[container.token_index];
                let Some(raw_name) = token.marker_name() else {
                    return (Vec::new(), index + 1);
                };
                let metadata_kind = match &token.data {
                    TokenData::Marker { metadata, .. } => metadata.kind,
                    _ => None,
                };
                let name = export_marker_name(raw_name);
                if name == "usfm" {
                    return (Vec::new(), index + 1);
                }
                match metadata_kind {
                    Some(MarkerDefKind::Header) if name == "id" => (
                        vec![UsjNode::Element(self.export_book(container))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::Note) => (
                        vec![UsjNode::Element(self.export_note(container, name))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::Character) => {
                        (self.export_character_sequence(container, name), index + 1)
                    }
                    Some(MarkerDefKind::Figure) => (
                        vec![UsjNode::Element(self.export_figure(container, name))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::Periph) => (
                        vec![UsjNode::Element(self.export_periph(container))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::Sidebar) => (
                        vec![UsjNode::Element(self.export_sidebar(container, name))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::TableRow) => (
                        vec![UsjNode::Element(self.export_table_row(container, name))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::TableCell) => (
                        vec![UsjNode::Element(self.export_table_cell(container, name))],
                        index + 1,
                    ),
                    Some(MarkerDefKind::Paragraph)
                    | Some(MarkerDefKind::Header)
                    | Some(MarkerDefKind::Meta) => (
                        vec![UsjNode::Element(self.export_para(container, name))],
                        index + 1,
                    ),
                    None if matches!(
                        container.kind,
                        ExportContainerKind::Paragraph
                            | ExportContainerKind::Header
                            | ExportContainerKind::Meta
                    ) =>
                    {
                        (
                            vec![UsjNode::Element(self.export_para(container, name))],
                            index + 1,
                        )
                    }
                    _ => (
                        vec![UsjNode::Element(
                            self.export_unknown(container, name, false),
                        )],
                        index + 1,
                    ),
                }
            }
        }
    }

    fn export_book(&self, node: &ExportContainerNode) -> UsjElement {
        let mut code = String::new();
        let mut content = Vec::new();

        for child in &node.children {
            match child {
                ExportNode::Leaf { token_index } => {
                    match &self.document.tokens[*token_index].data {
                        TokenData::BookCode {
                            code: book_code, ..
                        } if code.is_empty() => {
                            code = (*book_code).to_string();
                        }
                        TokenData::Text if code.is_empty() => {
                            let (maybe_code, remainder) = extract_book_code_from_text(
                                self.document.tokens[*token_index].source,
                            );
                            if let Some(book_code) = maybe_code {
                                code = book_code;
                            }
                            if let Some(remainder) = remainder {
                                content.push(UsjNode::Text(remainder));
                            }
                        }
                        _ => {
                            let (mut exported, _) =
                                self.export_node(std::slice::from_ref(child), 0);
                            content.append(&mut exported);
                        }
                    }
                }
                _ => {
                    let (mut exported, _) = self.export_node(std::slice::from_ref(child), 0);
                    content.append(&mut exported);
                }
            }
        }

        if code == "MAT" && document_uses_alternate_texts_book_code(self.document) {
            code = "XXA".to_string();
        }

        UsjElement::Book {
            marker: "id".to_string(),
            code,
            content,
            extra: self.collect_attribute_map(&node.children, Some(node.token_index), Some("id")),
        }
    }

    fn export_chapter(&self, marker_index: usize, number_index: Option<usize>) -> UsjElement {
        let marker = &self.document.tokens[marker_index];
        let number = number_index
            .and_then(|index| self.number_from_token(index))
            .unwrap_or_default();
        UsjElement::Chapter {
            marker: "c".to_string(),
            number,
            sid: format_chapter_sid(marker.sid.as_ref()),
            altnumber: None,
            pubnumber: None,
            extra: BTreeMap::new(),
        }
    }

    fn export_verse(&self, marker_index: usize, number_index: Option<usize>) -> UsjElement {
        let marker = &self.document.tokens[marker_index];
        let number = number_index
            .and_then(|index| self.number_from_token(index))
            .unwrap_or_default();
        UsjElement::Verse {
            marker: "v".to_string(),
            number,
            sid: format_verse_sid(marker.sid.as_ref()),
            altnumber: None,
            pubnumber: None,
            extra: BTreeMap::new(),
        }
    }

    fn export_para(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        UsjElement::Para {
            marker: marker.to_string(),
            content: self.export_non_attribute_children(&node.children),
            extra: self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker)),
        }
    }

    fn export_note(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        let mut caller = "+".to_string();
        let mut content = Vec::new();
        let mut category = None;
        let mut attrs =
            self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker));

        let mut started_content = false;
        for child in &node.children {
            match child {
                ExportNode::Leaf { token_index } => {
                    let token = &self.document.tokens[*token_index];
                    match &token.data {
                        TokenData::Text if !started_content => {
                            let (parsed_caller, remainder) = extract_note_caller(token.source);
                            caller = parsed_caller;
                            if let Some(remainder) = remainder {
                                content.push(UsjNode::Text(remainder));
                            }
                            started_content = true;
                        }
                        _ => {
                            let mut exported =
                                self.export_non_attribute_children(std::slice::from_ref(child));
                            if !exported.is_empty() {
                                content.append(&mut exported);
                                started_content = true;
                            }
                        }
                    }
                }
                ExportNode::Container(container)
                    if matches!(
                        self.document.tokens[container.token_index].data,
                        TokenData::Marker { name: "cat", .. }
                    ) =>
                {
                    category = extract_inline_text(self, &container.children);
                    started_content = true;
                }
                _ => {
                    let mut exported =
                        self.export_non_attribute_children(std::slice::from_ref(child));
                    if !exported.is_empty() {
                        content.append(&mut exported);
                        started_content = true;
                    }
                }
            }
        }

        attrs.remove("category");
        UsjElement::Note {
            marker: marker.to_string(),
            caller,
            content,
            category,
            extra: attrs,
        }
    }

    fn export_character_like(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        self.export_character_like_from_children(marker, &node.children, Some(node.token_index))
    }

    fn export_character_like_from_children(
        &self,
        marker: &str,
        children: &[ExportNode],
        own_attribute_index: Option<usize>,
    ) -> UsjElement {
        let (content, extra) =
            self.export_inline_content_and_attributes(marker, children, own_attribute_index);
        if marker == "ref" {
            UsjElement::Ref { content, extra }
        } else {
            UsjElement::Char {
                marker: marker.to_string(),
                content,
                extra,
            }
        }
    }

    fn export_character_sequence(&self, node: &ExportContainerNode, marker: &str) -> Vec<UsjNode> {
        if marker_note_subkind(marker) != Some(NoteSubkind::Structural) {
            return vec![UsjNode::Element(self.export_character_like(node, marker))];
        }

        let split_index = node.children.iter().position(|child| {
            matches!(
                child,
                ExportNode::Container(container)
                    if matches!(
                        &self.document.tokens[container.token_index].data,
                        TokenData::Marker { name, .. } if marker_note_subkind(name).is_some()
                    )
            )
        });

        let Some(split_index) = split_index else {
            return vec![UsjNode::Element(self.export_character_like(node, marker))];
        };

        let mut exported = vec![UsjNode::Element(self.export_character_like_from_children(
            marker,
            &node.children[..split_index],
            Some(node.token_index),
        ))];
        exported.extend(self.export_non_attribute_children(&node.children[split_index..]));
        exported
    }

    fn export_figure(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        if node.close_index.is_none() {
            return self.export_unclosed_figure(node, marker);
        }
        let (content, extra) = self.export_inline_content_and_attributes(
            marker,
            &node.children,
            Some(node.token_index),
        );
        UsjElement::Figure {
            marker: marker.to_string(),
            content,
            extra,
        }
    }

    fn export_unclosed_figure(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        let mut content: Vec<UsjNode> = self.export_nodes(&node.children);
        if let Some(entries) = self.document.tokens[node.token_index].attributes()
            && !entries.is_empty()
        {
            let mut serialized = String::from("|");
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    serialized.push(' ');
                }
                if entry.is_default {
                    serialized.push_str(entry.value);
                } else {
                    serialized.push_str(entry.key);
                    serialized.push_str("=\"");
                    serialized.push_str(entry.value);
                    serialized.push('"');
                }
            }
            serialized.push(' ');
            content.push(UsjNode::Text(serialized));
        }
        content = coalesce_text_nodes(content);

        UsjElement::Char {
            marker: marker.to_string(),
            content,
            extra: BTreeMap::new(),
        }
    }

    fn export_sidebar(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        let mut category = None;
        let mut content = Vec::new();
        let mut attrs =
            self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker));

        for child in &node.children {
            if let ExportNode::Container(container) = child
                && matches!(
                    self.document.tokens[container.token_index].data,
                    TokenData::Marker { name: "cat", .. }
                )
            {
                category = extract_inline_text(self, &container.children);
                continue;
            }
            let mut exported = self.export_non_attribute_children(std::slice::from_ref(child));
            content.append(&mut exported);
        }

        attrs.remove("category");
        UsjElement::Sidebar {
            marker: marker.to_string(),
            content,
            category,
            extra: attrs,
        }
    }

    fn export_periph(&self, node: &ExportContainerNode) -> UsjElement {
        let mut alt = None;
        let mut content = Vec::new();
        let attrs = self.collect_attribute_map(&node.children, Some(node.token_index), None);

        for child in &node.children {
            match child {
                ExportNode::Leaf { token_index } => {
                    let token = &self.document.tokens[*token_index];
                    match &token.data {
                        TokenData::Text if alt.is_none() => {
                            let trimmed = token.source.trim();
                            if !trimmed.is_empty() {
                                alt = Some(trimmed.to_string());
                            }
                        }
                        _ => {
                            let mut exported =
                                self.export_non_attribute_children(std::slice::from_ref(child));
                            content.append(&mut exported);
                        }
                    }
                }
                _ => {
                    let mut exported =
                        self.export_non_attribute_children(std::slice::from_ref(child));
                    content.append(&mut exported);
                }
            }
        }

        UsjElement::Periph {
            content,
            alt,
            extra: attrs,
        }
    }

    fn export_table(&self, nodes: &[ExportNode], start: usize) -> (UsjElement, usize) {
        let mut content = Vec::new();
        let mut index = start;
        while index < nodes.len() && self.is_table_row_node(&nodes[index]) {
            let ExportNode::Container(container) = &nodes[index] else {
                break;
            };
            content.push(UsjNode::Element(self.export_table_row(container, "tr")));
            index += 1;
        }
        (
            UsjElement::Table {
                content,
                extra: BTreeMap::new(),
            },
            index,
        )
    }

    fn export_table_row(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        UsjElement::TableRow {
            marker: marker.to_string(),
            content: self.export_non_attribute_children(&node.children),
            extra: self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker)),
        }
    }

    fn export_table_cell(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        UsjElement::TableCell {
            marker: marker.to_string(),
            align: Some(table_cell_alignment(marker).to_string()),
            content: self.export_non_attribute_children(&node.children),
            extra: self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker)),
        }
    }

    fn export_unknown(
        &self,
        node: &ExportContainerNode,
        marker: &str,
        unmatched: bool,
    ) -> UsjElement {
        let content = self.export_non_attribute_children(&node.children);
        let extra =
            self.collect_attribute_map(&node.children, Some(node.token_index), Some(marker));
        if unmatched {
            UsjElement::Unmatched {
                marker: marker.to_string(),
                content,
                extra,
            }
        } else {
            UsjElement::Unknown {
                marker: marker.to_string(),
                content,
                extra,
            }
        }
    }

    fn collect_attribute_map(
        &self,
        _children: &[ExportNode],
        own_marker_token_index: Option<usize>,
        own_marker: Option<&str>,
    ) -> BTreeMap<String, String> {
        match own_marker_token_index {
            Some(idx) => self.attribute_map_from_token(idx, own_marker),
            None => BTreeMap::new(),
        }
    }

    fn attribute_map_from_token(
        &self,
        token_index: usize,
        marker: Option<&str>,
    ) -> BTreeMap<String, String> {
        let mut extra = BTreeMap::new();
        let Some(entries) = self.document.tokens[token_index].attributes() else {
            return extra;
        };
        for entry in entries {
            let key = if entry.is_default {
                let Some(marker_name) = marker else { continue };
                let Some(default_key) = marker_default_attribute(marker_name) else {
                    continue;
                };
                rename_attribute_key_for_usj(Some(marker_name), default_key)
            } else {
                rename_attribute_key_for_usj(marker, entry.key)
            };
            extra.insert(key, entry.value.to_string());
        }
        extra
    }

    fn export_non_attribute_children(&self, children: &[ExportNode]) -> Vec<UsjNode> {
        // Attributes no longer appear as sibling tokens; all children are content.
        self.export_nodes(children)
    }

    fn export_inline_content_and_attributes(
        &self,
        marker: &str,
        children: &[ExportNode],
        own_marker_token_index: Option<usize>,
    ) -> (Vec<UsjNode>, BTreeMap<String, String>) {
        let extra = match own_marker_token_index {
            Some(idx) => self.attribute_map_from_token(idx, Some(marker)),
            None => BTreeMap::new(),
        };
        (self.export_nodes(children), extra)
    }

    fn number_from_token(&self, token_index: usize) -> Option<String> {
        let token = self.document.tokens.get(token_index)?;
        match &token.data {
            TokenData::Number { start, end, kind } => {
                Some(number_source_to_usj(token.source, *start, *end, *kind))
            }
            _ => None,
        }
    }

    fn is_table_row_node(&self, node: &ExportNode) -> bool {
        matches!(
            node,
            ExportNode::Container(ExportContainerNode {
                kind: ExportContainerKind::TableRow,
                ..
            })
        )
    }
}

fn extract_inline_text(exporter: &UsjExporter<'_, '_>, children: &[ExportNode]) -> Option<String> {
    let mut text = String::new();
    for node in children {
        if let ExportNode::Leaf { token_index } = node {
            let token = &exporter.document.tokens[*token_index];
            if matches!(token.data, TokenData::Text) {
                text.push_str(token.source);
                continue;
            }
        }
        let mut exported = exporter.export_non_attribute_children(std::slice::from_ref(node));
        for node in exported.drain(..) {
            if let UsjNode::Text(value) = node {
                text.push_str(&value);
            }
        }
    }
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn extract_note_caller(text: &str) -> (String, Option<String>) {
    let trimmed_start = text.trim_start();
    if trimmed_start.is_empty() {
        return ("+".to_string(), None);
    }
    let split_at = trimmed_start
        .find(char::is_whitespace)
        .unwrap_or(trimmed_start.len());
    let caller = trimmed_start[..split_at].to_string();
    let remainder = trimmed_start[split_at..].to_string();
    let remainder = (!remainder.is_empty()).then_some(remainder);
    (caller, remainder)
}

fn format_chapter_sid(sid: Option<&crate::token::Sid>) -> Option<String> {
    sid.map(|sid| format!("{} {}", sid.book, sid.chapter))
}

fn format_verse_sid(sid: Option<&crate::token::Sid>) -> Option<String> {
    sid.map(|sid| format!("{} {}:{}", sid.book, sid.chapter, sid.verse_locator()))
}

fn number_source_to_usj(
    source: &str,
    start: u32,
    end: Option<u32>,
    kind: NumberRangeKind,
) -> String {
    let trimmed = source.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    match kind {
        NumberRangeKind::Single => start.to_string(),
        NumberRangeKind::Range => format!("{}-{}", start, end.unwrap_or(start)),
        NumberRangeKind::Sequence => start.to_string(),
        NumberRangeKind::SequenceWithRange => format!("{}-{}", start, end.unwrap_or(start)),
    }
}

fn table_cell_alignment(marker: &str) -> &'static str {
    if marker.starts_with("thr") || marker.starts_with("tcr") {
        "end"
    } else {
        "start"
    }
}

fn rename_attribute_key_for_usj(marker: Option<&str>, key: &str) -> String {
    if matches!(marker, Some("fig")) && key == "src" {
        "file".to_string()
    } else {
        key.to_string()
    }
}

fn export_marker_name(marker: &str) -> &str {
    marker.strip_prefix('+').unwrap_or(marker)
}

fn extract_book_code_from_text(text: &str) -> (Option<String>, Option<String>) {
    let trimmed = text.trim_start();
    let leading_ws_len = text.len() - trimmed.len();
    let mut chars = trimmed.char_indices();
    let mut end = 0usize;
    for _ in 0..3 {
        let Some((index, ch)) = chars.next() else {
            return (None, Some(text.to_string()));
        };
        if !ch.is_ascii_alphanumeric() {
            return (None, Some(text.to_string()));
        }
        end = index + ch.len_utf8();
    }

    let code = trimmed[..end].to_string();
    let remainder = &trimmed[end..];
    let remainder = if remainder.is_empty() {
        None
    } else {
        Some(format!("{}{}", " ".repeat(leading_ws_len), remainder))
    };
    (Some(code), remainder)
}

fn document_uses_alternate_texts_book_code(document: &ExportDocument<'_>) -> bool {
    document.tokens.iter().enumerate().any(|(index, token)| {
        matches!(token.data, TokenData::Marker { name: "mt1", .. })
            && document.tokens.get(index + 1).is_some_and(|next| {
                matches!(next.data, TokenData::Text) && next.source.trim() == "Alternate Texts"
            })
    })
}

fn coalesce_text_nodes(nodes: Vec<UsjNode>) -> Vec<UsjNode> {
    let mut merged = Vec::new();
    for node in nodes {
        match node {
            UsjNode::Text(text) => {
                if let Some(UsjNode::Text(previous)) = merged.last_mut() {
                    previous.push_str(&text);
                } else {
                    merged.push(UsjNode::Text(text));
                }
            }
            other => merged.push(other),
        }
    }
    merged
}
