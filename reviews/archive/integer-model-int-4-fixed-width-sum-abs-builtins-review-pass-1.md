

# Code Review: INT-4 Fixed-Width Sum/Abs Builtins

## Severity 1 — Blocking

### 1. `expressions.rs` now exceeds the HIR maintainability guardrail
The guardrail limit is 3800 lines; the file is now **3821 lines** (21 lines over). The `scripts/check_hir_maintainability_guardrails.py` check explicitly fails:
```
HIR maintainability guardrails: FAIL
- crates/sifr_hir/src/lower/expressions.rs is 3821 lines (limit 3800)
```

The 15 lines added (`fixed_width_abs_widens_to_current_int` helper + ~6 lines of widening logic in `lower_call`) pushed it over the limit. This blocks the local validation gate.

**Fix:** Move the helper function to a shared location (e.g., `super::builtin_calls` or a new `expressions_helpers.rs`) and keep only the call-site logic in `expressions.rs`. The widening decision in `lower_call` is small enough to stay inline.

---

## Severity 2 — Code Quality / Internal Consistency

### 2. Four identical `fixed_width_*_widens_to_current_int` helpers
The same `matches!(FixedIntType::I8|I16|I32|U8|U16|U32)` predicate is copy-pasted in four locations:

| Location | Function name |
|---|---|
| `expressions.rs` | `fixed_width_abs_widens_to_current_int` |
| `expression_sum_sorted.rs` | `fixed_width_sum_widens_to_current_int` |
| `nested_function_inference.rs` | `fixed_width_builtin_widens_to_current_int` |
| `intrinsic_method_emitters.rs` | `fixed_width_builtin_widens_to_current_int` |

These are byte-for-byte identical except for the function name. With the conservative fixed-width set being a deliberate implementation choice (not expected to change per-slice), this duplication is a maintenance hazard: when the set expands in a future slice, all four must be kept in sync.

**Fix:** Extract to a shared helper. Since `sifr_type_system` already has `FixedIntType`, this could live as `pub fn fixed_width_widens_to_int(fixed: FixedIntType) -> bool` in `sifr_type_system` or in a shared `hir::lower::builtin_helpers` module. The function name should be generic (`*_widens_to_int`) since it applies to both `sum` and `abs`.

---

## Severity 3 — Diagnostics

### 3. `abs(int8.MIN)` edge case is handled correctly in codegen but not reflected in the error message

The codegen correctly widens fixed-width to `i64` before calling `.abs()`, so `abs(-128i8)` generates `(-128i8 as i64).abs()` which produces `128i64`. This matches the contract.

However, the current error message for `abs()` on non-numeric types says `"abs() argument must be numeric, got 'X'"`. This is slightly imprecise for fixed-width integers — they *are* numeric, they just don't pass the `ty.is_numeric()` check. The widened types are technically valid, so the error path for `abs()` with a fixed-width type is unreachable. But if a user somehow triggers the old path (shouldn't happen), the message is misleading.

**Verdict:** Not blocking — the widening is correct and the guard `!ty.is_numeric() && !fixed_width_abs_widens` correctly lets in fixed-width types. No action needed.

---

## Severity 4 — Test Coverage (Informational)

### 4. The e2e fixture is not registered in the test harness
The file `crates/sifr/tests/e2e/pass/fixed_width_sum_abs_builtins.sifr` exists but is not included in the e2e test runner's discovery mechanism. Running `scripts/run_e2e_pass.sh` with fixture selection fails because the harness doesn't discover untracked fixtures. The unit tests in `expressions_tests.rs` cover the HIR lowering correctly, but the e2e fixture would never run in CI.

**Fix:** Register the fixture or confirm it's intentionally a manual-only test. Given the unit test coverage, this is low urgency, but the fixture won't serve its purpose if it can't run in CI.

---

## Contract Alignment Assessment

The implementation correctly implements the slice's stated acceptance criteria:

- **`sum(list[int32])` returns source-level `int`**: ✓ Widening via `.map(|x| x as i64).sum::<i64>()` and HIR type set to `Type::Int`. The `expressions_tests.rs` test confirms the inferred type is `Type::Int`.
- **`abs(int8.MIN)` returns source-level `int`**: ✓ Widen-to-i64 before `.abs()` prevents overflow. The e2e fixture asserts `abs(-128)` yields `128` and the codegen confirms the generated Rust is `(-128i8 as i64).abs()`.

The conservative set (`I8/I16/I32/U8/U16/U32`) is correctly scoped — `int64`/`uint64`/`isize`/`usize` would overflow an i64-backed accumulator and are correctly excluded.

---

## Verdict

**Another pass is required after fixes.**

The guardrail violation in `expressions.rs` is a blocking failure — the local validation gate does not pass. The duplicate helpers are a code quality issue that should be addressed before merging, as they create a synchronization hazard for future widening set expansion.
