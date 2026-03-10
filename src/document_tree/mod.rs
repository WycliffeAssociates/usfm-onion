pub use crate::internal::api::{DocumentError, DocumentFormat, read_document};
pub use crate::model::document_tree::{
    DocumentTreeDocument, DocumentTreeElement, DocumentTreeNode,
};

use crate::convert;
use crate::internal::api;
use crate::model::token::Token;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub fn usfm_to_document_tree(source: &str) -> DocumentTreeDocument {
    let handle = crate::internal::parse::parse(source);
    api::into_document_tree(&handle)
}

pub fn usj_to_document_tree(source: &str) -> Result<DocumentTreeDocument, DocumentError> {
    let usfm = convert::from_usj_str(source)?;
    Ok(usfm_to_document_tree(&usfm))
}

pub fn usx_to_document_tree(source: &str) -> Result<DocumentTreeDocument, DocumentError> {
    let usfm = convert::from_usx_str(source)?;
    Ok(usfm_to_document_tree(&usfm))
}

pub fn read_usfm_to_document_tree(
    path: impl AsRef<Path>,
) -> Result<DocumentTreeDocument, DocumentError> {
    let source = read_document(path, DocumentFormat::Usfm)?;
    Ok(usfm_to_document_tree(&source))
}

pub fn read_usj_to_document_tree(
    path: impl AsRef<Path>,
) -> Result<DocumentTreeDocument, DocumentError> {
    let source = read_document(path, DocumentFormat::Usj)?;
    usj_to_document_tree(&source)
}

pub fn read_usx_to_document_tree(
    path: impl AsRef<Path>,
) -> Result<DocumentTreeDocument, DocumentError> {
    let source = read_document(path, DocumentFormat::Usx)?;
    usx_to_document_tree(&source)
}

pub fn tokens_to_document_tree(tokens: &[Token]) -> DocumentTreeDocument {
    usfm_to_document_tree(&crate::tokens::tokens_to_usfm(tokens))
}

pub fn document_tree_to_tokens(
    document: &DocumentTreeDocument,
) -> Result<Vec<Token>, DocumentError> {
    if !document.tokens.is_empty() {
        return Ok(document.tokens.clone());
    }

    Ok(crate::tokens::usfm_to_tokens(&document_tree_to_usfm(
        document,
    )?))
}

pub fn document_tree_to_usfm(document: &DocumentTreeDocument) -> Result<String, DocumentError> {
    let mut serializer = DocumentTreeToUsfmSerializer::default();
    serializer.serialize_nodes(document.content.as_slice())?;
    Ok(serializer.finish())
}

#[derive(Default)]
struct DocumentTreeToUsfmSerializer {
    output: String,
    consume_one_leading_space_on_next_text: bool,
}

impl DocumentTreeToUsfmSerializer {
    fn finish(self) -> String {
        self.output
    }

    fn serialize_nodes(&mut self, nodes: &[DocumentTreeNode]) -> Result<(), DocumentError> {
        for node in nodes {
            self.serialize_node(node)?;
        }
        Ok(())
    }

    fn serialize_node(&mut self, node: &DocumentTreeNode) -> Result<(), DocumentError> {
        match node {
            DocumentTreeNode::Element(DocumentTreeElement::Text { value }) => {
                if self.consume_one_leading_space_on_next_text {
                    self.consume_one_leading_space_on_next_text = false;
                    self.output
                        .push_str(value.strip_prefix(' ').unwrap_or(value));
                } else {
                    self.output.push_str(value);
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::LineBreak { value }) => {
                self.consume_one_leading_space_on_next_text = false;
                self.output.push_str(value);
            }
            DocumentTreeNode::Element(DocumentTreeElement::OptBreak {}) => {
                self.output.push_str("//");
            }
            DocumentTreeNode::Element(DocumentTreeElement::Book {
                marker,
                code,
                content,
                extra,
            }) => {
                self.write_marker_text(extra, marker, true);
                self.output.push_str(code);
                self.consume_one_leading_space_on_next_text = true;
                self.serialize_nodes(content)?;
            }
            DocumentTreeNode::Element(DocumentTreeElement::Chapter {
                marker,
                number,
                extra,
            }) => {
                self.write_marker_text(extra, marker, true);
                self.output.push_str(number);
                if let Some(altnumber) = extra_string(extra, "altnumber") {
                    self.output.push_str(" \\ca ");
                    self.output.push_str(altnumber);
                    self.output.push_str("\\ca*");
                }
                if let Some(pubnumber) = extra_string(extra, "pubnumber") {
                    self.output.push_str(" \\cp ");
                    self.output.push_str(pubnumber);
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Verse {
                marker,
                number,
                extra,
            }) => {
                self.write_marker_text(extra, marker, true);
                self.output.push_str(number);
                if let Some(altnumber) = extra_string(extra, "altnumber") {
                    self.output.push_str("\\va ");
                    self.output.push_str(altnumber);
                    self.output.push_str("\\va*");
                }
                if let Some(pubnumber) = extra_string(extra, "pubnumber") {
                    self.output.push_str("\\vp ");
                    self.output.push_str(pubnumber);
                    self.output.push_str("\\vp*");
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Para {
                marker,
                content,
                extra,
            }) => {
                self.write_marker_text(extra, marker, false);
                self.serialize_nodes(content)?;
            }
            DocumentTreeNode::Element(DocumentTreeElement::Char {
                marker,
                content,
                extra,
            }) => {
                if let Some(marker_text) = extra_string(extra, "markerText") {
                    self.output.push_str(marker_text);
                } else {
                    self.output.push_str(&default_open_marker(marker));
                }
                self.serialize_nodes(content)?;
                self.serialize_attributes(extra, &["closed", "closeSuffix", "closeMarkerText"]);
                if should_emit_char_close(marker, extra) {
                    if marker == "w"
                        && has_serialized_attributes(extra)
                        && !self.output.ends_with([' ', '\t', '\n'])
                    {
                        self.output.push(' ');
                    }
                    if let Some(close_marker_text) = extra_string(extra, "closeMarkerText") {
                        self.output.push_str(close_marker_text);
                    } else {
                        self.output.push('\\');
                        self.output.push_str(marker);
                        self.output.push('*');
                    }
                    if let Some(suffix) = extra_string(extra, "closeSuffix") {
                        self.output.push_str(suffix);
                        self.consume_one_leading_space_on_next_text =
                            suffix.chars().next().is_some_and(char::is_whitespace);
                    }
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Note {
                marker,
                caller,
                content,
                extra,
            }) => {
                if let Some(prefix_gap) = extra_string(extra, "prefixGap") {
                    if !self.output.ends_with([' ', '\t', '\n', '\r']) {
                        self.output.push_str(prefix_gap);
                    }
                }
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push(' ');
                self.output.push_str(caller);
                self.output
                    .push_str(extra_string(extra, "callerSuffix").unwrap_or(" "));
                if let Some(category) = extra_string(extra, "category") {
                    self.output.push_str("\\cat ");
                    self.output.push_str(category);
                    self.output.push_str("\\cat*");
                    if !content.is_empty() {
                        self.output.push(' ');
                    }
                }
                self.serialize_nodes(content)?;
                if extra_bool(extra, "closed").unwrap_or(true) {
                    self.output.push('\\');
                    self.output.push_str(marker);
                    self.output.push('*');
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Figure {
                marker,
                content,
                extra,
            }) => {
                self.output.push_str(&default_open_marker(marker));
                self.serialize_nodes(content)?;
                self.serialize_attributes(extra, &[]);
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('*');
            }
            DocumentTreeNode::Element(DocumentTreeElement::Sidebar {
                marker,
                content,
                extra,
            }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('\n');
                if let Some(category) = extra_string(extra, "category") {
                    self.output.push_str("\\cat ");
                    self.output.push_str(category);
                    self.output.push_str("\\cat*");
                    self.output.push('\n');
                }
                self.serialize_nodes(content)?;
                self.output.push_str("\\esbe");
            }
            DocumentTreeNode::Element(DocumentTreeElement::Periph { content, extra }) => {
                self.output.push_str("\\periph");
                if let Some(alt) = extra_string(extra, "alt") {
                    self.output.push(' ');
                    self.output.push_str(alt);
                }
                self.serialize_attributes(extra, &["alt"]);
                if !content.is_empty() {
                    self.output.push('\n');
                    self.serialize_nodes(content)?;
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Table { content, .. }) => {
                self.serialize_nodes(content)?;
            }
            DocumentTreeNode::Element(DocumentTreeElement::TableRow {
                marker, content, ..
            }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::TableCell {
                marker, content, ..
            }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Ref { content, extra }) => {
                self.output.push_str("\\ref ");
                self.serialize_nodes(content)?;
                self.serialize_attributes(extra, &[]);
                self.output.push_str("\\ref*");
            }
            DocumentTreeNode::Element(DocumentTreeElement::Unknown {
                marker,
                content,
                extra,
            }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.serialize_attributes(extra, &[]);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                    self.output.push('\\');
                    self.output.push_str(marker);
                    self.output.push('*');
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Unmatched {
                marker,
                content,
                extra,
            }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.serialize_attributes(extra, &[]);
                if !content.is_empty() {
                    self.output.push(' ');
                    self.serialize_nodes(content)?;
                }
            }
            DocumentTreeNode::Element(DocumentTreeElement::Milestone { marker, extra }) => {
                self.output.push('\\');
                self.output.push_str(marker);
                self.serialize_attributes(extra, &[]);
                self.output.push_str("\\*");
            }
        }

        Ok(())
    }

    fn serialize_attributes(&mut self, extra: &BTreeMap<String, Value>, ignored: &[&str]) {
        let attrs = extra
            .iter()
            .filter(|(key, value)| {
                !ignored
                    .iter()
                    .any(|ignored_key| ignored_key == &key.as_str())
                    && !matches!(
                        key.as_str(),
                        "markerText"
                            | "sid"
                            | "category"
                            | "closeSuffix"
                            | "closeMarkerText"
                            | "closed"
                            | "altnumber"
                            | "pubnumber"
                            | "prefixGap"
                            | "callerSuffix"
                    )
                    && value.is_string()
            })
            .collect::<Vec<_>>();

        if attrs.is_empty() {
            return;
        }

        self.output.push('|');
        for (index, (key, value)) in attrs.iter().enumerate() {
            if index > 0 {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value.as_str().unwrap_or_default());
            self.output.push('"');
        }
    }

    fn write_marker_text(
        &mut self,
        extra: &BTreeMap<String, Value>,
        marker: &str,
        trailing_space: bool,
    ) {
        if let Some(marker_text) = extra_string(extra, "markerText") {
            self.output.push_str(marker_text);
            return;
        }

        self.output.push('\\');
        self.output.push_str(marker);
        if trailing_space {
            self.output.push(' ');
        }
    }
}

fn default_open_marker(marker: &str) -> String {
    let mut text = String::with_capacity(marker.len() + 2);
    text.push('\\');
    text.push_str(marker);
    text.push(' ');
    text
}

fn extra_string<'a>(extra: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    extra.get(key).and_then(Value::as_str)
}

fn extra_bool(extra: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    extra.get(key).and_then(Value::as_bool)
}

fn should_emit_char_close(marker: &str, extra: &BTreeMap<String, Value>) -> bool {
    if extra_bool(extra, "closed").unwrap_or(true) {
        return true;
    }

    extra_string(extra, "closeMarkerText").is_some()
        || (marker == "w" && has_serialized_attributes(extra))
}

fn has_serialized_attributes(extra: &BTreeMap<String, Value>) -> bool {
    extra.iter().any(|(key, value)| {
        value.is_string()
            && !matches!(
                key.as_str(),
                "markerText"
                    | "sid"
                    | "closeSuffix"
                    | "closeMarkerText"
                    | "closed"
                    | "category"
                    | "altnumber"
                    | "pubnumber"
                    | "prefixGap"
                    | "callerSuffix"
            )
    })
}
