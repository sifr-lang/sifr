# stdlib_parity_runtime_0 CPython Traceability Matrix

Capability: `stdlib_parity_runtime_0`
Scope: architecture lock for runtime/file-object parity expansion

## CPython Harvest Inputs

- `Lib/test/test_io/`
- `Lib/test/test_tempfile.py`
- `Lib/test/test_zipfile/`
- `Lib/test/test_logging.py`
- `Lib/test/test_time.py`
- `Lib/test/test_timeit.py`
- `Lib/test/test_subprocess.py`

## Adopt / Adapt / Waive (Capability 0 Lock)

| CPython family | Sifr surface direction | State | Owning capability |
| --- | --- | --- | --- |
| `test_io` sealed stream hierarchy, text/binary handles, in-memory stream object families | ship adapted sealed hierarchy (`IOBase`/`TextIOBase`/`BinaryIOBase`, `FileHandle`, `BinaryFileHandle`, `BytesIO`, `StringIO`) | `adapted` (planned) | `stdlib_parity_runtime_1` |
| `test_tempfile` object wrappers and cleanup semantics | ship deterministic ownership wrappers (`NamedTemporaryFile`, `TemporaryDirectory`) with explicit cleanup rules | `adapted` (planned) | `stdlib_parity_runtime_2` |
| `test_zipfile` archive object helpers and metadata/file-handle behavior | expand beyond narrow create/write/read subset with explicit read-handle boundary | `adapted` (planned) | `stdlib_parity_runtime_2` |
| `test_logging` handler/formatter hierarchy and deterministic process-local behavior | expand host-safe deterministic logger model without dynamic graph/thread-order guarantees | `adapted` (planned) | `stdlib_parity_runtime_3` |
| `test_time` clock/object surfaces (`struct_time`, `gmtime`, `localtime`, `mktime`, constants) | ship adapted typed-safe object/time APIs where host/runtime permits | `adapted` (planned) | `stdlib_parity_runtime_3` |
| `test_timeit` callable timing APIs | ship callable-only timing model; reject string-eval execution | `adapted` (planned) | `stdlib_parity_runtime_3` |
| `test_subprocess` sync process boundary and option matrix | expand synchronous option matrix; keep async process lifecycle waived | `adapted` (planned) | `stdlib_parity_runtime_4` |

## Explicit Waivers Locked in Capability 0

- Full `_pyio` inheritance parity remains `unsupported` in this capability.
- Async `subprocess.Popen` lifecycle remains `unsupported`.
- `logging.dictConfig` and dynamic handler graph mutation remain `unsupported`.
- Thread-aware logging ordering guarantees remain `unsupported`.
- Logging file-sink IO failures are currently `fail-soft` (suppressed rather than surfaced) as a capability-set-0 host-limited baseline; locale-formatting capability owns final logging error-policy readiness.
- `SpooledTemporaryFile` remains `unsupported`.
- String-eval `timeit` execution remains `unsupported`.
- Timezone mutation helpers remain `unsupported`.

## Local Fixture Anchors (Capability 0)

- Positive lock fixture: `crates/sifr/tests/e2e/pass/runtime_file_basics.sifr`
- Demos:
  - `demos/file_streams/main.sifr`
  - `demos/tempfiles_and_zip/main.sifr`
  - `demos/binary_files/main.sifr`
- Negative lock fixtures:
  - `crates/sifr/tests/e2e/fail/pyio_inheritance_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/async_popen_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/logging_dictconfig_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/logging_loggeradapter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/spooled_tempfile_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/timeit_string_eval_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/timezone_mutation_unsupported.sifr`
