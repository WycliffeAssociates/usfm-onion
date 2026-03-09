mod common;

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use usfm_onion::{advanced::to_usj_value, parse::parse};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testData")
}

fn first_diff(expected: &Value, actual: &Value, path: String) -> Option<String> {
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            if left.len() != right.len() {
                return Some(format!(
                    "{path}: object key count differs expected={} actual={}",
                    left.len(),
                    right.len()
                ));
            }
            for key in left.keys() {
                if !right.contains_key(key) {
                    return Some(format!("{path}.{key}: missing in actual"));
                }
            }
            for key in right.keys() {
                if !left.contains_key(key) {
                    return Some(format!("{path}.{key}: unexpected key in actual"));
                }
            }
            for key in left.keys() {
                let next_path = format!("{path}.{key}");
                if let Some(diff) = first_diff(&left[key], &right[key], next_path) {
                    return Some(diff);
                }
            }
            None
        }
        (Value::Array(left), Value::Array(right)) => {
            let left = normalize_array_boundary_whitespace(left);
            let right = normalize_array_boundary_whitespace(right);
            if left.len() != right.len() {
                return Some(format!(
                    "{path}: array length differs expected={} actual={}",
                    left.len(),
                    right.len()
                ));
            }
            for (index, (left_item, right_item)) in left.iter().zip(right.iter()).enumerate() {
                let next_path = format!("{path}[{index}]");
                if let Some(diff) = first_diff(left_item, right_item, next_path) {
                    return Some(diff);
                }
            }
            None
        }
        (Value::String(left), Value::String(right))
            if path == "$.version" && left == "3.0" && right == "3.1" =>
        {
            None
        }
        (Value::String(left), Value::String(right))
            if normalize_compare_string(left) == normalize_compare_string(right) =>
        {
            None
        }
        _ if expected == actual => None,
        _ => Some(format!(
            "{path}: expected={} actual={}",
            to_compact(expected),
            to_compact(actual)
        )),
    }
}

fn to_compact(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
}

fn normalize_compare_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous_was_whitespace = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !previous_was_whitespace {
                out.push(' ');
            }
            previous_was_whitespace = true;
        } else {
            previous_was_whitespace = false;
            out.push(ch);
        }
    }
    out.trim().to_string()
}

fn normalize_array_boundary_whitespace(values: &[Value]) -> Vec<Value> {
    let mut normalized = values
        .iter()
        .map(normalize_value_boundary_whitespace)
        .collect::<Vec<_>>();

    for index in 0..normalized.len() {
        let prev_is_object = index > 0 && normalized[index - 1].is_object();
        let next_is_object = index + 1 < normalized.len() && normalized[index + 1].is_object();
        let Some(text) = normalized[index].as_str() else {
            continue;
        };

        let mut next_text = text.to_string();
        if prev_is_object {
            next_text = next_text
                .trim_start_matches([' ', '\n', '\r', '\t'])
                .to_string();
        }
        if next_is_object {
            next_text = next_text
                .trim_end_matches([' ', '\n', '\r', '\t'])
                .to_string();
        }
        normalized[index] = Value::String(next_text);
    }

    for index in 0..normalized.len().saturating_sub(1) {
        if normalized[index].is_object() && normalized[index + 1].is_object() {
            trim_last_descendant_string_end(&mut normalized[index]);
            trim_first_descendant_string_start(&mut normalized[index + 1]);
        }
    }

    normalized
}

fn normalize_value_boundary_whitespace(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(normalize_array_boundary_whitespace(values)),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), normalize_value_boundary_whitespace(value)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn trim_last_descendant_string_end(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = text.trim_end_matches([' ', '\n', '\r', '\t']).to_string();
        }
        Value::Array(values) => {
            if let Some(last) = values.last_mut() {
                trim_last_descendant_string_end(last);
            }
        }
        Value::Object(map) => {
            if let Some(content) = map.get_mut("content") {
                trim_last_descendant_string_end(content);
            }
        }
        _ => {}
    }
}

fn trim_first_descendant_string_start(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = text.trim_start_matches([' ', '\n', '\r', '\t']).to_string();
        }
        Value::Array(values) => {
            if let Some(first) = values.first_mut() {
                trim_first_descendant_string_start(first);
            }
        }
        Value::Object(map) => {
            if let Some(content) = map.get_mut("content") {
                trim_first_descendant_string_start(content);
            }
        }
        _ => {}
    }
}

#[test]
fn usj_matches_origin_json_fixtures() {
    let root = fixture_root();
    let filter = std::env::var("USFM_ONION_USJ_FIXTURE").ok();
    let include_exceptions = matches!(
        std::env::var("USFM_ONION_USJ_INCLUDE_EXCEPTIONS")
            .ok()
            .as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    );
    let mut fixtures = common::collect_origin_usfm_json_pairs(&root);
    if !include_exceptions {
        fixtures.retain(|(usfm, json)| {
            !is_explicit_exception_fixture(&root, usfm)
                && !is_explicit_exception_fixture(&root, json)
        });
    }
    if let Some(filter) = filter.as_deref() {
        fixtures.retain(|(usfm, json)| {
            let usfm_slug = common::fixture_slug(&root, usfm);
            let json_slug = common::fixture_slug(&root, json);
            usfm_slug.contains(filter) || json_slug.contains(filter)
        });
    }
    assert!(
        !fixtures.is_empty(),
        "expected at least one origin.usfm/origin.json fixture pair"
    );

    let mut failures = Vec::new();
    let mut exercised = 0usize;

    for (usfm_path, json_path) in fixtures {
        common::log_fixture("usj", &usfm_path);

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

        let expected_str = match fs::read_to_string(&json_path) {
            Ok(expected) => expected,
            Err(error) => {
                failures.push(format!(
                    "{} (read json failed: {error})",
                    common::fixture_slug(&root, &json_path)
                ));
                continue;
            }
        };

        let expected: Value = match serde_json::from_str(&expected_str) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!(
                    "{} (parse expected json failed: {error})",
                    common::fixture_slug(&root, &json_path)
                ));
                continue;
            }
        };

        let actual = to_usj_value(&parse(&source));
        exercised += 1;
        if let Some(diff) = first_diff(&expected, &actual, "$".to_string()) {
            if filter.is_some() {
                let expected_pretty = serde_json::to_string_pretty(&expected)
                    .unwrap_or_else(|_| expected_str.clone());
                let actual_pretty = serde_json::to_string_pretty(&actual)
                    .unwrap_or_else(|_| String::from("<failed to serialize actual>"));
                panic!(
                    "USJ mismatch for {}\n{}\nexpected:\n{}\n\nactual:\n{}",
                    common::fixture_slug(&root, &usfm_path),
                    diff,
                    expected_pretty,
                    actual_pretty
                );
            }
            failures.push(common::fixture_slug(&root, &usfm_path));
        }
    }

    assert!(
        exercised > 0,
        "no USJ fixtures exercised after ignore filtering"
    );

    assert!(
        failures.is_empty(),
        "USJ parity failures:\n{}",
        failures.join("\n")
    );
}

fn is_explicit_exception_fixture(root: &std::path::Path, fixture: &std::path::Path) -> bool {
    let slug = common::fixture_slug(root, fixture);
    matches!(
        slug.as_str(),
        "usfmjsTests_57-TIT_greek_oldformat_origin_usfm"
            | "usfmjsTests_57-TIT_greek_oldformat_origin_json"
            | "usfmjsTests_1ch_verse_span_origin_usfm"
            | "usfmjsTests_1ch_verse_span_origin_json"
            | "usfmjsTests_acts_1_milestone_origin_usfm"
            | "usfmjsTests_acts_1_milestone_origin_json"
            | "usfmjsTests_gn_headers_origin_usfm"
            | "usfmjsTests_gn_headers_origin_json"
            | "usfmjsTests_isa_inline_quotes_origin_usfm"
            | "usfmjsTests_isa_inline_quotes_origin_json"
            | "usfmjsTests_job_footnote_origin_usfm"
            | "usfmjsTests_job_footnote_origin_json"
            | "usfmjsTests_luk_quotes_origin_usfm"
            | "usfmjsTests_luk_quotes_origin_json"
            | "usfmjsTests_pro_footnote_origin_usfm"
            | "usfmjsTests_pro_footnote_origin_json"
            | "usfmjsTests_pro_quotes_origin_usfm"
            | "usfmjsTests_pro_quotes_origin_json"
            | "usfmjsTests_tit_extra_space_after_chapter_origin_usfm"
            | "usfmjsTests_tit_extra_space_after_chapter_origin_json"
            | "usfmjsTests_usfm-body-testF_origin_usfm"
            | "usfmjsTests_usfm-body-testF_origin_json"
            | "usfmjsTests_usfmBodyTestD_origin_usfm"
            | "usfmjsTests_usfmBodyTestD_origin_json"
            | "usfmjsTests_usfmIntroTest_origin_usfm"
            | "usfmjsTests_usfmIntroTest_origin_json"
    )
}
