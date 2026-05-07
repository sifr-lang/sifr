# INT-3 Generic Addable Output Boundary - Review Pass 1

## Findings

### Required Changes

1. Diagnostic code mismatch in the fail fixture.

The new fail fixture expected `SIFR-PROTO-0001`, but the compiler emits `SIFR-TYPE-0005` at the generic `left + right` expression before the call-site protocol bound check can fire. Fix by changing the fixture to expect `SIFR-TYPE-0005`, or by changing the implementation to emit a protocol diagnostic.

2. `addition_type_var` is too broad for mixed TypeVar/non-TypeVar addition.

The helper returned a type variable when only one side was a TypeVar, which made the guard apply to mixed expressions such as `T + int32`. The INT-3 boundary being closed here is the canonical `T + T -> T` case, so the helper should require both operands to be the same TypeVar.

## Required Changes

- Change `crates/sifr/tests/e2e/fail/fixed_width_generic_addable_output_boundary.sifr` to expect `SIFR-TYPE-0005`.
- Restrict `addition_type_var` to the same-TypeVar-on-both-sides case.

## Non-blocking Notes

- The e2e fail run still prints a pre-existing caught CFG ICE from an unrelated fixture.
- `uint64` and `usize` remain excluded from the temporary fixed-width promotion path until broader `SifrInt` promotion support lands.

## Verdict

Approved for this milestone after the two required changes above are applied and validation is rerun.
