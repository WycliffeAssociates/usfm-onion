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
