# Phase 17: Import and Externals Correctness

## Objective
Fix import/external resolution correctness across `check`, `run`, `build`, and `test` pipelines.

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
  - Milestone demo: `cargo run -q -p sifr -- run demos/m17_1_frontend_only_check_path_demo.sifr`.

### milestone_17_2: Non-Main Externals Resolution
status: done (2026-03-04, PR #TBD)
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
    - `test_generate_rust_multi_skips_stdlib_use_paths_in_non_main_modules` (`crates/sifr_codegen/src/lib_codegen_tests.rs`)
  - Milestone demo: `cargo run -q -p sifr -- run demos/m17_2_non_main_externals_resolution_demo/main.sifr`.

### milestone_17_3: Test and Constant Import Parity
- Scope:
  - Align `sifr test` import behavior with regular compilation.
  - Support local-module constant imports in externals model.
- Definition of done:
  - Test runner imports behave like compile pipeline.
  - Local constants import successfully across modules.

## Quality Contract
- Entry criteria: Phase 16 is completed and deterministic local profiles are in place.
- Exit criteria: Import semantics are correct and consistent in all execution modes.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_17_1` (Frontend-Only Check Path): validation goals cover: Ensure `check` stops after frontend/type phases; Remove codegen/runtime coupling from check flow. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_2` (Non-Main Externals Resolution): validation goals cover: Resolve stdlib/local externals in non-main modules; Ensure multi-file projects type-check consistently. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_3` (Test and Constant Import Parity): validation goals cover: Align `sifr test` import behavior with regular compilation; Support local-module constant imports in externals model. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Import semantics are correct and consistent in all execution modes.

## Exit Gate
- Import semantics are correct and consistent in all execution modes.
