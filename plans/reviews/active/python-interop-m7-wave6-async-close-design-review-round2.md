I've reviewed the round-1 review and cross-checked the revisions against the current runtime and codegen surface.

**Round-1 blocker-by-blocker verification:**

1. **Cancellation-vs-poison rule** — Revision keys terminal transition only to Python task outcome after output conversion: clean `None` → Closed (including Python-suppressed cancellation), CancelledError/exception/conversion/runtime error → Poisoned. Cancellation request alone never poisons. This aligns with `internal_docs/python_interop_protocol_architecture.md` "later return or exception wins normally." ✓

2. **Pre-registration failure path** — Revision makes the `PythonAsyncRequest` a self-guarding value with an ownership-transfer completion mode and a `Drop` impl. `semantic_close_method` atomically flips Open→Closing and owns the request. Any drop before handoff to a running submission — keyword validation, `ensure_started` (matches `async_declaration.rs:25`), `reserve_submission` under `AsyncRuntimeStopping`, `call_soon_threadsafe`, loop catastrophe — idempotently finalizes Poisoned. Successful/failing done callbacks explicitly transition first; `Drop` then no-ops. Pre-registration shutdown-rejection test named. ✓

3. **Emitter-selection guard** — Revision requires ALL predicates conjunctively (class cleanup == AsyncClose, decorator kind == Coroutine, target segments == `[Self, aclose]`, `consumes_receiver`, empty params, return `Result[None, PythonError]`). Otherwise the Wave-5 method path handles or the shape is rejected. HIR remains user-facing authority. ✓

4. **900-line guardrail** — Revision splits `python_interop_async.rs` (currently 826/900) into a small coordinator + `python_interop_async/conversions.rs` **before** adding semantic behavior; `class_method_emitter` (756) only threads the owning declaration reference. Both remain comfortably under cap. ✓

5. **Shutdown drain of in-flight semantic closes** — Revision states the in-flight explicit semantic close is an ordinary registered typed submission. Shutdown ordering (`async_runtime.rs:372-455`) already provides AdmissionsStopped → SubmissionCancellation (which cancels the exact task and awaits Python `finally` + done callback via `wait_for_submissions_to_drain`) → LoopStop → LoopJoin. The AsyncCleanup hook explicitly is **not** a second close authority (avoiding dual authority). Shutdown-drain ordering test enumerated (finally marker, Poisoned state, zero tasks, joined thread). ✓

6. **Owner-drop-without-close runtime invariant** — Revision explicitly states last-Arc `Drop for ForeignObjectInner` while Open performs safe automatic reference release via the existing immediate/pending queue (matches `foreign_object.rs:132-142`) — no panic, no diagnostic, no fake Poisoned. AsyncClose no-abandonment is statically enforced by existing affine must-use analysis; runtime semantic-close constructors reject Open-state races, Closing, Closed, and Poisoned aliases. Framed as safety containment for compiler bugs, not user-visible cleanup. Policy-agnostic justification (same identity backs drop/close/async_close/context/raw) is sound. ✓

**Nice-to-have list from round-1** — Test enumeration covers begin-close vs lease race, duplicate begin-close, success exact-once, Python failure, non-awaitable, non-None conversion, retained-alias closed/poisoned rejection, cancellation-suppressed → Closed, cancellation-observed → Poisoned with finally, pre-registration shutdown → Poisoned/no task, and shutdown drain ordering. Public coroutine and cleanup gates remain closed; create-pr gate specified. ✓

No remaining dual authority (AsyncCleanup phase excluded; single terminal callback owns state transition), no unsafe races (Open→Closing under the state mutex that guards `lease()`; `Drop` guard is idempotent), no missed guardrail edges.

VERDICT: SATISFIED
