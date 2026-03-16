// // AGENT: USE THIS FILE TO TEST AND BENCHMARK CODE

use std::fs;
use std::path::Path;
use usfm_onion::{parse, tokens_to_usfm};

fn main() {
    let path = Path::new("example-corpora/examples.bsb/631JNBSB.usfm");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    let parsed = parse(&source);
    println!("{:#?}", parsed);
    println!("{}", tokens_to_usfm(&parsed.tokens) == source);
    let output_path = Path::new("playgroundOut.json");
    serde_json::to_writer(
        fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        &parsed,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
}
// fn main() {
//     let path =
//         Path::new(env!("CARGO_MANIFEST_DIR")).join("example-corpora/examples.bsb/642JNBSB.usfm");
//     let source = fs::read_to_string(&path)
//         .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
//     let document = cst::parse_usfm(&source);

//     let output_path = Path::new("playgroundOut.json");
//     serde_json::to_writer(
//         fs::File::create(output_path)
//             .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
//         &document,
//     )
//     .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

//     println!(
//         "{}",
//         serde_json::to_string_pretty(&document).expect("CST should serialize")
//     );
// }
