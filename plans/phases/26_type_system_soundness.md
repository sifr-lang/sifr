# Phase 26: Type-System Soundness

## Objective
Close known type-soundness holes before stable release promotion.

## Depends on
- Phase 25

## Milestones

### milestone_26_1: TypeVar Constraint Enforcement
- Scope:
  - Replace permissive TypeVar assignability with bound/constraint validation.
- Definition of done:
  - Generic code is type-checked against declared constraints.

### milestone_26_2: Inheritance and Variance Corrections
- Scope:
  - Implement multi-level inheritance assignability.
  - Remove special-case inheritance hacks.
  - Enforce invariance on mutable collections.
- Definition of done:
  - Subtyping and mutable variance behavior are sound.

### milestone_26_3: Optional Arithmetic Soundness
- Scope:
  - Eliminate unsound optional arithmetic acceptance in type checking.
- Definition of done:
  - Optional arithmetic requires explicit safe handling.

### milestone_26_4: Protocol-Bound Strictness Closure
- Scope:
  - Remove permissive protocol-bound acceptance shortcuts in generic/type-bound checks.
  - Enforce explicit protocol conformance checks instead of default-allow behavior.
- Definition of done:
  - Protocol-bound validation is strict, explicit, and regression-covered.

## Quality Contract
- Entry criteria: Phase 25 is completed and traversal/control-flow behavior is stable.
- Exit criteria: Critical type-system soundness issues are resolved and regression-covered.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_26_1` (TypeVar Constraint Enforcement): validation goals cover: Replace permissive TypeVar assignability with bound/constraint validation. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_26_2` (Inheritance and Variance Corrections): validation goals cover: Implement multi-level inheritance assignability; Remove special-case inheritance hacks; Enforce invariance on mutable collections. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_26_3` (Optional Arithmetic Soundness): validation goals cover: Eliminate unsound optional arithmetic acceptance in type checking. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_26_4` (Protocol-Bound Strictness Closure): validation goals cover: Remove permissive protocol-bound acceptance shortcuts in generic/type-bound checks; Enforce explicit protocol conformance checks instead of default-allow behavior. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Critical type-system soundness issues are resolved and regression-covered.

## Exit Gate
- Critical type-system soundness issues are resolved and regression-covered.

## Post-Phase Follow-Ups
- Multiple TypeVar bounds/intersection bounds (`T: A & B`) are not part of Phase 26 scope and remain an enhancement item. Track in `plans/issues/archive/phase26-followup-multiple-bounds-gap.md`.
