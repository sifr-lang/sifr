# Ad Hoc Optional/None Closure: Wave-R3c Reviewer Pass-1 (2026-03-30)

Reviewer: Claude (talk-to-claude skill; recovered from handoff logs due Claude-side file-write permission block)  
Status: `ready-with-guardrails`

## Scope Reviewed

- Empty-list specialization in HIR for `append`/`insert`/`extend`.
- Len-alias-aware sequence guard detection for `while`/compare guard shapes.
- No-step slice lowering semantics for negative `start`/`stop` indices.

## Findings

- Root-cause ownership is correct:
  - empty-list inference and container specialization are HIR responsibilities,
  - negative slice bound normalization is codegen responsibility.
- Proposed fixes align with Sifr principles:
  - no fallback behavior,
  - no fixture rewrites,
  - static and explicit narrowing/specialization.
- Guardrail noted:
  - keep sequence-guard changes scoped to guard-detection semantics (len anchors / aliases), not as a separate ad-hoc type pass.

## Decision

- Proceed with implementation for all three sub-slices, with regression tests and targeted fixture reruns recorded in wave evidence.
