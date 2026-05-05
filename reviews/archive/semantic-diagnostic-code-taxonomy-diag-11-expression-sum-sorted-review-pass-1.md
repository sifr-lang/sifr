# Review: Milestone Diag_11 — Expression sum/sorted Raw HIR Diagnostic Migration

**Branch:** `codex/diag-11-raw-hir-expression-sum-sorted`
**Review pass:** 1
**Date:** 2026-05-03

## Scope

- `expression_sum_sorted.rs` — new module containing sum/sorted lowering helpers and structured diagnostics
- `expression_diagnostics.rs` — expanded with sum/sorted diagnostic helpers
- `expressions.rs` — sum/sorted lowering extracted into the new module
- `expressions_tests.rs` — new test cases verifying error codes on sum/sorted diagnostic paths
- `mod.rs` — new module declaration

## Findings / Verdict

### Module split: `expression_sum_sorted.rs`

The sum and sorted lowering logic (~155 lines) was extracted from `expressions.rs` into a new focused module. The module exposes a clean, narrow API:

- `unsupported_form` — generic expression form error (TYPE_UNSUPPORTED_EXPRESSION_FORM)
- `unsupported_operator` — TYPE_UNSUPPORTED_OPERATOR
- `matrix_multiplication` — TYPE_UNSUPPORTED_OPERATOR
- `call_not_callable_or_arity` — CALL_NOT_CALLABLE_OR_ARITY
- `call_unexpected_keyword` — CALL_UNEXPECTED_KEYWORD
- `call_wrong_positional_count` — CALL_WRONG_POSITIONAL_COUNT
- `call_duplicate_argument` — CALL_DUPLICATE_ARGUMENT
- `type_mismatch` — TYPE_MISMATCH

These cover all diagnostic paths for sum and sorted lowering. The helpers delegate to `ctx.error_with_code_at(...)` with properly selected `DiagnosticCode` variants. No flat `ctx.error(...)` fallback remains in the extracted code.

### `expression_diagnostics.rs`

Four new helpers added: `call_not_callable_or_arity`, `call_unexpected_keyword`, `call_wrong_positional_count`, `call_duplicate_argument`. The existing `type_mismatch` helper was already present. All sum/sorted diagnostic paths now route through these structured helpers with explicit diagnostic codes.

### `expressions.rs`

The inlined sum and sorted lowering was replaced with delegation to `lower_sum_call` and `lower_sorted_call`. `callable_signature` was promoted from private to `pub(super)` to allow the new module to use it.

### Test coverage

New tests added:
- `test_sum_keyword_and_type_errors_have_codes` — validates CALL_UNEXPECTED_KEYWORD and TYPE_MISMATCH codes for sum paths
- `test_sorted_positional_and_duplicate_errors_have_codes` — validates CALL_WRONG_POSITIONAL_COUNT for sorted
- `test_sorted_rejects_duplicate_iterable_argument` — updated to assert CALL_DUPLICATE_ARGUMENT code and primary range
- `test_sorted_type_and_key_errors_have_codes` — validates TYPE_MISMATCH (iterable), CALL_NOT_CALLABLE_OR_ARITY (key), TYPE_MISMATCH (reverse) codes

### Validation

- **361 tests pass** (`cargo test -p sifr_hir -- --skip test_e2e_pass`)
- All diagnostic paths verified to use explicit `DiagnosticCode` variants
- Primary range assertions confirm error positioning is correct

## Verdict

**No required fixes remain.** The module split is clean, all diagnostic paths use structured error helpers with explicit codes, and test coverage confirms correct behavior. Ready for PR.
