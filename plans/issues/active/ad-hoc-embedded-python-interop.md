# Ad Hoc Phase: Embedded Python Interop

> Status: complete. `milestone_py_0` through `milestone_py_12` are merged through PR #2677; `milestone_py_12` completed public/internal docs, diagnostic evidence, py12 Opus review, phase-level final implementation review, and local validation. Post-closeout verification hardening for runnable dependency examples is merged through PR #2691 without changing the binding design decisions below.

## Execution Status

- [x] `milestone_py_0`: Planning lock and verification scaffold.
  - Added `verification/areas/python_interop/` runner, package matrices, fixture directories, and report path.
  - Reserved diagnostic families: `SIFR-PYENV`, `SIFR-PYIMP`, `SIFR-PYCALL`, `SIFR-PYCONV`, `SIFR-PYRES`, `SIFR-PYZC`, `SIFR-PYCB`, `SIFR-PYTRUST`.
  - Linked the phase from roadmap, phase index, and architecture docs.
  - Merged via PR [#2665](https://github.com/sifr-lang/sifr/pull/2665).
- [x] `milestone_py_1`: Environment discovery and probe.
  - Added `[python]` manifest parsing for root environment selection, `allow-imports`, and library `requires-imports`.
  - Added `[trust].python` and `[trust].python-native` manifest parsing.
  - Added package-graph Python environment resolution with missing-selection and multiple-venv diagnostics.
  - Added canonical CPython probe JSON validation for implementation, free-threaded status, venv prefix, site-packages, declared imports, native imports, and configured `pyproject.toml`/`uv.lock` digests.
  - Added active `SIFR-PYENV-0001` through `SIFR-PYENV-0011` diagnostics and generated docs.
  - Wired package `check`/cached `run` contexts to resolve/probe the root-selected environment and feed the probe digest into generated-artifact cache keys.
  - Promoted `verification/areas/python_interop/run.sh --group env` to live environment evidence with positive and negative fixture coverage.
  - Merged via PR [#2666](https://github.com/sifr-lang/sifr/pull/2666).
- [x] `milestone_py_2`: Embedded runtime lifecycle.
  - Added optional `sifr_runtime/python` feature backed by PyO3 embedding APIs and CPython `PyConfig` initialization.
  - Added generated package runtime metadata carrying the validated probe executable, prefixes, path set, version tuple, and digest into generated binaries.
  - Added generated bootstrap that initializes the Python runtime before user `main` code for Python-enabled package builds.
  - Added `PYO3_PYTHON` build environment wiring so generated package binaries link/configure against the selected interpreter.
  - Added runtime lifecycle state with same-config repeated init, conflicting-config rejection, GIL attach/detach helpers, owned Python object tracking, and shutdown diagnostics.
  - Added focused runtime, stdlib dependency, and driver bootstrap tests; Opus review approved and local `create-pr` validation passed.
  - Merged via PR [#2667](https://github.com/sifr-lang/sifr/pull/2667).
- [x] `milestone_py_3`: Opaque object operations and errors.
  - Implementation prepared on branch `ad-hoc-python-interop-py3`; Opus review round 3 found no remaining blocking findings for py3, and local `create-pr` validation passed.
  - Added `sifr.python` capability-token `Object`/structured `PythonError` surface over `_sifr.python` intrinsics; py3 accepts capability tokens as the practical opaque-object gate until a future sealed/extern class representation can remove the structural Sifr class surface.
  - Added runtime object handle operations for import, attr, string item access, calls, kwargs, explicit close, and context manager enter/exit.
  - Added runtime traceback capture and Python error family fields for import/attribute/item/call/conversion/resource/trust failures.
  - Added package graph `SIFR-PYTRUST` validation for dependency wildcard rejection, untrusted allowed imports, root wildcard allowance, and native roots without `allow-imports`.
  - Added lowering-time `SIFR-PYTRUST` diagnostics for static literal imports outside package allow/trust policy and dynamic imports without `@trust_python_dynamic`; single-file mode has no package trust policy, so static `sifr.python.import_module("...")` is rejected instead of falling through to runtime.
  - Added generated runtime config allow/trust/native root metadata and runtime root checks, with native roots detected from probe extension-module origins independently of trusted native roots.
  - Added py3 verification fixture manifests for positive, negative, cleanup, and trust cases.
  - Deferred `__exit__` failure-triple plumbing to `milestone_py_6` context-helper lowering, where Sifr/Python failure context will be available.
  - Deferred bootstrap reporter/`eprintln!` cleanup to `milestone_py_5` package-runtime startup work; py3 keeps init failure as pre-main process exit.
  - Focused validation passing: runtime Python tests, package Python tests, lowering trust tests, stdlib/codegen feature tests, `verification/areas/python_interop/run.sh --group scaffold`, diagnostics coverage checks, and `scripts/run_all_tests.sh --profile create-pr`.
  - Merged via PR [#2668](https://github.com/sifr-lang/sifr/pull/2668).
- [x] `milestone_py_4`: Primitive and typed conversion.
  - Added explicit `sifr.python` primitive constructors/extractors for `None`, bool, exact int, checked fixed-width integers, float, str, and exact bytes.
  - Added handle-based Python list, tuple, dict, and record construction so deep conversion remains explicit before Python calls.
  - Added copy-oriented typed list/tuple/dict conversions for core primitive values plus record-field extraction as `Object` handles.
  - Preserved nested conversion contexts for indexed/keyed failures and added fixed-width overflow diagnostics coverage.
  - Fixed imported `list[Object]` wrapper compatibility by lowering public object containers to raw handle tuples at the intrinsic boundary.
  - Added a regression test proving failed record-field conversion does not leak partially-created handles.
  - Added py4 JSON/source fixtures and required them from the Python interop scaffold runner.
  - Opus review round 2 reported no remaining blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2669](https://github.com/sifr-lang/sifr/pull/2669).
- [x] `milestone_py_5`: Async/blocking integration.
  - Classified every public `sifr.python` boundary operation as `@blocking_io`, including explicit constructors, extractors, copy APIs, context operations, calls, and `run_coroutine_blocking`.
  - Added `py.run_coroutine_blocking` through `_sifr.python` intrinsic metadata, codegen lowering, and runtime `asyncio.run` integration for Python-owned event loops returning explicit `Object` handles to Sifr.
  - Made `py.Object` inherit `NonSend` and fixed stdlib bootstrap export metadata so imported stdlib parent markers survive real single-file and project checks.
  - Added async/offload lowering tests for direct Python call rejection, explicit coroutine blocking rejection, offloaded sendable Python work, non-Send object return rejection, and unclassified offload target rejection.
  - Added async-blocking verification fixtures and required them from the Python interop scaffold runner.
  - Opus review round 1 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2670](https://github.com/sifr-lang/sifr/pull/2670).
- [x] `milestone_py_6`: Resource cleanup and leak diagnostics.
  - Added `py.with_context` with failure-aware `__exit__` plumbing and deterministic entered-handle cleanup on success, body failure, and normal `__exit__` failure paths.
  - Added runtime `resource_diagnostics` and `exit_context_with_error` operations with object double-close resource errors and recording context-manager coverage.
  - Added resource cleanup source fixtures for context-manager success, missing-enter failure, body failure, and outstanding diagnostics.
  - Documented the parser reservation for exact `py.with(...)` and the helper-owned entered-object rule.
  - Opus review round 2 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2671](https://github.com/sifr-lang/sifr/pull/2671).
- [x] `milestone_py_7`: `Py_buffer` zero-copy core.
  - Added `py.BufferView` and explicit u8 buffer helpers for zero-copy acquisition, checked writable requests, explicit copy, and deterministic release.
  - Added runtime `PyBuffer<u8>` ownership tracking with shape, stride, contiguity, format, readonly, and double-release diagnostics.
  - Added numpy-buffer contract/source fixtures covering readonly bytes, memoryview, explicit copy, wrong dtype, writable-on-readonly, and use-after-release behavior.
  - Opus review round 2 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2672](https://github.com/sifr-lang/sifr/pull/2672).
- [x] `milestone_py_8`: Arrow PyCapsule interop.
  - Added `py.ArrowCapsule` export, zero-copy-proof, copy-possible, and deterministic release helpers for Arrow array, stream, and schema PyCapsules.
  - Added runtime validation for exact Arrow capsule names, PyCapsule destructors, malformed capsules, destructor-less capsules, and double release.
  - Added pyarrow capsule contract/source fixtures covering pyarrow, polars, pandas, Pillow, unknown producer, malformed capsule, and copy-possible behavior.
  - Opus review round 3 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2673](https://github.com/sifr-lang/sifr/pull/2673).
- [x] `milestone_py_9`: DLPack tensor interop.
  - Added `py.DlpackTensor` with DLPack one-shot capsule consumption, `"used_dltensor"` marking, metadata extraction, and exact-once deleter release.
  - Added runtime checks for CPU tensors, scalar null-shape tensors, double consumption, double release, unsupported dtype/device, and invalid capsule names.
  - Added torch DLPack contract/source fixtures covering NumPy, torch, tensorflow CPU paths and rejection cases.
  - Opus review round 2 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2674](https://github.com/sifr-lang/sifr/pull/2674).
- [x] `milestone_py_10`: Python-to-Sifr callbacks.
  - Added compiler-lowered `py.local_callback` and `py.threadsafe_callback` registration for Sifr handlers returning `py.Object` results.
  - Added runtime callback registry dispatch, local same-stack escape checks, deterministic close/after-close behavior, and Sifr-error-to-Python-exception mapping.
  - Split shared Python wrapper types into `sifr.python_core` with targeted `sifr.python` re-export metadata preservation for blocking workloads and constructor defaults.
  - Added callback contract/source fixtures and native build coverage for real Sifr handler invocation through Python callables.
  - Opus review round 4 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2675](https://github.com/sifr-lang/sifr/pull/2675).
- [x] `milestone_py_11`: Package certification matrix.
  - Expanded Tier 1 certification matrices, Tier 2/Tier 3 deterministic smoke matrices, and Tier 4 host-dependent skip evidence.
  - Added deterministic package certification reports with `matrix-passed` status for matrix-only evidence and explicit host-dependent skip counts.
  - Added representative package-family contract fixtures for web, data, DB, broker, cloud/AI, TLS/native, callbacks, and tensor surfaces.
  - Opus review round 4 reported no blockers; local `create-pr` validation passed with only a warm wall-time advisory.
  - Merged via PR [#2676](https://github.com/sifr-lang/sifr/pull/2676).
- [x] `milestone_py_12`: Documentation, diagnostics, and closeout.
  - Published public Python interop examples covering biip/schwifty, FastAPI, Kafka callbacks, pandas/pyarrow/polars Arrow, torch/tensorflow DLPack, and cloud/AI clients.
  - Recorded internal architecture evidence for runtime lifecycle, GIL/refcount ownership, environment probes, verification gates, and example-to-fixture mapping.
  - Confirmed active Python environment/trust diagnostics have stable codes, generated docs, structured JSON arguments, and positive/negative test references.
  - Recorded exit evidence in `verification/areas/python_interop/reports/python_interop_exit_evidence.md`.
  - Opus py12 review round 4 and final implementation review round 4 reported no blockers after documented fixes.
  - Local py12 validation passed on 2026-06-19: `scripts/run_all_tests.sh --profile create-pr` completed 132 e2e pass tests with cache hits `44/44` and 6 hardening variants with 0 failures, plus a warm wall-time advisory; default `scripts/run_all_tests.sh` completed 651 e2e pass tests and 260 hardening variants with 0 failures, plus warm wall-time and group-skew advisories.
  - Merged via PR [#2677](https://github.com/sifr-lang/sifr/pull/2677).
- [x] `post_closeout_dataframe_examples`: Full runnable NumPy, pandas, and Polars examples.
  - Add area-local locked dependencies for NumPy, pandas, Polars, and pyarrow.
  - Add compiled Sifr examples that construct library objects, run real operations, convert results back to typed Sifr values, and assert expected outputs.
  - Add a `dataframe-examples` verification case that executes the examples through a temporary Sifr package linked to the area uv environment.
  - Opus review round 1 reported no blockers and round 2 reported no blockers after stdout-marker, trust-root, timeout, cleanup, and `@blocking_io` helper fixes.
  - Local create-pr validation passed on 2026-06-20: `scripts/run_all_tests.sh --profile create-pr` completed 132 e2e pass tests, Python interop `dataframe-examples` passed, and hardening variants reported 0 failures; warm wall-time/cache advisories only.
  - Merged via PR [#2691](https://github.com/sifr-lang/sifr/pull/2691).

## Objective

Deliver production-grade in-process CPython interoperability so Sifr programs can call into Python code and Python ecosystem packages from uv-created environments while preserving Sifr's safety, error, ownership, diagnostics, and local-verification guarantees.

Python interop is a separate lane from Rust-backed Sifr packages and raw C ABI interop. It is not a Deno-style generic `dlopen` layer and must not be collapsed into the Rust/C FFI model.

## Self-Contained Scope

This phase owns the complete embedded Python implementation surface required for production use. It may reuse existing Sifr compiler, package, async/offload, validation, diagnostics, runtime, and verification infrastructure, but it does not depend on Phase 42, Phase 43, or any separate interop/data-science phase being implemented first.

The phase must implement every Python-specific contract it relies on:

- root application Python environment ownership and verification;
- compiler metadata and build-plan plumbing for Python interop;
- trust policy for Python imports and native Python extension modules;
- `@blocking_io` diagnostics for Python calls in async Sifr code;
- typed Python object handles, primitive conversion, callback, cleanup, and error semantics;
- fixed-width integer, bytes, buffer, tensor, dataframe, and Arrow/DLPack interchange rules required by Python interop;
- package certification and verification infrastructure for the supported Python package tiers.

If an existing Sifr subsystem is not ready for one of these contracts, this phase includes the required implementation work inside the Python interop milestone sequence rather than waiting for another phase.

## Core Decisions

- Python interop is embedded CPython only.
- The root Sifr application selects exactly one uv-created Python environment for the final process.
- Sifr consumes and verifies the Python environment; Sifr does not resolve, install, or sync Python packages by default.
- CPython is initialized once per process, uses the main interpreter only, and is not finalized during normal shutdown.
- Subinterpreters are not supported in this phase.
- Multiple `.venv` environments in one binary/process are rejected.
- `py.Object` is an opaque foreign object, not `Any`.
- Every Python boundary operation is fallible and returns a Sifr `Result`.
- Python calls are synchronous from Sifr's perspective and classified as `@blocking_io`.
- `py.Object` is not `Send` by default and cannot cross Sifr task/thread boundaries without an explicit audited bridge.
- Zero-copy support is first-class in this phase: `Py_buffer`, Arrow PyCapsule, DLPack, and the `__array_interface__`/`__array_struct__`/`__cuda_array_interface__` protocols are all part of the contract.
- Python-to-Sifr callbacks are first-class and have explicit local/threadsafe lifetime modes.
- Python imports and native Python extensions are trusted in-process code.
- No subprocess, worker, or IPC-isolated Python mode is part of this phase.
- No backward-compatibility shims, legacy compatibility layers, fallback paths, or degraded modes are in scope; this phase designs the correct production contract directly.
- No silent fallback from zero-copy APIs to copying is allowed.
- No transitive Python import scanner is required or allowed as a correctness dependency; uv owns transitive Python dependency resolution.

## Non-Goals

- Do not introduce a Sifr-owned Python package manager.
- Do not run `uv sync` automatically during ordinary `sifr build`, `sifr run`, or `sifr check`.
- Do not support PyPy, GraalPy, MicroPython, or non-CPython implementations.
- Do not support free-threaded CPython in this phase.
- Do not support multiple Python interpreters or multiple virtual environments in one process.
- Do not support CPython subinterpreters.
- Do not expose Python objects as Sifr `Any`.
- Do not implicitly deep-convert Python lists, dicts, tuples, arrays, tensors, or dataframes.
- Do not let Python exceptions, Rust panics, or native extension failures cross into Sifr user code as panics.
- Do not rely on Python `__del__` for correctness-critical resource cleanup.
- Do not treat Arrow export, pandas conversion, memoryview creation, or Python dataframe conversion as inherently zero-copy.

## Configuration Model

The root application owns the Python environment:

```toml
[python]
venv = ".venv"
pyproject = "pyproject.toml"
lock = "uv.lock"
interpreter = ".venv/bin/python" # optional; if omitted, Sifr resolves from venv per platform
allow-imports = ["torch", "polars", "pandas", "pyarrow"]

[trust]
python = ["torch", "polars", "pandas", "pyarrow"]
python-native = ["numpy", "torch", "polars", "pyarrow", "cryptography"]
```

Library packages may declare required Python import roots but do not select the interpreter, Python version, virtual environment, lockfile, or package resolver:

```toml
[python]
requires-imports = ["polars"]
```

Rules:

- The final application must provide one selected Python environment if any reachable package requires Python.
- If no selected environment is available, Sifr reports a deterministic environment diagnostic before codegen/build.
- If multiple packages require incompatible Python import roots, uv dependency resolution is the user's responsibility; Sifr verifies the final environment.
- If two packages attempt to select different `.venv` roots, Sifr rejects the build.
- `allow-imports` defines the root modules Sifr code may request; `trust.python` authorizes executing those roots; `trust.python-native` separately authorizes roots that load extension modules.
- Applications may use `python = ["*"]` and `python-native = ["*"]` during local control. Published libraries using wildcards are rejected by package publish/check gates and package-graph loading.
- Static string imports are checked in HIR against `allow-imports` and trust. Dynamic import names are rejected unless the call site uses an explicit unsafe `@trust_python_dynamic` annotation, in which case runtime trust checks still gate the resolved root.
- Sifr gates declared/root imports only. Transitive Python imports remain uv's responsibility.

## Environment Probe

Sifr must probe the selected interpreter before any Python-enabled generated binary is considered valid.

Probe command shape:

```text
<venv>/bin/python        # Unix/macOS
<venv>\Scripts\python.exe # Windows
```

The probe must produce canonical JSON with:

- implementation name and version;
- CPython version tuple;
- executable path;
- `sys.prefix` and `sys.base_prefix`;
- site-packages paths;
- normalized `sys.path`;
- SOABI;
- extension suffixes;
- pointer width;
- platform and machine;
- `libpython` path when discoverable;
- free-threaded/GIL-disabled status;
- import metadata for declared roots;
- import/load results for declared native-extension roots;
- `pyproject.toml` and `uv.lock` digests when configured.

Validation rules:

- Reject non-CPython implementations.
- Reject free-threaded CPython.
- Reject a configured `.venv` whose interpreter does not report `sys.prefix` inside that `.venv`.
- Reject missing `site-packages` for the selected environment.
- Reject missing declared imports.
- Reject native extension import/load failures for trusted native Python roots.
- Add `pyproject.toml`, `uv.lock`, interpreter path, probe JSON, and declared import roots to cache keys.
- Treat the live interpreter probe as the source of truth. `uv.lock` digests are cache/diagnostic inputs, not correctness proof that the environment is synced.
- If lock/env state appears stale, report "run `uv sync`" guidance without invoking uv automatically.

## Build and Link Contract

- Generated binaries use the selected venv/interpreter probe as build metadata and cache input.
- Cache invalidates on interpreter path, CPython major/minor, SOABI, extension suffix set, pointer width, platform, `libpython` path when linked, `pyproject.toml` digest, `uv.lock` digest, declared imports, or trust config changes.
- Runtime resolution must prefer the configured interpreter/venv. No host-global Python fallback is allowed.
- PyO3 is built for embedding CPython, not for producing a Python extension module.
- If dynamic `libpython` linking is required on a target, generated Cargo metadata records the link path and runtime loader requirements; missing loader resolution is a build/probe diagnostic, not a runtime surprise.

## Runtime Lifecycle

`sifr_runtime::python` owns embedded CPython lifecycle.

Rules:

- Initialize CPython once per process.
- Use the main interpreter only.
- Do not call `Py_FinalizeEx` during normal program shutdown.
- Track initialization state and reject attempts to reinitialize with a different environment.
- Store the selected `PythonConfig` for diagnostics.
- Acquire the GIL for every CPython C API operation that requires it.
- Acquire the GIL before decref in `py.Object` destruction.
- Track outstanding `py.Object`, buffer, Arrow, DLPack, and callback resources for leak diagnostics and verification.

Rationale:

- CPython finalization is fragile with native extensions, background threads, callbacks, and `PyGILState_*`.
- Subinterpreters are incompatible with common extension-module and `PyGILState_*` assumptions.
- Python native packages are trusted process-local code, not sandboxed code.

## Conceptual Runtime API

The Rust runtime surface should be explicit and small:

```rust
pub mod python {
    pub struct PythonConfig {
        pub venv_root: PathBuf,
        pub executable: PathBuf,
        pub site_packages: Vec<PathBuf>,
        pub sys_path: Vec<PathBuf>,
        pub imports: Vec<String>,
        pub native_imports: Vec<String>,
        pub probe: PythonEnvironmentProbe,
    }

    pub struct PythonRuntime;
    pub struct PyGilScope;
    pub enum PyValue<'a> { None_, Bool(bool), IntExact(SifrExactInt), IntFixed(SifrFixedInt), Float(f64), Str(&'a str), Bytes(&'a [u8]), Object(&'a PyObjectHandle) }
    pub struct PyObjectHandle;
    pub struct PyBufferView;
    pub struct PyArrowCapsule;
    pub struct PyDlpackCapsule;
    pub struct PyCallbackHandle;

    // Generic wrappers specialize into Sifr-level ArrowArray, ArrowStream, and ArrowSchema handles.

    pub fn initialize(config: PythonConfig) -> Result<(), PythonInitError>;
    pub fn import_module(name: &str) -> Result<PyObjectHandle, PythonError>;
    pub fn with_gil<T>(f: impl FnOnce(PyGilScope) -> Result<T, PythonError>) -> Result<T, PythonError>;
    pub fn getattr(obj: &PyObjectHandle, name: &str) -> Result<PyObjectHandle, PythonError>;
    pub fn getitem(obj: &PyObjectHandle, key: PyValue) -> Result<PyObjectHandle, PythonError>;
    pub fn call(callable: &PyObjectHandle, args: &[PyValue], kwargs: &[(&str, PyValue)]) -> Result<PyObjectHandle, PythonError>;
}
```

Implementation uses PyO3 for interpreter, GIL, owned-reference, exception, and ordinary object-operation safety. It uses `pyo3-ffi` or narrowly wrapped CPython C APIs only where PyO3 does not expose the required production surface, including `Py_buffer`, capsule validation, Arrow PyCapsule names, DLPack capsules, and low-level environment initialization details.

## Sifr Object Model

Conceptual Sifr types:

```sifr
py.Object
py.Module
py.Callable
py.BufferView[T]
py.ArrowArray
py.ArrowStream
py.ArrowSchema
py.DlpackTensor
py.LocalCallback
py.ThreadsafeCallback
py.PythonError
py.ImportError
py.AttributeError
py.CallError
py.TypeConversionError
py.BufferError
py.ZeroCopyError
py.EnvironmentError
```

Rules:

- `py.Object` is opaque and foreign.
- `py.Object` is not `Any`.
- `py.Object` is not `Send` by default.
- `py.Object` cannot be pattern-matched or structurally typed as a Sifr class.
- Operations on `py.Object` are only allowed through Python interop APIs or compiler-approved Python syntax sugar.
- Every operation that may touch Python returns `Result`.

## User-Facing API

The canonical surface is explicit:

```sifr
from sifr import python as py

def main() -> Result[None, py.PythonError]:
    try:
        torch = py.import_module("torch")
        x = py.call_attr(torch, "tensor", [[1.0, 2.0], [3.0, 4.0]], [])
        y = py.call_attr(torch, "matmul", [x, x], [])
        text = py.to_str(y)
        print(text)
        return None
    except py.PythonError as e:
        raise e
```

Core operations:

```sifr
try:
    module = py.import_module("module.name")
    attr = py.get_attr(obj, "name")
    updated_attr = py.set_attr(obj, "name", value)
    item = py.get_item(obj, key)
    updated_item = py.set_item(obj, key, value)
    result = py.call(callable, args, kwargs)
    method_result = py.call_attr(obj, "method", args, kwargs)
    text = py.to_str(result)
    raw = py.to_bytes(result)
    closed = py.close(result)
    context_result = py.with_context(obj, lambda entered: ...)
    coroutine_result = py.run_coroutine_blocking(coro)
except py.PythonError as e:
    raise e
```

This phase does not introduce `import python <name>` syntax sugar; explicit `py.*` operations are the only user-facing surface.

## Operation Lowering

| Sifr operation | Semantic lowering |
| --- | --- |
| `py.import_module("torch")` | `Result[py.Module, py.PythonError]` |
| `py.get_attr(obj, "name")` | `Result[py.Object, py.PythonError]` |
| `py.call_attr(obj, "method", args, kwargs)` | `Result[py.Object, py.PythonError]` |
| `py.call(callable, args, kwargs)` | `Result[py.Object, py.PythonError]` |
| `py.get_item(obj, key)` | `Result[py.Object, py.PythonError]` |
| `py.to[T](obj)` | `Result[T, py.TypeConversionError]` |
| `py.zero_copy_as[T](obj)` | `Result[T, py.ZeroCopyError]`; never copies |
| `py.copy_as[T](obj)` | `Result[T, py.TypeConversionError]`; explicit copy |
| `py.scope(fn)` | `Result[T, py.PythonError]`; holds the GIL for batched operations |
| `py.close(obj)` | `Result[None, py.PythonError]` |
| `py.with_context(obj, fn)` | `Result[T, py.PythonError]` |
| `py.run_coroutine_blocking(coro)` | `Result[py.Object, py.PythonError]`, classified `@blocking_io` |

Outside a `try`/`Result` handling context, fallible Python operations are compile-time errors.

## Conversion Rules

Automatic conversion is conservative.

| Python value | Default Sifr type | Explicit conversion API |
| --- | --- | --- |
| `None`, bool, float, str, bytes | matching Sifr value | `py.to[T]`, `py.copy_as[T]` when needed |
| Python `int` | exact `int` | fixed-width `py.to[int32]`/etc. is checked and fallible |
| bytearray, memoryview, buffers | `py.Object` | `py.zero_copy_as[py.BufferView[T]]`, `py.copy_as[bytes]` |
| list, tuple, dict | `py.Object` | `py.to[list[T]]`, `py.to[dict[str, T]]`, record conversion |
| numpy arrays | `py.Object` | `py.BufferView`, array-interface protocols, DLPack where available, explicit copy |
| torch/tensorflow tensors | `py.Object` | DLPack or explicit copy |
| pandas/polars/pyarrow dataframes | `py.Object` | Arrow PyCapsule/stream or explicit copy |
| arbitrary Python object | `py.Object` | explicit protocol-specific conversion only |

Rules:

- Do not eagerly convert Python containers.
- Do not silently narrow Python integers to fixed-width Sifr integers.
- Do not assume Python sequences are finite, cheap, or pure.
- Typed conversion of lists/dicts/records must validate every element and fail with path-rich diagnostics.
- `bytes` conversion copies into owned immutable Sifr bytes unless an explicit zero-copy buffer view is requested.
- Sifr-to-Python call arguments accept `None`, bool, exact/fixed-width integers, float, str, bytes, and existing `py.Object` handles. Sifr containers and records require explicit Python object construction; overflow or unsupported argument packing returns `py.TypeConversionError`.

## Error Model

Every Python boundary failure returns a structured `py.PythonError` family value. It must preserve:

- Python exception type;
- message;
- traceback;
- module/function/attribute/item context;
- argument conversion context;
- return conversion context;
- dtype/shape/stride/device metadata for data interchange failures;
- environment and interpreter metadata for environment failures;
- whether the failure happened during import, call, callback dispatch, conversion, resource cleanup, or zero-copy export.

No Python exception crosses into Sifr as a panic. No CPython/PyO3 unwrap/expect/panic may be emitted in user-triggerable runtime paths.

## Async and Blocking Semantics

Python execution is synchronous from Sifr's perspective:

```text
Sifr calls Python.
Python may run threads, event loops, async code, native kernels, or callbacks internally.
Sifr waits until Python returns control.
```

Rules:

- Every Python call is classified as `@blocking_io`.
- Direct Python calls in async Sifr code are compile-time errors unless explicitly offloaded.
- The existing Sifr blocking task/offload primitive is the only async escape hatch. There is no Python-specific `py.blocking` alias.
- User-defined Sifr wrappers around Python work inherit or declare `@blocking_io`.
- `py.Object` values cannot cross async task/thread boundaries by default.
- Python cancellation is cooperative only. Sifr cannot safely kill embedded Python execution mid-call.
- `py.run_coroutine_blocking` is `@blocking_io`, creates or reuses a runtime-owned per-thread Python event loop, rejects reentry into an already running loop on that thread, respects installed loop policies such as uvloop, and returns only after the coroutine finishes.

## Resource Cleanup

Python resource cleanup must be explicit and ergonomic.

Supported cleanup APIs:

```sifr
try:
    closed = py.close(obj)
    close_result = py.call_attr(obj, "close", [], [])
    context_result = py.with_context(obj, lambda entered: ...)
    coro = py.call_attr(obj, "aclose", [], [])
    coroutine_result = py.run_coroutine_blocking(coro)
    callback_closed = py.close_callback(callback)
    buffer_released = py.release_buffer(buffer)
except py.PythonError as e:
    raise e
```

Rules:

- Do not rely on Python `__del__` for correctness.
- Context-manager helpers must call `__enter__` and `__exit__` and preserve Python traceback on failure.
- `py.with_context(obj, fn)` is scoped: `entered` cannot escape the lambda, the lambda returns generic `T`, and Python `__exit__(exc_type, exc, tb)` receives Sifr/Python failure context before the final `Result` is produced. The exact `py.with(...)` spelling is reserved until the parser supports hard-keyword member calls.
- The `entered` object passed to a `py.with_context` body is helper-owned. The body may inspect or call through it, but must not call `py.close(entered)`; the helper closes it after `__exit__` on both success and failure paths.
- Async context managers require a Python wrapper or explicit `py.run_coroutine_blocking`.
- Callback handles, buffer views, Arrow capsules, and DLPack capsules are resource handles with deterministic close/release semantics.
- Double close/release is either idempotent by design or reports a deterministic resource-state error; the decision must be documented per resource type.

## Zero-Copy Data Interchange

Zero-copy support is required in this phase.

Sifr exposes two families:

```sifr
try py.zero_copy_as[T](obj)  # rejects if zero-copy cannot be proven
try py.copy_as[T](obj)       # explicit copy
```

Rules:

- Zero-copy APIs never silently copy.
- Copying APIs never claim view semantics.
- Every view tracks the Python owner/resource needed to keep memory valid.
- `py.BufferView`, `py.ArrowArray`, `py.ArrowStream`, `py.ArrowSchema`, and `py.DlpackTensor` are non-`Send` by default. A view becomes transferable only through an explicit audited bridge that proves owner lifetime, device/stream sync, mutability, and thread-safety constraints.
- Writable views require both Python exporter writeability and Sifr-side exclusivity/borrow rules.
- Non-contiguous data is represented with strides when the target type supports strides; contiguity-required targets reject non-contiguous sources.

### `Py_buffer`

Used for:

- bytes-like objects;
- `memoryview`;
- NumPy arrays;
- psycopg/pymongo/aio buffers;
- Pillow/aiokafka buffers;
- general CPython buffer protocol producers.

`py.BufferView[T]` owns:

- exporter owner object;
- `Py_buffer`;
- pointer;
- length;
- readonly flag;
- item size;
- PEP 3118 format string;
- ndim;
- shape;
- strides;
- suboffsets;
- contiguity class;
- requested flags.

It is acquired via `try py.zero_copy_as[py.BufferView[T]](obj)`.

Drop/release must call `PyBuffer_Release` exactly once while holding the GIL.

### Arrow PyCapsule

Used for:

- pyarrow;
- polars;
- pandas Arrow export;
- Pillow Arrow images;
- dataframe/columnar interop.

Supported protocols:

- `__arrow_c_array__`;
- `__arrow_c_stream__`;
- `__arrow_c_schema__`.

Rules:

- Validate capsule names: `arrow_array`, `arrow_array_stream`, `arrow_schema`.
- Preserve capsule ownership and release callbacks.
- Distinguish real zero-copy exports from conversions that may copy.
- pandas Arrow export may copy through pyarrow; Sifr must not label it zero-copy unless the producer path proves that.
- Polars Arrow stream export is the preferred dataframe zero-copy target.

### DLPack

Used for:

- torch;
- tensorflow;
- NumPy DLPack support;
- future tensor libraries.

Rules:

- DLPack capsules are one-shot. Mark consumed capsules and reject double consumption.
- Track dtype, shape, strides, device, byte offset, deleter, and stream/device synchronization requirements.
- Handle CPU and device tensors explicitly.
- Reject unsupported DLPack dtypes/devices with typed errors.
- Do not hide synchronization/copy requirements behind a zero-copy API.

### Array Interface

Support `__array_interface__`, `__array_struct__`, and `__cuda_array_interface__` as additional zero-copy interchange protocols.

Rules:

- Retain the owner object.
- Validate pointer, dtype, shape, strides, readonly flag, version, and device metadata.
- Reject malformed or unsupported metadata.
- Prefer `Py_buffer`, Arrow, or DLPack when available.

## Python-to-Sifr Callbacks

Callbacks are first-class and explicitly resource-managed.

Two callback kinds:

```sifr
py.LocalCallback
py.ThreadsafeCallback
```

`py.LocalCallback`:

- valid only during the active Sifr-to-Python call;
- non-`Send`;
- same-thread/same-stack reentry only;
- may borrow scoped Sifr values;
- cannot be stored by Python beyond the active call;
- runtime detects escape attempts when possible and reports deterministic errors.

`py.ThreadsafeCallback`:

- may be called later;
- may be called from Python-created threads, native extension callback threads, thread pools, or event-loop schedulers;
- must own or clone captured state;
- requires Send-like Sifr constraints on captured values;
- is the explicit audited bridge exception to non-`Send` `py.Object` defaults;
- stores Python callable references only inside the runtime registry, reacquires the GIL before dispatch, and never exposes those references as Send Sifr values;
- dispatches non-Sifr-thread calls through the Sifr runtime scheduler unless the callback is explicitly marked same-thread reentrant;
- is registered in a runtime callback registry;
- has explicit `close`/`cancel`;
- converts Sifr errors into Python exceptions;
- converts Python callback-dispatch failures back into Sifr `Result` when control returns.

Required package patterns:

- confluent-kafka polling and background-thread callbacks;
- Google Pub/Sub scheduler thread-pool callbacks;
- boto3/botocore refreshable-credentials callbacks and transfer-manager progress callbacks;
- CFFI callbacks;
- Pika message callbacks;
- Python code invoking Sifr handlers while Sifr is blocked in Python;
- async client callback schedulers that call user handlers from Python-managed event loops.

## Native Extension Trust Boundary

Many supported Python packages load native code:

- NumPy, pandas, scipy, pyarrow, torch, tensorflow, scikit-learn, xgboost;
- cryptography, cffi, grpcio, google-crc32c, lxml, pydantic-core, tiktoken, uvloop, httptools, hiredis, fastavro, ujson, psutil;
- psycopg, psycopg2, pymongo, Pillow, confluent-kafka, aiokafka extensions.

Rules:

- Native Python extensions are trusted in-process code.
- Sifr cannot sandbox native extensions in embedded mode.
- Native crashes can terminate the process.
- Sifr's no-user-triggerable-runtime-panic guarantee applies to Sifr-attributable paths. `[trust].python-native` is an explicit opt-in boundary where process-abort safety is delegated to the trusted extension.
- Native extensions may release the GIL.
- Native extensions may create background threads.
- Native extensions may call back into Python/Sifr from non-Sifr threads.
- Trust diagnostics must name the import root, extension module, selected interpreter, extension suffix, and load error.

## Package Certification Matrix

Sifr should support any CPython-compatible package installed in the selected uv environment. Certification is a verification promise for representative package surfaces, not a package whitelist.

### Tier 1a: Core Interop Certification Gate

These packages must pass at the full Python interop gate because they exercise the core embedded, native, conversion, zero-copy, and production-client surfaces: `pydantic`, `pydantic-core`, `httpx`, `requests`, `cryptography`, `cffi`, `numpy`, `pandas`, `pyarrow`, `polars`, `torch` CPU, `psycopg`, `sqlalchemy`, `redis`, `confluent-kafka`, `boto3`, `botocore`, `openai`, `google-genai`, `biip`, `schwifty`.

### Tier 1b: Ecosystem Certification Gate

Packages already covered by Tier 1a may reappear here when they anchor a Tier 1b category; the Tier 1a gate is authoritative.

- Runtime/package loading: `pip`, `setuptools`, `wheel`, `build`, `hatchling`, `poetry-core`, `uv-build`, `pyproject-hooks`, `packaging`, `pkginfo`, `importlib-metadata`, `zipp`, `platformdirs`, `appdirs`, `typing-extensions`, `exceptiongroup`.
- Data validation/structured objects: `pydantic`, `pydantic-core`, `pydantic-extra-types`, `annotated-types`, `attrs`, `marshmallow`, `cerberus`, `jsonpickle`, `deepdiff`, `protobuf`, `proto-plus`, `pyyaml`, `toml`, `tomli`, `python-dotenv`.
- HTTP/async/networking: `requests`, `urllib3`, `httpx`, `httpcore`, `aiohttp`, `anyio`, `async-timeout`, `sniffio`, `h11`, `httptools`, `websockets`, `uvicorn`, `uvloop`, `grpcio`.
- Web frameworks: `fastapi`, `starlette`, `starlette-context`, `django`, `sanic`, `sanic-routing`, `sanic-testing`, `webargs`, `webargs-sanic`, `jinja2`, `markupsafe`, `python-multipart`.
- Databases/queues/brokers: `sqlalchemy`, `sqlalchemy-utils`, `alembic`, `psycopg`, `psycopg2`, `psycopg2-binary`, `asyncpg`, `pymongo`, `motor`, `mongoengine`, `redis`, `fakeredis`, `hiredis`, `valkey`, `aiokafka`, `confluent-kafka`, `kafka-python`, `python-schema-registry-client`, `pika`, `moto`.
- Cloud/AI clients: `boto3`, `botocore`, `openai`, `google-genai`, `google-api-core`, `google-api-python-client`, `google-auth`, `google-auth-httplib2`, `google-auth-oauthlib`, `google-cloud-core`, `google-cloud-storage`, `google-cloud-firestore`, `google-cloud-aiplatform`, `google-cloud-pubsub`, `firebase-admin`, `kubernetes`, `pinecone-client`, `gspread`, `gspread-asyncio`.
- Security/crypto/native loading: `cryptography`, `cffi`, `pycparser`, `certifi`, `idna`, `charset-normalizer`, `chardet`, `oauthlib`, `requests-oauthlib`, `python-jose`, `rsa`, `pyasn1`, `pyasn1-modules`.
- Data/binary/parser: `pandas`, `pyarrow`, `openpyxl`, `lxml`, `bleach`, `defusedxml`, `nh3`, `regex`, `ujson`, `fastavro`, `avro`, `avro-python3`, `google-crc32c`, `tiktoken`, `Pillow`, `tensorflow`, `scikit-learn`.
- Observability/production infra: `opentelemetry-api`, `opentelemetry-semantic-conventions`, `opentracing`, `sentry-sdk`, `sentry-asgi`, `datadog`, `structlog`, `logstash-formatter`.
- Domain examples: `biip`, `schwifty`, `pycountry`, `babel`, `holidays`, `dateparser`, `pendulum`, `python-dateutil`, `pytz`, `tzdata`, `uuid-utils`, `rank-bm25`.

### Tier 2: Broad Smoke Gate

- `click`, `typer`, `rich`, `pygments`, `tqdm`, `colorama`, `decorator`, `deprecated`, `backoff`, `backoff-utils`, `tenacity`, `ratelimit`, `cachetools`, `cachecontrol`, `aiocache`, `aiofiles`, `asgi-lifespan`, `basicauth`, `faker`, `freezegun`, `time-machine`, `testfixtures`, `responses`, `requests-mock`, `pytest-httpx`, `sortedcontainers`, `more-itertools`, `inflect`, `text-unidecode`, `python-slugify`, `pyhumps`, `parse`, `docopt`, `argh`, `commonmark`, `markdown`, `markdown-it-py`, `mdurl`, `pymdown-extensions`, `docutils`, `readme-renderer`, `rfc3986`, `uritemplate`, `webencodings`, `wrapt`, `wcwidth`.

### Tier 3: Dev/Tooling Compatibility Gate

- `pytest`, `pytest-asyncio`, `pytest-bdd`, `pytest-cov`, `pytest-mock`, `pytest-env`, `pytest-freezegun`, `pytest-redis`, `pytest-postgresql`, `pytest-pgsql`, `pytest-sugar`, `coverage`, `tox`, `pre-commit`, `black`, `flake8`, `flake8-black`, `flake8-isort`, `flake8-eradicate`, `flake8-pyprojecttoml`, `isort`, `mypy`, `mypy-extensions`, `pycodestyle`, `pyflakes`, `mccabe`, `bandit`, `eradicate`, `yapf`, `reno`, `sphinx`, `mkdocs`, `mkdocs-material`, `mkdocs-material-extensions`, `ghp-import`, `twine`, `pep517`, `setuptools-scm`, `cython`.

### Tier 4: Host-Dependent Compatibility Gate

- `gunicorn`, `uvicorn-worker`, `testcontainers`, `pyngrok`, `wmctrl`, `watchdog`, `psutil`, `pexpect`, `ptyprocess`, `sh`, `keyring`, `secretstorage`, `jeepney`, `keyrings-google-artifactregistry-auth`, `ipython`, `ipykernel`, `ipdb`, `pdbpp`, `pdbp`, `pdbr`, `fancycompleter`, `jedi`, `parso`, `prompt-toolkit`, `pyrepl`, `asttokens`, `executing`, `pure-eval`, `stack-data`, `traitlets`.

### Version-Conditional/Static Typing Artifacts

These should install/import when applicable but do not need special runtime certification unless dependency resolution pulls them into a tested environment:

- `enum34`, `typing`, `funcsigs`, `aiocontextvars`, `backports-tarfile`, `types-*`, `pandas-stubs`.

## Verification Area

Create a dedicated verification area:

```text
verification/areas/python_interop/
  README.md
  pyproject.toml
  uv.lock
  packages/
    tier1.toml
    tier2.toml
    tier3.toml
    tier4.toml
    native.toml
    async.toml
    data.toml
    cloud.toml
    brokers.toml
  fixtures/
    simple_import/
    primitive_conversion/
    pydantic_models/
    async_http/
    fastapi_app/
    sqlalchemy_psycopg/
    redis/
    kafka/
    pubsub/
    aws_sqs/
    aws_sns/
    aws_sns_sqs_subscription/
    pandas_arrow/
    polars_arrow/
    pyarrow_capsule/
    numpy_buffer/
    torch_dlpack/
    tensorflow_dlpack/
    cffi_callback/
    cryptography_tls/
    resource_cleanup/
  runner/
    run.py
    env.py
    import_matrix.py
    smoke_matrix.py
    native_probe.py
    callback_probe.py
    zero_copy_probe.py
    resource_probe.py
    report.py
  reports/
    .gitkeep
```

Verification groups:

- `env`: interpreter, venv, ABI, platform, lock/env freshness.
- `imports`: root imports and native extension load diagnostics.
- `native`: trusted native Python package load/use smoke.
- `async`: Python event-loop/client behavior under Sifr blocking semantics.
- `callbacks`: local and threadsafe callback behavior.
- `buffers`: `Py_buffer` ownership, contiguity, readonly/writable, dtype/format.
- `arrow`: Arrow PyCapsule schema/array/stream, zero-copy-vs-copy diagnostics.
- `dlpack`: one-shot tensor capsules, dtype/device/stride handling.
- `dataframes`: pandas/polars/pyarrow dataframe interop.
- `tensors`: NumPy/torch/tensorflow tensor interop.
- `databases`: SQLAlchemy, psycopg, asyncpg, pymongo, motor, redis.
- `brokers`: confluent-kafka, aiokafka, kafka-python, SQS long polling, SNS-to-SQS delivery, Pub/Sub-style callbacks. `brokers` covers messaging semantics; SDK surface is covered by `cloud`.
- `cloud`: boto3/botocore SQS/SNS clients/resources/stubbers, Google/AWS auth/import surfaces, without requiring live credentials in the default gate.
- `web`: FastAPI/Starlette/Django/Sanic import and in-process smoke fixtures.
- `cleanup`: close/context-manager/callback release/leak diagnostics.

Runner shape: `run.sh` is canonical; it validates environment prerequisites and delegates to `runner/run.py`.

```bash
verification/areas/python_interop/run.sh --group env
verification/areas/python_interop/run.sh --tier tier1
verification/areas/python_interop/run.sh --group native
verification/areas/python_interop/run.sh --group dataframes
verification/areas/python_interop/run.sh --package pandas
```

Validation profiles:

- Fast gate: environment probe, core embedded runtime, pure imports, pydantic/httpx/cryptography/cffi/pandas/pyarrow smoke.
- Full Python interop gate: all Tier 1 packages, native extension probes, zero-copy checks, callback/threading checks, local service-backed tests.
- External integration gate: Kafka, Postgres, Redis, AWS SQS/SNS via moto in-process mocking, LocalStack service emulation, or live AWS credentials, Pub/Sub, RabbitMQ, cloud SDK auth, and other service-backed tests.

## Milestones

Milestones are delivery order only. They do not reduce the design scope.

### milestone_py_0: Planning Lock and Verification Scaffold

Scope:
- Land this full phase contract.
- Create `verification/areas/python_interop/` scaffold and package matrix files.
- Define diagnostic families for Python environment/import/call/conversion/resource/zero-copy errors.
- Reserve diagnostic families: `SIFR-PYENV`, `SIFR-PYIMP`, `SIFR-PYCALL`, `SIFR-PYCONV`, `SIFR-PYRES`, `SIFR-PYZC`, `SIFR-PYCB`, `SIFR-PYTRUST`.
- Record package certification policy and host-dependent test policy.

Definition of done:
- Phase design is reviewed and accepted.
- Verification area exists with documented runner contract and empty/initial package matrices.
- No implementation milestone may start until this phase contract is linked from roadmap/index docs.

### milestone_py_1: Environment Discovery and Probe

Scope:
- Add Python config parsing for root packages and library package `requires-imports`.
- Implement selected interpreter discovery and canonical probe JSON.
- Validate CPython, `.venv`, site-packages, ABI, extension suffix, pointer width, platform, lock/env digests, and declared root imports.
- Reject unsupported interpreters, missing envs, stale envs, missing imports, native load failures, free-threaded CPython, and multiple venv selections.

Definition of done:
- Positive and negative fixtures cover every validation rule.
- Probe output is deterministic and part of build cache keys.
- Sifr never runs uv implicitly.

### milestone_py_2: Embedded Runtime Lifecycle

Scope:
- Add optional Python runtime feature and generated build metadata.
- Initialize CPython once with selected environment configuration.
- Implement GIL attach/detach discipline and owned `py.Object` refcount management.
- Add runtime diagnostics for reinitialization with a different environment and outstanding resource tracking.

Definition of done:
- Runtime lifecycle tests cover init, repeated init with same config, rejected conflicting init, GIL-bound destruction, and shutdown diagnostics.
- No user-triggerable panic paths exist.

### milestone_py_3: Opaque Object Operations and Errors

Scope:
- Implement `py.import_module`, attribute/item access, calls, kwargs, and explicit close/context-manager helpers.
- Add structured `py.PythonError` families with traceback capture.
- Enforce `Result` handling in lowering/type checking.
- Enforce `allow-imports`, `[trust] python`, `[trust] python-native`, wildcard rejection, and `@trust_python_dynamic` runtime root checks with `SIFR-PYTRUST` diagnostics.
- Keep `py.Object` distinct from `Any`.

Definition of done:
- Positive fixtures cover import, attr, item, call, kwargs, close, and context manager behavior.
- Negative fixtures cover import failure, attr failure, call failure, wrong args, conversion failure, and unhandled `Result`.
- Trust fixtures cover static imports, dynamic imports, native roots, wildcard rejection, and package-graph load failures.

### milestone_py_4: Primitive and Typed Conversion

Scope:
- Implement conservative conversions for `None`, bool, exact int, fixed-width checked ints, float, str, bytes.
- Implement explicit typed conversions for lists, tuples, dicts, and records.
- Preserve path-rich diagnostics for nested conversion failures.

Definition of done:
- Deep conversion is never implicit.
- Fixed-width conversion overflow/underflow is rejected with typed diagnostics.
- Large containers are not copied unless an explicit conversion requests copying.

### milestone_py_5: Async/Blocking Integration

Scope:
- Classify Python calls as `@blocking_io`.
- Reject direct Python calls in async Sifr code unless offloaded.
- Integrate with the existing blocking task primitive; do not add `py.blocking`.
- Implement `py.run_coroutine_blocking` as an explicitly blocking operation.
- Enforce non-Send default behavior for `py.Object`.

Definition of done:
- Async negative fixtures catch direct Python calls, object crossing, and unclassified blocking.
- Positive fixtures cover offloaded Python calls and Python-owned event loops returning results to Sifr.

### milestone_py_6: Resource Cleanup and Leak Diagnostics

Scope:
- Implement `py.close`, `py.with_context`, callback close, buffer release, Arrow/DLPack release tracking, and leak diagnostics.
- Define idempotence/error behavior for double close/release per resource.

Definition of done:
- Fixtures cover close success/failure, context manager success/failure, callback closure, buffer release, double release, and outstanding-resource diagnostics.

### milestone_py_7: `Py_buffer` Zero-Copy Core

Scope:
- Implement `py.BufferView[T]` using `PyObject_GetBuffer` and `PyBuffer_Release`.
- Track owner, pointer, length, readonly, itemsize, format, ndim, shape, strides, suboffsets, and contiguity.
- Implement `zero_copy_as` vs `copy_as` behavior for buffer producers.

Definition of done:
- Positive fixtures cover bytes-like, memoryview, NumPy, psycopg/mongo-style buffers, and readonly views.
- Negative fixtures cover unsupported format, wrong dtype, non-contiguous target, writable request on readonly data, and use-after-release prevention.

### milestone_py_8: Arrow PyCapsule Interop

Scope:
- Support `__arrow_c_array__`, `__arrow_c_stream__`, and `__arrow_c_schema__`.
- Validate capsule names and release callbacks.
- Distinguish zero-copy export from conversion paths that may copy.
- Cover pyarrow, polars, pandas, and Pillow Arrow paths.

Definition of done:
- Polars and pyarrow zero-copy paths are verified.
- pandas paths are marked copy-possible unless proven otherwise.
- Malformed capsule and double-release paths are rejected deterministically.

### milestone_py_9: DLPack Tensor Interop

Scope:
- Support DLPack capsule import/export and one-shot consumption semantics.
- Track dtype, shape, strides, device, byte offset, deleter, and stream/device sync requirements.
- Cover NumPy, torch, and tensorflow paths.

Definition of done:
- Positive fixtures cover CPU tensors and supported dtypes.
- Negative fixtures cover double consumption, unsupported dtype, unsupported device, invalid capsule name, and sync/copy-required rejection in zero-copy APIs.

### milestone_py_10: Python-to-Sifr Callbacks

Scope:
- Implement `py.LocalCallback` and `py.ThreadsafeCallback`.
- Add runtime callback registry, close/cancel, reentrancy tracking, and callback error conversion.
- Support Python callback invocation from same-stack calls, Python background threads, native extension callback threads, and Python scheduler/thread-pool callbacks.

Definition of done:
- Fixtures cover local callback success, local callback escape rejection, threadsafe callback success, callback close, callback after close, captured-state constraints, and Python exception mapping.
- Kafka/PubSub/CFFI-style callback examples pass.

### milestone_py_11: Package Certification Matrix

Scope:
- Implement Tier 1 certification fixtures and Tier 2/Tier 3 smoke gates.
- Add host-dependent Tier 4 policy and skip/evidence format.
- Add representative app fixtures combining web, data, DB, broker, cloud/AI, and observability clients.

Definition of done:
- Tier 1 package certification is green in the full Python interop gate.
- Tier 2/Tier 3 smoke gates are deterministic.
- Host-dependent skips are explicit and reported.
- Tier 1 packages that are host-dependent because of wheel or CPU feature availability must report explicit skip evidence in matrix-only gates and require live evidence in the full external gate.

### milestone_py_12: Documentation, Diagnostics, and Closeout

Scope:
- Publish public docs for Python interop configuration, trust, runtime semantics, error handling, blocking/offload, callbacks, resources, and zero-copy.
- Publish internal architecture docs for runtime lifecycle, GIL/refcount ownership, environment probes, and verification gates.
- Add final examples for biip/schwifty, FastAPI, Kafka, pandas/pyarrow/polars, torch/tensorflow DLPack, and cloud/AI clients.

Definition of done:
- Docs match implemented semantics exactly.
- Every diagnostic has stable code, URL, structured JSON fields, and positive/negative tests.
- Phase exit gate evidence is recorded.

## Quality Contract

Entry criteria:

- Existing local validation gates remain green before implementation starts.
- Python phase contract is linked from phase index and roadmap docs.

Global invariants:

- No user-triggerable compiler or runtime panics in Sifr-attributable paths; trusted Python native extensions are the explicit in-process trust boundary exception.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` in user runtime paths.
- Python errors are `Result` values with structured diagnostics.
- Python calls in async Sifr code require explicit blocking/offload.
- `py.Object` is not `Any` and is not `Send` by default.
- Zero-copy APIs never silently copy.
- Native Python extension trust is explicit.
- uv remains the Python package manager; Sifr verifies and consumes the environment.
- All behavior is deterministic except explicitly host/service-dependent gates, which must report structured skip/evidence.

Validation requirements:

- Every milestone must include at least one positive-path and one negative-path fixture mapped to its definition of done.
- Every resource-owning milestone must include leak/double-release/error-path tests.
- Every data-interchange milestone must include copy-vs-zero-copy tests.
- Every callback milestone must include callback-after-close and cross-thread behavior tests.
- Every package-certification milestone must update verification reports.

Local validation before PRs:

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test -p sifr -- --skip test_e2e_pass
scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile create-pr
```

Python interop gates, once implemented:

```bash
verification/areas/python_interop/run.sh --group env
verification/areas/python_interop/run.sh --tier tier1
verification/areas/python_interop/run.sh --group native
verification/areas/python_interop/run.sh --group dataframes
verification/areas/python_interop/run.sh --group callbacks
```

## Exit Gate

- Embedded Python interop is usable end-to-end from Sifr with a uv-created `.venv`.
- Root app environment ownership, trust, and probing are enforced.
- CPython lifecycle, GIL/refcount handling, and resource cleanup are deterministic.
- `py.Object` operations are opaque, fallible, non-`Any`, and non-Send by default.
- Python calls obey `@blocking_io`/offload rules.
- Python-to-Sifr callbacks support local and threadsafe modes.
- `Py_buffer`, Arrow PyCapsule, DLPack, and array-interface protocols are implemented with no silent zero-copy fallback to copying.
- Tier 1 package certification passes in the full Python interop gate.
- Verification reports exist under `verification/areas/python_interop/reports/`.
- Public and internal docs describe the exact production contract.
- Existing non-regression contracts remain green.
