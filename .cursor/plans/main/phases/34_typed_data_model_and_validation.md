# Phase 34: Typed Data Model and Validation (Pydantic-Parity Track)

> Note: Needs more planning before execution (which pydantic subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Introduce a dedicated typed model layer with validation semantics, stable error behavior, and explicit pydantic-parity boundaries.

## Depends on
- Phase 33

## Milestones

### milestone_34_1: Typed Model Core
- Scope:
  - Class-to-model mapping with field metadata and defaults.
  - Optional/union/list/dict model handling.
  - Baseline serialization/deserialization (`dumps`/`loads`).
- Definition of done:
  - Typed model core is usable independent of async/web runtime concerns.

### milestone_34_2: Validation Engine
- Scope:
  - Strict vs coercion modes.
  - Nested model validation and collection constraints.
  - Field/model validator hooks with deterministic order.
- Definition of done:
  - Validation behavior is deterministic, testable, and documented.

### milestone_34_3: Error Model and Diagnostics Contract
- Scope:
  - Structured validation errors (path, code, message, context).
  - Stable parse/validation error-code contract.
- Definition of done:
  - Validation failures produce stable, structured, and actionable errors.

### milestone_34_4: Parity and Compatibility Matrix
- Scope:
  - Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`.
  - Port representative pydantic behavior tests.
- Definition of done:
  - Target pydantic subset is explicit and regression-locked.

## Quality Contract
- Entry criteria: Phase 33 is completed and release governance is active.
- Exit criteria: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 34 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 34 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
