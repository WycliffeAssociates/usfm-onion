use serde::Serialize;

use crate::marker_defs::StructuralScopeKind;
use crate::parse::parse;
use crate::structure::{
    ScopeSpec, StructuralToken, effective_context, is_inline_scope,
    marker_valid_in_current_context, structural_token,
};
use crate::token::{Token, tokens_to_usfm};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CstNode {
    pub token_index: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CstNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CstDocument<'a> {
    pub tokens: Vec<Token<'a>>,
    pub roots: Vec<CstNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkItem<'doc, 'tok> {
    pub node: &'doc CstNode,
    pub token: &'doc Token<'tok>,
    pub depth: usize,
    pub ancestor_token_indexes: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CstWalkIter<'doc, 'tok> {
    stack: Vec<WalkFrame<'doc>>,
    tokens: &'doc [Token<'tok>],
}

#[derive(Debug, Clone)]
struct WalkFrame<'doc> {
    nodes: &'doc [CstNode],
    next_index: usize,
    depth: usize,
    ancestor_token_indexes: Vec<usize>,
}

#[derive(Debug, Clone)]
struct OpenFrame<'a> {
    node_index: usize,
    scope: ScopeSpec<'a>,
}

#[derive(Debug, Clone)]
struct NodeBuilder {
    token_index: usize,
    children: Vec<usize>,
}

pub fn build_cst<'a>(tokens: Vec<Token<'a>>) -> CstDocument<'a> {
    let roots = build_cst_roots(&tokens);
    CstDocument { tokens, roots }
}

pub fn build_cst_roots<'a>(tokens: &[Token<'a>]) -> Vec<CstNode> {
    let mut arena = Vec::with_capacity(tokens.len());
    let mut root_indexes = Vec::new();
    let mut stack: Vec<OpenFrame<'a>> = Vec::new();

    for index in 0..tokens.len() {
        recover_stack(&tokens, index, &mut stack);

        let node_index = append_node(
            &mut arena,
            &mut root_indexes,
            current_parent_index(&stack),
            index,
        );

        if let StructuralToken::Open(scope) = structural_token(&tokens, index)
            && !matches!(
                scope.kind,
                StructuralScopeKind::Chapter | StructuralScopeKind::Verse
            )
        {
            stack.push(OpenFrame { node_index, scope });
        }
    }

    finalize_roots(&arena, &root_indexes)
}

pub fn parse_cst(source: &str) -> CstDocument<'_> {
    let parsed = parse(source);
    build_cst(parsed.tokens)
}

impl<'tok> CstDocument<'tok> {
    pub fn iter_walk(&self) -> CstWalkIter<'_, 'tok> {
        CstWalkIter {
            stack: vec![WalkFrame {
                nodes: &self.roots,
                next_index: 0,
                depth: 0,
                ancestor_token_indexes: Vec::new(),
            }],
            tokens: &self.tokens,
        }
    }
}

impl<'doc, 'tok> Iterator for CstWalkIter<'doc, 'tok> {
    type Item = WalkItem<'doc, 'tok>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, depth, ancestor_token_indexes) = {
                let frame = self.stack.last_mut()?;
                if frame.next_index >= frame.nodes.len() {
                    self.stack.pop();
                    continue;
                }

                let node = &frame.nodes[frame.next_index];
                frame.next_index += 1;
                (node, frame.depth, frame.ancestor_token_indexes.clone())
            };

            let item = WalkItem {
                node,
                token: &self.tokens[node.token_index],
                depth,
                ancestor_token_indexes,
            };

            if !node.children.is_empty() {
                let mut child_ancestors = item.ancestor_token_indexes.clone();
                child_ancestors.push(node.token_index);
                self.stack.push(WalkFrame {
                    nodes: &node.children,
                    next_index: 0,
                    depth: depth + 1,
                    ancestor_token_indexes: child_ancestors,
                });
            }

            return Some(item);
        }
    }
}

pub fn cst_to_tokens<'a>(document: &'a CstDocument<'a>) -> Vec<Token<'a>> {
    let mut ordered = Vec::with_capacity(document.tokens.len());
    flatten_nodes(&document.roots, &document.tokens, &mut ordered);
    ordered
}

pub fn cst_to_usfm(document: &CstDocument<'_>) -> String {
    tokens_to_usfm(&cst_to_tokens(document))
}

fn recover_stack<'a>(tokens: &[Token<'a>], index: usize, stack: &mut Vec<OpenFrame<'a>>) {
    match structural_token(tokens, index) {
        StructuralToken::CloseMarker(name) => {
            if let Some(match_pos) = stack.iter().rposition(|frame| {
                matches!(
                    frame.scope.kind,
                    StructuralScopeKind::Note | StructuralScopeKind::Character
                ) && frame.scope.marker == name
            }) {
                stack.truncate(match_pos);
            }
        }
        StructuralToken::MilestoneEnd => {
            if let Some(match_pos) = stack
                .iter()
                .rposition(|frame| frame.scope.kind == StructuralScopeKind::Milestone)
            {
                stack.truncate(match_pos);
            }
        }
        StructuralToken::UnknownMarker(_) => {
            pop_to_structural_parent(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::Chapter
                        | StructuralScopeKind::Periph
                        | StructuralScopeKind::Sidebar
                )
            });
        }
        StructuralToken::Open(scope) => {
            pop_for_open_scope(stack, scope);
        }
        StructuralToken::Leaf => {}
    }
}

fn pop_for_open_scope<'a>(stack: &mut Vec<OpenFrame<'a>>, incoming: ScopeSpec<'a>) {
    if matches!(
        incoming.kind,
        StructuralScopeKind::Note | StructuralScopeKind::Character | StructuralScopeKind::Milestone
    ) {
        while marker_needs_note_recovery(stack, incoming.marker) {
            stack.pop();
        }
        return;
    }

    match incoming.kind {
        StructuralScopeKind::Chapter => stack.clear(),
        StructuralScopeKind::Verse => {
            pop_while(stack, |kind| {
                is_inline_scope(kind) || kind == StructuralScopeKind::Verse
            });
            pop_while(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::Header | StructuralScopeKind::Meta
                )
            });
        }
        StructuralScopeKind::TableCell => {
            pop_while(stack, |kind| {
                is_inline_scope(kind) || kind == StructuralScopeKind::Verse
            });
            pop_while(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::TableCell | StructuralScopeKind::Block
                )
            });
            pop_to_structural_parent(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::TableRow
                        | StructuralScopeKind::Chapter
                        | StructuralScopeKind::Periph
                        | StructuralScopeKind::Sidebar
                )
            });
        }
        StructuralScopeKind::TableRow => {
            pop_while(stack, |kind| {
                is_inline_scope(kind) || kind == StructuralScopeKind::Verse
            });
            pop_while(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::TableCell
                        | StructuralScopeKind::TableRow
                        | StructuralScopeKind::Block
                )
            });
            pop_to_structural_parent(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::Chapter
                        | StructuralScopeKind::Periph
                        | StructuralScopeKind::Sidebar
                )
            });
        }
        StructuralScopeKind::Header | StructuralScopeKind::Meta | StructuralScopeKind::Periph => {
            stack.clear()
        }
        StructuralScopeKind::Sidebar => {
            pop_while(stack, |kind| kind != StructuralScopeKind::Chapter);
        }
        StructuralScopeKind::Block => {
            pop_while(stack, |kind| {
                is_inline_scope(kind) || kind == StructuralScopeKind::Verse
            });
            pop_while(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::Block
                        | StructuralScopeKind::TableCell
                        | StructuralScopeKind::TableRow
                        | StructuralScopeKind::Header
                        | StructuralScopeKind::Meta
                )
            });
            pop_to_structural_parent(stack, |kind| {
                matches!(
                    kind,
                    StructuralScopeKind::Chapter
                        | StructuralScopeKind::Periph
                        | StructuralScopeKind::Sidebar
                )
            });
        }
        StructuralScopeKind::Note
        | StructuralScopeKind::Character
        | StructuralScopeKind::Milestone
        | StructuralScopeKind::Unknown => {}
    }
}

fn marker_needs_note_recovery(stack: &[OpenFrame<'_>], incoming_marker: &str) -> bool {
    let active: Vec<ScopeSpec<'_>> = stack.iter().map(|frame| frame.scope).collect();
    let Some(context) = effective_context(&active) else {
        return false;
    };

    matches!(
        context,
        crate::marker_defs::SpecContext::Footnote | crate::marker_defs::SpecContext::CrossReference
    ) && !marker_valid_in_current_context(incoming_marker, &active)
}

fn pop_to_structural_parent<'a, F>(stack: &mut Vec<OpenFrame<'a>>, keep: F)
where
    F: Fn(StructuralScopeKind) -> bool,
{
    while stack.last().is_some_and(|frame| !keep(frame.scope.kind)) {
        stack.pop();
    }
}

fn pop_while<'a, F>(stack: &mut Vec<OpenFrame<'a>>, predicate: F)
where
    F: Fn(StructuralScopeKind) -> bool,
{
    while stack
        .last()
        .is_some_and(|frame| predicate(frame.scope.kind))
    {
        stack.pop();
    }
}

fn current_parent_index(stack: &[OpenFrame<'_>]) -> Option<usize> {
    stack.last().map(|frame| frame.node_index)
}

fn append_node(
    arena: &mut Vec<NodeBuilder>,
    root_indexes: &mut Vec<usize>,
    parent_index: Option<usize>,
    token_index: usize,
) -> usize {
    let node_index = arena.len();
    arena.push(NodeBuilder {
        token_index,
        children: Vec::new(),
    });

    match parent_index {
        Some(parent_index) => arena[parent_index].children.push(node_index),
        None => root_indexes.push(node_index),
    }

    node_index
}

fn finalize_roots(arena: &[NodeBuilder], root_indexes: &[usize]) -> Vec<CstNode> {
    root_indexes
        .iter()
        .map(|&index| finalize_node(arena, index))
        .collect()
}

fn finalize_node(arena: &[NodeBuilder], index: usize) -> CstNode {
    let node = &arena[index];
    CstNode {
        token_index: node.token_index,
        children: node
            .children
            .iter()
            .map(|&child| finalize_node(arena, child))
            .collect(),
    }
}

fn flatten_nodes<'a>(nodes: &[CstNode], tokens: &[Token<'a>], output: &mut Vec<Token<'a>>) {
    for node in nodes {
        output.push(tokens[node.token_index].clone());
        flatten_nodes(&node.children, tokens, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use crate::token::tokens_to_usfm;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn cst_roundtrips_all_usfm_sources() {
        for path in collect_usfm_paths(Path::new("testdata"))
            .into_iter()
            .chain(collect_usfm_paths(Path::new("example-corpora")))
        {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let document = parse_cst(&source);
            assert_eq!(
                cst_to_usfm(&document),
                source,
                "roundtrip failed for {}",
                path.display()
            );
            assert_eq!(
                tokens_to_usfm(&cst_to_tokens(&document)),
                source,
                "flatten failed for {}",
                path.display()
            );
        }
    }

    #[test]
    fn chapter_without_number_does_not_open_scope() {
        let document = parse_cst("\\c\n\\p text");
        let parsed = parse("\\c\n\\p text");

        let chapter_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "c", .. }
                )
            })
            .expect("chapter marker");
        let paragraph_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "p", .. }
                )
            })
            .expect("paragraph marker");

        let chapter_path = find_node_path(&document.roots, chapter_index).expect("chapter path");
        let paragraph_path =
            find_node_path(&document.roots, paragraph_index).expect("paragraph path");

        assert_eq!(chapter_path.len(), 1);
        assert_eq!(paragraph_path.len(), 1);
    }

    #[test]
    fn chapter_and_number_remain_adjacent_siblings() {
        let document = parse_cst("\\c 1\n\\p text");
        let parsed = parse("\\c 1\n\\p text");

        let chapter_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "c", .. }
                )
            })
            .expect("chapter marker");
        let number_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(token.data, crate::token::TokenData::Number { start: 1, .. })
            })
            .expect("chapter number");

        let chapter_path = find_node_path(&document.roots, chapter_index).expect("chapter path");
        let number_path = find_node_path(&document.roots, number_index).expect("number path");

        assert_eq!(chapter_path.len(), 1);
        assert_eq!(number_path.len(), 1);
    }

    #[test]
    fn unclosed_footnote_does_not_capture_unknown_marker_boundary() {
        let source = "\\v 28 \\f + \\ft note\n\\s5\n\\v 29 text";
        let document = parse_cst(source);
        let parsed = parse(source);

        let footnote_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "f", .. }
                )
            })
            .expect("footnote marker");
        let boundary_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "s5", .. }
                )
            })
            .expect("boundary marker");

        let footnote_path = find_node_path(&document.roots, footnote_index).expect("footnote path");
        let boundary_path = find_node_path(&document.roots, boundary_index).expect("boundary path");

        assert!(!path_is_ancestor(&footnote_path, &boundary_path));
    }

    #[test]
    fn unclosed_note_character_does_not_capture_following_paragraph() {
        let source = "\\q2 and mankind is not respected.\\f + \\ft text \\p thing";
        let document = parse_cst(source);
        let parsed = parse(source);

        let ft_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "ft", .. }
                )
            })
            .expect("ft marker");
        let paragraph_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "p", .. }
                )
            })
            .expect("paragraph marker");

        let ft_path = find_node_path(&document.roots, ft_index).expect("ft path");
        let paragraph_path =
            find_node_path(&document.roots, paragraph_index).expect("paragraph path");

        assert!(!path_is_ancestor(&ft_path, &paragraph_path));
    }

    #[test]
    fn paragraph_boundary_pops_previous_paragraph_scope() {
        let source = "\\m\n\\p thing";
        let document = parse_cst(source);
        let parsed = parse(source);

        let m_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "m", .. }
                )
            })
            .expect("m marker");
        let p_index = parsed
            .tokens
            .iter()
            .position(|token| {
                matches!(
                    token.data,
                    crate::token::TokenData::Marker { name: "p", .. }
                )
            })
            .expect("p marker");

        let m_path = find_node_path(&document.roots, m_index).expect("m path");
        let p_path = find_node_path(&document.roots, p_index).expect("p path");

        assert_eq!(m_path.len(), 1);
        assert_eq!(p_path.len(), 1);
    }

    #[test]
    fn iter_walk_reports_depth_and_ancestor_token_indexes() {
        let document = parse_cst("\\p text \\f + \\ft note\\f*");
        let walked: Vec<_> = document
            .iter_walk()
            .map(|item| {
                (
                    item.token.kind(),
                    item.token.source.to_string(),
                    item.depth,
                    item.ancestor_token_indexes,
                )
            })
            .collect();

        assert_eq!(walked[0].0, crate::token::TokenKind::Marker);
        assert_eq!(walked[0].2, 0);
        assert!(walked[0].3.is_empty());

        let footnote = walked
            .iter()
            .find(|(_, source, _, _)| source == "\\f")
            .expect("footnote marker");
        assert_eq!(footnote.2, 1);
        assert_eq!(footnote.3.len(), 1);

        let footnote_text = walked
            .iter()
            .find(|(_, source, _, _)| source == " + ")
            .expect("footnote text");
        assert_eq!(footnote_text.2, 2);
        assert_eq!(footnote_text.3.len(), 2);
    }

    fn collect_usfm_paths(root: &Path) -> Vec<PathBuf> {
        if !root.exists() {
            return Vec::new();
        }

        let mut paths = Vec::new();
        collect_into(root, &mut paths);
        paths.sort();
        paths
    }

    fn collect_into(root: &Path, paths: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(root)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                collect_into(&path, paths);
            } else if path.extension().is_some_and(|ext| ext == "usfm") {
                paths.push(path);
            }
        }
    }

    fn find_node_path(nodes: &[CstNode], token_index: usize) -> Option<Vec<usize>> {
        for (index, node) in nodes.iter().enumerate() {
            if node.token_index == token_index {
                return Some(vec![index]);
            }
            if let Some(mut child_path) = find_node_path(&node.children, token_index) {
                let mut path = vec![index];
                path.append(&mut child_path);
                return Some(path);
            }
        }
        None
    }

    fn path_is_ancestor(ancestor: &[usize], descendant: &[usize]) -> bool {
        ancestor.len() < descendant.len() && descendant.starts_with(ancestor)
    }
}
