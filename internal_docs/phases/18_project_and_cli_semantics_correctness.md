# Phase 18: Project and CLI Semantics Correctness

## Objective
Make CLI behavior predictable for single-file and multi-file workflows.

Status: completed (2026-03-04)
Follow-up planning extension: added milestone_18_4 (2026-03-05)

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
  - Milestone demo: `cargo run -q -p sifr -- run demos/run_and_build/main.sifr`.

### milestone_18_2: Auto-Detection Rule Tightening
status: done (2026-03-04, PR #819)
- Scope:
  - Replace over-aggressive auto project mode with explicit, documented rules.
- Definition of done:
  - Nearby scratch files do not unexpectedly break single-file runs.
- Evidence:
  - CLI project-mode detection now requires `main.sifr` with at least one resolvable local-module import (`has_local_project_imports`) instead of sibling-file count heuristics.
  - Added resolver regression tests for `main` without local imports and stdlib-only imports to keep single-file mode stable.
  - Added explicit resolver regressions proving `typing.sifr`/`enum.sifr` local files do not activate project mode for stdlib-like imports.
  - Milestone demo: `cargo run -q -p sifr -- run demos/auto_detection/main.sifr`.

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
    - relative-import single-file fallback when sibling module is missing,
    - multi-level and bare relative import single-file fallback,
    - stdlib-only/no-import single-file fallback,
    - invalid-source and missing-module single-file fallback.
  - Milestone demo: `cargo run -q -p sifr -- run demos/cli_modes/main.sifr`.

### milestone_18_4: CLI Resolver Trigger-Matrix Closure
status: done (2026-03-05, PR #831)
- Scope:
  - Define canonical CLI project-mode activation semantics for `from x import ...`, relative import levels, bare relative imports, and regular `import x`.
  - Build on milestone_18_3 regression coverage by requiring explicit trigger-matrix definitions for every covered import form.
  - Ensure run/build resolver behavior is deterministic, explicit, and contract-synchronized with docs/tests.
- Definition of done:
  - A full trigger matrix exists for all import forms and is documented in the CLI semantics contract.
  - Resolver behavior for each form is regression-protected with positive and negative tests.
  - Run/build mode resolution remains equivalent for identical inputs across all matrix cases.
- Evidence:
  - Resolver tests in `crates/sifr/src/main.rs` now include regular-import fallback and run/build consistency checks for:
    - regular import (`import helper`)
    - bare relative import (`from . import helper`)
    - multi-level relative import (`from ..helper import value`)
  - CLI contract docs now include an explicit resolver trigger matrix in `docs/cli_command_semantics.md`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/resolver_triggers/main.sifr` (uses supported `from .helper import value`).

## Quality Contract
- Entry criteria: Phase 17 is completed and import/external behavior is stable.
- Exit criteria: CLI project semantics are stable, documented, and test-covered.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - CLI project-mode trigger rules must be captured as an explicit import-form matrix and synchronized between implementation, tests, and CLI contract docs.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_18_1` (Run/Build Semantics Alignment): validation goals cover: Align project detection and compilation scope between `run` and `build`. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_2` (Auto-Detection Rule Tightening): validation goals cover: Replace over-aggressive auto project mode with explicit, documented rules. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_3` (CLI Contract and Regression Suite): validation goals cover: Document stable CLI semantics and edge cases; Add regression tests for command-mode behavior. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_4` (CLI Resolver Trigger-Matrix Closure): validation goals cover: Define project-mode trigger behavior for `from`/relative/bare-relative/`import` forms; Keep run/build behavior equivalent across all matrix entries. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: CLI project semantics are stable, documented, and test-covered, including the trigger matrix.

## Exit Gate
- CLI project semantics are stable, documented, and test-covered, with an explicit and regression-protected trigger matrix.
