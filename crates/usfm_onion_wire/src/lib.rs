//! Wire contract for `usfm_onion`: serde/tsify boundary DTOs, the frozen packed
//! container schema, and the codec that reads and writes it.
//!
//! This crate owns the *format*. It does not own lifecycle state, lint
//! orchestration, or host glue, and it never reimplements core logic — core
//! types cross the boundary through the conversions in [`dto`].

// Internal substrate for the semantic codecs; kept private so construction
// helpers do not become a second, lower-level public wire API.
#[allow(dead_code)]
mod container;
pub mod dto;
pub mod error;
#[allow(dead_code)]
mod primitives;
pub mod schema;

#[cfg(test)]
mod container_tests;

pub use error::{DecodeError, EncodeError, LayoutRefusal};
