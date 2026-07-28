# Gate 0H — spike charter and unchanged replay

Status: **SIGNED OFF by owner 2026-07-27** — charter frozen. Execution still gated on 0F
completion (per §3.1, 0A–0G pass before any timing run). Dispatch: Sonnet, high effort — the
replay runs documented commands and records; the judgment lives in this charter. Any
correctness-gate failure or Δ-classification ambiguity is a stop-and-report, never resolved by
the worker.

Owner rationale recorded at sign-off, attaching to Δ1: the cold open pays ~66 IO-bound per-book
file reads regardless, so embedding source in the container buys zero reads while duplicating
every source byte on disk; external source costs one additional sidecar read against that
baseline. The replay's Δ1 job is only to quantify the embedded section's share of the historical
size/timing numbers so the old measurements can be interpreted for the v1 layout.

## Replay target (historical evidence, not production design)

`.claude/worktrees/agent-af68c779deab4e90a` — commit `c341de5` plus its preserved dirty state
(modified `src/lib.rs`, `src/marker_defs.rs`; untracked `src/wire_spike.rs`,
`examples/wire_spike.rs`, `examples/wire_probe.rs`, `js/{wire_decode,wasm_vs_bin,spike_b}.mjs`,
`GEN.{bin,json}`, `PSA.{bin,json}`) — per the 0A worktree ledger. The worktree is replayed
**byte-for-byte unchanged**: no cleaning, no updating, no edits, ever.

Documented replay commands (run inside the worktree):

```bash
cargo run --release --example wire_spike
node js/wire_decode.mjs <output-dir>
node js/wasm_vs_bin.mjs <output-dir>
node js/spike_b.mjs <output-dir> PSA
node js/spike_b.mjs <output-dir> GEN
```

Replay runs **locally** (same machine class as the original evidence; record exact toolchain from
the 0A ledger plus `node --version`). A quiet-box repetition via `./bench-remote.sh` is optional
follow-up if local noise obscures a verdict — absolutes are per-machine series; only ratios
transfer.

## Hypotheses (what the replay is evidence FOR)

1. Exact external-source token reconstruction without lex/parse: packed metadata + the exact USFM
   bytes reconstruct borrowed tokens.
2. Semantic equality: decoded tokens equal `parse(source)` in every public field.
3. Malformed-input rejection: corrupt buffers and wrong source reject without panic or partial
   views.
4. Object-materialization and transfer cost: the wasm→JS object path is the dominant boundary
   cost, and the packed path avoids it.
5. Size/heap benefit: packed sidecar + external source beats materialized JS objects on bytes and
   peak heap.

## Non-hypotheses (explicitly NOT tested by this replay)

Chapter-local lint; threaded wasm; cache IO policy; production API polish. Also NOT evidence for:
the v1 external-source layout's timings (the spike embeds a `SOURCE` section — see contract
deltas), the conditional token-id dictionary (§7.4 `positional_ids`, adjudicated after the spike),
or the position-keyed finding order (§2.2#15).

## Protocol the replay must record

- Exact input paths and sha256 hashes (the worktree's GEN/PSA fixtures and any corpus files its
  harness reads); build profile (`--release`), target triple, machine, rustc/cargo/node versions.
- Warmup count, measured iterations, and reported statistic exactly as the spike harness defines
  them — record what the harness does, do not modify it.
- Whether filesystem IO is inside or outside each timed region (read the harness source and say).
- Correctness checks execute BEFORE timers and their results are recorded first: semantic
  equality vs `parse(source)` (every public field); token→USFM byte identity; Rust serial/parallel
  decode agreement where available; JS materialization equals Rust; wrong-source and
  malformed-buffer rejection.
- Independent timing phases, reported separately, never summed into one number: read,
  checksum/validation, decode-view construction, semantic object materialization, wasm parse,
  wasm→JS marshalling, structured clone/transfer, peak heap.

## Stop thresholds (any one stops adoption of spike evidence)

1. Any semantic mismatch in the correctness gate.
2. Any payload the packed form cannot represent.
3. Any dependency on the private raw marker index that the v1 catalog-stamp design cannot replace.
4. Loss of the measured boundary advantage: if packed decode + materialization does not retain a
   material advantage over wasm object materialization on GEN, stop and diagnose rather than
   proceed on historical numbers (§12).

## Contract deltas (spike vs normative v1 — record evidence for each)

The replay is historical evidence; v1 rejects these spike behaviors, so classify each with
evidence from the run:

| # | spike behavior | v1 contract | replay must record |
| --- | --- | --- | --- |
| Δ1 | embedded `S_SOURCE` section | external source bytes bound by length+xxhash (§2.2#3, §7.4) | size/timing share attributable to the embedded source |
| Δ2 | private/raw marker index, enum-order arrays | deterministic catalog stamp + explicit stable discriminants (§7.4, §7.7) | every site depending on marker ordinal/raw index |
| Δ3 | `assert!`/unchecked indexing | validated checked decode, typed `DecodeError` (§7) | panic sites and unchecked reads inventory |
| Δ4 | 10-byte SID record | 8-byte `PackedSid` with fidelity bit (§7.5) | per-book SID dictionary size delta estimate |
| Δ5 | token-id dictionary always present | conditional on `positional_ids` (§7.4, adjudicated 2026-07-27) | measured dictionary share (0E predicts 31–41%) |

## Deliverables

1. Charter sign-off recorded (owner) — this file's status flipped from DRAFT with a dated line.
2. Replay results appended to `./progress-braid-epic.md` as
   `## <date> — Gate 0H: spike charter and unchanged replay`: correctness-gate results, per-phase
   timings, sizes, heap, the Δ1–Δ5 evidence, and machine/toolchain provenance.
3. A yes/no **reuse / rewrite / discard** verdict per Phase A component (checked reader/writer
   primitives, header/TOC/directory validation, token columns, dictionaries, SID packing, JS
   decode), each justified in one line against the deltas above.
4. Gate 0 closure statement: with 0A–0H all recorded, Phase 0 steps 2–3 (contract verification and
   discriminant/API-ledger freeze) become the next packets before Phase A code.
