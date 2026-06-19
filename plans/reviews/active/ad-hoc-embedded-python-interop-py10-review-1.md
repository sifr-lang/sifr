I've read the working tree changes and the py10 milestone contract end-to-end. There are several blockers — the implementation as it stands is an echo-stub registry, not a Python-to-Sifr callback surface.

## Blockers

### 1. No user-facing way to register a Sifr function as a Python callback
The only constructors are `local_callback_echo()` and `threadsafe_callback_echo()` (`crates/sifr_runtime/src/python/callback_ops.rs:61-67`, `lib/sifr/python_core.sifr:102-116`). Both produce identical `PyCFunction` wrappers around the closure at `callback_ops.rs:99-102`, which simply returns `args.get_item(0)`. There is no intrinsic, no stdlib helper, and no runtime API that accepts a Sifr `Callable[..., ..]` / lambda / function reference and turns it into a registered Python callable.

Consequence: the milestone goal "Python-to-Sifr callbacks" is not delivered. No Sifr user code is invoked from Python in this branch — only an echo loop. Every other DOD item below depends on this being real.

**Fix:** add an intrinsic like `py_register_local_callback(fn: Callable[…], capture: …)` / `py_register_threadsafe_callback(...)` whose lowering builds a `PyCFunction::new_closure` that calls the registered Sifr function and translates result/error.

### 2. LocalCallback semantic is wrong
`invoke_callback` at `callback_ops.rs:148-152` rejects any second invocation of a local callback as "escape". The contract explicitly allows "same-thread/same-stack reentry" within the active Sifr-to-Python call (e.g., sort comparators, `map`/`filter`, iteration callbacks). With this code a Kafka poller, a `list.sort(key=cb)`, or a `for x in py.iter(...)` would fail on the second element.

There is also no concept of the "active Sifr-to-Python call" anywhere — `call_object` (`object_ops.rs:158-185`) doesn't push/pop a call-depth marker, and `invoke_callback` doesn't consult one. Escape should be detected by "invocation after the enclosing Sifr-to-Python frame returned", not "invocation count > 1".

**Fix:** track a per-runtime call-depth counter incremented around each `call_object`/`call_attr`, stamp it onto each `LocalCallback` at creation, and reject only when the runtime's current depth has fallen below that stamp.

### 3. ThreadsafeCallback does not honor its contract
- `lib/sifr/python_core.sifr:63` declares `class ThreadsafeCallback(NonSend)`, and it embeds `callable: Object` (also NonSend). The contract calls it "the explicit audited bridge exception to non-Send `py.Object` defaults"; here it is just another NonSend handle, so no Send-like guarantees on captured state are enforced.
- `invoke_callback` (`callback_ops.rs:128-159`) runs whatever closure on whichever Python thread reaches the GIL. There is no "dispatch through the Sifr runtime scheduler unless explicitly same-thread reentrant" — the contract clause at the bottom of the py10 design section is unimplemented.
- Beyond the `invocations > 1` check, the two kinds are byte-identical. The current threadsafe test only proves a `PyCFunction` can be invoked from a `threading.Thread`, which is a PyO3 property, not a Sifr-runtime property.

**Fix (minimum):** make `ThreadsafeCallback` carry a `Send` capture pack, mark the class as Send in stdlib metadata, and route invocations that aren't on the Sifr scheduler thread through a runtime channel + scheduler hop before running Sifr code.

### 4. No Sifr-error → Python-exception bridge
The DOD calls out "captured-state constraints" and "Python exception mapping"; the contract says callbacks "convert Sifr errors into Python exceptions; convert Python callback-dispatch failures back into Sifr `Result` when control returns." Nothing in `callback_ops.rs` does this — the closure body never sees a Sifr `Result`, only `args[0]`. The contract JSON case `python_exception_mapping` in `verification/python_interop/fixtures/cffi_callback/callback_contract.json:46-50` is declared but unexercised.

### 5. Verification is contract-shaped, not contract-proving
`verification/python_interop/fixtures/cffi_callback/callback_roundtrip.sifr` invokes `local.callable` and `threaded.callable` from Sifr (`call(...)` at lines 20-21). It never has Python invoke a Sifr handler. The runner (`verification/python_interop/runner/run.py:99-148`) is still in scaffold mode — it doesn't compile, link, or execute the fixture; it just enumerates required files. The user's caveat about `cargo run -q -p sifr -- run … callback_roundtrip.sifr` failing with "Python runtime has not been initialized" is the corollary: there is no positive-path execution proof for the milestone. Per the Quality Contract "Every callback milestone must include callback-after-close and cross-thread behavior tests", what's present is a unit test in `callback_ops.rs` that calls `PyCFunction` from a `threading.Thread`; that doesn't constitute fixture-level evidence.

**Fix:** wire a package-mode fixture that actually initializes the runtime, registers a Sifr function as a callback, has Python invoke it (kafka-style poll or a simple `sorted(..., key=cb)`), and asserts both result mapping and error mapping.

### 6. CALLBACK_STORE ↔ RUNTIME_STATE lock-order inversion
- `create_callback` (`callback_ops.rs:108-110`): acquires `RUNTIME_STATE` (via `update_object_count(1)`), then `CALLBACK_STORE` (via `callback_store()`).
- `close_callback` (`callback_ops.rs:70-83`): acquires `CALLBACK_STORE` first; the subsequent `store.callbacks.remove(&handle)` drops `CallbackEntry`, whose `Drop` impl (`callback_ops.rs:40-44`) calls `update_object_count(-1)` — i.e. `RUNTIME_STATE` while `CALLBACK_STORE` is still held.

The opposite orderings collide; under concurrent create/close (which is exactly what the contract enables) two threads can deadlock. The same shape already exists between `OBJECT_STORE` and `RUNTIME_STATE` in `object_ops.rs:226-238` vs `store_object`, but py10 adds a third lock to the matrix and the threadsafe-callback feature is what makes it reachable in normal use.

**Fix:** either drop the `CallbackEntry` *after* releasing the `CALLBACK_STORE` guard (take ownership out of the map, release the lock, then let the entry drop), or stop touching `RUNTIME_STATE` from `Drop` paths and refactor the resource counters into a single lock-free counter (or have callbacks not double-count the underlying object).

## Smaller / non-blocking observations

- `CallbackEntry::drop`'s `let _ignored = super::update_object_count(-1)` (`callback_ops.rs:41-43`) silently swallows `StateUnavailable`. Same pattern as `Object::drop` so not new, but it means leak diagnostics may under-count under poison.
- `close_callback` removes the entry, decrements via `Drop`, and only then attempts `close_object`. If the user has already independently closed the underlying object, `close_object` returns `PythonError::closed(...)` while the registry has already moved on — the surfaced error message says the object is closed, not that the callback was. Worth a more specific error category.
- `CallbackKind`/`invocations` are stored on threadsafe entries but never read — dead state at runtime.

## Recommendation

The current py10 working tree is best described as a callback-registry scaffold with an echo placeholder. To honestly close milestone_py_10 as written, the missing pieces above (Sifr-function-as-callback registration, real local-scope tracking, threadsafe Send-bridge with scheduler dispatch, error mapping, and an executing verification fixture) need to land before merge. The lock-ordering inversion in §6 should be fixed in the same change since the contract explicitly exercises concurrent registry/close.
