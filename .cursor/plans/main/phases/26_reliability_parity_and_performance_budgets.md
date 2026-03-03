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
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 26 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 26 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Reliability claims are backed by stdlib parity evidence with explicit parity governance.
