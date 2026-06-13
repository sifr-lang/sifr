

**Code review: satisfied.**

The slice is clean. Key observations:

1. **Diagnostic codes correct**: `CALL_WRONG_POSITIONAL_COUNT` for arity errors, `STDLIB_UNSUPPORTED_SURFACE` for unknown method — both defined in `codes.rs` and used consistently across decimal and bigdecimal paths.

2. **Primary ranges precise**: 
   - Arity errors (`sqrt(arg)`) range to the offending argument (`"1"`) via `decimal_method_arity_range`
   - Unknown method errors range to the method name token

3. **Existing scale/literal diagnostics untouched**: `decimal_scale_diagnostic_code` remains in place and unmodified.

4. **Script extension correct**: `decimal_methods.rs` added to `RAW_HIR_ERROR_FREE_FILES` — the linter will now reject any re-introduction of raw `ctx.error` in this file.

5. **Tests adequate**: `test_decimal_method_surface_errors_have_structured_codes` covers both `decimal.sqrt(1)` (arity) and `decimal.magnitude()` (unknown method) with full code/message/range assertions.

6. **No fallback/inference paths**: All method cases branch explicitly with no `ctx.error` anywhere in the file after the migration.

7. **No retired symbols or wrong codes**: `CALL_WRONG_POSITIONAL_COUNT` and `STDLIB_UNSUPPORTED_SURFACE` are live codes in `codes.rs`.
