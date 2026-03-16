// #![allow(dead_code)]

// use quick_xml::Reader;
// use quick_xml::events::Event;

// use crate::internal::tree_walk::TreeWalkControl;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct XmlDocument {
//     pub root: XmlElement,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct XmlElement {
//     pub name: String,
//     pub attrs: Vec<(String, String)>,
//     pub children: Vec<XmlNode>,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum XmlNode {
//     Element(XmlElement),
//     Text(String),
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum XmlError {
//     Parse(String),
// }

// impl std::fmt::Display for XmlError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Parse(message) => write!(f, "xml parse error: {message}"),
//         }
//     }
// }

// impl std::error::Error for XmlError {}

// pub struct XmlVisit<'a> {
//     pub node: &'a XmlNode,
//     pub depth: usize,
//     pub index_path: &'a [usize],
//     pub parent_name: Option<&'a str>,
// }

// impl XmlElement {
//     pub fn attr(&self, key: &str) -> Option<&str> {
//         self.attrs
//             .iter()
//             .find(|(candidate, _)| candidate == key)
//             .map(|(_, value)| value.as_str())
//     }
// }

// impl XmlNode {
//     pub fn node_kind(&self) -> &'static str {
//         match self {
//             Self::Element(_) => "element",
//             Self::Text(_) => "text",
//         }
//     }

//     pub fn children(&self) -> Option<&[XmlNode]> {
//         match self {
//             Self::Element(element) => Some(element.children.as_slice()),
//             Self::Text(_) => None,
//         }
//     }
// }

// pub fn parse_xml_document(input: &str) -> Result<XmlDocument, XmlError> {
//     Ok(XmlDocument {
//         root: parse_xml_root(input)?,
//     })
// }

// pub fn walk_xml_document_depth_first(
//     document: &XmlDocument,
//     mut visitor: impl for<'a> FnMut(XmlVisit<'a>) -> TreeWalkControl,
// ) {
//     let mut path = Vec::new();
//     let root = XmlNode::Element(document.root.clone());
//     walk_xml_node(&root, 0, None, &mut path, &mut visitor);
// }

// pub fn walk_xml_nodes_depth_first(
//     nodes: &[XmlNode],
//     mut visitor: impl for<'a> FnMut(XmlVisit<'a>) -> TreeWalkControl,
// ) {
//     let mut path = Vec::new();
//     walk_xml_nodes(nodes, 0, None, &mut path, &mut visitor);
// }

// fn walk_xml_nodes(
//     nodes: &[XmlNode],
//     depth: usize,
//     parent_name: Option<&str>,
//     path: &mut Vec<usize>,
//     visitor: &mut impl for<'a> FnMut(XmlVisit<'a>) -> TreeWalkControl,
// ) -> bool {
//     for (index, node) in nodes.iter().enumerate() {
//         path.push(index);
//         if !walk_xml_node(node, depth, parent_name, path, visitor) {
//             path.pop();
//             return false;
//         }
//         path.pop();
//     }
//     true
// }

// fn walk_xml_node(
//     node: &XmlNode,
//     depth: usize,
//     parent_name: Option<&str>,
//     path: &mut Vec<usize>,
//     visitor: &mut impl for<'a> FnMut(XmlVisit<'a>) -> TreeWalkControl,
// ) -> bool {
//     let control = visitor(XmlVisit {
//         node,
//         depth,
//         index_path: path.as_slice(),
//         parent_name,
//     });

//     match control {
//         TreeWalkControl::Continue => {
//             if let Some(children) = node.children() {
//                 let next_parent = match node {
//                     XmlNode::Element(element) => Some(element.name.as_str()),
//                     XmlNode::Text(_) => None,
//                 };
//                 if !walk_xml_nodes(children, depth + 1, next_parent, path, visitor) {
//                     return false;
//                 }
//             }
//         }
//         TreeWalkControl::SkipChildren => {}
//         TreeWalkControl::Stop => return false,
//     }

//     true
// }

// fn parse_xml_root(input: &str) -> Result<XmlElement, XmlError> {
//     let mut reader = Reader::from_str(input);
//     reader.config_mut().trim_text(false);
//     let mut buf = Vec::new();
//     let mut stack: Vec<XmlElement> = Vec::new();
//     let mut root = None;

//     loop {
//         match reader.read_event_into(&mut buf) {
//             Ok(Event::Decl(_) | Event::PI(_) | Event::DocType(_) | Event::Comment(_)) => {}
//             Ok(Event::Start(event)) => {
//                 stack.push(XmlElement {
//                     name: String::from_utf8_lossy(event.name().as_ref()).into_owned(),
//                     attrs: parse_attrs(&event, &reader)?,
//                     children: Vec::new(),
//                 });
//             }
//             Ok(Event::Empty(event)) => {
//                 let elem = XmlElement {
//                     name: String::from_utf8_lossy(event.name().as_ref()).into_owned(),
//                     attrs: parse_attrs(&event, &reader)?,
//                     children: Vec::new(),
//                 };
//                 if let Some(parent) = stack.last_mut() {
//                     parent.children.push(XmlNode::Element(elem));
//                 } else {
//                     root = Some(elem);
//                 }
//             }
//             Ok(Event::Text(event)) => {
//                 let unescaped = event
//                     .unescape()
//                     .map_err(|error| XmlError::Parse(error.to_string()))?;
//                 let text = normalize_xml_text(unescaped.as_ref());
//                 if text.is_empty() {
//                     buf.clear();
//                     continue;
//                 }
//                 if let Some(parent) = stack.last_mut() {
//                     parent.children.push(XmlNode::Text(text));
//                 } else {
//                     return Err(XmlError::Parse(
//                         "text node outside root element".to_string(),
//                     ));
//                 }
//             }
//             Ok(Event::CData(event)) => {
//                 let decoded = event
//                     .decode()
//                     .map_err(|error| XmlError::Parse(error.to_string()))?;
//                 let text = normalize_xml_text(decoded.as_ref());
//                 if text.is_empty() {
//                     buf.clear();
//                     continue;
//                 }
//                 if let Some(parent) = stack.last_mut() {
//                     parent.children.push(XmlNode::Text(text));
//                 } else {
//                     return Err(XmlError::Parse(
//                         "CDATA node outside root element".to_string(),
//                     ));
//                 }
//             }
//             Ok(Event::End(_)) => {
//                 let Some(elem) = stack.pop() else {
//                     return Err(XmlError::Parse("unexpected closing tag".to_string()));
//                 };
//                 if let Some(parent) = stack.last_mut() {
//                     parent.children.push(XmlNode::Element(elem));
//                 } else {
//                     root = Some(elem);
//                 }
//             }
//             Ok(Event::Eof) => break,
//             Err(error) => return Err(XmlError::Parse(error.to_string())),
//         }
//         buf.clear();
//     }

//     if !stack.is_empty() {
//         return Err(XmlError::Parse("unclosed element stack".to_string()));
//     }

//     root.ok_or_else(|| XmlError::Parse("missing root element".to_string()))
// }

// fn normalize_xml_text(text: &str) -> String {
//     if text.contains('\n') || text.contains('\r') {
//         text.lines()
//             .map(str::trim)
//             .filter(|line| !line.is_empty())
//             .collect::<Vec<_>>()
//             .join(" ")
//     } else {
//         text.to_string()
//     }
// }

// fn parse_attrs(
//     event: &quick_xml::events::BytesStart<'_>,
//     reader: &Reader<&[u8]>,
// ) -> Result<Vec<(String, String)>, XmlError> {
//     let mut attrs = Vec::new();
//     for attr in event.attributes() {
//         let attr = attr.map_err(|error| XmlError::Parse(error.to_string()))?;
//         let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
//         let value = attr
//             .decode_and_unescape_value(reader.decoder())
//             .map_err(|error| XmlError::Parse(error.to_string()))?
//             .into_owned();
//         attrs.push((key, value));
//     }
//     Ok(attrs)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parses_generic_xml_tree() {
//         let document = parse_xml_document(r#"<root a="1"><child>text</child><empty /></root>"#)
//             .expect("xml should parse");

//         assert_eq!(document.root.name, "root");
//         assert_eq!(document.root.attr("a"), Some("1"));
//         assert_eq!(document.root.children.len(), 2);
//     }

//     #[test]
//     fn walks_xml_document_in_preorder() {
//         let document = parse_xml_document(r#"<root><child>text</child><sibling /></root>"#)
//             .expect("xml should parse");
//         let mut seen = Vec::new();

//         walk_xml_document_depth_first(&document, |visit| {
//             seen.push((
//                 visit.node.node_kind().to_string(),
//                 visit.depth,
//                 visit.index_path.to_vec(),
//             ));
//             TreeWalkControl::Continue
//         });

//         assert_eq!(seen[0], ("element".to_string(), 0, Vec::<usize>::new()));
//         assert_eq!(seen[1], ("element".to_string(), 1, vec![0]));
//         assert_eq!(seen[2], ("text".to_string(), 2, vec![0, 0]));
//         assert_eq!(seen[3], ("element".to_string(), 1, vec![1]));
//     }

//     #[test]
//     fn stop_halts_xml_walk() {
//         let document = parse_xml_document(r#"<root><a /><b /></root>"#).expect("xml should parse");
//         let mut count = 0usize;

//         walk_xml_document_depth_first(&document, |_| {
//             count += 1;
//             TreeWalkControl::Stop
//         });

//         assert_eq!(count, 1);
//     }
// }
