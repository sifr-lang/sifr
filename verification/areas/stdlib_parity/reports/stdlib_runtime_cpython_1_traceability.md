# stdlib_parity_runtime_1 CPython Traceability Matrix

Wave: `stdlib_parity_runtime_1`
Scope: `io` and in-memory stream hierarchy (`BytesIO`, `StringIO`)

## CPython Harvest Inputs

- `Lib/test/test_io/` (`StringIO`, `BytesIO`, base stream lifecycle families)
- `Lib/test/test_memoryio.py` (when mirrored under local CPython tree)

## Adopt / Adapt / Waive (Wave 1)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_io` `StringIO` read/write/seek/tell/getvalue behavior | ship typed in-memory `StringIO` with deterministic cursor semantics and typed error surface | `adapted` | `crates/sifr/tests/e2e/pass/in_memory_streams.sifr` |
| `test_io` `BytesIO` read/write/seek/tell/getvalue behavior | ship typed in-memory `BytesIO` over first-class `bytes` with deterministic cursor semantics | `adapted` | `crates/sifr/tests/e2e/pass/in_memory_streams.sifr` |
| `test_io` binary file-handle entry paths | keep binary payload contract on first-class `bytes`; add `open_binary(...)` typed entry | `adapted` | `demos/in_memory_streams/main.sifr` |
| full `_pyio` inheritance graph and advanced buffered classes | keep explicitly deferred from wave 0 lock | `unsupported` | `crates/sifr/tests/e2e/fail/pyio_inheritance_unsupported.sifr` |

## Explicit Waivers / Boundaries (Wave 1)

- Advanced `_pyio` hierarchy (`BufferedReader`/`BufferedWriter`/`BufferedRWPair`/`BufferedRandom`) remains `unsupported`.
- File-handle `seek`/`tell` remain explicitly unsupported in this wave until dedicated file-position intrinsic support is introduced.
- `BinaryFileHandle.read_bytes(size=...)` currently treats `size` as compatibility-only and reads the full remaining stream; partial-size reads are deferred until dedicated binary file-position/read-range intrinsics land.
- `StringIO.read_bytes()` and `BytesIO.write(str)` remain intentionally rejected by typed surfaces.
- Negative seek positions on `StringIO`/`BytesIO` are explicitly rejected with `IOError` (no silent clamp-to-zero behavior).

## Local Fixture Anchors (Wave 1)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/in_memory_streams.sifr`
- Demo:
  - `demos/in_memory_streams/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/stringio_read_bytes_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytesio_text_write_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/pyio_inheritance_unsupported.sifr`
