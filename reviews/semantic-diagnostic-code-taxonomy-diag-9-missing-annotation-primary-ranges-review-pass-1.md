# Review: diag-9 — missing type annotation primary ranges

## Scope reviewed

All touched files from the current uncommitted diff:

- `crates/sifr_hir/src/lower/typing_and_functions.rs`
- `crates/sifr_hir/src/lower/classes.rs`
- `crates/sifr_hir/src/lower/nested_function_inference.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr_hir/src/lower/nested_function_tests.rs`
- 9 e2e fail fixtures (`missing_type_annotation`, `missing_vararg_type_annotation`, `missing_keyword_only_type_annotation`, `nested_function_missing_type_annotation`, `class_init_missing_type_annotation`, `class_method_missing_type_annotation`, `protocol_method_missing_type_annotation`, `enum_method_missing_type_annotation`, `newtype_method_missing_type_annotation`)

## Changes

### 1. `typing_and_functions.rs`

Three locations updated: regular params, vararg params, and keyword-only params.
All switched from `ctx.error_with_code(...)` → `ctx.error_with_code_at(..., param.parameter.name.range())`.
Range is the parameter identifier itself — correct for all three cases.

### 2. `classes.rs`

Helper function `missing_method_param_annotation` gained a `range: TextRange` parameter and
switched to `error_with_code_at`. Six call sites updated, all passing `param.parameter.name.range()`.
Correct.

### 3. `nested_function_inference.rs`

`ParamState` gained a `name_range: TextRange` field, populated at collection time from
`param.parameter.name.range()`. In `finalize_nested_function_types`, the error call switched to
`error_with_code_at(..., param.name_range)`. Correct.

### 4. Test files

- `expressions_tests.rs`: two new tests covering function and class method param primary ranges.
  Both assert `primary_range == Some(range_for(source, "value"))` — matches the expected span.
- `nested_function_tests.rs`: one new test for nested function inference failure.
  Uses `range_for_after(source, "helper(", "value")` — correct (identifier is not a top-level token,
  needs an anchor).

### 5. E2E fixtures

All nine fixtures updated from bare `# expect-error: SIFR-TYPE-0004` to
`# expect-error[col=N]: SIFR-TYPE-0004`. Column positions verified manually:

| Fixture | col | Param identifier starts at |
|---|---|---|
| `missing_type_annotation` | 14 | `value` at position 14 |
| `missing_vararg_type_annotation` | 12 | `values` at position 12 |
| `missing_keyword_only_type_annotation` | 16 | `verbose` at position 16 |
| `nested_function_missing_type_annotation` | 16 | `n` at position 16 |
| `class_init_missing_type_annotation` | 24 | `value` at position 24 |
| `class_method_missing_type_annotation` | 21 | `value` at position 21 |
| `protocol_method_missing_type_annotation` | 22 | `value` at position 22 |
| `enum_method_missing_type_annotation` | 20 | `value` at position 20 |
| `newtype_method_missing_type_annotation` | 19 | `amount` at position 19 |

All column values are correct.

## Issues found

None.

## Residuals

No fallback/raw `error_with_code` calls for the touched paths remain in scope.
All five categories (regular params, varargs, keyword-only, nested inference failures, class/protocol/enum/newtype method params) have concrete primary ranges attached.

## Validation

Local validation was run and passed: `scripts/run_all_tests.sh --profile quick` completed in ~57s with exit 0. Clippy clean, fmt clean, guardrails clean.

## Verdict

**Satisfied.** The implementation is correct, consistent, and consistent with the established pattern in this codebase. No further passes required.
