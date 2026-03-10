use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::internal::marker_defs::marker_default_attribute;
use crate::internal::markers::lookup_marker;
use crate::internal::recovery::{ParseRecovery, RecoveryCode, RecoveryPayload};
use crate::internal::syntax::{ContainerKind, ContainerNode, LeafKind, Node};
use crate::model::document_tree::{DocumentTreeDocument, DocumentTreeElement, DocumentTreeNode};
use crate::model::token::TokenViewOptions;
use crate::model::usj::{UsjDocument, UsjElement, UsjNode};
use crate::parse::handle::ParseHandle;
use crate::parse::handle::tokens;

pub fn to_usj_value(handle: &ParseHandle) -> Value {
    to_usj_value_with_options(handle, UsjSerializerOptions::for_usj(handle))
}

pub(crate) fn to_usj_value_with_options(
    handle: &ParseHandle,
    options: UsjSerializerOptions,
) -> Value {
    let mut serializer = UsjSerializer::new(handle, options);
    let content = serializer.serialize_children(
        handle.document().children.as_slice(),
        ContentTrim::none(options.preserve_vertical_whitespace),
    );

    Value::Object(Map::from_iter([
        ("type".to_string(), Value::String("USJ".to_string())),
        (
            "version".to_string(),
            Value::String(usj_version(handle.source()).to_string()),
        ),
        ("content".to_string(), Value::Array(content)),
    ]))
}

pub fn to_usj_document(handle: &ParseHandle) -> UsjDocument {
    serde_json::from_value(to_usj_value(handle)).expect("USJ serializer should produce typed USJ")
}

pub fn to_document_tree_document(handle: &ParseHandle) -> DocumentTreeDocument {
    let mut serializer = UsjSerializer::new(handle, UsjSerializerOptions::for_document_tree());
    let content = serializer.serialize_tree_children(
        handle.document().children.as_slice(),
        ContentTrim::none(true),
    );
    DocumentTreeDocument {
        doc_type: "USJ".to_string(),
        version: usj_version(handle.source()).to_string(),
        tokens: tokens(handle, TokenViewOptions::default()),
        content,
    }
}

pub fn document_tree_to_usj_document(document: &DocumentTreeDocument) -> UsjDocument {
    let mut content = Vec::new();
    append_usj_nodes(&mut content, document.content.as_slice());
    UsjDocument {
        doc_type: "USJ".to_string(),
        version: document.version.clone(),
        content,
    }
}

pub fn to_usj_string(handle: &ParseHandle) -> Result<String, serde_json::Error> {
    serde_json::to_string(&to_usj_value(handle))
}

pub fn to_usj_string_pretty(handle: &ParseHandle) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&to_usj_value(handle))
}

fn document_tree_document_from_value(value: Value) -> DocumentTreeDocument {
    let Value::Object(mut object) = value else {
        panic!("editor tree serializer should produce an object");
    };

    DocumentTreeDocument {
        doc_type: take_string(&mut object, "type"),
        version: take_string(&mut object, "version"),
        tokens: Vec::new(),
        content: take_content(&mut object),
    }
}

fn document_tree_node_from_value(value: Value) -> DocumentTreeNode {
    match value {
        Value::String(value) => DocumentTreeNode::Element(DocumentTreeElement::Text { value }),
        Value::Object(object) => {
            DocumentTreeNode::Element(document_tree_element_from_object(object))
        }
        other => panic!("unexpected editor tree node value: {other:?}"),
    }
}

fn document_tree_element_from_object(mut object: Map<String, Value>) -> DocumentTreeElement {
    match take_string(&mut object, "type").as_str() {
        "book" => DocumentTreeElement::Book {
            marker: take_string(&mut object, "marker"),
            code: take_string(&mut object, "code"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "chapter" => DocumentTreeElement::Chapter {
            marker: take_string(&mut object, "marker"),
            number: take_string(&mut object, "number"),
            extra: take_extra(object),
        },
        "verse" => DocumentTreeElement::Verse {
            marker: take_string(&mut object, "marker"),
            number: take_string(&mut object, "number"),
            extra: take_extra(object),
        },
        "para" => DocumentTreeElement::Para {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "char" => DocumentTreeElement::Char {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "note" => DocumentTreeElement::Note {
            marker: take_string(&mut object, "marker"),
            caller: take_string(&mut object, "caller"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "ms" => DocumentTreeElement::Milestone {
            marker: take_string(&mut object, "marker"),
            extra: take_extra(object),
        },
        "figure" => DocumentTreeElement::Figure {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "sidebar" => DocumentTreeElement::Sidebar {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "periph" => DocumentTreeElement::Periph {
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "table" => DocumentTreeElement::Table {
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "table:row" => DocumentTreeElement::TableRow {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "table:cell" => DocumentTreeElement::TableCell {
            marker: take_string(&mut object, "marker"),
            align: take_string(&mut object, "align"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "ref" => DocumentTreeElement::Ref {
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "unknown" => DocumentTreeElement::Unknown {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "unmatched" => DocumentTreeElement::Unmatched {
            marker: take_string(&mut object, "marker"),
            content: take_content(&mut object),
            extra: take_extra(object),
        },
        "optbreak" => DocumentTreeElement::OptBreak {},
        "linebreak" => DocumentTreeElement::LineBreak {
            value: object
                .remove("value")
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "\n".to_string()),
        },
        other => panic!("unexpected editor tree element type: {other}"),
    }
}

fn take_string(object: &mut Map<String, Value>, key: &str) -> String {
    match object.remove(key) {
        Some(Value::String(value)) => value,
        Some(other) => panic!("expected string for {key}, got {other:?}"),
        None => panic!("missing string field {key}"),
    }
}

fn take_content(object: &mut Map<String, Value>) -> Vec<DocumentTreeNode> {
    match object.remove("content") {
        Some(Value::Array(content)) => content
            .into_iter()
            .map(document_tree_node_from_value)
            .collect(),
        Some(other) => panic!("expected array for content, got {other:?}"),
        None => Vec::new(),
    }
}

fn take_extra(object: Map<String, Value>) -> BTreeMap<String, Value> {
    object.into_iter().collect()
}

struct UsjSerializer<'a> {
    source: &'a str,
    recoveries: &'a [ParseRecovery],
    book_code: String,
    current_chapter: String,
    emit_sid: bool,
    preserve_vertical_whitespace: bool,
    chapter_sid_uses_zero_verse: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct UsjSerializerOptions {
    emit_sid: bool,
    preserve_vertical_whitespace: bool,
    chapter_sid_uses_zero_verse: bool,
}

#[derive(Clone, Copy)]
struct ContentTrim {
    trim_first_string_start: bool,
    trim_last_string_end: bool,
    preserve_newlines: bool,
    trim_leading_after_chapter_or_verse: bool,
}

impl ContentTrim {
    const fn none(preserve_newlines: bool) -> Self {
        Self {
            trim_first_string_start: false,
            trim_last_string_end: false,
            preserve_newlines,
            trim_leading_after_chapter_or_verse: !preserve_newlines,
        }
    }

    const fn container(preserve_newlines: bool) -> Self {
        Self {
            trim_first_string_start: true,
            trim_last_string_end: true,
            preserve_newlines,
            trim_leading_after_chapter_or_verse: !preserve_newlines,
        }
    }

    const fn inline(preserve_newlines: bool) -> Self {
        Self {
            trim_first_string_start: true,
            trim_last_string_end: false,
            preserve_newlines,
            trim_leading_after_chapter_or_verse: !preserve_newlines,
        }
    }

    const fn note_inline(preserve_newlines: bool) -> Self {
        Self {
            trim_first_string_start: true,
            trim_last_string_end: false,
            preserve_newlines,
            trim_leading_after_chapter_or_verse: !preserve_newlines,
        }
    }
}

impl<'a> UsjSerializer<'a> {
    fn new(handle: &'a ParseHandle, options: UsjSerializerOptions) -> Self {
        Self {
            source: handle.source(),
            recoveries: handle.analysis().recoveries.as_slice(),
            book_code: handle.book_code().unwrap_or_default().to_string(),
            current_chapter: String::new(),
            emit_sid: options.emit_sid,
            preserve_vertical_whitespace: options.preserve_vertical_whitespace,
            chapter_sid_uses_zero_verse: options.chapter_sid_uses_zero_verse,
        }
    }

    fn serialize_children(&mut self, nodes: &[Node], trim: ContentTrim) -> Vec<Value> {
        let mut out = Vec::new();
        let mut index = 0usize;

        while index < nodes.len() {
            if let Some(table) = self.consume_table(nodes, &mut index) {
                push_value(&mut out, table);
                continue;
            }

            if let Some(value) = self.serialize_container_with_trailing_ts_gap(nodes, index) {
                push_value(&mut out, value);
                index += 1;
                continue;
            }

            if let Some(value) = self.serialize_container_with_trailing_separator_gap(nodes, index)
            {
                push_value(&mut out, value);
                index += 1;
                continue;
            }

            if let Some(chapter) = self.consume_chapter(nodes, &mut index) {
                push_value(&mut out, chapter);
                continue;
            }

            if let Some(verse) = self.consume_verse(nodes, &mut index) {
                push_value(&mut out, verse);
                continue;
            }

            self.serialize_node_into(&nodes[index], trim.preserve_newlines, &mut out);
            index += 1;
        }

        normalize_content(&mut out, trim);
        out
    }

    fn serialize_tree_children(
        &mut self,
        nodes: &[Node],
        trim: ContentTrim,
    ) -> Vec<DocumentTreeNode> {
        let mut out = Vec::new();
        let mut index = 0usize;

        while index < nodes.len() {
            if let Some(table) = self.consume_tree_table(nodes, &mut index) {
                push_tree_node(&mut out, table);
                continue;
            }

            if let Some(chapter) = self.consume_tree_chapter(nodes, &mut index) {
                push_tree_node(&mut out, chapter);
                continue;
            }

            if let Some(verse) = self.consume_tree_verse(nodes, &mut index) {
                push_tree_node(&mut out, verse);
                continue;
            }

            self.serialize_tree_node_into(&nodes[index], trim.preserve_newlines, &mut out);
            index += 1;
        }

        normalize_tree_content(&mut out, trim);
        out
    }

    fn consume_tree_table(
        &mut self,
        nodes: &[Node],
        index: &mut usize,
    ) -> Option<DocumentTreeNode> {
        let Node::Container(container) = nodes.get(*index)? else {
            return None;
        };
        if container.kind != ContainerKind::TableRow {
            return None;
        }

        let mut rows = Vec::new();
        while let Some(Node::Container(row)) = nodes.get(*index) {
            if row.kind != ContainerKind::TableRow {
                break;
            }
            rows.push(self.serialize_tree_table_row(row));
            *index += 1;
        }

        Some(DocumentTreeNode::Element(DocumentTreeElement::Table {
            content: rows,
            extra: BTreeMap::new(),
        }))
    }

    fn serialize_container_with_trailing_ts_gap(
        &mut self,
        nodes: &[Node],
        index: usize,
    ) -> Option<Value> {
        let Node::Container(container) = nodes.get(index)? else {
            return None;
        };
        let followed_by_paragraph_break = matches!(
            nodes.get(index + 1),
            Some(Node::Container(ContainerNode {
                kind: ContainerKind::Paragraph | ContainerKind::Header | ContainerKind::Meta,
                ..
            }))
        );
        if !followed_by_paragraph_break || !container_has_trailing_standalone_ts_gap(container) {
            return None;
        }

        let mut value = self.serialize_container(container)?;
        append_gap_before_trailing_ts(&mut value);
        Some(value)
    }

    fn serialize_container_with_trailing_separator_gap(
        &mut self,
        nodes: &[Node],
        index: usize,
    ) -> Option<Value> {
        let Node::Container(container) = nodes.get(index)? else {
            return None;
        };
        if !container_has_trailing_newline(container) || !next_sibling_is_unknown_para(nodes, index)
        {
            return None;
        }

        let mut value = self.serialize_container(container)?;
        append_trailing_space_to_last_string(&mut value);
        Some(value)
    }

    fn consume_table(&mut self, nodes: &[Node], index: &mut usize) -> Option<Value> {
        let Node::Container(container) = nodes.get(*index)? else {
            return None;
        };
        if container.kind != ContainerKind::TableRow {
            return None;
        }

        let mut rows = Vec::new();
        while let Some(Node::Container(row)) = nodes.get(*index) {
            if row.kind != ContainerKind::TableRow {
                break;
            }
            rows.push(self.serialize_table_row(row));
            *index += 1;
        }

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("table".to_string()));
        map.insert("content".to_string(), Value::Array(rows));
        Some(Value::Object(map))
    }

    fn consume_chapter(&mut self, nodes: &[Node], index: &mut usize) -> Option<Value> {
        let Node::Chapter {
            marker_span,
            number_span,
        } = nodes.get(*index)?
        else {
            return None;
        };

        let mut altnumber = None;
        let mut pubnumber = None;
        let mut next = *index + 1;
        while let Some(node) = nodes.get(next) {
            match node {
                Node::Container(container) => match container.marker.as_str() {
                    "ca" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            altnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    "cp" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            pubnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
                _ if self.is_ignorable_metadata_trivia(node) => next += 1,
                _ => break,
            }
        }

        *index = next;
        Some(self.serialize_chapter(marker_span, number_span.as_ref(), altnumber, pubnumber))
    }

    fn consume_tree_chapter(
        &mut self,
        nodes: &[Node],
        index: &mut usize,
    ) -> Option<DocumentTreeNode> {
        let Node::Chapter {
            marker_span,
            number_span,
        } = nodes.get(*index)?
        else {
            return None;
        };

        let mut altnumber = None;
        let mut pubnumber = None;
        let mut next = *index + 1;
        while let Some(node) = nodes.get(next) {
            match node {
                Node::Container(container) => match container.marker.as_str() {
                    "ca" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            altnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    "cp" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            pubnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
                _ if self.is_ignorable_metadata_trivia(node) => next += 1,
                _ => break,
            }
        }

        *index = next;
        Some(self.serialize_tree_chapter(marker_span, number_span.as_ref(), altnumber, pubnumber))
    }

    fn consume_verse(&mut self, nodes: &[Node], index: &mut usize) -> Option<Value> {
        let Node::Verse {
            marker_span,
            number_span,
        } = nodes.get(*index)?
        else {
            return None;
        };

        let mut altnumber = None;
        let mut pubnumber = None;
        let mut next = *index + 1;
        while let Some(node) = nodes.get(next) {
            match node {
                Node::Container(container) => match container.marker.as_str() {
                    "va" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            altnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    "vp" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            pubnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
                _ if self.is_ignorable_metadata_trivia(node) => next += 1,
                _ => break,
            }
        }

        *index = next;
        Some(self.serialize_verse(marker_span, number_span.as_ref(), altnumber, pubnumber))
    }

    fn consume_tree_verse(
        &mut self,
        nodes: &[Node],
        index: &mut usize,
    ) -> Option<DocumentTreeNode> {
        let Node::Verse {
            marker_span,
            number_span,
        } = nodes.get(*index)?
        else {
            return None;
        };

        let mut altnumber = None;
        let mut pubnumber = None;
        let mut next = *index + 1;
        while let Some(node) = nodes.get(next) {
            match node {
                Node::Container(container) => match container.marker.as_str() {
                    "va" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            altnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    "vp" => {
                        if let Some(text) =
                            plain_text_from_nodes(container.children.as_slice(), self.source)
                        {
                            pubnumber = Some(text);
                            next += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
                _ if self.is_ignorable_metadata_trivia(node) => next += 1,
                _ => break,
            }
        }

        *index = next;
        Some(self.serialize_tree_verse(marker_span, number_span.as_ref(), altnumber, pubnumber))
    }

    fn serialize_node_into(&mut self, node: &Node, preserve_newlines: bool, out: &mut Vec<Value>) {
        match node {
            Node::Container(container) => {
                if let Some(value) = self.serialize_container(container) {
                    push_value(out, value);
                }
            }
            Node::Chapter {
                marker_span,
                number_span,
            } => push_value(
                out,
                self.serialize_chapter(marker_span, number_span.as_ref(), None, None),
            ),
            Node::Verse {
                marker_span,
                number_span,
            } => push_value(
                out,
                self.serialize_verse(marker_span, number_span.as_ref(), None, None),
            ),
            Node::Milestone {
                marker,
                marker_span,
                attribute_spans,
                closed,
            } => {
                if *closed {
                    push_value(out, self.serialize_milestone(marker, attribute_spans));
                } else {
                    self.serialize_unclosed_milestone_into(marker_span, attribute_spans, out);
                }
            }
            Node::Leaf { kind, span } => {
                self.serialize_leaf_into(*kind, span, preserve_newlines, out)
            }
        }
    }

    fn serialize_tree_node_into(
        &mut self,
        node: &Node,
        preserve_newlines: bool,
        out: &mut Vec<DocumentTreeNode>,
    ) {
        match node {
            Node::Container(container) => match container.kind {
                ContainerKind::Book
                | ContainerKind::Paragraph
                | ContainerKind::Header
                | ContainerKind::Meta
                | ContainerKind::Character
                | ContainerKind::Note
                | ContainerKind::Figure
                | ContainerKind::Sidebar
                | ContainerKind::Periph
                | ContainerKind::TableRow
                | ContainerKind::TableCell
                | ContainerKind::Unknown => {
                    if let Some(node) = self.serialize_tree_container(container) {
                        push_tree_node(out, node);
                    }
                }
            },
            Node::Chapter {
                marker_span,
                number_span,
            } => push_tree_node(
                out,
                self.serialize_tree_chapter(marker_span, number_span.as_ref(), None, None),
            ),
            Node::Verse {
                marker_span,
                number_span,
            } => push_tree_node(
                out,
                self.serialize_tree_verse(marker_span, number_span.as_ref(), None, None),
            ),
            Node::Milestone {
                marker,
                marker_span,
                attribute_spans,
                closed,
            } => {
                if *closed {
                    push_tree_node(out, self.serialize_tree_milestone(marker, attribute_spans));
                } else {
                    self.serialize_tree_unclosed_milestone_into(marker_span, attribute_spans, out);
                }
            }
            Node::Leaf { kind, span } => {
                self.serialize_tree_leaf_into(*kind, span, preserve_newlines, out)
            }
        }
    }

    fn serialize_tree_container(&mut self, container: &ContainerNode) -> Option<DocumentTreeNode> {
        let marker = container.marker.as_str();
        if marker == "usfm" {
            return None;
        }

        Some(match container.kind {
            ContainerKind::Book => self.serialize_tree_book(container),
            ContainerKind::Paragraph | ContainerKind::Header | ContainerKind::Meta => {
                self.serialize_tree_para(container)
            }
            ContainerKind::Character => self.serialize_tree_character(container),
            ContainerKind::Note => self.serialize_tree_note(container),
            ContainerKind::Figure => self.serialize_tree_figure(container),
            ContainerKind::Sidebar => self.serialize_tree_sidebar(container),
            ContainerKind::Periph => self.serialize_tree_periph(container),
            ContainerKind::TableRow => self.serialize_tree_table_row(container),
            ContainerKind::TableCell => self.serialize_tree_table_cell(container),
            ContainerKind::Unknown => self.serialize_tree_unknown(container),
        })
    }

    fn serialize_container(&mut self, container: &ContainerNode) -> Option<Value> {
        let marker = container.marker.as_str();
        if marker == "usfm" {
            return None;
        }

        match container.kind {
            ContainerKind::Book => Some(self.serialize_book(container)),
            ContainerKind::Paragraph | ContainerKind::Header | ContainerKind::Meta => {
                Some(self.serialize_para(container))
            }
            ContainerKind::Character => Some(self.serialize_character(container)),
            ContainerKind::Note => Some(self.serialize_note(container)),
            ContainerKind::Figure => Some(self.serialize_figure(container)),
            ContainerKind::Sidebar => Some(self.serialize_sidebar(container)),
            ContainerKind::Periph => Some(self.serialize_periph(container)),
            ContainerKind::TableRow => Some(self.serialize_table_row(container)),
            ContainerKind::TableCell => Some(self.serialize_table_cell(container)),
            ContainerKind::Unknown => Some(self.serialize_unknown(container)),
        }
    }

    fn serialize_book(&mut self, container: &ContainerNode) -> Value {
        let mut code = container
            .special_span
            .as_ref()
            .map(|span| self.slice(span).trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.book_code.clone());
        if code == "MAT" && source_uses_alternate_texts_book_code(self.source) {
            code = "XXA".to_string();
        }
        if self.book_code.is_empty() || source_uses_alternate_texts_book_code(self.source) {
            self.book_code = code.clone();
        }

        let content_trim = if self.preserve_vertical_whitespace {
            ContentTrim::none(true)
        } else {
            ContentTrim::container(false)
        };
        let content = self.serialize_children(container.children.as_slice(), content_trim);
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("book".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(&container.marker_span),
                marker_horizontal_suffix(self.source, &container.marker_span)
            );
            map.insert("markerText".to_string(), Value::String(marker_text));
        }
        map.insert("code".to_string(), Value::String(code));
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_book(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let mut code = container
            .special_span
            .as_ref()
            .map(|span| self.slice(span).trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.book_code.clone());
        if code == "MAT" && source_uses_alternate_texts_book_code(self.source) {
            code = "XXA".to_string();
        }
        if self.book_code.is_empty() || source_uses_alternate_texts_book_code(self.source) {
            self.book_code = code.clone();
        }

        let content_trim = if self.preserve_vertical_whitespace {
            ContentTrim::none(true)
        } else {
            ContentTrim::container(false)
        };
        let content = self.serialize_tree_children(container.children.as_slice(), content_trim);
        let mut extra = BTreeMap::new();
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(&container.marker_span),
                marker_horizontal_suffix(self.source, &container.marker_span)
            );
            extra.insert("markerText".to_string(), Value::String(marker_text));
        }
        DocumentTreeNode::Element(DocumentTreeElement::Book {
            marker: container.marker.clone(),
            code,
            content,
            extra,
        })
    }

    fn serialize_para(&mut self, container: &ContainerNode) -> Value {
        let content = self.serialize_children(
            container.children.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("para".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(&container.marker_span),
                marker_horizontal_suffix(self.source, &container.marker_span)
            );
            map.insert("markerText".to_string(), Value::String(marker_text));
        }
        if !(container.marker == "b" && content.is_empty()) {
            map.insert("content".to_string(), Value::Array(content));
        }
        Value::Object(map)
    }

    fn serialize_tree_para(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let content = self.serialize_tree_children(
            container.children.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        let mut extra = BTreeMap::new();
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(&container.marker_span),
                marker_horizontal_suffix(self.source, &container.marker_span)
            );
            extra.insert("markerText".to_string(), Value::String(marker_text));
        }
        DocumentTreeNode::Element(DocumentTreeElement::Para {
            marker: container.marker.clone(),
            content: if container.marker == "b" && content.is_empty() {
                Vec::new()
            } else {
                content
            },
            extra,
        })
    }

    fn serialize_character(&mut self, container: &ContainerNode) -> Value {
        let marker_info = lookup_marker(container.marker.as_str());
        let explicitly_closed = marker_has_explicit_close(container, self.source);
        let close_suffix = explicit_close_horizontal_suffix(container, self.source);
        let inline_trim = if marker_info.valid_in_note {
            ContentTrim::note_inline(self.preserve_vertical_whitespace)
        } else {
            ContentTrim::inline(self.preserve_vertical_whitespace)
        };
        let mut content = self.serialize_children(container.children.as_slice(), inline_trim);
        let attr_gap_before =
            attribute_spans_have_leading_gap(self.source, &container.attribute_spans);
        let content_has_trailing_whitespace =
            content_ends_with_whitespace(container.children.as_slice(), self.source);
        let attr_behavior = resolve_attribute_behavior(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
            container.children.as_slice(),
        );
        if marker_info.valid_in_note
            && needs_close_gap_prefix(container.marker.as_str())
            && marker_follows_closing_marker(self.source, &container.marker_span)
        {
            prefix_first_descendant_string(&mut content, " ");
        }
        if should_trim_char_close_gap(container, self.source)
            && content_ends_with_whitespace(container.children.as_slice(), self.source)
        {
            trim_last_descendant_string_end(&mut content);
        }
        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str())
            || (content_ends_with_newline_node(container.children.as_slice())
                && !marker_info.valid_in_note)
        {
            trim_last_descendant_string_end(&mut content);
        }
        let attrs = match attr_behavior {
            AttributeBehavior::Flatten(attrs) => attrs,
            AttributeBehavior::RawText(raw) => {
                push_text_segments(
                    &mut content,
                    &normalize_preserved_raw_attributes(
                        container.marker.as_str(),
                        &raw,
                        attr_gap_before,
                        content_has_trailing_whitespace,
                    ),
                );
                Vec::new()
            }
        };
        if container.marker == "ref" {
            let mut map = Map::new();
            map.insert("type".to_string(), Value::String("ref".to_string()));
            flatten_attributes(&mut map, attrs);
            map.insert("content".to_string(), Value::Array(content));
            return Value::Object(map);
        }

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("char".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        if self.preserve_vertical_whitespace {
            map.insert("closed".to_string(), Value::Bool(explicitly_closed));
            if !close_suffix.is_empty() {
                map.insert("closeSuffix".to_string(), Value::String(close_suffix));
            }
        }
        flatten_attributes(&mut map, attrs);
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_character(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let marker_info = lookup_marker(container.marker.as_str());
        let explicitly_closed = marker_has_explicit_close(container, self.source);
        let close_suffix = explicit_close_horizontal_suffix(container, self.source);
        let inline_trim = if marker_info.valid_in_note {
            ContentTrim::note_inline(self.preserve_vertical_whitespace)
        } else {
            ContentTrim::inline(self.preserve_vertical_whitespace)
        };
        let mut content = self.serialize_tree_children(container.children.as_slice(), inline_trim);
        let attr_gap_before =
            attribute_spans_have_leading_gap(self.source, &container.attribute_spans);
        let content_has_trailing_whitespace =
            content_ends_with_whitespace(container.children.as_slice(), self.source);
        let attr_behavior = resolve_attribute_behavior(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
            container.children.as_slice(),
        );
        if marker_info.valid_in_note
            && needs_close_gap_prefix(container.marker.as_str())
            && marker_follows_closing_marker(self.source, &container.marker_span)
        {
            prefix_tree_first_descendant_string(&mut content, " ");
        }
        if should_trim_char_close_gap(container, self.source)
            && content_ends_with_whitespace(container.children.as_slice(), self.source)
        {
            trim_tree_last_descendant_string_end(&mut content);
        }
        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str())
            || (content_ends_with_newline_node(container.children.as_slice())
                && !marker_info.valid_in_note)
        {
            trim_tree_last_descendant_string_end(&mut content);
        }
        let attrs = match attr_behavior {
            AttributeBehavior::Flatten(attrs) => attrs,
            AttributeBehavior::RawText(raw) => {
                push_tree_text_segments(
                    &mut content,
                    &normalize_preserved_raw_attributes(
                        container.marker.as_str(),
                        &raw,
                        attr_gap_before,
                        content_has_trailing_whitespace,
                    ),
                );
                Vec::new()
            }
        };
        let mut extra = attributes_to_extra(attrs);
        if container.marker == "ref" {
            return DocumentTreeNode::Element(DocumentTreeElement::Ref { content, extra });
        }

        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(&container.marker_span),
                marker_horizontal_suffix(self.source, &container.marker_span)
            );
            extra.insert("markerText".to_string(), Value::String(marker_text));
            extra.insert("closed".to_string(), Value::Bool(explicitly_closed));
            if let Some(close_marker_text) = explicit_close_marker_text(container, self.source) {
                extra.insert(
                    "closeMarkerText".to_string(),
                    Value::String(close_marker_text),
                );
            }
            if !close_suffix.is_empty() {
                extra.insert("closeSuffix".to_string(), Value::String(close_suffix));
            }
        }

        DocumentTreeNode::Element(DocumentTreeElement::Char {
            marker: container.marker.clone(),
            content,
            extra,
        })
    }

    fn serialize_note(&mut self, container: &ContainerNode) -> Value {
        let caller = container
            .special_span
            .as_ref()
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        let mut content = self.serialize_children(
            filtered.as_slice(),
            ContentTrim::note_inline(self.preserve_vertical_whitespace),
        );
        if self.note_was_unclosed(&container.marker_span) {
            trim_last_descendant_string_end(&mut content);
        }
        preserve_note_continuation_spacing(&mut content);
        hoist_trailing_fv_from_fqa(&mut content);

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("note".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        map.insert("caller".to_string(), Value::String(caller));
        if self.preserve_vertical_whitespace {
            let explicitly_closed = !self.note_was_unclosed(&container.marker_span);
            map.insert("closed".to_string(), Value::Bool(explicitly_closed));
        }
        if let Some(category) = category {
            map.insert("category".to_string(), Value::String(category));
        }
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_note(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let caller = container
            .special_span
            .as_ref()
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        let mut content = self.serialize_tree_children(
            filtered.as_slice(),
            ContentTrim::note_inline(self.preserve_vertical_whitespace),
        );
        if self.note_was_unclosed(&container.marker_span) {
            trim_tree_last_descendant_string_end(&mut content);
        }
        preserve_tree_note_continuation_spacing(&mut content);
        hoist_trailing_fv_from_fqa_tree(&mut content);

        let mut extra = BTreeMap::new();
        if self.preserve_vertical_whitespace {
            let explicitly_closed = !self.note_was_unclosed(&container.marker_span);
            extra.insert("closed".to_string(), Value::Bool(explicitly_closed));
            if note_has_prefix_gap(self.source, &container.marker_span) {
                extra.insert("prefixGap".to_string(), Value::String(" ".to_string()));
            }
            let caller_suffix = note_caller_suffix(self.source, &container.marker_span, &caller);
            if caller_suffix != " " {
                extra.insert("callerSuffix".to_string(), Value::String(caller_suffix));
            }
        }
        if let Some(category) = category {
            extra.insert("category".to_string(), Value::String(category));
        }

        DocumentTreeNode::Element(DocumentTreeElement::Note {
            marker: container.marker.clone(),
            caller,
            content,
            extra,
        })
    }

    fn serialize_figure(&mut self, container: &ContainerNode) -> Value {
        let mut content = self.serialize_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        let attrs = collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        );

        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str()) {
            if !container.attribute_spans.is_empty() {
                let raw = join_attribute_spans(self.source, &container.attribute_spans);
                push_text_segments(&mut content, &normalize_text(&raw, true));
            }

            let mut map = Map::new();
            map.insert("type".to_string(), Value::String("char".to_string()));
            map.insert(
                "marker".to_string(),
                Value::String(container.marker.clone()),
            );
            map.insert("content".to_string(), Value::Array(content));
            return Value::Object(map);
        }

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("figure".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        flatten_attributes(&mut map, attrs);
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_figure(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let mut content = self.serialize_tree_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        let attrs = collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        );

        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str()) {
            if !container.attribute_spans.is_empty() {
                let raw = join_attribute_spans(self.source, &container.attribute_spans);
                push_tree_text_segments(&mut content, &normalize_text(&raw, true));
            }

            return DocumentTreeNode::Element(DocumentTreeElement::Char {
                marker: container.marker.clone(),
                content,
                extra: BTreeMap::new(),
            });
        }

        DocumentTreeNode::Element(DocumentTreeElement::Figure {
            marker: container.marker.clone(),
            content,
            extra: attributes_to_extra(attrs),
        })
    }

    fn serialize_sidebar(&mut self, container: &ContainerNode) -> Value {
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        let content = self.serialize_children(
            filtered.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("sidebar".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        if let Some(category) = category {
            map.insert("category".to_string(), Value::String(category));
        }
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_sidebar(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        let content = self.serialize_tree_children(
            filtered.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );

        let mut extra = BTreeMap::new();
        if let Some(category) = category {
            extra.insert("category".to_string(), Value::String(category));
        }

        DocumentTreeNode::Element(DocumentTreeElement::Sidebar {
            marker: container.marker.clone(),
            content,
            extra,
        })
    }

    fn serialize_periph(&mut self, container: &ContainerNode) -> Value {
        let (alt, skip_count) = extract_periph_alt(container.children.as_slice(), self.source);
        let content = self.serialize_children(
            container.children.get(skip_count..).unwrap_or(&[]),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        let attrs = collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        );

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("periph".to_string()));
        if let Some(alt) = alt {
            map.insert("alt".to_string(), Value::String(alt));
        }
        flatten_attributes(&mut map, attrs);
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_periph(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let (alt, skip_count) = extract_periph_alt(container.children.as_slice(), self.source);
        let content = self.serialize_tree_children(
            container.children.get(skip_count..).unwrap_or(&[]),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        let mut extra = attributes_to_extra(collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        ));
        if let Some(alt) = alt {
            extra.insert("alt".to_string(), Value::String(alt));
        }

        DocumentTreeNode::Element(DocumentTreeElement::Periph { content, extra })
    }

    fn serialize_table_row(&mut self, container: &ContainerNode) -> Value {
        let content = self.serialize_children(
            container.children.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("table:row".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_table_row(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let content = self.serialize_tree_children(
            container.children.as_slice(),
            ContentTrim::container(self.preserve_vertical_whitespace),
        );
        DocumentTreeNode::Element(DocumentTreeElement::TableRow {
            marker: container.marker.clone(),
            content,
            extra: BTreeMap::new(),
        })
    }

    fn serialize_table_cell(&mut self, container: &ContainerNode) -> Value {
        let content = self.serialize_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("table:cell".to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        map.insert(
            "align".to_string(),
            Value::String(table_cell_alignment(container.marker.as_str())),
        );
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_table_cell(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let content = self.serialize_tree_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        DocumentTreeNode::Element(DocumentTreeElement::TableCell {
            marker: container.marker.clone(),
            align: table_cell_alignment(container.marker.as_str()),
            content,
            extra: BTreeMap::new(),
        })
    }

    fn serialize_unknown(&mut self, container: &ContainerNode) -> Value {
        let content = self.serialize_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        let mut map = Map::new();
        let node_type = if container.marker == "esbe" || container.marker == "*" {
            "unmatched"
        } else {
            "unknown"
        };
        map.insert("type".to_string(), Value::String(node_type.to_string()));
        map.insert(
            "marker".to_string(),
            Value::String(container.marker.clone()),
        );
        map.insert("content".to_string(), Value::Array(content));
        Value::Object(map)
    }

    fn serialize_tree_unknown(&mut self, container: &ContainerNode) -> DocumentTreeNode {
        let content = self.serialize_tree_children(
            container.children.as_slice(),
            ContentTrim::inline(self.preserve_vertical_whitespace),
        );
        let node_type = if container.marker == "esbe" || container.marker == "*" {
            DocumentTreeElement::Unmatched {
                marker: container.marker.clone(),
                content,
                extra: BTreeMap::new(),
            }
        } else {
            DocumentTreeElement::Unknown {
                marker: container.marker.clone(),
                content,
                extra: BTreeMap::new(),
            }
        };
        DocumentTreeNode::Element(node_type)
    }

    fn serialize_chapter(
        &mut self,
        marker_span: &std::ops::Range<usize>,
        number_span: Option<&std::ops::Range<usize>>,
        altnumber: Option<String>,
        pubnumber: Option<String>,
    ) -> Value {
        let marker = self.slice(marker_span).trim_start_matches('\\').to_string();
        let number = number_span
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();
        self.current_chapter = strip_leading_zeros(&number);
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("chapter".to_string()));
        map.insert("marker".to_string(), Value::String(marker));
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(marker_span),
                marker_horizontal_suffix(self.source, marker_span)
            );
            map.insert("markerText".to_string(), Value::String(marker_text));
        }
        map.insert("number".to_string(), Value::String(number));
        let sid = if self.current_chapter.is_empty() {
            String::new()
        } else if self.chapter_sid_uses_zero_verse {
            format!("{} {}:0", self.book_code, self.current_chapter)
        } else {
            format!("{} {}", self.book_code, self.current_chapter)
        };
        if self.emit_sid {
            map.insert("sid".to_string(), Value::String(sid));
        }
        if let Some(altnumber) = altnumber {
            map.insert("altnumber".to_string(), Value::String(altnumber));
        }
        if let Some(pubnumber) = pubnumber {
            map.insert("pubnumber".to_string(), Value::String(pubnumber));
        }
        Value::Object(map)
    }

    fn serialize_tree_chapter(
        &mut self,
        marker_span: &std::ops::Range<usize>,
        number_span: Option<&std::ops::Range<usize>>,
        altnumber: Option<String>,
        pubnumber: Option<String>,
    ) -> DocumentTreeNode {
        let marker = self.slice(marker_span).trim_start_matches('\\').to_string();
        let number = number_span
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();
        self.current_chapter = strip_leading_zeros(&number);
        let mut extra = BTreeMap::new();
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(marker_span),
                marker_horizontal_suffix(self.source, marker_span)
            );
            extra.insert("markerText".to_string(), Value::String(marker_text));
        }
        let sid = if self.current_chapter.is_empty() {
            String::new()
        } else if self.chapter_sid_uses_zero_verse {
            format!("{} {}:0", self.book_code, self.current_chapter)
        } else {
            format!("{} {}", self.book_code, self.current_chapter)
        };
        if self.emit_sid {
            extra.insert("sid".to_string(), Value::String(sid));
        }
        if let Some(altnumber) = altnumber {
            extra.insert("altnumber".to_string(), Value::String(altnumber));
        }
        if let Some(pubnumber) = pubnumber {
            extra.insert("pubnumber".to_string(), Value::String(pubnumber));
        }
        DocumentTreeNode::Element(DocumentTreeElement::Chapter {
            marker,
            number,
            extra,
        })
    }

    fn serialize_verse(
        &mut self,
        marker_span: &std::ops::Range<usize>,
        number_span: Option<&std::ops::Range<usize>>,
        altnumber: Option<String>,
        pubnumber: Option<String>,
    ) -> Value {
        let marker = self.slice(marker_span).trim_start_matches('\\').to_string();
        let number = number_span
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();

        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("verse".to_string()));
        map.insert("marker".to_string(), Value::String(marker));
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(marker_span),
                marker_horizontal_suffix(self.source, marker_span)
            );
            map.insert("markerText".to_string(), Value::String(marker_text));
        }
        map.insert("number".to_string(), Value::String(number.clone()));

        let sid =
            if !self.book_code.is_empty() && !self.current_chapter.is_empty() && !number.is_empty()
            {
                format!(
                    "{} {}:{}",
                    self.book_code,
                    self.current_chapter,
                    strip_leading_zeros(&number)
                )
            } else {
                String::new()
            };
        if self.emit_sid {
            map.insert("sid".to_string(), Value::String(sid));
        }
        if let Some(altnumber) = altnumber {
            map.insert("altnumber".to_string(), Value::String(altnumber));
        }
        if let Some(pubnumber) = pubnumber {
            map.insert("pubnumber".to_string(), Value::String(pubnumber));
        }

        Value::Object(map)
    }

    fn serialize_tree_verse(
        &mut self,
        marker_span: &std::ops::Range<usize>,
        number_span: Option<&std::ops::Range<usize>>,
        altnumber: Option<String>,
        pubnumber: Option<String>,
    ) -> DocumentTreeNode {
        let marker = self.slice(marker_span).trim_start_matches('\\').to_string();
        let number = number_span
            .map(|span| self.slice(span).trim().to_string())
            .unwrap_or_default();

        let mut extra = BTreeMap::new();
        if self.preserve_vertical_whitespace {
            let marker_text = format!(
                "{}{}",
                self.slice(marker_span),
                marker_horizontal_suffix(self.source, marker_span)
            );
            extra.insert("markerText".to_string(), Value::String(marker_text));
        }
        let sid =
            if !self.book_code.is_empty() && !self.current_chapter.is_empty() && !number.is_empty()
            {
                format!(
                    "{} {}:{}",
                    self.book_code,
                    self.current_chapter,
                    strip_leading_zeros(&number)
                )
            } else {
                String::new()
            };
        if self.emit_sid {
            extra.insert("sid".to_string(), Value::String(sid));
        }
        if let Some(altnumber) = altnumber {
            extra.insert("altnumber".to_string(), Value::String(altnumber));
        }
        if let Some(pubnumber) = pubnumber {
            extra.insert("pubnumber".to_string(), Value::String(pubnumber));
        }

        DocumentTreeNode::Element(DocumentTreeElement::Verse {
            marker,
            number,
            extra,
        })
    }

    fn serialize_milestone(
        &self,
        marker: &str,
        attribute_spans: &[std::ops::Range<usize>],
    ) -> Value {
        let attrs = collect_attributes(self.source, marker, attribute_spans);
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("ms".to_string()));
        map.insert("marker".to_string(), Value::String(marker.to_string()));
        flatten_attributes(&mut map, attrs);
        Value::Object(map)
    }

    fn serialize_tree_milestone(
        &self,
        marker: &str,
        attribute_spans: &[std::ops::Range<usize>],
    ) -> DocumentTreeNode {
        DocumentTreeNode::Element(DocumentTreeElement::Milestone {
            marker: marker.to_string(),
            extra: attributes_to_extra(collect_attributes(self.source, marker, attribute_spans)),
        })
    }

    fn serialize_unclosed_milestone_into(
        &self,
        marker_span: &std::ops::Range<usize>,
        attribute_spans: &[std::ops::Range<usize>],
        out: &mut Vec<Value>,
    ) {
        let mut raw = self.slice(marker_span).to_string();
        raw.push_str(&join_attribute_spans(self.source, attribute_spans));
        push_text_segments(out, &normalize_text(&raw, true));
    }

    fn serialize_tree_unclosed_milestone_into(
        &self,
        marker_span: &std::ops::Range<usize>,
        attribute_spans: &[std::ops::Range<usize>],
        out: &mut Vec<DocumentTreeNode>,
    ) {
        let mut raw = self.slice(marker_span).to_string();
        raw.push_str(&join_attribute_spans(self.source, attribute_spans));
        push_tree_text_segments(out, &normalize_text(&raw, true));
    }

    fn serialize_leaf_into(
        &self,
        kind: LeafKind,
        span: &std::ops::Range<usize>,
        preserve_newlines: bool,
        out: &mut Vec<Value>,
    ) {
        let text = match kind {
            LeafKind::Text | LeafKind::Whitespace | LeafKind::Attributes => {
                normalize_text(self.slice(span), preserve_newlines)
            }
            LeafKind::OptBreak => {
                push_value(
                    out,
                    Value::Object(Map::from_iter([(
                        "type".to_string(),
                        Value::String("optbreak".to_string()),
                    )])),
                );
                return;
            }
            LeafKind::Newline => {
                if preserve_newlines {
                    push_value(
                        out,
                        Value::Object(Map::from_iter([(
                            "type".to_string(),
                            Value::String("linebreak".to_string()),
                        )])),
                    );
                    return;
                } else {
                    normalize_text(self.slice(span), false)
                }
            }
        };
        push_text_segments(out, &text);
    }

    fn serialize_tree_leaf_into(
        &self,
        kind: LeafKind,
        span: &std::ops::Range<usize>,
        preserve_newlines: bool,
        out: &mut Vec<DocumentTreeNode>,
    ) {
        let text = match kind {
            LeafKind::Text | LeafKind::Whitespace | LeafKind::Attributes => {
                normalize_text(self.slice(span), preserve_newlines)
            }
            LeafKind::OptBreak => {
                push_tree_node(
                    out,
                    DocumentTreeNode::Element(DocumentTreeElement::OptBreak {}),
                );
                return;
            }
            LeafKind::Newline => {
                if preserve_newlines {
                    push_tree_node(
                        out,
                        DocumentTreeNode::Element(DocumentTreeElement::LineBreak {
                            value: self.slice(span).to_string(),
                        }),
                    );
                    return;
                } else {
                    normalize_text(self.slice(span), false)
                }
            }
        };
        push_tree_text_segments(out, &text);
    }

    fn slice(&self, span: &std::ops::Range<usize>) -> &str {
        &self.source[span.clone()]
    }

    fn is_ignorable_metadata_trivia(&self, node: &Node) -> bool {
        if self.preserve_vertical_whitespace
            && matches!(
                node,
                Node::Leaf {
                    kind: LeafKind::Newline,
                    ..
                }
            )
        {
            return false;
        }

        is_ignorable_trivia_node(node, self.source)
    }

    fn note_was_unclosed(&self, marker_span: &std::ops::Range<usize>) -> bool {
        self.recoveries.iter().any(|recovery| {
            recovery.code == RecoveryCode::UnclosedNote && recovery.span == *marker_span
        })
    }

    fn marker_was_unclosed(&self, marker_span: &std::ops::Range<usize>, marker: &str) -> bool {
        self.recoveries.iter().any(|recovery| {
            ((recovery.code == RecoveryCode::UnclosedMarkerAtEof && recovery.span == *marker_span)
                && matches!(
                    recovery.payload.as_ref(),
                    Some(RecoveryPayload::Marker { marker: recovery_marker }) if recovery_marker == marker
                ))
                || (recovery.code == RecoveryCode::ImplicitlyClosedMarker
                    && recovery.related_span.as_ref() == Some(marker_span)
                    && matches!(
                        recovery.payload.as_ref(),
                        Some(RecoveryPayload::Close { open, .. }) if open == marker
                    ))
        })
    }
}

impl UsjSerializerOptions {
    pub(crate) fn for_usj(handle: &ParseHandle) -> Self {
        Self {
            emit_sid: should_emit_sid(handle.source()),
            preserve_vertical_whitespace: false,
            chapter_sid_uses_zero_verse: false,
        }
    }

    pub(crate) const fn for_document_tree() -> Self {
        Self {
            emit_sid: true,
            preserve_vertical_whitespace: true,
            chapter_sid_uses_zero_verse: true,
        }
    }
}

fn push_value(out: &mut Vec<Value>, value: Value) {
    match value {
        Value::String(text) => {
            if let Some(Value::String(previous)) = out.last_mut() {
                previous.push_str(&text);
            } else if !text.is_empty() {
                out.push(Value::String(text));
            }
        }
        other => out.push(other),
    }
}

fn push_tree_node(out: &mut Vec<DocumentTreeNode>, node: DocumentTreeNode) {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Text { value }) => {
            if value.is_empty() {
                return;
            }
            if let Some(previous) = out.last_mut().and_then(tree_text_mut) {
                previous.push_str(&value);
            } else {
                out.push(DocumentTreeNode::Element(DocumentTreeElement::Text {
                    value,
                }));
            }
        }
        other => out.push(other),
    }
}

fn append_usj_nodes(out: &mut Vec<UsjNode>, nodes: &[DocumentTreeNode]) {
    for node in nodes {
        let DocumentTreeNode::Element(element) = node;
        match element {
            DocumentTreeElement::Text { value } => push_usj_text(out, value),
            DocumentTreeElement::LineBreak { .. } => {}
            DocumentTreeElement::OptBreak {} => {
                out.push(UsjNode::Element(UsjElement::OptBreak {}));
            }
            other => out.push(UsjNode::Element(document_tree_element_to_usj_element(
                other,
            ))),
        }
    }
}

fn push_usj_text(out: &mut Vec<UsjNode>, text: &str) {
    if text.is_empty() {
        return;
    }

    match out.last_mut() {
        Some(UsjNode::Text(previous)) => previous.push_str(text),
        _ => out.push(UsjNode::Text(text.to_string())),
    }
}

fn document_tree_element_to_usj_element(element: &DocumentTreeElement) -> UsjElement {
    match element {
        DocumentTreeElement::Text { value: _ } => {
            unreachable!("text is handled as a UsjNode::Text")
        }
        DocumentTreeElement::Book {
            marker,
            code,
            content,
            extra,
        } => UsjElement::Book {
            marker: marker.clone(),
            code: code.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Chapter {
            marker,
            number,
            extra,
        } => UsjElement::Chapter {
            marker: marker.clone(),
            number: number.clone(),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Verse {
            marker,
            number,
            extra,
        } => UsjElement::Verse {
            marker: marker.clone(),
            number: number.clone(),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Para {
            marker,
            content,
            extra,
        } => UsjElement::Para {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Char {
            marker,
            content,
            extra,
        } => UsjElement::Char {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Note {
            marker,
            caller,
            content,
            extra,
        } => UsjElement::Note {
            marker: marker.clone(),
            caller: caller.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Milestone { marker, extra } => UsjElement::Milestone {
            marker: marker.clone(),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Figure {
            marker,
            content,
            extra,
        } => UsjElement::Figure {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Sidebar {
            marker,
            content,
            extra,
        } => UsjElement::Sidebar {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Periph { content, extra } => UsjElement::Periph {
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Table { content, extra } => UsjElement::Table {
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::TableRow {
            marker,
            content,
            extra,
        } => UsjElement::TableRow {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::TableCell {
            marker,
            align,
            content,
            extra,
        } => UsjElement::TableCell {
            marker: marker.clone(),
            align: align.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Ref { content, extra } => UsjElement::Ref {
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Unknown {
            marker,
            content,
            extra,
        } => UsjElement::Unknown {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::Unmatched {
            marker,
            content,
            extra,
        } => UsjElement::Unmatched {
            marker: marker.clone(),
            content: document_tree_children_to_usj_nodes(content),
            extra: semantic_extra(extra),
        },
        DocumentTreeElement::OptBreak {} => UsjElement::OptBreak {},
        DocumentTreeElement::LineBreak { .. } => {
            unreachable!("linebreak nodes are not emitted in semantic USJ")
        }
    }
}

fn document_tree_children_to_usj_nodes(children: &[DocumentTreeNode]) -> Vec<UsjNode> {
    let mut out = Vec::new();
    append_usj_nodes(&mut out, children);
    out
}

fn semantic_extra(
    extra: &std::collections::BTreeMap<String, Value>,
) -> std::collections::BTreeMap<String, Value> {
    extra
        .iter()
        .filter(|(key, _)| key.as_str() != "markerText" && key.as_str() != "closed")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn push_text_segments(out: &mut Vec<Value>, text: &str) {
    if text.is_empty() {
        return;
    }

    if !text.contains("//") {
        push_value(out, Value::String(text.to_string()));
        return;
    }

    let mut remainder = text;
    while let Some(index) = remainder.find("//") {
        if index > 0 {
            push_value(out, Value::String(remainder[..index].to_string()));
        }
        out.push(Value::Object(Map::from_iter([(
            "type".to_string(),
            Value::String("optbreak".to_string()),
        )])));
        remainder = &remainder[index + 2..];
    }

    if !remainder.is_empty() {
        push_value(out, Value::String(remainder.to_string()));
    }
}

fn push_tree_text_segments(out: &mut Vec<DocumentTreeNode>, text: &str) {
    if text.is_empty() {
        return;
    }

    if !text.contains("//") {
        push_tree_node(
            out,
            DocumentTreeNode::Element(DocumentTreeElement::Text {
                value: text.to_string(),
            }),
        );
        return;
    }

    let mut remainder = text;
    while let Some(index) = remainder.find("//") {
        if index > 0 {
            push_tree_node(
                out,
                DocumentTreeNode::Element(DocumentTreeElement::Text {
                    value: remainder[..index].to_string(),
                }),
            );
        }
        out.push(DocumentTreeNode::Element(DocumentTreeElement::OptBreak {}));
        remainder = &remainder[index + 2..];
    }

    if !remainder.is_empty() {
        push_tree_node(
            out,
            DocumentTreeNode::Element(DocumentTreeElement::Text {
                value: remainder.to_string(),
            }),
        );
    }
}

fn normalize_content(values: &mut Vec<Value>, trim: ContentTrim) {
    if trim.trim_first_string_start {
        while let Some(index) = values.iter().position(Value::is_string) {
            let Some(text) = values[index].as_str() else {
                break;
            };
            let trimmed = if index == 0 {
                trim_ascii_start(text).to_string()
            } else {
                text.to_string()
            };
            if trimmed.is_empty() {
                values.remove(index);
                continue;
            }
            values[index] = Value::String(trimmed);
            break;
        }
    }

    if trim.trim_last_string_end {
        while matches!(values.last(), Some(Value::String(_))) {
            let index = values.len() - 1;
            let Some(text) = values[index].as_str() else {
                break;
            };
            let trimmed = trim_ascii_end(text).to_string();
            if trimmed.is_empty() {
                values.pop();
                continue;
            }
            values[index] = Value::String(trimmed);
            break;
        }
    }

    for index in 0..values.len() {
        let Some(text) = values[index].as_str() else {
            continue;
        };
        let mut normalized = if trim.preserve_newlines || text.contains('\n') {
            text.to_string()
        } else {
            collapse_spaces(text)
        };
        if trim.trim_leading_after_chapter_or_verse
            && index > 0
            && let Some(previous_type) = values[index - 1]
                .as_object()
                .and_then(|object| object.get("type"))
                .and_then(Value::as_str)
            && matches!(previous_type, "chapter" | "verse")
        {
            normalized = trim_ascii_start(&normalized).to_string();
        }
        values[index] = Value::String(normalized);
    }

    values.retain(|value| !matches!(value, Value::String(text) if text.trim().is_empty()));
}

fn normalize_tree_content(values: &mut Vec<DocumentTreeNode>, trim: ContentTrim) {
    if trim.trim_first_string_start {
        while let Some(index) = values.iter().position(tree_node_is_text) {
            let Some(text) = tree_text(values.get(index).expect("index should exist")) else {
                break;
            };
            let trimmed = if index == 0 {
                trim_ascii_start(text).to_string()
            } else {
                text.to_string()
            };
            if trimmed.is_empty() {
                values.remove(index);
                continue;
            }
            if let Some(text) = values.get_mut(index).and_then(tree_text_mut) {
                *text = trimmed;
            }
            break;
        }
    }

    if trim.trim_last_string_end {
        while values.last().is_some_and(tree_node_is_text) {
            let index = values.len() - 1;
            let Some(text) = values.get(index).and_then(tree_text) else {
                break;
            };
            let trimmed = trim_ascii_end(text).to_string();
            if trimmed.is_empty() {
                values.pop();
                continue;
            }
            if let Some(text) = values.get_mut(index).and_then(tree_text_mut) {
                *text = trimmed;
            }
            break;
        }
    }

    for index in 0..values.len() {
        let Some(text) = values.get(index).and_then(tree_text) else {
            continue;
        };
        let mut normalized = if trim.preserve_newlines || text.contains('\n') {
            text.to_string()
        } else {
            collapse_spaces(text)
        };
        if trim.trim_leading_after_chapter_or_verse
            && index > 0
            && values
                .get(index - 1)
                .and_then(tree_node_type)
                .is_some_and(|node_type| matches!(node_type, "chapter" | "verse"))
        {
            normalized = trim_ascii_start(&normalized).to_string();
        }
        if let Some(text) = values.get_mut(index).and_then(tree_text_mut) {
            *text = normalized;
        }
    }

    values.retain(|value| !matches!(tree_text(value), Some(text) if text.trim().is_empty()));
}

fn attributes_to_extra(attrs: Vec<(String, String)>) -> BTreeMap<String, Value> {
    attrs
        .into_iter()
        .map(|(key, value)| (key, Value::String(value)))
        .collect()
}

fn tree_node_is_text(node: &DocumentTreeNode) -> bool {
    matches!(
        node,
        DocumentTreeNode::Element(DocumentTreeElement::Text { .. })
    )
}

fn tree_text(node: &DocumentTreeNode) -> Option<&str> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Text { value }) => Some(value.as_str()),
        _ => None,
    }
}

fn tree_text_mut(node: &mut DocumentTreeNode) -> Option<&mut String> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Text { value }) => Some(value),
        _ => None,
    }
}

fn tree_node_type(node: &DocumentTreeNode) -> Option<&'static str> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Book { .. }) => Some("book"),
        DocumentTreeNode::Element(DocumentTreeElement::Chapter { .. }) => Some("chapter"),
        DocumentTreeNode::Element(DocumentTreeElement::Verse { .. }) => Some("verse"),
        DocumentTreeNode::Element(DocumentTreeElement::Para { .. }) => Some("para"),
        DocumentTreeNode::Element(DocumentTreeElement::Char { .. }) => Some("char"),
        DocumentTreeNode::Element(DocumentTreeElement::Note { .. }) => Some("note"),
        DocumentTreeNode::Element(DocumentTreeElement::Milestone { .. }) => Some("ms"),
        DocumentTreeNode::Element(DocumentTreeElement::Figure { .. }) => Some("figure"),
        DocumentTreeNode::Element(DocumentTreeElement::Sidebar { .. }) => Some("sidebar"),
        DocumentTreeNode::Element(DocumentTreeElement::Periph { .. }) => Some("periph"),
        DocumentTreeNode::Element(DocumentTreeElement::Table { .. }) => Some("table"),
        DocumentTreeNode::Element(DocumentTreeElement::TableRow { .. }) => Some("table:row"),
        DocumentTreeNode::Element(DocumentTreeElement::TableCell { .. }) => Some("table:cell"),
        DocumentTreeNode::Element(DocumentTreeElement::Ref { .. }) => Some("ref"),
        DocumentTreeNode::Element(DocumentTreeElement::Unknown { .. }) => Some("unknown"),
        DocumentTreeNode::Element(DocumentTreeElement::Unmatched { .. }) => Some("unmatched"),
        DocumentTreeNode::Element(DocumentTreeElement::OptBreak { .. }) => Some("optbreak"),
        DocumentTreeNode::Element(DocumentTreeElement::LineBreak { .. }) => Some("linebreak"),
        DocumentTreeNode::Element(DocumentTreeElement::Text { .. }) => Some("text"),
    }
}

fn tree_marker(node: &DocumentTreeNode) -> Option<&str> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Book { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Chapter { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Verse { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Para { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Char { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Note { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Milestone { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Figure { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Sidebar { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableRow { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableCell { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unknown { marker, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unmatched { marker, .. }) => {
            Some(marker.as_str())
        }
        _ => None,
    }
}

fn tree_content(node: &DocumentTreeNode) -> Option<&[DocumentTreeNode]> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Book { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Para { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Char { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Note { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Figure { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Sidebar { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Periph { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Table { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableRow { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableCell { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Ref { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unknown { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unmatched { content, .. }) => {
            Some(content.as_slice())
        }
        _ => None,
    }
}

fn tree_content_mut(node: &mut DocumentTreeNode) -> Option<&mut Vec<DocumentTreeNode>> {
    match node {
        DocumentTreeNode::Element(DocumentTreeElement::Book { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Para { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Char { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Note { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Figure { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Sidebar { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Periph { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Table { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableRow { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::TableCell { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Ref { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unknown { content, .. })
        | DocumentTreeNode::Element(DocumentTreeElement::Unmatched { content, .. }) => {
            Some(content)
        }
        _ => None,
    }
}

fn trim_tree_last_descendant_string_end(values: &mut [DocumentTreeNode]) {
    let Some(last) = values.last_mut() else {
        return;
    };
    trim_tree_last_string_end(last);
}

fn trim_tree_last_string_end(value: &mut DocumentTreeNode) {
    if let Some(text) = tree_text_mut(value) {
        *text = trim_ascii_end(text).to_string();
        return;
    }
    if let Some(content) = tree_content_mut(value) {
        trim_tree_last_descendant_string_end(content);
    }
}

fn prefix_tree_first_descendant_string(values: &mut [DocumentTreeNode], prefix: &str) {
    for value in values {
        if prefix_tree_first_string(value, prefix) {
            return;
        }
    }
}

fn prefix_tree_first_string(value: &mut DocumentTreeNode, prefix: &str) -> bool {
    if let Some(text) = tree_text_mut(value) {
        text.insert_str(0, prefix);
        return true;
    }
    if let Some(content) = tree_content_mut(value) {
        for child in content {
            if prefix_tree_first_string(child, prefix) {
                return true;
            }
        }
    }
    false
}

fn preserve_tree_note_continuation_spacing(content: &mut [DocumentTreeNode]) {
    for index in 1..content.len() {
        let previous_marker = tree_marker(&content[index - 1]);
        let current_marker = tree_marker(&content[index]);

        if !(previous_marker == Some("fqa") && current_marker == Some("ft")) {
            continue;
        }

        if tree_first_descendant_string_starts_with_trimmed_joining_punctuation(&content[index])
            || tree_first_descendant_string_starts_with_trimmed_char(&content[index], ',')
        {
            trim_tree_first_descendant_string_start(&mut content[index]);
        } else if tree_first_descendant_string_starts_with_trimmed_period_then_quote(
            &content[index],
        ) {
            continue;
        } else if tree_first_descendant_string_starts_with_trimmed_period(&content[index]) {
            if !tree_last_descendant_string_ends_with_whitespace(&content[index - 1])
                && !tree_first_descendant_string_starts_with_whitespace(&content[index])
            {
                ensure_tree_last_descendant_string_suffix(&mut content[index - 1], " ");
                prefix_tree_first_descendant_string(std::slice::from_mut(&mut content[index]), " ");
            }
        } else if tree_first_descendant_string_starts_with_trimmed_word(&content[index])
            && !tree_first_descendant_string_starts_with_whitespace(&content[index])
            && (tree_last_descendant_string_ends_with_trimmed_char(&content[index - 1], ',')
                || !tree_last_descendant_string_ends_with_whitespace(&content[index - 1]))
        {
            prefix_tree_first_descendant_string(std::slice::from_mut(&mut content[index]), " ");
        }
    }
}

fn hoist_trailing_fv_from_fqa_tree(content: &mut Vec<DocumentTreeNode>) {
    let mut normalized = Vec::with_capacity(content.len());

    for value in content.drain(..) {
        let Some((before, fv, after)) = split_tree_fqa_with_trailing_fv(&value) else {
            normalized.push(value);
            continue;
        };

        if !before.is_empty() {
            let mut fqa = value.clone();
            if let Some(content) = tree_content_mut(&mut fqa) {
                *content = before;
            }
            normalized.push(fqa);
        }

        normalized.push(fv);
        normalized.extend(after);
    }

    *content = normalized;
}

fn split_tree_fqa_with_trailing_fv(
    value: &DocumentTreeNode,
) -> Option<(
    Vec<DocumentTreeNode>,
    DocumentTreeNode,
    Vec<DocumentTreeNode>,
)> {
    if tree_node_type(value)? != "char" || tree_marker(value)? != "fqa" {
        return None;
    }

    let content = tree_content(value)?;
    let fv_index = content
        .iter()
        .position(|item| tree_node_type(item) == Some("char") && tree_marker(item) == Some("fv"))?;

    if fv_index + 1 >= content.len() {
        return None;
    }

    let before = content[..fv_index].to_vec();
    let fv = content[fv_index].clone();
    let after = content[fv_index + 1..].to_vec();
    Some((before, fv, after))
}

fn tree_first_descendant_string_starts_with_trimmed_joining_punctuation(
    value: &DocumentTreeNode,
) -> bool {
    if let Some(text) = tree_text(value) {
        return text
            .trim_start()
            .chars()
            .next()
            .is_some_and(is_joining_punctuation);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(tree_first_descendant_string_starts_with_trimmed_joining_punctuation)
    })
}

fn tree_first_descendant_string_starts_with_trimmed_period(value: &DocumentTreeNode) -> bool {
    if let Some(text) = tree_text(value) {
        return text.trim_start().starts_with('.');
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(tree_first_descendant_string_starts_with_trimmed_period)
    })
}

fn tree_first_descendant_string_starts_with_trimmed_char(
    value: &DocumentTreeNode,
    ch: char,
) -> bool {
    if let Some(text) = tree_text(value) {
        return text.trim_start().starts_with(ch);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(|child| tree_first_descendant_string_starts_with_trimmed_char(child, ch))
    })
}

fn tree_first_descendant_string_starts_with_trimmed_period_then_quote(
    value: &DocumentTreeNode,
) -> bool {
    if let Some(text) = tree_text(value) {
        let mut chars = text.trim_start().chars();
        return chars.next() == Some('.')
            && chars
                .next()
                .is_some_and(|ch| matches!(ch, '"' | '\'' | '”' | '’'));
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(tree_first_descendant_string_starts_with_trimmed_period_then_quote)
    })
}

fn tree_first_descendant_string_starts_with_trimmed_word(value: &DocumentTreeNode) -> bool {
    if let Some(text) = tree_text(value) {
        return text
            .trim_start()
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(tree_first_descendant_string_starts_with_trimmed_word)
    })
}

fn tree_first_descendant_string_starts_with_whitespace(value: &DocumentTreeNode) -> bool {
    if let Some(text) = tree_text(value) {
        return text.chars().next().is_some_and(char::is_whitespace);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .any(tree_first_descendant_string_starts_with_whitespace)
    })
}

fn trim_tree_first_descendant_string_start(value: &mut DocumentTreeNode) {
    if let Some(text) = tree_text_mut(value) {
        *text = text.trim_start().to_string();
        return;
    }
    if let Some(content) = tree_content_mut(value) {
        for child in content {
            trim_tree_first_descendant_string_start(child);
            if !matches!(tree_text(child), Some(text) if text.is_empty()) {
                break;
            }
        }
    }
}

fn ensure_tree_last_descendant_string_suffix(value: &mut DocumentTreeNode, suffix: &str) {
    if !tree_last_descendant_string_has_suffix(value, suffix) {
        append_tree_last_string(value, suffix);
    }
}

fn tree_last_descendant_string_has_suffix(value: &DocumentTreeNode, suffix: &str) -> bool {
    if let Some(text) = tree_text(value) {
        return text.ends_with(suffix);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .rev()
            .any(|child| tree_last_descendant_string_has_suffix(child, suffix))
    })
}

fn tree_last_descendant_string_ends_with_trimmed_char(value: &DocumentTreeNode, ch: char) -> bool {
    if let Some(text) = tree_text(value) {
        return text.trim_end().ends_with(ch);
    }
    tree_content(value).is_some_and(|content| {
        content
            .iter()
            .rev()
            .any(|child| tree_last_descendant_string_ends_with_trimmed_char(child, ch))
    })
}

fn tree_last_descendant_string_ends_with_whitespace(value: &DocumentTreeNode) -> bool {
    if let Some(text) = tree_text(value) {
        return text.chars().last().is_some_and(char::is_whitespace);
    }
    tree_content(value)
        .and_then(|content| content.last())
        .is_some_and(tree_last_descendant_string_ends_with_whitespace)
}

fn append_tree_last_string(value: &mut DocumentTreeNode, suffix: &str) -> bool {
    if let Some(text) = tree_text_mut(value) {
        text.push_str(suffix);
        return true;
    }
    if let Some(content) = tree_content_mut(value) {
        for child in content.iter_mut().rev() {
            if append_tree_last_string(child, suffix) {
                return true;
            }
        }
    }
    false
}

fn trim_last_descendant_string_end(values: &mut [Value]) {
    let Some(last) = values.last_mut() else {
        return;
    };
    trim_last_string_end(last);
}

fn prefix_first_descendant_string(values: &mut [Value], prefix: &str) {
    for value in values {
        if prefix_first_string(value, prefix) {
            return;
        }
    }
}

fn prefix_first_string(value: &mut Value, prefix: &str) -> bool {
    match value {
        Value::String(text) => {
            text.insert_str(0, prefix);
            true
        }
        Value::Object(map) => {
            if let Some(Value::Array(content)) = map.get_mut("content") {
                for child in content {
                    if prefix_first_string(child, prefix) {
                        return true;
                    }
                }
            }
            false
        }
        Value::Array(items) => {
            for item in items {
                if prefix_first_string(item, prefix) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

fn marker_follows_closing_marker(source: &str, marker_span: &std::ops::Range<usize>) -> bool {
    marker_span.start > 0 && source.as_bytes().get(marker_span.start - 1) == Some(&b'*')
}

fn needs_close_gap_prefix(marker: &str) -> bool {
    matches!(marker, "ft" | "fdc")
}

fn should_trim_char_close_gap(container: &ContainerNode, source: &str) -> bool {
    if container.marker != "k" {
        return false;
    }

    let Some(span) = container.children.iter().rev().find_map(|node| match node {
        Node::Leaf {
            kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
            span,
        } => Some(span),
        _ => None,
    }) else {
        return false;
    };

    let close_marker = format!("\\{}*", container.marker);
    if !source[span.end..].starts_with(&close_marker) {
        return false;
    }

    let after_close = span.end + close_marker.len();
    source[after_close..]
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, ' ' | '\t'))
}

fn marker_has_explicit_close(container: &ContainerNode, source: &str) -> bool {
    matching_explicit_close_marker_text(container, source).is_some()
}

fn explicit_close_horizontal_suffix(container: &ContainerNode, source: &str) -> String {
    let Some(close_marker) = matching_explicit_close_marker_text(container, source) else {
        return String::new();
    };
    let end =
        last_descendant_end(container.children.as_slice()).unwrap_or(container.marker_span.end);
    let Some(after_close) = source[end..].strip_prefix(&close_marker) else {
        return String::new();
    };

    after_close
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .collect()
}

fn explicit_close_marker_text(container: &ContainerNode, source: &str) -> Option<String> {
    matching_explicit_close_marker_text(container, source)
}

fn matching_explicit_close_marker_text(container: &ContainerNode, source: &str) -> Option<String> {
    let end =
        last_descendant_end(container.children.as_slice()).unwrap_or(container.marker_span.end);
    let rest = source.get(end..)?;
    let canonical = format!("\\{}*", container.marker);
    if rest.starts_with(&canonical) {
        return Some(canonical);
    }

    let plus_prefixed = format!("\\+{}*", container.marker);
    rest.starts_with(&plus_prefixed).then_some(plus_prefixed)
}

fn marker_horizontal_suffix(source: &str, marker_span: &std::ops::Range<usize>) -> String {
    let Some(rest) = source.get(marker_span.end..) else {
        return String::new();
    };
    rest.chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .collect()
}

fn note_has_prefix_gap(source: &str, marker_span: &std::ops::Range<usize>) -> bool {
    marker_span.start > 0
        && source[..marker_span.start]
            .chars()
            .next_back()
            .is_some_and(|ch| matches!(ch, ' ' | '\t'))
}

fn note_caller_suffix(source: &str, marker_span: &std::ops::Range<usize>, caller: &str) -> String {
    let Some(after_marker) = source.get(marker_span.end..) else {
        return " ".to_string();
    };
    let after_marker_gap = after_marker
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();
    let Some(after_caller) = after_marker
        .get(after_marker_gap..)
        .and_then(|rest| rest.strip_prefix(caller))
    else {
        return " ".to_string();
    };
    after_caller
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .collect()
}

fn last_descendant_end(nodes: &[Node]) -> Option<usize> {
    nodes.iter().rev().find_map(last_node_end)
}

fn last_node_end(node: &Node) -> Option<usize> {
    match node {
        Node::Leaf { span, .. } => Some(span.end),
        Node::Chapter {
            number_span,
            marker_span,
        } => Some(number_span.as_ref().unwrap_or(marker_span).end),
        Node::Verse {
            number_span,
            marker_span,
        } => Some(number_span.as_ref().unwrap_or(marker_span).end),
        Node::Milestone {
            attribute_spans,
            marker_span,
            ..
        } => attribute_spans
            .last()
            .map(|span| span.end)
            .or(Some(marker_span.end)),
        Node::Container(container) => {
            last_descendant_end(container.children.as_slice()).or_else(|| {
                container
                    .attribute_spans
                    .last()
                    .map(|span| span.end)
                    .or(container.special_span.as_ref().map(|span| span.end))
                    .or(Some(container.marker_span.end))
            })
        }
    }
}

fn preserve_note_continuation_spacing(content: &mut [Value]) {
    for index in 1..content.len() {
        let previous_marker = content[index - 1]
            .as_object()
            .and_then(|object| object.get("marker"))
            .and_then(Value::as_str);
        let current_marker = content[index]
            .as_object()
            .and_then(|object| object.get("marker"))
            .and_then(Value::as_str);

        if !(previous_marker == Some("fqa") && current_marker == Some("ft")) {
            continue;
        }

        if first_descendant_string_starts_with_trimmed_joining_punctuation(&content[index])
            || first_descendant_string_starts_with_trimmed_char(&content[index], ',')
        {
            trim_first_descendant_string_start(&mut content[index]);
        } else if first_descendant_string_starts_with_trimmed_period_then_quote(&content[index]) {
            // Keep the period and quote attached to the ft segment.
        } else if first_descendant_string_starts_with_trimmed_period(&content[index]) {
            if !last_descendant_string_ends_with_whitespace(&content[index - 1])
                && !first_descendant_string_starts_with_whitespace(&content[index])
            {
                ensure_last_descendant_string_suffix(&mut content[index - 1], " ");
                prefix_first_descendant_string(std::slice::from_mut(&mut content[index]), " ");
            }
        } else if first_descendant_string_starts_with_trimmed_word(&content[index])
            && !first_descendant_string_starts_with_whitespace(&content[index])
            && (last_descendant_string_ends_with_trimmed_char(&content[index - 1], ',')
                || !last_descendant_string_ends_with_whitespace(&content[index - 1]))
        {
            prefix_first_descendant_string(std::slice::from_mut(&mut content[index]), " ");
        }
    }
}

fn hoist_trailing_fv_from_fqa(content: &mut Vec<Value>) {
    let mut normalized = Vec::with_capacity(content.len());

    for value in content.drain(..) {
        let Some((before, fv, after)) = split_fqa_with_trailing_fv(&value) else {
            normalized.push(value);
            continue;
        };

        if !before.is_empty() {
            let mut fqa = value.clone();
            if let Some(content) = fqa
                .as_object_mut()
                .and_then(|object| object.get_mut("content"))
                .and_then(Value::as_array_mut)
            {
                *content = before;
            }
            normalized.push(fqa);
        }

        normalized.push(fv);
        normalized.extend(after);
    }

    *content = normalized;
}

fn split_fqa_with_trailing_fv(value: &Value) -> Option<(Vec<Value>, Value, Vec<Value>)> {
    let object = value.as_object()?;
    if object.get("type")?.as_str()? != "char" || object.get("marker")?.as_str()? != "fqa" {
        return None;
    }

    let content = object.get("content")?.as_array()?;
    let fv_index = content.iter().position(|item| {
        item.as_object()
            .and_then(|object| object.get("type"))
            .and_then(Value::as_str)
            == Some("char")
            && item
                .as_object()
                .and_then(|object| object.get("marker"))
                .and_then(Value::as_str)
                == Some("fv")
    })?;

    if fv_index + 1 >= content.len() {
        return None;
    }

    let before = content[..fv_index].to_vec();
    let fv = content[fv_index].clone();
    let after = content[fv_index + 1..].to_vec();
    Some((before, fv, after))
}

fn first_descendant_string_starts_with_trimmed_joining_punctuation(value: &Value) -> bool {
    match value {
        Value::String(text) => text
            .trim_start()
            .chars()
            .next()
            .is_some_and(is_joining_punctuation),
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(first_descendant_string_starts_with_trimmed_joining_punctuation)
                })
        }
        Value::Array(items) => items
            .iter()
            .any(first_descendant_string_starts_with_trimmed_joining_punctuation),
        _ => false,
    }
}

fn is_joining_punctuation(ch: char) -> bool {
    matches!(ch, ';' | ':' | ')' | ']' | '}')
}

fn first_descendant_string_starts_with_trimmed_period(value: &Value) -> bool {
    match value {
        Value::String(text) => text.trim_start().starts_with('.'),
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(first_descendant_string_starts_with_trimmed_period)
                })
        }
        Value::Array(items) => items
            .iter()
            .any(first_descendant_string_starts_with_trimmed_period),
        _ => false,
    }
}

fn first_descendant_string_starts_with_trimmed_char(value: &Value, ch: char) -> bool {
    match value {
        Value::String(text) => text.trim_start().starts_with(ch),
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(|child| first_descendant_string_starts_with_trimmed_char(child, ch))
                })
        }
        Value::Array(items) => items
            .iter()
            .any(|item| first_descendant_string_starts_with_trimmed_char(item, ch)),
        _ => false,
    }
}

fn first_descendant_string_starts_with_trimmed_period_then_quote(value: &Value) -> bool {
    match value {
        Value::String(text) => {
            let mut chars = text.trim_start().chars();
            chars.next() == Some('.')
                && chars
                    .next()
                    .is_some_and(|ch| matches!(ch, '"' | '\'' | '”' | '’'))
        }
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(first_descendant_string_starts_with_trimmed_period_then_quote)
                })
        }
        Value::Array(items) => items
            .iter()
            .any(first_descendant_string_starts_with_trimmed_period_then_quote),
        _ => false,
    }
}

fn first_descendant_string_starts_with_trimmed_word(value: &Value) -> bool {
    match value {
        Value::String(text) => text
            .trim_start()
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric),
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(first_descendant_string_starts_with_trimmed_word)
                })
        }
        Value::Array(items) => items
            .iter()
            .any(first_descendant_string_starts_with_trimmed_word),
        _ => false,
    }
}

fn first_descendant_string_starts_with_whitespace(value: &Value) -> bool {
    match value {
        Value::String(text) => text.chars().next().is_some_and(char::is_whitespace),
        Value::Object(object) => {
            object
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|content| {
                    content
                        .iter()
                        .any(first_descendant_string_starts_with_whitespace)
                })
        }
        Value::Array(items) => items
            .iter()
            .any(first_descendant_string_starts_with_whitespace),
        _ => false,
    }
}

fn trim_first_descendant_string_start(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = text.trim_start().to_string();
        }
        Value::Object(map) => {
            if let Some(Value::Array(content)) = map.get_mut("content") {
                for child in content {
                    trim_first_descendant_string_start(child);
                    if !matches!(child, Value::String(text) if text.is_empty()) {
                        break;
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                trim_first_descendant_string_start(item);
                if !matches!(item, Value::String(text) if text.is_empty()) {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn ensure_last_descendant_string_suffix(value: &mut Value, suffix: &str) {
    if !last_descendant_string_has_suffix(value, suffix) {
        append_last_string(value, suffix);
    }
}

fn last_descendant_string_has_suffix(value: &Value, suffix: &str) -> bool {
    match value {
        Value::String(text) => text.ends_with(suffix),
        Value::Object(map) => map
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|content| {
                content
                    .iter()
                    .rev()
                    .any(|child| last_descendant_string_has_suffix(child, suffix))
            }),
        Value::Array(items) => items
            .iter()
            .rev()
            .any(|item| last_descendant_string_has_suffix(item, suffix)),
        _ => false,
    }
}

fn last_descendant_string_ends_with_trimmed_char(value: &Value, ch: char) -> bool {
    match value {
        Value::String(text) => text.trim_end().ends_with(ch),
        Value::Object(map) => map
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|content| {
                content
                    .iter()
                    .rev()
                    .any(|child| last_descendant_string_ends_with_trimmed_char(child, ch))
            }),
        Value::Array(items) => items
            .iter()
            .rev()
            .any(|item| last_descendant_string_ends_with_trimmed_char(item, ch)),
        _ => false,
    }
}

fn last_descendant_string_ends_with_whitespace(value: &Value) -> bool {
    match value {
        Value::String(text) => text.chars().last().is_some_and(char::is_whitespace),
        Value::Object(object) => object
            .get("content")
            .and_then(Value::as_array)
            .and_then(|content| content.last())
            .is_some_and(last_descendant_string_ends_with_whitespace),
        Value::Array(items) => items
            .last()
            .is_some_and(last_descendant_string_ends_with_whitespace),
        _ => false,
    }
}

fn append_last_string(value: &mut Value, suffix: &str) -> bool {
    match value {
        Value::String(text) => {
            text.push_str(suffix);
            true
        }
        Value::Object(map) => {
            if let Some(Value::Array(content)) = map.get_mut("content") {
                for child in content.iter_mut().rev() {
                    if append_last_string(child, suffix) {
                        return true;
                    }
                }
            }
            false
        }
        Value::Array(items) => {
            for item in items.iter_mut().rev() {
                if append_last_string(item, suffix) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

fn trim_last_string_end(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = trim_ascii_end(text).to_string();
        }
        Value::Object(map) => {
            if let Some(Value::Array(content)) = map.get_mut("content") {
                trim_last_descendant_string_end(content);
            }
        }
        Value::Array(items) => trim_last_descendant_string_end(items),
        _ => {}
    }
}

fn normalize_text(text: &str, preserve_newlines: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    let no_cr = text.replace('\r', "");
    let with_spaces = if preserve_newlines {
        no_cr.replace('~', "\u{00a0}")
    } else {
        no_cr.replace('\n', " ").replace('~', "\u{00a0}")
    };
    if preserve_newlines {
        with_spaces
    } else {
        collapse_spaces(&with_spaces)
    }
}

fn trim_ascii_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\n', '\r', '\t'])
}

fn trim_ascii_end(text: &str) -> &str {
    text.trim_end_matches([' ', '\n', '\r', '\t'])
}

fn collapse_spaces(text: &str) -> String {
    if !text.contains("  ") {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut previous_space = false;
    for ch in text.chars() {
        if ch == ' ' {
            if !previous_space {
                result.push(ch);
            }
            previous_space = true;
        } else {
            previous_space = false;
            result.push(ch);
        }
    }
    result
}

fn collect_attributes(
    source: &str,
    marker: &str,
    attribute_spans: &[std::ops::Range<usize>],
) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    for span in attribute_spans {
        let raw = &source[span.clone()];
        let Some(parsed) = parse_attributes(raw) else {
            continue;
        };
        attrs.extend(
            resolve_default_attr_keys(marker, parsed)
                .into_iter()
                .filter(|(key, value)| !is_ignorable_figure_placeholder(marker, key, value)),
        );
    }
    attrs
}

fn is_ignorable_figure_placeholder(marker: &str, key: &str, value: &str) -> bool {
    marker == "fig" && key == "file" && value.chars().all(|ch| ch == '|' || ch.is_whitespace())
}

enum AttributeBehavior {
    Flatten(Vec<(String, String)>),
    RawText(String),
}

fn resolve_attribute_behavior(
    source: &str,
    marker: &str,
    attribute_spans: &[std::ops::Range<usize>],
    children: &[Node],
) -> AttributeBehavior {
    if attribute_spans.is_empty() {
        return AttributeBehavior::Flatten(Vec::new());
    }

    let content_has_trailing_whitespace = content_ends_with_whitespace(children, source);
    let mut parsed_spans = Vec::new();
    let mut attrs = Vec::new();
    for span in attribute_spans {
        let raw = &source[span.clone()];
        let Some(parsed) = parse_attributes(raw) else {
            return AttributeBehavior::RawText(join_attribute_spans(source, attribute_spans));
        };
        parsed_spans.push((raw, parsed));
    }

    for (raw, parsed) in &parsed_spans {
        if should_preserve_raw_attributes(
            marker,
            raw,
            parsed.as_slice(),
            content_has_trailing_whitespace,
            total_attribute_count(&parsed_spans),
        ) {
            return AttributeBehavior::RawText(join_attribute_spans(source, attribute_spans));
        }
    }

    for (_, parsed) in parsed_spans {
        attrs.extend(resolve_default_attr_keys(marker, parsed));
    }

    AttributeBehavior::Flatten(attrs)
}

fn join_attribute_spans(source: &str, attribute_spans: &[std::ops::Range<usize>]) -> String {
    let mut raw = String::new();
    for span in attribute_spans {
        raw.push_str(&source[span.clone()]);
    }
    raw
}

fn attribute_spans_have_leading_gap(
    source: &str,
    attribute_spans: &[std::ops::Range<usize>],
) -> bool {
    attribute_spans.first().is_some_and(|span| {
        span.start > 0
            && source[..span.start]
                .chars()
                .next_back()
                .is_some_and(|ch| matches!(ch, ' ' | '\t'))
    })
}

fn content_ends_with_whitespace(children: &[Node], source: &str) -> bool {
    let has_text_content = children.iter().any(|node| match node {
        Node::Leaf {
            kind: LeafKind::Text,
            span,
        } => !source[span.clone()].is_empty(),
        _ => false,
    });

    has_text_content
        && children
            .iter()
            .rev()
            .find_map(|node| match node {
                Node::Leaf {
                    kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
                    span,
                } => source[span.clone()].chars().last(),
                _ => None,
            })
            .is_some_and(char::is_whitespace)
}

fn content_ends_with_newline_node(children: &[Node]) -> bool {
    children.iter().rev().find_map(|node| match node {
        Node::Leaf {
            kind: LeafKind::Newline,
            ..
        } => Some(true),
        Node::Leaf {
            kind: LeafKind::Whitespace
                | LeafKind::Text
                | LeafKind::Attributes
                | LeafKind::OptBreak,
            ..
        } => Some(false),
        _ => None,
    })
    .unwrap_or(false)
}

fn source_uses_alternate_texts_book_code(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim() == "\\mt1 Alternate Texts")
}

fn should_emit_sid(source: &str) -> bool {
    !matches_legacy_freeform_id_without_usfm(source)
}

fn usj_version(source: &str) -> &str {
    source
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("\\usfm ").map(|rest| {
                if rest.starts_with("3.1") {
                    "3.1"
                } else if rest.starts_with("2.") {
                    "2.0"
                } else {
                    "3.0"
                }
            })
        })
        .unwrap_or("3.1")
}

fn matches_legacy_freeform_id_without_usfm(source: &str) -> bool {
    if source.contains("\\usfm") {
        return false;
    }

    let Some(first_line) = source.lines().next() else {
        return false;
    };
    let Some(rest) = first_line.strip_prefix("\\id ") else {
        return false;
    };
    let mut parts = rest.splitn(2, char::is_whitespace);
    let _code = parts.next();
    let Some(description) = parts.next().map(str::trim_start) else {
        return false;
    };

    description.starts_with("a ") || description.starts_with("A ")
}

fn container_has_trailing_standalone_ts_gap(container: &ContainerNode) -> bool {
    let children = container.children.as_slice();
    let Some(last_significant_index) = children.iter().rposition(|node| {
        !matches!(
            node,
            Node::Leaf {
                kind: LeafKind::Whitespace | LeafKind::Newline,
                ..
            }
        )
    }) else {
        return false;
    };

    matches!(
        children.get(last_significant_index),
        Some(Node::Milestone { marker, .. }) if marker == "ts"
    ) && matches!(
        children.get(last_significant_index.saturating_sub(1)),
        Some(Node::Leaf {
            kind: LeafKind::Newline,
            ..
        })
    )
}

fn append_gap_before_trailing_ts(value: &mut Value) {
    let Some(content) = value
        .as_object_mut()
        .and_then(|object| object.get_mut("content"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    if content.len() < 2 {
        return;
    }

    let last_is_ts = content
        .last()
        .and_then(Value::as_object)
        .and_then(|object| object.get("marker"))
        .and_then(Value::as_str)
        == Some("ts");
    if !last_is_ts {
        return;
    }

    let previous_index = content.len() - 2;
    if let Some(Value::String(previous)) = content.get_mut(previous_index) {
        *previous = previous.trim_end_matches(' ').to_string();
        previous.push_str("\n  ");
    }
}

fn container_has_trailing_newline(container: &ContainerNode) -> bool {
    container.children.iter().rev().find_map(|node| match node {
        Node::Leaf {
            kind: LeafKind::Newline,
            ..
        } => Some(true),
        Node::Leaf {
            kind: LeafKind::Whitespace
                | LeafKind::Text
                | LeafKind::Attributes
                | LeafKind::OptBreak,
            ..
        } => Some(false),
        _ => None,
    }).unwrap_or(false)
}

fn next_sibling_is_unknown_para(nodes: &[Node], index: usize) -> bool {
    matches!(
        nodes.get(index + 1),
        Some(Node::Container(ContainerNode { marker, kind: ContainerKind::Paragraph, .. }))
            if lookup_marker(marker.as_str()).kind == crate::internal::markers::MarkerKind::Unknown
    )
}

fn append_trailing_space_to_last_string(value: &mut Value) {
    let Some(content) = value
        .as_object_mut()
        .and_then(|object| object.get_mut("content"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    let Some(Value::String(last)) = content.last_mut() else {
        return;
    };
    if !last.ends_with(' ') {
        last.push(' ');
    }
}

fn total_attribute_count(parsed_spans: &[(&str, Vec<(String, String)>)]) -> usize {
    parsed_spans.iter().map(|(_, parsed)| parsed.len()).sum()
}

fn should_preserve_raw_attributes(
    marker: &str,
    raw: &str,
    attrs: &[(String, String)],
    content_has_trailing_whitespace: bool,
    total_attr_count: usize,
) -> bool {
    let default_key = marker_default_attribute(marker);

    if attrs
        .iter()
        .any(|(key, _)| key == "default" && default_key.is_none())
    {
        return true;
    }

    if marker == "w"
        && raw.starts_with('|')
        && raw.contains("=\"")
        && raw.contains(" strong=")
        && content_has_trailing_whitespace
    {
        return true;
    }

    if content_has_trailing_whitespace && total_attr_count > 1 {
        return true;
    }

    if content_has_trailing_whitespace
        && total_attr_count == 1
        && let Some((key, _)) = attrs.first()
        && key != "default"
        && Some(key.as_str()) != default_key
    {
        return true;
    }

    let Some(default_key) = default_key else {
        return false;
    };

    attrs.iter().any(|(key, value)| {
        key == default_key
            && value.is_empty()
            && raw
                .strip_prefix('|')
                .is_some_and(|raw| raw.trim_start().starts_with(&format!("{default_key}=")))
    })
}

fn normalize_preserved_raw_attributes(
    marker: &str,
    raw: &str,
    attr_gap_before: bool,
    content_has_trailing_whitespace: bool,
) -> String {
    let mut text = normalize_text(raw, true);
    if marker == "w" && !content_has_trailing_whitespace && text.starts_with('|') {
        text.insert(0, ' ');
    }
    if marker == "w"
        && attr_gap_before
        && let Some(default_key) = marker_default_attribute(marker)
    {
        let prefixed = format!(" |{default_key}=\"");
        let unprefixed = format!("|{default_key}=\"");
        let stripped = text
            .strip_prefix(&prefixed)
            .map(|remainder| (" ", remainder))
            .or_else(|| {
                text.strip_prefix(&unprefixed)
                    .map(|remainder| ("", remainder))
            });
        if let Some((prefix_gap, remainder)) = stripped
            && let Some(end_quote) = remainder.find('"')
        {
            let value = &remainder[..end_quote];
            if !value.is_empty() && !value.chars().any(char::is_whitespace) {
                text = format!(
                    "{prefix_gap}|{default_key}={value}{}",
                    &remainder[end_quote + 1..]
                );
            }
        }
    }
    text
}

fn flatten_attributes(map: &mut Map<String, Value>, attrs: Vec<(String, String)>) {
    for (key, value) in attrs {
        map.insert(key, Value::String(value));
    }
}

fn parse_attributes(attr_str: &str) -> Option<Vec<(String, String)>> {
    let attrs = attr_str.strip_prefix('|').unwrap_or(attr_str);
    if attrs.is_empty() {
        return Some(Vec::new());
    }

    if !attrs.contains('=') {
        return Some(vec![("default".to_string(), attrs.to_string())]);
    }

    let mut out = Vec::new();
    let mut remaining = attrs.trim_start();

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        let eq_pos = remaining.find('=')?;
        let before_eq = &remaining[..eq_pos];
        if before_eq.contains(' ') || before_eq.contains('"') {
            return None;
        }

        let key = before_eq.trim().to_string();
        remaining = &remaining[eq_pos + 1..];
        if !remaining.starts_with('"') {
            return None;
        }
        remaining = &remaining[1..];

        match find_unescaped_quote(remaining) {
            Some(end_quote) => {
                let value = remaining[..end_quote].replace("\\\"", "\"");
                out.push((key, value));
                remaining = &remaining[end_quote + 1..];
            }
            None => {
                out.push((key, remaining.replace("\\\"", "\"")));
                break;
            }
        }
    }

    Some(out)
}

fn find_unescaped_quote(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    (0..bytes.len())
        .find(|&index| bytes[index] == b'"' && (index == 0 || bytes[index - 1] != b'\\'))
}

fn resolve_default_attr_keys(marker: &str, attrs: Vec<(String, String)>) -> Vec<(String, String)> {
    let default_key = marker_default_attribute(marker);
    attrs
        .into_iter()
        .map(|(key, value)| {
            if key == "default"
                && let Some(default_key) = default_key
            {
                return (rename_attribute_key(marker, default_key), value);
            }
            (rename_attribute_key(marker, &key), value)
        })
        .collect()
}

fn rename_attribute_key(marker: &str, key: &str) -> String {
    if marker == "fig" && key == "src" {
        "file".to_string()
    } else {
        key.to_string()
    }
}

fn extract_category_nodes(nodes: &[Node], source: &str) -> (Option<String>, Vec<Node>) {
    let Some(index) = nodes.iter().position(is_category_node) else {
        return (None, nodes.to_vec());
    };

    let category = match &nodes[index] {
        Node::Container(container) => plain_text_from_nodes(container.children.as_slice(), source),
        _ => None,
    };

    let mut filtered = Vec::with_capacity(nodes.len().saturating_sub(1));
    for (current_index, node) in nodes.iter().enumerate() {
        if current_index == index {
            continue;
        }
        if current_index + 1 == index && is_whitespace_only_leaf(node, source) {
            continue;
        }
        filtered.push(node.clone());
    }

    (category, filtered)
}

fn extract_periph_alt(nodes: &[Node], source: &str) -> (Option<String>, usize) {
    let mut collected = String::new();
    let mut consumed = 0usize;

    for node in nodes {
        match node {
            Node::Leaf {
                kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
                span,
            } => {
                consumed += 1;
                collected.push_str(&normalize_text(&source[span.clone()], false));
            }
            _ => break,
        }
    }

    let trimmed = collected.trim().to_string();
    if trimmed.is_empty() {
        (None, 0)
    } else {
        (Some(trimmed), consumed)
    }
}

fn plain_text_from_nodes(nodes: &[Node], source: &str) -> Option<String> {
    let mut text = String::new();
    for node in nodes {
        match node {
            Node::Leaf {
                kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
                span,
            } => text.push_str(&normalize_text(&source[span.clone()], false)),
            _ => return None,
        }
    }

    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn is_category_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Container(ContainerNode { marker, .. }) if marker == "cat"
    )
}

fn is_whitespace_only_leaf(node: &Node, source: &str) -> bool {
    match node {
        Node::Leaf {
            kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
            span,
        } => normalize_text(&source[span.clone()], false)
            .trim()
            .is_empty(),
        _ => false,
    }
}

fn is_ignorable_trivia_node(node: &Node, source: &str) -> bool {
    match node {
        Node::Leaf {
            kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
            span,
        } => normalize_text(&source[span.clone()], false)
            .trim()
            .is_empty(),
        _ => false,
    }
}

fn table_cell_alignment(marker: &str) -> String {
    let without_span = if let Some(dash) = marker.rfind('-') {
        let after = &marker[dash + 1..];
        if !after.is_empty() && after.chars().all(|ch| ch.is_ascii_digit()) {
            &marker[..dash]
        } else {
            marker
        }
    } else {
        marker
    };

    let base = without_span.trim_end_matches(|ch: char| ch.is_ascii_digit());
    if base.ends_with('r') {
        "end".to_string()
    } else if matches!(base, "thc" | "tcc") {
        "center".to_string()
    } else {
        "start".to_string()
    }
}

fn strip_leading_zeros(value: &str) -> String {
    value
        .parse::<u64>()
        .map(|number| number.to_string())
        .unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn collect_by_type<'a>(value: &'a Value, ty: &str, out: &mut Vec<&'a Value>) {
        match value {
            Value::Object(map) => {
                if map.get("type").and_then(Value::as_str) == Some(ty) {
                    out.push(value);
                }
                if let Some(Value::Array(content)) = map.get("content") {
                    for child in content {
                        collect_by_type(child, ty, out);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    collect_by_type(item, ty, out);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn serializes_external_usj_shape() {
        let handle = parse(
            "\\id GEN Genesis\n\\usfm 3.1\n\\c 1\n\\p\n\\v 1 \\w beginning|lemma=\"H7225\" strong=\"H7225\"\\w*.\n",
        );
        let value = to_usj_value(&handle);
        let mut chapters = Vec::new();
        let mut chars = Vec::new();
        collect_by_type(&value, "chapter", &mut chapters);
        collect_by_type(&value, "char", &mut chars);

        assert_eq!(value["type"], "USJ");
        assert_eq!(value["version"], "3.1");
        assert_eq!(value["content"][0]["type"], "book");
        assert_eq!(value["content"][0]["code"], "GEN");
        assert_eq!(value["content"][0]["content"][0], "Genesis");
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0]["sid"], "GEN 1");

        let char_node = chars
            .into_iter()
            .find(|node| node["marker"] == "w")
            .expect("expected \\w char node");
        assert_eq!(char_node["type"], "char");
        assert_eq!(char_node["marker"], "w");
        assert_eq!(char_node["lemma"], "H7225");
        assert_eq!(char_node["strong"], "H7225");
    }

    #[test]
    fn extracts_note_category_and_periph_alt() {
        let handle = parse(
            "\\id FRT\n\\periph My Title|id=\"title\"\n\\p Detail\n\\p \\f + \\cat People\\cat* \\ft note\\f*\n",
        );
        let value = to_usj_value(&handle);

        assert_eq!(value["content"][1]["type"], "periph");
        assert_eq!(value["content"][1]["alt"], "My Title");
        assert_eq!(value["content"][1]["id"], "title");

        let note = &value["content"][1]["content"][1]["content"][0];
        assert_eq!(note["type"], "note");
        assert_eq!(note["category"], "People");
    }

    #[test]
    fn wraps_consecutive_rows_in_table() {
        let handle =
            parse("\\id MAT\n\\c 1\n\\tr \\th1 Day\\tc2 Tribe\n\\tr \\tc1 1st\\tc2 Judah\n");
        let value = to_usj_value(&handle);
        let mut tables = Vec::new();
        collect_by_type(&value, "table", &mut tables);

        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0]["content"][0]["type"], "table:row");
        assert_eq!(tables[0]["content"][0]["content"][0]["align"], "start");
    }

    #[test]
    fn converts_ref_and_optbreak() {
        let handle = parse("\\id MAT\n\\c 1\n\\cd \\ref 1|GEN 2:1\\ref* a//b");
        let value = to_usj_value(&handle);
        let mut refs = Vec::new();
        let mut optbreaks = Vec::new();
        collect_by_type(&value, "ref", &mut refs);
        collect_by_type(&value, "optbreak", &mut optbreaks);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["loc"], "GEN 2:1");
        assert_eq!(optbreaks.len(), 1);
    }

    #[test]
    fn preserves_standalone_ts_trivia_before_next_paragraph() {
        let handle =
            parse("\\id MAT\n\\c 1\n\\p\n\\v 14 But Jesus refused to answer ...\n\\ts\\*\n\\p\n");
        let value = to_usj_value(&handle);
        let paragraph = &value["content"][2];

        assert_eq!(paragraph["marker"], "p");
        assert_eq!(
            paragraph["content"][1],
            "But Jesus refused to answer ...\n  "
        );
        assert_eq!(paragraph["content"][2]["marker"], "ts");
    }

    #[test]
    fn keeps_ft_text_tight_after_fqa_when_not_punctuation() {
        let handle = parse(
            "\\id MRK\n\\c 1\n\\p\n\\v 1 verse\\f + \\fqa footies \\ft very very rarely.\\f*\n",
        );
        let value = to_usj_value(&handle);
        let note = &value["content"][2]["content"][2];

        assert_eq!(note["type"], "note");
        assert_eq!(note["content"][0]["marker"], "fqa");
        assert_eq!(note["content"][0]["content"][0], "footies ");
        assert_eq!(note["content"][1]["marker"], "ft");
        assert_eq!(note["content"][1]["content"][0], "very very rarely.");
    }

    #[test]
    fn keeps_punctuation_with_following_ft_after_fqa() {
        let handle = parse(
            "\\id GEN\n\\usfm 3.1\n\\c 2\n\\p\n\\v 5 text\\f + \\fqa land\\ft ; also in \\ref verse 6|GEN 2:6\\ref*\\f*\n",
        );
        let value = to_usj_value(&handle);
        let note = &value["content"][2]["content"][2];

        assert_eq!(note["content"][0]["marker"], "fqa");
        assert_eq!(note["content"][0]["content"][0], "land");
        assert_eq!(note["content"][1]["marker"], "ft");
        assert_eq!(note["content"][1]["content"][0], "; also in ");
    }

    #[test]
    fn keeps_word_gap_after_comma_fqa_transition() {
        let handle = parse(
            "\\id TIT\n\\c 1\n\\q\nText\n\\f + \\ft Some early versions omit, \\fqa in Ephesus, \\ft but this expression is probably in Paul's original letter. \\f*\n",
        );
        let value = to_usj_value(&handle);
        let note = &value["content"][2]["content"][1];

        assert_eq!(note["content"][1]["content"][0], "in Ephesus, ");
        assert_eq!(
            note["content"][2]["content"][0],
            " but this expression is probably in Paul's original letter. "
        );
    }

    #[test]
    fn preserves_separator_gap_before_unknown_next_paragraph() {
        let handle = parse("\\id GEN\n\\c 1\n\\p \\v 1 Hi \\nd Bob\\nd*.\n\\ix text\n");
        let value = to_usj_value(&handle);

        assert_eq!(value["content"][2]["content"][3], ". ");
        assert_eq!(value["content"][3]["marker"], "ix");
    }

    #[test]
    fn trims_unclosed_char_style_trailing_space() {
        let handle = parse("\\id GEN\n\\c 1\n\\p \\v 1 Hi \\nd Bob.\n");
        let value = to_usj_value(&handle);

        assert_eq!(value["content"][2]["content"][2]["content"][0], "Bob.");
    }

    #[test]
    fn serializes_stray_sidebar_end_as_unmatched() {
        let handle = parse("\\id GEN\n\\c 1\n\\esbe\n");
        let value = to_usj_value(&handle);

        assert_eq!(value["content"][2]["type"], "unmatched");
        assert_eq!(value["content"][2]["marker"], "esbe");
    }

    #[test]
    fn preserves_detached_word_attributes_as_raw_text() {
        let handle = parse(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 text \\w word |lemma=\"grace\" strong=\"H1\"\\w* more",
        );
        let value = to_usj_value(&handle);
        let mut chars = Vec::new();
        collect_by_type(&value, "char", &mut chars);

        let char_node = chars
            .into_iter()
            .find(|node| node["marker"] == "w")
            .expect("expected \\w char node");
        assert!(char_node.get("lemma").is_none());
        assert_eq!(char_node["content"][0], "word |lemma=grace strong=\"H1\"");
    }

    #[test]
    fn preserves_explicit_empty_default_attribute_as_raw_text() {
        let handle =
            parse("\\id GEN\n\\c 1\n\\p\n\\v 1 text \\w word|lemma=\"\" strong=\"H1\"\\w* more");
        let value = to_usj_value(&handle);
        let mut chars = Vec::new();
        collect_by_type(&value, "char", &mut chars);

        let char_node = chars
            .into_iter()
            .find(|node| node["marker"] == "w")
            .expect("expected \\w char node");
        assert!(char_node.get("lemma").is_none());
        assert_eq!(char_node["content"][0], "word |lemma=\"\" strong=\"H1\"");
    }

    #[test]
    fn keeps_fv_as_sibling_note_char_after_fqa() {
        let handle = parse(
            "\\id TIT\n\\c 3\n\\p\n\\v 8 text \\f + \\fr 7.38: \\ft intro \\fqa quote \\fv 8\\fv* tail\\f*",
        );
        let value = to_usj_value(&handle);
        let note = &value["content"][2]["content"][2];

        assert_eq!(note["type"], "note");
        assert_eq!(note["content"][2]["marker"], "fqa");
        assert_eq!(note["content"][2]["content"][0], "quote ");
        assert_eq!(note["content"][3]["marker"], "fv");
        assert_eq!(note["content"][3]["content"][0], "8");
        assert_eq!(note["content"][4], " tail");
    }

    #[test]
    fn keeps_fv_nested_in_fqa_when_no_trailing_tail_exists() {
        let handle = parse(
            "\\id TIT\n\\c 3\n\\p\n\\v 8 text \\f + \\fr 7.38: \\ft intro \\fqa quote \\fv 38\\fv*\\ft tail\\f*",
        );
        let value = to_usj_value(&handle);
        let note = &value["content"][2]["content"][2];

        assert_eq!(note["content"][2]["marker"], "fqa");
        assert_eq!(note["content"][2]["content"][0], "quote ");
        assert_eq!(note["content"][2]["content"][1]["marker"], "fv");
        assert_eq!(note["content"][2]["content"][1]["content"][0], "38");
        assert_eq!(note["content"][3]["marker"], "ft");
        assert_eq!(note["content"][3]["content"][0], " tail");
    }

    #[test]
    fn does_not_fold_styled_vp_into_verse_metadata() {
        let handle = parse(
            "\\id MAT\n\\c 1\n\\p\n\\v 20 text\n\\v 21 \\vp \\it \\wj 21\\wj*\\it*\\vp* \\it \\wj body\\wj*\\it*\n",
        );
        let value = to_usj_value(&handle);
        let para = &value["content"][2];

        assert_eq!(para["content"][2]["marker"], "v");
        assert_eq!(para["content"][3]["marker"], "vp");
        assert_eq!(para["content"][3]["type"], "char");
        assert_eq!(para["content"][3]["content"][0]["marker"], "it");
        assert_eq!(para["content"][4]["marker"], "it");
    }
}
