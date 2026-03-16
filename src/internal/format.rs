// use std::time::Duration;

// use crate::internal::markers::{MarkerKind, lookup_marker};
// use crate::model::token::{Span, Token, TokenKind, normalized_marker_name};

// const POETRY_MARKERS: &[&str] = &[
//     "q", "q1", "q2", "q3", "q4", "q5", "qc", "qa", "qm", "qm1", "qm2", "qm3", "qd",
// ];

// const LINEBREAK_BEFORE_AND_AFTER_MARKERS: &[&str] = &[
//     "p", "m", "pi", "pi1", "pi2", "pi3", "pi4", "ms", "ms1", "ms2", "ms3", "li", "li1", "li2",
//     "li3", "li4", "b",
// ];

// const LINEBREAK_BEFORE_ONLY_MARKERS: &[&str] = &[
//     "cl", "cd", "d", "sp", "r", "mr", "sr", "s", "s1", "s2", "s3", "s4",
// ];

// /// Toggleable formatter rules.
// ///
// /// `FormatOptions::default()` enables every rule. Use
// /// `FormatOptions::only(...)` or `FormatOptions::excluding(...)` to build a
// /// narrower ruleset.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum FormatRule {
//     /// Split malformed marker-looking text so the output can still be parsed.
//     RecoverMalformedMarkers,
//     /// Collapse repeated horizontal whitespace runs inside text nodes.
//     CollapseWhitespaceInText,
//     /// Ensure there is visible separation before inline markers.
//     EnsureInlineSeparators,
//     /// Remove duplicate verse number markers when the second one adds no value.
//     RemoveDuplicateVerseNumbers,
//     /// Insert a separator after paragraph markers when text starts immediately.
//     NormalizeSpacingAfterParagraphMarkers,
//     /// Remove stray linebreaks that should not interrupt inline content.
//     RemoveUnwantedLinebreaks,
//     /// Merge adjacent verse markers into a single bridged verse range where valid.
//     BridgeConsecutiveVerseMarkers,
//     /// Remove an empty verse marker when the following verse carries the content.
//     RemoveOrphanEmptyVerseBeforeContentfulVerse,
//     /// Remove repeated verse enumerators that duplicate a bridged verse range.
//     RemoveBridgeVerseEnumerators,
//     /// Move chapter labels after the chapter marker they describe.
//     MoveChapterLabelAfterChapterMarker,
//     /// Insert a default paragraph marker before verse content after chapter intro material.
//     InsertDefaultParagraphAfterChapterIntro,
//     /// Remove empty paragraph markers that only separate into another block marker.
//     RemoveEmptyParagraphs,
//     /// Insert canonical structural linebreaks around structural markers.
//     InsertStructuralLinebreaks,
//     /// Collapse repeated structural blank lines into a single canonical break.
//     CollapseConsecutiveLinebreaks,
//     /// Remove leading horizontal whitespace before line-start markers.
//     NormalizeMarkerWhitespaceAtLineStart,
// }

// /// Built-in formatter rule selection.
// ///
// /// Default behavior enables every built-in rule. If you need a smaller set, use
// /// `FormatOptions::only(...)` or `FormatOptions::excluding(...)`.
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct FormatOptions {
//     /// Before: `text \zzbad`
//     /// After: `text \zz bad`
//     pub recover_malformed_markers: bool,
//     /// Before: `In   the   beginning`
//     /// After: `In the beginning`
//     pub collapse_whitespace_in_text: bool,
//     /// Before: `word\\w lemma="x"`
//     /// After: `word \\w lemma="x"`
//     pub ensure_inline_separators: bool,
//     /// Before: `\\v 1 text \\v 1 text`
//     /// After: one verse number retained
//     pub remove_duplicate_verse_numbers: bool,
//     /// Before: `\\ptext`
//     /// After: `\\p text`
//     pub normalize_spacing_after_paragraph_markers: bool,
//     /// Before: stray/newline noise around inline content
//     /// After: unwanted linebreaks removed
//     pub remove_unwanted_linebreaks: bool,
//     /// Before: `\\v 1 ... \\v 2-3 ...`
//     /// After: consecutive verse markers bridged into one sequence where possible
//     pub bridge_consecutive_verse_markers: bool,
//     /// Before: empty verse marker immediately before the real verse text
//     /// After: orphan empty verse marker removed
//     pub remove_orphan_empty_verse_before_contentful_verse: bool,
//     /// Before: `\\v 2-3 2 text`
//     /// After: the repeated enumerator after a bridged verse marker is removed
//     pub remove_bridge_verse_enumerators: bool,
//     /// Before: `\\cl Chapter 1\\c 1`
//     /// After: `\\c 1\\cl Chapter 1`
//     pub move_chapter_label_after_chapter_marker: bool,
//     /// Before: chapter intro content with no paragraph marker
//     /// After: default paragraph inserted before verse-bearing content
//     pub insert_default_paragraph_after_chapter_intro: bool,
//     /// Before: paragraph marker with no content before the next block marker
//     /// After: empty paragraph marker and its trailing blank whitespace are removed
//     pub remove_empty_paragraphs: bool,
//     /// Before: structural markers packed together
//     /// After: canonical structural linebreaks inserted
//     pub insert_structural_linebreaks: bool,
//     /// Before: many blank structural breaks in a row
//     /// After: collapsed to a single canonical break
//     pub collapse_consecutive_linebreaks: bool,
//     /// Before: leading whitespace before line-start markers
//     /// After: marker starts flush at line start
//     pub normalize_marker_whitespace_at_line_start: bool,
// }

// impl Default for FormatOptions {
//     fn default() -> Self {
//         Self::all_enabled()
//     }
// }

// impl FormatOptions {
//     pub const fn all_enabled() -> Self {
//         Self {
//             recover_malformed_markers: true,
//             collapse_whitespace_in_text: true,
//             ensure_inline_separators: true,
//             remove_duplicate_verse_numbers: true,
//             normalize_spacing_after_paragraph_markers: true,
//             remove_unwanted_linebreaks: true,
//             bridge_consecutive_verse_markers: true,
//             remove_orphan_empty_verse_before_contentful_verse: true,
//             remove_bridge_verse_enumerators: true,
//             move_chapter_label_after_chapter_marker: true,
//             insert_default_paragraph_after_chapter_intro: true,
//             remove_empty_paragraphs: false,
//             insert_structural_linebreaks: true,
//             collapse_consecutive_linebreaks: true,
//             normalize_marker_whitespace_at_line_start: true,
//         }
//     }

//     pub const fn none() -> Self {
//         Self {
//             recover_malformed_markers: false,
//             collapse_whitespace_in_text: false,
//             ensure_inline_separators: false,
//             remove_duplicate_verse_numbers: false,
//             normalize_spacing_after_paragraph_markers: false,
//             remove_unwanted_linebreaks: false,
//             bridge_consecutive_verse_markers: false,
//             remove_orphan_empty_verse_before_contentful_verse: false,
//             remove_bridge_verse_enumerators: false,
//             move_chapter_label_after_chapter_marker: false,
//             insert_default_paragraph_after_chapter_intro: false,
//             remove_empty_paragraphs: false,
//             insert_structural_linebreaks: false,
//             collapse_consecutive_linebreaks: false,
//             normalize_marker_whitespace_at_line_start: false,
//         }
//     }

//     pub fn only(rules: &[FormatRule]) -> Self {
//         let mut options = Self::none();
//         for rule in rules {
//             options.set(*rule, true);
//         }
//         options
//     }

//     pub fn excluding(rules: &[FormatRule]) -> Self {
//         let mut options = Self::all_enabled();
//         for rule in rules {
//             options.set(*rule, false);
//         }
//         options
//     }

//     pub fn set(&mut self, rule: FormatRule, enabled: bool) {
//         match rule {
//             FormatRule::RecoverMalformedMarkers => self.recover_malformed_markers = enabled,
//             FormatRule::CollapseWhitespaceInText => self.collapse_whitespace_in_text = enabled,
//             FormatRule::EnsureInlineSeparators => self.ensure_inline_separators = enabled,
//             FormatRule::RemoveDuplicateVerseNumbers => {
//                 self.remove_duplicate_verse_numbers = enabled
//             }
//             FormatRule::NormalizeSpacingAfterParagraphMarkers => {
//                 self.normalize_spacing_after_paragraph_markers = enabled
//             }
//             FormatRule::RemoveUnwantedLinebreaks => self.remove_unwanted_linebreaks = enabled,
//             FormatRule::BridgeConsecutiveVerseMarkers => {
//                 self.bridge_consecutive_verse_markers = enabled
//             }
//             FormatRule::RemoveOrphanEmptyVerseBeforeContentfulVerse => {
//                 self.remove_orphan_empty_verse_before_contentful_verse = enabled
//             }
//             FormatRule::RemoveBridgeVerseEnumerators => {
//                 self.remove_bridge_verse_enumerators = enabled
//             }
//             FormatRule::MoveChapterLabelAfterChapterMarker => {
//                 self.move_chapter_label_after_chapter_marker = enabled
//             }
//             FormatRule::InsertDefaultParagraphAfterChapterIntro => {
//                 self.insert_default_paragraph_after_chapter_intro = enabled
//             }
//             FormatRule::RemoveEmptyParagraphs => self.remove_empty_paragraphs = enabled,
//             FormatRule::InsertStructuralLinebreaks => self.insert_structural_linebreaks = enabled,
//             FormatRule::CollapseConsecutiveLinebreaks => {
//                 self.collapse_consecutive_linebreaks = enabled
//             }
//             FormatRule::NormalizeMarkerWhitespaceAtLineStart => {
//                 self.normalize_marker_whitespace_at_line_start = enabled
//             }
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Default)]
// pub struct FormatProfile {
//     pub normalize: Duration,
//     pub verse_normalize: Duration,
//     pub default_paragraphs: Duration,
//     pub structural_linebreaks: Duration,
//     pub collapse_linebreaks: Duration,
//     pub normalize_line_start: Duration,
//     pub total: Duration,
// }

// pub trait FormattableToken: Clone {
//     fn id(&self) -> Option<&str> {
//         None
//     }
//     fn set_id(&mut self, _id: String) {}
//     fn kind(&self) -> &TokenKind;
//     fn set_kind(&mut self, kind: TokenKind);
//     fn text(&self) -> &str;
//     fn set_text(&mut self, text: String);
//     fn marker(&self) -> Option<&str>;
//     fn set_marker(&mut self, marker: Option<String>);
//     fn sid(&self) -> Option<&str>;
//     fn set_sid(&mut self, sid: Option<String>);
//     fn span(&self) -> Option<&Span> {
//         None
//     }
//     fn synthetic_like(
//         anchor: Option<&Self>,
//         kind: TokenKind,
//         text: String,
//         marker: Option<String>,
//         sid: Option<String>,
//     ) -> Self;
// }

// impl FormattableToken for Token {
//     fn id(&self) -> Option<&str> {
//         Some(&self.id)
//     }

//     fn set_id(&mut self, id: String) {
//         self.id = id;
//     }

//     fn kind(&self) -> &TokenKind {
//         &self.kind
//     }

//     fn set_kind(&mut self, kind: TokenKind) {
//         self.kind = kind;
//     }

//     fn text(&self) -> &str {
//         &self.text
//     }

//     fn set_text(&mut self, text: String) {
//         self.text = text;
//     }

//     fn marker(&self) -> Option<&str> {
//         self.marker.as_deref()
//     }

//     fn set_marker(&mut self, marker: Option<String>) {
//         self.marker = marker;
//     }

//     fn sid(&self) -> Option<&str> {
//         self.sid.as_deref()
//     }

//     fn set_sid(&mut self, sid: Option<String>) {
//         self.sid = sid;
//     }

//     fn span(&self) -> Option<&Span> {
//         Some(&self.span)
//     }

//     fn synthetic_like(
//         anchor: Option<&Self>,
//         kind: TokenKind,
//         text: String,
//         marker: Option<String>,
//         sid: Option<String>,
//     ) -> Self {
//         let span = anchor
//             .and_then(|token| token.span().cloned())
//             .map(|span| span.start..span.start)
//             .unwrap_or(0..0);
//         Self {
//             id: String::new(),
//             kind,
//             span,
//             sid,
//             marker,
//             text,
//         }
//     }
// }

// /// Custom formatter pass applied after the built-in rule set.
// ///
// /// Implement this when built-in `FormatRule`s are not enough for a downstream
// /// policy. Passes are ordered and mutate a working token buffer in place.
// pub trait TokenFormatPass<T: FormattableToken> {
//     fn label(&self) -> &str;
//     fn apply(&self, tokens: &mut Vec<T>);
// }

// /// Boxed formatter pass object.
// pub type BoxedTokenFormatPass<T> = Box<dyn TokenFormatPass<T>>;

// pub fn prettify_tokens<T: FormattableToken>(tokens: &[T], options: FormatOptions) -> Vec<T> {
//     format_tokens(tokens, options)
// }

// pub fn format<T: FormattableToken>(tokens: &[T]) -> Vec<T> {
//     format_tokens(tokens, FormatOptions::default())
// }

// pub fn format_with_passes<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<T>],
// ) -> Vec<T> {
//     let mut working = format_tokens(tokens, options);
//     apply_custom_passes(&mut working, passes);
//     working
// }

// pub fn format_mut<T: FormattableToken>(tokens: &mut Vec<T>) {
//     let formatted = format(tokens.as_slice());
//     *tokens = formatted;
// }

// pub fn format_mut_with_passes<T: FormattableToken>(
//     tokens: &mut Vec<T>,
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<T>],
// ) {
//     let formatted = format_with_passes(tokens.as_slice(), options, passes);
//     *tokens = formatted;
// }

// pub fn format_tokens<T: FormattableToken>(tokens: &[T], options: FormatOptions) -> Vec<T> {
//     format_tokens_profile(tokens, options).0
// }

// pub fn format_tokens_profile<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
// ) -> (Vec<T>, FormatProfile) {
//     let profile = FormatProfile::default();
//     let mut scratch = Vec::new();

//     let mut working = normalize_tokens(tokens, options);

//     if options.bridge_consecutive_verse_markers
//         || options.remove_orphan_empty_verse_before_contentful_verse
//         || options.remove_bridge_verse_enumerators
//     {
//         normalize_verse_sequences_in_place(
//             &mut working,
//             options.bridge_consecutive_verse_markers,
//             options.remove_orphan_empty_verse_before_contentful_verse,
//             options.remove_bridge_verse_enumerators,
//         );
//     }

//     if options.move_chapter_label_after_chapter_marker
//         || options.insert_default_paragraph_after_chapter_intro
//     {
//         if options.move_chapter_label_after_chapter_marker
//             && has_movable_chapter_label(working.as_slice())
//         {
//             rewrite_tokens(
//                 &mut working,
//                 &mut scratch,
//                 move_chapter_labels_after_chapter_into,
//             );
//         }
//         if options.insert_default_paragraph_after_chapter_intro
//             && needs_default_paragraph_after_chapter_intro(working.as_slice())
//         {
//             rewrite_tokens(
//                 &mut working,
//                 &mut scratch,
//                 insert_default_paragraph_after_chapter_intro_into,
//             );
//         }
//     }

//     if options.remove_empty_paragraphs {
//         remove_empty_paragraphs_in_place(&mut working);
//     }

//     if options.insert_structural_linebreaks {
//         rewrite_tokens(
//             &mut working,
//             &mut scratch,
//             insert_structural_linebreaks_into,
//         );
//     }

//     if options.collapse_consecutive_linebreaks {
//         collapse_consecutive_linebreaks_in_place(&mut working);
//     }

//     if options.normalize_marker_whitespace_at_line_start {
//         normalize_marker_whitespace_at_line_start_in_place(&mut working);
//     }

//     (working, profile)
// }

// fn apply_custom_passes<T: FormattableToken>(
//     working: &mut Vec<T>,
//     passes: &[BoxedTokenFormatPass<T>],
// ) {
//     for pass in passes {
//         pass.apply(working);
//     }
// }

// fn normalize_tokens<T: FormattableToken>(tokens: &[T], options: FormatOptions) -> Vec<T> {
//     let mut out = Vec::with_capacity(tokens.len());

//     let mut index = 0usize;
//     while index < tokens.len() {
//         let token = &tokens[index];
//         let prev = out.last();
//         let next = tokens.get(index + 1);

//         if options.recover_malformed_markers
//             && let Some(recovered) = recover_malformed_markers(token)
//         {
//             for recovered_token in recovered {
//                 push_token_merging_text(&mut out, recovered_token);
//             }
//             index += 1;
//             continue;
//         }

//         let mut current = token.clone();

//         if options.ensure_inline_separators {
//             current = ensure_space_between_nodes(current, prev);
//         }

//         if current.kind() == &TokenKind::Text {
//             if options.collapse_whitespace_in_text {
//                 current = collapse_whitespace_in_text_node(current);
//             }
//             if options.remove_duplicate_verse_numbers {
//                 current = remove_duplicate_verse_numbers(current, prev, &out);
//             }
//             if options.normalize_spacing_after_paragraph_markers {
//                 current = normalize_spacing_after_paragraph_markers(current, prev);
//             }
//         }

//         if current.kind() == &TokenKind::Newline
//             && options.remove_unwanted_linebreaks
//             && should_remove_unwanted_linebreak(prev, next, &out, tokens.get(index + 2))
//         {
//             index += 1;
//             continue;
//         }

//         push_token_merging_text(&mut out, current);
//         index += 1;
//     }

//     out
// }

// fn push_token_merging_text<T: FormattableToken>(tokens: &mut Vec<T>, token: T) {
//     if let Some(last) = tokens.last_mut()
//         && token.kind() == &TokenKind::Text
//         && last.kind() == &TokenKind::Text
//         && last.sid() == token.sid()
//         && last.marker() == token.marker()
//     {
//         let mut text = String::with_capacity(last.text().len() + token.text().len());
//         text.push_str(last.text());
//         text.push_str(token.text());
//         last.set_text(text);
//         return;
//     }

//     tokens.push(token);
// }

// fn rewrite_tokens<T, F>(tokens: &mut Vec<T>, scratch: &mut Vec<T>, mut rewrite: F)
// where
//     T: FormattableToken,
//     F: FnMut(&[T], &mut Vec<T>),
// {
//     std::mem::swap(tokens, scratch);
//     tokens.clear();
//     tokens.reserve(scratch.len());
//     rewrite(scratch.as_slice(), tokens);
//     scratch.clear();
// }

// fn recover_malformed_markers<T: FormattableToken>(token: &T) -> Option<Vec<T>> {
//     if token.kind() != &TokenKind::Text {
//         return None;
//     }

//     let text = token.text();
//     let slash_index = text.find('\\')?;
//     let mut chars = text[slash_index + 1..].chars().peekable();
//     let mut marker = String::new();
//     while let Some(ch) = chars.peek().copied() {
//         if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
//             marker.push(ch);
//             chars.next();
//         } else {
//             break;
//         }
//     }

//     if marker.is_empty() {
//         return None;
//     }
//     let rest = &text[slash_index + 1 + marker.len()..];
//     let first_rest = rest.chars().next()?;
//     if !matches!(first_rest, ' ' | '\t') {
//         return None;
//     }
//     if lookup_marker(&marker).kind == MarkerKind::Unknown {
//         return None;
//     }

//     let mut out = Vec::new();
//     if slash_index > 0 {
//         let mut prefix = token.clone();
//         prefix.set_text(text[..slash_index].to_string());
//         out.push(prefix);
//     }

//     let marker_text = format!("\\{marker}");
//     out.push(T::synthetic_like(
//         Some(token),
//         TokenKind::Marker,
//         marker_text,
//         Some(marker.clone()),
//         token.sid().map(ToOwned::to_owned),
//     ));

//     if rest.len() > 1 {
//         let mut suffix = token.clone();
//         suffix.set_text(rest[1..].to_string());
//         out.push(suffix);
//     }

//     Some(out)
// }

// fn ensure_space_between_nodes<T: FormattableToken>(mut token: T, prev: Option<&T>) -> T {
//     if token.kind() == &TokenKind::Newline {
//         return token;
//     }
//     let Some(prev) = prev else {
//         return token;
//     };
//     if prev.kind() == &TokenKind::Newline {
//         return token;
//     }
//     if !is_text_like(prev.kind()) || !is_text_like(token.kind()) {
//         return token;
//     }
//     if is_protected_whitespace_boundary(prev, &token) {
//         return token;
//     }

//     if !ends_with_whitespace(prev.text()) && !starts_with_whitespace(token.text()) {
//         let mut text = String::with_capacity(token.text().len() + 1);
//         text.push(' ');
//         text.push_str(token.text());
//         token.set_text(text);
//     }
//     token
// }

// fn collapse_whitespace_in_text_node<T: FormattableToken>(mut token: T) -> T {
//     if !needs_horizontal_whitespace_collapse(token.text()) {
//         return token;
//     }

//     let text = token.text();
//     let mut collapsed = String::with_capacity(text.len());
//     let mut previous_was_horizontal_ws = false;
//     for ch in text.chars() {
//         if matches!(ch, ' ' | '\t') {
//             if !previous_was_horizontal_ws {
//                 collapsed.push(' ');
//             }
//             previous_was_horizontal_ws = true;
//         } else {
//             previous_was_horizontal_ws = false;
//             collapsed.push(ch);
//         }
//     }
//     token.set_text(collapsed);
//     token
// }

// fn needs_horizontal_whitespace_collapse(text: &str) -> bool {
//     let mut previous_was_space = false;
//     for ch in text.chars() {
//         match ch {
//             '\t' => return true,
//             ' ' => {
//                 if previous_was_space {
//                     return true;
//                 }
//                 previous_was_space = true;
//             }
//             _ => previous_was_space = false,
//         }
//     }
//     false
// }

// fn remove_duplicate_verse_numbers<T: FormattableToken>(
//     mut token: T,
//     prev: Option<&T>,
//     cleaned: &[T],
// ) -> T {
//     let Some(prev) = prev else {
//         return token;
//     };
//     if prev.kind() != &TokenKind::Number {
//         return token;
//     }
//     if !number_belongs_to_marker(cleaned, cleaned.len().saturating_sub(1), "v") {
//         return token;
//     }

//     let verse_number = prev.text().trim();
//     if verse_number.is_empty() {
//         return token;
//     }

//     let trimmed_start = token.text().trim_start_matches([' ', '\t']);
//     if let Some(remainder) = trimmed_start.strip_prefix(verse_number) {
//         let leading_len = token.text().len() - trimmed_start.len();
//         let leading = &token.text()[..leading_len];
//         let mut text = String::with_capacity(leading.len() + remainder.len());
//         text.push_str(leading);
//         text.push_str(remainder);
//         token.set_text(text);
//     }
//     token
// }

// fn normalize_spacing_after_paragraph_markers<T: FormattableToken>(
//     mut token: T,
//     prev: Option<&T>,
// ) -> T {
//     let Some(prev) = prev else {
//         return token;
//     };
//     if prev.kind() != &TokenKind::Marker {
//         return token;
//     }
//     let Some(marker) = prev.marker() else {
//         return token;
//     };
//     if !linebreak_before_marker(marker) {
//         return token;
//     }

//     let rest = token.text().trim_start_matches(' ');
//     if rest.len() != token.text().len() {
//         let mut text = String::with_capacity(rest.len() + 1);
//         text.push(' ');
//         text.push_str(rest);
//         token.set_text(text);
//     }
//     token
// }

// fn should_remove_unwanted_linebreak<T: FormattableToken>(
//     prev: Option<&T>,
//     next: Option<&T>,
//     cleaned: &[T],
//     next_after_next: Option<&T>,
// ) -> bool {
//     let prev_marker = prev
//         .filter(|token| token.kind() == &TokenKind::Marker)
//         .and_then(|token| token.marker());
//     let next_is_marker = next.is_some_and(|token| token.kind() == &TokenKind::Marker);
//     let next_marker = next
//         .filter(|token| token.kind() == &TokenKind::Marker)
//         .and_then(|t| t.marker());

//     if let Some(marker) = prev_marker {
//         if linebreak_before_and_after_marker(marker) {
//             return false;
//         }
//         if linebreak_before_if_next_marker(marker) {
//             return !next_is_marker;
//         }
//         if linebreak_before_marker(marker) {
//             return true;
//         }
//     }

//     if next_marker == Some("v") {
//         if let Some(prev) = prev
//             && prev.kind() == &TokenKind::Number
//             && number_belongs_to_marker(cleaned, cleaned.len().saturating_sub(1), "c")
//         {
//             return false;
//         }
//         if next_after_next.is_some_and(|token| token.kind() == &TokenKind::Number) {
//             return true;
//         }
//     }

//     false
// }

// fn normalize_verse_sequences_in_place<T: FormattableToken>(
//     tokens: &mut Vec<T>,
//     enable_bridge: bool,
//     enable_orphan_cleanup: bool,
//     enable_enumerator_cleanup: bool,
// ) {
//     let mut index = 0usize;
//     while index + 1 < tokens.len() {
//         if !is_immediate_verse_pair(tokens, index) {
//             index += 1;
//             continue;
//         }

//         if enable_bridge && bridge_verse_run(tokens, index) {
//             if enable_enumerator_cleanup {
//                 cleanup_bridge_enumerator_at(tokens, index);
//             }
//             continue;
//         }

//         if enable_orphan_cleanup
//             && let Some(next_marker_index) = orphan_next_marker_index(tokens, index)
//         {
//             tokens.drain(index..next_marker_index);
//             continue;
//         }

//         if enable_enumerator_cleanup {
//             cleanup_bridge_enumerator_at(tokens, index);
//         }

//         index += 1;
//     }
// }

// fn is_immediate_verse_pair<T: FormattableToken>(tokens: &[T], index: usize) -> bool {
//     tokens
//         .get(index)
//         .is_some_and(|token| token.kind() == &TokenKind::Marker && token.marker() == Some("v"))
//         && tokens
//             .get(index + 1)
//             .is_some_and(|token| token.kind() == &TokenKind::Number)
// }

// fn bridge_verse_run<T: FormattableToken>(tokens: &mut Vec<T>, index: usize) -> bool {
//     let Some(first_verse) = tokens
//         .get(index + 1)
//         .and_then(|token| parse_plain_verse(token.text()))
//     else {
//         return false;
//     };

//     let mut end_verse = first_verse;
//     let mut scan = index + 2;

//     while scan + 1 < tokens.len() {
//         let mut candidate_marker_index = scan;
//         while candidate_marker_index < tokens.len()
//             && tokens[candidate_marker_index].kind() == &TokenKind::Text
//             && tokens[candidate_marker_index].text().trim().is_empty()
//         {
//             candidate_marker_index += 1;
//         }

//         if !is_immediate_verse_pair(tokens, candidate_marker_index) {
//             break;
//         }

//         let Some(next_verse) = tokens
//             .get(candidate_marker_index + 1)
//             .and_then(|token| parse_plain_verse(token.text()))
//         else {
//             break;
//         };
//         if next_verse != end_verse + 1 {
//             break;
//         }

//         end_verse = next_verse;
//         scan = candidate_marker_index + 2;
//     }

//     if end_verse == first_verse {
//         return false;
//     }

//     let range = bridge_range_string(first_verse, end_verse);
//     let updated = with_original_spacing(tokens[index + 1].text(), &range);
//     tokens[index + 1].set_text(updated);
//     tokens.drain(index + 2..scan);
//     true
// }

// fn orphan_next_marker_index<T: FormattableToken>(tokens: &[T], index: usize) -> Option<usize> {
//     let mut next_marker_index = index + 2;
//     while next_marker_index < tokens.len()
//         && tokens[next_marker_index].kind() == &TokenKind::Text
//         && tokens[next_marker_index].text().trim().is_empty()
//     {
//         next_marker_index += 1;
//     }

//     if !is_immediate_verse_pair(tokens, next_marker_index) {
//         return None;
//     }

//     let next_text = tokens.get(next_marker_index + 2)?;
//     if next_text.kind() == &TokenKind::Text && !next_text.text().trim().is_empty() {
//         Some(next_marker_index)
//     } else {
//         None
//     }
// }

// fn cleanup_bridge_enumerator_at<T: FormattableToken>(tokens: &mut [T], index: usize) {
//     if !is_immediate_verse_pair(tokens, index) {
//         return;
//     }
//     let Some(range_token) = tokens.get(index + 1) else {
//         return;
//     };
//     let Some(next) = tokens.get(index + 2) else {
//         return;
//     };
//     if next.kind() != &TokenKind::Text {
//         return;
//     }
//     let Some((start, end)) = parse_bridge_range(range_token.text()) else {
//         return;
//     };
//     let updated = strip_bridge_enumerators(next.text(), start, end);
//     if updated != next.text() {
//         tokens[index + 2].set_text(updated);
//     }
// }

// fn insert_default_paragraph_after_chapter_intro_into<T: FormattableToken>(
//     tokens: &[T],
//     out: &mut Vec<T>,
// ) {
//     let mut in_chapter_intro = false;
//     let mut saw_para_marker_in_intro = false;
//     let mut saw_chapter_marker = false;
//     let mut saw_chapter_number = false;

//     for token in tokens {
//         let is_chapter_marker = token.kind() == &TokenKind::Marker && token.marker() == Some("c");
//         let is_verse_marker = token.kind() == &TokenKind::Marker && token.marker() == Some("v");
//         let is_paragraph_marker = token.kind() == &TokenKind::Marker
//             && token
//                 .marker()
//                 .is_some_and(is_valid_paragraph_or_heading_marker);

//         if is_chapter_marker {
//             saw_chapter_marker = true;
//             saw_chapter_number = false;
//             in_chapter_intro = false;
//             saw_para_marker_in_intro = false;
//             out.push(token.clone());
//             continue;
//         }

//         if saw_chapter_marker && !saw_chapter_number {
//             if token.kind() == &TokenKind::Number {
//                 saw_chapter_number = true;
//             }
//             out.push(token.clone());
//             continue;
//         }

//         if saw_chapter_marker && saw_chapter_number && !in_chapter_intro {
//             in_chapter_intro = true;
//         }

//         if in_chapter_intro {
//             if is_paragraph_marker {
//                 saw_para_marker_in_intro = true;
//             }

//             if is_verse_marker && !saw_para_marker_in_intro {
//                 out.push(T::synthetic_like(
//                     Some(token),
//                     TokenKind::Marker,
//                     "\\p".to_string(),
//                     Some("p".to_string()),
//                     token.sid().map(ToOwned::to_owned),
//                 ));
//                 saw_para_marker_in_intro = true;
//             }

//             if is_verse_marker {
//                 in_chapter_intro = false;
//             }
//         }

//         out.push(token.clone());
//     }
// }

// fn has_movable_chapter_label<T: FormattableToken>(tokens: &[T]) -> bool {
//     let mut index = 0usize;
//     while index < tokens.len() {
//         let token = &tokens[index];
//         if token.kind() != &TokenKind::Marker || token.marker() != Some("cl") {
//             index += 1;
//             continue;
//         }

//         let mut chapter_marker_index = index + 1;
//         while chapter_marker_index < tokens.len() {
//             let probe = &tokens[chapter_marker_index];
//             match probe.kind() {
//                 TokenKind::Newline | TokenKind::Text => {
//                     chapter_marker_index += 1;
//                 }
//                 TokenKind::Marker if probe.marker() == Some("c") => return true,
//                 _ => break,
//             }
//         }

//         index += 1;
//     }

//     false
// }

// fn needs_default_paragraph_after_chapter_intro<T: FormattableToken>(tokens: &[T]) -> bool {
//     let mut saw_chapter_marker = false;
//     let mut saw_chapter_number = false;
//     let mut in_chapter_intro = false;
//     let mut saw_para_marker_in_intro = false;

//     for token in tokens {
//         let is_chapter_marker = token.kind() == &TokenKind::Marker && token.marker() == Some("c");
//         let is_verse_marker = token.kind() == &TokenKind::Marker && token.marker() == Some("v");
//         let is_paragraph_marker = token.kind() == &TokenKind::Marker
//             && token
//                 .marker()
//                 .is_some_and(is_valid_paragraph_or_heading_marker);

//         if is_chapter_marker {
//             saw_chapter_marker = true;
//             saw_chapter_number = false;
//             in_chapter_intro = false;
//             saw_para_marker_in_intro = false;
//             continue;
//         }

//         if saw_chapter_marker && !saw_chapter_number {
//             if token.kind() == &TokenKind::Number {
//                 saw_chapter_number = true;
//             }
//             continue;
//         }

//         if saw_chapter_marker && saw_chapter_number && !in_chapter_intro {
//             in_chapter_intro = true;
//         }

//         if !in_chapter_intro {
//             continue;
//         }

//         if is_paragraph_marker {
//             saw_para_marker_in_intro = true;
//             continue;
//         }

//         if is_verse_marker {
//             return !saw_para_marker_in_intro;
//         }
//     }

//     false
// }

// fn move_chapter_labels_after_chapter_into<T: FormattableToken>(tokens: &[T], out: &mut Vec<T>) {
//     let mut index = 0usize;
//     while index < tokens.len() {
//         let token = &tokens[index];
//         let is_chapter_label = token.kind() == &TokenKind::Marker && token.marker() == Some("cl");
//         if !is_chapter_label {
//             out.push(token.clone());
//             index += 1;
//             continue;
//         }

//         let mut chapter_marker_index = index + 1;
//         let mut movable = true;
//         while chapter_marker_index < tokens.len() {
//             let probe = &tokens[chapter_marker_index];
//             match probe.kind() {
//                 TokenKind::Newline | TokenKind::Text => {
//                     chapter_marker_index += 1;
//                 }
//                 TokenKind::Marker if probe.marker() == Some("c") => {
//                     break;
//                 }
//                 _ => {
//                     movable = false;
//                     break;
//                 }
//             }
//         }

//         if !movable || chapter_marker_index >= tokens.len() {
//             out.push(token.clone());
//             index += 1;
//             continue;
//         }

//         let mut chapter_block_end = chapter_marker_index + 1;
//         while chapter_block_end < tokens.len() {
//             let probe = &tokens[chapter_block_end];
//             match probe.kind() {
//                 TokenKind::Number => {
//                     chapter_block_end += 1;
//                     break;
//                 }
//                 _ => break,
//             }
//         }

//         out.extend(
//             tokens[chapter_marker_index..chapter_block_end]
//                 .iter()
//                 .cloned(),
//         );
//         out.extend(tokens[index..chapter_marker_index].iter().cloned());
//         index = chapter_block_end;
//     }
// }

// fn insert_structural_linebreaks_into<T: FormattableToken>(tokens: &[T], out: &mut Vec<T>) {
//     for (index, token) in tokens.iter().enumerate() {
//         let prev_out = out.last();
//         let next_in = tokens.get(index + 1);

//         if token.kind() == &TokenKind::Marker
//             && token.marker().is_some_and(linebreak_before_marker)
//             && prev_out.is_some()
//             && !prev_out.is_some_and(|t: &T| t.kind() == &TokenKind::Newline)
//         {
//             out.push(new_newline_like(token));
//         }

//         out.push(token.clone());

//         if token.kind() == &TokenKind::Marker
//             && let Some(marker) = token.marker()
//         {
//             if linebreak_before_if_next_marker(marker) {
//                 if next_in.is_some_and(|t| t.kind() == &TokenKind::Marker)
//                     && !next_in.is_some_and(|t| t.kind() == &TokenKind::Newline)
//                 {
//                     out.push(new_newline_like(token));
//                 }
//                 continue;
//             }

//             if linebreak_before_and_after_marker(marker)
//                 && !next_in.is_some_and(|t| t.kind() == &TokenKind::Newline)
//             {
//                 out.push(new_newline_like(token));
//             }
//         }

//         if token.kind() == &TokenKind::Number
//             && number_belongs_to_marker(tokens, index, "c")
//             && !next_in.is_some_and(|t| t.kind() == &TokenKind::Newline)
//         {
//             out.push(new_newline_like(token));
//         }
//     }
// }

// fn collapse_consecutive_linebreaks_in_place<T: FormattableToken>(tokens: &mut Vec<T>) {
//     let mut write = 0usize;
//     let mut previous_was_linebreak = false;

//     for read in 0..tokens.len() {
//         let is_linebreak = tokens[read].kind() == &TokenKind::Newline;
//         if is_linebreak && previous_was_linebreak {
//             continue;
//         }
//         if write != read {
//             tokens.swap(write, read);
//         }
//         previous_was_linebreak = is_linebreak;
//         write += 1;
//     }

//     tokens.truncate(write);
// }

// fn normalize_marker_whitespace_at_line_start_in_place<T: FormattableToken>(tokens: &mut [T]) {
//     for index in 0..tokens.len() {
//         if tokens[index].kind() != &TokenKind::Marker {
//             continue;
//         }
//         let at_line_start = index == 0 || tokens[index - 1].kind() == &TokenKind::Newline;
//         if !at_line_start {
//             continue;
//         }
//         let trimmed = tokens[index].text().trim_start();
//         if trimmed.len() == tokens[index].text().len() {
//             continue;
//         }
//         tokens[index].set_text(trimmed.to_string());
//     }
// }

// fn new_newline_like<T: FormattableToken>(anchor: &T) -> T {
//     T::synthetic_like(
//         Some(anchor),
//         TokenKind::Newline,
//         "\n".to_string(),
//         None,
//         anchor.sid().map(ToOwned::to_owned),
//     )
// }

// fn is_text_like(kind: &TokenKind) -> bool {
//     matches!(
//         kind,
//         TokenKind::Marker | TokenKind::EndMarker | TokenKind::Number | TokenKind::Text
//     )
// }

// fn is_protected_whitespace_boundary<T: FormattableToken>(prev: &T, curr: &T) -> bool {
//     is_char_or_note_markerish(prev) || is_char_or_note_markerish(curr)
// }

// fn is_char_or_note_markerish<T: FormattableToken>(token: &T) -> bool {
//     if !matches!(
//         token.kind(),
//         TokenKind::Marker | TokenKind::EndMarker | TokenKind::Milestone | TokenKind::MilestoneEnd
//     ) {
//         return false;
//     }
//     let Some(marker) = token.marker() else {
//         return false;
//     };
//     matches!(
//         lookup_marker(normalized_marker_name(marker)).kind,
//         MarkerKind::Character
//             | MarkerKind::Note
//             | MarkerKind::MilestoneStart
//             | MarkerKind::MilestoneEnd
//     )
// }

// fn linebreak_before_and_after_marker(marker: &str) -> bool {
//     contains_marker(LINEBREAK_BEFORE_AND_AFTER_MARKERS, marker)
//         || unknown_marker_defaults_to_own_line(marker)
// }

// fn linebreak_before_if_next_marker(marker: &str) -> bool {
//     contains_marker(POETRY_MARKERS, marker)
// }

// fn linebreak_before_marker(marker: &str) -> bool {
//     linebreak_before_and_after_marker(marker)
//         || contains_marker(LINEBREAK_BEFORE_ONLY_MARKERS, marker)
//         || linebreak_before_if_next_marker(marker)
// }

// fn contains_marker(markers: &[&str], marker: &str) -> bool {
//     markers.contains(&marker)
// }

// fn is_empty_paragraph_candidate(marker: &str) -> bool {
//     matches!(
//         marker,
//         "p" | "m"
//             | "po"
//             | "pr"
//             | "cls"
//             | "pmo"
//             | "pm"
//             | "pmc"
//             | "pmr"
//             | "pi"
//             | "pi1"
//             | "pi2"
//             | "pi3"
//             | "mi"
//             | "nb"
//             | "pc"
//             | "ph"
//             | "ph1"
//             | "ph2"
//             | "ph3"
//             | "b"
//             | "pb"
//             | "q"
//             | "q1"
//             | "q2"
//             | "q3"
//             | "q4"
//             | "qr"
//             | "qc"
//             | "qa"
//             | "qm"
//             | "qm1"
//             | "qm2"
//             | "qm3"
//             | "qd"
//             | "lh"
//             | "li"
//             | "li1"
//             | "li2"
//             | "li3"
//             | "li4"
//             | "lf"
//             | "lim"
//             | "lim1"
//             | "lim2"
//             | "lim3"
//     )
// }

// fn is_empty_paragraph_boundary_marker(marker: &str) -> bool {
//     if is_empty_paragraph_candidate(marker) {
//         return true;
//     }
//     matches!(
//         lookup_marker(marker).kind,
//         MarkerKind::Header
//             | MarkerKind::Chapter
//             | MarkerKind::Periph
//             | MarkerKind::SidebarStart
//             | MarkerKind::TableRow
//             | MarkerKind::Unknown
//     )
// }

// fn remove_empty_paragraphs_in_place<T: FormattableToken>(tokens: &mut Vec<T>) {
//     let mut write = 0usize;
//     let mut read = 0usize;

//     while read < tokens.len() {
//         let token = &tokens[read];
//         let Some(marker) = (token.kind() == &TokenKind::Marker)
//             .then(|| token.marker())
//             .flatten()
//         else {
//             if write != read {
//                 tokens.swap(write, read);
//             }
//             write += 1;
//             read += 1;
//             continue;
//         };

//         if !is_empty_paragraph_candidate(marker) {
//             if write != read {
//                 tokens.swap(write, read);
//             }
//             write += 1;
//             read += 1;
//             continue;
//         }

//         let mut probe = read + 1;
//         let mut remove_until = None;
//         while probe < tokens.len() {
//             let next = &tokens[probe];
//             match next.kind() {
//                 TokenKind::Newline | TokenKind::OptBreak => {
//                     probe += 1;
//                 }
//                 TokenKind::Text if next.text().trim().is_empty() => {
//                     probe += 1;
//                 }
//                 TokenKind::Marker
//                     if next
//                         .marker()
//                         .is_some_and(is_empty_paragraph_boundary_marker) =>
//                 {
//                     remove_until = Some(probe);
//                     break;
//                 }
//                 _ => break,
//             }
//         }

//         if let Some(next_boundary) = remove_until {
//             read = next_boundary;
//             continue;
//         }

//         if write != read {
//             tokens.swap(write, read);
//         }
//         write += 1;
//         read += 1;
//     }

//     tokens.truncate(write);
// }

// fn unknown_marker_defaults_to_own_line(marker: &str) -> bool {
//     !marker.starts_with('z') && lookup_marker(marker).kind == MarkerKind::Unknown
// }

// fn is_valid_paragraph_or_heading_marker(marker: &str) -> bool {
//     matches!(
//         lookup_marker(marker).kind,
//         MarkerKind::Paragraph | MarkerKind::Header | MarkerKind::Meta
//     )
// }

// fn parse_plain_verse(text: &str) -> Option<u32> {
//     let trimmed = text.trim();
//     if !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
//         return None;
//     }
//     trimmed.parse().ok()
// }

// fn bridge_range_string(start: u32, end: u32) -> String {
//     let start = start.to_string();
//     let end = end.to_string();
//     let mut text = String::with_capacity(start.len() + end.len() + 1);
//     text.push_str(&start);
//     text.push('-');
//     text.push_str(&end);
//     text
// }

// fn with_original_spacing(original: &str, normalized: &str) -> String {
//     let leading_len = original.len() - original.trim_start().len();
//     let trailing_len = original.len() - original.trim_end().len();
//     let leading = &original[..leading_len];
//     let trailing = &original[original.len() - trailing_len..];
//     let mut text = String::with_capacity(leading.len() + normalized.len() + trailing.len());
//     text.push_str(leading);
//     text.push_str(normalized);
//     text.push_str(trailing);
//     text
// }

// fn parse_bridge_range(text: &str) -> Option<(u32, u32)> {
//     let trimmed = text.trim();
//     let (start, end) = trimmed.split_once('-')?;
//     let start: u32 = start.trim().parse().ok()?;
//     let end: u32 = end.trim().parse().ok()?;
//     if start > end {
//         return None;
//     }
//     Some((start, end))
// }

// fn strip_bridge_enumerators(text: &str, start: u32, end: u32) -> String {
//     let bytes = text.as_bytes();
//     let mut index = 0usize;
//     let mut last_copied = 0usize;
//     let mut output = String::with_capacity(text.len());

//     while index < bytes.len() {
//         let current = bytes[index];
//         let at_boundary = index == 0 || current_boundary_byte(bytes[index - 1]);
//         if !at_boundary || !current.is_ascii_digit() {
//             index += 1;
//             continue;
//         }

//         let digit_start = index;
//         let mut verse_num = 0u32;
//         while index < bytes.len() && bytes[index].is_ascii_digit() {
//             verse_num = verse_num
//                 .saturating_mul(10)
//                 .saturating_add((bytes[index] - b'0') as u32);
//             index += 1;
//         }

//         let mut after_digits = index;
//         while after_digits < bytes.len() && bytes[after_digits].is_ascii_whitespace() {
//             after_digits += 1;
//         }
//         if after_digits >= bytes.len() || !is_enumerator_punctuation(bytes[after_digits] as char) {
//             index = after_digits;
//             continue;
//         }

//         let mut after_enum = after_digits + 1;
//         while after_enum < bytes.len() && bytes[after_enum].is_ascii_whitespace() {
//             after_enum += 1;
//         }

//         if verse_num >= start && verse_num <= end {
//             output.push_str(&text[last_copied..digit_start]);
//             last_copied = after_enum;
//         }

//         index = after_enum;
//     }

//     if last_copied == 0 {
//         return text.to_string();
//     }

//     output.push_str(&text[last_copied..]);
//     output
// }

// fn current_boundary_byte(byte: u8) -> bool {
//     byte.is_ascii_whitespace() || byte == b'('
// }

// fn is_enumerator_punctuation(ch: char) -> bool {
//     matches!(
//         ch,
//         '!' | '"'
//             | '#'
//             | '$'
//             | '%'
//             | '&'
//             | '\''
//             | '('
//             | ')'
//             | '*'
//             | '+'
//             | ','
//             | '.'
//             | '/'
//             | ':'
//             | ';'
//             | '<'
//             | '='
//             | '>'
//             | '?'
//             | '@'
//             | '['
//             | '\\'
//             | ']'
//             | '^'
//             | '_'
//             | '`'
//             | '{'
//             | '|'
//             | '}'
//             | '~'
//             | '-'
//     )
// }

// fn number_belongs_to_marker<T: FormattableToken>(tokens: &[T], index: usize, marker: &str) -> bool {
//     if index == 0 {
//         return false;
//     }
//     let mut cursor = index;
//     while cursor > 0 {
//         cursor -= 1;
//         match tokens[cursor].kind() {
//             TokenKind::Newline => continue,
//             TokenKind::Marker => return tokens[cursor].marker() == Some(marker),
//             _ => return false,
//         }
//     }
//     false
// }

// fn starts_with_whitespace(text: &str) -> bool {
//     text.chars().next().is_some_and(char::is_whitespace)
// }

// fn ends_with_whitespace(text: &str) -> bool {
//     text.chars().last().is_some_and(char::is_whitespace)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[derive(Debug, Clone, PartialEq, Eq)]
//     struct EditorToken {
//         kind: TokenKind,
//         text: String,
//         marker: Option<String>,
//         sid: Option<String>,
//         id: String,
//         lane: u8,
//     }

//     impl FormattableToken for EditorToken {
//         fn id(&self) -> Option<&str> {
//             Some(&self.id)
//         }

//         fn set_id(&mut self, id: String) {
//             self.id = id;
//         }

//         fn kind(&self) -> &TokenKind {
//             &self.kind
//         }

//         fn set_kind(&mut self, kind: TokenKind) {
//             self.kind = kind;
//         }

//         fn text(&self) -> &str {
//             &self.text
//         }

//         fn set_text(&mut self, text: String) {
//             self.text = text;
//         }

//         fn marker(&self) -> Option<&str> {
//             self.marker.as_deref()
//         }

//         fn set_marker(&mut self, marker: Option<String>) {
//             self.marker = marker;
//         }

//         fn sid(&self) -> Option<&str> {
//             self.sid.as_deref()
//         }

//         fn set_sid(&mut self, sid: Option<String>) {
//             self.sid = sid;
//         }

//         fn synthetic_like(
//             anchor: Option<&Self>,
//             kind: TokenKind,
//             text: String,
//             marker: Option<String>,
//             sid: Option<String>,
//         ) -> Self {
//             let lane = anchor.map(|token| token.lane).unwrap_or(0);
//             Self {
//                 kind,
//                 text,
//                 marker,
//                 sid,
//                 id: String::new(),
//                 lane,
//             }
//         }
//     }

//     fn token(kind: TokenKind, text: &str, marker: Option<&str>) -> EditorToken {
//         EditorToken {
//             kind,
//             text: text.to_string(),
//             marker: marker.map(ToOwned::to_owned),
//             sid: None,
//             id: String::new(),
//             lane: 1,
//         }
//     }

//     #[test]
//     fn preserves_unknown_metadata() {
//         let tokens = vec![EditorToken {
//             kind: TokenKind::Text,
//             text: "".to_string(),
//             marker: None,
//             sid: None,
//             id: "custom".to_string(),
//             lane: 7,
//         }];

//         let result = format_tokens(&tokens, FormatOptions::default());

//         assert_eq!(result.len(), 1);
//         assert_eq!(result[0].lane, 7);
//         assert_eq!(result[0].id, "custom");
//     }

//     #[test]
//     fn default_format_bridges_consecutive_verse_markers_into_range() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "2", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "3", None),
//             token(TokenKind::Text, "  asdf asdf", None),
//         ];

//         let result = format(&tokens);

//         assert_eq!(result.len(), 3);
//         assert_eq!(result[1].text, " 1-3");
//         assert_eq!(result[2].text, " asdf asdf");
//     }

//     #[test]
//     fn excluding_recovery_and_duplicate_cleanup_still_bridges_consecutive_verse_markers() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "2", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "3", None),
//             token(TokenKind::Text, "  asdf asdf", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );

//         assert_eq!(result.len(), 3);
//         assert_eq!(result[1].text, " 1-3");
//         assert_eq!(result[2].text, " asdf asdf");
//     }

//     #[test]
//     fn default_format_removes_duplicate_verse_number_then_bridges() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "14", None),
//             token(TokenKind::Text, " 14", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "15", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         let result = format(&tokens);

//         assert_eq!(result.len(), 3);
//         assert_eq!(result[1].text, " 14-15");
//         assert_eq!(result[2].text, " text");
//     }

//     #[test]
//     fn inserts_default_p_before_first_verse_after_chapter_intro() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\c", Some("c")),
//             token(TokenKind::Number, "1", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1", None),
//             token(TokenKind::Text, " In the beginning", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );
//         let p_index = result.iter().position(|token| token.marker() == Some("p"));
//         let v_index = result.iter().position(|token| token.marker() == Some("v"));

//         assert!(p_index.is_some());
//         assert!(v_index.is_some());
//         assert!(p_index < v_index);
//     }

//     #[test]
//     fn moves_cl_after_c_marker() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\cl", Some("cl")),
//             token(TokenKind::Text, " Chapter label", None),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Marker, "\\c", Some("c")),
//             token(TokenKind::Number, " 1", None),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, " 1", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );
//         let c_index = result
//             .iter()
//             .position(|token| token.marker() == Some("c"))
//             .expect("expected chapter marker");
//         let cl_index = result
//             .iter()
//             .position(|token| token.marker() == Some("cl"))
//             .expect("expected chapter label");

//         assert!(c_index < cl_index);
//     }

//     #[test]
//     fn unknown_s5_marker_defaults_to_its_own_line() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\s5", Some("s5")),
//             token(TokenKind::Marker, "\\c", Some("c")),
//             token(TokenKind::Number, " 1", None),
//         ];

//         let result = format(&tokens);
//         let rendered = result
//             .iter()
//             .map(|token| token.text.as_str())
//             .collect::<String>();

//         assert!(rendered.starts_with("\\s5\n"));
//     }

//     #[test]
//     fn unknown_z_marker_does_not_default_to_its_own_line() {
//         let tokens = vec![
//             token(TokenKind::Text, " text", None),
//             token(TokenKind::Marker, "\\zfoo", Some("zfoo")),
//             token(TokenKind::Text, " tail", None),
//         ];

//         let result = format(&tokens);
//         let rendered = result
//             .iter()
//             .map(|token| token.text.as_str())
//             .collect::<String>();

//         assert!(!rendered.contains("\n\\zfoo"));
//     }

//     #[test]
//     fn remove_empty_paragraphs_option_drops_redundant_m_before_q() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\m", Some("m")),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Marker, "\\q", Some("q")),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, " 1", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::only(&[FormatRule::RemoveEmptyParagraphs]),
//         );
//         let rendered = result
//             .iter()
//             .map(|token| token.text.as_str())
//             .collect::<String>();

//         assert_eq!(rendered, "\\q\n\\v 1 text");
//     }

//     #[test]
//     fn remove_empty_paragraphs_option_keeps_p_before_verse_content() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\p", Some("p")),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, " 1", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::only(&[FormatRule::RemoveEmptyParagraphs]),
//         );
//         let rendered = result
//             .iter()
//             .map(|token| token.text.as_str())
//             .collect::<String>();

//         assert_eq!(rendered, "\\p\n\\v 1 text");
//     }

//     #[test]
//     fn removes_leading_enumerator_from_bridge_text() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1-3", None),
//             token(TokenKind::Text, " 1. James, a servant...", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );
//         assert_eq!(result[1].text, " 1-3");
//         assert_eq!(result[2].text, " James, a servant...");
//     }

//     #[test]
//     fn keeps_enumerator_outside_bridge_range() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "2", None),
//             token(TokenKind::Text, " 3. Outside range text", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );

//         assert_eq!(result[1].text, " 1-2");
//         assert_eq!(result[2].text, " 3. Outside range text");
//     }

//     #[test]
//     fn removes_inline_enumerators_inside_bridge_range() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "1-3", None),
//             token(
//                 TokenKind::Text,
//                 " 1. James... 2. Consider it pure joy... 3. because you know...",
//                 None,
//             ),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );
//         assert_eq!(
//             result[2].text,
//             " James... Consider it pure joy... because you know..."
//         );
//     }

//     #[test]
//     fn drops_orphan_empty_verse_before_contentful_verse() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "5", None),
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "4", None),
//             token(TokenKind::Text, " Let perseverance finish", None),
//         ];

//         let result = format_tokens(
//             &tokens,
//             FormatOptions::excluding(&[
//                 FormatRule::RecoverMalformedMarkers,
//                 FormatRule::RemoveDuplicateVerseNumbers,
//             ]),
//         );

//         assert_eq!(result.len(), 3);
//         assert_eq!(result[1].text, " 4");
//         assert_eq!(result[2].text, " Let perseverance finish");
//     }

//     #[test]
//     fn does_not_inject_space_around_char_end_marker_near_punctuation() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\v", Some("v")),
//             token(TokenKind::Number, "9", None),
//             token(TokenKind::Text, " and mankind is not respected.", None),
//             token(TokenKind::Marker, "\\f", Some("f")),
//             token(TokenKind::Text, "+", None),
//             token(TokenKind::Marker, "\\ft", Some("ft")),
//             token(TokenKind::Text, " word for ", None),
//             token(TokenKind::Marker, "\\fqa", Some("fqa")),
//             token(TokenKind::Text, "cities", None),
//             token(TokenKind::EndMarker, "\\fqa*", Some("fqa")),
//             token(TokenKind::Text, ",", None),
//             token(TokenKind::Text, " but this is likely corrupted. ", None),
//             token(TokenKind::EndMarker, "\\f*", Some("f")),
//         ];

//         let result = format(&tokens);
//         let serialized = result
//             .iter()
//             .map(|token| token.text.as_str())
//             .collect::<String>();

//         assert!(!serialized.contains("\\fqa* ,"));
//         assert!(serialized.contains("\\fqa*,"));
//     }

//     #[test]
//     fn collapses_multiple_linebreaks_to_one() {
//         let tokens = vec![
//             token(TokenKind::Marker, "\\p", Some("p")),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         let result = format(&tokens);
//         assert_eq!(
//             result
//                 .iter()
//                 .filter(|token| token.kind() == &TokenKind::Newline)
//                 .count(),
//             1
//         );
//     }

//     struct ReplaceTextPass {
//         label: &'static str,
//         from: &'static str,
//         to: &'static str,
//     }

//     impl TokenFormatPass<EditorToken> for ReplaceTextPass {
//         fn label(&self) -> &str {
//             self.label
//         }

//         fn apply(&self, tokens: &mut Vec<EditorToken>) {
//             for token in tokens.iter_mut() {
//                 if token.kind == TokenKind::Text && token.text.contains(self.from) {
//                     token.text = token.text.replace(self.from, self.to);
//                 }
//             }
//         }
//     }

//     #[test]
//     fn format_mut_applies_spec_canonical_changes_in_place() {
//         let mut tokens = vec![
//             token(TokenKind::Marker, "\\p", Some("p")),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Newline, "\n", None),
//             token(TokenKind::Text, " text", None),
//         ];

//         format_mut(&mut tokens);

//         assert_eq!(
//             tokens
//                 .iter()
//                 .filter(|token| token.kind() == &TokenKind::Newline)
//                 .count(),
//             1
//         );
//     }

//     #[test]
//     fn format_with_passes_runs_custom_passes_in_order() {
//         let tokens = vec![token(TokenKind::Text, " alpha", None)];
//         let passes: Vec<BoxedTokenFormatPass<EditorToken>> = vec![
//             Box::new(ReplaceTextPass {
//                 label: "alpha-to-beta",
//                 from: "alpha",
//                 to: "beta",
//             }),
//             Box::new(ReplaceTextPass {
//                 label: "beta-to-gamma",
//                 from: "beta",
//                 to: "gamma",
//             }),
//         ];

//         let result = format_with_passes(&tokens, FormatOptions::default(), &passes);

//         assert_eq!(result[0].text, " gamma");
//         assert_eq!(tokens[0].text, " alpha");
//     }
// }
