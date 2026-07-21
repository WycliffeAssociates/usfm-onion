# Plan — parse/lex hot-path: alloc + one derived marker table + lexer-owned delimiter ws

Status: in progress on branch `chapter-parallelism`. Gated by the behavior
oracle (`tests/lint_oracle.rs`, `BLESS=1` to rebaseline) — the bar is
**byte-identical output across the real-world corpus**, not USFM conformance.
The oracle is the whole reason we can refactor the hot path aggressively here.

## Why

A rayon-free serial profile (`profile_ops parse/serial --book psalms`, the
`parse/serial` op added to `benches/common.rs`) shows where single-thread
ingest actually spends time — no parked-worker noise. After the Vec presize
(workstream 1, landed), the picture on Psalms (272 KB):

- **Lexer (logos DFA + driver): ~28%** — `lex` self + `logos::lex::state*`.
  Inherent scanning cost; no cheap lever (would mean replacing logos). Parked.
- **Token construction: `parse_lexemes_seeded` + `push_token` ~18%.**
- **Marker-catalog lookups: ~20%** — the top structural target. Same marker is
  normalized-and-probed ~3× per occurrence across lex + parse (see below).
- **Allocator/memmove: ~12%** — parser Vec now presized; remaining memmove is
  the *lexer's* unpresized lexeme Vec + per-iteration free (the free is a
  bench artifact — real single-call use pays it once).

The redundancy, traced to call sites:

| Phase | Call | Site |
|---|---|---|
| lex | `marker_metadata()` → `lookup_marker_metadata` | `lexer.rs:160` |
| lex | `marker_payload()` (cheap `match`) | `lexer.rs:319` |
| parse | `structural_marker_info(name, kind)` ×6 | `parse/mod.rs` |
| parse | `lookup_marker_whitespace(name)` via `park_ws`→`delimiter_absorption` | `parse/mod.rs:658` |

Each of `lookup_marker_metadata` / `structural_marker_info` /
`lookup_marker_whitespace` independently re-runs normalization (strip `+`,
milestone `-s/-e`, numbered variant, `table_cell_base`) + its own FxHashMap
probe. `park_ws`'s cost (`#5`) is mostly this lookup, not whitespace scanning —
so it is the *same* problem, not a SIMD candidate.

## Hard constraint: ONE source-of-truth table

The marker data (`MARKER_SPECS`, `MARKER_WHITESPACE` in `marker_defs_data.rs`)
is read crate-wide, not just by lex/parse:

- `lint_impl.rs` (10 sites, heaviest — incl. `marker_allows_context`),
  `parse` (8), public `markers.rs` catalog (8), `html` (6), `usj` (5),
  `walker` (4), `vref` (2), `token`/`lexer`/`export_tree` (1 each).

So the packed row **must be a derived projection** of the existing tables — NOT
a second hand-authored/codegen table that could drift. Marker data here is
hand-curated Rust (unlike ssc's Unicode-derived `gen_charclass_table`), so a
codegen xtask is the wrong tool. Derive in-process and guard drift with a test.

---

## Workstream 1 — right-size the token Vec (LANDED)

`parse_lexemes_seeded`: `Vec::new()` → `Vec::with_capacity(lexemes.len())`.
Parsing emits ≤1 token per lexeme (text runs merge; empty-doc flush adds ≤1),
so `lexemes.len()` is a tight, never-under bound. Byte-identical (capacity
never affects contents); parallel path presizes each chunk from its own slice.
Result: `push_token` 17%→8%, wall −16% on Psalms serial.

### 1b — lexer lexeme Vec (follow-on, candidate)

`lexer.rs:51` is still `Vec::new()`; it now dominates the remaining ~7.6%
memmove. No exact pre-count exists (first pass), so size heuristically from
source length. Measured **bytes-per-lexeme** across the corpus (throwaway
`measure_lex` example, since deleted):

| kind | B/lexeme |
|---|---|
| narrative prose (en_ulb/BSB Luke) | 14.6–16.7 (sparse end) |
| poetry (en_ulb Psalms) | 7.35 |
| `\w` dense (en_ult Hosea/1Pe) | 6.66–6.93 |
| `\zaln` aligned (en_ult GEN/PSA/LUK, 4–5 MB) | **6.58–6.90 (floor)** |

The density floor is **~6.5 B/lexeme**. Size at **`source.len() / 6`** (just
under the floor) → essentially never reallocs. Over-allocation is tight where
it matters (5 MB aligned GEN: ~11% over) and loose only where absolute counts
are tiny (small narrative prose: ~2.8× of a few-thousand-slot Vec, <1 MB
transient, freed after parse). Do after WS2 and re-measure.

- **Arena is the wrong tool**: the lexeme buffer is one contiguous growable Vec;
  the cost is the realloc-copy on growth, which happens under any allocator. A
  capacity estimate removes it; an allocator does not.
- **Buffer reuse/pooling** (thread-local Vec `.clear()`ed across chunks) is the
  legit arena-adjacent idea, but it targets alloc+free churn (~4% `vm_dealloc`,
  partly a bench artifact), not memmove, and is lifetime-fiddly
  (`ScanToken<'a>` borrows source). Documented fallback, not first move.
- Note: for `\w`/`\zaln`-dense books the *parser* Vc's `lexemes.len()` capacity
  (WS1) over-allocates ~2.9× (lexemes ≫ tokens: GEN 764 k lexemes → 263 k
  tokens) — transient and freed; acceptable, don't `shrink_to_fit` (a memmove).

## Workstream 2 — one derived, packed marker row (the main event)

Add a `MarkerRow` derived once from `MARKER_SPECS`/`MARKER_WHITESPACE`, keyed by
a dense `u16` marker id (≤238 base markers). Every hot lookup reads fields off
`ROWS[id]` instead of re-normalizing + re-probing.

### Layout — a single `u64` per row

Bit budget (fits u64 with ~14 spare; u128 only if we later fold in all four
whitespace positions + family + canonical-id for cold consumers — not needed
for the hot path):

| Field | Variants | Bits |
|---|---|---|
| `MarkerDefKind` | 13 | 4 |
| `ParagraphCategory` (+None) | 11 | 4 |
| `StructuralScopeKind` | 13 | 4 |
| `InlineContext` (+None) | 5 | 3 |
| `note_context` (`SpecContext` +None) | 21 | 5 |
| `ClosingBehavior` | 4 | 2 |
| `MarkerPayload` (None/Book/Num) | 3 | 2 |
| bools: `absorbs_trailing_ws`, `deprecated`, `is_section`, `is_list`, `is_table_cell`, `milestone_end` | — | 6 |
| **flags subtotal** | | **~30** |
| **context-validity bitmask** over 20 `SpecContext` | | **20** |

- `marker_allows_context(m, c)` → `ROWS[id].contexts & (1<<c as u32)` — replaces
  the `&'static [SpecContext]` slice scan (this is `lint_impl`'s hot predicate,
  not just parse's).
- `absorbs_trailing_ws` bit → `delimiter_absorption` becomes a field read,
  killing most of `park_ws`'s cost.
- Whole table ≈ 238 × 8 B ≈ **1.9 KB → permanently L1/L2-resident.** This is why
  "keep v/p/c cached" needs no special-casing for the *data*: it's all hot.

### string → id (where "front-load hot markers" actually pays)

The data is always cache-hot; the real cost is computing the id. Fast-path the
key:
- Single-byte markers (`v p c q m s b d f x w …`) need no normalization → direct
  dispatch (e.g. `[u16; 128]` by first byte, or pack ≤6 ASCII bytes into a u64
  and match). Covers the bulk of real text.
- Everything else falls to the existing normalize + interned-id map
  (`FxHashMap<&'static str, u16>` built once, or a perfect hash).
- Normalization itself (`table_cell_base` etc.) is unchanged work, but now done
  **once** per marker, not 3×. The redundancy is the win, not new normalization.

### Deriving + drift guard

- Build `ROWS` at first use (`LazyLock`) by packing each `MARKER_SPECS` /
  `MARKER_WHITESPACE` row — or a `const fn` packer into a `static` if we want
  zero runtime setup. Single edit point stays `marker_defs_data.rs`.
- Test: for every marker, assert the packed row's unpacked fields `==` the
  struct fields (`lookup_marker_metadata`, `structural_marker_info`,
  `lookup_marker_whitespace().required_after_open_name` → absorb bit,
  `contexts`). Mirrors ssc's "table pinned to predicate" discipline. This is
  what makes it safe to be a projection rather than a parallel table.

### Wiring

- Lexer computes the id once (it already visits every marker at `lexer.rs:160`),
  stash it on the `MarkerToken`/`Lexeme` (add a `u16` field).
- `structural_marker_info`, `lookup_marker_whitespace` (absorb bit),
  `marker_payload`, and `lint_impl`'s context checks read `ROWS[id]`.
- Rich/cold consumers (`markers.rs` public catalog, formatter's full 4-position
  whitespace, family/source) keep reading the existing structs — untouched.
- Keep the `MarkerMetadata`/`StructuralMarkerInfo`-by-value fields on `Token`
  for now (public shape). *Endgame (separate, wider):* store just the `u16` id on
  `Token` → shrinks `Token` (cheaper memmove, compounding WS1) and deletes the
  parse-time `structural_marker_info` call. Big blast radius (CST/export/lint
  read `token.data.structural`) — defer.

Verify: oracle at 1 thread + full threads byte-identical; re-profile serial to
confirm the ~20% lookup bucket + `park_ws` share actually drop.

## Workstream 3 — lexer-owned delimiter whitespace (optional, riskiest)

Goal: stop *moving whitespace around* in the parser. The lexer already does
contextual post-matching (`pending_payload_for` + `consume_contextual_payload`
consume the `\c`/`\v` number). Mirror that: right after emitting a marker
(`lexer.rs:120`), if `ROWS[id].absorbs_trailing_ws`, absorb the tag-end
delimiter into the marker on the first scan — so `park_ws` / `pending_ws` /
`delimiter_absorption` / `flush_pending_whitespace` largely disappear.

Depends on WS2's absorb bit. This is a **structural simplification**, not the
perf win (WS2 already makes the check a field read). Do it only if we want the
parser state gone.

Byte-identical subtleties to preserve (all oracle-checked):
- `park_ws` absorbs only the FIRST ws char (`absorbed_end = ws.start + 1`); the
  remainder stays separate. Lexer must replicate.
- `delimiter_absorption` also fires for `Number` (cv args, gated on
  `cv_number == JustEmitted`) and `BookCode` — handle in
  `consume_contextual_payload`, not just the marker arm.
- `flush_pending_whitespace` reattaches non-absorbed pending ws to the *prior*
  token's span; ensure moving absorption earlier doesn't change that.
- Interaction with `push_token`'s adjacent-Text merge.

Not in logos regex itself (it can't table-lookup mid-match); this is driver-loop
logic in `lex`, same as the existing payload consumption.

## Sequencing

1. WS1 token presize — **done**.
2. WS2 packed derived row + drift test + wire hot lookups → oracle → re-profile.
3. WS1b lexer Vec presize (re-measure after WS2).
4. WS3 lexer-owned delimiter ws (only if the state removal is worth the risk).
5. Endgame: `u16` id on `Token`, drop embedded structural/metadata (separate PR).

## Risks / notes

- Every step is a behavior-preserving representation change → the oracle is the
  gate. Rebaseline (`BLESS=1`) only if we *intend* a behavior change (we don't
  here).
- Don't add a codegen table (drift risk + wrong tool for hand-curated data);
  derive + guard instead.
- `profiling`-profile timings are for spotting hot spots, not quoting. Quote
  from `cargo bench --bench operations` (release).
