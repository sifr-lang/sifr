# Phase 23 Execution Checklist (Project Graph and Isolation Correctness)

Status: completed (2026-03-06)
Owner: phase_23 execution loop
Reference phase doc: `internal_docs/phases/23_project_graph_and_isolation_correctness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 23 To-Do Plan

### Part 1: milestone_23_1 Import-Closure Discovery
- [x] Replace directory-wide sibling `.sifr` discovery with import-closure graph discovery for project build/check/run
- [x] Ensure test discovery/lowering only parses modules reachable from discovered test roots
- [x] Add regression tests proving unrelated sibling files do not affect outcomes
- [x] Add milestone 23.1 positive demo
- [x] Add milestone 23.1 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 2: milestone_23_2 Deterministic Module Graph and Cycle Diagnostics
- [x] Enforce deterministic module graph resolution independent of map iteration order
- [x] Ensure cycle diagnostics are explicit, stable, and reproducible across runs
- [x] Add deterministic-order and cycle-diagnostic regression tests
- [x] Add milestone 23.2 positive demo
- [x] Add milestone 23.2 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 3: milestone_23_3 Project/Test Discovery Parity Contract
- [x] Introduce one shared discovery contract for project and test paths
- [x] Align module membership decisions for equivalent imports across `build`/`run`/`check` and `test`
- [x] Add parity-focused regression tests for project/test discovery behavior
- [x] Add milestone 23.3 positive demo
- [x] Add milestone 23.3 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 4: milestone_23_4 Invocation-Scoped Temp Workspace Isolation
- [x] Replace fixed temp workspaces with per-invocation isolated directories
- [x] Ensure parallel `run`/`test` invocations cannot overwrite each other's artifacts
- [x] Add isolation/parallel-safety regression tests
- [x] Add milestone 23.4 positive demo
- [x] Add milestone 23.4 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 5: milestone_23_5 Graph and Isolation Regression Matrix
- [x] Add regression matrix for import closure, unrelated siblings, deterministic ordering, cycle errors, and parallel workspace isolation
- [x] Include both single-file and multi-file corpus fixtures
- [x] Wire matrix into local validation so regressions fail before merge
- [x] Add milestone 23.5 positive demo
- [x] Add milestone 23.5 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

## Part 1: milestone_23_1 Import-Closure Discovery
status: done (2026-03-06, PR #863)

- [x] Import-closure project discovery implemented
- [x] Import-closure test discovery implemented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_check_project_ignores_unrelated_non_closure_parse_errors` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_run_tests_ignores_unrelated_non_closure_parse_errors` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_1_import_closure_discovery_demo/main.sifr` -> prints `m23_1 import-closure discovery demo:` and `42` even with an invalid unrelated sibling module in the same directory.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo test -q -p sifr_driver test_check_project_reports_reachable_parse_errors_in_import_closure` -> pass (reachable parse error is still reported).
- Negative path: `cargo run -q -p sifr -- run demos/m23_1_import_closure_discovery_demo/negative_cases/reachable_dependency_parse_error/main.sifr` -> exits `1` with parse error for `[helper]`.

## Part 2: milestone_23_2 Deterministic Module Graph and Cycle Diagnostics
status: done (2026-03-06, PR #865)

- [x] Deterministic module order hardening implemented
- [x] Canonical stable cycle diagnostics implemented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/main.sifr` -> prints `m23_2 deterministic module graph and cycle diagnostics demo:` and `42`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo test -q -p sifr_driver test_compute_module_compile_order_cycle_diagnostics_are_canonical_and_stable` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/negative_cases/module_cycle/main.sifr` -> exits `1` with canonical cycle diagnostic `a -> b -> c -> a`.

## Part 3: milestone_23_3 Project/Test Discovery Parity Contract
status: done (2026-03-06, PR #867)

- [x] Shared project/test discovery contract factored and reused
- [x] Graph-membership parity checks added for equivalent import closures
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_discover_test_root_modules_is_deterministic` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_project_and_test_discovery_share_import_closure_membership` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_3_project_test_discovery_parity_contract_demo/main.sifr` -> prints `m23_3 project/test discovery parity contract demo:` and `42`.
- Positive path: `cargo run -q -p sifr -- test demos/m23_3_project_test_discovery_parity_contract_demo` -> passes `test_value`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo test -q -p sifr_driver test_project_and_test_discovery_parity_reports_reachable_parse_errors` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m23_3_project_test_discovery_parity_contract_demo/negative_cases/reachable_parse_error/main.sifr` -> exits `1` with reachable helper parse error.
- Negative path: `cargo run -q -p sifr -- test demos/m23_3_project_test_discovery_parity_contract_demo/negative_cases/reachable_parse_error` -> exits `1` with reachable helper parse error.

## Part 4: milestone_23_4 Invocation-Scoped Temp Workspace Isolation
status: done (2026-03-06, PR #869)

- [x] Per-invocation isolated workspaces implemented for `run` and `test`
- [x] Parallel isolation regressions added and passing
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr test_invocation_workspace_create_returns_unique_paths` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_create_invocation_workspace_returns_unique_paths` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_run_tests_parallel_invocations_are_isolated` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/main.sifr` -> prints `m23_4 invocation-scoped temp workspace isolation demo:` and `44`.
- Positive path: parallel invocations from demo fixtures:
  - concurrent `sifr run` on `parallel_runs/a/main.sifr` and `parallel_runs/b/main.sifr` each complete with isolated outputs (`parallel-run-a`, `parallel-run-b`).
  - concurrent `sifr test` on `parallel_tests/a` and `parallel_tests/b` each complete successfully.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/negative_cases/reachable_parse_error/main.sifr` -> exits `1` with reachable helper parse error.

## Part 5: milestone_23_5 Graph and Isolation Regression Matrix
status: done (2026-03-06, PR #871)

- [x] Phase-23 graph/isolation regression matrix script added
- [x] Single-file and multi-file fixtures added and exercised
- [x] Matrix wired into `scripts/run_all_tests.sh`
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `bash scripts/run_phase23_graph_isolation_matrix.sh` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_5_graph_isolation_regression_matrix_demo/main.sifr` -> prints `m23_5 graph and isolation regression matrix demo:` and `55`.
- Positive path: matrix `single_file_layout_smoke` row validates single-file `check/build/run` success on `demos/m23_5_graph_isolation_regression_matrix_demo/single_file/main.sifr`.
- Positive path: matrix `multi_file_import_closure_and_test` row validates multi-file `check/build/run/test` success on `demos/m23_5_graph_isolation_regression_matrix_demo`.
- Positive path: matrix `parallel_invocation_isolation` row validates concurrent `run` and concurrent `test` invocations are isolated and successful.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes phase-23 matrix gate).
- Negative path: `cargo run -q -p sifr -- run demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/module_cycle/main.sifr` -> exits `1` with canonical cycle diagnostic.
- Negative path: matrix `reachable_parse_error_contract` row validates `check/build/run/test` failures for reachable parse errors.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/863
- Part 2: https://github.com/sifr-lang/sifr/pull/865
- Part 3: https://github.com/sifr-lang/sifr/pull/867
- Part 4: https://github.com/sifr-lang/sifr/pull/869
- Part 5: https://github.com/sifr-lang/sifr/pull/871

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase23-review.md` (2026-03-06, approved)
- Pass-1 triage outcome:
  - Reviewer approved phase 23 implementation as complete across milestones 23.1-23.5.
  - No blocking defects were identified; no compiler-code remediation was required.
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/873
- External review pass 2 output: `reviews/phase23-production-grade-review.md` (2026-03-06, approved for production)
- Pass-2 triage outcome:
  - Reviewer confirmed production-grade readiness with no blocking defects.
  - Applied actionable hardening: documented multi-level relative import exclusion in closure discovery and added retry-based workspace allocation for collision-proof temp isolation.
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/874
