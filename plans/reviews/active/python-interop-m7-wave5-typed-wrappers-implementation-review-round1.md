I've completed the review of the Wave 5 typed async Python declaration wrappers implementation. Here is my assessment.

## Review Summary

**Files reviewed:**
- Codegen: `python_interop_async.rs` (new, 826 lines), `python_interop_common.rs` (new), `python_interop_direct.rs` (895 lines), `python_interop_plan.rs`, `entrypoints.rs`, `lib.rs`, `lib_modules_and_codegen.rs`, `generator_bodies.rs`
- Runtime: `async_declaration.rs` (new, 305 lines), `async_value.rs` (new, 558 lines), `foreign_object.rs`, `async_runtime.rs`, `async_terminal.rs`, `python.rs`
- Tests: `python_interop_async_tests.rs`, `python_interop_plan_tests.rs`, `async_declaration_tests.rs`, `async_runtime_tests.rs`
- Lowering gate: `sifr_lowering/src/lower/python_interop.rs`

## Blocking findings

**None.**

## Verified correctness properties

1. **Single terminal/registry.** Both raw `submit_coroutine` and typed `submit_async_declaration` use the same `terminal_for_submission`/`reserve_submission`/`register_submission`/`finish_submission` and shared `PythonTerminal` machinery. `PythonTerminalOutcome` now carries `PythonTerminalValue::{Raw,Typed}` and the raw path rejects `Typed` (async_runtime.rs:177–184), the typed path rejects `Raw` (async_declaration.rs:29–37).

2. **All Python work on the loop thread.** In `submit_typed`, `call_soon_threadsafe(setup)` queues resolve_callable → materialize args → call → `inspect.isawaitable` → `asyncio.ensure_future(..., loop=...)` → `add_done_callback` → `register_submission` → `publish` on the loop thread; the done_callback runs `task.result()` + `convert_output` + `resolve_target` for `Opaque` + `isinstance` + `store_object` under the same GIL.

3. **No Py<PyAny> across threads.** Public `PythonAsyncValue::Object` wraps `PythonAsyncObject`, whose fields (`lease`, `owner`) are `pub(super)`. `PythonAsyncObject` itself is not re-exported. The `Py<PyAny>` is only materialized via `ForeignObjectLease::clone_ref(py)` on the loop thread. `PythonAsyncRequest`, `PythonAsyncValue`, and their transitive contents are `Send + Sync` because they hold `Arc<Mutex<...>>`-wrapped state, never a bare `Py<PyAny>` in a public position.

4. **ForeignObjectLease correctness.** `ForeignObject::lease()` rejects `Closed`/`Poisoned` or missing object (foreign_object.rs:82–94); `close()` sets `Closed` but defers releasing the `Py<PyAny>` when `active_leases > 0` (foreign_object.rs:103–116); lease `Drop` re-checks `Closed + last lease` before taking the object (foreign_object.rs:162–177). `lease.clone_ref` reads `state.object` regardless of status so in-flight requests survive concurrent close.

5. **Receiver freeze across await.** Generated borrowed methods take `&self`; the code builds `PythonAsyncRequest::borrowed_method(&self.__sifr_python_object, ...)` before the `.await`, and the lease pins the identity for the full duration. Consuming methods use `RustParam::SelfValue` (class_method_emitter.rs:664) and `PythonAsyncRequest::owned_method(self.__sifr_python_object, ...)`, transferring ownership without touching raw pointers.

6. **SIFR-PYRES-0002 preserved.** Both `Coroutine + async def` function and method lowering call `reserved_declaration(...)` (python_interop.rs:60–63, 122–126) which emits `PYRES_UNIMPLEMENTED_DECLARATION` at `Severity::Error`, blocking user compilation. Codegen tests exercise the wrapper via HIR fixtures, and runtime tests call `submit_async_declaration` directly; no `#[cfg(test)]` or feature-gated compiler bypass exists.

7. **Cancellation carrier.** `submit_typed` calls `terminal_for_submission(carrier)` (shared with raw) which either claims the exact hook or returns `CancelledBeforeClaim`/`AlreadyClaimed`/`StateUnavailable`. The setup callback atomically `publish`es the submission_id after `register_submission`, and either the carrier's late `request()` finds the id and issues `cancel_submission`, or `publish` observes `requested=true` and cancels inline. Race paths verified from the shared `SubmissionCancellationBridge` implementation.

8. **Panic safety.** `catch_unwind(AssertUnwindSafe(…))` wraps the setup and done bodies (async_declaration.rs:109, 161); panics degrade to `AsyncRuntimeFailed` and complete the terminal.

9. **Method-only async loop plan.** `python_interop_plan.rs` now ORs `class.methods` async effect into `plan.requires_async_loop` (line 93–98), with dedicated test coverage (`method_only_async_python_declaration_requires_owned_loop`).

10. **Codegen preamble.** `module_uses_async_python_declaration` triggers `build_task_cancellation_items()` for modules that use only async python declarations, without dragging in task_scope/join_set (lib_modules_and_codegen.rs:540–553, entrypoints.rs:56–79).

11. **Shape coverage in codegen.** Positional/keyword-only/positional-variadic/keyword-variadic/`python.omit` and recursive list/tuple/dict/record/`Option`/opaque/`Object` all lower in `argument_frame`, `async_input_conversion`, `output_schema`, and `async_output_value`. Bridge targets pass structured segments through verbatim (verified by `resolved_bridge_target_stays_structured_in_typed_request`).

12. **File-size guardrail.** All touched/added files remain under 900 lines: `python_interop_direct.rs` 895, `python_interop_async.rs` 826, `async_runtime.rs` 812, `async_value.rs` 558. The design review's flagged pressure was resolved by extracting `python_interop_async.rs` and the shared `python_interop_common.rs`.

13. **Runtime acceptance.** Runtime tests exercise recursive frames, opaque factory + borrowed method with lease-across-await, concurrent one-loop identity, non-awaitable/Python/conversion failures, and bridge target resolution on the owned loop.

## Non-blocking observations (not verdict-changing)

- `async_from_owned_object` is exported but unused by current codegen (reserved for future owned-parameter transfer).
- The `close_object(client)` call in `typed_factory_and_borrowed_method_preserve_sealed_identity_across_await` proves the lease keeps the identity alive after the public wrapper is dropped; the comment's "public identity should close" phrasing is slightly aspirational because `close_object` drops the wrapper rather than calling `ForeignObject::close()`, but the lease-across-cleanup invariant it tests is a strict superset (it also holds against explicit `.close()` since the lease branch of `close()` leaves `state.object` intact).
- No runtime test exercises consuming async methods or omitted-argument runtime materialization; both are exercised at codegen and are consistent with Wave 5's gating (cancellation and consuming close activate in Waves 6–7).
- Every typed wrapper re-resolves opaque `expected` types on the loop thread; matches the design's stated "loop-thread-only" contract with a note for later profile-driven caching.

VERDICT: SATISFIED
