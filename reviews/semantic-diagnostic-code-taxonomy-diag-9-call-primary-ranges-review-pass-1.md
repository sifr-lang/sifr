# Review: diag-9 Call Diagnostic Primary-Range Slice

**Status: SATISFIED — No blocking findings.**

## Scope

Uncommitted diff only (HEAD), covering:
- `crates/sifr_hir/src/lower/method_call_args.rs`
- `crates/sifr_hir/src/lower/builtin_calls.rs`
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- 10 e2e fail fixtures under `crates/sifr/tests/e2e/fail/`

---

## SIFR-CALL Emitter Coverage

All five `SIFR-CALL-*` codes are present and instrumented with `error_with_code_at`:

| Code | Constant | Files | Primary Range Choice |
|------|----------|-------|----------------------|
| 0001 | `CALL_WRONG_POSITIONAL_COUNT` | `method_call_args.rs:94`, `expressions.rs:1167` | Offending argument (excess arg or `call.func.range()`) |
| 0002 | `CALL_UNEXPECTED_KEYWORD` | `builtin_calls.rs:26,843`, `expressions.rs:1234,1384` | `name.range()` (the unknown keyword identifier) |
| 0003 | `CALL_DUPLICATE_ARGUMENT` | `builtin_calls.rs:811,822,833`, `method_call_args.rs:278,328` | `name.range()` (duplicate keyword name) |
| 0004 | `CALL_MISSING_REQUIRED_ARGUMENT` | `builtin_calls.rs:854`, `expressions.rs:1251`, `method_call_args.rs:342` | `call.func.range()` (the callee) |
| 0005 | `CALL_NOT_CALLABLE_OR_ARITY` | `expressions.rs:1505` | Offending iterable argument or `call.func.range()` |

Span choices are consistent with prior diagnostic primary-range work (e.g., class/match/protocol/result phases) and correct per error type.

---

## Code Quality

### `method_call_args.rs`

- `LoweredKeyword` struct (name + value + name_range) is a clean abstraction. `take_keyword`, `keyword_arg`, `lower_keyword_args` all updated consistently.
- `VarargCallArgs` struct bundles the call context including `missing_range = call.func.range()`, which is then threaded through `lower_vararg_function_call_args` and forwarded to `missing_argument_error`. This is the correct span for a missing-argument diagnostic — it points at the callee, not at the call site tail.
- All error helpers (`duplicate_argument_error`, `missing_argument_error`, `unexpected_keyword_error`) now accept `range: TextRange` and delegate to `error_with_code_at`.
- Keyword-normalization helpers (`append_start_stop_args`, `normalize_list_method_args`, `normalize_dict_method_args`, `normalize_string_method_args`) all correctly use `keyword.name_range` for duplicate/unexpected keyword errors.
- `lower_keyword_args` (line 278) now emits `CALL_DUPLICATE_ARGUMENT` with `name.range()` for duplicate keyword detection in the loop — previously this used `ctx.error` without a code or span. Now fully SIFR-CALL-0003 compliant.

### `builtin_calls.rs`

- `reject_zip_keywords_if_present`: unexpected keyword uses `name.range()` — correct.
- `lower_range_call`: duplicate keys (start/stop/step) all use `name.range()`. Unexpected key uses `name.range()`. Missing `stop` uses `call.func.range()` — correct.
- No remaining `ctx.error_with_code` (without `_at`) in the diff hunk.

### `expressions.rs`

- `sum` wrong arity (line 1167): uses the second (excess) argument's range, falling back to `call.func.range()` if none — correct.
- `sorted` unexpected keyword (line 1234): uses `name.range()` — correct.
- `sorted` missing iterable (line 1251): uses `call.func.range()` — correct.
- `enumerate` unexpected keyword (line 1384): uses `name.range()` — correct.
- `map` arity mismatch (line 1505): uses `call.arguments.args[expected_count + 1].range()` for the excess iterable — correct.

### `expressions_tests.rs`

All 15+ existing call-code tests augmented with `primary_range` assertions using `range_for` / `range_for_after_anchor`. Assertions are precise and follow the established pattern from prior diagnostic phases.

---

## Pre-existing Non-Blocker: `enumerate` duplicate-start at line 1391

```rust
// expressions.rs:1390-1392
if call.arguments.args.len() == 2 {
    ctx.error("enumerate() got multiple values for argument 'start'".to_string());
    return None;
}
```

This path detects "duplicate start" when BOTH a positional start arg AND a `start=` keyword are provided. It uses `ctx.error` (no code, no span). This is **pre-existing** — it existed before this diff and is outside the scope of this slice (which focused on keyword normalization paths, not positional+keyword conflict detection). The outer loop at line 1382 already covers `CALL_UNEXPECTED_KEYWORD` with `name.range()` for non-"start" keywords. Not a blocker.

---

## E2E Fixtures

All 10 fixtures updated from `# expect-error: CODE` to `# expect-error[col=N]: CODE`. Column values are consistent with the expected span in each test. No fixture was broken or misaligned.

---

## Refactoring Soundness

The `LoweredKeyword` struct correctly carries `name_range` alongside value. All call sites that construct `LoweredKeyword` pass `name.range()`. All consumers (duplicate detection, unexpected keyword traversal, `take_keyword` → `append_start_stop_args` / normalize helpers) are updated in lockstep. No silent loss of range information.

---

## No Fallback Behavior

All paths that emit a SIFR-CALL diagnostic go through `error_with_code_at` with a concrete `TextRange`. No `ctx.error_with_code` (without `_at`) remains in any diff hunk for SIFR-CALL codes. The only `ctx.error_with_code` occurrences are in the removed lines (shown with `-` prefix in diff).

---

## Summary

The slice is complete, correct, and consistent. All SIFR-CALL-* diagnostics are now primary-range-aware by construction. No regressions, no missed emitters, no questionable span choices. Local validation (quick profile) has already passed.
