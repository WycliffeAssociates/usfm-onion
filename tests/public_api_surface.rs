use usfm_onion::{DocumentFormat, convert, diff, document_tree, format, lint, tokens};

#[test]
fn public_modules_support_happy_path_usage() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";

    let token_list = tokens::usfm_to_tokens(source);
    assert!(!token_list.is_empty(), "expected projected tokens");

    let tree = document_tree::usfm_to_document_tree(source);
    assert!(!tree.content.is_empty(), "expected document tree content");

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
