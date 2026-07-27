# Candidate: persistent stable token ids for editor linebreaks

Status: **candidate in this repo; implementation belongs in `scripture-editor-proto-2`.**

## Problem

The editor's ordinary USFM nodes carry stable GUID-backed ids, but
`lexicalToTokens()` currently synthesizes linebreak token ids as `linebreak-0`, `linebreak-1`, …
for each serialization. Those ids:

- repeat across chapters within the same book;
- shift when an earlier linebreak is inserted or deleted; and
- therefore cannot satisfy Braid's book-wide stable token-id/reconciliation contract.

Generating a new UUID inside every `lexicalToTokens()` call would remove collisions but make
identity churn worse. The id must be minted once for the logical linebreak and survive later
serializations.

## Candidate direction

Investigate Lexical NodeState as the preferred way to attach a persistent local GUID to built-in
`LineBreakNode` instances without introducing a custom linebreak node class.

Desired lifecycle:

```text
linebreak created/imported without stable id
    → mint local GUID once
    → persist it in node state / serialized editor state
    → lexicalToTokens reads that id
    → undo/redo and ordinary edits retain it
    → deleted linebreak's id disappears and is never reused
```

Use the editor's existing local GUID maker/state conventions if they can be applied to built-in
nodes. The cost should be one GUID when the linebreak is created or first normalized—not one GUID
per lint, save, or tokenization.

## Questions to answer in the editor repo

1. Can Lexical NodeState attach serializable state to a built-in `LineBreakNode` without replacing
   it with a custom node?
2. Does that state survive editor JSON export/import, cloning, history capture, undo/redo,
   collaboration/recovery paths, and chapter rebuilds?
3. At which update boundary can a missing id be minted without mutating during a read-only
   serialization pass?
4. Do paste/import operations clone an existing id and therefore require collision repair?
5. Can normalization prove book-wide uniqueness cheaply when a chapter enters working state?
6. Are there other synthesized token kinds with the same counter-based identity problem?
7. Does persistence of a GUID change existing editor-state snapshots or migration requirements?

## Acceptance contract

- Every token id in a complete book is nonempty and unique.
- A linebreak's id is stable across repeated `lexicalToTokens()` calls with no edit.
- Inserting/deleting an earlier linebreak does not change surviving linebreak ids.
- Undo restores the prior logical linebreak and its id; redo behaves consistently with the
  editor's node-identity policy.
- Save/reopen and crash-recovery round trips preserve ids where the editor promises working-state
  identity.
- Paste/import cannot leave duplicate ids in one book.
- No UUID is minted during each lint/tokenization pass.

## Fallbacks if NodeState cannot support built-in linebreaks

Evaluate in this order:

1. a minimal custom linebreak node carrying the same stable state;
2. a persisted side table owned by chapter/editor state and keyed by a genuinely stable Lexical
   identity;
3. a chapter-stable prefix plus persistent per-linebreak id.

Avoid a position/occurrence-derived id as the final solution: insertion before it rebases identity
and defeats reconciliation. Avoid weakening Braid to accept duplicate ids unless a separate,
evidence-backed identity design replaces the invariant.

## Non-goals

- No editor implementation from this repository.
- No UUID generation on every serialization.
- No global/cross-project identity requirement; book-wide uniqueness is sufficient for Braid v1.
- No custom node unless built-in NodeState is proven inadequate.

Relates: `../approved/braid-epic.md` §§2.2, 10 Phase F, 17.
