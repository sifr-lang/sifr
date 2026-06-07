I reviewed the working tree against the criteria in your prompt. The technical M2 work is sound, but the working tree contains untracked artifacts that will contaminate the M2 PR.

## Result: **NOT PASS — minor housekeeping remediations required**

The M2 implementation itself is correct on every criterion you named; what blocks PASS is non-code state in the tree.

### Technical M2 closure — all correct
- **Notify wakeups, no-lost-wake**: `lib_runtime_needs.rs:202-213` (sender) and `:253-265` (receiver) create `notified()`, pin, call `notified.as_mut().enable()` before `try_push_ref`/`try_pop`, then `notified.await` on `Full`/`Empty`. Standard Tokio multi-waiter pattern; a `notify_one()` after `enable()` is honored even before the first poll of `.await`.
- **State/wakeup atomicity**: `notify_one()` fires inside the `with_state` lock after the buffer mutation (`:148`, `:159`, `:167`). Holding `std::sync::Mutex` over `Notify`'s independent internal lock is fine — no await involved, no deadlock.
- **close/drop**: `close`/`release_sender`/`release_receiver` mutate state first, then call `notify_waiters()`. Waiters re-enter the loop and observe `Closed` via `try_push`/`try_pop`. `release_sender` only wakes `_recv_notify` — correct, since other senders can't make progress on sender-drop unless it was the last sender (which also flips `closed`).
- **FIFO/backpressure/cancellation**: `VecDeque` push_back/pop_front; Notify queue is FIFO; capacity gate in `try_push_ref`; dropping a `Notified` is deregister-only, so cancellation does not lose messages.
- **No raw Tokio leakage**: `_send_notify`/`_recv_notify` are private fields with no Sifr-level counterpart. Public Notify is only via the Sifr `sync.Notify` wrapper.
- **SemaphorePermit guard policy**: `sync_guard_type_label_by_name` in `task_scope_calls.rs:408-415` adds the "semaphore permit" classification alongside the three lock guards. Both await diagnostic (`async_await.rs:23-25`) and return diagnostic (`return_lowering.rs:69-72`) gate on `!ctx.allow_intrinsic_imports`, so `lib/sifr/sync.sifr`'s `Semaphore.acquire` continues to compile internally.
- **Lock regression risk**: prefix `lock guard `name` cannot cross await…` and prefix `cannot return lock guard…` both preserved — existing `test_lock_guard_across_await_rejected` and `test_lock_guard_return_escape_rejected` `.contains(...)` assertions still hold under the new label-substituted messages.
- **Docs/traceability/inventory/host matrix/merge manifest/ledger**: traceability file added; host matrix flips concurrency rows m1/m2→supported with correct scope notes; inventory `.md`/`.json` rewritten to record permit-cannot-cross-await + permit-cannot-escape; async-concurrency-model doc updated; merge manifest adds `channel_backpressure`, `channel_cancel_receive_no_loss`, `lock_basic`, `semaphore_basic`, `notify_basic`; M2 ledger ticked.
- **Validation**: `create-pr.latest.json` shows all 14 lane steps pass (`platform_golden`, `e2e_pass_suite`, `crate_tests`, etc.); only advisory is warm wall-time budget — non-blocking.

### Required remediations before opening the M2 PR

1. **Empty M2 review artifacts** (`reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-2.md`, `pass-3.md`): both are 0-byte untracked files. The ledger references only pass-1. Either delete them or populate them; do not let them land in the M2 PR as empty files.

2. **Unrelated network-http scope mixed into the working tree** (these are NOT M2 sync changes and should not ride in this PR):
   - `issues/ad-hoc-production-network-http-platform-substrate.md` (145 line diff: byte-buffer/DNS/TLS-stream/IDNA/HTTP-substrate contracts, hyper-util conditionality, M0 lockfile pin policy, …)
   - `issues/ad-hoc-production-network-http-platform-substrate-execution.md` (network reviewer remediations + readiness-review entries)
   - Untracked: `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md`, `…-pass-2.md`
   Stage and PR these on a separate branch so the M2 sync PR diff stays focused. The merge-manifest M2 additions and supported-host matrix concurrency rows are the correct M2 scope; the network/http changes are not.

3. **Minor (non-blocker, optional)**: `sync_guard_type_label_by_name` calls `public_type_name(name)` twice; harmless but worth a single-bind cleanup if you touch the function again.

Once items 1 and 2 are handled (delete-or-fill the empty review files; split network-http changes onto their own branch), M2 sync is ready to PR — the technical closure is complete and validation evidence supports it.
