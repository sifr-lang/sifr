## Review: diag-9-default-arg-primary-ranges (Pass 1)

## Summary

Minimal, focused diff. All four touched files implement the same change: replace
`ctx.error_with_code(...)` with `ctx.error_with_code_at(..., default_expr.range())`
for `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT` diagnostics in default argument collection.

## Changes

### `crates/sifr_hir/src/lower/default_args.rs`

`collect_param_default` now calls `ctx.error_with_code_at` with `default_expr.range()`
as the primary range. This handles regular positional parameters on free functions.
The file gains `use ruff_text_size::Ranged;`. `Ranged` is already imported
in the other two files so this is consistent.

**Status: correct.**

### `crates/sifr_hir/src/lower/typing_and_functions.rs`

`collect_function_defaults` has two error sites (regular args, kwonly args), both
now use `ctx.error_with_code_at` with `default_expr.range()`. Keyword-only parameters
are fully covered here.

**Status: correct.**

### `crates/sifr_hir/src/lower/classes.rs`

Two error sites updated:

1. Constructor (`__init__`) defaults: `ctx.error_with_code` to `ctx.error_with_code_at`
   with `default_expr.range()`. Handles constructor parameter defaults.

2. Regular/class/static method defaults: same pattern for non-init methods.

`ruff_text_size::Ranged` is already imported. `Expr` implements `Ranged`,
so `default_expr.range()` is valid.

**Note**: The field-default path in `AnnAssign` handling still uses `ctx.error` without
code or range. This is out-of-scope for this slice (target is constructors and class
methods, not class field annotations), so this is fine.

**Status: correct.**

### `crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr`

Single test fixture. `col=19` points to `seed()` in `x: int = seed()`. `SIFR-TYPE-0011`
is correctly attached.

**Status: correct.**

### `crates/sifr_hir/src/lower/expressions_tests.rs`

Two new tests added:

- `test_unsupported_function_default_argument_has_primary_range`: free function
- `test_unsupported_method_default_argument_has_primary_range`: class method

Both verify the diagnostic code, message, and primary range via
`range_for_after_anchor(source, "= ", "seed()")`. Pattern is consistent with existing
tests in the file.

**Status: correct.**

## Findings

**No issues found.** The diff is clean, minimal, and correctly targeted.

### Scope confirmation

Targeted paths:

- Free functions -> `default_args.rs` + `typing_and_functions.rs`
- Keyword-only params -> `typing_and_functions.rs`
- Constructors -> `classes.rs` (`__init__` branch)
- Class methods -> `classes.rs` (non-init branch)

Out-of-scope path left unchanged:

- Class field annotation defaults (`AnnAssign`) -> `ctx.error` without code

## Verdict

**Satisfied.** The implementation is correct and complete for the stated scope.
No additional pass required.
