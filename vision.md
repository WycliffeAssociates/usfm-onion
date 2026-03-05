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

## Non-goals

- This repo is not a UI framework or editor runtime.
- This repo is not a silent autocorrection engine.
- This repo is not performance-first before correctness and maintainability are established.
- This repo is not limited to one downstream consumer.

## Success Signals (Next 4-8 Weeks)

- Core operations behave consistently with explicit mutation boundaries.
- Vision principles can be mapped directly to observable API behavior and tests.
- Current-state documentation remains evidence-backed and usable for correctness/compliance passes.
