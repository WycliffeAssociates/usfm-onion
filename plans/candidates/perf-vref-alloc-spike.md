# Candidate — vref projection allocation spike

Date: 2026-08-01. Status: candidate — owner-deferred at the v0.1.0 tag ("leave as a candidate
idea to speed up"). Measurement-backed; see the 2026-08-01 perf report in
`plans/approved/braid/progress-braid-epic.md` (commit `c56aa39`).

## Evidence (samply, native release, whole-corpus `tokens_to_vref_index`)

After discarding ~25% attribution noise (confirmed not real panics), ~35% of remaining
self-time is alloc/fmt machinery: `String`-as-`fmt::Write` → `core::fmt::write` →
`format_inner`, immediately followed by `BTreeMap::insert` — the signature of one
`format!`-built String sid key allocated and inserted per verse. The other ~65% is honest
projection work (`walker::walk` ~12% of total, `utf16_len` ~9%).

Absolute numbers are NOT a problem today (whole corpus ~39.7ms native; keystroke chapter scope
15µs native / 0.05ms wasm) — this is an optimization opportunity, not a defect.

## Hypotheses, in priority order (prove in a throwaway spike before any production refactor)

1. Per-verse sid-string formatting (`verse_ref_str`-style `format!`) — replace with a
   pre-sized push-based builder or interned/typed key.
2. `BTreeMap<String, _>` keying compounds #1 — a `Sid`-typed compact key (already the
   direction settled for n-way diff; see the diff-perf SID-stringification finding) removes
   both the format and the string-compare cost. Note the RFC 1 ordered-entries shape already
   moved the public surface off BTreeMap; internal accumulation may still key on String.
3. Per-token `Segment`/text `Vec`/`String` reallocation in `push_token`/`flush_current_verse`
   — pre-size from token counts.

## Gates for any landed fix

- Byte-identical output vs current projection over the corpus (vref oracle hashes unmoved).
- Quotable numbers from the quiet bench box, not a local Mac.
- Spike-first per standing practice; no invasive refactor without a proven win.
