# Phase 37: Package Management

> Note: Needs more planning before execution (scope boundaries, dependency model, and acceptance gates are still draft-level).

## Objective
Establish package management workflows as a dedicated post-hardening phase.

## Depends on
- Phase 36

## Milestones

### milestone_37_1: Package Management
- Scope:
  - Dependency declaration, lockfile semantics, resolution workflow.
- Definition of done:
  - Package workflows are deterministic and reproducible.

## Quality Contract
- Entry criteria: Phase 36 is completed and tooling contracts are stable.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Package management workflows are stable enough for broader ecosystem usage.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_37_1` (Package Management): validation goals cover: Dependency declaration, lockfile semantics, resolution workflow. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Package management workflows are stable enough for broader ecosystem usage.

## Exit Gate
- Package management workflows are stable enough for broader ecosystem usage.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
