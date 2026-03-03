# Phase 25: Verification Hardening

## Objective
Scale validation breadth and depth so reliability claims are continuously provable.

## Depends on
- Phase 24

## Milestones

### milestone_25_1: Regression Matrix Expansion
- Scope:
  - Ensure each fixed bug has dedicated regression coverage.
  - Expand cross-phase regression suites.
- Definition of done:
  - Regression matrix maps directly to resolved findings.

### milestone_25_2: Fuzz and Property Scale-Out
- Scope:
  - Move from smoke fuzz/property checks to sustained coverage.
  - Track and triage fuzz findings systematically.
- Definition of done:
  - Fuzz/property suite is part of standard hardening gates.

### milestone_25_3: Real-World E2E Parallel Gate
- Scope:
  - Validate representative multi-module real-world projects end-to-end (`check/build/run/test`).
- Definition of done:
  - E2E suites pass deterministically in local parallel mode.

## Quality Contract
- Entry criteria: Phase 24 is completed and diagnostic stability contract is in place.
- Exit criteria: Reliability hardening is broad, deterministic, and locally enforceable.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 25 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 25 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Reliability hardening is broad, deterministic, and locally enforceable.
