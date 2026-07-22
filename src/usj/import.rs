use std::collections::BTreeMap;

use crate::marker_defs::{marker_default_attribute, marker_is_note_sub};

use super::{UsjElement, UsjError, UsjNode};

#[derive(Default)]
pub(super) struct UsjSerializer {
    output: String,
    at_line_start: bool,
    note_depth: usize,
    char_depth: usize,
}

impl UsjSerializer {
    pub(super) fn finish(mut self) -> String {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output
    }

    pub(super) fn serialize_nodes(&mut self, nodes: &[UsjNode]) -> Result<(), UsjError> {
        for (index, node) in nodes.iter().enumerate() {
            let next = nodes.get(index + 1);
            self.serialize_node(node, next)?;
        }
        Ok(())
    }

    fn serialize_node(&mut self, node: &UsjNode, next: Option<&UsjNode>) -> Result<(), UsjError> {
        match node {
            UsjNode::Text(text) => {
                let mut text = text.as_str();
                if self.output.ends_with(' ') && text.starts_with(' ') {
                    text = &text[1..];
                }
                if should_trim_trailing_text_before(next) {
                    let trimmed = text.trim_end_matches(' ');
                    self.output.push_str(trimmed);
                    self.at_line_start = trimmed.ends_with('\n');
                } else {
                    self.output.push_str(text);
                    self.at_line_start = text.ends_with('\n');
                }
                Ok(())
            }
            UsjNode::Element(element) => self.serialize_element(element, next),
        }
    }

    fn serialize_element(
        &mut self,
        element: &UsjElement,
        next: Option<&UsjNode>,
    ) -> Result<(), UsjError> {
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
                    self.ensure_newline();
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
                if next.is_none() {
                    self.at_line_start = false;
                } else if matches!(
                    next,
                    Some(UsjNode::Element(UsjElement::Milestone { marker, extra }))
                        if uses_spaced_block_milestone_layout(marker, extra)
                ) {
                    self.output.push('\n');
                    self.at_line_start = true;
                } else {
                    self.output.push(' ');
                    self.at_line_start = false;
                }
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
                    } else {
                        self.output.push('\n');
                        self.at_line_start = true;
                    }
                    self.serialize_nodes(content)?;
                }
            }
            UsjElement::Char {
                marker,
                content,
                extra,
            } => {
                if marker_is_note_sub(marker) && self.note_depth > 0 && self.char_depth == 0 {
                    self.output.push('\\');
                    self.output.push_str(marker);
                    self.output.push(' ');
                    self.at_line_start = false;
                    self.char_depth += 1;
                    self.serialize_nodes(content)?;
                    self.char_depth -= 1;
                    self.write_attributes_for_marker(marker, extra);
                    if next.is_some()
                        && !self.output.ends_with(' ')
                        && !self.output.ends_with('\n')
                        && !self.output.ends_with('*')
                    {
                        self.output.push(' ');
                    }
                    return Ok(());
                }
                if marker == "fig" && extra.is_empty() {
                    let emitted_marker = emitted_char_marker(marker);
                    self.output.push('\\');
                    self.output.push_str(&emitted_marker);
                    self.output.push(' ');
                    self.at_line_start = false;
                    self.serialize_nodes(content)?;
                    return Ok(());
                }
                let emitted_marker = emitted_char_marker(marker);
                if marker == "w" {
                    self.ensure_space();
                }
                if marker == "fv" && self.note_depth > 0 && self.char_depth > 0 {
                    self.ensure_newline();
                }
                self.output.push('\\');
                self.output.push_str(&emitted_marker);
                self.output.push(' ');
                self.at_line_start = false;
                self.char_depth += 1;
                self.serialize_nodes(content)?;
                self.char_depth -= 1;
                if marker == "k"
                    && matches!(next, Some(UsjNode::Text(text)) if text.starts_with(' '))
                    && !self.output.ends_with(' ')
                {
                    self.output.push(' ');
                }
                self.write_attributes_for_marker(marker, extra);
                self.output.push('\\');
                self.output.push_str(&emitted_marker);
                self.output.push('*');
            }
            UsjElement::Ref { content, extra } => {
                self.output.push_str("\\ref ");
                self.at_line_start = false;
                self.serialize_nodes(content)?;
                self.write_attributes_for_marker("ref", extra);
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
                self.write_attributes_for_marker(marker, extra);
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
                self.note_depth += 1;
                self.serialize_nodes(content)?;
                self.note_depth -= 1;
                self.write_attributes(extra);
                self.output.push('\\');
                self.output.push_str(marker);
                self.output.push('*');
            }
            UsjElement::Milestone { marker, extra } => {
                if uses_spaced_block_milestone_layout(marker, extra) {
                    self.ensure_newline();
                    self.output.push('\\');
                    self.output.push_str(marker);
                    if !extra.is_empty() {
                        self.output.push(' ');
                        self.write_attributes(extra);
                        self.output.push(' ');
                    }
                    self.output.push_str("\\*");
                    self.at_line_start = false;
                    return Ok(());
                }
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

    fn write_attributes_for_marker(&mut self, marker: &str, extra: &BTreeMap<String, String>) {
        if extra.is_empty() {
            return;
        }

        if marker == "fig" {
            self.write_figure_attributes(extra);
            return;
        }

        self.output.push('|');
        let default_key = marker_default_attribute(marker);
        let mut first = true;

        if let Some(default_key) = default_key
            && let Some(value) = extra.get(default_key)
        {
            self.output.push_str(value);
            first = false;
        }

        for (key, value) in extra {
            if Some(key.as_str()) == default_key {
                continue;
            }
            if !first {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
            first = false;
        }
    }

    fn write_figure_attributes(&mut self, extra: &BTreeMap<String, String>) {
        self.output.push('|');
        let ordered = ["alt", "src", "file", "size", "loc", "copy", "ref", "rotate"];
        let mut first = true;

        for key in ordered {
            let value = match key {
                "src" => extra.get("src").or_else(|| extra.get("file")),
                "file" => None,
                _ => extra.get(key),
            };
            let Some(value) = value else {
                continue;
            };
            if !first {
                self.output.push(' ');
            }
            let out_key = if key == "file" || key == "src" {
                "src"
            } else {
                key
            };
            self.output.push_str(out_key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
            first = false;
        }

        for (key, value) in extra {
            if matches!(
                key.as_str(),
                "alt" | "src" | "file" | "size" | "loc" | "copy" | "ref" | "rotate"
            ) {
                continue;
            }
            if !first {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
            first = false;
        }
    }

    fn write_attributes(&mut self, extra: &BTreeMap<String, String>) {
        if extra.is_empty() {
            return;
        }
        self.output.push('|');
        let mut first = true;
        for (key, value) in extra {
            if !first {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
            first = false;
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

fn uses_spaced_block_milestone_layout(marker: &str, extra: &BTreeMap<String, String>) -> bool {
    matches!(marker, "qt-s" | "qt-e") && (extra.contains_key("sid") || extra.contains_key("eid"))
}

fn emitted_char_marker(marker: &str) -> String {
    // USFM 3.1 allows nested character markup without the legacy '+' prefix.
    // We serialize using the canonical 3.1 form rather than re-inferring legacy syntax.
    marker.to_string()
}

fn should_trim_trailing_text_before(next: Option<&UsjNode>) -> bool {
    matches!(next, Some(UsjNode::Element(UsjElement::Verse { .. })))
        || matches!(
            next,
            Some(UsjNode::Element(UsjElement::Milestone { marker, extra }))
                if uses_spaced_block_milestone_layout(marker, extra)
        )
        || matches!(
            next,
            Some(UsjNode::Element(UsjElement::Char { marker, .. })) if marker == "fv"
        )
}
