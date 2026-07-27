# Discussing: braid-adjacent / next-pass candidates

Status: **discussing** (not approved). Parking lot for work that is out of scope for the
first braid pass but likely-next. Each is framed enough to pick up, not fully planned. When
one firms up, graduate it to its own `plans/approved/` doc. Relates to the braid epic and
[[serializable-token-defork]].

Guiding split for the first braid pass (see the epic): braid is opinionated coordination
glue over core's pure functions; the only genuinely stateful/algorithmic pieces are the
binary token transport and corpus ingestion/checksum/eviction; v1 is recompute-not-incremental.
Everything below is deferred out of that first pass.

---

## a. Chapter-level lint (editor-facing)

**What:** lint scoped to a single chapter — re-lint just the edited chapter instead of the
whole book, and/or report lint results bucketed per chapter.

**Why:** editor incrementality (on edit, braid re-lints the touched chapter, not the book)
and it composes with the existing chapter-split perf work.

**Distinct from** the existing `parallelism-strategy` (chapter-split lint = parse-once-then-slice,
a *perf* strategy that produces the identical whole-book result via parallel chapter slices,
gated by `tests/lint_oracle.rs`). This item is the *capability* of scoping lint to a chapter
as a first-class unit.

**Open questions:** Do any lint rules span chapters (cross-chapter references, book-level
structure)? If so a chapter-scoped lint can't see them — need to classify rules as
chapter-local vs. book-global. Must a chapter-scoped run be byte-identical to slicing the
whole-book run? (Yes, per the oracle discipline.)

## b. Patch trait for tokens (avoid full mut Vec round-trips)

**What:** a `TokenPatch` abstraction — express token mutations (insert / delete / replace at
index or range) as patches, so operations return patches instead of a whole mutated
`Vec<Token>` handed back as `&mut`. Editor applies the patch.

**Why:** two motives converge — (1) the long-standing "formatToken eventually isn't `mut` but
rather a patch" intuition; (2) the binary-transport finding that the *boundary* (not parse) is
the cost — shipping a small patch beats round-tripping the full token vec across wasm on every
op (merge, format, fix-application).

**Open questions:** patch representation (index-based vs id-based — id stability matters if the
editor reorders); how merge / format / apply-fix emit patches; how a patch carries *new* tokens
(they'd be `SerializableToken`-shaped); composition/ordering of multiple patches; interaction
with the `attributeSource` verbatim-vs-reconstruct contract.

## c. New lint categories over the token stream (consistency, monotonicity, …)

**What:** lint families beyond today's local/structural rules —
- **Monotonicity:** verse/chapter numbers increase, no unexpected gaps/dupes, ordering sane.
- **Consistency:** term/spelling/quote-style consistency across the stream.

**Why:** the token stream is the natural substrate for whole-stream analytical rules; the
architecture likely already supports adding rule categories (a `LintCode` + a pass).

**Open questions — ownership boundary** (see `lint-sous-chef-whitespace-division`): monotonicity
(verse/chapter ordering) reads as onion's structural domain; content *consistency* (terms,
spelling) may belong to `scripture-sous-chef`, not onion. Decide the line before building.
Also: are these whole-book (conflict with (a) chapter-scoping)?

## d. Emit trait-shaped TS interfaces (enables `exportJson(): LintableToken & SerializableToken`)

**What:** today the wasm boundary emits a single fat `interface Token` (all fields); the Rust
trait family (`UsfmToken` / `SerializableToken` / `WalkableToken` / `LintableToken`) does NOT
appear in `.d.ts` (traits don't cross the ABI). Emit field-subset TS interfaces per trait so an
editor can annotate `exportJson(): LintableToken & SerializableToken` — the fat `Token`
structurally satisfies the intersection.

**Why:** gives the editor the same per-capability contract in TS that Rust/braid get from trait
bounds; structural subtyping is the JS analog of trait bounds. This is the handoff's open Q1.

**Open questions:** mechanism — subset DTO structs with `#[derive(Tsify)]` vs. hand-authored
interfaces in the wasm `typescript_custom_section` overriding the fat `Token`; whether entry
points should be re-typed to accept the narrow interface (`tokensToUsfm(t: SerializableToken[])`).
