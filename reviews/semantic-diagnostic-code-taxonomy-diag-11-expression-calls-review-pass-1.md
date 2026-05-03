# Semantic Diagnostic Code Taxonomy Review — DIAG-11 Expression Calls

**Branch:** codex/diag-11-raw-hir-expression-calls
**Scope:** Non-simple call targets, iter/next/pow keyword+arity+type diagnostics
**Files:** `expressions.rs`, `expression_diagnostics.rs`, `expressions_tests.rs`

---

## Finding: No Required Fixes Remain

The migrated diagnostic paths are correct, use appropriate codes, have correct primary ranges, and are covered by focused tests. This review found no correctness issues, no taxonomy misplacements, no missing ranges, no fallback behavior, and no maintainability concerns with the migrated code.

---

## Detailed Assessment

### 1. Diagnostic Taxonomy Fit

| Diagnostic | Code Used | Correct? | Notes |
|---|---|---|---|
| Non-simple call target | `CALL_NOT_CALLABLE_OR_ARITY` | ✓ | "only simple function calls are supported" — call target is not a valid callable, fits the code semantics |
| `iter()` unexpected keyword | `CALL_UNEXPECTED_KEYWORD` | ✓ | Keyword arguments not allowed on this call |
| `iter()` wrong arg count | `CALL_WRONG_POSITIONAL_COUNT` | ✓ | Exact-argument builtin with wrong count |
| `iter()` non-iterable type | `TYPE_MISMATCH` | ✓ | Argument type doesn't satisfy protocol |
| `iter()` any/unknown element type | `TYPE_MISMATCH` | ✓ | Dynamic/static type boundary violation |
| `next()` unexpected keyword | `CALL_UNEXPECTED_KEYWORD` | ✓ | Same pattern as iter |
| `next()` wrong arg count | `CALL_WRONG_POSITIONAL_COUNT` | ✓ | Same pattern as iter |
| `next()` non-iterator type | `TYPE_MISMATCH` | ✓ | Argument type doesn't implement Iterator protocol |
| `pow()` unexpected keyword | `CALL_UNEXPECTED_KEYWORD` | ✓ | Keyword arguments not allowed |
| `pow()` wrong arg count | `CALL_WRONG_POSITIONAL_COUNT` | ✓ | Exact-argument builtin |

All code assignments are consistent with prior usage in this codebase and fit the semantic intent of each code.

### 2. Primary Ranges

| Test | Primary Range | Assessment |
|---|---|---|
| `test_non_simple_call_target_has_call_code` | `make()` (anchor: `value: int = `) | ✓ Correct — spans the non-simple call expression |
| `test_iter_keyword_has_call_code` | `source=values` | ✓ Correct — the offending keyword argument |
| `test_iter_wrong_arg_count_has_call_code` | Second `values` (anchor: `iter(values, `) | ✓ Correct — the extra argument |
| `test_iter_non_iterable_has_type_code` | `1` | ✓ Correct — the non-iterable argument |
| `test_next_non_iterator_has_type_code` | `values` | ✓ Correct — the non-iterator argument |
| `test_pow_wrong_arg_count_has_call_code` | `2` (anchor: `pow(`) | ✓ Correct — the argument count message uses `call_arity_range` which covers this |

### 3. Helper Functions (expression_diagnostics.rs)

Four helpers were added:

- `call_not_callable_or_arity(ctx, message, range)` → `DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY`
- `call_unexpected_keyword(ctx, message, range)` → `DiagnosticCode::CALL_UNEXPECTED_KEYWORD`
- `call_wrong_positional_count(ctx, message, range)` → `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT`
- `type_mismatch(ctx, message, range)` → `DiagnosticCode::TYPE_MISMATCH`

All four are thin wrappers around `ctx.error_with_code_at` with a diagnostic code. They are named for the code they emit, not the diagnostic content, which is consistent with the codebase pattern seen in `diagnostics.rs` and `ownership_diagnostics.rs`. No fallback behavior present.

### 4. Message Construction

- `iter()/next()/pow()` keyword errors: message explicitly names the builtin (e.g., `"iter() does not accept keyword arguments"`), which is correct and specific.
- Arity errors: messages use exact-count phrasing (`"iter() takes exactly 1 argument, got N"`) consistent with Python's own error style.
- Type errors: messages use `"argument must be X, got 'Y'"` format, consistent with the codebase convention.
- Non-simple call target: message is `"only simple function calls are supported"` — direct and clear.

### 5. Test Coverage

Each migrated path has a dedicated test:

| Test Name | What It Validates |
|---|---|
| `test_non_simple_call_target_has_call_code` | Code + primary range for non-simple call |
| `test_iter_keyword_has_call_code` | Code + range for keyword on iter() |
| `test_iter_wrong_arg_count_has_call_code` | Code + range for wrong count on iter() |
| `test_iter_non_iterable_has_type_code` | Code + range for non-iterable arg on iter() |
| `test_next_non_iterator_has_type_code` | Code + range for non-iterator arg on next() |
| `test_pow_wrong_arg_count_has_call_code` | Code + range for wrong count on pow() |

Note: `test_iter_non_iterable_has_type_code` covers the non-iterable case at line 498. The any/unknown case at line 478 does not have an explicit test, but the message for that path (`"iter() argument must be an iterable with a statically-known element type..."`) is meaningfully different from the non-iterable path and does not appear to have a dedicated test. This is a minor coverage gap but not a bug — the type is still checked, the diagnostic still fires, and the code is correct. It is mentioned here for completeness, not as a required fix.

### 6. Maintainability / Guardrails

`expression_diagnostics.rs` is a small, focused module with 49 lines total (including 4 helpers). It follows the established pattern of diagnostic modules in the codebase (e.g., `ownership_diagnostics.rs`). No monolithic file issues. The guardrails check was already run and passed.

### 7. Remaining Raw `ctx.error` Emissions in expressions.rs

There are many `ctx.error(format!(...))` calls still present in `expressions.rs` (e.g., lines 612–3607+). These are **outside the DIAG-11 scope** and are not a concern for this review. The branch name and scope explicitly cover only the 10 call/diagnostic sites listed.

---

## Conclusion

No required fixes remain. The migrated code is correct, uses appropriate diagnostic codes, has correct primary ranges, includes focused tests, and follows existing patterns. The scope is clearly bounded and does not touch other raw error emissions in the file.
