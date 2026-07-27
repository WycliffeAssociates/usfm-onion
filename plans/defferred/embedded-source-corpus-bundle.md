# Deferred: embedded-source whole-corpus bundle

Status: **deferred** — plausible one-read packaging, not an active braid follow-up.

## Idea

The normative token cache is:

```text
exact UTF-8 USFM bytes + packed token metadata sidecar
→ verify source length/hash
→ reconstruct borrowed Tokens without lex/parse
```

This candidate packages the same source bytes and sidecars into one whole-corpus artifact:

```text
container
  ordered book source blobs
  matching token sections
  matching finding/patch sections
  TOC + versions + hashes
```

It changes packaging, not token semantics or column layout. Token spans still index the exact UTF-8
source blob for their book. The source appears once per book, never once per token.

## Potential value

- one IO read/open for a complete cached corpus;
- one transferable/shareable artifact;
- no path-pairing problem between USFM files and sidecars;
- atomic validation of source, token metadata, findings, patches, and snapshot identity;
- convenient artifact-store or download format.

The artifact remains binary even though it contains raw UTF-8 sections. “Binary” describes the
container/metadata encoding; it does not require every payload byte to be non-text.

## Why it is not v1

- Applications already need canonical `.usfm` files, so embedding them duplicates disk bytes.
- IO latency is not the measured bottleneck that motivated packed tokens.
- External source plus sidecar already avoids lex/parse and allows borrowed token reconstruction.
- Whole-corpus bundling introduces artifact update, atomic-write, and stale-file policy that braid
  itself should not own.
- A candidate should be driven by measured open/share/package needs, not the theoretical minimum
  number of reads.

## Revisit questions

1. Is corpus open measurably IO-bound after external-source sidecars land?
2. Is the application opening many individual files/sidecars or already reading a packed project?
3. Is duplication acceptable, or would the bundle replace rather than accompany source files in
   a distribution artifact?
4. Must one changed book rewrite the whole bundle, or can sections be independently replaced?
5. Who owns atomic writes, cache eviction, corruption recovery, and source-of-truth selection?
6. Does sharing one artifact justify optional compression, and can source remain directly
   sliceable after it?

## Constraints if pursued

- Reuse the normative token/finding/patch schemas; do not create a second codec.
- Preserve caller book order and source bytes exactly.
- Keep braid free of filesystem and artifact-store policy.
- Validate every embedded source against its token section length/hash before decode.
- Permit extraction of the exact original `.usfm` bytes.
- Keep an external-source decode path for applications that already own the source.
- Measure full rewrite amplification and peak memory before selecting whole-file atomic updates.

## Non-goals

- No replacement of canonical USFM as the authoring/source-control format.
- No database, archive filesystem, compression framework, or LFS policy by default.
- No claim that one read is faster without a product-shaped benchmark.
- No change to Braid's resident lifecycle or token identity.

Relates: `../approved/braid-epic.md` §§2.2, 7.4, 17.
