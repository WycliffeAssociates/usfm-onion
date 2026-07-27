# Candidate: index-range decision units + wire string-materialization

Status: **candidate** — doubtful until needed; explicitly NOT required for braid v1.

## Idea

The diff's remaining allocation cost (after the 2026-07-24 `Sid`-key work, branch
`diff-sid-key-spike`) is `build_skeleton`'s `to_vec` clone of token slices + the per-block
`text_full` `String`. With a **resident** corpus (braid holds the tokens, stable across
edits), a `DecisionUnit` could reference a `Range<usize>` into that corpus instead of owning
a cloned `Vec<Token>` / a concatenated `text_full` — no clone, no per-block String, and no
lifetime leak (the handle owns the backing store). This is the single biggest structural
perf lever left, but it only pays once the corpus is resident.

## The catch (why a wire convenience comes with it)

Index-ranges are cheap for a native/resident consumer but push a "slice dance" onto the wire
consumer (JS would have to resolve ranges against its own token array). So pair it with a
**wire-layer convenience that materializes** the token/text strings on demand — the boundary
offers the ergonomic materialized shape so JS never does range arithmetic itself. Core stays
index-based (fast); the adapter opts into materialization.

## When

Only if a heap/latency profile of resident diffing demands it. `1.71 ms`/whole-book today is
already fine (ops works a few chapters at a time). Relates to
[[project-braid-4-crate-architecture]] (§ resident corpus) and the diff-perf memory.
