I've audited the current tree against the six listed blockers. Result: all addressed, no new blockers.

**Per-blocker verification:**

1. ✅ Public handler registration: `lib/sifr/python.sifr:461-467` exposes `local_callback(handler)` / `threadsafe_callback(handler)` with `Callable[[Object], Result[Object, PythonError]]`. `crates/sifr_driver/src/stdlib/bootstrap.rs:101-104` flags both as intrinsic-only in `sifr.python`, and `crates/sifr_codegen/src/intrinsics/registry.rs:792-795` dispatches them through `python::lower_python_intrinsic`.

2. ✅ Fixture lowering: `cargo run -q -p sifr -- emit verification/python_interop/fixtures/cffi_callback/callback_roundtrip.sifr` emits, at the `main` body (lines ~660 and ~696 of the emit), `sifr_runtime::python::local_callback(move |__sifr_callback_arg| { … match increment_callback(&__sifr_callback_object) { … } })` and the matching `threadsafe_callback(...)` call. The Sifr `increment_callback` (emit line 639) is invoked from each generated closure exactly as required.

3. ✅ Call-depth gate: `crates/sifr_runtime/src/python/call_depth.rs` defines a thread-local depth counter via `PythonCallDepthGuard`. `crates/sifr_runtime/src/python/object_ops.rs:178` calls `enter_python_call()` inside `call_object`. `callback_ops.rs:176` rejects local invocations when `python_call_depth() == 0`, while threadsafe callbacks skip that check. The new test `local_callback_allows_same_stack_reentry_and_rejects_escape` (`callback_ops.rs:277-314`) loops `call_object` twice (proving same-stack reentry) and asserts a direct PyO3 `call1` outside any `enter_python_call` returns the escape error.

4. ✅ Close ordering: `callback_ops.rs:93-112` now does `store.callbacks.remove(&handle)` while holding `CALLBACK_STORE`, returns the `Option<CallbackEntry>` from the closure (releasing `CALLBACK_STORE` when the guard drops), and only then calls `update_object_count(-1)` (RUNTIME_STATE) and `close_object(entry.object)` (OBJECT_STORE → RUNTIME_STATE via `Object::drop`). `grep -n "impl Drop\|impl.*CallbackEntry" crates/sifr_runtime/src/python/callback_ops.rs` returns nothing, confirming no `Drop` on `CallbackEntry` re-introduces the prior inversion.

5. ✅ Re-export metadata: `crates/sifr_driver/src/stdlib/re_exports.rs:71-102` propagates `function_workloads`, `function_defaults`, and `function_varargs` for imported names (and class-owned `Class.method` entries via `is_imported_callable` at line 104). `bootstrap.rs:836-860` (`python_core_re_exports_preserve_callable_metadata`) pins `threadsafe_callback_echo` / `close_local_callback` workloads to `"blocking_io"` and `PythonError` constructor defaults to length 4. `sources.rs:90-96` orders `sifr.python_core` before `sifr.python`, so `stdlib_defs` is populated when the re-export runs.

6. ✅ `ThreadsafeCallback` send marker: `lib/sifr/python_core.sifr:63` now declares `class ThreadsafeCallback:` (no explicit `NonSend` parent), while `LocalCallback` (line 50) retains it.

`reviewer satisfied: no blockers`

**Non-blocking follow-ups worth tracking before milestone closeout** (carried/refined from review-3, none introduced by this round):

- `reset_runtime_state_for_tests` (`python.rs:567-571`) clears RUNTIME_STATE and the object store but not `CALLBACK_STORE`; a future test that forgets `close_callback` will silently leak across tests while live-object accounting reads clean.
- `ThreadsafeCallback` is no longer `NonSend`, but the Sifr type system does not yet enforce that closures passed to `threadsafe_callback(...)` only capture Send-like state — the Rust signature requires `Send + Sync + 'static`, so violations surface at codegen, not at HIR. Phase-contract goal "ThreadsafeCallback requires Send-like Sifr constraints on captured values" remains a type-system task.
- The `kind: String` field of `PythonCallbackMetadata` returned by the runtime is discarded by both `python_core.sifr` (lines 88/97) and the codegen's `lower_py_callback` (which hard-codes `"local"` / `"threadsafe"`); the 5th tuple element is dead.
- `callback_ops.rs:120-122` advances `next_handle` / `next_nonce` before `store_object` runs; a `PyCFunction::new_closure` failure between handle reservation and the final `store.callbacks.insert` permanently wastes that handle/nonce pair (counter waste only).
- `verification/python_interop/run.py` is still in scaffold mode for the callback fixture — it validates file presence but does not compile/execute `callback_roundtrip.sifr`, so contract cases `captured_state_constraints_scaffolded` and `python_exception_mapping` (`callback_contract.json:42-50`) are not yet executed end-to-end.
