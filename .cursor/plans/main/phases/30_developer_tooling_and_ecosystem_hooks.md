# Phase 30: Developer Tooling and Ecosystem Hooks

## Objective
Enable core developer tooling integration as a dedicated phase immediately after performance hardening.

## Depends on
- Phase 29

## Milestones

### milestone_30_1: Developer Tooling and Ecosystem Hooks
- Scope:
  - LSP/formatter/linter/doc hooks aligned with new phase contracts.
- Definition of done:
  - Tooling integrates with language/runtime capabilities added in prior phases.

## Quality Contract
- Entry criteria: Phase 29 is completed and performance budgets are enforced.
- Exit criteria: Tooling hooks are coherent, stable, and aligned with current phase contracts.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 30 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 30 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Tooling hooks are coherent, stable, and aligned with current phase contracts.
