# Plan: marker data additions for USFM 3.1.1 / 3.1.2 / 3.2

Status: ready. Existing rows in `src/marker_defs_data.rs` are assumed
correct. This plan is purely **additive** — bring the table up to the
3.2 release-notes baseline.

Source of truth: <https://docs.usfm.bible/usfm/3.2/release-notes.html>
sections 3.2, 3.1.2, 3.1.1.

Related prior art in `Bridgeconn/usfm-grammar`:
- #394 `\ipc` — already present in our table (`src/marker_defs_data.rs:784`).
- #393 `\list-s` / `\list-e` — **missing**.
- #396 `\ta` — already present (`:1720`).
- #397 `@lang` attribute on `\tl` / `\wl` — attribute, not a row.
- #398 `\wl` — already present (`:1992`).
- #400 inline markup in titles/headings — context audit, not a row.

## What's actually missing

A grep over `marker_defs_data.rs` confirms:

1. **`\list` milestone (3.2).** No `list` row exists. Spec page:
   <https://docs.usfm.bible/usfm/3.2/ms/list.html>. Should be a single
   `Milestone` row keyed `"list"` (matching how the codebase handles
   `qt` → `qt-s`/`qt-e` normalization).

2. **`\table` milestone (3.2).** Same shape as `list` — start/end
   milestone form for tables. Single `Milestone` row keyed `"table"`.

3. **`p1` / `p2` re-categorization (3.1.1).** The release notes move
   these from `OtherPara` to `PeriphPara`. Check whether our rows (if
   any) reflect that; if absent, decide whether we register them at all
   (they only appear in peripherals).

4. **`@lang` attribute on `\tl` and `\wl` (3.1.2).** This is an
   attribute schema concern, not a `MarkerSpec` row. Lives wherever the
   attribute allow-lists per marker are encoded (post-3.1 attribute
   commit `f1cfda1` lifted attributes onto the owning marker token —
   confirm `tl` and `wl` accept `lang`).

## Declarative groundwork (do this here, not in the walker)

The walker plan (`plan-walker-architecture.md`) reads structural
metadata off `WalkContext` rather than re-deriving it per visitor.
That only works if the metadata is declarative on the marker row.
Several 3.2 release-note items map naturally to per-row data and
should be added **here**, ahead of the walker migration, rather than
encoded ad-hoc in each consumer later.

### 1. Paragraph category on every `Paragraph`-kind row

Per <https://docs.usfm.bible/usfm/3.2/para/index.html>, every
paragraph marker belongs to exactly one category:

| Category           | Markers (representative)                                   |
| ------------------ | ---------------------------------------------------------- |
| `Identification`   | `ide`, `sts`, `rem`, `h`, `toc#`, `toca#`                  |
| `Introduction`     | `imt#`, `is#`, `ip`, `ipi`, `im`, `imi`, `ipq`, `imq`, `ipr`, `ipc`, `iq#`, `ili#`, `ib`, `iot`, `io#`, `iex`, `imte`, `ie` |
| `Title`            | `mt#`, `mte#`, `cl`, `cd`                                  |
| `Section`          | `ms#`, `mr`, `s#`, `sr`, `r`, `d`, `sp`, `sd#`             |
| `Body`             | `p`, `m`, `po`, `cls`, `pr`, `pc`, `pm`, `pmo`, `pmc`, `pmr`, `pi#`, `mi#`, `lit`, `nb`, `b`, `ph` (deprecated) |
| `Poetry`           | `q#`, `qr`, `qc`, `qa`, `qm#`, `qd`                        |
| `List`             | `lh`, `li#`, `lf`, `lim#`                                  |
| `Table`            | `tr`                                                       |
| `Peripheral`       | `p1`, `p2` (3.1.1 re-categorization), plus other peripheral-only paragraphs |
| `Other`            | catch-all for anything not above                           |

Action: introduce `ParagraphCategory` enum and a
`paragraph_category: Option<ParagraphCategory>` field on
`MarkerSpec`. Populate for every existing `Paragraph`-kind row. The
walker's `WalkContext` then exposes
`current_paragraph_category() -> Option<ParagraphCategory>`, which
makes the 3.2 `\v` context refinement a one-line declarative check
instead of bespoke per-consumer code.

### 2. New milestone rows (3.2)

- `\list` — `Milestone`, spec: <https://docs.usfm.bible/usfm/3.2/ms/list.html>
- `\table` — `Milestone`, spec: <https://docs.usfm.bible/usfm/3.2/ms/table.html>

### 3. `@lang` attribute on `\tl` and `\wl` (3.1.2)

Wire through whatever attribute allow-list mechanism `f1cfda1`
introduced. Audit: do any other character markers gain a `@lang`
attribute in 3.1.2? (Spec only lists `tl` and `wl`.)

### 4. `\v` not allowed in `Section` / `Other` paragraphs (3.2)

Once paragraph category is on rows, this becomes a declarative lint
rule keyed off `WalkContext.current_paragraph_category()`. No row
edit needed for `v` itself — just the rule, but it's only
expressible once category lands.

### 5. `\lit` allowed in book introductions (3.1.1)

Audit the `lit` row's `contexts` and add introduction context if
missing. Cheap, declarative, do it now.

## What stays out of scope

These 3.2 features need work outside the marker data table:

- **Anchors and inline referencing** (3.2). New syntactic construct.
  Parser/CST work; separate plan.
- **`markers.ext` wildcard `\*`** (3.2). Extension-loading mechanism;
  not built-in rows.
- **`standalone` category in `markers.ext`** (3.2). Same.
- **Generalized node initial attributes on paragraphs** (3.2).
  Attribute-lifting pass (commit `f1cfda1`), not the marker catalog.
- **PR #400 (titles/headings accept inline markup)**. Context-system
  refinement at the CST/lint layer; orthogonal to row data.

## Implementation steps

Ordered so the declarative groundwork lands before the walker plan
starts.

1. **Add `ParagraphCategory` enum and field.** Define the enum
   (variants from the table above). Add
   `paragraph_category: Option<ParagraphCategory>` to `MarkerSpec`.
   None for non-paragraph kinds; required for `Paragraph` kind.

2. **Populate paragraph category for every existing `Paragraph` row.**
   ~80 rows. Use the spec's `/usfm/3.2/para/index.html` grouping as
   the authority. `MarkerDefKind::Paragraph` rows with no category
   should fail a debug-assert / test.

3. **Add `\list` milestone row** to `MARKER_SPECS`, keyed `"list"`,
   kind `Milestone`. Source: `repos_to_compare/tcdocs-main/markers/ms/list.adoc`
   if present, else point at the spec URL.

4. **Add `\table` milestone row**, same shape, source
   `…/markers/ms/table.adoc` if present.

5. **`@lang` attribute on `tl` / `wl`.** Wire into the attribute
   allow-list mechanism from `f1cfda1`. If the mechanism is also
   declarative (per-row attribute list), update the rows; if it's
   coded centrally, update there. Match whatever shape `f1cfda1`
   chose.

6. **`\lit` context audit** for introduction allowance (3.1.1).

7. **Decide `p1` / `p2`.** Grep confirms no rows today. Recommend
   adding as `Paragraph` kind with `Peripheral` category, since the
   walker's category lookup will fall over if they show up in
   peripherals and aren't registered. Cheap to add now while
   populating the category field anyway.

8. **Run existing tests.** `cargo test -p usfm_onion`. The
   `marker_defs::tests` suite is the regression net.

## Estimated scope

- New enum + field: ~20 lines.
- Populating `paragraph_category` across ~80 paragraph rows: ~80
  lines of additions, mechanical.
- 2 new milestone rows (`list`, `table`): ~20 lines.
- Attribute wiring for `@lang`: depends on `f1cfda1` shape; likely
  under 10 lines.
- Possibly 2 new paragraph rows (`p1`, `p2`): ~15 lines.
- Total: ~150 lines, mostly mechanical data entry.

## Sequencing

Do this **before** both `plan-walker-architecture.md` and
`plan-whitespace-lint-rules.md`.

- The walker plan's `WalkContext` exposes
  `current_paragraph_category()`. That helper is only meaningful if
  every paragraph row carries a category, which this plan delivers.
  Landing the walker first and back-filling category later means
  every visitor that wants the field has to handle `None` as "I
  don't know yet" — exactly the kind of drift the walker plan is
  trying to delete.
- Whitespace lint rules walk `MARKER_WHITESPACE`; the new milestone
  rows (`list`, `table`) need to resolve via `lookup_spec_marker`
  before those rules can rely on them.

In other words: this is the declarative-data foundation. Walker and
lint sit on top of it.
