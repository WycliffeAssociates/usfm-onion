# Plan: unified walker architecture

Status: deferred. Hold on implementation until the in-flight
token-shape / CST-attribute refactor (the parallel agent's branch)
lands, since this plan sits one layer above the token shape and
benefits from a stable foundation. The plan itself can be agreed on
now; the code change is the larger follow-up.

## Why this exists

We have five lossy or semi-lossy consumers of the flat token stream
that each re-implement the same structural-interpretation logic:

- `export_tree.rs` (USJ + USX) — builds an intermediate tree.
- `html.rs` — walks tokens with its own `OpenElement` stack.
- `vref.rs` — walks tokens with a small state struct.
- `lint_impl.rs` — walks tokens with its own context tracking,
  recovery deliberately disabled.
- `diff.rs` — walks parse output for sid-block comparison.

Plus `cst::parse_cst`, the lossless tree builder, walking tokens
independently again.

These walkers agree on a lot of rules — "what closes an unclosed
note," "what counts as a block-scope boundary," "is this marker
valid in the current context" — but each expresses the rule in its
own code. Two concrete drift bugs motivate this:

1. **Unclosed-note recovery in html (commit `8ed7439`).** HTML had
   drifted from vref and the export tree, and unclosed footnotes
   silently swallowed every subsequent chapter. Fixed across surfaces
   by consolidating the boundary predicate
   (`StructuralScopeKind::closes_unclosed_note`).
2. **Verse drift in `export_tree`, latent in master right now.**
   `export_tree.rs:226` does not call `force_close_notes()` on
   `Verse`, but `StructuralScopeKind::closes_unclosed_note()` at
   `marker_defs.rs:236` lists `Verse` as note-closing. html and vref
   close on Verse; export_tree does not. The codebase already
   self-identifies the hazard: `marker_defs.rs:231-234` carries a
   comment warning to "audit `export_tree::dispatch` as well" if the
   closure set changes. Migrating html + export_tree to the shared
   walker fixes this bug as a side effect.

Beyond these, the duplication itself is large. An audit found ~1,500
lines of scope-stack and recovery logic across the consumers — five
independent open-scope stacks, each with its own frame type. The
next time the rule set grows, the same drift risk applies.

This plan replaces the *structural* walkers with a single
visitor-based walker over the flat token stream. CST, USJ, USX,
HTML, vref, and lint become *visitors*. Recovery is a flag on the
walker, not a property of each consumer.

**Format and diff are out of scope for v1.** Format is a token
rewriter, not a structural consumer — it already operates on
`&mut [FormatToken]` and naturally lives outside an
event-emission walker. Diff is an algorithm over already-tokenized
output, not a walker. Their **public trait surfaces**
(`FormattableToken`, `DiffableToken`) remain unchanged stability
contracts; their internals are untouched by this plan.

## Consumer contract (load-bearing)

Three operations are **first-class token-in** with stable
trait-based public APIs:

- `lint_tokens<T: LintableToken>(...)` — migrating to the walker
  internally; trait surface preserved.
- `format_tokens<T: FormattableToken>(...)` and friends in `format/`
  — **not migrating**; trait surface preserved.
- `diff_*<T: DiffableToken>(...)` in `diff/` — **not migrating**;
  trait surface preserved.

These are repeated ops in downstream consumers (the Lexical-based
WYSIWYG editor flattens its own JSON to tokens and calls
`lint_tokens` directly — no USFM round-trip). The traits already
exist and work; this plan must preserve them as-is. The token shape
may evolve (e.g., the in-flight attribute-fragment work), but the
trait contract is a public stability surface that requires
deliberate change.

Only `LintableToken` flows through the walker. `FormattableToken`
requires mutation (`set_kind`, `set_text`, …) which doesn't fit a
read-only event-emission model, and forcing `&mut [T]` walk modes
into the walker would double its API for one beneficiary.
`DiffableToken` operates on already-parsed tokens at the algorithm
layer; it doesn't need structural events. Both keep their existing
implementations.

The other five operations are **source-in by design**:

- `usj`, `usx`, `html`, `vref`, `cst`

Consumers detour through USFM string for these. The library is
fast enough that this overhead doesn't matter, and keeping their
public API single-shape (source-in only) keeps the surface focused.
Internally they will still consume the same walker — they just
don't expose a token-trait-generic entry point.

This means: **the walker's internal trait surface is independent of
which public APIs are trait-generic.** The walker can be driven by
either a concrete `Token<'a>` slice (the source-in operations,
post-parse) or by a user-supplied trait-bound slice (the three
first-class operations). The walker itself, and all visitors, are
generic over a token trait; the public-API ergonomics decide which
operations expose that generic to callers.

```rust
// Internal trait the walker is generic over. LintableToken is the
// only existing public trait that flows through; the walker's needs
// are a superset of LintableToken's read-only surface.
pub trait WalkableToken {
    fn kind(&self) -> TokenKind;
    fn marker(&self) -> Option<&str>;
    fn text(&self) -> &str;
    fn structural(&self) -> Option<StructuralMarkerInfo>;
    // ...maximal surface the walker needs; resolved during impl
}

impl WalkableToken for Token<'_> { /* native path */ }
```

The relationship between `WalkableToken` and `LintableToken` is
resolved during implementation — `LintableToken` likely becomes a
subtrait or alias. `FormattableToken` and `DiffableToken` are
unrelated to the walker and stay independent.

## Pipeline

```
source
  → lex                              (bytes → lexemes, lossless)
  → parse                            (lexemes → flat tokens, lossless)
  → walker (THE state machine)
       │
       │  emits events with full structural context attached.
       │  recovery is a config flag, not a per-consumer concern.
       │
       ├── apply_recovery = false   →   CST, lint, diff
       └── apply_recovery = true    →   USJ, USX, HTML, vref
```

The flat token stream remains the lossless ground truth. The walker
is what every consumer that wants structural meaning above the
token level shares. There is no longer a separate "export tree" —
the export tree was a partial version of this walker that only
USJ/USX used.

## Event shape

The walker emits events as it traverses the flat token stream.
Every event carries a reference to a `WalkContext` describing the
state of the world *at that moment*: structural scope stack,
current chapter SID, current verse SID, current in-note depth, etc.
Visitors read from `WalkContext` rather than re-tracking state.

### API constraint: `WalkContext` references are event-local

Visitors **cannot store** `&WalkContext` or any borrowed slice from
it across method calls. The scope stack is mutated between events;
references handed to a visitor are valid only for the duration of
that callback. Cross-event state (a visitor's own accumulators, sid
maps, lint-rule running state) must be **owned** by the visitor.

This falls out of the borrow checker — the walker's scope stack is
a `&mut Vec<ScopeFrame<'a>>` internally, and visitors receive `&'_`
snapshots per event. It is not a limitation we can paper over; it
shapes the visitor API.

### Unified scope stack

**Revised in step 2.** `WalkContext` tracks **all** open scopes on
a single stack: paragraphs, notes, sidebars, chapters, verses,
table rows/cells, headers, periphs, character markers (`\nd`,
`\bk`, …), and milestone-starts (`\zaln-s` …).

The original plan's "structural scopes only" framing was wrong:
current CST pushes character markers as parent frames in its
tree, and so does any walker-driven visitor that needs to mirror
that shape (CST, eventually USJ/USX). Helpers like
`current_paragraph_category()` and `current_scope_kind()` filter
the stack by frame kind, so consumers that only care about a
subset are unaffected. Visitor-local inline stacks are no longer
required.

```rust
pub struct WalkContext<'a> {
    pub scope_stack: &'a [ScopeFrame<'a>],
    pub current_chapter: Option<&'a Sid<'a>>,
    pub current_verse:   Option<&'a Sid<'a>>,
    pub in_note:         bool,
    pub note_depth:      usize,
    // ... whatever else proves load-bearing during implementation
}

pub struct ScopeFrame<'a> {
    pub kind: StructuralScopeKind,
    pub marker: &'a str,
    pub source_token_id: TokenId,
    pub attributes: &'a [AttributeItem<'a>],
    // closing_behavior, note_subkind, note_family, etc. as needed
}
```

Events:

| Event              | Fires on                                  | Notes                                                                          |
| ------------------ | ----------------------------------------- | ------------------------------------------------------------------------------ |
| `on_enter_scope`   | A marker opens any scope                  | Fires for chapter, verse, paragraph, note, sidebar, character, milestone, etc. |
| `on_leave_scope`   | A scope closes for any reason             | `reason: Explicit \| RecoveryClosure \| ImplicitByOpen \| EndOfInput`          |
| `on_end_marker`    | An EndMarker token (`\f*`, `\nd*`, …)     | Fires *after* `on_leave_scope(Explicit)`. Visitors append the closer as a leaf.|
| `on_milestone`     | A milestone token without scope semantics | Fallback dispatch — most milestones fire `on_enter_scope` instead.             |
| `on_milestone_end` | A `\*` MilestoneEnd token                 | Fires *after* `on_leave_scope(Explicit)` for the matched milestone.            |
| `on_text`          | A `TokenData::Text` token                 | Caller can choose to skip when `ctx.in_note()`                                 |
| `on_chapter`       | A chapter `Number` token resolved         | Walker tracks `current_chapter` sid in `WalkContext`                           |
| `on_verse`         | A verse `Number` token resolved           | Walker tracks `current_verse` sid in `WalkContext`                             |
| `on_book_code`     | A `\id` resolves a book code              |                                                                                |
| `on_opt_break`     | `TokenData::OptBreak`                     | Most visitors ignore                                                           |
| `on_newline`       | `TokenData::Newline`                      | Formatter cares; most others ignore                                            |
| `on_other`         | Anything not above                        | Unknown markers, numbers outside chapter/verse, stray end markers, etc.        |

Every event receives `token_index: usize` so visitors that build
index-based structures (CST, USJ) can wire the triggering token
into their output. `on_enter_scope` additionally receives the
`ScopeFrame` and the opening token; `on_leave_scope` receives the
frame and a `LeaveReason`.

Granularity: one method per event with a no-op default
implementation. Visitors override only what they care about. The
vref visitor will override four methods and ignore the rest; the
USJ visitor will override most of them.

## Recovery model

**Revised in step 2** (commit landing alongside CST migration): the
`apply_recovery` flag was removed. The walker always applies
precedence rules, including the implicit closure of unclosed notes
at block-scope boundaries. `LeaveReason::RecoveryClosure` annotates
each note-implicit-close so consumers that care (lint at step 6)
can subscribe to it.

The earlier framing — "CST and lint pass `false` to see
source-as-written" — turned out to be incoherent. Current CST's
tree shape already depended on precedence-based pops (Block closes
Note via `pop_for_open_scope`'s third pass). Disabling those pops
would produce a tree where unclosed notes contain every subsequent
paragraph as a child, which neither matches current CST output nor
is useful to any downstream consumer.

The model that landed:

- The walker emits `LeaveScope` for every popped frame, with a
  `LeaveReason`:
  - `Explicit` — closed by `\f*`, `\nd*`, or milestone-end token.
  - `RecoveryClosure` — a Note frame that was popped because an
    opening marker satisfying `closes_unclosed_note` arrived.
  - `ImplicitByOpen` — any non-Note frame popped because a higher-
    precedence scope opened (e.g. `\p` popping previous `\p`).
  - `EndOfInput` — drained at EOF.
- All consumers behave correctly by observing events; no flag is
  needed.
- Lint (step 6) detects "unclosed note in source" via
  `RecoveryClosure` events.
- The latent Verse drift bug in `export_tree.rs:226` is fixed when
  HTML + export_tree migrate (step 3) because the walker's single
  precedence implementation now drives all consumers.

## What stays lossless

- `lex` — lexemes preserve every byte.
- `parse` — flat tokens preserve every byte; `tokens_to_usfm` is a
  parity test that round-trips.
- **The walker with `apply_recovery: false`** — emits events that
  correspond exactly to source structure; no events are synthesized
  or suppressed.
- **CST**, built from the no-recovery walker, remains lossless by
  construction. The existing `cst_roundtrips_all_usfm_sources` test
  is the migration oracle.
- **lint** consumes the same no-recovery walker, so its view of
  structure matches CST's.
- **diff** is unchanged by this plan; it operates above the walker
  on already-tokenized output.

## What becomes lossy (and is fine)

- The walker with `apply_recovery: true` — emits synthesized
  `LeaveScope` events that don't correspond to source closures.
- **USJ, USX, HTML, vref** consume this recovered stream. They
  cannot round-trip back to byte-exact source, which matches the
  spec (USJ/USX have insufficient trivia for round-trip; HTML and
  vref are explicitly summary outputs).

## Per-consumer migration

| Consumer    | Today                                           | Under this plan                                               | Public API shape                                                |
| ----------- | ----------------------------------------------- | ------------------------------------------------------------- | --------------------------------------------------------------- |
| CST         | `cst::parse_cst(source)` with `recover_stack()` | Visitor that builds the tree from walker events               | source-in (unchanged)                                           |
| HTML        | Own `OpenElement` stack + own inline tracking   | Visitor with HTML's buffering/extraction + local inline stack | source-in; `tokens_to_html` stays as internal/native helper     |
| export_tree | Own `BuilderState`, MarkerKind dispatch         | Retired; USJ/USX become direct visitors                       | n/a (internal)                                                  |
| USJ         | `export_tree` → typed serializer                | Visitor that builds the typed tree                            | source-in (unchanged)                                           |
| USX         | `export_tree` → XML serializer                  | Visitor that emits XML elements                               | source-in (unchanged)                                           |
| vref        | Own `VrefState`                                 | Visitor; overrides ~5 methods                                 | source-in; `tokens_to_vref_map` stays as internal/native helper |
| lint        | Own structural + inline stacks                  | Visitor with `apply_recovery: false`, local inline stack      | **token-in via `LintableToken` (preserved)**                    |
| format      | Token-stream rewriter                           | **Not migrated.** Stays as-is.                                | **token-in via `FormattableToken` (preserved)**                 |
| diff        | Sid-block comparison over parse output          | **Not migrated.** Stays as-is.                                | **token-in via `DiffableToken` (preserved)**                    |

`export_tree.rs` retires entirely once USJ and USX are migrated.

## Migration order

Sequenced to minimize risk: cheapest losslessness check first,
then cheapest bug-fix-delivers-value second, then the rest.

1. ✅ **Walker + `WalkContext` skeleton** (commit `<step 1>`) —
   types, traits, two unit tests over hand-crafted token slices.
2. ✅ **CST** (commit `<step 2>`) — first consumer migrated. The
   walker absorbed the full `pop_for_open_scope` precedence
   algorithm from `cst::recover_stack`; `cst::build_cst_roots` is
   now a visitor that mirrors the historical tree shape exactly.
   `src/structure.rs` deleted (orphaned). The
   `cst_roundtrips_all_usfm_sources` test now covers the en_ult
   alignment-heavy corpus as well (depth-400 trees), validated by
   making `finalize_roots` and `flatten_nodes` iterative — the
   previous recursive helpers would have overflowed on those
   inputs.
3. ✅ **HTML** (commit `<step 3>`) — migrated as a walker visitor.
   `OpenElement` + buffer-stack pattern preserved; all scope
   tracking (Block precedence, sidebar pops, inline closes) is now
   delegated to the walker via `on_enter_scope` / `on_leave_scope`.
   Notes are deferred: caller is captured from the first body text
   token (mirroring `parse_note_tokens`' first-text-is-caller
   rule); `<aside>` extraction or inline-span emission happens on
   `on_leave_scope(Note)`. `\esbe` is modelled as a phantom frame
   (occupies a walker scope slot but emits no HTML element).
   Original step 3 also included `export_tree`; deferred to step 4
   because USJ/USX migration retires `export_tree.rs` entirely and
   making it a walker visitor in between would be wasted work.
4. ✅ **USJ + USX** (commit `<step 4>`) — `export_tree::build_export_document`
   is now a walker visitor (`ExportTreeBuilder`). The previous
   ad-hoc state machine — `BuilderState`, `handle_open`,
   `force_close_notes`, `close_paragraph`, `close_open_meta`,
   `close_sidebar`, `close_table_cell_in_row`, `close_table_row`,
   `close_character_in_note`, `close_inline_above_paragraph`,
   `in_note_context`, `unknown_marker_starts_new_block`, and the
   imperative `MarkerKind`-keyed dispatch — is deleted. The
   `ExportNode` / `ExportContainerNode` types are unchanged, so
   `UsjExporter` and `UsxExporter` (the actual format-specific
   serializers) are untouched. The latent Verse drift bug
   (`export_tree.rs:226` vs `marker_defs.rs:236`) is fixed as a
   side effect — Verse pops via the walker's unified precedence
   rules now. The plan's original framing said "export_tree.rs
   deletes" — in practice it shrank to the visitor + the type
   definitions consumed by UsjExporter / UsxExporter. The
   duplication is gone; the file remains as a useful intermediate
   AST.

   Walker change required at this step: `handle_milestone` now
   overrides `scope_kind` to `Milestone` for every `TokenData::Milestone`
   token, regardless of spec data. Alignment milestones like
   `\zaln-s` / `\zaln-e` are not in the marker spec table, so the
   prior path classified them as `Unknown` scope, which prevented
   `handle_milestone_end` from pairing them with their `\*`
   closer — they accumulated on the walker's stack indefinitely
   in alignment-heavy corpora (acts_1_11.aligned, en_ult). The
   `TokenData::Milestone` variant is syntactic truth; the walker
   now honors it.
5. **vref** — trivial port once the API is settled by the harder
   consumers above.
6. **lint** — last. Largest rule surface and most demanding state
   (scope stack from the walker + per-rule running state). By now
   the visitor API is mature against five consumers; lint's
   idiosyncrasies fit on top without forcing churn upstream.

Each step keeps tests passing on master. No big-bang rewrite.

**Format and diff are not in this migration.** Their public traits
remain stable; their implementations are untouched.

## Intersection with other deferred plans

- **`plan-whitespace-lint-rules.md`** — those six rules become
  methods on the lint visitor. The `MARKER_WHITESPACE` table stays
  the data source. Same auto-fix shape, same `TokenFix` emission.
- **`plan-marker-data-curation.md`** — the cleaned-up `lookup_spec_marker`
  data feeds `WalkContext` (specifically, which markers count as
  valid in which `SpecContext`). The lint plan's preconditions
  apply here too: marker-data cleanup must land before either lint
  rules or walker correctness can depend on it.
- **`plan-cli.md`** — unaffected by this plan; the CLI consumes
  library functions that happen to be implemented via the walker.

## Latent bugs this surfaces or fixes

- **`export_tree` Verse drift** (live in master): `export_tree.rs:226`
  does not call `force_close_notes()` on `Verse`, but
  `StructuralScopeKind::closes_unclosed_note()` at
  `marker_defs.rs:236` lists `Verse` as note-closing. html and vref
  close on Verse; export_tree does not. The codebase comment at
  `marker_defs.rs:231-234` already flags this hazard. Fixed when
  HTML + export_tree migrate (step 3).
- **General drift risk**: any future addition to the
  `closes_unclosed_note` set, the structural-scope kind list, or
  the recovery-trigger rules currently requires editing five
  walkers. The walker plan reduces this to one.

## Explicitly out of scope

- Streaming output (lazy iterators that produce bytes incrementally
  before the input is fully walked). The walker is push-based on
  the consumer side; if streaming output becomes a need, it lives
  inside specific visitors, not in the walker contract.
- Replacing the flat token stream. The flat stream is the lossless
  foundation; this plan sits above it.
- Replacing parse. Parse stays; the walker is a separate layer.
- Re-deriving `closes_unclosed_note` or other predicates. They
  already live in `marker_defs`; the walker calls them.
- Performance work. The walker is a structural simplification, not
  a perf project. The HTML 1.94× scaling outlier is its own
  follow-up — likely unaffected by this refactor, possibly easier
  to diagnose afterwards because the renderer becomes smaller.

## WASM surface

No new token-in WASM bindings. The existing surface is preserved:

- `wasm_lint_tokens` (token-in, via `LintableToken`).
- Format and diff token-in bindings (via `FormattableToken` /
  `DiffableToken`).
- USJ / USX / HTML / CST / vref remain reachable from JS only via
  the source-in route (`ParsedUsfm.to_usj` etc.). The walker
  refactor changes their *implementation*, not their JS surface.

`wasm_tokens_to_html` exists today but is not first-class — it
stays available for now; no commitment to extend the same shape to
USJ/USX/CST/vref.

The token-trait stability contract therefore applies only to the
three first-class trait surfaces. The in-flight CST/attribute
refactor must keep those compatible (or coordinate a deliberate
break); the walker plan itself is consumer-API-neutral for the
other five operations.

## Open questions to resolve during implementation

- Does `WalkContext` expose the scope stack by reference, or expose
  helper methods (`in_note()`, `in_table()`, `current_paragraph()`)?
  Probably both — methods for common queries, stack reference for
  rules that need the full picture. Remember: any reference is
  event-local; visitors that need persistent state own it.
- How are attribute fragments (the in-flight default-attribute
  shorthand work) surfaced on `on_enter_scope`? Resolve after that
  refactor lands; this plan does not pre-commit a shape.
- Exact shape of `WalkableToken` vs `LintableToken` —
  subtrait, alias, or shared signatures. Pick whichever minimizes
  churn for existing `lint_tokens` callers.
- Should there be a `Cow`-like mechanism for visitors to mutate
  the event stream (e.g., autofix-applying lint)? Probably not in
  v1 — autofixes emit `TokenFix` objects via the existing
  mechanism, not via event-stream rewriting.
