# Phase 38: Docs and Documentation

> Note: Needs more planning before execution (doc tooling, doc structure, scope boundaries, ownership model, and acceptance gates are still draft-level).

## Objective
Establish a production-grade documentation layer (developer, user, and operations) so packaging and release governance rest on clear, versioned contracts.

## Depends on
- Phase 37

## Milestones

### milestone_38_1: Documentation Information Architecture
- Scope:
  - Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations.
  - Remove duplicated/contradictory guidance and centralize source-of-truth ownership.
- Definition of done:
  - Documentation map is approved and all core sections have canonical owners.

### milestone_38_2: Reference and Contract Documentation
- Scope:
  - Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts.
  - Publish the diagnostic catalog, output-format contract (`human`/`json`/`compact`), and stability guarantees for codes/help/suggestion fields.
  - Publish one page per stable diagnostic code at `https://docs.sifr.sh/errors/<CODE>`.
  - Document the canonical severity enum exactly as `Error | Warning | Note | Help`.
  - Document the compact renderer contract and examples so downstream tools do not guess at grouping/truncation behavior.
  - Document expected compatibility/stability guarantees for users and contributors.
- Definition of done:
  - Contract docs are complete, versioned, and linked from roadmap/architecture entry points.

### milestone_38_3: Documentation Quality Gates
- Scope:
  - Add local docs validation for link integrity, required sections, and drift checks against phase files.
  - Ensure docs checks are runnable in local `create-pr/merge/release` workflows.
- Definition of done:
  - Documentation quality gates pass locally and are mirrored in CI.

## Quality Contract
- Entry criteria: Phase 37 is completed and package workflows are deterministic.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_38_1` (Documentation Information Architecture): validation goals cover: Define canonical docs structure for language, compiler internals, stdlib, CLI, packaging, and operations; Remove duplicated/contradictory guidance and centralize source-of-truth ownership. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_38_2` (Reference and Contract Documentation): validation goals cover: Publish versioned references for CLI behavior, diagnostics, package workflows, and phase contracts; Publish the diagnostic catalog, output-format contract, and stability guarantees for codes/help/suggestion fields; Publish one page per stable diagnostic code at `https://docs.sifr.sh/errors/<CODE>`; Document the canonical severity enum and compact renderer contract; Document expected compatibility/stability guarantees for users and contributors. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_38_3` (Documentation Quality Gates): validation goals cover: Add local docs validation for link integrity, required sections, and drift checks against phase files; Ensure docs checks are runnable in local `create-pr/merge/release` workflows. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.

## Exit Gate
- Core documentation is canonical, navigable, and quality-gated for ongoing phase execution and release usage.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
