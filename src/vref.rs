use std::collections::BTreeMap;

use crate::marker_defs::{StructuralScopeKind, marker_paragraph_supports_verse};
use crate::parse::parse;
use crate::token::{Sid, Token};
use crate::walker::{ScopeFrame, Visitor, WalkContext, walk_tokens};

pub type VrefMap = BTreeMap<String, String>;

pub fn usfm_to_vref_map(source: &str) -> VrefMap {
    let parsed = parse(source);
    tokens_to_vref_map(&parsed.tokens)
}

pub fn tokens_to_vref_map(tokens: &[Token<'_>]) -> VrefMap {
    let mut visitor = VrefVisitor::default();
    walk_tokens(tokens, &mut visitor);
    visitor.finish()
}

pub fn vref_map_to_json_string(map: &VrefMap) -> String {
    serde_json::to_string_pretty(map).expect("vref map should serialize")
}

#[derive(Debug, Default)]
struct VrefVisitor {
    map: VrefMap,
    current_ref: Option<String>,
    current_text: String,
    // Persists across paragraph boundaries: once any Block has been
    // seen, this reflects the latest Block's supports-verse status, even
    // after that Block closes. Matches the pre-walker behaviour.
    current_block_supports_verse: Option<bool>,
}

impl VrefVisitor {
    fn finish(mut self) -> VrefMap {
        self.flush_current_verse();
        self.map
    }

    fn flush_current_verse(&mut self) {
        let Some(reference) = self.current_ref.take() else {
            self.current_text.clear();
            return;
        };
        let trimmed = self.current_text.trim();
        if !trimmed.is_empty() {
            self.map.insert(reference, trimmed.to_string());
        }
        self.current_text.clear();
    }

    fn can_collect_text(&self, ctx: &WalkContext<'_, '_>) -> bool {
        self.current_ref.is_some()
            && !ctx.in_note()
            && self.current_block_supports_verse.unwrap_or(true)
    }
}

impl<'tokens, 'src> Visitor<'tokens, Token<'src>> for VrefVisitor {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if frame.scope_kind == StructuralScopeKind::Block {
            self.current_block_supports_verse = Some(marker_paragraph_supports_verse(frame.marker));
        }
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        self.flush_current_verse();
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        self.flush_current_verse();
        if let Some(reference) = verse_ref(token.sid, token.source) {
            self.current_ref = Some(reference);
        }
    }

    fn on_text(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if self.can_collect_text(ctx) {
            push_text(&mut self.current_text, token.source);
        }
    }
}

/// Build the `BOOK CHAPTER:VERSE` sid for a verse number token.
///
/// `lexeme` is the verbatim verse-number form from source (e.g. `"1"`,
/// `"1-2"`, `"1,3"`, `"1-3,5"`). For a bridge or sequence, the lexeme is
/// preserved as-is in the sid — matching the USX/USJ normative form
/// (`"GEN 1:1-2"`) and the convention used by usfm-grammar, usfmtc, and
/// usfm3. Consumers that need per-verse lookup against a bridged source
/// should split the lexeme themselves.
fn verse_ref(sid: Option<Sid>, lexeme: &str) -> Option<String> {
    let sid = sid?;
    let chapter = sid.chapter;
    if chapter == 0 {
        return None;
    }

    let verse_part = lexeme.trim();
    if verse_part.is_empty() {
        return None;
    }

    Some(format!("{} {}:{}", sid.book, chapter, verse_part))
}

fn push_text(current: &mut String, fragment: &str) {
    if current.is_empty() {
        current.push_str(fragment);
        return;
    }

    let current_ends_with_ws = current.chars().last().is_some_and(char::is_whitespace);
    let fragment_starts_with_ws = fragment.chars().next().is_some_and(char::is_whitespace);

    if current_ends_with_ws && fragment_starts_with_ws {
        current.push_str(fragment.trim_start());
    } else {
        current.push_str(fragment);
    }
}

#[cfg(test)]
mod tests {
    use super::{tokens_to_vref_map, usfm_to_vref_map, vref_map_to_json_string};
    use crate::parse::parse;

    #[test]
    fn basic_vref_extracts_plain_verse_text() {
        let map = usfm_to_vref_map(
            "\\id GEN Genesis\n\\c 1\n\\p\n\\v 1 In the beginning God created the heavens and the earth.\n\\v 2 The earth was without form and void.\n",
        );

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning God created the heavens and the earth.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("The earth was without form and void.")
        );
    }

    #[test]
    fn footnotes_are_stripped() {
        let map = usfm_to_vref_map(
            "\\id GEN\n\\c 1\n\\p\n\\v 1 Text \\f + \\fr 1:1 \\ft note text \\f* rest.",
        );
        let verse = map.get("GEN 1:1").map(String::as_str).unwrap_or("");

        assert!(verse.contains("Text"));
        assert!(verse.contains("rest."));
        assert!(!verse.contains("note text"));
    }

    #[test]
    fn section_headings_are_skipped() {
        let map =
            usfm_to_vref_map("\\id GEN\n\\c 1\n\\s1 The Creation\n\\p\n\\v 1 In the beginning.");

        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
    }

    #[test]
    fn verse_spanning_paragraphs_is_concatenated() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 First part.\n\\q1 Second part.");

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("First part. Second part.")
        );
    }

    #[test]
    fn root_level_verses_are_collected() {
        let parsed = parse("\\id GEN\n\\c 1\n\\v 1 In the beginning.\n\\v 2 And God said.");
        let map = tokens_to_vref_map(&parsed.tokens);

        assert_eq!(
            map.get("GEN 1:1").map(String::as_str),
            Some("In the beginning.")
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("And God said.")
        );
    }

    #[test]
    fn json_output_contains_refs_and_text() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.");
        let json = vref_map_to_json_string(&map);

        assert!(json.contains("\"GEN 1:1\""));
        assert!(json.contains("\"In the beginning.\""));
    }

    #[test]
    fn verse_bridge_produces_compound_sid() {
        // Matches USX/USJ normative form: \v 1-2 → sid="GEN 1:1-2".
        // usfm-grammar, usfmtc, and usfm3 all use the same compound key.
        let map = usfm_to_vref_map(
            "\\id GEN\n\\c 1\n\\p\n\\v 1-2 The first two verses share text.\n\\v 3 Third.",
        );
        assert_eq!(
            map.get("GEN 1:1-2").map(String::as_str),
            Some("The first two verses share text."),
        );
        assert_eq!(map.get("GEN 1:3").map(String::as_str), Some("Third."));
        assert!(
            !map.contains_key("GEN 1:1"),
            "bridge should not produce a separate GEN 1:1 entry; got keys: {:?}",
            map.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn verse_sequence_lexeme_is_preserved() {
        let map = usfm_to_vref_map("\\id GEN\n\\c 1\n\\p\n\\v 1,3 Combined.\n");
        assert_eq!(
            map.get("GEN 1:1,3").map(String::as_str),
            Some("Combined."),
        );
    }

    #[test]
    fn unclosed_footnote_does_not_swallow_subsequent_verses() {
        // Reproducer from the gitea-dashboard bug report
        // (2026-05-08-usfm-onion-unclosed-note-recovery): a single \f with
        // no \f* used to drop every subsequent verse and chapter from
        // to_vref() output.
        let src = "\\id GEN Sample\n\
                   \\c 1\n\
                   \\p\n\
                   \\v 1 First verse with an unclosed footnote.\\f + \\ft Note text never terminated.\n\
                   \\v 2 Second verse — should still appear.\n\
                   \\c 2\n\
                   \\p\n\
                   \\v 1 Chapter 2 should also still appear.\n";
        let map = usfm_to_vref_map(src);
        assert!(
            map.contains_key("GEN 1:1"),
            "v1 of ch1 missing: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(
            map.contains_key("GEN 1:2"),
            "v2 of ch1 dropped by unclosed-note bug: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(
            map.contains_key("GEN 2:1"),
            "ch2 dropped by unclosed-note bug: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("Second verse — should still appear."),
            "v2 text should not be polluted by note content",
        );
    }

    #[test]
    fn unclosed_footnote_with_nested_chars_recovers_at_next_paragraph() {
        // Mirrors the en_ulb Isaiah pattern: outer \f never closed, but
        // inner \fqa pairs are well-formed. The next \q1 paragraph marker
        // should close the outer note.
        let src = "\\id ISA\n\
                   \\c 33\n\
                   \\q2 Lebanon is ashamed and withers away;\\f + \\ft The word \\fqa mourns \\fqa* can be also be read as \\fqa dries up\\fqa*.\n\
                   \\q1 Sharon is like a desert plain;\n\
                   \\c 34\n\
                   \\p\n\
                   \\v 1 Chapter 34 should appear.\n";
        let map = usfm_to_vref_map(src);
        assert!(
            map.contains_key("ISA 34:1"),
            "ch34 dropped: {:?}",
            map.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn ulb_isaiah_full_corpus_all_chapters_present() {
        // Real-world reproducer from the bug report: en_ulb/23-ISA.usfm has
        // an unclosed \f at chapter 33 verse 9. Pre-fix, vref output stopped
        // at chapter 33 and dropped chapters 34-66 silently. Assert all
        // 66 chapters round-trip through to_vref.
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = repo_root.join("example-corpora/en_ulb/23-ISA.usfm");
        if !path.exists() {
            // Allow the test to no-op when the corpus isn't checked out.
            return;
        }
        let source = std::fs::read_to_string(&path).expect("isaiah should read");
        let map = usfm_to_vref_map(&source);

        let mut chapters_present = std::collections::BTreeSet::new();
        for key in map.keys() {
            // Key format: "ISA <chapter>:<verse-or-bridge>"
            if let Some(after_book) = key.strip_prefix("ISA ")
                && let Some((chapter_str, _)) = after_book.split_once(':')
                && let Ok(chapter) = chapter_str.parse::<u32>()
            {
                chapters_present.insert(chapter);
            }
        }

        assert_eq!(
            chapters_present.len(),
            66,
            "expected 66 chapters, got {} (last present: {:?})",
            chapters_present.len(),
            chapters_present.iter().next_back()
        );
        assert!(
            map.len() >= 1200,
            "expected ~1290 verse entries, got {}",
            map.len()
        );
    }

    #[test]
    fn unclosed_cross_reference_recovers_at_next_verse() {
        let src = "\\id GEN\n\
                   \\c 1\n\
                   \\p\n\
                   \\v 1 Text \\x - \\xt 1.1: ref text never closed.\n\
                   \\v 2 Second verse.\n";
        let map = usfm_to_vref_map(src);
        assert_eq!(
            map.get("GEN 1:2").map(String::as_str),
            Some("Second verse."),
        );
    }
}
