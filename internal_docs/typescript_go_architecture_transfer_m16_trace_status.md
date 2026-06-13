# TypeScript-Go Architecture Transfer M16: Trace And Status Surfaces

Status: merged via [#2263](https://github.com/sifr-lang/sifr/pull/2263)

M16 normalizes compiler-service trace and status output so stale requests,
cache state, invalidation, and editor-facing readiness can be explained from a
single snapshot.

## Trace

`WorkspaceTracePhase` defines the deterministic phase vocabulary:
`SourceUpdate`, `Parse`, `Lower`, `TypeCheck`, `Ownership`, `Flow`, `Cache`,
`Invalidation`, `Scheduler`, `Cancellation`, `StaleRejection`, and `LspTiming`.
`WorkspaceSession` records source updates, compiler phase summaries,
invalidation details, cache summaries, and stale rejections into
`WorkspaceTraceLog`. Workspace trace retention is bounded to the newest 256
events.

LSP request scheduling, cancellation, stale diagnostic rejection, and timing
markers use the same trace phase vocabulary in the LSP session trace buffer,
which is also bounded to the newest 256 events and exposed through the custom
`sifr/debugTrace` request. Analysis snapshot stale rejection records the
captured/current workspace and graph/source revisions before returning
`StaleSnapshot`.

## Status

`WorkspaceDebugSnapshot` combines `WorkspaceStatusSnapshot` with the trace log.
Status output includes open-file, project, source, module, dependency, cache,
index-readiness, last-update latency, watcher/config, build-info, and retained
source-byte counters. The memory counters are deterministic local counts, not
telemetry or heap introspection.

`AnalysisHost::debug_snapshot` enriches frontend status with
`SymbolBucketReadiness` for workspace/package/stdlib index buckets without
building the symbol index on demand. If the index has not been built yet, the
status reports unavailable readiness. LSP analysis open/update paths feed
last-update latency counters into workspace status. `sifr trace <file>` prints
a representative CLI trace/status snapshot for project or single-file inputs.

## Validation

- `cargo test -p sifr_frontend workspace_session -- --nocapture` -> PASS, 9 tests
- `cargo test -p sifr_frontend` -> PASS, 43 tests
- `cargo test -p sifr_analysis dependency_sensitive_invalidation_is_explained_in_trace -- --nocapture` -> PASS
- `cargo test -p sifr_analysis stale_snapshot_is_rejected_after_update -- --nocapture` -> PASS
- `cargo test -p sifr_analysis` -> PASS, 21 tests
- `cargo test -p sifr_lsp debug_trace_request_exposes_lsp_trace_events -- --nocapture` -> PASS
- `cargo test -p sifr_lsp active_request_cancellation_fails_phase_boundary_checks -- --nocapture` -> PASS
- `cargo test -p sifr_lsp diagnostic_job_version_guard_rejects_stale_capture -- --nocapture` -> PASS
- `cargo test -p sifr_lsp` -> PASS, 24 tests
- `cargo test -p sifr trace_entrypoint_renders_status_and_trace_snapshot -- --nocapture` -> PASS
- `cargo test -p sifr -- --skip test_e2e_pass` -> PASS, 57 unit tests and 33 non-pass e2e tests
- `cargo check -p sifr_frontend -p sifr_analysis -p sifr_lsp -p sifr` -> PASS
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_frontend -p sifr_analysis -p sifr_lsp -p sifr -- -D warnings` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED
- Claude reviewer pass 2 -> SATISFIED with residual recommendations
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time 295.57s, advisory: group skew is high
