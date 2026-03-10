use std::env;
use std::fs;

use usfm_onion::document_tree::{document_tree_to_usfm, usfm_to_document_tree};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let dump_tree = args.iter().any(|arg| arg == "--dump-tree");
    let path = args
        .iter()
        .find(|arg| arg.as_str() != "--dump-tree")
        .expect("usage: cargo run --bin document-tree-roundtrip-diff -- <path>");
    let source = fs::read_to_string(path).expect("fixture should be readable");
    let tree = usfm_to_document_tree(&source);
    let reconstructed = document_tree_to_usfm(&tree).expect("tree should serialize");

    if dump_tree {
        println!(
            "TREE:\n{}",
            serde_json::to_string_pretty(&tree).expect("tree should serialize")
        );
    }

    if source == reconstructed {
        println!("exact match");
        return;
    }

    println!("SOURCE:\n---\n{source}\n---");
    println!("RECONSTRUCTED:\n---\n{reconstructed}\n---");
}
