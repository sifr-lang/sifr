# Phase 18 Execution Checklist (Project and CLI Semantics Correctness)

Status: completed (2026-03-04)
Owner: phase_18 execution loop
Reference phase doc: `internal_docs/phases/18_project_and_cli_semantics_correctness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `./scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_18_1 Run/Build Semantics Alignment
status: done (2026-03-04, PR #818)

- [x] Align project detection and compilation scope between `run` and `build`
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m18_1_run_build_semantics_alignment_demo/main.sifr` -> prints `m18_1 run/build alignment demo:` and `aligned`.
- Positive path: `cargo run -q -p sifr -- build demos/m18_1_run_build_semantics_alignment_demo/main.sifr -o <tmp_output_dir>` -> succeeds and emits compiled binary path.
- Positive path: `cargo test -q -p sifr` -> pass (includes CLI mode resolver tests).
- Negative path: `cargo run -q -p sifr -- run <tmp_project_with_missing_module>/main.sifr` -> exits `1` with `[main] unknown module 'missing_mod'`.
- Negative path: `cargo run -q -p sifr -- build <tmp_project_with_missing_module>/main.sifr -o <tmp_output_dir>` -> exits `1` with `[main] unknown module 'missing_mod'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 2: milestone_18_2 Auto-Detection Rule Tightening
status: done (2026-03-04, PR #819)

- [x] Replace over-aggressive auto project mode with explicit, documented rules
- [x] Ensure nearby scratch files do not unexpectedly break single-file runs
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m18_2_auto_detection_rule_tightening_demo/main.sifr` -> prints `m18_2 auto-detection demo:` and `3` despite invalid neighboring `scratch.sifr`.
- Positive path: `cargo run -q -p sifr -- build demos/m18_2_auto_detection_rule_tightening_demo/main.sifr -o <tmp_output_dir>` -> succeeds in single-file mode.
- Positive path: `cargo test -q -p sifr test_resolve_compilation_mode_` -> pass.
- Negative path: `cargo run -q -p sifr -- run <tmp_project_with_main_importing_local_helper_and_invalid_helper>/main.sifr` -> exits `1` with parse error in `helper`.
- Negative path: `cargo run -q -p sifr -- build <tmp_project_with_main_importing_local_helper_and_invalid_helper>/main.sifr -o <tmp_output_dir>` -> exits `1` with parse error in `helper`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 3: milestone_18_3 CLI Contract and Regression Suite
status: done (2026-03-04, PR #820)

- [x] Document stable CLI semantics and edge cases
- [x] Add regression tests for command-mode behavior
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m18_3_cli_contract_and_regression_suite_demo.sifr` -> prints `m18_3 cli contract and regression suite demo`.
- Positive path: `cargo test -q -p sifr test_resolve_compilation_mode_` -> pass with command-mode resolver contract tests.
- Positive path: docs contract published in `docs/cli_command_semantics.md` and linked in `README.md`.
- Negative path: `cargo run -q -p sifr -- run <tmp_main_importing_missing_local_module_with_invalid_scratch>/main.sifr` -> exits `1` with `unknown module 'helper'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 4: milestone_18_4 CLI Resolver Trigger-Matrix Closure
status: done (2026-03-05, PR #831)

- [x] Define canonical trigger-matrix semantics for `from`/relative/bare-relative/`import` forms
- [x] Synchronize trigger matrix across implementation, tests, and CLI contract docs
- [x] Ensure run/build mode-resolution equivalence for matrix entries
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Documentation path: explicit resolver trigger matrix recorded in `docs/cli_command_semantics.md`.
- Positive path: `cargo run -q -p sifr -- run demos/m18_4_cli_resolver_trigger_matrix_closure_demo/main.sifr` -> prints `m18_4 resolver trigger matrix demo:` and `18` (demo exercises supported `from .helper import value`).
- Positive path: `cargo test -q -p sifr test_resolve_compilation_mode_` -> pass (includes regular import fallback + relative-import resolver matrix tests).
- Positive path: `cargo test -q -p sifr test_compile_entrypoint_error_consistency_for_` -> pass (includes run/build consistency checks for project mode, regular import, bare relative, and multi-level relative).
- Negative path: `cargo run -q -p sifr -- run demos/m18_4_cli_resolver_trigger_matrix_closure_demo/negative_cases/main_regular_import.sifr` -> exits `1` with `unsupported import statement`.
- Negative path: `cargo run -q -p sifr -- run demos/m18_4_cli_resolver_trigger_matrix_closure_demo/negative_cases/main_bare_relative.sifr` -> exits `1` with `unsupported bare relative import`.
- Negative path: `cargo run -q -p sifr -- run demos/m18_4_cli_resolver_trigger_matrix_closure_demo/negative_cases/main_multi_level_relative.sifr` -> exits `1` with `unsupported relative import level 2`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/818 (merged)
- Part 2: https://github.com/sifr-lang/sifr/pull/819 (merged)
- Part 3: https://github.com/sifr-lang/sifr/pull/820 (merged)
- Part 4: https://github.com/sifr-lang/sifr/pull/831 (merged)

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase18-review.md`
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/821 (merged)
- Pass-1 triage + actions:
  - Added explicit resolver regression tests for `typing`, `enum`, and package-like `__init__.sifr` imports.
  - Expanded CLI contract docs to cover unsupported package-style auto-detect and parse/read fallback behavior.
- Validation commands used for pass-1 remediation:
  - `cargo test -q -p sifr test_resolve_compilation_mode_`
- External review pass 2 output: `reviews/phase18-production-grade-review.md`
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/822 (merged)
- Pass-2 triage + actions:
  - Added regression test for relative-import project-mode activation (`from .helper import ...` with sibling module).
  - Added regression test for run/build project-mode error consistency via shared `compile_entrypoint`.
  - Clarified CLI contract notes for relative import behavior and stdlib-like local module names.
- Validation commands used for pass-2 remediation:
  - `cargo test -q -p sifr test_resolve_compilation_mode_`
  - `cargo test -q -p sifr test_compile_entrypoint_error_consistency_for_project_mode`
- External review pass 3 output: `reviews/phase18-review-2.md`
- Remediation PR (pass 3): https://github.com/sifr-lang/sifr/pull/826
- Pass-3 remediation summary:
  - Added resolver regression test for relative import without sibling module to enforce single-file fallback.
  - Added resolver regression tests proving local `typing.sifr`/`enum.sifr` files do not activate project mode for stdlib-like imports.
  - Corrected CLI contract note to match implemented semantics for stdlib-like local filenames.
- Validation commands used during pass-3 remediation:
  - `cargo test -q -p sifr test_resolve_compilation_mode_`
  - `cargo run -q -p sifr -- run demos/m18_2_auto_detection_rule_tightening_demo/main.sifr`
  - `cargo run -q -p sifr -- run demos/m18_3_cli_contract_and_regression_suite_demo.sifr`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- External review pass 4 output: `reviews/phase18-production-grade-review-2.md`
- Remediation PR (pass 4): none (pass-4 remediation changes already landed; this entry records final verified status)
- Pass-4 remediation summary:
  - Enforced resolver behavior that only single-dot relative imports are considered for local project auto-detect.
  - Added resolver regression tests for multi-level relative imports and bare relative imports to prevent accidental project-mode activation.
  - Updated CLI contract docs to explicitly document multi-level and bare-relative import behavior.
- Validation commands used during pass-4 remediation:
  - `cargo test -q -p sifr test_resolve_compilation_mode_`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- External review pass 5 output: `reviews/phase18-review-3.md`
- Remediation PR (pass 5): https://github.com/sifr-lang/sifr/pull/832
- Pass-5 triage outcome:
  - No new still-valid gaps; reviewer confirmed milestone `18_4` trigger-matrix coverage and phase-18 quality-contract coverage are complete.
- External review pass 6 output: `reviews/phase18-production-grade-review-3.md`
- Remediation PR (pass 6): https://github.com/sifr-lang/sifr/pull/833
- Pass-6 triage outcome:
  - No concrete production-grade defects identified; phase remains production-ready for milestone `18_4`.
