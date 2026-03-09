use usfm_onion::{DocumentFormat, convert, diff, format, lint, model, parse};

#[test]
fn public_modules_support_happy_path_usage() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";

    let handle = parse::parse_content(source, DocumentFormat::Usfm).expect("parse should succeed");
    let tokens = parse::into_tokens(&handle, parse::IntoTokensOptions::default());
    assert!(!tokens.is_empty(), "expected projected tokens");

    let issues = lint::lint_document(&handle, lint::LintOptions::default());
    assert!(issues.is_empty(), "expected clean lint for minimal fixture");

    let formatted = format::format_content(
        source,
        format::DocumentFormat::Usfm,
        parse::IntoTokensOptions::default(),
    )
    .expect("format should succeed");
    assert!(
        parse::into_usfm_from_tokens(&formatted.tokens).contains("\\id GEN"),
        "expected formatted USFM to preserve content"
    );

    let usj = convert::convert_content(
        source,
        convert::DocumentFormat::Usfm,
        convert::DocumentFormat::Usj,
    )
    .expect("convert should succeed");
    assert!(usj.contains("\"type\""), "expected USJ output");

    let diffs = diff::diff_content(
        source,
        DocumentFormat::Usfm,
        source,
        DocumentFormat::Usfm,
        &model::TokenViewOptions::default(),
        &diff::BuildSidBlocksOptions::default(),
    )
    .expect("diff should succeed");
    assert!(
        diffs
            .iter()
            .all(|entry| entry.status == diff::DiffStatus::Unchanged),
        "expected identical sources to produce only unchanged diff entries"
    );
}
