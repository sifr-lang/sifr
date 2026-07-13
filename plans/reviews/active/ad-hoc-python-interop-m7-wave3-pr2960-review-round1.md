I have enough evidence to finalize the review. The PR head, base, and full diff match the reviewed tree at 0a45cc74; validation ran green; and every design/implementation invariant is verifiable in the code.

# Review — PR #2960 (M7 Wave 3 cancellation carrier)

## Blockers
None.

## Wave-boundary verification (invariants that make the wave safe to land)

- **Hard-gated claim.** `crates/sifr_codegen/src/preamble/task_cancellation_runtime.rs:9` emits `const __SIFR_COOPERATIVE_SUPERVISORS_READY: bool = false` and `task_cancellation_runtime.rs:52-61` short-circuits `__sifr_claim_current_task_cancellation` when the const is false — no generated user code can produce a claimed carrier in wave 3.
- **Supervisor debug trip-wire.** `task_cancellation_runtime.rs:35-44` `debug_assert!`s "!is_claimed()" inside `abort()`; release falls back to `request_cancel` best-effort. Wave-ordering slips will trip in debug.
- **`__SifrBlockingTask` carrier-free.** `preamble/task_runtime.rs:26-40, 255-303` — no `cancellation` field; `cancel`/`cancel_and_join` still use `handle.abort()`.
- **Emission ordering.** Both `lib_modules_and_codegen.rs:543-546` and `entrypoints.rs:71-75` emit `build_task_cancellation_items` before `build_task_scope_items` and `build_task_supervisor_items` — struct is defined before use.

## Carrier linearization (`crates/sifr_runtime/src/cancellation.rs`)

- `claim` (`cancellation.rs:76-88`): under one mutex, reads `requested` → reads `exact` → installs hook. Yields exactly one of `Claimed`, `CancelledBeforeClaim`, or `AlreadyClaimed`.
- `request_cancel` (`cancellation.rs:90-111`): under one mutex, latches `requested=true` and snapshots exact-or-fallback hook Arc; hook runs *after* dropping the lock. Idempotent via `AlreadyRequested`.
- `bind_fallback` (`cancellation.rs:55-74`): late-bind with a pending request replays as `InvokedPendingCancellation`.
- 128-round contended barrier test (`cancellation.rs:216-262`) asserts exactly one of `(Claimed, Claimed)` or `(CancelledBeforeClaim, Fallback)` and exactly one hook fire — the strongest form of the atomicity claim.

## Asyncio task lifecycle (`crates/sifr_runtime/src/python/async_runtime.rs`)

- **Claim/publish/request atomicity.** `SubmissionCancellationBridge` (`async_runtime.rs:77-101`): `publish` snapshots `requested` under the bridge mutex after setting `submission_id`; `request` latches `requested` and reads `submission_id` under the same mutex, then calls `cancel_submission` *outside* the lock. Every race resolves to a task-cancel that either fires inline (setup sees `requested=true` on publish) or fires via `cancel_submission(id)` on the loop thread.
- **Setup containment.** `build_setup_callback` (`async_runtime.rs:277-339`) wraps the entire body in `catch_unwind(AssertUnwindSafe(...))`, cancels the created task on failure, unwinds pending vs. registered bookkeeping, and completes the terminal with the error. Every branch is closed.
- **Done containment.** `build_done_callback` (`async_runtime.rs:237-274`) `catch_unwind`s `task.result()`, always finishes the submission, and always resolves the terminal (panic → `AsyncRuntimeFailed`). Registration cannot be leaked.
- **Py-ref drop discipline.** `finish_submission` (`async_runtime.rs:509-518`) removes the `RegisteredSubmission` under the lock, releases the lock, then `drop(removed)` — Py<PyAny> refs never drop under `ASYNC_STATE`. `cancel_submission` and `cancel_registered_submissions` clone `(loop_object, exact_task)` under the lock, release, then `queue_exact_task_cancel` inside `Python::try_attach`.
- **GIL-detached sync waiter.** `run_coroutine_blocking` (`async_runtime.rs:163-172`) uses `super::detach(py, || terminal.wait())` — no `blocking_recv` under GIL, no Rust-blocking inside a Tokio runtime.

## Terminal latch (`crates/sifr_runtime/src/python/async_terminal.rs`)

- **Single-completion.** `complete` (`async_terminal.rs:42-56`) returns `false` if `outcome.is_some()`; wake fires outside the mutex.
- **Waker re-poll rule.** `poll` (`async_terminal.rs:83-96`) overwrites the stored waker only when `!waker.will_wake(context.waker())`, matching the standard idiom.

## Deterministic shutdown (`async_runtime.rs:346-395`)

Transitions Running/Failed → Stopping (`Starting` correctly rejects), waits `pending_submissions == 0`, `cancel_registered_submissions()` queues per-task `cancel` via `call_soon_threadsafe`, waits `submissions.is_empty()`, then queues `loop.stop`, joins the thread, normalizes to Stopped. `shutdown_terminally_drains_claimed_task_and_finally` (test at `async_runtime_tests.rs:210-250`) exercises this path with a claimed carrier and observes the finally-marker before the terminal resolves.

## Generated task/supervisor/JoinSet

- `__SifrTask<T,E>` now carries `cancellation: __SifrCancellationCarrier` (`preamble/task_runtime.rs:8-25`). `cancel` (`task_runtime.rs:213-222`) routes through `cancellation.request_cancel()`. `cancel_and_join` (`task_runtime.rs:223-233`) awaits receiver then returns `Cancelled` — matches design §F.
- `__sifr_timeout` (`task_runtime.rs:234-252`) contains the correct Wave-4-ready Claimed branch (`matches!(request, CancellationRequest::Claimed)` → honor Ok/Err terminal, else Timeout). Unclaimed branch always returns Timeout — dark in wave 3 per const.
- Spawn seams (`task_runtime.rs:437, 562, 575`, plus `preamble/task_scope_offload_runtime.rs:168-171`) create a fresh `CancellationCarrier`, clone for the child task-local scope (`__SIFR_TASK_CANCELLATION.scope(child_cancellation, ...)`), then wrap parent-side with `__SifrCancellationCarrier::new(inner, child.abort_handle())`. Every spawn site uses the same shape.
- Supervisors (`task_supervisor_runtime.rs`) still call `cancellation.abort()` — pre-carrier semantics preserved. Because the const is false, all of these route through `fallback_abort.abort()`.
- JoinSet (`preamble/join_set_runtime.rs:250`) extracts `cancellation.abort_handle()` into `abort_handle: Some(...)`. `__sifr_cancel_all` (`join_set_runtime.rs:319`) still prefers the extracted abort handle. Matches design §G.

## Emission/certification integration

- `internal_docs/stdlib_retained_compiler_intrinsics.toml:122-134` adds `task_cancellation_runtime.rs` and `task_supervisor_runtime.rs` to `_sifr.task::language_runtime_glue` `preamble_files`, and `sifr_runtime::cancellation` to `direct_runtime_roots`. Both `preamble_files` and `direct_runtime_roots` are `unique_owner_keys` — no duplicate ownership.
- `sifr_runtime/src/lib.rs:4` exports `pub mod cancellation`.
- `sifr_runtime/src/python.rs` (`python.rs:10-11`) declares `async_runtime` + `async_terminal`; new `PythonRuntimeError` variants `AsyncSubmissionCancelled` / `AsyncCancellationAlreadyClaimed` (`python.rs:153-154, 201-206`) surface at the boundary.

## Tests

- Cancellation carrier: 5 (`cancellation.rs:147-262`) — unclaimed fallback, late-bind, claimed-request, cancel-before-claim, 128-round contended.
- Terminal latch: 2 (`async_terminal.rs:118-174`) — blocking wait single-outcome, future re-poll waker replacement.
- Owned-runtime Python-facing: 9 (`async_runtime_tests.rs`) — loop setup failure, cancel-before-claim, finally-before-terminal, suppression-wins, independent-tasks, shutdown drain, invalid awaitable, queue failure, terminal panic containment.
- Codegen snapshots: 15 (`async_task_runtime_codegen_tests.rs`) — including two new ones pinning the carrier field, `__SIFR_COOPERATIVE_SUPERVISORS_READY`, `__SIFR_TASK_CANCELLATION.scope(...)`, the `debug_assert!` string, `cancellation.request_cancel()` path, the `__sifr_timeout` Claimed matches, and the JoinSet `cancellation.abort_handle()` extraction.

## No user-triggerable panic

- `debug_assert!` in `__SifrCancellationCarrier::abort` is a trip-wire (release promotes to `request_cancel`).
- `catch_unwind(AssertUnwindSafe(...))` contains setup and done callbacks; panic maps to `AsyncRuntimeFailed`.
- Mutex poisoning is normalized: `PythonTerminal` uses `PoisonError::into_inner`; carrier and async state return `StateUnavailable` variants.
- No data-dependent `.unwrap()`/`.expect()` in runtime paths.

## PR description accuracy

PR body enumerates: carrier, exact-task lifecycle, terminal latch, `__SifrTask` cancel replacement, gated supervisors, JoinSet/timeout wiring, responsibility split, and the specific test set. Every claim maps to code in the diff. Additions/deletions (1677/-214), head SHA, base main, and mergeable-clean state confirmed via `gh api`.

## Non-blocking observations (already recorded in prior review rounds; no need to fix in this wave)

1. `__SifrCancellationCarrier::abort` (`task_cancellation_runtime.rs:35-44`) double-locks in debug — `is_claimed()` twice. Can be collapsed to a single call bound to a local.
2. `CancellationCarrier::fallback_hook` (`cancellation.rs:113-118`) is currently unused — reserved for wave 4; either annotate or drop.
3. `run_coroutine_blocking` (`async_runtime.rs:163-172`) has no `debug_assert!(Handle::try_current().is_err())`; relies on `blocking_io`/`explicit_offload` classification.
4. File split is coarser than design §H's five-file sketch — everything is well under the 900-line cap; deferrable.
5. Each `RegisteredSubmission` stores its own `loop_object` `Py<PyAny>` clone — memory-only; a single state-level `loop_object` would do.

VERDICT: SATISFIED
