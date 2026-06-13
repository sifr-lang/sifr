# Phase 22 Execution Checklist (Frontend Mode Parity Hardening)

Status: completed (2026-03-06)
Owner: phase_22 execution loop
Reference phase doc: `internal_docs/phases/22_frontend_mode_parity_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 22 To-Do Plan

### Part 1: milestone_22_1 Canonical Frontend Entry Path
- [ ] Define one shared frontend module-orchestration path in `sifr_driver` consumed by `check`, `build`, `run`, and `test`
- [ ] Remove mode-specific lowering/resolution forks; allow only explicit mode flags for documented diagnostic surface differences
- [ ] Add regression tests to lock shared frontend behavior across single-module and project-module lowering entrypoints
- [ ] Add milestone 22.1 positive demo
- [ ] Add milestone 22.1 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_22_2 Project-Aware `check` Parity
- [ ] Make `sifr check` path-aware for project entries and local module resolution parity with `build`/`run`
- [ ] Ensure stdlib external resolution parity in `check` for multi-file projects
- [ ] Add tests covering known gap closure for valid local imports in `check`
- [ ] Add milestone 22.2 positive demo
- [ ] Add milestone 22.2 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_22_3 Cross-Mode Diagnostic and Exit Contract
- [ ] Define and document explicit parity rules for frontend diagnostics, exit codes, and ordering guarantees across `check`/`build`/`run`/`test`
- [ ] Add regression tests for equivalent frontend failures surfacing equivalent diagnostics across modes
- [ ] Add tests for deterministic ordering of frontend errors/diagnostics
- [ ] Add milestone 22.3 positive demo
- [ ] Add milestone 22.3 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_22_4 Parity Regression Matrix
- [ ] Add an explicit parity matrix that runs representative positive/negative fixtures through `check`, `build`, `run`, and `test` frontend paths
- [ ] Include fixtures for local imports, stdlib externals, and multi-module resolution
- [ ] Wire matrix into local validation workflow so mode drift fails before merge
- [ ] Add milestone 22.4 positive demo
- [ ] Add milestone 22.4 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_22_1 Canonical Frontend Entry Path
status: done (2026-03-06, PR #856)

- [x] Shared frontend orchestration path implemented and reused across modes
- [x] Explicit mode flag behavior documented in code/tests
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_compile_frontend_modules_uses_explicit_diagnostic_style` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_check_and_project_lowering_share_typecheck_contract` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m22_1_canonical_frontend_entry_path_demo/main.sifr` -> prints `m22_1 canonical frontend entry path demo:` and `7`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m22_1_canonical_frontend_entry_path_demo/negative_cases/type_error_dependency/main.sifr` -> exits `1` with `type error: [helper] return type mismatch: expected 'int', got 'str'`.

## Part 2: milestone_22_2 Project-Aware `check` Parity
status: done (2026-03-06, PR #857)

- [x] Project-aware `check` implementation complete
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_check_project_resolves_valid_local_imports` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_check_project_error_messages_match_build_project` -> pass.
- Positive path: `cargo test -q -p sifr test_check_entrypoint_project_mode_resolves_local_imports` -> pass.
- Positive path: `cargo test -q -p sifr test_check_entrypoint_project_mode_error_parity_with_compile_entrypoint` -> pass.
- Positive path: `cargo run -q -p sifr -- check demos/m22_2_project_aware_check_parity_demo/main.sifr` -> `no errors found`.
- Positive path: `cargo run -q -p sifr -- run demos/m22_2_project_aware_check_parity_demo/main.sifr` -> prints `m22_2 project-aware check parity demo:` and `9.42477796076938`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- check demos/m22_2_project_aware_check_parity_demo/negative_cases/helper_type_error/main.sifr` -> exits `1` with `type error: [helper] return type mismatch: expected 'float', got 'str'`.

## Part 3: milestone_22_3 Cross-Mode Diagnostic and Exit Contract
status: done (2026-03-06, PR #858)

- [x] Diagnostic/exit/ordering contract implemented and documented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_run_tests_reports_deterministic_parse_error_order` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_run_tests_frontend_type_errors_use_single_path_prefix` -> pass.
- Positive path: `cargo test -q -p sifr test_frontend_error_messages_match_across_check_build_and_run_paths` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m22_3_cross_mode_diagnostic_exit_contract_demo/main.sifr` -> prints `m22_3 cross-mode diagnostic and exit contract demo:` and `42`.
- Positive path: manual contract check (`check/build/run`) on `demos/m22_3_cross_mode_diagnostic_exit_contract_demo/negative_cases/helper_type_error/main.sifr` -> `check_exit=1`, `build_exit=1`, `run_exit=1`, and `check_build_err_diff=0`, `check_run_err_diff=0`.
- Positive path: manual contract check (`test`) on synthetic parse-error fixtures -> `test_exit=1` and first error references lexicographically first file (`a_bad.sifr`).
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- check demos/m22_3_cross_mode_diagnostic_exit_contract_demo/negative_cases/helper_type_error/main.sifr` -> exits `1` with `type error: [helper] return type mismatch: expected 'int', got 'str'`.

## Part 4: milestone_22_4 Parity Regression Matrix
status: done (2026-03-06, PR #859)

- [x] Parity matrix implemented and wired into local validation
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `bash scripts/run_frontend_mode_parity_matrix.sh` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m22_4_parity_regression_matrix_demo/main.sifr` -> prints `m22_4 parity regression matrix demo:` and `8`.
- Positive path: matrix positive row asserts all frontend modes succeed on same representative corpus (`check`, `build`, `run`, `test`).
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes matrix gate).
- Negative path: `cargo run -q -p sifr -- run demos/m22_4_parity_regression_matrix_demo/negative_cases/type_error_project/main.sifr` -> exits `1` with `type error: [helper] return type mismatch: expected 'int', got 'str'`.
- Negative path: matrix negative row asserts:
  - `check/build/run` all fail with exit `1` and byte-identical diagnostics for equivalent frontend failure.
  - `test` fails with exit `1` and expected frontend type error message on the same fixture family.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/856
- Part 2: https://github.com/sifr-lang/sifr/pull/857
- Part 3: https://github.com/sifr-lang/sifr/pull/858
- Part 4: https://github.com/sifr-lang/sifr/pull/859

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase22-review.md` (2026-03-06, approved)
- Pass-1 triage outcome:
  - Reviewer approved phase 22 implementation as complete across milestones 22.1-22.4.
  - No blocking defects were identified; no compiler-code remediation was required.
  - Non-blocking future considerations were recorded as advisory notes only.
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/860
- External review pass 2 output: `reviews/phase22-production-grade-review.md` (2026-03-06, approved for production)
- Pass-2 triage outcome:
  - Reviewer confirmed production-grade readiness with no blocking defects.
  - Applied the actionable recommendation by documenting `FrontendDiagnosticStyle` variants inline for long-term contract clarity.
  - Broader matrix-expansion suggestions were retained as future advisory work outside phase 22 scope.
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/861
