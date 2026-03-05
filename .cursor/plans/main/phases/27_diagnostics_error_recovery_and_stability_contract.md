# Phase 27: Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic

## Objective
Deliver production-quality diagnostics with recovery and explicit stability guarantees.

## Depends on
- Phase 26

## Milestones

### milestone_27_4: Span and Diagnostic Schema Quality
- Scope:
  - Thread precise spans through frontend/codegen errors.
  - Standardize stable diagnostic codes/categories.
- Definition of done:
  - Diagnostics include accurate source locations and stable codes.

### milestone_27_5: Bounded Multi-Error Recovery
- Scope:
  - Add parser/type-check recovery to report multiple actionable errors.
  - Control error cascades with bounded recovery policy.
- Definition of done:
  - Compiler reports multiple useful errors without crash storms.

### milestone_27_6: Stability Contract Finalization
- Scope:
  - Define documented exit codes, CLI flag stability/versioning, and diagnostic-text policy.
  - Convert remaining user-triggerable panics to diagnostics.
- Definition of done:
  - Stability policy is explicit and enforced by tests/docs.

## Quality Contract
- Entry criteria: Phase 26 is completed and runtime-safe codegen invariants are active.
- Exit criteria: Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_27_4` (Span and Diagnostic Schema Quality): validation goals cover: Thread precise spans through frontend/codegen errors; Standardize stable diagnostic codes/categories. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_5` (Bounded Multi-Error Recovery): validation goals cover: Add parser/type-check recovery to report multiple actionable errors; Control error cascades with bounded recovery policy. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_6` (Stability Contract Finalization): validation goals cover: Define documented exit codes, CLI flag stability/versioning, and diagnostic-text policy; Convert remaining user-triggerable panics to diagnostics. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.

## Exit Gate
- Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
