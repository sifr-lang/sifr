

Reviewing the diff:

**Diagnostic code selection** — All codes are appropriately chosen:
- `CALL_UNEXPECTED_KEYWORD` for keyword rejection (including unpacked variants)
- `CALL_WRONG_POSITIONAL_COUNT` for arity violations
- `TYPE_MISMATCH` for type errors on arguments
- `STDLIB_UNSUPPORTED_SURFACE` for invalid stdlib surface forms (factory names, `defaultdict` factory values)
- `PROTO_INVALID_ITERATOR_SIGNATURE` for non-iterable arguments to reversibles

**Primary range quality** — Verified each site:
- Keyword/unpacked errors: `first_keyword_range(call)` ✓
- Arity errors: `arity_range(call)` ✓
- Type errors on args: `arg_expr.range()` or `call.arguments.args[0].range()` ✓
- `defaultdict` factory name errors: `call.arguments.args[0].range()` ✓
- `lower_builtin_reverseable_arg` branches correctly on `keywords.is_empty()` to pick either `arity_range` or `first_keyword_range` ✓

**Fallback paths** — None introduced. All helper functions (`reject_keywords`, `reject_wrong_positional_count`, `reject_type_mismatch`, `reject_unsupported_surface`, `reject_unpacked_keyword_at`) consistently delegate to `ctx.error_with_code_at` with no raw `ctx.error` remaining.

**Retired transport symbols** — `RAW_CTX_ERROR` regex `\bctx\.error\s*\(` is correctly scoped only to `RAW_HIR_ERROR_FREE_FILES`. No risk of false positives on the broader codebase.

**One observation** — `lower_builtin_reverseable_arg` (lines 753–783) uses inline `ctx.error_with_code_at` rather than a helper. This is consistent with the pattern of keeping the function self-contained. Not a finding.

**Test gap** — No new snapshot tests were added for the structured diagnostic codes. Existing builtin tests pass, but without snapshot updates there is no machine-checkable record that the specific code/message combinations are correct. This is pre-existing debt in the test suite, not introduced by this slice.

**Conclusion**: No blocking findings. The migration is correct and consistent. Reviewer is satisfied.
