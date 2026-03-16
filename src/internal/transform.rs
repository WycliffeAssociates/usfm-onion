// use crate::internal::format::{
//     BoxedTokenFormatPass, FormatOptions, FormattableToken, format_tokens,
// };
// use crate::internal::lint::MessageParams;
// use crate::model::token::TokenKind;
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TokenTemplate {
//     pub kind: TokenKind,
//     pub text: String,
//     pub marker: Option<String>,
//     pub sid: Option<String>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum TokenFix {
//     ReplaceToken {
//         code: String,
//         label: String,
//         label_params: MessageParams,
//         target_token_id: String,
//         replacements: Vec<TokenTemplate>,
//     },
//     DeleteToken {
//         code: String,
//         label: String,
//         label_params: MessageParams,
//         target_token_id: String,
//     },
//     InsertAfter {
//         code: String,
//         label: String,
//         label_params: MessageParams,
//         target_token_id: String,
//         insert: Vec<TokenTemplate>,
//     },
// }

// impl TokenFix {
//     pub fn code(&self) -> &str {
//         match self {
//             TokenFix::ReplaceToken { code, .. }
//             | TokenFix::DeleteToken { code, .. }
//             | TokenFix::InsertAfter { code, .. } => code,
//         }
//     }

//     pub fn label(&self) -> &str {
//         match self {
//             TokenFix::ReplaceToken { label, .. }
//             | TokenFix::DeleteToken { label, .. }
//             | TokenFix::InsertAfter { label, .. } => label,
//         }
//     }

//     pub fn label_params(&self) -> &MessageParams {
//         match self {
//             TokenFix::ReplaceToken { label_params, .. }
//             | TokenFix::DeleteToken { label_params, .. }
//             | TokenFix::InsertAfter { label_params, .. } => label_params,
//         }
//     }

//     pub fn target_token_id(&self) -> &str {
//         match self {
//             TokenFix::ReplaceToken {
//                 target_token_id, ..
//             }
//             | TokenFix::DeleteToken {
//                 target_token_id, ..
//             }
//             | TokenFix::InsertAfter {
//                 target_token_id, ..
//             } => target_token_id,
//         }
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// pub enum TokenTransformKind {
//     Fix,
//     Format,
//     CustomFormatPass,
// }

// pub const TOKEN_FIX_CODES: &[&str] = &[
//     "insert-separator-after-marker",
//     "remove-empty-paragraph",
//     "set-number",
//     "split-unknown-token",
//     "insert-close-marker",
// ];

// pub const TOKEN_TRANSFORM_CHANGE_CODES: &[&str] = &["format-tokens", "custom-format-pass"];

// pub const TOKEN_TRANSFORM_SKIP_REASON_CODES: &[&str] = &["token-not-found", "empty-replacement"];

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TokenTransformChange {
//     pub kind: TokenTransformKind,
//     pub code: String,
//     pub label: String,
//     pub label_params: MessageParams,
//     pub target_token_id: Option<String>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum TokenTransformSkipReason {
//     TokenNotFound,
//     EmptyReplacement,
// }

// impl TokenTransformSkipReason {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             TokenTransformSkipReason::TokenNotFound => "token-not-found",
//             TokenTransformSkipReason::EmptyReplacement => "empty-replacement",
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct SkippedTokenTransform {
//     pub kind: TokenTransformKind,
//     pub code: String,
//     pub label: String,
//     pub label_params: MessageParams,
//     pub target_token_id: Option<String>,
//     pub reason: TokenTransformSkipReason,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TokenTransformResult<T> {
//     pub tokens: Vec<T>,
//     pub applied_changes: Vec<TokenTransformChange>,
//     pub skipped_changes: Vec<SkippedTokenTransform>,
// }

// pub fn apply_fixes<T: FormattableToken>(
//     tokens: &[T],
//     fixes: &[TokenFix],
// ) -> TokenTransformResult<T> {
//     let mut next_tokens = tokens.to_vec();
//     let mut applied_changes = Vec::new();
//     let mut skipped_changes = Vec::new();

//     for fix in fixes {
//         let Some(index) = next_tokens
//             .iter()
//             .position(|token| token.id() == Some(fix.target_token_id()))
//         else {
//             skipped_changes.push(SkippedTokenTransform {
//                 kind: TokenTransformKind::Fix,
//                 code: fix.code().to_string(),
//                 label: fix.label().to_string(),
//                 label_params: fix.label_params().clone(),
//                 target_token_id: Some(fix.target_token_id().to_string()),
//                 reason: TokenTransformSkipReason::TokenNotFound,
//             });
//             continue;
//         };

//         let anchor = next_tokens[index].clone();
//         match fix {
//             TokenFix::ReplaceToken {
//                 code,
//                 label,
//                 label_params,
//                 target_token_id,
//                 replacements,
//             } => {
//                 if replacements.is_empty() {
//                     skipped_changes.push(SkippedTokenTransform {
//                         kind: TokenTransformKind::Fix,
//                         code: code.clone(),
//                         label: label.clone(),
//                         label_params: label_params.clone(),
//                         target_token_id: Some(target_token_id.clone()),
//                         reason: TokenTransformSkipReason::EmptyReplacement,
//                     });
//                     continue;
//                 }

//                 let replacement_tokens =
//                     build_replacement_tokens(&anchor, replacements, ReplacementMode::Replace);
//                 next_tokens.splice(index..=index, replacement_tokens);
//                 applied_changes.push(TokenTransformChange {
//                     kind: TokenTransformKind::Fix,
//                     code: code.clone(),
//                     label: label.clone(),
//                     label_params: label_params.clone(),
//                     target_token_id: Some(target_token_id.clone()),
//                 });
//             }
//             TokenFix::DeleteToken {
//                 code,
//                 label,
//                 label_params,
//                 target_token_id,
//             } => {
//                 next_tokens.remove(index);
//                 applied_changes.push(TokenTransformChange {
//                     kind: TokenTransformKind::Fix,
//                     code: code.clone(),
//                     label: label.clone(),
//                     label_params: label_params.clone(),
//                     target_token_id: Some(target_token_id.clone()),
//                 });
//             }
//             TokenFix::InsertAfter {
//                 code,
//                 label,
//                 label_params,
//                 target_token_id,
//                 insert,
//             } => {
//                 if insert.is_empty() {
//                     skipped_changes.push(SkippedTokenTransform {
//                         kind: TokenTransformKind::Fix,
//                         code: code.clone(),
//                         label: label.clone(),
//                         label_params: label_params.clone(),
//                         target_token_id: Some(target_token_id.clone()),
//                         reason: TokenTransformSkipReason::EmptyReplacement,
//                     });
//                     continue;
//                 }

//                 let insert_tokens =
//                     build_replacement_tokens(&anchor, insert, ReplacementMode::InsertAfter);
//                 next_tokens.splice(index + 1..index + 1, insert_tokens);
//                 applied_changes.push(TokenTransformChange {
//                     kind: TokenTransformKind::Fix,
//                     code: code.clone(),
//                     label: label.clone(),
//                     label_params: label_params.clone(),
//                     target_token_id: Some(target_token_id.clone()),
//                 });
//             }
//         }
//     }

//     TokenTransformResult {
//         tokens: next_tokens,
//         applied_changes,
//         skipped_changes,
//     }
// }

// pub fn format_tokens_result<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
// ) -> TokenTransformResult<T> {
//     let formatted = format_tokens(tokens, options);
//     let applied_changes = if tokens_equivalent(tokens, &formatted) {
//         Vec::new()
//     } else {
//         vec![TokenTransformChange {
//             kind: TokenTransformKind::Format,
//             code: "format-tokens".to_string(),
//             label: "format tokens".to_string(),
//             label_params: MessageParams::new(),
//             target_token_id: None,
//         }]
//     };

//     TokenTransformResult {
//         tokens: formatted,
//         applied_changes,
//         skipped_changes: Vec::new(),
//     }
// }

// pub fn format_tokens_result_with_passes<T: FormattableToken>(
//     tokens: &[T],
//     options: FormatOptions,
//     passes: &[BoxedTokenFormatPass<T>],
// ) -> TokenTransformResult<T> {
//     let mut working = format_tokens(tokens, options);
//     let mut applied_changes = if tokens_equivalent(tokens, &working) {
//         Vec::new()
//     } else {
//         vec![TokenTransformChange {
//             kind: TokenTransformKind::Format,
//             code: "format-tokens".to_string(),
//             label: "format tokens".to_string(),
//             label_params: MessageParams::new(),
//             target_token_id: None,
//         }]
//     };

//     for pass in passes {
//         let before = working.clone();
//         let label = pass.label().to_string();
//         pass.apply(&mut working);
//         if !tokens_equivalent(&before, &working) {
//             applied_changes.push(TokenTransformChange {
//                 kind: TokenTransformKind::CustomFormatPass,
//                 code: "custom-format-pass".to_string(),
//                 label,
//                 label_params: MessageParams::new(),
//                 target_token_id: None,
//             });
//         }
//     }

//     TokenTransformResult {
//         tokens: working,
//         applied_changes,
//         skipped_changes: Vec::new(),
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum ReplacementMode {
//     Replace,
//     InsertAfter,
// }

// fn build_replacement_tokens<T: FormattableToken>(
//     anchor: &T,
//     templates: &[TokenTemplate],
//     mode: ReplacementMode,
// ) -> Vec<T> {
//     let base_id = anchor.id().unwrap_or("");
//     templates
//         .iter()
//         .enumerate()
//         .map(|(index, template)| {
//             if index == 0 && mode == ReplacementMode::Replace {
//                 let mut token = anchor.clone();
//                 token.set_kind(template.kind.clone());
//                 token.set_text(template.text.clone());
//                 token.set_marker(template.marker.clone());
//                 token.set_sid(
//                     template
//                         .sid
//                         .clone()
//                         .or_else(|| anchor.sid().map(ToOwned::to_owned)),
//                 );
//                 token
//             } else {
//                 let mut token = T::synthetic_like(
//                     Some(anchor),
//                     template.kind.clone(),
//                     template.text.clone(),
//                     template.marker.clone(),
//                     template
//                         .sid
//                         .clone()
//                         .or_else(|| anchor.sid().map(ToOwned::to_owned)),
//                 );
//                 if !base_id.is_empty() {
//                     let suffix = match mode {
//                         ReplacementMode::Replace => format!("~{}", index),
//                         ReplacementMode::InsertAfter => format!("+{}", index + 1),
//                     };
//                     token.set_id(format!("{base_id}{suffix}"));
//                 }
//                 token
//             }
//         })
//         .collect()
// }

// fn tokens_equivalent<T: FormattableToken>(left: &[T], right: &[T]) -> bool {
//     if left.len() != right.len() {
//         return false;
//     }

//     left.iter().zip(right).all(|(a, b)| {
//         a.id() == b.id()
//             && a.kind() == b.kind()
//             && a.text() == b.text()
//             && a.marker() == b.marker()
//             && a.sid() == b.sid()
//     })
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::format::FormattableToken;
//     use crate::format::{FormatOptions, TokenFormatPass};

//     #[derive(Debug, Clone, PartialEq, Eq)]
//     struct EditorToken {
//         id: String,
//         kind: TokenKind,
//         text: String,
//         marker: Option<String>,
//         sid: Option<String>,
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
//             Self {
//                 id: String::new(),
//                 kind,
//                 text,
//                 marker,
//                 sid,
//                 lane: anchor.map(|token| token.lane).unwrap_or(0),
//             }
//         }
//     }

//     fn token(id: &str, kind: TokenKind, text: &str, marker: Option<&str>) -> EditorToken {
//         EditorToken {
//             id: id.to_string(),
//             kind,
//             text: text.to_string(),
//             marker: marker.map(ToOwned::to_owned),
//             sid: Some("REV 19:8".to_string()),
//             lane: 4,
//         }
//     }

//     #[test]
//     fn apply_fixes_replaces_token_and_preserves_extra_fields() {
//         let tokens = vec![token("REV-1", TokenKind::Text, "(for fine linen)", None)];
//         let fixes = vec![TokenFix::ReplaceToken {
//             code: "insert-leading-space".to_string(),
//             label: "insert leading space".to_string(),
//             label_params: MessageParams::new(),
//             target_token_id: "REV-1".to_string(),
//             replacements: vec![TokenTemplate {
//                 kind: TokenKind::Text,
//                 text: " (for fine linen)".to_string(),
//                 marker: None,
//                 sid: Some("REV 19:8".to_string()),
//             }],
//         }];

//         let result = apply_fixes(&tokens, &fixes);

//         assert_eq!(result.applied_changes.len(), 1);
//         assert!(result.skipped_changes.is_empty());
//         assert_eq!(result.tokens[0].text, " (for fine linen)");
//         assert_eq!(result.tokens[0].lane, 4);
//         assert_eq!(result.tokens[0].id, "REV-1");
//     }

//     #[test]
//     fn apply_fixes_can_insert_after_anchor_token() {
//         let tokens = vec![token("REV-1", TokenKind::Text, "note text", None)];
//         let fixes = vec![TokenFix::InsertAfter {
//             code: "insert-end-marker".to_string(),
//             label: "insert end marker".to_string(),
//             label_params: MessageParams::new(),
//             target_token_id: "REV-1".to_string(),
//             insert: vec![TokenTemplate {
//                 kind: TokenKind::EndMarker,
//                 text: "\\f*".to_string(),
//                 marker: Some("f".to_string()),
//                 sid: Some("REV 19:8".to_string()),
//             }],
//         }];

//         let result = apply_fixes(&tokens, &fixes);

//         assert_eq!(result.tokens.len(), 2);
//         assert_eq!(result.tokens[1].text, "\\f*");
//         assert_eq!(result.tokens[1].id, "REV-1+1");
//         assert_eq!(result.tokens[1].lane, 4);
//     }

//     #[test]
//     fn apply_fixes_can_delete_anchor_token() {
//         let tokens = vec![
//             token("REV-1", TokenKind::Marker, "\\m", Some("m")),
//             token("REV-2", TokenKind::Newline, "\n", None),
//         ];
//         let fixes = vec![TokenFix::DeleteToken {
//             code: "remove-empty-paragraph".to_string(),
//             label: "remove empty paragraph".to_string(),
//             label_params: MessageParams::new(),
//             target_token_id: "REV-1".to_string(),
//         }];

//         let result = apply_fixes(&tokens, &fixes);

//         assert_eq!(result.tokens.len(), 1);
//         assert_eq!(result.tokens[0].id, "REV-2");
//         assert_eq!(result.tokens[0].kind, TokenKind::Newline);
//     }

//     #[test]
//     fn format_tokens_result_reports_change_when_tokens_were_rewritten() {
//         let tokens = vec![token("REV-1", TokenKind::Text, "a  b", None)];
//         let result = format_tokens_result(&tokens, FormatOptions::default());

//         assert_eq!(result.applied_changes.len(), 1);
//         assert_eq!(result.applied_changes[0].kind, TokenTransformKind::Format);
//         assert_eq!(result.tokens[0].text, "a b");
//     }

//     struct ReplaceTextPass;

//     impl TokenFormatPass<EditorToken> for ReplaceTextPass {
//         fn label(&self) -> &str {
//             "replace-text"
//         }

//         fn apply(&self, tokens: &mut Vec<EditorToken>) {
//             for token in tokens.iter_mut() {
//                 if token.kind == TokenKind::Text {
//                     token.text = token.text.replace("alpha", "beta");
//                 }
//             }
//         }
//     }

//     #[test]
//     fn format_tokens_result_with_passes_reports_custom_pass_changes() {
//         let tokens = vec![token("REV-1", TokenKind::Text, "alpha  alpha", None)];
//         let passes = vec![Box::new(ReplaceTextPass) as Box<_>];

//         let result = format_tokens_result_with_passes(&tokens, FormatOptions::default(), &passes);

//         assert_eq!(result.applied_changes.len(), 2);
//         assert_eq!(result.applied_changes[0].kind, TokenTransformKind::Format);
//         assert_eq!(
//             result.applied_changes[1].kind,
//             TokenTransformKind::CustomFormatPass
//         );
//         assert_eq!(result.applied_changes[1].label, "replace-text");
//         assert_eq!(result.tokens[0].text, "beta beta");
//     }
// }
