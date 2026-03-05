use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorTreeDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: String,
    pub content: Vec<EditorTreeNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EditorTreeNode {
    Text(String),
    Element(EditorTreeElement),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditorTreeElement {
    #[serde(rename = "book")]
    Book {
        marker: String,
        code: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "chapter")]
    Chapter {
        marker: String,
        number: String,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "verse")]
    Verse {
        marker: String,
        number: String,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "para")]
    Para {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "char")]
    Char {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "note")]
    Note {
        marker: String,
        caller: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "ms")]
    Milestone {
        marker: String,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "figure")]
    Figure {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "sidebar")]
    Sidebar {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "periph")]
    Periph {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table")]
    Table {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table:row")]
    TableRow {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table:cell")]
    TableCell {
        marker: String,
        align: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "ref")]
    Ref {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "unknown")]
    Unknown {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "unmatched")]
    Unmatched {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<EditorTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "optbreak")]
    OptBreak {},
    #[serde(rename = "linebreak")]
    LineBreak {},
}
