# Phase 36 LSP Query Budget IDs

Phase 35 reserves the protocol-level budget id namespace below so Phase 36 can add LSP benchmarks without changing the performance policy model.

Rules:

- Use `perf.lsp.<query>.<scenario>` for protocol latency budgets.
- Use `perf.lsp.document_sync.<scenario>` for didOpen/didChange/didClose cache and invalidation budgets.
- Use `perf.lsp.transport.<scenario>` only for JSON-RPC framing or startup costs that are outside editor query semantics.
- Every LSP budget id must map to one `verification/performance/manifest.json` case once `sifr lsp` exists.
- LSP waivers use the same owner, issue, expiry, override, and correctness-non-waiver policy as compiler benchmark waivers.

Reserved ids:

- `perf.lsp.cold_start.workspace`
- `perf.lsp.completion.local_scope`
- `perf.lsp.completion.auto_import`
- `perf.lsp.hover.symbol`
- `perf.lsp.definition.local_symbol`
- `perf.lsp.references.workspace_symbol`
- `perf.lsp.rename.prepare`
- `perf.lsp.semantic_tokens.full`
- `perf.lsp.semantic_tokens.delta`
- `perf.lsp.document_symbols.module`
- `perf.lsp.folding_ranges.module`
- `perf.lsp.selection_ranges.nested`
- `perf.lsp.inlay_hints.module`
- `perf.lsp.code_action.diagnostic`
- `perf.lsp.document_sync.did_open`
- `perf.lsp.document_sync.unchanged_did_change`
- `perf.lsp.document_sync.changed_did_change`
- `perf.lsp.document_sync.parse_error_recovery`
- `perf.lsp.transport.initialize`
- `perf.lsp.transport.shutdown`
