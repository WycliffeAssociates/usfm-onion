mod common;

use std::fs;
use std::path::Path;

use usfm_onion::tokens;

#[test]
fn all_origin_usfm_fixtures_tokenize_to_contiguous_roundtrippable_tokens() {
    let root = Path::new("testData");
    let fixtures = common::collect_origin_usfm_fixtures(root);
    let mut failures = Vec::new();

    for fixture in fixtures {
        let slug = common::fixture_slug(root, &fixture);
        common::log_fixture("tokens", &fixture);
        let source = fs::read_to_string(&fixture).expect("fixture should be readable");
        let token_list = tokens::usfm_to_tokens(&source);

        if token_list.is_empty() && !source.is_empty() {
            failures.push(format!(
                "{slug}: tokenizer returned no tokens for non-empty source"
            ));
            continue;
        }

        let mut expected_start = 0usize;
        let mut reconstructed = String::new();
        for (index, token) in token_list.iter().enumerate() {
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

            reconstructed.push_str(&token.text);
            expected_start = token.span.end;
        }

        if expected_start != source.len() {
            failures.push(format!(
                "{slug}: tokenizer ended at char {}, expected {}",
                expected_start,
                source.len()
            ));
        }

        if reconstructed != source {
            failures.push(format!(
                "{slug}: token reconstruction differs from original source"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "token fixture failures:\n{}",
        failures.join("\n")
    );
}
