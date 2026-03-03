# Phase 18: Project and CLI Semantics Correctness

## Objective
Make CLI behavior predictable for single-file and multi-file workflows.

## Depends on
- Phase 17

## Milestones

### milestone_18_1: Run/Build Semantics Alignment
- Scope:
  - Align project detection and compilation scope between `run` and `build`.
- Definition of done:
  - Equivalent project inputs yield equivalent resolution behavior.

### milestone_18_2: Auto-Detection Rule Tightening
- Scope:
  - Replace over-aggressive auto project mode with explicit, documented rules.
- Definition of done:
  - Nearby scratch files do not unexpectedly break single-file runs.

### milestone_18_3: CLI Contract and Regression Suite
- Scope:
  - Document stable CLI semantics and edge cases.
  - Add regression tests for command-mode behavior.
- Definition of done:
  - CLI behavior contract exists and is regression-protected.

## Quality Contract
- Entry criteria: Phase 17 is completed and import/external behavior is stable.
- Exit criteria: CLI project semantics are stable, documented, and test-covered.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 18 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 18 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- CLI project semantics are stable, documented, and test-covered.
