# Phase 26: Stdlib Parity (Behavior + Complexity)

## Objective
Prove module-by-module stdlib parity against Python behavior and complexity expectations.

## Depends on
- Phase 25

## Milestones

### milestone_26_1: Python Test Porting by Module
- Scope:
  - Port upstream Python stdlib tests module-by-module (adapted for Sifr syntax/runtime).
- Definition of done:
  - Each targeted stdlib module has a maintained parity test suite.

### milestone_26_2: Behavioral Parity Classification
- Scope:
  - Classify each test/API as `parity`, `intentional-diff`, or `unsupported`.
  - Require explicit rationale for non-parity cases.
- Definition of done:
  - Per-module parity matrix is published and current.

### milestone_26_3: Complexity and Resource Audit vs CPython
- Scope:
  - Run scaling benchmarks for exposed stdlib APIs (time + memory).
  - Compare asymptotic class and constant-factor deltas against CPython baselines.
- Definition of done:
  - Same-or-better asymptotic class is verified.
  - Constant-factor regressions are within thresholds or linked waivers.

## Exit Gate
- Stdlib parity and complexity posture are measured, documented, and gated.
