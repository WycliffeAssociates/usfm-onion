# Sid occurrence ordinals (v0.1.6)

## Motivation

An editor reported seed refusal: token sids like `"DEU 16:14_dup_1"` —
produced by our own shipped `js/token-sids.js` (`normalizeTokenSids`, the JS
twin of `derive_canonical_sids` in `src/diff/mod.rs`) — were refused by
`OwnedToken::from_parts` with `TokenBuildError::UnresolvableSid`. The
structured `Sid` had no way to represent the positional occurrence, so the
boundary constructor (correctly, at the time) refused to invent one.

The suffix is genuine address information, not noise: duplicate verses are
real data in some corpora, and `"the second 16:14"` is a real anchor an
editor's own tooling already computes and needs to round-trip.

## Two-phase design

**Phase 1 (this release, v0.1.6) — ingest/export only.** Make the spelling
representable everywhere it already flows: `Sid`'s structured form, its
`parse`/`Display`, the packed wire encoding, and the boundary constructors
that accept an editor-authored anchor. `parse` (the USFM lexer/parser) is
**unchanged** — it still mints only bare sids, even for a genuinely
duplicate verse. Nothing in this phase asks parse to detect or number
duplicates itself.

**Phase 2 (deferred) — parse minting and lane unification.** Once occurrence
ordinals are a first-class `Sid` fact, a later release can:
- Have `parse` itself mint occurrences for a duplicate verse/chapter it
  encounters, so a freshly parsed book and a token-pushed one agree.
- Delete `derive_canonical_sids` from `src/diff/mod.rs` — once `Sid` itself
  carries occurrences, diff no longer needs its own parallel derivation.
- Split `vref`'s duplicate handling the same way (`to_vref`'s multi-hit
  entries currently rely on position, not a occurrence-bearing sid).
- Retire `js/token-sids.js`'s `normalizeTokenSids` — once parse mints
  occurrences, the editor no longer needs to stamp them itself.

The phase split means **editor code needs no change** for phase 1 to land:
the exact spelling `normalizeTokenSids` already produces is now accepted
verbatim, with no migration on the editor side.

## `Sid` changes

- Two new fields, default `0`: `verse_occurrence: u8` (`_dup_N`) and
  `chapter_occurrence: u8` (`_cdup_N`). Both derive-carried into equality,
  `Hash`, `Ord`.
- `Sid::with_occurrences(chapter_occurrence, verse_occurrence)` — the only
  writer of non-zero occurrences in this phase; every existing constructor
  (`new`, `with_range`) still defaults both to `0`.
- `Sid::parse` accepts an optional `"_cdup_N"` then an optional `"_dup_N"`
  suffix on the verse segment, in that exact order (matching
  `derive_canonical_sids`'s own write order). `N` must be `1..=255`; `0`, a
  missing/non-numeric/overflowing number, or the wrong order all refuse
  (`None`) rather than repair. Occurrence-0 spellings parse exactly as
  before — this is a pure grammar extension, not a behavior change for
  existing content.
- `Display` appends `"_cdup_N"` (if `chapter_occurrence > 0`) then
  `"_dup_N"` (if `verse_occurrence > 0`), byte-identical to
  `derive_canonical_sids`' own spelling. Occurrence-0 output is
  byte-identical to today.
- `OwnedToken::from_parts`/`from_format_token` needed no logic change —
  acceptance of a dup-suffixed spelling falls straight out of `Sid::parse`
  now succeeding where it used to return `None`.

## Wire layout

The packed-sid dictionary entry (`crates/usfm_onion_wire/src/token_section.rs`,
`PackedSid`; layout table in `crates/usfm_onion_wire/src/schema.rs`,
`layout::packed_sid`) changes from 8 bytes to 16.

### What the codebase actually had (v1, 8 bytes)

| Offset | Field | Width |
|---|---|---|
| 0 | book (ASCII) | 3 |
| 3 | chapter (u16 LE) | 2 |
| 5 | verse (u16 LE) | 2 |
| 7 | delta (low 7 bits) + fidelity (bit 7) | 1 |

The delta and the fidelity bit shared byte 7, which capped an `Exact`
bridge at 127 verses wide before it silently degraded to `AnchorOnly`.

### New layout (v2, 16 bytes)

| Offset | Field | Width |
|---|---|---|
| 0 | book (ASCII) | 3 |
| 3 | chapter (u16 LE) | 2 |
| 5 | verse (u16 LE) | 2 |
| 7 | delta (full byte, 0-255) | 1 |
| 8 | `verse_occurrence` (`_dup_N`) | 1 |
| 9 | `chapter_occurrence` (`_cdup_N`) | 1 |
| 10 | flags (bit 7 = fidelity `AnchorOnly`) | 1 |
| 11-15 | spare (unused, same convention as the sparse number/book-code records' own trailing padding) | 5 |

Two deviations from the original sketch, discovered by verifying against the
real code and real round-trip tests rather than assumed:

1. **The book stays inline; it is not hoisted to the section header.** The
   original design assumed the per-entry book was redundant with the owning
   section's own `book` header field, since every section is already
   one-book. That is *usually* true, but not always: a resident token can
   mint a sid naming a book other than the section's declared one — the
   concrete case is a non-canonical `\id` book code (e.g. `\id xyz lower
   case`), which still needs a lossless round trip per this crate's
   lossless-by-design contract. Two existing native round-trip tests
   (`token_codec_tests::round_trips_every_token_kind` and
   `owned_round_trips_every_token_kind`) exercise exactly this and would have
   silently lost fidelity had the book been dropped. Surfaced rather than
   averaged: correctness of an existing, tested guarantee wins over the
   byte-optimization.
2. **The entry is 16 bytes, not 8.** With the book staying inline, fitting
   the two occurrence bytes plus an unshared flags byte needs 11 bytes
   (3+2+2+1+1+1+1), and this schema's `ElementWidth` vocabulary has no
   variant between `Eight` and `Sixteen` — so the record pads to 16 rather
   than adding a new width case crate-wide (JS mirror included) to save two
   bytes. The 5 spare bytes are unchecked padding, the same convention the
   sparse `NUMBER_RECORD`/`BOOK_CODE_RECORD` (16 bytes each) already use.

A side effect of giving delta its own unshared byte: a bridge up to 255
verses wide (`Sid`'s own ceiling) now stays `Exact`, where v1 degraded past
127. `token_codec_tests::fidelity_comes_from_the_designator_not_the_anchor`
and the token golden `sid-fidelity-wide-bridge-stays-exact` (replacing the
retired `sid-fidelity-overwide-bridge-anchor-only`) cover this.

### Version bump and consequence

`crate::schema::FORMAT_VERSION` bumps `1 -> 2`. The existing version-gate
machinery (`DecodeError::UnsupportedVersion`) refuses a v1 container cleanly
— there is no dual-version decoder. **Consequence:** any v0.1.5-published
container is rejected on restore by this build; a consumer holding one
re-seeds from source (the same recovery path an unreadable/corrupt container
already requires).

## Bounds and refusal rules

- `Sid::parse`'s occurrence suffix: `N` in `1..=255`; `0` is refused (an
  occurrence of "none" is spelled by *absence* of the suffix, never
  `_dup_0`); a non-numeric, leading-zero, or overflowing `N` refuses;
  `_dup_` before `_cdup_` (wrong order) refuses; trailing garbage after a
  recognized suffix refuses. Every refusal is `None` — never a repaired
  guess.
- `PackedSid::encode`'s delta: `Sid::with_range` already saturates
  `verse_end_delta` at `u8::MAX` (255), so the packed delta byte cannot
  overflow given today's `Sid` — the encoder still asserts the bound
  (`debug_assert!`) rather than silently trusting it, so a future change to
  that invariant fails loudly here instead of truncating quietly. No
  `EncodeError` variant was added for this: there is currently no reachable
  input that trips it, and adding refusal machinery for an input that cannot
  occur would be exactly the "error handling for impossible scenarios" this
  codebase's engineering values rule against.

## The mixed-vocabulary consequence

Parse is **unchanged** in this phase. A USFM-parsed book with a real
duplicate verse still mints the *same bare sid twice* — `"GEN 1:1"` both
times, not `"GEN 1:1"` then `"GEN 1:1_dup_1"`. Under `Sid` equality,
`"GEN 1:1" != "GEN 1:1_dup_1"`: they are not the same value, and nothing in
phase 1 pairs them up. This is documented, deliberate contract for phase 1
(see `src/token.rs`'s `mixed_vocabulary_pin` test module), not an oversight
— phase 2's job is to unify the two lanes by having parse mint occurrences
itself.

## Editor-side consequence

None. The phase split means the editor's existing `normalizeTokenSids`
output is now accepted verbatim by `from_parts` — no migration, no new
field the editor must populate, no behavior it must change.
