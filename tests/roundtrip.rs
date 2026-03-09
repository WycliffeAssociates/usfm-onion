mod common;

use std::fs;
use std::path::Path;

use usfm_onion::parse::{parse, write_exact};

#[test]
fn all_origin_usfm_fixtures_roundtrip_exactly() {
    let root = Path::new("testData");
    let fixtures = common::collect_origin_usfm_fixtures(root);
    let mut failures = Vec::new();

    for fixture in fixtures {
        let slug = common::fixture_slug(root, &fixture);
        common::log_fixture("roundtrip", &fixture);
        let source = fs::read_to_string(&fixture).expect("fixture should be readable");
        let handle = parse(&source);
        let written = write_exact(&handle);
        if written != source {
            failures.push(format!(
                "{slug}: roundtrip mismatch for {}",
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
