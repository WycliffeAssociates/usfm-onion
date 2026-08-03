//! Native composition layer: braid's resident-corpus semantics on one side,
//! wire's packed bytes on the other.
//!
//! This is the layer the two crates are deliberately kept from being: braid
//! never learns a byte layout and wire never learns what a dirty book or a
//! snapshot id means, so *something* has to know both, and this crate is it.
//! It exists so that composition is written once -- previously it lived only
//! inside `usfm_onion_wasm`, which meant every other Rust host (a native
//! Tauri backend, a CLI, a test harness) had to hand-duplicate it.
//!
//! Rust-first: this is the primary API, not a byproduct of the wasm bindings.
//! `usfm_onion_wasm`'s `Braid.publish`/`restorePublishedCorpus` are thin
//! `#[wasm_bindgen]` projections over exactly the functions here --
//! borrowed/owned/serde glue, nothing more. A native host calls
//! [`publish_corpus`]/[`restore_published_corpus`] directly, with no wasm
//! runtime involved at all.

mod publication;
mod restore;

pub use publication::{
    PublicationCache, PublishError, PublishedBookInfo, PublishedCorpus, publish_corpus,
};
pub use restore::{PublishedCorpusSource, RestoreError, restore_published_corpus};
