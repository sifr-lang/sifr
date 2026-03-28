# Phase 19: Module Graph Safety, Determinism, and Cache

## Objective
Make multi-module compilation dependency-safe, deterministic, and efficient for repeated local loops.

Status: completed (2026-03-05; milestone_19_1 PR #834, milestone_19_2 PR #835, milestone_19_3 PR #836)

## Depends on
- Phase 18

## Milestones

### milestone_19_1: Dependency-Safe Module Ordering
status: done (2026-03-05, PR #834)
- Scope:
  - Introduce topological ordering for module compilation.
  - Add cycle diagnostics with actionable context.
- Definition of done:
  - Module compile order is dependency-correct and cycle-safe.
- Evidence:
  - `collect_project_hir_modules` now lowers project modules using an explicit dependency graph and deterministic topological compile order.
  - Dependency graph extraction intentionally follows Phase 18 import-form semantics by including only project-local `from <module> import ...` and level-1 relative imports (`from .module import ...`), while unsupported relative depths remain excluded from local graph edges.
  - Cycles are detected before lowering and reported as actionable diagnostics with the import chain path (for example, `a -> b -> a`).
  - Regression coverage added for dependency-safe ordering and cycle diagnostics in `crates/sifr_driver/src/lib.rs`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/module_ordering/main.sifr`.

### milestone_19_2: Deterministic Assembly
status: done (2026-03-05, PR #835)
- Scope:
  - Remove nondeterministic HashMap-order behavior from module assembly/output.
- Definition of done:
  - Repeated builds produce stable module output order.
- Evidence:
  - Project assembly now emits `main.rs` module declarations using deterministic dependency-safe compile order instead of `HashMap` key iteration.
  - Non-main module file emission now follows deterministic ordered module names, preventing random output ordering drift.
  - Regression test `test_assemble_project_main_rs_is_deterministic_against_hashmap_order` locks deterministic output against insertion-order variation.
  - Milestone demo: `cargo run -q -p sifr -- run demos/module_assembly/main.sifr`.

### milestone_19_3: Stdlib Cache for Local Loops
status: done (2026-03-05, PR #836)
- Scope:
  - Cache stdlib compilation artifacts for repeated check/test cycles.
- Definition of done:
  - Repeated local runs avoid redundant stdlib recompilation.
- Evidence:
  - Driver stdlib compilation now routes through a single process-local `OnceLock` cache (`get_or_init_stdlib_cache`) so repeated compilation flows reuse compiled stdlib artifacts within the same run.
  - Added cache regressions in `crates/sifr_driver/src/lib.rs`:
    - `test_get_or_init_stdlib_cache_reuses_successful_compilation`
    - `test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild`
  - Milestone demo: `cargo run -q -p sifr -- run demos/local_imports/main.sifr`.

## Quality Contract
- Entry criteria: Phase 18 is completed and project-mode semantics are stable.
- Exit criteria: Multi-module builds are deterministic, cycle-safe, and faster in local iteration.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
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
