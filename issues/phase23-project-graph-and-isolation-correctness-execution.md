# Phase 23 Execution Checklist (Project Graph and Isolation Correctness)

Status: in_progress
Owner: phase_23 execution loop
Reference phase doc: `.cursor/plans/main/phases/23_project_graph_and_isolation_correctness.md`

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
- [ ] Open PR, review, and merge

### Part 3: milestone_23_3 Project/Test Discovery Parity Contract
- [ ] Introduce one shared discovery contract for project and test paths
- [ ] Align module membership decisions for equivalent imports across `build`/`run`/`check` and `test`
- [ ] Add parity-focused regression tests for project/test discovery behavior
- [ ] Add milestone 23.3 positive demo
- [ ] Add milestone 23.3 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_23_4 Invocation-Scoped Temp Workspace Isolation
- [ ] Replace fixed temp workspaces with per-invocation isolated directories
- [ ] Ensure parallel `run`/`test` invocations cannot overwrite each other's artifacts
- [ ] Add isolation/parallel-safety regression tests
- [ ] Add milestone 23.4 positive demo
- [ ] Add milestone 23.4 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_23_5 Graph and Isolation Regression Matrix
- [ ] Add regression matrix for import closure, unrelated siblings, deterministic ordering, cycle errors, and parallel workspace isolation
- [ ] Include both single-file and multi-file corpus fixtures
- [ ] Wire matrix into local validation so regressions fail before merge
- [ ] Add milestone 23.5 positive demo
- [ ] Add milestone 23.5 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

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
status: validating (pending PR)

- [x] Deterministic module order hardening implemented
- [x] Canonical stable cycle diagnostics implemented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/main.sifr` -> prints `m23_2 deterministic module graph and cycle diagnostics demo:` and `42`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo test -q -p sifr_driver test_compute_module_compile_order_cycle_diagnostics_are_canonical_and_stable` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/negative_cases/module_cycle/main.sifr` -> exits `1` with canonical cycle diagnostic `a -> b -> c -> a`.

## Part 3: milestone_23_3 Project/Test Discovery Parity Contract
status: pending

## Part 4: milestone_23_4 Invocation-Scoped Temp Workspace Isolation
status: pending

## Part 5: milestone_23_5 Graph and Isolation Regression Matrix
status: pending

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/863

## Reviewer Follow-up
- pending
