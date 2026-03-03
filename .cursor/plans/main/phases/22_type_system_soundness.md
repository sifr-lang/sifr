# Phase 22: Type-System Soundness

## Objective
Close known type-soundness holes before stable release promotion.

## Depends on
- Phase 21

## Milestones

### milestone_22_1: TypeVar Constraint Enforcement
- Scope:
  - Replace permissive TypeVar assignability with bound/constraint validation.
- Definition of done:
  - Generic code is type-checked against declared constraints.

### milestone_22_2: Inheritance and Variance Corrections
- Scope:
  - Implement multi-level inheritance assignability.
  - Remove special-case inheritance hacks.
  - Enforce invariance on mutable collections.
- Definition of done:
  - Subtyping and mutable variance behavior are sound.

### milestone_22_3: Optional Arithmetic Soundness
- Scope:
  - Eliminate unsound optional arithmetic acceptance in type checking.
- Definition of done:
  - Optional arithmetic requires explicit safe handling.

## Quality Contract
- Entry criteria: Phase 21 is completed and traversal/control-flow behavior is stable.
- Exit criteria: Critical type-system soundness issues are resolved and regression-covered.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 22 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 22 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Critical type-system soundness issues are resolved and regression-covered.
