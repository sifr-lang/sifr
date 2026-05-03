# Review: diag-11 raw HIR method call args migration - pass 2

**Status:** APPROVED

The guardrail fix is clean and correct.

## Key Findings

1. **dict.update keyword normalization preserved** - `resolved_method_arg_ranges` correctly handles the special case: for `dict.update()` it appends the first keyword's range to the positional ranges, so `arg_ranges[1]` in `resolve_method_type` correctly points to the keyword source span for type checking.

2. **RAW_HIR_ERROR_FREE_FILES is accurate** - All three `validate_*_arg` functions now use `ctx.error_with_code_at(...)` with the correct semantic codes (`TYPE_MISMATCH` for element mismatches, `PROTO_INVALID_ITERATOR_SIGNATURE` for non-iterables). Zero raw `ctx.error(format!(...))` calls remain.

3. **Source ranges improved** - Call sites now pass explicit `arg_ranges[index]` so `primary_range` points precisely to the offending sub-expression, for example `"1"` in `list.extend(1)`, rather than a default or undefined range.

4. **Maintainability** - `expressions.rs` is exactly 3800 lines, at the guardrail limit and not over. `method_call_args.rs` is 663 lines. The three new tests cover the newly migrated error paths.

5. **All guardrails green** - fmt, clippy, both guardrail scripts, unit tests, and quick profile all pass.

No further review pass required.
