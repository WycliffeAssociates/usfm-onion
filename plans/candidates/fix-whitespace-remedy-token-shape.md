# Candidate — reshape whitespace remedies as inserted tokens

Date: 2026-07-31. Status: candidate — owner-deferred during the braid epic (C4 census finding;
ruled 2026-07-31: braid ships with editor-parity behavior; fix post-epic).

## Problem

The fixes for `missing-whitespace-before-marker` (and the related whitespace remedies) prepend
the whitespace into the **marker token's own source** (`"\n\\p"`) instead of inserting a
whitespace/newline token before it. Consequences, pinned by test during C4:

- Patched bytes are correct and a fresh parse of them is clean.
- But after an in-place (resident token stream) application, the rule still fires — it reads the
  *previous* token's trailing character, which the fix never changed.
- The stream holds a marker token whose source no lexer would produce (`"\n\\p"`).

The shipped editor's fix button has the same behavior today, so this is an inherited wart, not a
braid regression.

## Fix shape

Express the remedy as an **inserted newline/whitespace token** before the marker:

- Needs an insert-before op (the frozen §5.2 patch discriminant set has insert/replace/delete;
  C4's census found `InsertAfter` producerless — this fix would give insertion its first real
  producer, and the before/after semantics need pinning).
- Re-runs the 92-payload fix census (0D ledger §9) — the census results are shape-dependent.
- Touches core's remedy construction for codes 24/25 (26's tag-delimiter fix is a same-token
  text replacement and stays as-is).

## Gates

- Resident application of the fix clears the finding on re-lint without a re-parse.
- Every resident token's source is lexer-producible (no `"\n\\p"` composites).
- Patched bytes byte-identical to today's output (the *bytes* were never wrong).
- Census re-run recorded; wire patch-table goldens re-blessed knowingly.

## Relates to

- C4 census, 0D ledger §9; freeze §J.1/§5.2/§N; the C4 test that pins today's behavior.
