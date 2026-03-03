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

## Quality Contract
- Entry criteria: Phase 28 is completed and preview artifacts are reproducible.
- Exit criteria: Performance regressions are systematically detected and controlled.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 29 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 29 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Performance regressions are systematically detected and controlled.
