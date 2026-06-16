# TypeScript-Go Architecture Transfer M6 Event Compaction And Dirty Scope

status: M6 implementation review

M6 adds the first precise invalidation vocabulary for the serialized compiler
service. The implementation still runs synchronously, but raw LSP and watcher
events now collapse into compact summaries before they update analysis state.

## Document Event Compaction

`sifr_lsp::document_events` compacts each `textDocument/didChange` batch before
`DocumentStore` mutates document state:

- the latest full-document replacement discards earlier edits in the same
  notification;
- incremental edits after that replacement remain ordered;
- empty change batches are rejected as invalid parameters;
- the session records the raw edit count, compacted edit count, and whether the
  final text changed.

This prevents repeated full-buffer edits in one notification from producing
redundant analysis work while preserving the final editor-visible text.

## Watcher Event Summaries

`workspace/didChangeWatchedFiles` counts the reported changes and sends one
summary through the session-owned analysis workspace. Non-empty watcher batches
select graph-structure invalidation by default. Batches above the LSP watcher
storm threshold degrade to workspace invalidation with `WatcherStorm`.

The threshold is intentionally local to the LSP analysis workspace until later
milestones add first-class watcher registries and scheduler queues.

## Dirty Scope Reports

`WorkspaceDirtyScopeReport` now carries both a scope and a deduplicated ordered
reason set. The scope vocabulary is:

- `None`;
- `OneModule`;
- `ReverseDependencies`;
- `GraphStructure`;
- `ConfigProject`;
- `Workspace`.

Reasons cover no-op document-version changes, source text, imports, exports,
compiler/config/package state, filesystem structure, failed lookups, watcher
storms, and unknown conservative reloads.

Report merging uses a conservative priority order. Higher-severity scopes win;
same-path module or reverse-dependency reports stay narrow; incompatible
module-level reports degrade to graph structure so the report never claims a
single-file invalidation when multiple unrelated files contributed reasons.

## Validation

M6 focused validation so far:

- `cargo test -p sifr_frontend workspace_session`
- `cargo test -p sifr_lsp`
- `cargo fmt --check`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `python3 verification/areas/developer_tooling/lsp_protocol_smoke.py`
- `python3 verification/areas/developer_tooling/lsp_protocol_smoke.py --self-test`
- `python3 verification/areas/developer_tooling/lsp_protocol_stress.py`
- `python3 verification/areas/developer_tooling/lsp_protocol_stress.py --self-test`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test`
- `cargo clippy --workspace -- -D warnings`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 227.48s
