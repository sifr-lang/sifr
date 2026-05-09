# INT-3 Generic Addable Output Boundary - Review Pass 2

## Findings

### Pass-1 Required Changes — Both Verified

1. **`addition_type_var` narrowed to same-TypeVar operands** (`expression_operators.rs:109`):

   ```rust
   fn addition_type_var<'a>(left: &'a Type, right: &'a Type) -> Option<&'a str> {
       match (left.resolve_alias(), right.resolve_alias()) {
           (Type::TypeVar(left), Type::TypeVar(right)) if left == right => Some(left.as_str()),
           _ => None,
       }
   }
   ```

   Both operands must resolve to the same `Type::TypeVar`. Mixed cases (`T + int32`) return `None` and do not trigger the bound check. This correctly scopes the INT-3 boundary to the canonical `T + T -> T` case.

2. **Fail fixture correctly expects `SIFR-PROTO-0001`**: The call site diagnostic is `SIFR-PROTO-0001` ("type 'int32' does not implement protocol 'Addable' required by type parameter 'T'") because the `Addable` bound check fires before the return-type unification. The fixture is correct.

### Semantic Correctness

- **Unbounded `T + T` without `Addable` bound**: Rejected with `SIFR-TYPE-0005` (`TYPE_UNSUPPORTED_OPERATOR`) at the `left + right` expression, covered by `test_unbounded_generic_addition_requires_addable_bound`.
- **`Addable` with exact `int`**: Accepted, covered by `test_addable_generic_addition_accepts_int`.
- **`int32` at `Addable` call site**: Correctly rejected — `int32 + int32 -> int` (ordinary fixed-width promotion), which is not assignable to `T = int32`, so `int32` does not satisfy `Addable`'s output requirement.
- **`int32` direct addition without generics**: Not affected by this change; `int32 + int32 -> int` continues to work through the fixed-width promotion path from INT-3 scalar promotion.

### Validation Results

- `cargo test -p sifr_hir generic_addition -- --nocapture`: 2 passed (unbounded rejection + bounded exact-int acceptance)
- `cargo test -p sifr --test e2e test_e2e_fail -- fixed_width_generic_addable_output_boundary --nocapture`: 1 passed
- `cargo fmt --check`: clean
- `scripts/run_all_tests.sh --profile quick`: 52.42s wall time, `e1bf653aaa770517`, all lanes green

### Non-blocking Notes

- The pre-existing CFG ICE from an unrelated fixture surfaces in the e2e fail run (`crates/sifr_hir/src/cfg.rs:540`). This is not introduced by this PR.
- `uint64` and `usize` remain excluded from the temporary fixed-width promotion path, as in prior INT-3 work.

## Required Changes

None.

## Verdict

Approved for this milestone. The implementation correctly:
- Rejects unbounded `T + T` without `Addable` at the expression level
- Accepts `Addable` generic functions with exact `int` operands
- Rejects `int32` at an `Addable` call site because `int32 + int32 -> int`, not `int32`
- Uses the call-site protocol diagnostic (`SIFR-PROTO-0001`) rather than the expression-level type operator diagnostic, because the bound check fires at the call site during type unification
