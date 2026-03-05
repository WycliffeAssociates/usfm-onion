use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use usfm3_v2::{parse, to_usj_value};

fn main() {
    let mut args = env::args().skip(1);
    let usfm = PathBuf::from(args.next().expect("expected origin.usfm path"));
    let json = PathBuf::from(args.next().expect("expected origin.json path"));

    let source = fs::read_to_string(&usfm).expect("failed to read usfm");
    let expected_str = fs::read_to_string(&json).expect("failed to read json");
    let expected: Value = serde_json::from_str(&expected_str).expect("failed to parse json");
    let actual = to_usj_value(&parse(&source));

    match first_diff(&expected, &actual, "$".to_string()) {
        Some(diff) => println!("{diff}"),
        None => println!("no diff"),
    }
}

fn first_diff(expected: &Value, actual: &Value, path: String) -> Option<String> {
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            for key in left.keys() {
                if !right.contains_key(key) {
                    return Some(format!("{path}.{key}: missing in actual"));
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
                .trim_start_matches(|ch: char| matches!(ch, ' ' | '\n' | '\r' | '\t'))
                .to_string();
        }
        if next_is_object {
            next_text = next_text
                .trim_end_matches(|ch: char| matches!(ch, ' ' | '\n' | '\r' | '\t'))
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

fn trim_last_descendant_string_end(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = text
                .trim_end_matches(|ch: char| matches!(ch, ' ' | '\n' | '\r' | '\t'))
                .to_string();
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
            *text = text
                .trim_start_matches(|ch: char| matches!(ch, ' ' | '\n' | '\r' | '\t'))
                .to_string();
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
