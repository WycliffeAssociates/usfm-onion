pub use crate::internal::api::{DocumentError, DocumentFormat, read_document};
pub use crate::internal::recovery::{ParseRecovery, RecoveryCode, RecoveryPayload};
pub use crate::model::cst::{
    CstChapter, CstContainer, CstContainerKind, CstDocument, CstLeaf, CstLeafKind, CstMilestone,
    CstNode, CstTokenRef, CstVerse,
};

use crate::internal::syntax::{ContainerKind, LeafKind, Node};
use crate::model::token::{Span, Token};
use crate::parse::handle::ParseHandle;
use std::path::Path;

pub fn parse_usfm(source: &str) -> CstDocument {
    let handle = crate::internal::parse::parse(source);
    handle_to_cst(&handle)
}

pub fn usj_to_cst(source: &str) -> Result<CstDocument, DocumentError> {
    let usfm = crate::convert::from_usj_str(source)?;
    Ok(parse_usfm(&usfm))
}

pub fn usx_to_cst(source: &str) -> Result<CstDocument, DocumentError> {
    let usfm = crate::convert::from_usx_str(source)?;
    Ok(parse_usfm(&usfm))
}

pub fn read_usfm_to_cst(path: impl AsRef<Path>) -> Result<CstDocument, DocumentError> {
    Ok(parse_usfm(&read_document(path, DocumentFormat::Usfm)?))
}

pub fn read_usj_to_cst(path: impl AsRef<Path>) -> Result<CstDocument, DocumentError> {
    Ok(usj_to_cst(&read_document(path, DocumentFormat::Usj)?)?)
}

pub fn read_usx_to_cst(path: impl AsRef<Path>) -> Result<CstDocument, DocumentError> {
    Ok(usx_to_cst(&read_document(path, DocumentFormat::Usx)?)?)
}

pub fn tokens_to_cst(tokens: &[Token]) -> CstDocument {
    parse_usfm(&crate::tokens::tokens_to_usfm(tokens))
}

pub fn cst_tokens(document: &CstDocument) -> &[Token] {
    document.tokens()
}

pub fn cst_to_tokens(document: &CstDocument) -> Vec<Token> {
    document.tokens.clone()
}

pub fn dfs_source_text(document: &CstDocument) -> String {
    crate::tokens::tokens_to_usfm(document.tokens())
}

fn handle_to_cst(handle: &ParseHandle) -> CstDocument {
    let tokens = crate::parse::handle::tokens(handle, crate::model::TokenViewOptions::default());
    let content = handle
        .document()
        .children
        .iter()
        .map(|node| syntax_node_to_cst(node, &tokens))
        .collect();

    CstDocument {
        doc_type: "CST".to_string(),
        book_code: handle.book_code().map(ToOwned::to_owned),
        recoveries: crate::parse::handle::recoveries(handle).to_vec(),
        tokens,
        content,
    }
}

fn syntax_node_to_cst(node: &Node, tokens: &[Token]) -> CstNode {
    match node {
        Node::Container(container) => CstNode::Container(CstContainer {
            kind: match container.kind {
                ContainerKind::Book => CstContainerKind::Book,
                ContainerKind::Paragraph => CstContainerKind::Paragraph,
                ContainerKind::Character => CstContainerKind::Character,
                ContainerKind::Note => CstContainerKind::Note,
                ContainerKind::Figure => CstContainerKind::Figure,
                ContainerKind::Sidebar => CstContainerKind::Sidebar,
                ContainerKind::Periph => CstContainerKind::Periph,
                ContainerKind::TableRow => CstContainerKind::TableRow,
                ContainerKind::TableCell => CstContainerKind::TableCell,
                ContainerKind::Header => CstContainerKind::Header,
                ContainerKind::Meta => CstContainerKind::Meta,
                ContainerKind::Unknown => CstContainerKind::Unknown,
            },
            marker: container.marker.clone(),
            marker_token: token_ref(container.marker_span.clone(), tokens),
            close_token: container
                .close_span
                .clone()
                .and_then(|span| token_ref(span, tokens)),
            special_token: container
                .special_span
                .clone()
                .and_then(|span| token_ref(span, tokens)),
            attribute_tokens: token_refs(container.attribute_spans.as_slice(), tokens),
            children: container
                .children
                .iter()
                .map(|child| syntax_node_to_cst(child, tokens))
                .collect(),
        }),
        Node::Chapter {
            marker_span,
            number_span,
        } => CstNode::Chapter(CstChapter {
            marker_token: token_ref_required(marker_span.clone(), tokens),
            number_token: number_span.clone().and_then(|span| token_ref(span, tokens)),
        }),
        Node::Verse {
            marker_span,
            number_span,
        } => CstNode::Verse(CstVerse {
            marker_token: token_ref_required(marker_span.clone(), tokens),
            number_token: number_span.clone().and_then(|span| token_ref(span, tokens)),
        }),
        Node::Milestone {
            marker,
            marker_span,
            attribute_spans,
            end_span,
            closed,
        } => CstNode::Milestone(CstMilestone {
            marker: marker.clone(),
            marker_token: token_ref_required(marker_span.clone(), tokens),
            attribute_tokens: token_refs(attribute_spans.as_slice(), tokens),
            end_token: end_span.clone().and_then(|span| token_ref(span, tokens)),
            closed: *closed,
        }),
        Node::Leaf { kind, span } => CstNode::Leaf(CstLeaf {
            kind: match kind {
                LeafKind::Text => CstLeafKind::Text,
                LeafKind::Whitespace => CstLeafKind::Whitespace,
                LeafKind::Newline => CstLeafKind::Newline,
                LeafKind::OptBreak => CstLeafKind::OptBreak,
                LeafKind::Attributes => CstLeafKind::Attributes,
            },
            token: token_ref_required(span.clone(), tokens),
        }),
    }
}

fn token_ref(span: Span, tokens: &[Token]) -> Option<CstTokenRef> {
    if span.start == span.end {
        return None;
    }

    // Tokens are ordered by source offset, so the containing token must be the
    // last token whose start offset is <= the target span start.
    let insertion_point = tokens.partition_point(|token| token.span.start <= span.start);
    let index = insertion_point.checked_sub(1)?;
    let token = tokens.get(index)?;

    (token.span.start <= span.start && token.span.end >= span.end)
        .then(|| CstTokenRef { token: index, span })
}

fn token_ref_required(span: Span, tokens: &[Token]) -> CstTokenRef {
    token_ref(span.clone(), tokens)
        .unwrap_or_else(|| panic!("missing token index for span {:?}", span))
}

fn token_refs(spans: &[Span], tokens: &[Token]) -> Vec<CstTokenRef> {
    spans
        .iter()
        .filter_map(|span| token_ref(span.clone(), tokens))
        .collect()
}
