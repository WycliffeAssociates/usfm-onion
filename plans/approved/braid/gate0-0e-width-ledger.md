# Gate 0E — corpus envelope and width study

Executes `./braid-epic.md` §3.1 0E. Evidence only; no production code, snapshot, golden, or
`testData/` change. Base commit `c22caa9` (branch `braid`), collected 2026-07-27.

Companion: [0D payload ledger](./gate0-0d-payload-ledger.md).

---

## 1. Scanner, replay, and input provenance

### 1.1 Tooling

A deterministic read-only Rust scanner plus two probes, all in an **out-of-tree** cargo project that
is detached from the `usfm_onion` workspace (`[workspace]` empty table, path dependency on the repo
root). Nothing was added under `src/`, `tests/`, `examples/`, or `testData/`.

| file | sha256 |
| --- | --- |
| `target/braid-gate0/scanner/src/main.rs` | `3fa299c6658620bb60731e1c4698d38d3e16997b37729f43037c88dc041a0e85` |
| `target/braid-gate0/scanner/src/bin/probe.rs` | `606ea1cffc57009fda7480fc04ff4707ad52513ce81e47c3896539cef4fcc098` |
| `target/braid-gate0/scanner/src/bin/probe2.rs` | `e33e470ab94738c3bf0e8c0e489e8964a6425d436acf98ecdaf77cc5db61344f` |

Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` — same as
the 0A ledger.

### 1.2 Replay commands

```bash
cd target/braid-gate0/scanner && cargo build --release
cd -   # repo root
B=target/braid-gate0/scanner/target/release/gate0e_scanner
$B testData      testData                              > target/braid-gate0/scan-testdata.jsonl
$B golden-inputs crates/usfm_onion_wasm/golden/inputs   > target/braid-gate0/scan-golden.jsonl
$B en_ult        example-corpora/en_ult                 > target/braid-gate0/scan-en_ult.jsonl
$B en_ulb        example-corpora/en_ulb                 > target/braid-gate0/scan-en_ulb.jsonl
$B synthetic     target/braid-gate0/synthetic           > target/braid-gate0/scan-synthetic.jsonl
target/braid-gate0/scanner/target/release/probe   > target/braid-gate0/probe-output.txt
target/braid-gate0/scanner/target/release/probe2  > target/braid-gate0/probe2-output.txt
python3 target/braid-gate0/summarize.py           > target/braid-gate0/summary.txt
```

Each `.jsonl` holds one record per book plus one final `{"kind":"set-total"}` record. **The four sets
are scanned separately and never concatenated** — per-book maxima live in the per-book records; only
genuinely whole-corpus quantities are in the set-total record.

### 1.3 Input hashes (replay provenance)

Rollup method, chosen to reproduce the 0A ledger exactly:
`find <root> -name '*.usfm' -type f | xargs shasum -a 256 | sort | shasum -a 256`.

| set | `.usfm` files | rollup sha256 |
| --- | ---: | --- |
| `testData` | 262 | `2e00e92ca401542d93b757bac5034fbbc66e2a053096f2e99ebacd19a35626fc` |
| `crates/usfm_onion_wasm/golden/inputs` | 7 | `38557e8b6b1010e634fad140c26e4dd758353fe3ccf48e5424e5d028c89414c9` |
| `example-corpora/en_ult` | 67 | `05cb2016dbdc3e2196c5a5e5f64f6d23d5adec654100bc4515448a612649c92b` |
| `example-corpora/en_ulb` | 66 | `b4b53813d6d39ac414ee4f135f82def92e24e9561018c4316bf46cddc3a3e77c` |
| `target/braid-gate0/synthetic` | 30 | `c7901836bcdadfab7572661415ea2d4aacfcb578433f98a7b8397d49fc786991` |

**Cross-check against 0A** — all three 0A rollups reproduce byte-for-byte, so the inputs are provably
unchanged since the baseline:

| 0A entry | recomputed (all files, same method) | matches |
| --- | --- | --- |
| `testData/**/*.usfm` `2e00e92c…` | `2e00e92ca401542d93b757bac5034fbbc66e2a053096f2e99ebacd19a35626fc` | ✓ |
| `example-corpora/en_ult` `dae6682c…` | `dae6682c6286d1d77938146e550d85550ef8bf931de0e5aeca9b4500b5ed693a` | ✓ |
| `example-corpora/en_ulb` `b3d06c69…` | `b3d06c690cea12fe537dccb6051c3e659c4408e51f551c63318b18ebdaa02b6b` | ✓ |

Reconciled file counts: 0A's 78/74 for en_ult/en_ulb are **all-file** counts (manifests, licenses,
`.github/`); the `.usfm` counts this gate scans are **67/66**. Both are recorded above so neither
number reads as an error later.

### 1.4 Scan output hashes

| file | sha256 |
| --- | --- |
| `scan-testdata.jsonl` | `9944ca8f2a2938246ac9cf306b7956b89c0c991f42a26465d2e0cf09286df420` |
| `scan-golden.jsonl` | `152c2d1dd4af3097aa4f4ae73d799500fef19cc56835af4f6e0662785aca1fd9` |
| `scan-en_ult.jsonl` | `1fe008c34d5ccf33991a8db4a3569aea64126ed62070778bc69cc095e9584e45` |
| `scan-en_ulb.jsonl` | `e72ca57ea7706b120a0615fc581531f71b636ce60c5670f4f135894017433450` |
| `scan-synthetic.jsonl` | `f56d27b7aeee9700f3560c8430625176b25b9d7adbb4c14d720f4e1ce30a1b92` |

---

## 2. Per-set envelopes

### 2.1 Whole-corpus totals (set-total records)

These are the only figures that aggregate across books.

| quantity | testData | golden-inputs | en_ult | en_ulb |
| --- | ---: | ---: | ---: | ---: |
| books (`.usfm` files) | 262 | 7 | 67 | 66 |
| total source bytes | 5,958,077 | 1,007 | 103,215,595 | 4,506,101 |
| total tokens | 173,462 | 169 | 5,288,948 | 254,559 |
| total findings | 2,301 | 3 | 46,398 | 14,249 |
| distinct `LintCode`s produced | 26 | 3 | 7 | 5 |
| distinct source book codes | 34 | 2 | 67 | 66 |
| distinct markers | 255 | 13 | 46 | 28 |
| distinct attribute keys | 39 | 1 | 6 | 0 |
| duplicate **declared** book codes | GEN×107, GLO×27, TIT×24, MAT×23, MRK×11, ACT×10, … | GEN×5, MAT×2 | **none** | **none** |
| `tokens_to_usfm` byte-lossless | **262/262** | **7/7** | **67/67** | **66/66** |
| `TokenFix::ReplaceToken` produced | 72 | 1 | 19 | 1 |
| `TokenFix::DeleteToken` / `InsertAfter` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

`testData` and `golden-inputs` are **fixture trees, not corpora**: each file is an independent book
and duplicate declared codes are expected. `en_ult` and `en_ulb` have **zero** duplicate declared
book codes, so both are valid braid corpora under §5.2's uniqueness rule. Both also have exactly one
`\id` token per book (no duplicate source keys, no duplicate `\id`).

### 2.2 Per-book distributions (p50 / p95 / max, with the book producing the max)

Real sets only; the argmax book is named so a later run can target it.

| quantity | testData p50/p95/max (argmax) | en_ult p50/p95/max (argmax) | en_ulb p50/p95/max (argmax) |
| --- | --- | --- | --- |
| source bytes | 331 / 162,598 / **585,637** (origin) | 757,804 / 4,290,430 / **5,154,281** (01-GEN) | 35,144 / 217,155 / **272,592** (19-PSA) |
| token count | 46 / 4,358 / **20,479** (origin) | 39,356 / 225,630 / **276,987** (19-PSA) | 2,234 / 11,472 / **30,857** (19-PSA) |
| max span endpoint | 331 / 162,598 / **585,637** (origin) | 757,804 / 4,290,430 / **5,154,281** (01-GEN) | 35,144 / 217,155 / **272,592** (19-PSA) |
| max single-token bytes | 40 / 783 / **1,852** (origin) | 82 / 202 / **1,198** (A0-FRT) | 280 / 430 / **498** (17-EST) |
| unique token ids | = token count (no duplicates anywhere) | = token count | = token count |
| token-id dict bytes | 272 / 46,798 / **173,201** (origin) | 343,094 / 2,145,193 / **2,658,760** (19-PSA) | 16,766 / 92,140 / **266,603** (19-PSA) |
| non-catalog dict bytes | 0 / 802 / **12,403** (origin) | 52,593 / 250,280 / **316,534** (19-PSA) | 2 / 2 / **2** (01-GEN) |
| unique SIDs | 4 / 451 / **1,584** (origin) | 226 / 1,349 / **2,612** (19-PSA) | 230 / 1,350 / **2,612** (19-PSA) |
| unique markers | 8 / 23 / **221** (kitchen-sink) | 19 / 25 / **28** (19-PSA) | 16 / 19 / **24** (19-PSA) |
| max marker bytes | 3 / 6 / **6** | 6 / 6 / **6** | 4 / 4 / **4** |
| max chapter (from `Sid`) | 1 / 34 / **136** (origin) | 12 / 49 / **150** (19-PSA) | 12 / 50 / **150** (19-PSA) |
| max verse (from `Sid`) | 2 / 44 / **75** (origin) | 32 / 79 / **176** (19-PSA) | 32 / 79 / **176** (19-PSA) |
| max bridge delta (raw `Number`) | 0 / 1 / **71** (origin) | 0 / 0 / **0** | 0 / 0 / **0** |
| max `Sid` delta (saturated `u8`) | 0 / 1 / **71** (origin) | 0 | 0 |
| attribute-bearing tokens | 0 / 69 / **2,902** (origin) | 9,470 / 52,814 / **63,615** (19-PSA) | 0 |
| total attributes | 0 / 266 / **2,902** (origin) | 31,848 / 179,845 / **216,890** (01-GEN) | 0 |
| max attributes / token | 0 / 0 / **9** (origin) | 6 / 6 / **6** (01-GEN) | 0 |
| max attr key bytes | 0 / 13 / **17** (origin) | 13 / 13 / **13** | 0 |
| max attr value bytes | 0 / 29 / **43** (origin) | 40 / 51 / **56** (27-DAN) | 0 |
| max verbatim attr-source bytes | 0 / 139 / **187** (origin) | 166 / 180 / **191** (27-DAN) | 0 |
| findings / book | 1 / 43 / **358** (origin) | 192 / 1,255 / **17,404** (04-NUM) | 113 / 575 / **1,188** (19-PSA) |
| distinct codes / book | 1 / 4 / **12** (common-errors) | 2 / 3 / **7** (19-PSA) | 1 / 2 / **4** (67-REV) |
| max message-params bytes | 0 / 30 / **40** (origin) | 26 / 26 / **29** (04-NUM) | 8 / 8 / **34** (23-ISA) |
| max message-params count | 0 / 3 / **3** | 2 / 2 / **2** | 1 / 1 / **3** |
| max rendered message bytes | 26 / 129 / **141** | 39 / 100 / **104** | 31 / 99 / **141** |
| findings with a related address | 0 / 1 / **14** (origin) | 0 / 2 / **29** (22-SNG) | 0 / 46 / **262** (23-ISA) |
| max finding token-relative length | 2 / 7 / **9** | 3 / 4 / **5** | 3 / 3 / **35** (67-REV) |
| max finding token-relative offset | **0** everywhere | **0** everywhere | **0** everywhere |
| max replacement tokens per fix | 0 / 1 / **1** | 0 / 0 / **1** | 0 / 0 / **1** |
| max `label_params` per fix | **0** everywhere | **0** everywhere | **0** everywhere |

`golden-inputs` is tiny throughout (max 215 source bytes, 32 tokens, 7 unique SIDs, 2 findings) and
sets no maximum for anything.

### 2.3 Modelled packed sizes

Section-size model per §7.3/§7.4: 48-byte section header + 12 field-directory entries (16 B each) +
required columns (`kind` 1 B, `span_start` 4 B, `span_end` 4 B, `token_id_index` 4 B, `sid_index`
2 B, `marker_descriptor_index` 2 B = 17 B/token) + token-id dictionary (bytes + 4 B/entry offset) +
non-catalog dictionary + SID dictionary (8 B/entry) + sparse number/book-code/attribute records +
marker descriptors (16 B each). A model, not a measurement — no encoder exists yet.

| set | largest book | source | tokens | modelled token section | ×source | of which token-id dict |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| testData | origin | 283,270 | 20,479 | 624,308 | 2.20× | 255,117 (41%) |
| golden-inputs | footnote | 215 | 32 | 1,338 | 6.22× | 310 (23%) |
| en_ult | 19-PSA | 5,122,298 | 276,987 | **11,989,965** | 2.34× | 3,766,708 (31%) |
| en_ulb | 19-PSA | 272,592 | 30,857 | 967,466 | 3.55× | 390,031 (40%) |
| synthetic | s22-many-sids | 645,500 | 301,503 | 10,741,803 | 16.64× | 4,109,932 (38%) |

Whole-corpus container estimates (32-byte header + 32 B/TOC entry + sections):

| set | sections | TOC bytes | Σ sections | corpus source |
| --- | ---: | ---: | ---: | ---: |
| testData | 262 | 8,416 | 5,896,366 | 5,958,077 |
| golden-inputs | 7 | 256 | 7,535 | 1,007 |
| en_ult | 67 | 2,176 | **229,892,746** | 103,215,595 |
| en_ulb | 66 | 2,144 | 8,040,420 | 4,506,101 |

**Size observation (not a width breach):** the token-id string dictionary is 31–41% of every
modelled section. For a **cold parse** it is entirely redundant — ids are exactly
`{book}-{index}`, derivable from the section's book id and the token index (§5.1). Only
caller-supplied (editor) ids need storing. A one-bit "ids are positional" section flag would remove
3.77 MB from en_ult's largest book. Worth considering in Phase A step 3; recorded here because 0E is
where the number exists.

---

## 3. Synthetic boundary inputs

30 files under `target/braid-gate0/synthetic/` (rollup hash in §1.3), covering every §7 sentinel and
overflow branch that no real corpus reaches. All 30 are `tokens_to_usfm` byte-lossless.

| file | probes | result |
| --- | --- | --- |
| `s01-bridge-delta-127` | `\v 1-128` — the exact packed-SID `Exact` ceiling | raw delta 127, `Sid` delta 127 → **`Exact`, boundary reached** |
| `s02-bridge-delta-128` | `\v 1-129` — first value past the 7-bit field | raw 128, `Sid` delta 128 → **`AnchorOnly` required** |
| `s03-bridge-delta-255` | `\v 1-256` — core `Sid` `u8` ceiling | raw 255, `Sid` delta 255 → `AnchorOnly` |
| `s04-bridge-delta-256` | `\v 1-257` — past the core `u8` | raw **256**, `Sid` delta **255** → core saturates (documented, `src/token.rs:831`); raw value survives in `TokenData::Number.end` |
| `s05-chapter-65535` | `\c 65535` — `Sid.chapter` `u16` ceiling | `Sid` chapter 65535, raw 65535 |
| `s06-chapter-65536` | `\c 65536` — one past | `Sid` chapter **65535** (saturated), raw **65536** preserved |
| `s07-chapter-999999` | `\c 999999` | `Sid` chapter 65535, raw **999999** preserved |
| `s08-verse-65535` / `s09-verse-999999` | `Sid.verse` `u16` ceiling and past | verse 65535 saturated; raw preserved |
| `s10-blank-marker-long-content` | `\b` + 70,000-byte content token → finding length > `u16` | finding token-relative length **70,001**; but `whole_token == true`, so §7.6's `length = 0` ("whole token") sentinel covers it — **the overflow sidecar has no producer** |
| `s11-invalid-book-code` | `\id ZZZ` | `invalid-book-code` fires (unseen in every real set); `book_code_valid = false`; **lossless** |
| `s12-missing-chapter-number` | bare `\c` | `missing-chapter-number` fires (unseen in every real set) |
| `s13-unknown-close-marker` | `\zqq*` | `unknown-close-marker` + `stray-close-marker` fire (former unseen in every real set) |
| `s14`–`s18` | attempts at `number-range-not-preceded-…`, `unknown-token`, `invalid-number-range` from source | **all failed** → those three codes are token-path-only (0D §2.1). `\v 4-2` parses as `Number{4, Some(2)}` with **no** finding; `\v 1-` parses with no finding. |
| `s19-many-attributes` | 40 attributes on one `\w` | 40 (real max 9) |
| `s20-long-attr-value` | 4,000-byte attribute value | 4,000 (real max 56) |
| `s21-long-marker-name` | 200-byte unknown marker | 200 (real max 6) |
| `s22-many-sids` | 300 chapters × 250 verses | **75,301 unique SIDs > 65,535** → `sid_index: u16` **breach**, see §4 |
| `s23-id-says-2CO` | valid-but-wrong `\id` | 12 tokens, lossless, **zero findings** → confirms the `BookIdMismatch` gap |
| `s24-id-lowercase` | `\id php` | `book-code-not-uppercase`; `book_code_valid = false`; lossless |
| `s25`–`s27` | `\v 1,3`, `\v 1,3-5`, `\v 1-3,7` | first `Sequence` / `SequenceWithRange` tokens in any set (both zero in all real corpora). `s25` gives `Sid` delta **0** — see 0D D2 |
| `s28`–`s29` | `\v 1a`, `\v 1a-2b` | suffix consumed; payload indistinguishable from unsuffixed — see 0D D2 |
| `s30-chapter-sequence` | `\c 1,2` | `Sequence` on a chapter number |

---

## 4. §7 field-by-field verdict

`R` = real-corpus max (testData ∪ golden ∪ en_ult ∪ en_ulb, per book). `S` = synthetic max.
Verdicts: **fits** / **fits, unused** / **breach**.

### 4.1 Container header (§7.1) and TOC (§7.2)

| field | type | R | S | sentinel | headroom | verdict |
| --- | --- | ---: | ---: | --- | --- | --- |
| magic `[u8;4]` = `uson` | — | — | — | — | — | fits |
| format version | `u16` | 1 | — | — | 65,534 revisions | fits |
| header length | `u16` = 32 | 32 | — | — | — | fits |
| flags | `u32` | 0 | — | unknown bits reject | 32 bits | fits |
| section count | `u32` | 262 (testData tree) / 67 (en_ult) | 30 | — | ~4.29e9 | fits |
| TOC offset | `u64` | 32 | — | — | — | fits |
| integrity checksum | `u64` | — | — | `0` = omitted | — | fits |
| TOC: kind | `u8` | 2 kinds | — | — | 253 | fits |
| TOC: book | `[u8;3]` | 67 distinct | — | — | — | fits — core `BookId` is already `[u8;3]` (§2.2 adjudication) |
| TOC: section version | `u16` = 1 | 1 | — | — | — | fits |
| TOC: flags | `u16` | 0 | — | unknown bits reject | 16 bits | fits |
| TOC: absolute offset | `u64` | ≤ 230 MB (en_ult) | — | 16-B aligned, non-overlapping | — | fits |
| TOC: byte length | `u64` | ≤ 11,989,965 | 10,741,803 | — | — | fits |
| TOC: source hash | `u64` | — | — | — | — | fits (xxhash3-64) |

16-byte alignment: no measured quantity conflicts with it; padding cost is ≤ 15 B per section
(≤ 1,005 B for en_ult's 67 sections).

### 4.2 Section header and field directory (§7.3)

| field | type | R | S | verdict |
| --- | --- | ---: | ---: | --- |
| record count | `u32` | 276,987 tokens (en_ult 19-PSA); 17,404 findings (en_ult 04-NUM) | 301,503 | fits — 4 orders of magnitude of headroom |
| directory count | `u16` | ~12 | — | fits |
| directory entry size | `u16` = 16 | 16 | — | fits |
| section byte length | `u64` | 11,989,965 | 10,741,803 | fits |
| field entry: `field_id` | `u16` | ~12 | — | fits |
| field entry: `element_width` | `u8` | 8 | — | fits |
| field entry: `section_relative_offset` | `u32` | ≤ 11,989,965 | ≤ 10,741,803 | fits — **but this caps one section at 4 GiB**; largest modelled section is 12 MB, i.e. 0.3% of the cap |
| field entry: `byte_len` | `u32` | 4,708,779 (largest column) | 5,125,551 | fits |
| field entry: `count` | `u32` | 276,987 | 301,503 | fits |

### 4.3 Token section (§7.4)

| field | type | R | S | sentinel | verdict |
| --- | --- | ---: | ---: | --- | --- |
| `kind[N]` | `u8` | 9 variants | — | — | **fits** — 246 spare |
| `span_start[N]`, `span_end[N]` | `u32` | 5,154,281 (en_ult 01-GEN) | 645,500 | — | **fits** — 0.12% of the `u32` range |
| `token_id_index[N]` | `u32` | 276,987 entries | 301,503 | — | **fits** |
| `sid_index[N]` | `u16` | **2,612** (en_ult/en_ulb 19-PSA) | **75,301** (`s22`) | `0xffff = None` | **BREACH — see §4.6** |
| `marker_descriptor_index[N]` | `u16` | **221** (testData kitchen-sink) | 4 | `0xffff = None` | **fits** — 65,314 spare; corpus-wide distinct markers 255 |
| sparse number records | unspecified | 32,292/book | 75,344 | — | fits — **but the payload must be `u32`**: `TokenData::Number.start/end` reach 999,999 synthetically while `Sid` saturates at 65,535. A `u16` number payload would silently normalize malformed input, violating §0's "never silently normalize on ingest". |
| sparse book-code records | unspecified | 1/book | 1/book | — | fits |
| sparse attribute-list records | unspecified | 216,890 attrs (en_ult 01-GEN); 9 attrs/token max | 40/token | — | fits; verbatim attr-source slice max 191 B real / 4,000 B synthetic |
| string dictionary bytes | `u32` offsets | 2,658,760 (token ids) + 316,534 (non-catalog) | 4,109,932 | — | fits |
| string dictionary entries | `u32` | 276,987 | 301,503 | — | fits |
| marker descriptors | — | 221/book, 255 corpus-wide | 4 | — | fits |
| `token_idx` | `u32` | 276,987 | 301,503 | — | fits |

### 4.4 Packed SID (§7.5)

8 bytes: `book[3] | chapter:u16 | verse:u16 | delta_and_fidelity:u8` (high bit `AnchorOnly`, low
7 bits delta `0..=127`).

| field | R | S | verdict |
| --- | ---: | ---: | --- |
| `book[3]` | 67 distinct | — | fits |
| `chapter:u16` | **150** (PSA) | 65,535 (`s05`); raw 999,999 (`s07`) | **fits** — and this is exactly core's own ceiling: `saturating_u16` (`src/parse/mod.rs:551`) already clamps `Sid.chapter` to 65,535, so the wire adds **no** new loss. The unclamped value survives in the number record. |
| `verse:u16` | **176** (PSA) | 65,535 (`s08`); raw 999,999 (`s09`) | **fits**, same reasoning |
| delta `0..=127` | **71** (testData origin) | 127 (`s01`), 128 (`s02`), 255 (`s03`/`s04`) | **fits with 56 spare in real data.** Boundary is now exercised: 4 tokens at delta 127 (`Exact`), 4 at 128 and 8 at 255 (→ `AnchorOnly`). 12 synthetic tokens exceed 127; **zero** real tokens do. |
| `AnchorOnly` high bit | — | — | fits — **but see 0D D2**: the bit is not computable from `Sid` or `NumberRangeKind` alone for sequences and suffixed verses; it needs the number token's source text. |

Real bridge-delta distribution (all deltas > 0, all sets): `{1: 528, 2: 110, 3: 23, 4: 67, 5: 70,
6: 31, 71: 52}` — testData only; en_ult, en_ulb, and golden-inputs contain **no** bridge verses.

### 4.5 Finding record and sidecars (§7.6)

| field | type | R | S | sentinel | verdict |
| --- | --- | ---: | ---: | --- | --- |
| token index | `u32` | 276,987 | 301,503 | `u32::MAX` = anchor-only | **fits**; 1 real anchor-only finding (testData `origin.usfm`) |
| offset in token | `u16` | **0** | **0** | overflow sidecar when flagged | **fits, unused** — every finding starts at its token's start (0D §3.4), so no value other than 0 has ever been produced |
| length | `u16` | **35** (en_ulb 67-REV) | 70,001 (`s10`) | `0` = whole token | **fits** — every finding covers exactly one whole token, so `0` always applies; the 70,001-byte case is a whole token and needs no sidecar |
| chapter | `u16` | 150 | 65,535 | — | fits |
| verse | `u16` | 176 | 65,535 | — | fits |
| range end | `u8` | 71 | 255 | `0` = none | fits |
| **rule code** | `u8` | **32 codes** | 32 | append-only, tombstones | **fits — but the binding ceiling is 64, not 255.** See §4.7. |
| flags | `u8` | — | — | exact/anchor-only, range, related, payload, fix, overflow | fits — **but see 0D D3**: no bit distinguishes "finding has no SID at all" from the legal `BOOK 0:0` chapter anchor |
| reserved | `u8` = 0 | — | — | — | fits |
| `related_token_idx[N]` | `u32` | 262 related findings (en_ulb 23-ISA) | 0 | — | fits; `related_span` and `related_token_id` co-occur exactly in every set |
| `overflow_span[N]{u32,u32}` | — | **0** | **0** | flagged | **fits, unused — no producer exists** |
| `message_payload_idx[N]` | `u32` | max 3 params, 40 payload bytes | — | — | fits |
| `marker_string_idx[N]` | `u32` | markers ≤ 6 B real / 200 B synthetic | 200 | — | fits |
| `patch_id[N]` | `u32` | ≤ 72 fixes per set; 1 replacement token each | 2 | `u32::MAX` = no patch | fits |
| packed patch table | — | `ReplaceToken` only, 1 replacement, empty `label_params` | — | — | fits — but `DeleteToken`/`InsertAfter` have **no producer** (0D §2.2) |

### 4.6 BREACH — `sid_index: u16` has no specified overflow path

- **Field:** §7.4 `sid_index[N]:u16` with `0xffff = None` → capacity **65,535** distinct SIDs per
  book section.
- **Real max:** 2,612 (en_ult and en_ulb `19-PSA`) — 4.0% of capacity, 62,923 spare.
- **Synthetic max:** **75,301** (`s22-many-sids`, 300 chapters × 250 verses) — **exceeds capacity by
  9,766** and collides with the `0xffff` sentinel long before that.
- **Analytical bound for real scripture:** the whole Protestant canon is ~31,102 verses; the largest
  single book (Psalms) has 2,461 verses. A canonical book cannot approach 65,535 distinct anchors, so
  this is unreachable with real scripture — but §7 constrains a *decoder*, and `s22` is a
  structurally legal USFM file that braid must be able to ingest without panicking or silently
  aliasing SIDs.
- **§7 specifies no overflow behavior** for the SID dictionary — no format-version bump, no
  `DecodeError` variant, no sidecar. §7.7 covers rule-code overflow explicitly ("Crossing 255
  requires a format-version change, not truncation") and §5.6's `DecodeError` has no
  `TooManySids`-shaped variant.

**PROPOSED plan amendment (for owner sign-off, not applied):** pick one of —
(a) widen to `sid_index[N]:u32` with `u32::MAX = None` (cost: +2 B/token, ≈ +554 KB on en_ult
19-PSA, ~4.6% of that section); (b) keep `u16` and add a specified encoder error plus
`DecodeError::TooManySids { found }`, refusing to pack a book with more than 65,534 distinct SIDs;
(c) keep `u16` and specify a format-version-gated wide variant. This ledger does not choose.

### 4.7 `LintCode` capacity — core's `u64` bitmask binds before the wire's `u8`

| ceiling | value | source |
| --- | ---: | --- |
| current variant count | **32** | `golden/outputs/lint-codes.json`, `LintCode` (`src/lint_impl.rs:142-225`) |
| core `EnabledCodes` bitmask | **64** | `LintCode::bit()` = `1u64 << (self as u32)` (`src/lint_impl.rs:440`), guarded by the `enabled_codes_bitmask_matches_btreeset` test |
| wire `rule code: u8` (§7.6) | **255** | §7.6 |

So the **real growth headroom is 32 codes**, not 223: variant 65 breaks core's `u64` mask (and its
drift-guard test) long before the wire's `u8`. §7.7's "Gate 0 must prove the v1 `u8` capacity" is
satisfied — 32/255 with a specified format-version path at 255 — but the operative constraint is
core's, and §7.7 does not mention it.

**PROPOSED plan amendment (documentation only):** note in §7.7 that core's `EnabledCodes` `u64`
bitmask caps `LintCode` at 64 variants, so a wire `u8` can never be the first thing to overflow.

### 4.8 Summary verdict table

| §7 field group | verdict |
| --- | --- |
| container header, TOC | fits, large headroom |
| section header, field directory | fits (one section capped at 4 GiB by `u32` relative offsets; largest modelled section 12 MB) |
| token columns `kind`/`span_*`/`token_id_index`/`marker_descriptor_index` | fits |
| `sid_index: u16` | **breach, synthetic only, no specified overflow path** → §4.6 |
| sparse number records | fits, **must be `u32`** (raw numbers reach 999,999 while `Sid` saturates at 65,535) |
| packed SID chapter/verse `u16` | fits — identical to core's existing saturation ceiling |
| packed SID delta 7-bit | fits (real max 71); boundary now exercised synthetically |
| finding common row | fits; `offset`/`length` are always 0/whole-token — **two fields and one sidecar have no producer** |
| rule code `u8` | fits; **effective ceiling is core's 64** → §4.7 |
| finding sidecars | fits; `overflow_span` has no producer |

---

## 5. Stop conditions

| §3.1 0E stop condition | status |
| --- | --- |
| a maximum at/over a proposed field | **one — `sid_index: u16`**, reached only by synthetic `s22` (75,301 vs 65,535). No **real** corpus maximum reaches any §7 field: the closest is `marker_descriptor_index` at 221/65,535 (0.3%) and the packed-SID delta at 71/127 (56%). |
| collision with a sentinel | same field — `0xffff = None` is reachable before the width limit. Also **0D D3**: the finding row's `chapter/verse` cannot distinguish "no SID" from the legal `BOOK 0:0`. |
| an unbounded value without a sidecar/format-version path | the per-book SID dictionary is the one unbounded quantity with no specified path (§4.6). Everything else is bounded by token count (`u32`) or has a stated overflow rule. |

**Verdict: 0E passes on real corpora with wide margins; one synthetic-only breach and one sentinel
ambiguity need a plan amendment before Phase A freezes the layout.** Two proposed amendments (§4.6,
§4.7) are recorded for owner sign-off and are **not** applied.

Additionally, everything scanned is byte-lossless through `tokens_to_usfm`: **425 of 425 files**
(262 + 7 + 67 + 66 + 30 synthetic, including the pathological chapter/verse/bridge overflow and
70 KB-single-token cases). That is the §9 / §11.5 serialization baseline this gate can assert.
