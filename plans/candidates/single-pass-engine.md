# Candidate — single-pass engine (lex⊕parse fusion, streaming rule engine, one Token)

Date: 2026-08-03. Status: candidate — owner idea, filled out post-braid-v1 (v0.1.2 shipped).
Source: owner sketch + the "Logos Runtime vs Compile Time Costs" brainstorm (winnow/hand-rolled
single-pass parsing, memchr chapter pre-scan, SWAR notes). Companion charter:
`consolidation-epic-charter.md`.

## The idea (owner's sketch)

Three passes today: lex → parse → lint. Fuse them: one pass over bytes producing only `Token`
(100% binary-compatible with the wire), lint running during the pass. One minimal `Token` type,
extras boxed as `Option`, populated by parse but not when an editor hands tokens back. Output
formats stay walkers over Token.

## Assessment, piece by piece

### 1. Lex⊕parse fusion — endorse, essentially free

We already hand-roll (the brainstorm's own recommendation over a DFA lexer, for the same
reason: USFM whitespace is context-dependent). Change: stop materializing the intermediate
lexeme stream; the parser pulls lexemes as it scans. One byte pass, one `Vec<Token>` out.
No architectural cost. Win is modest (parse ≈0.8ms/Psalms native) — take it as part of the
consolidation, don't sell it as the headline.

The memchr/memmem `\n\c ` chapter pre-scan + rayon parallel parse + sequential stitch fold is
sound and half-built (chapter-split lint, the vref parallel path). The stitch stage is REAL and
we have scar tissue proving it: block-supports-verse state crosses `\c` (RFC 3's P1), and
milestones span chapters. Any parallel parse keeps an explicit sequential stitch/resolve fold.

### 2. Lint-during-parse — fuse the DRIVER, never the engine

Two facts forbid a literal fused parse+lint:

- **Invalidation asymmetry.** Lint invalidates on config/rules changes; parse doesn't. Braid
  relints without reparsing today; fused, every config flip repays the parse.
- **The editor-token lane.** Tokens enter braid with no parse anywhere (caller tokens), so
  lint-over-tokens must exist regardless. A fused pass would be a SECOND implementation of the
  rules — the drift class the braid reviews spent five rounds killing.

The sound version: rules become **streaming reducers** (fold-style state machines over a token
iterator — most already are, underneath the walker). One engine, two drivers:

- parse driver: feeds tokens as they are born (first ingest — the owner's single pass);
- slice driver: runs over resident tokens (relint, config change, editor tokens).

Same rules, same findings, oracle-provable. Honest expectation: fusion saves one Vec traversal
(GB/s) while rule compute dominates — small single-digit % end-to-end. The streaming-reducer
refactor is worth doing EVEN IF the parse driver is never wired: it is the enabler for
chapter-grain rule-state checkpointing (see `chapter-grain-braid-lint.md`).

**Gate:** spike measures ingest+lint end-to-end before/after. If <10%, land the reducer
refactor, skip the parse driver.

### 3. One minimal Token, extras boxed — yes, with one keep and one measurement

- KEEP the borrowed/owned split: `Lexeme` (zero-copy, span+kind) is load-bearing for parse
  speed. Collapse everything ABOVE it.
- `Token` = { id, kind, source, sid } + `Option<Box<MarkerExtras>>` (metadata, structural,
  nested, attributes). Populated by parse; absent/verbatim from an editor — exactly the
  presence semantics `from_parts` already enforces.
- Trait family (Walkable/Diffable/Lintable/Formattable/…) collapses toward one iteration trait.
- MEASURE before committing: boxing shrinks the hot Vec (text-heavy walks: vref) but adds a
  pointer chase on marker-heavy walks (lint reads marker metadata constantly). Cheap A/B;
  remeasure the remembered ~320-byte token while there.

### 4. The sleeper: columnar resident store ("internally SoA" taken literally)

The wire format is already columnar SoA. The ambitious endpoint: the resident store IS the
decoded column set; `Token` is a materialized view. Publish approaches memcpy; token identity
hashes columnar; the wasm boundary gets cheaper again. Big swing (walker/lint/format read
views), so: spike question with a kill criterion, never a commitment ahead of evidence.

### 5. Unchanged

Output formats stay walkers over Token. Lint stays in token space. Packed `Sid`
(u8 u8 u8 u16 u16 u8 + exactness high bit) rides along — evidence-backed twice already
(diff-perf sid stringification; vref alloc spike).

## Standing constraints (from the braid ledger, non-negotiable)

- One rule engine; drivers may vary. Two implementations of anything is the drift class.
- Every projection preserves user/file-supplied order; a sorted container at a boundary is a
  silent sort.
- Oracle/parity/packed/publish gates are the refactor harness; no external surface change
  until internals settle. Byte-identical outputs or knowingly re-blessed.
