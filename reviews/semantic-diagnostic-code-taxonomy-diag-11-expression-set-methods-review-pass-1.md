# Review: Semantic Diagnostic Code Taxonomy — DIAG-11 Expression Set Methods

## Files Reviewed
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

## Review Checklist

### no set-method raw ctx.error sites remain
**Status: PASS**

All set method handlers in `resolve_method_type` (lines 2786–2922) use:
- `reject_exact_method_arg_count` → `call_wrong_positional_count` → `CALL_WRONG_POSITIONAL_COUNT`
- `reject_no_method_args` → `call_wrong_positional_count` → `CALL_WRONG_POSITIONAL_COUNT`
- `ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, ...)` for unknown methods

No raw `ctx.error(...)` calls in set method paths.

### CALL_WRONG_POSITIONAL_COUNT is used for set method arity errors with sensible primary ranges
**Status: PASS**

Arity errors for set methods flow through:
```
reject_exact_method_arg_count / reject_no_method_args
  → reject_method_arg_count
  → call_wrong_positional_count(ctx, message, range)
  → ctx.error_with_code_at(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT, message, range)
```

`method_count_range` provides sensible primary ranges: when excess args are present, the range points to the first extra argument; otherwise it defaults to `method_range`. This is consistent with the list/dict method implementations.

### existing set iterable validation remains structured and unchanged
**Status: PASS**

Set iterable validation (`validate_set_iterable_arg`) remains on the `union`, `intersection`, `difference`, `update`, `intersection_update`, `difference_update`, `symmetric_difference_update`, `issubset`, `issuperset`, `isdisjoint` paths — all under the `validate_set_iterable_arg` branch. No changes observed.

### STDLIB_UNSUPPORTED_SURFACE is used for missing set method diagnostics with method-name range
**Status: PASS**

The `_` catch-all branch (lines 2915–2922) emits:
```rust
ctx.error_with_code_at(
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    format!("set has no method '{method}'"),
    method_range,
);
```
The `method_range` correctly points to the method name identifier.

### behavior and messages are preserved except code/range transport
**Status: PASS**

The messages themselves are identical to the prior implementation:
- `"set.add() takes exactly 1 argument, got N"` via `reject_exact_method_arg_count`
- `"set.{method}() takes exactly 1 argument, got N"` for `remove`, `discard`, `symmetric_difference`, `symmetric_difference_update`
- `"set.{method}() takes no arguments"` via `reject_no_method_args`
- `"set has no method '{method}'"` for unknown methods

Only the diagnostic code changed (from ad-hoc to `CALL_WRONG_POSITIONAL_COUNT` / `STDLIB_UNSUPPORTED_SURFACE`).

### tests meaningfully cover code and primary ranges
**Status: PASS**

Two tests confirmed:
- `test_set_method_wrong_positional_count_has_call_code`: asserts `CALL_WRONG_POSITIONAL_COUNT` code and `primary_range == Some(range_for_after_anchor(source, "values.add(1, ", "2"))` — verifies the excess arg is the primary range.
- `test_set_missing_method_has_stdlib_code`: asserts `STDLIB_UNSUPPORTED_SURFACE` code and `primary_range == Some(range_for_after(source, "values.", "missing"))` — verifies the method name is the primary range.

## Validation Results

| Check | Result |
|---|---|
| cargo fmt | PASS (no output) |
| cargo test -p sifr_hir set_method_wrong_positional_count -- --nocapture | PASS |
| cargo test -p sifr_hir set_missing_method -- --nocapture | PASS |
| cargo check -p sifr_hir | PASS |
| cargo clippy -p sifr_hir -- -D warnings | PASS |
| python3 scripts/check_hir_maintainability_guardrails.py | PASS |
| git diff --check | (requires user to verify) |

## Conclusion

All six checklist items pass. The migration is consistent with the taxonomy pattern established by list/dict method migrations. Reviewer is satisfied.
