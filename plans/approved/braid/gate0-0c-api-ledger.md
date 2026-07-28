# Gate 0C — public-contract and ownership census

Executes `./braid-epic.md` §3.1 0C. Evidence only; no production code, snapshots, or commits were
changed. Base commit `c22caa9` (branch `braid`), collected 2026-07-27.

Columns are the plan's exact set:
`symbol/export | current owner/file | exact Rust/TS signature | serialized shape | known consumers |
target owner | migration action | compatibility disposition | proof`

Target-owner values are the §4.1 final workspace: **core** (`usfm_onion`), **wire**
(`usfm_onion_wire`), **braid**, **wasm** (`usfm_onion_wasm`), **js** (official JS helper).
`OWNER-DECISION` marks a row the plan does not determine; those rows are restated in §8 and are
**not** decided here.

---

## 0. Headline counts

| quantity | count |
| --- | ---: |
| npm exported functions (`pkg-bundler/usfm_onion_web.d.ts`) | 24 |
| npm exported classes | 2 (`ParsedUsfm` 14 members, `UsfmMarkerCatalog` 5 members) |
| npm exported types (`interface` + `type`) | 73 |
| — of those, defined in `usfm_onion_dto` | 40 |
| — of those, defined locally in `usfm_onion_wasm` | 29 |
| — of those, hand-written TS in the `TS_TYPES` custom section | 4 |
| root `package.json` export entries | 3 (`.`, `./web`, `./token-sids`) |
| `usfm_onion_dto` public items | 43 (40 types + 3 fns) |
| — re-exported by `usfm_onion_wasm` | 42 (all but `decode_attr_value`) |
| `usfm_onion` (core) public declarations (struct/enum/trait/type/fn/const, excl. tests/bins) | 423 |
| core token traits crossing into lint/format/diff | 6 (`UsfmToken`, `WalkableToken`, `LintableToken`, `FormattableToken`, `DiffableToken`, `SerializableToken` + `SerializableAttribute`) |
| `LintCode` variants | 32 |
| in-repo crates depending on `usfm_onion_dto` | 1 (`usfm_onion_wasm`) |

Disposition tallies (§7).

---

## 1. Declaration regeneration — the 0A/0B carry-forward, resolved

The 2026-07-27 adjudication required Gate 0C to regenerate declarations rather than trust the
committed trees. Done, to scratch only:

```bash
wasm-pack build crates/usfm_onion_wasm --target bundler --dev \
  --out-dir <scratch>/gate0c-pkg-bundler --out-name usfm_onion_web
wasm-pack build crates/usfm_onion_wasm --target web --dev \
  --out-dir <scratch>/gate0c-pkg-web --out-name usfm_onion_web
```

Flags match `npm run build:wasm:*:dev` (`scripts/build-wasm.mjs:43-54`) with only `--out-dir`
redirected. `scripts/restore-wasm-package-layout.mjs` was deliberately **not** run — it writes
`.gitignore` into the tracked `pkg-bundler`/`pkg-web` directories and has no bearing on
declarations. Both builds exited 0.

### Classification of every regenerated-vs-committed difference

| artifact | result | classification |
| --- | --- | --- |
| `pkg-bundler/usfm_onion_web.d.ts` | **byte-identical** (`diff` exit 0) | no difference |
| `pkg-web/usfm_onion_web.d.ts` | differs **only** inside the `InitOutput` interface body (lines 520-571) | bindgen-ABI-shape-only |
| `pkg-bundler/usfm_onion_web_bg.wasm.d.ts` | differs throughout | bindgen-ABI-shape-only |
| `pkg-web/usfm_onion_web_bg.wasm.d.ts` | differs throughout | bindgen-ABI-shape-only |
| `pkg-bundler/package.json`, `pkg-web/package.json` | byte-identical | no difference |

The ABI delta is a wasm-bindgen version change in calling convention, exactly as the 0A/0B
adjudication suspected: output-pointer style (`(a,b,c,d) => void`) becomes multi-value/externref
style (`(a,b,c) => [number, number]`, `=> any`), and the raw export names change
(`__wbindgen_export{,2,3,4}` + `__wbindgen_add_to_stack_pointer` → `__wbindgen_malloc`,
`__wbindgen_realloc`, `__wbindgen_free`, `__wbindgen_exn_store`, `__externref_table_alloc`,
`__externref_table_dealloc`, `__externref_drop_slice`, `__wbindgen_externrefs`,
`__wbindgen_start`).

**Finding (not a fix):** zero public-API-surface differences. Every user-facing function, class,
interface, type, and `package.json` export map regenerates identically under the local toolchain.
The committed trees are therefore trustworthy *as an API contract* for this census, and the earlier
0B "materially different bindgen ABI shape" observation is confined to internal plumbing.

One caveat worth recording: `InitOutput` **is** an exported TypeScript type in the `pkg-web`
declarations, so a toolchain bump does change one named public type's member list even though every
member is internal plumbing. Row in §4.

### Bundler vs web: one schema source

`diff` of the two committed public `.d.ts` files over lines 1-515 is empty. `pkg-web` then adds only
loader glue (`InitInput`, `InitOutput`, `SyncInitInput`, `initSync`, `__wbg_init` default export).
Both targets are produced from the same crate by the same script with only `--target` differing,
and both are verified against the **same** golden directory (`scripts/wasm-golden.mjs:24`
resolves `outputs/` independent of target). See the 0G ledger for the drift-check row.

---

## 2. Core (`usfm_onion`) — semantic tokens, spans, ids, attributes, marker metadata

All rows: target owner **core**, action **retain in core, unchanged**, disposition **retain**,
unless stated. These are Rust-only (`crate-type = ["rlib"]`); they are *not* npm exports, so the
"serialized shape" column records the `Serialize` derive where present (core derives `Serialize`
one-way only, never `Deserialize`).

| symbol | owner/file | exact signature | serialized shape | known consumers | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `Token<'a>` | `src/token.rs:324` | `struct Token<'a> { id: TokenId<'a>, sid: Option<Sid>, span: Span, source: &'a str, data: TokenData<'a> }` | `Serialize`, `data` `#[serde(flatten)]` | lint, format, diff, cst, usj, usx, html, vref, wasm, dto | core | retain | retain | src |
| `TokenData<'a>` | `src/token.rs:280` | 9-variant enum: `Newline`, `OptBreak`, `Marker{name,metadata,structural,nested,attrs}`, `EndMarker{name,metadata,structural,nested}`, `Milestone{name,metadata,structural,attrs}`, `MilestoneEnd`, `BookCode{code,is_valid}`, `Number{start:u32,end:Option<u32>,kind}`, `Text` | `Serialize`, `#[serde(tag="type")]`, `attrs` flattened | as above | core | retain | retain | src |
| `TokenKind` | `src/token.rs:237` | 9-variant `Copy` enum | `Serialize` (PascalCase natively; camelCase on the wire via the dto mirror) | everything | core | retain | retain | src |
| `TokenId<'a>` | `src/token.rs:874` | `struct TokenId<'a> { book_code: &'a str, index: u32 }` | `Serialize` | parse, diff, dto | core | retain; **new** `StableTokenId` added alongside (§5.1) | retain | src |
| `Span` / `BytePos` | `src/token.rs:10`, `:7` | `struct Span { start: BytePos, end: BytePos }`, `type BytePos = u32` | `Serialize` | everywhere | core | retain | retain | src |
| `AttributeItem<'a>` | `src/token.rs:250` | `{ span: Span, source: &'a str, key: &'a str, value: &'a str, is_default: bool }` | `Serialize` | token emit, dto | core | retain | retain | src |
| `MarkerAttrs<'a>` | `src/token.rs:265` | `{ attributes: Vec<AttributeItem<'a>>, attribute_source: Option<(Span, &'a str)> }` | `Serialize`, both fields `skip_serializing_if` | `TokenData::{Marker,Milestone}` | core | retain | retain | src |
| `MarkerMetadata` | `src/token.rs:50` | `{ canonical, kind, family }` (all optional) | `Serialize` | lint, dto | core | retain | retain | src |
| `Sid` | `src/token.rs:809` | `{ book: BookId, chapter: u16, verse: u16, verse_end_delta: u8 /*private*/ }`, 8 bytes, `Copy` | `Serialize` — `verse_end_delta` never leaks; `verse_end()`/`verse_locator()` are the accessors | diff, lint, dto, vref | core | retain unchanged (§2.2#6: no core SID change in wire work) | retain | src + `sid_size_guard` test `src/token.rs:896` |
| `BookId` | `src/token.rs:758` | `struct BookId([u8;3])`, `from_str -> Option<Self>`, `as_str`, `UNKNOWN` sentinel | `Serialize` as a 3-char string | `Sid`, vref | core | **OWNER-DECISION** (§8.1): is braid's declared `BookId` this type? | retain | src |
| `NumberRangeKind` | `src/token.rs:92` | 4-variant enum | `Serialize` | number tokens, dto | core | retain | retain | src |
| `BookCodeToken<'a>`, `MarkerToken<'a>`, `AttributeEntryToken<'a>`, `NumberRangeToken<'a>`, `TriviaToken<'a>`, `ScanToken<'a>`, `ScanTokenKind`, `ScanResult<'a>`, `Lexeme`/`LexemeKind`/`LexResult` aliases | `src/token.rs:44-234` | lexer-stage types; 11 items | `Serialize` where derived | lexer, parse | core | retain | retain | src |
| `ParseResult<'a>`, `ParseAnalysis<'a>` | `src/token.rs:886-894` | `{ tokens: Vec<Token<'a>>, analysis }`, `{ book_code: Option<&'a str> }` | `Serialize` | parse, api, wasm | core | retain | retain | src |
| `tokens_to_usfm(&[Token]) -> String` | `src/token.rs:403` | span-drain emitter | n/a | api, cst | core | retain | retain | src |
| `tokens_to_usfm_reconstruct<T: SerializableToken>(&[T]) -> String` | `src/token.rs:710` | spanless emitter | n/a | wasm `tokensToUsfm`/`tokensToHtml` | core | retain | retain | src; parity test `src/token.rs:1003` |
| `format_attribute_list<A: SerializableAttribute>`, `encode_attr_value`, `strip_marker_backslash`, `strip_closing_star`, `marker_text_name`, `marker_metadata` | `src/token.rs:201-596` | 6 free fns | n/a | emit path, dto | core | retain | retain | src |
| marker registry: `marker_catalog()`, `marker_info()`, `is_known_marker()`, `UsfmMarkerCatalog`, `UsfmMarkerInfo`, `MarkerKind`, `MarkerCategory`, `lookup_marker` | `src/markers.rs` | 8 public items | `Serialize` | wasm re-export, dto | core | retain (§2.1#9: core-owned, adapters re-export, never copy) | retain | src |
| marker definitions: `MarkerId` + 25 `MARKER_*` consts, `MarkerDefKind`, `SpecContext`, `MarkerFamily`, `MarkerFamilyRole`, `NoteFamily`, `NoteSubkind`, `InlineContext`, `BlockBehavior`, `StructuralScopeKind`, `StructuralMarkerInfo`, `ClosingBehavior`, `ParagraphCategory`, `MarkerSpec`, `NormalizedMarkerRef<'a>`, `MarkerWhitespace`, `MarkerDef`, `MarkerPayload`, + 22 lookup fns | `src/marker_defs.rs` | 65 public items | `Serialize` on the enums | lint, format, walker, dto | core | retain | retain | src (`target/braid-gate0/core-pub.txt`) |
| whitespace: `StructuralWhitespaceRequirement`, `FormatWhitespacePreference`, `WhitespaceFormatCategory` + 7 predicates | `src/whitespace.rs` | 10 public items | `Serialize` on enums | lint, format | core | retain | retain | src |

### 2.1 Token traits used by lint/format/diff

| trait | owner/file | exact signature | implementors today | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `UsfmToken` | `src/token.rs:526` | `fn kind(&self)->TokenKind; fn source(&self)->&str; fn marker(&self)->Option<&str>` | `Token<'a>`, `FormatToken`, `dto::Token`, `wasm::WalkToken` | core | retain; **new** `OwnedToken` implements it | retain | src |
| `SerializableAttribute` | `src/token.rs:447` | `key/value/is_default` | `AttributeItem<'a>`, `dto::AttributeItem` | core | retain | retain | src |
| `SerializableToken: UsfmToken` | `src/token.rs:532` | `type Attr: SerializableAttribute; fn attributes(&self)->&[Self::Attr]; fn attribute_list(&self)->Option<&str>` | `Token<'a>`, `dto::Token` | core | retain; `OwnedToken` must implement it | retain | src; landed 2026-07-24 |
| `WalkableToken: UsfmToken` | `src/walker/mod.rs:62` | `fn structural(&self)->Option<StructuralMarkerInfo>; fn next_is_number(&self)->bool = false` | `Token<'a>`, `FormatToken`, `wasm::WalkToken` | core | retain | retain | src |
| `LintableToken: WalkableToken` | `src/lint_impl.rs:28` | `fn span(&self)->Option<Span>=None; fn sid(&self)->Option<String>=None; fn id(&self)->Option<String>=None; fn number_info(&self)->Option<(u32,Option<u32>,NumberRangeKind)>=None; fn allows_effective_context(&self,SpecContext)->bool` | `Token<'a>`, `FormatToken`, `wasm::WalkToken` | core | retain | retain | src |
| `FormattableToken: Clone` | `src/format/mod.rs:449` | 14 methods incl. mutators `set_id/set_kind/set_text/set_marker/set_sid`, `marker_profile`, `synthetic_like`. **No attribute accessor of any kind.** | `FormatToken` | core | retain — but see §6 finding F1 | retain | src |
| `DiffableToken: Clone` | `src/diff/mod.rs:55` | `sid/sid_string/sid_key/text/id/id_string/kind_key/marker_key/number_range/book_code` (all but `text` defaulted) | `Token<'a>`, `FormatToken`, `wasm::WalkToken` | core | retain | retain | src |

### 2.2 Lint surface

| symbol | owner/file | signature | serialized shape | consumers | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `LintCode` | `src/lint_impl.rs:142` | 32-variant enum + `code()`, `template()`, `category()`, `severity()`, `issue_type()`, `bit()` | `Serialize`+`Deserialize`, `rename_all="kebab-case"` | lint, dto mirror, npm | core | retain; wire adds explicit stable integer codes (§7.7) — **not** enum ordinals | retain | src; 32 codes in `golden/outputs/lint-codes.json` |
| `LintCategory` / `LintSeverity` / `LintIssueType` | `src/lint_impl.rs:119/128/135` | 4 / 2 / 2 variants | `Serialize`+`Deserialize`, kebab-case | as above | core | retain | retain | src |
| `LintIssue` | `src/lint_impl.rs:445` | `{ code, category, severity, issue_type, template: &'static str, message: String, message_params: MessageParams, span: Option<Span>, related_span: Option<Span>, token_id: Option<String>, related_token_id: Option<String>, sid: Option<String>, marker: Option<String>, fix: Option<TokenFix> }` | `Serialize` only | lint, wasm mirror | core | retain | retain | src |
| `LintSummary` | `src/lint_impl.rs:472` | `{ by_category: BTreeMap<LintCategory,usize>, by_severity, by_issue_type, total_count, suppressed_count }` | `Serialize`, `Default` | lint | core | retain | retain | src |
| `LintResult` | `src/lint_impl.rs:481` | `{ issues: Vec<LintIssue>, summary: LintSummary }` | `Serialize` | lint, braid `BookLintSnapshot` | core | retain | retain | src |
| `TokenFix` | `src/lint_impl.rs:491` | 3 variants `ReplaceToken{code,label,label_params,target_token_id,replacements:Vec<TokenTemplate>}`, `DeleteToken{..}`, `InsertAfter{..,insert:Vec<TokenTemplate>}` + `target_token_id()` | `Serialize`, externally tagged natively | lint, wasm mirror (internally tagged there) | core | retain; braid resolves it into `TokenPatch` (§5.3) | retain | src |
| `LintOptions` | `src/lint_impl.rs:549` | `{ scope: LintScope, enabled_codes: Option<Vec<LintCode>>, disabled_codes: Vec<LintCode>, suppressed: Vec<LintSuppression>, allow_implicit_chapter_content_verse: bool }` + `scoped(scope)`; **no `Default`** | `Serialize`+`Deserialize` | lint, `BraidConfig` | core | retain | retain | src |
| `LintScope` | `src/lint_impl.rs:528` | `Front | Chapter(u32) | Book` | `Serialize`+`Deserialize` (externally tagged) | lint | core | retain | retain | src |
| `LintSuppression` | `src/lint_impl.rs:515` | `{ code: LintCode, sid: String }` | `Serialize`+`Deserialize` | lint options | core | retain | retain | src |
| `MessageParams` | `src/lint_impl/message.rs:3` | `type MessageParams = BTreeMap<String,String>` | map | lint, format (own alias at `src/format/mod.rs:24`) | core | retain (two aliases for one type — cosmetic duplication, not drift) | retain | src |
| `lint_usfm`, `lint_tokens<T: LintableToken + Sync>`, `apply_token_fix<T: FormattableToken>`, `render_template` | `src/lint_impl.rs:801/806/1036`, `message.rs:22` | 4 fns | n/a | api, wasm, braid | core | retain; braid calls `lint_tokens(.., LintScope::Book)` per dirty book (§6.4) | retain | src |
| canonical finding sort | `src/lint_impl.rs` (private) | not a public symbol | n/a | — | core | retain; 0D must pin the key and prove JS can reproduce it | retain | plan §3.1 0D#3 |

### 2.3 Diff, format, and projection surface

| symbol | owner/file | signature | consumers | target | action | disposition |
| --- | --- | --- | --- | --- | --- | --- |
| `DiffSkeleton<T>` | `src/diff/skeleton.rs:137` | `{ slots: Vec<Slot>, units: Vec<DecisionUnit<T>> }` | diff, api, wasm | core | retain; braid returns `DiffSkeleton<OwnedToken>` (§5.3) | retain |
| `DecisionUnit<T>` | `src/diff/skeleton.rs:120` | 14 fields incl. `baseline_tokens/current_tokens: Vec<T>`, `dup_context`, `covered_by`, `text_diff` | as above | core | retain | retain |
| `Slot`, `Anchor`, `UnitId`, `SlotRole`, `DecisionUnitKind`, `DecisionStatus`, `DupContext`, `CoveredBy`, `CoveredSide`, `MergeSide`, `MergeError`, `SidBlock`, `SidKey<'a>` | `src/diff/skeleton.rs`, `src/diff/mod.rs:14/226` | 13 items | as above | core | retain | retain |
| `diff_skeleton`, `diff_skeleton_canonical`, `diff_skeleton_by_chapter`, `diff_skeleton_by_chapter_from_tokens`, `merge_skeleton`, `merge_diff_blocks`, `revert_diff_block`, `derive_canonical_sids`, `build_sid_blocks`, `build_sid_blocks_canonical` | `src/diff/*` | 10 fns | api, wasm, js helper | core | retain | retain |
| `TextDiffMode`, `TextDiffRun`, `TextDiffRunKind`, `UnitTextDiff`, `unit_text_diff`, `diff_skeleton_with_text` | `src/diff/text_diff.rs` | 6 items | diff, wasm | core | retain | retain |
| `FormatToken` | `src/format/mod.rs:396` | `{ kind, text: String, marker, sid, id, span, structural, number_info, marker_profile }` — **9 fields, none for attributes** | format, lint, diff, wasm | core | retain; see finding F1 | retain |
| `FormatOptions` (15 rule flags) + `FormatRule` (15 variants, `ALL`, `code()`, `label_key()`), `FormatProfile`, `FormatFix`, `FormatLabel`, `FormatTimings`, `FormatMarkerProfile`, `LinebreakBehavior`, `TokenTemplate` | `src/format/mod.rs` | 10 items | format, wasm, braid `prepare_format_patch` | core | retain | retain |
| `format`, `format_mut`, `format_mut_default`, `format_tokens`, `format_tokens_profile`, `format_usfm`, `into_format_tokens`, `format_tokens_to_usfm` | `src/format/mod.rs` | 8 fns | api, wasm | core | retain | retain |
| `CstDocument<'a>`, `CstNode`, `CstWalkIter`, `WalkItem`, `build_cst`, `build_cst_roots`, `parse_cst`, `cst_to_tokens`, `cst_to_usfm` | `src/cst/mod.rs` | 9 items | api, usj, usx, wasm | core | retain | retain |
| `UsjDocument`, `UsjNode`, `UsjElement`, `UsjError`, `usfm_to_usj`, `cst_to_usj`, `from_usj`, `from_usj_str`, `collect_usj_fixture_pairs` | `src/usj/mod.rs` | 9 items | api, usx, wasm | core | retain | retain |
| `UsxError`, `usfm_to_usx`, `cst_to_usx`, `usj_to_usx`, `usx_to_usj`, `from_usx_str` | `src/usx.rs` | 6 items | api, wasm | core | retain | retain |
| `HtmlOptions`, `HtmlNoteMode`, `HtmlCallerStyle`, `HtmlCallerScope`, `tokens_to_html`, `usfm_to_html` | `src/html.rs` | 6 items | api, wasm | core | retain | retain |
| `VrefMap`, `VrefIndex`, `VerseProjection`, `Segment`, `Utf16Span`, `VrefOptions`, `usfm_to_vref_map(+_with_options)`, `tokens_to_vref_map(+_with_options)`, `usfm_to_vref_index`, `tokens_to_vref_index`, `vref_map_to_json_string` | `src/vref.rs` | 11 items | api, wasm | core | retain | retain |
| `walker`: `Visitor`, `WalkContext`, `ScopeFrame`, `LeaveReason`, `WalkBoundary`, `ChapterSegment`, `walk`, `walk_tokens`, `walk_range`, `chapter_segments` | `src/walker/mod.rs` | 10 items | lint, format, vref | core | retain | retain |
| `api`: `Usfm`, `ParsedUsfm`, `OwnedParseAnalysis`, `Usj`, `Usx`, `TokenStream<T>`, `SourceTokenText`, 5 diff builders + ~60 methods | `src/api.rs` | 12 types | native Rust callers | core | retain (native one-stop API; braid is the *resident* peer, not a replacement) | retain |
| `convert` re-export module (9 fns), `par::map_ordered`, `lexer::lex`, `parse::parse`, `parse::parse_lexemes`, `parse::into_usfm_from_tokens` | `src/convert/mod.rs`, `src/par.rs`, `src/lexer.rs`, `src/parse/mod.rs` | 14 items | api, wasm | core | retain | retain |

**Grouping disclosure:** rows above marked with an item count group symbols whose owner, target
owner, action, and disposition are identical and whose signatures are unchanged by this epic. The
full enumerated list is retained at `target/braid-gate0/core-pub.txt` (423 lines, generated by
`grep -nE '^\s*pub (struct|enum|trait|type|fn|const|static|mod|use)' src/*.rs src/*/*.rs`
excluding `src/bin/`, `*_fixtures.rs`, `*_proptest.rs`). No core symbol is silently omitted: every
line in that file falls under exactly one row above.

---

## 3. `usfm_onion_dto` — 43 public items, all moving to wire

Every item here is `pub`-fielded (usable from native Rust/Tauri), derives
`Serialize + Deserialize`, and gains `tsify::Tsify` + `wasm_bindgen` only under `feature = "wasm"`.
Target owner for **all 43**: **wire**. Migration action for all: *move into
`usfm_onion_wire::dto`, declarations unchanged* (Phase A step 1). Disposition for all: **retain**
(the type survives; only its crate path changes, and the npm-visible declaration does not change at
all). Proof: `crates/usfm_onion_dto/src/lib.rs` at the cited line; npm visibility per
`pkg-bundler/usfm_onion_web.d.ts`.

| # | symbol | line | Rust shape | wire/TS shape | npm-visible |
| ---: | --- | ---: | --- | --- | --- |
| 1 | `SpecContext` | 72 | 20-variant enum | camelCase union | yes |
| 2 | `InlineContext` | 157 | 4 variants | lowercase union | yes |
| 3 | `BlockBehavior` | 190 | 6 variants | camelCase | yes |
| 4 | `ClosingBehavior` | 216 | 4 variants | camelCase | yes |
| 5 | `MarkerPayload` | 245 | 2 variants | camelCase | yes |
| 6 | `MarkerCategory` | 263 | 18 variants | camelCase | yes |
| 7 | `MarkerKind` | 313 | 16 variants | camelCase | yes |
| 8 | `MarkerFamily` | 359 | 7 variants | camelCase | yes |
| 9 | `MarkerFamilyRole` | 387 | 5 variants | camelCase | yes |
| 10 | `ParagraphCategory` | 413 | 10 variants | camelCase | yes |
| 11 | `NoteFamily` | 447 | 2 variants | camelCase | yes |
| 12 | `NoteSubkind` | 465 | 2 variants | camelCase | yes |
| 13 | `MarkerInfo` | 485 | 16 fields, 12 optional | camelCase interface | yes |
| 14 | `TokenKind` | 559 | 9 variants | camelCase | yes |
| 15 | `NumberRangeKind` | 607 | 4 variants | camelCase | yes |
| 16 | `StructuralScopeKind` | 640 | 13 variants | camelCase | yes |
| 17 | `MarkerDefKind` | 703 | 13 variants | camelCase | yes |
| 18 | `Span` | 743 | `{start:u32,end:u32}` | `{start:number,end:number}` | yes |
| 19 | `MarkerMetadata` | 770 | `{canonical,kind,family}` all optional | camelCase | yes |
| 20 | `AttributeItem` | 793 | `{span,text,key,value,is_default}` | camelCase; note **`source` → `text`** rename vs native | yes |
| 21 | `StructuralMarkerInfo` | 818 | `{scope_kind,inline_context,note_context}` | camelCase | yes |
| 22 | `NumberInfo` | 840 | `{start,end,kind}` | camelCase | yes |
| 23 | `Token` | 851 | 14 fields (see §3.1) | camelCase, single tagged optional-field shape | yes |
| 24 | `LintCategory` | 1057 | 4 variants | kebab-case | yes |
| 25 | `LintSeverity` | 1079 | 2 variants | kebab-case | yes |
| 26 | `LintIssueType` | 1097 | 2 variants | kebab-case | yes |
| 27 | `LintCode` | 1115 | 32 variants | kebab-case | yes |
| 28 | `SlotRole` | 1253 | 5 variants | camelCase | yes |
| 29 | `DecisionUnitKind` | 1277 | 4 variants | camelCase | yes |
| 30 | `DecisionStatus` | 1299 | 5 variants | camelCase | yes |
| 31 | `MergeSide` | 1323 | 2 variants | camelCase | yes |
| 32 | `CoveredSide` | 1350 | 2 variants | camelCase | yes |
| 33 | `TextDiffMode` | 1377 | 3 variants, `Default` | camelCase | yes |
| 34 | `TextDiffRunKind` | 1398 | 3 variants | camelCase | yes |
| 35 | `TextDiffRun` | 1418 | `{text,kind}` | camelCase | yes |
| 36 | `UnitTextDiff` | 1436 | `{baseline,current}` | camelCase | yes |
| 37 | `DiffOptions` | 1458 | `{text_diff: Option<TextDiffMode>}` | camelCase | yes |
| 38 | `HtmlNoteMode` | 1479 | 2 variants | camelCase | yes |
| 39 | `HtmlCallerStyle` | 1497 | 6 variants | camelCase | yes |
| 40 | `HtmlCallerScope` | 1523 | 2 variants | camelCase | yes |
| 41 | `map_marker_info(NativeUsfmMarkerInfo) -> MarkerInfo` | 543 | fn | n/a | no (Rust only) |
| 42 | `format_sid(NativeSid) -> String` | 1015 | fn | n/a | no (Rust only) |
| 43 | `decode_attr_value(&str) -> String` | 1023 | fn | n/a | no — **and not re-exported by `usfm_onion_wasm`** |

Row 43 is the single `usfm_onion_dto` public item with no `usfm_onion_wasm` re-export
(`crates/usfm_onion_wasm/src/lib.rs:48-56` lists 42 of 43). It has no in-repo caller outside the dto
crate itself. Disposition **retain** (it is the documented inverse of core `encode_attr_value`);
flagged so the absorption commit does not lose it by following the re-export list alone.

### 3.1 `usfm_onion_dto::Token` — the de-facto `OwnedToken` today

```rust
pub struct Token {
    pub id: String,                                  // "{book}-{index}", flattened from TokenId
    pub kind: TokenKind,
    pub source: String,
    pub span: Option<Span>,
    pub sid: Option<String>,                         // formatted "GEN 1:1", NOT the 8-byte Sid
    pub marker: Option<String>,
    pub nested: Option<bool>,
    pub marker_metadata: Option<MarkerMetadata>,
    pub structural: Option<StructuralMarkerInfo>,
    pub number_info: Option<NumberInfo>,
    pub book_code: Option<String>,
    pub book_code_valid: Option<bool>,
    pub attributes: Vec<AttributeItem>,
    pub attribute_source: Option<String>,
}
```

Relation to plan §5.1 `OwnedToken`: this type already carries every field §5.1 names, **plus**
`span`, `marker_metadata`, and `structural`. `structural` is not optional in practice — core
`WalkableToken::structural()` drives the lint scope machine, and `token_to_walk_token`
(`crates/usfm_onion_wasm/src/lib.rs:1080`) passes it straight through with no re-derivation
fallback. §5.1's note that "the production type may retain additional marker structural metadata if
equivalence proves it is required" is therefore already answered *yes* by the live code for
`structural`; `span` and `marker_metadata` remain open. Recorded here as evidence for Phase A, not
decided.

---

## 4. `usfm_onion_wasm` — 29 local DTO types, 2 classes, 24 exports

### 4.1 Local boundary DTO types (target owner: **wire**)

All 29 derive `Serialize + Deserialize + Tsify` with `#[tsify(into_wasm_abi, from_wasm_abi)]` and
`#[serde(rename_all = "camelCase")]`. **All 29 have private fields** — unlike `usfm_onion_dto`'s
`pub`-fielded types, they are unusable from a native Rust/Tauri consumer and exist only for the
tsify/serde boundary. Migration action for all: *move into `usfm_onion_wire` and make fields `pub`
so the native boundary can use them too* (§4.1 assigns wire both boundaries). Disposition for all:
**retain** (npm declaration unchanged). Proof: `crates/usfm_onion_wasm/src/lib.rs` at the cited
line, plus the matching `export interface`/`export type` in `pkg-bundler/usfm_onion_web.d.ts`.

| # | symbol | line | shape | note |
| ---: | --- | ---: | --- | --- |
| 1 | `DiffsByChapterMap` | 114 | `#[serde(transparent)]` over `BTreeMap<String, BTreeMap<u32, DiffSkeleton>>` | TS `Record<string, Record<number, DiffSkeleton>>` |
| 2 | `VrefMap` | 122 | transparent `BTreeMap<String,String>` | mirrors core `vref::VrefMap` |
| 3 | `VrefOptions` | 127 | `{trim: Option<bool>}` | |
| 4 | `Utf16Span` | 138 | `{start,end}` | deliberately distinct from `Span` |
| 5 | `Segment` | 149 | `{token_id, source_span, text_span}` | |
| 6 | `VerseProjection` | 160 | `{text, segments}` | |
| 7 | `VrefIndex` | 170 | transparent `BTreeMap<String, VerseProjection>` | |
| 8 | `CstNode` | 175 | `{token_index, children}` | |
| 9 | `CstDocument` | 183 | `{tokens: Vec<Token>, roots}` | |
| 10 | `LintSuppression` | 191 | `{code, sid}` | mirrors core |
| 11 | `LintScope` | 202 | `Front | Chapter(u32) | Book` → TS `"front" | {chapter:number} | "book"` | wire casing differs from core's `Serialize` |
| 12 | `LintOptions` | 211 | 5 fields, 4 `#[serde(default)]` | `scope` required by design |
| 13 | `LintIssue` | 228 | 14 fields | `template: String` — core's `&'static str` **lifetime erased + allocated** |
| 14 | `LintSummary` | 255 | 5 fields | |
| 15 | `LintResult` | 266 | `{issues, summary}` | |
| 16 | `TokenFix` | 278 | `#[serde(tag="type", rename_all="camelCase", rename_all_fields="camelCase")]`, 3 variants | **internally tagged** — core is externally tagged; a genuine wire-vs-native shape translation |
| 17 | `TokenTemplate` | 304 | `{kind,text,marker,sid}` | |
| 18 | `FormatOptions` | 316 | 15 `Option<bool>` flags | native `FormatOptions` is 15 plain `bool`s; `None` = leave default (`apply_opt`, line 1489) |
| 19 | `FormatResult` | 352 | `{tokens: Vec<Token>, usfm: String}` | see finding F1 |
| 20 | `HtmlOptions` | 360 | `{wrap_root: bool, 4 × Option<..>}` | |
| 21 | `Anchor` | 376 | `{unit_id, sid}` | `UnitId` newtype flattened to `String` |
| 22 | `Slot` | 384 | `{unit_id, role, after}` | |
| 23 | `DupContext` | 394 | `{baseline_count, current_count}` | |
| 24 | `CoveredBy` | 402 | `{unit_id, sid, side}` | |
| 25 | `DecisionUnit` | 411 | 14 fields | `DiffSkeleton<T>`'s generic erased to the concrete wire `Token` |
| 26 | `DiffSkeleton` | 435 | `{slots, units}` | generic erased |
| 27 | `MergeRequest` | 445 | `{decisions: BTreeMap<String,MergeSide>, default_side}` | wasm-only input type; no native counterpart |
| 28 | `LintCodeMeta` | 453 | `{code, category, severity, issue_type}` | wasm-only projection |
| 29 | `FormatRuleMeta` | 463 | `{code, label_key}` | wasm-only projection |

### 4.2 Hand-written TS in the `typescript_custom_section` (target owner: **OWNER-DECISION**)

| symbol | owner/file | TS signature | serialized shape | consumers | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `Value` | `crates/usfm_onion_wasm/src/lib.rs:68` | recursive JSON value union | n/a (type-only) | `UsjElement` | OWNER-DECISION §8.2 | none / promote to tsify | retain | src + d.ts:11 |
| `UsjDocument` | `:76` | `{type,version,content}` | `serde_json` via `to_js_value` | `toUsj()` | OWNER-DECISION §8.2 | as above | retain | d.ts:19 |
| `UsjNode` | `:82` | `string | UsjElement` | as above | as above | OWNER-DECISION §8.2 | as above | retain | d.ts:25 |
| `UsjElement` | `:84` | 17-arm discriminated union | as above | as above | OWNER-DECISION §8.2 | as above | retain | d.ts:27 |

These four are the only npm types **not** derived from a Rust type. `ParsedUsfm.toUsj()` is
correspondingly typed `any` in the generated declarations (`d.ts:448`), so the declared
`UsjDocument` type is not actually connected to the function that produces it. The in-source comment
(`lib.rs:58-66`) calls tsify-ing it "follow-up work"; the epic does not assign it.

### 4.3 Private wasm types that the plan promotes

| symbol | owner/file | signature | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- |
| `WalkToken` (private) | `crates/usfm_onion_wasm/src/lib.rs:478` | `{id: String, kind, text, span, sid, marker, structural, number_info}` — 8 fields; implements `UsfmToken`, `WalkableToken`, `LintableToken`, `DiffableToken` with **native** enums | core (becomes/merges into `OwnedToken`) | replace with core `OwnedToken`; delete the local type | replace | src:468-547 |
| `token_to_walk_token`, `token_value_to_format_token`, `parse_walk_tokens_from_values`, `format_token_with_identity`, `map_token`, `map_format_token`, `map_walk_token` | `:1072-1202`, `:1393` | 7 owned↔owned conversions between three owned token shapes (`dto::Token`, `WalkToken`, `FormatToken`) | wire (DTO↔`OwnedToken`) + core (`OwnedToken::from_parsed`) | collapse to the two conversions in §5.1 | replace | src |

`WalkToken` is the strongest live evidence for §5.1: an owned token satisfying three core traits
already exists and works. It is however **lint/diff-only** — it carries no `attributes`,
`attribute_source`, `book_code`, `nested`, or `marker_metadata`, so it is not a `SerializableToken`.
`OwnedToken` must be the union of `WalkToken`'s trait coverage and `dto::Token`'s payload.

### 4.4 The two npm classes

| export | owner/file | TS signature | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- |
| `class ParsedUsfm` | `:550`, `impl` at `:558` | `private constructor(); free(); [Symbol.dispose]();` + `applyTokenFix(fix): Token[]`, `cst(): CstDocument`, `diff(other, options?): DiffSkeleton`, `diffByChapter(other, options?): DiffsByChapterMap`, `format(options?): string`, `lint(options): LintResult`, `revertDiffBlock(current, block_id): Token[]`, `toHtml(options?): string`, `toUsfm(): string`, `toUsj(): any`, `toUsx(): string`, `toVref(options?): VrefMap`, `tokens(): Token[]`, `vrefIndex(): VrefIndex` | wasm | retain through the migration release (§8.2: "existing stateless exports remain unless the ledger marks one for deletion") | **retain** | d.ts:435-453 |
| `class UsfmMarkerCatalog` | `:555`, `impl` at `:674` | `private constructor(); free(); [Symbol.dispose](); all(): MarkerInfo[]; contains(marker): boolean; get(marker): MarkerInfo | undefined` | wasm | retain (core registry re-export) | **retain** | d.ts:455-462 |

`ParsedUsfm.toUsx(): string` and `toUsj(): any` are the only two methods that can throw
(`Result<_, JsError>` in Rust, `:622`/`:628`); every other member is infallible. Under §2.2#11
future braid methods use tagged `ApiResult`, but these two are pre-existing and unchanged.

### 4.5 The 24 npm free functions

Target owner **wasm** and disposition **retain** for all 24 (§8.2). Rust `js_name` mapping is
`wasm_*` → camelCase. Action column records only where the plan changes something.

| # | export | TS signature | Rust fn | routing / lossiness | action |
| ---: | --- | --- | --- | --- | --- |
| 1 | `parse` | `(source: string) => ParsedUsfm` | `wasm_parse` `:698` | native parse, borrowed→owned `FormatToken` | none |
| 2 | `lintUsfm` | `(source, options: LintOptions) => LintResult` | `:703` | native path | none |
| 3 | `lintTokens` | `(tokens: Token[], options) => LintResult` | `:722` | `Token`→`WalkToken`→`lint_tokens` | none |
| 4 | `applyTokenFix` | `(tokens, fix) => Token[]` | `:728` | `Token`→`FormatToken`→`Token` — **F1 loss** | see F1 |
| 5 | `formatUsfm` | `(source, options?) => string` | `:738` | native, string-in/string-out | none |
| 6 | `formatTokens` | `(tokens, options?) => FormatResult` | `:743` | `Token`→`FormatToken`→`Token` — **F1 loss** | see F1 |
| 7 | `formatTokensMut` | `(tokens, options?) => Token[]` | `:756` | same — **F1 loss** | see F1 |
| 8 | `tokensToUsfm` | `(tokens) => string` | `:766` | `tokens_to_usfm_reconstruct` via `SerializableToken` — **lossless** | none |
| 9 | `tokensToHtml` | `(tokens, options?) => string` | `:771` | reconstruct→`usfm_to_html` — lossless input | none |
| 10 | `diffUsfm` | `(left, right, options?) => DiffSkeleton` | `:793` | native tokens via `map_token` — full payload | none |
| 11 | `diffUsfmByChapter` | `(left, right, options?) => DiffsByChapterMap` | `:802` | as above | none |
| 12 | `diffTokens` | `(left, right, options?) => DiffSkeleton` | `:815` | `WalkToken` via `map_walk_token` — **F1 loss** in unit tokens | see F1 |
| 13 | `mergeDiffBlocks` | `(baseline, current, request) => Token[]` | `:830` | `FormatToken` — **F1 loss**; throws `JsError` on unknown unit | see F1 |
| 14 | `revertDiffBlock` | `(baseline, current, block_id) => Token[]` | `:855` | same | see F1 |
| 15 | `normalizeTokenSids` | `(tokens, book_code) => Token[]` | `:873` | mutates only `sid` on the **input** `Token`s → payload preserved | none |
| 16 | `markerCatalog` | `() => UsfmMarkerCatalog` | `:891` | core registry | none |
| 17 | `markerInfo` | `(marker) => MarkerInfo` | `:896` | core registry | none |
| 18 | `isKnownMarker` | `(marker) => boolean` | `:901` | core registry | none |
| 19 | `lintCodes` | `() => LintCode[]` | `:906` | via `lint_code_variants()` `:1503` — a **hand-maintained** 32-entry list | see F2 |
| 20 | `lintCodeMeta` | `() => LintCodeMeta[]` | `:914` | same hand list | see F2 |
| 21 | `formatRules` | `() => string[]` | `:927` | `FormatRule::ALL` — compiler-guarded | none |
| 22 | `formatRuleMeta` | `() => FormatRuleMeta[]` | `:935` | `FormatRule::ALL` | none |
| 23 | `vrefIndexUsfm` | `(source) => VrefIndex` | `:708` | native | none; **no golden/smoke coverage** (F3) |
| 24 | `vrefIndexTokens` | `(tokens) => VrefIndex` | `:716` | `WalkToken`→`tokens_to_vref_index` | none; **no golden/smoke coverage** (F3) |

---

## 5. npm package export map and the JS helper

| export | owner/file | signature | target | action | disposition | proof |
| --- | --- | --- | --- | --- | --- | --- |
| `"."` → `pkg-bundler/usfm_onion_web.{d.ts,js}` | `package.json:19-22` | bundler entry, also `"types"` at top level | wasm | retain; add the official decoder/reconciler exports (§10 Phase F.3) | retain | package.json |
| `"./web"` → `pkg-web/usfm_onion_web.{d.ts,js}` | `package.json:23-26` | web entry | wasm | retain | retain | package.json |
| `"./token-sids"` → `js/token-sids.{d.ts,js}` | `package.json:27-30` | JS helper subpath | js | retain | retain | package.json |
| `normalizeTokenSids(tokens: readonly Token[], bookCode: string): Token[]` | `js/token-sids.js:20`, `.d.ts:10` | pure JS, **no wasm dependency**; hand-written re-implementation of core `derive_canonical_sids` | js | **OWNER-DECISION §8.3** | retain | src |
| `js/token-sids.d.ts` `import type { Token } from "../pkg-bundler/usfm_onion_web.js"` | `js/token-sids.d.ts:1` | the JS helper's types are structurally coupled to the **bundler** package path | js | retain, but the path must survive wire absorption (the type moves crates; the emitted `.d.ts` path does not) | retain | src |
| `sideEffects`, `files`, `types` | `package.json:6-16,31` | packaging metadata | wasm | retain; `files` must grow when the official JS decoder ships | retain | package.json |
| `pkg-*/package.json` (`name: "usfm_onion_wasm"`, `main`, `types`, `files`, `sideEffects`) | generated | inner package manifests | wasm | retain (regenerated identically — §1) | retain | diff |

`js/token-sids.js`'s duplication **is** guarded: `scripts/test-token-sids.mjs:89-97` asserts the
plain-JS output equals the live Rust/wasm `normalizeTokenSids` output over its fixtures, plus
input-immutability and idempotence. So this is a *tested* duplicate, not silent drift — which is why
§8.3 is a disposition choice rather than a stop.

---

## 6. Conversion inventory — renames, lifetime erasure, cloning, enum translation, exceptions

Per the plan's fifth inventory bullet.

| # | conversion | site | what it does to the contract |
| ---: | --- | --- | --- |
| C1 | `AttributeItem.source` → `text` | `crates/usfm_onion_dto/src/lib.rs:793-812` | **field rename** across the boundary. Native `source`, wire `text`. |
| C2 | `TokenId<'a> { book_code, index }` → `Token.id: String` | dto `:888` | structured id **flattened to `format!("{}-{}")`**; the `{book}-{index}` convention becomes a string convention. Plan §5.1 keeps this mapping for cold parse. |
| C3 | `Option<Sid>` (8-byte `Copy`) → `Option<String>` | dto `:892` via `format_sid` `:1015` | compact anchor **stringified per token**. Recorded in the `diff-perf-sid-stringification` memory as the #1 diff hot spot; §7.5 replaces it with `PackedSid` on the wire only. |
| C4 | `LintIssue.template: &'static str` → `String` | `crates/usfm_onion_wasm/src/lib.rs:1316` | **lifetime erased + allocation** per issue. Recoverable from `code` (§7.6 says the catalog supplies it). |
| C5 | `TokenFix` externally tagged → internally tagged | core `src/lint_impl.rs:491` vs wasm `:273-278` | **enum representation translation**; TS sees `{type:"replaceToken", ...}`. |
| C6 | `LintScope` core `Serialize` vs wasm `rename_all="camelCase"` | core `:527` vs wasm `:199-206` | two different serializations of one logical type; TS shape is `"front" \| {chapter:number} \| "book"`. |
| C7 | `FormatOptions` 15 `bool` → 15 `Option<bool>` | wasm `:313-347`, applied by `apply_opt` `:1489` | **tri-state widening**: `None` means "leave the native default", so the wire type cannot express "explicitly default". |
| C8 | `DiffSkeleton<T>`/`DecisionUnit<T>` generic → concrete wire `Token` | wasm `:411`,`:435`, mapped `:1329-1383` | **generic erased** at the boundary; `UnitId` newtype → `String` (`:1346`,`:1365`). |
| C9 | `MarkerAttrs.attribute_source: Option<(Span, &'a str)>` → `Token.attribute_source: Option<String>` | dto `:919-922` | span dropped, slice **cloned**; the verbatim text survives, its position does not. |
| C10 | every `&'a str` field → `String` | dto `:885-969` | wholesale **source cloning** at the boundary; this is the cost §7.4's borrowed decode exists to remove. |
| C11 | 40 `From<Native…>` enum impls | dto, throughout | **exhaustive matches** — the primary compile-time drift guard (documented at dto `:16-19`). |
| C12 | `Result<_, JsError>` → thrown JS exception | wasm `js_error` `:1495`, `js_serde_error` `:1499`; used by `toUsj`, `toUsx`, `mergeDiffBlocks`, `revertDiffBlock` | **Rust error → exception**. §2.2#11 replaces this with tagged `ApiResult` for new braid APIs; these four pre-existing throws are unchanged by this epic. |
| C13 | `to_js_value<T: Serialize>` | wasm `:1485` | `toUsj()` bypasses tsify entirely and returns `any` (`serde_wasm_bindgen`), which is why §4.2's declared `UsjDocument` is unconnected. |

### Findings

**F1 — `attributes` / `attributeSource` do not survive the format, fix, merge, revert, or
token-diff legs of the boundary. Currently blessed in goldens.**

Root cause is in **core**, not the adapter: `FormatToken` (`src/format/mod.rs:396`) has no
attribute fields, and `FormattableToken` (`:449`) has no attribute accessor. Every wasm export
routed through `FormatToken` or `WalkToken` therefore reconstructs the output `Token` with
`attributes: Vec::new(), attribute_source: None` hardcoded (`map_format_token` `:1199-1200`,
`map_walk_token` `:1411-1412`).

Reproduced from the committed goldens, fixture `crates/usfm_onion_wasm/golden/inputs/attributes.usfm`
whose last line is `\v 2 the second verse \w gracious|lemma="grace" \w*`:

| golden | attribute list present? |
| --- | --- |
| `attributes/to-usfm.usfm` (`ParsedUsfm.toUsfm`) | yes — byte-exact |
| `attributes/tokens-to-usfm.usfm` (`tokensToUsfm`) | yes — byte-exact |
| `attributes/tokens.json`, `cst.json`, `diff.json`, `diff-by-chapter.json` | yes (`attributeSource` present) |
| `attributes/format.usfm` (`ParsedUsfm.format`) | **no** — emits `\w gracious\w*` |
| `attributes/format-tokens.json` (`formatTokens`) | **no** — `usfm` field is `\w gracious\w*`; the `\w` token has no `attributes`/`attributeSource` |
| `attributes/diff-tokens.json` (`diffTokens`) | **no** — zero occurrences of `attributeSource` |

Untested paths with the same root cause: `applyTokenFix`, `formatTokensMut`, `mergeDiffBlocks`,
`revertDiffBlock`, `ParsedUsfm.applyTokenFix`, `ParsedUsfm.revertDiffBlock`. They are untested
because both harnesses use attribute-free fixtures (`scripts/test-web-package.mjs:18`
`"\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning\n"`, `:120` `"\\id GEN\n\\c 1\n\\p\\v 1 Word\n"`),
so `test-web-package.mjs:136`'s "merge all-baseline reproduces source byte-exact" assertion passes
without ever exercising an attribute.

Why this matters to the epic, stated but **not** decided here: plan §9 requires "Serialization is
token-to-USFM lossless"; §5.3 has `prepare_format_patch` + `apply_patch` rebuild "authoritative
bytes/hash" from formatted resident tokens; §6.2 requires "exact serialization" on every mutation.
If braid's format/fix path reuses core `format()` over a `FormattableToken`, the resident book's
rebuilt bytes lose every attribute list — a §15 "Payload loss" footgun and stop clause 5 ("any
mutation that can change tokens without rebuilding authoritative source/hash" — here the rebuild
happens but is lossy). This is a **pre-existing, blessed behavior**, not something introduced or
fixed by Gate 0. Escalated to §8/§9 for owner adjudication; 0D's token-variant matrix should treat
it as a known non-round-tripping field rather than rediscovering it.

**F2 — `lint_code_variants()` is a hand-maintained list, not compiler-guarded.**
`crates/usfm_onion_wasm/src/lib.rs:1503-1541` enumerates all 32 `LintCode`s by hand to back
`lintCodes()` / `lintCodeMeta()`. The dto crate's own docs already name this as one of two
non-compiler-guarded hand lists (`crates/usfm_onion_dto/src/lib.rs:20-22`, the other being core's
`ALL_LINT_CODES` test array). Under §7.7's "no JS rule list is handwritten" and §7.6's "generated
stable rule catalog", wire's generated catalog replaces it. Disposition: **replace**.

**F3 — `vrefIndexUsfm`, `vrefIndexTokens`, `normalizeTokenSids`, `formatTokensMut`,
`applyTokenFix`, `mergeDiffBlocks`, `revertDiffBlock`, `formatUsfm`, `lintUsfm`, `diffUsfm`,
`diffUsfmByChapter`, `markerInfo`, `isKnownMarker` have no golden coverage.**
`scripts/wasm-golden.mjs` captures 5 corpus-wide outputs and 16 per-fixture outputs; the vref
goldens come from `ParsedUsfm.toVref()`, never from `vrefIndexUsfm`/`vrefIndexTokens`. Most of the
list is at least smoke-asserted in `scripts/test-web-package.mjs`; `vrefIndexUsfm` and
`vrefIndexTokens` appear in **neither** harness. Disposition: **retain**, but Phase F's declaration
diff cannot rely on goldens to prove their shape.

**F4 — the §4.3 / §17#12 "known existing violation" is already retired.**
The plan describes `usfm_onion_wasm::token_values_to_usfm` (with `format_attribute_list` /
`closer_shape` / `token_closes`) as a live USFM-emission fork to retire during adoption. It no
longer exists: `closer_shape` and `token_closes` are private core helpers
(`src/token.rs:627`,`:671`), and wasm calls the one core emitter `tokens_to_usfm_reconstruct`
(`crates/usfm_onion_wasm/src/lib.rs:767`,`:772`). Consistent with the landed
`serializable-token-contract` work. Plan-text staleness only; no action.

---

## 7. Disposition tallies

Every current export is classified. Counts are over distinct named exports/symbols.

| disposition | count | contents |
| --- | ---: | --- |
| **retain** | 97 npm exports (24 fns + 2 classes + 71 types) + 43 dto items + all 423 core declarations | everything except the rows below |
| **replace** | 9 | `WalkToken` and its 7 conversion helpers (→ core `OwnedToken` + the two §5.1 conversions); `lint_code_variants()` (→ generated wire catalog, F2) |
| **delete-in-breaking-release** | 0 | — |
| **OWNER-DECISION** | 4 rows (§8) | `BookId` reuse; USJ hand-written TS ownership; `js/token-sids` duplicate; home of braid lifecycle TS mirrors |

Nothing is marked for deletion. Per §3.1 0C, silence is not permission to delete: the two npm
classes and all 24 free functions survive the Phase F release, and every one of the 71 Rust-derived
npm types keeps its declaration (only its defining crate changes). `usfm_onion_dto` the *crate*
disappears (§2.2#2) but none of its 43 public items do.

---

## 8. OWNER-DECISION rows

Framed only; not decided here.

**8.1 — Does braid's declared `BookId` reuse core `usfm_onion::token::BookId`?**
Core already has `BookId([u8;3])` (`src/token.rs:758`) with 3-ASCII-alphanumeric validation and
deliberately *no* canonical-66-book membership check. Plan §5.2 requires "a unique canonical
`BookId` from the caller's agreed manifest" and §7.2 stores it as the 3-byte TOC field. Reuse the
existing core type (cheap, already `Copy`/`Hash`/`Ord`, already `Serialize`s as 3 chars, but accepts
non-canonical codes like `ZZZ`), or introduce a distinct validated type. Note core's `BookId` is
reachable only as `usfm_onion::token::BookId` — it is not in the `src/lib.rs` root re-export list.

**8.2 — Who owns the four hand-written USJ TypeScript declarations?**
`Value`, `UsjDocument`, `UsjNode`, `UsjElement` are emitted from a `typescript_custom_section`
string in the wasm crate (`lib.rs:59-102`) and are the only npm types not derived from Rust;
`toUsj()` is typed `any`, so the declarations are decorative. §4.1 gives wire "generated JS schema
data" and forbids wasm "copied DTO enums", which suggests wire — but promoting them to real tsify
types over core `UsjDocument`/`UsjNode`/`UsjElement` is a declaration change (`any` → typed) that
the epic does not authorize. Choice: (a) move the string verbatim to wire, (b) promote to tsify in
wire and change `toUsj()`'s declared return type, (c) leave in wasm as the documented exception.

**8.3 — Does `js/token-sids.js` remain a hand-written duplicate of core
`derive_canonical_sids`?**
It is a deliberate wasm-free re-implementation (`js/token-sids.js:1-5`) and is conformance-tested
against the Rust export (`scripts/test-token-sids.mjs:89-97`). §4.1 says the official JS helper must
not own "lint rules"; SID derivation is not a lint rule but it is core semantics, and §15 names
"Algorithm fork" a footgun. Choice: retain as a tested duplicate (its no-wasm-dependency property is
the whole point), or delete-in-breaking-release now that the wasm export exists.

**8.4 — Where do the TypeScript mirrors of braid's lifecycle types live?**
§8.2 of the plan specifies generated TS for `BraidConfig`, `MutationEffect`, `CorpusInput`,
`BookInput`, `ChapterInput`, `ChapterTarget`, `CorpusScope`, `PatchHandle`, `PatchPreparation`,
`ScopedOutput<T>`, `LintPrimeInput`, `PrimeReport`, `ApiResult<T,E>` and every typed error enum.
Those semantic types live in **braid**, but §4.1 forbids braid owning "JS values" and wire owning
anything braid-shaped (wire and braid are siblings; wire depending on braid is a reverse edge). The
two DAG-legal homes are (a) braid gains a feature-gated `serde`/`tsify` layer over its own types, or
(b) wasm defines tsify mirror types. See the 0G ledger §4 for why (a) creates no cycle and (b)
reintroduces exactly the mirror-drift §4.1 was written to prevent.

---

## 9. Stop conditions

Against the handoff's four stop conditions:

| stop condition | status |
| --- | --- |
| any type forcing core → wire/braid/wasm, or braid → wire | **not hit.** See 0G ledger §3. Core is wasm-bindgen/tsify-free today and `OwnedToken` needs no new dependency (core already has non-optional `serde` with `derive`). |
| any dependency or feature cycle in the target DAG | **not hit.** See 0G ledger §4. |
| any duplicated discriminant source or target-specific semantic type | **not hit for discriminants** (§1: bundler and web share one generated source and one golden directory; F2's hand list is a single Rust-side list, not a second source). No target-specific semantic type exists: core compiles identically for both targets apart from `cfg(not(wasm32))` rayon/memchr. |
| any public npm export that cannot be classified retain/replace/delete | **not hit.** All 97 classified (§7). |

**One stop-adjacent finding raised for owner adjudication, outside the handoff's four:** F1
(attribute payload loss through the format/fix/merge/revert leg). It is not one of 0C's declared stop
conditions, and it is pre-existing blessed behavior rather than a migration consequence — but it
directly contradicts plan §9's losslessness requirement for the resident `prepare_format_patch` /
`apply_patch` path and matches §15's "Payload loss" footgun. Recorded, not worked around, not fixed.

---

## 10. Proposed final Cargo edge list

Per the 0C deliverable. Full matrices in the 0G ledger.

```text
usfm_onion            -> logos, quick-xml, regex, rustc-hash, serde, serde_json, similar
                         + cfg(not(wasm32)): rayon, memchr
                         (no workspace deps — sink of the DAG)
usfm_onion_wire       -> usfm_onion, serde [, tsify + wasm-bindgen under feature "wasm"]
braid                 -> usfm_onion [, xxhash implementation for SourceHash]
usfm_onion_wasm       -> usfm_onion, usfm_onion_wire (feature "wasm"), braid,
                         wasm-bindgen, js-sys, serde, serde_json, serde-wasm-bindgen, tsify
usfm_onion_dto        -> DELETED (all 43 items absorbed into usfm_onion_wire)
```

`braid`'s xxhash dependency is new (§2.1#8 makes xxhash3 the persisted fingerprint; core currently
has only `rustc-hash`/FxHash, which §2.1#8 keeps internal-only). It is a leaf dependency of braid
and creates no edge toward wire or wasm.
