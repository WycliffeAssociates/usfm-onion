# Plan: `usfm-onion` CLI

Status: deferred. Design locked; implementation lives in a future
`src/bin/usfm-onion.rs` (or similar) once the parallel CST work has
settled, since `convert` and `format` will surface any token-shape
changes from that branch.

## Why this exists

The library exposes `lint`, `format`, `convert` (to USJ/USX/HTML),
`diff`, and `vref`. There's no user-facing CLI today — only
`src/bin/playground.rs`, which is an internal sampling/profiling
harness (toggle-comment ops, `samply`-friendly). The two have
different jobs and shouldn't share a binary: the playground is a
dev kitchen sink with zero-ceremony argument handling; a real CLI
needs subcommands, predictable I/O, and exit codes that CI can
consume.

The goal is a focused, intuitive surface — five subcommands, smart
defaults, Unix-composable. Resisting "every library function gets
a subcommand" sprawl is the design constraint.

## Binary name

`usfm-onion` — matches the crate, unambiguous on `$PATH`. Longer
to type than `usfm` but tab-completion handles it, and `usfm` is
generic enough to risk collisions.

## Subcommand shape

```
usfm-onion lint     [<path>...]            [--format json|text]
usfm-onion format   [<path>]               [--write | --check]
usfm-onion convert  [-f <fmt>] [-t <fmt>]  [<path>] [-o <path>]
usfm-onion diff     <a> <b>                [--format json|text]
usfm-onion vref     [<path>]               [--format json|text]
```

Five subcommands. No more. Each maps to exactly one library entry
point. Help text is per-subcommand, so the top-level surface stays
small and discoverable.

## I/O conventions (apply to all subcommands)

- **Stdin/stdout default.** No path argument → read stdin (error
  cleanly if stdin is a TTY). Output goes to stdout unless `-o` is
  given or the subcommand has a write-in-place mode.
- **`-` is the explicit stdin sentinel.** Useful in pipelines that
  also pass flags: `cat foo.usfm | usfm-onion lint -`.
- **`--format text` is the human default; `--format json` is the
  tooling contract.** Matches `cargo`, `ruff`, `eslint`. JSON output
  shapes should be considered stable once published.

## Per-subcommand details

### `lint`

- Accepts one or more paths; recurses into directories collecting
  `*.usfm`. The only subcommand that recurses — batch linting is a
  real workflow (CI, editors). Other ops force shell loops.
- `--format text` (default): one finding per line, `path:line:col:
  rule-id  message`. Group by file with a header.
- `--format json`: array of `{path, findings: [...]}` objects.
- No `--fix` flag until the linter actually emits autofixes. Don't
  ship the surface before the feature.

### `format`

- Single input. Three output modes:
  - Default (no flag): print formatted USFM to stdout. Composable.
  - `--write`: rewrite the input file in place. Rustfmt/prettier
    convention. Errors if input is stdin.
  - `--check`: don't write; exit nonzero if any change would be
    made. CI's preferred shape.
- `--write` and `--check` are mutually exclusive.

### `convert`

- Bidirectional `--from`/`--to` model (pandoc-style). Today only
  `--from usfm` is wired; the surface accommodates future
  `usj → usfm`, `usx → usj`, etc. without renames.
- **Supported pairs today:**

  | from → to | usj | usx | html |
  | --------- | --- | --- | ---- |
  | **usfm**  | ✅   | ✅   | ✅    |

  All other pairs return: `conversion <a> → <b> not yet supported`.
  Cells flip as library functions land; no CLI changes needed.

- **Format auto-detection** (unambiguous extensions only):
  - `.usfm` → `usfm`, `.usj` → `usj`, `.usx` → `usx`.
  - `.json` / `.xml` / no extension / stdin → require explicit
    `-f`/`-t`. (`.json` could be anything; refusing to guess is
    safer than wrong-guess silent failures.)
- **`-o` extension implies `--to`** when unambiguous. Explicit
  `--to` always wins. So `convert foo.usfm -o foo.html` works
  with zero flags.
- `-f` / `-t` short flags match pandoc.

### `diff`

- Two required positional args (baseline, current). No stdin
  shortcut — diffing inherently needs two inputs.
- `--format text` (default): unified-ish summary of changed
  verses/blocks, readable by humans.
- `--format json`: the raw diff structure the library already
  emits.

### `vref`

- Single input. Emits the verse-reference map.
- `--format json` (default — vref is fundamentally tooling data).
- `--format text`: line-per-verse `BOOK CHAPTER:VERSE  <text snippet>`
  or similar. Worth defining when implementing; not load-bearing
  for design.

## Exit codes (rustfmt / ruff convention)

| Subcommand        | 0                 | 1                | 2              |
| ----------------- | ----------------- | ---------------- | -------------- |
| `lint`            | clean             | findings present | internal error |
| `format --check`  | no changes needed | would reformat   | internal error |
| `format --write`  | success           | —                | internal error |
| `format` (stdout) | success           | —                | internal error |
| `convert`         | success           | —                | internal error |
| `diff`            | success           | —                | internal error |
| `vref`            | success           | —                | internal error |

CI scripts can distinguish "has findings" from "crashed" — the
distinction `lint`/`format --check` need most.

## Explicitly out of scope

- **`parse` / `cst` / `tokens` subcommands.** Dev/debug only.
  Playground covers them. If a real user-facing need emerges,
  re-evaluate under a `debug` subcommand to avoid top-level sprawl.
- **`--fix` on `lint`.** Add when the linter emits autofixes; the
  flag without the feature is a usability lie.
- **Config files (`.usfm-onionrc` etc.).** Premature. Flags suffice
  until someone has a real workflow that demands persistent config.
- **Verbosity flags (`-v`, `-vv`).** Add when there's something
  meaningful to gate.
- **Batch convert (recursing on directories).** Output shape is
  ambiguous (mirror tree? flat? what extension?). Shell loops are
  the right tool for a rare need.
- **Inverse conversions** (`usj` → `usfm`, etc.). Matrix entries
  flip when the library supports them; design is already
  forward-compatible.
- **Lint/format/diff/vref on non-USFM input.** All four operate on
  USFM semantics specifically. Linting a USJ document would need a
  different linter; not something to plumb `--from` through
  speculatively.

## Dependencies

- `clap` v4 with the `derive` feature.
- Value enums for `--from`, `--to`, `--format`.
- `anyhow` (or equivalent) for the binary's error handling — the
  library's typed errors get rendered with context at the CLI
  boundary.

## Implementation sketch

- `src/bin/usfm-onion.rs` as the entry point.
- One module per subcommand under `src/bin/` is overkill for five
  commands; keep them as functions in the same file unless one
  grows past ~150 lines.
- A small shared `io.rs`-style helper for "resolve input → String"
  and "resolve output → impl Write", handling stdin/stdout/`-`/file
  uniformly. The playground can borrow from this once it exists.
- Snapshot tests for help text (catches accidental surface changes)
  and a handful of golden-file integration tests per subcommand.

## Order of operations when building

1. `clap` skeleton with all five subcommands and `--help` only;
   no library calls yet. Verify the help text reads well.
2. Shared I/O helper.
3. `lint`, `format`, `vref` (single library call each, simple
   output).
4. `convert` (the most complex — `--from`/`--to` matrix, auto-detect,
   `-o` extension inference).
5. `diff` (two inputs, but otherwise mechanical).
6. Exit-code wiring and CI-shape tests last, since they depend on
   the prior wiring being correct.
