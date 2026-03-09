# USFM Onion Vision

## Purpose

USFM Onion is a core USFM engine for parsing, projection, linting, formatting, diff/revert, and conversion across multiple representations. It is editor-agnostic and designed to be reusable by any consumer that needs reliable USFM workflows.

## Guiding Principles

1. **Engine-first and consumer-agnostic**
   - The repo exists to provide reusable USFM capabilities, not app-specific UI behavior.

2. **Correctness first, then maintainability, then performance**
   - Behavioral correctness is the first priority.
   - Maintainable and idiomatic implementation is second.
   - Performance optimization follows once behavior is trustworthy.

3. **Native USFM fidelity is non-negotiable by default**
   - Native `USFM -> parse/token -> USFM` must be lossless unless a mutation is explicitly requested.

4. **Flat token streams are first-class**
   - Flat token APIs are a public core surface, not an internal-only detail.

5. **Mutations are strict opt-in**
   - Fix, format, or conversion mutations must be explicit operations.
   - Parse/projection paths must not silently rewrite source content.

6. **Spec USJ/USX are interchange; fidelity may require a distinct internal representation**
   - Spec USJ/USX are important interchange formats.
   - Exact editor-grade fidelity may require a distinct internal path (directionally, a Lossless USJ Tree).

7. **Static analysis is a directional extension, not current core scope**
   - Current core scope is parse/projection/lint/format/diff/conversion.
   - Richer static analysis is a follow-on capability.

8. **Do not become a UI/editor monolith**
   - Rendering, editor state, and interaction UX are not the core responsibility of this engine.

## Boundaries

- **In scope**
  - USFM-native parse/projection/write fidelity behavior.
  - Token-first lint/format/diff/revert/conversion primitives.
  - Interchange in/out for USJ, USX, and VREF outputs.

- **Out of scope**
  - Editor rendering trees, view state, and UI orchestration as product concerns.
  - Silent mutation side effects in parse/projection flows.

## API Entry Tables

### Intake -> Output (Core Conversion Surface)

| Output | usfm intake | usx intake | usj intake | tokens intake |
| --- | --- | --- | --- | --- |
| `into_tokens` | direct (`into_tokens_from_content/path`) | indirect (`usx -> usfm -> parse -> tokens`) | indirect (`usj -> usfm -> parse -> tokens`) | n/a |
| `into_usfm` | identity / direct (`parse_usfm_*` or source passthrough) | direct (`from_usx`, `usx_content_to_usfm`, `usx_path_to_usfm`) | direct (`from_usj`, `usj_content_to_usfm`, `usj_path_to_usfm`) | direct (`into_usfm_from_tokens`) |
| `into_usj` | direct (`parse -> into_usj`, `usfm_content_to_usj`, `usfm_path_to_usj`) | indirect (`usx -> usfm -> parse -> into_usj`, `usx_content_to_usj`, `usx_path_to_usj`) | identity / direct (`parse_usj_*` or source passthrough) | indirect (`into_usj_from_tokens`) |
| `into_usx` | direct (`parse -> into_usx`, `usfm_content_to_usx`, `usfm_path_to_usx`) | identity / direct (`parse_usx_*` or source passthrough) | indirect (`usj -> usfm -> parse -> into_usx`, `usj_content_to_usx`, `usj_path_to_usx`) | indirect (`into_usx_from_tokens`) |
| `into_vref` | direct (`parse -> into_vref`) | indirect (`usx -> usfm -> parse -> into_vref`) | indirect (`usj -> usfm -> parse -> into_vref`) | indirect (`into_vref_from_tokens`) |
| `into_usj_lossless` | direct (`parse -> into_usj_lossless`) | indirect (`usx -> usfm -> parse -> into_usj_lossless`) | indirect (`usj -> usfm -> parse -> into_usj_lossless`) | indirect (`into_usj_lossless_from_tokens`) |
| `into_usx_lossless` | direct (`parse -> into_usx_lossless`) | n/a (not exposed as first-class intake path) | n/a (not exposed as first-class intake path) | indirect (`into_usx_lossless_from_tokens`) |

### Lint / Format / Diff Entry Surface

| Concern | content entrypoints | path entrypoints | token entrypoints | parallel control |
| --- | --- | --- | --- | --- |
| lint | `lint_content`, `lint_contents` | `lint_path`, `lint_paths` | `lint_flat_tokens`, `lint_flat_token_batches` | `BatchExecutionOptions.parallel` + `rayon` feature |
| format | `format_content`, `format_content_with_passes`, `format_contents` | `format_path`, `format_paths` | `format_flat_tokens`, `format_flat_tokens_mut`, `format_flat_tokens_with_passes`, `format_flat_tokens_mut_with_passes`, `format_flat_token_batches` | `BatchExecutionOptions.parallel` + `rayon` feature |
| diff | `diff_content` | `diff_paths` | `diff_tokens` | file/batch-level via `BatchExecutionOptions.parallel` where batch APIs are used |

Notes:
- Content/path APIs accept `DocumentFormat` (`Usfm`, `Usx`, `Usj`) and normalize through USFM parse unless identity/direct conversion is available.
- The intended public reading surface prefers explicit input-typed wrappers such as `parse_usfm_content`, `into_tokens_from_usx_content`, `lint_usj_content`, `format_usfm_content`, and `project_usfm_content`; generic `*_content(..., DocumentFormat)` helpers remain as lower-level adapters.
- Conversion naming follows the same rule: prefer explicit one-to-another names such as `usfm_content_to_usj`, `usfm_content_to_usx`, `usj_content_to_usfm`, and `usx_content_to_usfm`; generic `convert_*` helpers remain as lower-level adapters.
- Token entrypoints are first-class for lint/format/diff, with direct token-to-USFM reconstruction via `into_usfm_from_tokens`.
- Formatting enables the built-in ruleset by default; downstream cleanup policy can narrow rules explicitly or add ordered custom passes.
- Parallelism is opt-in and intended for file/batch-level work, not intra-file parsing.

## Non-goals

- This repo is not a UI framework or editor runtime.
- This repo is not a silent autocorrection engine.
- This repo is not performance-first before correctness and maintainability are established.
- This repo is not limited to one downstream consumer.

## Success Signals (Next 4-8 Weeks)

- Core operations behave consistently with explicit mutation boundaries.
- Vision principles can be mapped directly to observable API behavior and tests.
- Current-state documentation remains evidence-backed and usable for correctness/compliance passes.
