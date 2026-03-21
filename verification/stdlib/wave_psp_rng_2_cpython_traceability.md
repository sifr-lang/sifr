# wave_psp_rng_2 CPython Traceability Matrix

Wave: `wave_psp_rng_2`  
Scope: advanced hash + binary surface expansion for `sifr.hashlib` and `sifr.base64`

## CPython Harvest Inputs

- `Lib/test/test_hashlib.py`
- `Lib/test/test_base64.py`

## Adopt / Adapt / Waive (Wave 2)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_hashlib` bytes-native digest/object model | ship `HashObject.digest() -> bytes`, `digest_bytes() -> bytes`, `update_bytes(bytes)`, and bytes-first constructor `new_bytes(name, data: bytes = b"")` while keeping `update(str)`/`hexdigest()` compatibility paths | `adapted` (shipped) | `lib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` |
| `test_hashlib` str-facing compatibility | preserve existing string constructor/update/hexdigest behavior on top of bytes-native state ownership | `adapted` (shipped) | `lib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_e1_core_modules_numeric_patterns_crypto.sifr` |
| `test_base64` binary-oriented API behavior | add bytes-native base64 encode/decode (`b64encode_bytes`, `b64decode_bytes`, standard/urlsafe bytes variants) while retaining text helpers | `adapted` (shipped) | `lib/sifr/base64.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr`, `demos/ad_hoc_rng_wave2_hashlib_base64_bytes_demo.sifr` |
| `test_hashlib` SHA3/SHAKE constructor families | keep SHA3/SHAKE object constructors unsupported in this wave pending explicit dependency/surface expansion | `unsupported` | `lib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/fail/phase_psp_rng_2_sha3_object_model_unsupported.sifr` |

## Dependency Audit Note (Wave 2)

- Active generated-runtime hash dependencies in this wave remain:
  - `sha2 = "0.10"`
  - `md5 = "0.7"`
  - `sha1 = "0.10"`
  - `blake2 = "0.10"`
- No SHA3/SHAKE dependency is currently registered for generated runtime crates in this wave.
- Outcome: SHA3/SHAKE remains explicitly unsupported and guarded by typed boundaries.

## Explicit Waivers / Boundaries After Wave 2

- SHA3/SHAKE constructor families remain explicitly unsupported (`sha3_256_obj`, `sha3_512_obj`, `shake_128_obj`, `shake_256_obj`).
- Weighted `random.choices(weights=...)` and `SystemRandom.getstate/setstate` remain outside wave-2 scope (tracked by wave-1 governance).

## Local Fixture Anchors (Wave 2)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr`
- Demo:
  - `demos/ad_hoc_rng_wave2_hashlib_base64_bytes_demo.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_rng_2_sha3_object_model_unsupported.sifr`
