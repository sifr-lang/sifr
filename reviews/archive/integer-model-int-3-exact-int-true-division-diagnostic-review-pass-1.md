# INT-3 Exact Integer True Division Diagnostic — Review Pass 1

## Findings

### Correctness

**SIFR-INT-0006 registration and activation — correct.**
The diagnostic code is declared at `codes.rs:67-68`, registered in `DIAGNOSTIC_REGISTRY` with the representative fail fixture at `codes.rs:887-897`, and included in `ACTIVE_DIAGNOSTIC_CODES` at `codes.rs:1644`. Owner is `sifr_type_system` — correct per the implementation location.

**Type-system check at `check.rs:220-225` — correct fail-closed behavior.**
When both operands of `/` are exact integer types (`Int`, `LiteralInt`, `BigInt`, `FixedInt`), the check fires before reaching the fallthrough `left.is_numeric() && right.is_numeric() -> Type::Float` path. This means `int / int` no longer silently produces `float` — it now emits `SIFR-INT-0006`. This matches the integer model: `int / int` should be `Result[float, DivisionError | FloatOverflowError | FloatPrecisionLossError]` unless the divisor is proven non-zero and float-representability is proven (acceptance criteria for INT-3).

**HIR test at `expressions_tests.rs:330-347` — correct.**
Verifies that `numerator / denominator` (both `int`) produces `SIFR-INT-0006` with the correct message and the correct primary range covering `numerator / denominator`.

**Type-system unit test at `check.rs:661-675` — correct.**
Tests all four exact integer categories: `Int/Int`, `LiteralInt/LiteralInt`, `FixedInt/Int`, `BigInt/BigInt`. All expect `INT_EXACT_TO_FLOAT_REQUIRES_HANDLING` with the precision-loss message.

**Legacy pass fixture removed — correct.**
`codegen_int_division.sifr` expected `int / int -> float` to succeed and tested `assert str(result) == '3.3333333333333335'`. This was valid under the old silent-cast model and is correctly removed.

**New fail fixture at `crates/sifr/tests/e2e/fail/exact_int_true_division_requires_handling.sifr` — correct.**
Uses `# expect-error: SIFR-INT-0006` and tests `numerator / denominator` where both are `int`. The e2e fail harness will enforce this diagnostic.

**Docs updated — correct.**
`docs/errors/SIFR-INT-0006.md` generated, `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` both have the INT0006 row with `Active` status.

### Non-blocking: Code duplication

`is_exact_to_float_integer_type` at `check.rs:41-45` is byte-for-byte identical to `is_integer_type` at `check.rs:34-38`. Both expand to:

```rust
matches!(
    ty.resolve_alias(),
    Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::FixedInt(_)
)
```

The new function name is self-documenting in context, but the duplication is a maintenance hazard — a future edit to one without the other would silently break the invariant. The preferred resolution is to either alias `is_exact_to_float_integer_type = is_integer_type` or eliminate the new name entirely and call `is_integer_type` at the `/` check site. This is **non-blocking** for merge since the logic is correct and the naming clarifies intent.

## Required Changes

**None.** The implementation is correct. The diagnostic fires for all exact integer true-division forms, the legacy pass fixture is removed, the fail fixture is in place, and docs are updated. The `is_exact_to_float_integer_type` duplication is a non-blocking cleanup note.

## Non-blocking Notes

1. **Code duplication (`is_exact_to_float_integer_type` vs `is_integer_type`)**: Both functions have identical logic. Consider either `type alias is_exact_to_float_integer_type = is_integer_type` or inlining `is_integer_type` at the `/` check. Not a correctness issue.

2. **`BigInt / BigInt` is now rejected**: The pre-existing `BigInt / BigInt -> BigInt` (floor division) path at `check.rs:244-246` is now shadowed by the exact-integer check at line 220, so `BigInt / BigInt` returns `INT0006` rather than `BigInt`. This is acceptable — `BigInt` is a temporary transition alias (INT-2B) and the long-term model has no separate `bigint` type. The test at `check.rs:666` (`BigInt, BigInt`) already encodes this as expected behavior.

## Verdict

**APPROVED for this milestone.** The PR correctly fails closed on exact integer true division (`int / int`, `LiteralInt / LiteralInt`, `FixedInt / FixedInt`, `BigInt / BigInt`) with active `SIFR-INT-0006` instead of silently lowering through a float cast. The legacy `codegen_int_division.sifr` pass fixture is removed, the fail fixture is in place, and all docs are updated. No blocking issues found.
