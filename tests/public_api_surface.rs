use usfm_onion::{DocumentFormat, TokenVariant, ast, convert, cst, diff, format, lint, tokens};

#[test]
fn public_modules_support_happy_path_usage() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";

    let token_list = tokens::usfm_to_tokens(source);
    assert!(!token_list.is_empty(), "expected projected tokens");
    let variants = tokens::classify_tokens(&token_list);
    assert!(!variants.is_empty(), "expected token variants");
    assert!(matches!(variants[0], TokenVariant::Marker { .. }));

    let parsed = cst::parse_usfm(source);
    assert!(!parsed.content.is_empty(), "expected CST content");
    assert_eq!(
        tokens::tokens_to_usfm(cst::cst_tokens(&parsed)),
        source,
        "expected CST tokens to round-trip"
    );

    let tree = ast::cst_to_ast(&parsed);
    assert!(!tree.content.is_empty(), "expected AST content");

    let issues = lint::lint_content(source, DocumentFormat::Usfm, lint::LintOptions::default())
        .expect("lint should succeed");
    assert!(issues.is_empty(), "expected clean lint for minimal fixture");

    let formatted = format::format_content(source, DocumentFormat::Usfm, Default::default())
        .expect("format should succeed");
    assert!(
        tokens::tokens_to_usfm(&formatted.tokens).contains("\\id GEN"),
        "expected formatted USFM to preserve content"
    );

    let usj = convert::usfm_to_usj(source).expect("convert should succeed");
    assert_eq!(usj.doc_type, "USJ");

    let diffs = diff::diff_content(
        source,
        DocumentFormat::Usfm,
        source,
        DocumentFormat::Usfm,
        &tokens::TokenViewOptions::default(),
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
