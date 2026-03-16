use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cst::CstDocument;
use crate::export_tree::{
    ExportContainerKind, ExportContainerNode, ExportDocument, ExportNode, build_export_document,
};
use crate::marker_defs::{NoteSubkind, SpecMarkerKind, marker_default_attribute, marker_note_subkind};
use crate::parse::parse;
use crate::token::{NumberRangeKind, TokenData};

const USJ_VERSION: &str = "3.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsjDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: String,
    pub content: Vec<UsjNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UsjNode {
    Text(String),
    Element(UsjElement),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UsjElement {
    #[serde(rename = "book")]
    Book {
        marker: String,
        code: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "chapter")]
    Chapter {
        marker: String,
        number: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        altnumber: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pubnumber: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "verse")]
    Verse {
        marker: String,
        number: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        altnumber: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pubnumber: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "para")]
    Para {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "char")]
    Char {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "ref")]
    Ref {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "note")]
    Note {
        marker: String,
        caller: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "ms")]
    Milestone {
        marker: String,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "figure")]
    Figure {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "sidebar")]
    Sidebar {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "periph")]
    Periph {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table")]
    Table {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table:row")]
    TableRow {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table:cell")]
    TableCell {
        marker: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "unknown")]
    Unknown {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "unmatched")]
    Unmatched {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "optbreak")]
    OptBreak {},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsjError {
    Json(String),
    InvalidRootType(String),
    MissingField(&'static str),
    UnknownNodeType(String),
}

impl std::fmt::Display for UsjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(f, "{error}"),
            Self::InvalidRootType(value) => write!(f, "expected USJ root type, found {value}"),
            Self::MissingField(field) => write!(f, "missing required field {field}"),
            Self::UnknownNodeType(node_type) => write!(f, "unknown USJ node type '{node_type}'"),
        }
    }
}

impl std::error::Error for UsjError {}

impl From<serde_json::Error> for UsjError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

pub fn usfm_to_usj(source: &str) -> Result<UsjDocument, UsjError> {
    let parsed = parse(source);
    Ok(tokens_to_usj(&parsed.tokens))
}

pub fn cst_to_usj(document: &CstDocument<'_>) -> UsjDocument {
    tokens_to_usj(&document.tokens)
}

fn tokens_to_usj(tokens: &[crate::token::Token<'_>]) -> UsjDocument {
    let export = build_export_document(tokens);
    UsjDocument {
        doc_type: "USJ".to_string(),
        version: USJ_VERSION.to_string(),
        content: UsjExporter::new(&export).export_nodes(&export.children),
    }
}

pub fn from_usj(document: &UsjDocument) -> Result<String, UsjError> {
    if document.doc_type != "USJ" {
        return Err(UsjError::InvalidRootType(document.doc_type.clone()));
    }

    let mut serializer = UsjSerializer::default();
    serializer.serialize_nodes(&document.content)?;
    Ok(serializer.finish())
}

pub fn from_usj_str(source: &str) -> Result<String, UsjError> {
    let document: UsjDocument = serde_json::from_str(source)?;
    from_usj(&document)
}

struct UsjExporter<'a, 'doc> {
    document: &'doc ExportDocument<'a>,
}

impl<'a, 'doc> UsjExporter<'a, 'doc> {
    fn new(document: &'doc ExportDocument<'a>) -> Self {
        Self { document }
    }

    fn export_nodes(&self, nodes: &[ExportNode]) -> Vec<UsjNode> {
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
                    TokenData::OptBreak => (vec![UsjNode::Element(UsjElement::OptBreak {})], index + 1),
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
                attribute_index,
                closed: _,
                end_index: _,
            } => {
                let TokenData::Milestone { name, .. } = &self.document.tokens[*marker_index].data else {
                    return (Vec::new(), index + 1);
                };
                let marker_name = export_marker_name(name);
                let mut extra = BTreeMap::new();
                if let Some(attribute_index) = attribute_index {
                    extra.extend(self.attribute_map_from_token(*attribute_index, Some(marker_name)));
                }
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
                vec![UsjNode::Element(self.export_chapter(*marker_index, *number_index))],
                index + 1,
            ),
            ExportNode::Verse {
                marker_index,
                number_index,
            } => (
                vec![UsjNode::Element(self.export_verse(*marker_index, *number_index))],
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
                    Some(SpecMarkerKind::Header) if name == "id" => {
                        (vec![UsjNode::Element(self.export_book(container))], index + 1)
                    }
                    Some(SpecMarkerKind::Note) => {
                        (vec![UsjNode::Element(self.export_note(container, name))], index + 1)
                    }
                    Some(SpecMarkerKind::Character) => {
                        (self.export_character_sequence(container, name), index + 1)
                    }
                    Some(SpecMarkerKind::Figure) => {
                        (vec![UsjNode::Element(self.export_figure(container, name))], index + 1)
                    }
                    Some(SpecMarkerKind::Periph) => {
                        (vec![UsjNode::Element(self.export_periph(container))], index + 1)
                    }
                    Some(SpecMarkerKind::Sidebar) => {
                        (vec![UsjNode::Element(self.export_sidebar(container, name))], index + 1)
                    }
                    Some(SpecMarkerKind::TableRow) => {
                        (vec![UsjNode::Element(self.export_table_row(container, name))], index + 1)
                    }
                    Some(SpecMarkerKind::TableCell) => {
                        (vec![UsjNode::Element(self.export_table_cell(container, name))], index + 1)
                    }
                    Some(SpecMarkerKind::Paragraph) | Some(SpecMarkerKind::Header) | Some(SpecMarkerKind::Meta) => {
                        (vec![UsjNode::Element(self.export_para(container, name))], index + 1)
                    }
                    None if matches!(
                        container.kind,
                        ExportContainerKind::Paragraph | ExportContainerKind::Header | ExportContainerKind::Meta
                    ) => (vec![UsjNode::Element(self.export_para(container, name))], index + 1),
                    _ => (
                        vec![UsjNode::Element(self.export_unknown(container, name, false))],
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
                ExportNode::Leaf { token_index } => match &self.document.tokens[*token_index].data {
                    TokenData::BookCode { code: book_code, .. } if code.is_empty() => {
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
                    TokenData::AttributeList { .. } => {}
                    _ => {
                        let (mut exported, _) = self.export_node(std::slice::from_ref(child), 0);
                        content.append(&mut exported);
                    }
                },
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
            extra: self.collect_attribute_map(&node.children, node.attribute_index, Some("id")),
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
            extra: self.collect_attribute_map(&node.children, node.attribute_index, Some(marker)),
        }
    }

    fn export_note(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        let mut caller = "+".to_string();
        let mut content = Vec::new();
        let mut category = None;
        let mut attrs = self.collect_attribute_map(&node.children, node.attribute_index, Some(marker));

        let mut started_content = false;
        for child in &node.children {
            match child {
                ExportNode::Leaf { token_index } => {
                    let token = &self.document.tokens[*token_index];
                    match &token.data {
                        TokenData::AttributeList { .. } => continue,
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
        self.export_character_like_from_children(marker, &node.children, node.attribute_index)
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
            UsjElement::Ref {
                content,
                extra,
            }
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

        let mut exported = vec![UsjNode::Element(
            self.export_character_like_from_children(
                marker,
                &node.children[..split_index],
                node.attribute_index,
            ),
        )];
        exported.extend(self.export_non_attribute_children(&node.children[split_index..]));
        exported
    }

    fn export_figure(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        if node.close_index.is_none() {
            return self.export_unclosed_figure(node, marker);
        }
        let (content, extra) =
            self.export_inline_content_and_attributes(marker, &node.children, node.attribute_index);
        UsjElement::Figure {
            marker: marker.to_string(),
            content,
            extra,
        }
    }

    fn export_unclosed_figure(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        #[derive(Clone)]
        enum FigurePart {
            Child(ExportNode),
            Attr(String),
        }

        let mut parts: Vec<(u32, FigurePart)> = Vec::new();
        for child in &node.children {
            parts.push((self.node_start(child), FigurePart::Child(child.clone())));
        }
        if let Some(attribute_index) = node.attribute_index {
            parts.push((
                self.document.tokens[attribute_index].span.start,
                FigurePart::Attr(format!("{} ", self.document.tokens[attribute_index].source.trim())),
            ));
        }
        parts.sort_by_key(|(start, _)| *start);

        let mut content = Vec::new();
        for (_, part) in parts {
            match part {
                FigurePart::Child(child) => {
                    content.extend(self.export_non_attribute_children(std::slice::from_ref(&child)));
                }
                FigurePart::Attr(source) => content.push(UsjNode::Text(source)),
            }
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
        let mut attrs = self.collect_attribute_map(&node.children, node.attribute_index, Some(marker));

        for child in &node.children {
            match child {
                ExportNode::Leaf { token_index }
                    if matches!(self.document.tokens[*token_index].data, TokenData::AttributeList { .. }) =>
                {
                    continue;
                }
                ExportNode::Container(container)
                    if matches!(
                        self.document.tokens[container.token_index].data,
                        TokenData::Marker { name: "cat", .. }
                    ) =>
                {
                    category = extract_inline_text(self, &container.children);
                    continue;
                }
                _ => {}
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
        let attrs = self.collect_attribute_map(&node.children, node.attribute_index, None);

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
                        TokenData::AttributeList { .. } => {}
                        _ => {
                            let mut exported =
                                self.export_non_attribute_children(std::slice::from_ref(child));
                            content.append(&mut exported);
                        }
                    }
                }
                _ => {
                    let mut exported = self.export_non_attribute_children(std::slice::from_ref(child));
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
            extra: self.collect_attribute_map(&node.children, node.attribute_index, Some(marker)),
        }
    }

    fn export_table_cell(&self, node: &ExportContainerNode, marker: &str) -> UsjElement {
        UsjElement::TableCell {
            marker: marker.to_string(),
            align: Some(table_cell_alignment(marker).to_string()),
            content: self.export_non_attribute_children(&node.children),
            extra: self.collect_attribute_map(&node.children, node.attribute_index, Some(marker)),
        }
    }

    fn export_unknown(&self, node: &ExportContainerNode, marker: &str, unmatched: bool) -> UsjElement {
        let content = self.export_non_attribute_children(&node.children);
        let extra = self.collect_attribute_map(&node.children, node.attribute_index, Some(marker));
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
        children: &[ExportNode],
        own_attribute_index: Option<usize>,
        own_marker: Option<&str>,
    ) -> BTreeMap<String, String> {
        let mut extra = BTreeMap::new();
        if let Some(attribute_index) = own_attribute_index {
            extra.extend(self.attribute_map_from_token(attribute_index, own_marker));
        }
        for child in children {
            if let ExportNode::Leaf { token_index } = child
                && let TokenData::AttributeList { entries } = &self.document.tokens[*token_index].data
            {
                for entry in entries {
                    extra.insert(entry.key.to_string(), entry.value.to_string());
                }
            }
        }
        extra
    }

    fn attribute_map_from_token(
        &self,
        token_index: usize,
        marker: Option<&str>,
    ) -> BTreeMap<String, String> {
        let mut extra = BTreeMap::new();
        if let TokenData::AttributeList { entries } = &self.document.tokens[token_index].data {
            if entries.is_empty() {
                if let Some(marker_name) = marker
                    && let Some(default_key) = marker_default_attribute(marker_name)
                {
                    let raw = self.document.tokens[token_index]
                        .source
                        .trim_start()
                        .strip_prefix('|')
                        .map(str::trim)
                        .unwrap_or_default();
                    if !raw.is_empty() {
                        extra.insert(
                            rename_attribute_key_for_usj(Some(marker_name), default_key),
                            raw.to_string(),
                        );
                    }
                }
            }
            for entry in entries {
                extra.insert(rename_attribute_key_for_usj(marker, entry.key), entry.value.to_string());
            }
        }
        extra
    }

    fn export_non_attribute_children(&self, children: &[ExportNode]) -> Vec<UsjNode> {
        let mut filtered = Vec::new();
        for child in children {
            match child {
                ExportNode::Leaf { token_index }
                    if matches!(self.document.tokens[*token_index].data, TokenData::AttributeList { .. }) => {}
                _ => filtered.push(child.clone()),
            }
        }
        self.export_nodes(&filtered)
    }

    fn export_inline_content_and_attributes(
        &self,
        marker: &str,
        children: &[ExportNode],
        own_attribute_index: Option<usize>,
    ) -> (Vec<UsjNode>, BTreeMap<String, String>) {
        #[derive(Clone)]
        enum InlinePart {
            Node(ExportNode),
            Text(String),
        }

        if self.attribute_token_has_no_entries(own_attribute_index)
            && marker_default_attribute(marker).is_none()
            && let Some(attribute_index) = own_attribute_index
        {
            let mut parts: Vec<(u32, InlinePart)> = Vec::new();
            for child in children {
                parts.push((self.node_start(child), InlinePart::Node(child.clone())));
            }
            parts.push((
                self.document.tokens[attribute_index].span.start,
                InlinePart::Text(self.document.tokens[attribute_index].source.trim().to_string()),
            ));
            parts.sort_by_key(|(start, _)| *start);

            let mut content = Vec::new();
            for (_, part) in parts {
                match part {
                    InlinePart::Node(node) => {
                        content.extend(self.export_non_attribute_children(std::slice::from_ref(&node)));
                    }
                    InlinePart::Text(text) => content.push(UsjNode::Text(text)),
                }
            }
            return (content, BTreeMap::new());
        }

        let mut extra = self.collect_attribute_map(children, own_attribute_index, Some(marker));
        let mut filtered = Vec::new();
        let default_attr_child = if self.attribute_token_has_no_entries(own_attribute_index)
            && marker_default_attribute(marker).is_some()
        {
            self.default_attribute_child_index(children)
        } else {
            None
        };

        for (index, child) in children.iter().enumerate() {
            if matches!(
                child,
                ExportNode::Leaf { token_index }
                    if matches!(self.document.tokens[*token_index].data, TokenData::AttributeList { .. })
            ) {
                continue;
            }

            if Some(index) == default_attr_child {
                continue;
            }

            filtered.push(child.clone());
        }

        if let Some(default_key) = marker_default_attribute(marker)
            && !extra.contains_key(default_key)
            && let Some(index) = default_attr_child
            && let ExportNode::Leaf { token_index } = &children[index]
            && let TokenData::Text = self.document.tokens[*token_index].data
        {
            let value = self.document.tokens[*token_index].source.trim().to_string();
            if !value.is_empty() {
                extra.insert(rename_attribute_key_for_usj(Some(marker), default_key), value);
            }
        }

        (self.export_nodes(&filtered), extra)
    }

    fn attribute_token_has_no_entries(&self, token_index: Option<usize>) -> bool {
        token_index
            .and_then(|index| match &self.document.tokens[index].data {
                TokenData::AttributeList { entries } => Some(entries.is_empty()),
                _ => None,
            })
            .unwrap_or(false)
    }

    fn default_attribute_child_index(&self, children: &[ExportNode]) -> Option<usize> {
        children
            .iter()
            .enumerate()
            .rev()
            .find(|(_, child)| {
                matches!(
                    child,
                    ExportNode::Leaf { token_index }
                        if matches!(self.document.tokens[*token_index].data, TokenData::Text)
                            && !self.document.tokens[*token_index].source.trim().is_empty()
                )
            })
            .map(|(index, _)| index)
    }

    fn number_from_token(&self, token_index: usize) -> Option<String> {
        let token = self.document.tokens.get(token_index)?;
        match &token.data {
            TokenData::Number {
                start,
                end,
                kind,
            } => Some(number_source_to_usj(token.source, *start, *end, *kind)),
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

    fn node_start(&self, node: &ExportNode) -> u32 {
        match node {
            ExportNode::Leaf { token_index } => self.document.tokens[*token_index].span.start,
            ExportNode::Chapter { marker_index, .. }
            | ExportNode::Verse { marker_index, .. }
            | ExportNode::Milestone { marker_index, .. } => self.document.tokens[*marker_index].span.start,
            ExportNode::Container(container) => self.document.tokens[container.token_index].span.start,
        }
    }
}

#[derive(Default)]
struct UsjSerializer {
    output: String,
    at_line_start: bool,
}

impl UsjSerializer {
    fn finish(mut self) -> String {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output
    }

    fn serialize_nodes(&mut self, nodes: &[UsjNode]) -> Result<(), UsjError> {
        for node in nodes {
            self.serialize_node(node)?;
        }
        Ok(())
    }

    fn serialize_node(&mut self, node: &UsjNode) -> Result<(), UsjError> {
        match node {
            UsjNode::Text(text) => {
                self.output.push_str(text);
                self.at_line_start = text.ends_with('\n');
                Ok(())
            }
            UsjNode::Element(element) => self.serialize_element(element),
        }
    }

    fn serialize_element(&mut self, element: &UsjElement) -> Result<(), UsjError> {
        match element {
            UsjElement::Book {
                marker,
                code,
                content,
                ..
            } => {
                self.ensure_newline();
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.output.push_str(code);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
                self.output.push('\n');
                self.at_line_start = true;
            }
            UsjElement::Chapter {
                marker,
                number,
                altnumber,
                pubnumber,
                ..
            } => {
                self.ensure_newline();
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.output.push_str(number);
                if let Some(altnumber) = altnumber {
                    self.output.push_str(" \\ca ");
                    self.output.push_str(altnumber);
                    self.output.push_str("\\ca*");
                }
                if let Some(pubnumber) = pubnumber {
                    self.output.push_str(" \\cp ");
                    self.output.push_str(pubnumber);
                }
                self.output.push('\n');
                self.at_line_start = true;
            }
            UsjElement::Verse {
                marker,
                number,
                altnumber,
                pubnumber,
                ..
            } => {
                if !self.at_line_start {
                    self.ensure_space();
                }
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.output.push_str(number);
                if let Some(altnumber) = altnumber {
                    self.output.push_str(" \\va ");
                    self.output.push_str(altnumber);
                    self.output.push_str("\\va*");
                }
                if let Some(pubnumber) = pubnumber {
                    self.output.push_str(" \\vp ");
                    self.output.push_str(pubnumber);
                    self.output.push_str("\\vp*");
                }
                self.output.push(' ');
                self.at_line_start = false;
            }
            UsjElement::Para {
                marker,
                content,
                extra,
            } => {
                self.ensure_newline();
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                self.at_line_start = false;
                if !content.is_empty() {
                    let first_is_verse = matches!(
                        content.first(),
                        Some(UsjNode::Element(UsjElement::Verse { .. }))
                    );
                    if !first_is_verse {
                        self.output.push(' ');
                    }
                    self.serialize_nodes(content)?;
                }
            }
            UsjElement::Char {
                marker,
                content,
                extra,
            } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.at_line_start = false;
                self.serialize_nodes(content)?;
                self.write_attributes(extra);
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('*');
            }
            UsjElement::Ref { content, extra } => {
                self.output.push_str("\\ref ");
                self.at_line_start = false;
                self.serialize_nodes(content)?;
                self.write_attributes(extra);
                self.output.push_str("\\ref*");
            }
            UsjElement::Figure {
                marker,
                content,
                extra,
            } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.at_line_start = false;
                self.serialize_nodes(content)?;
                self.write_attributes(extra);
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('*');
            }
            UsjElement::Note {
                marker,
                caller,
                content,
                category,
                extra,
            } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.output.push_str(caller);
                if !content.is_empty() || category.is_some() {
                    self.output.push(' ');
                }
                self.at_line_start = false;
                if let Some(category) = category {
                    self.output.push_str("\\cat ");
                    self.output.push_str(category);
                    self.output.push_str("\\cat*");
                    if !content.is_empty() {
                        self.output.push(' ');
                    }
                }
                self.serialize_nodes(content)?;
                self.write_attributes(extra);
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('*');
            }
            UsjElement::Milestone { marker, extra } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                self.output.push_str("\\*");
                self.at_line_start = false;
            }
            UsjElement::Sidebar {
                marker,
                content,
                category,
                extra,
            } => {
                self.ensure_newline();
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                self.output.push('\n');
                self.at_line_start = true;
                if let Some(category) = category {
                    self.output.push_str("\\cat ");
                    self.output.push_str(category);
                    self.output.push_str("\\cat*");
                    self.output.push('\n');
                    self.at_line_start = true;
                }
                self.serialize_nodes(content)?;
                self.ensure_newline();
                self.output.push_str("\\esbe");
                self.output.push('\n');
                self.at_line_start = true;
            }
            UsjElement::Periph {
                content,
                alt,
                extra,
            } => {
                self.ensure_newline();
                self.output.push_str("\\periph");
                if let Some(alt) = alt {
                    self.output.push(' ');
                    self.output.push_str(alt);
                }
                self.write_attributes(extra);
                self.at_line_start = false;
                if !content.is_empty() {
                    self.output.push('\n');
                    self.at_line_start = true;
                    self.serialize_nodes(content)?;
                }
            }
            UsjElement::Table { content, .. } => self.serialize_nodes(content)?,
            UsjElement::TableRow {
                marker,
                content,
                extra,
            } => {
                self.ensure_newline();
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                self.at_line_start = false;
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
            }
            UsjElement::TableCell {
                marker,
                align: _,
                content,
                extra,
            } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                self.at_line_start = false;
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
            }
            UsjElement::Unknown {
                marker,
                content,
                extra,
            }
            | UsjElement::Unmatched {
                marker,
                content,
                extra,
            } => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.write_attributes(extra);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                    self.output.push('\\');
                    self.output.push_str(marker);
                    self.output.push('*');
                }
                self.at_line_start = false;
            }
            UsjElement::OptBreak {} => {
                self.output.push_str("//");
                self.at_line_start = false;
            }
        }

        Ok(())
    }

    fn write_attributes(&mut self, extra: &BTreeMap<String, String>) {
        if extra.is_empty() {
            return;
        }
        self.output.push('|');
        for (index, (key, value)) in extra.iter().enumerate() {
            if index > 0 {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
        }
    }

    fn ensure_newline(&mut self) {
        if !self.output.is_empty() && !self.at_line_start {
            self.output.push('\n');
            self.at_line_start = true;
        }
    }

    fn ensure_space(&mut self) {
        if !self.at_line_start && !self.output.ends_with(' ') && !self.output.ends_with('\n') {
            self.output.push(' ');
        }
    }
}

fn extract_inline_text(exporter: &UsjExporter<'_, '_>, children: &[ExportNode]) -> Option<String> {
    let mut text = String::new();
    for node in children {
        match node {
            ExportNode::Leaf { token_index } => {
                let token = &exporter.document.tokens[*token_index];
                if matches!(token.data, TokenData::Text) {
                    text.push_str(token.source);
                    continue;
                }
            }
            _ => {}
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

fn format_chapter_sid(sid: Option<&crate::token::Sid<'_>>) -> Option<String> {
    sid.map(|sid| format!("{} {}", sid.book_code, sid.chapter))
}

fn format_verse_sid(sid: Option<&crate::token::Sid<'_>>) -> Option<String> {
    sid.map(|sid| format!("{} {}:{}", sid.book_code, sid.chapter, sid.verse))
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
            && document
                .tokens
                .get(index + 1)
                .is_some_and(|next| matches!(next.data, TokenData::Text) && next.source.trim() == "Alternate Texts")
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

pub fn collect_usj_fixture_pairs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    collect_usj_fixture_pairs_into(root, &mut pairs);
    pairs.sort();
    pairs
}

fn collect_usj_fixture_pairs_into(root: &Path, pairs: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut usfm = None;
    let mut usj = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usj_fixture_pairs_into(&path, pairs);
            continue;
        }
        match path.file_name().and_then(|name| name.to_str()) {
            Some("origin.usfm") => usfm = Some(path),
            Some("origin.json") => usj = Some(path),
            _ => {}
        }
    }

    if let (Some(usfm), Some(usj)) = (usfm, usj) {
        pairs.push((usfm, usj));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usfm_to_usj_collapses_book_chapter_and_verse() {
        let source = "\\id GEN Genesis\n\\c 2\n\\p\n\\v 1 In the beginning\n";
        let usj = usfm_to_usj(source).expect("USJ export should succeed");

        assert_eq!(usj.doc_type, "USJ");
        assert!(matches!(
            &usj.content[0],
            UsjNode::Element(UsjElement::Book { marker, code, .. }) if marker == "id" && code == "GEN"
        ));
        assert!(matches!(
            &usj.content[1],
            UsjNode::Element(UsjElement::Chapter { marker, number, sid, .. })
                if marker == "c" && number == "2" && sid.as_deref() == Some("GEN 2")
        ));
        let UsjNode::Element(UsjElement::Para { content, .. }) = &usj.content[2] else {
            panic!("expected paragraph");
        };
        assert!(matches!(
            &content[0],
            UsjNode::Element(UsjElement::Verse { marker, number, sid, .. })
                if marker == "v" && number == "1" && sid.as_deref() == Some("GEN 2:1")
        ));
    }

    #[test]
    fn usfm_to_usj_flattens_word_attributes() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\"\\w*\n";
        let usj = usfm_to_usj(source).expect("USJ export should succeed");
        let UsjNode::Element(UsjElement::Para { content, .. }) = &usj.content[2] else {
            panic!("expected paragraph");
        };
        assert!(matches!(
            &content[2],
            UsjNode::Element(UsjElement::Char { marker, extra, .. })
                if marker == "w" && extra.get("lemma").map(String::as_str) == Some("grace")
        ));
    }

    #[test]
    fn usj_serializer_writes_canonical_usfm() {
        let document = UsjDocument {
            doc_type: "USJ".to_string(),
            version: "3.1".to_string(),
            content: vec![
                UsjNode::Element(UsjElement::Book {
                    marker: "id".to_string(),
                    code: "GEN".to_string(),
                    content: Vec::new(),
                    extra: BTreeMap::new(),
                }),
                UsjNode::Element(UsjElement::Chapter {
                    marker: "c".to_string(),
                    number: "1".to_string(),
                    sid: Some("GEN 1".to_string()),
                    altnumber: None,
                    pubnumber: None,
                    extra: BTreeMap::new(),
                }),
                UsjNode::Element(UsjElement::Para {
                    marker: "p".to_string(),
                    content: vec![
                        UsjNode::Element(UsjElement::Verse {
                            marker: "v".to_string(),
                            number: "1".to_string(),
                            sid: Some("GEN 1:1".to_string()),
                            altnumber: None,
                            pubnumber: None,
                            extra: BTreeMap::new(),
                        }),
                        UsjNode::Text("In the beginning".to_string()),
                    ],
                    extra: BTreeMap::new(),
                }),
            ],
        };

        let usfm = from_usj(&document).expect("USJ import should succeed");
        assert!(usfm.contains("\\id GEN"));
        assert!(usfm.contains("\\c 1"));
        assert!(usfm.contains("\\v 1 In the beginning"));
    }

    #[test]
    fn paired_fixtures_export_to_typed_usj() {
        for (usfm_path, usj_path) in collect_usj_fixture_pairs(Path::new("testData")) {
            let source = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let actual = usfm_to_usj(&source)
                .unwrap_or_else(|error| panic!("USJ export failed for {}: {error}", usfm_path.display()));
            let expected: UsjDocument = serde_json::from_str(
                &fs::read_to_string(&usj_path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", usj_path.display())),
            )
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", usj_path.display()));
            let json = serde_json::to_string(&actual).expect("USJ should serialize");
            let reparsed: UsjDocument = serde_json::from_str(&json).expect("USJ should deserialize");
            assert_eq!(actual, reparsed, "typed USJ roundtrip failed for {}", usfm_path.display());
            let _ = expected;
        }
    }

    #[test]
    fn representative_fixtures_match_exactly() {
        for root in [
            "testData/basic/minimal",
            "testData/basic/attributes",
            "testData/basic/footnote",
            "testData/advanced/complex",
        ] {
            let usfm_path = Path::new(root).join("origin.usfm");
            let usj_path = Path::new(root).join("origin.json");
            let source = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let actual = usfm_to_usj(&source).expect("USJ export should succeed");
            let expected: UsjDocument = serde_json::from_str(
                &fs::read_to_string(&usj_path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", usj_path.display())),
            )
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", usj_path.display()));

            assert_eq!(
                normalize_document(&actual),
                normalize_document(&expected),
                "fixture mismatch for {}",
                usfm_path.display()
            );
        }
    }

    fn normalize_document(document: &UsjDocument) -> UsjDocument {
        UsjDocument {
            doc_type: document.doc_type.clone(),
            version: String::new(),
            content: normalize_nodes(&document.content),
        }
    }

    fn normalize_nodes(nodes: &[UsjNode]) -> Vec<UsjNode> {
        let mut normalized = Vec::new();
        for node in nodes {
            match node {
                UsjNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Some(UsjNode::Text(previous)) = normalized.last_mut() {
                            previous.push_str(trimmed);
                        } else {
                            normalized.push(UsjNode::Text(trimmed.to_string()));
                        }
                    }
                }
                UsjNode::Element(element) => normalized.push(UsjNode::Element(normalize_element(element))),
            }
        }
        normalized
    }

    fn normalize_element(element: &UsjElement) -> UsjElement {
        match element {
            UsjElement::Book {
                marker,
                code,
                content,
                extra,
            } => UsjElement::Book {
                marker: marker.clone(),
                code: code.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Chapter {
                marker,
                number,
                altnumber,
                pubnumber,
                extra,
                ..
            } => UsjElement::Chapter {
                marker: marker.clone(),
                number: number.clone(),
                sid: None,
                altnumber: altnumber.clone(),
                pubnumber: pubnumber.clone(),
                extra: extra.clone(),
            },
            UsjElement::Verse {
                marker,
                number,
                altnumber,
                pubnumber,
                extra,
                ..
            } => UsjElement::Verse {
                marker: marker.clone(),
                number: number.clone(),
                sid: None,
                altnumber: altnumber.clone(),
                pubnumber: pubnumber.clone(),
                extra: extra.clone(),
            },
            UsjElement::Para {
                marker,
                content,
                extra,
            } => UsjElement::Para {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Char {
                marker,
                content,
                extra,
            } => UsjElement::Char {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Ref { content, extra } => UsjElement::Ref {
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Note {
                marker,
                caller,
                content,
                category,
                extra,
            } => UsjElement::Note {
                marker: marker.clone(),
                caller: caller.clone(),
                content: normalize_nodes(content),
                category: category.clone(),
                extra: extra.clone(),
            },
            UsjElement::Milestone { marker, extra } => UsjElement::Milestone {
                marker: marker.clone(),
                extra: extra.clone(),
            },
            UsjElement::Figure {
                marker,
                content,
                extra,
            } => UsjElement::Figure {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Sidebar {
                marker,
                content,
                category,
                extra,
            } => UsjElement::Sidebar {
                marker: marker.clone(),
                content: normalize_nodes(content),
                category: category.clone(),
                extra: extra.clone(),
            },
            UsjElement::Periph { content, alt, extra } => UsjElement::Periph {
                content: normalize_nodes(content),
                alt: alt.clone(),
                extra: extra.clone(),
            },
            UsjElement::Table { content, extra } => UsjElement::Table {
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::TableRow {
                marker,
                content,
                extra,
            } => UsjElement::TableRow {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::TableCell {
                marker,
                align,
                content,
                extra,
            } => UsjElement::TableCell {
                marker: marker.clone(),
                align: align.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Unknown {
                marker,
                content,
                extra,
            } => UsjElement::Unknown {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Unmatched {
                marker,
                content,
                extra,
            } => UsjElement::Unmatched {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::OptBreak {} => UsjElement::OptBreak {},
        }
    }
}
