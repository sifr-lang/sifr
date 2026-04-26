# Phase 23: Project Graph and Isolation Correctness

## Objective
Make project and test compilation graph-correct, deterministic, and isolated per invocation so results do not depend on sibling files or shared temp paths.

## Depends on
- Phase 22

## Technical Context
- Project and test graph discovery/resolution behavior is orchestrated in `crates/sifr_driver`.
- CLI run/build flow and temp workspace setup are wired through `crates/sifr/src` and `crates/sifr_driver`.
- Import-closure in this phase means:
  - Start from declared entry/test roots.
  - Recursively resolve reachable local and stdlib module imports according to Sifr import semantics.
  - Exclude unrelated sibling files that are not in the resolved import closure.
- Invocation isolation in this phase means each command invocation uses an independent workspace path and cannot conflict with parallel invocations.

## Milestones

### milestone_23_1: Import-Closure Discovery
- Scope:
  - Replace directory-wide sibling `.sifr` discovery with import-closure graph discovery.
  - Ensure only reachable modules from the entrypoint/test roots are parsed/lowered.
- Definition of done:
  - Unrelated sibling files no longer affect `build`, `run`, or `test` outcomes.

### milestone_23_2: Deterministic Module Graph and Cycle Diagnostics
- Scope:
  - Build a deterministic module graph resolution order independent of map iteration order.
  - Add explicit cycle diagnostics with stable, reproducible reporting.
- Definition of done:
  - Module resolution and cycle failures are deterministic across repeated runs.

### milestone_23_3: Project/Test Discovery Parity Contract
- Scope:
  - Align graph discovery behavior between project build and test runner paths.
  - Enforce one shared discovery contract for main modules, support modules, and test modules.
- Definition of done:
  - Project and test paths produce consistent graph membership decisions for equivalent imports.

### milestone_23_4: Invocation-Scoped Temp Workspace Isolation
- Scope:
  - Replace fixed shared temp directories with per-invocation isolated workspaces.
  - Ensure parallel local runs (`run`/`test`) cannot overwrite each other’s artifacts.
- Definition of done:
  - Parallel invocations are race-free and artifact-isolated by design.

### milestone_23_5: Graph and Isolation Regression Matrix
- Scope:
  - Add regression suites covering: unrelated sibling files, import closure correctness, deterministic ordering, cycle errors, and parallel invocation isolation.
  - Include corpus fixtures for both single-file and multi-file project layouts.
- Definition of done:
  - Graph/discovery/isolation regressions are automatically caught before merge.

## Execution Progress
- 2026-03-06: `milestone_23_1` completed (PR [#863](https://github.com/sifr-lang/sifr/pull/863)).
- 2026-03-06: `milestone_23_2` completed (PR [#865](https://github.com/sifr-lang/sifr/pull/865)).
- 2026-03-06: `milestone_23_3` completed (PR [#867](https://github.com/sifr-lang/sifr/pull/867)).
- 2026-03-06: `milestone_23_4` completed (PR [#869](https://github.com/sifr-lang/sifr/pull/869)).
- 2026-03-06: `milestone_23_5` completed (PR [#871](https://github.com/sifr-lang/sifr/pull/871)).

## Quality Contract
- Entry criteria: Phase 22 is completed and frontend mode parity contract is in place.
- Exit criteria: Project graph discovery is import-closure based, deterministic, and invocation-isolated with full regression coverage.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_23_1` (Import-Closure Discovery): validation goals cover: Replace directory-wide sibling `.sifr` discovery with import-closure graph discovery; Ensure only reachable modules from the entrypoint/test roots are parsed/lowered. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_23_2` (Deterministic Module Graph and Cycle Diagnostics): validation goals cover: Build a deterministic module graph resolution order independent of map iteration order; Add explicit cycle diagnostics with stable, reproducible reporting. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_23_3` (Project/Test Discovery Parity Contract): validation goals cover: Align graph discovery behavior between project build and test runner paths; Enforce one shared discovery contract for main modules, support modules, and test modules. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_23_4` (Invocation-Scoped Temp Workspace Isolation): validation goals cover: Replace fixed shared temp directories with per-invocation isolated workspaces; Ensure parallel local runs (`run`/`test`) cannot overwrite each other’s artifacts. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_23_5` (Graph and Isolation Regression Matrix): validation goals cover: Add regression suites covering unrelated sibling files, import closure correctness, deterministic ordering, cycle errors, and parallel invocation isolation; Include corpus fixtures for both single-file and multi-file project layouts. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Project graph discovery is import-closure based, deterministic, and invocation-isolated with full regression coverage.

## Exit Gate
- Project graph discovery is import-closure based, deterministic, and invocation-isolated with full regression coverage.
