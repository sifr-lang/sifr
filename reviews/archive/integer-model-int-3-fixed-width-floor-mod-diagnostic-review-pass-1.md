# INT-3 Fixed-Width Floor/Modulo Diagnostic Scaffold — Review Pass 1

## Summary

PR #1863 extends the INT-0005 failure scaffold so fixed-width integer floor division (`//`) and modulo (`%`) fail closed instead of lowering through ordinary Rust operators. The change generalizes the diagnostic message from "exact integer" to "integer" to reflect the broader scope. Existing exact-`int` non-zero proof behavior for `//` and `%` is preserved.

**Verdict: approved — no blocking issues.**

---

## Findings

### Correctness: `involves_fixed_width_integer` fires INT-0005 unconditionally

The new `involves_fixed_width_integer` check (lines 76–80 in `integer_failure_diagnostics.rs`) fires INT-0005 whenever **either** operand of a `//` or `%` expression is a fixed-width integer type, regardless of whether the divisor is a non-zero literal. This is the correct fail-closed behavior per the integer model:

> Fixed-width division, floor division, modulo, exponentiation, and shifts use the same no-silent-failure rule. Ordinary scalar `int32 // int32` promotes to `Result[int, DivisionError]`.

The implementation correctly treats:
- `uint8 // uint8` → INT-0005 (no lowering through Rust `/`)
- `uint8 // 2` (literal) → INT-0005 (fail closed even with syntactically non-zero literal divisor)
- `int // int` with unproven divisor → INT-0005 (existing behavior preserved)
- `int // 2` (non-zero literal) → **no INT-0005** (exact-int non-zero proof still works)

The new `is_exact_or_fixed_integer_like` helper is correctly composed: it returns `true` for both exact `int`/`LiteralInt` and all `FixedInt` variants, while `involves_fixed_width_integer` additionally requires at least one side to be a fixed-width type. This means pure-`int` vs pure-`int` traffic takes the existing proven-nonzero path (lines 26–32), while any fixed-width involvement bypasses that path entirely and goes straight to the error.

### Diagnostic message generalization

The message was changed from:
```
"exact integer division or modulo requires handling Result[int, DivisionError] unless the divisor is proven non-zero"
```
to:
```
"integer division or modulo requires handling Result[int, DivisionError] unless the compiler can prove this operation is safe"
```

This is appropriate. The "exact" qualifier was misleading now that fixed-width operands also trigger INT-0005, and "divisor is proven non-zero" was specific to the exact-`int` non-zero proof path. The new phrasing covers both exact and fixed-width cases correctly.

### Augassign path parity

`exact_int_augassign_requires_handling` was updated with the same logic structure (lines 36–58). The fixed-width check fires before the exact-int check, and the exact-int branch still contains the `!is_proven_nonzero_integer_expr` guard, preserving augassign behavior for exact `int` operands.

### Test coverage

| Test | What it checks | Coverage |
|---|---|---|
| `test_fixed_width_floor_division_requires_handling_even_with_literal_divisor` | `uint8 // 2` → INT-0005 | Binary op, literal divisor, fixed-width |
| `test_fixed_width_mod_augassign_requires_handling` | `uint8 %= divisor` → INT-0005 | Augassign, variable divisor, fixed-width |
| `test_exact_int_division_by_nonzero_literal_still_lowers_as_int` | `int // 2` → no error | Exact int non-zero literal proof intact |
| `test_exact_int_division_by_unproven_divisor_has_int0005` | `int // divisor` → INT-0005 | Exact int unproven divisor |
| `test_exact_int_mod_augassign_by_unproven_divisor_has_int0005` | `int %= divisor` → INT-0005 | Exact int augassign unproven divisor |
| `e2e test_e2e_fail -- fixed_width_division_requires_handling` | `uint8 // uint8` → INT-0005 | E2E fixture |

The augassign test only covers `%=` but the `//=` case is tested by the existing exact-int augassign tests and the code path is symmetric (`base_op` covers both `//` and `%`). Non-blocking.

### No regressions in existing behavior

The PR does not touch:
- `sifr_codegen` — no codegen change needed at this scaffold stage since INT-0005 is a type-checking/lowering diagnostic that gates code generation
- The nonzero guard proof mechanism (`is_proven_nonzero_integer_expr`, `integer_nonzero_guards.rs`)
- Stdlib lowering exemption path (`ctx.is_stdlib_lowering()`)

---

## Required Changes

**None.** No blocking issues found.

---

## Non-blocking Notes

1. **`//=` not explicitly covered in the unit test**: The augassign test `test_fixed_width_mod_augassign_requires_handling` covers `%=` but not `//=` for fixed-width. The `base_op` discriminator correctly handles both, but a symmetric `test_fixed_width_div_augassign_requires_handling` would harden coverage. Not a blocker.

2. **INT-3 checklist item**: The PR checklist tracks "Continue the broader `Type::Int` migration beyond direct helper/local expression rewrites: full `Result[int, DivisionError]` expression/codegen integration and non-literal proven-nonzero analysis still need support" under the ongoing INT-1 item. The scaffold coverage added here is correctly scoped and does not need to resolve that broader item.

3. **Documentation cleanup**: The diagnostic message generalization in `codes.rs`, `SIFR-INT-0005.md`, `diagnostic-codes.md`, and `internal_docs/diagnostic_codes.md` is consistent. No stale references remain.

---

## Verdict

The PR correctly implements the INT-3 milestone for fixed-width floor division and modulo diagnostic scaffolding. Fixed-width `//` and `%` now fail closed with INT-0005 regardless of divisor, exact-`int` non-zero literal proof behavior is preserved, and test coverage is adequate. No blocking issues.

**Approved for this milestone.**
