# wave_psp_bytes_1 CPython Traceability Matrix

Wave: `wave_psp_bytes_1`  
Scope: core first-class `bytes` type, object-model foundation, and immutability enforcement

## CPython Harvest Inputs

- `Lib/test/test_bytes.py`
- `Lib/test/test_iter.py` (bytes iteration behavior family)

## Adopt / Adapt / Waive (Wave 1 Foundation)

| CPython family | Sifr surface direction | State | Local regression/demo |
| --- | --- | --- | --- |
| `test_bytes` core immutable behaviors (`len`, equality, concat, indexing, slicing, iteration) | first-class `bytes` value type with explicit typed operations | `adapted` (closed for wave-1 scope) | `crates/sifr/tests/e2e/pass/bytes_basics.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_bytes_subset.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_bytes.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_bytes_safety.sifr` |
| bytes mutation families (`append`, item assignment) | compile-time immutability enforcement for `bytes` | `adapted` (closed for wave-1 scope) | `crates/sifr/tests/e2e/fail/bytes_append_unsupported.sifr`<br>`crates/sifr/tests/e2e/fail/bytes_subscript_assignment_unsupported.sifr` |
| compatibility helper parity (`lib/sifr/bytes.sifr`) | keep compatibility exports while moving canonical semantics to first-class `bytes` | `adapted` | `demos/bytes_basics/main.sifr`<br>`demos/bytes_iteration/main.sifr` |

## Classified waivers carried from wave 1

| Surface | State | Rationale |
| --- | --- | --- |
| Mutable `bytearray` object-model parity | `unsupported` | Wave 1 closes immutable `bytes` only. |
| `memoryview` and generic buffer protocol | `unsupported` | View/protocol semantics are intentionally deferred. |
| Conversion families (`bytes(size)`, `from_ints`, `from_hex`, codec-aware encode/decode) | `adapted` in successor wave | Closed in `wave_psp_bytes_2`; not claimed as wave-1 complete. |

## Local fixture anchors (wave 1)

- Pass fixture:
  - `crates/sifr/tests/e2e/pass/bytes_basics.sifr`
- Fail fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_append_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_subscript_assignment_unsupported.sifr`
- Demos:
  - `demos/bytes_basics/main.sifr`
  - `demos/bytes_iteration/main.sifr`
