# Review: semantic-diagnostic-code-taxonomy diag-11 expression str-methods (pass 1)

## Reviewer: agent

## Date: 2026-05-03

## Files Reviewed
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

## Validation Results

| Check | Command | Result |
|-------|---------|--------|
| cargo fmt | `cargo fmt --check` | PASS |
| cargo test str_method_wrong_positional_count | `cargo test -p sifr_hir str_method_wrong_positional_count -- --nocapture` | PASS |
| cargo test str_method_type_mismatch | `cargo test -p sifr_hir str_method_type_mismatch -- --nocapture` | PASS |
| cargo test str_missing_method | `cargo test -p sifr_hir str_missing_method -- --nocapture` | PASS |
| cargo check -p sifr_hir | `cargo check -p sifr_hir` | PASS |
| cargo clippy -p sifr_hir | `cargo clippy -p sifr_hir -- -D warnings` | PASS |
| hir maintainability guardrails | `python3 scripts/check_hir_maintainability_guardrails.py` | PASS |
| git diff --check | `git diff --check` | PASS |

## Checklist

### no str-method raw ctx.error sites remain
**PASS** — In `resolve_method_type` (lines 2318–3261), the `Type::Str` match arm (lines 2924–3061) contains zero raw `ctx.error(...)` calls. All error emission uses:
- `reject_exact_method_arg_count` → `call_wrong_positional_count` → `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT`
- `reject_method_arg_count` → `call_wrong_positional_count` → `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT`
- `expression_diagnostics::type_mismatch` → `DiagnosticCode::TYPE_MISMATCH`
- `ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, ...)`

### CALL_WRONG_POSITIONAL_COUNT used for str method arity errors with sensible primary ranges
**PASS** — `reject_exact_method_arg_count` and `reject_method_arg_count` delegate to `call_wrong_positional_count` which emits `CALL_WRONG_POSITIONAL_COUNT`. The primary range is the result of `method_count_range(actual, expected, arg_ranges, method_range)`:
- `startswith`/`endswith` (lines 2928–2940): if 1 arg expected but got more, range points to the extra argument
- `isdigit`/etc (lines 2942–2948): if args provided to no-arg method, range points to first extra arg
- `split` (lines 2949–2971): arity error range via `method_count_range`
- `replace` (lines 2972–2994): arity error range via `method_count_range`
- `join`/`count`/`find`/`center`/`ljust`/`rjust`/`zfill` (lines 2995–3051): range via `method_count_range`

### TYPE_MISMATCH used for split maxsplit and replace count type errors with argument ranges
**PASS** — Both use `expression_diagnostics::type_mismatch`:
- `split` maxsplit type error (lines 2958–2968): `arg_ranges[1]` — the offending second argument
- `replace` count type error (lines 2981–2991): `arg_ranges[2]` — the offending third argument

### STDLIB_UNSUPPORTED_SURFACE used for missing str method diagnostics with method-name range
**PASS** — Lines 3053–3059:
```rust
ctx.error_with_code_at(
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    format!("str has no method '{method}'"),
    method_range,
);
```
`method_range` correctly points to the method name (e.g., `"missing"` in `text.missing()`).

### behavior and messages are preserved except code/range transport
**PASS** — Message strings are unchanged:
- `"str.{method}() takes exactly N argument(s), got {actual}"` — preserved in `reject_exact_method_arg_count`
- `"str.split() takes 0 to 2 arguments, got {actual}"` — preserved
- `"str.replace() takes 2 or 3 arguments, got {actual}"` — preserved
- `"str.split() maxsplit must be 'int', got '{actual}'"` — preserved
- `"str.replace() count must be 'int', got '{actual}'"` — preserved
- `"str has no method '{method}'"` — preserved

### tests meaningfully cover code and primary ranges
**PASS** — Three tests in `expressions_tests.rs`:
- `test_str_method_wrong_positional_count_has_call_code` (line 3265): verifies `CALL_WRONG_POSITIONAL_COUNT` with primary range on the extra argument (`"1"` in `text.find("a", 1)`)
- `test_str_method_type_mismatch_has_type_code` (line 3278): verifies `TYPE_MISMATCH` with primary range on the type-mismatched argument (`"bad"` in `text.split(",", "bad")`)
- `test_str_missing_method_has_stdlib_code` (line 3291): verifies `STDLIB_UNSUPPORTED_SURFACE` with primary range on the method name (`"missing"` in `text.missing()`)

## Summary

All six checklist items pass. The str-method diagnostic migration in `resolve_method_type` correctly:
1. Eliminates all raw `ctx.error` sites in the `Type::Str` branch
2. Uses `CALL_WRONG_POSITIONAL_COUNT` for arity errors with argument ranges
3. Uses `TYPE_MISMATCH` for type errors on split maxsplit and replace count with argument ranges
4. Uses `STDLIB_UNSUPPORTED_SURFACE` for missing methods with method-name range
5. Preserves message text exactly
6. Has meaningful test coverage for all three diagnostic codes and their primary ranges

**reviewer is satisfied.**
