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


## For benches with copora
USFM_BENCH_CORPORA=examples.bsb cargo bench --bench lint
USFM_BENCH_CORPORA="en_ulb en_ult" cargo bench --bench lint
USFM_BENCH_CORPORA=all cargo bench --bench lint
