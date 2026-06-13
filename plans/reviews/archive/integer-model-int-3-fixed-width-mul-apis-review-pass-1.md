# INT-3 Fixed-Width Mul APIs — Review Pass 1

## Summary

PR #1870 implements `checked_mul`, `wrapping_mul`, `saturating_mul`, and `overflowing_mul`
for same-width fixed-width integer operands. The implementation is consistent with the
canonical design in `internal_docs/integer_model.md` (lines 158-164), follows the same
explicit-representation-preserving API pattern established by PRs #1868 and #1869 for add/sub,
and is safe to merge.

## HIR Typing

| Method | Output type | Correct? |
|---|---|---|
| `checked_mul` | `Result[fixed-width, OverflowError]` | ✓ |
| `wrapping_mul` | `fixed-width` | ✓ |
| `saturating_mul` | `fixed-width` | ✓ |
| `overflowing_mul` | `tuple[fixed-width, bool]` | ✓ |

Mixed-width argument rejection (line 33-46 of `fixed_width_arithmetic_methods.rs`) uses
`arg_ty.resolve_alias() != fixed_ty.resolve_alias()` which correctly compares the underlying
types. The test `test_fixed_width_mul_api_rejects_mixed_width_argument` validates this
for `uint8.wrapping_mul(uint16)` at the HIR level.

## Codegen

- `checked_mul`: `receiver.checked_mul(rhs).ok_or_else(|| OverflowError{ message })` — no
  unwrap/expect in user paths.
- `wrapping_mul`, `saturating_mul`, `overflowing_mul`: direct primitive method calls via
  `lower_primitive_method`. No panics in user paths.

The `args.first()?` in both `lower_primitive_method` and `lower_checked_method` is safe
because the HIR type system enforces exactly 1 argument for all four methods.

## Canonical Design Compliance

- Rust-style names (line 164: "Rust-style names are intentional") ✓
- Representation-preserving output types match add/sub pattern ✓
- No-panic guarantee (line 491: "Integer overflow inside Sifr-generated fixed-width helper
  methods must not panic in user-triggerable paths") ✓

## Test Coverage

The e2e fixture exercises overflow (16*16 for uint8), non-overflow (3*4 for uint8), and
all four method variants. HIR unit tests verify correct output types and mixed-width
rejection. The validation already run by the author covers this scope.

## Minor Observations (Non-Blockers)

1. In the generated Rust for `checked_mul`, the error message closure body has a
   redundant `.to_string().to_string()` — the inner call converts `&str` to `String`,
   and the outer call is `String.to_string()` which is a no-op. Harmless but unnecessary.

2. The `||` closure parameter in `ok_or_else(|| OverflowError{...})` is unused (the
   parameter slot exists because `ok_or_else` signature is `FnOnce() -> E`, not
   `FnOnce(T) -> E`). No correctness impact.

## Required Changes

None.
