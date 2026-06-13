# Phase 36 LSP Query Budget IDs

Phase 35 reserves the protocol-level budget id namespace below so Phase 36 can add LSP benchmarks without changing the performance policy model.

Rules:

- Use `perf.lsp.<query>.<scenario>` for protocol latency budgets.
- Use `perf.lsp.document_sync.<scenario>` for didOpen/didChange/didClose cache and invalidation budgets.
- Use `perf.lsp.transport.<scenario>` only for JSON-RPC framing or startup costs that are outside editor query semantics.
- Every LSP budget id must map to one `verification/areas/performance/data/benchmark_manifest.json` case once `sifr lsp` exists.
- LSP waivers use the same owner, issue, expiry, override, and correctness-non-waiver policy as compiler benchmark waivers.

Reserved ids:

- `perf.lsp.cold_start.workspace`
- `perf.lsp.diagnostics.document`
- `perf.lsp.diagnostics.workspace`
- `perf.lsp.completion.local_scope`
- `perf.lsp.completion.auto_import`
- `perf.lsp.hover.symbol`
- `perf.lsp.signature_help.call`
- `perf.lsp.definition.local_symbol`
- `perf.lsp.navigation.symbol`
- `perf.lsp.references.workspace_symbol`
- `perf.lsp.rename.prepare`
- `perf.lsp.rename.workspace_edit`
- `perf.lsp.semantic_tokens.full`
- `perf.lsp.semantic_tokens.delta`
- `perf.lsp.document_symbols.module`
- `perf.lsp.folding_ranges.module`
- `perf.lsp.selection_ranges.nested`
- `perf.lsp.inlay_hints.module`
- `perf.lsp.code_action.diagnostic`
- `perf.lsp.formatting.document`
- `perf.lsp.generated_rust_preview.document`
- `perf.lsp.document_sync.did_open`
- `perf.lsp.document_sync.unchanged_did_change`
- `perf.lsp.document_sync.changed_did_change`
- `perf.lsp.document_sync.parse_error_recovery`
- `perf.lsp.transport.initialize`
- `perf.lsp.transport.shutdown`
- `perf.lsp.request_families`

## m36.5 Implemented Evidence

- `perf.lsp.request_families` maps to `verification/areas/performance/data/benchmark_manifest.json` case
  `lsp-query-001-request-families` and covers document sync plus document symbols,
  workspace symbols, completion, hover, definition, references, semantic tokens,
  inlay hints, folding ranges, code actions, formatting, and pull diagnostics
  through one deterministic stdio LSP session.

## m36.8 Closeout Coverage

The Phase 36 protocol matrix keeps user-facing budget labels on individual LSP
request families. Before M12 those labels were covered by the aggregate
stdio-session budget `perf.lsp.request_families`; after M12, the labels map to
the concrete per-family budgets recorded below while the aggregate remains smoke
coverage only:

- `lsp-cold-start`
- `lsp-code-actions`
- `lsp-completion`
- `lsp-definition`
- `lsp-did-change-diagnostics`
- `lsp-did-open-diagnostics`
- `lsp-document-highlights`
- `lsp-document-symbols`
- `lsp-folding-ranges`
- `lsp-formatting`
- `lsp-hover`
- `lsp-inlay-hints`
- `lsp-references`
- `lsp-rename`
- `lsp-selection-range`
- `lsp-semantic-tokens`
- `lsp-signature-help`
- `lsp-type-hierarchy`
- `lsp-workspace-diagnostics`
- `lsp-workspace-symbols`

## TypeScript-Go Transfer M12 Implemented Evidence

M12 splits the aggregate LSP benchmark into per-family performance manifest
cases. `perf.lsp.request_families` remains as aggregate smoke coverage only.
The enforced request-family mappings are:

| Protocol label | Budget id | Manifest case |
| --- | --- | --- |
| `lsp-cold-start` | `perf.lsp.cold_start.workspace` | `lsp-query-002-cold-start` |
| `lsp-did-open-diagnostics` | `perf.lsp.document_sync.did_open` | `lsp-query-018-did-open-diagnostics` |
| `lsp-did-change-diagnostics` | `perf.lsp.diagnostics.document` | `lsp-query-003-diagnostics` |
| `lsp-workspace-diagnostics` | `perf.lsp.diagnostics.workspace` | `lsp-query-004-workspace-diagnostics` |
| `lsp-completion` | `perf.lsp.completion.local_scope` | `lsp-query-005-completion` |
| `lsp-hover` | `perf.lsp.hover.symbol` | `lsp-query-006-hover` |
| `lsp-signature-help` | `perf.lsp.signature_help.call` | `lsp-query-007-signature-help` |
| `lsp-definition`, `lsp-document-highlights`, `lsp-document-symbols`, `lsp-folding-ranges`, `lsp-workspace-symbols` | `perf.lsp.navigation.symbol` | `lsp-query-008-navigation` |
| `lsp-references` | `perf.lsp.references.workspace_symbol` | `lsp-query-009-references` |
| `lsp-rename` | `perf.lsp.rename.workspace_edit` | `lsp-query-010-rename` |
| `lsp-semantic-tokens` | `perf.lsp.semantic_tokens.full` | `lsp-query-011-semantic-tokens` |
| `lsp-inlay-hints` | `perf.lsp.inlay_hints.module` | `lsp-query-012-inlay-hints` |
| `lsp-selection-range` | `perf.lsp.selection_ranges.nested` | `lsp-query-013-selection-range` |
| `lsp-type-hierarchy` | `perf.lsp.type_hierarchy.symbol` | `lsp-query-014-type-hierarchy` |
| `lsp-code-actions` | `perf.lsp.code_action.diagnostic` | `lsp-query-015-code-actions` |
| `lsp-formatting` | `perf.lsp.formatting.document` | `lsp-query-016-formatting` |
| generated Rust preview command | `perf.lsp.generated_rust_preview.document` | `lsp-query-017-generated-rust-preview` |

Reserved ids not listed in the implemented evidence table are intentionally
deferred. They keep the Phase 36 budget namespace stable for later auto-import,
delta-token, document-symbol, folding-range, rename-prepare, document-highlight,
workspace-symbol, didChange, recovery, transport initialize, and shutdown cases,
but M12 does not claim enforcement for them until a manifest case maps to the id.
