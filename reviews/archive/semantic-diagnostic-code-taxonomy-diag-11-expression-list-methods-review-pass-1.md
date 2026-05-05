# Review: semantic-diagnostic-code-taxonomy Diag-11 Expression List Methods

## Summary

Review of the list-method diagnostic migration in `resolve_method_type` (expressions.rs:2317).

## Checklist Verification

### 1. No list-method raw ctx.error sites remain

**PASS.** All list method error sites use migrated helpers:

- Arity errors: `reject_exact_method_arg_count`, `reject_max_method_arg_count`, `reject_no_method_args`, `reject_method_arg_count` all delegate to `expression_diagnostics::call_wrong_positional_count` → `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT`
- Type errors: `expression_diagnostics::type_mismatch` → `DiagnosticCode::TYPE_MISMATCH`
- Missing method: `ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, ...)` at line 2558-2562

No raw `ctx.error` calls appear in the `Type::List` arm of `resolve_method_type`.

### 2. CALL_WRONG_POSITIONAL_COUNT for list method arity errors with sensible primary ranges

**PASS.** Arity errors use `CALL_WRONG_POSITIONAL_COUNT` via the rejection helper chain. Primary range is computed by `method_count_range(actual, expected, arg_ranges, method_range)` which produces a range covering the excess arguments.

### 3. TYPE_MISMATCH for list sort/pop/index argument type errors with argument ranges

**PASS.** Type mismatch errors for list methods use `TYPE_MISMATCH`:

- `list.sort() reverse` (line 2423-2430): `expression_diagnostics::type_mismatch(ctx, ..., arg_ranges[0])`
- `list.pop() index` (line 2485-2492): `expression_diagnostics::type_mismatch(ctx, ..., arg_ranges[0])`
- `list.index() bounds` (line 2544-2551): `expression_diagnostics::type_mismatch(ctx, ..., arg_ranges.get(bound_index).copied().unwrap_or(method_range))`

All use the argument range as primary range.

### 4. STDLIB_UNSUPPORTED_SURFACE for missing list method diagnostics with method-name range

**PASS.** The catch-all `_` case at line 2557-2564 uses:

```rust
ctx.error_with_code_at(
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    format!("list has no method '{method}'"),
    method_range,
);
```

### 5. Behavior and messages are preserved except code/range transport

**PASS.** Messages unchanged from original:

- `list.append() takes exactly 1 argument, got {n}`
- `list.sort() argument 'reverse' must be 'bool', got '{ty}'`
- `list.pop() index must be 'int', got '{ty}'`
- `list.index() takes 1 to 3 arguments, got {n}`
- `list.index() bounds must be 'int', got '{ty}'`
- `list has no method '{method}'`

### 6. Tests meaningfully cover code and primary ranges

**PASS.** Three dedicated tests:

- `test_list_method_wrong_positional_count_has_call_code` (line 3147): verifies `CALL_WRONG_POSITIONAL_COUNT` and primary range `"2"` in `xs.append(1, 2)`
- `test_list_method_type_mismatch_has_type_code` (line 3160): verifies `TYPE_MISMATCH` and primary range `"0"` in `xs.pop("0")`
- `test_list_missing_method_has_stdlib_code` (line 3173): verifies `STDLIB_UNSUPPORTED_SURFACE` and primary range `missing` in `xs.missing()`

## Validation Results

```
cargo fmt --check                    ✓ (no output = pass)
cargo test -p sifr_hir list_method_wrong_positional_count  ✓
cargo test -p sifr_hir list_method_type_mismatch            ✓
cargo test -p sifr_hir list_missing_method                  ✓
cargo check -p sifr_hir                                     ✓
cargo clippy -p sifr_hir -- -D warnings                     ✓
python3 scripts/check_hir_maintainability_guardrails.py      ✓ PASS
git diff --check                                            ✓
```

## Conclusion

All review criteria satisfied. The migration correctly:
- Eliminates all raw `ctx.error` sites in list method handling
- Uses `CALL_WRONG_POSITIONAL_COUNT` for arity errors with argument ranges
- Uses `TYPE_MISMATCH` for type errors on sort/pop/index arguments
- Uses `STDLIB_UNSUPPORTED_SURFACE` for missing method diagnostics with method name range
- Preserves original error messages
- Is covered by tests verifying both diagnostic code and primary range

**reviewer is satisfied.**
