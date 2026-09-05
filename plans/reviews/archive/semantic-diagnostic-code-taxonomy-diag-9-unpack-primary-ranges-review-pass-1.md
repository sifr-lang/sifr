# Review: diag-9 unpack primary ranges

**Branch:** `codex/diag-9-unpack-primary-ranges`
**Date:** 2026-05-03
**Reviewer:** agent review
**Validation passed:** `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=56.42s`)

## Summary

The slice attaches primary ranges to the remaining tuple-unpack and star-unpack structured diagnostics in HIR and removes the deprecated spanless `LowerCtx::error_with_code` helper.

## Findings

No issues found.

## Range Review

- Tuple unpack wrong-count diagnostics anchor on the tuple target range. Correct.
- Tuple unpack non-tuple RHS diagnostics anchor on the RHS expression. Correct.
- Tuple unpack reassignment type mismatches anchor on the specific target binding range. Correct.
- Star unpack list-shape diagnostics anchor on the RHS expression. Correct.
- For-loop tuple target arity and non-tuple-element diagnostics anchor on the tuple target range. Correct.

## Helper Removal Review

Deleting `LowerCtx::error_with_code` is safe. `rg "error_with_code\\(" crates/sifr_hir/src/lower -n` has no matches after the slice, and the replacement `error_with_code_at` enforces primary ranges for structured HIR diagnostics.

## Fixture Review

The reviewed e2e column assertions are aligned with emitted primary-span start columns:

- `tuple_unpack_shape_mismatch.sifr`: col 5
- `tuple_unpack_non_tuple_shape_mismatch.sifr`: col 19
- `tuple_unpack_reassignment_type_mismatch.sifr`: col 5
- `star_unpack_requires_list_type.sifr`: col 20
- `for_loop_tuple_target_arity_mismatch.sifr`: col 9
- `for_loop_tuple_target_non_tuple_element.sifr`: col 9

## Verdict

Approved. No semantic range issues, helper-deletion risks, fixture misalignments, guardrail risks, or recovery-behavior regressions found.
