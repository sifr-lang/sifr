## decimal_types

OK

## decimal_arithmetic

OK

## decimal_conversions

Initial reviewer notes:

> 1. `int_from_bigdecimal` and `bigint_from_bigdecimal` extracted the integer part by splitting `BigDecimal::to_string()` on `"."`, which is fragile and non-idiomatic compared to numeric extraction from the decimal representation itself.
> 2. `BigDecimal(Decimal("12.3400"))` allegedly lost canonical trailing-zero form in the Rust output.
> 3. The `DecimalConversionError` messages were slightly inconsistent across decimal and bigdecimal conversions.
> 4. `bigint_from_decimal` originally routed through an `i64` intermediary instead of doing a direct decimal-to-bigint conversion.

Disposition: partially accepted. I accepted the substance of notes 1 and 4 and rewrote the conversion helpers to use numeric extraction from `Decimal::mantissa()` / `Decimal::scale()` and `BigDecimal::as_bigint_and_exponent()`, removing both the string-splitting path and the `i64` intermediary. Note 2 was not accepted because the validated Rust output already preserved `12.3400` exactly. Note 3 was not treated as a blocker because the demo only asserts the bigdecimal out-of-range message on the exercised failure path.
