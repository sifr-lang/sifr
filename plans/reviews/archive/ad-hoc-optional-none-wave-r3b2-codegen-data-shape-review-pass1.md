# Review: Wave-R3b2 Codegen Data-Shape Plan (Pass 1)

Date: 2026-03-30
Reviewer channel: agent review skill (inline response; artifact persisted by agent)
Plan under review: `issues/ad-hoc-optional-none-and-narrowing-wave-r3b2-codegen-data-shape-plan-2026-03-30.md`

## Verdict

`NOT READY`

## Blocking Findings

1. Incomplete fixture reconciliation with the probe baseline:
   - plan scope lists `0187`, `1461`, `1582`, `0441`, `1905`
   - baseline probe has `RUN_ERROR=10` and includes additional run fixtures (`0054`, `0071`, `0349`, `0459`, `0763`)
2. `0459_repeated_substring_pattern` was not explicitly accounted for in scope/out-of-scope.

## Required Changes Before Implementation

1. Reconcile scope with the probe baseline.
2. Explicitly classify `0459` as either:
   - in-scope for R3b2, or
   - out-of-scope with rationale and wave ownership.

## Reviewer Principle Check

- Codegen-only plan direction is aligned with Sifr principles.
- No fixture rewrites required in this wave.
- Restrict boolop condition coercion to bool-typed boolops only.
- Ownership hint path for `list(arg)` is acceptable if it materially changes yield-mode from borrow to owned-clone where target element type is known.
