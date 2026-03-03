# Phase 32: Docs and Documentation

> Note: Needs more planning before execution (doc tooling, doc structure, scope boundaries, ownership model, and acceptance gates are still draft-level).

## Objective
Establish a production-grade documentation layer (developer, user, and operations) so packaging and release governance rest on clear, versioned contracts.

## Depends on
- Phase 31

## Milestones

### milestone_32_1: Documentation Information Architecture
- Scope:
  - Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations.
  - Remove duplicated/contradictory guidance and centralize source-of-truth ownership.
- Definition of done:
  - Documentation map is approved and all core sections have canonical owners.

### milestone_32_2: Reference and Contract Documentation
- Scope:
  - Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts.
  - Document expected compatibility/stability guarantees for users and contributors.
- Definition of done:
  - Contract docs are complete, versioned, and linked from roadmap/architecture entry points.

### milestone_32_3: Documentation Quality Gates
- Scope:
  - Add local docs validation for link integrity, required sections, and drift checks against phase files.
  - Ensure docs checks are runnable in local `quick/full/stress` workflows.
- Definition of done:
  - Documentation quality gates pass locally and are mirrored in CI.

## Quality Contract
- Entry criteria: Phase 31 is completed and package workflows are deterministic.
- Exit criteria: Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Validation planning goals:
  - `milestone_32_1` (Documentation Information Architecture): validation goals cover: Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations; Remove duplicated/contradictory guidance and centralize source-of-truth ownership. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_32_2` (Reference and Contract Documentation): validation goals cover: Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts; Document expected compatibility/stability guarantees for users and contributors. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_32_3` (Documentation Quality Gates): validation goals cover: Add local docs validation for link integrity, required sections, and drift checks against phase files; Ensure docs checks are runnable in local `quick/full/stress` workflows. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.

## Exit Gate
- Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
