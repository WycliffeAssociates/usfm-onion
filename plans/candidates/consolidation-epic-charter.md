# Candidate — consolidation epic charter (post-braid type & pass reduction)

Date: 2026-08-03. Status: candidate — owner-requested charter, drafted at v0.1.2. Runs only
when editor integration has settled. Companion idea doc: `single-pass-engine.md`.

## Why (owner retrospective, agreed)

The braid epic accreted types and passes phase-by-phase instead of designing them once: ~6
token traits, several token representations, a formatted-String Sid in hot paths, and a lint
pass that re-walks what parse just produced. None of it is architecturally wrong (the shipped
semantics match the owner's own mental model), but the presentation costs comprehension and
some measured performance. This epic deletes types; it must not add any.

## Charter rules

1. **Delete-targets are named up front** (below); the epic's success metric is types REMOVED.
   Anything new must replace ≥2 existing things.
2. **No external surface change** until internals settle: npm .d.ts byte-identical, wire bytes
   golden-stable, braid verb signatures frozen. Internal-only until a deliberate final surface
   commit.
3. **The harness is the enabler**: oracle byte-identical, parity transcript, packed
   equivalence, publish round-trip, --all-features deny-warnings, strict rustdoc, fresh-clone.
   Every step lands green against ALL of them.
4. **Spike-first for anything with a perf claim** (standing rule): prove the win in a
   throwaway before invasive refactor.
5. Process rules earned during braid apply: contract-closure review question; no
   framing/hardening before a consumer exists; one implementation per behavior.

## Work items (rough order)

1. **Glossary** (day one, cheap): one doc defining Lexeme, Token, Sid, Finding, Fix, Patch,
   Snapshot, Corpus/Braid, Publication — each mapped to its ONE type. Every rename below
   conforms to it.
2. **Packed Sid** — `u8 u8 u8 u16 u16 u8` + exactness high bit, one definition in core;
   delete formatted-String sid keys from hot paths (diff n-way keying, vref accumulation).
   Evidence already in hand: diff-perf memory, vref alloc spike (~1/3 of vref self-time is
   per-verse `format!` + BTreeMap<String> insert).
3. **One Token** — keep borrowed `Lexeme` (zero-copy); collapse owned representations to one
   `Token` + `Option<Box<MarkerExtras>>`; A/B the boxing (text-heavy vs marker-heavy walks)
   before committing. `from_parts` presence semantics carry over unchanged.
4. **Trait collapse** — the Walkable/Diffable/Lintable/Formattable family reduces toward one
   iteration trait; walkers/exporters read Token directly.
5. **Streaming rule engine** — rules as fold-style reducers, slice driver first (behavior
   identical, oracle-proven). Parse driver (lint-during-parse) only if the spike shows ≥10%
   end-to-end ingest+lint; see single-pass-engine.md §2 for why the engine must never fork.
6. **Lex⊕parse fusion** — drop the materialized lexeme stream; parser pulls. Keep the
   chapter-boundary parallel lane with its explicit sequential stitch fold.
7. **LintIssue probing** (owner: "a type we should have probed further") — size audit,
   template/param rendering shape, the parked lint-issue-size-trim candidate folds in here.
8. **Spike question, no commitment: columnar resident store** — resident memory mirrors the
   wire's column layout; Token becomes a view; publish ≈ splice. Kill criterion required
   before any production work.

## Explicit delete-targets

- The trait family above (target: ≥4 traits removed).
- All but one owned token type (FormatToken folds into Token + builder, or is justified in
  writing as the one exception).
- Formatted-String sid keys in diff/vref internals.
- The braid lifecycle DTO hand-mirror in wasm resident.rs (build braid's `wasm` feature —
  already ledger-recorded debt).
- The materialized lexeme intermediate.

## Non-goals

- No wire format changes (schema stays; columnar store spike must PROVE compatibility).
- No behavior changes observable through any shipped surface; oracle stays byte-identical
  except knowingly re-blessed internal-representation artifacts (expect none).
- No new abstractions "for flexibility."
