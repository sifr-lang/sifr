# CPython Parity Fixture Format (Baseline)

Use a simple vector table shape that maps cleanly to Sifr lists:

- `inputs: list[str]`
- `expected: list[str]`
- `actual: list[str]` (computed during test run)

Guidelines:

1. Keep vectors deterministic and order-stable.
2. Encode CPython expected outputs literally in `expected`.
3. Compute `actual` in a loop over `inputs`.
4. Compare with `sifr.test.assert_vector_eq(...)`.
5. For error-path fixtures, maintain parallel boolean vectors (`expected_ok`/`actual_ok`) and use `assert_bool_vector_eq(...)`.

This format is intentionally minimal so it can be applied uniformly across stdlib parity ports without requiring higher-order function support.
