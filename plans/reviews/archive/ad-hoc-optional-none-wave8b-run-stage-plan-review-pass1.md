## Review Pass 1: Optional/None Wave-8b Run-Stage Ownership Plan

Reviewed artifact:
- `issues/ad-hoc-optional-none-and-narrowing-wave8b-run-stage-ownership-plan-2026-03-29.md`

Verdict:
- **Ready**

Blocking issues:
- none

Non-blocking improvements:

1. Keep release-binary freshness explicit in rerun commands (`cargo build --release -p sifr` before full-corpus artifacts).
2. Preserve negative ownership checks in codegen unit coverage (non-copy clone expected; copy types must remain uncloned).

Principle compliance:
- fix is ownership-lowering scoped and does not alter Optional semantics.
- no hidden unwrap, no truthiness widening, no fixture-specific path.
