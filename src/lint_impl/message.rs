use std::collections::BTreeMap;

pub type MessageParams = BTreeMap<String, String>;

pub(super) fn message_params<const N: usize>(pairs: [(&str, String); N]) -> MessageParams {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

/// Render an ICU MessageFormat template against `params`. Supports:
/// - `{name}` and `{name, number}` — substitutes `params[name]`.
/// - `{name, select, key1 {…} key2 {…} other {…}}` — branches on
///   `params[name]`, falling through to `other {…}` when no key matches.
///   Literal `{` / `}` are not currently escapable; templates avoid them.
///
/// Localisers can replace this implementation with a full
/// ICU-MessageFormat-compatible renderer without touching emit sites —
/// each `LintIssue` carries both the canonical `template` and the
/// already-populated `message_params`.
pub fn render_template(template: &str, params: &MessageParams) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let (rendered, advance) = render_placeholder(&template[i..], params);
            out.push_str(&rendered);
            i += advance;
        } else {
            // Walk to the next `{` or end. Push the literal span as-is.
            let start = i;
            while i < bytes.len() && bytes[i] != b'{' {
                i += 1;
            }
            out.push_str(&template[start..i]);
        }
    }
    out
}

/// Parse a single `{...}` placeholder starting at offset 0 of `s`.
/// Returns the rendered text and the number of bytes consumed
/// (including the surrounding braces).
fn render_placeholder(s: &str, params: &MessageParams) -> (String, usize) {
    debug_assert!(s.starts_with('{'));
    let close = match find_matching_brace(s, 0) {
        Some(idx) => idx,
        None => return (s.to_string(), s.len()),
    };
    let inner = &s[1..close];
    let consumed = close + 1;

    // Split off the name (everything up to first comma at depth 0).
    let comma = find_top_level_comma(inner);
    let name = match comma {
        Some(c) => inner[..c].trim(),
        None => inner.trim(),
    };
    let rest = comma.map(|c| inner[c + 1..].trim()).unwrap_or("");

    if rest.is_empty() || rest.starts_with("number") {
        // Plain or number substitution: look up params[name].
        let value = params.get(name).cloned().unwrap_or_default();
        return (value, consumed);
    }

    if let Some(after_select) = rest.strip_prefix("select") {
        let after_select = after_select.trim_start_matches(|c: char| c == ',' || c.is_whitespace());
        let value = params.get(name).map(String::as_str).unwrap_or("");
        let arm = pick_select_arm(after_select, value);
        return (render_template(&arm, params), consumed);
    }

    // Unknown format — leave the placeholder as-is so missing renderer
    // features are visible in output rather than silently dropped.
    (s[..consumed].to_string(), consumed)
}

fn find_matching_brace(s: &str, open_idx: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    debug_assert_eq!(bytes[open_idx], b'{');
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_top_level_comma(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Given the body of a `select` (`key1 {arm1} key2 {arm2} other {default}`),
/// return the arm matching `value`, falling back to `other` and then
/// to an empty string.
fn pick_select_arm(body: &str, value: &str) -> String {
    let mut i = 0usize;
    let mut other: Option<String> = None;
    let bytes = body.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Read key up to whitespace or `{`.
        let key_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'{' {
            i += 1;
        }
        let key = &body[key_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'{' {
            break;
        }
        let close = match find_matching_brace(&body[i..], 0) {
            Some(c) => i + c,
            None => break,
        };
        let arm = body[i + 1..close].to_string();
        i = close + 1;
        if key == value {
            return arm;
        }
        if key == "other" {
            other = Some(arm);
        }
    }
    other.unwrap_or_default()
}
