mod common;

use std::fs;
use std::path::Path;

use usfm3_v2::{RawTokenKind, lex};

#[test]
fn scanner_covers_and_reconstructs_all_origin_usfm_fixtures() {
    let root = Path::new("testData");
    let fixtures = common::collect_origin_usfm_fixtures(root);
    let mut failures = Vec::new();

    for fixture in fixtures {
        let slug = common::fixture_slug(root, &fixture);
        common::log_fixture("scanner", &fixture);
        let source = fs::read_to_string(&fixture).expect("fixture should be readable");
        let result = lex(&source);

        if result.tokens.is_empty() && !source.is_empty() {
            failures.push(format!(
                "{slug}: scanner returned no tokens for non-empty source"
            ));
            continue;
        }

        let mut reconstructed = String::new();
        let mut expected_start = 0usize;

        for (index, token) in result.tokens.iter().enumerate() {
            if token.span.start != expected_start {
                failures.push(format!(
                    "{slug}: token {index} starts at {}, expected {}",
                    token.span.start, expected_start
                ));
                break;
            }

            if token.span.end < token.span.start {
                failures.push(format!(
                    "{slug}: token {index} has invalid span {:?}",
                    token.span
                ));
                break;
            }

            let slice = &source[token.span.clone()];
            if slice != token.text {
                failures.push(format!(
                    "{slug}: token {index} text/span mismatch, span {:?}",
                    token.span
                ));
                break;
            }

            if token.kind == RawTokenKind::Text && token.text.is_empty() {
                failures.push(format!("{slug}: token {index} is empty text"));
                break;
            }

            reconstructed.push_str(&token.text);
            expected_start = token.span.end;
        }

        if expected_start != source.len() {
            failures.push(format!(
                "{slug}: scanner ended at byte {}, expected {}",
                expected_start,
                source.len()
            ));
        }

        if reconstructed != source {
            failures.push(format!(
                "{slug}: reconstructed source differs from original"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "scanner fixture failures:\n{}",
        failures.join("\n")
    );
}
