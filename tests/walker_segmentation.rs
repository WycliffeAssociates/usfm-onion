//! Proves the chapter-segmented walk is event-for-event identical to the whole-
//! stream walk. This is the correctness foundation every later chapter-parallel
//! stage rests on: if segment-then-concatenate reproduces the walker's exact
//! event sequence (kinds, indices, close reasons), then any visitor driven by
//! `walk_range` over `chapter_segments` produces the same output as today's
//! whole-book walk — regardless of thread count.
//!
//! The subtle hazard being guarded is the segment terminal: an isolated slice
//! must NOT drain as `EndOfInput`; non-final segments close open scopes with the
//! reasons an incoming `\c` produces (`ImplicitByOpen`/`RecoveryClosure`). See
//! `WalkBoundary`.

use usfm_onion::parse::parse;
use usfm_onion::token::Token;
use usfm_onion::walker::{
    ChapterSegment, LeaveReason, ScopeFrame, Visitor, WalkContext, chapter_segments, walk_range,
    walk_tokens,
};

/// Records every walker event as a compact tag. Token indices are included so a
/// segment-local index drift would show up as a mismatch; `walk_range` keeps
/// them absolute, so they must match the whole-stream walk.
#[derive(Default)]
struct Recorder {
    events: Vec<String>,
}

impl<'t, 's> Visitor<'t, Token<'s>> for Recorder {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'t, '_>,
        frame: &ScopeFrame<'t>,
        _token: &'t Token<'s>,
        token_index: usize,
    ) {
        self.events
            .push(format!("enter@{token_index} {}", frame.marker));
    }

    fn on_leave_scope(
        &mut self,
        _ctx: &WalkContext<'t, '_>,
        frame: &ScopeFrame<'t>,
        reason: LeaveReason,
    ) {
        self.events.push(format!(
            "leave@{} {} {reason:?}",
            frame.source_token_index, frame.marker
        ));
    }

    fn on_end_marker(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("endmarker@{i}"));
    }
    fn on_milestone(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("milestone@{i}"));
    }
    fn on_milestone_end(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("milestoneend@{i}"));
    }
    fn on_text(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("text@{i}"));
    }
    fn on_chapter(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("chapter@{i}"));
    }
    fn on_verse(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("verse@{i}"));
    }
    fn on_book_code(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("bookcode@{i}"));
    }
    fn on_opt_break(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("optbreak@{i}"));
    }
    fn on_newline(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("newline@{i}"));
    }
    fn on_other(&mut self, _c: &WalkContext<'t, '_>, _t: &'t Token<'s>, i: usize) {
        self.events.push(format!("other@{i}"));
    }
}

fn whole_events(tokens: &[Token<'_>]) -> Vec<String> {
    let mut rec = Recorder::default();
    walk_tokens(tokens, &mut rec);
    rec.events
}

fn segmented_events(tokens: &[Token<'_>]) -> Vec<String> {
    let mut rec = Recorder::default();
    for ChapterSegment {
        range, boundary, ..
    } in chapter_segments(tokens)
    {
        walk_range(tokens, range, boundary, &mut rec);
    }
    rec.events
}

#[track_caller]
fn assert_walk_equivalent(source: &str, label: &str) {
    let parsed = parse(source);
    let whole = whole_events(&parsed.tokens);
    let segmented = segmented_events(&parsed.tokens);

    if whole != segmented {
        let first = whole
            .iter()
            .zip(&segmented)
            .position(|(a, b)| a != b)
            .unwrap_or(whole.len().min(segmented.len()));
        panic!(
            "segmented walk diverged from whole-book walk for {label}\n\
             first diff at event {first}:\n  whole:     {:?}\n  segmented: {:?}\n\
             (whole {} events, segmented {} events)",
            whole.get(first),
            segmented.get(first),
            whole.len(),
            segmented.len()
        );
    }
}

#[test]
fn targeted_chapter_boundary_cases_are_equivalent() {
    // Each case leaves some scope open (or strays a closer) right at a `\c`, the
    // exact spot where a naive EOF drain would emit the wrong close reason.
    let cases: &[(&str, &str)] = &[
        (
            "open paragraph before \\c",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 open para\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "open character marker before \\c",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 \\nd Lord never closed\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "open (unterminated) note before \\c",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 text\\f + \\ft note not closed\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "self-closed milestone before \\c",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 \\qt-s\\*quote\\qt-e\\*\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "logical milestone pair crossing \\c",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 \\qt-s\\*Jesus said\n\\c 2\n\\p\n\\v 2 more\\qt-e\\*\n",
        ),
        (
            "stray close marker",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 text\\nd*\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "duplicate \\id after chapter one",
            "\\id GEN\n\\c 1\n\\p\n\\v 1 text\n\\id MRK\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "chapter at index zero (no front matter)",
            "\\c 1\n\\p\n\\v 1 no id or headers\n\\c 2\n\\p\n\\v 1 next\n",
        ),
        (
            "no chapters at all (single front segment)",
            "\\id GEN\n\\h Genesis\n\\mt Genesis\n\\ip intro paragraph never closed\n",
        ),
    ];

    for (label, source) in cases {
        assert_walk_equivalent(source, label);
    }
}

#[test]
fn all_testdata_fixtures_are_equivalent() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testData");
    let mut paths = Vec::new();
    collect_usfm(&root, &mut paths);
    paths.sort();
    assert!(!paths.is_empty(), "expected testData/**/*.usfm fixtures");

    for path in paths {
        let source = std::fs::read_to_string(&path).expect("read fixture");
        assert_walk_equivalent(&source, &path.to_string_lossy());
    }
}

fn collect_usfm(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
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
