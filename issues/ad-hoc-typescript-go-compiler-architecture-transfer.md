# Ad Hoc Phase: TypeScript-Go Compiler Architecture Transfer

Status: completed via [#2267](https://github.com/sifr-lang/sifr/pull/2267)

## Execution Tracker

| Milestone | State | PR | Notes |
| --- | --- | --- | --- |
| M0 Source And Position Foundation | merged | [#2229](https://github.com/sifr-lang/sifr/pull/2229) | Added `sifr_source`, real source-map conversions, and source-position guardrails. |
| M1 Architecture Contract And Guardrails | merged | [#2230](https://github.com/sifr-lang/sifr/pull/2230), [#2232](https://github.com/sifr-lang/sifr/pull/2232) | Added pre-flight direct-read/LSP/budget guardrails and follow-up tracker update. |
| M2 Source Provider And Overlay Store | merged | [#2233](https://github.com/sifr-lang/sifr/pull/2233) | Adds `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, `TrackingSourceProvider`, `OverlayDocument`, `SourceDependency*`, provider-backed project/package/lint/format reads, `PackageImportAmbiguity`, and `PackageImportResolutionResult`; new tests cover overlay shadowing, nested overlay directories, tracked reads, provider-backed project loading, package ambiguity, unresolved/private/fatal import states, and existing lint/format/package behavior. |
| M3 Workspace Session Data Model | merged | [#2235](https://github.com/sifr-lang/sifr/pull/2235) | Adds `WorkspaceSession` and `WorkspaceSnapshot` as the serialized mutable compiler-service owner and frozen inspection handle for overlays, tracked dependencies, source maps, module graphs, compiler options, package/config identity, cache-registry handles, and revision counters while leaving analysis/LSP migration to M4/M5. |
| M4 Analysis Snapshot Migration | merged | [#2237](https://github.com/sifr-lang/sifr/pull/2237) | Migrates `sifr_analysis::AnalysisSnapshot` to carry a captured `WorkspaceSnapshot`, routes LSP analysis requests through snapshot methods, adds conservative snapshot dirty-scope state, and keeps execution serialized before scheduler work. |
| M5 LSP Persistent Session Integration | merged | [#2238](https://github.com/sifr-lang/sifr/pull/2238) | Moves LSP analysis ownership from `DocumentStore` into the serialized `Session`, feeds open/change/save buffers into `WorkspaceSession` overlays, and rejects stale request publication by captured snapshot plus document version while preserving serialized request handling. |
| M6 Event Compaction And Dirty Scope | merged | [#2239](https://github.com/sifr-lang/sifr/pull/2239) | Compacts batched document edits before analysis updates, summarizes watcher events before dirty-scope classification, records precise dirty scope/reason reports, and degrades incompatible or stormy invalidation conservatively. |
| M7 Module Signatures And Dependency Invalidation | merged | [#2241](https://github.com/sifr-lang/sifr/pull/2241) | Adds import/export/module signatures, reverse-dependency closure invalidation, and local private-body edit reuse for unchanged public/import signatures. |
| M8 First-Class Flow Graph | merged | [#2243](https://github.com/sifr-lang/sifr/pull/2243), [#2244](https://github.com/sifr-lang/sifr/pull/2244), [#2245](https://github.com/sifr-lang/sifr/pull/2245) | Adds `sifr_hir::flow_graph`, snapshot-scoped `LoweringResult.flow_graph`, graph-backed `FlowFacts` debug/fingerprint access, and lowering-time flow effects for narrowing, mutation invalidation, moves, and borrows. |
| M9 Fingerprints And Cache Keys | merged | [#2246](https://github.com/sifr-lang/sifr/pull/2246), [#2247](https://github.com/sifr-lang/sifr/pull/2247), [#2248](https://github.com/sifr-lang/sifr/pull/2248), [#2249](https://github.com/sifr-lang/sifr/pull/2249) | Adds deterministic compiler/cache fingerprints and typed key identities for parse, source-map, HIR/lowering, diagnostics, lint, format, package graph, symbol bucket, and flow graph caches before reuse lands. |
| M10 Snapshot Reuse And Structural Replacement | merged | [#2251](https://github.com/sifr-lang/sifr/pull/2251) | Adds ref-counted M9-keyed parse/source-map/HIR/diagnostics/index reuse, Arc-backed snapshot payloads, and conservative safe one-module replacement when import/export signatures are unchanged. |
| M11 LSP Scheduler Queues | merged | [#2253](https://github.com/sifr-lang/sifr/pull/2253) | Adds real request priority queues for latency-sensitive, formatting, workspace, and background lanes plus debounced diagnostic jobs guarded by captured document versions. |
| M12 Per-Request Editor Latency Budgets | merged | [#2255](https://github.com/sifr-lang/sifr/pull/2255) | Splits aggregate LSP request-family performance evidence into per-request protocol latency scenarios with explicit `perf.lsp.*` budgets while retaining the aggregate case as smoke coverage. |
| M13 LSP Cancellation, Progress, And Watchdog | merged | [#2257](https://github.com/sifr-lang/sifr/pull/2257) | Adds queued/in-flight request cancellation state, phase-boundary cancellation checks, delayed work progress for multi-document diagnostics, and `sifr lsp --parent-pid` watchdog plumbing. |
| M14 Bucketed Indexes And Safe Parallel Lanes | merged | [#2259](https://github.com/sifr-lang/sifr/pull/2259) | Adds workspace/package/stdlib symbol and import bucket readiness states, dirty-bucket symbol-index refreshes, and explicit approved worker-lane versus single-owner compiler-phase policy; package and stdlib buckets are explicit unavailable states until frontend graph views carry those identities. |
| M15 Project Residency, Watchers, And Build Info | merged | [#2261](https://github.com/sifr-lang/sifr/pull/2261) | Adds project residency snapshots, config pending reload state, deduped watch registrations, and verified non-authoritative `.sifrbuildinfo` metadata. |
| M16 Trace And Status Surfaces | merged | [#2263](https://github.com/sifr-lang/sifr/pull/2263) | Adds deterministic compiler-service trace phases, bounded debug status snapshots, LSP scheduler/cancellation/stale/timing trace events via `sifr/debugTrace`, analysis index-readiness status, and `sifr trace` CLI output. |
| M17 Editor Corpus And Snapshot Handles | merged | [#2265](https://github.com/sifr-lang/sifr/pull/2265) | Adds marker-based multi-file editor query corpus fixtures, internal snapshot-scoped handles, and runtime package diagnostic non-duplication fixtures. |

Final phase closure:

- Claude full implementation review pass 1 -> SATISFIED (`reviews/typescript-go-architecture-transfer-full-implementation-review-pass-1.md`)
- Latest authoritative milestone gates: M16 `scripts/run_all_tests.sh --profile quick` -> PASS, wall time 295.57s; M17 `scripts/run_all_tests.sh --profile quick` -> PASS, wall time 279.93s
- All listed milestones M0-M17 are merged, and the final review found no blocking phase-level findings.

M2 local validation so far:

- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo fmt --check`
- `python3 scripts/check_file_size_guardrails.py`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo test -p sifr_driver -p sifr_package -p sifr_frontend -p sifr_format -p sifr_lint`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 274.59s

M3 local validation so far:

- `cargo test -p sifr_frontend workspace_session`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --workspace -- -D warnings`

M10 local validation so far:

- `cargo test -p sifr_frontend ref_counted_module_caches_reuse_identity_on_hits`
- `cargo test -p sifr_frontend structural_one_module_replacement_reuses_unchanged_cache_entries`
- `cargo test -p sifr_frontend document_version_only_update_recaches_source_file_view`
- `cargo test -p sifr_frontend reverse_dependent_invalidation_reuses_unchanged_parse_entry`
- `cargo test -p sifr_frontend dunder_method_signature_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend single_underscore_method_signature_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend class_decorator_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend leading_whitespace_edit_preserves_export_signature_scope`
- `cargo test -p sifr_frontend public_constant_value_update_invalidates_reverse_dependents`
- `cargo test -p sifr_frontend` -> PASS, 38 tests
- `cargo check --workspace`
- `cargo fmt --check`
- `cargo clippy -p sifr_frontend -- -D warnings`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m10-snapshot-reuse-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual recommendations (`reviews/typescript-go-m10-snapshot-reuse-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m10-snapshot-reuse-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 330.61s, advisories: warm wall-time budget exceeded; group skew is high

M11 local validation so far:

- `cargo test -p sifr_lsp` -> PASS, 13 tests
- `python3 verification/tooling/lsp_protocol_smoke.py` -> PASS
- `python3 verification/tooling/lsp_protocol_stress.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_lsp -- -D warnings` -> PASS
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m11-lsp-scheduler-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual low-priority cleanup (`reviews/typescript-go-m11-lsp-scheduler-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m11-lsp-scheduler-review-pass-3.md`)
- `cargo clippy --workspace -- -D warnings` -> PASS
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 263.26s, advisory: group skew is high

M12 local validation so far:

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
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m12-lsp-latency-budgets-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual cleanup (`reviews/typescript-go-m12-lsp-latency-budgets-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m12-lsp-latency-budgets-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 256.95s, advisory: group skew is high

M13 local validation so far:

- `cargo fmt --check` -> PASS
- `cargo build -p sifr` -> PASS
- `cargo test -p sifr_lsp` -> PASS, 23 tests
- `cargo clippy -p sifr_lsp -p sifr -- -D warnings` -> PASS
- `python3 -m py_compile verification/tooling/lsp_protocol.py verification/tooling/lsp_protocol_smoke.py verification/tooling/lsp_protocol_stress.py` -> PASS
- `python3 verification/tooling/lsp_protocol_smoke.py` -> PASS
- `python3 verification/tooling/lsp_protocol_stress.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py --self-test` -> PASS
- `python3 scripts/check_diagnostic_cancel_usage.py` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> SATISFIED with residual cleanup (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual cleanup (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 280.41s, advisory: group skew is high

M14 local validation so far:

- `cargo test -p sifr_analysis symbol_index -- --nocapture` -> PASS
- `cargo test -p sifr_analysis worker_lanes -- --nocapture` -> PASS
- `cargo test -p sifr_analysis` -> PASS, 20 tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_analysis -p sifr_frontend -- -D warnings` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m14-bucketed-indexes-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual cleanup (`reviews/typescript-go-m14-bucketed-indexes-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m14-bucketed-indexes-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 261.03s, advisory: group skew is high

M15 local validation so far:

- `cargo test -p sifr_frontend workspace_session` -> PASS, 8 tests
- `cargo test -p sifr_frontend` -> PASS, 42 tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_frontend -- -D warnings` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m15-project-residency-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED (`reviews/typescript-go-m15-project-residency-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m15-project-residency-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 292.53s, advisory: group skew is high

M16 local validation so far:

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
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m16-trace-status-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual recommendations (`reviews/typescript-go-m16-trace-status-review-pass-2.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 295.57s, advisory: group skew is high

M17 local validation so far:

- `cargo test -p sifr_analysis marker_editor_corpus_covers_multifile_queries_and_stale_snapshots -- --nocapture` -> PASS
- `cargo test -p sifr_analysis snapshot_handles_are_internal_and_reject_wrong_snapshot_resolution -- --nocapture` -> PASS
- `cargo test -p sifr_analysis` -> PASS, 23 tests
- `python3 verification/tooling/check_diagnostic_source_canonicalization_contract.py` -> PASS
- `python3 verification/tooling/check_diagnostic_source_canonicalization_contract.py --self-test` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `cargo test -p sifr -- --skip test_e2e_pass` -> PASS, 57 unit tests and 33 non-pass e2e tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_analysis -p sifr -- -D warnings` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- `git diff --check` -> PASS
- Claude reviewer pass 1 -> SATISFIED with residual recommendations (`reviews/typescript-go-m17-editor-corpus-snapshot-handles-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED (`reviews/typescript-go-m17-editor-corpus-snapshot-handles-review-pass-2.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 279.93s, advisory: group skew is high

M4 local validation so far:

- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo fmt --check`
- `git diff --check`
- `cargo clippy -p sifr_analysis -p sifr_lsp -p sifr_frontend -- -D warnings`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 234.97s

M5 local validation so far:

- `cargo test -p sifr_lsp`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `python3 verification/tooling/lsp_protocol_smoke.py`
- `python3 verification/tooling/lsp_protocol_smoke.py --self-test`
- `python3 verification/tooling/lsp_protocol_stress.py`
- `python3 verification/tooling/lsp_protocol_stress.py --self-test`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 227.66s

M6 local validation so far:

- `cargo test -p sifr_frontend workspace_session`
- `cargo test -p sifr_lsp`
- `cargo fmt --check`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `python3 verification/tooling/lsp_protocol_smoke.py`
- `python3 verification/tooling/lsp_protocol_smoke.py --self-test`
- `python3 verification/tooling/lsp_protocol_stress.py`
- `python3 verification/tooling/lsp_protocol_stress.py --self-test`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo clippy --workspace -- -D warnings`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 227.48s

M7 local validation so far:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- `cargo clippy -p sifr_frontend -p sifr_analysis -p sifr_lsp -- -D warnings`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `python3 verification/tooling/lsp_protocol_smoke.py`
- `python3 verification/tooling/lsp_protocol_smoke.py --self-test`
- `python3 verification/tooling/lsp_protocol_stress.py`
- `python3 verification/tooling/lsp_protocol_stress.py --self-test`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 254.29s, advisory: group skew is high

M8 local validation so far:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_hir flow_graph -- --nocapture`
- `cargo test -p sifr_hir`
- `cargo test -p sifr_driver`
- `cargo test -p sifr_frontend -p sifr_analysis -p sifr_lsp`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 306.27s, advisories: warm wall-time budget exceeded; group skew is high
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m8-first-class-flow-graph-review-pass-3.md`)
- M8 loop-else follow-up: `cargo fmt --check`, `cargo test -p sifr_hir flow_graph -- --nocapture`, `cargo clippy -p sifr_hir -- -D warnings`
- Claude reviewer loop-else follow-up pass 1 -> SATISFIED (`reviews/typescript-go-m8-loop-else-follow-up-review-pass-1.md`)
- M8 closeout quick validation: `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 280.58s, advisory: group skew is high
- Claude reviewer closeout pass 1 -> SATISFIED (`reviews/typescript-go-m8-closeout-review-pass-1.md`)

M9 local validation so far:

- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo test -p sifr_frontend cache_key -- --nocapture`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis -p sifr_lsp`
- `cargo test -p sifr_driver`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy -p sifr_frontend -- -D warnings`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 338.91s, advisories: warm wall-time budget exceeded; group skew is high
- Claude reviewer pass 7 -> SATISFIED (`reviews/typescript-go-m9-fingerprints-cache-keys-review-pass-7.md`)

## Purpose

Turn the TypeScript-Go architecture review into concrete Sifr work.

The goal is not to copy TypeScript or JavaScript compatibility behavior. The goal is to adopt the TypeScript-Go compiler-service ideas that are valuable for Sifr's own guarantees:

- coherent workspace snapshots
- overlay-aware file access
- reusable parse/HIR caches
- dependency-sensitive invalidation
- structural module replacement
- copy-on-write project state
- deterministic background analysis
- project residency and watcher hygiene
- bucketed import/symbol indexes
- first-class flow graph state
- compiler tracing and editor regression infrastructure
- per-request editor latency budgets and enforcement

This phase should move Sifr from request-local analysis toward a production compiler service model that can support CLI, LSP, package, formatter, linter, and future compiler API consumers without split-brain state.

## Source Inputs

This phase is based on the local TypeScript-Go codebase review and current Sifr frontend/tooling contracts.

TypeScript-Go source evidence:

- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/session.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/snapshot.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/snapshotfs.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/overlayfs.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/parsecache.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/refcountcache.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/checkerpool.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/projectcollection.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/projectcollectionbuilder.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/configfileregistry.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/configfileregistrybuilder.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/watch.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/background/queue.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/project/dirty`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/compiler/program.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/compiler/checkerpool.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/core/workgroup.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/core/semaphore.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/ast/flow.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/checker/flow.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/tracing/tracing.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/vfs/vfs.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/vfs/cachedvfs/cachedvfs.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/vfs/trackingvfs/trackingvfs.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/vfs/vfswatch/vfswatch.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/ls/autoimport/registry.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/ls/autoimport/index.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/ls/lsconv/linemap.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/lsp/server.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/lsp/progress.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/api/session.go`
- `/Users/yaseralnajjar/work/sifr/typescript-go/_packages/native-preview/src/api/objectRegistry.ts`
- `/Users/yaseralnajjar/work/sifr/typescript-go/internal/fourslash`

Sifr source evidence:

- `internal_docs/architecture.md`
- `internal_docs/frontend_query_architecture.md`
- `internal_docs/frontend_cache_invalidation.md`
- `internal_docs/lsp_server.md`
- `internal_docs/narrowing_flow_facts_design.md`
- `internal_docs/performance_budgets.md`
- `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
- `verification/performance/manifest.json`
- `verification/performance/budgets.json`
- `crates/sifr_frontend/src/graph_cache_and_queries.rs`
- `crates/sifr_analysis/src/lib.rs`
- `crates/sifr_analysis/src/host/implementation.rs`
- `crates/sifr_analysis/src/snapshot.rs`
- `crates/sifr_lsp/src/session.rs`
- `crates/sifr_lsp/src/scheduler.rs`
- `crates/sifr_lsp/src/document_store.rs`
- `crates/sifr_hir/src/lower/narrowing.rs`
- `crates/sifr_hir/src/lower/flow_helpers.rs`
- `crates/sifr_type_system/src/narrow.rs`
- `crates/sifr_diagnostics/src/source_map/mod.rs`
- `crates/sifr_driver/src/project/discovery.rs`
- `crates/sifr_driver/src/project/compile_order.rs`

## Quality Contract

Entry criteria:

- The TypeScript-Go architecture findings are reviewed and accepted as Sifr-relevant, not copied blindly.
- Phase 35 query architecture and Phase 36 tooling contracts remain the baseline.
- Existing formatter, linter, diagnostics, package, and LSP contracts are not regressed.
- The phase plan is reviewed before implementation begins.

Exit criteria:

- Sifr has a workspace/session snapshot model shared by CLI analysis and LSP requests.
- Open-buffer overlays, disk files, package discovery, and dependency reads flow through a typed file-system boundary.
- Parse and HIR results are reused across snapshots when content and compiler options are unchanged.
- Module invalidation is dependency-sensitive instead of whole-project for every text change.
- Sifr can classify dirty work as none, one module, reverse-dependency fanout, graph structure, or workspace-level invalidation.
- A structural replacement predicate decides when a changed module can be swapped into an existing project snapshot without rebuilding the project graph.
- Immutable snapshots are built with copy-on-write state so unchanged maps, source maps, indexes, config entries, and diagnostics are retained by identity.
- LSP request handling uses captured snapshots and rejects stale publications.
- Watch and editor events are compacted before invalidation and excessive event storms degrade intentionally to workspace-cache invalidation.
- Config/project/watch registries retain only projects, configs, and watchers needed by open files, explicit API sessions, and referenced projects.
- CLI/build mode has a reviewed persistent incremental metadata design for cold-start acceleration.
- Flow graph state is represented explicitly enough to support narrowing, ownership facts, and future exhaustiveness checks.
- Deterministic compiler traces can explain parse, lower, check, flow, cache, invalidation, and LSP latency.
- A debug status surface can report open files, projects, snapshots, cache sizes, index readiness, last update latency, and memory counters.
- Per-request editor/LSP latency budgets are split, enforced, and reported separately instead of relying only on an aggregate request-family benchmark.
- Editor regression tests cover multi-file snapshot behavior and stale result handling.

Required quality controls:

- No fallback analysis path may silently bypass the snapshot/session model after it is introduced.
- Structural replacement is allowed only when parse options, imports, exported signatures, top-level declarations, and package-visible metadata are unchanged.
- Dirty-scope classification must degrade conservatively: uncertainty becomes graph or workspace invalidation, not stale reuse.
- No cache key may omit source content hash, parser/options fingerprint, Sifr version-sensitive compiler configuration, or package/workspace context when it affects results.
- Cache reuse must never mix diagnostics, HIR nodes, type identities, or source spans from incompatible snapshots.
- Persistent incremental metadata may accelerate cold start but must never be the source of truth for correctness.
- Public API changes must update internal docs and any affected tooling verification manifests.
- Parallel analysis must preserve deterministic diagnostic order and must not expose non-thread-safe type or HIR ownership across worker boundaries.
- Stale LSP results must be rejected by snapshot/version identity, not by timing assumptions.
- File watching and tracking must record both successful reads and failed lookup paths that affect module resolution.
- Raw LSP and watcher events must be compacted before invalidation; repeated edits or file storms must not enqueue unbounded redundant work.
- LSP progress and debug/status reporting must be delayed and bounded so fast operations stay quiet and long operations are explainable.
- Editor latency budgets must measure protocol-level behavior, not only frontend-query internals.
- Aggregate LSP request-family benchmarks may remain as broad smoke coverage, but they cannot be the only enforced LSP latency gate after this phase.
- Each per-feature LSP budget must record representative fixture shape, warm/cold state, baseline median/p95, threshold, and whether the query requires a warm index.
- Flow graph work must improve general compiler reasoning. Do not add fixture-specific narrowing recognizers.
- Local validation evidence must be recorded in the execution tracker before each milestone closes.

## Problem Statement

Sifr has strong compiler correctness goals, but the current frontend and LSP shape still has service-level gaps:

- LSP paths can rebuild request-local analysis hosts from open documents instead of sharing a coherent workspace snapshot.
- `FrontendContext::update_module_source` invalidates broad analysis state for all modules after a text change.
- Sifr has no explicit dirty-scope model that distinguishes one-module edits, reverse-dependency fanout, graph-structure changes, config changes, and workspace invalidation.
- Sifr has no structural predicate comparable to TypeScript-Go's fast path for replacing one source file while preserving the project program.
- Project loading and module discovery read the physical filesystem directly in several places instead of going through a layered VFS that can model open editor buffers, disk state, package archives, and failed lookups.
- Package source-map construction can reject duplicate package module paths before import resolution can emit the canonical source-level ambiguous-import diagnostic for a concrete import site.
- Open/change/save/watch notifications are handled directly instead of being compacted into a smaller event summary before invalidation.
- There is no copy-on-write project state model for cheaply finalizing immutable snapshots from mostly-unchanged maps.
- The query/cache foundation exists, but parse/HIR reuse is not yet a snapshot-scoped service with ref-counted lifetime management.
- There is no persistent `.sifrbuildinfo`-style metadata design for build-mode cold-start acceleration.
- The LSP lane model is currently a dispatch label rather than a real scheduler with cancellation, debounce, background priority, progress, and stale-result control.
- Project retention, config retention, and watcher deduplication are not yet explicit concepts.
- Workspace symbol/import indexes are not bucketed by project, package, and stdlib scope, so future auto-import/completion scaling would tend toward broad rebuilds.
- Narrowing and flow facts are present, but there is no single first-class flow graph substrate comparable to TypeScript-Go's `FlowNode` model.
- Performance budgets exist, but compiler-service tracing is not yet rich enough to explain why a request was slow or why a cache entry was invalidated.
- Editor latency budgets are split across Phase 35 frontend-query budgets and Phase 36 protocol-level LSP budget plans.
- The enforced performance manifest currently has editor-adjacent frontend cases for cold context load, warm diagnostics, unchanged update, changed-file invalidation, and source-map lookup, plus one aggregate LSP request-family case.
- Phase 36 planned per-feature LSP budgets for cold start, didOpen/didChange diagnostics, completion, hover, signature help, definition, references, rename, semantic tokens, inlay hints, selection ranges, type hierarchy, code actions, formatting, and generated Rust preview, but those budgets are not fully split into separate enforced manifest cases.
- There is no LSP status/debug command that exposes project, snapshot, cache, index, and memory state for editor bug reports.
- Editor protocol tests exist, but Sifr lacks a marker-rich, multi-file corpus equivalent to TypeScript-Go's fourslash tests for hover, rename, completion, references, diagnostics, and stale snapshots.

The root issue is architectural: Sifr needs a production compiler-service layer, not more request-local one-off analysis paths.

## Product Decision

Sifr will implement a Sifr-owned compiler-service architecture inspired by TypeScript-Go.

The target architecture is:

```text
disk/package files + open editor buffers
  -> sifr_frontend file-system abstraction
  -> workspace session
  -> immutable workspace snapshots
  -> dirty-scope and structural-replacement classifier
  -> copy-on-write project state
  -> parse/HIR/cache services
  -> dependency-sensitive invalidation
  -> bucketed indexes and scheduler lanes
  -> analysis, lint, format, diagnostics, LSP, package tooling, build metadata
  -> deterministic trace, per-feature budgets, and editor regression artifacts
```

The TypeScript-Go concepts are adopted as design inputs, not semantic authority. Sifr remains a Python-syntax compiled language with Rust codegen, static typing, Result/Option safety, and ownership semantics. JavaScript compatibility behavior, TypeScript declaration semantics, CommonJS/Node resolution, JSDoc support, and TypeScript-specific checker APIs are out of scope.

## Scope

In scope:

1. Add a workspace/session layer in `sifr_frontend` that owns open-file overlays, disk/package file access, compiler options, package graph identity, and immutable snapshot creation.
2. Introduce a `WorkspaceSnapshot` identity that analysis, LSP, lint, format, and package queries can carry through results.
3. Replace direct filesystem reads in frontend/project discovery paths with a typed file-system abstraction, including package source-map reads and ambiguity records.
4. Add overlay-aware file handles with content hash, version, source text, source map, and optional editor-document metadata.
5. Track dependency reads, failed lookups, directory reads, config reads, and package metadata reads so watch invalidation can be precise.
6. Add ref-counted parse and HIR caches keyed by content hash plus parser/lower/options fingerprints.
7. Add a `DirtyScope` classification that distinguishes no-op, one-module, reverse-dependency, graph-structure, config/project, and workspace invalidation.
8. Add a `can_replace_module_in_project` structural predicate that compares imports, exported names, function/class/type signatures, top-level declarations, public constants, parser options, and package-visible metadata.
9. Add export-signature and dependency-graph invalidation so private body edits stay local and public API edits invalidate reverse dependents.
10. Add copy-on-write project maps for module graph, source maps, diagnostics, symbol indexes, config registries, and package metadata so snapshots reuse unchanged state by identity.
11. Preserve deterministic diagnostics and query results while allowing safe parallel parse/lower/check work.
12. Add a file-affine analysis pool or worker lane model for expensive per-module analysis where identity isolation is clear.
13. Integrate snapshots into `sifr_lsp` request handling and publication logic.
14. Ensure diagnostics, hover, completion, definition, rename, formatting, linting, and code actions all operate on captured snapshots.
15. Add stale-result rejection based on snapshot and document version identity.
16. Compact open/change/save/close/watch events before invalidation and handle watcher storms with an explicit workspace-cache invalidation path.
17. Make the request scheduler real: priority queues, cancellation tokens, debounce, background lanes, delayed progress, and bounded result publication.
18. Add project residency for open-file owner projects, ancestor solutions, referenced projects, explicit API sessions, and evictable projects.
19. Add config and watcher registries with reverse retention and reference-counted registrations.
20. Derive watched globs from actually seen files, directories, configs, package roots, stdlib roots, and external imports.
21. Add bucketed symbol/import indexes by workspace project, package, and stdlib scope with readiness states for completion and import suggestions.
22. Add a reviewed persistent `.sifrbuildinfo` or equivalent metadata design for CLI/build cold-start acceleration.
23. Introduce first-class flow graph nodes/edges in HIR or a dedicated flow crate for assignments, conditions, branches, loops, calls, mutations, and joins.
24. Rebase narrowing and ownership facts on the flow graph incrementally, without weakening current diagnostics.
25. Complete source-position conversion gaps needed for LSP and editor edits, including UTF-8/UTF-16 boundary tests where protocol conversion requires it.
26. Add deterministic compiler-service tracing for parse, lower, type check, ownership check, flow analysis, cache hit/miss, invalidation, and LSP request phases.
27. Add an LSP status/debug command that reports open files, projects, snapshots, cache entries, index readiness, update latency, and memory counters.
28. Add `sifr lsp --parent-pid` watchdog support so orphaned servers exit when the parent editor process dies.
29. Add marker-based multi-file editor fixtures inspired by fourslash for request/response regression coverage.
30. Add future-facing snapshot-scoped compiler API handles only after snapshots and cache lifetimes are stable.
31. Split aggregate LSP performance coverage into enforced per-request editor latency budgets.
32. Update internal architecture, frontend query, LSP, cache invalidation, performance budget, and tooling verification docs.

Out of scope:

- Copying TypeScript language semantics into Sifr.
- Node/CommonJS/TypeScript module resolution.
- JSDoc, `.d.ts`, JavaScript checking, or TypeScript declaration emit.
- Replacing Ruff-backed parsing, formatting, or lint infrastructure decisions from prior phases.
- Introducing nondeterministic background diagnostics or best-effort stale publications.
- Weakening ownership, Result/Option, panic-safety, or type-soundness guarantees for service performance.
- A public compiler API before snapshot handles and lifetime contracts are proven internally.
- TypeScript auto-import package semantics, Node package resolution, or TypeScript project-reference behavior as semantic authority. Sifr may copy the bucketed indexing and residency patterns only.
- Persistent build metadata that can hide source changes, stale package metadata, or changed compiler options.

## TypeScript-Go Concept Transfer Matrix

| TypeScript-Go concept | Sifr decision | Required Sifr outcome |
| --- | --- | --- |
| Immutable project snapshots | adopt | `WorkspaceSession` produces immutable `WorkspaceSnapshot`s with version identity and coherent file/package/config state. |
| Overlay filesystem | adopt | Open editor buffers overlay disk/package files without mutating canonical disk state. |
| Tracking/cached VFS | adopt | Reads, directory entries, realpaths, config files, package files, and failed lookups are tracked for invalidation. |
| File-change summary and event compaction | adopt | Raw editor/watch events are compacted before dirty-scope classification and invalidation. |
| Ref-counted parse cache | adopt | Parse trees/source-map file views are reused by content/options hash while active frontend context state or retained snapshot payloads reference them. |
| HIR/lowering cache | adopt with Sifr ownership | HIR lowering results are reused by content/options/package context and invalidated by syntax or semantic-boundary changes. |
| Dirty file precision | adopt | `DirtyScope` records whether an edit is none, one module, reverse-dependency fanout, graph structure, config/project, or workspace. |
| Structural program cloning | adopt | `can_replace_module_in_project` allows one-module replacement only when project structure and public interface remain stable. |
| Copy-on-write dirty maps | adopt | Snapshot finalization reuses unchanged maps and indexes by identity. |
| Lazy program-level facts | adopt | Expensive facts such as import closure, exports, reverse dependencies, stdlib scope, and doc index are lazy and clone-aware. |
| Dependency-sensitive program update | adopt | Private edits invalidate local diagnostics; export changes invalidate reverse dependents; imports/config/package changes invalidate affected graph slices. |
| Config registry with reverse retention | adapt | Sifr manifests/configs retain only projects/open files/extended configs that actually depend on them. |
| Project collection and residency | adapt | Open-file, ancestor, referenced, explicit API, and evictable projects have explicit residency state. |
| Watcher registry | adapt | Watch registrations are deduplicated, reference-counted, and derived from seen files/directories/configs/packages. |
| Checker pool | adapt | Parallel analysis lanes may process disjoint modules, but canonical Sifr type/HIR identity cannot be mixed unsafely across workers. |
| Workgroup/semaphore background scheduling | adapt | LSP and CLI background tasks use bounded worker lanes with cancellation and deterministic result ordering. |
| Delayed progress | adapt | Long project loads, workspace diagnostics, references, and index warming report progress only after a threshold. |
| Parent-process watchdog | adopt | `sifr lsp --parent-pid` exits when the parent editor process dies. |
| Bucketed auto-import registry | adapt | Sifr completion/import suggestions use workspace, package, and stdlib buckets with independent dirty/readiness state. |
| Query readiness | adopt | Expensive editor features can report exact, stale-but-usable, refreshing, or unavailable readiness without lying about correctness. |
| FlowNode graph | adopt | Sifr gets explicit flow nodes for narrowing, assignment, mutation, branch/loop joins, calls, and future exhaustiveness analysis. |
| Shared flow-node cache | adapt | Flow analysis caches must be snapshot-scoped and invalidated when HIR/control flow changes. |
| Deterministic tracing | adopt | `sifr trace` or equivalent trace output records compiler-service phases and cache/invalidation decisions. |
| Status/telemetry shape | adopt locally | A debug command exposes project/cache/index/memory state without sending telemetry anywhere. |
| Per-feature LSP budgets | adopt | Protocol-level editor latency budgets are enforced separately for each request family instead of only through one aggregate case. |
| Package source-map ambiguity | adapt | Package maps distinguish fatal package construction errors from import-site ambiguity that must surface as canonical source diagnostics. |
| Disk incremental metadata | adapt | `.sifrbuildinfo`-style metadata may accelerate CLI/build cold start but is never correctness authority. |
| Snapshot-scoped API object registry | future | Add only after internal snapshots are stable; useful for future SDK/editor tools. |
| Fourslash editor corpus | adopt pattern | Add marker-based multi-file fixtures for editor queries and stale snapshot behavior. |

## Prerequisite Discovery Decisions

Discovery status: completed on 2026-05-29.

The current repo has enough compiler-service foundation to start a TypeScript-Go-inspired phase, but several pieces are prerequisites rather than optional cleanup. These decisions lock what must be true before dependent implementation milestones start.

| ID | Decision | Current evidence | Consequence |
| --- | --- | --- | --- |
| D0-1 | Complete source maps are a hard prerequisite. | `SourceMapView` stores file metadata only, and both `text_position_to_span` and `span_to_text_range` return `None` in `crates/sifr_frontend/src/graph_cache_and_queries.rs`. `interactive.source_map_lookup` calls the stub and ignores the result. LSP range conversion uses ad hoc `sifr_syntax::SourceText`, while diagnostic LSP ranges are derived from rendered 1-based line/column fields. | Core source-map completion moves into M0. LSP snapshots, stale-result rejection, diagnostics, semantic tokens, formatting ranges, rename edits, and perf budgets cannot depend on stub conversion behavior. |
| D0-2 | A shared source text and line-map authority is required before overlay snapshots. | `sifr_frontend::SourceText` is a string wrapper, `sifr_syntax::SourceText` has UTF-8 line starts, and `sifr_diagnostics::SourceMap` has a separate line-start model. | M0 must define which type owns canonical line maps and UTF-8/UTF-16/UTF-32 conversions so frontend, diagnostics, and LSP do not keep diverging conversion logic. |
| D0-3 | The VFS/source-provider boundary is a phase-start blocker for project snapshots. | `FrontendContext::load_project` reads entrypoints, project directories, and modules through `std::fs` directly. Additional examples include `crates/sifr_driver/src/project/discovery.rs` project directory and module reads, `crates/sifr_lint/src/engine.rs` lint file reads, `crates/sifr_format/src/lib.rs` formatter directory/file reads, and `crates/sifr_package/src/manifest/sifr.rs` manifest reads. | M1 must inventory every production direct read with file/line references. M2 cannot close until workspace-backed compiler reads go through a typed source provider with disk, overlay, and tracked-read implementations. |
| D0-4 | LSP must stop owning per-document compiler hosts before snapshot scheduling can be meaningful. | `DocumentState::new`, `change_full`, `change_incremental`, and `save` call `rebuild`; `rebuild` creates `FrontendMode::SingleFile` and calls `AnalysisHost::open_single_file`. | Persistent workspace `AnalysisSession` is required before M4 scheduler/cancellation work. Until then, LSP performance evidence mostly measures fast single-file rebuilds, not a long-lived project service. |
| D0-5 | The current snapshot type is only a revision token. | `AnalysisSnapshot` stores only `AnalysisRevision`; it does not own immutable file, graph, source-map, cache, symbol-index, or overlay state. | M1 defines real snapshot contents. M3 introduces the workspace session data model and M4 migrates analysis onto immutable snapshot handles before any copy-on-write or stale-result guarantees are claimed. |
| D0-6 | Dirty-scope and module-signature design must precede structural replacement. | `FrontendContext::update_module_source` clears lower, diagnostics, and analysis for all modules on any text change and bumps graph revision. Repository search finds no existing `DirtyScope`, `ModuleSignature`, `ExportSignature`, `can_replace_module_in_project`, `CowProjectState`, `WorkspaceSession`, or `WorkspaceSnapshot` implementation outside this planning issue. | M1 defines `DirtyScope` and `ModuleSignature`; M3 implements them before `can_replace_module_in_project` is allowed to reuse state. |
| D0-7 | A real scheduler needs cancellation tokens and snapshot identity first. | `Scheduler` only maps methods to labels, `RequestQueue` only tracks pending IDs, request handling is synchronous, and `$/cancelRequest` only removes an ID from the pending set. | M1-M3 implementation must stay serialized unless cancellation/snapshot tokens are introduced earlier. M4 builds cancellation around captured snapshots and request tokens before background diagnostics, references, index warming, progress, or parallel request work are enabled. |
| D0-8 | Raw editor/watch events must be compacted before asynchronous scheduling or fine-grained watcher invalidation. | `workspace/didChangeWatchedFiles` republishes diagnostics directly; `didChange` applies each edit immediately and republishes diagnostics. | Event compaction is not a standalone phase-start correctness blocker while the server stays synchronous. M6 introduces editor-event compaction and dirty-scope classification for open/change/save/close; M15 extends watcher-storm degradation and seen-file-derived watch globs. |
| D0-9 | Protocol-level performance gates are not yet split enough to prove editor latency. | `verification/performance/manifest.json` has Phase 35 `perf.interactive.*` cases and one aggregate `perf.lsp.request_families` case. `lsp_query_bench.py` only implements the aggregate scenario. | M10 must split the harness and manifest into per-request LSP cases. Aggregate request-family timing remains smoke coverage only. |
| D0-10 | Symbol/index ranges need the M0 source-map foundation before editor features can claim range correctness. | `SymbolIndex::document_symbols` currently returns `range: None`, and workspace symbols use one whole-project index keyed by graph/source revision. | M1/M4 LSP correctness work must not claim precise symbol/navigation ranges until M0 conversion is complete. M5 bucketed indexes then require project/package/stdlib buckets before auto-import or workspace completion scaling is claimed. |
| D0-11 | Current docs overstate some desired layers as implemented reality. | `internal_docs/lsp_server.md` describes `RequestQueue`, `Scheduler`, `SnapshotLayer`, line indexes, cancellation tokens, and UTF conversion as required layers, but the current code only implements a small subset. | M0/M1 must update issue wording and implementation trackers to distinguish completed foundations from planned architecture. Future closeout gates must verify behavior, not only names. |
| D0-12 | Package source-map ambiguity needs an ownership boundary before source-map redesign starts. | `PackageSourceMap::build` rejects duplicate package module paths as package manifest diagnostics, while workspace import resolution already emits `SIFR-IMPORT-0005` with import-site span and candidate paths. | M2 must model import-site ambiguity as queryable package source-map state when the package map is otherwise valid. M17 must add runtime package fixtures proving canonical `SIFR-IMPORT-0005` behavior with package context. |

## Locked Architecture Decisions

These are phase decisions, not open design questions and not claims of current implementation. Implementation PRs may refine names only if they preserve the ownership and correctness contracts below. Each milestone must provide implementation evidence for the relevant locked decision before it can close.

### Source Text, Line Maps, And Position Conversion

Decision: create a new low-level `sifr_source` crate as the canonical owner of source text, line maps, position encodings, source spans, and source-file metadata.

Rationale: `sifr_syntax::SourceText` cannot be the shared owner because `sifr_syntax` already depends on `sifr_diagnostics`; making diagnostics depend on syntax would create a cycle. `sifr_frontend::SourceText` is too high in the stack for diagnostics and syntax. `sifr_diagnostics::SourceMap` is renderer-oriented and currently uses separate 1-based display positions. A small dependency-free source crate is the clean boundary.

Dependency rule: `sifr_source` sits below diagnostics, syntax, frontend, analysis, LSP, lint, format, driver, package, and CLI crates. It may depend only on the standard library and source-position primitives such as `ruff_text_size`. It must not depend on `sifr_diagnostics`, `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, `sifr_lsp`, or any higher compiler/tooling crate.

Required shape:

- `sifr_source::SourceText` owns immutable text plus a `LineMap`.
- `LineMap` stores byte line starts as immutable `Arc<[TextSize]>` data and exposes total line count, line byte ranges, EOF position, and CRLF-safe line slicing.
- `PositionEncoding` lives in `sifr_source` with `Utf8`, `Utf16`, and `Utf32`.
- `TextPosition` and `TextRangeUtf` move to or are re-exported from `sifr_source`; syntax/frontend/LSP must not define separate incompatible position structs.
- Conversion APIs are byte-range authoritative: `byte_offset(position, encoding)`, `position_at(offset, encoding)`, and `range_at(TextRange, encoding)`.
- UTF-16 conversion rejects positions inside a surrogate-pair scalar. UTF-8 and UTF-32 conversion reject positions that do not land on a Rust string character boundary.
- `SourceFile` metadata includes canonical path, optional URI, source hash, optional document version, and `SourceText`.
- `sifr_frontend::SourceMapView` becomes a read-only view over these source files and no longer returns `None` for valid conversions.
- `sifr_diagnostics::SourceMap` uses `sifr_source::SourceText`/`LineMap` internally for rendering, but keeps the existing rendered diagnostic JSON schema.
- `sifr_lsp` conversion must use frontend snapshot source maps, not raw document strings, for diagnostics, locations, text edits, semantic tokens, formatting ranges, and incremental edits once a workspace snapshot is available.

### Source Provider And VFS Boundary

Decision: `sifr_frontend` owns the compiler-service source-provider boundary. The boundary is semantic, not just an IO helper.

Required shape:

- `SourceProvider` supports reading files, reading directories, checking existence, canonicalizing paths, and reporting failed lookups that affected module resolution.
- Implementations are `DiskSourceProvider`, `OverlaySourceProvider`, and `TrackingSourceProvider`.
- Open editor buffers are overlays with content hash, document version, source text, line map, URI, and a flag for whether the overlay matches disk text.
- Frontend project loading, module import discovery, driver project discovery, package source reads, package manifest/config reads that affect compilation, lint source reads, format source reads, and LSP workspace reads go through the provider.
- Package manifest reads are part of package identity and cache keys; `sifr_package` manifest reads must not remain hidden direct IO once package-aware snapshots are introduced.
- CLI stdin and generated output reads may stay outside the provider when they are not part of workspace identity, but that exception must be documented at each call site in the direct-read inventory.
- Formatter and linter standalone modes may create short-lived providers; they must not introduce another source-map or line-map authority.

### Package Source Maps And Diagnostic Identity

Decision: diagnostic identity follows the user-facing source problem, not the subsystem that first observes related state.

Required rules:

- Emit `SIFR-PACKAGE-*` only when the package manifest or package source map is structurally invalid independent of any source import site.
- Emit `SIFR-IMPORT-0005` when a concrete source import target can resolve to multiple legal package source candidates inside an otherwise usable package/source context.
- `PackageSourceMap` must be able to retain ambiguous candidate sets without failing construction when the ambiguity belongs to import resolution.
- `PackageSourceMap::resolve_import` must return a rich result shape: resolved, ambiguous, unresolved, private access, or fatal package-map failure. Names may change, but those states must stay distinct.
- The ambiguous state carries candidate paths, resolution scope, package id, cargo package id, import root or source root context, and enough source-map context for JSON diagnostics.
- The driver/frontend converts package import ambiguity to `SIFR-IMPORT-0005` with the primary span on the written import target. Package diagnostics must not duplicate the same ambiguity.
- Fatal package-map failures short-circuit source import diagnostics for that package until the map is valid enough to answer import-resolution queries.

### Workspace Session And Snapshot Ownership

Decision: `WorkspaceSession` lives in `sifr_frontend` and is the only owner of mutable compiler-service workspace state. `WorkspaceSnapshot` is immutable and shared by `sifr_analysis`, LSP, lint, format, diagnostics, package tooling, and future API handles.

Minimum `WorkspaceSnapshot` state:

- snapshot id and monotonic source/config/package revisions
- workspace root and entrypoint identity
- overlay table and disk-file fingerprints used for the snapshot
- source map with `sifr_source` source files
- module graph with import edges and failed lookup dependencies
- compiler options and parser/lower fingerprints
- package graph/config identity fingerprints
- dirty-scope report for the transition that produced the snapshot
- references to parse/HIR/source-map/diagnostic/index caches owned by the session
- optional lazy facts keyed by snapshot id, including reverse dependencies, exports, stdlib scope, and symbol buckets

`sifr_analysis::AnalysisSnapshot` becomes an analysis-facing handle to a `WorkspaceSnapshot`, not a revision-only token. Results carry snapshot id plus document version where applicable.

### Dirty Scope, Signatures, And Invalidation

Decision: Sifr uses conservative dirty-scope classification before choosing reuse.

Required shape:

```rust
enum DirtyScope {
    None,
    OneModule { module: ModuleId, text_changed: bool },
    ReverseDependencies { root: ModuleId, reason: DirtyReason },
    GraphStructure { reason: DirtyReason },
    ConfigProject { reason: DirtyReason },
    Workspace { reason: DirtyReason },
}
```

`ModuleSignature` is split into `ImportSignature` and `ExportSignature`. `ImportSignature` covers normalized import specifiers, resolved module ids when known, failed lookup dependencies, parser mode, and config/package-visible import context. `ExportSignature` covers exported names, public function/class/type signatures, public constants by type/shape, package-visible metadata, and top-level declarations that affect dependents. Private function bodies and local statements do not affect `ExportSignature`.

`DirtyReason` is a closed enum, not free text. Initial reasons are `DocumentVersionOnly`, `SourceTextChanged`, `ImportSignatureChanged`, `ExportSignatureChanged`, `ParseOptionsChanged`, `CompilerOptionsChanged`, `ConfigChanged`, `PackageManifestChanged`, `PackageGraphChanged`, `FileCreated`, `FileDeleted`, `FileMoved`, `FailedLookupChanged`, `DirectoryEntriesChanged`, `WatcherStorm`, and `Unknown`. `Unknown` always degrades to `DirtyScope::Workspace`.

When multiple dirty causes are present, scope merges choose the highest severity in this order: `Workspace`, `ConfigProject`, `GraphStructure`, `ReverseDependencies`, `OneModule`, `None`. Reasons are accumulated, but the selected scope is always the most conservative required by any reason. `WatcherStorm` and `Unknown` produce `DirtyScope::Workspace` unless a later reviewed implementation proves a narrower safe scope.

`ModuleSignature` is the atomic unchanged-interface bundle for structural reuse. `can_replace_module_in_project` is allowed only for `DirtyScope::OneModule` when parse/lower/compiler options, `ModuleSignature`, package-visible metadata, and entrypoint identity are unchanged. Uncertainty degrades to graph or workspace invalidation.

### Cache, Copy-On-Write, And Lifetime

Decision: cache reuse is snapshot-scoped and content-addressed.

Required shape:

- parse cache key: source hash, parser mode, syntax feature flags, Sifr compiler version-sensitive syntax options
- source-map cache key: source hash and line-map algorithm version
- HIR/lowering cache key: source hash, parser fingerprint, lowering options, compiler options, package/workspace context, and import signature where it affects names
- diagnostics/cache keys include query kind, source/HIR fingerprints, package context, and diagnostic policy settings
- `CompilerFingerprint` is a stable hash of compiler-version-sensitive options that affect parse, source maps, lowering, type checking, ownership checks, diagnostics, package resolution, or codegen preview output
- `CacheKeyFingerprint` values must be deterministic across processes and include the relevant `CompilerFingerprint`
- cache entries are reference-counted by active frontend context state and retained snapshot payloads, then released when no live owner can observe them
- copy-on-write maps use `Arc<BTreeMap<...>>` plus dirty overlays for source files, module graph, reverse dependencies, diagnostics, symbol buckets, config entries, package metadata, and watcher state
- identity reuse across snapshots must be testable for unchanged maps

### LSP Session, Scheduling, And Cancellation

Decision: M1-M3 remain serialized unless captured snapshot identity and cancellation tokens land earlier. Concurrency starts only after stale-result rejection is mechanical.

Required shape:

- LSP `DocumentStore` owns text/version/URI state and feeds overlays; it does not own per-document compiler hosts for workspace-backed files.
- `SifrLspSession` owns the `WorkspaceSession`, request queue, cancellation registry, diagnostics mode, settings, and progress/status state.
- LSP notifications are applied to the compiler service by calling explicit `WorkspaceSession::apply_document_events` / `update_overlay` APIs before semantic requests capture a snapshot. `WorkspaceSnapshot` never pulls mutable text out of `DocumentStore`; snapshot creation clones the session overlay table that has already received the latest open/change/save/close event batch.
- Every request captures a `WorkspaceSnapshot` before semantic work starts.
- `CancellationToken` is per request and exposes `is_cancelled()` plus `check_cancelled() -> Result<(), Cancelled>`.
- Compiler-service APIs that may parse, lower, check, index, search references, produce workspace diagnostics, warm indexes, format large files, or walk project/package graphs accept a token or documented noncancelable short-section marker.
- Cancelable phases check the token before starting the phase, between modules/files, before publishing results, and in worker-loop boundaries.
- Scheduler lanes are `LatencySensitive`, `Formatting`, `Workspace`, and `Background`.
- Result publication checks snapshot id and document version, not wall-clock order.
- Parent-process watchdog support is part of scheduler/session operational hardening.

### Event Compaction And Watchers

Decision: event compaction is mandatory before asynchronous scheduling or fine-grained watcher invalidation. It can be introduced earlier in serialized mode.

Rules:

- Multiple changes for one open document collapse to the latest text/version before invalidation.
- `didOpen + didChange + didSave` for the same file collapses to one opened/changed/saved summary.
- `delete + create` for the same path collapses to changed unless identity metadata proves replacement matters.
- Close drops the overlay only after in-flight snapshot readers no longer need it.
- Watcher storms above a configured threshold degrade to explicit workspace-cache invalidation.
- Watchers are derived from successful file reads, directory reads, config/package reads, stdlib roots, generated artifacts if any, and failed lookups that affect resolution.

### Project Residency, Config, And Build Metadata

Decision: large workspace behavior is residency-based.

Required shape:

- Project residency states are `OpenFileOwner`, `AncestorSolution`, `ReferencedByOpenProject`, `ExplicitApiOpen`, and `Evictable`.
- Config entries track parsed manifest/config, retaining projects, retaining open files, extended-by configs, pending reload state, and fingerprint.
- Watcher registrations are deduplicated and reference-counted.
- `.sifrbuildinfo` is non-authoritative and may contain module graph, source hashes, export hashes, dependency edges, config/package identity, and serialized summaries. It is used only after verifying current source/config/package/compiler fingerprints.

### Symbol And Import Indexes

Decision: indexes are bucketed by visibility and invalidated by dirty scope.

Required shape:

- buckets are `WorkspaceBucket(ProjectId)`, `PackageBucket(PackageId)`, and `StdlibBucket`
- each bucket tracks dirty files/packages, readiness, source-map-backed symbol ranges, and last snapshot id
- completion/import suggestions may use `Exact`, `StaleButUsable`, `NeedsBackgroundRefresh`, or `Unavailable` readiness, but correctness-sensitive features must not report stale data as exact

### Flow Graph

Decision: the flow graph is a compiler substrate, not an editor-only artifact.

Required shape:

- flow nodes cover entry, assignment, condition, branch, loop, call, mutation, move/borrow, join, unreachable, and exit
- `FlowGraph` stores node ids, node kinds, outgoing/incoming edges, source ranges, enclosing HIR owner, mutation/move/borrow effects, and graph revision
- HIR lowering emits flow graph ids or a companion flow graph linked by HIR owner ids
- narrowing and ownership facts migrate onto graph-backed queries incrementally; move/borrow invalidation must use flow effects instead of ad hoc statement scans after migration
- graph caches are snapshot-scoped and invalidated by HIR/control-flow changes, module replacement, or compiler-option changes that affect flow construction

### Tracing, Status, Editor Corpus, And Budgets

Decision: observability is part of correctness for a long-lived compiler service.

Required shape:

- deterministic traces record source update, parse, lower, type check, ownership check, flow analysis, cache hit/miss, invalidation, scheduler lane, cancellation, stale-result rejection, and LSP request timing
- status/debug reports open files, projects, snapshots, retained cache entries, index readiness, last update latency, and memory counters
- marker-based editor fixtures cover multi-file hover, completion, definition, references, rename, diagnostics, semantic tokens, formatting, and stale snapshots
- per-request LSP budgets are enforced separately with one manifest case, budget entry, and `lsp_query_bench.py` scenario per request family
- Phase 36 default targets apply: cold start <= 1000ms median, didOpen diagnostics <= 500ms median, didChange diagnostics <= 250ms median, completion <= 200ms median, hover <= 100ms median, signature help <= 150ms median, definition/declaration/type-definition <= 150ms median, references <= 500ms median, rename prepare <= 150ms median, rename edit generation <= 750ms median, semantic tokens <= 250ms median, inlay hints <= 250ms median, selection range <= 100ms median, type hierarchy <= 250ms median, code actions <= 250ms median, formatting <= 500ms median, generated Rust preview <= 750ms median, and workspace diagnostics <= 2000ms p95. `perf.lsp.request_families` remains smoke coverage only

## Milestones

### M0: Source And Position Foundation

Depends on: phase plan approval.

Goal: remove the source-map stub risk before any snapshot or LSP architecture work depends on positions.

Scope:

- add `sifr_source` at the bottom of the dependency hierarchy
- lock `sifr_source` public API for M0: `SourceText`, `LineMap`, `PositionEncoding`, `TextPosition`, `TextRangeUtf`, and `SourceFile`
- migrate syntax, diagnostics, frontend, and LSP conversion to shared `SourceText`, `LineMap`, `PositionEncoding`, `TextPosition`, and `TextRangeUtf`
- migrate known M0 call sites: parser-side `sifr_syntax::SourceText` consumers, `sifr_diagnostics::SourceMap` line-map storage/rendering, `crates/sifr_frontend/src/graph_cache_and_queries.rs` source-map view construction/conversion, `crates/sifr_frontend/src/bin/frontend_query_bench.rs` `interactive.source_map_lookup`, and `crates/sifr_lsp/src/conversion.rs` range/position conversion helpers
- replace `SourceMapView` stubs with UTF-8/UTF-16/UTF-32 conversions
- make `interactive.source_map_lookup` assert real round trips
- add multibyte, CRLF, EOF, invalid-boundary, and rendered-diagnostic parity tests
- add a dependency-direction guardrail proving `sifr_source` does not depend on higher compiler/tooling crates

Out of scope:

- no `SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, cache reuse, scheduler behavior, or LSP request-flow migration
- no direct filesystem read migration; M1 inventories direct reads and M2 moves semantic reads behind the provider boundary

Closeout:

- frontend, diagnostics, and LSP have one source-position authority
- source conversions return `Some` for positions on character boundaries inside registered source files and return `None` only for genuinely invalid positions, such as unregistered files, byte offsets inside multibyte scalars or CRLF line endings, surrogate-pair interiors, and out-of-range positions
- UTF-8, UTF-16, UTF-32, multibyte, CRLF, EOF, invalid-boundary, rendered-diagnostic parity, and LSP UTF-16 conversion tests pass
- `interactive.source_map_lookup` is no longer a no-op benchmark path
- dependency-direction guardrail is part of local validation
- symbol/navigation range correctness is unblocked for later milestones

### M1: Architecture Contract And Guardrails

Depends on: M0.

Goal: make the locked decisions enforceable before behavior changes begin.

Scope:

- mark M1 as the pre-flight gate that must pass before M2 starts
- record the locked terms for source provider, snapshot, dirty scope, signatures, cache fingerprints, scheduler, flow graph, build info, and query readiness
- add the direct-read inventory with file/line references and permitted exceptions
- add guardrails for `sifr_source` dependency direction, source-map stubs, current LSP single-file rebuilds, and aggregate-only LSP budgets
- update docs that currently describe target LSP layers as if they were fully implemented
- record that M1-M4 stay serialized until cancellation and snapshot publication checks exist

Closeout:

- implementation dependency order is explicit and reviewed
- guardrails fail on new semantic bypasses or untracked source-position forks
- M1 is a pre-flight gate for M2-M5; no source-provider, session, snapshot, or LSP behavior migration starts until it passes

### M2: Source Provider And Overlay Store

Depends on: M1.

Goal: route compiler-service reads through one typed source boundary.

Scope:

- implement `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, and `TrackingSourceProvider`
- model overlays with URI, path, version, text hash, line map, and disk-match state
- migrate frontend project/module reads and package/config reads that affect compilation
- let formatter and linter create short-lived providers instead of reading source through separate line-map paths
- track dependency reads, including successful file reads, directory reads, canonicalization, and failed lookups
- split package source-map fatal construction diagnostics from import-site ambiguity records

Closeout:

- workspace-backed source reads are overlay-aware and dependency-tracked
- M2 owns the overlay record model and provider behavior; M3 owns overlay lifecycle inside `WorkspaceSession`
- direct production reads are either migrated or listed as non-semantic exceptions
- package source maps preserve legal ambiguous candidates for import resolution instead of failing construction early or emitting `SIFR-PACKAGE-*` for that case
- package import-resolution unit tests cover resolved, ambiguous, unresolved, private, and fatal package-map states

### M3: Workspace Session Data Model

Depends on: M2.

Goal: introduce the mutable compiler-service owner while keeping execution serialized.

Scope:

- add `WorkspaceSession` as the mutable owner of workspace compiler state
- move overlay lifecycle ownership from ad hoc document state into `WorkspaceSession`
- add session-owned source map, module graph, compiler options, package/config identity, cache registry handles, and revision counters
- add snapshot creation API that freezes current session state without adding analysis-query migration yet
- preserve CLI behavior on top of the session data model

Closeout:

- `WorkspaceSession` owns overlays and workspace state, not LSP document state
- a frozen snapshot object can be produced and inspected in tests

### M4: Analysis Snapshot Migration

Depends on: M3.

Goal: make analysis queries consume captured immutable snapshots.

Scope:

- complete `WorkspaceSnapshot` with snapshot id, revisions, overlays, source map, module graph, options, package/config identity, dirty-scope report, and cache references
- migrate `AnalysisHost` from revision-token snapshot checks to workspace snapshot handles
- convert `sifr_analysis::AnalysisSnapshot` from revision token to snapshot handle
- route diagnostics, symbols, formatting handoff, lint handoff, generated Rust preview, and package-aware query inputs through captured snapshots
- preserve current serialized execution semantics

Closeout:

- analysis, diagnostics, lint, format, and package queries accept a captured snapshot
- stale-result identity is mechanically available before async scheduling is enabled

### M5: LSP Persistent Session Integration

Depends on: M4.

Goal: stop LSP from rebuilding detached single-file hosts for workspace-backed files.

Scope:

- make `SifrLspSession` own `WorkspaceSession`
- make `DocumentStore` feed open/change/save/close events into overlays instead of owning compiler hosts
- capture snapshots for diagnostics and semantic requests
- reject stale diagnostics/results by snapshot id and document version
- keep request handling serialized in this milestone

Closeout:

- `DocumentStore` no longer calls `AnalysisHost::open_single_file` on every workspace-backed edit
- unsaved editor buffers are analyzed from overlays, not disk text

### M6: Event Compaction And Dirty Scope

Depends on: M5.

Goal: classify updates precisely enough to avoid accidental broad invalidation.

Scope:

- implement compacted document and watcher event summaries
- implement `DirtyReason`, dirty-scope merge priority, and conservative degradation
- report no-op, one-module, reverse-dependency, graph, config/project, and workspace dirty scopes
- handle watcher storms as `DirtyScope::Workspace { reason: WatcherStorm }`

Closeout:

- repeated edits do not enqueue redundant work
- every invalidation report explains the selected dirty scope and reason set

### M7: Module Signatures And Dependency Invalidation

Depends on: M6.

Goal: make invalidation dependency-sensitive before structural reuse is allowed.

Scope:

- compute `ImportSignature`, `ExportSignature`, and `ModuleSignature`
- maintain reverse dependencies for imports, config/package reads, and failed lookups
- keep private body edits local when signatures are unchanged
- invalidate reverse dependents for public API/import graph changes

Closeout:

- unrelated modules retain diagnostics/analysis across private edits
- public signature changes invalidate affected dependents deterministically

### M8: First-Class Flow Graph

Depends on: M7.

Goal: move narrowing and ownership facts onto an explicit compiler graph while invalidation work is still fresh.

Scope:

- emit flow nodes and edges during HIR lowering or as a companion graph
- represent assignments, conditions, branches, loops, calls, mutations, moves/borrows, joins, unreachable, and exits
- migrate Option/None narrowing and mutation invalidation onto graph-backed queries
- migrate ownership fact invalidation onto flow effects

Closeout:

- existing narrowing and ownership tests remain green
- graph-backed facts are snapshot-scoped and visible in debug/trace output

### M9: Fingerprints And Cache Keys

Depends on: M8.

Goal: make cache identity explicit before cache reuse exists.

Scope:

- add `CompilerFingerprint` and deterministic `CacheKeyFingerprint`
- add key types for parse, source-map, HIR/lowering, diagnostics, lint, format, package graph, symbol buckets, and flow graph caches
- ensure each key includes source hash, relevant options, compiler fingerprint, package/workspace context, and query policy settings where they affect results
- add negative tests for intentionally omitted cache-key inputs

Closeout:

- cache identity is deterministic and complete before reuse is introduced
- no cache key omits a correctness-relevant input documented in the locked decisions

### M10: Snapshot Reuse And Structural Replacement

Depends on: M9.

Goal: make snapshot reuse cheap and correct after cache identity is locked.

Scope:

- add ref-counted parse, source-map, HIR, diagnostics, and index caches using M9 keys
- add copy-on-write maps for graph, source maps, diagnostics, indexes, config, and package metadata
- implement `can_replace_module_in_project` only for safe one-module replacements
- add identity-reuse tests for unchanged maps and cache entries

Closeout:

- structurally stable edits reuse unchanged project state by identity
- stale cache entries cannot be observed through diagnostics, HIR, or LSP queries

### M11: LSP Scheduler Queues

Depends on: M5 and M10.

Goal: make request prioritization real while keeping cancellation/progress separate.

Scope:

- implement priority queues for latency-sensitive, formatting, workspace, and background lanes
- debounce diagnostics and background index work behind captured snapshots
- enforce no starvation between lanes
- keep result publication guarded by snapshot id and document version

Closeout:

- workspace/background work cannot starve completion, hover, diagnostics, or formatting
- queued work publishes only when its captured snapshot is still valid

### M12: Per-Request Editor Latency Budgets

Depends on: M11.

Goal: replace aggregate-only LSP performance evidence early enough to guide later work.

Scope:

- split `lsp_query_bench.py` into per-request scenarios
- add manifest and budget entries for cold start, diagnostics, completion, hover, signature help, navigation, references, rename, semantic tokens, inlay hints, selection range, type hierarchy, code actions, formatting, and generated Rust preview
- retain `perf.lsp.request_families` as smoke coverage only
- record warm/cold index assumptions and median/p95 thresholds

Closeout:

- protocol-level editor latency is enforced per request family
- performance docs explain the Phase 35 frontend budget to LSP budget relationship

### M13: LSP Cancellation, Progress, And Watchdog

Depends on: M11.

Goal: make running and long-lived LSP work cancellable and operationally bounded.

Scope:

- add request cancellation tokens and phase-boundary cancellation checks
- propagate cancellation through cancelable compiler-service APIs
- add delayed progress for workspace load, full diagnostics, references, and index warming
- add `sifr lsp --parent-pid`
- preserve deterministic cancellation and publication behavior

Closeout:

- canceled or superseded work cannot publish stale results
- long work is explainable through delayed progress without noisy fast-path updates

### M14: Bucketed Indexes And Safe Parallel Lanes

Depends on: M10, M11, and M12.

Goal: scale editor queries without weakening deterministic compiler identity.

Scope:

- add workspace, package, and stdlib symbol/import buckets with readiness state
- refresh only dirty buckets for completion/import suggestions
- introduce worker lanes only for approved phases such as parse, source-map creation, independent HIR lower, lint file rules, formatter checks, and selected diagnostics
- keep type identity creation, ownership mutation, package graph mutation, and codegen state single-owner until proven safe

Closeout:

- diagnostic ordering is stable across repeated runs
- bucket refreshes do not rebuild the whole workspace index for local edits

### M15: Project Residency, Watchers, And Build Info

Depends on: M3, M5, M6, and M10.

Goal: keep long-lived sessions bounded in large workspaces.

Scope:

- implement project residency states and lazy default-project loading
- add config registry reverse retention and pending reloads
- add deduplicated reference-counted watcher registrations
- derive watched globs from seen files, directories, configs, packages, stdlib roots, generated artifacts, and failed lookups
- implement verified non-authoritative `.sifrbuildinfo`

Closeout:

- closing files releases unneeded projects, watchers, configs, and cache refs
- build metadata never hides source/config/package/compiler-option changes

### M16: Trace And Status Surfaces

Depends on: M5, M10, M13, M14, and M15.

Goal: make long-lived compiler-service behavior consistently explainable across all previously added surfaces.

Scope:

- normalize the trace/status hooks added incrementally by earlier milestones
- add deterministic trace phases for source update, parse, lower, type check, ownership, flow, cache, invalidation, scheduler, cancellation, stale rejection, and LSP timing
- add status/debug output for open files, projects, snapshots, cache entries, index readiness, update latency, and memory counters
- add representative CLI and LSP trace snapshots

Closeout:

- a stale rejection and a dependency-sensitive invalidation can be explained from trace output
- status output is useful for editor bug reports without exposing private internals

### M17: Editor Corpus And Snapshot Handles

Depends on: M5, M12, M14, and M16.

Goal: lock the editor regression surface and prepare future compiler API handles without exposing them publicly.

Scope:

- add marker-based multi-file editor fixtures for hover, completion, definition, references, rename, diagnostics, semantic tokens, formatting, code actions, and stale snapshots
- add package ambiguous-import fixtures proving runtime `SIFR-IMPORT-0005` diagnostic output with package/source-map context
- add internal snapshot-scoped handles for symbols, types, signatures, diagnostics, and source spans
- prove handles cannot resolve against the wrong snapshot
- document remaining work before any public compiler API

Closeout:

- editor query fixtures cover multi-file and stale-snapshot behavior
- runtime package fixtures prove that one ambiguous import emits `SIFR-IMPORT-0005` and no companion `SIFR-PACKAGE-*`
- runtime package fixtures prove fatal package-map construction errors emit `SIFR-PACKAGE-*` only, without a companion `SIFR-IMPORT-0005`
- API-handle preparation remains internal-only unless a later phase approves public exposure

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | `sifr_frontend` exposes a workspace/session snapshot API used by CLI analysis and LSP request paths. |
| AC-2 | Open editor buffers overlay disk files through a typed file-system boundary. |
| AC-3 | Frontend/project module discovery records successful reads, directory reads, config/package reads, and failed lookup dependencies. |
| AC-4 | Parse and HIR cache keys include content hash, parser/lower options, compiler version-sensitive settings, and package/workspace context. |
| AC-5 | Private body edits reuse unaffected module results and do not invalidate the whole project. |
| AC-6 | Public export-signature changes invalidate reverse dependents deterministically. |
| AC-7 | LSP diagnostics and request results carry snapshot/document version identity and reject stale outputs. |
| AC-8 | Parallel analysis, where enabled, preserves deterministic diagnostic order. |
| AC-9 | First-class flow graph nodes are available for narrowing and ownership facts. |
| AC-10 | Existing narrowing, ownership, diagnostics, formatter, linter, package, and LSP tests remain green. |
| AC-11 | Source-position conversion supports editor-safe UTF-8/UTF-16 round trips where required. |
| AC-12 | Compiler trace output explains phase timing, cache reuse, invalidation causes, snapshot ids, and stale-result rejection. |
| AC-13 | Marker-based editor fixtures cover multi-file requests and stale-snapshot races. |
| AC-14 | Internal docs are updated for architecture, frontend queries, cache invalidation, LSP, performance budgets, and tooling verification. |
| AC-15 | No TypeScript/JavaScript compatibility behavior becomes Sifr semantic authority. |
| AC-16 | Dirty-scope reports distinguish no-op, one-module, reverse-dependency, graph-structure, config/project, and workspace invalidation. |
| AC-17 | `can_replace_module_in_project` preserves project state for structurally stable one-module edits and rejects unsafe replacements. |
| AC-18 | Copy-on-write snapshot finalization reuses unchanged project maps, source maps, diagnostics, indexes, and config/package metadata by identity. |
| AC-19 | Raw editor and watcher events are compacted before invalidation, and event storms degrade to explicit workspace-cache invalidation. |
| AC-20 | The LSP scheduler has real priority, cancellation, debounce, delayed progress, and stale-result behavior. |
| AC-21 | Project residency and config/watch registries release unneeded projects, configs, watchers, and cache references. |
| AC-22 | Symbol/import indexes are bucketed by workspace project, package, and stdlib scope with independent dirty/readiness state. |
| AC-23 | Persistent build metadata accelerates CLI/build cold start only after source/config/package/compiler fingerprints are verified. |
| AC-24 | `sifr lsp --parent-pid` exits the server when the parent editor process dies. |
| AC-25 | A status/debug surface reports project, snapshot, cache, index, update-latency, and memory state for editor bug reports. |
| AC-26 | Per-feature LSP/editor latency budgets are enforced separately for cold start, diagnostics, completion, hover, signature help, navigation, references, rename, semantic tokens, inlay hints, selection range, type hierarchy, code actions, formatting, and generated Rust preview. |
| AC-27 | `perf.lsp.request_families` remains only an aggregate smoke benchmark and is not the sole enforced LSP latency gate. |
| AC-28 | Performance docs explain the relationship between Phase 35 `perf.interactive.*` frontend budgets and protocol-level LSP request budgets. |
| AC-29 | Package ambiguous source imports use `SIFR-IMPORT-0005` with import-site span, candidate paths, resolution scope, and package/source-map JSON context when the package map is otherwise valid; fatal package-map errors remain `SIFR-PACKAGE-*` and do not duplicate source import diagnostics. |
| AC-30 | Package source-map construction preserves ambiguous candidate sets for otherwise valid packages instead of rejecting or dropping them. |
| AC-31 | Package import-resolution tests cover resolved, ambiguous, unresolved, private-access, and fatal package-map outcomes, including a negative check that one import ambiguity does not emit both `SIFR-IMPORT-0005` and `SIFR-PACKAGE-*`. |

## Validation

Minimum validation for planning and milestone PRs:

```bash
git diff --check
python3 scripts/check_file_size_guardrails.py
cargo fmt --check
cargo test -p sifr_frontend
cargo test -p sifr_analysis
cargo test -p sifr_lsp
cargo test -p sifr -- --skip test_e2e_pass
scripts/run_all_tests.sh --profile quick
```

Full phase closure validation:

```bash
scripts/run_all_tests.sh
```

If full validation is blocked by an unrelated inherited failure, the execution tracker must record the exact command, failure, and why this phase did not cause it.
