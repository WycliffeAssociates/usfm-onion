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
own code. The recent unclosed-note bug (commit `8ed7439`) is the
canonical case: HTML had drifted from vref and the export tree, and
unclosed footnotes silently swallowed every subsequent chapter. We
fixed it across surfaces and consolidated the boundary predicate
(`StructuralScopeKind::closes_unclosed_note`), but the underlying
duplication is still there. The next time the rule set grows, the
same drift risk applies.

This plan replaces all of those walkers with a single visitor-based
walker over the flat token stream. CST, USJ, USX, HTML, vref, lint,
and (eventually) diff all become *visitors*. Recovery is a flag on
the walker, not a property of each consumer.

## Consumer contract (load-bearing)

Three operations are **first-class token-in** with stable
trait-based public APIs:

- `lint_tokens<T: LintableToken>(...)`
- `format_tokens<T: FormattableToken>(...)` and friends in `format/`
- `diff_*<T: DiffableToken>(...)` in `diff/`

These are repeated ops in downstream consumers (the Lexical-based
WYSIWYG editor flattens its own JSON to tokens and calls
`lint_tokens` directly — no USFM round-trip). The traits already
exist and work; this plan must preserve them as-is. The token shape
may evolve (e.g., the in-flight attribute-fragment work), but the
trait contract is a public stability surface that requires
deliberate change.

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
// Internal trait the walker is generic over. The existing
// LintableToken / FormattableToken / DiffableToken each pull from
// this surface (or extend it with op-specific methods like format's
// mutable-access requirements).
pub trait WalkableToken {
    fn kind(&self) -> TokenKind;
    fn marker(&self) -> Option<&str>;
    fn text(&self) -> &str;
    // ...maximal surface the walker needs; resolved during impl
}

impl WalkableToken for Token<'_> { /* native path */ }
```

The relationship between `WalkableToken` and the three existing
traits is resolved during implementation — they may become
subtraits, aliases, or stay independent with shared method
signatures. Whatever shape minimizes churn for the existing public
callers wins.

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
state of the world *at that moment*: scope stack, current chapter
SID, current verse SID, current in-note depth, etc. Visitors read
from `WalkContext` rather than re-tracking state.

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

| Event              | Fires on                                      | Notes                                                    |
|--------------------|-----------------------------------------------|----------------------------------------------------------|
| `on_enter_scope`   | A marker opens a new structural scope         | Fires for chapter, verse, paragraph, note, sidebar, etc. |
| `on_leave_scope`   | A scope closes (explicit, recovery, EOF)      | `reason: Explicit \| RecoveryClosure \| EndOfInput`      |
| `on_text`          | A `TokenData::Text` token                     | Caller can choose to skip when `ctx.in_note`             |
| `on_chapter`       | A chapter `Number` token resolved             | Includes parsed chapter number + sid                     |
| `on_verse`         | A verse `Number` token resolved               | Includes parsed verse lexeme + sid                       |
| `on_milestone`     | A self-closing or paired milestone            |                                                          |
| `on_book_code`     | A `\id` resolves a book code                  |                                                          |
| `on_opt_break`     | `TokenData::OptBreak`                         | Most visitors ignore                                     |
| `on_newline`       | `TokenData::Newline`                          | Formatter cares; most others ignore                      |
| `on_unknown`       | Unknown marker the walker can't classify      | Lint cares; others fall back to "treat as block"         |

Granularity: one method per event with a no-op default
implementation. Visitors override only what they care about. The
vref visitor will override four methods and ignore the rest; the
USJ visitor will override most of them.

## Recovery model

Recovery — the implicit closure of an unclosed note when a
block-scope marker opens — is a property of the walker, not the
visitor. The walker is configured with a flag:

```rust
walk(&tokens, WalkOptions { apply_recovery: true, .. }, &mut visitor);
```

When `apply_recovery == true` and the walker sees a marker that
satisfies `closes_unclosed_note` while a note scope is open, it
emits `on_leave_scope` for the note with
`reason: RecoveryClosure` *before* emitting `on_enter_scope` for
the new block-scope marker. From the visitor's perspective, the
note simply closed.

When `apply_recovery == false`, no recovery closures fire; the note
stays open until either an explicit `\f*` or end-of-input. The
`on_leave_scope { reason: EndOfInput }` at EOF is what lint
consumes to report `UnclosedNote`.

This means:

- **CST, lint, diff** pass `false`. They see source-as-written.
- **USJ, USX, HTML, vref** pass `true`. They see the recovered
  structure that prevents one missing `\f*` from corrupting an
  entire chapter.
- The recovery rule itself lives in one place (the walker), driven
  by the existing `closes_unclosed_note` predicate.

## What stays lossless

- `lex` — lexemes preserve every byte.
- `parse` — flat tokens preserve every byte; `tokens_to_usfm` is a
  parity test that round-trips.
- **The walker with `apply_recovery: false`** — emits events that
  correspond exactly to source structure; no events are synthesized
  or suppressed.
- **CST**, built from the no-recovery walker, remains lossless by
  construction.
- **lint** and **diff** consume the same no-recovery walker, so
  their view of structure matches CST's.

## What becomes lossy (and is fine)

- The walker with `apply_recovery: true` — emits synthesized
  `LeaveScope` events that don't correspond to source closures.
- **USJ, USX, HTML, vref** consume this recovered stream. They
  cannot round-trip back to byte-exact source, which matches the
  spec (USJ/USX have insufficient trivia for round-trip; HTML and
  vref are explicitly summary outputs).

## Per-consumer migration

| Consumer | Today                                            | Under this plan                                    | Public API shape                              |
|----------|--------------------------------------------------|----------------------------------------------------|-----------------------------------------------|
| CST      | `cst::parse_cst(source)`                         | Visitor that builds the tree from walker events    | source-in (unchanged)                         |
| USJ      | `export_tree` → typed serializer                 | Visitor that builds the typed tree                 | source-in (unchanged)                         |
| USX      | `export_tree` → XML serializer                   | Visitor that emits XML elements                    | source-in (unchanged)                         |
| HTML     | Own `OpenElement` stack                          | Visitor with HTML's buffering/extraction           | source-in; `tokens_to_html` stays as internal/native helper |
| vref     | Own `VrefState`                                  | Visitor; overrides ~5 methods                      | source-in; `tokens_to_vref_map` stays as internal/native helper |
| lint     | Own context tracker                              | Visitor with `apply_recovery: false`               | **token-in via `LintableToken` (preserved)**  |
| format   | Token-stream rewriter                            | Visitor (or post-walker pass over events)          | **token-in via `FormattableToken` (preserved)** |
| diff     | Sid-block comparison over parse output           | Visitor or layer atop CST (decide during impl)     | **token-in via `DiffableToken` (preserved)**  |

`export_tree.rs` retires entirely once USJ and USX are migrated.

## Migration order

1. **Walker + `WalkContext` skeleton** — types, traits, no
   consumers yet. Validates the shape against the existing rules.
2. **vref** — smallest visitor; proves the pattern; existing tests
   keep behavior pinned.
3. **HTML** — biggest dedup win; currently the most
   bug-prone of the walkers.
4. **USJ + USX** — replace `export_tree.rs` together. They share
   enough that doing them in sequence is wasted work.
5. **CST** — re-express as a visitor. Existing CST tests
   (`cst_roundtrips_all_usfm_sources`, parity tests) prove
   losslessness preserved.
6. **lint** — last. Largest rule surface; benefits most from a
   stable visitor API on the others.
7. **diff** — evaluate after lint. May not become a visitor; could
   instead consume CST output and compare trees.

Each step keeps tests passing on master. No big-bang rewrite.

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
  rules that need the full picture.
- How are attribute fragments (the in-flight default-attribute
  shorthand work) surfaced on `on_enter_scope`? Resolve after that
  refactor lands; this plan does not pre-commit a shape.
- Does diff become a visitor or stay above CST? Decide after lint
  migration informs the answer.
- Should there be a `Cow`-like mechanism for visitors to mutate
  the event stream (e.g., autofix-applying lint)? Probably not in
  v1 — autofixes emit `TokenFix` objects via the existing
  mechanism, not via event-stream rewriting.
