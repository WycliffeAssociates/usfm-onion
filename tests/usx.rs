mod common;

use std::fs;
use std::path::Path;

use usfm_onion::{convert, tokens};

#[test]
fn usfm_to_usx_emits_usx_root() {
    let source = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usx = convert::usfm_to_usx(source).expect("USX should serialize");

    assert!(usx.contains("<usx"));
    assert!(usx.contains("<book"));
}

#[test]
fn usx_reimports_semantically_via_tokens() {
    let source = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usx = convert::usfm_to_usx(source).expect("USX should serialize");
    let usfm = convert::from_usx_str(&usx).expect("USX should import");

    assert_eq!(tokens::tokens_to_usfm(&tokens::usfm_to_tokens(&usfm)), usfm);
}

#[test]
fn usx_reimports_semantically_for_all_origin_usfm_fixtures() {
    let root = Path::new("testData");
    let mut failures = Vec::new();

    for usfm_path in common::collect_origin_usfm_fixtures(root) {
        common::log_fixture("usx", &usfm_path);

        let source = fs::read_to_string(&usfm_path).expect("fixture usfm should read");
        let actual = convert::usfm_to_usx(&source).expect("USX should serialize");
        let regenerated = convert::from_usx_str(&actual).expect("USX should import");

        if tokens::tokens_to_usfm(&tokens::usfm_to_tokens(&regenerated)) != regenerated {
            failures.push(common::fixture_slug(root, &usfm_path));
        }
    }

    assert!(
        failures.is_empty(),
        "USX semantic reimport failures:\n{}",
        failures.join("\n")
    );
}
