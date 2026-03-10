use std::collections::BTreeMap;

use crate::model::token::Token;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTreeDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<Token>,
    pub content: Vec<DocumentTreeNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentTreeNode {
    Element(DocumentTreeElement),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentTreeElement {
    #[serde(rename = "text")]
    Text { value: String },
    #[serde(rename = "book")]
    Book {
        marker: String,
        code: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
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
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "char")]
    Char {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "note")]
    Note {
        marker: String,
        caller: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
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
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "sidebar")]
    Sidebar {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "periph")]
    Periph {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table")]
    Table {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table:row")]
    TableRow {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "table:cell")]
    TableCell {
        marker: String,
        align: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "ref")]
    Ref {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "unknown")]
    Unknown {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "unmatched")]
    Unmatched {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<DocumentTreeNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "optbreak")]
    OptBreak {},
    #[serde(rename = "linebreak")]
    LineBreak { value: String },
}

impl Serialize for DocumentTreeNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Element(element) => element.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DocumentTreeNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(text) => Ok(Self::Element(DocumentTreeElement::Text { value: text })),
            other => serde_json::from_value::<DocumentTreeElement>(other)
                .map(Self::Element)
                .map_err(serde::de::Error::custom),
        }
    }
}
