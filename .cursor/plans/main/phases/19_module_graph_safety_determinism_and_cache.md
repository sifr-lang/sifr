# Phase 19: Module Graph Safety, Determinism, and Cache

## Objective
Make multi-module compilation dependency-safe, deterministic, and efficient for repeated local loops.

## Depends on
- Phase 18

## Milestones

### milestone_19_1: Dependency-Safe Module Ordering
- Scope:
  - Introduce topological ordering for module compilation.
  - Add cycle diagnostics with actionable context.
- Definition of done:
  - Module compile order is dependency-correct and cycle-safe.

### milestone_19_2: Deterministic Assembly
- Scope:
  - Remove nondeterministic HashMap-order behavior from module assembly/output.
- Definition of done:
  - Repeated builds produce stable module output order.

### milestone_19_3: Stdlib Cache for Local Loops
- Scope:
  - Cache stdlib compilation artifacts for repeated check/test cycles.
- Definition of done:
  - Repeated local runs avoid redundant stdlib recompilation.

## Quality Contract
- Entry criteria: Phase 18 is completed and project-mode semantics are stable.
- Exit criteria: Multi-module builds are deterministic, cycle-safe, and faster in local iteration.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_19_1` (Dependency-Safe Module Ordering): validation goals cover: Introduce topological ordering for module compilation; Add cycle diagnostics with actionable context. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_19_2` (Deterministic Assembly): validation goals cover: Remove nondeterministic HashMap-order behavior from module assembly/output. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_19_3` (Stdlib Cache for Local Loops): validation goals cover: Cache stdlib compilation artifacts for repeated check/test cycles. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Multi-module builds are deterministic, cycle-safe, and faster in local iteration.

## Exit Gate
- Multi-module builds are deterministic, cycle-safe, and faster in local iteration.
