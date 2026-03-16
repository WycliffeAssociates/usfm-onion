// use serde_json::Value;

// use crate::internal::usj::to_document_tree_document;
// use crate::model::document_tree::{DocumentTreeDocument, DocumentTreeElement, DocumentTreeNode};
// use crate::parse::handle::ParseHandle;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HtmlNoteMode {
//     Extracted,
//     Inline,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HtmlCallerStyle {
//     Numeric,
//     AlphaLower,
//     AlphaUpper,
//     RomanLower,
//     RomanUpper,
//     Source,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HtmlCallerScope {
//     DocumentSequential,
//     VerseSequential,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct HtmlOptions {
//     pub wrap_root: bool,
//     pub prefer_native_elements: bool,
//     pub note_mode: HtmlNoteMode,
//     pub caller_style: HtmlCallerStyle,
//     pub caller_scope: HtmlCallerScope,
// }

// impl Default for HtmlOptions {
//     fn default() -> Self {
//         Self {
//             wrap_root: false,
//             prefer_native_elements: true,
//             note_mode: HtmlNoteMode::Extracted,
//             caller_style: HtmlCallerStyle::Numeric,
//             caller_scope: HtmlCallerScope::VerseSequential,
//         }
//     }
// }

// pub fn into_html(handle: &ParseHandle, options: HtmlOptions) -> String {
//     let tree = to_document_tree_document(handle);
//     render_document_tree(&tree, options)
// }

// pub fn usfm_content_to_html(source: &str, options: HtmlOptions) -> String {
//     let handle = crate::parse::parse(source);
//     into_html(&handle, options)
// }

// pub fn render_document_tree(tree: &DocumentTreeDocument, options: HtmlOptions) -> String {
//     let mut renderer = HtmlRenderer::new(options);
//     renderer.render_nodes(tree.content.as_slice(), false);
//     let body = renderer.finish();

//     if options.wrap_root {
//         format!(r#"<div data-usfm-root="true">{body}</div>"#)
//     } else {
//         body
//     }
// }

// struct HtmlRenderer {
//     options: HtmlOptions,
//     output: String,
//     current_verse: Option<String>,
//     note_count_in_verse: usize,
//     document_note_count: usize,
//     footnote_id: usize,
//     crossref_id: usize,
//     footnotes: Vec<String>,
//     crossrefs: Vec<String>,
// }

// impl HtmlRenderer {
//     fn new(options: HtmlOptions) -> Self {
//         Self {
//             options,
//             output: String::new(),
//             current_verse: None,
//             note_count_in_verse: 0,
//             document_note_count: 0,
//             footnote_id: 0,
//             crossref_id: 0,
//             footnotes: Vec::new(),
//             crossrefs: Vec::new(),
//         }
//     }

//     fn finish(mut self) -> String {
//         if !self.footnotes.is_empty() {
//             self.output
//                 .push_str(r#"<section id="linkedFootnotes" data-usfm-notes="footnotes">"#);
//             for note in self.footnotes {
//                 self.output.push_str(note.as_str());
//             }
//             self.output.push_str("</section>");
//         }
//         if !self.crossrefs.is_empty() {
//             self.output
//                 .push_str(r#"<section id="linkedCrossrefs" data-usfm-notes="crossrefs">"#);
//             for note in self.crossrefs {
//                 self.output.push_str(note.as_str());
//             }
//             self.output.push_str("</section>");
//         }
//         self.output
//     }

//     fn render_nodes(&mut self, nodes: &[DocumentTreeNode], in_note_body: bool) {
//         for node in nodes {
//             self.render_node(node, in_note_body);
//         }
//     }

//     fn render_node(&mut self, node: &DocumentTreeNode, in_note_body: bool) {
//         match node {
//             DocumentTreeNode::Element(element) => self.render_element(element, in_note_body),
//         }
//     }

//     fn render_element(&mut self, element: &DocumentTreeElement, in_note_body: bool) {
//         match element {
//             DocumentTreeElement::Text { value } => {
//                 self.output.push_str(escape_html(value).as_str());
//             }
//             DocumentTreeElement::Verse { number, .. } => {
//                 self.current_verse = Some(number.clone());
//                 self.note_count_in_verse = 0;
//                 self.render_wrapped("span", element, &[]);
//             }
//             DocumentTreeElement::Note {
//                 marker,
//                 caller,
//                 content,
//                 extra,
//             } => {
//                 if in_note_body || matches!(self.options.note_mode, HtmlNoteMode::Inline) {
//                     let label = self.note_label(caller.as_str());
//                     self.render_inline_note(
//                         marker.as_str(),
//                         caller.as_str(),
//                         label.as_str(),
//                         content,
//                     );
//                     return;
//                 }

//                 let note_kind = note_kind(marker.as_str());
//                 let label = self.note_label(caller.as_str());
//                 let id_index = match note_kind {
//                     NoteKind::Footnote => {
//                         self.footnote_id += 1;
//                         self.footnote_id
//                     }
//                     NoteKind::Crossref => {
//                         self.crossref_id += 1;
//                         self.crossref_id
//                     }
//                 };

//                 let (call_id, note_id) = match note_kind {
//                     NoteKind::Footnote => (format!("fnref-{id_index}"), format!("fn-{id_index}")),
//                     NoteKind::Crossref => (format!("xrref-{id_index}"), format!("xr-{id_index}")),
//                 };

//                 self.output.push_str("<sup><a");
//                 push_attr(&mut self.output, "href", format!("#{note_id}").as_str());
//                 push_attr(&mut self.output, "id", call_id.as_str());
//                 push_attr(&mut self.output, "data-usfm-note-kind", note_kind.as_str());
//                 push_attr(&mut self.output, "data-usfm-caller", label.as_str());
//                 push_attr(&mut self.output, "data-usfm-source-caller", caller.as_str());
//                 self.output.push('>');
//                 self.output.push_str(escape_html(label.as_str()).as_str());
//                 self.output.push_str("</a></sup>");

//                 let note_html = self.render_extracted_note(
//                     marker.as_str(),
//                     caller.as_str(),
//                     label.as_str(),
//                     call_id.as_str(),
//                     note_id.as_str(),
//                     content.as_slice(),
//                     extra,
//                 );

//                 match note_kind {
//                     NoteKind::Footnote => self.footnotes.push(note_html),
//                     NoteKind::Crossref => self.crossrefs.push(note_html),
//                 }
//             }
//             DocumentTreeElement::OptBreak {} => self.output.push_str("<wbr>"),
//             DocumentTreeElement::LineBreak { .. } => self.output.push_str("<br>"),
//             _ => {
//                 let tag = tag_name(element, self.options.prefer_native_elements);
//                 let children = element_children(element);
//                 self.output.push('<');
//                 self.output.push_str(tag);
//                 push_common_element_attrs(&mut self.output, element);
//                 self.output.push('>');
//                 if let Some(children) = children {
//                     self.render_nodes(children, in_note_body);
//                 }
//                 self.output.push_str("</");
//                 self.output.push_str(tag);
//                 self.output.push('>');
//             }
//         }
//     }

//     fn render_wrapped(
//         &mut self,
//         tag: &str,
//         element: &DocumentTreeElement,
//         extra_attrs: &[(&str, &str)],
//     ) {
//         self.output.push('<');
//         self.output.push_str(tag);
//         push_common_element_attrs(&mut self.output, element);
//         for (key, value) in extra_attrs {
//             push_attr(&mut self.output, key, value);
//         }
//         self.output.push_str("></");
//         self.output.push_str(tag);
//         self.output.push('>');
//     }

//     fn render_inline_note(
//         &mut self,
//         marker: &str,
//         source_caller: &str,
//         label: &str,
//         content: &[DocumentTreeNode],
//     ) {
//         self.output.push_str("<span");
//         push_attr(&mut self.output, "data-usfm-type", "note");
//         push_attr(&mut self.output, "data-usfm-marker", marker);
//         push_attr(&mut self.output, "data-usfm-caller", label);
//         push_attr(&mut self.output, "data-usfm-source-caller", source_caller);
//         self.output.push('>');
//         self.output.push_str("<sup>");
//         self.output.push_str(escape_html(label).as_str());
//         self.output.push_str("</sup>");
//         self.render_nodes(content, true);
//         self.output.push_str("</span>");
//     }

//     fn render_extracted_note(
//         &self,
//         marker: &str,
//         source_caller: &str,
//         label: &str,
//         call_id: &str,
//         note_id: &str,
//         content: &[DocumentTreeNode],
//         extra: &std::collections::BTreeMap<String, Value>,
//     ) -> String {
//         let mut nested = HtmlRenderer::new(self.options);
//         nested.current_verse = self.current_verse.clone();
//         nested.render_nodes(content, true);

//         let mut out = String::from("<aside");
//         push_attr(&mut out, "id", note_id);
//         push_attr(&mut out, "data-usfm-type", "note");
//         push_attr(&mut out, "data-usfm-marker", marker);
//         push_attr(&mut out, "data-usfm-caller", label);
//         push_attr(&mut out, "data-usfm-source-caller", source_caller);
//         push_extra_attrs(&mut out, extra);
//         out.push('>');
//         out.push_str("<a");
//         push_attr(&mut out, "href", format!("#{call_id}").as_str());
//         out.push('>');
//         out.push_str(escape_html(label).as_str());
//         out.push_str("</a>");
//         out.push_str(nested.output.as_str());
//         out.push_str("</aside>");
//         out
//     }

//     fn note_label(&mut self, source_caller: &str) -> String {
//         if matches!(self.options.caller_style, HtmlCallerStyle::Source) && !source_caller.is_empty()
//         {
//             return source_caller.to_string();
//         }

//         match self.options.caller_scope {
//             HtmlCallerScope::DocumentSequential => {
//                 self.document_note_count += 1;
//                 format_ordinal(self.document_note_count, self.options.caller_style)
//             }
//             HtmlCallerScope::VerseSequential => {
//                 if let Some(verse) = self.current_verse.as_deref() {
//                     self.note_count_in_verse += 1;
//                     format!(
//                         "{}.{}",
//                         verse,
//                         format_ordinal(self.note_count_in_verse, self.options.caller_style)
//                     )
//                 } else {
//                     self.document_note_count += 1;
//                     format_ordinal(self.document_note_count, self.options.caller_style)
//                 }
//             }
//         }
//     }
// }

// #[derive(Clone, Copy)]
// enum NoteKind {
//     Footnote,
//     Crossref,
// }

// impl NoteKind {
//     fn as_str(self) -> &'static str {
//         match self {
//             Self::Footnote => "footnote",
//             Self::Crossref => "crossref",
//         }
//     }
// }

// fn note_kind(marker: &str) -> NoteKind {
//     if marker.starts_with('x') {
//         NoteKind::Crossref
//     } else {
//         NoteKind::Footnote
//     }
// }

// fn tag_name(element: &DocumentTreeElement, prefer_native_elements: bool) -> &'static str {
//     match element {
//         DocumentTreeElement::Text { .. } => "span",
//         DocumentTreeElement::Book { .. } if prefer_native_elements => "section",
//         DocumentTreeElement::Figure { .. } if prefer_native_elements => "figure",
//         DocumentTreeElement::Table { .. } => "table",
//         DocumentTreeElement::TableRow { .. } => "tr",
//         DocumentTreeElement::TableCell { .. } => "td",
//         DocumentTreeElement::Book { .. }
//         | DocumentTreeElement::Para { .. }
//         | DocumentTreeElement::Sidebar { .. }
//         | DocumentTreeElement::Periph { .. }
//         | DocumentTreeElement::Unknown { .. }
//         | DocumentTreeElement::Unmatched { .. }
//         | DocumentTreeElement::Figure { .. } => "div",
//         DocumentTreeElement::Char { .. }
//         | DocumentTreeElement::Ref { .. }
//         | DocumentTreeElement::Chapter { .. }
//         | DocumentTreeElement::Verse { .. }
//         | DocumentTreeElement::Milestone { .. }
//         | DocumentTreeElement::Note { .. } => "span",
//         DocumentTreeElement::OptBreak {} => "wbr",
//         DocumentTreeElement::LineBreak { .. } => "br",
//     }
// }

// fn element_children(element: &DocumentTreeElement) -> Option<&[DocumentTreeNode]> {
//     match element {
//         DocumentTreeElement::Book { content, .. }
//         | DocumentTreeElement::Para { content, .. }
//         | DocumentTreeElement::Char { content, .. }
//         | DocumentTreeElement::Note { content, .. }
//         | DocumentTreeElement::Figure { content, .. }
//         | DocumentTreeElement::Sidebar { content, .. }
//         | DocumentTreeElement::Periph { content, .. }
//         | DocumentTreeElement::Table { content, .. }
//         | DocumentTreeElement::TableRow { content, .. }
//         | DocumentTreeElement::TableCell { content, .. }
//         | DocumentTreeElement::Ref { content, .. }
//         | DocumentTreeElement::Unknown { content, .. }
//         | DocumentTreeElement::Unmatched { content, .. } => Some(content.as_slice()),
//         DocumentTreeElement::Text { .. }
//         | DocumentTreeElement::Chapter { .. }
//         | DocumentTreeElement::Verse { .. }
//         | DocumentTreeElement::Milestone { .. }
//         | DocumentTreeElement::OptBreak {}
//         | DocumentTreeElement::LineBreak { .. } => None,
//     }
// }

// fn push_common_element_attrs(out: &mut String, element: &DocumentTreeElement) {
//     match element {
//         DocumentTreeElement::Text { .. } => {}
//         DocumentTreeElement::Book {
//             marker,
//             code,
//             extra,
//             ..
//         } => {
//             push_attr(out, "data-usfm-type", "book");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_attr(out, "data-usfm-code", code.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Chapter {
//             marker,
//             number,
//             extra,
//         } => {
//             push_attr(out, "data-usfm-type", "chapter");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_attr(out, "data-usfm-number", number.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Verse {
//             marker,
//             number,
//             extra,
//         } => {
//             push_attr(out, "data-usfm-type", "verse");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_attr(out, "data-usfm-number", number.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Para { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "para");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Char { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "char");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Note {
//             marker,
//             caller,
//             extra,
//             ..
//         } => {
//             push_attr(out, "data-usfm-type", "note");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_attr(out, "data-usfm-source-caller", caller.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Milestone { marker, extra } => {
//             push_attr(out, "data-usfm-type", "ms");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Figure { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "figure");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Sidebar { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "sidebar");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Periph { extra, .. } => {
//             push_attr(out, "data-usfm-type", "periph");
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Table { extra, .. } => {
//             push_attr(out, "data-usfm-type", "table");
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::TableRow { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "table:row");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::TableCell {
//             marker,
//             align,
//             extra,
//             ..
//         } => {
//             push_attr(out, "data-usfm-type", "table:cell");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_attr(out, "data-usfm-align", align.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Ref { extra, .. } => {
//             push_attr(out, "data-usfm-type", "ref");
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Unknown { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "unknown");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::Unmatched { marker, extra, .. } => {
//             push_attr(out, "data-usfm-type", "unmatched");
//             push_attr(out, "data-usfm-marker", marker.as_str());
//             push_extra_attrs(out, extra);
//         }
//         DocumentTreeElement::OptBreak {} => {
//             push_attr(out, "data-usfm-type", "optbreak");
//         }
//         DocumentTreeElement::LineBreak { .. } => {
//             push_attr(out, "data-usfm-type", "linebreak");
//         }
//     }
// }

// fn push_extra_attrs(out: &mut String, extra: &std::collections::BTreeMap<String, Value>) {
//     for (key, value) in extra {
//         let attr_name = format!("data-usfm-{}", kebab_case(key));
//         let attr_value = match value {
//             Value::Null => String::new(),
//             Value::Bool(value) => value.to_string(),
//             Value::Number(value) => value.to_string(),
//             Value::String(value) => value.clone(),
//             Value::Array(_) | Value::Object(_) => {
//                 serde_json::to_string(value).expect("serde_json::Value should serialize")
//             }
//         };
//         push_attr(out, attr_name.as_str(), attr_value.as_str());
//     }
// }

// fn push_attr(out: &mut String, key: &str, value: &str) {
//     out.push(' ');
//     out.push_str(key);
//     out.push_str("=\"");
//     out.push_str(escape_html(value).as_str());
//     out.push('"');
// }

// fn escape_html(value: &str) -> String {
//     let mut out = String::with_capacity(value.len());
//     for ch in value.chars() {
//         match ch {
//             '&' => out.push_str("&amp;"),
//             '<' => out.push_str("&lt;"),
//             '>' => out.push_str("&gt;"),
//             '"' => out.push_str("&quot;"),
//             '\'' => out.push_str("&#39;"),
//             _ => out.push(ch),
//         }
//     }
//     out
// }

// fn kebab_case(value: &str) -> String {
//     let mut out = String::new();
//     for (index, ch) in value.chars().enumerate() {
//         if ch.is_ascii_uppercase() {
//             if index > 0 {
//                 out.push('-');
//             }
//             out.push(ch.to_ascii_lowercase());
//         } else if ch == '_' || ch == ' ' {
//             out.push('-');
//         } else {
//             out.push(ch);
//         }
//     }
//     out
// }

// fn format_ordinal(index: usize, style: HtmlCallerStyle) -> String {
//     match style {
//         HtmlCallerStyle::Numeric => index.to_string(),
//         HtmlCallerStyle::AlphaLower => alpha_label(index, false),
//         HtmlCallerStyle::AlphaUpper => alpha_label(index, true),
//         HtmlCallerStyle::RomanLower => roman_label(index, false),
//         HtmlCallerStyle::RomanUpper => roman_label(index, true),
//         HtmlCallerStyle::Source => index.to_string(),
//     }
// }

// fn alpha_label(mut index: usize, uppercase: bool) -> String {
//     let mut out = String::new();
//     while index > 0 {
//         let rem = (index - 1) % 26;
//         let base = if uppercase { b'A' } else { b'a' };
//         out.insert(0, (base + rem as u8) as char);
//         index = (index - 1) / 26;
//     }
//     out
// }

// fn roman_label(mut index: usize, uppercase: bool) -> String {
//     let numerals = [
//         (1000, "M"),
//         (900, "CM"),
//         (500, "D"),
//         (400, "CD"),
//         (100, "C"),
//         (90, "XC"),
//         (50, "L"),
//         (40, "XL"),
//         (10, "X"),
//         (9, "IX"),
//         (5, "V"),
//         (4, "IV"),
//         (1, "I"),
//     ];
//     let mut out = String::new();
//     for (value, numeral) in numerals {
//         while index >= value {
//             out.push_str(numeral);
//             index -= value;
//         }
//     }
//     if uppercase {
//         out
//     } else {
//         out.to_ascii_lowercase()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn default_html_extracts_footnotes_with_verse_scoped_callers() {
//         let html = usfm_content_to_html(
//             "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note one\\f* more\\f + \\ft note two\\f*\n",
//             HtmlOptions::default(),
//         );

//         assert!(html.contains(r#"data-usfm-caller="1.1""#));
//         assert!(html.contains(r#"data-usfm-caller="1.2""#));
//         assert!(html.contains(r#"id="linkedFootnotes""#));
//         assert!(
//             html.contains(r#"data-usfm-source-caller="+" "#)
//                 || html.contains(r#"data-usfm-source-caller="+" "#.trim())
//         );
//     }

//     #[test]
//     fn crossrefs_are_extracted_into_separate_group() {
//         let html = usfm_content_to_html(
//             "\\c 1\n\\p\n\\v 1 Text\\x - \\xo 1.1 \\xt cross ref\\x*\n",
//             HtmlOptions::default(),
//         );

//         assert!(html.contains(r#"id="linkedCrossrefs""#));
//         assert!(html.contains(r#"data-usfm-note-kind="crossref""#));
//     }

//     #[test]
//     fn preverse_notes_fall_back_to_document_sequential_labels() {
//         let html =
//             usfm_content_to_html("\\s1 Heading\\f + \\ft note\\f*\n", HtmlOptions::default());
//         assert!(html.contains(r#"data-usfm-caller="1""#));
//     }
// }
