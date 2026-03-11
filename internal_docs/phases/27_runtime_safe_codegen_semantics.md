# Phase 27: Runtime-Safe Codegen Semantics

## Objective
Ensure generated runtime code does not encode avoidable panic behavior for normal user data flows.

## Depends on
- Phase 26

## Milestones

### milestone_27_1: Remove Data-Dependent `unwrap/expect`
- Scope:
  - Replace generated data-dependent unwrap/expect with explicit safe propagation.
- Definition of done:
  - User-facing generated paths avoid data-dependent unwrap/expect panics.
  - Emitted-code sweep over `crates/sifr/tests/e2e/pass/*.sifr` contains zero `.unwrap(` and zero `.expect(` in generated Rust.

### milestone_27_2: Indexing and Semantics Parity Fixes
- Scope:
  - Correct negative indexing and related parity semantics.
- Definition of done:
  - Indexing behavior matches language intent and tests.

### milestone_27_3: Defaults and Panic-to-Diagnostic Conversion
- Scope:
  - Preserve non-literal default argument semantics.
  - Replace remaining user-triggerable panic/assert codegen paths with diagnostics.
- Definition of done:
  - Runtime/codegen semantics are safe and diagnostic-driven.

## Quality Contract
- Entry criteria: Phase 26 is completed and type-system soundness baseline is met.
- Exit criteria: Generated code semantics are safe-by-default for supported language behavior.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_27_1` (Remove Data-Dependent `unwrap/expect`): validation goals cover: Replace generated data-dependent unwrap/expect with explicit safe propagation. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_2` (Indexing and Semantics Parity Fixes): validation goals cover: Correct negative indexing and related parity semantics. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_3` (Defaults and Panic-to-Diagnostic Conversion): validation goals cover: Preserve non-literal default argument semantics; Replace remaining user-triggerable panic/assert codegen paths with diagnostics. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Generated code semantics are safe-by-default for supported language behavior.

## Exit Gate
- Generated code semantics are safe-by-default for supported language behavior.
