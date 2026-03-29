## Review Pass 1: Optional/None Wave-9a Residual Canonicalization Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave9a-residual-canonicalization-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Keep fixture rewrites minimal and algorithm-preserving; avoid introducing new helper abstractions that obscure parity with original intent.
2. Record per-fixture rationale in the phase execution ledger to distinguish canonicalization from compiler semantic changes.

Principle compliance:
- explicit Optional safety is preserved.
- no hidden unwrap/coercion paths are introduced.
- no compiler semantics are weakened for this wave.
