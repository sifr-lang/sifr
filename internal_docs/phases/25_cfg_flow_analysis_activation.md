# Phase 25: CFG/Flow Analysis Activation

## Objective
Activate CFG-backed control-flow analysis as the canonical source for correctness-critical flow facts that are too fragile for pure tree-walk heuristics.

## Depends on
- Phase 24

## Technical Context
- Existing CFG implementation is defined in `crates/sifr_lowering/src/cfg.rs` and exported via `crates/sifr_lowering/src/lib.rs`.
- Current gap: CFG exists as a module but is not yet the canonical control-flow truth source used across active frontend/codegen decisions.
- Activation target for this phase:
  - Integrate CFG construction/consumption into normal analysis flow after HIR lowering contracts.
  - Route correctness-critical flow queries to CFG-backed analysis rather than local tree-walk heuristics.
- CFG invariants required by this phase include:
  - stable block identity and deterministic ordering,
  - edge correctness and terminator completeness,
  - deterministic query results for equivalent inputs.

## Milestones

### milestone_25_1: CFG Integration Contract
- Scope:
  - Wire the existing CFG subsystem into the active compiler analysis flow instead of leaving it as an unused side module.
  - Define CFG construction entrypoints, ownership boundaries, and pass inputs/outputs.
- Definition of done:
  - CFG is generated and consumed in the canonical analysis path for selected flow queries.

### milestone_25_2: CFG Validity Invariants
- Scope:
  - Define and enforce CFG invariants (block identity, edge correctness, terminator completeness, and deterministic block ordering).
  - Add invariant checks that fail fast in compiler-internal validation, not user runtime paths.
- Definition of done:
  - CFG construction is deterministic and invariant-checked across repeated runs.

### milestone_25_3: Canonical Flow Truth Queries
- Scope:
  - Implement CFG-backed queries for correctness-critical flow facts (at minimum: reachability and always-exits behavior).
  - Replace tree-walk fallback logic for these queries where CFG-based truth is required.
- Definition of done:
  - Reachability and always-exits analysis use one CFG-backed query path.

### milestone_25_4: Diagnostics and Consumer Integration
- Scope:
  - Integrate CFG query results into lowering/codegen decision points and diagnostics generation.
  - Ensure control-flow diagnostics remain stable and deterministic when switching from heuristic analysis to CFG truth.
- Definition of done:
  - Affected compiler decisions and diagnostics consume CFG-derived flow facts consistently.

### milestone_25_5: Regression and Determinism Matrix
- Scope:
  - Add focused regression coverage for nested branching, loop exits, early return/raise paths, and unreachable blocks.
  - Add deterministic repeat-run checks for CFG graph shape and query results on the same corpus.
- Definition of done:
  - CFG/flow regressions and nondeterminism are automatically detected before merge.

## Quality Contract
- Entry criteria: Phase 24 is completed and traversal/query consolidation is stable.
- Exit criteria: CFG-backed flow analysis is active, deterministic, and canonical for correctness-critical control-flow truths.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_25_1` (CFG Integration Contract): validation goals cover: Wire the existing CFG subsystem into the active compiler analysis flow instead of leaving it as an unused side module; Define CFG construction entrypoints, ownership boundaries, and pass inputs/outputs. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_25_2` (CFG Validity Invariants): validation goals cover: Define and enforce CFG invariants (block identity, edge correctness, terminator completeness, and deterministic block ordering); Add invariant checks that fail fast in compiler-internal validation. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_25_3` (Canonical Flow Truth Queries): validation goals cover: Implement CFG-backed queries for correctness-critical flow facts (at minimum: reachability and always-exits behavior); Replace tree-walk fallback logic for these queries where CFG-based truth is required. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_25_4` (Diagnostics and Consumer Integration): validation goals cover: Integrate CFG query results into lowering/codegen decision points and diagnostics generation; Ensure control-flow diagnostics remain stable and deterministic when switching from heuristic analysis to CFG truth. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_25_5` (Regression and Determinism Matrix): validation goals cover: Add focused regression coverage for nested branching, loop exits, early return/raise paths, and unreachable blocks; Add deterministic repeat-run checks for CFG graph shape and query results on the same corpus. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: CFG-backed flow analysis is active, deterministic, and canonical for correctness-critical control-flow truths.

## Exit Gate
- CFG-backed flow analysis is active, deterministic, and canonical for correctness-critical control-flow truths.
