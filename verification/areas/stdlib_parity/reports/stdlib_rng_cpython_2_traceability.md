# stdlib_parity_rng_2 CPython Traceability Matrix

Capability: `stdlib_parity_rng_2`
Scope: advanced hash + binary surface expansion for `sifr.hashlib` and `sifr.base64`

## CPython Harvest Inputs

- `Lib/test/test_hashlib.py`
- `Lib/test/test_base64.py`

## Adopt / Adapt / Waive (Capability 2)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_hashlib` bytes-native digest/object model | `HashObject.digest() -> bytes`, `HashObject.update(bytes)`, `new(name, data: bytes = b"")`, and bytes-native named constructors form the single canonical contract; `hexdigest()` is the distinct text-output operation | `adapted` (shipped) | `stdlib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` |
| `test_hashlib` text input | callers explicitly encode text before hashing; no constructor or update method performs implicit UTF-8 conversion | `intentional-diff` (shipped) | `stdlib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`, `crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr` |
| `test_base64` binary-oriented API behavior | retain distinct bytes-native `b64encode_bytes`/`b64decode_bytes` and URL-safe bytes operations beside the text signatures; same-operation standard/encodebytes aliases are removed | `adapted` (shipped) | `stdlib/sifr/base64.sifr`, `crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr`, `demos/binary_hashing/main.sifr` |
| `test_base64` invalid-bytes decode boundaries | typed decode failures on invalid base64 bytes payloads remain explicit `ParseError` boundaries for both standard and urlsafe bytes decode paths | `adapted` (shipped) | `crates/sifr/tests/e2e/pass/base64_bytes_decode_errors.sifr`, `crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr` |
| `test_hashlib` SHA3/SHAKE constructor families | keep SHA3/SHAKE object constructors unsupported in this implementation pass pending explicit dependency/surface expansion | `unsupported` | `stdlib/sifr/hashlib.sifr`, `crates/sifr/tests/e2e/fail/sha3_object_model_unsupported.sifr` |

## Dependency Audit Note (Capability 2)

- Active generated-runtime hash dependencies in this implementation pass are pinned to:
  - `sha2 = "0.11.0"`
  - `md5 = "0.8.0"`
  - `sha1 = "0.11.0"`
  - `blake2 = "0.10.6"`
- No SHA3/SHAKE dependency is currently registered for generated runtime crates in this implementation pass.
- Outcome: SHA3/SHAKE remains explicitly unsupported and guarded by typed boundaries.

## CPython `test_hashlib.py` / `test_base64.py` Case Mapping (Capability 2)

| CPython test case | Sifr adaptation direction | Local anchor(s) | Coverage status |
| --- | --- | --- | --- |
| `HashLibTestCase.test_new_upper_to_lower` | constructor accepts uppercase algorithm names while normalizing to canonical internal algorithm state | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`, `stdlib/sifr/hashlib.sifr` | covered |
| `HashLibTestCase.test_case_sha1_1` / `test_case_sha512_1` | deterministic digest vectors for shipped algorithms (`sha1`, `sha512`) | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` | covered |
| `HashLibTestCase.test_copy` | `copy_hash` preserves source hash state while allowing independent updates on copied object | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` | covered |
| `HashLibTestCase.test_large_update` | incremental `HashObject.update(...)` chunked writes must match one-shot constructor digest on large multi-block payloads | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`, `stdlib/sifr/hashlib.sifr` | covered |
| `TestBase64.test_b64encode` / `test_b64decode` and roundtrip matrix families | shipped text+bytes base64 encode/decode surfaces preserve deterministic roundtrip behavior | `crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`, `crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr` | covered |
| `TestBase64.test_b64decode_invalid_chars` / strict validate families | invalid payloads remain typed `ParseError` boundaries for standard and urlsafe decode paths | `crates/sifr/tests/e2e/pass/cpython_base64_strictness_subset.sifr`, `crates/sifr/tests/e2e/pass/base64_bytes_decode_errors.sifr` | covered |
| `TestBase64.test_b32decode_casefold` | lowercase Base32 payloads decode through current intrinsic behavior (casefold-style acceptance) | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`, `stdlib/sifr/base64.sifr` | covered |

## Explicit Waivers / Boundaries After Capability 2

- SHA3/SHAKE constructor families remain explicitly unsupported (`sha3_256_obj`, `sha3_512_obj`, `shake_128_obj`, `shake_256_obj`).
- Weighted `random.choices(weights=...)` and `SystemRandom.getstate/setstate` remain outside capability-set-2 scope (tracked by capability-set-1 governance).
- Key-derivation helpers `pbkdf2_hmac` and `scrypt` remain explicitly unsupported in this implementation pass.
- ASCII85/Base85/Z85 codec families (`a85*`, `b85*`, `z85*`) remain explicitly unsupported.

## Local Fixture Anchors (Capability 2)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/base64_bytes_decode_errors.sifr`
  - `crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` (post-readiness hashlib case/vector adaptation)
- Demo:
  - `demos/binary_hashing/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/sha3_object_model_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/hashlib_pbkdf2_hmac_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/hashlib_scrypt_unsupported.sifr`
