# Phase 31 Execution Checklist (sifr_driver Decomposition and Boundary Hardening)

Status: in_progress (started 2026-03-10)
Owner: phase31_sifr_driver_decomposition execution loop
Reference planning doc:
- `issues/ad-hoc-sifr-driver-decomposition-and-boundary-hardening.md`

Loop per part: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Update docs -> Next part

## Global Gates
- [x] Scope remains constrained to the active milestone part
- [x] Root cause addressed without fallback or compatibility shims
- [x] CLI/API-visible behavior remains unchanged
- [x] Positive-path and negative-path validation recorded for the active milestone
- [x] Milestone demo runs successfully before the PR is opened
- [x] Local validation is run before the PR is opened
- [ ] PR is opened, reviewed, and merged before the next milestone starts
- [ ] Docs/checklists/PR links are updated before moving on

## Full Phase To-Do Plan
1. [x] `milestone_driver_1`: extract diagnostics and public result types into a dedicated API spine while shrinking `lib.rs` toward crate wiring
2. [ ] `milestone_driver_2`: extract stdlib embedding, intrinsic mapping, cache, and bootstrap into a dedicated module tree with unchanged export behavior
3. [ ] `milestone_driver_3`: extract frontend compile/check plumbing, module-export collection, dependency ordering, and cycle canonicalization into coherent frontend/project modules
4. [ ] `milestone_driver_4`: extract discovery, import-closure parsing, workspace allocation, project build orchestration, and file-write helpers into dedicated modules
5. [ ] `milestone_driver_5`: extract test-runner orchestration, composed test-lib generation, and test-runner Cargo manifest logic into a dedicated module
6. [ ] `milestone_driver_6`: split the embedded driver tests into focused modules and add the checked maintainability guardrail script, docs, and local-validation wiring

## Baseline Revalidation
- 2026-03-10: full local validation passed with `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- 2026-03-10: full local validation result -> `verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`
- 2026-03-10: driver smoke checks passed:
  - `cargo test -q -p sifr_driver -- --test-threads=1` -> passed (`59 passed, 0 failed`)
  - `cargo test -q -p sifr_driver test_check_project_error_messages_match_build_project` -> passed (`1 passed, 0 failed`)
  - `cargo test -q -p sifr_driver test_run_tests_resolves_local_imports_and_constants` -> passed (`1 passed, 0 failed`)
  - `cargo test -q -p sifr_driver test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order` -> passed (`1 passed, 0 failed`)
  - `cargo test -q -p sifr test_frontend_error_messages_match_across_check_build_and_run_paths` -> passed (`1 passed, 0 failed`)
  - `cargo test -q -p sifr test_runner_mode_resolution` -> passed (`1 passed, 0 failed`)

## Milestone Progress

### milestone_driver_1: Public API Spine and Diagnostic Extraction
- Status: complete
- Implementation PR: https://github.com/yaseralnajjar/sifr/pull/1089
- Implementation target:
  - reduce `crates/sifr_driver/src/lib.rs` to module wiring plus public re-exports for the stable public API
  - extract diagnostics, compile/public result types, panic-boundary helpers, and crate-root API surface into dedicated modules
  - preserve `compile`, `compile_with_metadata`, `check`, `parse_source`, `lower_source`, `type_check_source`, `build`, `build_project`, `check_project`, `run_tests`, `compile_errors_to_diagnostics`, and `apply_diagnostic_recovery_limits` from the crate root
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_driver_1_api_spine_demo.sifr`
- Validation target:
  - `cargo test -p sifr_driver diagnostics -- --nocapture`
  - `cargo test -p sifr_driver single_file_frontend -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -q -p sifr_driver diagnostics -- --nocapture` -> passed (`4 passed, 0 failed`)
  - positive path: `cargo test -q -p sifr_driver test_compile_hello_world -- --nocapture` -> passed (`1 passed, 0 failed`)
  - positive path: `cargo run -q -p sifr -- run demos/m_driver_1_api_spine_demo.sifr` -> printed `driver milestone 1 api spine demo: 42`
  - negative path: `cargo test -q -p sifr_driver test_lower_source_and_type_check_source_surface_type_errors -- --nocapture` -> passed (`1 passed, 0 failed`), proving the extracted crate spine still surfaces type-check failures through the stable single-file API
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1089 merged into `main` on 2026-03-10

### milestone_driver_2: Stdlib Bootstrap Extraction
- Status: in_progress
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_driver_2_stdlib_bootstrap_demo.sifr`
- Validation target:
  - `cargo test -p sifr_driver stdlib -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -q -p sifr_driver stdlib -- --nocapture` -> passed (`6 passed, 0 failed`)
  - positive path: `cargo run -q -p sifr -- run demos/m_driver_2_stdlib_bootstrap_demo.sifr` -> printed `3.141592653589793`
  - negative path: `cargo test -q -p sifr_driver test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild -- --nocapture` -> passed (`1 passed, 0 failed`), proving cached stdlib failures are reused instead of silently rebuilding or falling back
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### milestone_driver_3: Frontend and Project-Graph Extraction
- Status: pending
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_driver_3_frontend_project_graph_demo/main.sifr`
- Validation target:
  - `cargo test -p sifr_driver frontend -- --nocapture`
  - `cargo test -p sifr_driver project_graph -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - pending

### milestone_driver_4: Discovery, Workspace, and Build Orchestration Extraction
- Status: pending
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_driver_4_build_orchestration_demo/main.sifr`
- Validation target:
  - `cargo test -p sifr_driver discovery -- --nocapture`
  - `cargo test -p sifr_driver project_build -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - pending

### milestone_driver_5: Test Runner Extraction
- Status: pending
- Demo target:
  - `cargo run -q -p sifr -- test demos/m_driver_5_test_runner_demo`
- Validation target:
  - `cargo test -p sifr_driver test_runner -- --nocapture`
  - `cargo test -p sifr test_runner_mode_resolution -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - pending

### milestone_driver_6: Test Suite Decomposition and Maintainability Guardrail
- Status: pending
- Demo target:
  - `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- Validation target:
  - `cargo test -p sifr_driver -- --test-threads=1`
  - `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
  - `SIFR_DRIVER_GUARDRAIL_EXPECT_FAILURE=1 python3 scripts/check_sifr_driver_maintainability_guardrails.py`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - pending

## External Review Passes

### review_pass_1
- Reviewer artifact:
  - pending
- Status: pending
- Validation evidence:
  - pending
- Follow-up PR:
  - pending

### review_pass_2
- Reviewer artifact:
  - pending
- Status: pending
- Validation evidence:
  - pending
- Follow-up PR:
  - pending
