I've reviewed the design, the M8 tracker section, the two normative internal-docs, and the current M5/M7/native async-with implementation. My findings — most severe first — are below.

## Findings

### 1. `propagate()` return semantics collide with a select-notify child fallback (correctness gap)

**File/section:** design §"Cancellation-Unwind Substrate" step 2; existing `crates/sifr_runtime/src/python/async_cancellation.rs:24-39`; call sites in `crates/sifr_runtime/src/python/async_declaration.rs:31-34,45-47`.

M7's `propagate` today calls `carrier.resume_fallback_after_claim()` and *expects* the fallback to abort the current Tokio task; on `Invoked`/`AlreadyResumed` it `yield_once().await` and, if control returns, returns `Err(PythonRuntimeError::AsyncRuntimeFailed("… returned without terminating the native task"))`. That contract is only sound when the fallback is `AbortHandle::abort`.

Wave 1 rebinds the child carrier's fallback to a "private cancellation notification" that wakes the parent `select!` rather than aborting the task. Consequently, when Python-task cancellation reaches `submit_async_declaration` from inside the body:

- `propagate` resumes the child fallback → the select-notify hook fires (waking the select on a Waker/atomic, not aborting).
- `yield_once` returns.
- `propagate` returns `Err(PythonError::runtime(AsyncRuntimeFailed(…)))`.
- The body outcome classifier (`classify_cause_kind` in `python_context.rs:609-620`) sees a plain `PythonError`, not `CancellationError`, so cancellation is *mislabelled as `OrdinaryError`* (or as `PythonException` if `PythonError.__sifr_python_error` is populated), and the resulting `__aexit__` call would receive a `SifrExitCause` of the wrong kind.

The design must (a) specify a distinct M7 propagate variant that yields a designated "cancellation-received" marker rather than a runtime error, or (b) make the select-notify fallback ensure `propagate`'s future is dropped rather than allowed to return, or (c) keep the abort-based fallback and instead express masking a different way. As written the two-wave plan cannot preserve "cancellation remains primary" once the child fallback is not `abort`.

### 2. "Own self" transfer for async exit is not specified against the M7 request substrate

**File/section:** design §"Owned-Loop Enter And Exit Operations" bullets 2–3; contrast `crates/sifr_runtime/src/python/context_ops.rs:80-95` (sync consumes `ObjectHandle` by value) with `crates/sifr_runtime/src/python/async_declaration.rs:60-122` (submits an `Arc<PythonAsyncRequest>` and `receiver.clone_ref(py)` on the loop thread).

Sync async_context requires exactly-once consuming identity of `own self`; sync M5 achieves this by passing the `ObjectHandle` by value and doing `object.close()`/`object.poison()` on the loop thread. The design says async exit "owns or leases only compiler-owned sealed handles" but does not describe:

- how the sole live `ObjectHandle` is placed into the async request payload before submission,
- what happens if setup fails, the request panics, or the terminal callback panics mid-flight (see `build_setup_callback` at `async_declaration.rs:263-283`, which currently drops the arc/target refs — but a consumed handle needs to be *poisoned* rather than merely released),
- how the identity is discharged in the enter-succeeded-conversion-failed branch, where the entered value has to be dropped but the *manager* must still be exited (see finding 3).

Without an explicit ownership channel, an implementation is likely to leak a still-open manager on any panic path, violating exactly-once. Specify the request-side ownership model as concretely as `context_exit_normal(ObjectHandle)` does for sync.

### 3. Enter-conversion failure after `__aenter__` succeeded has no handle-preservation strategy

**File/section:** design §"Concrete Body Outcome And Suppression" bullet 6.

For async, the "existing recursive typed output converter" runs on the loop thread inside M7's done-callback (`async_declaration.rs:144-155`), which materializes the converted value into `PythonTerminalValue::Typed`. Sync M5 handles this by keeping the manager identity alive independently of the entered value (`python_context.rs:280-326`, where the manager handle is retained even if conversion fails). The design asserts "aexit still runs exactly once with an ordinary Sifr conversion cause" but never explains how the manager identity is preserved for a subsequent async-exit submission when the loop-thread conversion fails inside M7 — the current typed done-callback returns a `PythonError` and terminates the submission, and the manager handle lives outside the request payload.

Specify: the manager `ObjectHandle` is owned by generated code (not consumed by the async enter request), so it remains available for exit; async enter submits a *borrowed* handle and returns the converted entered value or a distinct conversion-failure result, distinct from receiver ownership. Otherwise "aexit still runs" is not implementable.

### 4. `__aexit__` cancellation masking does not describe the task-local at the cleanup site

**File/section:** design §"Cancellation-Unwind Substrate" step 3.

Design says "run `__aexit__` under a fresh, unrequested cleanup carrier so the original request is masked." But whether that fresh carrier is *installed as the task-local* for the aexit future or only *passed as the `carrier: Option<&CancellationCarrier>`* into `submit_async_declaration` is unstated. If it is only passed to M7, any nested Python operation triggered inside `__aexit__` that reads `__sifr_current_task_cancellation()` (e.g., a further generated aenter/aexit) picks up the *cancelled* child carrier and re-runs the entire cancellation dance. If it is installed as task-local, the design should say so and describe the scoping (`__SIFR_TASK_CANCELLATION.scope(cleanup_carrier, aexit_future)`). Fix by making the task-local installation an explicit obligation.

Also missing: what happens if the caller's runtime shutdown fires *during* aexit. The design references M7 registry drain, but does not say the registry cancels the exact Python task without going through the (fresh, unrequested) carrier. Add that guarantee explicitly (it is likely already true of `cancel_submission`, but needs to be normative for M8).

### 5. Top-level and mixed-scope carrier absence is not specified

**File/section:** design §"Cancellation-Unwind Substrate" step 1; contrast `crates/sifr_codegen/src/preamble/task_cancellation_runtime.rs:1-38` (task_local is installed only when `__SifrCancellationCarrier::new` runs, i.e., under TaskScope/TaskGroup children).

`async with` in a `main()` body or in any async function invoked without a wrapping task carrier will find `__sifr_current_task_cancellation() == None`. Under such conditions the "claim the current task's cancellation carrier" step cannot occur, and outer cancellation is a raw Tokio task abort, which will drop the body future *without* running `__aexit__`. That directly violates "cancellation cannot abandon async cleanup" (tracker line 615).

The design must either (a) mandate that any function containing `async with` on a Python-async_context manager runs under an ambient carrier (compiler-inserted top-level carrier), or (b) prove the abort path still runs generated Drop-based cleanup that executes `__aexit__` via a blocking terminal path. Neither is documented. Without one of these, top-level `main` usage in the demo will silently skip cleanup on cancellation.

### 6. Wave 1 leaves an executable-but-unreachable public path — reviewability tension

**File/section:** design §"Wave 1: Gated Async-Context Substrate" §"Declaration Contract Behind The Gate" point 3, and §"Wave 1 Validation".

The plan gates the visible public form with `SIFR-PYRES-0002` while lowering, dedicated HIR, codegen, and runtime are fully wired. That means the *only* Wave 1 evidence for correctness will be synthetic (construct-HIR-directly codegen tests and unit tests). If the reservation is emitted *before* the shape checks, invalid-shape diagnostics (`SIFR-PYCTX-*`) would be masked; if *after*, then any `SIFR-PYCTX-*` regression in a real user program is invisible until Wave 2 lifts the gate. The design says "Invalid shapes emit their stable `SIFR-PYCTX-*` … rather than being hidden by the reservation" but does not specify the diagnostic ordering rule in the LowerCtx pipeline (`crates/sifr_lowering/src/lower/python_interop.rs:632-641` currently emits PYRES immediately on the reserved arm). Make the ordering explicit and add a Wave 1 test that both shape rejection and cleanup-obligation rejection fire even when the reservation is active. Otherwise the two-wave split is not independently reviewable in the sense the design claims.

### 7. `PythonError` replay borrowing across the async exit request is under-specified

**File/section:** design §"Owned-Loop Enter And Exit Operations" bullet 2 ("cloned original `PythonError` replay"), plus interop-protocol lines 180-197 (shared borrow / release-once semantics).

For sync M5, `context_exit_python_error` takes `&PythonError` (`context_ops.rs:84-89`) and calls `error.replay(py)` under the GIL from the same thread. For async, the design says "cloned original PythonError replay" is placed in a "closed private cause enum". If the clone is a whole `PythonError` value, the request now owns a replay borrow that is released when the request drops. But the primary error still owns its own copy and remains live until the async-with block exits. The protocol says "The capability can be replayed by multiple nested exits while borrowed but released exactly once by its final owner." The design must describe which of the two `PythonError` instances is the "final owner", how the runtime keeps the retained triple pinned across a possibly-different thread's request execution, and how a `Suppress` decision releases the triple exactly once even though *two* PythonError values hold it. Concretely, either (a) the request holds an `Arc`-shared replay handle and the primary's `PythonError` is downgraded on Suppress, or (b) the request only borrows the triple identity and the primary owns the ref. Pick one and document it — otherwise this is a live double-release / leak risk.

### 8. `SifrBoundaryError` construction across the loop thread is not covered

**File/section:** design §"Owned-Loop Enter And Exit Operations" bullet 3 (materialize `SifrBoundaryError`), current `crates/sifr_runtime/src/python/context_ops.rs:51-77,173-195` (registered lazily once per process, constructed under GIL).

Sync M5 registers `SifrBoundaryError` on demand during `register_boundary_error(py)` and constructs it under the same GIL attach. Async exit occurs on the loop thread inside M7; the design does not say whether the SifrBoundaryError type is registered from the loop thread's first attach, or shared with sync via the `OnceLock`. If the loop thread hits async_context first, ensure `register_boundary_error` is invoked before the async_context enter/exit ever runs. Cheap fix; but the design should say who owns that registration.

### 9. Async closure design must specify break/continue lowering inside the select body

**File/section:** design §"Concrete Body Outcome And Suppression" ("async closure whose private outcome …").

Existing M5 achieves this via `rewrite_context_control_flow` in `python_context.rs:523-599` for a *synchronous* closure. Wave 1 will need the same rewriting inside an `async {}` closure — including inside `select!` arms — with the extra constraint that returning from an async block does not exit the enclosing function unless the enclosing function's `Result` matches the private outcome enum. The design gestures at "share control-flow rewriting helpers with M5 by responsibility" but does not spell out that async-block return-of-outcome is a *different* return type than an outer `?` and that the outcome must be lifted back to the outer function outside the `select!`. Not correctness-fatal, but reviewers cannot judge whether the "async closure preserves borrowed locals and all fallthrough/return/break/continue/error outcomes without spawning" without seeing the outcome enum's exact shape (in particular how it distinguishes cancellation-selected from body-error and from control-flow) and how it composes with the `try_closure_error_type_info` stack that `python_context.rs:87-98` currently depends on.

### 10. File-size guardrail: split axis under-specified

**File/section:** design §"Concrete Body Outcome And Suppression" ("Split new async-context lowering/codegen/runtime modules"); design §"Likely Implementation Surfaces".

`python_context.rs` is 675 lines today. Adding async-context lowering (dedicated HIR variant), async body-outcome rewriting, and cancellation-scope emission inside the same file will trip the 900-line cap. The design lists "stmt_support_emitter/ split into shared context outcome, Python async-context, and cancellation-scope responsibilities" without naming the modules or drawing the responsibility line between sync M5 (`python_context.rs`) and async M8. Because AGENTS.md forbids adding more code to an oversized module, and the existing M5 emitter is already large, a specific pre-split plan is needed before Wave 1 codegen begins. Nominate concrete module names and target owners (e.g., a `python_context/` directory with `outcome.rs`, `sync.rs`, `async.rs`, `cancellation_scope.rs`).

### 11. Wave 2 live-compiled evidence: no candidate driver named

**File/section:** design §"Wave 2: Atomic Activation And Evidence" point 3.

Wave 2 requires "one offline compiled database/session package using a real async Python library or a real async driver surface available in the locked area environment." M5's sync gate uses SQLite (bundled). No obvious pure-stdlib async driver exists in the locked area (`aiosqlite`, `asyncpg`, etc. are external). The design leaves this open with "No external service or network is permitted" but nominates no candidate. If no in-area driver is available, Wave 2 blocks indefinitely. Either name the specific package (e.g., document that `aiosqlite` will be added to the locked area, or fall back to `asyncio.Queue`-based session over stdlib), or explicitly commit to shipping a bundled equivalent. Otherwise the evidence gate is aspirational.

### 12. Ordering guarantee for mixed sync-inside-async and native `AsyncExitCause` clarity

**File/section:** design §"Existing Authorities To Reuse" bullet 3; §"Cancellation-Unwind Substrate" step 5.

The design correctly says "Native user-defined async context managers retain their own `AsyncExitCause` protocol and never provide Python classification." Fine. But `AsyncExitCause` today (`preamble/types_and_errors.rs:599-644`) has variants `Normal`, `Return`, `OrdinaryError(String)`, `Timeout`, `Cancellation`, `RuntimeFault(String)` — noticeably missing `Break`/`Continue`. Native `async with` lowering in `async_with_and_for.rs:65-159` currently emits only `AsyncExitCause::Normal`/`Return` (no break/continue and no error paths). The design states the native path is "an implementation substrate to generalize" — but if M8 does not extend `AsyncExitCause` (or the native lowering) to cover break/continue and error paths, then any *native* async-with reachability of loop control inside its body remains buggy in a way that will be exposed once M8 lands and shares helpers. Either scope out generalizing the native path to M8, or explicitly declare it out-of-scope so the two paths stay decoupled. Right now the design is ambiguous.

### 13. Discharge point for `AsyncContextOnly` obligation across every outcome

**File/section:** design §"Concrete Body Outcome And Suppression"; existing `crates/sifr_lowering/src/lower/mod_context.rs:236-247`.

The design lists the outcomes (fallthrough, return, break/continue, `Err`, timeout, cancellation, runtime fault, and conversion-after-enter) but never says explicitly that the `AsyncContextOnly` must-use obligation is discharged in *all* of them exactly once. Sync M5 discharges by calling `context_exit_*` in every branch; the design should mirror that literally, and Wave 1 tests should include a construct-HIR test that exercises each branch and asserts the obligation is discharged (via the existing liveness side table). Otherwise a later refactor can silently drop a branch.

### 14. Nested-carrier scope pop and generation invariants

**File/section:** design §"Cancellation-Unwind Substrate" step 5.

Nested async-with produces a chain of `_ = OUT.claim(child=CHILD1); _ = CHILD1.claim(child=CHILD2); …`. The design must state that each claim lease is dropped in strict LIFO order (guaranteed by the enclosing scopes) and that `resume_fallback_after_claim` is invoked *before* the enclosing scope's own claim is released, so that the outer body observes a resumed fallback rather than a plain unclaimed state. Without this ordering rule spelled out, a subtle codegen bug could release the outer claim first, leaving the child carrier fallback un-resumed and the cancellation observed only when the outer scope's own next await fires — which changes cancellation timing. Nail down the ordering.

### 15. `PythonExitDecision::Suppress` on originating async Python error must be reachable from the loop-thread response

**File/section:** design §"Owned-Loop Enter And Exit Operations" bullet 3 ("closed `PythonExitDecision`"); sync counterpart `context_ops.rs:213-221`.

`PythonExitDecision` is a Rust enum computed by `exit_decision(value.is_truthy())` under GIL. In sync M5 that value is returned via the direct call. In async M8, the request result flowing back through the M7 typed converter must carry the enum, not just a bool. The design says the request "converts Python truthiness into the closed `PythonExitDecision`" but does not describe the request output type: is it a specialization of `PythonAsyncValue`, a distinct terminal variant, or a re-encoding into a Sifr `Result[python.ExitDecision, PythonError]`? Sync's `finish_context_exit` also handles `close()`/`poison()` at that boundary. Specify concretely so reviewers can check that (a) the exit decision survives without a spurious `PythonError` round-trip and (b) close/poison are invoked on the same thread that observed the terminal outcome (loop thread), consistent with sync semantics.

---

Overall: the direction (masking cancellation, exact-once, replay preservation, no native `AsyncExitCause` reuse, no `sifr_runtime` Tokio dependency, dedicated HIR) is sound and consistent with the interop-protocol and M7 substrate. But the coupling between (a) the child-carrier fallback semantics and (b) M7's `propagate` contract is the load-bearing correctness step, and the design as written is incompatible with the current `propagate` implementation. Combined with the unspecified enter-conversion-recovery path, ambient-carrier requirement at top-level main, and unnamed live driver, these are blocking issues rather than nits.

VERDICT: CHANGES_REQUIRED
