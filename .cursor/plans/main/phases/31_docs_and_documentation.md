# Phase 31: Docs and Documentation

> Note: Needs more planning before execution (scope boundaries, ownership model, and acceptance gates are still draft-level).

## Objective
Establish a production-grade documentation layer (developer, user, and operations) so packaging and release governance rest on clear, versioned contracts.

## Depends on
- Phase 30

## Milestones

### milestone_31_1: Documentation Information Architecture
- Scope:
  - Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations.
  - Remove duplicated/contradictory guidance and centralize source-of-truth ownership.
- Definition of done:
  - Documentation map is approved and all core sections have canonical owners.

### milestone_31_2: Reference and Contract Documentation
- Scope:
  - Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts.
  - Document expected compatibility/stability guarantees for users and contributors.
- Definition of done:
  - Contract docs are complete, versioned, and linked from roadmap/architecture entry points.

### milestone_31_3: Documentation Quality Gates
- Scope:
  - Add local docs validation for link integrity, required sections, and drift checks against phase files.
  - Ensure docs checks are runnable in local `quick/full/stress` workflows.
- Definition of done:
  - Documentation quality gates pass locally and are mirrored in CI.

## Exit Gate
- Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
