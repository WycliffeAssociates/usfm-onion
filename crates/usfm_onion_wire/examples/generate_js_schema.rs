//! Regenerates `js/wire-schema.js` and `js/wire-schema.d.ts` from
//! `usfm_onion_wire::js_schema::render`, which reads this crate's own
//! `schema` module — the same constants the Rust codec compiles against.
//! Never hand-edit the generated files.
//!
//! Run: `cargo run --example generate_js_schema -p usfm_onion_wire`
//!
//! `usfm_onion_wire::js_schema::tests::wire_schema_matches_generator` fails
//! if the checked-in files drift from what this produces.

use std::path::Path;

fn main() {
    let (js, dts) = usfm_onion_wire::js_schema::render();

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("crates/usfm_onion_wire is two levels under the repo root");
    let js_path = repo_root.join("js/wire-schema.js");
    let dts_path = repo_root.join("js/wire-schema.d.ts");

    std::fs::write(&js_path, js).unwrap_or_else(|e| panic!("write {js_path:?}: {e}"));
    std::fs::write(&dts_path, dts).unwrap_or_else(|e| panic!("write {dts_path:?}: {e}"));
    println!("wrote {js_path:?} and {dts_path:?}");
}
