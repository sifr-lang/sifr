# Phase 29: Performance Benchmarking and Budgets

## Objective
Establish and enforce compiler-focused performance budgets (compile-time, compiler memory, and check/build latency).

## Depends on
- Phase 28

## Milestones

### milestone_29_1: Baseline Benchmark Suite
- Scope:
  - Define compiler benchmark suites for `check`, `build`, and incremental local loops.
- Definition of done:
  - Baselines are versioned and reproducible locally.

### milestone_29_2: Budget and Threshold Policy
- Scope:
  - Set compiler regression thresholds and waiver process.
- Definition of done:
  - Performance budget policy is documented and testable.

### milestone_29_3: Enforcement Integration
- Scope:
  - Add local and CI gates for benchmark regressions.
- Definition of done:
  - Regressions fail gates unless approved waiver exists.

## Exit Gate
- Performance regressions are systematically detected and controlled.
