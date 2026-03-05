# Phase 22: Frontend Mode Parity Hardening

## Objective
Eliminate semantic drift between `check`, `build`, `run`, and `test` by enforcing one canonical frontend contract and project-aware behavior for `check`.

## Depends on
- Phase 21

## Milestones

### milestone_22_1: Canonical Frontend Entry Path
- Scope:
  - Define one shared frontend orchestration path in `sifr_driver` used by all CLI modes (`check`, `build`, `run`, `test`).
  - Remove mode-specific lowering/resolution forks that can produce divergent behavior for equivalent inputs.
- Definition of done:
  - All modes call the same frontend analysis/resolution API with explicit mode flags only for allowed differences.

### milestone_22_2: Project-Aware `check` Parity
- Scope:
  - Make `sifr check` resolve local project modules and stdlib externals with the same correctness contract as `build`/`run`.
  - Close the known gap where `check` fails on valid local imports in multi-file projects.
- Definition of done:
  - Multi-file projects type-check consistently in `check` and `build` for identical source/configuration.

### milestone_22_3: Cross-Mode Diagnostic and Exit Contract
- Scope:
  - Define mode-parity rules for diagnostics, exit codes, and ordering guarantees for frontend-phase failures.
  - Ensure equivalent frontend errors surface with equivalent diagnostics across modes.
- Definition of done:
  - Diagnostics/exit behavior is documented and regression-locked for all frontend modes.

### milestone_22_4: Parity Regression Matrix
- Scope:
  - Add an explicit parity matrix that runs the same representative corpus through `check`, `build`, `run`, and `test` frontend paths.
  - Include positive and negative fixtures covering imports, externals, and multi-module resolution.
- Definition of done:
  - Mode drift regressions are caught automatically before merge.

## Quality Contract
- Entry criteria: Phase 21 is completed and traversal/control-flow behavior is stable.
- Exit criteria: Frontend semantic parity across `check/build/run/test` is enforced and regression-covered.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_22_1` (Canonical Frontend Entry Path): validation goals cover: Define one shared frontend orchestration path in `sifr_driver` used by all CLI modes (`check`, `build`, `run`, `test`); Remove mode-specific lowering/resolution forks that can produce divergent behavior for equivalent inputs. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_22_2` (Project-Aware `check` Parity): validation goals cover: Make `sifr check` resolve local project modules and stdlib externals with the same correctness contract as `build`/`run`; Close the known gap where `check` fails on valid local imports in multi-file projects. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_22_3` (Cross-Mode Diagnostic and Exit Contract): validation goals cover: Define mode-parity rules for diagnostics, exit codes, and ordering guarantees for frontend-phase failures; Ensure equivalent frontend errors surface with equivalent diagnostics across modes. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_22_4` (Parity Regression Matrix): validation goals cover: Add an explicit parity matrix that runs the same representative corpus through `check`, `build`, `run`, and `test` frontend paths; Include positive and negative fixtures covering imports, externals, and multi-module resolution. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Frontend semantic parity across `check/build/run/test` is enforced and regression-covered.

## Exit Gate
- Frontend semantic parity across `check/build/run/test` is enforced and regression-covered.
