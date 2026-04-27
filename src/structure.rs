use crate::marker_defs::{
    InlineContext, SpecContext, StructuralMarkerInfo, StructuralScopeKind,
    marker_allows_effective_context,
};
use crate::token::{Token, TokenData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScopeSpec<'a> {
    pub kind: StructuralScopeKind,
    pub marker: &'a str,
    pub note_context: Option<SpecContext>,
    pub inline_context: Option<InlineContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructuralToken<'a> {
    Leaf,
    UnknownMarker(&'a str),
    CloseMarker(&'a str),
    MilestoneEnd,
    Open(ScopeSpec<'a>),
}

pub(crate) fn structural_token<'a>(tokens: &[Token<'a>], index: usize) -> StructuralToken<'a> {
    let token = &tokens[index];
    match &token.data {
        TokenData::Marker {
            name, structural, ..
        } => {
            if structural.scope_kind == StructuralScopeKind::Unknown {
                return StructuralToken::UnknownMarker(name);
            }

            open_token(tokens, index, name, *structural)
        }
        TokenData::Milestone {
            name, structural, ..
        } => open_token(tokens, index, name, *structural),
        TokenData::EndMarker { name, .. } => StructuralToken::CloseMarker(name),
        TokenData::MilestoneEnd => StructuralToken::MilestoneEnd,
        TokenData::Newline
        | TokenData::OptBreak
        | TokenData::BookCode { .. }
        | TokenData::Number { .. }
        | TokenData::Text
        | TokenData::AttributeList { .. } => StructuralToken::Leaf,
    }
}

fn open_token<'a>(
    tokens: &[Token<'a>],
    index: usize,
    name: &'a str,
    structural: StructuralMarkerInfo,
) -> StructuralToken<'a> {
    match structural.scope_kind {
        StructuralScopeKind::Unknown => StructuralToken::UnknownMarker(name),
        StructuralScopeKind::Chapter | StructuralScopeKind::Verse => {
            if matches!(
                tokens.get(index + 1).map(|token| &token.data),
                Some(TokenData::Number { .. })
            ) {
                StructuralToken::Open(ScopeSpec {
                    kind: structural.scope_kind,
                    marker: name,
                    note_context: structural.note_context,
                    inline_context: structural.inline_context,
                })
            } else {
                StructuralToken::Leaf
            }
        }
        _ => StructuralToken::Open(ScopeSpec {
            kind: structural.scope_kind,
            marker: name,
            note_context: structural.note_context,
            inline_context: structural.inline_context,
        }),
    }
}

pub(crate) fn effective_context(stack: &[ScopeSpec<'_>]) -> Option<SpecContext> {
    for scope in stack.iter().rev() {
        match scope.kind {
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

pub(crate) fn marker_valid_in_current_context(marker: &str, stack: &[ScopeSpec<'_>]) -> bool {
    let Some(context) = effective_context(stack) else {
        return true;
    };
    marker_allows_effective_context(marker, context)
}

pub(crate) fn is_inline_scope(kind: StructuralScopeKind) -> bool {
    matches!(
        kind,
        StructuralScopeKind::Note | StructuralScopeKind::Character | StructuralScopeKind::Milestone
    )
}
