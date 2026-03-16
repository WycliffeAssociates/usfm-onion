pub mod lexer;
pub mod marker_defs;
mod marker_defs_data;
#[path = "markers.rs"]
pub mod markers;
pub mod parse;
pub mod token;

pub use lexer::lex;
pub use parse::{into_usfm_from_tokens, parse, parse_lexemes};
pub use token::{
    AttributeEntryToken, AttributeItem, BookCodeToken, LexResult, Lexeme, LexemeKind,
    MarkerMetadata, MarkerToken, NumberRangeKind, NumberRangeToken, ParseAnalysis, ParseResult,
    ScanResult, ScanToken, ScanTokenKind, Token, TokenData, TokenKind, tokens_to_usfm,
};
