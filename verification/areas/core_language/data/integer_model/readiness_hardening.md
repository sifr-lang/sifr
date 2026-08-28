# Integer Model Readiness Hardening

INT-8 locks the integer-model work with concrete local evidence instead of a CI-only or manual checklist.

## Performance Gate

- Fixture: `verification/areas/performance/fixtures/sifr_int_loop.sifr`
- Runner: `verification/areas/performance/tools/run_integer_model_readiness_perf.py`
- Command: `python3 verification/areas/performance/tools/run_integer_model_readiness_perf.py`

The runner first executes the Sifr fixture through the normal CLI. It then builds a temporary Rust probe under `target/integer_model_readiness_perf_probe` that depends on the local `sifr_runtime` crate and checks:

- small `SifrInt` accumulation loop allocation count is zero;
- small `SifrInt` counter loop allocation count is zero;
- small `SifrInt` hashing loop allocation count is zero;
- small `SifrInt` accumulation throughput is within the ratified INT-8 threshold.

The performance benchmark rules is not active yet, so INT-8 records a local ratified threshold here rather than wiring a broader benchmark governance lane. The long-term target remains within `2x` of an equivalent optimized Rust `i64` loop once the performance benchmark rules owns statistically governed performance budgets. Until that tooling exists, the INT-8 blocking threshold is `10x`; the stricter `2x` target is tracked as the future performance budget target, not an unratified blocker for this readiness record.

## Fuzz And Property Coverage

Integer-specific fuzz/property seeds are checked into `verification/areas/fuzz_property/seeds/`:

- `integer_external_boundaries_seed.sifr` covers JSON web/string integer profile handling and JSON digit-limit rejection.
- `integer_fixed_width_helpers_seed.sifr` covers fixed-width checked/wrapping/saturating/overflowing helper surfaces.

The seeds are registered in both the deterministic property manifest and the mutation-smoke seed corpus, so the verification hardening framework runs them.

## Panic-Shape Sweep

Generated-code panic-shape coverage remains owned by the existing create-pr, merge, and release validation profiles:

- `scripts/run_all_tests.sh --profile create-pr` runs unit and representative e2e coverage, including integer result paths and driver panic-boundary tests.
- `scripts/run_all_tests.sh` runs the authoritative PR profile with selected hardening suites.
- Full integer-model readiness must also run `scripts/run_all_tests.sh` and the explicit INT-8 performance runner.
