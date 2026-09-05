# CPython Parity Fixture Format (Baseline)

This document defines the baseline fixture rules for CPython-derived parity
tests. It is intentionally minimal so it can be applied uniformly across stdlib parity
ports without requiring higher-order function support.

Use a simple vector table shape that maps cleanly to Sifr lists:

- `inputs: list[str]`
- `expected: list[str]`
- `actual: list[str]` (computed during test run)

## Baseline vector rules

1. Keep vectors deterministic and order-stable.
2. Encode CPython expected outputs literally in `expected`.
3. Compute `actual` in a loop over `inputs`.
4. Compare with `sifr.test.assert_vector_eq(...)`.
5. For error-path fixtures, maintain parallel boolean vectors (`expected_ok`/`actual_ok`) and use `assert_bool_vector_eq(...)`.

## Fixture structure rules

These structure rules apply to CPython parity fixtures. They are not a blanket style
rule for unrelated non-parity e2e fixtures.

Enforcement model:
- These rules are intended to be enforced through normal module validation and readiness validation.
- A dedicated structural-validation script is optional future hardening, not a baseline
  requirement for CPython parity readiness.

1. Organize a module's parity corpus into a small number of semantic fixtures rather than
   one oversized catch-all fixture or a large number of microscopic files.
2. Keep `main()` as the orchestration layer only. Put behavior into small helper
   functions or clearly separated canonical vector sections when the fixture would
   otherwise become monolithic.
3. Keep positive-path, negative-path, and safety-adaptation assertions explicit and easy
   to locate inside the approved parity scope.
4. Keep fixture ordering, test data, and assertion grouping deterministic so failures are
   reproducible and validator-friendly.
5. Reuse this baseline format unless a module-specific extension is explicitly justified
   in the readiness tracking docs.
