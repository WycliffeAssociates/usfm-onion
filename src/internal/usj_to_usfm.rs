use serde_json::{Map, Value};

use crate::model::usj::UsjDocument;

pub fn from_usj_value(value: &Value) -> Result<String, UsjToUsfmError> {
    let root = value
        .as_object()
        .ok_or(UsjToUsfmError::ExpectedObject("root"))?;
    let doc_type = get_required_string(root, "type", "root")?;
    if doc_type != "USJ" {
        return Err(UsjToUsfmError::UnexpectedValue {
            context: "root.type",
            expected: "USJ",
            actual: doc_type.to_string(),
        });
    }

    let content = get_required_array(root, "content", "root")?;
    let mut serializer = UsjToUsfmSerializer::default();
    serializer.serialize_nodes(content)?;
    Ok(serializer.finish())
}

pub fn from_usj_document(document: &UsjDocument) -> Result<String, UsjToUsfmError> {
    let value = serde_json::to_value(document).expect("typed USJ should serialize to JSON");
    from_usj_value(&value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsjToUsfmError {
    ExpectedObject(&'static str),
    ExpectedArray(&'static str),
    ExpectedString(&'static str),
    MissingField {
        context: &'static str,
        field: &'static str,
    },
    UnexpectedValue {
        context: &'static str,
        expected: &'static str,
        actual: String,
    },
    UnknownNodeType(String),
}

impl std::fmt::Display for UsjToUsfmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedObject(context) => write!(f, "expected object for {context}"),
            Self::ExpectedArray(context) => write!(f, "expected array for {context}"),
            Self::ExpectedString(context) => write!(f, "expected string for {context}"),
            Self::MissingField { context, field } => {
                write!(f, "missing field '{field}' in {context}")
            }
            Self::UnexpectedValue {
                context,
                expected,
                actual,
            } => write!(
                f,
                "unexpected value for {context}: expected {expected}, got {actual}"
            ),
            Self::UnknownNodeType(node_type) => write!(f, "unknown USJ node type '{node_type}'"),
        }
    }
}

impl std::error::Error for UsjToUsfmError {}

#[derive(Default)]
struct UsjToUsfmSerializer {
    output: String,
    at_line_start: bool,
}

impl UsjToUsfmSerializer {
    fn finish(mut self) -> String {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output
    }

    fn serialize_nodes(&mut self, nodes: &[Value]) -> Result<(), UsjToUsfmError> {
        for node in nodes {
            self.serialize_node(node)?;
        }
        Ok(())
    }

    fn serialize_node(&mut self, node: &Value) -> Result<(), UsjToUsfmError> {
        match node {
            Value::String(text) => {
                self.output.push_str(text);
                self.at_line_start = text.ends_with('\n');
                Ok(())
            }
            Value::Object(map) => {
                let node_type = get_required_string(map, "type", "node")?;
                match node_type {
                    "book" => self.serialize_book(map),
                    "chapter" => self.serialize_chapter(map),
                    "verse" => self.serialize_verse(map),
                    "para" => self.serialize_para(map),
                    "char" => self.serialize_char(map),
                    "ref" => self.serialize_ref(map),
                    "note" => self.serialize_note(map),
                    "figure" => self.serialize_figure(map),
                    "sidebar" => self.serialize_sidebar(map),
                    "periph" => self.serialize_periph(map),
                    "table" => self.serialize_table(map),
                    "table:row" => self.serialize_table_row(map),
                    "table:cell" => self.serialize_table_cell(map),
                    "ms" => self.serialize_milestone(map),
                    "optbreak" => {
                        self.output.push_str("//");
                        self.at_line_start = false;
                        Ok(())
                    }
                    "unknown" | "unmatched" => self.serialize_unknown(map),
                    other => Err(UsjToUsfmError::UnknownNodeType(other.to_string())),
                }
            }
            Value::Array(items) => self.serialize_nodes(items),
            _ => Err(UsjToUsfmError::ExpectedObject("node")),
        }
    }

    fn serialize_book(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let code = get_required_string(map, "code", "book")?;
        self.ensure_newline();
        self.output.push_str("\\id ");
        self.output.push_str(code);
        if let Some(content) = map.get("content") {
            let content = content
                .as_array()
                .ok_or(UsjToUsfmError::ExpectedArray("book.content"))?;
            if !content.is_empty() {
                self.output.push(' ');
                self.at_line_start = false;
                self.serialize_inline_items(content)?;
            }
        }
        self.output.push('\n');
        self.at_line_start = true;
        Ok(())
    }

    fn serialize_chapter(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let number = get_required_string(map, "number", "chapter")?;
        self.ensure_newline();
        self.output.push_str("\\c ");
        self.output.push_str(number);
        if let Some(altnumber) = get_optional_string(map, "altnumber", "chapter")? {
            self.output.push_str(" \\ca ");
            self.output.push_str(altnumber);
            self.output.push_str("\\ca*");
        }
        if let Some(pubnumber) = get_optional_string(map, "pubnumber", "chapter")? {
            self.output.push_str(" \\cp ");
            self.output.push_str(pubnumber);
        }
        self.output.push('\n');
        self.at_line_start = true;
        Ok(())
    }

    fn serialize_verse(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let number = get_required_string(map, "number", "verse")?;
        if !self.at_line_start {
            self.ensure_space();
        }
        self.output.push_str("\\v ");
        self.output.push_str(number);
        self.output.push(' ');
        self.at_line_start = false;
        if let Some(altnumber) = get_optional_string(map, "altnumber", "verse")? {
            self.output.push_str("\\va ");
            self.output.push_str(altnumber);
            self.output.push_str("\\va*");
        }
        if let Some(pubnumber) = get_optional_string(map, "pubnumber", "verse")? {
            self.output.push_str("\\vp ");
            self.output.push_str(pubnumber);
            self.output.push_str("\\vp*");
        }
        Ok(())
    }

    fn serialize_para(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "para")?;
        let content = get_required_array(map, "content", "para").unwrap_or(&[]);
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(marker);
        self.at_line_start = false;
        if !content.is_empty() {
            let first_is_verse = content
                .first()
                .and_then(Value::as_object)
                .and_then(|first| first.get("type"))
                .and_then(Value::as_str)
                .is_some_and(|node_type| node_type == "verse");
            if !first_is_verse {
                self.output.push(' ');
            }
            self.serialize_inline_items(content)?;
        }
        Ok(())
    }

    fn serialize_char(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "char")?;
        let content = get_required_array(map, "content", "char").unwrap_or(&[]);
        let attrs = collect_attributes(map, &["type", "marker", "content"]);
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_inline_items(content)?;
        self.serialize_attributes(attrs.as_slice());
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push('*');
        Ok(())
    }

    fn serialize_ref(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let content = get_required_array(map, "content", "ref").unwrap_or(&[]);
        let attrs = collect_attributes(map, &["type", "content"]);
        self.output.push_str("\\ref ");
        self.at_line_start = false;
        self.serialize_inline_items(content)?;
        self.serialize_attributes(attrs.as_slice());
        self.output.push_str("\\ref*");
        Ok(())
    }

    fn serialize_note(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "note")?;
        let caller = get_required_string(map, "caller", "note")?;
        let content = get_required_array(map, "content", "note").unwrap_or(&[]);
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push(' ');
        self.output.push_str(caller);
        self.output.push(' ');
        self.at_line_start = false;
        if let Some(category) = get_optional_string(map, "category", "note")? {
            self.output.push_str("\\cat ");
            self.output.push_str(category);
            self.output.push_str("\\cat*");
            if !content.is_empty() {
                self.output.push(' ');
            }
        }
        self.serialize_note_items(content)?;
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push('*');
        Ok(())
    }

    fn serialize_figure(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "figure")?;
        let content = get_required_array(map, "content", "figure").unwrap_or(&[]);
        let attrs = collect_attributes(map, &["type", "marker", "content"]);
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_inline_items(content)?;
        self.serialize_attributes(attrs.as_slice());
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push('*');
        Ok(())
    }

    fn serialize_sidebar(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_optional_string(map, "marker", "sidebar")?.unwrap_or("esb");
        let content = get_required_array(map, "content", "sidebar").unwrap_or(&[]);
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(marker);
        self.output.push('\n');
        self.at_line_start = true;
        if let Some(category) = get_optional_string(map, "category", "sidebar")? {
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
        Ok(())
    }

    fn serialize_periph(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let content = get_required_array(map, "content", "periph").unwrap_or(&[]);
        self.ensure_newline();
        self.output.push_str("\\periph");
        if let Some(alt) = get_optional_string(map, "alt", "periph")? {
            self.output.push(' ');
            self.output.push_str(alt);
        }
        let attrs = collect_attributes(map, &["type", "alt", "content"]);
        self.serialize_attributes(attrs.as_slice());
        self.at_line_start = false;
        if !content.is_empty() {
            self.output.push('\n');
            self.at_line_start = true;
            self.serialize_nodes(content)?;
        }
        Ok(())
    }

    fn serialize_table(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let content = get_required_array(map, "content", "table").unwrap_or(&[]);
        self.serialize_nodes(content)
    }

    fn serialize_table_row(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "table:row")?;
        let content = get_required_array(map, "content", "table:row").unwrap_or(&[]);
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(marker);
        self.at_line_start = false;
        if !content.is_empty() {
            self.output.push(' ');
            self.serialize_inline_items(content)?;
        }
        Ok(())
    }

    fn serialize_table_cell(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "table:cell")?;
        let content = get_required_array(map, "content", "table:cell").unwrap_or(&[]);
        self.output.push('\\');
        self.output.push_str(marker);
        self.at_line_start = false;
        if !content.is_empty() {
            self.output.push(' ');
            self.serialize_inline_items(content)?;
        }
        Ok(())
    }

    fn serialize_milestone(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "ms")?;
        let attrs = collect_attributes(map, &["type", "marker"]);
        self.output.push('\\');
        self.output.push_str(marker);
        self.serialize_attributes(attrs.as_slice());
        self.output.push_str("\\*");
        self.at_line_start = false;
        Ok(())
    }

    fn serialize_unknown(&mut self, map: &Map<String, Value>) -> Result<(), UsjToUsfmError> {
        let marker = get_required_string(map, "marker", "unknown")?;
        let content = get_required_array(map, "content", "unknown").unwrap_or(&[]);
        self.output.push('\\');
        self.output.push_str(marker);
        if !content.is_empty() {
            self.output.push(' ');
            self.at_line_start = false;
            self.serialize_inline_items(content)?;
            self.output.push('\\');
            self.output.push_str(marker);
            self.output.push('*');
        } else {
            self.at_line_start = false;
        }
        Ok(())
    }

    fn serialize_inline_items(&mut self, items: &[Value]) -> Result<(), UsjToUsfmError> {
        for item in items {
            self.serialize_node(item)?;
        }
        Ok(())
    }

    fn serialize_note_items(&mut self, items: &[Value]) -> Result<(), UsjToUsfmError> {
        let mut previous_was_markerish = false;
        for item in items {
            let current_is_markerish = is_note_markerish(item);
            if current_is_markerish && previous_was_markerish && !self.output.ends_with(' ') {
                self.output.push(' ');
            }
            self.serialize_node(item)?;
            previous_was_markerish = current_is_markerish;
        }
        Ok(())
    }

    fn serialize_attributes(&mut self, attributes: &[(String, String)]) {
        if attributes.is_empty() {
            return;
        }
        self.output.push('|');
        for (index, (key, value)) in attributes.iter().enumerate() {
            if index > 0 {
                self.output.push(' ');
            }
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(value);
            self.output.push('"');
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

fn get_required_string<'a>(
    map: &'a Map<String, Value>,
    field: &'static str,
    context: &'static str,
) -> Result<&'a str, UsjToUsfmError> {
    map.get(field)
        .ok_or(UsjToUsfmError::MissingField { context, field })?
        .as_str()
        .ok_or(UsjToUsfmError::ExpectedString(context))
}

fn get_optional_string<'a>(
    map: &'a Map<String, Value>,
    field: &'static str,
    context: &'static str,
) -> Result<Option<&'a str>, UsjToUsfmError> {
    match map.get(field) {
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or(UsjToUsfmError::ExpectedString(context)),
        None => Ok(None),
    }
}

fn get_required_array<'a>(
    map: &'a Map<String, Value>,
    field: &'static str,
    context: &'static str,
) -> Result<&'a [Value], UsjToUsfmError> {
    match map.get(field) {
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or(UsjToUsfmError::ExpectedArray(context)),
        None => Ok(&[]),
    }
}

fn collect_attributes(map: &Map<String, Value>, reserved: &[&str]) -> Vec<(String, String)> {
    map.iter()
        .filter(|(key, _)| !reserved.iter().any(|reserved_key| reserved_key == key))
        .filter_map(|(key, value)| value.as_str().map(|text| (key.clone(), text.to_string())))
        .collect()
}

fn is_note_markerish(value: &Value) -> bool {
    value
        .as_object()
        .and_then(|map| map.get("type"))
        .and_then(Value::as_str)
        .is_some_and(|node_type| matches!(node_type, "char" | "ref"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::usj::to_usj_value;
    use crate::parse::parse;
    use serde_json::json;

    #[test]
    fn writes_simple_document_from_usj() {
        let value = json!({
            "type": "USJ",
            "version": "3.1",
            "content": [
                {"type": "book", "marker": "id", "code": "GEN", "content": ["Genesis"]},
                {"type": "chapter", "marker": "c", "number": "1", "sid": "GEN 1"},
                {"type": "para", "marker": "p", "content": [
                    {"type": "verse", "marker": "v", "number": "1", "sid": "GEN 1:1"},
                    "In the beginning"
                ]}
            ]
        });

        let usfm = from_usj_value(&value).expect("USJ should serialize to USFM");
        assert!(usfm.contains("\\id GEN Genesis"));
        assert!(usfm.contains("\\c 1"));
        assert!(
            usfm.contains("\\p\\v 1 In the beginning")
                || usfm.contains("\\p \\v 1 In the beginning")
        );
    }

    #[test]
    fn roundtrips_own_usj_for_basic_input() {
        let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
        let usj = to_usj_value(&handle);
        let usfm = from_usj_value(&usj).expect("USJ should serialize to USFM");
        let reparsed = parse(&usfm);
        let reparsed_usj = to_usj_value(&reparsed);
        assert_eq!(usj, reparsed_usj);
    }
}
