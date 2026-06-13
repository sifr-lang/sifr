# Phase 17: Import and Externals Correctness

## Objective
Fix import/external resolution correctness across `check`, `run`, `build`, and `test` pipelines.

Status: completed (2026-03-04)
Follow-up planning extension: added milestone_17_4 (2026-03-05)

## Depends on
- Phase 16

## Milestones

### milestone_17_1: Frontend-Only Check Path
status: done (2026-03-04, PR #813)
- Scope:
  - Ensure `check` stops after frontend/type phases.
  - Remove codegen/runtime coupling from check flow.
- Definition of done:
  - `check` no longer triggers full code generation.
- Evidence:
  - `check` now runs dedicated frontend/type-only lowering via `compile_frontend` instead of routing through `compile` codegen.
  - Frontend diagnostics printing is centralized and reused for both `check` and `compile`.
  - Regression guard: `test_check_only_reports_frontend_phases` in `crates/sifr_driver/src/lib.rs`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/type_checking/main.sifr`.

### milestone_17_2: Non-Main Externals Resolution
status: done (2026-03-04, PR #814)
- Scope:
  - Resolve stdlib/local externals in non-main modules.
  - Ensure multi-file projects type-check consistently.
- Definition of done:
  - Non-main modules can import stdlib/local modules correctly.
- Evidence:
  - Project lowering now uses dependency-aware retries with shared external definitions in `collect_project_hir_modules`.
  - Non-main modules are lowered through `lower_module_with_externals` (instead of isolated `lower_module`) so stdlib/local imports resolve consistently.
  - Multi-module codegen now skips invalid `use crate::sifr.*` emission for stdlib imports.
  - Regression guards:
    - `test_collect_project_modules_allows_non_main_stdlib_imports` (`crates/sifr_driver/src/lib.rs`)
    - `test_collect_project_modules_resolves_non_main_local_dependencies` (`crates/sifr_driver/src/lib.rs`)
    - `test_collect_project_modules_reports_unknown_module_in_non_main` (`crates/sifr_driver/src/lib.rs`)
    - `test_collect_project_modules_cycle_reports_error` (`crates/sifr_driver/src/lib.rs`)
    - `test_generate_rust_multi_skips_stdlib_use_paths_in_non_main_modules` (`crates/sifr_codegen/src/lib_codegen_tests.rs`)
  - Milestone demo: `cargo run -q -p sifr -- run demos/external_modules/main.sifr`.

### milestone_17_3: Test and Constant Import Parity
status: done (2026-03-04, PR #815)
- Scope:
  - Align `sifr test` import behavior with regular compilation.
  - Support local-module constant imports in externals model.
- Definition of done:
  - Test runner imports behave like compile pipeline.
  - Local constants import successfully across modules.
- Evidence:
  - Local-module constant exports are now registered in project externals (`collect_module_exports` in `crates/sifr_driver/src/lib.rs`).
  - Local import resolution now treats constants as module-members (`externals.constants`) in `crates/sifr_lowering/src/lower/`.
  - `run_tests` now builds support-module externals and lowers test modules via `lower_module_with_externals`, aligning with project compilation flow.
  - Generated test-runner crate source is now explicitly test-scoped via `compose_test_runner_lib` so non-test cargo builds do not emit unused-import/dead-code noise.
  - Test codegen now emits local-module `use crate::<module>::<name>` imports in test mode (`crates/sifr_codegen/src/entrypoints.rs`).
  - Milestone demo: `cargo run -q -p sifr -- test demos/test_imports_and_constants`.

### milestone_17_4: Import-Form Semantics Closure
status: done (2026-03-05, PR #828)
- Scope:
  - Define canonical compiler semantics for import forms: `from x import ...`, `from .x import ...`, `from ..x import ...`, `from . import ...`, and `import x`.
  - Ensure import-form behavior is explicit and consistent in `check`, `run`, `build`, and `test` pipelines (not only CLI mode detection).
- Definition of done:
  - Import-form support matrix is explicitly documented in phase/docs and reflected by deterministic compiler behavior.
  - Unsupported or non-activating forms fail or downgrade with explicit, stable diagnostics (no implicit heuristics).
  - Regression suite includes positive/negative coverage for all supported and unsupported import forms.
- Canonical import-form matrix:
  - `from x import ...` (level `0`, qualified module): supported; activates project/module resolution in all execution modes.
  - `from .x import ...` (level `1`, qualified module): supported; activates project/module resolution in all execution modes.
  - `from ..x import ...` (level `>1`): unsupported; frontend error `unsupported relative import level <n>; only one leading dot is supported`.
  - `from . import ...` (bare relative, no module): unsupported; frontend error `unsupported bare relative import; expected module name after '.'`.
  - `import x`: unsupported; frontend error `unsupported import statement; use 'from x import <name>'`.
  - `from typing import ...` (level `0`): non-activating for local module resolution; handled as typing support import.
  - `from enum import ...` (level `0`): non-activating for local module resolution; handled as enum support import.
- Evidence:
  - Lowering now rejects unsupported import forms with explicit diagnostics:
    - multi-level relative imports (`level > 1`)
    - bare relative imports (`from . import ...`)
    - regular import statements (`import x`)
  - Absolute-only typing/enum skip rules are now level-aware so relative imports do not silently bypass import semantics.
  - Regression guards in `crates/sifr_driver/src/lib.rs`:
    - `test_check_reports_unsupported_multi_level_relative_import`
    - `test_check_reports_unsupported_bare_relative_import`
    - `test_check_reports_unsupported_import_statement`
    - `test_collect_project_modules_supports_single_level_relative_import`
  - Milestone demo: `cargo run -q -p sifr -- run demos/import_forms/main.sifr` (uses supported `from .helper import value`).

## Quality Contract
- Entry criteria: Phase 16 is completed and deterministic local profiles are in place.
- Exit criteria: Import semantics are correct and consistent in all execution modes.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Import-form semantics must be covered by an explicit matrix (supported, unsupported, and non-activating forms) with deterministic outcomes and no implicit behavior.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_17_1` (Frontend-Only Check Path): validation goals cover: Ensure `check` stops after frontend/type phases; Remove codegen/runtime coupling from check flow. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_2` (Non-Main Externals Resolution): validation goals cover: Resolve stdlib/local externals in non-main modules; Ensure multi-file projects type-check consistently. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_3` (Test and Constant Import Parity): validation goals cover: Align `sifr test` import behavior with regular compilation; Support local-module constant imports in externals model. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_4` (Import-Form Semantics Closure): validation goals cover: Define canonical semantics for `from`/relative/bare-relative/`import` forms; Ensure consistent behavior and diagnostics across `check/run/build/test`. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Import semantics are correct and consistent in all execution modes, including the import-form matrix.

## Exit Gate
- Import semantics are correct and consistent in all execution modes, with an explicit and regression-protected import-form matrix.
