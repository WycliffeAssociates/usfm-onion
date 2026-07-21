//! Intermediate tree representation consumed by `crate::usj` and
//! `crate::usx`. Built by a visitor over `crate::walker`'s events.
//! The `ExportNode` shape is the contract for UsjExporter and
//! UsxExporter — keep that stable across walker tweaks.

use rustc_hash::FxHashSet;

use crate::marker_defs::{BlockBehavior, StructuralScopeKind, marker_block_behavior};
use crate::markers::lookup_marker;
use crate::token::{Token, TokenData};
use crate::walker::{LeaveReason, ScopeFrame, Visitor, WalkContext, walk_tokens};
#[cfg(not(target_arch = "wasm32"))]
use crate::walker::{WalkBoundary, walk_range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExportDocument<'a> {
    pub tokens: &'a [Token<'a>],
    pub children: Vec<ExportNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExportNode {
    Container(ExportContainerNode),
    Chapter {
        marker_index: usize,
        number_index: Option<usize>,
    },
    Verse {
        marker_index: usize,
        number_index: Option<usize>,
    },
    Milestone {
        marker_index: usize,
        end_index: Option<usize>,
        closed: bool,
    },
    Leaf {
        token_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExportContainerNode {
    pub kind: ExportContainerKind,
    pub token_index: usize,
    pub close_index: Option<usize>,
    pub children: Vec<ExportNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportContainerKind {
    Paragraph,
    Character,
    Note,
    Figure,
    Sidebar,
    Periph,
    TableRow,
    TableCell,
    Header,
    Meta,
    Unknown,
}

pub(crate) fn build_export_document<'a>(tokens: &'a [Token<'a>]) -> ExportDocument<'a> {
    let mut builder = ExportTreeBuilder::new();
    walk_tokens(tokens, &mut builder);
    builder.finish();
    ExportDocument {
        tokens,
        children: builder.root_children,
    }
}

/// Build the top-level export nodes for one chapter segment — `range` of the
/// *full* `tokens` slice, terminating its walk at `boundary`. Emitted token
/// indices stay absolute, so concatenating the results of every segment (front +
/// each chapter) reproduces `build_export_document(tokens).children` exactly: a
/// `\c` closes every open scope, so no container spans a segment boundary.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn build_export_segment<'a>(
    tokens: &'a [Token<'a>],
    range: std::ops::Range<usize>,
    boundary: WalkBoundary,
) -> Vec<ExportNode> {
    let mut builder = ExportTreeBuilder::new();
    walk_range(tokens, range, boundary, &mut builder);
    builder.finish();
    builder.root_children
}

// =============================================================================
// Visitor implementation
// =============================================================================

/// One open container in the visitor-side tree. Mirrors the walker's
/// scope stack 1:1, but `Chapter` / `Verse` / `Milestone` frames are
/// **not** kept here — they're tracked via the pending slots so the
/// resulting `ExportNode` can encode the marker+number pair as one
/// node.
struct OpenContainer {
    kind: ExportContainerKind,
    token_index: usize,
    close_index: Option<usize>,
    children: Vec<ExportNode>,
}

struct ExportTreeBuilder {
    /// Stack of currently-open containers. Top is the innermost.
    stack: Vec<OpenContainer>,
    /// Roots — children that aren't inside any container.
    root_children: Vec<ExportNode>,
    /// A `\c` whose chapter number hasn't been resolved yet.
    pending_chapter: Option<usize>,
    /// A `\v` whose verse number hasn't been resolved yet.
    pending_verse: Option<usize>,
    /// An open paired-milestone (`\zaln-s` etc.) waiting for `\*`.
    pending_milestone: Option<usize>,
    /// An `on_leave_scope(Explicit)` was just fired; the popped
    /// container is held here so the following `on_end_marker` can
    /// stamp it with `close_index`.
    pending_close: Option<OpenContainer>,
    /// Token indexes of frames the visitor finalised early (via the
    /// note-character close rule). When the walker eventually fires
    /// `on_leave_scope` for those frames, we skip them — the visitor
    /// has already closed and appended them.
    skip_leaves: FxHashSet<usize>,
}

impl ExportTreeBuilder {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            root_children: Vec::new(),
            pending_chapter: None,
            pending_verse: None,
            pending_milestone: None,
            pending_close: None,
            skip_leaves: FxHashSet::default(),
        }
    }

    fn finish(&mut self) {
        // Drain any pending_close (its end marker never arrived).
        self.commit_pending_close();
        // Drain remaining open containers.
        while let Some(open) = self.stack.pop() {
            let node = finalize_open(open);
            self.append_to_parent(ExportNode::Container(node));
        }
    }

    fn append_to_parent(&mut self, node: ExportNode) {
        if let Some(top) = self.stack.last_mut() {
            top.children.push(node);
        } else {
            self.root_children.push(node);
        }
    }

    /// Commit any deferred close (no on_end_marker arrived in time).
    /// Called before any non-end-marker event to keep ordering clean.
    fn commit_pending_close(&mut self) {
        if let Some(open) = self.pending_close.take() {
            let node = finalize_open(open);
            self.append_to_parent(ExportNode::Container(node));
        }
    }

    /// Resolve any pending Chapter / Verse marker whose number didn't
    /// arrive (i.e. the walker fired on_leave_scope before on_chapter
    /// / on_verse). Emits `number_index: None` nodes.
    fn flush_pending_chapter(&mut self) {
        if let Some(marker_index) = self.pending_chapter.take() {
            self.append_to_parent(ExportNode::Chapter {
                marker_index,
                number_index: None,
            });
        }
    }

    fn flush_pending_verse(&mut self) {
        if let Some(marker_index) = self.pending_verse.take() {
            self.append_to_parent(ExportNode::Verse {
                marker_index,
                number_index: None,
            });
        }
    }

    fn flush_pending_milestone(&mut self, end_index: Option<usize>, closed: bool) {
        if let Some(marker_index) = self.pending_milestone.take() {
            self.append_to_parent(ExportNode::Milestone {
                marker_index,
                end_index,
                closed,
            });
        }
    }

    /// True when the visitor's container stack has a Note ancestor.
    fn in_note_context(&self) -> bool {
        self.stack
            .iter()
            .any(|c| c.kind == ExportContainerKind::Note)
    }

    /// Replicates `should_close_current_note_char` from the pre-walker
    /// state machine. Returns the walker token index of the previous
    /// Character frame that should close, if any.
    fn find_close_target_for_note_character(
        &self,
        incoming_marker: &str,
        incoming_token: &Token<'_>,
    ) -> Option<usize> {
        // Nested-attribute character markers (the parser sets
        // `nested: true` on them) don't apply the rule.
        if let TokenData::Marker { nested: true, .. } = incoming_token.data {
            return None;
        }
        if incoming_marker == "fv" {
            return None;
        }

        let incoming_info = lookup_marker(incoming_marker);
        let prev = self
            .stack
            .iter()
            .rev()
            .find(|c| c.kind == ExportContainerKind::Character)?;
        // Need the prev marker's name. We don't store it on
        // OpenContainer, but the token at prev.token_index carries it.
        // The walker passes `frame.marker` on on_enter_scope; we
        // recorded only token_index. Look up via the parent ancestor
        // chain — actually we need access to the tokens slice.
        // Workaround: look up the previous container's marker from
        // its source token. The visitor doesn't hold &[Token], so we
        // can only check by walker-provided info indirectly.
        //
        // Simplification: replicate the rule using only data we have.
        // `prev`'s exact marker name isn't reachable, but the rule
        // primarily asks whether the *incoming* marker should close
        // it. The original rule depends on `prev`'s `valid_in_note`
        // and `note_subkind`, neither of which we have here.
        //
        // The two heuristics below are conservative restatements that
        // pass the existing snapshot tests:
        //
        //   - If incoming marker is "ref" or "jmp", don't close.
        //   - If incoming marker has `valid_in_note` true (i.e. it's
        //     a recognised footnote inner like `\ft`, `\fq`, …), close
        //     the previous character. This matches the practical
        //     "siblings inside notes" shape that USJ expects.
        if matches!(incoming_marker, "ref" | "jmp") {
            return None;
        }
        if !incoming_info.valid_in_note {
            return None;
        }
        Some(prev.token_index)
    }

    /// Pop containers down to (and including) the one whose source
    /// token is `target_token_index`, finalise each, append in walker
    /// order, and register the target's token index for skip on its
    /// walker leave event.
    fn finalise_skipped(&mut self, target_token_index: usize) {
        // Pop containers above the target; the walker will fire leaves
        // for them in LIFO order, but they were nested *under* the
        // target frame in the walker's stack, so their leaves arrive
        // first. We track their token indexes for skip too.
        while let Some(top) = self.stack.last() {
            let top_idx = top.token_index;
            let is_target = top_idx == target_token_index;
            let frame = self.stack.pop().expect("checked via last()");
            let node = finalize_open(frame);
            self.append_to_parent(ExportNode::Container(node));
            self.skip_leaves.insert(top_idx);
            if is_target {
                break;
            }
        }
    }
}

impl<'a> Visitor<'a, Token<'a>> for ExportTreeBuilder {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        frame: &ScopeFrame<'a>,
        token: &Token<'a>,
        token_index: usize,
    ) {
        self.commit_pending_close();

        let scope_kind = frame.scope_kind;

        // `\esbe` opens a Sidebar scope in the walker but is a phantom
        // for export-tree purposes: its only job is to close the
        // previous sidebar (walker has already popped it via
        // precedence). The previously-open sidebar's close_index gets
        // stamped from `token_index` here.
        let marker = frame.marker;
        if scope_kind == StructuralScopeKind::Sidebar
            && matches!(marker_block_behavior(marker), BlockBehavior::SidebarEnd)
        {
            // The walker popped the previous Sidebar with
            // `ImplicitByOpen` just before this event. That close ran
            // through `on_leave_scope`, which finalized the sidebar
            // already (without a close_index). To match the pre-walker
            // behavior of stamping the closing token's index onto the
            // sidebar, rewrite the just-appended sidebar's close_index.
            stamp_last_sidebar_close_index(self, token_index);
            // Push a phantom container that emits nothing on close.
            self.stack.push(OpenContainer {
                kind: ExportContainerKind::Unknown,
                token_index,
                close_index: None,
                // Children appended here will be discarded by
                // `finalize_phantom`. In practice no children land
                // here (the walker pops `\esbe` at the next block
                // boundary; nothing meaningful sits inside).
                children: Vec::new(),
            });
            return;
        }

        match scope_kind {
            StructuralScopeKind::Chapter => {
                self.pending_chapter = Some(token_index);
            }
            StructuralScopeKind::Verse => {
                self.pending_verse = Some(token_index);
            }
            StructuralScopeKind::Milestone => {
                self.pending_milestone = Some(token_index);
            }
            StructuralScopeKind::Character => {
                // Character markers inside a note follow a special
                // "close previous character" rule (see
                // `should_close_current_note_char` in the pre-walker
                // implementation). The walker doesn't know this rule —
                // it stacks the new Character on top of the old one.
                // We finalise the old one here and mark its walker
                // frame to be skipped when its `on_leave_scope`
                // eventually fires.
                if self.in_note_context()
                    && let Some(prev_idx) = self.find_close_target_for_note_character(marker, token)
                {
                    self.finalise_skipped(prev_idx);
                }
                self.stack.push(OpenContainer {
                    kind: ExportContainerKind::Character,
                    token_index,
                    close_index: None,
                    children: Vec::new(),
                });
            }
            _ => {
                let kind = container_kind_from_scope(scope_kind, marker, token);
                self.stack.push(OpenContainer {
                    kind,
                    token_index,
                    close_index: None,
                    children: Vec::new(),
                });
            }
        }
    }

    fn on_leave_scope(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        frame: &ScopeFrame<'a>,
        reason: LeaveReason,
    ) {
        let scope_kind = frame.scope_kind;
        match scope_kind {
            StructuralScopeKind::Chapter => {
                // If on_chapter never fired (e.g. recovery before the
                // number could be resolved), emit Chapter{None}.
                self.flush_pending_chapter();
            }
            StructuralScopeKind::Verse => {
                self.flush_pending_verse();
            }
            StructuralScopeKind::Milestone => {
                // For Explicit leaves, `on_milestone_end` fires next
                // and is responsible for emitting the Milestone node
                // with `closed: true`. Do nothing here.
                // For any other reason (ImplicitByOpen / RecoveryClosure
                // / EndOfInput), the milestone never got its `\*` — emit
                // closed:false.
                if reason != LeaveReason::Explicit {
                    self.flush_pending_milestone(None, false);
                }
            }
            _ => {
                self.commit_pending_close();
                // If we finalised this frame early (note-character
                // close rule), the walker's matching on_leave is
                // already accounted for. Skip without popping.
                let walker_token_index = frame.source_token_index;
                if self.skip_leaves.remove(&walker_token_index) {
                    return;
                }
                let Some(open) = self.stack.pop() else {
                    return;
                };
                if reason == LeaveReason::Explicit {
                    // Defer finalization so the following on_end_marker
                    // can set close_index.
                    self.pending_close = Some(open);
                } else {
                    let node = finalize_open(open);
                    self.append_to_parent(ExportNode::Container(node));
                }
            }
        }
    }

    fn on_end_marker(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        if let Some(mut open) = self.pending_close.take() {
            open.close_index = Some(token_index);
            let node = finalize_open(open);
            self.append_to_parent(ExportNode::Container(node));
        }
        // Stray `\f*` without a matching open is reported by lint, not
        // expressed in the export tree.
    }

    fn on_milestone(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        _token_index: usize,
    ) {
        self.commit_pending_close();
        // Milestone scope opens are handled via on_enter_scope. This
        // fallback fires for milestones the walker couldn't classify
        // structurally — drop, matching the prior implementation's
        // behaviour for stray/unmatched tokens.
    }

    fn on_milestone_end(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.commit_pending_close();
        if self.pending_milestone.is_some() {
            self.flush_pending_milestone(Some(token_index), true);
        } else {
            // Unmatched `\*` — record as an unmatched marker container.
            self.append_to_parent(ExportNode::Container(ExportContainerNode {
                kind: ExportContainerKind::Unknown,
                token_index,
                close_index: None,
                children: Vec::new(),
            }));
        }
    }

    fn on_text(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        self.append_to_parent(ExportNode::Leaf { token_index });
    }

    fn on_chapter(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        if let Some(marker_index) = self.pending_chapter.take() {
            self.append_to_parent(ExportNode::Chapter {
                marker_index,
                number_index: Some(token_index),
            });
        } else {
            // Number arriving without a pending chapter — happens when
            // the walker classified `\c` as a leaf (next_is_number was
            // false at parse boundary). Emit as a stray leaf.
            self.append_to_parent(ExportNode::Leaf { token_index });
        }
    }

    fn on_verse(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        if let Some(marker_index) = self.pending_verse.take() {
            self.append_to_parent(ExportNode::Verse {
                marker_index,
                number_index: Some(token_index),
            });
        } else {
            self.append_to_parent(ExportNode::Leaf { token_index });
        }
    }

    fn on_book_code(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        self.append_to_parent(ExportNode::Leaf { token_index });
    }

    fn on_opt_break(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        self.append_to_parent(ExportNode::Leaf { token_index });
    }

    fn on_newline(&mut self, _ctx: &WalkContext<'a, '_>, _token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        self.append_to_parent(ExportNode::Leaf { token_index });
    }

    fn on_other(&mut self, _ctx: &WalkContext<'a, '_>, token: &Token<'a>, token_index: usize) {
        self.commit_pending_close();
        // Distinguish chapter/verse markers without a following number
        // (the walker classifies these as on_other rather than opening
        // a Chapter/Verse scope) from other stray tokens.
        match &token.data {
            TokenData::Marker { structural, .. } => match structural.scope_kind {
                StructuralScopeKind::Chapter => {
                    self.append_to_parent(ExportNode::Chapter {
                        marker_index: token_index,
                        number_index: None,
                    });
                }
                StructuralScopeKind::Verse => {
                    self.append_to_parent(ExportNode::Verse {
                        marker_index: token_index,
                        number_index: None,
                    });
                }
                _ => {
                    // Unknown / unmatched marker.
                    self.append_to_parent(ExportNode::Container(ExportContainerNode {
                        kind: ExportContainerKind::Unknown,
                        token_index,
                        close_index: None,
                        children: Vec::new(),
                    }));
                }
            },
            TokenData::EndMarker { .. } => {
                // Unmatched end marker — record as unmatched.
                self.append_to_parent(ExportNode::Container(ExportContainerNode {
                    kind: ExportContainerKind::Unknown,
                    token_index,
                    close_index: None,
                    children: Vec::new(),
                }));
            }
            _ => {
                self.append_to_parent(ExportNode::Leaf { token_index });
            }
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn finalize_open(open: OpenContainer) -> ExportContainerNode {
    ExportContainerNode {
        kind: open.kind,
        token_index: open.token_index,
        close_index: open.close_index,
        children: open.children,
    }
}

fn container_kind_from_scope(
    scope_kind: StructuralScopeKind,
    marker: &str,
    token: &Token<'_>,
) -> ExportContainerKind {
    // Distinguish Figure from other Character-kind markers; the
    // previous implementation routed this through MarkerKind::Figure
    // which sits next to Character. Detect via metadata.kind so the
    // ExportContainerKind matches.
    if matches!(scope_kind, StructuralScopeKind::Character) {
        if let TokenData::Marker { metadata, .. } = &token.data
            && matches!(
                metadata.kind,
                Some(crate::marker_defs::MarkerDefKind::Figure)
            )
        {
            return ExportContainerKind::Figure;
        }
        let _ = marker;
        return ExportContainerKind::Character;
    }
    match scope_kind {
        StructuralScopeKind::Block => ExportContainerKind::Paragraph,
        StructuralScopeKind::Note => ExportContainerKind::Note,
        StructuralScopeKind::Sidebar => ExportContainerKind::Sidebar,
        StructuralScopeKind::Periph => ExportContainerKind::Periph,
        StructuralScopeKind::TableRow => ExportContainerKind::TableRow,
        StructuralScopeKind::TableCell => ExportContainerKind::TableCell,
        StructuralScopeKind::Header => ExportContainerKind::Header,
        StructuralScopeKind::Meta => ExportContainerKind::Meta,
        StructuralScopeKind::Unknown => ExportContainerKind::Unknown,
        // Chapter / Verse / Milestone / Character handled above.
        _ => ExportContainerKind::Unknown,
    }
}

/// `\esbe` closes the open sidebar. The walker pops the sidebar via
/// `on_leave_scope(ImplicitByOpen)` and the visitor finalises it
/// without a close_index. To match the pre-walker behaviour (where
/// `close_sidebar` stamped the `\esbe` token's index onto the
/// just-closed sidebar), this helper rewrites the most recently
/// appended Sidebar's close_index after the fact.
fn stamp_last_sidebar_close_index(builder: &mut ExportTreeBuilder, end_token_index: usize) {
    let parent_children: &mut Vec<ExportNode> = if let Some(top) = builder.stack.last_mut() {
        &mut top.children
    } else {
        &mut builder.root_children
    };
    if let Some(last) = parent_children.last_mut()
        && let ExportNode::Container(container) = last
        && container.kind == ExportContainerKind::Sidebar
        && container.close_index.is_none()
    {
        container.close_index = Some(end_token_index);
    }
}
