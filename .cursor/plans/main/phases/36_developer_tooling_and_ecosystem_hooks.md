# Phase 36: Developer Tooling and Ecosystem Hooks

## Objective
Prevent tooling/compiler split-brain by migrating CLI behavior onto the canonical shared frontend API, enforcing renderer/parity contracts, and proving thin tooling adapter boundaries before standalone tooling surfaces expand.

## Depends on
- Phase 35

## Milestones

### milestone_36_1: Shared Frontend API Contract
- Scope:
  - Adopt the canonical frontend API for parse/lower/type-check/diagnostics across all compiler CLI modes.
  - Disallow semantics reimplementation in tool-specific paths, including future `sifr_lsp`, `sifr_lint`, editor adapters, automation adapters, and any CLI-only analysis shims.
- Definition of done:
  - Compiler modes consume the same frontend contracts that future tooling integration points must use.

### milestone_36_2: Tooling/CLI Parity Matrix
- Scope:
  - Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs.
  - Define the minimum required parity corpus explicitly:
    - one parse diagnostic
    - one type-check diagnostic
    - one warning diagnostic
    - one diagnostic carrying `Help`
    - one diagnostic carrying a structured suggestion
    - one multi-file diagnostic
    - one recovery case that emits multiple diagnostics deterministically
  - Cover diagnostics codes, URLs, spans, child note/help payloads, structured suggestion payloads, renderer outputs, and type-check outcomes.
- Definition of done:
  - Divergence between tooling and compiler behavior is automatically detected before merge.
  - The required parity corpus is snapshot- or fixture-backed and runs locally.

### milestone_36_3: Thin Adapter and Renderer Boundaries
- Scope:
  - Define thin adapter boundaries for editor/automation consumers that transport canonical diagnostics without owning semantics.
  - Require the proof adapter for this phase to be a non-CLI editor/automation-facing diagnostic adapter that consumes the shared frontend API directly and emits canonical diagnostics without reimplementing parse/lower/type-check logic. A full LSP server is not required in this phase.
  - Permit `human`, `json`, and `compact` presentation layers only as renderers over the same diagnostic model.
  - Treat Phase 27 as the authoritative compact-renderer contract; this phase only validates and snapshot-locks that contract.
  - Explicitly forbid standalone tooling surfaces from bypassing the shared analysis/query API.
- Definition of done:
  - The required non-CLI editor/automation adapter proves canonical diagnostics can be consumed without semantic duplication.
  - Renderer selection does not alter diagnostic ownership or meaning.
  - Compact output is deterministic and regression-locked by snapshots.

## Quality Contract
- Entry criteria: Phase 35 is completed and compiler performance/query contracts plus the shared analysis/query foundation are enforced.
- Exit criteria: Tooling integration is split-brain-resistant, renderer-stable, and regression-covered against compiler behavior.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_36_1` (Shared Frontend API Contract): validation goals cover: Adopt the canonical frontend API for parse/lower/type-check/diagnostics across all compiler CLI modes; Disallow semantics reimplementation in tool-specific paths including future `sifr_lsp`, `sifr_lint`, editor adapters, automation adapters, and CLI-only analysis shims. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_36_2` (Tooling/CLI Parity Matrix): validation goals cover: Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs; Enforce the minimum required parity corpus for parse, type-check, warning, help, structured suggestion, multi-file, and recovery cases; Cover diagnostics codes, URLs, spans, child note/help payloads, structured suggestion payloads, renderer outputs, and type-check outcomes. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_36_3` (Thin Adapter and Renderer Boundaries): validation goals cover: Define thin adapter boundaries for editor/automation consumers that transport canonical diagnostics without owning semantics; Require the non-CLI editor/automation proof adapter to consume the shared frontend API directly; Permit `human`, `json`, and `compact` presentation layers only as renderers over the same diagnostic model; Treat Phase 27 as the authoritative compact-renderer contract and validate it via deterministic snapshots; Explicitly forbid standalone tooling surfaces from bypassing the shared analysis/query API. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Tooling integration is split-brain-resistant, renderer-stable, and regression-covered against compiler behavior.

## Exit Gate
- Tooling integration is split-brain-resistant, renderer-stable, and regression-covered against compiler behavior.
