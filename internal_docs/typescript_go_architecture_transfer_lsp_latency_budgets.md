# TypeScript-Go Architecture Transfer: LSP Latency Budgets

LSP latency budget replaces aggregate-only LSP performance evidence with protocol-level
per-request editor latency budgets. The existing
`lsp-query-001-request-families` benchmark remains in the manifest, but it is
now smoke coverage only. Enforced latency budgets live in individual
`lsp-query-*` cases for:

- cold start
- didOpen diagnostics
- document diagnostics
- workspace diagnostics
- completion
- hover
- signature help
- navigation
- references
- rename
- semantic tokens
- inlay hints
- selection range
- type hierarchy
- code actions
- formatting
- generated Rust preview

Warm request-family scenarios reuse one initialized `sifr lsp --stdio` session
with one open document. They record the average operation time across
`inner_repetitions` inside each measured sample, which keeps these budgets
focused on request dispatch, protocol conversion, scheduling, analysis query
execution, and command plumbing after the server has established its workspace
session. Cold start is the separate scenario that measures subprocess spawn
through the initialize response. didOpen diagnostics measures document sync
through diagnostics publication in a fresh warm session. Document diagnostics
measure a warm edit notification through diagnostics publication and a pull
diagnostics request. Workspace diagnostics measure the pull workspace
diagnostics request.

Per-request median and p95 thresholds are derived from the recorded baseline
and capped by the editor-facing SLO for each family. LSP budget entries do not
enforce cache-hit or cache-miss counters yet because the protocol server does
not export stable per-request cache counters; latency, RSS, and timeout gates
are enforced now.

The frontend query architecture frontend-query budgets still measure compiler-service behavior
below the protocol layer. LSP latency budget LSP budgets measure editor-observed JSON-RPC
latency on top of those APIs. A frontend-query regression usually points at
compiler-service or cache behavior; an LSP regression can also come from
protocol conversion, request scheduling, stdio dispatch, diagnostics
publication, or command handling.
