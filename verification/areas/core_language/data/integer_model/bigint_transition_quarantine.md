# Bigint Transition Fixture Quarantine

`bigint` is a temporary transition alias while the exact source-level `int`
migration completes. It is not the canonical arbitrary-precision spelling.

The following fixtures intentionally keep coverage for the transition alias
until the alias-removal PR deletes or rewrites them:

- `crates/sifr/tests/e2e/pass/bigint_arithmetic.sifr`
- `crates/sifr/tests/e2e/pass/bigint_as_dict_key.sifr`
- `crates/sifr/tests/e2e/pass/bigint_basic.sifr`
- `crates/sifr/tests/e2e/pass/bigint_comparison.sifr`
- `crates/sifr/tests/e2e/pass/bigint_factorial.sifr`
- `crates/sifr/tests/e2e/pass/bigint_large_value.sifr`
- `crates/sifr/tests/e2e/pass/bigint_overflow_conversion.sifr`
- `crates/sifr/tests/e2e/pass/bigint_to_int.sifr`
- `crates/sifr/tests/e2e/pass/generic_accumulate_bigint.sifr`
- `crates/sifr/tests/e2e/pass/generic_counter_bigint.sifr`
- `crates/sifr/tests/e2e/pass/int_to_bigint.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_heapq_consolidated.sifr`
- `crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr`
- `crates/sifr/tests/e2e/fail/bigint_int_mixed_comparison.sifr`
- `crates/sifr/tests/e2e/fail/bigint_overflow_conversion.sifr`

The create-pr and merge e2e pass manifests must not include the transition-only
`bigint_arithmetic` fixture. Full e2e validation may continue to discover the
quarantined files until alias removal so regression coverage is preserved.

`SIFR-TYPE-0006` remains active only for these transition-alias paths. It should
be retired in the same PR that removes public `bigint` from the type system.
