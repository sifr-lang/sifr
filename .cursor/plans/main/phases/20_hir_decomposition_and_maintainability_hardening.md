# Phase 22: HIR Decomposition and Maintainability Hardening

## Objective
Decompose oversized HIR files into focused modules without changing behavior, and prevent future regrowth.

## Depends on
- Phase 21

## Milestones

### milestone_22_1: Split `lower.rs`
- Scope:
  - Extract lowering concerns into coherent submodules (imports, statements, expressions, typing hooks, diagnostics).
  - Preserve current semantics and test outcomes.
- Definition of done:
  - `lower.rs` is split into maintainable units with no behavior drift.

### milestone_22_2: Split `stdlib.rs`
- Scope:
  - Partition stdlib metadata/registration logic into focused modules.
- Definition of done:
  - `stdlib.rs` is modularized with equivalent behavior.

### milestone_22_3: Anti-Regrowth Guardrails
- Scope:
  - Add file-size and module-boundary conventions.
  - Add review checklist items for new lowering additions.
- Definition of done:
  - Guardrails are documented and enforced in local/CI checks where practical.

## Exit Gate
- HIR layer is materially more maintainable with regression-safe modular structure.
