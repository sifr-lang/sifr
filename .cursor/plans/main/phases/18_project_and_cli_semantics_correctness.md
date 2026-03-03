# Phase 18: Project and CLI Semantics Correctness

## Objective
Make CLI behavior predictable for single-file and multi-file workflows.

## Depends on
- Phase 17

## Milestones

### milestone_18_1: Run/Build Semantics Alignment
- Scope:
  - Align project detection and compilation scope between `run` and `build`.
- Definition of done:
  - Equivalent project inputs yield equivalent resolution behavior.

### milestone_18_2: Auto-Detection Rule Tightening
- Scope:
  - Replace over-aggressive auto project mode with explicit, documented rules.
- Definition of done:
  - Nearby scratch files do not unexpectedly break single-file runs.

### milestone_18_3: CLI Contract and Regression Suite
- Scope:
  - Document stable CLI semantics and edge cases.
  - Add regression tests for command-mode behavior.
- Definition of done:
  - CLI behavior contract exists and is regression-protected.

## Quality Contract
- Entry criteria: Phase 17 is completed and import/external behavior is stable.
- Exit criteria: CLI project semantics are stable, documented, and test-covered.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_18_1` (Run/Build Semantics Alignment): validation goals cover: Align project detection and compilation scope between `run` and `build`. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_2` (Auto-Detection Rule Tightening): validation goals cover: Replace over-aggressive auto project mode with explicit, documented rules. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_18_3` (CLI Contract and Regression Suite): validation goals cover: Document stable CLI semantics and edge cases; Add regression tests for command-mode behavior. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: CLI project semantics are stable, documented, and test-covered.

## Exit Gate
- CLI project semantics are stable, documented, and test-covered.
