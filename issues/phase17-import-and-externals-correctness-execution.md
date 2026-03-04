# Phase 17 Execution Checklist (Import and Externals Correctness)

Status: completed (2026-03-04)
Owner: phase_17 execution loop
Reference phase doc: `.cursor/plans/main/phases/17_import_and_externals_correctness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `./scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_17_1 Frontend-Only Check Path
status: done (2026-03-04, PR #813)

- [x] Ensure `check` stops after frontend/type phases
- [x] Remove codegen/runtime coupling from check flow
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- check demos/m17_1_frontend_only_check_path_demo.sifr` -> `no errors found`.
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes `test_check_only_reports_frontend_phases`).
- Negative path: `cargo run -q -p sifr -- check <temp_type_error_file.sifr>` -> exits `1` with `type mismatch`.
- Milestone demo: `cargo run -q -p sifr -- run demos/m17_1_frontend_only_check_path_demo.sifr` -> prints `m17_1 frontend-only check path demo:` and `17`.

## Part 2: milestone_17_2 Non-Main Externals Resolution
status: done (2026-03-04, PR #814)

- [x] Resolve stdlib/local externals in non-main modules
- [x] Ensure multi-file projects type-check consistently
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m17_2_non_main_externals_resolution_demo/main.sifr` -> prints `m17_2 non-main externals demo:` and `3`.
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes non-main stdlib/local dependency tests).
- Positive path: `cargo test -q -p sifr_codegen test_generate_rust_multi_skips_stdlib_use_paths_in_non_main_modules` -> pass.
- Negative path: `cargo run -q -p sifr -- run <project_with_non_main_importing_missing_module>/main.sifr` -> exits `1` with `[worker] unknown module 'missing_mod'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 3: milestone_17_3 Test and Constant Import Parity
status: done (2026-03-04, PR #815)

- [x] Align `sifr test` import behavior with regular compilation
- [x] Support local-module constant imports in externals model
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- test demos/m17_3_test_and_constant_import_parity_demo` -> pass (`test_import_parity` passes).
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes `test_collect_project_modules_exports_local_constants` and `test_run_tests_resolves_local_imports_and_constants`).
- Positive path: `cargo test -q -p sifr_codegen test_generate_rust_test_emits_local_module_import_uses` -> pass.
- Negative path: `cargo run -q -p sifr -- test <tmp_dir_with_missing_imported_constant>` -> exits `1` with `module 'helper' has no member 'MISSING'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/813 (merged)
- Part 2: https://github.com/yaseralnajjar/sifr/pull/814 (merged)
- Part 3: https://github.com/yaseralnajjar/sifr/pull/815 (merged)

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase17-review.md`
- Remediation PR (pass 1): https://github.com/yaseralnajjar/sifr/pull/816 (open)
- Triage outcome: reviewer findings were based on pre-merge code and were validated as already fixed by PRs #813, #814, and #815.
- Validation commands used during triage:
  - `cargo test -q -p sifr_driver test_check_only_reports_frontend_phases`
  - `cargo test -q -p sifr_driver test_run_tests_resolves_local_imports_and_constants`
  - `cargo test -q -p sifr_codegen test_generate_rust_test_emits_local_module_import_uses`
  - `cargo test -q -p sifr_codegen test_generate_rust_multi_exports_non_main_items`
