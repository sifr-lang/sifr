

Review is satisfied.

**Summary of the diff:**

1. **Shared helper** — `FixedIntType::supports_current_scalar_promotion_to_int()` added in `types.rs:137-139` with a doc comment marking it as temporary. Policy: all fixed-width types except `U64` and `USize`.

2. **check.rs** — `is_exact_or_fixed_integer_type()` at line 29 now calls the shared helper. The local `fixed_width_promotes_to_current_int()` function is gone.

3. **lower_expr.rs** — `is_fixed_width_int_like_simple()` at line 1355 now calls the shared helper. The local duplicate function is gone.

4. **Separation preserved** — `supports_current_int_builtin_widening()` (6 types) remains distinct and narrower than `supports_current_scalar_promotion_to_int()` (8 types).

5. **Coverage** — New test `test_fixed_width_current_scalar_promotion_policy()` in `types.rs` iterates all 10 `FixedIntType` variants, asserting 8 promote and 2 (`U64`, `USize`) do not.

No blockers. Deduplication is clean, scope is focused, and the validation results you ran confirm correctness.
