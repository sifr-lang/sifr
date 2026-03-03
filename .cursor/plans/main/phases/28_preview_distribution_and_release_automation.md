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

## Exit Gate
- Preview release lifecycle works reliably without enabling stable GA promotion.
