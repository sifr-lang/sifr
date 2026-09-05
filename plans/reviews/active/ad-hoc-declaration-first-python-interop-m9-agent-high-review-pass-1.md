## Blocking findings

1. **Critical — foreign/asyncio handler captures are not type-checked for sendability or Python identity.**
   [callbacks.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_lowering/src/lower/python_interop/callbacks.rs:489) validates only callback arguments, success values, and handler-error types—not the actual handler’s captures. The generated foreign bound in [scope_and_function_types.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/function_emitter/scope_and_function_types.rs:804) is only a Rust backstop. Worse, opaque Python wrappers contain sendable runtime fields without a Rust-level non-`Send` marker in [class_emitter.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/class_emitter.rs:139), so a nested handler capturing an opaque Python owner can satisfy `Send + Sync + 'static` and execute that captured identity on a foreign thread. The required static close-from-callback analysis is also absent; only the runtime thread-local guard exists in [state.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/state.rs:269).
   **Required fix:** analyze resolved handler captures at each callback attachment, reject all non-send captures, reject Python identity recursively for foreign dispatch, require immutable/shareable captures for parallel dispatch, add a Rust auto-trait backstop for opaque wrappers, and implement the promised static same-owner close rejection.

2. **Critical — asyncio cancellation is not bidirectional, and close/shutdown can wait forever.**
   The only cancellation edge is Python future → Sifr carrier in [asyncio.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/asyncio.rs:288). The fallback aborts the Tokio task at [asyncio.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/asyncio.rs:330), but Sifr-side cancellation never cancels the exact Python future. An aborted task also exits before `schedule_completion`, leaving that Python future pending. Owner close merely waits for `active_calls == 0` in [state.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/state.rs:463); it neither requests cancellation nor joins terminal acknowledgements. A pending handler can therefore hang semantic close or process shutdown indefinitely.
   **Required fix:** give each accepted asyncio entry one CAS-controlled terminal record containing its carrier, task, and Python future; implement both cancellation directions; always schedule terminal Python cancellation on Sifr abort; register active entries with the owner; and cancel/join them asynchronously during close.

3. **High — runtime shutdown cannot execute async unregister authority.**
   Shutdown changes the loop lifecycle to `Stopping` and removes the loop handle before callback shutdown in [async_runtime.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/async_runtime.rs:394). Callback owners are then shut down at [async_runtime.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/async_runtime.rs:419). Async-close and async-context unregister actions submit through the normal blocking declaration path in [ownership.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/ownership.rs:247), whose `ensure_started` rejects `Stopping`. Normal runtime shutdown of such a retained owner therefore reports failure and closes locally without successfully running Python unregister-first cleanup.
   **Required fix:** distinguish public admission shutdown from internal cleanup submission, retaining the loop authority until callback unregister and drain finish.

4. **High — retained asyncio rollback is not exception-safe and can synchronously deadlock the executor.**
   Explicit async rollback covers only a failed Python submission in [callback_frame.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/python_interop_async/callback_frame.rs:310). Handler reconciliation, result conversion, and owner conversion/commit can still return early afterward. The provisional group’s `Drop` then invokes synchronous condition-variable draining in [ownership.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/ownership.rs:103). If an accepted handler needs the same current-thread Tokio executor, this blocks that executor and deadlocks.
   **Required fix:** generate one async finalization frame covering submission, handler reconciliation, conversion, and attachment, with explicit awaited rollback on every failure path. Async groups must never depend on blocking `Drop` for correctness.

5. **High — retained typed handler failures are lost on async close/context and ordinary later owner operations.**
   Async method codegen receives but deliberately ignores `_owner_retained_errors` in [conversions.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/python_interop_async/conversions.rs:74). Runtime close uses the “typed observer” variant, which suppresses the redacted runtime error in [state.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/state.rs:257), assuming generated code will extract the typed slot—but async close does not. Async context exit similarly consumes the callback owner without generated typed-slot reconciliation in [async_context.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/async_context.rs:49). Non-consuming owner methods also never inspect retained failure slots; only the synchronous consuming-close branch does so in [python_interop_direct.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/python_interop_direct.rs:546).
   **Required fix:** capture the owner and every typed slot before consuming async close/context state, reconcile after drain, preserve primary/secondary ordering, and check retained failures on later owner operations as documented.

6. **High — synchronous Python-primary reconciliation can miss late handler failures and cleanup failures.**
   Sync wrappers attach callback evidence before draining at [python_interop_direct.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/python_interop_direct.rs:269). They drain afterward, but then map the Python result at [python_interop_direct.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_codegen/src/python_interop_direct.rs:288), which returns early on the Python error before post-drain typed reconciliation or cleanup-result processing. A background callback that fails during drain is therefore absent from the promised Python-primary secondary evidence. Context exit also discards `callback_close` whenever exit itself fails in [ownership.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/ownership.rs:241).
   **Required fix:** drain first, then perform one reconciliation step over the Python outcome, typed failures, and cleanup outcome, preserving deterministic primary/secondary ordering.

7. **High — escaped retained foreign callables retain Sifr captures after owner close.**
   `foreign_callback_with_owner` captures `decode`, `handler`, and `encode` directly inside the Python `PyCFunction` in [foreign.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/foreign.rs:320). `retain_in_owner` retains and later closes only one Python object handle in [foreign.rs](/Users/yaseralnajjar/work/sifr/codebase/crates/sifr_runtime/src/python/callbacks/foreign.rs:169). An escaped additional Python reference keeps the function closure—and all captured Sifr resources—alive after owner close. Calls reject correctly, but capture release and declared-owner lifetime do not hold.
   **Required fix:** make retained foreign shells capture only an owner/token indirection, with the actual typed target owned and released by the callback group after unregister and drain, matching the asyncio/current architecture.

8. **High — the claimed validation/evidence gate is not the gate that runs compiled callback examples.**
   Profiles select the `callbacks` suite, for example [create-pr.json](/Users/yaseralnajjar/work/sifr/codebase/verification/profiles/create-pr.json:112). That command maps only to `--group callbacks` in [runner.py](/Users/yaseralnajjar/work/sifr/codebase/verification/areas/python_interop/runner.py:57), not `--callback-examples`, whose compiled execution path is separate in [run.py](/Users/yaseralnajjar/work/sifr/codebase/verification/areas/python_interop/runner/run.py:266). Thus CFFI/Kafka/asyncio/PubSub binaries are not unconditionally registered in create-PR, merge, nightly, and release as claimed.

   Additional evidence overclaims:

   - Cancellation evidence covers only Python future → Sifr handler in [callback_evidence.json](/Users/yaseralnajjar/work/sifr/codebase/verification/areas/python_interop/fixtures/callback/callback_evidence.json:19), not exact bidirectional cancellation.
   - The Pub/Sub fixture awaits `emit` completely before `aclose` in [declaration_callback.sifr](/Users/yaseralnajjar/work/sifr/codebase/verification/areas/python_interop/fixtures/pubsub/declaration_callback.sifr:27), so its `close=drained` marker proves no concurrent drain.
   - CFFI exercises only current-thread dispatch in [declaration_callback.sifr](/Users/yaseralnajjar/work/sifr/codebase/verification/areas/python_interop/fixtures/cffi_callback/declaration_callback.sifr:4), not the design’s native/background foreign case.

   **Required fix:** add a manifest suite for compiled callback examples and select it in all four profiles; add true active-close/drain, Sifr→Python cancellation, async concurrent-close/shutdown, capture rejection, and CFFI foreign-thread cases before marking evidence passing.

9. **Medium — durable status documentation contradicts atomic activation.**
   The embedded architecture says `PYCB` remains reserved in [python_interop_architecture.md](/Users/yaseralnajjar/work/sifr/codebase/internal_docs/python_interop_architecture.md:173), the declaration architecture says callback decorators remain reserved in [python_interop_declaration_architecture.md](/Users/yaseralnajjar/work/sifr/codebase/internal_docs/python_interop_declaration_architecture.md:322), and the roadmap still says callbacks are sequenced next in [roadmap.md](/Users/yaseralnajjar/work/sifr/codebase/plans/roadmap.md:129). This directly misses the Wave 3 durable-doc/roadmap task.
   **Required fix:** reconcile all durable status authorities after the implementation and evidence gaps are closed.

## Non-blocking findings

None.

## M9 task mapping

| M9 task | Status |
|---|---|
| Callback lifetime/dispatch/concurrency metadata | Implemented |
| Checked argument/result conversion and `SifrCallbackError` | Partial; reconciliation gaps |
| Current-thread non-escape/non-send behavior | Implemented |
| Foreign `Send + Sync`, serial/parallel, identity exclusion | Blocked by missing capture analysis |
| Asyncio `AsyncCallable` and bidirectional cancellation | Blocked |
| Net-new `AsyncCallable` type/ABI support | Implemented; no independent ABI defect found |
| Required concurrency and pre-wait reentrancy | Implemented |
| Retained owner aggregation/rollback | Blocked by foreign capture retention and async rollback deadlock |
| Open/closing/closed, unregister-first, drain/release/shutdown | Sync paths largely implemented; async/runtime shutdown blocked |
| Static/runtime close-from-callback rejection | Runtime implemented; static analysis missing |

## Acceptance mapping

| Acceptance item | Result |
|---|---|
| No callback outlives its declared owner | Fail: retained foreign captures can outlive owner |
| Foreign callbacks cannot smuggle Python identity | Fail: captured identity is unchecked |
| Serial reentrancy fails deterministically | Pass for inspected foreign and asyncio paths |
| Owner close drains accepted calls and rejects later calls | Partial: synchronous paths pass; async close can hang and does not cancel |
| Async callbacks block neither executor | Fail: implicit retained-group rollback can block/deadlock Tokio |

## Validation and evidence mapping

- Current, foreign serial/parallel, and asyncio round trips: present.
- Wrong arity/type/result conversion: present.
- Serial reentrancy and foreign parallel overlap: present.
- Capture/sendability and Python-identity exclusion: insufficient; boundary types only.
- Concurrent close, callback-after-close, shutdown: synchronous coverage exists; async concurrent-close and shutdown semantics are absent/broken.
- Swallowed handler failure and lowest-entry selection: unit coverage exists.
- Python-primary plus secondary handler failure: does not cover failure arriving during drain.
- Bidirectional cancellation: only Python→Sifr is tested.
- Compiled CFFI/Kafka/asyncio/PubSub fixtures: present and reportedly runnable, but not selected by authoritative profiles; Pub/Sub does not exercise active drain.
- Atomic activation: unsupported because active capability rows overclaim cancellation, cleanup, and compiled-gate evidence.

Read-only inspection covered the complete requested commit range. `git diff --check` was clean, all touched callback/compiler modules remain under 900 lines, and the only current worktree change remains the explicitly ignored `third_party/ruff` submodule.

VERDICT: BLOCKED