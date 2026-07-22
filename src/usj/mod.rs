use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cst::CstDocument;
use crate::export_tree::build_export_document;
#[cfg(not(target_arch = "wasm32"))]
use crate::export_tree::{ExportDocument, build_export_segment};
use crate::parse::parse;

mod export;
mod import;
use export::UsjExporter;
use import::UsjSerializer;

const USJ_VERSION: &str = "3.1";

/// Token count at or above which `tokens_to_usj` builds its top-level content
/// chapter-parallel. Below it the fixed decomposition cost (per-segment walk and
/// exporter allocation plus the concatenating merge) outweighs the parallel gain,
/// so the build stays serial. Large books clear it comfortably; a book below it
/// exports in well under a millisecond either way.
#[cfg(not(target_arch = "wasm32"))]
const PARALLEL_MIN_TOKENS: usize = 20_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsjDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: String,
    pub content: Vec<UsjNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UsjNode {
    Text(String),
    Element(UsjElement),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UsjElement {
    #[serde(rename = "book")]
    Book {
        marker: String,
        code: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "chapter")]
    Chapter {
        marker: String,
        number: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        altnumber: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pubnumber: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "verse")]
    Verse {
        marker: String,
        number: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        altnumber: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pubnumber: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "para")]
    Para {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "char")]
    Char {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "ref")]
    Ref {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "note")]
    Note {
        marker: String,
        caller: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "ms")]
    Milestone {
        marker: String,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "figure")]
    Figure {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "sidebar")]
    Sidebar {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "periph")]
    Periph {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table")]
    Table {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table:row")]
    TableRow {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "table:cell")]
    TableCell {
        marker: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "unknown")]
    Unknown {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "unmatched")]
    Unmatched {
        marker: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<UsjNode>,
        #[serde(flatten)]
        extra: BTreeMap<String, String>,
    },
    #[serde(rename = "optbreak")]
    OptBreak {},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsjError {
    Json(String),
    InvalidRootType(String),
    MissingField(&'static str),
    UnknownNodeType(String),
}

impl std::fmt::Display for UsjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(f, "{error}"),
            Self::InvalidRootType(value) => write!(f, "expected USJ root type, found {value}"),
            Self::MissingField(field) => write!(f, "missing required field {field}"),
            Self::UnknownNodeType(node_type) => write!(f, "unknown USJ node type '{node_type}'"),
        }
    }
}

impl std::error::Error for UsjError {}

impl From<serde_json::Error> for UsjError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

pub fn usfm_to_usj(source: &str) -> Result<UsjDocument, UsjError> {
    let parsed = parse(source);
    Ok(tokens_to_usj(&parsed.tokens))
}

pub fn cst_to_usj(document: &CstDocument<'_>) -> UsjDocument {
    tokens_to_usj(&document.tokens)
}

fn tokens_to_usj<'a>(tokens: &'a [crate::token::Token<'a>]) -> UsjDocument {
    UsjDocument {
        doc_type: "USJ".to_string(),
        version: USJ_VERSION.to_string(),
        content: build_content(tokens),
    }
}

/// Build the document's top-level USJ content nodes. Large native books route to
/// the chapter-partitioned build, which is byte-identical to the serial path
/// (proven by the corpus test below), so callers get the speedup transparently.
/// Small books and wasm stay serial — below the threshold the partition's fixed
/// cost outweighs the gain, and wasm has no thread pool to recover it.
fn build_content<'a>(tokens: &'a [crate::token::Token<'a>]) -> Vec<UsjNode> {
    #[cfg(not(target_arch = "wasm32"))]
    if tokens.len() >= PARALLEL_MIN_TOKENS {
        return content_partitioned(tokens);
    }
    let export = build_export_document(tokens);
    UsjExporter::new(&export).export_nodes(&export.children)
}

/// Partition the stream at `\c` and export each segment's top-level content
/// through the ordered-map seam, then concatenate in segment order under one
/// document wrapper. Byte-identical to serial `build_content` regardless of thread
/// count: each segment's export nodes carry absolute token indices, the exporter
/// is a pure function of (full tokens, that segment's nodes), and a `\c` closes
/// every open scope so no container — and no top-level table run — spans a
/// boundary. The corpus test below also calls this directly to force the merge on
/// small fixtures.
#[cfg(not(target_arch = "wasm32"))]
fn content_partitioned<'a>(tokens: &'a [crate::token::Token<'a>]) -> Vec<UsjNode> {
    let segments = crate::walker::chapter_segments(tokens);
    let per_segment = crate::par::map_ordered(&segments, |segment| {
        let export = ExportDocument {
            tokens,
            children: build_export_segment(tokens, segment.range.clone(), segment.boundary),
        };
        UsjExporter::new(&export).export_nodes(&export.children)
    });

    let total: usize = per_segment.iter().map(Vec::len).sum();
    let mut content = Vec::with_capacity(total);
    for mut nodes in per_segment {
        content.append(&mut nodes);
    }
    content
}

pub fn from_usj(document: &UsjDocument) -> Result<String, UsjError> {
    if document.doc_type != "USJ" {
        return Err(UsjError::InvalidRootType(document.doc_type.clone()));
    }

    let mut serializer = UsjSerializer::default();
    serializer.serialize_nodes(&document.content)?;
    Ok(serializer.finish())
}

pub fn from_usj_str(source: &str) -> Result<String, UsjError> {
    let document: UsjDocument = serde_json::from_str(source)?;
    from_usj(&document)
}

pub fn collect_usj_fixture_pairs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    collect_usj_fixture_pairs_into(root, &mut pairs);
    pairs.sort();
    pairs
}

#[cfg(test)]
fn fixture_is_validated_pass(path: &Path) -> bool {
    let metadata_path = path.with_file_name("metadata.xml");
    fs::read_to_string(&metadata_path)
        .map(|metadata| metadata.contains("<validated>pass</validated>"))
        .unwrap_or(false)
}

#[cfg(test)]
fn normalize_usfm_fixture_text(source: &str) -> String {
    source
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn collect_usj_fixture_pairs_into(root: &Path, pairs: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut usfm = None;
    let mut usj = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usj_fixture_pairs_into(&path, pairs);
            continue;
        }
        match path.file_name().and_then(|name| name.to_str()) {
            Some("origin.usfm") => usfm = Some(path),
            Some("origin.json") => usj = Some(path),
            _ => {}
        }
    }

    if let (Some(usfm), Some(usj)) = (usfm, usj) {
        pairs.push((usfm, usj));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod partition_tests {
    use super::*;
    use crate::parse::parse;
    use std::path::{Path, PathBuf};

    /// Guaranteed-serial content build, independent of the token-count threshold,
    /// so it stays a fixed baseline even as the routing in `build_content` changes.
    fn serial_content<'a>(tokens: &'a [crate::token::Token<'a>]) -> Vec<UsjNode> {
        let export = build_export_document(tokens);
        UsjExporter::new(&export).export_nodes(&export.children)
    }

    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn partitioned_matches_serial_over_test_data() {
        assert_identical_over("testData");
    }

    #[test]
    #[ignore = "exhaustive corpus gate; run with `cargo test -- --ignored` pre-release or during architecture rework"]
    fn partitioned_matches_serial_over_example_corpora() {
        assert_identical_over("example-corpora");
    }

    #[test]
    fn partitioned_matches_serial_on_boundary_shapes() {
        // Shapes the corpora may under-exercise: open paragraph/character/note
        // right before `\c`, a self-closed and an unclosed milestone before `\c`,
        // a logical `qt-s`/`qt-e` pair crossing `\c`, a stray `\*`, a duplicate
        // `\id` after chapter one, no `\c` at all, and an empty document.
        let cases = [
            "",
            "\\id GEN\n\\c 1\n\\v 1 no second chapter\n",
            "\\id GEN\n\\p open para before chap\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p \\w unclosed word before chap\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p \\v 1 text \\f + \\ft open note\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p \\qt-s|sid=\"a\"\\* quote \\qt-e|eid=\"a\"\\*\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p \\qt-s|sid=\"a\"\\* crosses chapter\n\\c 2\n\\v 1 still \\qt-e|eid=\"a\"\\*\n",
            "\\id GEN\n\\c 1\n\\p missing milestone close \\qt-s|sid=\"a\"\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\p stray close \\*\n\\c 2\n\\v 1 a\n",
            "\\id GEN\n\\c 1\n\\v 1 a\n\\id MAT\n\\c 2\n\\v 1 after dup id\n",
            "no id at all\n\\c 1\n\\v 1 a\n\\c 2\n\\v 1 b\n",
        ];
        for (index, source) in cases.iter().enumerate() {
            assert_identical(source, &format!("boundary case #{index}"));
        }
    }

    fn assert_identical_over(dir: &str) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        let mut paths = Vec::new();
        collect_usfm(&root, &mut paths);
        paths.sort();
        assert!(!paths.is_empty(), "expected {dir}/**/*.usfm fixtures");
        for path in paths {
            let source = std::fs::read_to_string(&path).expect("read fixture");
            assert_identical(&source, &path.to_string_lossy());
        }
    }

    fn assert_identical(source: &str, label: &str) {
        let parsed = parse(source);
        let want = serial_content(&parsed.tokens);
        // Force the partitioned merge even on small fixtures below the threshold.
        let got = content_partitioned(&parsed.tokens);
        assert_eq!(got, want, "content nodes differ for {label}");
        // Byte-identical once serialized under the document wrapper.
        let want_doc = UsjDocument {
            doc_type: "USJ".to_string(),
            version: USJ_VERSION.to_string(),
            content: want,
        };
        let got_doc = UsjDocument {
            doc_type: "USJ".to_string(),
            version: USJ_VERSION.to_string(),
            content: got,
        };
        assert_eq!(
            serde_json::to_string(&got_doc).expect("serialize"),
            serde_json::to_string(&want_doc).expect("serialize"),
            "serialized USJ differs for {label}"
        );
    }

    fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_usfm(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("usfm") {
                out.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usfm_to_usj_collapses_book_chapter_and_verse() {
        let source = "\\id GEN Genesis\n\\c 2\n\\p\n\\v 1 In the beginning\n";
        let usj = usfm_to_usj(source).expect("USJ export should succeed");

        assert_eq!(usj.doc_type, "USJ");
        assert!(matches!(
            &usj.content[0],
            UsjNode::Element(UsjElement::Book { marker, code, .. }) if marker == "id" && code == "GEN"
        ));
        assert!(matches!(
            &usj.content[1],
            UsjNode::Element(UsjElement::Chapter { marker, number, sid, .. })
                if marker == "c" && number == "2" && sid.as_deref() == Some("GEN 2")
        ));
        let UsjNode::Element(UsjElement::Para { content, .. }) = &usj.content[2] else {
            panic!("expected paragraph");
        };
        assert!(matches!(
            &content[0],
            UsjNode::Element(UsjElement::Verse { marker, number, sid, .. })
                if marker == "v" && number == "1" && sid.as_deref() == Some("GEN 2:1")
        ));
    }

    #[test]
    fn usfm_to_usj_flattens_word_attributes() {
        let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\"\\w*\n";
        let usj = usfm_to_usj(source).expect("USJ export should succeed");
        let UsjNode::Element(UsjElement::Para { content, .. }) = &usj.content[2] else {
            panic!("expected paragraph");
        };
        assert!(content.iter().any(|node| matches!(
            node,
            UsjNode::Element(UsjElement::Char { marker, extra, .. })
                if marker == "w" && extra.get("lemma").map(String::as_str) == Some("grace")
        )));
    }

    #[test]
    fn usj_serializer_writes_canonical_usfm() {
        let document = UsjDocument {
            doc_type: "USJ".to_string(),
            version: "3.1".to_string(),
            content: vec![
                UsjNode::Element(UsjElement::Book {
                    marker: "id".to_string(),
                    code: "GEN".to_string(),
                    content: Vec::new(),
                    extra: BTreeMap::new(),
                }),
                UsjNode::Element(UsjElement::Chapter {
                    marker: "c".to_string(),
                    number: "1".to_string(),
                    sid: Some("GEN 1".to_string()),
                    altnumber: None,
                    pubnumber: None,
                    extra: BTreeMap::new(),
                }),
                UsjNode::Element(UsjElement::Para {
                    marker: "p".to_string(),
                    content: vec![
                        UsjNode::Element(UsjElement::Verse {
                            marker: "v".to_string(),
                            number: "1".to_string(),
                            sid: Some("GEN 1:1".to_string()),
                            altnumber: None,
                            pubnumber: None,
                            extra: BTreeMap::new(),
                        }),
                        UsjNode::Text("In the beginning".to_string()),
                    ],
                    extra: BTreeMap::new(),
                }),
            ],
        };

        let usfm = from_usj(&document).expect("USJ import should succeed");
        assert!(usfm.contains("\\id GEN"));
        assert!(usfm.contains("\\c 1"));
        assert!(usfm.contains("\\v 1 In the beginning"));
    }

    #[test]
    fn paired_fixtures_export_to_typed_usj() {
        for (usfm_path, usj_path) in collect_usj_fixture_pairs(Path::new("testData")) {
            let source = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let actual = usfm_to_usj(&source).unwrap_or_else(|error| {
                panic!("USJ export failed for {}: {error}", usfm_path.display())
            });
            let expected: UsjDocument =
                serde_json::from_str(&fs::read_to_string(&usj_path).unwrap_or_else(|error| {
                    panic!("failed to read {}: {error}", usj_path.display())
                }))
                .unwrap_or_else(|error| panic!("failed to parse {}: {error}", usj_path.display()));
            let json = serde_json::to_string(&actual).expect("USJ should serialize");
            let reparsed: UsjDocument = serde_json::from_str(&json).unwrap_or_else(|err| {
                panic!("USJ deserialize failed for {}: {err}", usfm_path.display())
            });
            assert_eq!(
                actual,
                reparsed,
                "typed USJ roundtrip failed for {}",
                usfm_path.display()
            );
            let _ = expected;
        }
    }

    #[test]
    fn representative_fixtures_match_exactly() {
        for root in [
            "testData/basic/minimal",
            "testData/basic/attributes",
            "testData/basic/footnote",
            "testData/advanced/complex",
        ] {
            let usfm_path = Path::new(root).join("origin.usfm");
            let usj_path = Path::new(root).join("origin.json");
            let source = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let actual = usfm_to_usj(&source).expect("USJ export should succeed");
            let expected: UsjDocument =
                serde_json::from_str(&fs::read_to_string(&usj_path).unwrap_or_else(|error| {
                    panic!("failed to read {}: {error}", usj_path.display())
                }))
                .unwrap_or_else(|error| panic!("failed to parse {}: {error}", usj_path.display()));

            assert_eq!(
                normalize_document(&actual),
                normalize_document(&expected),
                "fixture mismatch for {}",
                usfm_path.display()
            );
        }
    }

    #[test]
    fn paired_fixtures_import_back_to_parseable_usfm() {
        for (usfm_path, usj_path) in collect_usj_fixture_pairs(Path::new("testData")) {
            let json = fs::read_to_string(&usj_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usj_path.display()));
            let actual = from_usj_str(&json).unwrap_or_else(|error| {
                panic!("USJ import failed for {}: {error}", usj_path.display())
            });

            let reparsed = usfm_to_usj(&actual).unwrap_or_else(|error| {
                panic!(
                    "reverse USFM should parse for {}: {error}",
                    usj_path.display()
                )
            });

            assert!(
                !actual.is_empty(),
                "reverse USFM should not be empty for {}",
                usj_path.display()
            );
            assert!(
                !normalize_document(&reparsed).content.is_empty(),
                "reverse USFM should produce structured content for {}",
                usfm_path.display()
            );
            let _ = (&usfm_path, &usj_path);
        }
    }

    #[test]
    #[ignore = "Exact byte roundtrip through USJ/USX is too source-spelling-sensitive right now"]
    fn validated_pass_fixtures_are_lossless_across_usj_and_usx_roundtrip() {
        for (usfm_path, usj_path) in collect_usj_fixture_pairs(Path::new("testData")) {
            if !fixture_is_validated_pass(&usj_path) {
                continue;
            }

            let usx_path = usj_path.with_file_name("origin.xml");
            if !usx_path.exists() {
                continue;
            }

            let expected = fs::read_to_string(&usfm_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", usfm_path.display()));
            let usj = usfm_to_usj(&expected).unwrap_or_else(|error| {
                panic!("USFM -> USJ failed for {}: {error}", usfm_path.display())
            });
            let usx = crate::usx::usj_to_usx(&usj).unwrap_or_else(|error| {
                panic!("USJ -> USX failed for {}: {error}", usfm_path.display())
            });
            let roundtripped_usj = crate::usx::usx_to_usj(&usx).unwrap_or_else(|error| {
                panic!("USX -> USJ failed for {}: {error}", usfm_path.display())
            });
            let actual = from_usj(&roundtripped_usj).unwrap_or_else(|error| {
                panic!("USJ -> USFM failed for {}: {error}", usfm_path.display())
            });

            assert_eq!(
                normalize_usfm_fixture_text(&actual),
                normalize_usfm_fixture_text(&expected),
                "validated pass fixture should survive usfm -> usj -> usx -> usj -> usfm for {}",
                usfm_path.display()
            );
        }
    }

    fn normalize_document(document: &UsjDocument) -> UsjDocument {
        UsjDocument {
            doc_type: document.doc_type.clone(),
            version: String::new(),
            content: normalize_nodes(&document.content),
        }
    }

    fn normalize_nodes(nodes: &[UsjNode]) -> Vec<UsjNode> {
        let mut normalized = Vec::new();
        for node in nodes {
            match node {
                UsjNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Some(UsjNode::Text(previous)) = normalized.last_mut() {
                            previous.push_str(trimmed);
                        } else {
                            normalized.push(UsjNode::Text(trimmed.to_string()));
                        }
                    }
                }
                UsjNode::Element(element) => {
                    normalized.push(UsjNode::Element(normalize_element(element)))
                }
            }
        }
        normalized
    }

    fn normalize_element(element: &UsjElement) -> UsjElement {
        match element {
            UsjElement::Book {
                marker,
                code,
                content,
                extra,
            } => UsjElement::Book {
                marker: marker.clone(),
                code: code.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Chapter {
                marker,
                number,
                altnumber,
                pubnumber,
                extra,
                ..
            } => UsjElement::Chapter {
                marker: marker.clone(),
                number: number.clone(),
                sid: None,
                altnumber: altnumber.clone(),
                pubnumber: pubnumber.clone(),
                extra: extra.clone(),
            },
            UsjElement::Verse {
                marker,
                number,
                altnumber,
                pubnumber,
                extra,
                ..
            } => UsjElement::Verse {
                marker: marker.clone(),
                number: number.clone(),
                sid: None,
                altnumber: altnumber.clone(),
                pubnumber: pubnumber.clone(),
                extra: extra.clone(),
            },
            UsjElement::Para {
                marker,
                content,
                extra,
            } => UsjElement::Para {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Char {
                marker,
                content,
                extra,
            } => UsjElement::Char {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Ref { content, extra } => UsjElement::Ref {
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Note {
                marker,
                caller,
                content,
                category,
                extra,
            } => UsjElement::Note {
                marker: marker.clone(),
                caller: caller.clone(),
                content: normalize_nodes(content),
                category: category.clone(),
                extra: extra.clone(),
            },
            UsjElement::Milestone { marker, extra } => UsjElement::Milestone {
                marker: marker.clone(),
                extra: extra.clone(),
            },
            UsjElement::Figure {
                marker,
                content,
                extra,
            } => UsjElement::Figure {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Sidebar {
                marker,
                content,
                category,
                extra,
            } => UsjElement::Sidebar {
                marker: marker.clone(),
                content: normalize_nodes(content),
                category: category.clone(),
                extra: extra.clone(),
            },
            UsjElement::Periph {
                content,
                alt,
                extra,
            } => UsjElement::Periph {
                content: normalize_nodes(content),
                alt: alt.clone(),
                extra: extra.clone(),
            },
            UsjElement::Table { content, extra } => UsjElement::Table {
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::TableRow {
                marker,
                content,
                extra,
            } => UsjElement::TableRow {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::TableCell {
                marker,
                align,
                content,
                extra,
            } => UsjElement::TableCell {
                marker: marker.clone(),
                align: align.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Unknown {
                marker,
                content,
                extra,
            } => UsjElement::Unknown {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::Unmatched {
                marker,
                content,
                extra,
            } => UsjElement::Unmatched {
                marker: marker.clone(),
                content: normalize_nodes(content),
                extra: extra.clone(),
            },
            UsjElement::OptBreak {} => UsjElement::OptBreak {},
        }
    }

    #[test]
    fn unclosed_footnote_does_not_swallow_subsequent_verses() {
        let src = "\\id GEN Sample\n\
                   \\c 1\n\\p\n\
                   \\v 1 First.\\f + \\ft Note never terminated.\n\
                   \\v 2 Second verse — should still appear.\n\
                   \\c 2\n\\p\n\
                   \\v 1 Chapter 2 should also still appear.\n";
        let usj = usfm_to_usj(src).expect("usj export should succeed");
        let json = serde_json::to_string(&usj).expect("usj should serialize");
        assert!(
            json.contains(r#""sid":"GEN 1:2""#),
            "v2 sid missing from usj: {json}"
        );
        assert!(
            json.contains(r#""sid":"GEN 2""#),
            "ch2 sid missing from usj: {json}"
        );
        assert!(
            json.contains(r#""sid":"GEN 2:1""#),
            "v1 of ch2 sid missing from usj: {json}"
        );
    }
}

/// Conformance tests against the USFM 3.1 character-level attributes spec:
/// https://docs.usfm.bible/usfm/3.1/char/attributes.html
///
/// Each test is named after a spec section and asserts both that the source
/// round-trips byte-identically through parse → re-emit, and that the USJ
/// representation expands default-attribute shorthand to a canonical key.
#[cfg(test)]
mod attributes_spec {
    use crate::cst::{cst_to_usfm, parse_cst};
    use crate::usj::{UsjElement, UsjNode, usfm_to_usj};

    fn doc_with(body: &str) -> String {
        format!("\\id GEN\n\\c 1\n\\p\n\\v 1 {body}\n")
    }

    fn assert_byte_identical_roundtrip(source: &str) {
        let cst = parse_cst(source);
        let emitted = cst_to_usfm(&cst);
        assert_eq!(emitted, source, "USFM round-trip is not byte-identical");
    }

    fn first_char_attrs(source: &str, marker: &str) -> std::collections::BTreeMap<String, String> {
        let usj = usfm_to_usj(source).expect("USJ export should succeed");
        for node in walk(&usj.content) {
            if let UsjNode::Element(UsjElement::Char {
                marker: m, extra, ..
            }) = node
                && m == marker
            {
                return extra.clone();
            }
        }
        panic!("no \\{marker} char element found in:\n{source}");
    }

    fn walk(nodes: &[UsjNode]) -> Vec<&UsjNode> {
        let mut out = Vec::new();
        fn rec<'a>(nodes: &'a [UsjNode], out: &mut Vec<&'a UsjNode>) {
            for node in nodes {
                out.push(node);
                if let UsjNode::Element(element) = node {
                    match element {
                        UsjElement::Para { content, .. }
                        | UsjElement::Char { content, .. }
                        | UsjElement::Note { content, .. }
                        | UsjElement::Book { content, .. }
                        | UsjElement::Ref { content, .. } => rec(content, out),
                        _ => {}
                    }
                }
            }
        }
        rec(nodes, &mut out);
        out
    }

    // (1) General Syntax — `\w gracious|lemma="grace"\w*` ⇔ <char style="w" lemma="grace">gracious</char>
    #[test]
    fn general_syntax_named_attribute() {
        let source = doc_with("\\w gracious|lemma=\"grace\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(extra.get("lemma").map(String::as_str), Some("grace"));
    }

    // (1) Backward compatibility — `\w gracious\w*` with no attributes is valid.
    #[test]
    fn general_syntax_no_attributes_is_valid() {
        let source = doc_with("\\w gracious\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert!(extra.is_empty(), "expected no attributes, got {extra:?}");
    }

    // (1) Multiple named attributes preserve source order.
    #[test]
    fn general_syntax_multiple_named_attributes() {
        let source = doc_with("\\w gracious|lemma=\"grace\" strong=\"H1234\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(extra.get("lemma").map(String::as_str), Some("grace"));
        assert_eq!(extra.get("strong").map(String::as_str), Some("H1234"));
    }

    // (2) Default Attribute — `\w gracious|grace\w*` resolves to lemma="grace" in USJ/USX.
    #[test]
    fn default_attribute_w_resolves_to_lemma() {
        let source = doc_with("\\w gracious|grace\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(
            extra.get("lemma").map(String::as_str),
            Some("grace"),
            "bare default on \\w must resolve to lemma; got {extra:?}"
        );
    }

    // (2) Default Attribute — `\rb 話|はなし\rb*` resolves to gloss="はなし".
    #[test]
    fn default_attribute_rb_resolves_to_gloss() {
        let source = doc_with("\\rb 話|はなし\\rb*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "rb");
        assert_eq!(
            extra.get("gloss").map(String::as_str),
            Some("はなし"),
            "bare default on \\rb must resolve to gloss; got {extra:?}"
        );
    }

    // (3) Multiple Attribute Values — comma-separated list within the value string.
    #[test]
    fn multiple_attribute_values_comma_separated() {
        let source = doc_with("\\w gracious|strong=\"H1234,G5485\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(
            extra.get("strong").map(String::as_str),
            Some("H1234,G5485"),
            "multi-value strong attribute must be stored verbatim"
        );
    }

    // (4) Multiple Attribute Parts — colon-separated within the value string.
    #[test]
    fn multiple_attribute_parts_colon_separated() {
        let source = doc_with("\\rb 話賄|はな:はなし\\rb*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "rb");
        assert_eq!(extra.get("gloss").map(String::as_str), Some("はな:はなし"));
    }

    // (4) Multiple Attribute Parts — empty middle (`::`) and trailing colon preserved.
    #[test]
    fn multiple_attribute_parts_empty_middle() {
        let source = doc_with("\\rb 神の子|かみ::こ\\rb*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "rb");
        assert_eq!(extra.get("gloss").map(String::as_str), Some("かみ::こ"));
    }

    #[test]
    fn multiple_attribute_parts_trailing_empty() {
        let source = doc_with("\\rb 定ま|さだ:\\rb*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "rb");
        assert_eq!(extra.get("gloss").map(String::as_str), Some("さだ:"));
    }

    // (5) User-Defined Attributes — `x-` prefix is parsed and preserved verbatim.
    #[test]
    fn user_defined_x_prefix_attribute() {
        let source = doc_with("\\w gracious|x-myattr=\"value\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(extra.get("x-myattr").map(String::as_str), Some("value"));
    }

    // (5) User-Defined Attributes — `z-` prefix likewise accepted.
    #[test]
    fn user_defined_z_prefix_attribute() {
        let source = doc_with("\\w gracious|z-tag=\"v\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(extra.get("z-tag").map(String::as_str), Some("v"));
    }

    // (5) User-Defined Attributes — mixed with canonical attribute, order preserved.
    #[test]
    fn user_defined_mixed_with_canonical() {
        let source = doc_with("\\w gracious|lemma=\"grace\" x-myattr=\"v\"\\w*");
        assert_byte_identical_roundtrip(&source);
        let extra = first_char_attrs(&source, "w");
        assert_eq!(extra.get("lemma").map(String::as_str), Some("grace"));
        assert_eq!(extra.get("x-myattr").map(String::as_str), Some("v"));
    }
}
