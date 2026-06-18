I've inspected the diff and the relevant source. Here's my review against milestone_py_2.

## Blockers

### 1. Embedded CPython runtime does not honor the selected venv

`crates/sifr_runtime/src/python.rs:153` calls `Python::initialize()` (which dispatches to `Py_InitializeEx` with the default config) and then in `configure_interpreter` (`python.rs:196-212`) only *prepends* `site_packages` and `sys_path` to `sys.path`. Nothing tells CPython to treat the configured `.venv` as its prefix:

- `PythonRuntimeConfig::venv_root` and `PythonRuntimeConfig::interpreter` are stored but never used to set `PyConfig.executable`, `PyConfig.prefix`, `PyConfig.exec_prefix`, `PYTHONHOME`, etc.
- After init, `sys.executable`, `sys.prefix`, `sys.base_prefix` will reflect whatever libpython `pyo3-build-config` linked at compile time — i.e., the host-global Python — not the probed venv.
- Native extensions loaded via the prepended `site-packages` will read those mismatched prefixes.

This directly violates the contract in `plans/issues/active/ad-hoc-embedded-python-interop.md:175` ("Runtime resolution must prefer the configured interpreter/venv. No host-global Python fallback is allowed.") and the milestone_py_2 scope line ("Initialize CPython once with selected environment configuration").

**Expected fix:** drive `Python::initialize_with(PyConfig {...})` via `pyo3-ffi` (or set `PYTHONHOME` before `Python::initialize()` as a transitional measure), with `executable`, `prefix`, `exec_prefix`, and the venv path wired from `PythonRuntimeConfig`. The `verify_interpreter_version` check at `python.rs:214-231` should remain as a post-init safety net.

### 2. `cargo build` for the generated package is not parametrized by the selected interpreter

`crates/sifr_driver/src/build/materialize.rs:166-184` runs `cargo build --release --quiet` with no environment overrides. The probe digest is in the cache key, but `pyo3-build-config` chooses libpython at build time from `PYO3_PYTHON` / `PATH` — neither of which is forced to match the selected venv. So the cache key claims "binary X was built against probe Y," but cargo may have actually linked a different libpython.

Combined with blocker #1, this means a binary cached for venv A can be linked to host Python B. The runtime version check is the only thing protecting against the mismatch, and it only checks major/minor.

**Expected fix:** thread the resolved interpreter into `Command::new("cargo")` as `PYO3_PYTHON=<config.interpreter>` (and, for cross-config interpreter resolution, the related `PYO3_CROSS_*` knobs where applicable). The phase contract enumerates `interpreter path` as a cache-invalidation input (`plans/issues/active/...:172`); it should also be a build input.

## Non-blocking concerns

1. **Fragile bootstrap injection** — `python_runtime.rs:89-105` uses `main_rs.find("fn main")` and then `find('{')` from that offset. Any preceding doc comment or unrelated `fn main_*` symbol earlier in `main.rs` would silently mis-target the injection. Today's generated `main.rs` doesn't trip it, but it's an unchecked invariant. Recommend a stricter match (e.g., a regex anchored at line start, or emitting the bootstrap call from codegen instead of textual post-processing).

2. **Bootstrap failure path bypasses Sifr diagnostics** — `python_runtime.rs:101-103` emits `eprintln!` + `std::process::exit(1)` on init failure. That's acceptable for "no panic" but the user sees an unstructured message rather than a `SIFR-PYRUNTIME-*` (or equivalent) coded diagnostic. The milestone_py_0 family list reserved `SIFR-PYENV/PYIMP/PYCALL/PYCONV/PYRES/PYZC/PYCB/PYTRUST` but no runtime-init family. Worth either reusing `SIFR-PYENV-*` at runtime, reserving a new family, or noting this in milestone_py_12 (docs/diagnostics closeout).

3. **No regression test for `InterpreterVersionMismatch`** — `python.rs:283-294` test config sets `cpython_version_tuple: Vec::new()`, which short-circuits version verification at `python.rs:218-220`. The mismatch path is unexercised. Hard to assert against a real interpreter without rebuilding, but a unit test that fakes `version_info` via an injected verifier would close the gap.

4. **State mutex held across `Python::initialize()`** — `python.rs:140-157` holds the `RUNTIME_STATE` mutex while CPython runs site-init. If site.py or an extension import recursively reaches the runtime state (e.g., via a Sifr callback in a later milestone), it deadlocks. Probably moot for milestone_py_2 but worth releasing the lock before `Python::initialize()` and re-acquiring just to write the config back.

5. **Partial-init recovery** — If `configure_interpreter` fails after `Python::initialize()` succeeds (`python.rs:153-156`), `state.config` is never set. A subsequent call with a *different* config will pass the conflict check (because `state.config` is `None`) and silently switch configs, even though CPython has already been initialized. Set `state.config` (or a "pending" sentinel) *before* calling `configure_interpreter`, or eagerly mark the runtime as poisoned on partial failure.

6. **`verify_interpreter_version` only checks major/minor** — Acceptable for libpython ABI, but the probe digest is the real cache key. Worth documenting that the runtime mismatch error is a defense-in-depth check, not an ABI gate.

7. **`run_cargo_build` does not surface clear "PyO3 couldn't find a Python" diagnostics** — Indirectly related to blocker #2: today a missing/wrong `PYO3_PYTHON` produces a generic cargo build failure via `BUILD_RUSTC_OR_CARGO_FAILURE`. Once #2 is fixed by passing the resolved interpreter, this resolves.

## Verdict

**Not approved.** The two blockers above are direct violations of the phase contract's "No host-global Python fallback" and the milestone_py_2 scope sentence "Initialize CPython once with selected environment configuration." The lifecycle state machine, GIL discipline, `Object` GIL-bound drop, conflict rejection, and cache-key plumbing are all correctly shaped — the gap is specifically how the selected venv is wired into both compile-time linking and `PyConfig`. Once `Python::initialize` is configured with `PyConfig`/`PYTHONHOME` from `PythonRuntimeConfig` *and* `cargo build` receives `PYO3_PYTHON` matching the probe, the milestone DoD is satisfied.
