//! `lint_tokens` must produce correct results on token streams that
//! arrive without `structural` / marker-metadata populated.
//!
//! The motivating case: a JS host hands the linter tokens it produced
//! from its own document model (only `kind`, `marker`, `text`, etc.)
//! without re-running parse-time structural classification. The walker
//! should derive the missing context from the marker name on the fly.

use usfm_onion::lint::{LintCode, LintOptions, LintScope, LintableToken, lint_tokens};
use usfm_onion::marker_defs::StructuralMarkerInfo;
use usfm_onion::token::{NumberRangeKind, Span, TokenKind, UsfmToken};
use usfm_onion::walker::WalkableToken;

#[derive(Clone)]
struct BareToken {
    kind: TokenKind,
    marker: Option<String>,
    text: String,
}

impl BareToken {
    fn marker(name: &str) -> Self {
        Self {
            kind: TokenKind::Marker,
            marker: Some(name.to_string()),
            text: format!("\\{name}"),
        }
    }
    fn end(name: &str) -> Self {
        Self {
            kind: TokenKind::EndMarker,
            marker: Some(name.to_string()),
            text: format!("\\{name}*"),
        }
    }
    fn text(t: &str) -> Self {
        Self {
            kind: TokenKind::Text,
            marker: None,
            text: t.to_string(),
        }
    }
    fn number(t: &str) -> Self {
        Self {
            kind: TokenKind::Number,
            marker: None,
            text: t.to_string(),
        }
    }
    fn newline() -> Self {
        Self {
            kind: TokenKind::Newline,
            marker: None,
            text: "\n".to_string(),
        }
    }
}

impl UsfmToken for BareToken {
    fn kind(&self) -> TokenKind {
        self.kind
    }
    fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }
    fn source(&self) -> &str {
        &self.text
    }
}

impl WalkableToken for BareToken {
    fn structural(&self) -> Option<StructuralMarkerInfo> {
        // Deliberately bare — the walker should derive this from the marker name.
        None
    }
}

impl LintableToken for BareToken {
    fn span(&self) -> Option<Span> {
        None
    }
    fn sid(&self) -> Option<String> {
        None
    }
    fn id(&self) -> Option<String> {
        None
    }
    fn number_info(&self) -> Option<(u32, Option<u32>, NumberRangeKind)> {
        None
    }
}

fn fixture_nested_xt() -> Vec<BareToken> {
    // Equivalent to:
    // \id GEN\n\c 1\n\p\n\v 2 Male and female He created them,
    // \f + \fr 5:2 \ft Cited in \+xt Matthew 19:4\+xt* and \+xt Mark 10:6\+xt*\f*
    vec![
        BareToken::marker("id"),
        BareToken::text(" GEN"),
        BareToken::newline(),
        BareToken::marker("c"),
        BareToken::number(" 1"),
        BareToken::newline(),
        BareToken::marker("p"),
        BareToken::newline(),
        BareToken::marker("v"),
        BareToken::number(" 2"),
        BareToken::text(" Male and female He created them,"),
        BareToken::marker("f"),
        BareToken::text(" + "),
        BareToken::marker("fr"),
        BareToken::text(" 5:2 "),
        BareToken::marker("ft"),
        BareToken::text(" Cited in "),
        BareToken::marker("+xt"),
        BareToken::text(" Matthew 19:4"),
        BareToken::end("+xt"),
        BareToken::text(" and "),
        BareToken::marker("+xt"),
        BareToken::text(" Mark 10:6"),
        BareToken::end("+xt"),
        BareToken::end("f"),
        BareToken::newline(),
    ]
}

#[test]
fn bare_tokens_with_nested_xt_pair_correctly() {
    let tokens = fixture_nested_xt();
    let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
    let stray: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == LintCode::StrayCloseMarker)
        .collect();
    assert!(
        stray.is_empty(),
        "stray-close fired on bare-token \\+xt fixture: {stray:#?}"
    );
}

#[test]
fn bare_tokens_implicit_close_at_f_star_does_not_flag_stray_close() {
    // \fqa opens with no explicit \fqa* — \f* implicitly closes it.
    // Expect implicitly-closed-marker (informational), never stray-close-marker.
    let tokens = vec![
        BareToken::marker("id"),
        BareToken::text(" GEN"),
        BareToken::newline(),
        BareToken::marker("c"),
        BareToken::number(" 1"),
        BareToken::newline(),
        BareToken::marker("p"),
        BareToken::newline(),
        BareToken::marker("v"),
        BareToken::number(" 1"),
        BareToken::text(" hi "),
        BareToken::marker("f"),
        BareToken::text(" + body "),
        BareToken::marker("fqa"),
        BareToken::text(" alt"),
        BareToken::end("f"),
        BareToken::newline(),
    ];
    let result = lint_tokens(&tokens, LintOptions::scoped(LintScope::Book));
    let stray: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == LintCode::StrayCloseMarker)
        .collect();
    assert!(
        stray.is_empty(),
        "stray-close fired on implicit \\fqa close at \\f*: {stray:#?}"
    );
}
