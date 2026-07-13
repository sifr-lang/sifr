All 118 python-feature tests pass including all Wave 4 additions. The prior local review rounds' conclusions independently reproduce on the actual PR head.

## Independent PR #2962 review — findings

**PR-head verification.** PR head OID `d68fb8369c65c07ec62ccb06727017173ad72277` matches local HEAD. `gh pr diff 2962` is byte-identical to `git diff main...HEAD` (1564 lines).

**Correctness / races / ordering.**
- `CancellationCarrier::claim` bumps `next_generation` under the same mutex that gates `requested`/`exact`, and `.wrapping_add(1).max(1)` reserves generation 0 as a sentinel. Lease `Drop` clears `exact` only when its generation matches — stale drops after a lease-cycle are silent no-ops. `PoisonError::into_inner` in Drop keeps the exact slot best-effort recoverable (`cancellation.rs:130-145`).
- `PythonTerminal::complete` (`async_terminal.rs:55-70`) releases the state mutex, then **drops the lease before** `changed.notify_all()`/`waker.wake()` — the invariant the sequential-await path relies on. Verified by `terminal_completion_releases_claim_before_waking_waiter`.
- `SubmissionCancellationBridge` publish/request both take the same mutex before mutating; sees requested-first-or-publish-first collapses to exactly one of "setup callback calls `task.cancel()`" or "request calls `cancel_submission`". No lost-cancel.
- `pending_submissions: BTreeMap<u64, PythonTerminal>` and `submissions: BTreeMap<u64, RegisteredSubmission>` are both keyed by exact id and mutated only under `ASYNC_STATE`; `register_submission` performs an atomic remove-then-insert. Every setup-callback failure branch selects the correct unwind by id and completes the terminal with `PythonTerminalError::Runtime`.
- Loop-failure detection in shutdown waits (`async_runtime.rs:366` and `614-628`) re-checks `AsyncLifecycle::Failed` on every wake — closes the design-round-1 nit.
- `drain_outstanding_submissions` and `fail_live_runtime` both `mem::take` under lock, then complete terminals outside the lock, and drop `active` inside `Python::try_attach` so `loop_object`/`exact_task` DECREFs run with GIL (`async_runtime.rs:632-686`). Pending terminals hold no Py handles, so releasing them without GIL is safe.
- Shutdown order (`admissions → callback → async-cleanup → cancel+drain → loop-stop → loop-join → epilogue`) with `retain_first_error`; every phase records into `SHUTDOWN_PHASE_TRACE` and continues even on error. Verified by `shutdown_errors_do_not_skip_cancel_drain_stop_or_join` (all six phases run in order after callback-shutdown fails).

**Supervisor/JoinSet arbitration.** gather/race/select and scope fail-fast all switched to `cancellation.request_cancel()`. Process observers keep `stop_on_fail_fast` (unchanged, correct for blocking). `__SifrJoinEntry { cancellation, blocking_abort }` is a clean disjoint union — `add(__SifrTask)` stores the full carrier with `blocking_abort: None`; blocking/CPU spawns store the abort handle with `cancellation: None`; `__sifr_cancel_all` picks the right branch. No `cancellation.abort_handle()` extraction remains in async entries (pinned by `test_join_set_preserves_task_cancellation_carrier_until_terminal_drain`). `__sifr_join_all` fail-fast still awaits every observer.

**PyO3 ownership.** Every `Py<PyAny>` drop happens with the GIL (either inside `attach`/`try_attach` or via PyO3 deferred DECREF when `try_attach` returns None — safe).

**Generated-code validity.** Codegen strings compile through the e2e path (`create-pr` gate: 130/130 e2e passed). Format-heavy string bodies use consistent identifiers; the observer wrapper in `__sifr_add_task` destructures `_error` correctly.

**Tests.** Ran `cargo test -p sifr_runtime --lib --features python` locally: 118/118 pass — includes all 15 async-runtime tests and the 3 async-terminal tests, plus 7 cancellation carrier tests. The 128-round contended claim-vs-request test enforces the exclusion invariant. Warning `"Task was destroyed but it is pending!"` in `shutdown_cancels_and_joins_an_in_flight_raw_coroutine` is pre-existing (raw path, not modified in Wave 4).

**Scope.** File sizes all under 900 (`task_runtime.rs` 810, `async_runtime.rs` 795, `python.rs` 872). No frontend, typed-wrapper, or gate-lifting changes — matches the Wave 4 phase deferrals.

**Non-actionable minor observations** (not blocking):
- `reserve_submission` uses `saturating_add` for `next_submission_id`; theoretical ID collision after u64::MAX submissions. Not practical.
- Generation `.wrapping_add(1).max(1)` collision after 2^64 claim cycles. Not practical.
- `CancelOutcome::AlreadyStarted` is now only reachable for blocking entries (async cancellation is cooperative). Design review already flagged as documentation-level; enum is undocumented today.
- `fail_live_runtime` passes `&message` (a `&&str`) to `complete_drained_submissions`; auto-derefs, stylistic only.

Round-1 and round-2 verdicts hold on the published PR. No actionable defects found on this independent pass.

VERDICT: SATISFIED
