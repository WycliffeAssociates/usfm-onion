# Candidate: chapter-grain lint computation inside braid

Status: **candidate** — preserve the direction, do not build it in braid v1.

## Idea

Keep braid's public contract unchanged—complete resident corpus, complete chapter/book updates,
explicit `lint()`, and a complete finding snapshot—but allow a chapter edit to recompute only the
rule work whose semantics genuinely fit that grain.

This is the lint-compute analogue of the resident rule hierarchy used by scripture-sous-chef's
Galley. It is not permission to call today's `LintScope::Chapter` for every rule. Onion currently
has a mixed rule topology:

- range-local and walker-driven rules can already run over chapter segments;
- structure, duplicate-chapter, and number/verse families currently retain whole-book work; and
- the editor currently sends whole-book token batches and defaults Onion lint scope to `"book"`.

Braid v1 therefore marks the changed book dirty and reuses Onion's existing whole-book lint path.
This candidate is revisited only after product-shaped measurements show that dirty-book lint is
worth reducing or after the shared internal hierarchy provides enough maintenance value on
its own to justify the work.

## The internal rule-grain hierarchy

A lint rule family declares the smallest semantic lane it can honestly use. This is a closed,
core-owned hierarchy of rule grains, not a third-party plugin system and not a demand that every
rule fit chapter scope.

```text
changed chapter tokens
        ↓
chapter-local map lane
        ↓ typed chapter products
ordered book-reduce lane ─────┐
                              ├→ canonical complete-book findings
whole-book batch lane ────────┘
        ↓
replace that book's resident finding partition
        ↓
publish complete-corpus snapshot
```

Candidate lanes:

1. **Chapter-local map** — pure rule work whose result depends only on one complete chapter run.
2. **Ordered book reduce** — consumes cached chapter products in source order and carries explicit
   boundary/sequence state where semantics cross chapters.
3. **Whole-book batch** — permanent or transitional home for rules that still require the full
   token stream. This is a visible correctness lane, not a failure.
4. **Corpus reduce** — add only if a real rule spans books. Do not create it speculatively.

The rule registry and lane contracts must be core-owned. Braid owns resident products,
invalidation, and orchestration; it must not duplicate private rule lists from `lint_impl.rs`.

## Why it is a candidate

- Whole-book Onion lint is presently cheap enough that packed boundary/materialization work is the
  more important first target.
- `LintScope::Chapter` exists, but it primarily gates whether document-level rules are valid for a
  supplied slice; it is not proof that complete book semantics decompose into independent chapter
  calls.
- The editor does not currently thread chapter-grain scope. Its service layer deliberately
  defaults to whole-book lint and records chapter scope as a TODO.
- The existing parallel linter explicitly keeps several families whole-book. Replacing that with
  “chapter lint plus duplicate-chapter reduce” would silently lose behavior.
- A generic cache/framework built before a rule inventory would be speculative infrastructure.

## Gate 0 for reconsideration

Before designing types or changing core lint:

1. Measure braid dirty-book lint separately from wire encode, transfer, JS decode, and reconcile
   on editor-shaped MAT/PSA/GEN projects.
2. Inventory every `LintCode` and assign it provisionally to chapter map, ordered book reduce, or
   whole-book batch, with a reason.
3. Record each rule's actual inputs, boundary state, output ordering, dedupe behavior, and fix
   payload.
4. Pin the current serial/parallel finding order, including stable-sort ties and duplicates.
5. Prove the proposed chapter-run invariant over adversarial fixtures, including duplicate and
   reopened chapter labels.
6. Decide whether the motivation is measured edit latency, architectural consistency with Galley,
   or both. Do not claim a performance win from architecture alone.

Stop if the inventory cannot reproduce the existing whole-book oracle without rule-specific
special cases hidden in braid.

## Likely type direction, not yet normative

The eventual core seam may resemble:

```rust
pub enum LintGrain {
    ChapterMap,
    OrderedBookReduce,
    WholeBookBatch,
}

pub trait ChapterMapper {
    type Product;

    fn map_chapter(
        &self,
        chapter: ChapterTokenView<'_>,
        options: &LintOptions,
    ) -> Self::Product;
}

pub trait BookReducer {
    type Product;
    type State: Clone + Eq;

    fn initial_state(&self, front: Option<&Self::Product>) -> Self::State;

    fn reduce_chapter(
        &self,
        state: &Self::State,
        chapter: &Self::Product,
    ) -> (Self::State, Vec<LintIssue>);
}
```

These names and shapes are placeholders. Do not introduce `dyn Any`, a rule dependency graph,
boolean capability flags, or an abstraction with no migrated consumer. Prefer one real rule
family per lane to drive the interface.

The initial migration may legitimately keep the entire current linter as one
`WholeBookBatch` implementation. That preserves the destination architecture without pretending
incremental compute has already landed. Extract chapter products only in reviewable steps, each
guarded by whole-book parity.

## Correctness contract

- Mutation narrows computation, never the semantic scope of `lint()`.
- The returned finding snapshot remains complete for the resident corpus.
- Caller/source order and duplicate chapter runs remain intact.
- No rule reads another rule's enabled state or result; shared evidence becomes a typed product.
- Ordered reduction carries explicit state and converges by equality, not an arbitrary replay cap.
- Whole-book batch work remains available for rules that do not fit.
- Canonical finding order, dedupe, suppressions, summaries, message parameters, and token patches
  remain byte/semantic equivalent to current whole-book lint.
- Web/native and serial/parallel modes differ only in execution strategy, never results.

## Candidate execution shape

1. Add timing/inventory instrumentation without changing behavior.
2. Introduce one core-owned batch-lane seam around today's complete linter.
3. Move one already range-local rule family to a typed chapter product; prove oracle parity.
4. Add resident per-chapter product storage/invalidation in braid for that family only.
5. Introduce an ordered reducer only when a real cross-chapter family drives its state type.
6. Migrate further rules individually; retain an explicit batch ledger for the remainder.
7. Compare measured edit latency and complexity after every migration; stop when returns flatten.

Each step must be independently releasable and preserve the complete-snapshot public API.

## Non-goals

- No change to braid's public lifecycle.
- No requirement that every rule become chapter-local.
- No chapter-local finding publication or partial semantic answer.
- No copying private rule classifications into braid.
- No rule-to-rule dependency graph.
- No speculative corpus-wide lane.
- No threaded wasm, workers, cancellation, or background analysis.
- No claim that format/diff chapter scoping proves lint decomposability; their semantics differ.

## Revisit signal

Revisit when at least one is true:

- dirty-book lint is a measured interactive bottleneck after packed transport/reconcile lands;
- adding or maintaining rules is materially harder without a shared typed lane architecture; or
- an adjacent core refactor naturally exposes typed chapter products with little additional cost.

Relates: `../approved/braid-epic.md` §§2.2, 6.4, 16–17.
