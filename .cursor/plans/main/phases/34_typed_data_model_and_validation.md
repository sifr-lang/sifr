# Phase 33: Typed Data Model and Validation (Pydantic-Parity Track)

> Note: Needs more planning before execution (which pydantic subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Introduce a dedicated typed model layer with validation semantics, stable error behavior, and explicit pydantic-parity boundaries.

## Depends on
- Phase 32

## Milestones

### milestone_33_1: Typed Model Core
- Scope:
  - Class-to-model mapping with field metadata and defaults.
  - Optional/union/list/dict model handling.
  - Baseline serialization/deserialization (`dumps`/`loads`).
- Definition of done:
  - Typed model core is usable independent of async/web runtime concerns.

### milestone_33_2: Validation Engine
- Scope:
  - Strict vs coercion modes.
  - Nested model validation and collection constraints.
  - Field/model validator hooks with deterministic order.
- Definition of done:
  - Validation behavior is deterministic, testable, and documented.

### milestone_33_3: Error Model and Diagnostics Contract
- Scope:
  - Structured validation errors (path, code, message, context).
  - Stable parse/validation error-code contract.
- Definition of done:
  - Validation failures produce stable, structured, and actionable errors.

### milestone_33_4: Parity and Compatibility Matrix
- Scope:
  - Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`.
  - Port representative pydantic behavior tests.
- Definition of done:
  - Target pydantic subset is explicit and regression-locked.

## Exit Gate
- Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
