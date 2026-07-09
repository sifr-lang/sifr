## Code Review - M11i Python Callback Helper Migration

**PR-ready.** The migration is semantically correct, well tested, and its manifest/guardrail updates faithfully record the closing state of `_sifr.python` retained callback surface. No blockers.

### Blockers
None.

### Non-blocking suggestions

1. **Dead-code special case in `crates/sifr_driver/src/stdlib/bootstrap.rs:135-138`.** The block that inserts `local_callback` / `threadsafe_callback` into `intrinsic_names_for_module` when `module_name == "sifr.python"` is now unreachable - those functions moved to `sifr.python_core` and their `sifr.python` re-exports never enter this loop (only defined-in-module functions do). It's harmless but stale after M11i and can be removed as a follow-up. The new `python_callback_helpers_codegen_through_sifr_stdlib` test already checks the intrinsic-names surface, so deleting this branch is safe.

2. **`_object_from_handle` duplicated in `stdlib/sifr/python.sifr:179-183` and `stdlib/sifr/python_core.sifr:59-63`.** The bodies are identical; the duplication exists to avoid a `python -> python_core -> python` cycle. Consider promoting the helper to `python_core` and importing it into `python.sifr`, since `python.sifr` already imports many names from `python_core`. Optional; the duplication is small and private.

3. **Discarded `kind` field in the callback tuple (`stdlib/sifr/python_core.sifr:66-81`).** `_local_callback_from_raw` / `_threadsafe_callback_from_raw` hard-code `"local"` / `"threadsafe"` and ignore `raw[4]`, which is the `kind: str` element returned by the Rust shim (`sifr_stdlib::python::callback_raw`). This preserves the previous behavior (the compiler-lowered code also hard-coded the kind), but the runtime is paying to marshal a string that is always discarded. Either use it (`callback.kind = raw[4]`) or drop the field from `CallbackRaw` / the `_sifr.python` tuple signature. Non-blocking.

4. **Adapter probe path is narrow by design (`crates/sifr_driver/src/build/rust_interop_probe.rs:382-405`).** `is_python_raw_callback_probe` hard-codes two full Rust paths and returns a fixed probe template. That's correct today because generic signature probing can't spell the `Fn(...) + Send + Sync + 'static` closure bound with a concrete `sifr_stdlib::python::PythonError` in place of the bridge-error generic. Worth a brief comment above the check explaining *why* this special case exists so a future contributor doesn't try to fold it back into `signature_probe_source`. The special case still runs `cargo check`, so no unvalidated path is created.

5. **Adapter is scoped exactly to callback constructors (`crates/sifr_codegen/src/rust_interop_direct.rs:118-143`).** `is_python_callback_constructor_target` limits the raw-callback adapter to `sifr_stdlib::python::{py_local_callback, py_threadsafe_callback}`, and `is_python_raw_callback_type` requires an exact `Callable[[tuple[int, int]], Result[tuple[int, int], PythonError]]` shape. Package callback interop (which uses different target roots / signatures) is unaffected. Good.

### Verification of the stated goals
- **`python_core` wrapper preserves semantics.** `local_callback` / `threadsafe_callback` build the same `(handle, token, callable, kind)` `LocalCallback`/`ThreadsafeCallback` as the old compiler-lowered code did (both hard-code the kind); the handler-invocation path goes `raw -> Object -> handler(arg) -> (result._handle, result._token)`, matching the previous `handler(&object)` adapter. Ownership: `Object` is a plain `NonSend` handle wrapper with integer fields and no Sifr-level destructor, so passing by value in `_call_object_callback` is safe.
- **Raw callback adapter is sufficiently scoped.** Only fires on the two exact stdlib targets and the exact raw callback signature; package callback interop uses `@rust.callback` with different targets and is not affected.
- **Probe special case is justified.** Generic probe can't express the closure bound with a concrete error type; the special case still exercises `cargo check` on the real Rust symbol, so no bypass.
- **Manifest/guard updates accurately record closure.** `_sifr.python` is now `state = "closing"` with no retained items; `check_stdlib_native_intrinsic_allowlist.py` no longer expects any `py_` prefix dispatcher or `registry/python.rs`; `check_stdlib_migration_closure.py` retires the five old callback names; the fallback signature module was deleted and the registry lost the prefix branch.
