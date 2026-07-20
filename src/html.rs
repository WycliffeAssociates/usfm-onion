//! HTML rendering, driven by the unified walker.
//!
//! This module is a visitor over `crate::walker`'s events. All scope
//! tracking (Block precedence, sidebar pops, inline closes, unclosed-
//! note recovery) is the walker's responsibility — the visitor just
//! reacts to walker events with HTML emission. Notes are deferred:
//! the caller is captured from the first body text token (mirroring
//! `parse_note_tokens`' first-text-is-caller rule); `<aside>`
//! extraction or inline-span emission happens on
//! `on_leave_scope(Note)`. `\esbe` is modelled as a phantom frame
//! (occupies a walker scope slot but emits no HTML element).
//!
//! What remains here:
//! - HTML option types (`HtmlOptions`, `HtmlNoteMode`, `HtmlCallerStyle`,
//!   `HtmlCallerScope`) — unchanged public API.
//! - The element-stack buffering pattern (`OpenElement` + buffer) so
//!   children render into parents' buffers and the final HTML strings
//!   compose correctly.
//! - Note extraction: when a `Note` scope opens in `Extracted` mode,
//!   the body buffers separately and lands in a `<aside>` in either
//!   `linkedFootnotes` or `linkedCrossrefs` on scope close.
//! - Table synthesis: orphan `\tr` / `\tc` markers get wrapped in
//!   synthetic `<table>` / `<tr>` elements.

use crate::marker_defs::{
    BlockBehavior, MarkerDefKind, NoteFamily, NoteSubkind, StructuralScopeKind,
    marker_block_behavior, marker_note_family, marker_note_subkind,
};
use crate::parse::parse;
use crate::token::{AttributeItem, Token, TokenData, TokenId};
use crate::walker::{LeaveReason, ScopeFrame, Visitor, WalkContext, walk_tokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlNoteMode {
    Extracted,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlCallerStyle {
    Numeric,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlCallerScope {
    DocumentSequential,
    VerseSequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlOptions {
    pub wrap_root: bool,
    pub prefer_native_elements: bool,
    pub note_mode: HtmlNoteMode,
    pub caller_style: HtmlCallerStyle,
    pub caller_scope: HtmlCallerScope,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            wrap_root: false,
            prefer_native_elements: true,
            note_mode: HtmlNoteMode::Extracted,
            caller_style: HtmlCallerStyle::Numeric,
            caller_scope: HtmlCallerScope::VerseSequential,
        }
    }
}

/// Token count at or above which rendering fans out chapter-parallel. Below it the
/// fixed decomposition cost (partitioning, the note-count prepass, and the ordered
/// merge) outweighs the parallel gain, so the walk stays serial. Large books clear
/// it comfortably; a book below it renders in well under a millisecond either way.
#[cfg(not(target_arch = "wasm32"))]
const PARALLEL_MIN_TOKENS: usize = 20_000;

pub fn tokens_to_html(tokens: &[Token<'_>], options: HtmlOptions) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    if tokens.len() >= PARALLEL_MIN_TOKENS {
        return html_partitioned(tokens, options);
    }
    let mut visitor = HtmlVisitor::new(options);
    walk_tokens(tokens, &mut visitor);
    wrap_document(visitor.finalize(), options)
}

/// Apply the optional root wrapper once, around the fully assembled body (note
/// sections included). Shared by the serial and partitioned paths so both wrap
/// identically.
fn wrap_document(body: String, options: HtmlOptions) -> String {
    if options.wrap_root {
        format!(r#"<div data-usfm-root="true">{body}</div>"#)
    } else {
        body
    }
}

/// Append the extracted-note sections to `out` exactly as [`HtmlVisitor::finalize`]
/// does. Shared so the serial finalize and the partitioned merge emit byte-identical
/// section markup.
fn append_note_sections(out: &mut String, footnotes: &[String], crossrefs: &[String]) {
    if !footnotes.is_empty() {
        out.push_str(r#"<section id="linkedFootnotes" data-usfm-notes="footnotes">"#);
        for note in footnotes {
            out.push_str(note);
        }
        out.push_str("</section>");
    }
    if !crossrefs.is_empty() {
        out.push_str(r#"<section id="linkedCrossrefs" data-usfm-notes="crossrefs">"#);
        for note in crossrefs {
            out.push_str(note);
        }
        out.push_str("</section>");
    }
}

/// Partition the stream at `\c` and render each segment through the ordered-map
/// seam, then concatenate bodies and append the extracted-note sections once.
/// Byte-identical to the serial walk regardless of thread count.
///
/// A `\c` closes every open scope, so no element and no note body spans a segment
/// boundary — each segment's body, footnote asides, and crossref asides are
/// self-contained and concatenate in order. The only whole-document state a serial
/// walk threads across `\c` is three monotonic note counters: the extracted-note
/// anchor ids (`fn-N`/`xr-N`, live in every mode with extracted notes, including the
/// default) and the document-sequential caller number (live under
/// `DocumentSequential`, and for pre-verse notes under `VerseSequential`). None can
/// be repaired after the fact — they are baked into the rendered bytes — so a first
/// cheap pass counts each segment's counter increments (walking with string emission
/// suppressed), a serial fold turns those into exclusive-prefix starting values, and
/// the render pass seeds each segment with its starting counters. Verse-scoped
/// caller numbering resets at every `\c`, so it needs no seed. The corpus test below
/// calls this directly to force the merge on small fixtures across the option matrix.
#[cfg(not(target_arch = "wasm32"))]
fn html_partitioned(tokens: &[Token<'_>], options: HtmlOptions) -> String {
    let segments = crate::walker::chapter_segments(tokens);

    // Phase 1: count each segment's note-counter increments, cheaply (no strings).
    let counts = crate::par::map_ordered(&segments, |segment| {
        let mut visitor = HtmlVisitor::new_counting(options);
        crate::walker::walk_range(tokens, segment.range.clone(), segment.boundary, &mut visitor);
        visitor.into_counts()
    });

    // Serial: exclusive-prefix offsets — a segment starts each counter at the sum of
    // all earlier segments' increments, exactly where the serial walk would be.
    let mut seeds = Vec::with_capacity(segments.len());
    let mut acc = HtmlSeed::default();
    for count in &counts {
        seeds.push(acc);
        acc.footnote_id += count.footnote_id;
        acc.crossref_id += count.crossref_id;
        acc.document_note_count += count.document_note_count;
    }

    // Phase 2: render each segment seeded with its starting counters.
    let units: Vec<_> = segments
        .iter()
        .zip(seeds)
        .map(|(segment, seed)| (segment.range.clone(), segment.boundary, seed))
        .collect();
    let fragments = crate::par::map_ordered(&units, |(range, boundary, seed)| {
        let mut visitor = HtmlVisitor::new_seeded(options, *seed);
        crate::walker::walk_range(tokens, range.clone(), *boundary, &mut visitor);
        visitor.into_fragment()
    });

    // Merge: concatenate bodies in order, then append each note section once.
    let body_len: usize = fragments.iter().map(|f| f.body.len()).sum();
    let mut out = String::with_capacity(body_len);
    let mut footnotes = Vec::new();
    let mut crossrefs = Vec::new();
    for mut fragment in fragments {
        out.push_str(&fragment.body);
        footnotes.append(&mut fragment.footnotes);
        crossrefs.append(&mut fragment.crossrefs);
    }
    append_note_sections(&mut out, &footnotes, &crossrefs);
    wrap_document(out, options)
}

/// A segment's starting note counters (in the render pass) or its counter increments
/// (as reported by the count pass) — the same three-integer shape serves both, since
/// a prefix fold turns increments into starting values. See [`html_partitioned`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default)]
struct HtmlSeed {
    footnote_id: usize,
    crossref_id: usize,
    document_note_count: usize,
}

/// One rendered segment: its body plus the extracted-note asides it produced, kept
/// separate so the merge can append all footnote and crossref sections once, after
/// every body.
#[cfg(not(target_arch = "wasm32"))]
struct HtmlFragment {
    body: String,
    footnotes: Vec<String>,
    crossrefs: Vec<String>,
}

pub fn usfm_to_html(source: &str, options: HtmlOptions) -> String {
    let parsed = parse(source);
    tokens_to_html(&parsed.tokens, options)
}

// =============================================================================
// Visitor state
// =============================================================================

#[derive(Clone)]
struct OpenElement<'a> {
    /// How this element handles emission. See `Emission` for the cases.
    emission: Emission,
    marker: Option<&'a str>,
    tag: &'static str,
    attrs: Vec<(String, String)>,
    buffer: String,
    scope_kind: StructuralScopeKind,
    /// Reserved for future inspection of note-classified frames
    /// (USJ/USX migrations may want this).
    #[allow(dead_code)]
    note_subkind: Option<NoteSubkind>,
    #[allow(dead_code)]
    note_family: Option<NoteFamily>,
    synthetic: bool,
}

/// How an `OpenElement` participates in HTML emission.
///
/// The walker tells us when scopes open and close. HTML, though, has
/// per-scope behavior that doesn't always correspond 1:1 to a single
/// `<tag>…</tag>` pair — notes get rerouted; some markers (`\esbe`)
/// open a scope in the walker but emit nothing in HTML. `Emission`
/// records what to do when the element closes.
#[derive(Clone)]
enum Emission {
    /// Standard `<tag attrs>buffer</tag>` rendered into the parent.
    Element,
    /// No element is emitted on close. Used for walker scope frames
    /// that exist for structural bookkeeping but don't correspond to
    /// any HTML element (e.g. `\esbe`, which only signals "close the
    /// open sidebar").
    Phantom,
    /// Note rendered as inline `<span>` containing `<sup>label</sup>`
    /// then the buffer.
    InlineNote(NoteState),
    /// Note rendered into the footnotes/crossrefs `<aside>` collection.
    /// The caller already emitted the `<sup><a></a></sup>` link into
    /// the parent buffer when the note opened.
    ExtractedNote(NoteState),
}

#[derive(Clone)]
struct NoteState {
    marker: String,
    kind: NoteKind,
    label: String,
    source_caller: String,
    /// For `ExtractedNote`: `<sup>` anchor id and target `<aside>` id.
    call_id: Option<String>,
    note_id: Option<String>,
    /// Stringified token id of the opening `\f` / `\x` marker.
    token_id: String,
    /// The note's leading text token (caller `"+"`, `"a"`, etc.) is
    /// consumed for labeling. This flag lets `on_text` discard the
    /// first non-empty text once after entering.
    caller_consumed: bool,
}

struct HtmlVisitor<'tokens> {
    options: HtmlOptions,
    /// Top-level output (whatever isn't inside any open element).
    output: String,
    /// Open-element stack. Mirrors the walker's scope stack 1:1,
    /// with the addition that markers like `\esbe` that don't emit
    /// HTML still occupy a stack slot (with `Emission::Phantom`).
    stack: Vec<OpenElement<'tokens>>,
    /// Note collections appended at finalize time.
    footnotes: Vec<String>,
    crossrefs: Vec<String>,
    /// Note label state.
    current_verse: Option<String>,
    note_count_in_verse: usize,
    document_note_count: usize,
    /// Monotonic ids for extracted-note anchors.
    footnote_id: usize,
    crossref_id: usize,
    /// Count-only mode for the partitioned path's first pass: the walk runs and all
    /// counter/scope state advances exactly as in a render, but string emission is
    /// suppressed so the pass is cheap. Always `false` on the serial and render
    /// paths. See [`html_partitioned`].
    counting: bool,
}

impl<'tokens> HtmlVisitor<'tokens> {
    fn new(options: HtmlOptions) -> Self {
        Self {
            options,
            output: String::new(),
            stack: Vec::new(),
            footnotes: Vec::new(),
            crossrefs: Vec::new(),
            current_verse: None,
            note_count_in_verse: 0,
            document_note_count: 0,
            footnote_id: 0,
            crossref_id: 0,
            counting: false,
        }
    }

    /// A count-pass visitor: same options, string emission suppressed.
    #[cfg(not(target_arch = "wasm32"))]
    fn new_counting(options: HtmlOptions) -> Self {
        Self {
            counting: true,
            ..Self::new(options)
        }
    }

    /// A render-pass visitor whose whole-document note counters start at `seed`.
    #[cfg(not(target_arch = "wasm32"))]
    fn new_seeded(options: HtmlOptions, seed: HtmlSeed) -> Self {
        Self {
            footnote_id: seed.footnote_id,
            crossref_id: seed.crossref_id,
            document_note_count: seed.document_note_count,
            ..Self::new(options)
        }
    }

    /// The count pass's product: how far each whole-document counter advanced over
    /// this segment (the visitor started at zero). Byte spans and verse-scoped
    /// numbering are segment-local and need no reporting.
    #[cfg(not(target_arch = "wasm32"))]
    fn into_counts(self) -> HtmlSeed {
        HtmlSeed {
            footnote_id: self.footnote_id,
            crossref_id: self.crossref_id,
            document_note_count: self.document_note_count,
        }
    }

    /// The render pass's product: the segment body plus its extracted-note asides,
    /// unwrapped (no note sections, no root wrapper — the merge applies those once).
    /// Mirrors [`Self::finalize`]'s belt-and-suspenders stack drain.
    #[cfg(not(target_arch = "wasm32"))]
    fn into_fragment(mut self) -> HtmlFragment {
        while let Some(item) = self.stack.pop() {
            emit_close(&mut self.output, &mut self.stack, item);
        }
        HtmlFragment {
            body: self.output,
            footnotes: self.footnotes,
            crossrefs: self.crossrefs,
        }
    }

    fn finalize(mut self) -> String {
        // Anything still on the stack at end of walk drains via the
        // walker's EOF events; we just trust the events fired. But
        // belt-and-suspenders: if anything remains (shouldn't), close
        // it out.
        while let Some(item) = self.stack.pop() {
            emit_close(&mut self.output, &mut self.stack, item);
        }

        let mut out = std::mem::take(&mut self.output);
        append_note_sections(&mut out, &self.footnotes, &self.crossrefs);
        out
    }

    /// True when the innermost active scope is a note body (whether
    /// inline or extracted). Used to suppress the chapter/verse spans
    /// from rendering inside note bodies, matching pre-walker behavior
    /// where the recursive `render_fragment` call rendered notes with
    /// `in_note_body = true`.
    fn in_note_body(&self) -> bool {
        self.stack.iter().any(|item| {
            matches!(
                item.emission,
                Emission::InlineNote(_) | Emission::ExtractedNote(_)
            )
        })
    }

    fn push_html(&mut self, html: &str) {
        push_fragment(&mut self.output, &mut self.stack, html);
    }

    fn note_label(&mut self, source_caller: &str) -> String {
        if matches!(self.options.caller_style, HtmlCallerStyle::Source) && !source_caller.is_empty()
        {
            return source_caller.to_string();
        }

        match self.options.caller_scope {
            HtmlCallerScope::DocumentSequential => {
                self.document_note_count += 1;
                format_ordinal(self.document_note_count, self.options.caller_style)
            }
            HtmlCallerScope::VerseSequential => {
                if let Some(verse) = self.current_verse.as_deref() {
                    self.note_count_in_verse += 1;
                    format!(
                        "{}.{}",
                        verse,
                        format_ordinal(self.note_count_in_verse, self.options.caller_style)
                    )
                } else {
                    self.document_note_count += 1;
                    format_ordinal(self.document_note_count, self.options.caller_style)
                }
            }
        }
    }
}

// =============================================================================
// Visitor impl: each walker event maps to one HTML responsibility.
// =============================================================================

impl<'tokens, 'src: 'tokens> Visitor<'tokens, Token<'src>> for HtmlVisitor<'tokens> {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        let marker = frame.marker;
        let scope_kind = frame.scope_kind;
        let kind = match &token.data {
            TokenData::Marker { metadata, .. } | TokenData::Milestone { metadata, .. } => {
                metadata.kind
            }
            _ => None,
        };

        // Sidebar end markers (`\esbe`) open a Sidebar scope in the
        // walker, but in HTML they only signal "close the sidebar".
        // The walker has already popped the previous Sidebar frame via
        // its precedence rules and fired on_leave_scope for it. We
        // record the incoming frame as a phantom so its eventual
        // on_leave_scope is also a no-op.
        if marker_is_sidebar_end(marker, kind) {
            self.stack.push(OpenElement {
                emission: Emission::Phantom,
                marker: Some(marker),
                tag: "",
                attrs: Vec::new(),
                buffer: String::new(),
                scope_kind,
                note_subkind: None,
                note_family: None,
                synthetic: false,
            });
            return;
        }

        // Chapter and Verse scopes don't open container elements in
        // HTML. The corresponding empty `<span>` is emitted from
        // `on_chapter` / `on_verse` once the number is known. We still
        // push a phantom frame so the matching on_leave_scope is
        // ignored (and so the walker's structural depth is mirrored).
        if matches!(
            scope_kind,
            StructuralScopeKind::Chapter | StructuralScopeKind::Verse
        ) {
            self.stack.push(OpenElement {
                emission: Emission::Phantom,
                marker: Some(marker),
                tag: "",
                attrs: Vec::new(),
                buffer: String::new(),
                scope_kind,
                note_subkind: None,
                note_family: None,
                synthetic: false,
            });
            if scope_kind == StructuralScopeKind::Chapter {
                // Reset verse-scoped state at the chapter boundary, as
                // the prior implementation did when handling `\c`.
                self.current_verse = None;
                self.note_count_in_verse = 0;
            }
            return;
        }

        // Note scopes get diverted: build a NoteState now (so we can
        // emit the caller HTML immediately for extracted mode), push
        // an Emission::{Inline,Extracted}Note frame, and consume the
        // first body text as the caller.
        if matches!(scope_kind, StructuralScopeKind::Note) {
            self.open_note(token, marker);
            return;
        }

        // Table synthesis: orphan `\tr` and `\tc` markers need their
        // wrapping `<table>` / `<tr>` synthesised.
        match scope_kind {
            StructuralScopeKind::TableRow => {
                ensure_table_open(&mut self.stack, self.options.prefer_native_elements);
            }
            StructuralScopeKind::TableCell => {
                if !self
                    .stack
                    .iter()
                    .any(|item| item.scope_kind == StructuralScopeKind::TableRow)
                {
                    ensure_table_open(&mut self.stack, self.options.prefer_native_elements);
                    self.stack.push(synthetic_table_row());
                }
            }
            _ => {}
        }

        // Count pass: only the frame's scope kind matters (for note/table nesting);
        // the tag and attribute strings are emission we skip, so don't build them.
        if self.counting {
            self.stack.push(OpenElement {
                emission: Emission::Element,
                marker: Some(marker),
                tag: "",
                attrs: Vec::new(),
                buffer: String::new(),
                scope_kind,
                note_subkind: None,
                note_family: None,
                synthetic: false,
            });
            return;
        }

        // Standard element. Compute tag/attrs and push.
        let (tag, data_type) = tag_and_type_for_marker(
            marker,
            kind,
            scope_kind,
            self.options.prefer_native_elements,
        );
        let mut attrs = common_marker_attrs(data_type, marker);
        if scope_kind == StructuralScopeKind::Unknown {
            attrs.push(("data-unknown-marker".to_string(), marker.to_string()));
        }
        attrs.push(("data-usfm-id".to_string(), token_id_str(&token.id)));
        if scope_kind == StructuralScopeKind::TableCell {
            attrs.push((
                "data-usfm-align".to_string(),
                table_cell_align(marker).to_string(),
            ));
        }
        if let Some(entries) = token.attributes()
            && !entries.is_empty()
        {
            push_attribute_entries(&mut attrs, entries);
        }

        self.stack.push(OpenElement {
            emission: Emission::Element,
            marker: Some(marker),
            tag,
            attrs,
            buffer: String::new(),
            scope_kind,
            note_subkind: marker_note_subkind(marker),
            note_family: marker_note_family(marker),
            synthetic: false,
        });
    }

    fn on_leave_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _frame: &ScopeFrame<'tokens>,
        _reason: LeaveReason,
    ) {
        let Some(item) = self.stack.pop() else {
            return;
        };
        // Count pass: the pop above keeps scope nesting correct (note detection
        // depends on it); the aside/element bytes are what we skip. Extracted-note
        // frames were already counted when their caller was consumed.
        if self.counting {
            return;
        }
        match item.emission {
            Emission::ExtractedNote(ref state) => {
                let call_id = state.call_id.as_deref().unwrap_or_default();
                let note_id = state.note_id.as_deref().unwrap_or_default();
                let aside = render_extracted_note(
                    &state.marker,
                    state.kind,
                    &state.source_caller,
                    &state.label,
                    call_id,
                    note_id,
                    &state.token_id,
                    &item.buffer,
                );
                match state.kind {
                    NoteKind::Footnote => self.footnotes.push(aside),
                    NoteKind::Crossref => self.crossrefs.push(aside),
                }
            }
            _ => emit_close(&mut self.output, &mut self.stack, item),
        }
    }

    fn on_end_marker(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // The corresponding `on_leave_scope(Explicit)` already closed
        // the element. The EndMarker token itself produces no HTML.
    }

    fn on_milestone(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // Milestone open is handled via on_enter_scope; this fallback
        // catches stray milestones the walker didn't classify, which
        // currently produces no HTML (matching prior behaviour).
    }

    fn on_milestone_end(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // Same as on_end_marker — the matching close already fired.
    }

    fn on_text(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // If we're directly inside a note whose caller hasn't been
        // captured yet, the next Text token IS the caller. Consume
        // it (don't add to body) and emit the caller HTML now that
        // the label is computable. Mirrors `parse_note_tokens`'
        // first-text-is-caller rule from the prior implementation.
        let needs_caller = matches!(
            self.stack.last().map(|e| &e.emission),
            Some(Emission::InlineNote(s) | Emission::ExtractedNote(s)) if !s.caller_consumed
        );
        if needs_caller {
            let raw = token.source;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                // Pre-caller whitespace — drop, matching prior behavior.
                return;
            }
            self.consume_note_caller(trimmed);
            return;
        }

        // Count pass: body text carries no counter — skip escaping and emission.
        if self.counting {
            return;
        }
        push_fragment(
            &mut self.output,
            &mut self.stack,
            &escape_html(token.source),
        );
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if self.in_note_body() {
            return;
        }
        // Count pass: the chapter span carries no counter (verse-scoped numbering
        // reset lives in `on_enter_scope(Chapter)`, which always runs).
        if self.counting {
            return;
        }
        // Find the chapter marker just below us on the stack. The
        // walker placed a Phantom frame for `\c` immediately before
        // dispatching this number token.
        let marker = self
            .stack
            .iter()
            .rev()
            .find_map(|item| {
                (item.scope_kind == StructuralScopeKind::Chapter).then_some(item.marker)
            })
            .flatten()
            .unwrap_or("c");
        let number = number_text(token);
        let sid = token
            .sid
            .as_ref()
            .map(|s| format!("{} {}", s.book, s.chapter));
        let token_id = token_id_str(&token.id);
        // Per the prior implementation, `\c` resets verse-scoped note
        // numbering. `on_enter_scope(Chapter)` already did that; the
        // number itself just emits the span.
        let html = empty_marker_span("chapter", marker, &number, sid.as_deref(), &token_id);
        self.push_html(&html);
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if self.in_note_body() {
            return;
        }
        let number = number_text(token);
        // Verse-scoped caller numbering resets here and gates note labels — it must
        // advance in the count pass too, so set it before the count-pass short-circuit.
        self.current_verse = (!number.is_empty()).then_some(number.clone());
        self.note_count_in_verse = 0;
        if self.counting {
            return;
        }
        let marker = self
            .stack
            .iter()
            .rev()
            .find_map(|item| (item.scope_kind == StructuralScopeKind::Verse).then_some(item.marker))
            .flatten()
            .unwrap_or("v");
        let sid = token
            .sid
            .as_ref()
            .map(|s| format!("{} {}:{}", s.book, s.chapter, s.verse_locator()));
        let token_id = token_id_str(&token.id);
        let html = empty_marker_span("verse", marker, &number, sid.as_deref(), &token_id);
        self.push_html(&html);
    }

    fn on_book_code(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        let TokenData::BookCode { code, .. } = &token.data else {
            return;
        };
        // Count pass: book scope opens no note and touches no counter (its element
        // emission is all that happens here), so it need not run.
        if self.counting {
            return;
        }
        // If the current stack already has a book-Header element open
        // (e.g. from `\id GEN`), do nothing — the existing element
        // covers the book scope.
        let already_in_book = self.stack.iter().any(|item| {
            item.scope_kind == StructuralScopeKind::Header
                && item
                    .attrs
                    .iter()
                    .any(|(key, value)| key == "data-usfm-type" && value == "book")
        });
        if already_in_book {
            return;
        }
        // Otherwise close any blocks and open a book element.
        close_for_new_block(&mut self.output, &mut self.stack, false);
        self.stack.push(open_book_element(
            "id",
            code,
            self.options.prefer_native_elements,
            token_id_str(&token.id),
        ));
    }

    fn on_opt_break(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        if self.counting {
            return;
        }
        self.push_html("<wbr>");
    }

    fn on_newline(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // Newlines have no HTML representation.
    }

    fn on_other(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens Token<'src>,
        _token_index: usize,
    ) {
        // Numbers outside chapter/verse, unmatched end markers, etc.
        // Render nothing — prior behaviour was to drop these silently
        // unless they had a structural role.
    }
}

// =============================================================================
// Note opening (extracted vs inline)
// =============================================================================

impl<'tokens> HtmlVisitor<'tokens> {
    /// Push a note frame onto the stack. We do **not** emit the caller
    /// HTML yet — the source caller is the first text token inside the
    /// note (see `parse_note_tokens` in the pre-walker code) and we
    /// don't know it until `on_text` fires. `consume_note_caller`
    /// finishes the open-time work (label, id assignment, caller HTML
    /// emission for extracted mode) when that first text arrives.
    fn open_note<'src>(&mut self, token: &'tokens Token<'src>, marker: &'tokens str) {
        let token_id = token_id_str(&token.id);
        let note_kind_enum = note_kind(marker);
        let inline = matches!(self.options.note_mode, HtmlNoteMode::Inline) || self.in_note_body();

        let state = NoteState {
            marker: marker.to_string(),
            kind: note_kind_enum,
            // Label/source-caller/ids fill in once the first text
            // arrives. Until then they hold empty/None values.
            label: String::new(),
            source_caller: String::new(),
            call_id: None,
            note_id: None,
            token_id,
            caller_consumed: false,
        };

        let emission = if inline {
            Emission::InlineNote(state)
        } else {
            Emission::ExtractedNote(state)
        };

        self.stack.push(OpenElement {
            emission,
            marker: Some(marker),
            tag: "",
            attrs: Vec::new(),
            buffer: String::new(),
            scope_kind: StructuralScopeKind::Note,
            note_subkind: marker_note_subkind(marker),
            note_family: marker_note_family(marker),
            synthetic: false,
        });
    }

    /// Called when the first non-empty text token inside an open note
    /// arrives. Finalises the note state (label, ids), emits the
    /// caller HTML into the parent buffer (extracted mode), and marks
    /// the caller as consumed so subsequent text routes to the body.
    fn consume_note_caller(&mut self, source_caller: &str) {
        // Read what we need about the top frame without holding a
        // mutable borrow across the label computation.
        let stack_len = self.stack.len();
        let (is_extracted, kind) = {
            let top = self.stack.last().expect("note frame present");
            match &top.emission {
                Emission::ExtractedNote(s) => (true, s.kind),
                Emission::InlineNote(s) => (false, s.kind),
                _ => return,
            }
        };

        let label = self.note_label(source_caller);

        // For extracted notes, assign monotonic ids.
        let (call_id, note_id) = if is_extracted {
            let id_index = match kind {
                NoteKind::Footnote => {
                    self.footnote_id += 1;
                    self.footnote_id
                }
                NoteKind::Crossref => {
                    self.crossref_id += 1;
                    self.crossref_id
                }
            };
            let pair = match kind {
                NoteKind::Footnote => (format!("fnref-{id_index}"), format!("fn-{id_index}")),
                NoteKind::Crossref => (format!("xrref-{id_index}"), format!("xr-{id_index}")),
            };
            (Some(pair.0), Some(pair.1))
        } else {
            (None, None)
        };

        // Update the top frame's note state. Borrow scope kept tight.
        let token_id_for_caller;
        let label_for_caller;
        let source_caller_for_caller;
        let call_id_for_caller;
        let note_id_for_caller;
        let marker_for_inline;
        {
            let top = self.stack.last_mut().expect("note frame present");
            match &mut top.emission {
                Emission::ExtractedNote(state) | Emission::InlineNote(state) => {
                    state.source_caller = source_caller.to_string();
                    state.label = label.clone();
                    state.call_id = call_id.clone();
                    state.note_id = note_id.clone();
                    state.caller_consumed = true;
                    token_id_for_caller = state.token_id.clone();
                    label_for_caller = state.label.clone();
                    source_caller_for_caller = state.source_caller.clone();
                    call_id_for_caller = state.call_id.clone();
                    note_id_for_caller = state.note_id.clone();
                }
                _ => unreachable!("checked above"),
            }
            marker_for_inline = top.marker.unwrap_or("");
        }

        // Count pass: the counters (footnote/crossref ids, document note count) and
        // the frame's caller state are all set above — the caller/label markup we
        // skip is what the seed later reproduces.
        if self.counting {
            return;
        }

        if is_extracted {
            // Emit `<sup><a>…</a></sup>` into the parent buffer (the
            // note frame is on top; parent is one below).
            let mut caller_html = String::from("<sup><a");
            let call_id_str = call_id_for_caller.as_deref().unwrap_or_default();
            let note_id_str = note_id_for_caller.as_deref().unwrap_or_default();
            push_attr(&mut caller_html, "href", &format!("#{note_id_str}"));
            push_attr(&mut caller_html, "id", call_id_str);
            push_attr(&mut caller_html, "data-usfm-id", &token_id_for_caller);
            push_attr(&mut caller_html, "data-usfm-note-kind", kind.as_str());
            push_attr(&mut caller_html, "data-usfm-caller", &label_for_caller);
            push_attr(
                &mut caller_html,
                "data-usfm-source-caller",
                &source_caller_for_caller,
            );
            caller_html.push('>');
            caller_html.push_str(&escape_html(&label_for_caller));
            caller_html.push_str("</a></sup>");
            if stack_len >= 2 {
                self.stack[stack_len - 2].buffer.push_str(&caller_html);
            } else {
                self.output.push_str(&caller_html);
            }
        } else {
            // Inline mode: build attrs on the note frame now that we
            // know the source caller. The wrapping `<span>` and
            // `<sup>label</sup>` prefix are emitted at close time.
            let mut attrs = common_marker_attrs("note", marker_for_inline);
            attrs.push(("data-usfm-id".to_string(), token_id_for_caller.clone()));
            attrs.push(("data-usfm-caller".to_string(), label_for_caller.clone()));
            attrs.push((
                "data-usfm-source-caller".to_string(),
                source_caller_for_caller.clone(),
            ));
            attrs.push(("data-usfm-note-kind".to_string(), kind.as_str().to_string()));
            self.stack.last_mut().expect("note frame present").attrs = attrs;
        }
    }
}

// =============================================================================
// Close emission
// =============================================================================

/// Emit the closing HTML for `item`, taking into account its
/// `Emission` mode. Mirrors the previous `append_closed_element` for
/// `Element`, handles phantoms as no-ops, and reroutes note bodies
/// (inline span / extracted aside).
fn emit_close(output: &mut String, stack: &mut Vec<OpenElement<'_>>, item: OpenElement<'_>) {
    match item.emission {
        Emission::Phantom => {
            // Nothing to emit. Buffer should be empty (no text events
            // should have targeted this frame, since it's an open
            // marker like `\c` / `\v` / `\esbe` where text and
            // children attach at the surrounding paragraph level).
            // Belt-and-suspenders: if a stray bit ended up here,
            // append it to the parent so it doesn't get lost.
            if !item.buffer.is_empty() {
                push_fragment(output, stack, &item.buffer);
            }
        }
        Emission::Element => {
            let mut html = String::new();
            html.push('<');
            html.push_str(item.tag);
            push_attrs(&mut html, &item.attrs);
            html.push('>');
            html.push_str(&item.buffer);
            html.push_str("</");
            html.push_str(item.tag);
            html.push('>');
            push_fragment(output, stack, &html);
        }
        Emission::InlineNote(state) => {
            // Inline note: `<span ...><sup>label</sup>BODY</span>`.
            let mut html = String::from("<span");
            push_attrs(&mut html, &item.attrs);
            html.push('>');
            html.push_str("<sup>");
            html.push_str(&escape_html(&state.label));
            html.push_str("</sup>");
            html.push_str(&item.buffer);
            html.push_str("</span>");
            push_fragment(output, stack, &html);
        }
        Emission::ExtractedNote(_) => {
            // Should not be reached: `HtmlVisitor::on_leave_scope`
            // intercepts ExtractedNote frames before calling
            // `emit_close`. The fallthrough exists only for
            // `finalize()`'s belt-and-suspenders drain, which is
            // never expected to encounter an unclosed note (the
            // walker emits EndOfInput closures for those — which
            // route through `on_leave_scope` first).
            let _ = (output, stack);
        }
    }
}

// =============================================================================
// Helpers preserved from the prior implementation
// =============================================================================

fn synthetic_table(prefer_native_elements: bool) -> OpenElement<'static> {
    let _ = prefer_native_elements;
    OpenElement {
        emission: Emission::Element,
        marker: None,
        tag: "table",
        attrs: vec![("data-usfm-type".to_string(), "table".to_string())],
        buffer: String::new(),
        scope_kind: StructuralScopeKind::Block,
        note_subkind: None,
        note_family: None,
        synthetic: true,
    }
}

fn synthetic_table_row() -> OpenElement<'static> {
    OpenElement {
        emission: Emission::Element,
        marker: None,
        tag: "tr",
        attrs: vec![("data-usfm-type".to_string(), "table:row".to_string())],
        buffer: String::new(),
        scope_kind: StructuralScopeKind::TableRow,
        note_subkind: None,
        note_family: None,
        synthetic: true,
    }
}

fn ensure_table_open(stack: &mut Vec<OpenElement<'_>>, prefer_native_elements: bool) {
    if !stack.iter().any(|item| {
        item.attrs
            .iter()
            .any(|(key, value)| key == "data-usfm-type" && value == "table")
    }) {
        stack.push(synthetic_table(prefer_native_elements));
    }
}

fn tag_and_type_for_marker(
    marker: &str,
    kind: Option<MarkerDefKind>,
    scope_kind: StructuralScopeKind,
    prefer_native_elements: bool,
) -> (&'static str, &'static str) {
    match kind {
        Some(MarkerDefKind::Figure) => {
            if prefer_native_elements {
                ("figure", "figure")
            } else {
                ("div", "figure")
            }
        }
        Some(MarkerDefKind::Periph) => ("div", "periph"),
        Some(MarkerDefKind::Sidebar) => ("div", "sidebar"),
        Some(MarkerDefKind::TableRow) => ("tr", "table:row"),
        Some(MarkerDefKind::TableCell) => ("td", "table:cell"),
        Some(MarkerDefKind::Character) if marker == "ref" => ("span", "ref"),
        Some(MarkerDefKind::Character) => ("span", "char"),
        Some(MarkerDefKind::Milestone) => ("span", "ms"),
        Some(MarkerDefKind::Header | MarkerDefKind::Paragraph | MarkerDefKind::Meta) => {
            if marker == "id" {
                if prefer_native_elements {
                    ("section", "book")
                } else {
                    ("div", "book")
                }
            } else {
                ("div", "para")
            }
        }
        _ => match scope_kind {
            StructuralScopeKind::TableRow => ("tr", "table:row"),
            StructuralScopeKind::TableCell => ("td", "table:cell"),
            StructuralScopeKind::Sidebar => ("div", "sidebar"),
            StructuralScopeKind::Periph => ("div", "periph"),
            StructuralScopeKind::Character | StructuralScopeKind::Milestone => ("span", "char"),
            _ => ("div", "unknown"),
        },
    }
}

fn close_for_new_block(output: &mut String, stack: &mut Vec<OpenElement<'_>>, keep_book: bool) {
    // Used only by `on_book_code` for the rare BookCode-without-`\id`
    // path. Closes inline/table/block scopes down to the book wrapper.
    while matches!(
        stack.last().map(|item| item.scope_kind),
        Some(StructuralScopeKind::Character | StructuralScopeKind::Milestone)
    ) {
        let item = stack.pop().expect("inline scope present");
        emit_close(output, stack, item);
    }
    while let Some(top) = stack.last() {
        let is_table_wrapper = top.synthetic
            && top
                .attrs
                .iter()
                .any(|(key, value)| key == "data-usfm-type" && value == "table");
        let should_pop = is_table_wrapper
            || matches!(
                top.scope_kind,
                StructuralScopeKind::TableCell
                    | StructuralScopeKind::TableRow
                    | StructuralScopeKind::Block
                    | StructuralScopeKind::Sidebar
                    | StructuralScopeKind::Periph
                    | StructuralScopeKind::Meta
            )
            || (matches!(top.scope_kind, StructuralScopeKind::Header) && top.marker != Some("id"));
        if !should_pop {
            break;
        }
        let item = stack.pop().expect("block scope present");
        emit_close(output, stack, item);
    }
    if !keep_book {
        while matches!(
            stack.last().map(|item| item.scope_kind),
            Some(StructuralScopeKind::Header)
        ) && stack
            .last()
            .is_some_and(|item| !item.synthetic && item.marker == Some("id"))
        {
            let item = stack.pop().expect("book wrapper present");
            emit_close(output, stack, item);
        }
    }
}

fn open_book_element<'a>(
    marker: &'a str,
    code: &'a str,
    prefer_native_elements: bool,
    token_id: String,
) -> OpenElement<'a> {
    let tag = if prefer_native_elements {
        "section"
    } else {
        "div"
    };
    let mut attrs = common_marker_attrs("book", marker);
    attrs.push(("data-usfm-id".to_string(), token_id));
    attrs.push(("data-usfm-code".to_string(), code.to_string()));
    OpenElement {
        emission: Emission::Element,
        marker: Some(marker),
        tag,
        attrs,
        buffer: String::new(),
        scope_kind: StructuralScopeKind::Header,
        note_subkind: None,
        note_family: None,
        synthetic: false,
    }
}

fn push_fragment(output: &mut String, stack: &mut [OpenElement<'_>], html: &str) {
    if let Some(parent) = stack.last_mut() {
        parent.buffer.push_str(html);
    } else {
        output.push_str(html);
    }
}

fn token_id_str(id: &TokenId<'_>) -> String {
    format!("{}-{}", id.book_code, id.index)
}

fn common_marker_attrs(data_type: &str, marker: &str) -> Vec<(String, String)> {
    vec![
        ("data-usfm-type".to_string(), data_type.to_string()),
        ("data-usfm-marker".to_string(), marker.to_string()),
    ]
}

fn empty_marker_span(
    data_type: &str,
    marker: &str,
    number: &str,
    sid: Option<&str>,
    token_id: &str,
) -> String {
    let mut out = String::from("<span");
    push_attr(&mut out, "data-usfm-type", data_type);
    push_attr(&mut out, "data-usfm-marker", marker);
    push_attr(&mut out, "data-usfm-id", token_id);
    if !number.is_empty() {
        push_attr(&mut out, "data-usfm-number", number);
    }
    if let Some(s) = sid {
        push_attr(&mut out, "data-usfm-sid", s);
    }
    out.push_str("></span>");
    out
}

fn push_attribute_entries(attrs: &mut Vec<(String, String)>, entries: &[AttributeItem<'_>]) {
    for entry in entries {
        attrs.push((
            format!("data-usfm-{}", kebab_case(entry.key)),
            entry.value.to_string(),
        ));
    }
}

fn render_extracted_note(
    marker: &str,
    note_kind: NoteKind,
    source_caller: &str,
    label: &str,
    call_id: &str,
    note_id: &str,
    token_id: &str,
    body: &str,
) -> String {
    let mut out = String::from("<aside");
    push_attr(&mut out, "id", note_id);
    push_attr(&mut out, "data-usfm-type", "note");
    push_attr(&mut out, "data-usfm-marker", marker);
    push_attr(&mut out, "data-usfm-id", token_id);
    push_attr(&mut out, "data-usfm-caller", label);
    push_attr(&mut out, "data-usfm-source-caller", source_caller);
    push_attr(&mut out, "data-usfm-note-kind", note_kind.as_str());
    out.push('>');
    out.push_str("<a");
    push_attr(&mut out, "href", &format!("#{call_id}"));
    out.push('>');
    out.push_str(&escape_html(label));
    out.push_str("</a>");
    out.push_str(body);
    out.push_str("</aside>");
    out
}

#[derive(Clone, Copy)]
enum NoteKind {
    Footnote,
    Crossref,
}

impl NoteKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Footnote => "footnote",
            Self::Crossref => "crossref",
        }
    }
}

fn note_kind(marker: &str) -> NoteKind {
    match marker_note_family(marker) {
        Some(NoteFamily::CrossReference) => NoteKind::Crossref,
        _ => NoteKind::Footnote,
    }
}

fn marker_is_sidebar_end(marker: &str, kind: Option<MarkerDefKind>) -> bool {
    let _ = kind;
    matches!(marker_block_behavior(marker), BlockBehavior::SidebarEnd)
}

fn table_cell_align(marker: &str) -> &'static str {
    if marker.starts_with("thr") || marker.starts_with("tcr") || marker.ends_with('r') {
        "end"
    } else if marker.starts_with("thc") || marker.starts_with("tcc") || marker.ends_with('c') {
        "center"
    } else {
        "start"
    }
}

fn push_attrs(out: &mut String, attrs: &[(String, String)]) {
    for (key, value) in attrs {
        push_attr(out, key, value);
    }
}

fn push_attr(out: &mut String, key: &str, value: &str) {
    out.push(' ');
    out.push_str(key);
    out.push_str("=\"");
    out.push_str(&escape_html(value));
    out.push('"');
}

fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn kebab_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == ' ' {
            out.push('-');
        } else {
            out.push(ch);
        }
    }
    out
}

fn format_ordinal(index: usize, style: HtmlCallerStyle) -> String {
    match style {
        HtmlCallerStyle::Numeric => index.to_string(),
        HtmlCallerStyle::AlphaLower => alpha_label(index, false),
        HtmlCallerStyle::AlphaUpper => alpha_label(index, true),
        HtmlCallerStyle::RomanLower => roman_label(index, false),
        HtmlCallerStyle::RomanUpper => roman_label(index, true),
        HtmlCallerStyle::Source => index.to_string(),
    }
}

fn alpha_label(mut index: usize, uppercase: bool) -> String {
    let mut out = String::new();
    while index > 0 {
        let rem = (index - 1) % 26;
        let base = if uppercase { b'A' } else { b'a' };
        out.insert(0, (base + rem as u8) as char);
        index = (index - 1) / 26;
    }
    out
}

fn roman_label(mut index: usize, uppercase: bool) -> String {
    let numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (value, numeral) in numerals {
        while index >= value {
            out.push_str(numeral);
            index -= value;
        }
    }
    if uppercase {
        out
    } else {
        out.to_ascii_lowercase()
    }
}

fn number_text(token: &Token<'_>) -> String {
    match &token.data {
        TokenData::Number { start, end, .. } => match end {
            Some(end) => format!("{start}-{end}"),
            None => start.to_string(),
        },
        _ => String::new(),
    }
}

// =============================================================================
// Tests preserved from prior implementation
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_html_extracts_footnotes_with_verse_scoped_callers() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note one\\f* more\\f + \\ft note two\\f*\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"data-usfm-caller="1.1""#));
        assert!(html.contains(r#"data-usfm-caller="1.2""#));
        assert!(html.contains(r#"id="linkedFootnotes""#));
        assert!(html.contains(r#"data-usfm-source-caller="+""#));
    }

    #[test]
    fn crossrefs_are_extracted_into_separate_group() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\x - \\xo 1.1 \\xt cross ref\\x*\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"id="linkedCrossrefs""#));
        assert!(html.contains(r#"data-usfm-note-kind="crossref""#));
    }

    #[test]
    fn preverse_notes_fall_back_to_document_sequential_labels() {
        let html = usfm_to_html("\\s1 Heading\\f + \\ft note\\f*\n", HtmlOptions::default());
        assert!(html.contains(r#"data-usfm-caller="1""#));
    }

    #[test]
    fn inline_note_mode_renders_note_in_flow() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note\\f*\n",
            HtmlOptions {
                note_mode: HtmlNoteMode::Inline,
                ..HtmlOptions::default()
            },
        );

        assert!(html.contains(r#"data-usfm-type="note""#));
        assert!(!html.contains(r#"id="linkedFootnotes""#));
    }

    #[test]
    fn delayed_attribute_list_attaches_to_open_char() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\" strong=\"H1\"\\w*.\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"data-usfm-lemma="grace""#));
        assert!(html.contains(r#"data-usfm-strong="H1""#));
    }

    #[test]
    fn sidebar_open_and_close_produces_single_sidebar_element() {
        // `\esbe` opens a Sidebar scope in the walker but must NOT emit
        // its own HTML element — its only job is to close `\esb`.
        // Regression guard against the walker pushing `\esbe` as a
        // real container.
        let html = usfm_to_html(
            "\\c 1\n\\esb\n\\p Sidebar body\n\\esbe\n\\p After\n",
            HtmlOptions::default(),
        );
        let sidebar_count = html.matches(r#"data-usfm-type="sidebar""#).count();
        assert_eq!(
            sidebar_count, 1,
            "expected exactly one sidebar element, got {sidebar_count}: {html}"
        );
        // No element should carry data-usfm-marker="esbe" — that
        // would mean the phantom frame leaked into output.
        assert!(
            !html.contains(r#"data-usfm-marker="esbe""#),
            "esbe phantom leaked into html: {html}"
        );
    }

    #[test]
    fn nested_character_markers_inside_note_render_as_nested_spans() {
        // \nd Lord\nd* inside a footnote body must produce a nested
        // `<span data-usfm-marker="nd">` inside the `<aside>` body.
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note about \\nd Lord\\nd* here\\f*\n",
            HtmlOptions::default(),
        );
        let aside_start = html.find("<aside").expect("aside present");
        let aside_end = html[aside_start..]
            .find("</aside>")
            .map(|rel| aside_start + rel)
            .expect("aside closes");
        let aside = &html[aside_start..aside_end];
        assert!(
            aside.contains(r#"data-usfm-marker="nd""#),
            "nd character not nested in aside: {aside}"
        );
        assert!(
            aside.contains("Lord"),
            "nd content missing from aside: {aside}"
        );
    }

    #[test]
    fn unclosed_footnote_does_not_swallow_subsequent_verses_in_html() {
        let src = "\\id GEN Sample\n\
                   \\c 1\n\\p\n\
                   \\v 1 First verse with an unclosed footnote.\\f + \\ft Note text never terminated.\n\
                   \\v 2 Second verse — should still appear.\n\
                   \\c 2\n\\p\n\
                   \\v 1 Chapter 2 should also still appear.\n";
        let html = usfm_to_html(src, HtmlOptions::default());
        assert!(
            html.contains(r#"data-usfm-sid="GEN 1:2""#),
            "v2 of ch1 missing from html: {html}"
        );
        assert!(
            html.contains(r#"data-usfm-sid="GEN 2""#),
            "ch2 chapter marker missing from html: {html}"
        );
        assert!(
            html.contains(r#"data-usfm-sid="GEN 2:1""#),
            "v1 of ch2 missing from html: {html}"
        );
        let aside_start = html
            .find("<aside")
            .expect("expected an aside element for the extracted note");
        let aside_end = html[aside_start..]
            .find("</aside>")
            .map(|relative| aside_start + relative)
            .expect("expected the aside to close");
        let aside = &html[aside_start..aside_end];
        assert!(
            !aside.contains(r#"data-usfm-sid="GEN 1:2""#),
            "v2 marker leaked into note body: {aside}"
        );
        assert!(
            !aside.contains(r#"data-usfm-sid="GEN 2""#),
            "chapter-2 marker leaked into note body: {aside}"
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod partition_tests {
    use super::{
        HtmlCallerScope, HtmlCallerStyle, HtmlNoteMode, HtmlOptions, HtmlVisitor, html_partitioned,
        wrap_document,
    };
    use crate::parse::parse;
    use crate::token::Token;
    use crate::walker::walk_tokens;
    use std::path::{Path, PathBuf};

    /// Guaranteed-serial render, independent of the token-count threshold, so it
    /// stays a fixed baseline even as the routing in `tokens_to_html` changes.
    fn serial(tokens: &[Token<'_>], options: HtmlOptions) -> String {
        let mut visitor = HtmlVisitor::new(options);
        walk_tokens(tokens, &mut visitor);
        wrap_document(visitor.finalize(), options)
    }

    /// The counter-sensitive option matrix the oracle can't reach (it pins html at
    /// defaults only). `caller_scope` and `caller_style` drive the document-sequential
    /// caller number and its skip; `note_mode` decides whether notes extract (touching
    /// the anchor-id counters, the cross-chapter state the default also has) or render
    /// inline; `wrap_root` exercises the single root wrap. `prefer_native_elements` is
    /// orthogonal to the merge, so it stays default.
    fn option_matrix() -> Vec<HtmlOptions> {
        let mut matrix = Vec::new();
        for caller_scope in [
            HtmlCallerScope::VerseSequential,
            HtmlCallerScope::DocumentSequential,
        ] {
            for note_mode in [HtmlNoteMode::Extracted, HtmlNoteMode::Inline] {
                for wrap_root in [false, true] {
                    for caller_style in [HtmlCallerStyle::Numeric, HtmlCallerStyle::Source] {
                        matrix.push(HtmlOptions {
                            wrap_root,
                            prefer_native_elements: true,
                            note_mode,
                            caller_style,
                            caller_scope,
                        });
                    }
                }
            }
        }
        matrix
    }

    #[test]
    fn partitioned_matches_serial_over_test_data() {
        assert_identical_over("testData");
    }

    #[test]
    fn partitioned_matches_serial_over_example_corpora() {
        assert_identical_over("example-corpora");
    }

    #[test]
    fn partitioned_matches_serial_on_boundary_shapes() {
        // Shapes the corpora may under-exercise, all counter-sensitive at a `\c`:
        let cases = [
            // no `\c` at all (single front segment);
            "\\id GEN\n\\p\n\\v 1 text\\f + \\ft a note\\f*\n",
            // a pre-verse note before the first chapter (document-sequential fallback
            // under VerseSequential) plus one inside a verse;
            "\\id GEN\n\\s1 Heading\\f + \\ft pre\\f*\n\\c 1\n\\p\n\\v 1 a\\f + \\ft b\\f*\n",
            // an unclosed note open before `\c` — the walker closes it at the boundary,
            // and the next chapter's callers must continue the anchor-id run;
            "\\id GEN\n\\c 1\n\\p\n\\v 1 t\\f + \\ft open\n\\c 2\n\\p\n\\v 1 u\\f + \\ft second\\f*\n",
            // footnotes and crossrefs interleaved across chapters (separate id runs);
            "\\id GEN\n\\c 1\n\\p\n\\v 1 t\\x - \\xt r\\x*\\f + \\ft fn\\f*\n\\c 2\n\\p\n\\v 1 u\\f + \\ft fn2\\f*\\x - \\xt r2\\x*\n",
            // a note outside any verse in a later chapter (pre-verse in ch2);
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a\n\\c 2\n\\p pre\\f + \\ft n\\f*\n\\v 1 b\n",
            // multiple notes per verse across chapters (verse-scoped numbering resets);
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a\\f + \\ft x\\f* b\\f + \\ft y\\f*\n\\c 2\n\\p\n\\v 1 c\\f + \\ft z\\f*\n",
            // a caller-less extracted note before a real one: the anchor-id counter
            // must advance only for the note whose caller is consumed;
            "\\id GEN\n\\c 1\n\\p\n\\v 1 a\\f\\f* b\\f + \\ft real\\f*\n\\c 2\n\\p\n\\v 1 c\\f + \\ft r2\\f*\n",
            // an empty stream.
            "",
        ];
        for (index, source) in cases.iter().enumerate() {
            assert_identical(source, &format!("boundary case #{index}"));
        }
    }

    fn assert_identical_over(dir: &str) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        let mut paths = Vec::new();
        collect_usfm(&root, &mut paths);
        paths.sort();
        assert!(!paths.is_empty(), "expected {dir}/**/*.usfm fixtures");
        for path in paths {
            let source = std::fs::read_to_string(&path).expect("read fixture");
            assert_identical(&source, &path.to_string_lossy());
        }
    }

    fn assert_identical(source: &str, label: &str) {
        let parsed = parse(source);
        for options in option_matrix() {
            let want = serial(&parsed.tokens, options);
            // Force the partitioned merge even on small fixtures below the threshold.
            let got = html_partitioned(&parsed.tokens, options);
            assert_eq!(
                got, want,
                "html differs for {label} (scope={:?}, note_mode={:?}, wrap_root={}, style={:?})",
                options.caller_scope, options.note_mode, options.wrap_root, options.caller_style
            );
        }
    }

    fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
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
}
