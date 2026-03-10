use crate::internal::xml::{XmlElement, XmlError, XmlNode, parse_xml_document};

pub fn from_usx_string(input: &str) -> Result<String, UsxToUsfmError> {
    let document = parse_xml_document(input)?;
    if document.root.name != "usx" {
        return Err(UsxToUsfmError::UnexpectedElement {
            expected: "usx",
            actual: document.root.name,
        });
    }

    let mut serializer = UsxToUsfmSerializer::default();
    serializer.serialize_children(document.root.children.as_slice())?;
    Ok(serializer.finish())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsxToUsfmError {
    Xml(String),
    UnexpectedElement {
        expected: &'static str,
        actual: String,
    },
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    UnknownElement(String),
}

impl std::fmt::Display for UsxToUsfmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(error) => write!(f, "xml parse error: {error}"),
            Self::UnexpectedElement { expected, actual } => {
                write!(f, "expected <{expected}> root, got <{actual}>")
            }
            Self::MissingAttribute { element, attribute } => {
                write!(f, "missing '{attribute}' attribute on <{element}>")
            }
            Self::UnknownElement(element) => write!(f, "unknown usx element <{element}>"),
        }
    }
}

impl std::error::Error for UsxToUsfmError {}

#[derive(Default)]
struct UsxToUsfmSerializer {
    output: String,
    at_line_start: bool,
}

impl UsxToUsfmSerializer {
    fn finish(mut self) -> String {
        while self.output.ends_with([' ', '\t']) {
            self.output.pop();
        }
        self.output
    }

    fn serialize_children(&mut self, nodes: &[XmlNode]) -> Result<(), UsxToUsfmError> {
        for node in nodes {
            self.serialize_node(node)?;
        }
        Ok(())
    }

    fn serialize_node(&mut self, node: &XmlNode) -> Result<(), UsxToUsfmError> {
        match node {
            XmlNode::Text(text) => {
                self.output.push_str(text);
                self.at_line_start = text.ends_with('\n');
                Ok(())
            }
            XmlNode::Element(element) => self.serialize_element(element),
        }
    }

    fn serialize_element(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        match element.name.as_str() {
            "book" => self.serialize_book(element),
            "chapter" => self.serialize_chapter(element),
            "verse" => self.serialize_verse(element),
            "para" => self.serialize_para(element),
            "char" => self.serialize_char(element),
            "note" => self.serialize_note(element),
            "ref" => self.serialize_ref(element),
            "figure" => self.serialize_figure(element),
            "sidebar" => self.serialize_sidebar(element),
            "table" => self.serialize_table(element),
            "row" => self.serialize_row(element),
            "cell" => self.serialize_cell(element),
            "ms" => self.serialize_ms(element),
            "periph" => self.serialize_periph(element),
            "unmatched" => self.serialize_unmatched(element),
            "optbreak" => {
                self.output.push_str("//");
                self.at_line_start = false;
                Ok(())
            }
            other => Err(UsxToUsfmError::UnknownElement(other.to_string())),
        }
    }

    fn serialize_book(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let code = required_attr(element, "book", "code")?;
        self.ensure_newline();
        self.output.push_str("\\id ");
        self.output.push_str(code);
        let text = text_content(element.children.as_slice());
        if !text.is_empty() {
            self.output.push(' ');
            self.output.push_str(text.as_str());
        }
        self.output.push('\n');
        self.at_line_start = true;
        Ok(())
    }

    fn serialize_chapter(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        if attr(element, "eid").is_some() {
            return Ok(());
        }

        let number = required_attr(element, "chapter", "number")?;
        let style = attr(element, "style").unwrap_or("c");
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.output.push_str(number);
        if let Some(altnumber) = attr(element, "altnumber") {
            self.output.push_str(" \\ca ");
            self.output.push_str(altnumber);
            self.output.push_str("\\ca*");
        }
        if let Some(pubnumber) = attr(element, "pubnumber") {
            self.output.push_str(" \\cp ");
            self.output.push_str(pubnumber);
        }
        self.output.push('\n');
        self.at_line_start = true;
        Ok(())
    }

    fn serialize_verse(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        if attr(element, "eid").is_some() {
            return Ok(());
        }

        let number = required_attr(element, "verse", "number")?;
        let style = attr(element, "style").unwrap_or("v");
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.output.push_str(number);
        self.output.push(' ');
        if let Some(altnumber) = attr(element, "altnumber") {
            self.output.push_str("\\va ");
            self.output.push_str(altnumber);
            self.output.push_str("\\va*");
        }
        if let Some(pubnumber) = attr(element, "pubnumber") {
            self.output.push_str("\\vp ");
            self.output.push_str(pubnumber);
            if attr(element, "closed") != Some("false") {
                self.output.push_str("\\vp*");
            }
            self.output.push(' ');
        }
        self.at_line_start = false;
        Ok(())
    }

    fn serialize_para(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = attr(element, "style").unwrap_or("");
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(style);
        self.at_line_start = false;

        if !element.children.is_empty() {
            let first_is_verse = matches!(element.children.first(), Some(XmlNode::Element(first)) if first.name == "verse" && attr(first, "eid").is_none());
            if !first_is_verse {
                self.output.push(' ');
            } else {
                self.output.push('\n');
                self.at_line_start = true;
            }
            self.serialize_children(element.children.as_slice())?;
        }
        Ok(())
    }

    fn serialize_char(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = required_attr(element, "char", "style")?;
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_children(element.children.as_slice())?;
        self.serialize_attrs(element, &["style", "status", "closed"])?;
        if attr(element, "closed") != Some("false") {
            self.output.push('\\');
            self.output.push_str(style);
            self.output.push('*');
        }
        Ok(())
    }

    fn serialize_note(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = required_attr(element, "note", "style")?;
        let caller = required_attr(element, "note", "caller")?;
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.output.push_str(caller);
        self.output.push(' ');
        self.at_line_start = false;
        if let Some(category) = attr(element, "category") {
            self.output.push_str("\\cat ");
            self.output.push_str(category);
            self.output.push_str("\\cat*");
            if !element.children.is_empty() {
                self.output.push(' ');
            }
        }
        self.serialize_children(element.children.as_slice())?;
        if attr(element, "closed") != Some("false") {
            self.output.push('\\');
            self.output.push_str(style);
            self.output.push('*');
        }
        Ok(())
    }

    fn serialize_ref(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        self.output.push_str("\\ref ");
        self.at_line_start = false;
        self.serialize_children(element.children.as_slice())?;
        self.serialize_attrs(element, &["style"])?;
        self.output.push_str("\\ref*");
        Ok(())
    }

    fn serialize_figure(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = attr(element, "style").unwrap_or("fig");
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_children(element.children.as_slice())?;
        self.serialize_attrs(element, &["style"])?;
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push('*');
        Ok(())
    }

    fn serialize_sidebar(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = attr(element, "style").unwrap_or("esb");
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push('\n');
        self.at_line_start = true;
        if let Some(category) = attr(element, "category") {
            self.output.push_str("\\cat ");
            self.output.push_str(category);
            self.output.push_str("\\cat*");
            self.output.push('\n');
            self.at_line_start = true;
        }
        self.serialize_children(element.children.as_slice())?;
        if attr(element, "closed") != Some("false") {
            self.ensure_newline();
            self.output.push_str("\\esbe");
            self.output.push('\n');
            self.at_line_start = true;
        }
        Ok(())
    }

    fn serialize_table(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        self.ensure_newline();
        self.serialize_children(element.children.as_slice())
    }

    fn serialize_row(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = attr(element, "style").unwrap_or("tr");
        self.ensure_newline();
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_children(element.children.as_slice())?;
        self.output.push('\n');
        self.at_line_start = true;
        Ok(())
    }

    fn serialize_cell(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = required_attr(element, "cell", "style")?;
        self.output.push('\\');
        self.output.push_str(style);
        self.output.push(' ');
        self.at_line_start = false;
        self.serialize_children(element.children.as_slice())
    }

    fn serialize_ms(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        let style = required_attr(element, "ms", "style")?;
        self.output.push('\\');
        self.output.push_str(style);
        let attrs = collect_attrs(element, &["style"]);
        if !attrs.is_empty() {
            self.output.push(' ');
            self.output
                .push_str(serialize_usfm_attrs(attrs.as_slice()).as_str());
        }
        self.output.push_str("\\*");
        self.at_line_start = false;
        Ok(())
    }

    fn serialize_periph(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        self.ensure_newline();
        self.output.push_str("\\periph");
        if let Some(alt) = attr(element, "alt") {
            self.output.push(' ');
            self.output.push_str(alt);
        }
        let attrs = collect_attrs(element, &["alt"]);
        if !attrs.is_empty() {
            self.output
                .push_str(serialize_usfm_attrs(attrs.as_slice()).as_str());
        }
        if !element.children.is_empty() {
            self.output.push('\n');
            self.at_line_start = true;
            self.serialize_children(element.children.as_slice())?;
        }
        Ok(())
    }

    fn serialize_unmatched(&mut self, element: &XmlElement) -> Result<(), UsxToUsfmError> {
        if let Some(marker) = attr(element, "marker") {
            self.output.push('\\');
            self.output.push_str(marker);
        }
        self.at_line_start = false;
        Ok(())
    }

    fn serialize_attrs(
        &mut self,
        element: &XmlElement,
        excluded: &[&str],
    ) -> Result<(), UsxToUsfmError> {
        let attrs = collect_attrs(element, excluded);
        if !attrs.is_empty() {
            self.output
                .push_str(serialize_usfm_attrs(attrs.as_slice()).as_str());
        }
        Ok(())
    }

    fn ensure_newline(&mut self) {
        if !self.at_line_start && !self.output.is_empty() {
            while self.output.ends_with([' ', '\t']) {
                self.output.pop();
            }
            self.output.push('\n');
            self.at_line_start = true;
        }
    }
}

fn required_attr<'a>(
    element: &'a XmlElement,
    name: &'static str,
    key: &'static str,
) -> Result<&'a str, UsxToUsfmError> {
    attr(element, key).ok_or(UsxToUsfmError::MissingAttribute {
        element: name,
        attribute: key,
    })
}

fn attr<'a>(element: &'a XmlElement, key: &str) -> Option<&'a str> {
    element.attr(key)
}

fn collect_attrs<'a>(element: &'a XmlElement, excluded: &[&str]) -> Vec<(&'a str, &'a str)> {
    element
        .attrs
        .iter()
        .filter_map(|(key, value)| {
            if excluded
                .iter()
                .any(|excluded_key| excluded_key == &key.as_str())
                || key == "sid"
                || key == "eid"
                || key == "vid"
            {
                None
            } else {
                Some((key.as_str(), value.as_str()))
            }
        })
        .collect()
}

fn serialize_usfm_attrs(attrs: &[(&str, &str)]) -> String {
    if attrs.is_empty() {
        return String::new();
    }

    let mut out = String::from("|");
    for (index, (key, value)) in attrs.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        let key = if *key == "file" { "src" } else { key };
        out.push_str(key);
        out.push('=');
        out.push('"');
        out.push_str(value.replace('"', "\\\"").as_str());
        out.push('"');
    }
    out
}

fn text_content(nodes: &[XmlNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            XmlNode::Text(text) => out.push_str(text),
            XmlNode::Element(_) => {}
        }
    }
    out.trim().to_string()
}

impl From<XmlError> for UsxToUsfmError {
    fn from(value: XmlError) -> Self {
        Self::Xml(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usx_import_serializes_to_usfm() {
        let xml = r#"<usx version="3.1"><book code="GEN" style="id"/><chapter number="1" style="c" sid="GEN 1"/><para style="p"><verse number="1" style="v" sid="GEN 1:1"/>In the beginning.<verse eid="GEN 1:1"/></para><chapter eid="GEN 1"/></usx>"#;
        let usfm = from_usx_string(xml).expect("USX should import");
        assert_eq!(usfm, "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
    }

    #[test]
    fn imports_notes_and_chars() {
        let xml = r#"<usx version="3.1"><book code="GEN" style="id"/><chapter number="1" style="c" sid="GEN 1"/><para style="p"><verse number="1" style="v" sid="GEN 1:1"/>Text <note caller="+" style="f"><char style="fr">1.1: </char><char style="ft">tail</char></note><verse eid="GEN 1:1"/></para></usx>"#;
        let usfm = from_usx_string(xml).expect("USX should import");
        assert!(usfm.contains("\\f + \\fr 1.1: \\fr*\\ft tail\\ft*\\f*"));
    }
}
