//! The two lint-cache validity stamps a warm-restore or cache-prime input is
//! checked against, and their computation.
//!
//! Both stamps are xxh3-64 over a byte representation braid controls, not
//! serde JSON: derived `Debug` output on core's own [`LintOptions`] is already
//! deterministic (field order is struct declaration order, stable within one
//! build) and requires no new dependency to treat as a canonical form — it
//! only needs to be *a* fixed encoding, not a portable one, since nothing ever
//! decodes a stamp back into a value.

use usfm_onion::lint::{CRATE_VERSION, LintOptions, RULES_VERSION};
use xxhash_rust::xxh3::Xxh3;

/// xxh3-64 over the effective lint configuration's canonical form (the
/// enabled rule set and every per-rule setting).
///
/// This is a deterministic representation of the configuration *as supplied*,
/// not a semantic normalization of it: anything that could change a finding
/// changes the fingerprint, but two configurations that would lint identically
/// may still fingerprint differently (a reordered `enabled_codes` list, say).
/// That is the safe direction — it can only refuse a cache that would in fact
/// have been valid, never accept one that would not. A cached lint contribution
/// is reused only on an exact match: never partially, never "close enough".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LintConfigFingerprint(pub u64);

impl LintConfigFingerprint {
    /// Computed from the same [`LintOptions`] every resident lint pass already
    /// runs with — the config a caller's cached contribution must have been
    /// produced under to be trusted here.
    pub fn of(options: &LintOptions) -> Self {
        let mut hasher = Xxh3::new();
        hasher.update(format!("{options:?}").as_bytes());
        Self(hasher.digest())
    }
}

/// xxh3-64 over `"{crate}@{version}:rules{RULES_VERSION}"` — deliberately
/// coarse: any release of the rule engine this library ships, or any bump to
/// its own rules version, invalidates every lint cache built under a
/// different one. Errs safe rather than trying to prove a release changed no
/// rule's behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LintEngineStamp(pub u64);

impl LintEngineStamp {
    /// The stamp for the rule engine this build of braid actually runs lint
    /// with. There is exactly one honest value per build — nothing here is
    /// caller-supplied.
    pub fn current() -> Self {
        let source = format!("usfm_onion@{CRATE_VERSION}:rules{RULES_VERSION}");
        let mut hasher = Xxh3::new();
        hasher.update(source.as_bytes());
        Self(hasher.digest())
    }
}

#[cfg(test)]
mod tests {
    use usfm_onion::lint::{LintOptions, LintScope};

    use super::{LintConfigFingerprint, LintEngineStamp};

    #[test]
    fn identical_configs_fingerprint_identically_and_different_configs_do_not() {
        let a = LintOptions::scoped(LintScope::Book);
        let b = LintOptions::scoped(LintScope::Book);
        assert_eq!(LintConfigFingerprint::of(&a), LintConfigFingerprint::of(&b));

        let mut c = LintOptions::scoped(LintScope::Book);
        c.allow_implicit_chapter_content_verse = !c.allow_implicit_chapter_content_verse;
        assert_ne!(LintConfigFingerprint::of(&a), LintConfigFingerprint::of(&c));
    }

    #[test]
    fn engine_stamp_is_stable_within_one_build() {
        assert_eq!(LintEngineStamp::current(), LintEngineStamp::current());
    }
}
