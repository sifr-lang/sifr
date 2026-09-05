# TypeScript-Go Architecture Transfer: Analysis Snapshot

status: analysis snapshot implementation status

analysis snapshot migrates editor-facing analysis from revision-token snapshots to captured
`WorkspaceSnapshot` handles while preserving serialized execution. The
`AnalysisHost` now owns a `WorkspaceSession`; `AnalysisSnapshot` carries the
frozen workspace state plus the analysis graph/source revision used by existing
metadata and symbol-index keys.

## Snapshot Query Boundary

`AnalysisSnapshot` exposes forwarding methods for diagnostics, workspace
diagnostics, completion, hover, signature help, navigation, references, rename,
symbols, semantic tokens, inlay hints, folding/selection ranges, type hierarchy,
code actions, formatting, generated Rust preview, diagnostic explanation, and
test discovery/commands.

Each forwarding method first checks that the captured workspace revision and
analysis graph/source revision still match the live serialized host. Returned
query metadata records the captured `workspace_snapshot_id`, making stale-result
identity available before analysis snapshot introduces asynchronous scheduling.

Direct `AnalysisHost` query methods remain available for existing tests and
internal callers, but LSP requests now capture a snapshot in `DocumentState` and
execute through the snapshot methods. This keeps current request handling
serialized while making the snapshot boundary mechanical.

## Workspace Snapshot State

`WorkspaceSnapshot` now includes a `dirty_scope_report` field. analysis snapshot records a
conservative placeholder:

- loaded sessions start with `WorkspaceDirtyScope::None`;
- reloads, overlay changes, and analysis document updates record
  `WorkspaceDirtyScope::Workspace` with a reason.

Later dirty-scope work still owns event compaction and precise dirty-scope classification. This slice only
ensures the report is part of the captured snapshot shape consumed by analysis.

## Validation

analysis snapshot focused validation so far:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy -p sifr_analysis -p sifr_lsp -p sifr_frontend -- -D warnings`
- `cargo clippy --workspace -- -D warnings`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 234.97s

The full create-pr validation gate has passed for the analysis snapshot implementation branch.
