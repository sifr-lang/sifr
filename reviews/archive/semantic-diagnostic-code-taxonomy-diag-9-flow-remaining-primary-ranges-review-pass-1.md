# Review: milestone_diag_9 — flow diagnostic primary ranges (remaining)

**Reviewer:** Yaser Alnajjar
**Branch:** `codex/diag-9-next-primary-ranges`
**Date:** 2026-05-02

## Scope

Per the slice definition in `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`, this review covers the remaining flow-family diagnostics after break/continue and condition ranges were addressed:

1. **SIFR-FLOW-0003** — invalid nonlocal / captured nonlocal diagnostics
2. **SIFR-FLOW-0004** — missing return diagnostics
3. **SIFR-FLOW-0005** — invalid condition type diagnostics
4. **Removal of residual spanless `invalid_condition_type` fallback** (requiring caller-provided ranges)

## Files Changed (11 files, 120 insertions / 66 deletions)

| File | Change |
|------|--------|
| `lower/flow_diagnostics.rs` | All 8 diagnostic functions now take `TextRange`, call `error_with_code_at` |
| `lower/nonlocal_support.rs` | Pass `nonlocal.range()`, `name.range()` to flow diagnostics |
| `lower/typing_and_functions.rs` | Pass `func.name.range()` to `missing_return_value` |
| `lower/control_flow_conditions.rs` | `validate_control_flow_condition` now takes non-optional `TextRange`; old `invalid_condition_type` (no range) and `invalid_condition_type_at` merged into one function |
| `lower/aug_assign_lowering.rs` | Pass `name_range` to `captured_augassign_requires_nonlocal` |
| `lower/statements.rs` | Pass `func.name.range()` to `recursive_nonlocal_nested_function`; `Some(range)` → `range` in 3 `validate_control_flow_condition` calls |
| `lower/tuple_unpack.rs` | Pass `tuple.range()` to `tuple_unpack_nonlocal_rebind` |
| `lower/nested_function_tests.rs` | Added `range_for_after` helper; all 7 nonlocal tests now assert `primary_range` |
| `lower/expressions_tests.rs` | `test_non_none_return_annotation_requires_exhaustive_returns` now asserts `primary_range` |
| `tests/e2e/fail/missing_return_value.sifr` | Updated `expect-error` to `expect-error[col=5]` |
| `tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr` | Updated `expect-error` to `expect-error[col=9]` |

## Findings

### Correctness — PASS

- All diagnostic functions in `flow_diagnostics.rs` now uniformly take a `TextRange` and route to `error_with_code_at`. The old `invalid_condition_type` (no range, `error_with_code`) is fully removed; the only remaining function is the range-accepting `invalid_condition_type`.
- `validate_control_flow_condition` signature changed from `Option<TextRange>` to `TextRange` — callers in `lower_if`, `lower_elif`, `lower_while` all pass the condition expression range directly, eliminating the fallback path entirely.
- `nonlocal_support.rs` passes correct ranges: `nonlocal.range()` for "requires enclosing", `name.range()` for conflict and missing-binding.
- `typing_and_functions.rs` passes `func.name.range()` for missing return.
- `aug_assign_lowering.rs` passes the captured variable name range.
- `statements.rs` passes `func.name.range()` for recursive nonlocal helper.

### Missing Range Cases — NONE

Every diagnostic call site now provides a concrete, source-accurate `TextRange`. No diagnostic is emitted without a span.

### Bad Span Choices — NONE

| Diagnostic | Span chosen | Assessment |
|-----------|-------------|------------|
| `nonlocal_requires_enclosing_binding` | `nonlocal.range()` | Correct — entire nonlocal declaration |
| `nonlocal_conflicts_with_current_binding` | `name.range()` | Correct — specific conflicting name |
| `nonlocal_missing_enclosing_binding` | `name.range()` | Correct — unresolved name |
| `captured_augassign_requires_nonlocal` | `name_range` | Correct — captured variable name |
| `tuple_unpack_nonlocal_rebind` | `tuple.range()` | Correct — tuple being unpacked |
| `recursive_nonlocal_nested_function` | `func.name.range()` | Correct — recursive helper name |
| `missing_return_value` | `func.name.range()` | Correct — function name |
| `invalid_condition_type` (if/elif/while) | `test.range()` | Correct — condition expression |

### Fallback-Style Code — NONE

The `Option<TextRange>` fallback is fully eliminated from `validate_control_flow_condition`. All three call sites pass `test.range()` directly. No conditional fallback pattern remains.

### Test Gaps — NONE

- Unit tests (`nested_function_tests.rs`): 7 nonlocal tests assert `primary_range`
- Unit test (`expressions_tests.rs`): 1 missing-return test asserts `primary_range`
- E2E fixtures: `missing_return_value.sifr` updated to `expect-error[col=5]`, `nested_function_recursive_nonlocal_unsupported.sifr` updated to `expect-error[col=9]`
- `elif_condition_numeric_truthiness.sifr`: already has `expect-error[col=10]` — no change needed
- `if_condition_numeric_truthiness.sifr`: unchanged — still valid (exercises error, not precise span)

### Scope — CORRECT

- SIFR-FLOW-0003: all sub-cases covered with proper ranges
- SIFR-FLOW-0004: covered with `func.name.range()`
- SIFR-FLOW-0005: `Option` fallback removed, all 3 keyword sites updated
- No spillover into break/continue or other diagnostic families

### Validation

Pre-review runs (all passed):
- `cargo fmt --check`
- `check_hir_maintainability_guardrails.py`
- `cargo test -p sifr_hir condition_rejects_numeric_truthiness -- --nocapture`
- `cargo test -p sifr_hir nested_function_tests -- --nocapture`
- `cargo test -p sifr_hir test_non_none_return_annotation_requires_exhaustive_returns -- --nocapture --test-threads=1`
- `cargo test -p sifr --test e2e test_e2e_fail -- missing_return_value nested_function_recursive_nonlocal_unsupported elif_condition_numeric_truthiness --nocapture`
- `cargo clippy -p sifr_hir --no-deps -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` (wall_time=52.13s, report_signature=e1bf653aaa770517)

Re-run confirmations:
```
cargo test -p sifr -- test_e2e_fail -- missing_return_value nested_function_recursive_nonlocal_unsupported elif_condition_numeric_truthiness --nocapture  → ok
cargo test -p sifr -- test_e2e_fail -- if_condition_numeric_truthiness --nocapture  → ok
```

## Conclusion

**The reviewer is satisfied.** The implementation is correct, complete, and consistent with the slice specification. All diagnostic functions now carry stable primary `TextRange` spans. The spanless fallback pattern for conditions is fully eliminated. No scope gaps, missing cases, or fallback-style code remain.
