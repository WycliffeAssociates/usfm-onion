use std::fmt::Write as _;

use crate::handle::{tokens, ParseHandle};
use crate::lint::{lint, LintOptions};
use crate::recovery::RecoveryPayload;
use crate::syntax::{Document, LeafKind, Node};
use crate::token::TokenViewOptions;

#[derive(Debug, Clone, Copy)]
pub struct DebugDumpOptions {
    pub include_raw: bool,
    pub include_projected: bool,
    pub include_recoveries: bool,
    pub include_lint: bool,
    pub include_document: bool,
    pub limit: usize,
}

impl Default for DebugDumpOptions {
    fn default() -> Self {
        Self {
            include_raw: true,
            include_projected: true,
            include_recoveries: true,
            include_lint: true,
            include_document: true,
            limit: 80,
        }
    }
}

pub fn debug_dump(handle: &ParseHandle, options: DebugDumpOptions) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "book_code: {:?}", handle.book_code());
    let _ = writeln!(out, "source_len: {}", handle.source().len());
    let _ = writeln!(out, "recoveries: {}", crate::recoveries(handle).len());

    if options.include_raw {
        let _ = writeln!(out, "\n[raw tokens]");
        for (index, token) in handle.raw_tokens().iter().take(options.limit).enumerate() {
            let _ = writeln!(
                out,
                "{index:04} {:?} {:?} {:?}",
                token.kind,
                token.span,
                summarize_text(&token.text)
            );
        }
        if handle.raw_tokens().len() > options.limit {
            let _ = writeln!(
                out,
                "... {} more raw tokens",
                handle.raw_tokens().len() - options.limit
            );
        }
    }

    if options.include_projected {
        let projected = tokens(handle, TokenViewOptions::default());
        let _ = writeln!(out, "\n[projected tokens]");
        for (index, token) in projected.iter().take(options.limit).enumerate() {
            let _ = writeln!(
                out,
                "{index:04} {:?} span={:?} sid={:?} marker={:?} text={:?}",
                token.kind,
                token.span,
                token.sid,
                token.marker,
                summarize_text(&token.text)
            );
        }
        if projected.len() > options.limit {
            let _ = writeln!(
                out,
                "... {} more projected tokens",
                projected.len() - options.limit
            );
        }
    }

    if options.include_recoveries {
        let _ = writeln!(out, "\n[recoveries]");
        for (index, recovery) in crate::recoveries(handle).iter().enumerate() {
            if index >= options.limit {
                let _ = writeln!(
                    out,
                    "... {} more recoveries",
                    crate::recoveries(handle).len() - options.limit
                );
                break;
            }
            let payload = match &recovery.payload {
                Some(RecoveryPayload::Marker { marker }) => format!("marker={marker}"),
                Some(RecoveryPayload::Close { open, close }) => {
                    format!("open={open:?} close={close:?}")
                }
                None => String::new(),
            };
            let _ = writeln!(
                out,
                "{index:04} {:?} span={:?} related={:?} {}",
                recovery.code, recovery.span, recovery.related_span, payload
            );
        }
    }

    if options.include_lint {
        let lint_issues = lint(handle, LintOptions::default());
        let _ = writeln!(out, "\n[lint]");
        for (index, issue) in lint_issues.iter().enumerate() {
            if index >= options.limit {
                let _ = writeln!(
                    out,
                    "... {} more lint issues",
                    lint_issues.len() - options.limit
                );
                break;
            }
            let _ = writeln!(
                out,
                "{index:04} {:?} fixable={} sid={:?} token={:?} related_token={:?} {}",
                issue.code,
                issue.fix.is_some(),
                issue.sid,
                issue.token_id,
                issue.related_token_id,
                issue.message
            );
        }
    }

    if options.include_document {
        let _ = writeln!(out, "\n[document]");
        dump_document(&mut out, handle.document(), 0, options.limit, &mut 0usize);
    }

    out
}

fn dump_document(
    out: &mut String,
    document: &Document,
    depth: usize,
    limit: usize,
    seen: &mut usize,
) {
    for node in &document.children {
        dump_node(out, node, depth, limit, seen);
        if *seen >= limit {
            break;
        }
    }
}

fn dump_node(out: &mut String, node: &Node, depth: usize, limit: usize, seen: &mut usize) {
    if *seen >= limit {
        return;
    }
    *seen += 1;
    let indent = "  ".repeat(depth);
    match node {
        Node::Container(container) => {
            let _ = writeln!(
                out,
                "{}{:?} marker={} span={:?} special={:?} attrs={}",
                indent,
                container.kind,
                container.marker,
                container.marker_span,
                container.special_span,
                container.attribute_spans.len()
            );
            for child in &container.children {
                dump_node(out, child, depth + 1, limit, seen);
                if *seen >= limit {
                    break;
                }
            }
        }
        Node::Chapter {
            marker_span,
            number_span,
        } => {
            let _ = writeln!(
                out,
                "{}Chapter marker={:?} number={:?}",
                indent, marker_span, number_span
            );
        }
        Node::Verse {
            marker_span,
            number_span,
        } => {
            let _ = writeln!(
                out,
                "{}Verse marker={:?} number={:?}",
                indent, marker_span, number_span
            );
        }
        Node::Milestone {
            marker,
            marker_span,
            attribute_spans,
            closed,
        } => {
            let _ = writeln!(
                out,
                "{}Milestone marker={} span={:?} attrs={} closed={}",
                indent,
                marker,
                marker_span,
                attribute_spans.len(),
                closed
            );
        }
        Node::Leaf { kind, span } => {
            let label = match kind {
                LeafKind::Text => "Text",
                LeafKind::Whitespace => "Whitespace",
                LeafKind::Newline => "Newline",
                LeafKind::Attributes => "Attributes",
            };
            let _ = writeln!(out, "{}{} span={:?}", indent, label, span);
        }
    }
}

fn summarize_text(text: &str) -> String {
    let escaped = text.escape_debug().to_string();
    if escaped.len() <= 80 {
        escaped
    } else {
        format!("{}...", &escaped[..80])
    }
}
