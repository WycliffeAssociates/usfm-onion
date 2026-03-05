mod common;

use std::fs;
use std::path::PathBuf;

use usfm3_v2::{from_usx_string, parse, to_usx_string};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testData")
}

#[test]
fn usx_roundtrip_matches_origin_for_targeted_fixture() {
    let Some(filter) = std::env::var("USFM3_V2_USX_ROUNDTRIP_FIXTURE").ok() else {
        eprintln!("set USFM3_V2_USX_ROUNDTRIP_FIXTURE to run targeted USX roundtrip");
        return;
    };

    let root = fixture_root();
    let mut fixtures = common::collect_origin_usfm_xml_pairs(&root);
    fixtures.retain(|(usfm, xml)| {
        let usfm_slug = common::fixture_slug(&root, usfm);
        let xml_slug = common::fixture_slug(&root, xml);
        usfm_slug.contains(filter.as_str()) || xml_slug.contains(filter.as_str())
    });

    assert!(
        !fixtures.is_empty(),
        "no origin.usfm/origin.xml pair matched {filter}"
    );

    let mut failures = Vec::new();

    for (usfm_path, _xml_path) in fixtures {
        common::log_fixture("usx-roundtrip", &usfm_path);

        let source = fs::read_to_string(&usfm_path).expect("fixture usfm should read");
        let xml = to_usx_string(&parse(&source)).expect("USX should serialize");
        let regenerated = from_usx_string(&xml).expect("USX should import");

        if regenerated != source {
            failures.push(format!(
                "{}\nexpected:\n{}\nactual:\n{}",
                common::fixture_slug(&root, &usfm_path),
                source,
                regenerated
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "USX roundtrip mismatches:\n{}",
        failures.join("\n\n")
    );
}
