//! Checked little-endian read/write primitives.
//!
//! Every read is bounds-checked and every offset/length computation is checked
//! arithmetic, so no decode path can index out of range or size an allocation
//! from an unvalidated count. Nothing here panics on any input, including
//! adversarial counts and offsets at the `u32`/`u64` edges: length trouble is a
//! [`DecodeError`], never a slice panic.
//!
//! v1 is little-endian only; there is deliberately no big-endian reader.

use xxhash_rust::xxh3::Xxh3;

use crate::error::DecodeError;

/// Converts a wire-declared `u64` to a host index. Distinguishes "the number is
/// too large to be an address on this platform" from "the buffer is short", so a
/// 32-bit (wasm) host rejects a 64-bit offset instead of wrapping it.
pub(crate) fn index_from_u64(value: u64) -> Result<usize, DecodeError> {
    usize::try_from(value).map_err(|_| DecodeError::OffsetOverflow)
}

/// Borrows `len` bytes at absolute `offset`, validating the arithmetic before
/// the range. Overflowing arithmetic is [`DecodeError::OffsetOverflow`]; sound
/// arithmetic that runs off the end is [`DecodeError::Truncated`].
pub(crate) fn window(bytes: &[u8], offset: u64, len: u64) -> Result<&[u8], DecodeError> {
    let end = offset.checked_add(len).ok_or(DecodeError::OffsetOverflow)?;
    let start = index_from_u64(offset)?;
    let end = index_from_u64(end)?;
    bytes.get(start..end).ok_or(DecodeError::Truncated)
}

/// `count * width` as a `u64`, refusing the overflow rather than wrapping into a
/// small, plausible-looking allocation size.
pub(crate) fn checked_extent(count: u64, width: u64) -> Result<u64, DecodeError> {
    count.checked_mul(width).ok_or(DecodeError::OffsetOverflow)
}

/// xxhash3-64 over the concatenation of `parts`.
///
/// The encoder hashes its still-unassembled header and body through this;
/// [`integrity_checksum`] hashes an assembled buffer through the same function
/// with the checksum field replaced by zeros. One implementation means the two
/// directions cannot drift into hashing different byte sequences.
pub(crate) fn integrity_checksum_parts(parts: &[&[u8]]) -> u64 {
    let mut hasher = Xxh3::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.digest()
}

/// xxhash3-64 over `bytes` with the eight bytes at `hole` read as zero — the
/// integrity-checksum rule for both the container header and every section
/// header.
///
/// A slice too short to contain the hole is hashed as-is; the result cannot
/// match a canonically written checksum, so such input rejects rather than
/// being accepted on a partial hash. Callers validate the header length first,
/// so that path is unreachable in practice.
pub(crate) fn integrity_checksum(bytes: &[u8], hole: usize) -> u64 {
    let after = hole.saturating_add(8);
    match (bytes.get(..hole), bytes.get(after..)) {
        (Some(before), Some(rest)) => integrity_checksum_parts(&[before, &[0u8; 8], rest]),
        _ => integrity_checksum_parts(&[bytes]),
    }
}

/// xxhash3-64 of a book's exact source bytes. Paired with the source length in
/// the section header: the length is one compare and catches the common mistake,
/// the hash catches a same-length different-bytes source.
pub(crate) fn source_hash(source: &str) -> u64 {
    integrity_checksum_parts(&[source.as_bytes()])
}

/// Sequential little-endian reader over a validated byte window.
pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(DecodeError::OffsetOverflow)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(DecodeError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let slice = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.array()?))
    }
}

/// Append-only little-endian writer. Emission order is the only thing that
/// decides output bytes, so a given input sequence always produces the same
/// buffer — there is no map iteration or address-dependent ordering anywhere in
/// the write path.
#[derive(Default)]
pub(crate) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(crate) fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn bytes(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    /// Zero-pads until `base + len()` is a multiple of `align`. `base` is where
    /// this buffer sits in the structure being built, so alignment is computed
    /// against the final position rather than the local one. Padding is zero so
    /// that the same logical content always produces the same bytes, gaps
    /// included.
    pub(crate) fn pad_to_from(&mut self, base: usize, align: usize) {
        if align > 1 {
            let position = base + self.bytes.len();
            let padded = position.next_multiple_of(align);
            self.bytes.resize(self.bytes.len() + (padded - position), 0);
        }
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_separates_overflow_from_truncation() {
        let bytes = [0u8; 8];
        assert_eq!(
            window(&bytes, u64::MAX, 1),
            Err(DecodeError::OffsetOverflow)
        );
        assert_eq!(window(&bytes, 4, 8), Err(DecodeError::Truncated));
        assert_eq!(window(&bytes, 8, 0).map(<[u8]>::len), Ok(0));
    }

    #[test]
    fn cursor_never_panics_at_the_end_of_its_window() {
        let mut cursor = Cursor::new(&[1u8, 2, 3]);
        assert_eq!(cursor.u16(), Ok(0x0201));
        assert_eq!(cursor.u32(), Err(DecodeError::Truncated));
        assert_eq!(cursor.u8(), Ok(3));
        assert_eq!(cursor.u8(), Err(DecodeError::Truncated));
    }

    #[test]
    fn checksum_ignores_the_hole_contents() {
        let mut a = vec![7u8; 32];
        let mut b = a.clone();
        a[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
        b[24..32].copy_from_slice(&1u64.to_le_bytes());
        assert_eq!(integrity_checksum(&a, 24), integrity_checksum(&b, 24));
        assert_ne!(integrity_checksum(&a, 24), integrity_checksum(&a, 16));
    }

    #[test]
    fn padding_aligns_the_final_position_not_the_local_one() {
        let mut writer = Writer::default();
        writer.u8(1);
        writer.pad_to_from(48, 16);
        assert_eq!(writer.len(), 16);
        let bytes = writer.finish();
        assert!(bytes[1..].iter().all(|byte| *byte == 0));

        let mut writer = Writer::default();
        writer.u8(1);
        writer.pad_to_from(47, 16);
        assert_eq!(writer.len(), 1);
    }

    #[test]
    fn assembled_and_part_wise_checksums_agree() {
        // The encoder hashes parts it has not assembled yet; the decoder hashes
        // the assembled buffer with the checksum field zeroed. They must be the
        // same hash.
        let header = [1u8; 32];
        let body = [2u8; 40];
        let mut assembled = Vec::new();
        assembled.extend_from_slice(&header);
        assembled.extend_from_slice(&body);
        assembled[24..32].copy_from_slice(&0x1234_5678_9abc_def0u64.to_le_bytes());
        let mut hole_free = header;
        hole_free[24..32].copy_from_slice(&[0u8; 8]);
        assert_eq!(
            integrity_checksum(&assembled, 24),
            integrity_checksum_parts(&[&hole_free, &body])
        );
    }
}
