//! Shared bench fixtures for the criterion harnesses in this directory —
//! and, via `#[path = "../benches/common.rs"] mod common;`, for
//! `examples/profile_ops.rs`, the samply-wrappable runner. The point of
//! sharing this module rather than hand-writing a parallel harness is that
//! a flamegraph and a criterion number can never silently drift apart: both
//! call exactly [`run_named_op`] on exactly the [`Fixture`] [`build_fixture`]
//! builds.
//!
//! Surfaces:
//! - [`load_luke`] — the single representative book used by `operations.rs`
//!   for the per-op String-vs-Tokens matrix. Picked because it's mid-sized,
//!   exercises chapters/verses/notes/cross-references, and runs fast enough
//!   for tight iteration on perf-sensitive changes.
//! - [`load_en_ulb`] — the full English ULB corpus (~66 books) used by
//!   `parallelism.rs` to compare serial vs `rayon` orchestration on a
//!   realistic whole-Bible workload.
//! - [`build_fixture`] / [`run_named_op`] / [`OP_NAMES`] — the operation
//!   matrix itself: one named body per op, called identically whether the
//!   caller is criterion (`operations.rs`) or a raw timing loop
//!   (`examples/profile_ops.rs`).

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::cst::build_cst;
use usfm_onion::diff::{diff_skeleton, diff_skeleton_canonical};
use usfm_onion::format::{FormatOptions, FormatToken, format_tokens, format_usfm, into_format_tokens};
use usfm_onion::html::{HtmlOptions, tokens_to_html, usfm_to_html};
use usfm_onion::lexer::lex;
use usfm_onion::lint::{LintOptions, LintScope, lint_tokens, lint_usfm};
use usfm_onion::parse::parse;
use usfm_onion::token::ParseResult;
use usfm_onion::usj::usfm_to_usj;
use usfm_onion::usx::usfm_to_usx;

#[allow(dead_code)]
pub struct Book {
    pub label: String,
    pub source: String,
    pub bytes: usize,
}

#[allow(dead_code)]
pub struct Corpus {
    pub label: String,
    pub books: Vec<String>,
    pub total_bytes: usize,
}

#[allow(dead_code)]
pub fn load_luke() -> Book {
    load_book("example-corpora/en_ulb/43-LUK.usfm", "luke")
}

/// Same book, different corpus: BSB's Luke has no `\s5` chunk markers (the
/// en_ulb corpus's are pervasive — 491 in this same book — and undocumented
/// in the USFM spec, so every one fires a lint issue) but is otherwise more
/// USFM-varied (`\r`, `\pc`, footnote/poetry markup en_ulb's Luke doesn't
/// use). Useful for telling "hot because of a real, recurring lint finding"
/// apart from "hot because of inherent USFM complexity."
#[allow(dead_code)]
pub fn load_bsb_luke() -> Book {
    load_book("example-corpora/examples.bsb/43LUKBSB.usfm", "luke-bsb")
}

#[allow(dead_code)]
pub fn load_en_ulb() -> Corpus {
    load_corpus("en_ulb", "example-corpora/en_ulb")
}

fn load_book(relative_path: &str, label: &str) -> Book {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("benchmark book {} should read: {err}", path.display()));
    let bytes = source.len();
    Book {
        label: format!("{label} ({bytes} bytes)"),
        source,
        bytes,
    }
}

fn load_corpus(label: &str, relative_root: &str) -> Corpus {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_root);
    let mut paths = Vec::new();
    collect_usfm_paths(&root, &mut paths);
    paths.sort();

    let books = paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("benchmark corpus file {} should read: {err}", path.display())
            })
        })
        .collect::<Vec<_>>();

    let total_bytes = books.iter().map(|src| src.len()).sum::<usize>();

    Corpus {
        label: format!("{label} ({} books, {total_bytes} bytes)", books.len()),
        books,
        total_bytes,
    }
}

fn collect_usfm_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|err| panic!("read {}: {err}", root.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry under {}: {err}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_usfm_paths(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "usfm") {
            paths.push(path);
        }
    }
}

/// Every op name `run_named_op` understands — the single source of truth
/// both `operations.rs` (iterates this to register criterion benches) and
/// `profile_ops.rs` (validates its `--op` argument against it) read from.
#[allow(dead_code)]
pub const OP_NAMES: &[&str] = &[
    "lex/string",
    "parse/string",
    "usj/string",
    "usx/string",
    "cst/tokens",
    "lint/string",
    "lint/tokens",
    "format/string",
    "format/tokens",
    "html/string",
    "html/tokens",
    "diff/string",
    "diff/tokens",
];

/// Pre-parsed inputs shared by every op in [`OP_NAMES`] — built once,
/// borrowed by every criterion sample or profiling iteration, so none of
/// the parse/lint-option/format-token-seed setup cost leaks into a
/// per-iteration measurement.
#[allow(dead_code)]
pub struct Fixture<'a> {
    pub book: &'a Book,
    pub parsed: ParseResult<'a>,
    pub format_token_seed: Vec<FormatToken>,
    pub lint_options: LintOptions,
    pub format_options: FormatOptions,
    pub html_options: HtmlOptions,
}

#[allow(dead_code)]
pub fn build_fixture(book: &Book) -> Fixture<'_> {
    let parsed = parse(&book.source);
    let format_token_seed = into_format_tokens(&parsed.tokens);
    Fixture {
        book,
        parsed,
        format_token_seed,
        lint_options: LintOptions::scoped(LintScope::Book),
        format_options: FormatOptions::default(),
        html_options: HtmlOptions::default(),
    }
}

/// Run the op named `name` once against `fx`. This is the one body per op —
/// criterion's `b.iter(|| run_named_op(name, &fx))` and a raw
/// `for _ in 0..iters { run_named_op(&op, &fx) }` loop call exactly this,
/// so a flamegraph can never end up measuring something subtly different
/// from what criterion reports. Panics on an unknown name; see [`OP_NAMES`].
#[allow(dead_code)]
pub fn run_named_op(name: &str, fx: &Fixture) {
    match name {
        "lex/string" => {
            black_box(lex(&fx.book.source));
        }
        "parse/string" => {
            black_box(parse(&fx.book.source));
        }
        "usj/string" => {
            black_box(usfm_to_usj(&fx.book.source).expect("USJ export"));
        }
        "usx/string" => {
            black_box(usfm_to_usx(&fx.book.source).expect("USX export"));
        }
        "cst/tokens" => {
            black_box(build_cst(fx.parsed.tokens.clone()));
        }
        "lint/string" => {
            black_box(lint_usfm(&fx.book.source, fx.lint_options.clone()));
        }
        "lint/tokens" => {
            black_box(lint_tokens(&fx.parsed.tokens, fx.lint_options.clone()));
        }
        "format/string" => {
            black_box(format_usfm(&fx.book.source, fx.format_options));
        }
        "format/tokens" => {
            let mut working = fx.format_token_seed.clone();
            format_tokens(&mut working, fx.format_options);
            black_box(working);
        }
        "html/string" => {
            black_box(usfm_to_html(&fx.book.source, fx.html_options));
        }
        "html/tokens" => {
            black_box(tokens_to_html(&fx.parsed.tokens, fx.html_options));
        }
        "diff/string" => {
            let side = parse(&fx.book.source);
            black_box(diff_skeleton_canonical(&side.tokens, "LUK", &side.tokens, "LUK"));
        }
        "diff/tokens" => {
            black_box(diff_skeleton(&fx.parsed.tokens, &fx.parsed.tokens));
        }
        other => panic!("unknown op {other:?}; see common::OP_NAMES for the valid list"),
    }
}

fn black_box<T>(value: T) -> T {
    std::hint::black_box(value)
}
