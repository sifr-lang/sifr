I have enough context to deliver the review.

## Milestone py_5 Review — Async/Blocking Integration

### Scope verification

**1. Python calls classified as `@blocking_io`** ✓
Every public function in `lib/sifr/python.sifr` (import_module, get_attr, get_item, call, call_attr, close, enter_context, exit_context, all `from_*`/`to_*`/`copy_*`, and `run_coroutine_blocking`) carries `@blocking_io`. Imported workloads are propagated to local callsites via `imported_defaults.rs:32-45`.

**2. Direct Python calls in async Sifr code are rejected unless offloaded** ✓
- `workload_annotations.rs:96-117 reject_async_direct_call` flags any `@blocking_io` function called directly from an async function with `SIFR-ASYNC-0003`.
- `python_async_tests.rs:84-93` proves it fires for `from_int`.
- `python_async_tests.rs:95-106` proves it fires for `run_coroutine_blocking`.
- Negative fixture `direct_python_call_rejected.sifr` documents the same expectation.

**3. No `py.blocking` alias added** ✓
`lib/sifr/python.sifr` exports the existing primitives only; the offloaded fixture goes through `task.spawn_blocking` directly. No new offload alias is introduced.

**4. `py.run_coroutine_blocking` is an explicitly blocking operation** ✓
- Annotated `@blocking_io` (`python.sifr:494`).
- Intrinsic lowered through the same handle-conversion path as other blocking ops (`registry/python.rs:254`).
- Runtime entry (`coroutine_ops.rs:5-17`) acquires the GIL, clones the coroutine handle, drives `asyncio.run`, and stores the result.
- Loop policy: `asyncio.run` uses the installed event-loop policy, so uvloop is respected per the contract.
- Reentry rejection: implicit via `asyncio.run`’s `RuntimeError` when a loop is already running on the thread.

**5. `py.Object` is `NonSend` by default and cannot cross worker boundaries** ✓
- `lib/sifr/python.sifr:85 class Object(NonSend)`.
- `stdlib_class_exports_preserve_parent_markers` verifies the parent marker survives stdlib bootstrap export.
- `task_calls.rs:302-324` (`spawn_blocking`) and `task_calls.rs:174-200` (`spawn_cpu`) reject non-send Ok/Err return types; `offload_worker_captures.rs:23-34` rejects captured non-send values.
- `python_async_tests.rs:115-128 offloaded_python_worker_cannot_return_object_handle` proves NonSend propagation from `Object` to the `spawn_blocking` Result.
- Negative fixture `object_crossing_rejected.sifr` documents the same.

**6. Verification fixtures under `verification/python_interop/fixtures/async_blocking/`** ✓
Contract manifest, 1 positive, 3 negatives matching the DoD; `runner/run.py` requires all four `.sifr` files plus the JSON manifest.

### Non-blocking observations (not gating py_5)

- **No live runtime test for the “reentry rejection” claim**: `asyncio.run` provides it implicitly, but only the positive event-loop drive is covered by `run_coroutine_blocking_runs_python_owned_event_loop`. The DoD doesn’t require a reentry test for this milestone, so this is a follow-up note for py_6/12, not a blocker.
- **`asyncio` import in `coroutine_ops.rs:9` bypasses `validate_import_policy`**: This is correct (it’s runtime infrastructure, not user code), but worth keeping in mind when py_6 documents the trust-boundary surface.
- **Negative `.sifr` fixtures are required-to-exist by `run.py`, but never run through the compiler in this milestone’s validation list**: equivalent semantics are covered by `python_async_tests.rs`, so coverage is sound. py_11/12 verification gates can wire them through `sifr check` when fixture execution is in scope.

### Verdict

reviewer satisfied: no blockers
