//! Boundary-only byte-extent glue (v0.1.5, bytes-at-boundary convention).
//!
//! Bytes never cross the wasm boundary as JS number arrays. A corpus-grain
//! byte payload crosses as ONE buffer plus extent records (`byteOffset`/
//! `byteLength`) into it -- one memcpy per buffer regardless of how many
//! books it carries, and the extents themselves are cheap, transferable,
//! forwardable-verbatim values. This module owns the one shape every such
//! extent uses and the (non-`wasm_bindgen`) slicing it takes to resolve one
//! against its buffer, so it is unit-testable natively without a JS engine.
//!
//! [`ByteExtent`] is deliberately boundary-only: no native braid type carries
//! this shape, and it carries no field beyond the two offsets -- anything
//! semantic (a book code, a source key) rides alongside it on the record
//! that names the extent, never inside the extent itself.

use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// A byte range `[byte_offset, byte_offset + byte_length)` into whichever
/// sibling buffer the record pairs it with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ByteExtent {
    pub byte_offset: u32,
    pub byte_length: u32,
}

/// Slices `buf` at `extent`, refusing -- never clamping or truncating --
/// when the extent's end falls outside `buf`.
///
/// The end is computed in `u64` specifically so that an extent whose two
/// `u32` fields would overflow a narrower accumulator (or simply name a
/// range that runs off the end of a small buffer) is caught by the length
/// comparison below rather than by wrapping arithmetic -- `u32 + u32` can
/// never overflow `u64`, so this comparison is the only check needed, and it
/// never panics.
pub(crate) fn slice_extent(buf: &[u8], extent: ByteExtent) -> Option<&[u8]> {
    let start = extent.byte_offset as u64;
    let end = start.checked_add(extent.byte_length as u64)?;
    if end > buf.len() as u64 {
        return None;
    }
    Some(&buf[start as usize..end as usize])
}

/// The same slice, additionally required to be valid UTF-8.
///
/// Native source-bytes verbs take `&str`, so an extent whose bytes are not
/// valid UTF-8 -- including one that lands mid-codepoint in an otherwise
/// valid buffer -- refuses here rather than reaching a native call that
/// would have to lossy-convert or panic.
pub(crate) fn slice_extent_str(buf: &[u8], extent: ByteExtent) -> Option<&str> {
    std::str::from_utf8(slice_extent(buf, extent)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extent(byte_offset: u32, byte_length: u32) -> ByteExtent {
        ByteExtent {
            byte_offset,
            byte_length,
        }
    }

    #[test]
    fn slices_a_valid_extent() {
        let buf = b"hello world";
        assert_eq!(slice_extent(buf, extent(0, 5)), Some(&b"hello"[..]));
        assert_eq!(slice_extent(buf, extent(6, 5)), Some(&b"world"[..]));
    }

    /// Zero-length extents pass through as an empty slice rather than being
    /// special-cased into a refusal -- downstream classification of an empty
    /// source already has its own defined refusal path, and this layer
    /// should not pre-empt it.
    #[test]
    fn zero_length_extent_passes_through_as_empty() {
        let buf = b"hello";
        assert_eq!(slice_extent(buf, extent(0, 0)), Some(&b""[..]));
        assert_eq!(slice_extent(buf, extent(5, 0)), Some(&b""[..]));
    }

    #[test]
    fn out_of_bounds_extent_refuses_rather_than_clamping() {
        let buf = b"hello";
        assert_eq!(slice_extent(buf, extent(0, 6)), None);
        assert_eq!(slice_extent(buf, extent(6, 0)), None);
        assert_eq!(slice_extent(buf, extent(3, 3)), None);
    }

    /// An offset/length pair large enough that a naive same-width addition
    /// would overflow must still refuse cleanly -- never panic, never wrap
    /// into an in-bounds-looking value.
    #[test]
    fn an_overflowing_offset_plus_length_refuses_without_panicking() {
        let buf = b"hello";
        assert_eq!(slice_extent(buf, extent(u32::MAX, u32::MAX)), None);
        assert_eq!(slice_extent(buf, extent(u32::MAX, 1)), None);
        assert_eq!(slice_extent(buf, extent(1, u32::MAX)), None);
    }

    #[test]
    fn invalid_utf8_extent_refuses() {
        let buf = &[0xff, 0xfe, 0xfd];
        assert_eq!(slice_extent_str(buf, extent(0, 3)), None);
    }

    /// A boundary that lands mid-codepoint in an otherwise entirely valid
    /// UTF-8 buffer must still refuse -- the buffer's own validity is not
    /// enough; the extent's own slice must be independently valid.
    #[test]
    fn a_boundary_splitting_a_multibyte_codepoint_refuses() {
        let buf = "héllo".as_bytes();
        // 'é' is a two-byte codepoint starting at index 1.
        assert_eq!(slice_extent_str(buf, extent(0, 2)), None);
    }

    #[test]
    fn valid_utf8_extent_succeeds() {
        let buf = "héllo".as_bytes();
        assert_eq!(
            slice_extent_str(buf, extent(0, buf.len() as u32)),
            Some("héllo")
        );
    }
}
