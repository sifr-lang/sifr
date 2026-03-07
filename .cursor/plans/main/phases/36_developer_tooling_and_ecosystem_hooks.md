# Phase 36: Developer Tooling and Ecosystem Hooks

## Objective
Prevent tooling/compiler split-brain by migrating CLI behavior onto the canonical shared frontend API, enforcing renderer/parity contracts, and proving thin tooling adapter boundaries before standalone tooling surfaces expand.

## Depends on
- Phase 35

## Milestones

### milestone_36_1: Shared Frontend API Contract
- Scope:
  - Adopt the canonical frontend API for parse/lower/type-check/diagnostics across all compiler CLI modes.
  - Disallow semantics reimplementation in tool-specific paths.
- Definition of done:
  - Compiler modes consume the same frontend contracts that future tooling integration points must use.

### milestone_36_2: Tooling/CLI Parity Matrix
- Scope:
  - Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs.
  - Cover diagnostics codes, URLs, spans, child note/help payloads, structured suggestion payloads, renderer outputs, and type-check outcomes.
- Definition of done:
  - Divergence between tooling and compiler behavior is automatically detected before merge.

### milestone_36_3: Thin Adapter and Renderer Boundaries
- Scope:
  - Define thin adapter boundaries for editor/automation consumers that transport canonical diagnostics without owning semantics.
  - Permit `human`, `json`, and `compact` presentation layers only as renderers over the same diagnostic model.
  - Define the compact renderer precisely enough that an implementer can build snapshot tests without guessing:
    - line 1 is a summary count of `Error` and `Warning`
    - groups are ordered by severity, then code, then file/line
    - each group header contains severity, code, canonical message, and occurrence count
    - each group shows a bounded list of `path:line:column` exemplars
    - each group shows at most one help line and one `see https://sifr.dev/docs/errors/<CODE>` line
    - truncation is explicit via `... +N more`
    - compact mode never contains source snippets or ANSI styling
  - Explicitly forbid standalone tooling surfaces from bypassing the shared analysis/query API.
- Definition of done:
  - At least one non-CLI adapter path proves canonical diagnostics can be consumed without semantic duplication.
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
  - `milestone_36_1` (Shared Frontend API Contract): validation goals cover: Adopt the canonical frontend API for parse/lower/type-check/diagnostics across all compiler CLI modes; Disallow semantics reimplementation in tool-specific paths. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_36_2` (Tooling/CLI Parity Matrix): validation goals cover: Add parity test matrix comparing tooling-facing analysis results vs compiler CLI results for equivalent inputs; Cover diagnostics codes, URLs, spans, child note/help payloads, structured suggestion payloads, renderer outputs, and type-check outcomes. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_36_3` (Thin Adapter and Renderer Boundaries): validation goals cover: Define thin adapter boundaries for editor/automation consumers that transport canonical diagnostics without owning semantics; Permit `human`, `json`, and `compact` presentation layers only as renderers over the same diagnostic model; Define the compact renderer format precisely enough for deterministic snapshots; Explicitly forbid standalone tooling surfaces from bypassing the shared analysis/query API. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Tooling integration is split-brain-resistant, renderer-stable, and regression-covered against compiler behavior.

## Exit Gate
- Tooling integration is split-brain-resistant, renderer-stable, and regression-covered against compiler behavior.
