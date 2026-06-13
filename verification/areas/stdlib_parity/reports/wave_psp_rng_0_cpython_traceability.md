# wave_psp_rng_0 CPython Traceability Matrix

Wave: `wave_psp_rng_0`  
Scope: architecture lock and governance freeze for stateful RNG, bytes-native crypto, and residual polish ownership

## CPython Harvest Inputs

- `Lib/test/test_random.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_statistics.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Adopt / Adapt / Waive (Wave 0 Lock)

| CPython family | Sifr surface direction | State | Owning wave |
| --- | --- | --- | --- |
| `test_random` deterministic object/state semantics (`seed`, `getstate`, `setstate`, `Random`, `SystemRandom`) | replace current stateless host-random wrappers with typed deterministic state model (`RandomState`) + module-global delegation | `adapted` (planned) | `wave_psp_rng_1` |
| `test_random` bytes generation (`randbytes`) | ship bytes-native random byte generation on first-class raw-byte-backed `bytes` | `adapted` (planned) | `wave_psp_rng_1` |
| `test_hashlib` digest/object model parity | migrate from hex-string digest aliasing to bytes-native digest/object APIs (`digest`, `digest_bytes`, `update_bytes`, `new_bytes`) | `adapted` (planned) | `wave_psp_rng_2` |
| `test_hashlib` algorithm inventory expansion (`sha3`/`shake`) | close only algorithms available in runtime dependency stack; keep unsupported algorithms explicit | `adapted` (planned) | `wave_psp_rng_2` |
| `test_base64` binary-oriented API behavior | anchor binary-oriented parity on first-class `bytes` carrier while preserving text helpers | `adapted` (planned) | `wave_psp_rng_2` |
| `test_statistics` advanced deterministic helpers | close narrow float/int advanced helpers only; preserve deterministic typed errors | `adapted` (planned) | `wave_psp_rng_3` |
| `test_textwrap` residual formatter options | close only explicit residual waived options if low risk | `adapted` (planned) | `wave_psp_rng_3` |
| `test_html` residual top-level polish | keep package parser ecosystem out of scope; close only explicit residual top-level gaps | `adapted` (planned) | `wave_psp_rng_3` |

## Explicit Waivers / Boundaries Locked in Wave 0

- Full CPython buffer protocol, `memoryview`, and mutable `bytearray` ecosystem parity remain `unsupported` for this phase.
- `SystemRandom` state export/import parity remains `unsupported`.
- Decimal/Fraction/context-sensitive `statistics` semantics remain `unsupported`.
- Package-wide `html.parser` ecosystem remains `unsupported`.

## Local Fixture Anchors (Wave 0)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/random_hashing_and_text.sifr`
- Demo:
  - `demos/random_hashing/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/random_choices_weights_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/system_random_state_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/sha3_object_model_unsupported.sifr`

Historical note:
- `crates/sifr/tests/e2e/fail/phase_psp_rng_0_random_state_object_model_unsupported.sifr` was retired after `wave_psp_rng_1` shipped deterministic state/object parity.
- `crates/sifr/tests/e2e/fail/phase_psp_rng_0_hashlib_bytes_digest_api_unsupported.sifr` was retired after `wave_psp_rng_2` shipped bytes-native `hashlib` object parity.
- `crates/sifr/tests/e2e/fail/phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` was retired after `wave_psp_rng_3` shipped `TextWrapper` `max_lines`/`placeholder` support.
