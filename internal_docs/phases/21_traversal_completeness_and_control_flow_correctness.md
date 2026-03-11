# Phase 21: Traversal Completeness and Control-Flow Correctness

## Objective
Guarantee walkers and control-flow analyses cover all supported constructs correctly.

## Depends on
- Phase 20

## Milestones

### milestone_21_1: Canonical Walker Coverage
- Scope:
  - Standardize recursive traversal across statement/expression variants.
  - Remove partial traversal blind spots.
- Definition of done:
  - Traversal completeness matrix is satisfied for supported nodes.

### milestone_21_2: `while ... else` End-to-End Support
- Scope:
  - Implement intended Python-like `while ... else` semantics through HIR and codegen.
- Definition of done:
  - `while ... else` behavior matches language intent with regression tests.

### milestone_21_3: Yield and Exception-Path Coverage
- Scope:
  - Fix generator/yield detection across nested constructs.
  - Ensure try/except analysis includes loop-else and other missed paths.
- Definition of done:
  - No known missed traversal paths in generator/error analysis.

## Quality Contract
- Entry criteria: Phase 20 is completed and HIR decomposition guardrails are active.
- Exit criteria: Control-flow lowering/analysis is complete for supported syntax and semantics.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_21_1` (Canonical Walker Coverage): validation goals cover: Standardize recursive traversal across statement/expression variants; Remove partial traversal blind spots. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_21_2` (`while ... else` End-to-End Support): validation goals cover: Implement intended Python-like `while ... else` semantics through HIR and codegen. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_21_3` (Yield and Exception-Path Coverage): validation goals cover: Fix generator/yield detection across nested constructs; Ensure try/except analysis includes loop-else and other missed paths. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Control-flow lowering/analysis is complete for supported syntax and semantics.

## Exit Gate
- Control-flow lowering/analysis is complete for supported syntax and semantics.
