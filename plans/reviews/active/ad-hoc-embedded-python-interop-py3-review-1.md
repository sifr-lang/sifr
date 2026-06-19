I have sufficient material to produce the review. Let me write it now.

---

# Review: milestone_py_3 (Opaque Object Operations and Errors)

The implementation lands a working runtime surface and trust framework, but **several blocking issues** undermine the milestone's stated DoD ("static imports, dynamic imports, native roots, wildcard rejection, and package-graph load failures") and the broader phase contract ("`py.Object` is opaque", "wildcards allowed for applications, rejected for libraries", "no user-triggerable panic paths"). Findings below are ordered by severity.

---

## Blocking (DoD / Spec deviations)

### B1. `@trust_python_dynamic` annotation and static HIR import gating are missing
**Severity: HIGH (milestone DoD)**

Milestone scope line 766 (`plans/issues/active/ad-hoc-embedded-python-interop.md`) explicitly says:

> Enforce `allow-imports`, `[trust] python`, `[trust] python-native`, wildcard rejection, and `@trust_python_dynamic` runtime root checks with `SIFR-PYTRUST` diagnostics.

And spec line 135:

> Static string imports are checked in HIR against `allow-imports` and trust. Dynamic import names are rejected unless the call site uses an explicit unsafe `@trust_python_dynamic` annotation…

Grep over the codebase shows `@trust_python_dynamic` exists only in spec/review markdown; there is no Rust, Sifr, or HIR enforcement. Today a static literal `py.import_module("untrusted")` and a dynamic `py.import_module(unknown_var)` go down the same path (`object_ops.rs:198` `validate_import_policy`) and are checked only at runtime. Two missing layers:

1. **HIR static check** for literal-string `py.import_module(...)` calls — should be a compile-time `SIFR-PYTRUST` error, not a `Result`.
2. **HIR rejection of dynamic imports** unless the call site bears `@trust_python_dynamic`.

The fixture file `verification/python_interop/fixtures/simple_import/opaque_object_operations.json` already lists `dynamic_root_runtime_rejection` and `static_allowed_and_trusted_import` as separate trust cases — but the implementation can't distinguish them. Both are runtime checks. **The trust DoD bullet ("Trust fixtures cover static imports, dynamic imports, native roots, wildcard rejection, and package-graph load failures") is not satisfied in code.**

Concrete failure mode: a user can ship `py.import_module(user_input)` today; if `user_input` happens to resolve to an allowed root at runtime, it executes. Spec wants this to be a compile-time rejection.

---

### B2. `py.Object` opaqueness contract is broken — handles are forgeable
**Severity: HIGH (spec deviation)**

`lib/sifr/python.sifr:36-41`:

```python
class Object:
    _handle: int

    def __init__(self, handle: int):
        self._handle = handle
```

And `crates/sifr_codegen/src/intrinsics/registry/python.rs:60-66`:

```rust
.map(|__sifr_python_arg| __sifr_python_arg._handle)
```

Spec lines 273-276 are explicit:

> `py.Object` is opaque and foreign.
> `py.Object` is not `Any`.
> `py.Object` cannot be pattern-matched or structurally typed as a Sifr class.

Today any user can write `forged = Object(42)` then pass `forged` to `py.get_attr`/`py.call`/etc. The runtime safely returns `closed()` (`object_ops.rs:194`) instead of panicking, so the blast radius is "deterministic error" rather than "memory corruption" — but the **opaqueness contract itself is violated**. The fact that the constructor takes an arbitrary `int` and that the codegen reads `obj._handle` directly means there is no opaque/sealed concept enforcing the property at the language level.

Recommended root-cause fix path: introduce an opaque/sealed class concept in HIR, or generate the `Object` type as an extern handle (no user-callable constructor) so only intrinsics can mint values. Either way the issue is structural, not surface-level renaming.

---

### B3. Wildcard policy is stricter than spec — root applications cannot use `*`
**Severity: HIGH (spec deviation)**

Spec lines 134-135:

> Applications may use `python = ["*"]` and `python-native = ["*"]` during local control. Published libraries using wildcards are rejected by package publish/check gates and package-graph loading.

`crates/sifr_package/src/python/environment.rs:281-313` (`validate_python_trust_policy`) iterates **every package** in the graph and emits `PYTRUST_WILDCARD_REJECTED` if any contains `"*"` — including the root application. The matching test `python_trust_rejects_wildcard_roots` (`crates/sifr_package/src/python/tests.rs:145-165`) asserts this behavior on a single-package graph where the wildcard package IS the root app. This is the wrong direction.

Required: skip the wildcard rejection for the root application package; only enforce on non-root packages (libraries). The DoD bullet "Trust fixtures cover … wildcard rejection" passes today only because the fixture treats root-app wildcards as rejected, locking in the spec deviation.

Note also: even after a wildcard is accepted for an application, the runtime side `validate_import_policy` re-checks `reject_wildcards` on every import call (`object_ops.rs:235-243`) and rejects them too — meaning `["*"]` in the root app would never let any import through even if the package-graph layer accepted it. The runtime layer needs a paired fix.

---

### B4. Native-trust runtime check is dead code
**Severity: HIGH (silent security gap)**

`crates/sifr_runtime/src/python/object_ops.rs:220-227`:

```rust
if contains_root(&config.native_import_roots, root)
    && !contains_root(&config.trusted_native_roots, root)
{
    return Err(PythonError::trust(
        format!("native Python import root '{root}' is not listed in [trust].python-native"),
        name,
    ));
}
```

Trace the data flow:
- `RuntimeConfig.native_import_roots` is populated from `PackagePythonRuntime::from_probe(... request.native_imports.clone() ...)` (`crates/sifr_driver/src/build/python_runtime.rs:46`).
- `request.native_imports` is `ResolvedPythonEnvironment.native_imports` (`environment.rs:138`), which is `native_python_imports(graph)` = `trusted_python_native_imports(graph)` (`environment.rs:265-267`).
- `RuntimeConfig.trusted_native_roots` is `resolved.trusted_native_imports` = `trusted_python_native_imports(graph)`.

Both sets are populated from the **same** filtered `trust.python-native` list. The condition `contains(native_roots) && !contains(trusted_native_roots)` is **always false** by construction; the native-trust runtime check fires never.

To fix root-cause: introduce a meaningful distinction. The probe already separates `imports` from `native_imports` (the latter being declared native roots that actually loaded extension modules); `native_import_roots` should track roots known to load native code (declared or probe-detected), while `trusted_native_roots` stays as the explicit trust list. Today they are aliased.

---

## Significant (correctness / contract)

### M1. `exit_context` always passes `(None, None, None)` to `__exit__`
**Severity: MEDIUM (spec deviation)**

`crates/sifr_runtime/src/python/object_ops.rs:181-184`:

```rust
.call_method1("__exit__", (py.None(), py.None(), py.None()))
```

Spec line 416:

> Python `__exit__(exc_type, exc, tb)` receives Sifr/Python failure context before the final `Result` is produced.

Today there is no path for Sifr-side failure to be relayed to `__exit__`. Even if `py.with` is deferred to milestone_py_6, the `exit_context` primitive needs to accept a failure-info parameter (Sifr `Result` failure → Python exception triple) so the higher-level wrapper can plumb it. As shipped, the helper cannot be assembled into a correct `py.with` later without changing this signature — which is an avoidable second migration.

---

### M2. `inject_python_runtime_bootstrap` finds `fn main` by raw substring
**Severity: MEDIUM (fragility)**

`crates/sifr_driver/src/build/python_runtime.rs:132`:

```rust
let Some(main_start) = main_rs.find("fn main") else { ... };
let Some(body_offset) = main_rs[main_start..].find('{') else { ... };
```

This matches `fn maintain`, `fn main_helper`, `fn main` inside a string literal/comment, etc. It also picks the **first** match. For generated codegen output today this is deterministic, but the layered prelude/preamble pipeline (`lib_modules_and_codegen.rs`) emits user code and stdlib code together — if a stdlib helper ever ships a `fn main_something()` before the user's `fn main()`, the bootstrap is silently inserted into the wrong function.

Use a regex with word boundaries, parse via `syn`, or have the codegen emit a known marker (e.g., `// __SIFR_PYTHON_BOOTSTRAP_HERE__`) instead of pattern-matching after the fact.

---

### M3. Generated bootstrap uses `eprintln!`
**Severity: MEDIUM (workspace lint divergence)**

`crates/sifr_driver/src/build/python_runtime.rs:144-146`:

```rust
eprintln!(\"Sifr Python runtime initialization failed: {}\", __sifr_python_runtime_error);
```

Workspace clippy has `print_stderr = "warn"` (per `AGENTS.md` "Workspace lints"). Generated package Cargo projects don't inherit workspace lints today so this compiles, but a user who turns on pedantic lints on the generated project hits an error from generated code they cannot edit. Use the structured diagnostic surface or wire a `sifr_runtime`-owned reporter that owns the print-policy decision.

Secondary: the bootstrap calls `std::process::exit(1)` before user `main` runs, denying any Sifr-level recovery. The spec allows this for environment failures, but the milestone scope's "Result handling" promise (line 766) is not met for init failures — they cannot be caught from Sifr.

---

### M4. `call_attr` silently discards intermediate close errors
**Severity: MEDIUM**

`crates/sifr_runtime/src/python/object_ops.rs:153-163`:

```rust
let callable = get_attr(handle, name)?;
let result = call_object(callable, args, kwargs);
let _ignored = close_object(callable);
result
```

If `close_object` returns `Err` (concurrent removal, store mutex poisoned, etc.), the diagnostic is dropped. Per spec line 374 ("No CPython/PyO3 unwrap/expect/panic may be emitted in user-triggerable runtime paths") and the project's "Solve root causes, not superficial symptoms" rule, the close error should at minimum be surfaced when `result` is `Ok` (compose into the returned `PythonError`) — otherwise leak diagnostics will appear elsewhere without a traceable origin.

---

### M5. `store_object` handle counter saturates silently
**Severity: MEDIUM (correctness, low probability)**

`crates/sifr_runtime/src/python/object_ops.rs:245-252`:

```rust
store.next_handle = store.next_handle.saturating_add(1);
let handle = store.next_handle;
store.objects.insert(handle, object);
```

Once `next_handle` saturates at `i64::MAX`, every subsequent `store_object` returns the same handle, and `HashMap::insert` silently overwrites the prior `Object` (its `Drop` decrements `live_objects` while the user's old handle now silently points to a different Python value). Practical risk is nil (~290k years at 1M ops/sec), but it's a silent correctness violation. Either:
- return `Err` after saturation (deterministic), or
- use `checked_add` and return a typed runtime error.

The "no user-triggerable runtime panic" promise hides this — but silent corruption is worse than a typed error.

---

### M6. `Object` constructor in stdlib doesn't validate handle existence
**Severity: MEDIUM (Object contract slip)**

`lib/sifr/python.sifr:36-41` — `Object.__init__` accepts any `int`. Combined with B2, even if opaque-class concept is added, the constructor should reject handles not present in the runtime store. Today there's no validation. Pair with B2 fix.

---

## Lower-priority (residual risk / hygiene)

### L1. Sifr wrappers contain cargo-cult `try/except/raise`
`lib/sifr/python.sifr:43-89` — every public function wraps the intrinsic in `try: handle = py_*(...); return Object(handle); except PythonError as e: raise e`. The except branch is a no-op re-raise. Either Sifr requires this pattern syntactically (in which case it's harmless but unidiomatic), or it's confused error-handling code that should be removed. A cleaner shape is `handle: int = py_import_module(name); return Object(handle)`.

### L2. Test ordering implicit dependency on first CPython init
`crates/sifr_runtime/src/python.rs:637-848` — `reset_runtime_state_for_tests()` wipes Sifr-side state but `Py_IsInitialized()` remains true after the first test. `initialize_cpython_with_config` short-circuits (line 255-257), so the **second** test never re-runs `PyConfig_SetBytesString`. `verify_interpreter_config` then compares against the **first** test's executable/prefix. Today every test uses the same `local_python_config()` so paths match, but any future test that varies these fields would fail with `InterpreterConfigMismatch` based on ordering. Document or guard.

### L3. `sys_path` and `site_packages` are appended twice
`python.rs:236-251` inserts every `site_packages` and `sys_path` entry via `sys.path.insert(0, …)` after `Py_InitializeFromConfig` already populated `module_search_paths` from `sys_path` (line 335-337). Result: duplicated `sys.path` entries (the site_packages once, the sys_path twice). Wasted memory, not wrong. Consider whether the post-init insertion should be limited to entries CPython didn't already register.

### L4. `validate_import_policy` clones the entire `PythonRuntimeConfig` on every `import_module` call
`object_ops.rs:204` — `runtime_config()` clones the full config. Hot path overhead for high-frequency import-cached patterns. Use an `Arc<PythonRuntimeConfig>` or a borrow guard.

### L5. Two independent string searches for `sifr_runtime::python::`
`entrypoints.rs:166` and `lib_modules_and_codegen.rs:824` both substring-search the generated source to set `PythonRuntime`. Meanwhile, the intrinsic registry already declares `Some(StdlibFeature::PythonRuntime)` for `py_*` (`registry.rs:800`). Three independent mechanisms for the same fact — pick one; the substring search will silently fail when a future renaming uses an alias.

### L6. `Object` Drop on un-attached runtime leaks via `std::mem::forget`
`python.rs:136-155` — if `Python::try_attach` returns `None` (interpreter finalized or never initialized), the `Py<PyAny>` is forgotten and `record_leaked_object()` is called. Aligned with spec ("no user-triggerable panics"), but the spec also says Python is never finalized at normal shutdown. If `try_attach` ever returns `None` during normal operation, that's an invariant violation worth logging at a higher level than "incremented leaked_objects".

### L7. Cargo-cult Sifr surface lacks `set_attr` / `set_item` / `get_item` non-string
The milestone scope claims to deliver "import, attr, item, call, kwargs, close, and context manager" but the Sifr surface only ships `get_item` (string keys), no `set_attr`, no `set_item`. Spec line 304-308 lists all of these as core operations. If this is deferred, the milestone's DoD ("Positive fixtures cover import, attr, item, call, kwargs, close, and context manager behavior") quietly elides `set_attr`/`set_item`/non-string `get_item`.

---

## Spec / milestone definition gaps

### S1. `@blocking_io` not enforced on Python calls
Spec lines 79, 388-389:

> Every Python call is classified as `@blocking_io`.
> Direct Python calls in async Sifr code are compile-time errors unless explicitly offloaded.

Milestone_py_5 owns this in the phase, but milestone_py_3 ships a usable surface that can be called from async Sifr today with no warning. The milestone notes do not mention this deferral. Add an explicit "intentionally deferred to milestone_py_5" line to the phase doc so reviewers don't read past the gap.

### S2. `Result` non-handling is supposed to be a compile-time error
Spec line 334:

> Outside a `try`/`Result` handling context, fallible Python operations are compile-time errors.

I see no Python-specific HIR check enforcing this. It depends on Sifr's general Result-handling enforcement (the fixture references `SIFR-RESULT-0001` for unhandled close). If Sifr's general enforcement is solid, this is covered transitively, but worth a direct fixture-driven assertion in this milestone.

### S3. Phase doc says implementation is "merge ready"
The phase doc lists "Focused validation passing: …" but doesn't note the gaps in B1/B3/B4/M1 above. Either implement those before claiming the milestone done, or update the phase doc to list them as carry-over.

---

## Summary

The runtime/PyO3 plumbing, error families, traceback capture, and probe-keyed metadata flow look solid. The blocking items are spec-level: missing `@trust_python_dynamic` + static HIR trust gating (**B1**), forgeable opaque handles (**B2**), wildcard policy stricter than spec for root apps (**B3**), and a dead-code native-trust runtime check (**B4**). Two of these (B3, B4) also have tests/fixtures that lock in the deviation — fixing the implementation alone will not be enough; tests and fixtures need to follow.

Recommend not merging milestone_py_3 until B1–B4 land and are exercised by fixtures, M1 is decided (relay failure context to `__exit__` now vs. document the deferral and signature change), and M2 is hardened (the `fn main` substring search is the kind of fragility that will haunt later async-main or multi-entry-point work).
