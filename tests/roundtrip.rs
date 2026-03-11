mod common;

use std::fs;
use std::path::Path;

use usfm_onion::tokens;

#[test]
fn all_origin_usfm_fixtures_roundtrip_exactly_through_tokens() {
    let root = Path::new("testData");
    let fixtures = common::collect_origin_usfm_fixtures(root);
    let mut failures = Vec::new();

    for fixture in fixtures {
        let slug = common::fixture_slug(root, &fixture);
        common::log_fixture("roundtrip", &fixture);
        let source = fs::read_to_string(&fixture).expect("fixture should be readable");

        let token_list = tokens::usfm_to_tokens(&source);
        let written_from_tokens = tokens::tokens_to_usfm(&token_list);
        if written_from_tokens != source {
            failures.push(format!(
                "{slug}: token roundtrip mismatch for {}",
                fixture.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "roundtrip fixture failures:\n{}",
        failures.join("\n")
    );
}
