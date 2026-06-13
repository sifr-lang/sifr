# wave_psp_bytes_0 CPython Traceability Matrix

Wave: `wave_psp_bytes_0`  
Scope: architecture lock for first-class `bytes` and binary-surface foundation

## CPython Harvest Inputs

- `Objects/bytesobject.c`
- `Objects/bytearrayobject.c`
- `Lib/test/test_bytes.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_io/`

## Adopt / Adapt / Waive (Wave 0 Lock)

| CPython family | Sifr surface direction | State | Owning wave |
| --- | --- | --- | --- |
| `test_bytes` immutable bytes constructors/index/slice/iter/equality/concat | first-class `bytes` type surface (typed, immutable) | `adapted` (planned) | `wave_psp_bytes_1` + `wave_psp_bytes_2` |
| `test_bytes` mutable `bytearray` / subclass-heavy object-model features | out of scope for this phase | `unsupported` | locked permanent diff |
| `test_bytes` `memoryview` and buffer-protocol families | out of scope for this phase | `unsupported` | locked permanent diff |
| `test_base64` binary payload pathways | rewire to canonical `bytes` carrier in downstream contracts | `adapted` (planned) | `wave_psp_bytes_3` |
| `test_hashlib` binary payload pathways | rewire to canonical `bytes` carrier in downstream contracts | `adapted` (planned) | `wave_psp_bytes_3` |
| `test_io` binary file-handle pathways | rewire to canonical `bytes` carrier in downstream contracts | `adapted` (planned) | `wave_psp_bytes_3` |

## Explicit Waivers Locked in Wave 0

- `bytearray` mutable object-model parity remains `unsupported`.
- `memoryview` and general buffer protocol remain `unsupported`.
- Bytes-like duck typing / implicit coercions remain `unsupported`.
- Non-UTF-8 codec matrices remain `unsupported` in this phase.

## Local Fixture Anchors (Wave 0)

- Positive lock fixture: `crates/sifr/tests/e2e/pass/bytes_helpers.sifr`
- Demos:
  - `demos/bytes_roundtrip/main.sifr`
  - `demos/text_and_bytes/main.sifr`
- Negative lock fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_bytearray_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_memoryview_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_buffer_protocol_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_implicit_str_bytes_coercion_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_bytes_subclass_unsupported.sifr`

M1 update: non-UTF-8 codec labels moved from unsupported bytes-scope behavior to the production text/i18n encoding substrate. Coverage now lives in `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr`.
