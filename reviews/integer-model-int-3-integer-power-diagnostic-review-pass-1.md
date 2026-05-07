# INT-3 Integer Exponentiation Diagnostic Scaffold Review Pass 1

## Findings

1. The `SIFR-INT-0005` extension is correct. The diagnostic now covers `**` in both the message template and error constant. The guard in `integer_failure_diagnostics.rs` dispatches exponentiation through the same diagnostic path as division/modulo, keeping the existing code structure intact.
2. Exact `int` negative literal exponentiation is rejected. `2 ** -1` fires `SIFR-INT-0005` because the exponent is not proven non-negative, matching the design rule that integer `2 ** -1` must not implicitly become `0.5`.
3. Exact `int` non-negative literal exponentiation still lowers. The regression coverage proves `2 ** 3` and `value **= 2` both remain valid.
4. Exact `int` exponentiation by a variable is rejected, correctly failing closed for runtime-dependent exponents.
5. Fixed-width exponentiation fails closed. `uint8` power expressions and augmented assignment emit `SIFR-INT-0005`, so these paths do not silently become float or lower through unchecked exponent casts.
6. The stdlib lowering exemption is preserved.
7. The diagnostic message updates are consistent across the registry and generated docs surfaces.
8. The new HIR tests cover negative literal exponent, unproven variable exponent, fixed-width base with literal exponent, fixed-width augassign, and the allowed non-negative exact literal case.
9. The new e2e fail fixtures are minimal and correctly target `SIFR-INT-0005`.

## Required Changes

None.

## Non-blocking Notes

1. The internal Rust identifier is still named `INT_EXACT_DIVISION_REQUIRES_HANDLING`, which is a naming artifact from the earlier division/modulo-only implementation. The public diagnostic code and message are correct.
2. `is_proven_nonnegative_integer_expr` only handles direct integer literal shapes. More complex const-expression proofs can be added in a later PR.
3. One unproven-exponent test checks code and range rather than the full message. The negative-literal test already asserts the updated message template.

## Verdict

Approved for this milestone. The scaffold correctly rejects unsafe exact and fixed-width integer exponentiation paths while preserving non-negative exact literal exponent lowering. Local quick validation passed with report signature `e1bf653aaa770517`.
