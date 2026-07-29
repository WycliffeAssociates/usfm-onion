//! Rust half of the cross-language token equivalence gate.
//!
//! Emits, for one case, the packed bytes and the token DTOs the Rust decoder
//! materializes from them, as newline-delimited JSON — one token per line, so
//! the JS side can stream a 280,000-token book instead of holding two copies of
//! it. `scripts/test-packed-equivalence.mjs` drives this per book and deletes the
//! output as it goes.
//!
//!   emit_token_equivalence usfm <source.usfm> <out-dir>
//!   emit_token_equivalence bin  <vector.bin> <vector.usfm> <out-dir>
//!
//! `usfm` encodes a fresh parse plus its lint findings (the production container
//! shape: a token section paired with a finding section) and then decodes what it
//! just wrote. `bin` decodes committed golden bytes as-is, which is what makes the
//! golden arm a test of the *bytes* rather than of the encoder.

use std::fs;
use std::path::Path;

use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
use usfm_onion::parse::parse;
use usfm_onion::token::BookId;
use usfm_onion_wire::finding_codec::encode_book;
use usfm_onion_wire::verify::materialize_tokens;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (packed, source, out_dir) = match args.first().map(String::as_str) {
        Some("usfm") if args.len() == 3 => {
            let source = fs::read_to_string(&args[1]).expect("source reads");
            let parsed = parse(&source);
            let issues = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book)).issues;
            // The nominal section book is not derived from the file and does not
            // reach a token DTO (a token's sid book comes from its own anchor,
            // its id from the stream's own book-code tokens), so one constant
            // keeps this emitter honest about what it is testing — the corpus
            // gate in `tests/corpus.rs` uses the same one.
            let book = BookId::from_str("GEN").expect("nominal book code");
            let packed = encode_book(book, &source, &parsed.tokens, &issues).expect("encodes");
            (packed, source, args[2].clone())
        }
        Some("bin") if args.len() == 4 => (
            fs::read(&args[1]).expect("packed reads"),
            fs::read_to_string(&args[2]).expect("source reads"),
            args[3].clone(),
        ),
        _ => {
            eprintln!(
                "usage: emit_token_equivalence usfm <source.usfm> <out-dir>\n       emit_token_equivalence bin <vector.bin> <vector.usfm> <out-dir>"
            );
            std::process::exit(2);
        }
    };

    let materialized = materialize_tokens(&packed, &source).expect("packed bytes materialize");
    let out_dir = Path::new(&out_dir);
    fs::create_dir_all(out_dir).expect("out dir");
    fs::write(out_dir.join("packed.bin"), &packed).expect("write packed");
    fs::write(out_dir.join("source.usfm"), source.as_bytes()).expect("write source");

    let mut jsonl = String::new();
    for token in &materialized.tokens {
        jsonl.push_str(&serde_json::to_string(token).expect("token serializes"));
        jsonl.push('\n');
    }
    fs::write(out_dir.join("tokens.jsonl"), jsonl).expect("write tokens");
    fs::write(
        out_dir.join("stable-ids.json"),
        serde_json::to_string(&materialized.stable_ids).expect("ids serialize"),
    )
    .expect("write stable ids");
}
