# Phase 26: Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic

## Objective
Deliver production-quality diagnostics with recovery and explicit stability guarantees.

## Depends on
- Phase 25

## Milestones

### milestone_26_1: Span and Diagnostic Schema Quality
- Scope:
  - Thread precise spans through frontend/codegen errors.
  - Standardize stable diagnostic codes/categories.
- Definition of done:
  - Diagnostics include accurate source locations and stable codes.

### milestone_26_2: Bounded Multi-Error Recovery
- Scope:
  - Add parser/type-check recovery to report multiple actionable errors.
  - Control error cascades with bounded recovery policy.
- Definition of done:
  - Compiler reports multiple useful errors without crash storms.

### milestone_26_3: Stability Contract Finalization
- Scope:
  - Define documented exit codes, CLI flag stability/versioning, and diagnostic-text policy.
  - Convert remaining user-triggerable panics to diagnostics.
- Definition of done:
  - Stability policy is explicit and enforced by tests/docs.

## Exit Gate
- Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
