I have enough information to write the review. Let me compose it.

---

**Review of milestone_py_10**

## Blocking findings

### 1. `re_export_stdlib_imports` drops `@blocking_io` workload metadata for callback APIs (medium-high)

`crates/sifr_driver/src/stdlib/bootstrap.rs:462-495` only re-exports `functions`, `classes`, and `constants` from `sifr.python_core` into `sifr.python`. It never touches `function_workloads`, `function_defaults`, `function_varargs`, or `class_type_params`.

Concrete consequence for this milestone: all four newly added callback APIs in `python_core.sifr` — `local_callback_echo`, `threadsafe_callback_echo`, `close_local_callback`, `close_threadsafe_callback` — are decorated `@blocking_io`. After re-export, `stdlib_defs.function_workloads["sifr.python"]` does not contain entries for any of them, so the lowering in `crates/sifr_lowering/src/lower/imports.rs:149-174` cannot tag a `from sifr.python import threadsafe_callback_echo` call site as blocking. A user code path like

```python
from sifr.python import threadsafe_callback_echo
async def main() -> Result[None, PythonError]:
    cb = threadsafe_callback_echo()  # silently accepted in async
    ...
```

slips past the `ASYNC_DIRECT_BLOCKING_IO_CALL` check. That directly contradicts the phase contract "Every Python call is classified as `@blocking_io`" (plan §Async and Blocking Semantics, Core Decisions). The existing `python_async_tests.rs` doesn't catch it because it hand-builds externals against the `sifr.python` module key.

Same gap also strips `PythonError`'s 4 default-argument entries (constructor defaults are keyed by `class_name` in `function_defaults`, see `class_type_collection.rs:599`), so `PythonError("msg")` would fail to compile when imported from `sifr.python` even though it compiles when imported from `sifr.python_core`.

Fix shape: extend `re_export_stdlib_imports` to also copy matching entries from `stdlib_defs.function_workloads`, `function_defaults`, `function_varargs`, and `class_type_params` into the per-module entries built for `sifr.python`. Add a bootstrap test mirroring `stdlib_class_exports_preserve_parent_markers` that asserts `function_workloads["sifr.python"]["threadsafe_callback_echo"] == "blocking_io"`.

## Non-blocking findings

### 2. Local-callback "escape" detection is a global invocation counter, not active-call scope (low — scaffold)

`callback_ops.rs:148` rejects `CallbackKind::Local` whenever `invocations > 1`. The plan describes Local callbacks as "valid only during the active Sifr-to-Python call". The current scaffold marks every call after the first as "escape" regardless of whether we're still inside the same Sifr-to-Python call. This is acceptable for the "echo" foundation per the milestone description ("the Python-to-Sifr callback foundation and scaffold package contracts"), but the limitation should be tracked alongside the deferred Kafka/PubSub/CFFI surfaces called out in the DoD; the negative fixture `local_callback_escape_rejected` will pass for the wrong reason if a future implementation tries to support legitimate reentry without revising this counter.

### 3. Reserved-but-unused handle on failure in `create_callback` (very low)

`callback_ops.rs:90-118` reserves `(handle, token)` under one lock, releases it, then runs `attach + store_object + update_object_count(1) + callback_store + insert` outside that lock. If `store_object` fails (or any subsequent step fails), the handle counter has already advanced for an entry that will never exist. Pure counter waste, not a correctness issue, and the only failures here imply a poisoned mutex anyway. Could be tightened by reserving and inserting under the same final lock acquisition.

### 4. Missing not-initialized test for callback ops (low)

There's no test exercising `close_callback`/`local_callback_echo`/`threadsafe_callback_echo` when the runtime hasn't been initialized. The phase invariant is "no user-triggerable panic paths"; the `attach()` guard in `close_callback` should surface a `NotInitialized` runtime error, but it's worth a one-liner test to lock that contract — the comparable `attach_requires_runtime_initialization` test exists for `python.rs` but not for the callback module.

---

The remaining changes (callback object accounting via `CallbackEntry::Drop` + explicit `update_object_count(1)`, paired with the live-objects=2 assertion; mutex release prior to echo body; background-thread Python assertion; `attach` wrap on `close_callback`; codegen lowering for the new intrinsics) look correct and well-tested. Resource accounting balances at 2-in/2-out per callback. File-size guardrail honored (python.sifr 859, python_core.sifr 126, python.rs 898, callback_ops.rs 312).

The split itself (moving `Object`, `PythonError`, `ResourceDiagnostics`, `LocalCallback`, `ThreadsafeCallback`, and the callback echo/close helpers to `python_core.sifr`) is sound; the `STDLIB_SOURCES` ordering correctly lists `sifr.python_core` before `sifr.python` so the re-export sees populated `stdlib_defs`. The blocking finding above is in the re-export logic itself, not the ordering.

Recommendation: address finding #1 before merge — it is precisely the "stdlib bootstrap regression from the python_core split" the prompt flagged for inspection, and it leaves the phase's `@blocking_io` invariant violable through the canonical `from sifr.python import ...` surface.
