# TypeScript-Go Architecture Transfer M3 Workspace Session

status: M3 implementation review

M3 introduces the mutable compiler-service owner in `sifr_frontend` while
leaving analysis-query migration to M4. The session is intentionally serialized:
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
  generation handles for later M4-M10 expansion.

Project reload builds an `OverlaySourceProvider<DiskSourceProvider>` from the
session overlay table, wraps it in `TrackingSourceProvider`, and loads the
project through `FrontendContext::load_project_with_provider`. The resulting
dependency records become session-owned state instead of ad hoc provider output.
`upsert_overlay` and `remove_overlay` mutate session state and bump the workspace
revision, but callers still invoke `reload` before they expect `source_map` or
`module_graph` snapshot views to reflect the overlay change.

Single-file reload uses the session-owned overlay when present, or the original
single-file source captured during `open_single_file`. Single-file reloads do
not perform provider reads, so their tracked dependency list is intentionally
empty in M3.

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

The snapshot is an inspectable data object only in M3. M4 will convert
`sifr_analysis::AnalysisSnapshot` into an analysis-facing handle to this state
and make editor queries consume captured immutable snapshots.
`WorkspaceSession::snapshot` takes `&mut self` because the serialized M3 model
allocates snapshot ids from a session-owned counter. M11 scheduler work must
revisit that boundary before concurrent snapshot capture is introduced.
`WorkspaceSnapshot::source_map` and `module_graph` are optional only for
unreloaded sessions; the public `open_project`, `open_single_file`, and
successful `reload` paths populate both views.
`WorkspaceSession::context` is an inspection escape hatch during M3/M4 migration
and should not become the long-term query surface once snapshots are canonical.

## Validation

M3 focused validation so far:

- `cargo test -p sifr_frontend workspace_session`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `git diff --check`
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py --self-test`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 261.09s

The validation list is focused on the M3 data-model surface plus the phase's
authoritative create-pr gate.
