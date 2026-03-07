# Phase 33: Preview Distribution and Release Automation (`alpha`/`beta`)

## Objective
Ship preview channels early for adoption while keeping stable GA promotion gated for later phases.

## Depends on
- Phase 32

## Milestones

### milestone_33_1: Installer and Channel Resolution
- Scope:
  - Implement install entrypoint (`curl -fsSL https://sifr.sh/install | bash`).
  - Support `SIFR_CHANNEL`/`--channel` and explicit `--version` pinning.
- Definition of done:
  - Installer resolves `alpha`/`beta`/`stable` channel metadata correctly.

### milestone_33_2: Artifact and Manifest Pipeline
- Scope:
  - Publish multi-platform artifacts with checksums/signatures.
  - Maintain channel manifest pointers.
- Definition of done:
  - Installer validates checksums and installs matching artifacts.

### milestone_33_3: Agentic Preview Release Command
- Scope:
  - Add `/create-new-version` workflow for preview release automation.
  - Support dry-run and real-run paths.
- Definition of done:
  - Preview release flow is repeatable end-to-end for `alpha`/`beta`.

## Quality Contract
- Entry criteria: Phase 32 is completed and async/runtime ecosystem primitives are stable.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Preview release lifecycle works reliably without enabling stable GA promotion.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_33_1` (Installer and Channel Resolution): validation goals cover: Implement install entrypoint (`curl -fsSL https://sifr.sh/install | bash`); Support `SIFR_CHANNEL`/`--channel` and explicit `--version` pinning. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_33_2` (Artifact and Manifest Pipeline): validation goals cover: Publish multi-platform artifacts with checksums/signatures; Maintain channel manifest pointers. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_33_3` (Agentic Preview Release Command): validation goals cover: Add `/create-new-version` workflow for preview release automation; Support dry-run and real-run paths. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Preview release lifecycle works reliably without enabling stable GA promotion.

## Exit Gate
- Preview release lifecycle works reliably without enabling stable GA promotion.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
