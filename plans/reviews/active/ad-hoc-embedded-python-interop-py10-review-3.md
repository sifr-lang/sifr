I've reviewed the diff, the new `re_exports.rs`/`callback_ops.rs`/`python_core.sifr` files, prior reviews, and the phase contract.

# Review of milestone_py_10

## Blocking findings

None. The round-2 re-export blocker is correctly fixed:

- `re_exports.rs:71-102` propagates `function_workloads`, `function_defaults`, and `function_varargs` for any imported name (and via `is_imported_callable` line 104, for class-owned callable entries keyed `Class.method`). Class type params (`class_type_params`) are also forwarded `re_exports.rs:50-58`.
- `bootstrap.rs:156-172` wires the helper at the `sifr.python` ← `sifr.python_core` import boundary only, so the new path can't accidentally re-export across unrelated modules.
- The regression test `python_core_re_exports_preserve_callable_metadata` at `bootstrap.rs:830-855` pins `threadsafe_callback_echo` / `close_local_callback` workloads = `"blocking_io"` and `PythonError` constructor defaults = 4. That covers exactly what the round-2 finding called out.
- `STDLIB_SOURCES` (`sources.rs`) places `sifr.python_core` before `sifr.python`, so `stdlib_defs` is populated when the re-export runs. `features.rs:648` and `runtime_features.rs:55-59` correctly require `PythonRuntime` for `sifr.python_core`.

Round-1 #2 (local-callback semantics) and round-1 #6 (CALLBACK_STORE ↔ RUNTIME_STATE lock-order inversion) are also resolved in the current tree:

- `callback_ops.rs:158-179` now uses `super::python_call_depth() == 0` (thread-local, incremented by `enter_python_call()` in `call_object`/`call_attr`/`enter_context`/`exit_context`) instead of `invocations > 1`, so `list.sort(key=cb)` and same-stack reentry will now pass.
- `close_callback` (`callback_ops.rs:90-109`) hoists the `CallbackEntry` out of the closure before dropping it and explicitly calls `update_object_count(-1)` outside the `CALLBACK_STORE` guard. The `Drop` impl on `CallbackEntry` is gone, so no second lock is acquired while either store guard is held.

`cargo run -q -p sifr -- check verification/python_interop/fixtures/cffi_callback/callback_roundtrip.sifr` succeeds — confirming the re-exported `PythonError` constructor with defaults compiles through the `sifr.python` surface (the exact failure mode round 2 predicted).

## Phase-contract gaps (not blockers, but should be tracked before milestone closeout)

1. **No user-facing Sifr→callback registration.** `callback_ops.rs:73-88` ships Rust-level `local_callback` / `threadsafe_callback` that accept `Fn(ObjectHandle) -> Result<ObjectHandle, PythonError>`, but the only intrinsics exposed in `crates/sifr_stdlib/src/python.rs:195-218` and lowered in `crates/sifr_codegen/src/intrinsics/registry/python.rs:393-396` are the `_echo` variants. From Sifr source there is no way to register a Sifr handler — only echo handles can be created. The plan's "py.LocalCallback may borrow scoped Sifr values" and "py.ThreadsafeCallback requires Send-like Sifr constraints on captured values" cannot be exercised through the public surface.

2. **ThreadsafeCallback doesn't honor its Send contract.** `python_core.sifr:63` declares `class ThreadsafeCallback(NonSend)`, but the plan calls it "the explicit audited bridge exception to non-Send `py.Object` defaults." There is also no scheduler hop on non-Sifr threads — invocations run on whichever thread reacquires the GIL. The two classes remain byte-identical apart from the `kind` string and the local-escape gate.

3. **Verification fixture doesn't have Python invoke a Sifr handler.** `verification/python_interop/fixtures/cffi_callback/callback_roundtrip.sifr` only calls `local.callable`/`threaded.callable` via Sifr's own `call(...)`. The runner (`run.py:99-148`) is still scaffold-mode: it validates file presence but does not compile/execute the fixture. DoD line "Kafka/PubSub/CFFI-style callback examples pass" is not exercised end-to-end.

4. **Contract JSON cases declared but not proven.** `captured_state_constraints_scaffolded` and `python_exception_mapping` in `callback_contract.json:42-50` are scaffolded; no executing test maps to either. (`callback_ops.rs:194-217` does convert `PythonError` to `PyRuntimeError` via `py_runtime_error`, so Python-side error mapping exists in Rust for the unexposed `Sifr` target, but it's not contract-tested at the fixture level.)

## Lower-priority observations

- **`reset_runtime_state_for_tests` doesn't reset `CALLBACK_STORE`.** `python.rs:587-591` clears `RuntimeState` and the object store but leaves callback entries behind. All current tests close their callbacks, so accounting balances, but a future test that omits `close_callback` will leak into the next test silently while live-object counts read clean.
- **`re_exports.rs` covers `functions`/`classes`/`class_type_params`/`function_defaults`/`function_varargs`/`function_workloads`/`constants` but not `generic_functions`, `type_param_bounds`, or `constant_integer_values`.** Today `python_core` has none of those, so this is latent — but the helper looks designed for reuse. Either add the remaining fields or rename to make the python-core specificity explicit.
- **`reserve_handle` (`callback_ops.rs:223-238`) advances both counters before `store_object` runs.** Failure between `reserve_handle` and the final `store.callbacks.insert` wastes a handle/nonce pair. Pure counter waste; not a correctness issue.
- **`close_callback` after the underlying object was independently closed** returns `SifrPythonClosedObject` (from `close_object` at `object_ops.rs:236`), not a callback-specific error. Round-1 minor observation #2 is still open.
- **Round-2 finding #4 (no `NotInitialized` test for callback ops) is unaddressed.** Comparable to `attach_requires_runtime_initialization` in `python.rs`.
- **`PythonCallbackMetadata.kind: String` returned by the intrinsic is discarded** by `python_core.sifr:88` and line 97 (Sifr code hard-codes `"local"`/`"threadsafe"`). The intrinsic tuple's 5th element is dead. Cosmetic.

reviewer satisfied: no blockers
