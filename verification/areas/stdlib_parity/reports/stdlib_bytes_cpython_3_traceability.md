# stdlib_parity_bytes_3 CPython Traceability Matrix

Capability: `stdlib_parity_bytes_3`
Scope: downstream binary-carrier rules adoption and governance readiness

## CPython Harvest Inputs

- `Lib/test/test_io/`
- `Lib/test/test_base64.py`
- `Lib/test/test_hashlib.py`

## Adopt / Adapt / Waive (Capability 3 Readiness)

| CPython family | Sifr surface direction | State | Local regression/demo |
| --- | --- | --- | --- |
| `test_io` binary file-handle pathways | first-class `bytes` at `FileHandle.read_bytes()` / `FileHandle.write_bytes(...)` boundaries | `adapted` (closed for carrier rules) | `crates/sifr/tests/e2e/pass/bytes_file_io.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_io_subset.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr`<br>`demos/bytes_file_io/main.sifr` |
| `test_base64` binary payload pathways | keep existing text-friendly APIs while preserving bytes conversion boundary via first-class `bytes` (`bytes.from_hex`, `bytes.decode`) | `adapted` | `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr` |
| `test_hashlib` binary payload pathways | downstream rules anchored to first-class `bytes`; current shipped API remains `str`-input/hex-digest with explicit unsupported bytes-native digest families | `adapted` (rules aligned; bytes-native API still waived) | `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr` |

## Classified waivers (real remaining binary set after locale-formatting capability)

| Surface | State | Rationale |
| --- | --- | --- |
| `bytearray` mutable object-model parity | `unsupported` | This capability ships immutable first-class `bytes` only; mutable byte buffers remain deferred. |
| `memoryview` and general buffer-protocol families | `unsupported` | Zero-copy view semantics and generic buffer protocol are intentionally deferred. |
| Internal bytes backend storage (`Type::Bytes` lowering to `Vec<i64>` instead of `Vec<u8>`) | `intentional-diff` | Public rules is first-class immutable `bytes`; current codegen backend uses `Vec<i64>` to align `int` index/iteration semantics while byte-domain invariants are enforced at construction/file-conversion boundaries. |
| Non-UTF-8 codec matrices for `str.encode` / `bytes.decode` | `unsupported` | Capability scope intentionally keeps UTF-8-only typed conversion behavior. |
| `hashlib` bytes-native update/digest constructor families (`update_bytes`, `digest() -> bytes`, `new_bytes`) | `unsupported` | Current runtime closes string-input/hex-digest object model; bytes-native digest expansion is deferred to RNG/crypto successor scope. |
| Direct bytes-oriented base64 entrypoints as primary public parity claim | `unsupported` | Current readiness keeps text-friendly surface with explicit bytes conversion boundaries; full bytes-first API matrix remains successor work. |

## Local fixture anchors (locale-formatting capability)

- Pass fixture:
  - `crates/sifr/tests/e2e/pass/bytes_file_io.sifr`
- Fail fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_write_bytes_rejects_int_list.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_read_bytes_not_list.sifr`
- Demo:
  - `demos/bytes_file_io/main.sifr`
