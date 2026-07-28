//! Marker-catalog stamp.
//!
//! A section's packed marker ordinals, and its name-based metadata recovery, are
//! only sound while the marker registry the encoder saw and the registry the
//! decoder calls are the same. Crate version alone cannot prove that — nothing in
//! the build enforces a version bump per registry edit — so the stamp is a
//! content hash over the registry itself.

use std::sync::OnceLock;

use usfm_onion::marker_catalog;

use crate::primitives::integrity_checksum_parts;

/// xxhash3-64 over the ordered catalog entries' identifying fields.
///
/// Any addition, removal, reorder, or rename changes the value, which is the
/// property the stamp exists for: an ordinal that used to mean one marker must
/// never silently mean another. Computed once per process — the catalog is
/// immutable after its own initialisation.
pub(crate) fn catalog_stamp() -> u64 {
    static STAMP: OnceLock<u64> = OnceLock::new();
    *STAMP.get_or_init(|| {
        let mut bytes = Vec::new();
        for (ordinal, entry) in marker_catalog().all().iter().enumerate() {
            bytes.extend_from_slice(&(ordinal as u32).to_le_bytes());
            push_str(&mut bytes, &entry.marker);
            push_str(&mut bytes, entry.canonical.as_deref().unwrap_or(""));
            // Debug formatting is acceptable *inside a hash*: the value never
            // reaches an API, and any variant rename or addition is exactly the
            // change the stamp must notice.
            push_str(&mut bytes, &format!("{:?}", entry.kind));
            push_str(&mut bytes, &format!("{:?}", entry.family));
            push_str(&mut bytes, &format!("{:?}", entry.category));
        }
        integrity_checksum_parts(&[&bytes])
    })
}

/// Length-prefixed, so `("ab", "c")` and `("a", "bc")` cannot hash alike.
fn push_str(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_is_stable_and_nonzero() {
        // Zero is the "checksum omitted" value elsewhere in the format; a stamp
        // that collided with it would read as "unstamped".
        assert_ne!(catalog_stamp(), 0);
        assert_eq!(catalog_stamp(), catalog_stamp());
    }
}
