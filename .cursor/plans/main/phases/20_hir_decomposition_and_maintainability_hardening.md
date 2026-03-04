# Phase 20: HIR Decomposition and Maintainability Hardening

## Objective
Decompose oversized HIR files into focused modules without changing behavior, and prevent future regrowth.

## Depends on
- Phase 19

## Milestones

### milestone_20_1: Split `lower.rs`
- Scope:
  - Extract lowering concerns into coherent submodules (imports, statements, expressions, typing hooks, diagnostics).
  - Preserve current semantics and test outcomes.
- Definition of done:
  - `lower.rs` is split into maintainable units with no behavior drift.

### milestone_20_2: Split `stdlib.rs`
- Scope:
  - Partition stdlib metadata/registration logic into focused modules.
- Definition of done:
  - `stdlib.rs` is modularized with equivalent behavior.

### milestone_20_3: Anti-Regrowth Guardrails
- Scope:
  - Add file-size and module-boundary conventions.
  - Add review checklist items for new lowering additions.
- Definition of done:
  - Guardrails are documented and enforced in local/CI checks where practical.

## Quality Contract
- Entry criteria: Phase 19 is completed and module graph determinism is enforced.
- Exit criteria: HIR layer is materially more maintainable with regression-safe modular structure.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_20_1` (Split `lower.rs`): validation goals cover: Extract lowering concerns into coherent submodules (imports, statements, expressions, typing hooks, diagnostics); Preserve current semantics and test outcomes. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_20_2` (Split `stdlib.rs`): validation goals cover: Partition stdlib metadata/registration logic into focused modules. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_20_3` (Anti-Regrowth Guardrails): validation goals cover: Add file-size and module-boundary conventions; Add review checklist items for new lowering additions. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: HIR layer is materially more maintainable with regression-safe modular structure.

## Exit Gate
- HIR layer is materially more maintainable with regression-safe modular structure.
