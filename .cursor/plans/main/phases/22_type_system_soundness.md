# Phase 24: Type-System Soundness

## Objective
Close known type-soundness holes before stable release promotion.

## Depends on
- Phase 23

## Milestones

### milestone_24_1: TypeVar Constraint Enforcement
- Scope:
  - Replace permissive TypeVar assignability with bound/constraint validation.
- Definition of done:
  - Generic code is type-checked against declared constraints.

### milestone_24_2: Inheritance and Variance Corrections
- Scope:
  - Implement multi-level inheritance assignability.
  - Remove special-case inheritance hacks.
  - Enforce invariance on mutable collections.
- Definition of done:
  - Subtyping and mutable variance behavior are sound.

### milestone_24_3: Optional Arithmetic Soundness
- Scope:
  - Eliminate unsound optional arithmetic acceptance in type checking.
- Definition of done:
  - Optional arithmetic requires explicit safe handling.

## Exit Gate
- Critical type-system soundness issues are resolved and regression-covered.
