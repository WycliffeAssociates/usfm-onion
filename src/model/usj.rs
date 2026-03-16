// use std::collections::BTreeMap;

// use serde::{Deserialize, Serialize};
// use serde_json::Value;

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct UsjDocument {
//     #[serde(rename = "type")]
//     pub doc_type: String,
//     pub version: String,
//     pub content: Vec<UsjNode>,
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum UsjNode {
//     Text(String),
//     Element(UsjElement),
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(tag = "type")]
// pub enum UsjElement {
//     #[serde(rename = "book")]
//     Book {
//         marker: String,
//         code: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "chapter")]
//     Chapter {
//         marker: String,
//         number: String,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "verse")]
//     Verse {
//         marker: String,
//         number: String,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "para")]
//     Para {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "char")]
//     Char {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "note")]
//     Note {
//         marker: String,
//         caller: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "ms")]
//     Milestone {
//         marker: String,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "figure")]
//     Figure {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "sidebar")]
//     Sidebar {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "periph")]
//     Periph {
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "table")]
//     Table {
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "table:row")]
//     TableRow {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "table:cell")]
//     TableCell {
//         marker: String,
//         align: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "ref")]
//     Ref {
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "unknown")]
//     Unknown {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "unmatched")]
//     Unmatched {
//         marker: String,
//         #[serde(default, skip_serializing_if = "Vec::is_empty")]
//         content: Vec<UsjNode>,
//         #[serde(flatten)]
//         extra: BTreeMap<String, Value>,
//     },
//     #[serde(rename = "optbreak")]
//     OptBreak {},
// }

// impl UsjNode {
//     pub fn node_kind(&self) -> &'static str {
//         match self {
//             Self::Text(_) => "text",
//             Self::Element(element) => element.element_type(),
//         }
//     }

//     pub fn children(&self) -> Option<&[UsjNode]> {
//         match self {
//             Self::Text(_) => None,
//             Self::Element(element) => element.children(),
//         }
//     }
// }

// impl UsjElement {
//     pub fn element_type(&self) -> &'static str {
//         match self {
//             Self::Book { .. } => "book",
//             Self::Chapter { .. } => "chapter",
//             Self::Verse { .. } => "verse",
//             Self::Para { .. } => "para",
//             Self::Char { .. } => "char",
//             Self::Note { .. } => "note",
//             Self::Milestone { .. } => "ms",
//             Self::Figure { .. } => "figure",
//             Self::Sidebar { .. } => "sidebar",
//             Self::Periph { .. } => "periph",
//             Self::Table { .. } => "table",
//             Self::TableRow { .. } => "table:row",
//             Self::TableCell { .. } => "table:cell",
//             Self::Ref { .. } => "ref",
//             Self::Unknown { .. } => "unknown",
//             Self::Unmatched { .. } => "unmatched",
//             Self::OptBreak {} => "optbreak",
//         }
//     }

//     pub fn children(&self) -> Option<&[UsjNode]> {
//         match self {
//             Self::Book { content, .. }
//             | Self::Para { content, .. }
//             | Self::Char { content, .. }
//             | Self::Note { content, .. }
//             | Self::Figure { content, .. }
//             | Self::Sidebar { content, .. }
//             | Self::Periph { content, .. }
//             | Self::Table { content, .. }
//             | Self::TableRow { content, .. }
//             | Self::TableCell { content, .. }
//             | Self::Ref { content, .. }
//             | Self::Unknown { content, .. }
//             | Self::Unmatched { content, .. } => Some(content.as_slice()),
//             Self::Chapter { .. }
//             | Self::Verse { .. }
//             | Self::Milestone { .. }
//             | Self::OptBreak {} => None,
//         }
//     }
// }
