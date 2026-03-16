pub mod lexer;
pub mod marker_defs;
mod marker_defs_data;
#[path = "markers.rs"]
pub mod markers;
pub mod token;

// pub use marker_defs::{MarkerFamily, SpecMarkerKind};
// pub use token::{
//     BookCodeToken, BytePos, MarkerMetadata, MarkerToken, NumberRangeKind, NumberRangeToken,
//     ScanResult, ScanToken, ScanTokenKind, Span, TriviaToken,
// };
// pub use lexer::lex;
