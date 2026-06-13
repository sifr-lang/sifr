# Ad Hoc Phase: Runtime and File-Object Parity Expansion

Status: completed (started 2026-03-19; `wave_psp_runtime_0` completed with pass-1/pass-2 external reviews; `wave_psp_runtime_1` completed with pass-1/pass-2 external reviews; `wave_psp_runtime_2` completed with pass-1/pass-2 external review closure; `wave_psp_runtime_3` completed with pass-1/pass-2 external reviews; `wave_psp_runtime_4` implementation merged with pass-1/pass-2 external review approval; wave-level extra completion and production-grade review cycles approved; milestone-level completion and production-grade review cycles approved; phase-level completion and production-grade review cycles approved)
Context: follow-up phase after structured/class-surface parity expansion and the extended bytes/binary-surface foundation
Execution readiness: implementation-ready after completion of predecessor bytes extension waves `wave_psp_bytes_4` and `wave_psp_bytes_5`; runtime/file-object APIs now inherit the final raw-byte-backed `bytes` contract and successor/FFI governance baseline
Execution ledger: `issues/ad-hoc-runtime-and-file-object-parity-expansion-execution.md`

## Objective

Close the next major runtime and file-object parity frontier for modules whose current shipped subset is useful but intentionally narrowed around simple helpers and lightweight wrappers.

Primary module targets:

- `logging`
- `io`
- `zipfile`
- `tempfile`
- `time`

Secondary module target:

- `subprocess`
  - only where its remaining synchronous file-object and option-matrix work is directly entangled with the primary runtime/file-object root cause

## Source of Truth

- canonical parity inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant closure wave ledgers:
  - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_d2_cpython_traceability.md`
- architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/07_stdlib_parity.md`
- predecessor planning docs:
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream families:

- `Lib/test/test_io/`
- `Lib/test/test_logging.py`
- `Lib/test/test_zipfile/`
- `Lib/test/test_tempfile.py`
- `Lib/test/test_time.py`
- `Lib/test/test_timeit.py`
- `Lib/test/test_subprocess.py`

## Why This Needs Its Own Phase

These gaps share one architecture theme:

- richer object wrappers around host resources,
- lifecycle-sensitive file and stream semantics,
- context-manager-like ownership boundaries,
- host-backed clocks and logging subsystems,
- and archive/process option matrices that interact with runtime capabilities.

That is a separate implementation problem from parser/class expansion and from RNG/crypto parity.

The runtime and file-object design is fixed in this document. What remains before wave 1 is prototype evidence that the sealed hierarchy and cleanup model can be implemented cleanly on top of current Sifr runtime constraints.

## Depends on

- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - hard dependency for all binary stream, archive, and file-object surfaces
- milestone-7 canonical closure inventory remains the baseline
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Public Surface Contract

### `io`

- Do not chase the full CPython `_pyio` inheritance graph.
- Ship a sealed Sifr hierarchy:
  - `IOBase`
  - `TextIOBase`
  - `BinaryIOBase`
  - `FileHandle` as the concrete text-file type
  - `BinaryFileHandle` as the concrete binary-file type
  - `BytesIO`
  - `StringIO`
- `BufferedReader`, `BufferedWriter`, `BufferedRWPair`, and `BufferedRandom` are deferred unless they become necessary inside this same phase for `zipfile` or `tempfile`.
- Common required methods:
  - `close() -> None`
  - `closed() -> bool`
  - `flush() -> Result[None, IOError]`
  - `seek(offset: int, whence: int = 0) -> Result[int, IOError]`
  - `tell() -> Result[int, IOError]`
  - `readable() -> bool`
  - `writable() -> bool`
  - `seekable() -> bool`
- Required text surfaces:
  - `read() -> Result[str, IOError]`
  - `write(data: str) -> Result[int, IOError]`
- Required binary surfaces:
  - `read_bytes(size: int | None = None) -> Result[bytes, IOError]`
  - `write_bytes(data: bytes) -> Result[int, IOError]`
- Binary file and in-memory surfaces must consume and produce canonical raw-byte-backed `bytes`, not widened integer storage or `list[int]` stand-ins.
- No runtime/file-object implementation in this phase may reintroduce per-element byte-domain validation for typed `bytes` inputs or outputs; only explicit untyped conversion boundaries may validate.
- Text iteration over files is line-based and must reuse the iterator architecture from phase 1.
- Binary surfaces must use first-class `bytes` from the predecessor phase rather than `list[int]`.

### Resource lifecycle

- RAII / scope-exit cleanup is the default.
- Explicit cleanup APIs return `Result` where the operation itself is fallible.
- Implicit scope-exit cleanup is best-effort and must never panic.
- Every wrapper class must document whether it owns deletion or removal on exit.

### `tempfile`

- Phase targets:
  - `NamedTemporaryFile`
  - `TemporaryDirectory`
- `SpooledTemporaryFile` is deferred in this phase.
- `NamedTemporaryFile(delete: bool = True)` deletes on scope exit when `delete` is true.
- `NamedTemporaryFile.close()` and `NamedTemporaryFile.cleanup()` surface errors explicitly.
- Binary temporary-file modes use `bytes` as their payload type.
- `TemporaryDirectory.cleanup() -> Result[None, IOError]`.

### `zipfile`

- Add:
  - `is_zipfile`
  - compression constants
  - `ZipInfo`
  - `ZipFile.infolist()`
  - `ZipFile.getinfo()`
  - `ZipFile.extract()`
  - `ZipFile.extractall()`
- `ZipFile.open(name, mode="r")` returns a `BinaryIOBase`-compatible read handle only.
- The binary handle returned by `ZipFile.open(...)` reads `bytes`.
- Write-mode streamed file handles remain out of scope in this phase.
- Compression support stays limited to formats already available in the Rust backend when implementation begins; unsupported formats remain explicit waivers.

### `logging`

- Use a deterministic, single-process, synchronous logging model.
- Add:
  - `Handler`
  - `StreamHandler`
  - `FileHandler`
  - `NullHandler`
  - `Formatter`
- `LoggerAdapter` is deferred in this phase.
- No `dictConfig`, no dynamic handler graphs, and no thread-aware ordering claims.
- Logger registry is process-local and deterministic within one process.

### `time` / `timeit`

- Add immutable `struct_time`.
- Add:
  - `gmtime`
  - `localtime`
  - `mktime`
  - stable timezone constants
- `Timer` exists, but is callable-only; no string-eval execution model.
- No timezone mutation helpers.

### `subprocess`

- Keep async `Popen` fully out of scope.
- Expand only sync surfaces that depend on the file-object work:
  - `PIPE`
  - `STDOUT`
  - `DEVNULL`
  - `check_call`
  - `check_output`
- `run` remains the primary execution API.

## Permanent Sifr-Safe Diffs

The following are intentionally not part of this phase’s execution target:

- full CPython `_pyio` inheritance parity,
- async `Popen` lifecycle and process orchestration,
- `dictConfig` and dynamic logging handler graphs,
- thread-aware logging ordering guarantees,
- `SpooledTemporaryFile`,
- string-eval `timeit` execution,
- timezone mutation helpers.

If these remain unsupported at phase exit, they must be explicit and narrow in the waiver inventory.

## Scope

This phase owns:

- `io` stream hierarchy and in-memory wrapper parity under the sealed hierarchy above,
- `zipfile` file-object, option-matrix, and archive-control expansion,
- `tempfile` object-wrapper and lifecycle parity,
- `logging` hierarchy/configuration expansion within host-limited bounds,
- `time` / `timeit` richer object-model parity under the callable-only timer model,
- synchronous `subprocess` file-object and option-matrix improvements that share the same runtime boundary work.

This phase does not own:

- async `subprocess.Popen` lifecycle or full process orchestration,
- interpreter-mutation hooks in `sys`,
- broad `os` API expansion beyond what this runtime/file-object work requires,
- unrelated parser or collection expansion,
- crypto or RNG-state work.

## Non-goals

- weakening host-limited classifications without real runtime support,
- pretending full CPython parity exists where the host boundary remains intentionally narrower,
- mixing async runtime work into this phase,
- adding wrapper types that violate Sifr ownership and panic-free invariants.

## Priority Targets

### priority_1: Stream and file-object foundations

Modules:

- `io`
- `tempfile`
- `zipfile`

Required closure direction:

- ship the sealed `io` hierarchy above,
- add object-oriented tempfile wrappers with deterministic cleanup and lifecycle rules,
- expand `zipfile` beyond the current narrow create/write/read/namelist subset.

### priority_2: Runtime host surfaces

Modules:

- `logging`
- `time`

Required closure direction:

- expand logging hierarchy/configuration in ways that remain host-deterministic,
- expand `time` / `timeit` object-level parity such as `struct_time` and callable-only `Timer`.

### priority_3: Synchronous process/file-object cleanup

Modules:

- `subprocess`

Required closure direction:

- close more of the synchronous option matrix that depends on the new file-object infrastructure,
- keep async lifecycle and signal orchestration explicitly out of scope.

## Waves

### wave_psp_runtime_0: Architecture Lock

Scope:

- `io`
- lifecycle rules
- `zipfile`
- `tempfile`
- `logging`
- `time`

Definition of done:

- the sealed hierarchy, cleanup rules, host-model constraints, and deferred families in this document are reflected in traceability and waivers,
- all binary stream and archive surfaces in this document explicitly build on first-class `bytes`,
- no later wave needs to invent ownership or lifecycle semantics,
- every permanent deferral is explicitly classified before implementation proceeds.

### wave_psp_runtime_1: `io` and In-Memory Stream Hierarchy

Scope:

- `io`
- `BytesIO`
- `StringIO`

Definition of done:

- core in-memory and text/binary stream families are shipped or explicitly waived,
- ownership, seek/tell, and typed error contracts are documented and covered,
- no remaining “stream hierarchy” waiver entry stays vague.

### wave_psp_runtime_2: Tempfile and Archive Object Lifecycles

Scope:

- `tempfile`
- `zipfile`

Definition of done:

- tempfile object wrappers and lifecycle semantics are closed or sharply waived,
- archive object and option-matrix parity expands beyond the current narrow subset,
- validation proves deterministic cleanup and panic-free error behavior.

### wave_psp_runtime_3: Logging and Clock Object Expansion

Scope:

- `logging`
- `time`
- `timeit`

Definition of done:

- the next useful host-backed object-model tranche is shipped,
- remaining host-limited behavior is explicit and defendable,
- no public claims overstate what the runtime can actually guarantee.

### wave_psp_runtime_4: Synchronous Process Boundary Cleanup and Governance Closure

Scope:

- `subprocess`
- final ledger updates for the whole phase

Definition of done:

- synchronous process/file-object parity is materially stronger,
- async-only gaps remain explicitly waived,
- all owned waiver entries are updated and no owned surface remains `open`.

## CPython Test Porting Targets

- `Lib/test/test_io/`
- `Lib/test/test_logging.py`
- `Lib/test/test_zipfile/`
- `Lib/test/test_tempfile.py`
- `Lib/test/test_time.py`
- `Lib/test/test_timeit.py`
- `Lib/test/test_subprocess.py`

## Quality Contract

- Host-backed behavior must stay deterministic where Sifr claims determinism.
- No object-lifecycle parity may introduce user-triggerable panics or cleanup races.
- Every wave must include positive-path and negative-path host-boundary coverage.
- Every waiver that survives this phase must describe the concrete remaining host or runtime blocker.

## Architecture Lock Validation

Before `wave_psp_runtime_1` begins implementation, the phase must add:

- one implementation note showing how current `FileHandle` enter/exit behavior generalizes into the sealed hierarchy rather than introducing an unrelated cleanup model,
- one Sifr demo covering the sealed stream hierarchy,
- one Sifr demo covering deterministic tempfile or zipfile lifecycle behavior,
- one Sifr demo covering bytes-backed binary file or `BytesIO` behavior,
- one implementation note proving the sealed binary stream hierarchy consumes the raw-byte-backed `bytes` contract directly rather than compensating around widened integer storage,
- one negative-path test for every newly explicit permanent divergence,
- one CPython-family mapping table proving which upstream cases are adopted, adapted, or permanently waived,
- explicit phase test families covering `test_io`, `test_tempfile`, `test_zipfile`, `test_logging`, `test_time`, `test_timeit`, and `test_subprocess`,
- one compile-time rejection or negative runtime case for every new typed surface that proves the remaining Sifr-safe divergence is explicit rather than accidental,
- one narrow prototype note for tempfile cleanup or binary-handle lifecycle proving the chosen cleanup strategy is implementable on top of existing Sifr runtime constraints.

## Local Validation Commands

- quick gate:
  - `scripts/run_all_tests.sh --profile quick`
- full gate:
  - `scripts/run_all_tests.sh`
- targeted:
  - `cargo test -p sifr -- <test_name>`
  - `cargo run -q -p sifr -- run demos/<runtime-parity-demo>.sifr`

## Exit Gate

This phase is complete only when:

- the major stream, archive, tempfile, logging, and time object-model waivers are materially reduced,
- surviving host-limited or unsupported entries are explicit and narrow,
- the full validation suite is green,
- external review confirms the documented scope is production-grade.
