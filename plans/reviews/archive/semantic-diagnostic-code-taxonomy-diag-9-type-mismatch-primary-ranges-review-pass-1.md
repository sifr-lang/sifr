# Review: diag-9 type mismatch / container / operator primary ranges

**Branch:** `codex/diag-9-type-mismatch-primary-ranges`
**Date:** 2026-05-03
**Reviewer:** Claude Code review
**Validation passed:** `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=55.42s`)

## Summary

The slice attaches primary source ranges to spanless active HIR diagnostics in the type mismatch, operator, container, TypeVar-constraint, and hashability surfaces, and splits helper modules to satisfy HIR maintainability guardrails.

## Findings

No issues found.

## Range Review

- `SIFR-TYPE-0002` argument, generic, annotated-assignment, reassignment, and return mismatches now point at the offending argument, initializer, reassignment RHS, or return expression. Correct.
- `SIFR-TYPE-0003` if-expression branch mismatch points at the `else` branch expression. Correct and precise for the current comparison order.
- `SIFR-TYPE-0008` container literal conflicts point at the conflicting element, dict key, dict value, or `iter()` tuple argument. Correct.
- `SIFR-TYPE-0010` TypeVar constraint violations point at the argument bound to the constrained type parameter. Correct.
- Augmented-assignment operator-helper diagnostics point at the RHS operand. Correct.
- `hash()` hashability diagnostics point at the non-hashable argument. Correct.

## Helper Extraction Review

`call_argument_ranges.rs` and `container_literal_diagnostics.rs` are internal `lower` helpers with `pub(super)` functions only. Imports are acyclic and layering is clean.

## Fixture Review

The reviewed e2e column assertions are aligned with actual 1-indexed emitted columns:

- `type_mismatch.sifr`: col 14
- `reassignment_type_mismatch.sifr`: col 13
- `union_type_mismatch.sifr`: col 20
- `mutable_list_variance_invariant.sifr`: col 31
- `stdlib_test_assert_eq_type_mismatch.sifr`: col 18
- `ternary_type_mismatch.sifr`: col 33
- `container_literal_type_conflict.sifr`: col 18
- `typevar_constraints_violation.sifr`: col 23

## Verdict

Approved. No semantic issues, layering problems, fixture misalignments, or recovery-behavior regressions found.
