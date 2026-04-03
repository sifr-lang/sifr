## Review Pass 1: Optional/None Wave-9d Residual Canonicalization Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave9d-residual-canonicalization-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Prefer explicit running maxima/counts over helper calls that can introduce Optional return surfaces.
2. Keep index-to-value relationships encoded through iterator state or dict defaults, not implicit list-index assumptions.

Principle compliance:
- no Optional semantics weakening.
- no hidden unwrap/coercion behavior.
- no compiler rule changes in this wave.
