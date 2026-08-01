# Plan stub — format/fix attribute passthrough (pre-braid pull-forward)

Date: 2026-07-27. Status: **approved for planning — pulled forward out of the braid epic**, same
pattern as the landed `serializable-token-contract` pull-forward. This is a stub; a full plan is
written before implementation.

## Problem (Gate 0C finding F1)

`attributes` / `attributeSource` do not survive any core path routed through `FormatToken`:
format, `applyTokenFix`, merge, revert, and the walk-token legs. Root cause is core, not adapters:

- `FormatToken` (`src/format/mod.rs:396`) has 9 fields and none for attributes;
- `FormattableToken` (`src/format/mod.rs:449`) has no attribute accessor;
- wasm `map_format_token`/`map_walk_token` therefore hardcode
  `attributes: Vec::new(), attribute_source: None` on reconstruction.

This is pre-existing, golden-blessed behavior: fixture `attributes.usfm` contains
`\w gracious|lemma="grace" \w*` and the committed `attributes/format.usfm` golden emits
`\w gracious\w*` — the lemma is silently stripped.

## Why it blocks braid

Braid's `prepare_format_patch` / `apply_patch` path (braid-epic §5.3) routes through exactly these
legs, and §9 requires resident serialization losslessness. As things stand, applying a format patch
to a book containing `\w |attr=…` payload would corrupt resident state. §15 names this footgun
("Payload loss"). Phases C/E must not land on top of a lossy format leg.

## Directed design shape (owner, 2026-07-27)

Do NOT fix this by adding attribute fields to `FormatToken` and chasing fields forever. Instead,
make format/fix generic over the caller's token — the §4.3 option-2 pattern already proven by
`SerializableToken`:

- the algorithm demands only the minimal accessors/mutators it actually reads and mutates;
- it returns the caller's own token shape, with every field it did not touch passed through
  unmodified (in TS terms: `format<T extends MinimalFormatToken>(tokens: T[]): T[]` — give a token,
  get the rest of that object back).

Untouched payload (attributes today, anything added tomorrow) then survives by construction rather
than by enumeration.

## Consequences to adjudicate in the full plan

- The `attributes/format.usfm` golden (and any sibling goldens) will change: attributes will start
  surviving `format()`. That is an oracle-visible behavior change requiring its own explicit bless
  in a clean checkout (CRLF hermeticity rules apply), never bundled into a refactor.
- Audit which of format/fix/merge/revert/walk actually mutate attribute-bearing tokens and whether
  any rule legitimately must edit attribute text.
- `FormatToken`'s replacement/retirement plan and the wasm mapping cleanup
  (`map_format_token`/`map_walk_token` hardcoding).

## Relates to

- Gate 0C evidence: `plans/approved/braid/gate0-0c-api-ledger.md` §6 (finding F1, conversions C9/C10).
- Braid epic: `plans/approved/braid/braid-epic.md` §4.3, §9, §15; blocks Phases C/E.
- Precedent: the landed `SerializableToken` de-fork (two emitters, generic over a minimal trait).
