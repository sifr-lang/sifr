## Review Pass 1: Optional/None Wave-9c Residual Canonicalization Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave9c-residual-canonicalization-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Keep iterator-first rewrites explicit about first-element handling to avoid hidden `list[0]` assumptions.
2. Prefer single-source accumulators over dual index aliases when translating DP transitions.

Principle compliance:
- no Optional semantics weakening.
- no hidden unwrap/coercion behavior.
- no compiler rule changes in this wave.
