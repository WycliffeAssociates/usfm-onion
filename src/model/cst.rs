use crate::internal::recovery::ParseRecovery;
use crate::model::token::{Span, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CstDocument {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub source_usfm: String,
    pub book_code: Option<String>,
    pub recoveries: Vec<ParseRecovery>,
    pub tokens: Vec<Token>,
    pub content: Vec<CstNode>,
}

impl CstDocument {
    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    pub fn token(&self, token_ref: &CstTokenRef) -> &Token {
        self.tokens
            .get(token_ref.token)
            .unwrap_or_else(|| panic!("missing CST token at index {}", token_ref.token))
    }

    pub fn token_text<'a>(&'a self, token_ref: &CstTokenRef) -> &'a str {
        let token = self.token(token_ref);
        let start_delta = token_ref
            .span
            .start
            .checked_sub(token.span.start)
            .unwrap_or_else(|| {
                panic!(
                    "CST token ref span {:?} starts before token span {:?}",
                    token_ref.span, token.span
                )
            });
        let end_delta = token_ref
            .span
            .end
            .checked_sub(token.span.start)
            .unwrap_or_else(|| {
                panic!(
                    "CST token ref span {:?} ends before token span {:?}",
                    token_ref.span, token.span
                )
            });
        let start_byte = char_offset_to_byte_index(token.text.as_str(), start_delta)
            .unwrap_or_else(|| panic!("invalid CST token ref start delta {start_delta}"));
        let end_byte = char_offset_to_byte_index(token.text.as_str(), end_delta)
            .unwrap_or_else(|| panic!("invalid CST token ref end delta {end_delta}"));
        token
            .text
            .get(start_byte..end_byte)
            .unwrap_or_else(|| panic!("invalid CST token text slice {start_byte}..{end_byte}"))
    }

    pub fn text<'a>(&'a self, token_ref: &CstTokenRef) -> &'a str {
        self.token_text(token_ref)
    }

    pub fn token_value<'a>(&'a self, token_ref: &CstTokenRef) -> &'a str {
        self.token_text(token_ref)
    }

    pub fn value<'a>(&'a self, token_ref: &CstTokenRef) -> &'a str {
        self.token_text(token_ref)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CstNode {
    Container(CstContainer),
    Chapter(CstChapter),
    Verse(CstVerse),
    Milestone(CstMilestone),
    Leaf(CstLeaf),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstTokenRef {
    pub token: usize,
    pub span: Span,
}

impl CstTokenRef {
    pub fn token<'a>(&self, document: &'a CstDocument) -> &'a Token {
        document.token(self)
    }

    pub fn token_text<'a>(&self, document: &'a CstDocument) -> &'a str {
        document.token_text(self)
    }

    pub fn text<'a>(&self, document: &'a CstDocument) -> &'a str {
        document.text(self)
    }

    pub fn token_value<'a>(&self, document: &'a CstDocument) -> &'a str {
        document.token_value(self)
    }

    pub fn value<'a>(&self, document: &'a CstDocument) -> &'a str {
        document.value(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstContainer {
    pub kind: CstContainerKind,
    pub marker: String,
    pub marker_token: Option<CstTokenRef>,
    pub close_token: Option<CstTokenRef>,
    pub special_token: Option<CstTokenRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attribute_tokens: Vec<CstTokenRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CstNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstChapter {
    pub marker_token: CstTokenRef,
    pub number_token: Option<CstTokenRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstVerse {
    pub marker_token: CstTokenRef,
    pub number_token: Option<CstTokenRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstMilestone {
    pub marker: String,
    pub marker_token: CstTokenRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attribute_tokens: Vec<CstTokenRef>,
    pub end_token: Option<CstTokenRef>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstLeaf {
    pub kind: CstLeafKind,
    pub token: CstTokenRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CstContainerKind {
    Book,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CstLeafKind {
    Text,
    Whitespace,
    Newline,
    OptBreak,
    Attributes,
}

fn char_offset_to_byte_index(text: &str, char_offset: usize) -> Option<usize> {
    if char_offset == 0 {
        return Some(0);
    }
    let char_count = text.chars().count();
    if char_offset == char_count {
        return Some(text.len());
    }
    text.char_indices()
        .nth(char_offset)
        .map(|(byte_index, _)| byte_index)
}
