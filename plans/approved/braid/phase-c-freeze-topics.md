# Phase C freeze topics — owner decision sheet (drafted 2026-07-30)

Status: DRAFT for owner adjudication before packet C4 (patch resolution + finding-and-patch
gate). Decisions get recorded as dated freeze appendices in `phase0-freeze.md` once ruled.

Context: Phase B deliberately deferred the patch table (freeze §F.3) because a patch is
snapshot-bound and no snapshot identity existed. Phase C creates both. `TokenFix` today:
`{code, label, label_params, target_token_id, edits: Vec<TokenTemplate>}` with
`TokenTemplate = {kind, text, marker?, sid?}`; three producer rules in the corpus
(missing-whitespace-before-marker 48, missing-horizontal-whitespace-after-marker-name 32,
missing-tag-end-delimiter-after-marker 12 — 92 fixes total).

## A. Patch-table wire framing (finding-section fields 5/6)

- **A1 — edit addressing.** (a) byte-span edits into the book's snapshot source (flat
  `{op, start, end, payload}`; patches are snapshot-bound anyway) vs (b) token-addressed edits
  (row index + intra-token offset, like finding spans). Recommendation: **(a)** — application is
  a byte splice; token addressing adds indirection with no consumer.
- **A2 — patch identity.** (a) `PatchId = (SnapshotId, ordinal)` — the patch's position in the
  snapshot's own table; staleness falls out of snapshot comparison. (b) content hash. (c) minted
  GUID. Recommendation: **(a)** — no new id space, deterministic, and the frozen container
  header already carries `snapshot_id`.
- **A3 — string storage.** Reuse the finding section's field-7 string dictionary and the
  field-8 key/value pair framing for `label`/`label_params`/template strings, vs a separate
  patch-local dictionary. Recommendation: **reuse field 7/8 framing** — one interning mechanism
  per section, same checked reader.
- **A4 — already epic-stated, freeze names only (not decisions):** flat sorted non-overlapping
  edits, applied in reverse order; typed errors for stale-snapshot, overlap, out-of-bounds,
  unknown-patch; decode reconstructs the full `TokenFix` (fields 5/6 populated ⇒
  `LintIssue.fix` round-trips; closes §F.3's interim `fix: None`).

## B. Snapshot identity semantics

- **B1 — what `SnapshotId` (u64, header offset 32) is.** (a) content-derived: xxh3 over the
  ordered per-book source hashes — deterministic, survives restore/process boundaries, same
  corpus ⇒ same id. (b) per-instance monotonic counter — cheap but meaningless after restore.
  Recommendation: **(a)**.
- **B2 — what the id covers.** (a) source bytes only; catalog/rules/config remain separate
  stamps, and cache validity is the tuple (id + stamps). (b) bake config/rules into the id.
  Recommendation: **(a)** — a config flip shouldn't masquerade as a corpus change; the header
  already carries the stamps separately.
- **B3 — mutation contract (mostly epic-stated).** Any effective content mutation ⇒ new id;
  rejected mutations and no-ops preserve the id (and caches). Patch application validity =
  container `snapshot_id` matches resident id AND the target book's source hash matches;
  mismatch is the typed stale error, never partial application.

## Evidence pass before ruling (cheap, builder-run)

Census the 92 corpus fixes: payload shapes, template kinds used, label params, edit widths —
so A1/A3 are ruled on data, mirroring the Phase B payload-census method.
