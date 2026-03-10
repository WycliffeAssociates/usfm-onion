use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use serde_json::Value;
use std::io::Cursor;

use crate::internal::marker_defs::{
    marker_default_attribute, marker_forbidden_in_note_context, marker_is_heading_bridge,
};
use crate::internal::markers::lookup_marker;
use crate::internal::recovery::{ParseRecovery, RecoveryCode, RecoveryPayload};
use crate::internal::syntax::{ContainerKind, ContainerNode, LeafKind, Node};
use crate::model::document_tree::{DocumentTreeDocument, DocumentTreeElement, DocumentTreeNode};
use crate::parse::ParseHandle;

pub fn to_usx_string(handle: &ParseHandle) -> Result<String, UsxError> {
    UsxSerializer::new(handle).write()
}

pub fn document_tree_to_usx_string(document: &DocumentTreeDocument) -> Result<String, UsxError> {
    DocumentTreeUsxSerializer::new(document).write()
}

#[derive(Debug)]
pub enum UsxError {
    Xml(quick_xml::Error),
    Io(std::io::Error),
}

impl From<quick_xml::Error> for UsxError {
    fn from(value: quick_xml::Error) -> Self {
        Self::Xml(value)
    }
}

impl From<std::io::Error> for UsxError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for UsxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(error) => write!(f, "xml serialization error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
        }
    }
}

impl std::error::Error for UsxError {}

#[derive(Clone, Copy)]
struct ContentTrim {
    trim_first: bool,
    trim_last: bool,
}

impl ContentTrim {
    const fn none() -> Self {
        Self {
            trim_first: false,
            trim_last: false,
        }
    }

    const fn container() -> Self {
        Self {
            trim_first: true,
            trim_last: true,
        }
    }

    const fn inline() -> Self {
        Self {
            trim_first: true,
            trim_last: false,
        }
    }
}

struct UsxSerializer<'a> {
    source: &'a str,
    document: &'a crate::internal::syntax::Document,
    recoveries: &'a [ParseRecovery],
    book_code: String,
    current_chapter: String,
    current_chapter_sid: Option<String>,
    current_verse_sid: Option<String>,
    note_depth: usize,
    current_para_marker: Option<String>,
    trim_next_text_start: bool,
    pending_separator_before_verse_eid: bool,
    last_text_ends_with_whitespace: bool,
    preserve_next_leading_space: bool,
    writer: Writer<Cursor<Vec<u8>>>,
}

struct DocumentTreeUsxSerializer<'a> {
    document: &'a DocumentTreeDocument,
    book_code: String,
    current_chapter: String,
    current_chapter_sid: Option<String>,
    current_verse_sid: Option<String>,
    writer: Writer<Cursor<Vec<u8>>>,
}

impl<'a> DocumentTreeUsxSerializer<'a> {
    fn new(document: &'a DocumentTreeDocument) -> Self {
        Self {
            document,
            book_code: String::new(),
            current_chapter: String::new(),
            current_chapter_sid: None,
            current_verse_sid: None,
            writer: Writer::new(Cursor::new(Vec::new())),
        }
    }

    fn write(mut self) -> Result<String, UsxError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        let mut root = BytesStart::new("usx");
        root.push_attribute(("version", self.document.version.as_str()));
        self.writer.write_event(Event::Start(root))?;

        self.write_nodes(self.document.content.as_slice())?;
        self.close_verse()?;
        self.close_chapter()?;
        self.writer.write_event(Event::End(BytesEnd::new("usx")))?;

        let bytes = self.writer.into_inner().into_inner();
        Ok(String::from_utf8(bytes).expect("writer should emit utf-8"))
    }

    fn write_nodes(&mut self, nodes: &[DocumentTreeNode]) -> Result<(), UsxError> {
        for node in nodes {
            let DocumentTreeNode::Element(element) = node;
            self.write_element(element)?;
        }
        Ok(())
    }

    fn write_element(&mut self, element: &DocumentTreeElement) -> Result<(), UsxError> {
        match element {
            DocumentTreeElement::Text { value } => {
                self.writer
                    .write_event(Event::Text(BytesText::new(value)))?;
            }
            DocumentTreeElement::OptBreak {} => {
                self.writer
                    .write_event(Event::Empty(BytesStart::new("optbreak")))?;
            }
            DocumentTreeElement::LineBreak { .. } => {}
            DocumentTreeElement::Book {
                marker,
                code,
                content,
                ..
            } => {
                self.book_code = code.to_ascii_uppercase();
                let mut elem = BytesStart::new("book");
                elem.push_attribute(("code", code.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                if content.is_empty() {
                    self.writer.write_event(Event::Empty(elem))?;
                } else {
                    self.writer.write_event(Event::Start(elem))?;
                    self.write_nodes(content.as_slice())?;
                    self.writer.write_event(Event::End(BytesEnd::new("book")))?;
                }
            }
            DocumentTreeElement::Chapter {
                marker,
                number,
                extra,
            } => {
                self.close_verse()?;
                self.close_chapter()?;

                self.current_chapter = number.clone();
                let sid = string_extra(extra, "sid").unwrap_or_else(|| {
                    if self.book_code.is_empty() || number.trim().is_empty() {
                        String::new()
                    } else {
                        format!("{} {}", self.book_code, strip_leading_zeros(number))
                    }
                });

                let mut elem = BytesStart::new("chapter");
                elem.push_attribute(("number", number.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                if !sid.is_empty() {
                    elem.push_attribute(("sid", sid.as_str()));
                    self.current_chapter_sid = Some(sid);
                }
                push_optional_attr(&mut elem, "altnumber", string_extra(extra, "altnumber"));
                push_optional_attr(&mut elem, "pubnumber", string_extra(extra, "pubnumber"));
                self.writer.write_event(Event::Empty(elem))?;
            }
            DocumentTreeElement::Verse {
                marker,
                number,
                extra,
            } => {
                self.close_verse()?;

                let sid = string_extra(extra, "sid").unwrap_or_else(|| {
                    if self.book_code.is_empty()
                        || self.current_chapter.trim().is_empty()
                        || number.trim().is_empty()
                    {
                        String::new()
                    } else {
                        format!(
                            "{} {}:{}",
                            self.book_code,
                            strip_leading_zeros(self.current_chapter.as_str()),
                            strip_leading_zeros(number)
                        )
                    }
                });

                let mut elem = BytesStart::new("verse");
                elem.push_attribute(("number", number.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                if !sid.is_empty() {
                    elem.push_attribute(("sid", sid.as_str()));
                    self.current_verse_sid = Some(sid);
                }
                push_optional_attr(&mut elem, "altnumber", string_extra(extra, "altnumber"));
                push_optional_attr(&mut elem, "pubnumber", string_extra(extra, "pubnumber"));
                self.writer.write_event(Event::Empty(elem))?;
            }
            DocumentTreeElement::Para {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("para");
                elem.push_attribute(("style", marker.as_str()));
                if matches!(string_extra(extra, "status").as_deref(), Some("unknown")) {
                    elem.push_attribute(("status", "unknown"));
                }
                if content.is_empty() {
                    self.writer.write_event(Event::Empty(elem))?;
                } else {
                    self.writer.write_event(Event::Start(elem))?;
                    self.write_nodes(content.as_slice())?;
                    self.close_verse()?;
                    self.writer.write_event(Event::End(BytesEnd::new("para")))?;
                }
            }
            DocumentTreeElement::Char {
                marker,
                content,
                extra,
            } => self.write_container_with_attrs("char", "style", marker, content, extra)?,
            DocumentTreeElement::Note {
                marker,
                caller,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("note");
                elem.push_attribute(("style", marker.as_str()));
                elem.push_attribute(("caller", caller.as_str()));
                push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
                if content.is_empty() {
                    self.writer.write_event(Event::Empty(elem))?;
                } else {
                    self.writer.write_event(Event::Start(elem))?;
                    self.write_nodes(content.as_slice())?;
                    self.writer.write_event(Event::End(BytesEnd::new("note")))?;
                }
            }
            DocumentTreeElement::Milestone { marker, extra } => {
                let mut elem = BytesStart::new("ms");
                elem.push_attribute(("style", marker.as_str()));
                push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
                self.writer.write_event(Event::Empty(elem))?;
            }
            DocumentTreeElement::Figure {
                marker,
                content,
                extra,
            } => self.write_container_with_attrs("figure", "style", marker, content, extra)?,
            DocumentTreeElement::Sidebar {
                marker,
                content,
                extra,
            } => {
                let saved_verse_sid = self.current_verse_sid.take();
                let mut elem = BytesStart::new("sidebar");
                elem.push_attribute(("style", marker.as_str()));
                push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
                if content.is_empty() {
                    self.writer.write_event(Event::Empty(elem))?;
                } else {
                    self.writer.write_event(Event::Start(elem))?;
                    self.write_nodes(content.as_slice())?;
                    self.close_verse()?;
                    self.writer
                        .write_event(Event::End(BytesEnd::new("sidebar")))?;
                }
                self.current_verse_sid = saved_verse_sid;
            }
            DocumentTreeElement::Periph { content, extra } => {
                let mut elem = BytesStart::new("periph");
                push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
                if content.is_empty() {
                    self.writer.write_event(Event::Empty(elem))?;
                } else {
                    self.writer.write_event(Event::Start(elem))?;
                    self.write_nodes(content.as_slice())?;
                    self.writer
                        .write_event(Event::End(BytesEnd::new("periph")))?;
                }
            }
            DocumentTreeElement::Table { content, .. } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new("table")))?;
                self.write_nodes(content.as_slice())?;
                self.close_verse()?;
                self.writer
                    .write_event(Event::End(BytesEnd::new("table")))?;
            }
            DocumentTreeElement::TableRow {
                marker,
                content,
                extra,
            } => {
                self.write_named_container("row", Some(("style", marker.as_str())), content, extra)?
            }
            DocumentTreeElement::TableCell {
                marker,
                content,
                extra,
                ..
            } => self.write_named_container(
                "cell",
                Some(("style", marker.as_str())),
                content,
                extra,
            )?,
            DocumentTreeElement::Ref { content, extra } => {
                self.write_named_container("ref", None, content, extra)?
            }
            DocumentTreeElement::Unknown {
                marker,
                content,
                extra,
            } => {
                let mut merged_extra = extra.clone();
                merged_extra
                    .entry("status".to_string())
                    .or_insert_with(|| Value::String("unknown".to_string()));
                self.write_named_container(
                    "para",
                    Some(("style", marker.as_str())),
                    content,
                    &merged_extra,
                )?;
            }
            DocumentTreeElement::Unmatched { marker, extra, .. } => {
                let mut elem = BytesStart::new("unmatched");
                elem.push_attribute(("marker", marker.as_str()));
                push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
                self.writer.write_event(Event::Empty(elem))?;
            }
        }
        Ok(())
    }

    fn write_named_container(
        &mut self,
        name: &str,
        fixed_attr: Option<(&str, &str)>,
        content: &[DocumentTreeNode],
        extra: &std::collections::BTreeMap<String, Value>,
    ) -> Result<(), UsxError> {
        let mut elem = BytesStart::new(name);
        if let Some((key, value)) = fixed_attr {
            elem.push_attribute((key, value));
        }
        push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
        if content.is_empty() {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.write_nodes(content)?;
            self.writer.write_event(Event::End(BytesEnd::new(name)))?;
        }
        Ok(())
    }

    fn write_container_with_attrs(
        &mut self,
        name: &str,
        style_attr: &str,
        marker: &str,
        content: &[DocumentTreeNode],
        extra: &std::collections::BTreeMap<String, Value>,
    ) -> Result<(), UsxError> {
        let mut elem = BytesStart::new(name);
        elem.push_attribute((style_attr, marker));
        push_all_extra_attrs(&mut elem, extra, &["markerText", "closed"]);
        if content.is_empty() {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.write_nodes(content)?;
            self.writer.write_event(Event::End(BytesEnd::new(name)))?;
        }
        Ok(())
    }

    fn close_verse(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_verse_sid.take() {
            let mut elem = BytesStart::new("verse");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        Ok(())
    }

    fn close_chapter(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_chapter_sid.take() {
            let mut elem = BytesStart::new("chapter");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        Ok(())
    }
}

impl<'a> UsxSerializer<'a> {
    fn new(handle: &'a ParseHandle) -> Self {
        Self {
            source: handle.source(),
            document: handle.document(),
            recoveries: handle.analysis().recoveries.as_slice(),
            book_code: handle.book_code().unwrap_or_default().to_ascii_uppercase(),
            current_chapter: String::new(),
            current_chapter_sid: None,
            current_verse_sid: None,
            note_depth: 0,
            current_para_marker: None,
            trim_next_text_start: false,
            pending_separator_before_verse_eid: false,
            last_text_ends_with_whitespace: false,
            preserve_next_leading_space: false,
            writer: Writer::new(Cursor::new(Vec::new())),
        }
    }

    fn write(mut self) -> Result<String, UsxError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        let mut root = BytesStart::new("usx");
        root.push_attribute(("version", usx_version(self.source)));
        self.writer.write_event(Event::Start(root))?;

        self.serialize_children(self.document.children.as_slice(), ContentTrim::none())?;
        self.close_verse()?;
        self.close_chapter()?;

        self.writer.write_event(Event::End(BytesEnd::new("usx")))?;
        let bytes = self.writer.into_inner().into_inner();
        Ok(String::from_utf8(bytes).expect("writer should emit utf-8"))
    }

    fn serialize_children(&mut self, nodes: &[Node], trim: ContentTrim) -> Result<(), UsxError> {
        let first = nodes
            .iter()
            .position(|node| !is_ignorable_trivia_node(node, self.source));
        let last = nodes
            .iter()
            .rposition(|node| !is_ignorable_trivia_node(node, self.source));

        let Some(first) = first else {
            return Ok(());
        };
        let Some(last) = last else {
            return Ok(());
        };

        let mut index = first;
        while index <= last {
            if self.try_serialize_compacted_empty_intro_headers(nodes, &mut index)? {
                continue;
            }
            if self.should_skip_separator_trivia(nodes, index) {
                self.pending_separator_before_verse_eid = matches!(
                    nodes[index + 1..]
                        .iter()
                        .find(|candidate| !is_ignorable_trivia_node(candidate, self.source)),
                    Some(Node::Verse { .. } | Node::Chapter { .. })
                );
                index += 1;
                continue;
            }
            if self.try_serialize_table(nodes, &mut index)? {
                continue;
            }
            if self.try_serialize_chapter(nodes, &mut index)? {
                continue;
            }
            if self.try_serialize_verse(nodes, &mut index)? {
                continue;
            }

            let trim_start = trim.trim_first && index == first;
            let trim_end = trim.trim_last && index == last;
            self.serialize_node(&nodes[index], trim_start, trim_end, &nodes[index + 1..])?;
            index += 1;
        }

        Ok(())
    }

    fn try_serialize_compacted_empty_intro_headers(
        &mut self,
        nodes: &[Node],
        index: &mut usize,
    ) -> Result<bool, UsxError> {
        if !self.current_chapter.is_empty() || self.current_verse_sid.is_some() {
            return Ok(false);
        }

        let Some(Node::Container(container)) = nodes.get(*index) else {
            return Ok(false);
        };
        if container.marker != "rem"
            || !children_are_effectively_empty(container.children.as_slice(), self.source)
        {
            return Ok(false);
        }

        let mut next = *index + 1;
        let mut merged_markers = String::new();
        while let Some(node) = nodes.get(next) {
            match node {
                Node::Leaf { .. } if is_ignorable_trivia_node(node, self.source) => {
                    next += 1;
                }
                Node::Container(next_container)
                    if is_mergeable_empty_intro_header(next_container, self.source) =>
                {
                    merged_markers.push('\\');
                    merged_markers.push_str(next_container.marker.as_str());
                    next += 1;
                }
                _ => break,
            }
        }

        if merged_markers.is_empty() {
            return Ok(false);
        }

        let mut elem = BytesStart::new("para");
        elem.push_attribute(("style", "rem"));
        self.writer.write_event(Event::Start(elem))?;
        self.writer
            .write_event(Event::Text(BytesText::new(&merged_markers)))?;
        self.writer.write_event(Event::End(BytesEnd::new("para")))?;
        *index = next;
        Ok(true)
    }

    fn try_serialize_table(&mut self, nodes: &[Node], index: &mut usize) -> Result<bool, UsxError> {
        let Some(Node::Container(ContainerNode {
            kind: ContainerKind::TableRow,
            ..
        })) = nodes.get(*index)
        else {
            return Ok(false);
        };

        let mut elem = BytesStart::new("table");
        if let Some(vid) = self.current_verse_sid.as_deref() {
            elem.push_attribute(("vid", vid));
        }
        self.writer.write_event(Event::Start(elem))?;

        let start = *index;
        while matches!(
            nodes.get(*index),
            Some(Node::Container(ContainerNode {
                kind: ContainerKind::TableRow,
                ..
            }))
        ) {
            *index += 1;
        }

        for (offset, node) in nodes[start..*index].iter().enumerate() {
            let is_last_row = start + offset + 1 == *index;
            let Node::Container(row) = node else {
                continue;
            };
            self.serialize_table_row(row, is_last_row)?;
        }

        self.close_verse()?;
        self.writer
            .write_event(Event::End(BytesEnd::new("table")))?;
        Ok(true)
    }

    fn try_serialize_chapter(
        &mut self,
        nodes: &[Node],
        index: &mut usize,
    ) -> Result<bool, UsxError> {
        let Some(Node::Chapter {
            marker_span,
            number_span,
        }) = nodes.get(*index)
        else {
            return Ok(false);
        };

        self.close_verse()?;
        self.close_chapter()?;

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
                _ if is_ignorable_trivia_node(node, self.source) => next += 1,
                _ => break,
            }
        }

        let marker = normalized_marker(&self.source[marker_span.clone()]);
        let number = number_span
            .as_ref()
            .map(|span| self.source[span.clone()].trim().to_string())
            .unwrap_or_default();
        self.current_chapter = number.clone();

        let sid = if !self.book_code.is_empty() && !number.is_empty() {
            Some(format!(
                "{} {}",
                self.book_code,
                strip_leading_zeros(&number)
            ))
        } else if !number.is_empty() {
            Some(format!(" {}", strip_leading_zeros(&number)))
        } else {
            None
        };

        let mut elem = BytesStart::new("chapter");
        elem.push_attribute(("number", number.as_str()));
        elem.push_attribute(("style", marker.as_str()));
        if let Some(sid) = sid.as_deref() {
            elem.push_attribute(("sid", sid));
            self.current_chapter_sid = Some(sid.to_string());
        }
        if let Some(value) = altnumber.as_deref() {
            elem.push_attribute(("altnumber", value));
        }
        if let Some(value) = pubnumber.as_deref() {
            elem.push_attribute(("pubnumber", value));
        }
        self.writer.write_event(Event::Empty(elem))?;
        self.trim_next_text_start = true;
        *index = next;
        Ok(true)
    }

    fn try_serialize_verse(&mut self, nodes: &[Node], index: &mut usize) -> Result<bool, UsxError> {
        let Some(Node::Verse {
            marker_span,
            number_span,
        }) = nodes.get(*index)
        else {
            return Ok(false);
        };

        self.close_verse()?;

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
                _ if is_ignorable_trivia_node(node, self.source) => next += 1,
                _ => break,
            }
        }

        let marker = normalized_marker(&self.source[marker_span.clone()]);
        let number = number_span
            .as_ref()
            .map(|span| self.source[span.clone()].trim().to_string())
            .unwrap_or_default();
        let sid = if self.book_code.is_empty() {
            Some(String::new())
        } else {
            let chapter = if self.current_chapter.is_empty() {
                "1".to_string()
            } else {
                strip_leading_zeros(self.current_chapter.as_str())
            };
            let normalized_number = strip_leading_zeros(&number);
            Some(format!(
                "{} {}:{}",
                self.book_code, chapter, normalized_number
            ))
        };

        let mut elem = BytesStart::new("verse");
        elem.push_attribute(("number", number.as_str()));
        elem.push_attribute(("style", marker.as_str()));
        if let Some(sid) = sid.as_deref() {
            elem.push_attribute(("sid", sid));
            self.current_verse_sid = Some(sid.to_string());
        }
        if self
            .current_para_marker
            .as_deref()
            .is_some_and(|paragraph| !paragraph_allows_verse_start(paragraph))
        {
            elem.push_attribute(("status", "invalid"));
        }
        if let Some(value) = altnumber.as_deref() {
            elem.push_attribute(("altnumber", value));
        }
        if let Some(value) = pubnumber.as_deref() {
            elem.push_attribute(("pubnumber", value));
        }
        self.writer.write_event(Event::Empty(elem))?;
        self.trim_next_text_start = true;
        *index = next;
        Ok(true)
    }

    fn serialize_node(
        &mut self,
        node: &Node,
        trim_start: bool,
        trim_end: bool,
        following: &[Node],
    ) -> Result<(), UsxError> {
        match node {
            Node::Container(container) => self.serialize_container(container, following),
            Node::Chapter { .. } | Node::Verse { .. } => Ok(()),
            Node::Milestone {
                marker,
                marker_span,
                attribute_spans,
                closed,
            } => self.serialize_milestone(marker, marker_span, attribute_spans, *closed),
            Node::Leaf { kind, span } => self.serialize_leaf(*kind, span, trim_start, trim_end),
        }
    }

    fn serialize_container(
        &mut self,
        container: &ContainerNode,
        following: &[Node],
    ) -> Result<(), UsxError> {
        if container.marker == "usfm" {
            return Ok(());
        }

        match container.kind {
            ContainerKind::Book => self.serialize_book(container),
            ContainerKind::Paragraph
            | ContainerKind::Header
            | ContainerKind::Meta
            | ContainerKind::Unknown => {
                if container.marker == "esbe" || container.marker == "*" {
                    return self.serialize_unmatched(container);
                }
                let continues_verse = self.paragraph_chain_continues_verse(following);
                self.serialize_para(container, continues_verse)
            }
            ContainerKind::Character => self.serialize_char(container),
            ContainerKind::Note => self.serialize_note(container),
            ContainerKind::Figure => self.serialize_figure(container),
            ContainerKind::Sidebar => self.serialize_sidebar(container),
            ContainerKind::Periph => self.serialize_periph(container),
            ContainerKind::TableRow => self.serialize_table_row(container, false),
            ContainerKind::TableCell => self.serialize_table_cell(container, false),
        }
    }

    fn serialize_book(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("book");
        let code = container
            .special_span
            .as_ref()
            .map(|span| self.source[span.clone()].trim())
            .filter(|code| !code.is_empty())
            .map(|code| code.to_ascii_uppercase())
            .unwrap_or_else(|| self.book_code.clone());
        elem.push_attribute(("code", code.as_str()));
        elem.push_attribute(("style", container.marker.as_str()));
        if container
            .children
            .iter()
            .all(|node| is_ignorable_trivia_node(node, self.source))
        {
            self.writer.write_event(Event::Empty(elem))?;
            return Ok(());
        }
        self.writer.write_event(Event::Start(elem))?;
        self.serialize_children(container.children.as_slice(), ContentTrim::container())?;
        self.writer.write_event(Event::End(BytesEnd::new("book")))?;
        Ok(())
    }

    fn serialize_para(
        &mut self,
        container: &ContainerNode,
        continues_verse: bool,
    ) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("para");
        elem.push_attribute(("style", container.marker.as_str()));
        if let Some(vid) = para_vid(
            container.marker.as_str(),
            container.children.as_slice(),
            self.current_verse_sid.as_deref(),
            self.source,
            continues_verse,
        ) {
            elem.push_attribute(("vid", vid));
        }
        if matches!(container.kind, ContainerKind::Unknown)
            || lookup_marker(container.marker.as_str()).kind
                == crate::internal::markers::MarkerKind::Unknown
        {
            elem.push_attribute(("status", "unknown"));
        } else if container_contains_verse(container.children.as_slice())
            && self.current_chapter.is_empty()
            || paragraph_is_invalid_for_current_position(
                container.marker.as_str(),
                self.current_chapter.as_str(),
                self.current_verse_sid.as_deref(),
            )
        {
            elem.push_attribute(("status", "invalid"));
        }
        if children_are_effectively_empty(container.children.as_slice(), self.source) {
            self.writer.write_event(Event::Empty(elem))?;
            return Ok(());
        }
        self.writer.write_event(Event::Start(elem))?;
        let previous_para_marker = self.current_para_marker.replace(container.marker.clone());
        self.serialize_children(container.children.as_slice(), ContentTrim::container())?;
        self.current_para_marker = previous_para_marker;
        if !self.keep_verse_open_after_para(container, continues_verse) {
            self.close_verse()?;
        }
        self.writer.write_event(Event::End(BytesEnd::new("para")))?;
        Ok(())
    }

    fn serialize_char(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        if container.marker == "ref" {
            return self.serialize_ref(container);
        }

        let mut elem = BytesStart::new("char");
        elem.push_attribute(("style", container.marker.as_str()));
        let content_has_trailing_whitespace =
            content_ends_with_whitespace(container.children.as_slice(), self.source);
        let attr_gap_before =
            attribute_spans_have_leading_gap(self.source, &container.attribute_spans);
        let raw_fallback = match resolve_attribute_behavior_with_children(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
            container.children.as_slice(),
            attr_gap_before,
        ) {
            AttributeBehavior::Flatten(attributes) => {
                for (key, value) in attributes {
                    elem.push_attribute((key.as_str(), value.as_str()));
                }
                None
            }
            AttributeBehavior::RawText(raw) => Some(normalize_preserved_raw_attributes(
                container.marker.as_str(),
                &raw,
                attr_gap_before,
                content_has_trailing_whitespace,
            )),
        };
        if (lookup_marker(container.marker.as_str()).valid_in_note && self.note_depth == 0)
            || (self.note_depth > 0 && marker_is_invalid_in_note_context(container.marker.as_str()))
        {
            elem.push_attribute(("status", "invalid"));
        }
        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str()) {
            elem.push_attribute(("closed", "false"));
        }

        let trim_last = self.marker_was_unclosed(&container.marker_span, container.marker.as_str())
            || content_ends_with_newline_node(container.children.as_slice());
        let trim = ContentTrim {
            trim_first: true,
            trim_last,
        };

        if children_are_effectively_empty(container.children.as_slice(), self.source)
            && raw_fallback.is_none()
        {
            self.writer.write_event(Event::Empty(elem))?;
            return Ok(());
        }
        self.writer.write_event(Event::Start(elem))?;
        self.serialize_children(container.children.as_slice(), trim)?;
        if content_has_trailing_whitespace {
            self.preserve_next_leading_space = true;
        }
        if !trim_last
            && trailing_separator_after_last_content(container.children.as_slice(), self.source)
        {
            self.writer.write_event(Event::Text(BytesText::new(" ")))?;
            self.last_text_ends_with_whitespace = true;
            self.preserve_next_leading_space = true;
        }
        if let Some(raw) = raw_fallback.as_deref() {
            self.writer.write_event(Event::Text(BytesText::new(raw)))?;
        }
        self.writer.write_event(Event::End(BytesEnd::new("char")))?;
        Ok(())
    }

    fn serialize_ref(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("ref");
        for (key, value) in collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        ) {
            elem.push_attribute((key.as_str(), value.as_str()));
        }
        if children_are_effectively_empty(container.children.as_slice(), self.source) {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.serialize_children(container.children.as_slice(), ContentTrim::inline())?;
            self.writer.write_event(Event::End(BytesEnd::new("ref")))?;
        }
        Ok(())
    }

    fn serialize_note(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("note");
        let caller = container
            .special_span
            .as_ref()
            .map(|span| self.source[span.clone()].trim())
            .unwrap_or("");
        elem.push_attribute(("caller", caller));
        elem.push_attribute(("style", container.marker.as_str()));
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        let normalized = normalize_note_child_spacing(filtered.as_slice(), self.source);
        if let Some(category) = category.as_deref() {
            elem.push_attribute(("category", category));
        }
        if self.note_was_unclosed(&container.marker_span) {
            elem.push_attribute(("closed", "false"));
        }
        if normalized
            .iter()
            .all(|node| is_ignorable_trivia_node(node, self.source))
        {
            self.writer.write_event(Event::Empty(elem))?;
            return Ok(());
        }
        self.writer.write_event(Event::Start(elem))?;
        self.note_depth += 1;
        self.serialize_children(normalized.as_slice(), ContentTrim::none())?;
        self.note_depth -= 1;
        self.writer.write_event(Event::End(BytesEnd::new("note")))?;
        Ok(())
    }

    fn serialize_figure(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str()) {
            let mut elem = BytesStart::new("char");
            elem.push_attribute(("style", container.marker.as_str()));
            elem.push_attribute(("closed", "false"));
            self.writer.write_event(Event::Start(elem))?;
            self.serialize_children(container.children.as_slice(), ContentTrim::inline())?;
            if !container.attribute_spans.is_empty() {
                let raw = join_attribute_spans(self.source, &container.attribute_spans);
                self.writer.write_event(Event::Text(BytesText::new(&raw)))?;
            }
            self.writer.write_event(Event::End(BytesEnd::new("char")))?;
            return Ok(());
        }

        let mut elem = BytesStart::new("figure");
        elem.push_attribute(("style", container.marker.as_str()));
        for (key, value) in collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        ) {
            elem.push_attribute((key.as_str(), value.as_str()));
        }
        if children_are_effectively_empty(container.children.as_slice(), self.source) {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.serialize_children(container.children.as_slice(), ContentTrim::inline())?;
            self.writer
                .write_event(Event::End(BytesEnd::new("figure")))?;
        }
        Ok(())
    }

    fn serialize_sidebar(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("sidebar");
        elem.push_attribute(("style", container.marker.as_str()));
        let (category, filtered) =
            extract_category_nodes(container.children.as_slice(), self.source);
        if let Some(category) = category.as_deref() {
            elem.push_attribute(("category", category));
        }
        if self.marker_was_unclosed(&container.marker_span, container.marker.as_str()) {
            elem.push_attribute(("closed", "false"));
        }
        let saved_verse_sid = self.current_verse_sid.take();
        let saved_pending_separator = self.pending_separator_before_verse_eid;
        let saved_last_text_ends_with_whitespace = self.last_text_ends_with_whitespace;
        self.pending_separator_before_verse_eid = false;
        if filtered
            .iter()
            .all(|node| is_ignorable_trivia_node(node, self.source))
        {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.serialize_children(filtered.as_slice(), ContentTrim::container())?;
            self.writer
                .write_event(Event::End(BytesEnd::new("sidebar")))?;
        }
        self.current_verse_sid = saved_verse_sid;
        self.pending_separator_before_verse_eid = saved_pending_separator;
        self.last_text_ends_with_whitespace = saved_last_text_ends_with_whitespace;
        Ok(())
    }

    fn serialize_unmatched(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("unmatched");
        elem.push_attribute(("marker", container.marker.as_str()));
        self.writer.write_event(Event::Empty(elem))?;
        Ok(())
    }

    fn serialize_periph(&mut self, container: &ContainerNode) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("periph");
        for (key, value) in collect_attributes(
            self.source,
            container.marker.as_str(),
            &container.attribute_spans,
        ) {
            elem.push_attribute((key.as_str(), value.as_str()));
        }
        let (alt, consumed) = extract_periph_alt(container.children.as_slice(), self.source);
        if let Some(alt) = alt.as_deref() {
            elem.push_attribute(("alt", alt));
        }
        let remaining = &container.children[consumed..];
        if remaining
            .iter()
            .all(|node| is_ignorable_trivia_node(node, self.source))
        {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.serialize_children(remaining, ContentTrim::container())?;
            self.writer
                .write_event(Event::End(BytesEnd::new("periph")))?;
        }
        Ok(())
    }

    fn serialize_table_row(
        &mut self,
        container: &ContainerNode,
        close_verse_in_last_cell: bool,
    ) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("row");
        elem.push_attribute(("style", container.marker.as_str()));
        self.writer.write_event(Event::Start(elem))?;
        let last_cell_index = container.children.iter().rposition(|node| {
            matches!(
                node,
                Node::Container(ContainerNode {
                    kind: ContainerKind::TableCell,
                    ..
                })
            )
        });
        for (index, child) in container.children.iter().enumerate() {
            if is_ignorable_trivia_node(child, self.source) {
                continue;
            }
            if let Node::Container(cell) = child
                && matches!(cell.kind, ContainerKind::TableCell)
            {
                self.serialize_table_cell(
                    cell,
                    close_verse_in_last_cell && Some(index) == last_cell_index,
                )?;
                continue;
            }
            self.serialize_node(child, false, false, &container.children[index + 1..])?;
        }
        self.writer.write_event(Event::End(BytesEnd::new("row")))?;
        Ok(())
    }

    fn serialize_table_cell(
        &mut self,
        container: &ContainerNode,
        close_verse_after_content: bool,
    ) -> Result<(), UsxError> {
        let mut elem = BytesStart::new("cell");
        let align = table_cell_alignment(container.marker.as_str());
        elem.push_attribute(("style", container.marker.as_str()));
        elem.push_attribute(("align", align.as_str()));
        self.writer.write_event(Event::Start(elem))?;
        self.serialize_children(container.children.as_slice(), ContentTrim::none())?;
        if close_verse_after_content {
            self.close_verse()?;
        }
        self.writer.write_event(Event::End(BytesEnd::new("cell")))?;
        Ok(())
    }

    fn serialize_milestone(
        &mut self,
        marker: &str,
        marker_span: &std::ops::Range<usize>,
        attribute_spans: &[std::ops::Range<usize>],
        closed: bool,
    ) -> Result<(), UsxError> {
        if !closed {
            let mut raw = self.source[marker_span.clone()].to_string();
            raw.push_str(&join_attribute_spans(self.source, attribute_spans));
            self.writer.write_event(Event::Text(BytesText::new(&raw)))?;
            return Ok(());
        }

        let mut elem = BytesStart::new("ms");
        elem.push_attribute(("style", marker));
        for (key, value) in collect_attributes(self.source, marker, attribute_spans) {
            elem.push_attribute((key.as_str(), value.as_str()));
        }
        self.writer.write_event(Event::Empty(elem))?;
        Ok(())
    }

    fn serialize_leaf(
        &mut self,
        kind: LeafKind,
        span: &std::ops::Range<usize>,
        trim_start: bool,
        trim_end: bool,
    ) -> Result<(), UsxError> {
        let raw = &self.source[span.clone()];
        let mut text = match kind {
            LeafKind::Attributes => raw.to_string(),
            LeafKind::OptBreak => {
                self.writer
                    .write_event(Event::Empty(BytesStart::new("optbreak")))?;
                self.last_text_ends_with_whitespace = false;
                return Ok(());
            }
            LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline => normalize_text(raw, false),
        };
        if self.trim_next_text_start {
            text = trim_ascii_start(&text).to_string();
        }
        if trim_start {
            text = trim_ascii_start(&text).to_string();
        }
        if trim_end {
            text = trim_ascii_end(&text).to_string();
        }
        if self.last_text_ends_with_whitespace
            && text.starts_with(' ')
            && !self.preserve_next_leading_space
        {
            text = trim_ascii_start(&text).to_string();
        }
        if !text.is_empty() {
            self.write_text_with_optbreaks(&text)?;
            self.trim_next_text_start = false;
            self.preserve_next_leading_space = false;
        }
        Ok(())
    }

    fn write_text_with_optbreaks(&mut self, text: &str) -> Result<(), UsxError> {
        if !text.contains("//") {
            self.write_text_segment(text)?;
            return Ok(());
        }

        let mut remainder = text;
        while let Some(index) = remainder.find("//") {
            let before = &remainder[..index];
            self.write_text_segment(before)?;
            self.writer
                .write_event(Event::Empty(BytesStart::new("optbreak")))?;
            self.last_text_ends_with_whitespace = false;
            remainder = &remainder[index + 2..];
        }

        self.write_text_segment(remainder)
    }

    fn write_text_segment(&mut self, text: &str) -> Result<(), UsxError> {
        if text.is_empty() {
            return Ok(());
        }
        self.writer.write_event(Event::Text(BytesText::new(text)))?;
        self.last_text_ends_with_whitespace = text.chars().last().is_some_and(char::is_whitespace);
        Ok(())
    }

    fn close_verse(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_verse_sid.take() {
            if self.pending_separator_before_verse_eid && !self.last_text_ends_with_whitespace {
                self.writer.write_event(Event::Text(BytesText::new(" ")))?;
                self.last_text_ends_with_whitespace = true;
            }
            let mut elem = BytesStart::new("verse");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        self.pending_separator_before_verse_eid = false;
        Ok(())
    }

    fn close_chapter(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_chapter_sid.take() {
            let mut elem = BytesStart::new("chapter");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        Ok(())
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
                || (recovery.code == RecoveryCode::MisnestedCloseMarker
                    && recovery.related_span.as_ref() == Some(marker_span)
                    && matches!(
                        recovery.payload.as_ref(),
                        Some(RecoveryPayload::Close { open, .. }) if open == marker
                    ))
                || (recovery.code == RecoveryCode::ImplicitlyClosedMarker
                    && recovery.related_span.as_ref() == Some(marker_span)
                    && matches!(
                        recovery.payload.as_ref(),
                        Some(RecoveryPayload::Close { open, .. }) if open == marker
                    ))
        })
    }

    fn keep_verse_open_after_para(&self, container: &ContainerNode, continues_verse: bool) -> bool {
        let paragraph_has_or_inherits_verse = self.current_verse_sid.is_some()
            || container_contains_verse(container.children.as_slice());

        paragraph_has_or_inherits_verse
            && para_can_keep_verse_open(container.marker.as_str())
            && continues_verse
    }

    fn paragraph_chain_continues_verse(&self, nodes: &[Node]) -> bool {
        let mut index = 0usize;
        while index < nodes.len() {
            let node = &nodes[index];
            if is_ignorable_trivia_node(node, self.source) {
                index += 1;
                continue;
            }

            match node {
                Node::Chapter { .. } | Node::Verse { .. } => return false,
                Node::Container(container) if matches!(container.kind, ContainerKind::TableRow) => {
                    return true;
                }
                Node::Container(container) if matches!(container.kind, ContainerKind::Sidebar) => {
                    index += 1;
                    continue;
                }
                Node::Container(container)
                    if matches!(
                        container.kind,
                        ContainerKind::Paragraph
                            | ContainerKind::Header
                            | ContainerKind::Meta
                            | ContainerKind::Unknown
                    ) =>
                {
                    if children_are_effectively_empty(container.children.as_slice(), self.source) {
                        if allows_empty_vid_bridge(container.marker.as_str()) {
                            index += 1;
                            continue;
                        }
                        return false;
                    }
                    if starts_with_verse(container.children.as_slice(), self.source) {
                        return false;
                    }
                    if para_supports_vid(container.marker.as_str()) {
                        return true;
                    }
                    if is_heading_bridge_marker(container.marker.as_str()) {
                        index += 1;
                        continue;
                    }
                    return false;
                }
                _ => return false,
            }
        }

        false
    }

    fn should_skip_separator_trivia(&self, nodes: &[Node], index: usize) -> bool {
        let Some(node) = nodes.get(index) else {
            return false;
        };
        if !is_ignorable_trivia_node(node, self.source) {
            return false;
        }

        let next = nodes[index + 1..]
            .iter()
            .find(|candidate| !is_ignorable_trivia_node(candidate, self.source));

        let Some(next) = next else {
            return false;
        };

        let previous = nodes[..index]
            .iter()
            .rfind(|candidate| !is_ignorable_trivia_node(candidate, self.source));

        match next {
            Node::Chapter { .. } => true,
            Node::Verse { .. } => {
                self.current_verse_sid.is_some()
                    || previous.is_none()
                    || self.last_text_ends_with_whitespace
            }
            _ => false,
        }
    }
}

fn para_vid<'a>(
    marker: &str,
    children: &[Node],
    current_verse_sid: Option<&'a str>,
    source: &str,
    continues_verse: bool,
) -> Option<&'a str> {
    if starts_with_verse(children, source) {
        None
    } else if children_are_effectively_empty(children, source) {
        if allows_empty_vid_bridge(marker) && continues_verse {
            current_verse_sid
        } else {
            None
        }
    } else if para_supports_vid(marker) || (is_heading_bridge_marker(marker) && continues_verse) {
        current_verse_sid
    } else {
        None
    }
}

fn starts_with_verse(nodes: &[Node], source: &str) -> bool {
    nodes
        .iter()
        .find(|node| !is_ignorable_trivia_node(node, source))
        .is_some_and(|node| matches!(node, Node::Verse { .. }))
}

fn children_are_effectively_empty(children: &[Node], source: &str) -> bool {
    children
        .iter()
        .all(|node| is_ignorable_trivia_node(node, source))
}

fn container_contains_verse(children: &[Node]) -> bool {
    children
        .iter()
        .any(|node| matches!(node, Node::Verse { .. }))
}

fn para_supports_vid(marker: &str) -> bool {
    marker == "p"
        || marker == "m"
        || marker == "po"
        || marker == "pr"
        || marker == "pc"
        || marker == "pm"
        || marker == "pmo"
        || marker == "pmc"
        || marker == "pmr"
        || marker == "pi"
        || marker.starts_with("pi")
        || marker == "nb"
        || marker == "mi"
        || marker == "cls"
        || marker == "b"
        || marker == "lh"
        || marker == "lf"
        || marker == "li"
        || marker.strip_prefix("li").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
        })
        || marker == "q"
        || marker.starts_with('q')
}

fn is_mergeable_empty_intro_header(container: &ContainerNode, source: &str) -> bool {
    children_are_effectively_empty(container.children.as_slice(), source)
        && matches!(
            container.marker.as_str(),
            "h" | "mt" | "mt1" | "mt2" | "mt3" | "mt4"
        )
}

fn paragraph_allows_verse_start(marker: &str) -> bool {
    para_supports_vid(marker) || marker == "lit"
}

fn paragraph_is_invalid_for_current_position(
    marker: &str,
    current_chapter: &str,
    current_verse_sid: Option<&str>,
) -> bool {
    if current_chapter.is_empty() {
        return matches!(
            marker,
            "p" | "m"
                | "po"
                | "pr"
                | "pc"
                | "pm"
                | "pmo"
                | "pmc"
                | "pmr"
                | "pi"
                | "pi1"
                | "pi2"
                | "pi3"
                | "nb"
                | "mi"
                | "cls"
                | "li"
                | "li1"
                | "li2"
                | "li3"
                | "li4"
                | "q"
                | "q1"
                | "q2"
                | "q3"
                | "q4"
                | "qm"
                | "qm1"
                | "qm2"
                | "qm3"
                | "lh"
                | "lf"
                | "b"
        );
    }

    if is_intro_paragraph(marker) {
        return true;
    }

    current_verse_sid.is_some() && !para_supports_vid(marker) && !marker.starts_with('s')
}

fn is_intro_paragraph(marker: &str) -> bool {
    matches!(
        marker,
        "ip" | "ipi"
            | "im"
            | "imi"
            | "ipq"
            | "imq"
            | "ipr"
            | "iq"
            | "iq1"
            | "iq2"
            | "iq3"
            | "iot"
            | "io"
            | "io1"
            | "io2"
            | "io3"
            | "io4"
            | "ili"
            | "ili1"
            | "ili2"
            | "iex"
            | "ie"
    )
}

fn para_can_keep_verse_open(marker: &str) -> bool {
    para_supports_vid(marker) || marker == "lit" || marker_is_heading_bridge(marker)
}

fn is_heading_bridge_marker(marker: &str) -> bool {
    marker_is_heading_bridge(marker)
}

fn allows_empty_vid_bridge(marker: &str) -> bool {
    marker == "b" || marker == "p"
}

fn normalized_marker(text: &str) -> String {
    text.trim_start_matches('\\')
        .trim_end_matches('*')
        .to_string()
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

enum AttributeBehavior {
    Flatten(Vec<(String, String)>),
    RawText(String),
}

fn resolve_attribute_behavior_with_children(
    source: &str,
    marker: &str,
    attribute_spans: &[std::ops::Range<usize>],
    children: &[Node],
    attr_gap_before: bool,
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
            attr_gap_before,
        ) {
            return AttributeBehavior::RawText(join_attribute_spans(source, attribute_spans));
        }
    }

    for (_, parsed) in parsed_spans {
        attrs.extend(resolve_default_attr_keys(marker, parsed));
    }

    AttributeBehavior::Flatten(attrs)
}

fn is_ignorable_figure_placeholder(marker: &str, key: &str, value: &str) -> bool {
    marker == "fig" && key == "file" && value.chars().all(|ch| ch == '|' || ch.is_whitespace())
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
        let end_quote = find_unescaped_quote(remaining)?;
        let value = remaining[..end_quote].replace("\\\"", "\"");
        out.push((key, value));
        remaining = &remaining[end_quote + 1..];
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

fn normalize_note_child_spacing(nodes: &[Node], source: &str) -> Vec<Node> {
    let mut normalized = Vec::with_capacity(nodes.len());
    let mut index = 0usize;

    while index < nodes.len() {
        let Some(node) = nodes.get(index) else {
            break;
        };

        if let Node::Container(container) = node
            && matches!(container.kind, ContainerKind::Character)
        {
            let mut next = index + 1;
            let mut separator = Vec::new();
            while next < nodes.len() && is_ignorable_trivia_node(&nodes[next], source) {
                separator.push(nodes[next].clone());
                next += 1;
            }

            if !separator.is_empty()
                && matches!(
                    nodes.get(next),
                    Some(Node::Container(ContainerNode {
                        kind: ContainerKind::Character,
                        ..
                    }))
                )
            {
                let mut adjusted = container.clone();
                adjusted.children.extend(separator);
                normalized.push(Node::Container(adjusted));
                index = next;
                continue;
            }
        }

        normalized.push(node.clone());
        index += 1;
    }

    normalized
}

fn is_category_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Container(ContainerNode { marker, .. }) if marker == "cat"
    )
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

fn trailing_separator_after_last_content(nodes: &[Node], source: &str) -> bool {
    let Some(last_content_index) = nodes
        .iter()
        .rposition(|node| !is_ignorable_trivia_node(node, source))
    else {
        return false;
    };

    nodes[last_content_index + 1..]
        .iter()
        .any(|node| is_ignorable_trivia_node(node, source))
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
    children
        .iter()
        .rfind(|node| {
            !matches!(
                node,
                Node::Leaf {
                    kind: LeafKind::Whitespace,
                    ..
                }
            )
        })
        .is_some_and(|node| {
            matches!(
                node,
                Node::Leaf {
                    kind: LeafKind::Newline,
                    ..
                }
            )
        })
}

fn is_ignorable_trivia_node(node: &Node, source: &str) -> bool {
    match node {
        Node::Leaf {
            kind: LeafKind::Text | LeafKind::Whitespace | LeafKind::Newline,
            span,
        } => {
            if source.is_empty() {
                true
            } else {
                normalize_text(&source[span.clone()], false)
                    .trim()
                    .is_empty()
            }
        }
        Node::Leaf {
            kind: LeafKind::OptBreak | LeafKind::Attributes,
            ..
        } => false,
        _ => false,
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
    collapse_spaces(&with_spaces)
}

fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous_was_space = false;
    for ch in text.chars() {
        if matches!(ch, ' ' | '\t') {
            if !previous_was_space {
                out.push(' ');
            }
            previous_was_space = true;
        } else {
            previous_was_space = false;
            out.push(ch);
        }
    }
    out
}

fn trim_ascii_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\n', '\r', '\t'])
}

fn trim_ascii_end(text: &str) -> &str {
    text.trim_end_matches([' ', '\n', '\r', '\t'])
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

fn string_extra(extra: &std::collections::BTreeMap<String, Value>, key: &str) -> Option<String> {
    extra
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn push_optional_attr(elem: &mut BytesStart<'_>, key: &'static str, value: Option<String>) {
    if let Some(value) = value {
        elem.push_attribute((key, value.as_str()));
    }
}

fn push_all_extra_attrs(
    elem: &mut BytesStart<'_>,
    extra: &std::collections::BTreeMap<String, Value>,
    excluded: &[&str],
) {
    for (key, value) in extra {
        if excluded
            .iter()
            .any(|excluded_key| excluded_key == &key.as_str())
        {
            continue;
        }

        match value {
            Value::Null => {}
            Value::Bool(value) => elem.push_attribute((key.as_str(), bool_attr(value).as_str())),
            Value::Number(value) => elem.push_attribute((key.as_str(), value.to_string().as_str())),
            Value::String(value) => elem.push_attribute((key.as_str(), value.as_str())),
            Value::Array(_) | Value::Object(_) => {
                let encoded = serde_json::to_string(value)
                    .expect("serde_json::Value should serialize for XML attrs");
                elem.push_attribute((key.as_str(), encoded.as_str()));
            }
        }
    }
}

fn bool_attr(value: &bool) -> String {
    if *value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn total_attribute_count(parsed_spans: &[(&str, Vec<(String, String)>)]) -> usize {
    parsed_spans.iter().map(|(_, parsed)| parsed.len()).sum()
}

fn should_preserve_raw_attributes(
    marker: &str,
    _raw: &str,
    attrs: &[(String, String)],
    _content_has_trailing_whitespace: bool,
    _total_attr_count: usize,
    attr_gap_before: bool,
) -> bool {
    let default_key = marker_default_attribute(marker);

    if attrs
        .iter()
        .any(|(key, _)| key == "default" && default_key.is_none())
    {
        return true;
    }

    if marker == "w"
        && attr_gap_before
        && _total_attr_count > 1
        && let Some(default_key) = default_key
        && attrs.first().is_some_and(|(key, value)| {
            key == default_key && !value.is_empty() && !value.chars().any(char::is_whitespace)
        })
    {
        return true;
    }
    false
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

fn marker_is_invalid_in_note_context(marker: &str) -> bool {
    marker_forbidden_in_note_context(marker)
}

fn usx_version(source: &str) -> &str {
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
        .unwrap_or("3.0")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn writes_basic_usx() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let xml = to_usx_string(&handle).expect("USX should serialize");
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(xml.contains("<usx version=\"3.0\">"));
        assert!(xml.contains("<book code=\"GEN\" style=\"id\">Genesis</book>"));
        assert!(xml.contains("<chapter number=\"1\" style=\"c\" sid=\"GEN 1\"/>"));
        assert!(xml.contains("<verse number=\"1\" style=\"v\" sid=\"GEN 1:1\"/>"));
        assert!(xml.contains("In the beginning"));
        assert!(xml.contains("<verse eid=\"GEN 1:1\"/>"));
    }

    #[test]
    fn keeps_trailing_space_inside_inline_char() {
        let handle = parse("\\id MAT Test\n\\im \\k Book: \\k* Matthew\n");
        let xml = to_usx_string(&handle).expect("USX should serialize");
        assert!(xml.contains("<char style=\"k\">Book: </char> Matthew"));
    }

    #[test]
    fn serializes_numbered_list_value_markers_as_chars() {
        let handle = parse(
            "\\id 1CH Test\n\\c 27\n\\li1 \\lik Reuben\\lik* \\liv1 Eliezer son of Zichri\\liv1*\n",
        );
        let xml = to_usx_string(&handle).expect("USX should serialize");
        assert!(xml.contains("<para style=\"li1\"><char style=\"lik\">Reuben</char> <char style=\"liv1\">Eliezer son of Zichri</char></para>"));
    }
}
