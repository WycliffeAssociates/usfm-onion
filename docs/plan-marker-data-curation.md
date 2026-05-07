# Plan: marker data curation pass

Status: deferred. The data was originally seeded by an automated pass over
`repos_to_compare/tcdocs-main` that proved imperfect; the file is now
hand-curated but several rows still carry artifacts from that initial
auto-gen. This plan describes a focused curation session.

## Why this exists

`src/marker_defs_data.rs` is the single source of truth for which markers
exist, what kind they are, and which contexts they're valid in. The
queries in `src/marker_defs.rs` and the public catalog in
`src/markers.rs` both delegate to this table. If a row is wrong, every
consumer (HTML, lint, format, USJ/USX conversion) inherits the bug.

Two known classes of issue surfaced while building the
`MARKER_WHITESPACE` table:

1. **Compound marker names.** At least one row encodes two markers in
   one string: `"ts-s / ts-e"`. The lookup paths can't normalize that
   into the start/end milestone forms. It should be one row keyed
   `"ts"` with `MarkerDefKind::Milestone`, or two rows.
2. **Missing milestone forms.** `qt` is registered as
   `MarkerDefKind::Character` only. The spec also defines `qt` as a
   milestone (`qt-s` / `qt-e`); `lookup_spec_marker("qt-s")` falls
   through to the milestone-suffix branch and works *because* of the
   normalization, but the spec lookup for the milestone form returns
   the Character kind, which is incorrect for milestone-shaped
   instances. Either `qt` needs a second `Milestone` row, or the
   `MarkerDefKind` model needs to allow a marker to be of multiple
   kinds depending on form.

The 213 rows in `MARKER_SPECS` likely have more issues like these.
A focused pass should validate each row against `whitespace.md` and the
USFM 3.1 spec at <https://docs.usfm.bible/usfm/latest/>, not against the
auto-generated `repos_to_compare/tcdocs-main` `.adoc` files (which is
where the imperfections came from).

## Methodology

The aim is *human review*, not regeneration. The auto-gen path was
tried and failed; this is the curation path.

1. **Print the catalog.** Run a small bin (or a one-off script) that
   serializes every `MARKER_SPECS` row as a flat table:

   | marker | kind | contexts | deprecated | source |

   Sort by kind, then alphabetically by marker. This is the diffable
   review surface.

2. **Triage by kind.** Walk each `MarkerDefKind` group in turn:

   - `Header` (small group: `id`, `usfm`, ...): each row should be
     trivially verifiable from the spec.
   - `Chapter` / `Verse`: should be tiny (`c`, `v`, `ca`, `cp`, `cl`,
     `cd`, `va`, `vp`, ...).
   - `Note`: `f`, `fe`, `ef`, `x`, `ex`. Verify each.
   - `Milestone`: cross-reference against the spec's milestone list.
     **The known issue is that `qt` and `ts` should appear here** —
     `qt` only appears as Character today, and `ts` appears as the
     compound `"ts-s / ts-e"`. Other milestones (`zaln`, `zw`, `qt-s`,
     `qt-e`) should also be checked.
   - `Sidebar`: `esb`, `esbe`. Trivial.
   - `Periph`, `Meta`, `TableRow`, `TableCell`: small groups, easy.
   - `Paragraph`, `Character`: the bulk of the table. Group by family
     (heading, list, poetry, attribution, etc.) and verify each
     family's contexts as a unit rather than row-by-row.

3. **Verify contexts.** For each row, the `contexts: &[SpecContext]`
   array determines where the marker is allowed. Mistakes here cause
   `MarkerNotValidInContext` lint false-positives or false-negatives.
   The spec's "Allowed in" sections per marker are authoritative.

4. **Resolve compound and missing rows.**
   - Split `"ts-s / ts-e"` into either one `"ts"` `Milestone` row
     (preferred — matches the `qt` / `qt-s` / `qt-e` normalization
     pattern) or two rows.
   - Add a `Milestone` row for `qt` (in addition to its Character
     row, if both are needed). Verify all consumers handle the
     polysemy — `lookup_marker_def` returns one `MarkerDef`, so if a
     marker is of two kinds we need to either pick one canonically
     (probably Milestone, since that's the form with start/end split
     consumers care about) or thread an "instance form" hint into
     the lookup.
   - Audit other potential dual-kind markers: `zaln`, custom `z`
     markers, anything documented as both inline and milestone.

5. **Add a one-shot validator.** Once curated, ship a `cargo test`-only
   validator that:
   - Asserts no row's `marker` field contains `/` or whitespace.
   - Asserts no `Milestone`-kind row's `marker` ends in `-s` or `-e`
     (those should be derived, not stored).
   - Asserts each row's `source:` path exists under
     `repos_to_compare/tcdocs-main/` *if* that directory is present
     (already done in `marker_defs::tests`).
   - Asserts every marker in `MARKER_WHITESPACE` resolves via
     `lookup_spec_marker` to a row whose canonical name matches.

   These guards prevent the next regression without forcing the data
   to be auto-regenerated.

## What this is NOT

- Not a regeneration pass. The plan was approved as human-curated;
  re-running auto-gen against tcdocs reintroduces the original
  imperfections.
- Not a USFM 3.1.2 release-notes pass. Adding new markers from
  `https://docs.usfm.bible/usfm/latest/release-notes.html` is a
  separate, additive task. Curation comes first because adding new
  rows on top of a flawed schema multiplies the cleanup later.
- Not a context-system redesign. The `SpecContext` enum is what it is;
  this pass only adjusts which markers get which contexts, not the
  set of contexts.

## Estimated scope

- 213 rows × ~30 seconds of review = ~2 hours of focused human time.
- Code surgery: minor — splitting/adding rows, fixing kind tags.
  Probably under 100 lines of changes to `marker_defs_data.rs`.
- Validator: ~40 lines in `marker_defs.rs::tests`.
- The tests in `marker_defs::tests` (line 1186 onward) are the
  regression net. After curation they should still pass without
  modification.

## Sequencing

If both this plan and `plan-whitespace-lint-rules.md` are queued, do
**this one first**. Lint rules walk `MARKER_WHITESPACE`, which depends
on `lookup_spec_marker` resolving correctly — and `lookup_spec_marker`
is exactly where the data quality issues bite. Curating the data
first means the lint rules land on solid ground.
