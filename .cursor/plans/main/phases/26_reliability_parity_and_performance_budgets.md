# Phase 26: Reliability Parity and Performance Budgets

## Objective
Close the reliability track by proving stdlib parity and enforcing measurable performance budgets before feature expansion.

## Depends on
- Phase 25

## Milestones

### milestone_26_1: Stdlib Behavioral Parity
- Scope:
  - Port and maintain module-by-module parity tests against Python behavior.
  - Classify outcomes as `parity`, `intentional-diff`, or `unsupported` with rationale.
- Definition of done:
  - Targeted stdlib modules have parity suites and an up-to-date parity matrix.

### milestone_26_2: Complexity and Resource Parity
- Scope:
  - Run scaling benchmarks (time and memory) for exposed stdlib APIs.
  - Validate asymptotic class parity against CPython and track constant-factor deltas.
- Definition of done:
  - Asymptotic parity is verified; constant-factor regressions are budgeted or waived explicitly.

### milestone_26_3: Global Performance Budget Enforcement
- Scope:
  - Define compile-time, memory, and runtime benchmark suites and thresholds.
  - Add local/CI enforcement gates with waiver protocol.
- Definition of done:
  - Performance regressions are caught automatically unless approved waiver exists.

## Exit Gate
- Reliability claims are backed by stdlib parity evidence and enforced performance budgets.
