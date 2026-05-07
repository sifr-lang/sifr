# Review: INT-3 Fixed-Width Scalar Promotion (PR #1860)

## 1. Verdict: APPROVED

The PR correctly implements the conservative first INT-3 slice for ordinary fixed-width scalar `+`, `-`, and `*` promotion to source-level `int`. The implementation is sound, tests pass, and the design intent is followed.

---

## 2. Blocking Findings

**None.** No blocking issues found.

---

## 3. Non-Blocking Follow-ups

### 3.1 Test coverage gap for non-int32 fixed-width types

The e2e pass fixture (`fixed_width_scalar_arithmetic_promotion.sifr`) and HIR lowering test (`test_fixed_width_scalar_add_sub_mul_promote_to_int`) only exercise `int32`. The type system test (`test_fixed_width_integer_add_sub_mul_promote_to_int`) covers `int32` and `uint8`.

The implementation logic in `fixed_width_promotes_to_current_int` correctly includes all fixed-width families except `U64` and `USize`:
- **Included**: `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `ISize`
- **Excluded**: `U64`, `USize` (per design: "uint64, usize ... remain later INT-3 work")

**Gap**: No explicit positive test coverage for `int64`, `int16`, `int8`, `uint16`, `uint32`, or `isize` arithmetic promotion. Recommend adding a broader e2e pass fixture or expanding the existing one to cover all promoted fixed-width families before closing INT-3.

**Files affected**:
- `crates/sifr/tests/e2e/pass/fixed_width_scalar_arithmetic_promotion.sifr`
- `crates/sifr_hir/src/lower/expressions_tests.rs:504`
- `crates/sifr_type_system/src/check.rs:639`

### 3.2 Duplicate `fixed_width_promotes_to_current_int` function

The same `fixed_width_promotes_to_current_int` logic appears in two crates:

- `crates/sifr_type_system/src/check.rs:28` — type-system-level check
- `crates/sifr_codegen/src/lower_expr.rs:1352` — codegen-level check

Both return `!matches!(fixed, FixedIntType::U64 | FixedIntType::USize)` with identical semantics. This duplication is acceptable for now since the crates are separate and the logic is simple, but it creates a maintenance risk: if a new fixed-width type is added to `FixedIntType`, both copies must be updated in sync. Consider extracting to a shared constant or a minimal shared-types crate before INT-3 closure.

### 3.3 Pre-existing clippy warnings unrelated to this PR

`cargo clippy -p sifr_hir -- -D warnings` produces two errors in `crates/sifr_hir/src/lower/integer_nonzero_guards.rs` (lines 26 and 82). These exist on the base branch and are not introduced by this PR. They are mentioned here for awareness only and do not block this PR.

---

## 4. Validation Notes

### 4.1 Local validation

```bash
scripts/run_all_tests.sh --profile quick
```

**Result**: All tests pass. Quick profile completed in ~54s with 24 e2e pass tests, 2 type system tests, and 14 HIR tests.

### 4.2 Type system logic

`is_exact_or_fixed_integer_type` in `check.rs:23` correctly identifies `Int`, `LiteralInt`, and all fixed-width types except `U64` and `USize` as promoting to `int`. The `type_check_binary_op` modifications for `+`, `-`, `*` return `Type::Int` when both operands satisfy this predicate. Logic is sound.

### 4.3 Codegen logic

`is_promoted_fixed_width_integer_binop` in `lower_expr.rs:1482` correctly detects `+`, `-`, `*` between int and promoted fixed-width types. `try_lower_promoted_integer_operand_expr` in `lower_expr.rs:1710` casts fixed-width operands to `RustType::I64` before the operation.

Generated output for the pass fixture:
```rust
let left: i32 = 2000000000i32;
let right: i32 = 2000000000i32;
let total: i64 = (left as i64) + (right as i64);
let diff: i64 = (right as i64) - (left as i64);
let product: i64 = (2 as i64) * (right as i64);
```

This is consistent with the conservative slice: `int` is represented as `i64` in codegen, and fixed-width operands are explicitly cast before the operation.

### 4.4 Excluded types

`U64` and `USize` are correctly excluded from promotion. The type system test `test_uint64_integer_add_waits_for_sifrint_promotion` confirms `U64 + U64` produces an error. This matches the review scope: "uint64, usize ... remain later INT-3 work."

### 4.5 Design consistency

The implementation follows `internal_docs/integer_model.md`:

- "Ordinary fixed-width scalar arithmetic promotes to exact `int`" — `int32 + int32 -> int` ✓
- "`uint64`, `usize` remain later INT-3 work" — both are excluded ✓
- "It intentionally only handles fixed-width integer families that are losslessly representable by the current generated int backend" — `i64` can represent all promoted fixed-width types except `U64` ✓

---

## 5. Summary

| Area | Assessment |
|------|-----------|
| Correctness | ✓ Type rules and codegen lowering are correct |
| Codegen soundness | ✓ Fixed-width operands cast to i64 before `+`, `-`, `*`; result type is `i64` which maps to `int` |
| Test coverage | Partial — only int32/uint8 explicitly tested; other fixed-width types logically covered but not directly tested |
| Design consistency | ✓ Follows the conservative first-slice intent |
| Excluded types (U64, USize) | ✓ Correctly excluded per review scope |
| Pre-existing issues | Clippy warnings in `integer_nonzero_guards.rs` are unrelated to this PR |

**Recommendation**: Approve for merge. Address the non-blocking test coverage gap (3.1) before closing INT-3.
