# wave_psp_runtime_2 CPython Traceability Matrix

Wave: `wave_psp_runtime_2`  
Scope: `tempfile` and `zipfile` object lifecycle expansion

## CPython Harvest Inputs

- `Lib/test/test_tempfile.py` (`NamedTemporaryFile`, `TemporaryDirectory`, deterministic cleanup families)
- `Lib/test/test_zipfile/` (`ZipInfo`, metadata helpers, extract/open read-handle families)

## Adopt / Adapt / Waive (Wave 2)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_tempfile` `NamedTemporaryFile(delete=...)` lifecycle | ship deterministic owner wrapper with explicit `close()/cleanup()` error surfaces and best-effort scope-exit cleanup | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr` |
| `test_tempfile` `TemporaryDirectory` lifecycle | ship deterministic wrapper with explicit `cleanup()` and panic-free scope-exit cleanup | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr` |
| `test_zipfile` bytes payload I/O (`write`/`write_bytes`/`read`/`read_bytes`/`namelist`) | extend archive payload flow to first-class `bytes` with byte-native intrinsics (`zip_add_file_bytes`, `zip_read_file_bytes`) | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr`, `demos/ad_hoc_runtime_wave2_tempfile_zipfile_lifecycle_demo.sifr` |
| `test_zipfile` metadata/object helpers (`infolist`, `getinfo`) | keep deferred while `ZipInfo` is present as a typed placeholder surface | `unsupported` | `lib/sifr/zipfile.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr` |
| `test_zipfile` archive extraction and read handle (`open`, `extract`, `extractall`) | keep deferred in this wave; methods remain explicit `Result` errors | `unsupported` | `lib/sifr/zipfile.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr`, `demos/ad_hoc_runtime_wave2_tempfile_zipfile_lifecycle_demo.sifr` |
| streamed zip write-handle classes (`ZipExtFile` write-mode ecosystems) | keep deferred in this phase | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_ext_file_unsupported.sifr` |
| additional compression-option constants/families (for example `ZIP_BZIP2`) | keep explicitly unsupported until backend support and governance expansion land | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_bzip2_constant_unsupported.sifr` |

## Explicit Waivers / Boundaries (Wave 2)

- `SpooledTemporaryFile` remains explicitly unsupported from wave-0 lock and is still enforced by `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr`.
- `ZipFile.open(...)`, `ZipFile.infolist()`, `ZipFile.getinfo()`, `ZipFile.extract()`, and `ZipFile.extractall()` are intentionally deferred in this wave and return explicit `Result` errors.
- `ZipInfo` is intentionally narrowed to deterministic fields (`filename`, `file_size`, `compress_type`) and does not claim full CPython metadata parity yet.
- `ZipFile.write(...)` and `ZipFile.write_bytes(...)` enforce write/append mode (`"w"`/`"a"`); read-mode writes are rejected explicitly.

## Local Fixture Anchors (Wave 2)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr`
- Demo:
  - `demos/ad_hoc_runtime_wave2_tempfile_zipfile_lifecycle_demo.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_ext_file_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_bzip2_constant_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr`
