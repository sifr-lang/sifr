# Phase 26: Reliability Parity (Stdlib)

## Objective
Close the reliability track by proving stdlib behavioral and complexity parity before feature expansion.

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

### milestone_26_3: Parity Governance and Waiver Discipline
- Scope:
  - Enforce parity classification discipline (`parity`, `intentional-diff`, `unsupported`) with linked rationale.
  - Require explicit waiver records for unresolved parity gaps.
- Definition of done:
  - No unresolved parity gaps exist without documented waiver and owner.

## Quality Contract
- Entry criteria: Phase 25 is completed and verification hardening is active.
- Exit criteria: Reliability claims are backed by stdlib parity evidence with explicit parity governance.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_26_1` (Stdlib Behavioral Parity): validation goals cover: Port and maintain module-by-module parity tests against Python behavior; Classify outcomes as `parity`, `intentional-diff`, or `unsupported` with rationale. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_26_2` (Complexity and Resource Parity): validation goals cover: Run scaling benchmarks (time and memory) for exposed stdlib APIs; Validate asymptotic class parity against CPython and track constant-factor deltas. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_26_3` (Parity Governance and Waiver Discipline): validation goals cover: Enforce parity classification discipline (`parity`, `intentional-diff`, `unsupported`) with linked rationale; Require explicit waiver records for unresolved parity gaps. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Reliability claims are backed by stdlib parity evidence with explicit parity governance.

## Exit Gate
- Reliability claims are backed by stdlib parity evidence with explicit parity governance.
