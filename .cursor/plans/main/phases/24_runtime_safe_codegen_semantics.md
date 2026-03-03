# Phase 24: Runtime-Safe Codegen Semantics

## Objective
Ensure generated runtime code does not encode avoidable panic behavior for normal user data flows.

## Depends on
- Phase 23

## Milestones

### milestone_24_1: Remove Data-Dependent `unwrap/expect`
- Scope:
  - Replace generated data-dependent unwrap/expect with explicit safe propagation.
- Definition of done:
  - User-facing generated paths avoid data-dependent unwrap/expect panics.

### milestone_24_2: Indexing and Semantics Parity Fixes
- Scope:
  - Correct negative indexing and related parity semantics.
- Definition of done:
  - Indexing behavior matches language intent and tests.

### milestone_24_3: Defaults and Panic-to-Diagnostic Conversion
- Scope:
  - Preserve non-literal default argument semantics.
  - Replace remaining user-triggerable panic/assert codegen paths with diagnostics.
- Definition of done:
  - Runtime/codegen semantics are safe and diagnostic-driven.

## Exit Gate
- Generated code semantics are safe-by-default for supported language behavior.
