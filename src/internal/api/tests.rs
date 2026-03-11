use super::*;
use crate::lint::LintOptions;
use crate::parse::parse;
use crate::{document_tree, tokens};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn into_tokens_preserves_horizontal_whitespace() {
    let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
    let projected = into_tokens(&handle, IntoTokensOptions::default());
    assert!(projected.iter().any(|token| token.text.ends_with("  ")));
}

#[test]
fn into_tokens_can_merge_horizontal_whitespace() {
    let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
    let projected = into_tokens(
        &handle,
        IntoTokensOptions {
            merge_horizontal_whitespace: true,
        },
    );
    let canonical = into_tokens(&handle, IntoTokensOptions::default());
    assert_eq!(projected, canonical);
}

#[test]
fn parse_content_ingests_usfm_usj_and_usx() {
    let usfm = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let canonical = parse(usfm);
    let usj = serde_json::to_string(&into_usj(&canonical)).expect("usj should serialize");
    let usx = into_usx(&canonical).expect("usx should serialize");

    let from_usfm = parse_usfm_content(usfm).expect("parse usfm");
    let from_usj = parse_usj_content(&usj).expect("parse usj");
    let from_usx = parse_usx_content(&usx).expect("parse usx");

    assert_eq!(into_vref(&from_usfm), into_vref(&canonical));
    assert_eq!(into_vref(&from_usj), into_vref(&canonical));
    assert_eq!(into_vref(&from_usx), into_vref(&canonical));
}

#[test]
fn explicit_input_named_wrappers_match_generic_entrypoints() {
    let usfm = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let generic_tokens =
        into_tokens_from_content(usfm, DocumentFormat::Usfm, IntoTokensOptions::default())
            .expect("generic tokens");
    let explicit_tokens =
        into_tokens_from_usfm_content(usfm, IntoTokensOptions::default()).expect("explicit tokens");

    let generic_lint =
        lint_content(usfm, DocumentFormat::Usfm, LintOptions::default()).expect("generic lint");
    let explicit_lint = lint_usfm_content(usfm, LintOptions::default()).expect("explicit lint");

    let generic_format = format_content(usfm, DocumentFormat::Usfm, IntoTokensOptions::default())
        .expect("generic format");
    let explicit_format =
        format_usfm_content(usfm, IntoTokensOptions::default()).expect("explicit format");

    let generic_projection =
        project_content(usfm, DocumentFormat::Usfm, ProjectUsfmOptions::default())
            .expect("generic projection");
    let explicit_projection =
        project_usfm_content(usfm, ProjectUsfmOptions::default()).expect("explicit projection");

    assert_eq!(generic_tokens, explicit_tokens);
    assert_eq!(generic_lint, explicit_lint);
    assert_eq!(generic_format.tokens, explicit_format.tokens);
    assert_eq!(generic_projection.tokens, explicit_projection.tokens);
}

#[test]
fn convert_content_handles_cross_format_roundtrip() {
    let usfm = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usj = convert_content(usfm, DocumentFormat::Usfm, DocumentFormat::Usj)
        .expect("convert usfm to usj");
    let usx = convert_content(usfm, DocumentFormat::Usfm, DocumentFormat::Usx)
        .expect("convert usfm to usx");
    let usfm_from_usj = convert_content(&usj, DocumentFormat::Usj, DocumentFormat::Usfm)
        .expect("convert usj to usfm");
    let usfm_from_usx = convert_content(&usx, DocumentFormat::Usx, DocumentFormat::Usfm)
        .expect("convert usx to usfm");

    let canonical_handle = parse(usfm);
    let usj_handle = parse(&usfm_from_usj);
    let usx_handle = parse(&usfm_from_usx);

    assert_eq!(into_vref(&usj_handle), into_vref(&canonical_handle));
    assert_eq!(into_vref(&usx_handle), into_vref(&canonical_handle));
}

#[test]
fn explicit_conversion_wrappers_match_generic_conversion_helpers() {
    let usfm = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usj = usfm_content_to_usj(usfm).expect("explicit usj");
    let usx = usfm_content_to_usx(usfm).expect("explicit usx");
    let usfm_from_usj = usj_content_to_usfm(&usj).expect("explicit usj->usfm");
    let usfm_from_usx = usx_content_to_usfm(&usx).expect("explicit usx->usfm");

    assert_eq!(
        usj,
        convert_content(usfm, DocumentFormat::Usfm, DocumentFormat::Usj).expect("generic usj")
    );
    assert_eq!(
        usx,
        convert_content(usfm, DocumentFormat::Usfm, DocumentFormat::Usx).expect("generic usx")
    );
    assert_eq!(
        usfm_from_usj,
        convert_content(&usj, DocumentFormat::Usj, DocumentFormat::Usfm)
            .expect("generic usj->usfm")
    );
    assert_eq!(
        usfm_from_usx,
        convert_content(&usx, DocumentFormat::Usx, DocumentFormat::Usfm)
            .expect("generic usx->usfm")
    );
}

#[test]
fn parse_path_supports_non_usfm_formats() {
    let usfm = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let canonical = parse(usfm);
    let usj = serde_json::to_string(&into_usj(&canonical)).expect("usj should serialize");
    let usx = into_usx(&canonical).expect("usx should serialize");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should move forward")
        .as_nanos();
    let dir = std::env::temp_dir();
    let usfm_path = dir.join(format!("usfm_onion_{unique}.usfm"));
    let usj_path = dir.join(format!("usfm_onion_{unique}.json"));
    let usx_path = dir.join(format!("usfm_onion_{unique}.xml"));

    fs::write(&usfm_path, usfm).expect("write usfm fixture");
    fs::write(&usj_path, usj).expect("write usj fixture");
    fs::write(&usx_path, usx).expect("write usx fixture");

    let usfm_handle = parse_path(&usfm_path, DocumentFormat::Usfm).expect("parse usfm file");
    let usj_handle = parse_path(&usj_path, DocumentFormat::Usj).expect("parse usj file");
    let usx_handle = parse_path(&usx_path, DocumentFormat::Usx).expect("parse usx file");

    fs::remove_file(&usfm_path).expect("cleanup usfm fixture");
    fs::remove_file(&usj_path).expect("cleanup usj fixture");
    fs::remove_file(&usx_path).expect("cleanup usx fixture");

    let expected = into_vref(&canonical);
    assert_eq!(into_vref(&usfm_handle), expected);
    assert_eq!(into_vref(&usj_handle), expected);
    assert_eq!(into_vref(&usx_handle), expected);
}

#[test]
fn into_usfm_from_tokens_writes_direct_source_text() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let scan = crate::internal::lexer::lex(source);
    assert_eq!(into_usfm_from_tokens(scan.tokens.as_slice()), source);
}

#[test]
fn token_intake_outputs_are_available_via_usfm_bridge() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let scan = crate::internal::lexer::lex(source);

    let usj = into_usj_from_tokens(scan.tokens.as_slice());
    let usx = into_usx_from_tokens(scan.tokens.as_slice()).expect("usx should serialize");
    let vref = into_vref_from_tokens(scan.tokens.as_slice());

    assert_eq!(usj.doc_type, "USJ");
    assert!(usx.contains("<usx"));
    assert_eq!(
        vref.get("GEN 1:1").map(String::as_str),
        Some("In the beginning")
    );
}

#[test]
fn into_usj_and_into_vref_are_composable() {
    let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
    let usj = into_usj(&handle);
    let vref = into_vref(&handle);

    assert_eq!(usj.doc_type, "USJ");
    assert_eq!(
        vref.get("GEN 1:1").map(String::as_str),
        Some("In the beginning")
    );
}

#[test]
fn into_document_tree_preserves_linebreak_nodes() {
    let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let json = serde_json::to_string(&serialized).expect("editor tree json");

    assert!(json.contains("\"type\":\"linebreak\""));
}

#[test]
fn into_document_tree_preserves_linebreak_between_chapter_and_paragraph() {
    let handle = parse("\\s5\n\\c 1\n\\p\n\\v 1 In the beginning\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");

    assert!(
        content.iter().any(|node| {
            node.get("type").and_then(serde_json::Value::as_str) == Some("chapter")
        })
    );
    assert!(
        content.iter().any(|node| {
            node.get("type").and_then(serde_json::Value::as_str) == Some("linebreak")
        })
    );
}

#[test]
fn into_document_tree_preserves_space_after_verse_number() {
    let handle = parse("\\c 1\n\\p\n\\v 1 In the beginning\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let para = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
        .expect("paragraph node");
    let para_content = para
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("paragraph content");
    let verse_index = para_content
        .iter()
        .position(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
        .expect("verse node");
    let following_text = para_content
        .iter()
        .skip(verse_index + 1)
        .find_map(text_node_value)
        .expect("text after verse");

    assert!(following_text.starts_with(' '));
}

#[test]
fn into_document_tree_preserves_double_spaces_in_text_nodes() {
    let handle = parse("\\c 1\n\\p\n\\v 1 I will give  the inhabitants.\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let para = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
        .expect("paragraph node");
    let para_content = para
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("paragraph content");
    let verse_index = para_content
        .iter()
        .position(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
        .expect("verse node");
    let following_text = para_content
        .iter()
        .skip(verse_index + 1)
        .find_map(text_node_value)
        .expect("text after verse");

    assert!(
        following_text.contains("give  the"),
        "expected double space to be preserved in editor tree text"
    );
}

#[test]
fn into_document_tree_preserves_space_after_book_code() {
    let handle = parse("\\id GEN Unlocked Literal Bible\n\\c 1\n\\p\n\\v 1 In the beginning\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let book = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("book"))
        .expect("book node");
    let book_content = book
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("book content");
    let first_text = book_content
        .iter()
        .find_map(text_node_value)
        .expect("first text child");

    assert!(first_text.starts_with(' '));
}

#[test]
fn into_document_tree_uses_zero_verse_sid_for_chapters() {
    let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let chapter = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("chapter"))
        .expect("chapter node");

    assert_eq!(
        chapter.get("sid").and_then(serde_json::Value::as_str),
        Some("GEN 1:0")
    );
}

fn text_node_value(node: &serde_json::Value) -> Option<&str> {
    node.get("type")
        .and_then(serde_json::Value::as_str)
        .filter(|node_type| *node_type == "text")
        .and_then(|_| node.get("value"))
        .and_then(serde_json::Value::as_str)
}

#[test]
fn into_document_tree_preserves_explicit_note_char_closures() {
    let handle = parse("\\c 1\n\\p\n\\v 26 text\\f + \\ft intro \\fqa quote\\fqa* \\f*\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let para = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
        .expect("paragraph node");
    let note = para
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("paragraph content")
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("note"))
        .expect("note node");
    let fqa = note
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("note content")
        .iter()
        .find(|node| node.get("marker").and_then(serde_json::Value::as_str) == Some("fqa"))
        .expect("fqa node");

    assert_eq!(
        fqa.get("closed").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        fqa.get("closeSuffix").and_then(serde_json::Value::as_str),
        Some(" ")
    );
}

#[test]
fn into_document_tree_preserves_exact_paragraph_marker_text() {
    let handle = parse("\\m(for fine linen is righteous)\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");
    let para = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
        .expect("paragraph node");

    assert_eq!(
        para.get("markerText").and_then(serde_json::Value::as_str),
        Some("\\m")
    );
}

#[test]
fn into_document_tree_preserves_marker_text_for_book_chapter_and_verse() {
    let handle = parse("\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning.\n");
    let tree = into_document_tree(&handle);
    let serialized = serde_json::to_value(&tree).expect("editor tree should serialize");
    let content = serialized
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("root content array");

    let book = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("book"))
        .expect("book node");
    assert_eq!(
        book.get("markerText").and_then(serde_json::Value::as_str),
        Some("\\id ")
    );

    let chapter = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("chapter"))
        .expect("chapter node");
    assert_eq!(
        chapter
            .get("markerText")
            .and_then(serde_json::Value::as_str),
        Some("\\c ")
    );

    let para = content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("para"))
        .expect("paragraph node");
    let para_content = para
        .get("content")
        .and_then(serde_json::Value::as_array)
        .expect("paragraph content");
    let verse = para_content
        .iter()
        .find(|node| node.get("type").and_then(serde_json::Value::as_str) == Some("verse"))
        .expect("verse node");
    assert_eq!(
        verse.get("markerText").and_then(serde_json::Value::as_str),
        Some("\\v ")
    );
}

#[test]
fn project_usfm_can_return_tokens_tree_and_lint_from_one_parse() {
    let projection = project_usfm(
        "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n",
        ProjectUsfmOptions {
            token_options: IntoTokensOptions {
                merge_horizontal_whitespace: true,
            },
            lint_options: Some(LintOptions::default()),
        },
    );

    assert_eq!(projection.document_tree.doc_type, "USJ");
    assert!(!projection.tokens.is_empty());
    assert!(projection.lint_issues.is_some());
}

#[test]
fn format_then_project_preserves_id_separator_and_book_sids() {
    let source = "\\id GEN Unlocked Literal Bible\n\\ide UTF-8\n\\h Genesis\n\\toc1 The Book of Genesis\n\\toc2 Genesis\n\\toc3 Gen\n\\mt Genesis\n\n\\s5\n\\c 1\n\\p\n\\v 1 In the beginning.\n";
    let handle = parse(source);
    let baseline = into_tokens(
        &handle,
        IntoTokensOptions {
            merge_horizontal_whitespace: true,
        },
    );
    let formatted = format_flat_tokens(&baseline);
    let formatted_usfm = formatted
        .tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>();
    assert!(formatted_usfm.starts_with("\\id GEN Unlocked Literal Bible\n"));

    let projection = project_usfm(
        formatted_usfm.as_str(),
        ProjectUsfmOptions {
            token_options: IntoTokensOptions {
                merge_horizontal_whitespace: true,
            },
            lint_options: None,
        },
    );
    let projected_usfm = projection
        .tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>();

    assert!(projected_usfm.starts_with("\\id GEN Unlocked Literal Bible\n"));
    assert!(
        projection
            .tokens
            .iter()
            .filter_map(|token| token.sid.as_deref())
            .any(|sid| sid.starts_with("GEN ")),
        "expected at least one GEN SID after format/project"
    );
    assert!(
        projection
            .tokens
            .iter()
            .filter_map(|token| token.sid.as_deref())
            .all(|sid| !sid.starts_with(' ')),
        "unexpected blank book-code SID emitted"
    );
}

#[test]
fn push_whitespace_matches_flat_projection_policy() {
    let handle = parse("\\id GEN\n\\c 1\n\\p  \n\\v 1 In the beginning\n");
    let preserved = into_tokens(&handle, IntoTokensOptions::default());
    let merged = push_whitespace(&preserved);
    let flat = into_tokens(
        &handle,
        IntoTokensOptions {
            merge_horizontal_whitespace: true,
        },
    );

    let merged_shape = merged
        .iter()
        .map(|token| {
            (
                token.kind.clone(),
                token.span.clone(),
                token.sid.clone(),
                token.marker.clone(),
                token.text.clone(),
            )
        })
        .collect::<Vec<_>>();
    let flat_shape = flat
        .iter()
        .map(|token| {
            (
                token.kind.clone(),
                token.span.clone(),
                token.sid.clone(),
                token.marker.clone(),
                token.text.clone(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(merged_shape, flat_shape);
}
