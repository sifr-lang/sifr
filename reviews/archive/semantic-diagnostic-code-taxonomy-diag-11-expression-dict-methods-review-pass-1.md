# Review: Dict-method diagnostic migration (DIAG-11, pass 1)

## Scope
`resolve_method_type` in `expressions.rs` — dict-method branch only (`Type::Dict` match arm, lines 2567–2784).

## Checks

### 1. No dict-method raw `ctx.error` sites remain
- Dict branch (2567–2784): only helper calls (`reject_no_method_args`, `reject_exact_method_arg_count`, `reject_max_method_arg_count`, `reject_method_arg_count`, `validate_dict_update_arg`, `expression_diagnostics::type_mismatch`).
- No bare `ctx.error(...)` calls in the dict branch.
- The `default` arm (2777–2783) uses `ctx.error_with_code_at` with `DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE`.
**PASS**

### 2. `CALL_WRONG_POSITIONAL_COUNT` for dict method arity errors with sensible primary ranges
- `reject_no_method_args` → `reject_method_arg_count` → `expression_diagnostics::call_wrong_positional_count` → `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT`.
- `reject_exact_method_arg_count` and `reject_max_method_arg_count` similarly route through `reject_method_arg_count`.
- All use `method_count_range(...)` to compute the primary range.
- Test `test_dict_method_wrong_positional_count_has_call_code` confirms `clear(1)` gets `CALL_WRONG_POSITIONAL_COUNT` with primary on `"1"`.
**PASS**

### 3. `TYPE_MISMATCH` for dict key/default/value mismatch errors with argument ranges
- `expression_diagnostics::type_mismatch` → `DiagnosticCode::TYPE_MISMATCH`.
- Used in: `dict.contains` (arg_ranges[0]), `dict.get` key (arg_ranges[0]) / default (arg_ranges[1]), `dict.pop` key / default, `dict.setdefault` key / default.
- All use the relevant argument range.
- Test `test_dict_method_type_mismatch_has_type_code` confirms `get(1)` on `dict[str, int]` gets `TYPE_MISMATCH` with primary on the key arg `"1"`.
**PASS**

### 4. `STDLIB_UNSUPPORTED_SURFACE` for missing dict method diagnostics with method-name range
- Default arm (2777–2783): `ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, ..., method_range)`.
- Test `test_dict_missing_method_has_stdlib_code` confirms `missing()` gets `STDLIB_UNSUPPORTED_SURFACE` with primary on `"missing"`.
**PASS**

### 5. Behavior and messages are preserved except code/range transport
All messages match the raw `ctx.error` equivalents:
- `"dict.len() takes no arguments"` — preserved
- `"dict.keys() takes no arguments"` — preserved
- `"dict.values() takes no arguments"` — preserved
- `"dict.items() takes no arguments"` — preserved
- `"dict.update() takes at most 2 arguments, got N"` — preserved
- `"dict.clear() takes no arguments"` — preserved
- `"dict.copy() takes no arguments"` — preserved
- `"dict.contains() takes exactly 1 argument, got N"` — preserved
- `"dict.contains() key type 'X' is not compatible with dict key type 'Y'"` — preserved
- `"dict.get() takes 1 or 2 arguments, got N"` — preserved
- `"dict.get() key type 'X' is not compatible with dict key type 'Y'"` — preserved
- `"dict.get() default type 'X' is not compatible with dict value type 'Y'"` — preserved
- `"dict.pop() takes 1 or 2 arguments, got N"` — preserved
- `"dict.pop() key type 'X' is not compatible with dict key type 'Y'"` — preserved
- `"dict.pop() default type 'X' is not compatible with dict value type 'Y'"` — preserved
- `"dict.setdefault() takes exactly 2 arguments, got N"` — preserved
- `"dict.setdefault() key type 'X' is not compatible with dict key type 'Y'"` — preserved
- `"dict.setdefault() default type 'X' is not compatible with dict value type 'Y'"` — preserved
- `"dict has no method 'X'"` — preserved
**PASS**

### 6. Tests meaningfully cover code and primary ranges
- `test_dict_method_wrong_positional_count_has_call_code`: `clear(1)` → `CALL_WRONG_POSITIONAL_COUNT`, primary = `"1"`.
- `test_dict_method_type_mismatch_has_type_code`: `get(1)` on `dict[str, int]` → `TYPE_MISMATCH`, primary = `"1"`.
- `test_dict_missing_method_has_stdlib_code`: `missing()` → `STDLIB_UNSUPPORTED_SURFACE`, primary = `"missing"`.
**PASS**

## Validation

| Command | Result |
|---------|--------|
| `cargo fmt` | PASS |
| `cargo test -p sifr_hir dict_method_wrong_positional_count -- --nocapture` | PASS |
| `cargo test -p sifr_hir dict_method_type_mismatch -- --nocapture` | PASS |
| `cargo test -p sifr_hir dict_missing_method -- --nocapture` | PASS |
| `cargo check -p sifr_hir` | PASS |
| `cargo clippy -p sifr_hir -- -D warnings` | PASS |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS |
| `git diff --check` | PASS |

## Notes
- `validate_dict_update_arg` already routes through `expression_diagnostics::type_mismatch` — confirmed unchanged.
- Arity helpers (`reject_no_method_args`, `reject_exact_method_arg_count`, `reject_max_method_arg_count`, `reject_method_arg_count`) are shared with other type branches (list, set, etc.) and are not within scope for this review.

## Conclusion
All checks pass. **reviewer is satisfied.**
