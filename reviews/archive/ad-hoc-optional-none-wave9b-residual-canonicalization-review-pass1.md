## Review Pass 1: Optional/None Wave-9b Residual Canonicalization Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave9b-residual-canonicalization-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Keep first-element handling explicit for potentially empty iterables.
2. Prefer direct scalar temporaries over container-index fallback patterns.

Principle compliance:
- no Optional semantics weakening.
- no hidden unwrap/coercion behavior.
- no compiler rule changes in this wave.
