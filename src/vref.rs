use std::collections::BTreeMap;

use crate::marker_defs::marker_paragraph_supports_verse;
use crate::parse::parse;
use crate::token::{NumberRangeKind, Sid, Token, TokenData};

pub type VrefMap = BTreeMap<String, String>;

#[derive(Debug, Default)]
struct VrefState {
    current_ref: Option<String>,
    current_text: String,
    current_block_supports_verse: Option<bool>,
    open_note_markers: Vec<String>,
}

pub fn usfm_to_vref_map(source: &str) -> VrefMap {
    let parsed = parse(source);
    tokens_to_vref_map(&parsed.tokens)
}

pub fn tokens_to_vref_map(tokens: &[Token<'_>]) -> VrefMap {
    let mut map = VrefMap::new();
    let mut state = VrefState::default();
    let mut pending_marker_name: Option<&str> = None;

    for token in tokens {
        match &token.data {
            TokenData::Marker {
                name,
                structural,
                nested: _,
                ..
            } => {
                pending_marker_name = Some(name);

                if structural.scope_kind == crate::marker_defs::StructuralScopeKind::Note {
                    state.open_note_markers.push((*name).to_string());
                }

                if structural.scope_kind == crate::marker_defs::StructuralScopeKind::Block {
                    state.current_block_supports_verse =
                        Some(marker_paragraph_supports_verse(name));
                }
            }
            TokenData::EndMarker {
                name, structural, ..
            } => {
                pending_marker_name = None;

                if structural.scope_kind == crate::marker_defs::StructuralScopeKind::Note {
                    pop_matching_note(&mut state.open_note_markers, name);
                }
            }
            TokenData::Number { start, kind, .. } => {
                match pending_marker_name {
                    Some("c") => {
                        clear_current_verse(&mut state, &mut map);
                    }
                    Some("v") => {
                        clear_current_verse(&mut state, &mut map);
                        if let Some(reference) = verse_ref(token.sid, *start, *kind) {
                            state.current_ref = Some(reference);
                        }
                    }
                    _ => {}
                }
                pending_marker_name = None;
            }
            TokenData::Text => {
                pending_marker_name = None;
                if can_collect_text(&state) {
                    push_text(&mut state.current_text, token.source);
                }
            }
            TokenData::Newline
            | TokenData::OptBreak
            | TokenData::BookCode { .. }
            | TokenData::Milestone { .. }
            | TokenData::MilestoneEnd
            | TokenData::AttributeList { .. } => {
                pending_marker_name = None;
            }
        }
    }

    clear_current_verse(&mut state, &mut map);
    map
}

pub fn vref_map_to_json_string(map: &VrefMap) -> String {
    serde_json::to_string_pretty(map).expect("vref map should serialize")
}

fn can_collect_text(state: &VrefState) -> bool {
    state.current_ref.is_some()
        && state.open_note_markers.is_empty()
        && state.current_block_supports_verse.unwrap_or(true)
}

fn verse_ref(sid: Option<Sid<'_>>, number_start: u32, _kind: NumberRangeKind) -> Option<String> {
    let sid = sid?;
    if sid.book_code.is_empty() {
        return None;
    }

    let chapter = sid.chapter;
    let verse = number_start;

    if chapter == 0 || verse == 0 {
        return None;
    }

    Some(format!("{} {}:{}", sid.book_code, chapter, verse))
}

fn clear_current_verse(state: &mut VrefState, map: &mut VrefMap) {
    let Some(reference) = state.current_ref.take() else {
        state.current_text.clear();
        return;
    };

    let trimmed = state.current_text.trim();
    if !trimmed.is_empty() {
        map.insert(reference, trimmed.to_string());
    }
    state.current_text.clear();
}

fn push_text(current: &mut String, fragment: &str) {
    if current.is_empty() {
        current.push_str(fragment);
        return;
    }

    let current_ends_with_ws = current.chars().last().is_some_and(char::is_whitespace);
    let fragment_starts_with_ws = fragment.chars().next().is_some_and(char::is_whitespace);

    if current_ends_with_ws && fragment_starts_with_ws {
        current.push_str(fragment.trim_start());
    } else {
        current.push_str(fragment);
    }
}

fn pop_matching_note(stack: &mut Vec<String>, name: &str) {
    let Some(index) = stack.iter().rposition(|open| open == name) else {
        return;
    };
    stack.truncate(index);
}

#[cfg(test)]
mod tests {
    use super::{tokens_to_vref_map, usfm_to_vref_map, vref_map_to_json_string};
    use crate::parse::parse;

    #[test]
    fn basic_vref_extracts_plain_verse_text() {
        let map = usfm_to_vref_map(
            "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n",
        );

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning God created the heavens and the earth.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("The earth was without form and void.")
        );
    }

    #[test]
    fn footnotes_are_stripped() {
        let map = usfm_to_vref_map(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 Text \\f + \\fr 1:1 \\ft note text \\f* rest.",
        );
        let verse = map.get("GEN 1:1").map(String::as_str).unwrap_or("");

        assert!(verse.contains("Text"));
        assert!(verse.contains("rest."));
        assert!(!verse.contains("note text"));
    }

    #[test]
    fn section_headings_are_skipped() {
        let map =
            usfm_to_vref_map("\\id GEN\n\\c 1\n\\s1 The Creation\n\\p\n\\v 1 In the beginning.");

        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
    }

    #[test]
    fn verse_spanning_paragraphs_is_concatenated() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 First part.\n\\q1 Second part.");

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("First part. Second part.")
        );
    }

    #[test]
    fn root_level_verses_are_collected() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 In the beginning.\n\\v 2 And God said.");
        let map = tokens_to_vref_map(&parsed.tokens);

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("And God said.")
        );
    }

    #[test]
    fn json_output_contains_refs_and_text() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
        let json = vref_map_to_json_string(&map);

        assert!(json.contains("\"GEN 1:1\""));
        assert!(json.contains("\"In the beginning.\""));
    }
}
