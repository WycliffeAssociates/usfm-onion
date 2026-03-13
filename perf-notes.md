# Performance Notes

This file is a working backlog for likely performance improvements in
`usfm_onion`.

The goal is not to optimize blindly. The goal is to keep a short list of
changes that are:

- plausible for this codebase
- likely to matter on real corpora
- testable with the existing benchmark setup

Use [src/bin/playground.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/bin/playground.rs)
for throwaway experiments and targeted profiling helpers. Keep the real bench
suite in `benches/` as the source of truth for published numbers.

## Current heuristics

When evaluating an optimization candidate, prefer this order:

1. reduce memory footprint of hot types
2. reduce repeated work by precomputing once
3. reduce string cloning / allocation churn
4. only then reach for heavier architectural tools like arenas

For this crate, "less allocation" is good, but "less repeated work" is often a
better first lever.

## Priority ranking

### 1. Shrink spans from `usize` to `u32`

Why it matters:

- spans are everywhere
- spans appear in tokens, CST refs, syntax nodes, lint issues, diff structures,
  and recoveries
- USFM files in this domain will not approach 4 GiB

Likely impact:

- high

Difficulty:

- medium

Risk:

- medium, because spans are pervasive and serde/public API shapes will change

Suggested direction:

- replace `Range<usize>` with a dedicated span type
- likely shape:

```rust
pub struct Span {
    pub start: u32,
    pub end: u32,
}
```

- add helpers for converting to/from `usize` at boundaries where indexing is
  required

Things to measure:

- memory usage on `en_ult`
- `usfm -> tokens`
- `usfm -> cst`
- lint and diff

### 2. Replace marker strings with a compact representation

Why it matters:

- markers are short
- markers repeat constantly
- markers are carried through tokens, syntax, CST, AST, lint, format, and
  exports

Likely impact:

- high

Difficulty:

- medium to high

Risk:

- medium

Best likely shapes:

- small enum / integer ID for known markers
- owned string only for unknown markers

Possible model:

```rust
enum MarkerRepr {
    Known(KnownMarker),
    Unknown(String),
}
```

Or:

- `CompactStr` / similar inline-string type if we want lower migration cost

Notes:

- this is likely more useful here than generic string interning
- this should reduce both allocations and repeated string comparisons

Things to measure:

- marker-heavy corpora
- lint
- format
- CST / AST construction

### 3. Audit and reduce cloning in CST / AST / export paths

Why it matters:

- CST/AST/export code currently clones marker/text data in many places
- this is a likely real cost center before any arena work

Likely impact:

- medium to high

Difficulty:

- low to medium

Risk:

- low

Where to look:

- [src/cst/mod.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/cst/mod.rs)
- [src/internal/usj.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/internal/usj.rs)
- [src/internal/usx.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/internal/usx.rs)
- [src/internal/html.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/internal/html.rs)

What to look for:

- repeated `.clone()` on marker names
- repeated `.to_string()` in hot loops
- AST/CST construction doing more copying than necessary

Suggested method:

- use `src/bin/playground.rs` to prototype allocation counters or focused
  micro-benches
- only move improvements into main code after checking corpus benches

### 4. Add explicit size tests for hot structs and enums

Why it matters:

- large enums and structs silently hurt cache behavior
- Rust enum size is driven by the largest variant

Likely impact:

- medium

Difficulty:

- low

Risk:

- low

Suggested types to watch:

- `Token`
- `TokenVariant`
- `CstNode`
- `CstContainer`
- `AstNode` / `AstElement`
- lint issue and fix types

Suggested tests:

```rust
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[test]
fn hot_type_sizes() {
    use std::mem::size_of;
    assert!(size_of::<Token>() <= SOME_LIMIT);
    assert!(size_of::<CstNode>() <= SOME_LIMIT);
}
```

These should be guardrails, not dogma.

### 5. Borrow internally in lexer / parser where practical

Why it matters:

- lexing/parsing currently allocate owned strings in places where source slices
  could work
- public API can stay owned while internals borrow

Likely impact:

- medium

Difficulty:

- medium to high

Risk:

- medium to high because lifetimes spread quickly

Recommendation:

- only consider this for internal scan/parser layers
- do not rush public `Token` into lifetime-heavy APIs unless there is a strong
  measured need

Possible patterns:

- `&str` for purely borrowed source segments
- `Cow<'a, str>` where escapes/normalization occasionally require ownership

### 6. Improve lint with more precomputed context

Why it matters:

- lint is multi-pass and structurally aware
- repeated local scans and repeated marker-context work are more likely to hurt
  than loop syntax itself

Likely impact:

- medium

Difficulty:

- medium

Risk:

- low to medium

Recommendation:

- build shared context once
- example candidates:
  - previous/next significant token index
  - chapter/verse index
  - marker context
  - note/container balance state

This keeps lint token-first while avoiding duplicated work.

### 7. Revisit CST resolver only if profiles demand it

What we already learned:

- full linear scans were too slow
- a simple forward-only monotonic cursor is not valid with current span lookup
  order
- a full precomputed hashmap of spans was correct but slower than binary search

Current best-known approach:

- keep the `partition_point` lookup in
  [src/cst/mod.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/cst/mod.rs)

Likely impact of more work here:

- uncertain

Difficulty:

- medium

Risk:

- medium

Only revisit if a profile shows CST lookup is again the dominant cost after
other improvements land.

### 8. Arenas for syntax / CST / AST

Why it might matter:

- parse-related trees are short-lived and hierarchical
- dropping many small allocations can become noticeable

Likely impact:

- unknown to medium

Difficulty:

- high

Risk:

- high

Why this is not first:

- code complexity rises a lot
- lifetimes propagate
- architecture is still settling

Recommendation:

- do this only after the cheaper wins above are measured
- specifically after span shrink + marker repr + clone audit

### 9. SIMD / low-level lexer tricks

Why it might matter:

- whitespace scanning
- marker boundary scanning
- newline detection

Likely impact:

- low to medium

Difficulty:

- high

Risk:

- medium

Recommendation:

- late-stage only
- only after profiling shows lexer hot loops dominate

## Things I would not prioritize yet

### Global string interning

Probably not the best fit here right now.

Reasons:

- this crate parses corpora in parallel
- global/shared interning often introduces hashing and contention overhead
- marker representation improvements are likely more valuable than generic
  interning

If anything, prefer:

- known-marker enum/ID
- compact inline strings

before considering a full interner.

## Benchmark discipline

When trying a change:

1. add the smallest safe experiment in
   [src/bin/playground.rs](/Users/willkelly/Documents/Work/Code/usfm_onion/src/bin/playground.rs)
   if the idea is still exploratory
2. validate correctness with tests
3. run focused corpus timing first
4. only then update shared benches/docs

Useful focused commands:

```bash
cargo test -q
cargo check --benches -q
cargo build --release --features rayon --lib --bench corpus_matrix
target/release/deps/corpus_matrix-<hash> --iterations 1 --corpus en_ult --operation "usfm -> cst" --markdown
```

## Recommended next performance sequence

If we decide to keep pushing on perf, I would do:

1. span shrink to `u32`
2. marker representation cleanup
3. clone audit on CST / AST / exports
4. lint context precomputation
5. then re-measure

Only after that should we consider arenas or SIMD work.
