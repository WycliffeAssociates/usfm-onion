# Braid Phase B pause — 2026-07-28

## Where we are

Phase A is complete.

Phase B is partly complete, but it is not closed yet. The low-level finding storage now exists: it
can write and safely inspect the fixed finding rows, optional columns, marker references, and
message-parameter strings. The checked reader rejects malformed structures rather than trusting
bytes from disk.

A clean-room review found two problems in that layer. The first allowed invalid message parameters
for a lint rule. The second rejected valid sections where only some findings used an optional
column. Both were fixed in commit `7ad8f93` and passed re-review. Commit `98fbcb7` corrected the last
stale field count in the freeze document.

The next step was converting those checked rows into the ordinary semantic `LintIssue` objects.
That work stopped before making changes because it uncovered a real format ambiguity described
below.

The JavaScript boundary has also been clarified. JavaScript will not parse the packed format or
calculate XXH3. It hands the packed bytes and source bytes to wasm. Rust validates the container,
matches the source, and either returns ordinary semantic objects or a typed error. A typed error
tells the application to use the normal Braid USFM parse/ingest path.

## Decisions to make

### 1. How should a large related finding span be stored?

The current related-finding record stores a token index plus a 16-bit offset and 16-bit length.
Offsets or lengths above 65,535 do not fit. The existing overflow record belongs to the finding's
primary span, so it cannot also describe the related span without becoming ambiguous—especially
when both spans overflow.

Recommended decision: make the related record 16 bytes:

```text
token index:  u32
offset:       u32
length:       u32
reserved:     u32 (must be zero)
```

This uses an already-approved field width, needs no additional flag or sidecar, and adds only eight
bytes per related finding. The largest observed book had 262 related findings, so the observed
worst-case increase is roughly 2 KB.

Alternative: add a separate related-overflow column and presence contract. This saves bytes for
ordinary related findings but adds another field, flag/sentinel rules, and more reader complexity.

### 2. Should pure-JavaScript finding reconciliation wait for Braid?

The plan names `reconcileFindings(previous, next)`, but never defines its `MaterializedFindings`
type. The existing public finding span is an absolute source span, while the approved stable
identity requires a token-relative address. A JavaScript helper that receives findings alone
cannot derive that identity safely.

Recommended decision: defer this helper until Phase C, when resident Braid types can own a proper
finding address and identity. This does not block the clarified cold-open use case, which only
requires wasm to return valid tokens/semantic objects or a typed error.

Alternatives would be to add a new public identity/address field to materialized findings now, or
to weaken identity by using absolute spans. The latter contradicts the approved plan and should
not be chosen accidentally.

## Work remaining before Phase B can close

1. Record the related-span-width decision in the freeze and update the structural codec.
2. Implement and review semantic finding encode/decode, excluding snapshot-bound fixes.
3. Expose wasm-backed `decodeTokens` and `materialize` with typed errors.
4. Resolve or explicitly defer `reconcileFindings`.
5. Run the Phase B Rust, wasm, malformed-buffer, golden-vector, and corpus gates.

Fixes and their packed patch table remain intentionally deferred to Phase C, where Braid's
snapshot and patch identities exist.
