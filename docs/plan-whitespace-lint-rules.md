# Plan: whitespace lint rules consuming `MARKER_WHITESPACE`

Status: deferred. The infrastructure landed in commit `64f31e1`
(whitespace spec alignment); the rules themselves are next.

## Why this exists

The `MARKER_WHITESPACE` table in `src/marker_defs_data.rs` declares, for
each interesting marker, what structural whitespace the spec requires at
each position around it (before the open marker, after the marker name,
before/after the closing form). It also carries the formatter's preference
for how to resolve that whitespace when normalizing.

That same table is the right driver for lint rules: if a row says `\c`
requires `AtLeastOneHorizontalWhitespace` after the marker name, then a
linter that finds `\c1` (no space) should flag it — and the auto-fix is
exactly what the formatter would insert (a single space). One source of
truth, two consumers.

This plan describes the rules to add, why each one is worth its weight,
and the surgery needed in `src/lint_impl.rs`.

## The rules

Six rules cover the structural-whitespace and content-whitespace concerns
documented in `whitespace.md`. Each is paired with a `TokenFix` so the
linter can either auto-apply the fix individually or hand off to format
for a bulk pass. Same code path in both directions.

### 1. `MissingStructuralWhitespaceBeforeMarker`

**What it checks.** For any marker whose
`required_before_open` is `NewlineOrAnyWhitespaceBeforeMarker` or
`SingleNewline` or `AtLeastOneNewline` or `AtLeastOneWhitespace`, verify
the previous token in the stream satisfies the requirement. If not, flag.

**Auto-fix.** Insert the marker's `format_preference_before_open` (single
newline or single space) immediately before the marker token.

**Examples this catches.**
- `... text\p Word` — paragraph marker without preceding newline.
- `\c 1\v 1 Text` — verse without separator before it.
- `... text\esb` — sidebar opener jammed against text.

### 2. `MissingHorizontalWhitespaceAfterMarkerName`

**What it checks.** For any marker whose `required_after_open_name` is
`AtLeastOneHorizontalWhitespace` or `AtLeastOneWhitespace`, verify the
next token is whitespace of the right shape.

**Auto-fix.** Insert a single space after the marker name.

**Examples.**
- `\c1` (chapter without space after marker name).
- `\v1Text` — verse number jammed against marker AND against text.

This generalizes the existing `MissingSeparatorAfterMarker` rule (which
catches the `\v1Text` case via a different path). New rule lets us
remove the ad-hoc `starts_with_horizontal_whitespace` /
`ends_with_horizontal_whitespace` helpers in `lint_impl.rs:2094-2102`.

### 3. `MissingTagEndDelimiterAfterMarker`

**What it checks.** For markers with `required_after_open_name =
TagEndDelimiter` (paragraph markers, notes, sidebars), verify the next
token is whitespace, end-of-input, or `|` (start-of-attributes).

**Auto-fix.** Insert a single space.

**Why separate from rule 2.** The shape of valid tokens here is broader
than "just horizontal whitespace" — `\p` followed by `\n` is fine, as
is `\p` at end-of-input, as is `\p|attr=...`. The existing
`MissingSeparatorAfterMarker` essentially does this already; this rule
generalizes it via the table so future markers get the check for free.

### 4. `ExcessStructuralWhitespaceAroundMarker`

**What it checks.** For any marker, if the actual whitespace run before
or after the marker exceeds what the requirement specifies as the
*minimum*, flag.

- `AtLeastOneHorizontalWhitespace` actual = 5 spaces → flag.
- `SingleNewline` actual = 3 newlines → flag.
- `OptionalHorizontalWhitespace` actual = 2 spaces → flag (still
  excess; one space is the canonical form).

**Auto-fix.** Collapse the run to a single occurrence of whatever the
preference says (or a single space if the preference is unset).

**Why bother.** The current format module already has
`CollapseConsecutiveLinebreaks` and `NormalizeMarkerWhitespaceAtLineStart`
that do related work, but they're heuristic, not driven by the table.
This rule reports the same condition in lint terms, with the table as
ground truth.

### 5. `ExcessWhitespaceInContent`

**What it checks.** Inside a `Text` content token, flag any run of 2+
horizontal whitespace characters or any embedded newline. Does not
consult `MARKER_WHITESPACE`; reads pure text content.

**Carve-out.** Skip when the character immediately preceding the run is
one of `.`, `!`, `?`, `:`, `;` (use
`crate::whitespace::is_sentence_ending_punctuation_char`). Protects the
older typography convention of two spaces after a period without
needing per-document detection.

**Auto-fix.** Collapse the run to a single space.

**Why this matters.** Most multi-space inside content text is an
authoring artifact, not intentional. Catching it makes diffs cleaner.
The punctuation carve-out keeps the rule from being noisy on legitimate
typography choices.

### 6. `MissingContentSpaceBetweenAdjacentText`

**What it checks.** When a closing character marker (`\nd*`, `\bk*`,
etc.) is immediately followed by alphabetic text with no whitespace
between, flag.

**Auto-fix.** None. The same shape can be language-correct
(`\nd Lord\nd*'s` is intentional English contraction). Reports as a
content-style violation; downstream callers can suppress per-fixture or
per-file.

**Why include it without auto-fix.** The other five rules are crisp
either-or; this one needs human judgment. Worth flagging because the
violation is real (the example in `whitespace.md` —
`\v 2 \qac O\qac*lvidada` — is genuinely missing a space), but
auto-fixing risks corrupting valid content. Treating it as a flag-only
rule with metadata sufficient for suppression gives the user a path
forward without forcing a choice the linter can't make.

## The surgery

`src/lint_impl.rs` is 2829 lines. The changes touch four sections:

1. **Add variants to `LintCode`** (line 138 area): six new variants in
   the same style as existing ones. Update `code()`, `label_key()`,
   `category()`, and the default-enabled set in `LintOptions`.

2. **Add the rule implementations.** Each rule is a function over
   `&[Token]` (or `&[FormatToken]`) that walks the stream, looks up the
   `MARKER_WHITESPACE` row for marker tokens, checks the surrounding
   trivia, and emits a `LintIssue` with a `TokenFix` (except rule 6
   which has `fix: None`).

3. **Wire into `lint_tokens` / `lint_usfm`**: extend the existing rule
   dispatch around line 618 to call the new functions when the
   corresponding `LintCode` is enabled.

4. **Remove now-redundant helpers.** Once rules 1-3 are in place,
   `MissingSeparatorAfterMarker` likely becomes a redundant
   special-case of rule 3; the `starts_with_horizontal_whitespace` /
   `ends_with_horizontal_whitespace` helpers (lint_impl.rs:2094-2102)
   are subsumed by `crate::whitespace` predicates. Clean up.

## Tests

For each new rule, two tests in `lint_impl.rs::tests`:

1. **Detects.** Construct a fixture that violates the rule; assert the
   lint result contains the expected `LintCode` once.
2. **Auto-fix produces canonical output.** Apply the fix; serialize the
   result; assert it matches what `format_usfm` would produce on the
   original (with the matching profile). This ties the rule and the
   formatter to one ground truth.

For rule 6 (the no-fix one), the second test asserts `fix.is_none()`
and the issue has the `LintCategory` we'd choose for content-style.

## Open questions

- Should rule 4 (excess structural WS) live in the same family as rule
  5 (excess content WS), or separate? Probably separate — they're
  about different parts of the document and a downstream might want to
  suppress one without the other. But verify when wiring up.
- Rule 5's punctuation carve-out is a fixed set today (`.!?:;`).
  Worth making it configurable per-language? Probably not for v1; YAGNI.
- Rule 1 needs to reason about tokens at the START of input — is "no
  preceding token" satisfying `NewlineOrAnyWhitespaceBeforeMarker`?
  Yes (tokens at SOF are conceptually preceded by a newline). Test
  this case explicitly.

## Estimated scope

- ~150 lines added to `lint_impl.rs` (six rules + dispatch + LintCode
  expansions).
- ~80 lines of tests.
- ~20 lines removed (redundant helpers, generalized rule).

Net: +210 lines, all in lint_impl.rs.

## Adjacent rule to land alongside (USFM 3.2)

`verse-not-in-section-or-other-paragraph` — USFM 3.2 release notes
revise `\v` so it is **not allowed** inside paragraphs of category
`Section` or `Other`. The declarative data is already in place
(`MarkerSpec.paragraph_category`, populated per the 3.2 para index in
`marker_defs_data.rs`). Implement once the lint visitor lands per
`plan-walker-architecture.md`:

- Trigger: walker emits `on_enter_scope` for a `Verse` while
  `WalkContext.current_paragraph_category()` is
  `Some(ParagraphCategory::Section | ParagraphCategory::Other)`.
- Emit: a lint diagnostic at the `\v` token's span; no autofix
  (structural — user has to move the verse or change the paragraph).
- Out of scope for the six whitespace rules above, but worth
  bundling in the same lint-visitor migration since both depend on
  the same `WalkContext` plumbing.
