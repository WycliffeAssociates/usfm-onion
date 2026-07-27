# Rejected: lexer-owned delimiter whitespace (WS3)

Rejected 2026-07-22. WS1/WS2 already captured the parse win (~10%, dense
`MarkerIndex` row + single resolution). WS3 only moves delimiter-whitespace
absorption from parser into lexer — a refactor that removes `pending_ws` state
but is **not** a measured perf win, while carrying real byte-identical behavior
risk (first-char-only absorption, `cv_number==JustEmitted` gating, span exactness
at chunk boundaries). Not worth the risk for marginal/no gain.

Revisit only if `pending_ws` becomes a maintenance problem in its own right.
