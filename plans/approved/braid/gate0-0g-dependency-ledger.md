# Gate 0G — dependency, feature, and generation feasibility

Executes `./braid-epic.md` §3.1 0G. Evidence only; no manifest, source, or generated artifact was
changed. Base commit `c22caa9` (branch `braid`), collected 2026-07-27.

Retained raw evidence (not normative, not in the repo's tracked tree):

| file | sha256 |
| --- | --- |
| `target/braid-gate0/cargo-metadata-full.json` (`cargo metadata --format-version 1`) | `bdb185477c4d047fd962b7d2a42e38953ec8080ba674a6f3ac0ca6d6e602d654` |
| `target/braid-gate0/cargo-metadata-nodeps.json` (`--no-deps`) | `a57212542add63568407ab3e9bcd33f9c7baf497aa1cf756300d42f9b9a7baac` |
| `target/braid-gate0/core-pub.txt` (423 core public declarations) | generated inventory, see 0C ledger §2 |
| `target/braid-gate0/wasm-build-{bundler,web}.log` | wasm-pack `--dev` build logs, both exit 0 |

The `--no-deps` hash matches the 0A ledger's recorded value exactly, confirming the workspace
manifest set is unchanged since the 0A/0B run.

---

## 1. Current state

Three workspace members (`Cargo.toml` `members = [".", "crates/*"]`, `resolver = "3"`,
`edition = "2024"`, workspace version `0.0.9`):

```text
usfm_onion  ──────────────┐
   ▲                      │
   │                      ▼
usfm_onion_dto ────▶ usfm_onion_wasm
```

Declared features, verbatim from `cargo metadata --no-deps`:

| crate | declared features |
| --- | --- |
| `usfm_onion` | `{default: []}` |
| `usfm_onion_dto` | `{default: [], wasm: ["dep:tsify", "dep:wasm-bindgen"]}` |
| `usfm_onion_wasm` | `{}` (none) |

Resolved features in a workspace build: `usfm_onion` = `["default"]`, `usfm_onion_dto` =
`["default", "wasm"]`, `usfm_onion_wasm` = `[]`. `usfm_onion_dto`'s `wasm` feature is enabled
unconditionally by `usfm_onion_wasm`'s dependency declaration
(`crates/usfm_onion_wasm/Cargo.toml:19`), so it is *always* on in any workspace build — including
native `cargo test --workspace`. Consequence, recorded because it carries into wire:
`wasm-bindgen`, `js-sys`, `web-sys`, `gloo-utils`, and `tsify` are compiled for the **native** host
during `cargo test --workspace`, reached only via the dto and wasm crates:

```text
wasm-bindgen
├── gloo-utils → tsify → usfm_onion_dto → usfm_onion_wasm
├── js-sys → serde-wasm-bindgen → usfm_onion_wasm
├── usfm_onion_dto
└── usfm_onion_wasm
```

That is a build-time cost, not a correctness problem, and it does not reach core.

---

## 2. Where the canonical types and generated constants live

Per plan §2.2#8, §5.1, and §4.1.

| artifact | today | target crate | forces a reverse edge? |
| --- | --- | --- | --- |
| canonical semantic `OwnedToken` + `StableTokenId`, `OwnedNumberInfo`, `OwnedBookCode` | does not exist as such; nearest live shapes are `usfm_onion_dto::Token` (full payload, no trait impls beyond `UsfmToken`/`SerializableToken`) and private `usfm_onion_wasm::WalkToken` (trait-complete, payload-incomplete) | **core** (`usfm_onion`) | **no** — see §3 |
| `OwnedToken::from_parsed(&Token<'_>) -> Self` | scattered: `impl From<&NativeToken> for dto::Token`, `format_token_with_identity`, `map_token` | **core** | no |
| `TryFrom<TokenDto> for OwnedToken` | scattered: `token_to_walk_token`, `token_value_to_format_token` | **wire** (owns the DTO side of the conversion) | no — wire already depends on core |
| boundary `TokenDto` and all 69 other boundary DTOs | 40 in `usfm_onion_dto`, 29 in `usfm_onion_wasm` | **wire** | no |
| packed schema constants, magic, versions, field ids, stable discriminants (§7.1-7.7) | do not exist | **wire** (`schema.rs`) | no |
| generated stable rule catalog (severity/category/issue-type/ICU template per `LintCode`) | hand-maintained `usfm_onion_wasm::lint_code_variants()` (`src/lib.rs:1503-1541`) + core's `ALL_LINT_CODES` test array; JSON snapshots at `crates/usfm_onion_wasm/golden/outputs/lint-code{s,-meta}.json` | **wire** generates it from **core**'s `LintCode` | no — generation reads core, emits into wire |
| generated JS schema constants + golden vectors (Phase A step 6) | do not exist; today's analogue is the 5 corpus-wide goldens | **wire** | no |
| `SourceHash` (xxhash3-64) | **no xxhash dependency exists anywhere in the workspace today** (`grep xxhash\|twox` over all four manifests: no match). Core has `rustc-hash` (FxHash) only, which §2.1#8 keeps internal-only. | **braid** (leaf dep) | no |
| resident corpus, dirty state, snapshots, patch table | do not exist (the editor's `WorkspaceMirror` holds the equivalent) | **braid** | no |
| wasm-bindgen classes/functions, JS input mapping | `usfm_onion_wasm` | **wasm** | no |
| official JS decode/materialize/reconcile | do not exist; `js/token-sids.js` is the only JS helper today | **js** | n/a |

---

## 3. Can core own `OwnedToken` without acquiring serde/tsify/wasm-only dependencies?

**Yes, and it needs no new dependency at all.**

1. **serde is already unconditional in core.** `Cargo.toml` declares
   `serde = { version = "1", features = ["derive"] }` outside any `cfg`, and core types already
   derive it (`Token`, `TokenData`, `Sid`, `Span`, `LintIssue`, `FormatToken`, `DiffSkeleton`, …).
   So "core must not acquire serde for its default semantic layer" is moot — it has had it all
   along. Core derives `Serialize` **one-way**; the only core types that also derive `Deserialize`
   are the lint-input types (`LintCategory`, `LintSeverity`, `LintIssueType`, `LintCode`,
   `LintScope`, `LintOptions`, `LintSuppression`). `OwnedToken` needs no `Deserialize`: §5.1 puts
   deserialization behind wire's `TokenDto` plus `TryFrom<TokenDto> for OwnedToken`, and that impl
   lives in wire.
2. **core is tsify/wasm-bindgen-free and stays that way.**
   `cargo tree -p usfm_onion -e normal | grep -c 'wasm-bindgen\|tsify'` = **0**. The reverse-dependency
   tree for `wasm-bindgen` (§1) reaches only `usfm_onion_dto` and `usfm_onion_wasm`.
3. **`OwnedToken`'s trait impls are already proven possible on an owned shape.** Three owned token
   types already implement core's traits with owned `String` fields and no lifetime:
   `FormatToken` (core: `UsfmToken`, `WalkableToken`, `LintableToken`, `FormattableToken`,
   `DiffableToken`), `usfm_onion_dto::Token` (`UsfmToken`, `SerializableToken`), and
   `usfm_onion_wasm::WalkToken` (`UsfmToken`, `WalkableToken`, `LintableToken`, `DiffableToken`).
   `OwnedToken` is the union of those two coverage sets over the fuller payload — no new capability,
   and the traits are all core-owned already, so the impls sit in core beside them.
4. **No feature-gated serialization is required in core**, so §3.1 0G's "a feature-gated
   serialization that creates a reverse edge is a stop" cannot trigger from core.

---

## 4. Target DAG proof

```text
            usfm_onion            (sink: no workspace dependency)
             ↗      ↖
usfm_onion_wire      braid       (siblings: neither depends on the other)
             ↖      ↗
         usfm_onion_wasm          (composition root)
```

**Acyclicity.** The edge set is `{wire→core, braid→core, wasm→core, wasm→wire, wasm→braid}`. A
topological order exists — `core < {wire, braid} < wasm` — therefore no cycle. The property that
makes it hold is that **core is a sink**: it declares no workspace dependency today
(`cargo tree -p usfm_onion --depth 1` lists nine third-party crates and nothing else) and the epic
adds none, because every new type core acquires (`OwnedToken`, `StableTokenId`, `OwnedNumberInfo`,
`OwnedBookCode`) is defined in terms of types core already owns.

**No reverse edge is forced.** Checked type by type against the §5 domain surface:

| type the plan introduces | lives in | needs from | would it force a reverse edge? |
| --- | --- | --- | --- |
| `OwnedToken` + payload types | core | core only | no |
| `TokenDto`, packed schema, codecs, `DecodeError` | wire | core (`OwnedToken`, `LintResult`, marker catalog) | no |
| `Braid`, `CorpusInput`, `BookInput`, `ChapterInput`, `ChapterTarget`, `CorpusScope`, `LintSnapshot`, `BookLintSnapshot`, `SnapshotId`, `SourceHash`, `TokenPatch`, `PatchHandle`, `IngestError`, `PatchError`, `PrimeError`, `BaselineError`, `ScopeError` | braid | core (`OwnedToken`, `LintOptions`, `LintResult`, `TokenFix`, `FormatOptions`, `DiffSkeleton`) + xxhash | no |
| `LintPrimeInput` / `BookLintPrime` | braid | core `LintResult` + braid `TokenPatch` — **deliberately not** wire bytes (§5.3: "the composing adapter proves wire and catalog compatibility; Braid does not depend on wire") | no |
| `DiffSkeleton<OwnedToken>` from `braid.diff_baseline` | braid returns core's generic instantiated with core's `OwnedToken` | core | no |
| wasm `Braid` class, `ApiResult<T,E>`, TS error mirrors | wasm | core + wire + braid | no |

The one construction that *would* create a reverse edge is wire owning a serde/tsify mirror of a
braid lifecycle type (wire→braid). §4.1 forbids it structurally, and §8.4 of the 0C ledger records
the resulting placement question as an OWNER-DECISION rather than resolving it. Both candidate
answers are DAG-legal:

- **(a) braid gains a feature-gated `serde`/`tsify` layer over its own types.** Edge added:
  `braid → tsify → gloo-utils → wasm-bindgen`. No edge toward wire. No cycle. Cost: braid carries
  wasm-bindgen when the feature is on, and §4.1's "braid must not own JS values" needs a narrow
  reading (deriving a serialization for a type you already own is not owning a JS value).
- **(b) wasm defines tsify mirror types over braid's semantic types.** No new edges at all. Cost:
  reintroduces exactly the hand-mirrored-DTO pattern §4.1 and `reference-wire-dto-single-source`
  exist to prevent, for ~15 types.

**No feature cycle.** The target feature graph is a tree, not a graph:

```text
usfm_onion:       default = []                         (no features consumed by anyone)
usfm_onion_wire:  default = [], wasm = ["dep:tsify", "dep:wasm-bindgen"]
braid:            default = []  [+ optional "serde"/"tsify" under decision 8.4(a)]
usfm_onion_wasm:  none; enables usfm_onion_wire/wasm  [+ braid/wasm under 8.4(a)]
```

Only `wasm` sits downstream of any feature flag, and it enables flags on its dependencies rather
than the reverse. Because braid does not depend on wire, no feature enabled on wire can reach braid
or vice versa — the sibling split is what makes a feature cycle structurally impossible, not merely
absent. Core consumes no feature from anyone.

Cargo's own guard remains available: `cargo metadata` fails on a cyclic path dependency, and Phase A
step 1 ("create the crate and one-way dependencies") is where that check first runs for real.

---

## 5. Native and wasm dependency / feature matrices

### 5.1 Current, per member (`-e normal`, direct deps only)

| crate | native (`aarch64-apple-darwin`) | `wasm32-unknown-unknown` | delta |
| --- | --- | --- | --- |
| `usfm_onion` | logos, memchr, quick-xml, rayon, regex, rustc-hash, serde, serde_json, similar | logos, quick-xml, regex, rustc-hash, serde, serde_json, similar | native-only: `rayon`, `memchr` (both under `cfg(not(target_arch="wasm32"))`) |
| `usfm_onion_dto` | serde, usfm_onion, tsify, wasm-bindgen (`wasm` feature always resolved on) | identical | none |
| `usfm_onion_wasm` | js-sys, serde, serde-wasm-bindgen, serde_json, tsify, usfm_onion, usfm_onion_dto, wasm-bindgen | identical | none |
| dev-only | core: criterion, proptest; dto: serde_json | — | dev deps never reach either shipped artifact |

Whole-workspace resolved package counts: **58 native, 52 wasm32**. The six-package delta is exactly
rayon's subtree — `rayon`, `rayon-core`, `crossbeam-deque`, `crossbeam-epoch`, `crossbeam-utils`,
`either`. `memchr` is declared native-only in core but still resolves on wasm32 transitively through
`regex`, so it is not part of the package-count delta.

The parallelism seam is a single `cfg`-switched function, `usfm_onion::par::map_ordered`
(`src/par.rs`): rayon `par_iter` natively, plain `iter` on wasm, and the module documents that
because the seam is order-preserving, output is byte-identical regardless of target or thread count.
This is the mechanism by which **no target-specific semantic type exists** — the targets differ in
scheduling, never in representation.

### 5.2 Target, per member

| crate | native | wasm32 | notes |
| --- | --- | --- | --- |
| `usfm_onion` | unchanged from 5.1 | unchanged from 5.1 | gains `OwnedToken` etc. with **no** new dependency (§3) |
| `usfm_onion_wire` | usfm_onion, serde (+ tsify, wasm-bindgen when `wasm` on) | same | inherits dto's shape; `wasm` feature is what carries tsify |
| `braid` | usfm_onion, xxhash3 impl | same | **new third-party dependency**: no xxhash crate exists in the workspace today. Must be `no_std`-friendly / wasm-clean; verify at Phase C, not assumed here. |
| `usfm_onion_wasm` | usfm_onion, usfm_onion_wire (`wasm`), braid, wasm-bindgen, js-sys, serde, serde_json, serde-wasm-bindgen, tsify | same | composition root; unchanged third-party set |

Two feasibility notes for later phases, recorded now:

- **braid must compile for wasm32.** Its only new dependency is the xxhash implementation; the
  candidate must build for `wasm32-unknown-unknown` without `std::time`, threads, or IO. The
  existing gate `npm run check:wasm:web`
  (`cargo check --manifest-path crates/usfm_onion_wasm/Cargo.toml --target wasm32-unknown-unknown`)
  will cover braid automatically once wasm depends on it, because it type-checks the whole
  dependency closure.
- **rayon in braid is a non-goal.** §12 and §8.3 both forbid treating native rayon as evidence for
  webview wasm; if braid ever parallelises dirty-book lint it must route through core's existing
  `par::map_ordered` seam rather than adding its own rayon dependency (`parallelism-strategy`).

---

## 6. `usfm_onion_dto` consumers and the concrete removal step

### 6.1 Complete consumer list

Searched across all Cargo manifests and all `*.rs` in the primary checkout (worktree hits under
`.claude/worktrees/` are historical copies, excluded).

**Crate dependencies — exactly one:**

| consumer | declaration |
| --- | --- |
| `usfm_onion_wasm` | `crates/usfm_onion_wasm/Cargo.toml:19` — `usfm_onion_dto = { path = "../usfm_onion_dto", features = ["wasm"] }` |

**`use` sites — exactly one:**

| site | content |
| --- | --- |
| `crates/usfm_onion_wasm/src/lib.rs:48-56` | one `pub use usfm_onion_dto::{…}` block re-exporting 40 types + `format_sid` + `map_marker_info` |

Plus one non-code reference: a comment at `crates/usfm_onion_wasm/src/lib.rs:1173` explaining that
`map_token`'s conversion body lives in the dto crate. No `tests/`, `benches/`, `examples/`, or
`src/bin/` file references the crate. Core does **not** depend on it (that would already be the
forbidden reverse edge).

**npm-visible types:** 40 of the 73 exported types in `pkg-bundler/usfm_onion_web.d.ts` originate in
`usfm_onion_dto` (enumerated in the 0C ledger §3). They reach the declarations purely through the
`pub use` above — tsify emits from the defining crate, and the emitted declaration carries no crate
path, which is why moving the definitions changes no `.d.ts` byte.

**The one item the re-export list misses:** `decode_attr_value` (dto `src/lib.rs:1023`) is public but
absent from the `pub use` block. An absorption commit that follows the re-export list rather than the
crate's public surface would drop it.

### 6.2 Concrete removal step (§2.2#2, Phase A step 1)

One commit, mechanical:

1. `git mv crates/usfm_onion_dto/src/lib.rs crates/usfm_onion_wire/src/dto.rs`; create
   `crates/usfm_onion_wire/{Cargo.toml,src/lib.rs}` with `pub mod dto;` and the same
   `default`/`wasm` feature pair. Carry all **43** public items, `decode_attr_value` included.
2. Move the dto crate's own tests (`boundary_enums_*`, `token_deserializes_without_span_field`, and
   the rest of `src/lib.rs:1529-1761`) with the code; they are the drift guard §11.3 relies on.
3. Rewrite `crates/usfm_onion_wasm/Cargo.toml:19` to `usfm_onion_wire = { path = "../usfm_onion_wire", features = ["wasm"] }`
   and `crates/usfm_onion_wasm/src/lib.rs:48` to `pub use usfm_onion_wire::dto::{…}` — same 42
   names, same order.
4. `rm -r crates/usfm_onion_dto`. `members = [".", "crates/*"]` needs no edit (glob).
5. Prove the move is declaration-neutral: rebuild both targets to scratch and `diff` the public
   `.d.ts` against the committed trees. §1 of this ledger establishes that this diff is **empty**
   today for bundler and **`InitOutput`-only** for web, so any other difference after the move is a
   real regression, and the check has a known-good baseline to compare against.
6. Run `cargo test --workspace`, `cargo test --test lint_oracle -- --ignored`,
   `npm run check:wasm:web`, `npm run test:wasm`, `npm run golden:wasm`, `npm run golden:wasm:web`,
   `npm run test:token-sids:import` — never with `BLESS=1` / `UPDATE_GOLDEN=1`.

Per §4.1 no compatibility crate or deprecated alias is left behind, and per §3.2#14 the crate rename
is reviewed as part of the one deliberate breaking pre-1.0 release.

Ordering constraint: the 29 boundary DTOs currently local to `usfm_onion_wasm` (0C ledger §4.1) move
in the same direction but are a **separate** step — they must gain `pub` fields so the native/Tauri
boundary can use them, which is a source change, not a move. Keeping the two apart is what lets step
5's declaration diff stay meaningful.

---

## 7. One generated schema source; the drift check

### 7.1 Bundler and web cannot silently ship different discriminants

Four independent pieces of evidence:

1. **One crate, one script, one flag.** `package.json`'s `build:wasm:bundler` and `build:wasm:web`
   both run `node scripts/build-wasm.mjs <target>`, which invokes `wasm-pack build
   crates/usfm_onion_wasm --target <bundler|web> …` (`scripts/build-wasm.mjs:36,43-70`). The only
   difference is `--target` and the output directory. There is no second schema input, no
   per-target source file, and no `cfg`-gated DTO definition anywhere in the dto or wasm crates.
2. **The committed declarations agree.** `diff` of `pkg-bundler/usfm_onion_web.d.ts` and
   `pkg-web/usfm_onion_web.d.ts` over lines 1-515 is empty; `pkg-web` then adds only loader glue
   (`InitInput`, `InitOutput`, `SyncInitInput`, `initSync`, `__wbg_init`). Every discriminant string
   — all 32 `LintCode`s, all 18 `MarkerCategory`s, all 20 `SpecContext`s, `TokenKind`, `TokenFix`'s
   `type` tags, everything — is byte-identical between the two targets.
3. **Regeneration is stable.** §1: the bundler public `.d.ts` regenerates byte-identically under the
   local toolchain; the web one differs only inside `InitOutput`.
4. **One golden directory serves both targets.** `scripts/wasm-golden.mjs:22-24` picks `pkgDir` from
   `argv[2]` but resolves `outputsDir` to `crates/usfm_onion_wasm/golden/outputs` unconditionally.
   `npm run golden:wasm` (bundler) and `npm run golden:wasm:web` compare **the same 117 golden
   files**. A discriminant that differed between targets would fail one of the two runs. Both were
   green in the 0B baseline.

### 7.2 Declaration-generation and drift-check commands

| purpose | command |
| --- | --- |
| generate bundler declarations (ships) | `npm run build:wasm:bundler` → `wasm-pack build crates/usfm_onion_wasm --target bundler --profile wasm-release-fast --out-dir pkg-bundler --out-name usfm_onion_web --no-opt` then `wasm-opt -O3 --enable-bulk-memory` |
| generate web declarations (ships) | `npm run build:wasm:web` (same, `--target web`) |
| generate for iteration | `npm run build:wasm:{bundler,web}:dev` → `wasm-pack build … --dev` |
| **declaration drift check** | rebuild to a scratch `--out-dir`, then `diff pkg-<t>/usfm_onion_web.d.ts <scratch>/usfm_onion_web.d.ts`. Compare **only** `usfm_onion_web.d.ts`; treat `usfm_onion_web_bg.wasm.d.ts` and the `InitOutput` body as toolchain-derived (§1). |
| **schema-value drift check** | `npm run golden:wasm` **and** `npm run golden:wasm:web`, never with `UPDATE_GOLDEN=1` |
| wasm typecheck of the whole closure | `npm run check:wasm:web` |
| npm export-map smoke | `npm run test:wasm` (both targets), `npm run test:token-sids:import` |

What a drift check compares today, and the gaps a wire-generated schema must close:

| generated artifact | drift-checked by | gap |
| --- | --- | --- |
| `pkg-*/usfm_onion_web.d.ts` (73 types, 24 fns, 2 classes) | rebuild + `diff` (manual; **not** wired into any npm script or CI step) | no automated gate — the 0A/0B run is what surfaced the ABI question at all |
| `golden/outputs/lint-codes.json` (32), `lint-code-meta.json`, `format-rules.json` (15), `format-rule-meta.json`, `marker-catalog.json` | `golden:wasm` + `golden:wasm:web` | values are gated; the **source** is a hand-maintained list for lint codes (`lint_code_variants()`, 0C ledger F2) |
| 112 per-fixture goldens over 7 fixtures × 16 outputs | same | 13 exported functions have no golden (0C ledger F3); `vrefIndexUsfm`/`vrefIndexTokens` have no coverage in any harness |
| `pkg-*/package.json` | nothing | regenerates identically today (§1) |

`scripts/restore-wasm-package-layout.mjs` runs after every real build and rewrites `.gitignore`
inside `pkg-bundler`/`pkg-web` because wasm-pack overwrites it. It touches no declaration and was
deliberately not run during this gate.

Phase A step 6 ("generate JS schema constants and token golden vectors from one Rust-owned schema
source") should therefore land the missing piece rather than a parallel one: a wire-owned generator
whose output is compared, not blessed, and a `.d.ts` diff step that has an actual command behind it.

---

## 8. Stop conditions

| §3.1 0G stop condition | status |
| --- | --- |
| a cycle | **not hit.** Topological order `core < {wire, braid} < wasm` exists; core is a sink with zero workspace dependencies today and gains none (§4). |
| a feature cycle | **not hit.** The target feature graph is a tree; only `wasm` sits downstream of a flag, and the wire/braid sibling split makes a cross-feature path structurally impossible (§4). |
| a duplicated discriminant source | **not hit.** One crate, one script, one golden directory, byte-identical cross-target declarations (§7.1). The nearest thing is the hand-maintained `lint_code_variants()` list — a single Rust-side list feeding both targets, i.e. a *manual* source, not a *duplicated* one; wire's generated catalog replaces it (0C ledger F2, disposition **replace**). |
| a target-specific semantic type | **not hit.** No `cfg`-gated type exists in core, dto, or wasm. The only target divergence is `par::map_ordered`'s executor, and `src/par.rs` documents that the seam is order-preserving so output is byte-identical across targets. |
| a feature-gated serialization creating a reverse edge | **not hit.** Core already carries unconditional serde and needs no feature gate; `OwnedToken` requires no new dependency (§3). |

Two feasibility items to carry into later phases, neither a stop:

1. **braid's xxhash dependency does not exist yet.** No xxhash/twox crate is in any manifest. The
   chosen crate must build for `wasm32-unknown-unknown`; verify at Phase C via
   `npm run check:wasm:web` once wasm depends on braid.
2. **The `wasm` feature is always on for wire in a workspace build**, exactly as it is for dto today,
   so native `cargo test --workspace` will keep compiling `wasm-bindgen`/`tsify` for the host. It
   reaches only wire and wasm, never core or braid.
