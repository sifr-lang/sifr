# Phase 39: Stable Channel GA Promotion and Release Governance

## Objective
Promote stable channel only after reliability/parity/performance evidence is complete and governed.

## Depends on
- Phase 38

## Milestones

### milestone_39_1: Stable Promotion Policy
- Scope:
  - Define hard preconditions for `stable` promotion from preview channels.
- Definition of done:
  - Promotion checklist is documented and mandatory.

### milestone_39_2: Rollback and Incident Governance
- Scope:
  - Define rollback triggers, owner responsibilities, and communication protocol.
- Definition of done:
  - Rollback path is tested and documented.

### milestone_39_3: Release Sign-off Workflow
- Scope:
  - Enforce formal release sign-off and artifact provenance checks.
- Definition of done:
  - Stable releases require auditable approvals and pass governance gates.

## Quality Contract
- Entry criteria: Phase 38 is completed and release-facing documentation is canonical.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Stable GA promotion is policy-driven, auditable, and reversible.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_39_1` (Stable Promotion Policy): validation goals cover: Define hard preconditions for `stable` promotion from preview channels. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_39_2` (Rollback and Incident Governance): validation goals cover: Define rollback triggers, owner responsibilities, and communication protocol. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_39_3` (Release Sign-off Workflow): validation goals cover: Enforce formal release sign-off and artifact provenance checks. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Stable GA promotion is policy-driven, auditable, and reversible.

## Exit Gate
- Stable GA promotion is policy-driven, auditable, and reversible.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
