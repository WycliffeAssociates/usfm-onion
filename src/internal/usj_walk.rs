// use crate::internal::tree_walk::TreeWalkControl;
// use crate::model::usj::{UsjDocument, UsjNode};

// pub struct UsjVisit<'a> {
//     pub node: &'a UsjNode,
//     pub depth: usize,
//     pub index_path: &'a [usize],
//     pub parent_type: Option<&'a str>,
// }

// pub fn walk_usj_document_depth_first(
//     document: &UsjDocument,
//     visitor: impl for<'a> FnMut(UsjVisit<'a>) -> TreeWalkControl,
// ) {
//     walk_usj_node_depth_first(document.content.as_slice(), visitor);
// }

// pub fn walk_usj_node_depth_first(
//     content: &[UsjNode],
//     mut visitor: impl for<'a> FnMut(UsjVisit<'a>) -> TreeWalkControl,
// ) {
//     let mut path = Vec::new();
//     walk_usj_nodes(content, 0, None, &mut path, &mut visitor);
// }

// fn walk_usj_nodes(
//     nodes: &[UsjNode],
//     depth: usize,
//     parent_type: Option<&str>,
//     path: &mut Vec<usize>,
//     visitor: &mut impl for<'a> FnMut(UsjVisit<'a>) -> TreeWalkControl,
// ) -> bool {
//     for (index, node) in nodes.iter().enumerate() {
//         path.push(index);
//         let control = visitor(UsjVisit {
//             node,
//             depth,
//             index_path: path.as_slice(),
//             parent_type,
//         });
//         match control {
//             TreeWalkControl::Continue => {
//                 if let Some(children) = node.children() {
//                     let next_parent = match node {
//                         UsjNode::Element(element) => Some(element.element_type()),
//                         UsjNode::Text(_) => None,
//                     };
//                     if !walk_usj_nodes(children, depth + 1, next_parent, path, visitor) {
//                         path.pop();
//                         return false;
//                     }
//                 }
//             }
//             TreeWalkControl::SkipChildren => {}
//             TreeWalkControl::Stop => {
//                 path.pop();
//                 return false;
//             }
//         }
//         path.pop();
//     }
//     true
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::internal::usj::to_usj_document;
//     use crate::model::UsjElement;
//     use crate::parse::parse;

//     #[test]
//     fn walks_usj_in_preorder() {
//         let handle = parse("\\c 1\n\\p\n\\v 1 Text\\f + \\ft note\\f*\n");
//         let document = to_usj_document(&handle);
//         let mut seen = Vec::new();

//         walk_usj_document_depth_first(&document, |visit| {
//             seen.push((
//                 visit.node.node_kind().to_string(),
//                 visit.depth,
//                 visit.index_path.to_vec(),
//             ));
//             TreeWalkControl::Continue
//         });

//         assert!(seen.iter().any(|(kind, _, _)| kind == "para"));
//         assert!(seen.iter().any(|(kind, _, _)| kind == "verse"));
//         assert!(seen.iter().any(|(kind, _, _)| kind == "note"));
//         assert!(seen.iter().any(|(kind, _, _)| kind == "text"));
//         assert!(seen.contains(&("note".to_string(), 1, vec![1, 2])));
//     }

//     #[test]
//     fn skip_children_skips_only_current_subtree() {
//         let handle = parse("\\c 1\n\\p\n\\v 1 Text\\f + \\ft note\\f* tail\n");
//         let document = to_usj_document(&handle);
//         let mut seen = Vec::new();

//         walk_usj_document_depth_first(&document, |visit| {
//             seen.push(visit.node.node_kind().to_string());
//             if matches!(visit.node, UsjNode::Element(UsjElement::Note { .. })) {
//                 TreeWalkControl::SkipChildren
//             } else {
//                 TreeWalkControl::Continue
//             }
//         });

//         assert!(seen.iter().any(|kind| kind == "note"));
//         assert!(!seen.iter().any(|kind| kind == "char"));
//     }

//     #[test]
//     fn stop_halts_walk_immediately() {
//         let handle = parse("\\c 1\n\\p\n\\v 1 Text\n\\p\nmore\n");
//         let document = to_usj_document(&handle);
//         let mut count = 0usize;

//         walk_usj_document_depth_first(&document, |_| {
//             count += 1;
//             TreeWalkControl::Stop
//         });

//         assert_eq!(count, 1);
//     }
// }
