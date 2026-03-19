# `wave_psp_runtime_0` Architecture Lock (Runtime and File-Object Parity Expansion)

Phase: `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`  
Execution ledger: `issues/ad-hoc-runtime-and-file-object-parity-expansion-execution.md`

## Objective

Lock one explicit runtime/file-object architecture contract before feature-expansion waves begin.

This lock prevents later waves from re-inventing stream hierarchy, ownership cleanup, host limitations, or bytes-carrier assumptions.

## Locked Public Contract Snapshot

| Surface | Locked direction for this phase |
| --- | --- |
| `io` hierarchy | Use sealed Sifr hierarchy (`IOBase`, `TextIOBase`, `BinaryIOBase`, `FileHandle`, `BinaryFileHandle`, `BytesIO`, `StringIO`) as the sole expansion target for this phase. |
| Binary stream payloads | All binary stream and archive payloads consume first-class `bytes` from the predecessor bytes phase; no `list[int]` fallback contracts are allowed. |
| Lifecycle model | RAII scope-exit cleanup is default; explicit cleanup APIs use typed `Result` on fallible operations; implicit cleanup is best-effort and panic-free. |
| `tempfile` wrappers | `NamedTemporaryFile` and `TemporaryDirectory` are wave-owned targets; lifecycle ownership must be explicit and deterministic. |
| `zipfile` handles | `ZipFile.open(..., "r")` must produce binary-read handles only; streamed write-handle parity remains out of scope in this phase. |
| `timeit` model | Callable-only timing model is locked; string-eval execution is explicitly out of scope. |

## Implementation Notes (Wave 0)

### FileHandle lifecycle generalization note

Current `FileHandle` already defines a context-style enter/exit boundary (`__enter__`/`__exit__`) and explicit `close()`.  
Wave 1 must generalize this same cleanup pattern into the sealed hierarchy instead of introducing a second cleanup model.

Required continuity rule:
- `FileHandle`/binary handle classes and in-memory wrappers must share one lifecycle contract: explicit close/flush/seek/tell/closed state APIs plus panic-free scope-exit cleanup behavior.

### Raw-byte contract note

The predecessor bytes phase locked first-class `bytes` on raw-byte backend storage (`Vec<u8>`).

Required continuity rules for this runtime phase:
- binary stream APIs use typed `bytes` directly,
- no per-element byte-domain validation is reintroduced for already-typed `bytes`,
- validation remains only at explicit untyped conversion boundaries,
- no internal rebound to widened integer list carriers.

### Tempfile lifecycle prototype note

Prototype lifecycle for this phase:
- temporary resources are created with deterministic ownership,
- explicit cleanup APIs surface typed errors (`Result`),
- implicit scope-exit cleanup is best-effort and non-panicking,
- ownership/deletion behavior is documented per wrapper class (`delete=True` style semantics for owned temp files).

## Permanent Sifr-Safe Diffs (Locked for This Phase)

| Surface | Classification | Enforcement fixture |
| --- | --- | --- |
| Full CPython `_pyio` inheritance graph (`BufferedReader`, `BufferedWriter`, `BufferedRWPair`, `BufferedRandom`) | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr` |
| Async `subprocess.Popen` lifecycle/orchestration | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr` |
| Dynamic logging graph/config APIs (`dictConfig`, dynamic handler graph wiring) | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_dictconfig_unsupported.sifr` |
| Thread-aware logging ordering guarantees (`LoggerAdapter`-style threading contracts) | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_loggeradapter_unsupported.sifr` |
| `SpooledTemporaryFile` parity | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr` |
| String-eval `timeit` execution model | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timeit_string_eval_unsupported.sifr` |
| Timezone mutation helpers (`tzset`/mutable timezone state) | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timezone_mutation_unsupported.sifr` |

## CPython Family Mapping (Wave Ownership)

| CPython family | Direction | Owning wave | Local anchor |
| --- | --- | --- | --- |
| `Lib/test/test_io/` | `adapted` | `wave_psp_runtime_1` | sealed hierarchy + `BytesIO`/`StringIO` coverage |
| `Lib/test/test_tempfile.py` | `adapted` | `wave_psp_runtime_2` | tempfile wrappers + deterministic cleanup semantics |
| `Lib/test/test_zipfile/` | `adapted` | `wave_psp_runtime_2` | archive object lifecycle and metadata helpers |
| `Lib/test/test_logging.py` | `adapted` | `wave_psp_runtime_3` | deterministic single-process logging model |
| `Lib/test/test_time.py` | `adapted` | `wave_psp_runtime_3` | struct_time/clock object-model parity surfaces |
| `Lib/test/test_timeit.py` | `adapted` | `wave_psp_runtime_3` | callable-only timer model with explicit string-eval waiver |
| `Lib/test/test_subprocess.py` | `adapted` | `wave_psp_runtime_4` | sync process boundary cleanup; async lifecycle explicitly waived |

## Architecture-Lock Validation Artifacts (Wave 0)

- Positive lock fixture: `crates/sifr/tests/e2e/pass/phase_psp_runtime_0_architecture_lock.sifr`
- Wave-0 demos:
  - `demos/ad_hoc_runtime_wave0_stream_hierarchy_contract_demo.sifr`
  - `demos/ad_hoc_runtime_wave0_tempfile_zip_lifecycle_demo.sifr`
  - `demos/ad_hoc_runtime_wave0_bytes_binary_io_contract_demo.sifr`
- Permanent-diff negative fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_dictconfig_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_loggeradapter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timeit_string_eval_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timezone_mutation_unsupported.sifr`
