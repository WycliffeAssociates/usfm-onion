pub mod token;

pub use crate::internal::marker_defs::{MarkerFamily, SpecMarkerKind};
pub use token::{
    BookCodeToken, BytePos, MarkerMetadata, MarkerToken, NumberRangeKind, NumberRangeToken,
    ScanResult, ScanToken, ScanTokenKind, Span, TriviaToken,
};
