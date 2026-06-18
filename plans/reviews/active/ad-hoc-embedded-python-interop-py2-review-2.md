I've reviewed the diff against main, focusing on the milestone_py_2 surface and the two blockers from review-1.

## Blockers

**None.** Both blockers from review-1 are fully resolved.

1. **CPython init honors the selected venv.** `initialize_cpython_with_config` (`crates/sifr_runtime/src/python.rs:235-266`) now drives `Py_InitializeFromConfig` via `pyo3-ffi`. `configure_raw_python_config` (`python.rs:268-320`) sets `executable`, `base_executable`, `program_name`, `prefix`, `exec_prefix`, `base_prefix`, `base_exec_prefix`, `argv`, and the exact `module_search_paths` from the probe; flips `module_search_paths_set = 1`, `use_environment = 0`, `user_site_directory = 0`, `install_signal_handlers = 0`, `parse_argv = 0`; and `PyEval_SaveThread` runs once after init. `verify_interpreter_config` (`python.rs:437-467`) then asserts `sys.executable`, `sys.prefix`, and `sys.base_prefix` match the probe values, so any post-link drift surfaces as `InterpreterConfigMismatch` instead of silent host-global use. This satisfies the "No host-global Python fallback" clause and the milestone_py_2 scope sentence.

2. **Generated cargo build is parametrized by the interpreter.** `materialize_binary_project_at_path` (`crates/sifr_driver/src/build/materialize.rs:81-90`) pulls `interpreter` out of `generated_project.python_runtime`; `run_cargo_build` (`materialize.rs:170-194`) sets `PYO3_PYTHON=<interpreter>` when present. Combined with `apply_package_runtime_metadata` (`project_codegen.rs:99-117`) writing the probe digest into `cache_key_fragment`, the cache invalidates and the build links against the same interpreter the runtime will validate against.

## Non-blocking concerns

1. **Bootstrap text injection is still fragile.** `inject_python_runtime_bootstrap` (`python_runtime.rs:105-126`) continues to anchor on `main_rs.find("fn main")` + `find('{')`. Today's `assemble_project_main_rs` emits a single `fn main` with no preceding comments or `fn main_*` neighbors, so it works — but it remains an unchecked invariant. Moving the bootstrap into codegen (e.g., adding a `pre_main_statements` slot on the project assembler) would remove the textual coupling.

2. **Bootstrap failure still bypasses Sifr diagnostics.** `python_runtime.rs:119-123` emits `eprintln!` + `std::process::exit(1)` on init failure. Acceptable for "no panic" but still unstructured. milestone_py_0 reserved `SIFR-PYENV` through `SIFR-PYTRUST`; a `SIFR-PYRUNTIME-*` family (or reuse of `SIFR-PYENV-*` at runtime) would fit better. Worth slotting into milestone_py_12 if not addressed sooner.

3. **No coverage for `InterpreterVersionMismatch` or `InterpreterConfigMismatch`.** `test_config` derives everything from the live `python3`, so both verifiers always pass. The new `InterpreterConfigMismatch` arm (`python.rs:50-54`, `437-467`) and the existing version mismatch arm have no unit test. A test that constructs a config with deliberately wrong `executable`/`sys_prefix`/`cpython_version_tuple` against the actually-linked Python would exercise both paths without needing alternate interpreters.

4. **`RUNTIME_STATE` mutex is still held across init.** `initialize_runtime` (`python.rs:157-178`) keeps the lock for the duration of `initialize_cpython_with_config` and `configure_interpreter`. Site init or a future callback that reaches `runtime_state()` would deadlock. Not exercised in milestone_py_2 but flagged for milestone_py_10 (callbacks).

5. **`verify_sys_attr` silently no-ops on empty expected values.** `python.rs:452-454` returns `Ok` when `expected.is_empty()`. The probe should always populate these, but a hardening pass could enforce non-empty in `PythonRuntimeConfig` construction rather than relying on caller discipline.

6. **`sys.path` ends up with duplicated entries.** `configure_interpreter` (`python.rs:216-233`) inserts `site_packages` *and* `sys_path` at position 0 of `sys.path` post-init, even though `module_search_paths` already contains `sys_path`. Functionally fine — duplicate entries are harmless — but cosmetically messy and slightly slows the first import scan. Either drop the post-init `sys_path` insert or skip the in-PyConfig pass.

7. **`PyConfig.home` is not set.** With `module_search_paths_set = 1` this doesn't affect path resolution, but `home` also influences a handful of cpython internals (e.g., `sysconfig`'s view of install layout). Worth setting to the venv root for completeness; not required for the milestone DoD.

8. **`append_module_search_path` accepts empty strings.** `python.rs:316-318` does not filter empties; if the probe's `sys.path` ever contained `""` (which CPython itself sometimes emits to mean CWD) it would be propagated. Today the probe filter likely strips these, but adding a guard is cheap.

## Verdict

**Approved for PR after authoritative create-pr validation.** The lifecycle state machine, GIL discipline, GIL-bound `Object` drop, conflict rejection, probe-digest cache plumbing, and now the `PyConfig`-driven init and `PYO3_PYTHON` build wiring all match the milestone_py_2 scope and the phase contract. Run `scripts/run_all_tests.sh --profile create-pr` before opening the PR; the non-blocking items can be addressed in follow-up work or rolled into milestone_py_12.
