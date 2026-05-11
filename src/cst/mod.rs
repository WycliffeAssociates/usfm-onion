use serde::Serialize;

use crate::marker_defs::StructuralScopeKind;
use crate::parse::parse;
use crate::token::{Token, tokens_to_usfm};
use crate::walker::{LeaveReason, ScopeFrame, Visitor, WalkContext, walk_tokens};

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
struct NodeBuilder {
    token_index: usize,
    children: Vec<usize>,
}

pub fn build_cst<'a>(tokens: Vec<Token<'a>>) -> CstDocument<'a> {
    let roots = build_cst_roots(&tokens);
    CstDocument { tokens, roots }
}

/// CST visitor: builds an arena-backed tree as the walker emits events.
///
/// Tree-shape invariants matched against the prior `build_cst_roots`
/// implementation:
///
/// - Every token appears as exactly one node, in source order.
/// - `Block`, `Note`, `Character`, `Milestone`, `Sidebar`, `TableRow`,
///   `TableCell`, `Header`, `Meta`, `Periph` opens become parent nodes
///   for subsequent content until they close.
/// - `Chapter` and `Verse` opens are leaf nodes (not parents) — they
///   match the historical CST behaviour where their scope is tracked
///   semantically but children stay at the surrounding level.
/// - Explicit closes (`\f*`, `\nd*`, milestone-end) appear as sibling
///   leaves of their openers, not as children.
struct CstBuilder {
    arena: Vec<NodeBuilder>,
    root_indexes: Vec<usize>,
    /// Node indexes that are currently open as parents. Mutated only
    /// for scope kinds CST treats as containers (everything except
    /// Chapter and Verse).
    parent_stack: Vec<usize>,
}

impl CstBuilder {
    fn new(token_count: usize) -> Self {
        Self {
            arena: Vec::with_capacity(token_count),
            root_indexes: Vec::new(),
            parent_stack: Vec::new(),
        }
    }

    fn current_parent(&self) -> Option<usize> {
        self.parent_stack.last().copied()
    }

    fn append_leaf(&mut self, token_index: usize) -> usize {
        let parent = self.current_parent();
        append_node(&mut self.arena, &mut self.root_indexes, parent, token_index)
    }

    fn is_pushed_kind(kind: StructuralScopeKind) -> bool {
        // Chapter and Verse open scopes semantically but don't nest
        // children. Everything else does.
        !matches!(
            kind,
            StructuralScopeKind::Chapter | StructuralScopeKind::Verse
        )
    }
}

impl<'a> Visitor<'a, Token<'a>> for CstBuilder {
    fn on_enter_scope(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        frame: &ScopeFrame<'a>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        let node_index = self.append_leaf(token_index);
        if Self::is_pushed_kind(frame.scope_kind) {
            self.parent_stack.push(node_index);
        }
    }

    fn on_leave_scope(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        frame: &ScopeFrame<'a>,
        _reason: LeaveReason,
    ) {
        if Self::is_pushed_kind(frame.scope_kind) {
            self.parent_stack.pop();
        }
    }

    fn on_end_marker(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_milestone(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_milestone_end(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_text(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_chapter(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_verse(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_book_code(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_opt_break(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_newline(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }

    fn on_other(
        &mut self,
        _ctx: &WalkContext<'a, '_>,
        _token: &Token<'a>,
        token_index: usize,
    ) {
        self.append_leaf(token_index);
    }
}

pub fn build_cst_roots<'a>(tokens: &[Token<'a>]) -> Vec<CstNode> {
    let mut builder = CstBuilder::new(tokens.len());
    walk_tokens(tokens, &mut builder);
    finalize_roots(&builder.arena, &builder.root_indexes)
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
    // Iterative bottom-up finalize. Children always have higher arena
    // indexes than their parents (parents are appended before their
    // children), so processing the arena in reverse guarantees a
    // child's CstNode is ready when its parent looks it up.
    let mut finalized: Vec<Option<CstNode>> = (0..arena.len()).map(|_| None).collect();
    for i in (0..arena.len()).rev() {
        let node = &arena[i];
        let children = node
            .children
            .iter()
            .map(|&ci| finalized[ci].take().expect("child finalized before parent"))
            .collect();
        finalized[i] = Some(CstNode {
            token_index: node.token_index,
            children,
        });
    }
    root_indexes
        .iter()
        .map(|&i| finalized[i].take().expect("root present"))
        .collect()
}

fn flatten_nodes<'a>(nodes: &[CstNode], tokens: &[Token<'a>], output: &mut Vec<Token<'a>>) {
    // Iterative preorder traversal so depth doesn't blow the stack.
    let mut stack: Vec<std::slice::Iter<'_, CstNode>> = vec![nodes.iter()];
    while let Some(it) = stack.last_mut() {
        if let Some(node) = it.next() {
            output.push(tokens[node.token_index].clone());
            if !node.children.is_empty() {
                stack.push(node.children.iter());
            }
        } else {
            stack.pop();
        }
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

