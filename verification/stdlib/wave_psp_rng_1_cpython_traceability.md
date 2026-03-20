# wave_psp_rng_1 CPython Traceability Matrix

Wave: `wave_psp_rng_1`  
Scope: deterministic RNG state/object model closure for `sifr.random`

## CPython Harvest Inputs

- `Lib/test/test_random.py`

## Adopt / Adapt / Waive (Wave 1)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_random` deterministic state/object model (`Random`, `seed`, `getstate`, `setstate`) | ship typed deterministic mutable-state object model (`RandomState(version, state_words, index, gauss_next)`) with MT19937-style state words and deterministic replay via state round-trips | `adapted` (shipped) | `lib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr` |
| `test_random` module-level delegation parity | module-level `seed`, `getstate`, `setstate`, `randrange`, `randint`, `random`, `choice`, `choices`, `sample`, `shuffle`, `gauss`, `uniform`, and `randbytes` delegate through one shared module RNG state contract | `adapted` (shipped) | `lib/sifr/random.sifr`, `demos/ad_hoc_rng_wave1_stateful_object_model_demo.sifr` |
| `test_random` `SystemRandom` state-boundary semantics | host-random `SystemRandom` remains non-deterministic; `getstate`/`setstate` are explicitly unsupported and typed as `Result[..., ValueError]` | `adapted` (shipped boundary) | `lib/sifr/random.sifr`, `crates/sifr/tests/e2e/fail/phase_psp_rng_1_system_random_state_unsupported.sifr` |
| `test_random` byte generation (`randbytes`) | ship deterministic bytes-generation surface returning first-class `bytes` | `adapted` (shipped) | `lib/sifr/random.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr` |
| `test_random` weighted-distribution `choices(weights=...)` | keep weighted branch unsupported in this wave | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr` |

## Explicit Waivers / Boundaries After Wave 1

- `choices(weights=...)` remains explicitly unsupported for this phase.
- `SystemRandom` state export/import parity remains explicitly unsupported (`getstate`/`setstate`).

## Local Fixture Anchors (Wave 1)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr`
- Demo:
  - `demos/ad_hoc_rng_wave1_stateful_object_model_demo.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_rng_1_system_random_state_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr`
