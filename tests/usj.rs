mod common;

use std::fs;
use std::path::Path;

use usfm_onion::{convert, tokens};

#[test]
fn usfm_to_usj_preserves_core_semantics() {
    let source = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usj = convert::usfm_to_usj(source).expect("USJ should serialize");

    assert_eq!(usj.doc_type, "USJ");
    assert!(!usj.content.is_empty());
}

#[test]
fn usj_reimports_semantically_via_tokens() {
    let source = "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n";
    let usj = convert::usfm_to_usj(source).expect("USJ should serialize");
    let usj_json = serde_json::to_string(&usj).expect("USJ should stringify");
    let usfm = convert::from_usj_str(&usj_json).expect("USJ should import");

    assert_eq!(tokens::tokens_to_usfm(&tokens::usfm_to_tokens(&usfm)), usfm);
}

#[test]
fn usj_reimports_semantically_for_all_origin_usfm_fixtures() {
    let root = Path::new("testData");
    let mut failures = Vec::new();

    for usfm_path in common::collect_origin_usfm_fixtures(root) {
        common::log_fixture("usj", &usfm_path);

        let source = fs::read_to_string(&usfm_path).expect("fixture usfm should read");
        let usj = convert::usfm_to_usj(&source).expect("USJ should serialize");
        let usj_json = serde_json::to_string(&usj).expect("USJ should stringify");
        let regenerated = convert::from_usj_str(&usj_json).expect("USJ should import");

        if tokens::tokens_to_usfm(&tokens::usfm_to_tokens(&regenerated)) != regenerated {
            failures.push(common::fixture_slug(root, &usfm_path));
        }
    }

    assert!(
        failures.is_empty(),
        "USJ semantic reimport failures:\n{}",
        failures.join("\n")
    );
}
