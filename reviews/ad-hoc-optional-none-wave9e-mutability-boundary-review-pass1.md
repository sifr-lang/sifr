## Review Pass 1: Optional/None Wave-9e Mutability Boundary Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave9e-mutability-boundary-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Keep `mut` introduction strictly on parameters that are actually mutated.
2. For tuple-swap replacements, use explicit temporaries to keep ownership/mutation flow obvious.

Principle compliance:
- no Optional semantics weakening.
- explicit mutability boundaries preserved.
- no compiler rule changes in this wave.
