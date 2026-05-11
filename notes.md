1. cst::parse_usfm(&source);
2. let handle = crate::internal::parse::parse(source);
   1. let lexed = lex(source);
   2. let analysis = collect_analysis(&lexed.tokens);
   3. ParseHandle::new(source.to_string()) -> why an owned string?
3. handle_to_cst
   1.  let tokens = crate::parse::handle::tokens(handle, crate::model::TokenViewOptions::default());
       1.   let mut projected = project_raw_tokens(handle.raw_tokens(), handle.analysis());
            1.   append_horizontal_whitespace ->
       2.   merge_horizontal_whitespace(&mut projected);



3.1.1.1 Why project_raw_tokens? -> we already have a flat token list.  But why does it do it's own append_horizontal_whitespace? This seems redundant and now we're cloning and creating new owned structs. Ah nvm, I'm reading more now. and it's changing the semantics slightly coalescing stuff like closingMarker and nestedClosingMarker into a single endmarker. Still, so much cloning. A mut pass over the original lexed tokens would be better I think and mut that vec to transform. 
3.1.1.2 -> why we can't use a monotonic counter of 1v1 CST to tokens: We are merging them before CST can see them. But 

Why handle.rs:
#[allow(dead_code)]
    pub(crate) fn document(&self) -> &Document {
        &self.analysis.document
    }



# Education:
1. Why would you want to do this?
`as_deref` Converts from Option<T> (or &Option<T>) to Option<&T::Target>.

Leaves the original Option in-place, creating a new one with a reference to the original one, additionally coercing the contents via Deref.

2. What's a BTreeMap? 
3. tokens.iter().skip(start) -> Does this not still do a full pass and just throw away the skipped elements? Would vec index access not be better here?
4. what is ToOwned::to_owned? vs just clone?


# small notes: 
 let mut current_chapter = 0u32; -> Never more than 151 chapters in a bible book. Can be a u16. 




Overall:
Pass 1: Lex -> I think looks fine, though would be curious to bench against 





Compiling usfm_onion v0.1.0 (/Users/willkelly/Documents/Work/Code/usfm_onion)
    Finished `bench` profile [optimized] target(s) in 13.38s
     Running benches/lexer.rs (target/release/deps/lexer-0973d988701796dd)
lexer/corpus/lex/short  time:   [22.608 µs 23.045 µs 23.801 µs]
                        thrpt:  [75.089 MiB/s 77.551 MiB/s 79.050 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking lexer/corpus/lex/medium: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.0s, enable flat sampling, or reduce sample count to 50.
lexer/corpus/lex/medium time:   [1.6207 ms 1.6291 ms 1.6411 ms]
                        thrpt:  [84.312 MiB/s 84.930 MiB/s 85.370 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
lexer/corpus/lex/large  time:   [7.1522 ms 7.2670 ms 7.4751 ms]
                        thrpt:  [34.777 MiB/s 35.773 MiB/s 36.347 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
lexer/corpus/lex/xl     time:   [28.783 ms 29.526 ms 30.552 ms]
                        thrpt:  [159.89 MiB/s 165.45 MiB/s 169.72 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe




Let's start the next layer of the onion. 

The output of the Lexer should be a "Lexeme"

Parse:
Unlike our main inspo crate of usfm3, we don't want an AST but rather a concrete syntax tree. 
The entrypont is `parse` and it has two outputs: 
1. A pass over our lexed tokens that adds ids, sids, merges in horizontal whitespace, combines multiple semantic / logical units into a single itme (i.e. attributes) General Syntax. The output goal is Vec of Tokens that raises the semantic one more level. 
In USFM, within a character marker span an attributes list is separated from the text content by a vertical bar |. Attributes are listed as pairs of name + corresponding value using the syntax: attribute="value". The attribute name is a single ASCII string. The value is wrapped in quotes.

In USX, attributes are applied to elements in the standard XML syntax: attribute="value".

USFM

USX

USJ

Example 1. Glossary word with lemma attribute
\w gracious|lemma="grace"\w*. 


The Tokens must have a dedicated fn / impl to go back into usfm string format. 

The next downstream from it will be the concrete syntax tree I think. Where I'd love to design such that 1 Token (not lexeme) = 1 entry in the CST and the CST is doing ntohing more than placing Tokens into a tree relationship.


cargo bench --bench parse
    Finished `bench` profile [optimized] target(s) in 0.41s
     Running benches/parse.rs (target/release/deps/parse-47607e8129ac2e17)
parse/corpus/parse/short
                        time:   [14.592 µs 14.660 µs 14.777 µs]
                        thrpt:  [120.94 MiB/s 121.91 MiB/s 122.48 MiB/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking parse/corpus/parse/medium: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.5s, enable flat sampling, or reduce sample count to 60.
parse/corpus/parse/medium
                        time:   [1.0940 ms 1.1478 ms 1.2455 ms]
                        thrpt:  [111.09 MiB/s 120.55 MiB/s 126.47 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe
parse/corpus/parse/large
                        time:   [4.5921 ms 4.5977 ms 4.6035 ms]
                        thrpt:  [56.471 MiB/s 56.542 MiB/s 56.611 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking parse/corpus/parse/xl: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.5s, or reduce sample count to 70.
parse/corpus/parse/xl   time:   [60.486 ms 60.993 ms 61.550 ms]
                        thrpt:  [79.366 MiB/s 80.092 MiB/s 80.763 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe





spec formats:
usj / usx 

USJ
- see usfm3 crate where its AST mimic usj on purpose
- chapter becomes
  -  "type": "chapter",
      "marker": "c",
      "number": "2",
      "sid": "LAM 2". (not 0 prefixed)
- verse becomes:
  -   "type": "verse",
          "marker": "v",
          "number": "1",
          "sid": "LAM 2:1"

- Text is just a scalar value and not an object
- attributes are key-value pairs. 
not sure on iter over token vs walk concrete syntax tree on this one for what's simpler? Mostly our concrete syntax tree doesn't nest chars for example, and chapters are already a separate token from numbers where it collapses the two, so will have to have some logic to have to account for how usj is different than our eixsting concrete syntax tree that is more granular. 


NlP format
vref

web reading format
html

diff: 
I think can just wholesale keep things as they were unless you see a reason not too?



## For benches with copora
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench lint
USFM_BENCH_CORPORA="en_ulb en_ult" cargo bench --bench lint
USFM_BENCH_CORPORA=all cargo bench --bench lint



## Refactors
  1. Docs folder — Right, the two plans (plan-whitespace-lint-rules.md, plan-marker-data-curation.md) are still deferred work, not done. They
  were landed in 8040ed1 as roadmaps. The marker-data one should go first since lint rules depend on clean lookup_spec_marker resolution.

  2. HTML 1.94× scaling outlier — Worth profiling. Most likely culprits in order: (a) small-string allocation churn in the renderer (each tag
  open/close, attribute write), (b) a shared String or Vec being grown without with_capacity, (c) some accidental serialization point (Mutex,
  or a writer the books share). Easy first check: run samply on the html op and look for alloc/realloc dominance vs. actual formatting work.
  If it's allocs, pre-size the output buffer per book based on input size × a small multiplier.

  3. Attributes as their own CST node — I think you're right to question this. For losslessness with default-attribute shorthand (\w 
  word|lemma\w* style), the tuple-of-fragments approach is cleaner: each token carries an optional ordered attribute list where each entry is
  either Named(key, value) or DefaultValue(value), and round-tripping is just emit-in-order. That makes the "is this the default attribute"
  question a property of the position, not a flag on a separate node. Worth a small spike before committing — render-LTR round-trip parity is
  the test that matters.

  4. Wasm bindings — Agree, hand-written wasm_bindgen for a large surface is painful. Two paths: (a) tsify + serde-wasm-bindgen to auto-derive
   TS types from the Rust types you already have, or (b) a single run(op: &str, input: &str, opts: JsValue) -> JsValue entry that dispatches —
   fewer bindings, JS side gets a thin wrapper. (a) is nicer for DX, (b) is nicer for bundle size and maintenance.

  5. Playground → sampling harness — Sounds like you want two separate things: a real CLI (subcommands per op, like usfm3) and a
  samply-friendly iter harness where every op is a one-liner you toggle. I'd keep playground as the sampling harness — rename it
  bin/sample.rs, import the crate glob, have each op be // run_lint(&doc, N_ITERS); lines you uncomment. Move the user-facing ops to a proper
  bin/usfm.rs clap CLI. They have different jobs.

  6. README audit + drop execution.ndjson/omni.ndjson — Yes, those names predate the current shape of things. Worth a pass.

  7. ICU-style lint labels — Yes, that's the term — ICU MessageFormat. Friendly default messages with named placeholders ({marker}, {line},
  etc.) and a stable rule ID separate from the human message. Lets you localize later and lets tooling group by ID while humans read the
  message. Worth defining the shape (ID, severity, message template, placeholders) before adding more rules — ties into the
  whitespace-lint-rules plan.

  Want me to dig into any of these? My instinct on ordering: 3 (CST attributes) is the most architectural and blocks losslessness claims, so
  worth resolving early; 2 (html profiling) is a quick win; 7 (ICU labels) should land before the whitespace rules plan starts.



8. Code reuse / dedup / splitting: 
     Audit results:
  - USJ ✅ already correct (export tree has force_close_notes for chapter/paragraph/header/meta/periph/tablerow/sidebar; verse goes through
   close_open_meta which catches it).
  - USX ✅ same — uses the export tree.
  - HTML ❌ was broken — parse_note_tokens scanned to EOF when no \f* was found, nesting everything subsequent inside <aside>. Now fixed.

  The fix: parse_note_tokens now stops at the first block-scope marker when a note is open, matching the same StructuralScopeKind set vref
  uses. Helper returns a fourth value (resume_index) so the render loop knows where to pick up — body_end + 1 for an explicit \f* close
  (skip past it), or body_end itself for recovery (process the boundary marker normally next iteration).

  Tests added on all three surfaces asserting GEN 1:2, GEN 2, and GEN 2:1 all appear after the unclosed footnote, and that none leak into
  the note body. 98/98 pass. WASM still builds.

  On your point about lossless surfaces:
  - Token stream and CST: lossless by design, no recovery applied. Consumers that need exact source preservation use these.
  - to_vref, to_html, to_usj, to_usx: lossy by definition, recovery now consistently applied.

  The commit message documents this contract explicitly so future readers know which surfaces to reach for under what need.

  Code reuse note: I noticed all three lossy paths now have the same boundary set. There's a soft case for hoisting note_recovery_boundary
  into crate::marker_defs (next to StructuralScopeKind) so the rule lives in one place. I didn't do it because the call sites are tiny (one
   matches! macro each) and the indirection costs more than the dedup saves at this scale. If we add a fifth surface or the boundary set
  ever drifts, that's the moment to consolidate.