# Review: milestone_diag_11 expression functional builtin diagnostics

**Branch:** `codex/diag-11-raw-hir-expression-functional-builtins`
**Scope:** `expression_functional_builtins.rs`, `expressions.rs`, `expressions_tests.rs`, `mod.rs`
**Verdict:** **APPROVED — no required fixes**

---

## Diagnostic Code / Range Correctness

All five builtin slices (`zip`, `any`, `all`, `map`, `filter`) use structured `expression_diagnostics` helpers that route through `ctx.error_with_code_at`, which attaches a `DiagnosticCode` and `TextRange`. All error sites pass a `primary_range` derived from the call expression's argument structure:

| Builtin | Error category | Range anchor |
|---|---|---|
| `zip` — wrong keyword | `CALL_UNEXPECTED_KEYWORD` | `first_keyword_range` (first keyword token) |
| `zip` — wrong iterable type | `TYPE_MISMATCH` | per-argument `arg_expr.range()` |
| `any/all` — wrong arity | `CALL_WRONG_POSITIONAL_COUNT` | `call_arity_range` (last arg or function name) |
| `map` — unexpected keyword | `CALL_UNEXPECTED_KEYWORD` | `first_keyword_range` |
| `map` — too few args | `CALL_WRONG_POSITIONAL_COUNT` | `call_arity_range` |
| `map` — wrong iterable type | `TYPE_MISMATCH` | per-argument `arg_expr.range()` |
| `map` — callable/arity mismatch | `CALL_NOT_CALLABLE_OR_ARITY` | function name or excess iterable arg |
| `filter` — unexpected keyword | `CALL_UNEXPECTED_KEYWORD` | `first_keyword_range` |
| `filter` — wrong arity | `CALL_WRONG_POSITIONAL_COUNT` | `call_arity_range` |
| `filter` — wrong iterable type | `TYPE_MISMATCH` | second arg range |
| `filter` — callable not callable | `CALL_NOT_CALLABLE_OR_ARITY` | first arg range |
| `filter` — callable arity ≠ 1 | `CALL_NOT_CALLABLE_OR_ARITY` | first arg range |
| `filter` — callable return not bool | `TYPE_MISMATCH` | first arg range |

The range logic in `expression_functional_builtins.rs:11-23` (`first_keyword_range` and `call_arity_range`) is correct and consistent with the equivalently-named helpers in `expressions.rs:360-372`.

---

## Raw `ctx.error` Fallback Statement

**Confirmed: No raw `ctx.error` fallback exists in `expression_functional_builtins.rs`.**

Every error path in the four public entry points (`lower_zip_call`, `lower_any_all_call`, `lower_map_call`, `lower_filter_call`) routes through `expression_diagnostics::*` helpers. Grep across the file confirms zero occurrences of `ctx.error\b`.

---

## Module Split and Test Coverage

**Module split is appropriate.** Extracting `zip`/`any`/`all`/`map`/`filter` lowering into `expression_functional_builtins.rs` reduces the monolithic `expressions.rs` file and keeps closely related logic co-located. The `mod.rs` declaration at `expressions.rs:28` is correct, and all `pub(super)` entry points are reachable from `expressions.rs` via the `super::` path, exactly as the imports at `expression_functional_builtins.rs:5-8` demonstrate.

**Tests are sufficient.** `expressions_tests.rs` covers the full diagnostic surface:

- `test_any_all_wrong_arity_have_call_codes` — verifies `any()` zero-args and `all()` two-args emit `CALL_WRONG_POSITIONAL_COUNT` with correct anchors
- `test_zip_non_iterable_argument_has_type_code` — verifies `zip(nums, 1)` emits `TYPE_MISMATCH` with the non-iterable argument as primary range
- `test_zip_keyword_diagnostics_are_stable` — verifies `strict=True` and `bogus=` keyword errors emit `CALL_UNEXPECTED_KEYWORD` with stable anchors
- `test_map_callable_arity_mismatch_has_call_code` — verifies `map(inc, [1,2], [3,4])` emits `CALL_NOT_CALLABLE_OR_ARITY` with the excess iterable as range
- `test_map_argument_errors_have_codes` — covers missing iterable, wrong iterable type, and non-callable first arg
- `test_map_is_typed_as_iterator` — verifies the result type is `Iterator[int]`, not `list[int]`
- `test_map_rejects_plain_list_annotation_without_materialization` — confirms Iterator→list coercion is rejected without explicit materialization
- `test_map_rejects_keywords_with_stable_diagnostic` — verifies keyword argument rejection
- `test_filter_is_typed_as_iterator` — verifies result type
- `test_filter_rejects_plain_list_annotation_without_materialization` — confirms Iterator→list coercion is rejected
- `test_filter_rejects_keywords_with_stable_diagnostic` — verifies keyword argument rejection
- `test_filter_argument_errors_have_codes` — covers arity, non-callable, wrong return type

---

## Non-blocking Notes

1. **Naming asymmetry for range helpers**: `expression_functional_builtins.rs` defines `first_keyword_range` (line 11) while `expressions.rs` defines `first_call_keyword_range` (line 360). They are identical in body. This is a cosmetic inconsistency and not a bug.

2. **`call_arity_range` is defined in both modules**: `expression_functional_builtins.rs:18` defines `call_arity_range` locally, but `expressions.rs:367` also defines an identical `call_arity_range`. Since they are in separate module scopes this causes no conflict, but the duplication could be consolidated into a shared helper if the module structure is refactored later. Not a blocker.
