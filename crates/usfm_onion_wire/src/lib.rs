//! Wire contract for `usfm_onion`: serde/tsify boundary DTOs, the frozen packed
//! container schema, and the codec that reads and writes it.
//!
//! This crate owns the *format*. It does not own lifecycle state, lint
//! orchestration, or host glue, and it never reimplements core logic — core
//! types cross the boundary through the conversions in [`dto`].

pub mod container;
pub mod dto;
pub mod error;
mod primitives;
pub mod schema;

pub use error::{DecodeError, EncodeError, LayoutRefusal};
