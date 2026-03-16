// use std::collections::BTreeMap;

// use crate::internal::syntax::{ContainerKind, Node};
// use crate::model::document_tree::{DocumentTreeDocument, DocumentTreeElement, DocumentTreeNode};
// use crate::parse::handle::ParseHandle;

// pub type VrefMap = BTreeMap<String, String>;

// #[derive(Debug, Default)]
// struct VrefState {
//     book: String,
//     chapter: String,
//     current_ref: String,
//     current_text: String,
// }

// pub fn to_vref_map(handle: &ParseHandle) -> VrefMap {
//     let mut map = VrefMap::new();
//     let mut state = VrefState {
//         book: handle.book_code().unwrap_or("").to_string(),
//         ..VrefState::default()
//     };

//     collect_nodes(
//         handle.document().children.as_slice(),
//         handle.source(),
//         &mut state,
//         &mut map,
//     );
//     flush_current(&mut state, &mut map);
//     map
// }

// pub fn to_vref_json_string(handle: &ParseHandle) -> String {
//     let map = to_vref_map(handle);
//     let mut out = String::from("{\n");

//     for (index, (key, value)) in map.iter().enumerate() {
//         if index > 0 {
//             out.push_str(",\n");
//         }
//         out.push_str("  ");
//         push_json_string(&mut out, key);
//         out.push_str(": ");
//         push_json_string(&mut out, value);
//     }

//     out.push_str("\n}");
//     out
// }

// pub fn document_tree_to_vref_map(document: &DocumentTreeDocument) -> VrefMap {
//     let mut map = VrefMap::new();
//     let mut state = VrefState::default();
//     collect_tree_nodes(document.content.as_slice(), &mut state, &mut map);
//     flush_current(&mut state, &mut map);
//     map
// }

// fn collect_nodes(nodes: &[Node], source: &str, state: &mut VrefState, map: &mut VrefMap) {
//     for node in nodes {
//         match node {
//             Node::Container(container) => match container.kind {
//                 ContainerKind::Book => {
//                     collect_nodes(container.children.as_slice(), source, state, map);
//                 }
//                 ContainerKind::Paragraph if is_verse_paragraph(&container.marker) => {
//                     collect_paragraph_children(container.children.as_slice(), source, state, map);
//                 }
//                 ContainerKind::Character | ContainerKind::Unknown => {
//                     if !state.current_ref.is_empty() {
//                         collect_nodes(container.children.as_slice(), source, state, map);
//                     }
//                 }
//                 ContainerKind::Note => {}
//                 _ => {}
//             },
//             Node::Chapter {
//                 number_span: Some(span),
//                 ..
//             } => {
//                 state.chapter = source[span.clone()].trim().to_string();
//             }
//             Node::Chapter { .. } => {}
//             Node::Verse { number_span, .. } => {
//                 let Some(span) = number_span else {
//                     continue;
//                 };
//                 flush_current(state, map);
//                 let verse = source[span.clone()].trim();
//                 if state.book.is_empty() || state.chapter.is_empty() || verse.is_empty() {
//                     state.current_ref.clear();
//                     state.current_text.clear();
//                     continue;
//                 }
//                 state.current_ref = format!("{} {}:{}", state.book, state.chapter, verse);
//                 state.current_text.clear();
//             }
//             Node::Leaf { kind, span } if !state.current_ref.is_empty() => match kind {
//                 crate::internal::syntax::LeafKind::Text
//                 | crate::internal::syntax::LeafKind::Whitespace => {
//                     state.current_text.push_str(&source[span.clone()]);
//                 }
//                 crate::internal::syntax::LeafKind::Newline
//                 | crate::internal::syntax::LeafKind::OptBreak
//                 | crate::internal::syntax::LeafKind::Attributes => {}
//             },
//             Node::Milestone { .. } => {}
//             _ => {}
//         }
//     }
// }

// fn collect_tree_nodes(nodes: &[DocumentTreeNode], state: &mut VrefState, map: &mut VrefMap) {
//     for node in nodes {
//         let DocumentTreeNode::Element(element) = node;
//         match element {
//             DocumentTreeElement::Book { code, content, .. } => {
//                 state.book = code.clone();
//                 collect_tree_nodes(content.as_slice(), state, map);
//             }
//             DocumentTreeElement::Chapter { number, .. } => {
//                 state.chapter = number.trim().to_string();
//             }
//             DocumentTreeElement::Verse { number, .. } => {
//                 flush_current(state, map);
//                 let verse = number.trim();
//                 if state.book.is_empty() || state.chapter.is_empty() || verse.is_empty() {
//                     state.current_ref.clear();
//                     state.current_text.clear();
//                     continue;
//                 }
//                 state.current_ref = format!("{} {}:{}", state.book, state.chapter, verse);
//                 state.current_text.clear();
//             }
//             DocumentTreeElement::Para {
//                 marker, content, ..
//             } if is_verse_paragraph(marker) => {
//                 collect_tree_paragraph_children(content.as_slice(), state, map);
//             }
//             DocumentTreeElement::Char { content, .. }
//             | DocumentTreeElement::Unknown { content, .. }
//             | DocumentTreeElement::Unmatched { content, .. } => {
//                 if !state.current_ref.is_empty() {
//                     collect_tree_nodes(content.as_slice(), state, map);
//                 }
//             }
//             DocumentTreeElement::Text { value } => {
//                 if !state.current_ref.is_empty() {
//                     state.current_text.push_str(value);
//                 }
//             }
//             DocumentTreeElement::OptBreak {}
//             | DocumentTreeElement::LineBreak { .. }
//             | DocumentTreeElement::Note { .. }
//             | DocumentTreeElement::Milestone { .. }
//             | DocumentTreeElement::Figure { .. }
//             | DocumentTreeElement::Sidebar { .. }
//             | DocumentTreeElement::Periph { .. }
//             | DocumentTreeElement::Table { .. }
//             | DocumentTreeElement::TableRow { .. }
//             | DocumentTreeElement::TableCell { .. }
//             | DocumentTreeElement::Ref { .. }
//             | DocumentTreeElement::Para { .. } => {}
//         }
//     }
// }

// fn collect_tree_paragraph_children(
//     nodes: &[DocumentTreeNode],
//     state: &mut VrefState,
//     map: &mut VrefMap,
// ) {
//     let mut seen_meaningful_child = false;

//     for node in nodes {
//         let DocumentTreeNode::Element(element) = node;
//         if !seen_meaningful_child
//             && matches!(
//                 element,
//                 DocumentTreeElement::LineBreak { .. } | DocumentTreeElement::OptBreak {}
//             )
//         {
//             continue;
//         }

//         seen_meaningful_child = true;
//         collect_tree_nodes(std::slice::from_ref(node), state, map);
//     }
// }

// fn collect_paragraph_children(
//     nodes: &[Node],
//     source: &str,
//     state: &mut VrefState,
//     map: &mut VrefMap,
// ) {
//     let mut seen_meaningful_child = false;

//     for node in nodes {
//         if !seen_meaningful_child
//             && matches!(
//                 node,
//                 Node::Leaf {
//                     kind: crate::internal::syntax::LeafKind::Whitespace
//                         | crate::internal::syntax::LeafKind::Newline
//                         | crate::internal::syntax::LeafKind::OptBreak,
//                     ..
//                 }
//             )
//         {
//             continue;
//         }

//         seen_meaningful_child = true;
//         collect_nodes(std::slice::from_ref(node), source, state, map);
//     }
// }

// fn flush_current(state: &mut VrefState, map: &mut VrefMap) {
//     let trimmed = state.current_text.trim();
//     if !state.current_ref.is_empty() && !trimmed.is_empty() {
//         map.insert(state.current_ref.clone(), trimmed.to_string());
//     }
// }

// fn is_verse_paragraph(marker: &str) -> bool {
//     if matches!(
//         marker,
//         "p" | "m"
//             | "po"
//             | "pr"
//             | "cls"
//             | "pmo"
//             | "pm"
//             | "pmc"
//             | "pmr"
//             | "pi"
//             | "pi1"
//             | "pi2"
//             | "pi3"
//             | "mi"
//             | "nb"
//             | "pc"
//             | "ph"
//             | "ph1"
//             | "ph2"
//             | "ph3"
//             | "pb"
//             | "q"
//             | "q1"
//             | "q2"
//             | "q3"
//             | "q4"
//             | "qr"
//             | "qc"
//             | "qa"
//             | "qm"
//             | "qm1"
//             | "qm2"
//             | "qm3"
//             | "qd"
//             | "lh"
//             | "li"
//             | "li1"
//             | "li2"
//             | "li3"
//             | "li4"
//             | "lf"
//             | "lim"
//             | "lim1"
//             | "lim2"
//             | "lim3"
//     ) {
//         return true;
//     }

//     let base = marker.trim_end_matches(|ch: char| ch.is_ascii_digit());
//     !base.is_empty() && base != marker && is_verse_paragraph(base)
// }

// fn push_json_string(out: &mut String, value: &str) {
//     out.push('"');
//     for ch in value.chars() {
//         match ch {
//             '"' => out.push_str("\\\""),
//             '\\' => out.push_str("\\\\"),
//             '\n' => out.push_str("\\n"),
//             '\r' => out.push_str("\\r"),
//             '\t' => out.push_str("\\t"),
//             ch if ch.is_control() => {
//                 let code = ch as u32;
//                 out.push_str("\\u");
//                 out.push(hex_digit((code >> 12) & 0xF));
//                 out.push(hex_digit((code >> 8) & 0xF));
//                 out.push(hex_digit((code >> 4) & 0xF));
//                 out.push(hex_digit(code & 0xF));
//             }
//             _ => out.push(ch),
//         }
//     }
//     out.push('"');
// }

// fn hex_digit(value: u32) -> char {
//     match value {
//         0..=9 => char::from_u32(b'0' as u32 + value).unwrap_or('0'),
//         10..=15 => char::from_u32(b'a' as u32 + (value - 10)).unwrap_or('a'),
//         _ => '0',
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::parse::parse;

//     #[test]
//     fn basic_vref_extracts_plain_verse_text() {
//         let handle = parse(
//             "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n",
//         );
//         let map = to_vref_map(&handle);

//         assert_eq!(
//             map.get("GEN 1:1").map(String::as_str),
//             Some("In the beginning God created the heavens and the earth.")
//         );
//         assert_eq!(
//             map.get("GEN 1:2").map(String::as_str),
//             Some("The earth was without form and void.")
//         );
//     }

//     #[test]
//     fn footnotes_are_stripped() {
//         let handle =
//             parse("\\id GEN\n\\c 1\n\\p\n\\v 1 Text \\f + \\fr 1:1 \\ft note text \\f* rest.");
//         let map = to_vref_map(&handle);
//         let verse = map.get("GEN 1:1").map(String::as_str).unwrap_or("");

//         assert!(verse.contains("Text"));
//         assert!(verse.contains("rest."));
//         assert!(!verse.contains("note text"));
//     }

//     #[test]
//     fn section_headings_are_skipped() {
//         let handle = parse("\\id GEN\n\\c 1\n\\s1 The Creation\n\\p\n\\v 1 In the beginning.");
//         let map = to_vref_map(&handle);

//         assert_eq!(map.len(), 1);
//         assert_eq!(
//             map.get("GEN 1:1").map(String::as_str),
//             Some("In the beginning.")
//         );
//     }

//     #[test]
//     fn verse_spanning_paragraphs_is_concatenated() {
//         let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 First part.\n\\q1 Second part.");
//         let map = to_vref_map(&handle);

//         assert_eq!(
//             map.get("GEN 1:1").map(String::as_str),
//             Some("First part.Second part.")
//         );
//     }

//     #[test]
//     fn root_level_verses_are_collected() {
//         let handle = parse("\\id GEN\n\\c 1\n\\v 1 In the beginning.\n\\v 2 And God said.");
//         let map = to_vref_map(&handle);

//         assert_eq!(
//             map.get("GEN 1:1").map(String::as_str),
//             Some("In the beginning.")
//         );
//         assert_eq!(
//             map.get("GEN 1:2").map(String::as_str),
//             Some("And God said.")
//         );
//     }

//     #[test]
//     fn json_output_contains_refs_and_text() {
//         let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
//         let json = to_vref_json_string(&handle);

//         assert!(json.contains("\"GEN 1:1\""));
//         assert!(json.contains("\"In the beginning.\""));
//     }
// }
