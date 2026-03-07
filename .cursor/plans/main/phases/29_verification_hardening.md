# Phase 29: Verification Hardening

## Objective
Scale validation breadth and depth so reliability claims are continuously provable.

## Depends on
- Phase 28

## Milestones

### milestone_29_1: Regression Matrix Expansion
- Scope:
  - Ensure each fixed bug has dedicated regression coverage.
  - Expand cross-phase regression suites.
- Definition of done:
  - Regression matrix maps directly to resolved findings.

### milestone_29_2: Fuzz and Property Scale-Out
- Scope:
  - Move from smoke fuzz/property checks to sustained coverage.
  - Track and triage fuzz findings systematically.
- Definition of done:
  - Fuzz/property suite is part of standard hardening gates.

### milestone_29_3: Real-World E2E Parallel Gate
- Scope:
  - Validate representative multi-module real-world projects end-to-end (`check/build/run/test`).
- Definition of done:
  - E2E suites pass deterministically in local parallel mode.

## Quality Contract
- Entry criteria: Phase 28 is completed and decimal numeric semantics contract is in place.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Reliability hardening is broad, deterministic, and locally enforceable.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_29_1` (Regression Matrix Expansion): validation goals cover: Ensure each fixed bug has dedicated regression coverage; Expand cross-phase regression suites. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_29_2` (Fuzz and Property Scale-Out): validation goals cover: Move from smoke fuzz/property checks to sustained coverage; Track and triage fuzz findings systematically. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_29_3` (Real-World E2E Parallel Gate): validation goals cover: Validate representative multi-module real-world projects end-to-end (`check/build/run/test`). Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Reliability hardening is broad, deterministic, and locally enforceable.

## Exit Gate
- Reliability hardening is broad, deterministic, and locally enforceable.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
