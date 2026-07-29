//! Wire contract for `usfm_onion`: serde/tsify boundary DTOs, the frozen packed
//! container schema, and the codec that reads and writes it.
//!
//! This crate owns the *format*. It does not own lifecycle state, lint
//! orchestration, or host glue, and it never reimplements core logic — core
//! types cross the boundary through the conversions in [`dto`].

// Internal substrate for the semantic codecs; kept private so construction
// helpers do not become a second, lower-level public wire API.
#[allow(dead_code)]
mod catalog;
#[allow(dead_code)]
mod container;
pub mod dto;
pub mod error;
pub mod finding_codec;
#[allow(dead_code)]
mod finding_section;
// Not part of the wire boundary contract — used by the `generate_js_schema`
// example and its own drift-check test to render the checked-in
// `js/wire-schema.{js,d.ts}` constants module. `pub` only so the example
// binary (an external crate from this lib's point of view) can reach it.
pub mod js_schema;
#[allow(dead_code)]
mod primitives;
pub mod schema;
#[allow(dead_code)]
mod token_codec;
#[allow(dead_code)]
mod token_payload;
#[allow(dead_code)]
mod token_section;

#[cfg(test)]
mod container_tests;
#[cfg(test)]
mod finding_goldens;
#[cfg(test)]
mod token_codec_tests;
#[cfg(test)]
mod token_goldens;

pub use error::{DecodeError, EncodeError, LayoutRefusal};
