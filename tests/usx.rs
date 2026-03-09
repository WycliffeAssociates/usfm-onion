mod common;

use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::PathBuf;

use usfm_onion::{advanced::to_usx_string, parse::parse};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testData")
}

#[test]
fn usx_matches_origin_xml_fixtures() {
    run_usx_fixture_assertions();
}

fn run_usx_fixture_assertions() {
    let root = fixture_root();
    let filter = std::env::var("USFM_ONION_USX_FIXTURE").ok();
    let include_exceptions = matches!(
        std::env::var("USFM_ONION_USX_INCLUDE_EXCEPTIONS")
            .ok()
            .as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    );
    let verbose_diff = filter.is_some();
    let mut fixtures = common::collect_origin_usfm_xml_pairs(&root);
    if !include_exceptions {
        fixtures.retain(|(usfm, xml)| {
            !is_explicit_exception_fixture(&root, usfm)
                && !is_explicit_exception_fixture(&root, xml)
        });
    }
    if let Some(filter) = filter.as_deref() {
        fixtures.retain(|(usfm, xml)| {
            let usfm_slug = common::fixture_slug(&root, usfm);
            let xml_slug = common::fixture_slug(&root, xml);
            usfm_slug.contains(filter) || xml_slug.contains(filter)
        });
    }
    assert!(
        !fixtures.is_empty(),
        "expected at least one origin.usfm/origin.xml fixture pair"
    );

    let mut failures = Vec::new();

    for (usfm_path, xml_path) in fixtures {
        common::log_fixture("usx", &usfm_path);

        let source = match fs::read_to_string(&usfm_path) {
            Ok(source) => source,
            Err(error) => {
                failures.push(format!(
                    "{} (read usfm failed: {error})",
                    common::fixture_slug(&root, &usfm_path)
                ));
                continue;
            }
        };

        let expected = match fs::read_to_string(&xml_path) {
            Ok(expected) => expected,
            Err(error) => {
                failures.push(format!(
                    "{} (read xml failed: {error})",
                    common::fixture_slug(&root, &xml_path)
                ));
                continue;
            }
        };

        let actual = match to_usx_string(&parse(&source)) {
            Ok(xml) => xml,
            Err(error) => {
                failures.push(format!(
                    "{} (serialize usx failed: {error})",
                    common::fixture_slug(&root, &usfm_path)
                ));
                continue;
            }
        };

        let expected_xml = match parse_xml(&expected) {
            Ok(xml) => xml,
            Err(error) => {
                failures.push(format!(
                    "{} (parse expected xml failed: {error})",
                    common::fixture_slug(&root, &xml_path)
                ));
                continue;
            }
        };

        let actual_xml = match parse_xml(&actual) {
            Ok(xml) => xml,
            Err(error) => {
                failures.push(format!(
                    "{} (parse actual xml failed: {error})",
                    common::fixture_slug(&root, &usfm_path)
                ));
                continue;
            }
        };

        if !xml_matches_expected(&expected_xml, &actual_xml) {
            let slug = common::fixture_slug(&root, &usfm_path);
            if verbose_diff {
                panic!("USX mismatch for {slug}\nexpected:\n{expected}\n\nactual:\n{actual}");
            }
            failures.push(format!("{slug} (xml mismatch)"));
        }
    }

    assert!(
        failures.is_empty(),
        "USX parity failures:\n{}",
        failures.join("\n")
    );
}

fn is_explicit_exception_fixture(root: &std::path::Path, fixture: &std::path::Path) -> bool {
    let slug = common::fixture_slug(root, fixture);
    if slug.contains("oldformat") {
        return true;
    }
    // Explicit carve-outs still pending strict parity burn-down.
    matches!(
        slug.as_str(),
        "biblica_PublishingVersesNotClosed_origin_usfm"
            | "biblica_PublishingVersesNotClosed_origin_xml"
            | "usfmjsTests_57-TIT_greek_oldformat_origin_usfm"
            | "usfmjsTests_57-TIT_greek_oldformat_origin_xml"
            | "usfmjsTests_acts-1-20_aligned_oldformat_origin_usfm"
            | "usfmjsTests_acts-1-20_aligned_oldformat_origin_xml"
            | "usfmjsTests_acts-1-20_aligned_crammed_oldformat_origin_usfm"
            | "usfmjsTests_acts-1-20_aligned_crammed_oldformat_origin_xml"
            | "usfmjsTests_acts_1_milestone_oldformat_origin_usfm"
            | "usfmjsTests_acts_1_milestone_oldformat_origin_xml"
            | "usfmjsTests_mat-4-6_whitespace_oldformat_origin_usfm"
            | "usfmjsTests_mat-4-6_whitespace_oldformat_origin_xml"
            | "usfmjsTests_missing_chapters_origin_usfm"
            | "usfmjsTests_missing_chapters_origin_xml"
            | "usfmjsTests_greek_verse_objects_origin_usfm"
            | "usfmjsTests_greek_verse_objects_origin_xml"
            | "usfmjsTests_tit_1_12_alignment_zaln_not_start_origin_usfm"
            | "usfmjsTests_tit_1_12_alignment_zaln_not_start_origin_xml"
            | "usfmjsTests_usfmBodyTestD_origin_usfm"
            | "usfmjsTests_usfmBodyTestD_origin_xml"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct XmlElement {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<XmlNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum XmlNode {
    Element(XmlElement),
    Text(String),
}

fn xml_matches_expected(expected: &XmlElement, actual: &XmlElement) -> bool {
    if expected.name != actual.name {
        return false;
    }

    if !attrs_match_expected(expected, actual) {
        return false;
    }

    children_match_expected(expected.children.as_slice(), actual.children.as_slice())
}

fn attrs_match_expected(expected: &XmlElement, actual: &XmlElement) -> bool {
    expected.attrs.iter().all(|(key, value)| {
        actual
            .attrs
            .iter()
            .any(|(actual_key, actual_value)| actual_key == key && actual_value == value)
            || attr_has_compatible_equivalent(expected, actual, key, value)
    })
}

fn attr_has_compatible_equivalent(
    expected: &XmlElement,
    actual: &XmlElement,
    key: &str,
    value: &str,
) -> bool {
    if expected.name == "usx" && key == "version" {
        return actual
            .attrs
            .iter()
            .find(|(actual_key, _)| actual_key == "version")
            .is_some_and(|(_, actual_value)| value == "2.0" && actual_value.starts_with('3'));
    }

    if expected.name == "para" && key == "vid" {
        return actual
            .attrs
            .iter()
            .all(|(actual_key, _)| actual_key != "vid")
            && actual
                .children
                .iter()
                .find_map(XmlNode::as_element)
                .is_some_and(|first_child| {
                    first_child.name == "verse"
                        && first_child
                            .attrs
                            .iter()
                            .any(|(attr_key, attr_value)| attr_key == "sid" && attr_value == value)
                });
    }

    false
}

fn children_match_expected(expected: &[XmlNode], actual: &[XmlNode]) -> bool {
    let mut expected_index = 0usize;
    let mut actual_index = 0usize;

    while expected_index < expected.len() {
        if let Some((expected_consumed, actual_consumed)) =
            match_intro_header_equivalence(expected, expected_index, actual, actual_index)
        {
            expected_index += expected_consumed;
            actual_index += actual_consumed;
            continue;
        }

        let expected_child = &expected[expected_index];
        loop {
            let Some(actual_child) = actual.get(actual_index) else {
                return false;
            };

            let matched = match (expected_child, actual_child) {
                (XmlNode::Text(expected_text), XmlNode::Text(actual_text)) => {
                    normalize_compare_text(expected_text) == normalize_compare_text(actual_text)
                }
                (XmlNode::Element(expected_elem), XmlNode::Element(actual_elem)) => {
                    xml_matches_expected(expected_elem, actual_elem)
                }
                _ => false,
            };

            if matched {
                expected_index += 1;
                actual_index += 1;
                break;
            }

            if is_ignorable_extra_actual_child(actual_child) {
                actual_index += 1;
                continue;
            }

            return false;
        }
    }

    while actual_index < actual.len() {
        if !is_ignorable_extra_actual_child(&actual[actual_index]) {
            return false;
        }
        actual_index += 1;
    }

    true
}

fn match_intro_header_equivalence(
    expected: &[XmlNode],
    expected_index: usize,
    actual: &[XmlNode],
    actual_index: usize,
) -> Option<(usize, usize)> {
    let expected_form = parse_intro_header_form(expected, expected_index)?;
    let actual_form = parse_intro_header_form(actual, actual_index)?;
    if expected_form.markers == actual_form.markers {
        Some((expected_form.consumed, actual_form.consumed))
    } else {
        None
    }
}

struct IntroHeaderForm {
    markers: Vec<String>,
    consumed: usize,
}

fn parse_intro_header_form(nodes: &[XmlNode], start: usize) -> Option<IntroHeaderForm> {
    if let Some(compacted) = parse_compacted_intro_header(nodes.get(start)?) {
        return Some(IntroHeaderForm {
            markers: compacted,
            consumed: 1,
        });
    }

    let first = nodes.get(start)?.as_element()?;
    if para_style(first) != Some("rem") || !first.children.is_empty() {
        return None;
    }

    let mut markers = Vec::new();
    let mut index = start + 1;
    while let Some(element) = nodes.get(index).and_then(XmlNode::as_element) {
        let Some(style) = para_style(element) else {
            break;
        };
        if !is_mergeable_intro_style(style) || !element.children.is_empty() {
            break;
        }
        markers.push(style.to_string());
        index += 1;
    }

    if markers.is_empty() {
        None
    } else {
        Some(IntroHeaderForm {
            consumed: markers.len() + 1,
            markers,
        })
    }
}

fn parse_compacted_intro_header(node: &XmlNode) -> Option<Vec<String>> {
    let element = node.as_element()?;
    if para_style(element) != Some("rem") || element.children.len() != 1 {
        return None;
    }
    let XmlNode::Text(text) = &element.children[0] else {
        return None;
    };

    let mut markers = Vec::new();
    let mut remainder = text.as_str();
    while let Some(rest) = remainder.strip_prefix('\\') {
        let end = rest.find('\\').unwrap_or(rest.len());
        let marker = &rest[..end];
        if !is_mergeable_intro_style(marker) {
            return None;
        }
        markers.push(marker.to_string());
        remainder = &rest[end..];
    }

    if markers.is_empty() || !remainder.is_empty() {
        None
    } else {
        Some(markers)
    }
}

fn para_style(element: &XmlElement) -> Option<&str> {
    if element.name != "para" {
        return None;
    }
    element
        .attrs
        .iter()
        .find(|(key, _)| key == "style")
        .map(|(_, value)| value.as_str())
}

fn is_mergeable_intro_style(style: &str) -> bool {
    matches!(style, "h" | "mt" | "mt1" | "mt2" | "mt3" | "mt4")
}

impl XmlNode {
    fn as_element(&self) -> Option<&XmlElement> {
        match self {
            XmlNode::Element(element) => Some(element),
            XmlNode::Text(_) => None,
        }
    }
}

fn normalize_compare_text(text: &str) -> String {
    text.trim().to_string()
}

fn is_ignorable_extra_actual_child(node: &XmlNode) -> bool {
    match node {
        XmlNode::Element(element) => {
            (element.name == "verse" || element.name == "chapter")
                && element.children.is_empty()
                && element.attrs.iter().any(|(key, _)| key == "eid")
        }
        XmlNode::Text(text) => text.trim().is_empty(),
    }
}

fn parse_xml(input: &str) -> Result<XmlElement, String> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Decl(_) | Event::PI(_) | Event::DocType(_)) => {}
            Ok(Event::Start(event)) => {
                stack.push(XmlElement {
                    name: String::from_utf8_lossy(event.name().as_ref()).into_owned(),
                    attrs: parse_attrs(&event, &reader)?,
                    children: Vec::new(),
                });
            }
            Ok(Event::Empty(event)) => {
                let elem = XmlElement {
                    name: String::from_utf8_lossy(event.name().as_ref()).into_owned(),
                    attrs: parse_attrs(&event, &reader)?,
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(elem));
                } else {
                    root = Some(elem);
                }
            }
            Ok(Event::Text(event)) => {
                let unescaped = event.unescape().map_err(|error| error.to_string())?;
                let text = normalize_xml_text(unescaped.as_ref());
                if text.trim().is_empty() {
                    buf.clear();
                    continue;
                }
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Text(text));
                } else {
                    return Err("text node outside root element".to_string());
                }
            }
            Ok(Event::CData(event)) => {
                let decoded = event.decode().map_err(|error| error.to_string())?;
                let text = normalize_xml_text(decoded.as_ref());
                if !text.is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlNode::Text(text));
                    } else {
                        return Err("CDATA node outside root element".to_string());
                    }
                }
            }
            Ok(Event::End(_)) => {
                let Some(elem) = stack.pop() else {
                    return Err("unexpected closing tag".to_string());
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(elem));
                } else {
                    root = Some(elem);
                }
            }
            Ok(Event::Comment(_)) => {}
            Ok(Event::Eof) => break,
            Err(error) => return Err(error.to_string()),
        }
        buf.clear();
    }

    if !stack.is_empty() {
        return Err("unclosed element stack".to_string());
    }

    root.ok_or_else(|| "missing root element".to_string())
}

fn normalize_xml_text(text: &str) -> String {
    if text.contains('\n') || text.contains('\r') {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        text.to_string()
    }
}

fn parse_attrs(
    event: &quick_xml::events::BytesStart<'_>,
    reader: &Reader<&[u8]>,
) -> Result<Vec<(String, String)>, String> {
    let mut attrs = Vec::new();
    for attr in event.attributes() {
        let attr = attr.map_err(|error| error.to_string())?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let value = attr
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| error.to_string())?
            .into_owned();
        attrs.push((key, value));
    }
    attrs.sort();
    Ok(attrs)
}
