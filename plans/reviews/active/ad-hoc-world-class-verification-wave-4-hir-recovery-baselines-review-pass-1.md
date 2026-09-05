# Wave 4 HIR Recovery Baselines Review Pass 1

Reviewer: agent (`--effort xhigh`)
Date: 2026-06-14
Scope: uncommitted Wave 4 HIR recovery diagnostics-baseline slice.

## Blockers

None. The slice is structurally sound:

- The diagnostics manifest declares `hir_mixed_semantic_recovery` and `hir_repeated_type_recovery` with `human`, `json`, and `compact` renderers and `expect_exit_code: 1`.
- Generated baseline trios are present for both fixtures and all renderers.
- `code_baseline_coverage.json` clears five Wave 4 deferrals: `SIFR-CALL-0004`, `SIFR-NAME-0001`, `SIFR-OWN-0002`, `SIFR-TYPE-0002`, and `SIFR-INTERNAL-0002`.
- `SIFR-TYPE-0002` preserves its `presentation_rules_cases` suggestion fixture while adding rendered and repeated-recovery coverage.
- `recovery_surface_coverage.json` retargets HIR recovery surfaces to diagnostics-area rendered fixtures with expected code lists matching compact baseline evidence.
- Baseline metadata has valid source hashes, suite ownership, and required fields.
- The Wave 4 plan note records the 15 covered / 155 deferred status and validation evidence.

## Non-Blocking Notes

1. The new `json` renderer metadata entries should include the existing `json-sort` normalizer convention.
2. Manifest case order is cosmetic and not enforced.

## Approval

Approved after optionally aligning the `json-sort` convention.
