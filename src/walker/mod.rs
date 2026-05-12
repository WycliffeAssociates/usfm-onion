//! Unified walker over the flat token stream.
//!
//! See `docs/plan-walker-architecture.md` for the design contract.
//!
//! ## Design notes (step 2a, deviates from plan)
//!
//! Three things shifted from the original plan during step-2 design:
//!
//! 1. **No `apply_recovery` flag.** The walker always applies precedence
//!    rules — opening a block-scope marker pops an unclosed note above
//!    it, opening a new `\p` pops the previous `\p`, etc. The plan's
//!    notion that lossless consumers (CST, lint) would see "source-as-
//!    written" with the flag off was incoherent: CST's tree shape today
//!    relies on those very precedence pops. Instead, `LeaveReason`
//!    annotates each implicit close so lint (step 6) can subscribe to
//!    `RecoveryClosure` events as its `UnclosedNote` signal.
//!
//! 2. **Single unified scope stack.** Character markers (`\nd`,
//!    `\bk`, …) and milestone-starts (`\zaln-s` …) live on the same
//!    stack as paragraphs, notes, chapters, etc. The original plan's
//!    "structural scopes only" subsection was wrong: current CST
//!    pushes character markers as parent frames in its tree, and any
//!    walker-based CST visitor needs to mirror that. Helpers like
//!    `current_paragraph_category()` already filter by frame kind, so
//!    consumers that only care about structural scopes are unaffected.
//!
//! 3. **`on_end_marker` and `on_milestone_end` events.** When an
//!    explicit close fires (`\f*`, `\zaln-e\*`), the walker emits
//!    `on_leave_scope(reason: Explicit)` *and* a follow-up event for
//!    the closing token itself. CST appends the closer as a sibling
//!    leaf of the opener; without the follow-up event, the close
//!    token would be silently consumed.

use crate::marker_defs::{
    InlineContext, ParagraphCategory, SpecContext, StructuralMarkerInfo, StructuralScopeKind,
    marker_allows_effective_context,
};
use crate::token::{Token, TokenData, TokenKind};

/// Internal trait the walker is generic over.
///
/// Methods use self-elided lifetimes (`&self -> &str`). When the walker
/// holds `&'tokens T`, those methods return `&'tokens str` references,
/// which is what `ScopeFrame<'tokens>` borrows from. This is what lets
/// the walker drive owned-string token types (`FormatToken`,
/// `EditorToken`) as well as source-borrowed `Token<'_>` — neither
/// needs a single shared "source lifetime."
///
/// `LintableToken` (public surface) is a supertrait of this one.
pub trait WalkableToken {
    fn kind(&self) -> TokenKind;
    fn marker(&self) -> Option<&str>;
    fn structural(&self) -> Option<StructuralMarkerInfo>;
    /// Returns the token's source slice. For `Text` tokens this is the
    /// content; for marker/milestone tokens it is the literal source
    /// span. Visitors typically only read `text()` inside `on_text`.
    fn text(&self) -> &str;
    /// True when the *next* token in the original stream is a
    /// `Number` token. Only consulted for `\c` / `\v` markers — they
    /// open a scope iff a number follows. Defaults to `false`; callers
    /// using `walk_tokens` with `&[Token<'_>]` get the correct answer
    /// from the slice-aware entry point. Custom impls (editor tokens
    /// without parse context) can leave the default and accept that
    /// chapter/verse opens fall back to leaf behaviour.
    fn next_is_number(&self) -> bool {
        false
    }
}

impl<'a> WalkableToken for Token<'a> {
    fn kind(&self) -> TokenKind {
        Token::kind(self)
    }

    fn marker(&self) -> Option<&str> {
        self.marker_name()
    }

    fn structural(&self) -> Option<StructuralMarkerInfo> {
        match self.data {
            TokenData::Marker { structural, .. }
            | TokenData::EndMarker { structural, .. }
            | TokenData::Milestone { structural, .. } => Some(structural),
            _ => None,
        }
    }

    fn text(&self) -> &str {
        self.source
    }

    // next_is_number stays at the default for `Token<'a>` — see
    // `walk_tokens` for the slice-aware entry point that resolves it.
}

/// One frame on the walker's scope stack.
///
/// Holds every open scope, including character markers and milestone-
/// starts. Visitors filter by `scope_kind` when they only care about a
/// subset (e.g. paragraph queries skip `Character` frames).
///
/// `'tokens` is the **slice** lifetime — the lifetime of the input
/// `&[T]` the walker is iterating. `frame.marker` borrows from a token
/// in that slice, not from any "source string" — that's what lets the
/// walker drive both source-borrowed and owned-string token types.
#[derive(Debug, Clone, Copy)]
pub struct ScopeFrame<'tokens> {
    pub scope_kind: StructuralScopeKind,
    pub marker: &'tokens str,
    /// Index into the input token slice of the token that opened this
    /// scope. Visitors that need attributes, span, or other token data
    /// look it up via this index.
    pub source_token_index: usize,
    /// `Some(category)` when this frame is a paragraph scope; `None`
    /// otherwise. Populated from `MarkerSpec.paragraph_category`.
    pub paragraph_category: Option<ParagraphCategory>,
    /// Inline-context metadata attached to the opening marker (drives
    /// `effective_context` resolution for paragraph scopes).
    pub inline_context: Option<InlineContext>,
    /// Note context metadata for note-scope frames (drives note-recovery
    /// decisions about which inner markers are valid inside the note).
    pub note_context: Option<SpecContext>,
}

/// Why a scope closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveReason {
    /// An explicit `\f*`, `\nd*`, `\esbe`, or milestone-end closed this
    /// frame directly.
    Explicit,
    /// The walker synthesised the close because an opening marker
    /// satisfying `StructuralScopeKind::closes_unclosed_note` arrived
    /// while a note was open. Lint subscribes to this to detect
    /// implicitly-recovered notes.
    RecoveryClosure,
    /// The token stream ended while the scope was still open.
    EndOfInput,
    /// Another scope opened that the precedence rules say must close
    /// this one (e.g. a new `\p` closes the previous `\p`; a new `\c`
    /// closes everything below it).
    ImplicitByOpen,
}

/// Read-only view of the walker's state, handed to every visitor
/// callback.
///
/// `'tokens` is the input-slice lifetime (borrows from tokens in the
/// slice). `'ctx` is the event-callback lifetime: any reference
/// obtained from a `WalkContext` is only valid for the duration of the
/// visitor method that received it. Visitors that need cross-event
/// state own that state themselves — the borrow checker enforces this
/// via `'ctx`.
#[derive(Debug, Clone, Copy)]
pub struct WalkContext<'tokens, 'ctx> {
    scope_stack: &'ctx [ScopeFrame<'tokens>],
}

impl<'tokens, 'ctx> WalkContext<'tokens, 'ctx> {
    pub fn scope_stack(&self) -> &'ctx [ScopeFrame<'tokens>] {
        self.scope_stack
    }

    /// Returns the depth of the innermost open note scope. Zero when
    /// no note is open.
    pub fn note_depth(&self) -> usize {
        self.scope_stack
            .iter()
            .filter(|f| f.scope_kind == StructuralScopeKind::Note)
            .count()
    }

    pub fn in_note(&self) -> bool {
        self.note_depth() > 0
    }

    /// Kind of the innermost open scope, if any.
    pub fn current_scope_kind(&self) -> Option<StructuralScopeKind> {
        self.scope_stack.last().map(|f| f.scope_kind)
    }

    /// Paragraph category of the innermost open paragraph scope, if
    /// any. Walks the stack inside-out and returns the first frame
    /// carrying a category. Returns `None` when no paragraph is
    /// currently open — lint cases like "verse outside any paragraph"
    /// key off exactly this.
    pub fn current_paragraph_category(&self) -> Option<ParagraphCategory> {
        self.scope_stack
            .iter()
            .rev()
            .find_map(|f| f.paragraph_category)
    }
}

/// Visitor trait. Every method has a no-op default; override only what
/// you care about.
///
/// Every callback receives `token_index: usize` — the position of the
/// triggering token in the input slice. Visitors that build
/// index-based structures (CST's arena, USJ's tree) use this directly;
/// visitors that don't care (lint, vref) ignore it.
#[allow(unused_variables)]
pub trait Visitor<'tokens, T: WalkableToken> {
    /// Fires when a scope opens. `frame.source_token_index ==
    /// token_index`.
    fn on_enter_scope(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    /// Fires when a scope closes for any reason. For `Explicit` closes,
    /// the closing token is delivered separately via `on_end_marker`
    /// (for `\f*`-style closers) or `on_milestone_end` (for `\*` after
    /// a milestone-end name). Synthesised closes (`RecoveryClosure`,
    /// `ImplicitByOpen`, `EndOfInput`) have no associated closing token.
    fn on_leave_scope(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        reason: LeaveReason,
    ) {
    }

    /// Fires immediately after `on_leave_scope(Explicit)` for the
    /// `EndMarker` token (`\f*`, `\nd*`, …). Visitors that want the
    /// closer to appear as a leaf in their output (CST, USJ) append
    /// it here.
    fn on_end_marker(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    /// Fires for self-closing milestone tokens and for milestone-end
    /// tokens (the `\*` after `\zaln-e`). Paired-milestone *opens*
    /// (`\zaln-s`) fire `on_enter_scope` instead.
    fn on_milestone(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    /// Fires for the bare `\*` token that closes a paired milestone.
    /// Currently distinct from `on_milestone` so visitors can treat
    /// the closer as a separate leaf if they want; most consumers
    /// will route both into the same handler.
    fn on_milestone_end(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_text(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_chapter(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_verse(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_book_code(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_opt_break(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    fn on_newline(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }

    /// Catch-all for any token the walker couldn't classify
    /// structurally (unknown markers, numbers outside chapter/verse,
    /// etc.). Visitors that want full token coverage append here.
    fn on_other(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        token_index: usize,
    ) {
    }
}

/// Convenience entry point for walking a slice of native `Token<'a>`s.
/// Resolves `next_is_number` against the slice so chapter/verse opens
/// are classified correctly without requiring custom impls of
/// `WalkableToken::next_is_number`.
pub fn walk_tokens<'tokens, 'src, V>(tokens: &'tokens [Token<'src>], visitor: &mut V)
where
    'src: 'tokens,
    V: Visitor<'tokens, Token<'src>>,
{
    let mut state = WalkerState::new();
    for index in 0..tokens.len() {
        let next_is_number = matches!(
            tokens.get(index + 1).map(|t| &t.data),
            Some(TokenData::Number { .. })
        );
        state.step(index, &tokens[index], next_is_number, visitor);
    }
    state.drain_to_eof(visitor);
}

/// Generic entry point. Resolves `next_is_number` against the slice
/// itself (via `WalkableToken::kind` on the following token) so chapter
/// and verse opens classify correctly without requiring custom impls.
/// Falls back to `WalkableToken::next_is_number` when the slice has no
/// next token.
pub fn walk<'tokens, T, V>(tokens: &'tokens [T], visitor: &mut V)
where
    T: WalkableToken,
    V: Visitor<'tokens, T>,
{
    let mut state = WalkerState::new();
    for (index, token) in tokens.iter().enumerate() {
        let next_is_number = match tokens.get(index + 1) {
            Some(next) => next.kind() == TokenKind::Number,
            None => token.next_is_number(),
        };
        state.step(index, token, next_is_number, visitor);
    }
    state.drain_to_eof(visitor);
}

struct WalkerState<'tokens> {
    stack: Vec<ScopeFrame<'tokens>>,
}

impl<'tokens> WalkerState<'tokens> {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn make_ctx<'ctx>(&'ctx self) -> WalkContext<'tokens, 'ctx> {
        WalkContext {
            scope_stack: &self.stack,
        }
    }

    fn step<T, V>(
        &mut self,
        index: usize,
        token: &'tokens T,
        next_is_number: bool,
        visitor: &mut V,
    ) where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        match token.kind() {
            TokenKind::Marker => self.handle_marker(index, token, next_is_number, visitor),
            TokenKind::EndMarker => self.handle_end_marker(index, token, visitor),
            TokenKind::Milestone => self.handle_milestone(index, token, visitor),
            TokenKind::MilestoneEnd => self.handle_milestone_end(index, token, visitor),
            TokenKind::BookCode => {
                let ctx = self.make_ctx();
                visitor.on_book_code(&ctx, token, index);
            }
            TokenKind::Number => {
                let ctx = self.make_ctx();
                match self.stack.last().map(|f| f.scope_kind) {
                    Some(StructuralScopeKind::Chapter) => visitor.on_chapter(&ctx, token, index),
                    Some(StructuralScopeKind::Verse) => visitor.on_verse(&ctx, token, index),
                    _ => visitor.on_other(&ctx, token, index),
                }
            }
            TokenKind::Text => {
                let ctx = self.make_ctx();
                visitor.on_text(&ctx, token, index);
            }
            TokenKind::Newline => {
                let ctx = self.make_ctx();
                visitor.on_newline(&ctx, token, index);
            }
            TokenKind::OptBreak => {
                let ctx = self.make_ctx();
                visitor.on_opt_break(&ctx, token, index);
            }
        }
    }

    fn handle_marker<T, V>(
        &mut self,
        index: usize,
        token: &'tokens T,
        next_is_number: bool,
        visitor: &mut V,
    ) where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        let Some(info) = token.structural() else {
            let ctx = self.make_ctx();
            visitor.on_other(&ctx, token, index);
            return;
        };
        let Some(marker) = token.marker() else {
            let ctx = self.make_ctx();
            visitor.on_other(&ctx, token, index);
            return;
        };

        // `\c` / `\v` without a following number are not scope openers
        // — they're leaf markers. Match `structure::open_token`'s
        // classification.
        if matches!(
            info.scope_kind,
            StructuralScopeKind::Chapter | StructuralScopeKind::Verse
        ) && !next_is_number
        {
            let ctx = self.make_ctx();
            visitor.on_other(&ctx, token, index);
            return;
        }

        if matches!(info.scope_kind, StructuralScopeKind::Unknown) {
            self.handle_unknown_marker(index, token, marker, visitor);
            return;
        }

        // Apply precedence rules: pop frames that the incoming scope
        // displaces. Each popped frame fires `on_leave_scope` with a
        // reason derived from the popped frame's kind.
        self.apply_open_precedence(info.scope_kind, marker, visitor);

        let paragraph_category = if matches!(info.scope_kind, StructuralScopeKind::Block) {
            crate::marker_defs::lookup_spec_marker(marker)
                .and_then(|spec| spec.paragraph_category)
        } else {
            None
        };

        let frame = ScopeFrame {
            scope_kind: info.scope_kind,
            marker,
            source_token_index: index,
            paragraph_category,
            inline_context: info.inline_context,
            note_context: info.note_context,
        };
        self.stack.push(frame);
        let ctx = self.make_ctx();
        visitor.on_enter_scope(&ctx, &frame, token, index);
    }

    fn handle_unknown_marker<T, V>(
        &mut self,
        index: usize,
        token: &'tokens T,
        marker: &str,
        visitor: &mut V,
    ) where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        // Mirror `cst::recover_stack`'s UnknownMarker arm: pop down to
        // a structural parent (Chapter, Periph, Sidebar) before
        // appending. We model the pops as `ImplicitByOpen` since the
        // unknown marker is functionally an opener — even though we
        // don't push a frame for it.
        self.pop_while(visitor, |kind| {
            !matches!(
                kind,
                StructuralScopeKind::Chapter
                    | StructuralScopeKind::Periph
                    | StructuralScopeKind::Sidebar
            )
        });
        let ctx = self.make_ctx();
        let _ = marker;
        visitor.on_other(&ctx, token, index);
    }

    fn handle_end_marker<T, V>(&mut self, index: usize, token: &'tokens T, visitor: &mut V)
    where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        let Some(info) = token.structural() else {
            let ctx = self.make_ctx();
            visitor.on_other(&ctx, token, index);
            return;
        };
        let Some(marker) = token.marker() else {
            let ctx = self.make_ctx();
            visitor.on_other(&ctx, token, index);
            return;
        };

        // Match against Note / Character frames by marker name.
        // Mirrors `cst::recover_stack`'s CloseMarker arm.
        if matches!(
            info.scope_kind,
            StructuralScopeKind::Note | StructuralScopeKind::Character
        ) {
            if let Some(match_pos) = self.stack.iter().rposition(|frame| {
                matches!(
                    frame.scope_kind,
                    StructuralScopeKind::Note | StructuralScopeKind::Character
                ) && frame.marker == marker
            }) {
                // Pop everything above the match as ImplicitByOpen,
                // then pop the matched frame as Explicit.
                while self.stack.len() > match_pos + 1 {
                    let frame = self.stack.pop().expect("len checked");
                    self.bookkeep_on_pop(&frame);
                    let ctx = self.make_ctx();
                    visitor.on_leave_scope(&ctx, &frame, LeaveReason::ImplicitByOpen);
                }
                let frame = self.stack.pop().expect("match_pos exists");
                self.bookkeep_on_pop(&frame);
                let ctx = self.make_ctx();
                visitor.on_leave_scope(&ctx, &frame, LeaveReason::Explicit);
                // Fire the EndMarker token event so visitors can
                // append it as a leaf.
                let ctx = self.make_ctx();
                visitor.on_end_marker(&ctx, token, index);
                return;
            }
        }
        // Unmatched end marker: fall through as on_other. Lint (step
        // 6) will surface these.
        let ctx = self.make_ctx();
        visitor.on_other(&ctx, token, index);
    }

    fn handle_milestone_end<T, V>(&mut self, index: usize, token: &'tokens T, visitor: &mut V)
    where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        // `\*` closes the topmost open Milestone. Anything popped above
        // it (rare; only matters if other scopes were pushed inside the
        // milestone without closing first) fires ImplicitByOpen; the
        // milestone itself fires Explicit, followed by on_milestone_end
        // for the closing token.
        if let Some(match_pos) = self
            .stack
            .iter()
            .rposition(|frame| frame.scope_kind == StructuralScopeKind::Milestone)
        {
            while self.stack.len() > match_pos + 1 {
                let frame = self.stack.pop().expect("len checked");
                self.bookkeep_on_pop(&frame);
                let ctx = self.make_ctx();
                visitor.on_leave_scope(&ctx, &frame, LeaveReason::ImplicitByOpen);
            }
            let frame = self.stack.pop().expect("match_pos exists");
            self.bookkeep_on_pop(&frame);
            let ctx = self.make_ctx();
            visitor.on_leave_scope(&ctx, &frame, LeaveReason::Explicit);
            let ctx = self.make_ctx();
            visitor.on_milestone_end(&ctx, token, index);
        } else {
            // Stray `\*` with no open milestone. Pass through as
            // on_milestone_end so visitors that want every token can
            // still observe it.
            let ctx = self.make_ctx();
            visitor.on_milestone_end(&ctx, token, index);
        }
    }

    fn handle_milestone<T, V>(&mut self, index: usize, token: &'tokens T, visitor: &mut V)
    where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        // Milestones in the token stream are always Open events from
        // a structural standpoint (paired or self-closing). The end
        // form ("\*") arrives as a TokenKind::MilestoneEnd token,
        // which we handle separately. Step-2a treats all milestones
        // as enter/leave-bracketed scopes: open via on_enter_scope,
        // close via MilestoneEnd handling firing on_leave_scope.
        let Some(info) = token.structural() else {
            let ctx = self.make_ctx();
            visitor.on_milestone(&ctx, token, index);
            return;
        };
        let Some(marker) = token.marker() else {
            let ctx = self.make_ctx();
            visitor.on_milestone(&ctx, token, index);
            return;
        };

        // A `TokenData::Milestone` is syntactically a milestone
        // regardless of whether the spec data has a row for the
        // marker. Unknown-but-milestone-shaped markers (custom `\z…`
        // milestones, alignment `\zaln-s` / `\zaln-e`) still need to
        // pair with their `\*` closer, which `handle_milestone_end`
        // locates by `scope_kind == Milestone`. Override the spec
        // kind here so the pairing works.
        let scope_kind = StructuralScopeKind::Milestone;

        // Apply precedence (e.g. an inline-scope close would pop
        // higher inline scopes). `pop_for_open_scope`'s Milestone
        // branch only handles note-recovery for invalid markers
        // inside notes — same as Note/Character. We mirror that.
        self.apply_open_precedence(scope_kind, marker, visitor);

        let frame = ScopeFrame {
            scope_kind,
            marker,
            source_token_index: index,
            paragraph_category: None,
            inline_context: info.inline_context,
            note_context: info.note_context,
        };
        self.stack.push(frame);
        let ctx = self.make_ctx();
        visitor.on_enter_scope(&ctx, &frame, token, index);
        // Note: a self-closing milestone (`\ts\*`) produces both a
        // Milestone open token AND a MilestoneEnd token in the stream
        // — the MilestoneEnd handler will fire the matching
        // on_leave_scope and on_milestone_end events. The step-2a
        // skeleton does not yet emit on_milestone for self-closing
        // milestones as a single "point" event; consumers that care
        // (USX) will revisit at their migration step.
    }

    /// Apply precedence rules when opening a scope of `incoming_kind`.
    /// Direct port of `cst::pop_for_open_scope`.
    fn apply_open_precedence<T, V>(
        &mut self,
        incoming_kind: StructuralScopeKind,
        incoming_marker: &str,
        visitor: &mut V,
    ) where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        // Note / Character / Milestone opens only trigger note-recovery
        // when the incoming marker is not valid in the current note
        // context.
        if matches!(
            incoming_kind,
            StructuralScopeKind::Note
                | StructuralScopeKind::Character
                | StructuralScopeKind::Milestone
        ) {
            while self.marker_needs_note_recovery(incoming_marker) {
                let frame = self.stack.pop().expect("checked via predicate");
                self.bookkeep_on_pop(&frame);
                let reason = if frame.scope_kind == StructuralScopeKind::Note {
                    LeaveReason::RecoveryClosure
                } else {
                    LeaveReason::ImplicitByOpen
                };
                let ctx = self.make_ctx();
                visitor.on_leave_scope(&ctx, &frame, reason);
            }
            return;
        }

        // The three-pass algorithm from `cst::pop_for_open_scope`.
        match incoming_kind {
            StructuralScopeKind::Chapter => {
                while let Some(frame) = self.stack.pop() {
                    self.bookkeep_on_pop(&frame);
                    let reason = if frame.scope_kind == StructuralScopeKind::Note {
                        LeaveReason::RecoveryClosure
                    } else {
                        LeaveReason::ImplicitByOpen
                    };
                    let ctx = self.make_ctx();
                    visitor.on_leave_scope(&ctx, &frame, reason);
                }
            }
            StructuralScopeKind::Verse => {
                self.pop_while(visitor, |kind| {
                    is_inline_scope(kind) || kind == StructuralScopeKind::Verse
                });
                self.pop_while(visitor, |kind| {
                    matches!(
                        kind,
                        StructuralScopeKind::Header | StructuralScopeKind::Meta
                    )
                });
            }
            StructuralScopeKind::TableCell => {
                self.pop_while(visitor, |kind| {
                    is_inline_scope(kind) || kind == StructuralScopeKind::Verse
                });
                self.pop_while(visitor, |kind| {
                    matches!(
                        kind,
                        StructuralScopeKind::TableCell | StructuralScopeKind::Block
                    )
                });
                self.pop_while(visitor, |kind| {
                    !matches!(
                        kind,
                        StructuralScopeKind::TableRow
                            | StructuralScopeKind::Chapter
                            | StructuralScopeKind::Periph
                            | StructuralScopeKind::Sidebar
                    )
                });
            }
            StructuralScopeKind::TableRow => {
                self.pop_while(visitor, |kind| {
                    is_inline_scope(kind) || kind == StructuralScopeKind::Verse
                });
                self.pop_while(visitor, |kind| {
                    matches!(
                        kind,
                        StructuralScopeKind::TableCell
                            | StructuralScopeKind::TableRow
                            | StructuralScopeKind::Block
                    )
                });
                self.pop_while(visitor, |kind| {
                    !matches!(
                        kind,
                        StructuralScopeKind::Chapter
                            | StructuralScopeKind::Periph
                            | StructuralScopeKind::Sidebar
                    )
                });
            }
            StructuralScopeKind::Header
            | StructuralScopeKind::Meta
            | StructuralScopeKind::Periph => {
                while let Some(frame) = self.stack.pop() {
                    self.bookkeep_on_pop(&frame);
                    let reason = if frame.scope_kind == StructuralScopeKind::Note {
                        LeaveReason::RecoveryClosure
                    } else {
                        LeaveReason::ImplicitByOpen
                    };
                    let ctx = self.make_ctx();
                    visitor.on_leave_scope(&ctx, &frame, reason);
                }
            }
            StructuralScopeKind::Sidebar => {
                self.pop_while(visitor, |kind| kind != StructuralScopeKind::Chapter);
            }
            StructuralScopeKind::Block => {
                self.pop_while(visitor, |kind| {
                    is_inline_scope(kind) || kind == StructuralScopeKind::Verse
                });
                self.pop_while(visitor, |kind| {
                    matches!(
                        kind,
                        StructuralScopeKind::Block
                            | StructuralScopeKind::TableCell
                            | StructuralScopeKind::TableRow
                            | StructuralScopeKind::Header
                            | StructuralScopeKind::Meta
                    )
                });
                self.pop_while(visitor, |kind| {
                    !matches!(
                        kind,
                        StructuralScopeKind::Chapter
                            | StructuralScopeKind::Periph
                            | StructuralScopeKind::Sidebar
                    )
                });
            }
            StructuralScopeKind::Unknown
            | StructuralScopeKind::Note
            | StructuralScopeKind::Character
            | StructuralScopeKind::Milestone => {}
        }
    }

    /// Pop frames as long as `predicate(top.scope_kind)` holds. Each
    /// popped frame fires `on_leave_scope`. Notes that pop here fire
    /// with `RecoveryClosure`; everything else with `ImplicitByOpen`.
    fn pop_while<T, V, F>(&mut self, visitor: &mut V, predicate: F)
    where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
        F: Fn(StructuralScopeKind) -> bool,
    {
        while self
            .stack
            .last()
            .is_some_and(|frame| predicate(frame.scope_kind))
        {
            let frame = self.stack.pop().expect("checked via last()");
            self.bookkeep_on_pop(&frame);
            let reason = if frame.scope_kind == StructuralScopeKind::Note {
                LeaveReason::RecoveryClosure
            } else {
                LeaveReason::ImplicitByOpen
            };
            let ctx = self.make_ctx();
            visitor.on_leave_scope(&ctx, &frame, reason);
        }
    }

    fn marker_needs_note_recovery(&self, incoming_marker: &str) -> bool {
        let Some(context) = self.effective_context() else {
            return false;
        };
        matches!(
            context,
            SpecContext::Footnote | SpecContext::CrossReference
        ) && !marker_allows_effective_context(incoming_marker, context)
    }

    /// Mirrors `structure::effective_context` over our scope stack.
    fn effective_context(&self) -> Option<SpecContext> {
        for scope in self.stack.iter().rev() {
            match scope.scope_kind {
                StructuralScopeKind::Note => return scope.note_context,
                StructuralScopeKind::TableRow | StructuralScopeKind::TableCell => {
                    return Some(SpecContext::Table);
                }
                StructuralScopeKind::Block => {
                    return match scope.inline_context {
                        Some(InlineContext::Para) => Some(SpecContext::Para),
                        Some(InlineContext::List) => Some(SpecContext::List),
                        Some(InlineContext::Section) => Some(SpecContext::Section),
                        Some(InlineContext::Table) => Some(SpecContext::Table),
                        None => Some(SpecContext::ChapterContent),
                    };
                }
                StructuralScopeKind::Chapter => return Some(SpecContext::ChapterContent),
                StructuralScopeKind::Verse => continue,
                StructuralScopeKind::Periph => return Some(SpecContext::PeripheralContent),
                StructuralScopeKind::Sidebar => return Some(SpecContext::Sidebar),
                StructuralScopeKind::Header | StructuralScopeKind::Meta => {
                    return Some(SpecContext::Scripture);
                }
                StructuralScopeKind::Character | StructuralScopeKind::Milestone => continue,
                StructuralScopeKind::Unknown => continue,
            }
        }
        None
    }

    fn bookkeep_on_pop(&mut self, _frame: &ScopeFrame<'tokens>) {
        // Reserved for future per-pop bookkeeping. Currently a no-op
        // since the walker no longer tracks chapter/verse sids — those
        // were unused by every visitor.
    }

    fn drain_to_eof<T, V>(&mut self, visitor: &mut V)
    where
        T: WalkableToken,
        V: Visitor<'tokens, T>,
    {
        while let Some(frame) = self.stack.pop() {
            self.bookkeep_on_pop(&frame);
            let ctx = self.make_ctx();
            visitor.on_leave_scope(&ctx, &frame, LeaveReason::EndOfInput);
        }
    }
}

fn is_inline_scope(kind: StructuralScopeKind) -> bool {
    matches!(
        kind,
        StructuralScopeKind::Note | StructuralScopeKind::Character | StructuralScopeKind::Milestone
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[derive(Debug, Default)]
    struct EventLog {
        events: Vec<String>,
    }

    impl<'tokens, T: WalkableToken> Visitor<'tokens, T> for EventLog {
        fn on_enter_scope(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            frame: &ScopeFrame<'tokens>,
            _token: &'tokens T,
            _token_index: usize,
        ) {
            self.events
                .push(format!("enter {:?}({})", frame.scope_kind, frame.marker));
        }

        fn on_leave_scope(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            frame: &ScopeFrame<'tokens>,
            reason: LeaveReason,
        ) {
            self.events.push(format!(
                "leave {:?}({}) [{:?}]",
                frame.scope_kind, frame.marker, reason
            ));
        }

        fn on_end_marker(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            token: &'tokens T,
            _token_index: usize,
        ) {
            self.events
                .push(format!("end-marker[{}]", token.marker().unwrap_or("?")));
        }

        fn on_chapter(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            token: &'tokens T,
            _token_index: usize,
        ) {
            self.events
                .push(format!("chapter[{}]", token.text().trim()));
        }

        fn on_verse(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            token: &'tokens T,
            _token_index: usize,
        ) {
            self.events
                .push(format!("verse[{}]", token.text().trim()));
        }

        fn on_text(
            &mut self,
            _ctx: &WalkContext<'tokens, '_>,
            token: &'tokens T,
            _token_index: usize,
        ) {
            let s = token.text();
            if !s.trim().is_empty() {
                self.events.push(format!("text[{:?}]", s));
            }
        }
    }

    fn walk_source(source: &str) -> Vec<String> {
        let parsed = parse(source);
        let mut log = EventLog::default();
        walk_tokens(&parsed.tokens, &mut log);
        log.events
    }

    #[test]
    fn chapter_paragraph_verse_emits_expected_events() {
        let events = walk_source("\\id GEN\n\\c 1\n\\p\n\\v 1 hello\n");

        let joined = events.join(" | ");
        assert!(
            joined.contains("enter Chapter"),
            "missing Chapter enter: {joined}"
        );
        assert!(
            joined.contains("enter Block(p)"),
            "missing paragraph enter: {joined}"
        );
        assert!(
            joined.contains("enter Verse"),
            "missing Verse enter: {joined}"
        );
        assert!(
            events.iter().any(|e| e.contains("hello")),
            "missing hello text: {joined}"
        );

        // \c is followed by \p — a Block. Block precedence pops
        // Verse and inline scopes (none here) and other Blocks (none
        // yet), then pops to-structural-parent keeping Chapter. So at
        // EOF the open frames are Chapter, Block(p), Verse(v).
        // Inside-out close order at EOF: Verse, Block, Chapter.
        let verse_leave = events
            .iter()
            .position(|e| e.starts_with("leave Verse"))
            .expect("verse leave");
        let block_leave = events
            .iter()
            .position(|e| e.starts_with("leave Block"))
            .expect("block leave");
        let chapter_leave = events
            .iter()
            .position(|e| e.starts_with("leave Chapter"))
            .expect("chapter leave");
        assert!(
            verse_leave < block_leave && block_leave < chapter_leave,
            "EOF closures out of order: {events:?}"
        );
        for e in &events[verse_leave..=chapter_leave] {
            if e.starts_with("leave ") {
                assert!(
                    e.contains("[EndOfInput]"),
                    "expected EndOfInput reason on {e}"
                );
            }
        }
    }

    #[test]
    fn unclosed_note_closes_with_recovery_reason_on_block_boundary() {
        let source = "\\id GEN\n\\c 1\n\\p before \\f + missing close\n\\p after\n";
        let events = walk_source(source);
        let joined = events.join(" | ");

        let note_enter = events
            .iter()
            .position(|e| e.starts_with("enter Note"))
            .unwrap_or_else(|| panic!("no Note enter: {joined}"));
        let note_leave = events
            .iter()
            .position(|e| e.starts_with("leave Note") && e.contains("[RecoveryClosure]"))
            .unwrap_or_else(|| panic!("no RecoveryClosure for Note: {joined}"));
        let second_p_enter = events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.starts_with("enter Block(p)"))
            .map(|(i, _)| i)
            .nth(1)
            .unwrap_or_else(|| panic!("no second Block(p) enter: {joined}"));

        assert!(
            note_enter < note_leave && note_leave < second_p_enter,
            "recovery did not fire before second paragraph: {events:?}"
        );
    }

    #[test]
    fn explicit_end_marker_fires_leave_then_end_marker_event() {
        let events = walk_source("\\id GEN\n\\c 1\n\\p \\nd Lord\\nd*  done\n");
        let joined = events.join(" | ");

        let nd_enter = events
            .iter()
            .position(|e| e == "enter Character(nd)")
            .unwrap_or_else(|| panic!("no nd enter: {joined}"));
        let nd_leave = events
            .iter()
            .position(|e| e.starts_with("leave Character(nd)") && e.contains("[Explicit]"))
            .unwrap_or_else(|| panic!("no nd explicit leave: {joined}"));
        let nd_end_marker = events
            .iter()
            .position(|e| e == "end-marker[nd]")
            .unwrap_or_else(|| panic!("no end-marker event for nd: {joined}"));

        assert!(
            nd_enter < nd_leave && nd_leave < nd_end_marker,
            "expected enter < leave < end-marker order: {events:?}"
        );
    }
}
