# Phase 36: Developer Tooling and Ecosystem Hooks

## Objective
Prevent tooling/compiler split-brain by enforcing a single frontend API contract and parity matrix across compiler CLI and future tooling surfaces.

## Depends on
- Phase 35

## Milestones

### milestone_36_1: Shared Frontend API Contract
- Scope:
  - Define one canonical frontend API for parse/lower/type-check/diagnostics consumed by compiler and tooling.
  - Disallow semantics reimplementation in tool-specific paths.
- Definition of done:
  - Tooling integration points consume the same frontend contracts as compiler modes.

### milestone_36_2: Tooling/CLI Parity Matrix
- Scope:
  - Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs.
  - Cover diagnostics codes, spans, and type-check outcomes.
- Definition of done:
  - Divergence between tooling and compiler behavior is automatically detected before merge.

## Quality Contract
- Entry criteria: Phase 35 is completed and compiler performance/query contracts are enforced.
- Exit criteria: Tooling integration is split-brain-resistant and regression-covered against compiler behavior.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_36_1` (Shared Frontend API Contract): validation goals cover: Define one canonical frontend API for parse/lower/type-check/diagnostics consumed by compiler and tooling; Disallow semantics reimplementation in tool-specific paths. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_36_2` (Tooling/CLI Parity Matrix): validation goals cover: Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs; Cover diagnostics codes, spans, and type-check outcomes. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Tooling integration is split-brain-resistant and regression-covered against compiler behavior.

## Exit Gate
- Tooling integration is split-brain-resistant and regression-covered against compiler behavior.
