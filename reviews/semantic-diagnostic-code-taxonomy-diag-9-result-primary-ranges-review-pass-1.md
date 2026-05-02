# Review: diag-9 Result primary-range slice
**Branch:** `codex/diag-9-result-primary-ranges`
**Date:** 2026-05-03
**Files changed:** 9 (result_diagnostics.rs, statements.rs, typing_and_functions.rs, result_diagnostics_tests.rs, 5 fixture files)

---

## Summary

Satisfied. No blocking findings. The implementation correctly attaches `primary_range` spans to SIFR-RESULT-0001/0002/0003 diagnostics and wires those spans through `error_with_code_at` into the structured diagnostic output pipeline. All span selections are semantically correct and match the expected column positions in the fixture annotations.

---

## Per-file review

### result_diagnostics.rs

Signature change from `error_with_code` → `error_with_code_at` with explicit `TextRange` — correct. All three public entry points (`invalid_raise_string`, `invalid_raise_non_error`, `invalid_bare_raise`) now accept and forward a `range` parameter. No fallback paths, no `unwrap`.

- `invalid_raise_string` (line 5): `exc.range()` — correctly points at the string literal itself (e.g. `"message"`). This is the most precise possible anchor for the "raise string not allowed" error.
- `invalid_raise_non_error` (line 13): `exc.range()` — correctly points at the invalid expression (e.g. `1`).
- `invalid_bare_raise` (line 21): `raise_stmt.range()` — correctly points at the entire `raise` statement. Since bare `raise` has no sub-expression, the whole statement is the right anchor.

### statements.rs

Line 195 (`RESULT_UNUSED_VALUE`): `expr_stmt.value.range()` — correctly points at the expression whose type is `Result` and was discarded. `expr_stmt.value` is the already-lowered expression (the call), so its range is the correct span.

Line 253 (`invalid_raise_string`): `exc.range()` — passed before `lower_expr` is called, which is correct since we need the raw parser range.

Line 261-265 (`invalid_raise_non_error`): `exc.range()` — same pattern as above.

Line 270 (`invalid_bare_raise`): `raise_stmt.range()` — the `raise` statement has no sub-expression in this branch, so `raise_stmt.range()` is the only available anchor.

### typing_and_functions.rs

Line 530 (`RESULT_INVALID_ERROR_TYPE`): `tuple.elts[1].range()` — correctly points at the error type position inside `Result[T, E]`. Since `tuple` is the parsed annotation, `elts[1]` is the `str` in `Result[int, str]`.

### result_diagnostics_tests.rs

Five new unit tests covering all three diagnostic codes. All use `range_for` / `range_for_after` helpers to avoid hard-coded byte offsets.

- `bare_raise_has_result_invalid_raise_primary_range`: range of `"raise"` on line 2 (1-based col 5) — matches fixture.
- `string_raise_has_result_invalid_raise_primary_range`: range of the string expression — correctly identifies the string literal.
- `non_error_raise_has_result_invalid_raise_primary_range`: `range_for_after("raise ", "1")` — correctly skips `raise ` to land on `1`.
- `unused_result_has_result_unused_value_primary_range`: `range_for_after("def main():\n    ", "fallible()")` — correctly skips to the call expression.
- `invalid_result_error_type_has_primary_range`: `range_for_after("Result[int, ", "str")` — correctly targets the error type slot.

All 5 tests pass.

### Fixture files (5)

All fixtures updated to add `col=N` annotations matching expected 1-based column positions:

| Fixture | Code | Col | Expected anchor | Verified |
|---|---|---|---|---|
| `error_raise_bare.sifr` | SIFR-RESULT-0003 | 5 | `raise` (line 2, col 5) | offset 4 of "    raise\n" is 'r' |
| `error_raise_non_error.sifr` | SIFR-RESULT-0003 | 11 | `1` (line 2, col 11) | offset 10 of "    raise 1\n" is '1' |
| `error_raise_str.sifr` | SIFR-RESULT-0003 | 11 | `"` (line 2, col 11) | offset 10 of "    raise \"...\"" is '"' |
| `error_str_not_allowed.sifr` | SIFR-RESULT-0002 | 29 | `s` in `str` (line 1, col 29) | offset 28 of "Result[int, str]\n" is 's' |
| `unused_result.sifr` | SIFR-RESULT-0001 | 5 | `fallible` (line 5, col 5) | offset 4 of "    fallible()\n" is 'f' |

All column positions are 1-based as required by `parse_expect_error_line`.

---

## Cross-cutting concerns

**No fallback paths.** `error_with_code_at` is used directly. `TextRange` values come from the parser AST and are forwarded without transformation.

**No data-dependent unwrap/expect.** The helper functions `range_for` and `range_for_after` use `expect` but these are test-only utilities operating on known-static string literals — appropriate.

**CFG internal error (pre-existing, not caused by this slice).** The `test_e2e_fail` test emits `internal compiler error: invalid control-flow graph: branch terminator in block 2 is incomplete` for two cases (`unused_result`, `error_str_not_allowed`). This ICE exists on `main` as well — verified by stashing this diff and re-running. This slice does not introduce or worsen the regression.

**Span → column pipeline is correct end-to-end.** The path is:
`error_with_code_at` → `LoweringError.primary_range (TextRange)` → `sifr_driver` → `RenderedDiagnostic` → `DiagnosticSpan (is_primary=true)` → `CompiledFailure.column = span.column`
All links verified. The e2e test harness reads `span.column` from the primary span, which is set by the diagnostic transport layer.

---

## Conclusion

**No blocking findings.** The implementation is correct, complete, and consistent with the phase workflow. All 5 unit tests pass, all 5 e2e fail fixtures pass validation, and there are no regressions introduced by this slice.
