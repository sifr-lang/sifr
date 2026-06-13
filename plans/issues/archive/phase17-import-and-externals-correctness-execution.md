# Phase 17 Execution Checklist (Import and Externals Correctness)

Status: completed (2026-03-04)
Owner: phase_17 execution loop
Reference phase doc: `internal_docs/phases/17_import_and_externals_correctness.md`

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

## Part 4: milestone_17_4 Import-Form Semantics Closure
status: done (2026-03-05, PR #828)

- [x] Define canonical semantics for `from`/relative/bare-relative/`import` forms
- [x] Ensure explicit deterministic diagnostics for unsupported forms
- [x] Add positive/negative regression coverage
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Documentation path: explicit canonical import-form matrix recorded in `internal_docs/phases/17_import_and_externals_correctness.md` under milestone `17_4`.
- Positive path: `cargo run -q -p sifr -- run demos/m17_4_import_form_semantics_closure_demo/main.sifr` -> prints `m17_4 import-form semantics demo:` and `17` (demo exercises supported `from .helper import value`).
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes import-form semantics regression tests).
- Negative path: `cargo run -q -p sifr -- check demos/m17_4_import_form_semantics_closure_demo/negative_cases/unsupported_import_statement.sifr` -> exits `1` with `unsupported import statement`.
- Negative path: `cargo run -q -p sifr -- check demos/m17_4_import_form_semantics_closure_demo/negative_cases/unsupported_bare_relative.sifr` -> exits `1` with `unsupported bare relative import`.
- Negative path: `cargo run -q -p sifr -- check demos/m17_4_import_form_semantics_closure_demo/negative_cases/unsupported_multi_relative.sifr` -> exits `1` with `unsupported relative import level 2`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/813 (merged)
- Part 2: https://github.com/sifr-lang/sifr/pull/814 (merged)
- Part 3: https://github.com/sifr-lang/sifr/pull/815 (merged)
- Part 4: https://github.com/sifr-lang/sifr/pull/828 (merged)

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase17-review.md`
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/816 (merged)
- Triage outcome: reviewer findings were based on pre-merge code and were validated as already fixed by PRs #813, #814, and #815.
- Validation commands used during triage:
  - `cargo test -q -p sifr_driver test_check_only_reports_frontend_phases`
  - `cargo test -q -p sifr_driver test_run_tests_resolves_local_imports_and_constants`
  - `cargo test -q -p sifr_codegen test_generate_rust_test_emits_local_module_import_uses`
  - `cargo test -q -p sifr_codegen test_generate_rust_multi_exports_non_main_items`
- External review pass 2 output: `reviews/phase17-production-grade-review.md`
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/817 (merged)
- Pass-2 triage outcome: production-grade review findings were also stale against current `origin/main` and validated as already fixed by PRs #813, #814, #815.
- Validation commands used during pass-2 triage:
  - `cargo test -q -p sifr_driver test_check_only_reports_frontend_phases`
  - `cargo test -q -p sifr_driver test_run_tests_resolves_local_imports_and_constants`
  - `cargo test -q -p sifr_codegen test_generate_rust_test_emits_local_module_import_uses`
  - `cargo test -q -p sifr_codegen test_generate_rust_multi_exports_non_main_items`
- External review pass 3 output: `reviews/phase17-review-2.md`
- Remediation PR (pass 3): https://github.com/sifr-lang/sifr/pull/824
- Pass-3 remediation summary:
  - Scoped generated test-runner `lib.rs` to `cfg(test)` to eliminate non-test build unused-import/dead-code warnings without changing runtime behavior.
  - Added negative-path regression coverage for non-main unknown-module imports and cyclic/no-progress module lowering.
- Validation commands used during pass-3 remediation:
  - `cargo test -q -p sifr_driver`
  - `cargo run -q -p sifr -- test demos/m17_3_test_and_constant_import_parity_demo`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- External review pass 4 output: `reviews/phase17-production-grade-review-2.md`
- Remediation PR (pass 4): none (no code or docs remediation required beyond recorded verification)
- Pass-4 outcome: no new confirmed defects; phase remains production-ready against updated quality-contract checks.
- Additional verification command:
  - `cargo test -q -p sifr_driver test_compose_test_runner_lib_is_test_scoped`
- External review pass 5 output: `reviews/phase17-review-3.md`
- Remediation PR (pass 5): https://github.com/sifr-lang/sifr/pull/829
- Pass-5 remediation summary:
  - Added explicit canonical import-form matrix table under milestone `17_4` to satisfy the quality-contract requirement for documented supported/unsupported/non-activating forms.
- External review pass 6 output: `reviews/phase17-production-grade-review-3.md`
- Remediation PR (pass 6): https://github.com/sifr-lang/sifr/pull/830
- Pass-6 remediation summary:
  - Added explicit positive-path coverage for supported level-1 relative imports via `test_collect_project_modules_supports_single_level_relative_import`.
  - Updated milestone `17_4` demo to use supported relative form `from .helper import value`.
