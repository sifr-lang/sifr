# Phase 27: Diagnostics, Error Recovery, Stability Contract, Panic-to-Diagnostic

## Objective
Deliver production-quality diagnostics with recovery and explicit stability guarantees.

## Depends on
- Phase 26

## Sequencing Note
- Phase 27 is one sequential phase split across two files for readability:
  - [27_runtime_safe_codegen_semantics.md](./27_runtime_safe_codegen_semantics.md) defines milestones `27_1` through `27_3`
  - this file defines milestones `27_4` through `27_6`
- Milestones are completed in numeric order; this file is not a parallel track.
- Corrective amendment: the ad-hoc semantic diagnostic code taxonomy and structured HIR diagnostics phase re-closes the diagnostic-code taxonomy and structured HIR diagnostic contract after later review found Phase 27's implementation still used phase-derived buckets, string-oriented HIR diagnostics, message-prefix classifiers, and spanless frontend semantic diagnostics. Phase 27 remains completed historically; the ad-hoc phase is the active prerequisite for future stable-diagnostic work.

## Milestones

### milestone_27_4: Span and Diagnostic Schema Quality
- Scope:
  - Replace the current predominantly string-oriented frontend diagnostic plumbing with the canonical structured diagnostic model defined in `architecture.md`.
  - Introduce one canonical structured diagnostic model shared by parser/lowering/type-check/codegen.
  - Define the canonical top-level `Severity` enum exactly as `Error | Warning | Note`; help is attached through diagnostic help fields or child help messages.
  - Thread precise spans through frontend/codegen errors.
  - Standardize stable diagnostic codes, related-span labels, help text, deterministic documentation URLs, and structured fix-suggestion fields.
  - Require every top-level diagnostic to expose `url = "https://sifr.sh/docs/errors/<CODE>"`.
  - Define the canonical structured diagnostic schema with at least: `code`, `severity`, `message`, `message_template`, structured `args`, `url`, `spans`, `children`, `help`, and optional structured suggestions.
  - Define structured suggestions with applicability and replacement edits rather than renderer-only help text.
  - Define stable diagnostic renderers for `human` and `json` output modes without changing semantic ownership.
  - Require `json` mode to be the lossless machine-readable rendering of the canonical diagnostic schema.
- Definition of done:
  - Diagnostics include accurate source locations, stable codes, stable URLs, and a stable structured schema consumed by all compiler modes.
  - The severity enum and structured suggestion model are implemented exactly as amended by the ad-hoc semantic diagnostic taxonomy phase.

### milestone_27_5: Bounded Multi-Error Recovery
- Scope:
  - Add parser/type-check recovery to report multiple actionable errors.
  - Control error cascades with bounded recovery policy.
  - Fix the recovery contract to these hard limits:
    - at most 50 top-level diagnostics per compiler invocation
    - at most 5 repeated diagnostics with the same `(severity, code, canonical message, primary file)` before summarizing the remainder as `... +N more similar diagnostics`
    - at most 5 representative locations shown per compact-renderer group
  - Define diagnostic prioritization and deduplication rules so recovery does not produce noisy duplicate cascades.
  - Define stable ordering rules for recovered diagnostics so repeated runs emit the same grouped results.
- Definition of done:
  - Compiler reports multiple useful errors without crash storms or unbounded duplicate cascades.
  - Recovered diagnostics are emitted in deterministic order.
  - The documented recovery limits are enforced by regression tests.

### milestone_27_6: Stability Contract Finalization
- Scope:
  - Define documented exit codes, CLI flag stability/versioning, diagnostic-text policy, and output-format stability policy.
  - Enumerate the stable exit-code contract exactly as:
    - `0` success, including successful runs with warnings only
    - `1` user-facing compile/check/test diagnostics
    - `2` CLI usage or configuration error
    - `3` internal compiler failure after panic/error boundary handling
  - Define the stable CLI contract for `--diagnostic-format human|json|compact`.
  - Require unknown `--diagnostic-format` values to fail with exit code `2` and no semantic compilation work.
  - Define compact-renderer invariants inspired by `rtk` token-efficient grouping:
    - first line is a severity summary
    - diagnostics are grouped by `(severity, code, canonical message)`
    - each group prints a bounded list of representative locations
    - each group prints at most one help line and one documentation URL line
    - truncation uses `... +N more`
    - compact mode never invents or drops diagnostics relative to `json`
  - Require a checked-in panic inventory before panic-to-diagnostic conversion begins, covering parser/lowering/type-check/codegen/driver paths reachable from user input.
  - Convert remaining user-triggerable panics to diagnostics.
- Definition of done:
  - Stability policy is explicit and enforced by tests/docs.
  - `human`, `json`, and `compact` are stable contracts with documented equivalence boundaries.
  - The panic inventory exists, is linked from the phase checklist issue, and every remaining user-triggerable panic is either eliminated or tracked with an explicit owner/issue.

## Quality Contract
- Entry criteria: Phase 26 is completed and runtime-safe codegen invariants are active.
- Exit criteria: Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_27_4` (Span and Diagnostic Schema Quality): validation goals cover: Replace the current predominantly string-oriented frontend diagnostic plumbing with one canonical structured diagnostic model shared by parser/lowering/type-check/codegen; Define the canonical `Severity` enum exactly as `Error | Warning | Note | Help`; Thread precise spans through frontend/codegen errors; Standardize stable diagnostic codes, related-span labels, help text, deterministic documentation URLs, and structured suggestion kinds; Require every top-level diagnostic to expose `https://sifr.sh/docs/errors/<CODE>`; Define stable `human` and lossless `json` renderers without changing semantic ownership. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_5` (Bounded Multi-Error Recovery): validation goals cover: Add parser/type-check recovery to report multiple actionable errors; Control error cascades with bounded recovery policy; Enforce the documented hard limits for total diagnostics, duplicate grouping, and representative locations; Define prioritization, deduplication, and stable ordering rules for recovery output. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_27_6` (Stability Contract Finalization): validation goals cover: Define documented exit codes, CLI flag stability/versioning, diagnostic-text policy, and output-format stability policy; Enumerate the stable exit-code contract; Define the stable CLI contract for `--diagnostic-format human|json|compact`; Require unknown diagnostic-format values to fail with exit code `2`; Define compact-renderer invariants inspired by `rtk` grouping/truncation without changing semantics; Require a checked-in panic inventory; Convert remaining user-triggerable panics to diagnostics. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.

## Exit Gate
- Compiler diagnostics are stable, span-accurate, recovery-capable, and panic-free on user input.
