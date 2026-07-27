# Discussing: eliminate lint's whole-stream passes (dup-reduction / verse-as-segment)

Extracted 2026-07-22 from the now-mostly-shipped `plan-parallelism.md`. The rest of
that plan (chapter-parallel `Book` lint) **shipped and is byte-identical, oracle-gated**
— deleted. This is the one bit still needing adjudication + measurement.

## Shipped baseline (context)

`lint_tokens` decomposes a `Book` at `\c` into segments; range-local + marker-balance
rules run per segment via `map_ordered`. But three families still each scan the **whole
token stream**, just as concurrent work units rather than a serial tail
(`lint_impl.rs`, `BookWork::{Structure,DuplicateChapter,NumberVerse}`):
- `Structure` — heaviest single unit; sets the concurrent floor.
- `DuplicateChapter` — inherently cross-segment.
- `NumberVerse` — re-derives running verse/chapter context over the entire stream.

## The open idea

Make verse/chapter identity a **property of each chapter segment** so `NumberVerse`
decomposes per-segment, and catch cross-chapter duplicate chapter/verse numbers with a
**cheap post-hoc reduction** (collect each segment's numbers, reduce for dups) instead of
a whole-stream pass.

## What needs adjudicating / measuring

1. **Is it even worth it?** `Structure` is the floor — shrinking `NumberVerse` only helps
   if it's on the critical path, which it likely isn't. Prior read: probably marginal.
2. **Controlled measurement required first.** The parallelism perf magnitudes were noisy
   (Apple-Silicon P/E-core scheduling artifacts on single-thread baselines). Need P-core
   pinning or many-trial medians before any claim. Correctness was solid; magnitude is
   the open question.
3. Must stay byte-identical (oracle gate, incl. `RAYON_NUM_THREADS=1`).

Decision: measure on the quiet box before committing any complexity. Low urgency.
