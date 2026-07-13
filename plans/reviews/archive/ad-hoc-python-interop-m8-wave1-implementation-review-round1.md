I've completed the review. Let me produce the findings.

## M8 Wave 1 Review Findings

Overall the Wave 1 substrate landing is comprehensive: dedicated `HirAsyncWithKind::Python` with source-level active-error info; ordering discipline for shape/obligation vs `PYRES-0002`; scoped-borrow discharge; module splits under the 900-line cap; distinct `PythonTerminalValue::ExitDecision`; `submit_async_context_exit` rejects the other terminals; `PythonAsyncRequest::semantic_context_exit_method` routes through the existing `finish_semantic_close` poison edges; `register_boundary_error` runs before the loop starts; the ambient root carrier wraps async `main`; and the runtime test proves the child carrier waits for Python `finally` before the sticky arm wins. The seven lowering / nine codegen / five runtime tests do exercise the branches they enumerate and syn-parse the rendered Rust.

The following defects survived verification. All references are to files with the pending working-tree diff.

### 1. Cancellation-cleanup return races the parent abort — `Err(internal_error)` reaches the caller instead of cancellation (blocking)

`crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:190-191`

The `None =>` branch renders:

```
let _resume = {scope}.release_and_resume_parent();
return Err({internal_error});
```

`release_and_resume_parent` calls the parent's fallback hook, which for a Sifr task carrier is `AbortHandle::abort()` (see `crates/sifr_codegen/src/preamble/task_cancellation_runtime.rs:19-20`). `abort()` only takes effect at the next poll boundary; if the current poll returns `Ready(...)`, the task completes with that value and abort has no effect (tokio's documented behavior).

Because the generated code returns `Err(internal_error)` synchronously *without* a yield between the abort() call and the return, the enclosing task future returns `Ready(Err(SifrPythonAsyncContextError("async context cancellation handoff failed")))`. The parent `JoinHandle` therefore observes that internal error, not `JoinError::Cancelled`. The M8 acceptance property "cancellation cannot abandon async cleanup" holds (`__aexit__` did run), but the design's step-4 guarantee "Cancellation/timeout remains primary" is silently violated: the outer task surfaces a bounded runtime failure that looks like an internal bug rather than cancellation.

The M7 counterpart in `crates/sifr_runtime/src/python/async_cancellation.rs:28-38` shows the required shape — it does `yield_once().await` *after* `resume_fallback_after_claim` returns `Invoked | AlreadyResumed`, and only errors out on the `NotRequested`/`ExactClaimActive`/`FallbackUnavailable` legs (the design's "explicit bounded runtime failure" case).

Concrete failure scenario. A Sifr `TaskGroup` child runs `async with make_async_transaction() as tx: … await long_op(tx) …` on the M7 loop. The parent cancels the child. The child carrier's exact hook cancels the exact asyncio task, `__aenter__` finishes, `__aexit__` runs to completion under the fresh cleanup carrier, exit records evidence, we call `release_and_resume_parent()` which fires `AbortHandle::abort()`, then immediately `return Err(SifrPythonAsyncContextError)`. The parent `TaskGroup` sees this child failed with a "cancellation handoff failed" error and reports it as a task exception. The child was never observed as cancelled. Wave 2 aiosqlite cancellation fixtures will fail this contract.

Fix. Mirror `propagate()`: pattern-match the `CancellationResume` outcome; on `Invoked | AlreadyResumed`, do `tokio::task::yield_now().await` (or `std::future::pending::<()>().await` for a stronger guarantee) so tokio can preempt via abort before the `Err(internal_error)` is ever returned; return the internal error only on `FallbackUnavailable`/`NotRequested`/`ExactClaimActive`/`StateUnavailable`.

### 2. Successful `__aenter__` + concurrent parent cancel can drop the entered value without running `__aexit__` (blocking substrate defect — will manifest in Wave 2)

`crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:118-143`

The generated enter path races the sticky child-notification against the enter future in a biased select (cancel arm first). Between `PythonTerminal::complete` releasing the M7 exact claim (`crates/sifr_runtime/src/python/async_terminal.rs:65-79`) and the outer select's next re-poll, any parent `request_cancel` reaches the child carrier without an exact hook and fires the child's fallback → `StickyCancellation::notify` — even though `__aenter__` succeeded and the enter future is `Ready(Ok(value))`.

On the next outer poll, the biased cancel arm wins, we take the `None => { poison_object(manager…); return Err(internal_error); }` leg. The successfully entered Python resource (still `__aenter__`-side entered, its `ObjectHandle` inside the dropped enter future) is discarded without calling `__aexit__`. The design's "exit occurs exactly once for cancellation" invariant is violated for the enter-race window; Python-side finalizers such as file locks, DB sessions, or `aiosqlite.Connection` leaks are the concrete cost.

The design's step-2 argument that "the race does not select cancellation until Python `finally` is terminal" applies to a Python await that received `CancelledError` in-flight (via M7's exact hook + `propagate`'s sticky notify). It does not cover the "terminal completed Ok, then parent cancel arrives before the outer re-poll" ordering, and the runtime test `child_carrier_waits_for_python_finally_before_cancellation_race_wins` uses a `wait-enter` manager that only completes via cancellation, so it does not exercise this path.

Concrete failure scenario. `Transaction.__aenter__` completes quickly (e.g., grabs a semaphore); the outer task is cancelled roughly concurrently; done_callback completes the terminal and releases the exact claim; parent's exact hook fires child.request_cancel, which now finds no exact and fires the sticky fallback; the outer waker fires; biased select re-polls, cancel arm ready, `None` branch runs; the Sifr code poisons the manager but does not submit `__aexit__`; the transaction is Python-side entered forever.

Fix options (in order of my preference):

- Do not race the enter. Just `await` `submit_async_context_enter(...)` under the `child` scope. If external cancel fires, M7's exact hook cancels the Python task and `propagate()` returns `Err(propagation_error)`, which the code can distinguish from a genuine `__aenter__` failure and route to the `None`-equivalent cleanup (still with a fresh `cleanup_carrier` and eventual resume). The enter path never held an entered value, so there is nothing to `__aexit__`.
- Keep the biased race but, on the `None` arm, poll the pinned `enter_future` once more (`poll_unpin` / `future::poll_immediate`) before deciding: if it returns `Ready(Ok(value))`, thread that value into the same cancel-cleanup that the body-cancel path uses (submit `__aexit__` with `PythonAsyncExitCause::Sifr(Cancellation, …)`).

Either fix must also apply to the body select the same way — the current body select happens to be safe only because every successful body outcome already calls `__aexit__` via `normal_exit`/`return_arm`/`loop_arms`; a raced-away body error is subsumed by the cancel-cleanup `None` arm which does run exit, so that side is OK. The enter side is the asymmetric one.

### 3. Cancellation-cleanup path does not record ignored `Suppress` (correctness of evidence stream)

`crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:184-189`

The `None` cleanup only records evidence when `cleanup_result` is `Err`:

```
if let Err({cleanup_error}) = {cleanup_result} {
    sifr_runtime::python::record_context_cleanup_evidence(
        "cancellation:CancellationError",
        &{cleanup_error},
    );
}
```

Design §"Concrete Body Outcome And Suppression" line 183 mandates: "ordinary Sifr errors, timeout, cancellation, and runtime fault use `SifrBoundaryError`; truthy decisions are recorded and ignored". The parallel `sifr_error_exit` renderer at `async_context.rs:346-348` correctly emits `record_context_ignored_suppression("cancellation:CancellationError")` on `Ok(Suppress)`. The `None`-arm renderer omits this call, so a Python manager that returns truthy from `__aexit__` under a Sifr cancellation cause never appears in the evidence ledger. Wave 2's compiled positives verify evidence-stream shape and will regress here.

Fix. Add an `Ok(PythonExitDecision::Suppress) => record_context_ignored_suppression("cancellation:CancellationError")` arm (and an explicit `Ok(Propagate) => {}` for symmetry) instead of the current `if let Err(...)`. `Ok(Propagate)` requires no action, matching the sibling emitter.

### 4. Optional — enter-cancel `None` branch swallows the parent cancellation state without resuming (minor)

`crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:139-143`

If cancellation fires *before* `__aenter__` reaches Python (bridge published before submission) or during it, the enter future returns `Err(runtime propagation_error)` rather than being raced away as `None`. In that path the code executes `Some(Err(error)) => { poison_object(...); return Err(enter_error); }` and never calls `release_and_resume_parent`. The parent's `state.requested` is `true` but its `AbortHandle::abort()` is never fired — the enclosing task returns `Err(enter_error)`, mislabeling cancellation as an enter-Python error, similar in flavor to finding #1 but without a fresh cleanup carrier or evidence. Not strictly required by the design (which only mandates resume "after exit and evidence recording"), but the same "cancellation remains primary" intent is broken. Consider inspecting `enter_error`'s runtime kind (or the child carrier's `state.requested`) to detect the propagation-error case and route through the same cleanup shape.

### 5. Optional — codegen test coverage is syn-only; borrow-check and runtime semantics are not verified in Wave 1

`crates/sifr_codegen/src/stmt_support_emitter/python_async_context_tests.rs:84-87` and analogues

The three codegen tests parse the rendered Rust with `syn::parse_file` but never compile it. Because the reservation gates the public form of `async with` on `PYRES-0002`, no e2e demo compiles this codegen either. This means findings #1 and #2 above are pre-Wave-2 defects that Wave 1 cannot catch even in principle. Once the reservation is lifted, the aiosqlite path should exercise them and rustc should validate the borrow shape of `enter_future` vs. the subsequent `{manager_name}.__sifr_python_object` moves (which appear safe under NLL because `enter_future`'s last use is in the enter `select!`, but relying on NLL for a compiler-generated pattern is fragile). Consider adding a compile-only integration test that instantiates the emitter, writes the rendered Rust into a fixture crate, and invokes `cargo check` against it — behind Wave 1's existing reservation, without ever executing the code.

### 6. Optional — nested body-error paths never resume parent, matching design but leaves cancellation-requested state opaque

`crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:167` (body-error arm), `crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:280-317` (`python_error_exit`), `crates/sifr_codegen/src/stmt_support_emitter/python_context/async_context.rs:319-362` (`sifr_error_exit`)

If the body errors with a `PythonError` that is not itself cancellation, but parent had *already* been cancelled (child scheduled its exact hook but body's own Python operation raised something else first before it could observe cancellation), the code returns `Err(body_error)` via scope drop — parent's `abort` fallback still never fires. This matches design intent (cancellation is primary "under the existing task boundary rules"), but under those rules the task is expected to be aborted; without the yield-after-abort fix from finding #1, the same "abort queued but pre-empted by synchronous Ready" pattern is possible from the caller side. Not a Wave 1 defect on its own — flagged for follow-up if #1 is fixed and reviewers want symmetric handling.

---

Findings #1 and #2 are the substrate correctness defects that will block Wave 2's activation of aiosqlite cancellation fixtures and any real compiled evidence. Finding #3 corrupts the evidence stream for a specific corner. Findings #4–#6 are documentation, consistency, or coverage improvements. The declaration contract, module split, HIR/runtime/terminal/request/replay/registration/reservation-ordering pieces are all in good shape.

VERDICT: CHANGES_REQUIRED
