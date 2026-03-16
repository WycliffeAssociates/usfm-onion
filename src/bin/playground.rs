// // AGENT: USE THIS FILE TO TEST AND BENCHMARK CODE

fn main() {
    profile();
    // dump_usj();
    // dump_usx();
    // let path = Path::new("example-corpora/en_ulb/01-GEN.usfm");
    // let source = fs::read_to_string(&path)
    //     .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    // let document = parse_cst(&source);
    // println!("{:#?}", document);
    // println!("{}", cst_to_usfm(&document) == source);
    // let output_path = Path::new("playgroundOut.json");
    // serde_json::to_writer(
    //     fs::File::create(output_path)
    //         .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
    //     &document,
    // )
    // .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
}

fn profile() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let mut total = 0usize;

    for _ in 0..200 {
        let doc = usfm_onion::parse_cst(&source);
        total += doc.tokens.len();
        std::hint::black_box(&doc);
    }

    println!("{total}");
}

#[allow(dead_code)]
fn dump_usj() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let document = usfm_onion::usfm_to_usj(&source).expect("USJ export should succeed");

    let output_path = std::path::Path::new("playgroundOut.json");
    serde_json::to_writer_pretty(
        std::fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        &document,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!(
        "{}",
        serde_json::to_string_pretty(&document).expect("USJ should serialize")
    );
}

#[allow(dead_code)]
fn dump_usx() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let xml = usfm_onion::usfm_to_usx(&source).expect("USX export should succeed");

    let output_path = std::path::Path::new("playgroundOut.xml");
    std::fs::write(output_path, &xml)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{xml}");
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
