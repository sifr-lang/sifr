# Phase 40: Typed Data Model and Validation (Pydantic-Parity Track)

> Note: Needs more planning before execution (which pydantic subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Introduce a dedicated typed model layer with validation semantics, stable error behavior, and explicit pydantic-parity boundaries.

## Depends on
- Phase 39

## Milestones

### milestone_40_1: Typed Model Core
- Scope:
  - Class-to-model mapping with field metadata and defaults.
  - Optional/union/list/dict model handling.
  - Baseline serialization/deserialization (`dumps`/`loads`).
- Definition of done:
  - Typed model core is usable independent of async/web runtime concerns.

### milestone_40_2: Validation Engine
- Scope:
  - Strict vs coercion modes.
  - Nested model validation and collection constraints.
  - Field/model validator hooks with deterministic order.
- Definition of done:
  - Validation behavior is deterministic, testable, and documented.

### milestone_40_3: Error Model and Diagnostics Contract
- Scope:
  - Structured validation errors (path, code, message, context).
  - Stable parse/validation error-code contract.
- Definition of done:
  - Validation failures produce stable, structured, and actionable errors.

### milestone_40_4: Parity and Compatibility Matrix
- Scope:
  - Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`.
  - Port representative pydantic behavior tests.
- Definition of done:
  - Target pydantic subset is explicit and regression-locked.

## Quality Contract
- Entry criteria: Phase 39 is completed and release governance is active.
- Exit criteria: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_40_1` (Typed Model Core): validation goals cover: Class-to-model mapping with field metadata and defaults; Optional/union/list/dict model handling; Baseline serialization/deserialization (`dumps`/`loads`). Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_2` (Validation Engine): validation goals cover: Strict vs coercion modes; Nested model validation and collection constraints; Field/model validator hooks with deterministic order. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_3` (Error Model and Diagnostics Contract): validation goals cover: Structured validation errors (path, code, message, context); Stable parse/validation error-code contract. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_40_4` (Parity and Compatibility Matrix): validation goals cover: Feature matrix per capability: `parity`, `intentional-diff`, `unsupported`; Port representative pydantic behavior tests. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.

## Exit Gate
- Typed model + validation layer is stable, test-covered, and consumable by web extractors without redesign.
