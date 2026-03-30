# Wave 3 Batch 94 Review Pass 1

- `demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs`
  - OK: scaffold correctly records the explicit `0..=28` rounding-scale bound and the observed `[E2507]` diagnostic.
- `demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs`
  - OK: scaffold correctly documents the exact-construction rule rejecting `Decimal(float_value)` and the observed `[E2505]` diagnostic.
- `demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs`
  - OK: scaffold correctly documents the mixed `decimal`/`bigdecimal` arithmetic prohibition and the observed `[E2504]` diagnostic.

Result: `OK` for all three files. No blockers.
