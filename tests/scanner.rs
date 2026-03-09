mod common;

use std::fs;
use std::path::Path;

use usfm_onion::{
    model::ScanTokenKind,
    parse::{RecoveryCode, lex, parse, recoveries},
};

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

            if token.kind == ScanTokenKind::Text && token.text.is_empty() {
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

#[test]
fn glossary_newline_patterns_tokenize_as_newline_tokens() {
    let crlf = lex("\r\n");
    assert_eq!(crlf.tokens.len(), 1);
    assert_eq!(crlf.tokens[0].kind, ScanTokenKind::Newline);

    let cr = lex("\r");
    assert_eq!(cr.tokens.len(), 1);
    assert_eq!(cr.tokens[0].kind, ScanTokenKind::Newline);

    let lf = lex("\n");
    assert_eq!(lf.tokens.len(), 1);
    assert_eq!(lf.tokens[0].kind, ScanTokenKind::Newline);
}

#[test]
fn glossary_text_and_optbreak_boundaries_are_stable() {
    let result = lex("alpha/beta//gamma");
    assert_eq!(result.tokens.len(), 3);
    assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
    assert_eq!(result.tokens[0].text, "alpha/beta");
    assert_eq!(result.tokens[1].kind, ScanTokenKind::OptBreak);
    assert_eq!(result.tokens[1].text, "//");
    assert_eq!(result.tokens[2].kind, ScanTokenKind::Text);
    assert_eq!(result.tokens[2].text, "gamma");
}

#[test]
fn glossary_text_escapes_do_not_start_attributes_or_markers() {
    let result = lex("a\\|b \\/ c \\~ d \\\\ e");
    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].kind, ScanTokenKind::Text);
    assert_eq!(result.tokens[0].text, "a\\|b \\/ c \\~ d \\\\ e");
}

#[test]
fn invalid_lexical_sequences_report_stable_recovery_codes() {
    let handle = parse("\\id GEN\n\\c 1\n\\p\n\\v 1 text \\qt-s |sid=\"GEN 1:1\" next\n\\em*");
    let codes = recoveries(&handle)
        .iter()
        .map(|recovery| &recovery.code)
        .collect::<Vec<_>>();

    assert!(
        codes.contains(&&RecoveryCode::MissingMilestoneSelfClose),
        "expected MissingMilestoneSelfClose recovery code"
    );
    assert!(
        codes.contains(&&RecoveryCode::StrayCloseMarker),
        "expected StrayCloseMarker recovery code"
    );
}
