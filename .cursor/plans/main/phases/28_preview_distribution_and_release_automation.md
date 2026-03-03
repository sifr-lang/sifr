# Phase 28: Preview Distribution and Release Automation (`alpha`/`beta`)

## Objective
Ship preview channels early for adoption while keeping stable GA promotion gated for later phases.

## Depends on
- Phase 27

## Milestones

### milestone_28_1: Installer and Channel Resolution
- Scope:
  - Implement install entrypoint (`curl -fsSL https://sifr.sh/install | bash`).
  - Support `SIFR_CHANNEL`/`--channel` and explicit `--version` pinning.
- Definition of done:
  - Installer resolves `alpha`/`beta`/`stable` channel metadata correctly.

### milestone_28_2: Artifact and Manifest Pipeline
- Scope:
  - Publish multi-platform artifacts with checksums/signatures.
  - Maintain channel manifest pointers.
- Definition of done:
  - Installer validates checksums and installs matching artifacts.

### milestone_28_3: Agentic Preview Release Command
- Scope:
  - Add `/create-new-version` workflow for preview release automation.
  - Support dry-run and real-run paths.
- Definition of done:
  - Preview release flow is repeatable end-to-end for `alpha`/`beta`.

## Quality Contract
- Entry criteria: Phase 27 is completed and async/runtime ecosystem primitives are stable.
- Exit criteria: Preview release lifecycle works reliably without enabling stable GA promotion.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Mandatory local validation commands:
  - `python scripts/phase_contract_gate_check.py --phase 28 --check entry`
  - `python scripts/phase_contract_gate_check.py --phase 28 --check exit`
  - `python scripts/validate_phase_quality_contracts_15_35.py`
  - `./scripts/run_all_tests.sh`

## Exit Gate
- Preview release lifecycle works reliably without enabling stable GA promotion.
