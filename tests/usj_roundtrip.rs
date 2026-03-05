mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{collect_origin_usfm_fixtures, fixture_slug, log_fixture};
use usfm3_v2::{from_usj_value, parse, to_usj_roundtrip_value};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testData")
}

#[test]
fn usfm_usj_usfm_matches_origin_disk_strings() {
    let root = fixture_root();
    let selected = std::env::var("USFM3_V2_USJ_ROUNDTRIP_FIXTURE").ok();

    let fixtures: Vec<PathBuf> = if let Some(selected_slug) = selected.as_deref() {
        collect_origin_usfm_fixtures(&root)
            .into_iter()
            .filter(|origin| {
                let slug = fixture_slug(&root, &origin);
                slug == selected_slug
            })
            .collect()
    } else {
        collect_origin_usfm_fixtures(&root)
    };

    if fixtures.is_empty() {
        panic!("no USJ round-trip fixtures selected");
    }

    let mut failures = Vec::new();

    for origin in fixtures {
        let slug = fixture_slug(&root, &origin);
        log_fixture("usj-roundtrip", &origin);

        let source = fs::read_to_string(&origin).expect("origin fixture should be readable");
        let handle = parse(&source);
        let usj = to_usj_roundtrip_value(&handle);
        let regenerated_usfm = from_usj_value(&usj).expect("generated USJ should serialize back to USFM");

        if regenerated_usfm != source {
            failures.push(format!(
                "{slug}\nfixture: {}\n{}",
                origin.display(),
                first_difference_summary(&source, &regenerated_usfm)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "USFM <- USJ string mismatches ({} fixtures)\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

fn first_difference_summary(expected: &str, actual: &str) -> String {
    let mut expected_iter = expected.char_indices();
    let mut actual_iter = actual.char_indices();

    loop {
        match (expected_iter.next(), actual_iter.next()) {
            (Some((expected_idx, expected_char)), Some((actual_idx, actual_char))) => {
                if expected_char != actual_char {
                    let expected_snippet = snippet_from(expected, expected_idx);
                    let actual_snippet = snippet_from(actual, actual_idx);
                    return format!(
                        "first diff at byte {expected_idx}\nexpected: {:?}\nactual:   {:?}",
                        expected_snippet, actual_snippet
                    );
                }
            }
            (Some((expected_idx, _)), None) => {
                return format!(
                    "actual ended early at byte {expected_idx}\nexpected: {:?}\nactual:   {:?}",
                    snippet_from(expected, expected_idx),
                    ""
                );
            }
            (None, Some((actual_idx, _))) => {
                return format!(
                    "actual has extra content at byte {actual_idx}\nexpected: {:?}\nactual:   {:?}",
                    "",
                    snippet_from(actual, actual_idx)
                );
            }
            (None, None) => return "strings differ, but no byte diff was found".to_string(),
        }
    }
}

fn snippet_from(text: &str, start: usize) -> String {
    text[start..].chars().take(80).collect()
}
