# Review: milestone_diag_11 — expression open/callable-call diagnostics slice

**Branch:** `codex/diag-11-raw-hir-expression-open-file-calls`
**Files reviewed:** `expression_diagnostics.rs`, `expressions.rs`, `expressions_tests.rs`

---

## Verdict: APPROVED with no required fixes

All three review goals are satisfied. Local validation already passed.

---

## Goal 1 — `open()` missing path: migrated to structured code/range transport

**Location:** `expressions.rs:1156–1160`

```rust
expression_diagnostics::call_missing_required_argument(
    ctx,
    "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(),
    call.func.range(),   // primary range points at `open`
);
```

**Before:** `ctx.error("…")` with no code and no range.
**After:** Uses `call_missing_required_argument` helper → `CALL_MISSING_REQUIRED_ARGUMENT` (`SIFR-CALL-0004`), primary range is `call.func.range()`.

- No fallback path remains — the old `ctx.error` call is fully replaced.
- `DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT` is defined in `sifr_diagnostics::codes.rs` and listed in the `DIAGNOSTIC_CODES` array.
- Primary range `call.func.range()` correctly targets the `open` token in `open()`.

---

## Goal 2 — Callable-typed variable call arity/type: structured with no fallback

**Location:** `expressions.rs:1297–1306`

```rust
expression_diagnostics::call_not_callable_or_arity(
    ctx,
    format!(
        "callable '{}' expects {} argument(s), got {}",
        func_name, param_types.len(), args.len()
    ),
    range,   // range is args[param_types.len()] when over-saturated, else call.func
);
```

**Before:** `ctx.error(format!(…))` with no code and no range.
**After:** Uses `call_not_callable_or_arity` helper → `CALL_NOT_CALLABLE_OR_ARITY` (`SIFR-CALL-0005`), with a precise primary range:
- When too many args: `call.arguments.args[param_types.len()].range()` (the first excess argument)
- When too few args: `call.func.range()` (the callable itself)

**Type error for same path** (`expressions.rs:1312–1322`): also migrated to `expression_diagnostics::type_mismatch` → `TYPE_MISMATCH`, with `call.arguments.args[i].range()` as primary range. Before: `ctx.error(format!(…))` with no code and no range.

No fallback `ctx.error` call remains in either path.

---

## Goal 3 — Non-simple callable-object guard uses `CALL_NOT_CALLABLE_OR_ARITY`

**Location:** `expressions.rs:1357–1363`

```rust
let Expr::Name(name_expr) = call.func.as_ref() else {
    expression_diagnostics::call_not_callable_or_arity(
        ctx,
        "only simple function calls are supported".to_string(),
        call.func.range(),
    );
    return None;
};
```

This is the guard that rejects non-`Expr::Name` call targets (e.g. `make()(1)` where `make()` returns a callable object). It correctly uses `CALL_NOT_CALLABLE_OR_ARITY`. The primary range is `call.func.range()`, pointing at the whole call expression that cannot be lowered.

---

## Tests

Focused tests exist and assert both `code` and `primary_range`:

| Test | Code | Range |
|---|---|---|
| `test_open_missing_path_has_call_code` | `CALL_MISSING_REQUIRED_ARGUMENT` | `range_for(source, "open")` |
| `test_callable_variable_call_errors_have_codes` (arity) | `CALL_NOT_CALLABLE_OR_ARITY` | `range_for_after_anchor(arity_source, "return ", "f")` |
| `test_callable_variable_call_errors_have_codes` (type) | `TYPE_MISMATCH` | `range_for_after_anchor(type_source, "f(", "\"bad\"")` |
| `test_non_simple_call_target_has_call_code` | `CALL_NOT_CALLABLE_OR_ARITY` | `range_for_after_anchor(source, "value: int = ", "make()")` |

All four tests verify `error.code == Some(DiagnosticCode::…)` and `error.primary_range == Some(…)`.

---

## `expression_diagnostics.rs` additions

The new `call_missing_required_argument` helper (lines 51–60) follows the same pattern as existing helpers (`call_not_callable_or_arity`, `call_wrong_positional_count`, etc.) — thin wrapper around `ctx.error_with_code_at`. No behavioral divergence.

---

## No issues found

All three diagnostic paths are fully migrated. No fallback `ctx.error` calls remain. No `unwrap`/`expect` in user paths. Primary ranges are semantically appropriate. Diagnostic codes are defined in the canonical registry.
