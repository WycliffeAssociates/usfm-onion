// // AGENT: USE THIS FILE TO TEST AND BENCHMARK CODE

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() {
    let _bsb_corpus = "example-corpora/examples.bsb";
    let _en_ulb = "example-corpora/en_ulb";
    let _en_ult = "example-corpora/en_ult";
    // profile();
    // dump_usj();
    // dump_usx();
    // dump_vref();
    dump_lint();
    // dump_format();
    // dump_diff();
    // dump_html();
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| {
    //     usfm_onion::format_usfm(source, usfm_onion::FormatOptions::default())
    // });
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| {
    //     usfm_onion::usfm_to_html(source, usfm_onion::HtmlOptions::default())
    // });
    // dump_file("example-corpora/examples.bsb/19PSABSB.usfm", |source| usfm_onion::parse(source));
    // 1. Load the actual USFM data into memory first
    // let corpus_path = Path::new(_bsb_corpus);
    // let entries = load_corpus(corpus_path)
    //     .into_iter()
    //     .map(|(path, source)| CorpusEntry {
    //         path: relative_display(&path),
    //         value: source, // We store the source string here to lint it later
    //     })
    //     .collect::<Vec<_>>();

    // println!("Loaded {} files. Starting profile...", entries.len());

    // 2. Fix the profile closure
    // let started = Instant::now();
    // profile(
    //     || {
    //         // entries.iter().map(...) is lazy! We use for_each to actually run it.
    //         entries.iter().for_each(|entry| {
    //             // Pass the content (entry.value), not the directory path
    //             // let _ = usfm_onion::lint_usfm(&entry.value, usfm_onion::LintOptions::default());
    //             let _ = usfm_onion::format_usfm(&entry.value, usfm_onion::FormatOptions::default());
    //         });
    //     },
    //     20,
    // );
    // let elapsed = started.elapsed();
    // println!("took {:?} time for {} iters", elapsed, 20);

    // println!("Profile complete.");

    // dif_book_genesis();
}

#[allow(dead_code)]
fn profile_cst() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let mut total = 0usize;

    for _ in 0..200 {
        let doc = usfm_onion::cst::parse_cst(&source);
        total += doc.tokens.len();
        std::hint::black_box(&doc);
    }

    println!("{total}");
}
#[allow(dead_code)]
fn profile<F: Fn()>(f: F, iters: usize) {
    for _ in 0..iters {
        std::hint::black_box(f());
    }
}

#[allow(dead_code)]
fn dump_usj() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let document = usfm_onion::usj::usfm_to_usj(&source).expect("USJ export should succeed");

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
    let xml = usfm_onion::usx::usfm_to_usx(&source).expect("USX export should succeed");

    let output_path = std::path::Path::new("playgroundOut.xml");
    std::fs::write(output_path, &xml)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{xml}");
}

#[allow(dead_code)]
fn dump_vref() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/631JNBSB.usfm").unwrap();
    let map = usfm_onion::vref::usfm_to_vref_map(&source);
    let json = usfm_onion::vref::vref_map_to_json_string(&map);

    let output_path = std::path::Path::new("playgroundOut.json");
    std::fs::write(output_path, &json)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{json}");
}

#[allow(dead_code)]
fn dump_lint() {
    let usfm_text = r#"\\c 6 
\\p 
\\v 1 He gone from dere back up inta he hometown, wit he disciples followin' him. 
\\v 2 An' wen da Sabbath come, he start teachin' in da synagogue. Da people who was hearin' him was shock 'bout what  he was sayin'. Dey say, "Where dis man get dese teachin' from?" An' where he get all dis wisdom from?" "An what  is dese miracle he is do wit he hands?" 
\\v 3 "Een dis da carpenter son, Mary boy, da brudda uh James an' Joses, an' uh Judea an' uh Simon? Een he sista dem here wit us?" Dey een like  what  Jesus was sayin'. \\f + \\ft The man called Joses here is called Joseph in Matthew 27:56. The name Joseph represents how the name was spelled in Hebrew, an' the name Joses represents how his name was spelled in Greek. \\fqa  \\f* 


\\s5
\\p
\\v 4 Den Jesus say ta dem, "Da only place a prophet een gat no respect is in he oon hometown an' by he oon family an' in da same house wit da people he live wit." 
\\v 5 He cud'n do no kinda good work, excep' ta lay he hand' on some people what  was sick an' heal dem. 
\\v 6 He was shock dat dey didn't believe. Den he gone all 'round da villages teachin'.


\\s5
\\p
\\v 7 Den he call da twelve an' start sendin' dem out two by two, an' he gee dem autority ova da demons, 
\\v 8 an' tell dem ta not take anyting fa dey trip - 
\\v 9 no bread, no bag an' no money - jus' a walkin' stick, but dey cud wear slippas an' cudna wear two set uh undaclothes. 
\\v 10 He say ta dem, "Whenever you go inside a house, stay dere an' don' leave 'til ya finish. 
\\v 11 If any a dem towns don' wan' nuttin' ta do wit you or listen ta yinna, wen you leave dat place, shake da dust off ya foot as an' example ta dem how dey een welcome you."


\\s5
\\p
\\v 12 Dey gone out an' let da people dem know dat dey shud repent. 
\\v 13 Den dey start castin' alotta demons outta people an' puttin' oil on dem an' healin' dem. 
\\v 14 King Herod hear 'bout dis, cuz plenty people did hear 'bout Jesus.  Some people was sayin', "John da Baptist mussy get raise from da dead, an' maybe dat cud be why all dese powers in him." 
\\v 15 Some uddas say, "He mussy Elijah." An' uddas say, "He is one prophet jus' like dem prophets from da old days."


\\s5
\\p
\\v 16 But wen Herod did hear 'bout dis he say, "John, who head I cut off, mussy raise from da dead." 
\\v 17 Herod was da one who cause John ta get lockup in jail cuz uh Herodias (he brudda Phillip wife), cuz Herod get marrid ta her. 
\\v 18 Cuz John did tell Herod, "It een right fa ya ta marrid ta ya brudda wife." 
\\v 19 So, Herodias was holdin' in her vexation at John an' was ga kill him, but she cudn't do it, 
\\v 20 cause Herod was scared uh John; He know he was a good God-fearin' man, an' he keep him safe. Listenin' ta him kinda make him confuse, but he was still happy ta listen ta him.


\\s5
\\p
\\v 21 Den da time did come fa Herod birtday an' he had a dinna party fa all he big shot people dem: da officials, da commissionas, da chief justice an' all dem leadas uh Galilee. 
\\v 22 Herodias daughta come in ta dance fa dem, an' Herod an' he guest dem was please. Da king tell da gal, "Aks me fa anyting ya wan' an' I ga gee it ta you." 
\\v 23 Den he swear ta her sayin', "Watevea you aks me fa, I ga gee you up ta half uh my kingdom. 
\\v 24 She gone outside an' tell her mudda, "Wat I should aks him fa?" She say, "Aks him fa da head uh John da Baptist." 
\\v 25 Den she rush back in da room an' say ta da king, "I wan' you ta gee me da head uh John da Baptist on a wood plate right nah."


\\s5
\\p
\\v 26 Da king was dead sad, He cudn' say no ta what  she aks fa cuz he done gone make a promise in front uh he big shot guest. 
\\v 27 Den da king  sen' one uh he soldier ta bring John head ta him. So, da guard gone ta da jail an' cut he head off. 
\\v 28 He bring da head on a plate an' gee it ta da gal, an' da gal take it an' gee it ta her mudda.


\\s5
\\p
\\v 29 Wen he disciple hear dis, dey come an' get he body an' put it in a tomb. 
\\v 30 Da apostles come togedda wit Jesus an' tell him erryting dey dun do an' what  dey teach. 
\\v 31 Den he say ta dem, "Take a break, go ta a place were een nobody is an rest yasef fa lil bit." Plenty was comin' an' goin', an' dey een even had no time ta eat.


\\s5
\\p
\\v 32 So dey gone in da boat an' start ta go ta a place where een hardly nobody was ta be by dey sef. 
\\v 33 But da people see dem leavin', an' dey recognize dem, an' from all da towns gone runnin', an' dey reach dere before dem. Wen dey come on da shore, He see a big crowd waitin' fa him. 
\\v 34 An' wen Jesus see dis he did feel sorry fa da people dem, cuz dey was just like sheep witout a shepherd. So he begin ta teach dem all kinda tings


\\s5
\\p
\\v 35 It was gettin' late, so da disciple dem come ta him an' say, "Dis place dead an' it gettin' late. 
\\v 36 Sen' dem ta da outside uh town an' in da lil settlements to buy sometin' ta eat fa deysef." 
\\v 37 He say, "Yall gee dem sometin' ta eat." Dey say ta Him, "You wan us ta buy two hundred silva coins wort uh bread ta gee dem ta eat?"


\\s5
\\p
\\v 38 He say ta dem, "Go an' see how many loafa bread you have." When dey finda out, dey say, "Five loafa bread an' two fish." 
\\v 39 He tell all da people ta sit down in groups on da grass. 
\\v 40 So, dey sit down in groups uh hundreds an' fifties. 
\\v 41 He take da five loafa bread an' two fish, look up in da sky, bless dem an' break dem, den he gee it ta da disciples ta gee ta da people. Den split da two fish up an' gee it ta da people. 
\\v 42 Da people dem eat 'til dey belly was full. 
\\v 43 Dey pick up twelve basket full uh break up pieces uh bread an' fish dat was lef'. 
\\v 44 Dere was 'bout five thousand man dat eat da bread.


\\s5
\\p
\\v 45 Right away, he tell da disciple dem ta get in da boat an' go ova ta da udda side, where Bethsaida is, while he sen' da buncha people away. 
\\v 46 Afta he leave dem, he gone up da mountain ta pray. 
\\v 47 An' when evenin' come, da boat was in da middle uh da sea, an' he was on land all by hesef. 
\\v 48 He cud see da boat fightin' da strong waves an' da winds was comin' 'gainst da boat . About da time between tree in da morning an' when da sun come up , Jesus come to da disciples dem walkin' on da sea, cuz he did wanna ta pass by dem.


\\s5
\\p
\\v 49 But wen dey see him walkin' on da sea, dey take him fa a ghost an' dey start hollerin' out, 
\\v 50 cuz dey see him an' was scared. But he talk ta dem right away an' say, "Be brave, is only me, don' be scared. 
\\v 51 "He got on da boat wit dem, an' all uh a sudden da wind stop. Dey was shock. 
\\v 52 Cuz dey een understan' what  da loaf uh bread did mean. Instead dey heart become cold. 
\\v 53 Wen dey finally get ta da udda side uh da shore in a land call Gennesaret, dey anchor dey boat.


\\s5
\\p
\\v 54 Wen dey come outta da boat, da people dem know jus' who he was, 
\\v 55 an' dey run tru all da country an' da people dem start bringin' the sick on dey mat ta wherever dey hear he was. 
\\v 56 It een matter where he gone, village, city, or in da country, dey wud bring da sick people in da marketplaces. Dey beg him ta let da sick touch da edge uh he clothes: As much what  cudda touch him was heal from dey sickness.


\\s5 "#;
    // let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let result = usfm_onion::lint::lint_usfm(&usfm_text, usfm_onion::LintOptions::default());
    let json = serde_json::to_string_pretty(&result).expect("lint result should serialize");

    let output_path = std::path::Path::new("playgroundOut.json");
    std::fs::write(output_path, &json)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{json}");
}

#[allow(dead_code)]
fn dump_format() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let formatted = usfm_onion::format::format_usfm(&source, usfm_onion::FormatOptions::default());

    let output_path = std::path::Path::new("playgroundOut.usfm");
    std::fs::write(output_path, &formatted)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{formatted}");
}

#[allow(dead_code)]
fn dif_book_genesis() {
    let ulb_path = "example-corpora/en_ulb/01-GEN.usfm";
    let bsb_path = "example-corpora/examples.bsb/01GENBSB.usfm";

    // 1. Load the actual USFM data into memory first
    let ulb_source = fs::read_to_string(ulb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", ulb_path));
    let bsb_source = fs::read_to_string(bsb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", bsb_path));

    println!("Loaded files. Starting profile...");

    let iters = 20;
    let started = Instant::now();

    // 2. Profile the diffing operation
    profile(
        || {
            let _diffs = usfm_onion::diff::diff_usfm_sources(
                &ulb_source,
                &bsb_source,
                &usfm_onion::BuildSidBlocksOptions::default(),
            );
        },
        iters,
    );

    let elapsed = started.elapsed();
    println!("took {:?} time for {} iters", elapsed, iters);
    println!("Profile complete.");
}

#[allow(dead_code)]
fn dump_diff() {
    let baseline = std::fs::read_to_string("example-corpora/examples.bsb/01GENBSB.usfm").unwrap();
    let current = baseline.replace(
        "God saw that the light was good",
        "God saw the light was good",
    );
    let diffs = usfm_onion::diff::diff_usfm_sources(
        &baseline,
        &current,
        &usfm_onion::BuildSidBlocksOptions::default(),
    );

    let output_path = std::path::Path::new("playgroundOut.json");
    serde_json::to_writer_pretty(
        std::fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        &diffs,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!(
        "{}",
        serde_json::to_string_pretty(&diffs).expect("diff result should serialize")
    );
}

#[allow(dead_code)]
fn dump_html() {
    let source = std::fs::read_to_string("example-corpora/examples.bsb/19PSABSB.usfm").unwrap();
    let html = usfm_onion::html::usfm_to_html(&source, usfm_onion::HtmlOptions::default());

    let output_path = std::path::Path::new("playgroundOut.html");
    std::fs::write(output_path, &html)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));

    println!("{html}");
}

#[derive(serde::Serialize)]
struct CorpusEntry<T> {
    path: String,
    value: T,
}

#[allow(dead_code)]
fn dump_file<T, F>(path: &str, f: F)
where
    T: serde::Serialize,
    F: Fn(&str) -> T,
{
    let path = Path::new(path);
    let source = read_source(path);
    let value = f(&source);
    write_json("playgroundOut.json", &value);
    println!("wrote {} to playgroundOut.json", path.display());
}

#[allow(dead_code)]
fn dump_corpus<T, F>(root: &str, f: F)
where
    T: serde::Serialize,
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let entries = load_corpus(root)
        .into_iter()
        .map(|(path, source)| CorpusEntry {
            path: relative_display(&path),
            value: f(&source),
        })
        .collect::<Vec<_>>();
    write_json("playgroundOut.json", &entries);
    println!(
        "wrote {} corpus entries from {} to playgroundOut.json",
        entries.len(),
        root.display()
    );
}

#[allow(dead_code)]
fn time_file<T, F>(label: &str, path: &str, f: F)
where
    F: Fn(&str) -> T,
{
    let path = Path::new(path);
    let source = read_source(path);
    let started = Instant::now();
    let value = f(&source);
    let elapsed = started.elapsed();
    std::hint::black_box(value);

    print_timing(label, path, 1, source.len(), elapsed, None);
}

#[allow(dead_code)]
fn time_corpus<T, F>(label: &str, root: &str, iters: usize, f: F)
where
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let corpus = load_corpus(root);
    let bytes = corpus.iter().map(|(_, source)| source.len()).sum::<usize>();
    let started = Instant::now();
    run_corpus_iters(&corpus, iters, &f);
    let elapsed = started.elapsed();
    print_timing(
        label,
        root,
        corpus.len() * iters,
        bytes * iters,
        elapsed,
        None,
    );
}

#[allow(dead_code)]
fn profile_corpus<T, F>(label: &str, root: &str, iters: usize, f: F)
where
    F: Fn(&str) -> T,
{
    let root = Path::new(root);
    let corpus = load_corpus(root);
    let bytes = corpus.iter().map(|(_, source)| source.len()).sum::<usize>();
    let started = Instant::now();
    run_corpus_iters(&corpus, iters, &f);
    let elapsed = started.elapsed();
    print_timing(
        label,
        root,
        corpus.len() * iters,
        bytes * iters,
        elapsed,
        Some(iters),
    );
}

#[allow(dead_code)]
fn time_parse_cst_file(path: &str) {
    let path = Path::new(path);
    let source = read_source(path);
    let started = Instant::now();
    let document = usfm_onion::cst::parse_cst(&source);
    let elapsed = started.elapsed();

    println!(
        "parse_cst {} -> {} tokens in {:.2?}",
        path.display(),
        document.tokens.len(),
        elapsed
    );
}

fn load_corpus(root: &Path) -> Vec<(PathBuf, String)> {
    collect_usfm_paths(root)
        .into_iter()
        .map(|path| {
            let source = read_source(&path);
            (path, source)
        })
        .collect()
}

fn collect_usfm_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_usfm_paths_into(root, &mut paths);
    paths.sort();
    paths
}

fn collect_usfm_paths_into(root: &Path, paths: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths_into(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn relative_display(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .ok()
        .unwrap_or(path)
        .display()
        .to_string()
}

fn write_json<T: serde::Serialize>(path: &str, value: &T) {
    let output_path = Path::new(path);
    serde_json::to_writer_pretty(
        fs::File::create(output_path)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display())),
        value,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
}

fn run_corpus_iters<T, F>(corpus: &[(PathBuf, String)], iters: usize, f: &F)
where
    F: Fn(&str) -> T,
{
    for _ in 0..iters {
        for (_, source) in corpus {
            let value = f(source);
            std::hint::black_box(value);
        }
    }
}

fn print_timing(
    label: &str,
    root: &Path,
    docs: usize,
    bytes: usize,
    elapsed: Duration,
    iters: Option<usize>,
) {
    let millis = elapsed.as_secs_f64() * 1000.0;
    let docs_per_sec = if elapsed.is_zero() {
        0.0
    } else {
        docs as f64 / elapsed.as_secs_f64()
    };
    let mib_per_sec = if elapsed.is_zero() {
        0.0
    } else {
        (bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
    };

    let mut summary = BTreeMap::new();
    summary.insert("label", label.to_string());
    summary.insert("root", root.display().to_string());
    summary.insert("docs", docs.to_string());
    summary.insert("bytes", bytes.to_string());
    if let Some(iters) = iters {
        summary.insert("iters", iters.to_string());
    }
    summary.insert("elapsed_ms", format!("{millis:.3}"));
    summary.insert("docs_per_sec", format!("{docs_per_sec:.2}"));
    summary.insert("mib_per_sec", format!("{mib_per_sec:.2}"));

    println!(
        "{}",
        serde_json::to_string_pretty(&summary).expect("timing summary should serialize")
    );
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
