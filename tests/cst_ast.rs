use usfm_onion::{ast, cst, tokens};

#[test]
fn usfm_tokens_cst_tokens_roundtrips_exactly() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n\\f + \\ft note\\f*\n";

    let original_tokens = tokens::usfm_to_tokens(source);
    let document = cst::parse_usfm(source);
    let reconstructed_tokens = cst::cst_to_tokens(&document);

    assert_eq!(reconstructed_tokens, original_tokens);
    assert_eq!(tokens::tokens_to_usfm(document.tokens()), source);
    assert_eq!(cst::dfs_source_text(&document), source);
}

#[test]
fn cst_flows_into_ast_and_semantic_exports() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n";

    let document = cst::parse_usfm(source);
    let ast = ast::cst_to_ast(&document);

    let usj = usfm_onion::convert::ast_to_usj(&ast).expect("AST should export to USJ");
    let usx = usfm_onion::convert::ast_to_usx(&ast).expect("AST should export to USX");
    let vref = usfm_onion::convert::ast_to_vref(&ast).expect("AST should export to VREF");

    assert_eq!(usj.doc_type, "USJ");
    assert!(usx.contains("<usx"));
    assert!(vref.contains_key("GEN 1:1"));
}
