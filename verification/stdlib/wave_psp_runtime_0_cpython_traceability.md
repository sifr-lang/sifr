# wave_psp_runtime_0 CPython Traceability Matrix

Wave: `wave_psp_runtime_0`  
Scope: architecture lock for runtime/file-object parity expansion

## CPython Harvest Inputs

- `Lib/test/test_io/`
- `Lib/test/test_tempfile.py`
- `Lib/test/test_zipfile/`
- `Lib/test/test_logging.py`
- `Lib/test/test_time.py`
- `Lib/test/test_timeit.py`
- `Lib/test/test_subprocess.py`

## Adopt / Adapt / Waive (Wave 0 Lock)

| CPython family | Sifr surface direction | State | Owning wave |
| --- | --- | --- | --- |
| `test_io` sealed stream hierarchy, text/binary handles, in-memory stream object families | ship adapted sealed hierarchy (`IOBase`/`TextIOBase`/`BinaryIOBase`, `FileHandle`, `BinaryFileHandle`, `BytesIO`, `StringIO`) | `adapted` (planned) | `wave_psp_runtime_1` |
| `test_tempfile` object wrappers and cleanup semantics | ship deterministic ownership wrappers (`NamedTemporaryFile`, `TemporaryDirectory`) with explicit cleanup rules | `adapted` (planned) | `wave_psp_runtime_2` |
| `test_zipfile` archive object helpers and metadata/file-handle behavior | expand beyond narrow create/write/read subset with explicit read-handle boundary | `adapted` (planned) | `wave_psp_runtime_2` |
| `test_logging` handler/formatter hierarchy and deterministic process-local behavior | expand host-safe deterministic logger model without dynamic graph/thread-order guarantees | `adapted` (planned) | `wave_psp_runtime_3` |
| `test_time` clock/object surfaces (`struct_time`, `gmtime`, `localtime`, `mktime`, constants) | ship adapted typed-safe object/time APIs where host/runtime permits | `adapted` (planned) | `wave_psp_runtime_3` |
| `test_timeit` callable timing APIs | ship callable-only timing model; reject string-eval execution | `adapted` (planned) | `wave_psp_runtime_3` |
| `test_subprocess` sync process boundary and option matrix | expand synchronous option matrix; keep async process lifecycle waived | `adapted` (planned) | `wave_psp_runtime_4` |

## Explicit Waivers Locked in Wave 0

- Full `_pyio` inheritance parity remains `unsupported` in this phase.
- Async `subprocess.Popen` lifecycle remains `unsupported`.
- `logging.dictConfig` and dynamic handler graph mutation remain `unsupported`.
- Thread-aware logging ordering guarantees remain `unsupported`.
- Logging file-sink IO failures are currently `fail-soft` (suppressed rather than surfaced) as a wave-0 host-limited baseline; wave 3 owns final logging error-policy closure.
- `SpooledTemporaryFile` remains `unsupported`.
- String-eval `timeit` execution remains `unsupported`.
- Timezone mutation helpers remain `unsupported`.

## Local Fixture Anchors (Wave 0)

- Positive lock fixture: `crates/sifr/tests/e2e/pass/phase_psp_runtime_0_architecture_lock.sifr`
- Demos:
  - `demos/runtime_stream_hierarchy_contract/main.sifr`
  - `demos/runtime_tempfile_zip_lifecycle/main.sifr`
  - `demos/runtime_bytes_binary_io_contract/main.sifr`
- Negative lock fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_dictconfig_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_loggeradapter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timeit_string_eval_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timezone_mutation_unsupported.sifr`
