//! Native-vs-wasm parity transcript generator.
//!
//! `#[ignore]`d by design (a generator, not an assertion): it drives the
//! *native* `braid::Braid` through a scripted lifecycle, converts every
//! step's outcome through the exact same DTO conversions the wasm `Braid`
//! class uses (`crate::resident`/`crate::dto`), and writes the whole
//! sequence — both the arguments and the expected wasm-shaped output, per
//! step — to a committed JSON fixture. A companion node script drives the
//! *real* wasm `Braid` class (bundler and web builds) through the identical
//! sequence and deep-compares its actual output against this fixture.
//!
//! This lives inside the crate (not `tests/`, which only sees `pub` items)
//! specifically so it can call the same `pub(crate)` conversions the real
//! bindings call — `crate::resident::MutationEffect::from`,
//! `crate::dto::map_lint_issue`, and so on — rather than a second,
//! hand-maintained mirror of them. The only place this generator
//! legitimately diverges from calling production code directly is
//! `braid::Braid::new`'s minter: the wasm class's constructor takes a
//! `js_sys::Function`, which has no meaningful native behavior outside a
//! real JS engine, so this drives the native crate directly with a plain
//! Rust closure. Both sides use the identical deterministic `"minted-N"`
//! counter, called the same number of times in the same order, which is
//! what keeps synthesized ids identical across the two languages.
//!
//! Run with `cargo test -p usfm_onion_wasm --lib -- --ignored generate_parity_transcript`.

use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use braid::Braid as NativeBraid;
use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

use crate::dto::{self, LintOptions as DtoLintOptions, LintScope as DtoLintScope};
use crate::resident;

/// One lifecycle step: the verb name, the ingest lane it ran under, the
/// arguments (already in the wasm-facing JSON shape, so the node script can
/// hand them straight to the real wasm method), and the expected output in
/// that same shape.
#[derive(serde::Serialize)]
struct Step {
    step: String,
    lane: String,
    args: Value,
    output: Value,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parity-transcript.json")
}

fn owned(source: &str) -> Vec<OwnedToken> {
    parse(source)
        .tokens
        .iter()
        .map(OwnedToken::from_parsed)
        .collect()
}

fn dto_tokens(tokens: &[OwnedToken]) -> Vec<crate::Token> {
    tokens.iter().map(crate::Token::from).collect()
}

fn book_id(code: &str) -> BookId {
    BookId::from_str(code).unwrap_or_else(|| panic!("{code} is a three-character code"))
}

fn lint_options() -> DtoLintOptions {
    DtoLintOptions {
        scope: DtoLintScope::Book,
        enabled_codes: None,
        disabled_codes: Vec::new(),
        suppressed: Vec::new(),
        allow_implicit_chapter_content_verse: false,
    }
}

fn braid_config() -> braid::BraidConfig {
    braid::BraidConfig::new(dto::lint_options_into_native(lint_options()))
}

/// The one deterministic minter shape both languages reproduce: an
/// incrementing counter, called only when a fix/format pass admits a
/// synthesized token.
fn minter() -> impl FnMut() -> String {
    let mut next = 0u32;
    move || {
        next += 1;
        format!("minted-{next}")
    }
}

/// A chapter run's own tokens, sliced from a *whole*, freshly parsed
/// edited book — never a bare fragment parsed on its own, which has no
/// `\id` in scope and so derives no sid at all. Mirrors the same technique
/// braid's own resident test suite uses for exactly this reason.
fn chapter_slice(whole_edited_source: &str, label: &str) -> Vec<OwnedToken> {
    let tokens = owned(whole_edited_source);
    let segment = usfm_onion::walker::chapter_segments(&tokens)
        .into_iter()
        .find(|segment| !segment.is_front && tokens[segment.range.start + 1].source() == label)
        .unwrap_or_else(|| panic!("no chapter {label} in the edited source"));
    tokens[segment.range].to_vec()
}

// ---- dto helpers (wasm-facing JSON args) ----------------------------------

fn dto_book_usfm(key: &str, book: &str, source: &str) -> resident::BookInput {
    resident::BookInput::Usfm {
        source_key: key.to_string(),
        book: book.to_string(),
        source: source.to_string(),
    }
}

fn dto_book_tokens(
    key: &str,
    book: &str,
    tokens: &[OwnedToken],
    line_ending: resident::LineEnding,
) -> resident::BookInput {
    resident::BookInput::Tokens {
        source_key: key.to_string(),
        book: book.to_string(),
        tokens: dto_tokens(tokens),
        line_ending,
    }
}

fn native_book_usfm(key: &str, book: &str, source: &str) -> braid::BookInput {
    braid::BookInput::Usfm {
        source_key: braid::SourceKey::new(key.to_string()).unwrap(),
        book: book_id(book),
        source: source.to_string(),
    }
}

fn native_book_tokens(
    key: &str,
    book: &str,
    tokens: Vec<OwnedToken>,
    line_ending: braid::LineEnding,
) -> braid::BookInput {
    braid::BookInput::Tokens(braid::BookTokensInput {
        source_key: braid::SourceKey::new(key.to_string()).unwrap(),
        book: book_id(book),
        tokens,
        line_ending,
    })
}

fn dto_chapter_target(book: &str, label: &str) -> resident::ChapterTarget {
    resident::ChapterTarget {
        book: book.to_string(),
        label: resident::ChapterLabel::Number {
            label: label.to_string(),
        },
    }
}

fn native_chapter_target(book: &str, label: &str) -> braid::ChapterTarget {
    braid::ChapterTarget::new(book_id(book), braid::ChapterLabel::Number(label.into()))
}

// ---- fixture sources --------------------------------------------------

/// Two whitespace-missing-delimiter findings (the same shape braid's own
/// resident patch tests use) in chapter 1, plus a clean chapter 2, so
/// `update_chapter` on chapter 1 alone has something visibly *not* touched.
const GEN_SOURCE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth.\\q1\n\\v 3 Light.\n\\c 2\n\\p\n\\v 1 And God said.\n";
const GEN_EDITED_WHOLE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth below.\\q1\n\\v 3 Light.\n\\c 2\n\\p\n\\v 1 And God said.\n";
const EXO_SOURCE: &str = "\\id EXO\n\\c 1\n\\p\n\\v 1-2 These are the names.\n";
const DUP_SOURCE: &str = "\\id DUP\n\\c 1\n\\p\n\\v 1 a\n\\c 1\n\\p\n\\v 1 b\n";

/// Runs the whole scripted lifecycle once for one ingest lane, appending
/// every step's `{lane, args, output}` to `steps`.
fn run_lane(lane: &str, steps: &mut Vec<Step>) {
    let mut resident = NativeBraid::new(braid_config(), minter());
    let mut push = |step: &str, args: Value, output: Value| {
        steps.push(Step {
            step: step.to_string(),
            lane: lane.to_string(),
            args,
            output,
        });
    };

    let (gen_book, exo_book) = match lane {
        "usfm" => (
            dto_book_usfm("01-GEN.usfm", "GEN", GEN_SOURCE),
            dto_book_usfm("02-EXO.usfm", "EXO", EXO_SOURCE),
        ),
        "tokens" => (
            dto_book_tokens(
                "01-GEN.usfm",
                "GEN",
                &owned(GEN_SOURCE),
                resident::LineEnding::Lf,
            ),
            dto_book_tokens(
                "02-EXO.usfm",
                "EXO",
                &owned(EXO_SOURCE),
                resident::LineEnding::Lf,
            ),
        ),
        _ => unreachable!(),
    };
    let native_books = match lane {
        "usfm" => vec![
            native_book_usfm("01-GEN.usfm", "GEN", GEN_SOURCE),
            native_book_usfm("02-EXO.usfm", "EXO", EXO_SOURCE),
        ],
        "tokens" => vec![
            native_book_tokens(
                "01-GEN.usfm",
                "GEN",
                owned(GEN_SOURCE),
                braid::LineEnding::Lf,
            ),
            native_book_tokens(
                "02-EXO.usfm",
                "EXO",
                owned(EXO_SOURCE),
                braid::LineEnding::Lf,
            ),
        ],
        _ => unreachable!(),
    };

    // 1. replace_corpus
    let args = json!({ "corpus": { "books": [gen_book.clone(), exo_book.clone()] } });
    let effect = resident
        .replace_corpus(braid::CorpusInput::new(native_books))
        .expect("distinct books");
    push(
        "replace_corpus",
        args,
        json!(resident::MutationEffect::from(effect)),
    );

    // 2. books
    push(
        "books",
        json!({}),
        json!(
            resident
                .books()
                .into_iter()
                .map(|entry| resident::BookEntry {
                    source_key: entry.source_key.as_str().to_string(),
                    book: entry.book.as_str().to_string(),
                    source_hash: format!("{:016x}", entry.source_hash.0),
                    token_identity: format!("{:016x}", entry.token_identity.0),
                    line_ending: entry.line_ending.into(),
                })
                .collect::<Vec<_>>()
        ),
    );

    // 3. chapter_labels(GEN)
    push(
        "chapter_labels",
        json!({ "book": "GEN" }),
        json!(
            resident
                .chapter_labels(book_id("GEN"))
                .unwrap()
                .iter()
                .map(resident::ChapterLabel::from)
                .collect::<Vec<_>>()
        ),
    );

    // 3b. error: chapter_labels on an unknown book
    push(
        "chapter_labels_unknown_book",
        json!({ "book": "ZZZ" }),
        json!(resident::ScopeError::from(braid::ScopeError::BookNotFound(
            book_id("ZZZ")
        ))),
    );

    // 4. update_chapter(GEN, chapter 1, edited)
    let replacement_tokens = chapter_slice(GEN_EDITED_WHOLE, "1");
    let args = json!({
        "target": dto_chapter_target("GEN", "1"),
        "replacement": { "kind": "tokens", "tokens": dto_tokens(&replacement_tokens) },
    });
    let effect = resident
        .update_chapter(
            native_chapter_target("GEN", "1"),
            braid::ChapterInput::Tokens(replacement_tokens),
        )
        .expect("chapter 1 exists");
    push(
        "update_chapter",
        args,
        json!(resident::MutationEffect::from(effect)),
    );

    // 5. lint()
    let snapshot = resident.lint();
    let lint_output = json!({
        "snapshotId": format!("{:016x}", snapshot.id.0),
        "summary": dto::map_lint_summary(snapshot.summary.clone()),
        "books": snapshot
            .books
            .iter()
            .map(|book| resident::BookLintSnapshot {
                source_key: book.source_key.as_str().to_string(),
                book: book.book.as_str().to_string(),
                source_hash: format!("{:016x}", book.source_hash.0),
                token_identity: format!("{:016x}", book.token_identity.0),
                findings: book
                    .result
                    .issues
                    .iter()
                    .cloned()
                    .map(dto::map_lint_issue)
                    .collect(),
                summary: dto::map_lint_summary(book.result.summary.clone()),
            })
            .collect::<Vec<_>>(),
    });
    drop(snapshot);
    push("lint", json!({}), lint_output);

    // 6. patches()
    let patches = resident.patches();
    push(
        "patches",
        json!({}),
        json!(
            patches
                .clone()
                .into_iter()
                .map(resident::Patch::from)
                .collect::<Vec<_>>()
        ),
    );
    let first_patch = patches.first().cloned().expect("GEN carries a fix");
    let patch_id_dto = resident::PatchId::from(first_patch.id);

    // 7. preview_patch(first)
    let preview = resident
        .preview_patch(first_patch.id)
        .expect("a fresh patch previews");
    push(
        "preview_patch",
        json!({ "id": patch_id_dto }),
        json!(
            preview
                .iter()
                .map(dto::map_format_token)
                .collect::<Vec<_>>()
        ),
    );

    // 8. apply_patch(first)
    let effect = resident
        .apply_patch(first_patch.id)
        .expect("a fresh patch applies");
    push(
        "apply_patch",
        json!({ "id": patch_id_dto }),
        json!(resident::MutationEffect::from(effect.clone())),
    );

    // 8b. error: apply the same (now-stale) patch id again
    let stale = resident.apply_patch(first_patch.id);
    push(
        "apply_patch_stale",
        json!({ "id": patch_id_dto }),
        json!(resident::PatchError::from(
            stale.expect_err("the same id cannot apply twice")
        )),
    );

    // 9. pull: to_tokens(effect.changed)
    let scopes: Vec<braid::Scope> = effect.changed.clone();
    let dto_scopes: Vec<resident::Scope> = scopes
        .iter()
        .map(|scope| resident::Scope {
            book: scope.book.as_str().to_string(),
            chapter: scope.chapter.as_ref().map(resident::ChapterLabel::from),
        })
        .collect();
    let pulled = resident.to_tokens(scopes).expect("changed scopes resolve");
    push(
        "to_tokens_pull",
        json!({ "scopes": dto_scopes }),
        json!(
            pulled
                .into_iter()
                .map(|scope| resident::ScopeTokens {
                    book: scope.book.as_str().to_string(),
                    chapter: scope.chapter.as_ref().map(resident::ChapterLabel::from),
                    tokens: scope.tokens.iter().map(dto::map_owned_token).collect(),
                })
                .collect::<Vec<_>>()
        ),
    );

    // 10. set_baseline(GEN, current bytes)
    let current_gen = match resident
        .to_usfm(braid::CorpusScope::Book(book_id("GEN")))
        .unwrap()
    {
        braid::ScopedOutput::Single(source) => source,
        braid::ScopedOutput::All(_) => unreachable!(),
    };
    let baseline_book = match lane {
        "usfm" => dto_book_usfm("01-GEN.usfm", "GEN", &current_gen),
        "tokens" => dto_book_tokens(
            "01-GEN.usfm",
            "GEN",
            &owned(&current_gen),
            resident::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let native_baseline_book = match lane {
        "usfm" => native_book_usfm("01-GEN.usfm", "GEN", &current_gen),
        "tokens" => native_book_tokens(
            "01-GEN.usfm",
            "GEN",
            owned(&current_gen),
            braid::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let effect = resident
        .set_baseline(native_baseline_book)
        .expect("GEN is resident");
    push(
        "set_baseline",
        json!({ "book": baseline_book }),
        json!(resident::MutationEffect::from(effect)),
    );

    // 10b. error: set_baseline on a book that is not resident
    push(
        "set_baseline_not_resident",
        json!({ "book": dto_book_usfm("99-QQQ.usfm", "QQQ", "\\id QQQ\n\\c 1\n\\p\n\\v 1 x\n") }),
        json!(resident::SetBaselineError::from(
            resident
                .set_baseline(native_book_usfm(
                    "99-QQQ.usfm",
                    "QQQ",
                    "\\id QQQ\n\\c 1\n\\p\n\\v 1 x\n"
                ))
                .expect_err("QQQ is not resident")
        )),
    );

    // 11. is_dirty(GEN) — clean, right after set_baseline
    push(
        "is_dirty_clean",
        json!({ "scope": { "kind": "book", "book": "GEN" } }),
        json!(
            resident
                .is_dirty(braid::CorpusScope::Book(book_id("GEN")))
                .unwrap()
        ),
    );

    // 12. update_book(GEN, edited) to diverge from the just-set baseline
    let further_edit = match lane {
        "usfm" => native_book_usfm("01-GEN.usfm", "GEN", GEN_EDITED_WHOLE),
        "tokens" => native_book_tokens(
            "01-GEN.usfm",
            "GEN",
            owned(GEN_EDITED_WHOLE),
            braid::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let further_edit_dto = match lane {
        "usfm" => dto_book_usfm("01-GEN.usfm", "GEN", GEN_EDITED_WHOLE),
        "tokens" => dto_book_tokens(
            "01-GEN.usfm",
            "GEN",
            &owned(GEN_EDITED_WHOLE),
            resident::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let effect = resident
        .update_book(further_edit)
        .expect("valid replacement");
    push(
        "update_book",
        json!({ "book": further_edit_dto }),
        json!(resident::MutationEffect::from(effect)),
    );

    // 13. is_dirty(GEN) — dirty now
    push(
        "is_dirty_dirty",
        json!({ "scope": { "kind": "book", "book": "GEN" } }),
        json!(
            resident
                .is_dirty(braid::CorpusScope::Book(book_id("GEN")))
                .unwrap()
        ),
    );

    // 14. diff_baseline(GEN)
    let diff = resident
        .diff_baseline(braid::CorpusScope::Book(book_id("GEN")))
        .expect("GEN has a baseline");
    push(
        "diff_baseline",
        json!({ "scope": { "kind": "book", "book": "GEN" } }),
        json!(match diff {
            braid::ScopedOutput::Single(skeleton) => resident::ScopedOutput::Single {
                value: dto::map_native_skeleton(
                    &skeleton,
                    dto::map_owned_token,
                    usfm_onion::diff::TextDiffMode::None,
                ),
            },
            braid::ScopedOutput::All(_) => unreachable!(),
        }),
    );

    // 15. prepare_format_patch(GEN) / apply_format_patch
    let preparation = resident
        .prepare_format_patch(
            braid::CorpusScope::Book(book_id("GEN")),
            usfm_onion::format::FormatOptions::all_enabled(),
        )
        .expect("scope resolves");
    push(
        "prepare_format_patch",
        json!({ "scope": { "kind": "book", "book": "GEN" }, "options": Value::Null }),
        json!(resident::PatchPreparation::from(preparation.clone())),
    );
    if let braid::PatchPreparation::Ready(id) = preparation {
        let format_id_dto = resident::FormatPatchId::from(id);
        let effect = resident
            .apply_format_patch(id)
            .expect("a fresh preparation applies");
        push(
            "apply_format_patch",
            json!({ "id": format_id_dto }),
            json!(resident::MutationEffect::from(effect)),
        );
    }

    // 16. vref_index(GEN)
    let index = resident
        .vref_index(braid::CorpusScope::Book(book_id("GEN")))
        .unwrap();
    push(
        "vref_index",
        json!({ "scope": { "kind": "book", "book": "GEN" } }),
        json!(match index {
            braid::ScopedOutput::Single(entries) => resident::ScopedOutput::Single {
                value: crate::VrefIndex(
                    entries
                        .into_iter()
                        .map(|entry| (entry.sid, dto::map_verse_projection(entry.projection)))
                        .collect(),
                ),
            },
            braid::ScopedOutput::All(_) => unreachable!(),
        }),
    );

    // 17. to_usfm(GEN)
    let usfm = resident
        .to_usfm(braid::CorpusScope::Book(book_id("GEN")))
        .unwrap();
    push(
        "to_usfm",
        json!({ "scope": { "kind": "book", "book": "GEN" } }),
        json!(match usfm {
            braid::ScopedOutput::Single(value) => resident::ScopedOutput::Single { value },
            braid::ScopedOutput::All(_) => unreachable!(),
        }),
    );

    // 18. remove_book(EXO)
    push(
        "remove_book",
        json!({ "book": "EXO" }),
        json!(resident::MutationEffect::from(
            resident.remove_book(book_id("EXO"))
        )),
    );

    // 19. clear()
    push(
        "clear",
        json!({}),
        json!(resident::MutationEffect::from(resident.clear())),
    );

    // ---- error cases with no useful place earlier in the sequence -----

    // ambiguous chapter: a fresh book with duplicate `\c 1` runs.
    let mut ambiguous = NativeBraid::new(braid_config(), minter());
    let dup_book = match lane {
        "usfm" => native_book_usfm("03-DUP.usfm", "DUP", DUP_SOURCE),
        "tokens" => native_book_tokens(
            "03-DUP.usfm",
            "DUP",
            owned(DUP_SOURCE),
            braid::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let dup_book_dto = match lane {
        "usfm" => dto_book_usfm("03-DUP.usfm", "DUP", DUP_SOURCE),
        "tokens" => dto_book_tokens(
            "03-DUP.usfm",
            "DUP",
            &owned(DUP_SOURCE),
            resident::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let seed_effect = ambiguous
        .replace_corpus(braid::CorpusInput::new(vec![dup_book]))
        .expect("DUP is a valid candidate on its own");
    // Recorded as its own step (rather than folded silently into the setup)
    // so the node script — which cannot construct a `braid::Braid` directly
    // and must replay this exact call against a fresh wasm instance — has a
    // scripted step to reproduce it from, instead of an assumption baked
    // into the runner.
    push(
        "update_chapter_ambiguous_seed",
        json!({ "corpus": { "books": [dup_book_dto] } }),
        json!(resident::MutationEffect::from(seed_effect)),
    );
    let result = ambiguous.update_chapter(
        native_chapter_target("DUP", "1"),
        braid::ChapterInput::Tokens(owned("\\c 1\n\\p\n\\v 1 x\n")),
    );
    push(
        "update_chapter_ambiguous",
        json!({
            "target": dto_chapter_target("DUP", "1"),
            "replacement": { "kind": "tokens", "tokens": dto_tokens(&owned("\\c 1\n\\p\n\\v 1 x\n")) },
        }),
        json!(resident::IngestError::from(
            result.expect_err("DUP's chapter 1 is ambiguous")
        )),
    );

    // malformed input: two declared books sharing one BookId.
    let mut malformed = NativeBraid::new(braid_config(), minter());
    let dup_declared = vec![
        native_book_usfm("a.usfm", "GEN", GEN_SOURCE),
        native_book_usfm("b.usfm", "GEN", GEN_SOURCE),
    ];
    let result = malformed.replace_corpus(braid::CorpusInput::new(dup_declared));
    push(
        "replace_corpus_malformed",
        json!({
            "corpus": {
                "books": [
                    dto_book_usfm("a.usfm", "GEN", GEN_SOURCE),
                    dto_book_usfm("b.usfm", "GEN", GEN_SOURCE),
                ],
            },
        }),
        json!(resident::IngestError::from(
            result.expect_err("two books cannot declare the same BookId")
        )),
    );

    // duplicate token id: the same token object appears twice.
    let mut colliding = NativeBraid::new(braid_config(), minter());
    let base_tokens = owned("\\id COL\n\\c 1\n\\p\n\\v 1 a\n");
    let mut dup_tokens = base_tokens.clone();
    dup_tokens.push(base_tokens[0].clone());
    let dto_dup_tokens = dto_tokens(&dup_tokens);
    let result = colliding.replace_corpus(braid::CorpusInput::new(vec![native_book_tokens(
        "col.usfm",
        "COL",
        dup_tokens,
        braid::LineEnding::Lf,
    )]));
    push(
        "replace_corpus_duplicate_token_id",
        json!({
            "corpus": {
                "books": [{
                    "kind": "tokens",
                    "sourceKey": "col.usfm",
                    "book": "COL",
                    "tokens": dto_dup_tokens,
                    "lineEnding": "lf",
                }],
            },
        }),
        json!(resident::IngestError::from(
            result.expect_err("a repeated token id is refused")
        )),
    );
}

/// Publish → restore → compare, both findings and summary — the packed
/// cold-open path `run_lane` never touches. Clean-room review found two
/// real defects (a zeroed restored summary; a mixed-stamp batch silently
/// adopting the first record's stamps) that escaped exactly because this
/// path was outside the parity gate; this is that gap closed.
///
/// Builds the `resident::Braid` wasm wrapper directly via a struct literal
/// (`Braid { inner }`), the same way `resident.rs`'s own test module does —
/// legitimate here for the identical reason the rest of this generator
/// drives the native crate directly: `Braid::new`'s public constructor takes
/// a `js_sys::Function`, which has no meaningful native behavior. Everything
/// past construction (`restore_corpus`, `lint`) is the real wasm-bound
/// method, not a mirror of it.
fn run_restore(lane: &str, steps: &mut Vec<Step>) {
    use usfm_onion_wire::corpus_codec::{
        CorpusSection, CorpusSectionInput, CorpusSectionTokens, EncodedCorpus, LintStamps,
        encode_corpus,
    };

    const SOURCE: &str = "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n\\v 2 And the earth.\\q1\n\\v 3 Light.\n";

    let mut publisher = NativeBraid::new(braid_config(), minter());
    let native_book = match lane {
        "usfm" => native_book_usfm("01-GEN.usfm", "GEN", SOURCE),
        "tokens" => native_book_tokens("01-GEN.usfm", "GEN", owned(SOURCE), braid::LineEnding::Lf),
        _ => unreachable!(),
    };
    publisher
        .replace_corpus(braid::CorpusInput::new(vec![native_book]))
        .expect("one book");
    let stamps = LintStamps {
        config_fingerprint: braid::LintConfigFingerprint::of(&publisher.config().lint).0,
        engine_stamp: braid::LintEngineStamp::current().0,
    };
    let snapshot = publisher.lint();
    let found = snapshot
        .books
        .iter()
        .find(|book| book.book == book_id("GEN"))
        .expect("GEN is resident");
    assert!(
        !found.result.issues.is_empty(),
        "{lane}: the fixture must carry a real finding for the restore gate to mean anything"
    );
    let EncodedCorpus { bytes, sources, .. } = encode_corpus(
        snapshot.id.0,
        Some(stamps),
        &[CorpusSection::Fresh(CorpusSectionInput {
            book: book_id("GEN"),
            tokens: CorpusSectionTokens::Owned {
                tokens: found.tokens,
            },
            findings: Some(found.result),
        })],
    )
    .expect("one book encodes");
    let source = sources
        .into_iter()
        .find(|(candidate, _)| *candidate == book_id("GEN"))
        .expect("GEN has a source")
        .1;
    drop(snapshot);

    // `Vec<u8>` args travel as plain JSON number arrays; the node side turns
    // them into a `Uint8Array` before constructing the real `RestoreRecord`,
    // exactly what a host reading bytes off disk would hand over.
    let args = json!({
        "records": [{
            "path": "01-GEN.usfm",
            "packed": bytes,
            "source": source.as_bytes(),
        }],
    });

    let mut reopened = resident::Braid {
        inner: NativeBraid::new(braid_config(), minter()),
        publication: usfm_onion_host::PublicationCache::default(),
    };
    let outcome = reopened.restore_corpus(vec![resident::RestoreRecord {
        path: "01-GEN.usfm".to_string(),
        packed: bytes,
        source: source.as_bytes().to_vec(),
    }]);
    let resident::ApiResult::Ok { value: report } = outcome.0 else {
        panic!("{lane}: a fresh, matching-stamp restore must succeed: {outcome:?}");
    };
    steps.push(Step {
        step: "restore_corpus".to_string(),
        lane: lane.to_string(),
        args: args.clone(),
        output: json!(report),
    });

    // The gate itself: a lint pass over the *restored* corpus must equal the
    // publisher's own — findings and summary both, not just one.
    let restored_snapshot = reopened.lint();
    steps.push(Step {
        step: "restore_corpus_then_lint".to_string(),
        lane: lane.to_string(),
        args,
        output: json!(restored_snapshot),
    });
}

fn suppressed_lint_options() -> DtoLintOptions {
    DtoLintOptions {
        scope: DtoLintScope::Book,
        enabled_codes: None,
        disabled_codes: Vec::new(),
        suppressed: vec![dto::LintSuppression {
            code: dto::LintCode::DuplicateVerseNumber,
            sid: "GEN 1:1".to_string(),
        }],
        allow_implicit_chapter_content_verse: false,
    }
}

fn suppressed_braid_config() -> braid::BraidConfig {
    braid::BraidConfig::new(dto::lint_options_into_native(suppressed_lint_options()))
}

/// The suppressed-config case the plain `run_restore` fixture (`suppressed:
/// []`) could not see: packed bytes carry no `suppressed_count` at all, so a
/// restoring config with any suppression configured must decline to prime a
/// cached summary rather than claim a stale `0` — this is the transcript-level
/// regression for that fix. Builds with `suppressed_braid_config()` on both
/// the publishing and the restoring side, since restore adoption always
/// judges packed findings against the *restoring* side's own live config, not
/// anything carried in the bytes.
///
/// Both `Braid`s here need a config the node side can't get from
/// `transcript.config` (that top-level config is what every other step in the
/// transcript shares, and must stay suppression-free for them) — so this
/// step's own `args.config` carries it, and the harness constructs the fresh
/// restoring instance from that instead of the shared default.
fn run_restore_suppressed(lane: &str, steps: &mut Vec<Step>) {
    use usfm_onion_wire::corpus_codec::{
        CorpusSection, CorpusSectionInput, CorpusSectionTokens, EncodedCorpus, LintStamps,
        encode_corpus,
    };

    const SOURCE: &str = "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n";

    let mut publisher = NativeBraid::new(suppressed_braid_config(), minter());
    let native_book = match lane {
        "usfm" => native_book_usfm("01-GEN.usfm", "GEN", SOURCE),
        "tokens" => native_book_tokens("01-GEN.usfm", "GEN", owned(SOURCE), braid::LineEnding::Lf),
        _ => unreachable!(),
    };
    publisher
        .replace_corpus(braid::CorpusInput::new(vec![native_book]))
        .expect("one book");
    let stamps = LintStamps {
        config_fingerprint: braid::LintConfigFingerprint::of(&publisher.config().lint).0,
        engine_stamp: braid::LintEngineStamp::current().0,
    };
    let snapshot = publisher.lint();
    let found = snapshot
        .books
        .iter()
        .find(|book| book.book == book_id("GEN"))
        .expect("GEN is resident");
    assert!(
        found.result.summary.suppressed_count >= 1,
        "{lane}: the fixture must actually suppress a finding for this case to mean anything"
    );
    let EncodedCorpus { bytes, sources, .. } = encode_corpus(
        snapshot.id.0,
        Some(stamps),
        &[CorpusSection::Fresh(CorpusSectionInput {
            book: book_id("GEN"),
            tokens: CorpusSectionTokens::Owned {
                tokens: found.tokens,
            },
            findings: Some(found.result),
        })],
    )
    .expect("one book encodes");
    let source = sources
        .into_iter()
        .find(|(candidate, _)| *candidate == book_id("GEN"))
        .expect("GEN has a source")
        .1;
    drop(snapshot);

    let args = json!({
        "config": { "lint": suppressed_lint_options() },
        "records": [{
            "path": "01-GEN.usfm",
            "packed": bytes,
            "source": source.as_bytes(),
        }],
    });

    let mut reopened = resident::Braid {
        inner: NativeBraid::new(suppressed_braid_config(), minter()),
        publication: usfm_onion_host::PublicationCache::default(),
    };
    let outcome = reopened.restore_corpus(vec![resident::RestoreRecord {
        path: "01-GEN.usfm".to_string(),
        packed: bytes,
        source: source.as_bytes().to_vec(),
    }]);
    let resident::ApiResult::Ok { value: report } = outcome.0 else {
        panic!("{lane}: a fresh, matching-stamp restore must still seed the book: {outcome:?}");
    };
    // The book seeds -- residency and lint-priming are independent -- and
    // this is not a rejection either: priming was never attempted, so there
    // is nothing to report as refused.
    steps.push(Step {
        step: "restore_corpus_suppressed".to_string(),
        lane: lane.to_string(),
        args: args.clone(),
        output: json!(report),
    });

    // The gate: the honest recompute must still return findings, and the
    // summary it recomputes -- including the suppressed count a cached
    // summary could never have supplied -- must match the original publish.
    let restored_snapshot = reopened.lint();
    steps.push(Step {
        step: "restore_corpus_suppressed_then_lint".to_string(),
        lane: lane.to_string(),
        args,
        output: json!(restored_snapshot),
    });
}

/// Pins `Braid::publish`'s wasm projection against the native adapter it
/// wraps: both sides run `PublicationCache::publish` over the identical
/// corpus and must produce byte-identical `corpus.bin` bytes, since the wasm
/// verb is meant to be a thin pass-through and nothing else. That equality
/// is asserted directly here, in Rust, before the fixture is ever written --
/// the recorded step below then pins the wasm-shaped *value* the node
/// comparator re-checks against the real, built package.
fn run_publish(lane: &str, steps: &mut Vec<Step>) {
    let native_book = match lane {
        "usfm" => native_book_usfm("01-GEN.usfm", "GEN", GEN_SOURCE),
        "tokens" => native_book_tokens(
            "01-GEN.usfm",
            "GEN",
            owned(GEN_SOURCE),
            braid::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let mut native_publisher = NativeBraid::new(braid_config(), minter());
    native_publisher
        .replace_corpus(braid::CorpusInput::new(vec![native_book]))
        .expect("one book");
    let mut native_cache = usfm_onion_host::PublicationCache::default();
    let native_publication =
        usfm_onion_host::publish_corpus(&mut native_publisher, &mut native_cache)
            .expect("native adapter publishes");

    let wasm_book = match lane {
        "usfm" => dto_book_usfm("01-GEN.usfm", "GEN", GEN_SOURCE),
        "tokens" => dto_book_tokens(
            "01-GEN.usfm",
            "GEN",
            &owned(GEN_SOURCE),
            resident::LineEnding::Lf,
        ),
        _ => unreachable!(),
    };
    let mut wasm_braid = resident::Braid {
        inner: NativeBraid::new(braid_config(), minter()),
        publication: usfm_onion_host::PublicationCache::default(),
    };
    let corpus_args = json!({ "corpus": { "books": [wasm_book.clone()] } });
    let outcome = wasm_braid.replace_corpus(resident::CorpusInput {
        books: vec![wasm_book],
    });
    let resident::ApiResult::Ok { value: seeded } = outcome.0 else {
        panic!("{lane}: publish_seed's replaceCorpus must succeed: {outcome:?}");
    };
    // A fresh instance needs seeding before `publish` means anything; recorded
    // as its own step (`publish_seed`, the `FRESH_INSTANCE_STEPS` member) so
    // `publish` itself can continue on that same instance rather than racing
    // a second, un-seeded fresh handle.
    steps.push(Step {
        step: "publish_seed".to_string(),
        lane: lane.to_string(),
        args: corpus_args,
        output: json!(seeded),
    });

    let resident::ApiResult::Ok { value: published } = wasm_braid.publish().0 else {
        panic!("{lane}: publish must succeed over a clean, freshly ingested corpus");
    };

    assert_eq!(
        published.bytes, native_publication.bytes,
        "{lane}: the wasm verb's bytes must be byte-identical to the native adapter's -- \
         it is meant to be a thin projection, nothing else"
    );

    steps.push(Step {
        step: "publish".to_string(),
        lane: lane.to_string(),
        args: json!({}),
        output: json!(published),
    });

    // Clean-room re-review P1: an empty `sourceKey` on a `restorePublishedCorpus`
    // record must classify as `{kind: "ingest", error: {kind:
    // "duplicateSourceKey", source: ""}}` -- the pre-extraction wasm
    // classification, reproduced through `usfm_onion_host::RestoreError::
    // EmptySourceKey` -- not silently reclassified as a decode defect. Pinned
    // here so a future change to that mapping shows up as a parity
    // divergence, not a silent drift.
    let empty_source_key_records = vec![resident::PublishedCorpusSource {
        book: "GEN".to_string(),
        source_key: String::new(),
        source: published
            .books
            .iter()
            .find(|book| book.book == "GEN")
            .and_then(|book| book.source.clone())
            .expect("GEN's freshly-encoded source")
            .into_bytes(),
    }];
    let empty_source_key_args = json!({
        "packed": published.bytes,
        "records": empty_source_key_records,
    });
    let mut restore_target = resident::Braid {
        inner: NativeBraid::new(braid_config(), minter()),
        publication: usfm_onion_host::PublicationCache::default(),
    };
    let error = restore_target
        .restore_published_corpus(published.bytes.clone(), empty_source_key_records)
        .0;
    let resident::ApiResult::Error { error } = error else {
        panic!("{lane}: an empty source key must refuse: {error:?}");
    };
    steps.push(Step {
        step: "restore_published_corpus_empty_source_key".to_string(),
        lane: lane.to_string(),
        args: empty_source_key_args,
        output: json!(error),
    });
}

/// Regenerates `tests/fixtures/parity-transcript.json`. Run explicitly; not
/// part of the normal `cargo test` battery, the same convention every other
/// fixture generator in this workspace follows.
#[test]
#[ignore = "regenerates the committed native/wasm parity transcript; run explicitly"]
fn generate_parity_transcript() {
    let mut steps = Vec::new();
    for lane in ["usfm", "tokens"] {
        run_lane(lane, &mut steps);
        run_restore(lane, &mut steps);
        run_restore_suppressed(lane, &mut steps);
        run_publish(lane, &mut steps);
    }
    // The config and minter shape travel with the transcript rather than
    // being re-derived by the node script by hand: both sides must
    // construct byte-for-byte the same `Braid`, and a hand-written mirror of
    // the config JSON is exactly the kind of duplication this packet's DTO
    // rule exists to prevent.
    let transcript = json!({
        "config": { "lint": lint_options() },
        "minter": "counter",
        "steps": steps,
    });
    let json = serde_json::to_string_pretty(&transcript).expect("transcript serializes");
    let path = fixture_path();
    fs::create_dir_all(path.parent().unwrap()).expect("fixtures directory");
    fs::write(&path, json).expect("transcript writes");
    eprintln!("wrote {} steps to {}", steps.len(), path.display());
}
