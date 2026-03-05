# Phase 40: Interoperability

> Note: Needs more planning before execution (interop model, safety contract depth, and release gating are still draft-level).

## Objective
Deliver interoperability capabilities after typed model, package, tooling, and web foundations are stable.

## Depends on
- Phase 39

## Milestones

### milestone_40_1: Interoperability (FFI)
- Scope:
  - Rust/C FFI boundary model, safety constraints, and diagnostics.
- Definition of done:
  - Interop workflows are documented, test-covered, and safe-gated.

## Quality Contract
- Entry criteria: Phase 39 is completed and existing quality gates remain green.
- Exit criteria: Interoperability is stable and governed by existing quality gates.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_40_1` (Interoperability (FFI)): validation goals cover: Rust/C FFI boundary model, safety constraints, and diagnostics. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Interoperability is stable and governed by existing quality gates.

## Exit Gate
- Interoperability is stable and governed by existing quality gates.
