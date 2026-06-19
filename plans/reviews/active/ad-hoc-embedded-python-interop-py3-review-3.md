Now I have everything I need. Let me write my findings.

---

# Review 3: milestone_py_3 follow-ups after review 2

Scope: confirm whether the four review-2 residual blockers (B1 single-file caveat, B2 structural opaqueness, B4 dead runtime check, M1/M3 documentation) are now adequately addressed for milestone_py_3.

Net assessment: **no remaining blocking findings for py3**. All four residuals are addressed with code changes, tests, and/or explicit phase-doc decisions.

---

## Item-by-item verification

### 1. Single-file static literal trust gap — ADDRESSED

`crates/sifr_lowering/src/lower/expressions/regular_calls.rs:22-29` now derives `allowed_roots` and `trusted_roots` via `python_trust_policy.as_ref().map_or(&[][..], ...)`, so the missing-policy case is *empty roots*, not *skip the check*. With empty allow-lists, `python_root_allowed([], _)` is false, and the static literal path emits `PYTRUST_UNTRUSTED_IMPORT` (`SIFR-PYTRUST-0002`). The dynamic-import branch at `:49-58` is policy-independent and unchanged.

Coverage: `crates/sifr_lowering/src/lower/python_trust_tests.rs:81-88` (`static_python_import_literal_without_policy_is_rejected`) calls `lower_errors(source, None)` and asserts the diagnostic. The previously existing `static_python_import_literal_uses_package_trust_policy` and `…_accepts_root_wildcard_policy` still cover the package paths. The phase doc records the behaviour at `plans/issues/active/ad-hoc-embedded-python-interop.md:34` ("single-file mode has no package trust policy, so static `sifr.python.import_module("...")` is rejected instead of falling through to runtime").

Side observation, not blocking: single-file mode also has no `PythonRuntimeConfig` (single-file builds skip `PackagePythonRuntime`), so a `@trust_python_dynamic` dynamic call in single-file mode would fail at `runtime_config()` with `NotInitialized` rather than succeed. The combined effect — single-file Sifr cannot meaningfully use `py.import_module` at all — matches the documented "if you don't have a trust policy, you can't do Python" stance and does not contradict the milestone DoD.

### 2. py.Object structural opaqueness — ADDRESSED (with documented decision)

`lib/sifr/python.sifr:40-42` constructor is now zero-argument and yields a sentinel closed value (`_handle = -1`, `_token = 0`). The only path that produces a usable Object is the private `_object_from_handle(raw)` helper at `:45-49`, fed by the intrinsics' `(handle, token)` tuples. A user who calls `Object()` directly gets a deterministically closed handle; any operation returns `SifrPythonClosedObject` from `clone_handle` (`crates/sifr_runtime/src/python/object_ops.rs:276-285`).

Forgery via field mutation (`obj._handle = X; obj._token = Y`) is still syntactically possible because the Sifr class fields stay public, but the runtime requires the token to match `hash((handle, nonce))` where nonce is internally generated (`object_ops.rs:287-290`), so a forged pair fails with `SifrPythonClosedObject` rather than yielding a valid Object. The structural opaqueness contract (spec line 276) remains technically violated at the Sifr surface, but the practical blast radius is closed.

The phase doc records the trade-off at `plans/issues/active/ad-hoc-embedded-python-interop.md:30`: "py3 accepts capability tokens as the practical opaque-object gate until a future sealed/extern class representation can remove the structural Sifr class surface." This is exactly the carve-out review 2 asked for. Sealed/extern class work is properly milestone-future, not py3.

### 3. Native trust runtime check — ADDRESSED

`crates/sifr_driver/src/build/python_runtime.rs:185-203` (`detected_native_import_roots`) now derives `native_import_roots` by chaining `probe.imports` and `probe.native_imports`, filtering to `ok && origin.ends_with(one_of_extension_suffixes)`, and deduping. This is independent of `trusted_native_roots`, which still flows from `[trust].python-native` via the `from_probe` constructor at `:46-49`. Aliasing is broken.

Test at `:227-276` (`native_import_roots_are_detected_from_probe_origins`) constructs a probe where `declared_imports = ["numpy"]`, `native_imports = []`, the probe's `imports` entry has a `.cpython-313-darwin.so` origin, and `trusted_native_roots` is passed as `Vec::new()`. The rendered config has `native_import_roots: vec!["numpy".to_string()]` and `trusted_native_roots: vec![]`. The runtime condition at `crates/sifr_runtime/src/python/object_ops.rs:240-247` (`contains(native) && !contains(trusted_native)`) now fires for this case — i.e., the path is reachable. Wildcards in `trusted_native_roots` still short-circuit via `contains_root` (`:251-255`).

This becomes proper defense-in-depth on top of `PYTRUST_UNTRUSTED_NATIVE_IMPORT` at the package-graph layer and `PYENV_NATIVE_IMPORT_FAILED` at probe time. Phase doc records it at line 35.

### 4. M1 (`exit_context` failure triple) and M3 (`eprintln!`/`process::exit`) — DOCUMENTED DEFERRALS

`plans/issues/active/ad-hoc-embedded-python-interop.md:37-38`:

- "Deferred `__exit__` failure-triple plumbing to `milestone_py_6` `py.with` lowering, where Sifr/Python failure context will be available."
- "Deferred bootstrap reporter/`eprintln!` cleanup to `milestone_py_5` package-runtime startup work; py3 keeps init failure as pre-main process exit."

Both deferrals name the milestone that picks up the work and the reason. M1 sensibly waits until the call site that needs the failure triple (`py.with`) lands — the helper signature can change then as part of that scope without re-touching settled APIs. M3 routes the `print_stderr`/`process::exit` discussion through the package-runtime startup work where the broader reporter decisions live. Both are reasonable scope decisions for py3.

---

## Conclusion

There are no remaining blocking findings for milestone_py_3. The four review-2 residuals are each addressed by a code change with test coverage (1, 3), a code change plus a recorded scope decision (2), or an explicit phase-doc deferral pointing to a successor milestone (4).
