# Phase 33: Developer Tooling and Ecosystem Hooks

## Objective
Enable core developer tooling integration as a dedicated phase immediately after performance hardening.

## Depends on
- Phase 32

## Milestones

### milestone_33_1: Developer Tooling and Ecosystem Hooks
- Scope:
  - LSP/formatter/linter/doc hooks aligned with new phase contracts.
- Definition of done:
  - Tooling integrates with language/runtime capabilities added in prior phases.

## Quality Contract
- Entry criteria: Phase 32 is completed and compiler performance budgets are enforced.
- Exit criteria: Tooling hooks are coherent, stable, and aligned with current phase contracts.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_33_1` (Developer Tooling and Ecosystem Hooks): validation goals cover: LSP/formatter/linter/doc hooks aligned with new phase contracts. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Tooling hooks are coherent, stable, and aligned with current phase contracts.

## Exit Gate
- Tooling hooks are coherent, stable, and aligned with current phase contracts.
