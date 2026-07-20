//! Behavior oracle: a byte-stable snapshot of what the engine currently does
//! for every `testData/**/*.usfm` fixture — the token stream (digest) and the
//! full lint findings. It is deliberately captured over `testData` rather than
//! `example-corpora` because testData is the adversarial corpus (milestones,
//! wild samples, special-cases) where a behavior change is most likely to hide.
//!
//! This is the "thing that doesn't move" gate for the lint-parallelization
//! work: parse once, `lint_tokens` over the whole book. When that path is later
//! reworked to split the token stream at `\c` and lint chapters in parallel,
//! this snapshot must stay byte-identical — that is the proof that `par` ==
//! `serial`. Findings are emitted in `lint_tokens`' canonical source order (see
//! `canonical_sort`), so a chapter-parallel linter that collects findings out of
//! segment order and sorts reproduces this baseline exactly; even the cross-
//! chapter `duplicate-chapter-number` finding (folded in a final reduce) lands
//! at its source position.
//!
//! Workflow (mirrors the sous-chef oracle-gate "intentional vs regression"
//! contract):
//!   - `cargo test --test lint_oracle`            -> asserts against the baseline
//!   - `BLESS=1 cargo test --test lint_oracle`    -> (re)writes the baseline
//! Review the git diff of `tests/lint_oracle_baseline.txt` on every bless: a
//! reorder of the same findings is intentional; a changed/added/dropped finding
//! is a regression until proven otherwise.

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::cst::{CstNode, build_cst_roots};
use usfm_onion::html::{HtmlOptions, usfm_to_html};
use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
use usfm_onion::parse::parse;
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;
use usfm_onion::vref::{usfm_to_vref_map, vref_map_to_json_string};

const BASELINE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/lint_oracle_baseline.txt"
);

/// FNV-1a 64-bit — same digest family the sous-chef oracle-gate uses. A digest
/// (not the full token dump) keeps the baseline small; the token stream is
/// millions of tokens over the corpus and is uninteresting until it moves, at
/// which point you regenerate and inspect that one file.
fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Total nodes in a CST forest (roots + all descendants) — a human-facing
/// count; the digest is what actually gates.
fn count_cst_nodes(nodes: &[CstNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_cst_nodes(&node.children))
        .sum()
}

/// All `*.usfm` fixtures under `testData`, sorted for deterministic output.
fn fixture_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("testData");
    let mut out = Vec::new();
    collect_usfm(&root, &mut out);
    out.sort();
    out
}

fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usfm(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("usfm") {
            out.push(path);
        }
    }
}

/// Render one fixture's record: header + token digest + one JSON line per
/// issue + a summary line. One issue per line gives line-granular diffs.
fn render_record(relpath: &str, source: &str) -> String {
    let mut record = format!("### {relpath}\n");

    let parsed = parse(source);
    let token_bytes = serde_json::to_vec(&parsed.tokens).expect("tokens serialize");
    record.push_str(&format!(
        "tokens\t{}\t{:016x}\n",
        parsed.tokens.len(),
        fnv64(&token_bytes)
    ));

    // The CST nesting forest (token_index + children). It is the input every
    // walker-driven export (html/usj/usx/vref/format) reads, so token digest
    // (content) + cst digest (structure) together pin all downstream outputs
    // without snapshotting each format separately.
    let cst_roots = build_cst_roots(&parsed.tokens);
    let cst_bytes = serde_json::to_vec(&cst_roots).expect("cst serialize");
    record.push_str(&format!(
        "cst\t{}\t{:016x}\n",
        count_cst_nodes(&cst_roots),
        fnv64(&cst_bytes)
    ));

    // Direct anchors for every walker-driven export. Byte-identical tokens
    // prove the walker's *input* didn't move, but NOT that a future chapter-
    // parallel assembler/merge of each export is correct — so each output is
    // pinned directly. Default options here; the counter-sensitive HTML option
    // matrix is added when the HTML merge stage lands.
    match usfm_to_usj(source) {
        Ok(doc) => {
            let bytes = serde_json::to_vec(&doc).expect("usj serialize");
            record.push_str(&format!("usj\t{}\t{:016x}\n", bytes.len(), fnv64(&bytes)));
        }
        Err(err) => record.push_str(&format!("usj\terror\t{err}\n")),
    }
    match usfm_to_usx(source) {
        Ok(xml) => record.push_str(&format!(
            "usx\t{}\t{:016x}\n",
            xml.len(),
            fnv64(xml.as_bytes())
        )),
        Err(err) => record.push_str(&format!("usx\terror\t{err}\n")),
    }
    let vref = vref_map_to_json_string(&usfm_to_vref_map(source));
    record.push_str(&format!(
        "vref\t{}\t{:016x}\n",
        vref.len(),
        fnv64(vref.as_bytes())
    ));
    let html = usfm_to_html(source, HtmlOptions::default());
    record.push_str(&format!(
        "html\t{}\t{:016x}\n",
        html.len(),
        fnv64(html.as_bytes())
    ));

    // The exact path that gets parallelized: lint the already-parsed token
    // stream at book scope.
    let result = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book));
    for issue in &result.issues {
        let line = serde_json::to_string(issue).expect("issue serialize");
        record.push_str(&line);
        record.push('\n');
    }
    let summary = serde_json::to_string(&result.summary).expect("summary serialize");
    record.push_str(&format!("summary\t{summary}\n"));

    record
}

fn build_snapshot() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut snapshot = String::new();
    for path in fixture_paths() {
        let relpath = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        match fs::read_to_string(&path) {
            Ok(source) => snapshot.push_str(&render_record(&relpath, &source)),
            // Fail loud: a fixture we can't read is recorded, never skipped.
            Err(err) => snapshot.push_str(&format!("### {relpath}\nread_error\t{err}\n")),
        }
    }
    snapshot
}

#[test]
fn lint_oracle_is_stable() {
    let snapshot = build_snapshot();

    if std::env::var_os("BLESS").is_some() {
        fs::write(BASELINE, &snapshot).expect("write baseline");
        eprintln!("blessed lint oracle baseline: {BASELINE}");
        return;
    }

    let baseline = fs::read_to_string(BASELINE).unwrap_or_else(|_| {
        panic!(
            "no oracle baseline at {BASELINE}\n\
             generate it with: BLESS=1 cargo test --test lint_oracle"
        )
    });

    if snapshot != baseline {
        // Point at the first differing line so a regression is easy to locate.
        let first_diff = snapshot
            .lines()
            .zip(baseline.lines())
            .enumerate()
            .find(|(_, (a, b))| a != b);
        let detail = match first_diff {
            Some((n, (got, want))) => format!(
                "first diff at line {}:\n  baseline: {want}\n  current:  {got}",
                n + 1
            ),
            None => format!(
                "length differs: current {} lines, baseline {} lines",
                snapshot.lines().count(),
                baseline.lines().count()
            ),
        };
        panic!(
            "lint oracle moved.\n{detail}\n\n\
             If this change is intentional, review the diff then re-bless:\n  \
             BLESS=1 cargo test --test lint_oracle"
        );
    }
}
