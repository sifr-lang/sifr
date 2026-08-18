# stdlib_parity_rng_1 CPython Traceability Matrix

Capability: `stdlib_parity_rng_1`
Scope: deterministic RNG state/object model readiness for `sifr.random`

## CPython Harvest Inputs

- `Lib/test/test_random.py`

## Adopt / Adapt / Waive (Capability 1)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_random` deterministic state/object model (`Random`, `seed`, `getstate`, `setstate`) | ship typed deterministic mutable-state object model (`RandomState(version, state_words, index, gauss_next)`) with MT19937-style state words and deterministic replay via state round-trips | `adapted` (shipped) | `stdlib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/stateful_random.sifr` |
| `test_random` module-level delegation parity | module-level `seed`, `getstate`, `setstate`, `randrange`, `randint`, `random`, `choice`, `choices`, `sample`, `shuffle`, `gauss`, `uniform`, and `randbytes` delegate through one shared module RNG state rules | `adapted` (shipped) | `stdlib/sifr/random.sifr`, `demos/random_state/main.sifr` |
| `test_random` `SystemRandom` state-boundary semantics | host-random `SystemRandom` remains non-deterministic; `getstate`/`setstate` are explicitly unsupported and typed as `Result[..., ValueError]` | `adapted` (shipped boundary) | `stdlib/sifr/random.sifr`, `crates/sifr/tests/e2e/fail/system_random_state_unsupported.sifr` |
| `test_random` byte generation (`randbytes`) | ship deterministic bytes-generation surface returning first-class `bytes` | `adapted` (shipped) | `stdlib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/stateful_random.sifr` |
| `test_random` weighted-distribution `choices(weights=...)` | keep weighted branch unsupported in this implementation pass | `unsupported` | `crates/sifr/tests/e2e/fail/random_choices_weights_unsupported.sifr` |

## CPython `test_random.py` Case Mapping (Capability 1)

| CPython test case | Sifr adaptation direction | Local anchor(s) | Coverage status |
| --- | --- | --- | --- |
| `TestBasicOps.test_saverestore` | deterministic sequence replay through `getstate`/`setstate` | `crates/sifr/tests/e2e/pass/stateful_random.sifr`, `demos/random_state/main.sifr` | covered |
| `SystemRandom_TestBasicOps.test_saverestore` | host-backed `SystemRandom` state export/import stays unsupported | `crates/sifr/tests/e2e/fail/system_random_state_unsupported.sifr` | covered (adapted typed boundary) |
| `TestBasicOps.test_randbytes` / `MersenneTwister_TestBasicOps.test_randbytes` | deterministic bytes generation on typed first-class `bytes` | `crates/sifr/tests/e2e/pass/stateful_random.sifr` | covered |
| `MersenneTwister_TestBasicOps.test_setstate_first_arg` and `test_setstate_middle_arg` | invalid state version/index/shape reject through typed `ValueError` | `stdlib/sifr/random.sifr`, `crates/sifr_codegen/src/intrinsics/random.rs` (`lower_random_module_set_state`), `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |
| `TestBasicOps.test_choices` / `test_choices_algorithms` | module-level `choices(items, k)` deterministic selection path is replayable under explicit seeding, with large-sample frequency bounds validating near-uniform selection behavior | `stdlib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered (seeded deterministic + distribution-bound adaptation) |
| `TestBasicOps.test_gauss` | `Random.gauss` cached-pair semantics are exercised via state inspection (`gauss_next` set/cleared) and deterministic replay under fixed seed | `stdlib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered (typed deterministic adaptation) |
| `TestBasicOps.test_randrange_nonunit_step`, `test_randint`, `test_choice`, `test_sample`, `test_shuffle`, `test_gauss`, `test_getrandbits` | existing random helper-family parity remains active while capability-set-1 module delegation moves those helpers onto deterministic shared state | `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr`, `crates/sifr/tests/e2e/pass/stdlib_random.sifr`, `crates/sifr/tests/e2e/pass/stateful_random.sifr` | covered (legacy + capability-set-1 delegation replay) |

## Explicit Waivers / Boundaries After Capability 1

- `choices(weights=...)` remains explicitly unsupported for this capability.
- `SystemRandom` state export/import parity remains explicitly unsupported (`getstate`/`setstate`).

## Local Fixture Anchors (Capability 1)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/stateful_random.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` (post-readiness CPython state-validation adaptation)
- Demo:
  - `demos/random_state/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/system_random_state_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/random_choices_weights_unsupported.sifr`
