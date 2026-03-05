# Phase 35: Performance Benchmarking and Budgets

## Objective
Establish and enforce compiler-focused performance budgets (compile-time, compiler memory, and check/build latency), including incremental/query architecture and cache-invalidation contracts.

## Depends on
- Phase 34

## Milestones

### milestone_35_1: Baseline Benchmark Suite
- Scope:
  - Define compiler benchmark suites for `check`, `build`, and incremental local loops.
- Definition of done:
  - Baselines are versioned and reproducible locally.

### milestone_35_2: Budget and Threshold Policy
- Scope:
  - Set compiler regression thresholds and waiver process.
- Definition of done:
  - Performance budget policy is documented and testable.

### milestone_35_3: Enforcement Integration
- Scope:
  - Add local and CI gates for benchmark regressions.
- Definition of done:
  - Regressions fail gates unless approved waiver exists.

### milestone_35_4: Incremental Query Architecture and Cache Contracts
- Scope:
  - Define dependency-tracked query/cache architecture at module granularity.
  - Define deterministic invalidation rules and cache-consistency guarantees for local loops.
- Definition of done:
  - Incremental/query design and cache contracts are explicit, deterministic, and regression-covered.

## Quality Contract
- Entry criteria: Phase 34 is completed and generated-code quality gates are enforced.
- Exit criteria: Performance regressions are systematically detected and controlled.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_35_1` (Baseline Benchmark Suite): validation goals cover: Define compiler benchmark suites for `check`, `build`, and incremental local loops. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_2` (Budget and Threshold Policy): validation goals cover: Set compiler regression thresholds and waiver process. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_3` (Enforcement Integration): validation goals cover: Add local and CI gates for benchmark regressions. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_4` (Incremental Query Architecture and Cache Contracts): validation goals cover: Define dependency-tracked query/cache architecture at module granularity; Define deterministic invalidation rules and cache-consistency guarantees for local loops. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Performance regressions are systematically detected and controlled.

## Exit Gate
- Performance regressions are systematically detected and controlled.
