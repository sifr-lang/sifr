# Phase 17: Import and Externals Correctness

## Objective
Fix import/external resolution correctness across `check`, `run`, `build`, and `test` pipelines.

## Depends on
- Phase 16

## Milestones

### milestone_17_1: Frontend-Only Check Path
- Scope:
  - Ensure `check` stops after frontend/type phases.
  - Remove codegen/runtime coupling from check flow.
- Definition of done:
  - `check` no longer triggers full code generation.

### milestone_17_2: Non-Main Externals Resolution
- Scope:
  - Resolve stdlib/local externals in non-main modules.
  - Ensure multi-file projects type-check consistently.
- Definition of done:
  - Non-main modules can import stdlib/local modules correctly.

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
- Validation planning goals:
  - `milestone_17_1` (Frontend-Only Check Path): validation goals cover: Ensure `check` stops after frontend/type phases; Remove codegen/runtime coupling from check flow. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_2` (Non-Main Externals Resolution): validation goals cover: Resolve stdlib/local externals in non-main modules; Ensure multi-file projects type-check consistently. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_17_3` (Test and Constant Import Parity): validation goals cover: Align `sifr test` import behavior with regular compilation; Support local-module constant imports in externals model. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Import semantics are correct and consistent in all execution modes.

## Exit Gate
- Import semantics are correct and consistent in all execution modes.
