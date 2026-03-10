# Refactor goals:


Onion = layers that can be jumped into. 

## Workflow 1 from usfm: 
1. input usfm
2. parse to `lexed_tokens`
   1. These could be round tripped back to usfm
   2. They are a core primitive for ops
   3. Arg to merge the horizontal whitespace or not (default true)
3. parse to `parsed_document`
   1. This operates over `lexed_tokens`
   2. It's a typed tree superset following usfm semantics as much as possible regarding paragraphing, etc; Usj like, but more in that all tokens get a "type" (i.e) not just text scalars. linebreaks are explicitly tracked.  note / para / char nesting follows usfm like semantics as see inusj. 
   3. It must be able to be flattend back to `lexed_tokens` and thus be round-trippable to source usfm byte for byte via a dfs of this tree + write of it's "value" type field. No need to track "recoveries" to expose to user. 
4. To USJ | USX | HTML | VREF = 
   1. Built from a `parsed_document`
   2. lossy conversions - (though don't do lossy on purpose just cause, we just mean that maybe something like vrefs are inherently lossy) or maybe that new lines aren't recorded explicity in usj or usx or somethign.

## If input is usx or usj:
1. Convert these input formats into lexed_tokens as the ling franca for the onion to do ops on. Dont' create a lint_usj, lint_usx, etc;  must first convert to lexed_tokens.
2. Probably a dedicated converter module can do impl to_tokens for USJ and impl to_tokens for USFM and for usx too? Then we'll be at the lingu franca. For api, I think as long as it's predictable (see note on ops below) then flat is fine. usfm_to_tokens, usfm_to_document_tree, usfm_to_usj, usm_to_usx, usfm_to_html, usfm_to_vref, etc.   It might be nice to abstract some of these into a builder api though? What are you thoughts on cleanest and most rustacean way but also the most predictable and maintainable?

## Core principles 
1. All mutation, down to every single space and byte, must be opt in via lint fix or format. 

## Ops
1. Always operate over flat token stream to simplify logic and avoid nesting issues.
2. Should have rust like mechanic for op and op_mut (where mutation is in view). 
3. Should have parallel vs non parallel versions (rayon compiled optin). A cfg flag. if rayon, do parallel only at the file level for batch ops. 
4. Example. format, format_mut, format_batch, format_batch_mut. 

3. Lint
   1. Over a flat token stream, validate usfm for both syntax issues (i.e. char not closed type rules) or style issues (i.e. missing verse content)
   2. May be best as a two pass approach? To index and then evaluate? Not sure.  For example, you may have to scan forward or back in the token stream to validate a rule. Not sure what is most performant here. 
   3. probably no mut variant, but can return "fixes" which are functions that will rewrite the token stream with the fix. 
   4. Suppresion config should be code + sid. While I like the idea of suppression living here, putting in lib allows for lower ipc overhead and object creation potentially in something like a tauri app.

4. Format
   1. Mostly fine I think. opinionated opt in rules.  mut vs non mut variants make sense to return a before/after if someone is wants to keep a history stack and doesn't want to mut reference. 
   2. Let user pass in custom format functions as exists today. 

5. Diff
   1. I think fine as is, juust make sure api surface is consitent unless you think there is something here that should change.  I know it's sort of granular to do revert and what not, so maybe thre is something better here? Not sure?