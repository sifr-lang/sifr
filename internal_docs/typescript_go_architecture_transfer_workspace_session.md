# TypeScript-Go Architecture Transfer: Workspace Session

status: workspace-session owner implementation status

workspace-session owner introduces the mutable compiler-service owner in `sifr_frontend` while
leaving analysis-query migration to workspace-session owner. The session is intentionally serialized:
callers mutate overlays or reload project state, then freeze the current state
into an inspectable `WorkspaceSnapshot`.

## Session Model

`WorkspaceSession` owns:

- the workspace target, either project root or single-file target;
- open-file overlays as `OverlayDocument` records;
- the most recently loaded `FrontendContext`;
- provider dependency records captured during project reload;
- workspace revision and monotonic snapshot id counters;
- compiler options, package/config identity placeholders, and cache registry
  generation handles for later snapshot and cache expansion.

Project reload builds an `OverlaySourceProvider<DiskSourceProvider>` from the
session overlay table, wraps it in `TrackingSourceProvider`, and loads the
project through `FrontendContext::load_project_with_external_defs_and_auxiliary_sources`.
The resulting
dependency records become session-owned state instead of one-off provider output.
`upsert_overlay` and `remove_overlay` mutate session state and bump the workspace
revision, but callers still invoke `reload` before they expect `source_map` or
`module_graph` snapshot views to reflect the overlay change.

Single-file reload uses the session-owned overlay when present, or the original
single-file source captured during `open_single_file`. Single-file reloads do
not perform provider reads, so their tracked dependency list is intentionally
empty in workspace-session owner.

`WorkspaceSession` also stores caller-provided base `ExternalDefs`. Callers that
do not supply definitions keep an empty base, while `sifr_analysis::AnalysisHost`
passes the compiler's embedded stdlib definitions from
`sifr_driver::stdlib_external_defs()`. Every project and single-file reload
clones those base definitions before rebuilding workspace exports, so editor
analysis cannot lose `sifr.*` imports after opening, changing, or refreshing a
document.

## Snapshot Model

`WorkspaceSnapshot` currently freezes:

- snapshot id and workspace revision;
- session target;
- overlay records;
- tracked source dependencies;
- source map and module graph views cloned from `FrontendContext`;
- compiler options;
- package/config identity;
- cache registry generation handles.

The snapshot is currently an inspectable data object. Later analysis work will
convert `sifr_analysis::AnalysisSnapshot` into an analysis-facing handle to this
state and make editor queries consume captured immutable snapshots.
`WorkspaceSession::snapshot` takes `&mut self` because the serialized session
model allocates snapshot ids from a session-owned counter. Scheduler work must
revisit that boundary before concurrent snapshot capture is introduced.
`WorkspaceSnapshot::source_map` and `module_graph` are optional only for
unreloaded sessions; the public `open_project`, `open_single_file`, and
successful `reload` paths populate both views.
`WorkspaceSession::context` is an inspection escape hatch during the session and
snapshot migration and should not become the long-term query surface once
snapshots are canonical.

## Validation

workspace-session owner focused validation so far:

- `cargo test -p sifr_frontend workspace_session`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `git diff --check`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 261.09s

The validation list is focused on the workspace-session owner data-model surface plus the work's
authoritative create-pr gate.
