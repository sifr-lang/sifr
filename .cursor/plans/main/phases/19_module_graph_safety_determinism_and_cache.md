# Phase 21: Module Graph Safety, Determinism, and Cache

## Objective
Make multi-module compilation dependency-safe, deterministic, and efficient for repeated local loops.

## Depends on
- Phase 20 (and fix-back linkage to Phase 19 when needed)

## Milestones

### milestone_21_1: Dependency-Safe Module Ordering
- Scope:
  - Introduce topological ordering for module compilation.
  - Add cycle diagnostics with actionable context.
- Definition of done:
  - Module compile order is dependency-correct and cycle-safe.

### milestone_21_2: Deterministic Assembly
- Scope:
  - Remove nondeterministic HashMap-order behavior from module assembly/output.
- Definition of done:
  - Repeated builds produce stable module output order.

### milestone_21_3: Stdlib Cache for Local Loops
- Scope:
  - Cache stdlib compilation artifacts for repeated check/test cycles.
- Definition of done:
  - Repeated local runs avoid redundant stdlib recompilation.

## Exit Gate
- Multi-module builds are deterministic, cycle-safe, and faster in local iteration.
