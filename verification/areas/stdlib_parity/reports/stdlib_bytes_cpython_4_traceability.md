# stdlib_parity_bytes_4 CPython Traceability Matrix

Capability: `stdlib_parity_bytes_4`
Scope: raw-byte backend storage and bytes/list lowering disentanglement

## CPython Harvest Inputs

- `Lib/test/test_bytes.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_io/`

## Adopt / Adapt / Waive (Capability 4 Readiness)

| CPython family | Sifr surface direction | State | Local regression/demo |
| --- | --- | --- | --- |
| `test_bytes` immutable bytes storage/index/iteration families | typed `bytes` remains immutable and index/iteration continue yielding `int`, with raw-byte backend (`Vec<u8>`) and widening only at read boundaries | `adapted` (closed for capability-set-4 scope) | `crates/sifr/tests/e2e/pass/bytes_hex_and_binary_io.sifr`<br>`demos/binary_storage/main.sifr` |
| `test_io` binary file-handle pathways | typed `bytes` file paths (`read_bytes`/`write_bytes`) keep public behavior and remove internal widened-storage bounce | `adapted` | `crates/sifr/tests/e2e/pass/bytes_hex_and_binary_io.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_io_subset.sifr` |
| `test_base64` binary payload boundaries | keep current text-friendly API while preserving first-class typed bytes conversion boundaries on raw-byte backend | `adapted` | `crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr` |
| `test_hashlib` binary payload boundaries | keep current string-input/hex-digest shipped surface while preserving bytes-carrier rules for successor bytes-native digest APIs | `adapted` (rules preserved) | `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` |

## Classified waivers (real remaining binary set after translation-catalog capability)

| Surface | State | Rationale |
| --- | --- | --- |
| `bytearray` mutable object-model parity | `unsupported` | This capability closes immutable first-class `bytes` only; mutable byte buffers remain deferred. |
| `memoryview` and general buffer-protocol families | `unsupported` | Zero-copy view semantics and generic buffer protocol remain intentionally deferred. |
| Non-UTF-8 codec matrices for `str.encode` / `bytes.decode` | `unsupported` | Current readiness intentionally keeps UTF-8-only typed conversion behavior. |
| `hashlib` bytes-native update/digest constructor families (`update_bytes`, `digest() -> bytes`, `new_bytes`) | `unsupported` | Current runtime closes string-input/hex-digest object model; bytes-native digest expansion is deferred to RNG/crypto successor scope. |
| Direct bytes-oriented base64 entrypoints as primary public parity claim | `unsupported` | Current readiness keeps text-friendly surface with explicit bytes conversion boundaries; full bytes-first API matrix remains successor work. |

## Local fixture anchors (translation-catalog capability)

- Pass fixtures:
  - `crates/sifr/tests/e2e/pass/bytes_hex_and_binary_io.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr`
- Fail fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_from_ints_non_int_list.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_write_bytes_rejects_int_list.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_read_bytes_not_list.sifr`
- Demo:
  - `demos/binary_storage/main.sifr`
