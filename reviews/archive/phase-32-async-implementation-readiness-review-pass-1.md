

Verdict: Blockers found

**B1: Architecture doc references obsolete milestone names (blocking)**

`internal_docs/architecture.md:362` and `architecture.md:376` use the legacy milestone names `milestone_async_sync` and `milestone_async_core`. The model and phase files both use the current `milestone_async_N` naming scheme (e.g., `milestone_async_0`, `milestone_async_1`). The architecture doc's own §8 concurrency safety section says "Phase 32 planning follows `internal_docs/async_concurrency_model.md`" and lists responsibilities under `milestone_async_0` through `milestone_async_5` — but the borrow/cross-cutting section still references the old names. These must be reconciled before Phase 32 implementation begins, or the architecture doc will become a source of confusion about which milestone owns async capture rules.

**Fix:** Update `architecture.md:362` to reference `milestone_async_4` (ownership + Send/Sync boundary checking) and `architecture.md:376` to reference `milestone_async_1` or `milestone_async_2` (async function error handling). The error semantics table row for "Async function" currently says `(milestone_async_core)` — this label is orphaned from the old naming scheme.

---

**B2: Lock guard cross-await liveness check is underspecified (blocking)**

The model (§Lock Policy) and phase (`milestone_async_5` scope) both state "lock guards must not cross `await` points in v1" and "reject lock guards that remain live across an `await` point." The diagnostic message is even defined: "lock guard is still live at this await point; lock guards cannot cross await points in v1."

However, the mechanism for detecting this is completely absent. The phase defines `LockGuard`/`RwLockGuard` liveness as a v1 constraint but nowhere in the 9 milestones is there a work item to implement the **static analysis** for it. `milestone_async_5` has "add diagnostics for lock misuse where statically knowable" but this is bundled with channel and semaphore diagnostics — the lock-guard liveness check is not called out as a separate sub-item, and there are no fixture names targeting it specifically (unlike `spawn_non_send`, `borrow_across_await`, etc.).

**Fix:** Add a dedicated negative validation fixture `lock_guard_across_await_rejected.sifr` to `milestone_async_5` with a matching work item that explicitly names the static liveness check. Ensure the definition of done in `milestone_async_5` is updated to include "lock guard liveness at await points is rejected at compile time." This is critical because failing to detect live `LockGuard` across `await` is a soundness hole — the Rust mutex guard would be held across a `.await` yield point, and Tokio's cooperative scheduling would allow other tasks to run while holding the guard.

---

**B3: Calling async from sync is unaddressed (blocking)**

Both the model and phase correctly establish that `AsyncFunction` is not interchangeable with sync `Function`/`Callable`. The architecture doc reinforces this: "async functions cannot be stored, passed, or invoked through a sync callable path."

But neither document addresses the inverse problem: what happens when sync code tries to call an async function without awaiting? This is a common Python footgun — `http.get(url)` (async) called from a sync `def` produces a coroutine object that is never awaited. In Sifr, this would produce a `Task[T, E]` handle that is silently dropped if not awaited. The phase has `spawn_non_send_initial_diagnostic.sifr` and `spawn_non_send_rejected.sifr` but nothing targeting `async_fn()` call at sync scope without await.

For Sifr's no-silent-warnings guarantee, this scenario must be caught. `Task[T, E]` is `#[must_use]`, but the phase doesn't confirm that async call results are covered by must-use enforcement, and there's no fixture demonstrating that an un-awaited async call from sync context produces a diagnostic.

**Fix:** Add a negative validation fixture `async_call_without_await_from_sync_rejected.sifr` to `milestone_async_1`. Add a work item to `milestone_async_1` explicitly stating: "reject or warn when an async function call result (a `Task` handle) is produced in a sync context without being awaited, since the handle carries `#[must_use]` semantics."

---

**B4: Architecture doc has not received the Phase 32 contract copy (partial blocker)**

The architecture doc's §8 Concurrency Safety at `architecture.md:662-681` references `milestone_async_0` needing to "copy the complete async/concurrency type, task, cancellation, and runtime contracts from `internal_docs/async_concurrency_model.md` into the architecture contract." However, reading the architecture doc, the async/concurrency section is sparse — it has the high-level contract bullets but lacks:

- The full `Task[T, E]` / `CancellationError` / `TimeoutError` / `SecondaryError` type table (the error hierarchy table at architecture.md:523 is present but only shows the error types; the task-specific type semantics for `await` result types are not shown)
- The borrow rule matrix at async boundaries (model has it at lines 279-286; architecture does not)
- The `task.timeout` race behavior contract
- The `gather` fail-fast ordering behavior
- The `select`/`race` cancellation-of-losers default

The architecture doc says in §8: "Phase 32 planning follows `internal_docs/async_concurrency_model.md`; this section records the high-level contract that implementation milestones must preserve." But "high-level" is doing a lot of work here — the missing details above are not high-level, they are the semantic contract that developers implementing `milestone_async_3` will need to reference.

**Fix:** `milestone_async_0` should copy (or cross-reference with a stable anchor) the borrow rule matrix, the `gather` ordering contract, the `select`/`race` default-cancellation policy, and the `task.timeout` race semantics into the architecture doc's §8. The phase file's own `milestone_async_0` scope already lists "Copy the canonical type, task, cancellation, timeout, lock, and runtime contracts into `internal_docs/architecture.md`" — but this item is incomplete in the current architecture doc.

---

**B5: Deferral tracking for `sifr.asyncio.timeout` context-manager form is ambiguous (refinement)**

The model (compatibility mapping) and phase (compatibility mapping) both list `sifr.asyncio.timeout(duration)` mapping to `sifr.task.timeout(duration)` context-manager form. However:

1. Neither document defines what the `sifr.task.timeout(duration)` context-manager API looks like — only the function-call form `task.timeout(task, duration)`.
2. `milestone_async_2` implements `sifr.task.timeout` but describes it as a function: `task.timeout(task, duration)` — the race behavior is defined for function call, not context manager.
3. The context-manager form (`timeout(duration) as _:`) is a Python `asyncio.timeout`-derived API that wraps an async block. The phase's `milestone_async_0` defines "Define selection, channel, lock, annotation, and runtime-neutrality policies" but does not explicitly name the context-manager timeout form as a separate API to design.
4. `milestone_async_7` (async context managers) covers `async with` but does not link `task.timeout` as an async context-manager-like API that needs the same LIFO cleanup semantics.

If `sifr.asyncio.timeout` is committed to the compatibility mapping, the canonical `task.timeout(duration)` context-manager form must be specified. Is it `async with task.timeout(duration):`? Is it a function returning a context manager (`task.timeout(duration) as t:`)? The model says it maps to "context-manager form" but doesn't define the form.

**Fix:** In `milestone_async_0`, add a work item explicitly defining the `task.timeout(duration)` context-manager API surface and its semantics (inner wins on same-tick tie, outer cancellation cancels inner, cleanup awaited before scope exit). Confirm this is distinct from or the same as the function form `task.timeout(task, duration)`.

---

**B6: Exit gate inconsistency — `SecondaryError` treatment differs between model and phase**

The model (Phase Exit Gate, line 969) says:
> "Cancellation semantics are deterministic and typed."

The model also says `SecondaryError` "never masks the primary cancellation/failure result." The architecture doc's error table (architecture.md:525) shows `SecondaryError` with the description "Cleanup or sibling failure evidence attached to a primary cancellation/failure; never masks the primary result."

The phase exit gate at line 784 says:
> "Cancellation semantics are deterministic, typed, and not swallowed by broad `except Error`."

This is consistent. However, the phase's `milestone_async_7` DoD states:
> "`SecondaryError` never masks the primary result."

This is fine, but there is a subtle inconsistency in how `SecondaryError` propagates up through the `gather` fail-fast path. The model says later failures "are secondary evidence" and the phase says "later errors are secondary evidence." The question is: if `gather` has fail-fast behavior and a cleanup error occurs during the cancellation of the first-failing task, does that cleanup error produce a `SecondaryError` attached to the first error, or does it get dropped? The model says "panic-like failures from async exit must be caught at the runtime/codegen boundary and surfaced as secondary structured errors" — but the phase's `milestone_async_7` says cleanup errors "become `SecondaryError` evidence attached to the owning scope result."

These are consistent in intent, but the scope/handle that cleanup errors attach to is underspecified. When `gather` cancels a losing sibling, does cancellation cleanup on that sibling produce a `SecondaryError` that gets attached to the `gather` result? Or is it only `TaskScope.__aexit__` that propagates `SecondaryError`? This matters for implementers of `milestone_async_3` (`gather` error behavior) and `milestone_async_7` (cleanup error handling).

**Fix:** In `milestone_async_3`, add a work item explicitly defining: "When `gather` cancels unfinished children due to a first failure, any cleanup errors from those cancelled children surface as `SecondaryError` values attached to the primary `gather` result, following the same secondary-error attachment rules as scope-level cancellation." Add a positive fixture `task_gather_cleanup_error_secondary.sifr` to validate this behavior.

---

**Non-blocking refinements:**

**R1:** The phase file's `milestone_async_2` DoD includes `task_handle_unused_must_join_or_cancel.sifr` in negative validation, but the fixture name implies a compile-time check. If the check is only partially static (handle dropped at scope exit triggers runtime backstop), this should be clarified in the DoD — currently `milestone_async_3` defines the "scope exit cancels/awaits remaining children as safety backstop" but `milestone_async_2` already lists the fixture. The fixture belongs in `milestone_async_3` where the backstop is implemented, not `milestone_async_2`.

**R2:** The `milestone_async_0` validation planning says "review checklist rejects any plan that exposes raw event loops, detached spawn, process pools, subprocess/signal APIs, or raw Tokio types in public APIs." This checklist should be written down as an explicit artifact (even a comment block in the phase file) rather than living only in the description text, so that PR reviewers have a concrete list to check against.

**R3:** Both docs list `cancelled_task_except_error_does_not_swallow.sifr` as a negative fixture in `milestone_async_1` (phase) / `milestone_async_2` (model). The phase file says it's in `milestone_async_1`; the model says it's in `milestone_async_2`. Align on placement — `milestone_async_1` (async syntax + type system) is the right place since the check is about `CancellationError` not being an `Error` subclass, which is a type system property.

**R4:** The `milestone_async_4` positive fixtures in the phase file (`spawn_owned_send_ok.sifr`, `spawn_scoped_borrow_ok.sifr`, `await_immutable_borrow_ok.sifr`) don't match the model fixtures (`spawn_owned_send_value.sifr`, `spawn_scoped_borrow_ok.sifr`, `spawn_capture_immutable_shared_ok.sifr`, `await_without_live_borrow.sifr`). The model file is the authoritative source, so the phase file should be updated to match the model fixture names.

**R5:** The `rwlock_readers.sifr` fixture listed in the model's `milestone_async_5` positive validation is named differently from the phase's `rwlock_basic.sifr`. Again, the model wins — the phase should be updated.

---

**B7: `spawn_non_send_initial_diagnostic` placement in milestone_async_2 is premature**

The phase's `milestone_async_2` negative validation lists `spawn_non_send_initial_diagnostic.sifr`. But `milestone_async_4` ("Ownership, Borrowing, and Send/Sync Task Boundaries") is where the Send/Sync checking is implemented according to both the model and phase milestone ordering. `milestone_async_2` implements `scope.spawn` returning a typed task handle, but Send/Sync validation of captures is `milestone_async_4` work. Having `spawn_non_send_initial_diagnostic` in `milestone_async_2` implies a diagnostic fires before the full Send/Sync checking exists. This could be a "preliminary" diagnostic that gets replaced, but the phase doesn't document this intent.

**Fix:** Remove `spawn_non_send_initial_diagnostic.sifr` from `milestone_async_2` negative validation. The `milestone_async_2` scope says "translate obvious runtime/task-boundary failures into Sifr diagnostics" but Send/Sync checking is not an "obvious" failure — it's the subject of `milestone_async_4`. Keep `spawn_non_send_rejected.sifr` in `milestone_async_4` where the full check lives.

---

Recommendation: iterate again

The model and phase are broadly well-aligned and the semantic contract is sound. The blockers are concrete and fixable: B1 is a naming reconciliation, B2 is a missing static analysis work item, B3 is a missing scenario in the type system, B4 is a documentation gap in the architecture doc, B6 is underspecified secondary-error propagation through `gather`. B5 is an API form that needs explicit definition before `milestone_async_0` closes. B7 is fixture misplacement.

The phase is close to execution-ready — the nine-milestone structure, dependency graph, locked decisions, validation families, and quality contract are all solid. Resolving these 7 issues before code is written will prevent implementation drift.
