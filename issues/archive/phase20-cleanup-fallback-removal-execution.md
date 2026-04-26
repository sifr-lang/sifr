# Phase 20 Cleanup Execution Checklist (Fallback/Legacy Removal)

Status: completed (2026-03-05)
Owner: fallback-cleanup execution loop
Reference PR: https://github.com/sifr-lang/sifr/pull/845

Loop: Work -> Validate -> PR -> Review -> Merge

## Item: Remove Legacy/Compatibility Fallback Paths in HIR Lowering
status: done (2026-03-05, PR #845)

- [x] Remove backward-compat generic-class fallback in annotation lowering
- [x] Enforce strict type-parameter declaration/arity diagnostics
- [x] Remove tuple-target `for` fallback-to-`Any`
- [x] Preserve canonical generic-class imports by plumbing class type params through `ExternalDefs`
- [x] Add positive + negative regression coverage
- [x] Run full local suite
- [x] Open PR, review, and merge

Validation evidence:
- `cargo test -q -p sifr_hir` -> pass (36 tests).
- `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path (strict diagnostics):
  - generic class without declared type params -> `class 'LegacyBox' does not declare type parameters...`
  - tuple-target `for` on non-tuple iterable -> `for loop tuple target expects iterable elements of tuple type...`
  - tuple `match` sequence on non-tuple subject -> `tuple pattern requires subject of tuple type...`

## PR Log
- Main cleanup: https://github.com/sifr-lang/sifr/pull/845

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase20-cleanup-review-2.md`
- Pass-1 triage:
  - Applied reviewer observation to remove remaining tuple-sequence `match` fallback-to-`Any`.
  - Added regression tests for tuple-subject requirement and tuple-arity mismatch in match patterns.
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/846
- External review pass 2 output: `reviews/phase20-cleanup-production-grade-review.md`
- Pass-2 triage:
  - Reviewer confirmed production-grade readiness with no additional blocking defects.
  - No further compiler-code changes were required after validating pass-2 notes.
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/847
