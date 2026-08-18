# stdlib_parity_bytes_2 CPython Traceability Matrix

Capability: `stdlib_parity_bytes_2`
Scope: typed conversion surfaces and compatibility delegation on first-class `bytes`

## CPython Harvest Inputs

- `Lib/test/test_bytes.py` (constructor/conversion families)
- `Lib/test/test_codecs.py` (encode/decode codec behavior families)
- `Lib/test/test_base64.py` (hex/text conversion adjacency)

## Adopt / Adapt / Waive (Capability 2 Conversion Readiness)

| CPython family | Sifr surface direction | State | Local regression/demo |
| --- | --- | --- | --- |
| constructor and conversion families (`bytes(size)`, `bytes.from_ints`, `bytes.from_hex`) | explicit typed conversion APIs returning `Result` with safe failure semantics | `adapted` (closed for capability-set-2 scope) | `crates/sifr/tests/e2e/pass/bytes_constructors.sifr`<br>`crates/sifr/tests/e2e/pass/bytes_conversion_errors.sifr` |
| text/binary boundary (`str.encode`, `bytes.decode`) | explicit UTF-8-only typed boundary with `Result` errors | `adapted` | `demos/bytes_constructors/main.sifr`<br>`demos/bytes_errors/main.sifr` |
| public conversion ownership | first-class `bytes` constructors and methods are canonical; no compatibility module remains | `adapted` (shipped) | `crates/sifr/tests/e2e/pass/bytes_constructors.sifr`<br>`crates/sifr/tests/e2e/pass/bytes_helpers.sifr` |

## Classified waivers carried from Unicode core capability

| Surface | State | Rationale |
| --- | --- | --- |
| Non-UTF-8 codec matrices for encode/decode | `unsupported` | Capability-set-2 scope intentionally enforces UTF-8-only conversion behavior. |
| Implicit text/binary coercions | `unsupported` | Conversion remains explicit and typed; no implicit coercion is introduced. |
| Mutable/view/buffer binary families (`bytearray`, `memoryview`, buffer protocol) | `unsupported` | These remain out of scope for this capability and are tracked across bytes governance ledgers. |

## Local fixture anchors (Unicode core capability)

- Pass fixtures:
  - `crates/sifr/tests/e2e/pass/bytes_constructors.sifr`
  - `crates/sifr/tests/e2e/pass/bytes_conversion_errors.sifr`
- Fail fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_constructor_non_int.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_from_hex_non_string.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_from_ints_non_int_list.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_encode_non_string_codec.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_decode_non_string_codec.sifr`
- Demos:
  - `demos/bytes_constructors/main.sifr`
  - `demos/bytes_errors/main.sifr`
