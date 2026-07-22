use crate::marker_defs::{StructuralScopeKind, marker_note_subkind};
use crate::token::TokenKind;
use crate::walker::{LeaveReason, ScopeFrame, Visitor, WalkContext, walk};
#[cfg(not(target_arch = "wasm32"))]
use crate::walker::{WalkBoundary, walk_range};

#[cfg(not(target_arch = "wasm32"))]
use super::MARKER_BALANCE_CODES;
use super::{
    EnabledCodes, LintCode, LintIssue, LintableToken, is_note_close_marker, marker_params,
    message_params, render_template, simple_issue, simple_issue_with_marker,
};

/// Walker-driven marker-balance rules. Subscribes to the walker's
/// scope events so all the open/close tracking is delegated to the
/// unified state machine — no separate inline / structural stacks live
/// inside lint anymore.
///
/// Wires the following rules to walker events:
///
/// - `StrayCloseMarker` — falls out as `on_other(EndMarker)` (walker
///   couldn't pair the close to any open scope) and from
///   `on_milestone_end` when no Milestone was open.
/// - `MisnestedCloseMarker` + `ImplicitlyClosedMarker` — when the
///   walker pops scopes as `ImplicitByOpen` immediately before firing
///   `Explicit` for the matching scope, those popped frames are
///   misnest victims. Note submarkers (`\ft`, `\fqa`, …) that pop
///   silently during a `\f*` close are skipped, matching the prior
///   behaviour.
/// - `UnclosedMarker` — emitted for Note frames popped as
///   `RecoveryClosure` (block-boundary recovery), for Character
///   frames popped as `ImplicitByOpen` at a block boundary (drained
///   when the next non-leave event arrives), and for any Note/Character
///   still on the stack at EOF.
/// - `MissingMilestoneSelfClose` — fired immediately on
///   `on_enter_scope(Milestone)` when the *next* token isn't a
///   MilestoneEnd. (Pairwise check; not really walker-shaped, but
///   bundled here so the visitor owns the milestone logic too.)
pub(super) fn lint_marker_balance_rules<T: LintableToken>(
    tokens: &[T],
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    let mut visitor = MarkerBalanceVisitor::new(tokens, enabled);
    walk(tokens, &mut visitor);
    visitor.finish(issues);
}

/// Marker-balance rules over one chapter segment — `range` of the *full* `tokens`
/// slice, terminated at `boundary`. The visitor holds the full slice (so absolute
/// indices and the end-anchor resolve to the real tokens), and a `BeforeChapter`
/// segment flushes any implicitly-popped frames as boundary-`UnclosedMarker`s just
/// as an incoming `\c`'s `on_enter_scope` would in a whole-book walk. The final
/// (`EndOfInput`) segment leaves them undrained, matching the whole-book EOF.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn collect_marker_balance<T: LintableToken>(
    tokens: &[T],
    range: std::ops::Range<usize>,
    boundary: WalkBoundary,
    enabled: &EnabledCodes,
    issues: &mut Vec<LintIssue>,
) {
    if !enabled.has_any(MARKER_BALANCE_CODES) {
        return;
    }
    let mut visitor = MarkerBalanceVisitor::new(tokens, enabled);
    walk_range(tokens, range, boundary, &mut visitor);
    if boundary == WalkBoundary::BeforeChapter {
        visitor.drain_pending_as_boundary_unclosed();
    }
    visitor.finish(issues);
}

/// Pending state during a close cycle. Each entry is a frame popped
/// as `ImplicitByOpen`; its fate (misnest victim vs boundary recovery)
/// is decided by the next event.
#[derive(Debug, Clone)]
struct PendingImplicitPop {
    marker: String,
    scope_kind: StructuralScopeKind,
    token_index: usize,
}

struct MarkerBalanceVisitor<'a, T: LintableToken> {
    tokens: &'a [T],
    enabled: &'a EnabledCodes,
    pending_implicits: Vec<PendingImplicitPop>,
    /// Buffered issues. Held internally so we can deduplicate or amend
    /// them within the visitor's lifetime; flushed in `finish`.
    issues: Vec<LintIssue>,
    /// True when the most recent on_leave_scope was Explicit. Used by
    /// on_milestone_end to detect "stray \*" (no preceding Explicit).
    just_closed_via_explicit: bool,
}

impl<'a, T: LintableToken> MarkerBalanceVisitor<'a, T> {
    fn new(tokens: &'a [T], enabled: &'a EnabledCodes) -> Self {
        Self {
            tokens,
            enabled,
            pending_implicits: Vec::new(),
            issues: Vec::new(),
            just_closed_via_explicit: false,
        }
    }

    fn finish(self, sink: &mut Vec<LintIssue>) {
        sink.extend(self.issues);
    }

    /// Drain any `pending_implicits` as boundary-recovery
    /// `UnclosedMarker` emissions. Called when the walker fires an
    /// event that's neither another leave nor an Explicit close —
    /// i.e. the precedence pops were caused by an unrelated boundary,
    /// not a closing marker.
    fn drain_pending_as_boundary_unclosed(&mut self) {
        if self.pending_implicits.is_empty() {
            return;
        }
        if !self.enabled.has(LintCode::UnclosedMarker) {
            self.pending_implicits.clear();
            return;
        }
        for pending in std::mem::take(&mut self.pending_implicits) {
            // Note submarkers (Character frames with `valid_in_note`)
            // never trigger UnclosedMarker — they pop silently as
            // siblings of the note close. We only see them here when
            // they were popped by a block boundary, not a note close.
            if pending.scope_kind == StructuralScopeKind::Character
                && marker_note_subkind(&pending.marker).is_some()
            {
                continue;
            }
            self.push_unclosed_marker(&pending, "at-boundary");
        }
    }

    fn push_unclosed_marker(&mut self, pending: &PendingImplicitPop, location: &str) {
        let kind = match pending.scope_kind {
            StructuralScopeKind::Note => "note",
            StructuralScopeKind::Character => "character",
            _ => "other",
        };
        let code = LintCode::UnclosedMarker;
        let template = code.template();
        let params = message_params([
            ("marker", pending.marker.clone()),
            ("kind", kind.to_string()),
            ("location", location.to_string()),
        ]);
        let token = &self.tokens[pending.token_index];
        let anchor = self.tokens.last();
        self.issues.push(LintIssue {
            code,
            category: code.category(),
            severity: code.severity(),
            issue_type: code.issue_type(),
            template,
            message: render_template(template, &params),
            message_params: params,
            span: token.span(),
            related_span: anchor.and_then(|a| a.span()),
            token_id: token.id(),
            related_token_id: anchor.and_then(|a| a.id()),
            sid: token.sid().or_else(|| anchor.and_then(|a| a.sid())),
            marker: Some(pending.marker.clone()),
            fix: None,
        });
    }
}

impl<'a, 'tokens, T: LintableToken> Visitor<'tokens, T> for MarkerBalanceVisitor<'a, T> {
    fn on_enter_scope(
        &mut self,
        ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        token: &'tokens T,
        token_index: usize,
    ) {
        // Any pending implicit pops at this point were boundary
        // recoveries, not misnest victims (the next event is an open,
        // not an Explicit close).
        self.drain_pending_as_boundary_unclosed();
        self.just_closed_via_explicit = false;

        if frame.scope_kind == StructuralScopeKind::Milestone
            && self.enabled.has(LintCode::MissingMilestoneSelfClose)
            && self
                .tokens
                .get(token_index + 1)
                .is_none_or(|next| next.kind() != TokenKind::MilestoneEnd)
        {
            self.issues.push(simple_issue(
                LintCode::MissingMilestoneSelfClose,
                marker_params(frame.marker),
                token,
            ));
        }

        // USFM 3.2: a verse opening inside a Section / Other
        // paragraph is not allowed. The walker's
        // `current_paragraph_category` is the canonical signal — fires
        // exactly when the enclosing paragraph has a forbidden
        // category. `current_paragraph_category` walks the scope stack
        // inside-out, so the value it returns *now* is the paragraph
        // that contains this incoming Verse.
        if frame.scope_kind == StructuralScopeKind::Verse
            && self.enabled.has(LintCode::VerseInSectionOrOtherParagraph)
        {
            use crate::marker_defs::ParagraphCategory;
            let category = ctx.current_paragraph_category();
            let category_key = match category {
                Some(ParagraphCategory::Section) => Some("section"),
                Some(ParagraphCategory::Other) => Some("other"),
                _ => None,
            };
            if let Some(category_key) = category_key {
                self.issues.push(simple_issue(
                    LintCode::VerseInSectionOrOtherParagraph,
                    message_params([("category", category_key.to_string())]),
                    token,
                ));
            }
        }
    }

    fn on_leave_scope(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        frame: &ScopeFrame<'tokens>,
        reason: LeaveReason,
    ) {
        match reason {
            LeaveReason::Explicit => {
                // The matching `\X*` close. If we accumulated pending
                // implicit pops since the last anchor event, they were
                // misnest victims (popped to reach this match).
                //
                // Special case: when `\f*` (or `\x*`, …) closes a note,
                // any note-submarker Character frames above it
                // (`\ft`, `\fqa`, …) pop silently — they are
                // submarkers of the closing note and don't represent
                // misnesting. Mirrors the original lint code's
                // `is_note_close_marker` special-case.
                let closer_is_note_close = is_note_close_marker(frame.marker);
                if !self.pending_implicits.is_empty() {
                    let raw_victims = std::mem::take(&mut self.pending_implicits);
                    let victims: Vec<PendingImplicitPop> = raw_victims
                        .into_iter()
                        .filter(|v| {
                            !(closer_is_note_close
                                && v.scope_kind == StructuralScopeKind::Character
                                && marker_note_subkind(&v.marker).is_some())
                        })
                        .collect();
                    if !victims.is_empty()
                        && self.enabled.has(LintCode::MisnestedCloseMarker)
                        && let Some(expected) = victims.last()
                    {
                        let close_token = &self.tokens[frame.source_token_index];
                        let template = LintCode::MisnestedCloseMarker.template();
                        let params = message_params([
                            ("marker", frame.marker.to_string()),
                            ("expected", expected.marker.clone()),
                            ("has_expected", "true".to_string()),
                        ]);
                        let code = LintCode::MisnestedCloseMarker;
                        self.issues.push(LintIssue {
                            code,
                            category: code.category(),
                            severity: code.severity(),
                            issue_type: code.issue_type(),
                            template,
                            message: render_template(template, &params),
                            message_params: params,
                            span: close_token.span(),
                            related_span: None,
                            token_id: close_token.id(),
                            related_token_id: None,
                            sid: close_token.sid(),
                            marker: Some(frame.marker.to_string()),
                            fix: None,
                        });
                    }
                    if !victims.is_empty() && self.enabled.has(LintCode::ImplicitlyClosedMarker) {
                        for victim in victims {
                            let open_token = &self.tokens[victim.token_index];
                            self.issues.push(simple_issue_with_marker(
                                LintCode::ImplicitlyClosedMarker,
                                message_params([
                                    ("marker", victim.marker.clone()),
                                    ("closer", frame.marker.to_string()),
                                ]),
                                &victim.marker,
                                open_token,
                            ));
                        }
                    }
                }
                self.just_closed_via_explicit = true;
            }
            LeaveReason::ImplicitByOpen => {
                if matches!(
                    frame.scope_kind,
                    StructuralScopeKind::Note | StructuralScopeKind::Character
                ) {
                    self.pending_implicits.push(PendingImplicitPop {
                        marker: frame.marker.to_string(),
                        scope_kind: frame.scope_kind,
                        token_index: frame.source_token_index,
                    });
                }
                self.just_closed_via_explicit = false;
            }
            LeaveReason::RecoveryClosure => {
                // Notes auto-closed by a block-scope marker arriving.
                // Always Note kind (RecoveryClosure only fires for note
                // recovery in the walker).
                let pending = PendingImplicitPop {
                    marker: frame.marker.to_string(),
                    scope_kind: frame.scope_kind,
                    token_index: frame.source_token_index,
                };
                if self.enabled.has(LintCode::UnclosedMarker) {
                    self.push_unclosed_marker(&pending, "at-boundary");
                }
                self.just_closed_via_explicit = false;
            }
            LeaveReason::EndOfInput => {
                if matches!(
                    frame.scope_kind,
                    StructuralScopeKind::Note | StructuralScopeKind::Character
                ) && self.enabled.has(LintCode::UnclosedMarker)
                {
                    let pending = PendingImplicitPop {
                        marker: frame.marker.to_string(),
                        scope_kind: frame.scope_kind,
                        token_index: frame.source_token_index,
                    };
                    self.push_unclosed_marker(&pending, "at-eof");
                }
                self.just_closed_via_explicit = false;
            }
        }
    }

    fn on_end_marker(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        // Always preceded by on_leave_scope(Explicit) when the close
        // matched a real scope; nothing more to do here.
    }

    fn on_milestone_end(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        _token_index: usize,
    ) {
        if !self.just_closed_via_explicit && self.enabled.has(LintCode::StrayCloseMarker) {
            self.issues.push(simple_issue(
                LintCode::StrayCloseMarker,
                message_params([("form", "milestone-end".to_string())]),
                token,
            ));
        }
        self.just_closed_via_explicit = false;
    }

    fn on_text(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        self.drain_pending_as_boundary_unclosed();
        self.just_closed_via_explicit = false;
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        self.drain_pending_as_boundary_unclosed();
        self.just_closed_via_explicit = false;
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        _token: &'tokens T,
        _token_index: usize,
    ) {
        self.drain_pending_as_boundary_unclosed();
        self.just_closed_via_explicit = false;
    }

    fn on_other(
        &mut self,
        _ctx: &WalkContext<'tokens, '_>,
        token: &'tokens T,
        _token_index: usize,
    ) {
        // Walker routed an EndMarker here when no scope matched —
        // that's a stray close.
        if token.kind() == TokenKind::EndMarker
            && self.enabled.has(LintCode::StrayCloseMarker)
            && let Some(marker) = token.marker()
        {
            self.issues.push(simple_issue(
                LintCode::StrayCloseMarker,
                message_params([
                    ("marker", marker.to_string()),
                    ("form", "named".to_string()),
                ]),
                token,
            ));
        }
        self.drain_pending_as_boundary_unclosed();
        self.just_closed_via_explicit = false;
    }
}
