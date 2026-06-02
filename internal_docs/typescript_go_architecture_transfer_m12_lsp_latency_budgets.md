# TypeScript-Go Architecture Transfer M12: LSP Latency Budgets

status: in progress

M12 replaces aggregate-only LSP performance evidence with protocol-level
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

The Phase 35 frontend-query budgets still measure compiler-service behavior
below the protocol layer. M12 LSP budgets measure editor-observed JSON-RPC
latency on top of those APIs. A frontend-query regression usually points at
compiler-service or cache behavior; an LSP regression can also come from
protocol conversion, request scheduling, stdio dispatch, diagnostics
publication, or command handling.

Validation so far:

- `python3 verification/performance/run_benchmarks.py --validate-only` -> PASS, 65 cases
- `python3 verification/performance/check_budgets.py` -> PASS
- `python3 verification/performance/run_benchmarks.py --self-test` -> PASS
- `python3 verification/performance/check_budgets.py --self-test` -> PASS
- `python3 verification/performance/run_benchmarks.py --groups lsp-query --json-out target/performance/m12_lsp_query_run.json` -> PASS, evidence `target/performance/evidence/bench-1780400105-94623.json`
- `python3 verification/performance/run_benchmarks.py --groups lsp-query --sample-scale smoke --json-out target/performance/m12_lsp_query_smoke.json` -> PASS, evidence `target/performance/evidence/bench-1780400215-529.json`
- `python3 verification/performance/check_budgets.py --results target/performance/m12_lsp_query_run.json --allow-subset` -> PASS
- `python3 verification/performance/check_budgets.py --results target/performance/m12_lsp_query_smoke.json --allow-subset` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py --self-test` -> PASS
- `python3 -m py_compile verification/performance/lsp_query_bench.py verification/performance/check_budgets.py verification/performance/run_benchmarks.py verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
