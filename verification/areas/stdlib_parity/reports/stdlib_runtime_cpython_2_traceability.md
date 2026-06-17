# stdlib_parity_runtime_2 CPython Traceability Matrix

Capability: `stdlib_parity_runtime_2`
Scope: `tempfile` and `zipfile` object lifecycle expansion

## CPython Harvest Inputs

- `Lib/test/test_tempfile.py` (`NamedTemporaryFile`, `TemporaryDirectory`, deterministic cleanup families)
- `Lib/test/test_zipfile/` (`ZipInfo`, metadata helpers, extract/open read-handle families)

## Adopt / Adapt / Waive (Capability 2)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_tempfile` `NamedTemporaryFile(delete=...)` lifecycle | ship deterministic owner wrapper with explicit `close()/cleanup()` error surfaces and best-effort scope-exit cleanup | `adapted` | `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr` |
| `test_tempfile` `TemporaryDirectory` lifecycle | ship deterministic wrapper with explicit `cleanup()` and panic-free scope-exit cleanup | `adapted` | `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr` |
| `test_zipfile` bytes payload I/O (`write`/`write_bytes`/`read`/`read_bytes`/`namelist`) | extend archive payload flow to first-class `bytes` with byte-native intrinsics (`zip_add_file_bytes`, `zip_read_file_bytes`) | `adapted` | `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr`, `demos/zipfile_io/main.sifr` |
| `test_zipfile` metadata/object helpers (`infolist`, `getinfo`) | keep deferred while `ZipInfo` is present as a typed placeholder surface | `unsupported` | `lib/sifr/zipfile.sifr`, `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr` |
| `test_zipfile` archive extraction and read handle (`open`, `extract`, `extractall`) | keep deferred in this implementation pass; methods remain explicit `Result` errors | `unsupported` | `lib/sifr/zipfile.sifr`, `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr`, `demos/zipfile_io/main.sifr` |
| streamed zip write-handle classes (`ZipExtFile` write-mode ecosystems) | keep deferred in this capability | `unsupported` | `crates/sifr/tests/e2e/fail/zip_ext_file_unsupported.sifr` |
| additional compression-option constants/families (for example `ZIP_BZIP2`) | keep explicitly unsupported until backend support and governance expansion land | `unsupported` | `crates/sifr/tests/e2e/fail/zip_bzip2_constant_unsupported.sifr` |

## Explicit Waivers / Boundaries (Capability 2)

- `SpooledTemporaryFile` remains explicitly unsupported from capability-set-0 lock and is still enforced by `crates/sifr/tests/e2e/fail/spooled_tempfile_unsupported.sifr`.
- `ZipFile.open(...)`, `ZipFile.infolist()`, `ZipFile.getinfo()`, `ZipFile.extract()`, and `ZipFile.extractall()` are intentionally deferred in this implementation pass and return explicit `Result` errors.
- `ZipInfo` is intentionally narrowed to deterministic fields (`filename`, `file_size`, `compress_type`) and does not claim full CPython metadata parity yet.
- `ZipFile.write(...)` and `ZipFile.write_bytes(...)` enforce explicit write/append mode allowlist (`"w"`, `"a"`, `"wb"`, `"ab"`); mixed/invalid modes are rejected explicitly.
- `ZipReadHandle.read_bytes(size)` now handles `size < 0` explicitly as read-all (matching CPython-style negative-size semantics).
- current class-lowering does not allow constructor-time `Result` propagation for wrapper allocation paths; creation operations remain best-effort in `__init__`, while lifecycle cleanup/error surfaces (`close`/`cleanup`) remain explicit `Result` ruless.

## Local Fixture Anchors (Capability 2)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr`
- Demo:
  - `demos/zipfile_io/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/zip_ext_file_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/zip_bzip2_constant_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/spooled_tempfile_unsupported.sifr`
