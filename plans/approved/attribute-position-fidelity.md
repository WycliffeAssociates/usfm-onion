# Plan — attribute-position fidelity for the reconstruct emitter (pre-Phase-F pull-forward)

Date: 2026-07-28. Status: **approved** — owner adjudicated: fix before braid Phase F, on
conceptual-correctness grounds (the editor does not typically handle alignment files today, but
the model should be correct regardless).

## Problem

The generic reconstruct emitter (`tokens_to_usfm_reconstruct*`, the span-less/owned-token
serializer) re-attaches each token's verbatim `attribute_source` at its single placement rule —
adjacent to the marker's closer. The attribute TEXT always survives byte-for-byte; its POSITION
was dropped when the token became owned (Gate 0C conversion C9: "the verbatim text survives, its
position does not"). For 19 of 395 corpus fixtures — old-format alignment (`\k-s |x-tw="…"` at
the opener, lines above its `\k-e\*`), unclosed `\fig`, newline-split lists — the derived source
is content-identical but byte-shifted from the original. The span-based borrowed serializer is
unaffected (it remembers real offsets; 425/425 byte-lossless).

The divergent set is pinned by the owned corpus gate
(`corpus_owned_token_sections_round_trip`) as an enumerated 19-path constant with documented
causes; this plan's completion criterion is that constant going to **empty**.

## Fix shape

Give the owned token one more remembered fact — where its attribute list sat — captured at
token-creation time, and teach the reconstruct emitter to honor it:

- `OwnedToken` gains an attribute-placement field (roughly opener-adjacent vs closer-adjacent,
  or an offset-within-token representation; design decides the minimal encoding that covers all
  19 observed shapes, including the unclosed-marker and newline-split cases).
- `OwnedToken::from_parsed` populates it from the parsed token's real attribute span.
- Token-ingest paths (wire `TokenDto`, editor `lexicalToTokens`) default it to today's behavior
  (closer-adjacent) — no consumer change required; live editor tokens never carried positions.
- The reconstruct emitter places the verbatim list per the field; `ReconstructedSpans` continues
  to report where it landed.
- Wire: the owned-encode divergence list shrinks to empty; no layout change (attribute records
  already store spans into the derived source, which simply becomes byte-identical to the
  original for parse-origin tokens).

## Gates

- The owned corpus gate's divergent-path constant becomes empty; byte-identity asserted 395/395.
- All existing oracles byte-identical (this changes no parsed-path behavior).
- `OwnedToken` shape change lands BEFORE braid Phase C consumes the type broadly.

## Relates to

- Gate 0C ledger conversion C9; owned corpus gate in `crates/usfm_onion_wire`.
- Braid epic §4.3/§15 (algorithm-fork rules), §9 losslessness.
- Precedent: `format-token-attribute-passthrough.md` (same "carry the missing fact, don't chase
  fields" philosophy).
