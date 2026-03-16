use std::io::Cursor;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::cst::CstDocument;
use crate::usj::{UsjDocument, UsjElement, UsjNode, cst_to_usj, usfm_to_usj};

#[derive(Debug)]
pub enum UsxError {
    Xml(quick_xml::Error),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Usj(crate::usj::UsjError),
}

impl From<quick_xml::Error> for UsxError {
    fn from(value: quick_xml::Error) -> Self {
        Self::Xml(value)
    }
}

impl From<std::io::Error> for UsxError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::string::FromUtf8Error> for UsxError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<crate::usj::UsjError> for UsxError {
    fn from(value: crate::usj::UsjError) -> Self {
        Self::Usj(value)
    }
}

impl std::fmt::Display for UsxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(error) => write!(f, "xml serialization error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Utf8(error) => write!(f, "utf8 error: {error}"),
            Self::Usj(error) => write!(f, "usj conversion error: {error}"),
        }
    }
}

impl std::error::Error for UsxError {}

pub fn usfm_to_usx(source: &str) -> Result<String, UsxError> {
    let document = usfm_to_usj(source)?;
    usj_to_usx_with_version(&document, usx_version(source))
}

pub fn cst_to_usx(document: &CstDocument<'_>) -> Result<String, UsxError> {
    let usj = cst_to_usj(document);
    usj_to_usx_with_version(&usj, "3.0")
}

pub fn usj_to_usx(document: &UsjDocument) -> Result<String, UsxError> {
    usj_to_usx_with_version(document, document.version.as_str())
}

fn usj_to_usx_with_version(document: &UsjDocument, version: &str) -> Result<String, UsxError> {
    let mut serializer = UsxSerializer::new(version);
    serializer.write(document)
}

struct UsxSerializer<'a> {
    version: &'a str,
    writer: Writer<Cursor<Vec<u8>>>,
    current_chapter_sid: Option<String>,
    current_verse_sid: Option<String>,
}

impl<'a> UsxSerializer<'a> {
    fn new(version: &'a str) -> Self {
        Self {
            version,
            writer: Writer::new(Cursor::new(Vec::new())),
            current_chapter_sid: None,
            current_verse_sid: None,
        }
    }

    fn write(&mut self, document: &UsjDocument) -> Result<String, UsxError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        let mut root = BytesStart::new("usx");
        root.push_attribute(("version", self.version));
        self.writer.write_event(Event::Start(root))?;
        self.write_nodes(&document.content)?;
        self.close_verse()?;
        self.close_chapter()?;
        self.writer.write_event(Event::End(BytesEnd::new("usx")))?;

        Ok(String::from_utf8(self.writer.get_ref().get_ref().clone())?)
    }

    fn write_nodes(&mut self, nodes: &[UsjNode]) -> Result<(), UsxError> {
        for node in nodes {
            self.write_node(node)?;
        }
        Ok(())
    }

    fn write_node(&mut self, node: &UsjNode) -> Result<(), UsxError> {
        match node {
            UsjNode::Text(text) => {
                self.writer.write_event(Event::Text(BytesText::new(text)))?;
            }
            UsjNode::Element(element) => self.write_element(element)?,
        }
        Ok(())
    }

    fn write_element(&mut self, element: &UsjElement) -> Result<(), UsxError> {
        match element {
            UsjElement::Book {
                marker,
                code,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("book");
                elem.push_attribute(("code", code.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("book", elem, content)?;
            }
            UsjElement::Chapter {
                marker,
                number,
                sid,
                altnumber,
                pubnumber,
                extra,
            } => {
                self.close_verse()?;
                self.close_chapter()?;

                let mut elem = BytesStart::new("chapter");
                elem.push_attribute(("number", number.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                if let Some(sid) = sid.as_deref() {
                    elem.push_attribute(("sid", sid));
                    self.current_chapter_sid = Some(sid.to_string());
                }
                if let Some(value) = altnumber.as_deref() {
                    elem.push_attribute(("altnumber", value));
                }
                if let Some(value) = pubnumber.as_deref() {
                    elem.push_attribute(("pubnumber", value));
                }
                push_extra_attrs(&mut elem, extra, &[]);
                self.writer.write_event(Event::Empty(elem))?;
            }
            UsjElement::Verse {
                marker,
                number,
                sid,
                altnumber,
                pubnumber,
                extra,
            } => {
                self.close_verse()?;

                let mut elem = BytesStart::new("verse");
                elem.push_attribute(("number", number.as_str()));
                elem.push_attribute(("style", marker.as_str()));
                if let Some(sid) = sid.as_deref() {
                    elem.push_attribute(("sid", sid));
                    self.current_verse_sid = Some(sid.to_string());
                }
                if let Some(value) = altnumber.as_deref() {
                    elem.push_attribute(("altnumber", value));
                }
                if let Some(value) = pubnumber.as_deref() {
                    elem.push_attribute(("pubnumber", value));
                }
                push_extra_attrs(&mut elem, extra, &[]);
                self.writer.write_event(Event::Empty(elem))?;
            }
            UsjElement::Para {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("para");
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("para", elem, content)?;
                self.close_verse()?;
            }
            UsjElement::Char {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("char");
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("char", elem, content)?;
            }
            UsjElement::Ref { content, extra } => {
                let mut elem = BytesStart::new("ref");
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("ref", elem, content)?;
            }
            UsjElement::Note {
                marker,
                caller,
                content,
                category,
                extra,
            } => {
                let mut elem = BytesStart::new("note");
                elem.push_attribute(("style", marker.as_str()));
                elem.push_attribute(("caller", caller.as_str()));
                if let Some(category) = category.as_deref() {
                    elem.push_attribute(("category", category));
                }
                push_extra_attrs(&mut elem, extra, &["category"]);
                self.write_container("note", elem, content)?;
            }
            UsjElement::Milestone { marker, extra } => {
                let mut elem = BytesStart::new("ms");
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.writer.write_event(Event::Empty(elem))?;
            }
            UsjElement::Figure {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("figure");
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("figure", elem, content)?;
            }
            UsjElement::Sidebar {
                marker,
                content,
                category,
                extra,
            } => {
                let saved_verse = self.current_verse_sid.clone();
                let mut elem = BytesStart::new("sidebar");
                elem.push_attribute(("style", marker.as_str()));
                if let Some(category) = category.as_deref() {
                    elem.push_attribute(("category", category));
                }
                push_extra_attrs(&mut elem, extra, &["category"]);
                self.write_container("sidebar", elem, content)?;
                self.current_verse_sid = saved_verse;
            }
            UsjElement::Periph { content, alt, extra } => {
                let mut elem = BytesStart::new("periph");
                if let Some(alt) = alt.as_deref() {
                    elem.push_attribute(("alt", alt));
                }
                push_extra_attrs(&mut elem, extra, &["alt"]);
                self.write_container("periph", elem, content)?;
            }
            UsjElement::Table { content, extra } => {
                let mut elem = BytesStart::new("table");
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("table", elem, content)?;
                self.close_verse()?;
            }
            UsjElement::TableRow {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("row");
                elem.push_attribute(("style", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.write_container("row", elem, content)?;
            }
            UsjElement::TableCell {
                marker,
                align,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("cell");
                elem.push_attribute(("style", marker.as_str()));
                if let Some(align) = align.as_deref() {
                    elem.push_attribute(("align", align));
                }
                push_extra_attrs(&mut elem, extra, &["align"]);
                self.write_container("cell", elem, content)?;
            }
            UsjElement::Unknown {
                marker,
                content,
                extra,
            } => {
                let mut elem = BytesStart::new("para");
                elem.push_attribute(("style", marker.as_str()));
                elem.push_attribute(("status", "unknown"));
                push_extra_attrs(&mut elem, extra, &["status"]);
                self.write_container("para", elem, content)?;
            }
            UsjElement::Unmatched {
                marker,
                content: _,
                extra,
            } => {
                let mut elem = BytesStart::new("unmatched");
                elem.push_attribute(("marker", marker.as_str()));
                push_extra_attrs(&mut elem, extra, &[]);
                self.writer.write_event(Event::Empty(elem))?;
            }
            UsjElement::OptBreak {} => {
                self.writer.write_event(Event::Empty(BytesStart::new("optbreak")))?;
            }
        }
        Ok(())
    }

    fn write_container(
        &mut self,
        name: &str,
        elem: BytesStart<'_>,
        content: &[UsjNode],
    ) -> Result<(), UsxError> {
        if content.is_empty() {
            self.writer.write_event(Event::Empty(elem))?;
        } else {
            self.writer.write_event(Event::Start(elem))?;
            self.write_nodes(content)?;
            self.writer.write_event(Event::End(BytesEnd::new(name)))?;
        }
        Ok(())
    }

    fn close_verse(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_verse_sid.take() {
            let mut elem = BytesStart::new("verse");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        Ok(())
    }

    fn close_chapter(&mut self) -> Result<(), UsxError> {
        if let Some(sid) = self.current_chapter_sid.take() {
            let mut elem = BytesStart::new("chapter");
            elem.push_attribute(("eid", sid.as_str()));
            self.writer.write_event(Event::Empty(elem))?;
        }
        Ok(())
    }
}

fn push_extra_attrs(
    elem: &mut BytesStart<'_>,
    extra: &std::collections::BTreeMap<String, String>,
    skip: &[&str],
) {
    for (key, value) in extra {
        if skip.iter().any(|skip_key| *skip_key == key) {
            continue;
        }
        elem.push_attribute((key.as_str(), value.as_str()));
    }
}

fn usx_version(source: &str) -> &str {
    source
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("\\usfm ").map(|rest| {
                if rest.starts_with("3.1") {
                    "3.1"
                } else if rest.starts_with("2.") {
                    "2.0"
                } else {
                    "3.0"
                }
            })
        })
        .unwrap_or("3.0")
}

#[cfg(test)]
fn collect_usx_fixture_pairs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    collect_usx_fixture_pairs_into(root, &mut pairs);
    pairs.sort();
    pairs
}

#[cfg(test)]
fn collect_usx_fixture_pairs_into(root: &Path, pairs: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut usfm = None;
    let mut usx = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usx_fixture_pairs_into(&path, pairs);
            continue;
        }
        match path.file_name().and_then(|name| name.to_str()) {
            Some("origin.usfm") => usfm = Some(path),
            Some("origin.xml") => usx = Some(path),
            _ => {}
        }
    }

    if let (Some(usfm), Some(usx)) = (usfm, usx) {
        pairs.push((usfm, usx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_basic_usx() {
        let xml = usfm_to_usx("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n")
            .expect("USX should serialize");
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(xml.contains("<usx version=\"3.0\">"));
        assert!(xml.contains("<book code=\"GEN\" style=\"id\">"));
        assert!(xml.contains("Genesis</book>"));
        assert!(xml.contains("<chapter number=\"1\" style=\"c\" sid=\"GEN 1\"/>"));
        assert!(xml.contains("<verse number=\"1\" style=\"v\" sid=\"GEN 1:1\"/>"));
        assert!(xml.contains("<verse eid=\"GEN 1:1\"/>"));
        assert!(xml.contains("<chapter eid=\"GEN 1\"/>"));
    }

    #[test]
    fn writes_word_attributes_and_notes() {
        let xml = usfm_to_usx("\\id GEN\n\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\"\\w*\\f + \\fr 1:1 \\ft tail\\f*\n")
            .expect("USX should serialize");
        assert!(xml.contains("<char style=\"w\""));
        assert!(xml.contains("lemma=\"grace\""));
        assert!(xml.contains("gracious"));
        assert!(xml.contains("<note style=\"f\" caller=\"+\">"));
        assert!(xml.contains("<char style=\"fr\">"));
        assert!(xml.contains("1:1"));
        assert!(xml.contains("<char style=\"ft\">"));
        assert!(xml.contains("tail"));
    }

    #[test]
    fn paired_fixtures_serialize_valid_xml() {
        for (usfm_path, usx_path) in collect_usx_fixture_pairs(Path::new("testData")) {
            let source = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let actual = usfm_to_usx(&source)
                .unwrap_or_else(|error| panic!("USX export failed for {}: {error}", usfm_path.display()));
            let expected = fs::read_to_string(&usx_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usx_path.display()));

            assert!(xml_is_well_formed(&actual), "generated xml invalid for {}", usfm_path.display());
            assert!(xml_is_well_formed(&expected), "fixture xml invalid for {}", usx_path.display());
        }
    }

    fn xml_is_well_formed(source: &str) -> bool {
        let mut reader = quick_xml::Reader::from_str(source);
        reader.config_mut().trim_text(false);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => return true,
                Ok(_) => buf.clear(),
                Err(_) => return false,
            }
        }
    }
}
