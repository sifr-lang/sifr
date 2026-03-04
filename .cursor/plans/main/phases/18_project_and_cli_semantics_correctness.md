# Phase 18: Project and CLI Semantics Correctness

## Objective
Make CLI behavior predictable for single-file and multi-file workflows.

Status: completed (2026-03-04)

## Depends on
- Phase 17

## Milestones

### milestone_18_1: Run/Build Semantics Alignment
status: done (2026-03-04, PR #818)
- Scope:
  - Align project detection and compilation scope between `run` and `build`.
- Definition of done:
  - Equivalent project inputs yield equivalent resolution behavior.
- Evidence:
  - Shared CLI compilation mode resolver (`resolve_compilation_mode`) now drives both `cmd_run` and `cmd_build` in `crates/sifr/src/main.rs`.
  - Added resolver regression tests: `test_resolve_compilation_mode_project_for_main_with_siblings` and `test_resolve_compilation_mode_single_file_for_non_main_entry`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m18_1_run_build_semantics_alignment_demo/main.sifr`.

### milestone_18_2: Auto-Detection Rule Tightening
status: done (2026-03-04, PR #819)
- Scope:
  - Replace over-aggressive auto project mode with explicit, documented rules.
- Definition of done:
  - Nearby scratch files do not unexpectedly break single-file runs.
- Evidence:
  - CLI project-mode detection now requires `main.sifr` with at least one resolvable local-module import (`has_local_project_imports`) instead of sibling-file count heuristics.
  - Added resolver regression tests for `main` without local imports and stdlib-only imports to keep single-file mode stable.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m18_2_auto_detection_rule_tightening_demo/main.sifr`.

### milestone_18_3: CLI Contract and Regression Suite
status: done (2026-03-04, PR #820)
- Scope:
  - Document stable CLI semantics and edge cases.
  - Add regression tests for command-mode behavior.
- Definition of done:
  - CLI behavior contract exists and is regression-protected.
- Evidence:
  - Stable CLI behavior contract documented in `docs/cli_command_semantics.md` and linked from `README.md`.
  - Regression suite expanded in `crates/sifr/src/main.rs` for:
    - local-import project mode activation,
    - stdlib-only/no-import single-file fallback,
    - invalid-source and missing-module single-file fallback.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m18_3_cli_contract_and_regression_suite_demo.sifr`.

## Quality Contract
- Entry criteria: Phase 17 is completed and import/external behavior is stable.
- Exit criteria: CLI project semantics are stable, documented, and test-covered.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_18_1` (Run/Build Semantics Alignment): validation goals cover: Align project detection and compilation scope between `run` and `build`. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_2` (Auto-Detection Rule Tightening): validation goals cover: Replace over-aggressive auto project mode with explicit, documented rules. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_3` (CLI Contract and Regression Suite): validation goals cover: Document stable CLI semantics and edge cases; Add regression tests for command-mode behavior. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: CLI project semantics are stable, documented, and test-covered.

## Exit Gate
- CLI project semantics are stable, documented, and test-covered.
