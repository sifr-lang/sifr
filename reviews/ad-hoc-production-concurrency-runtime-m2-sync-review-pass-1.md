Now I have full coverage. Here is the review.

---

# Concurrency Runtime M2 Sync — Working-Tree Review

## 1. Result: **PASS**

## 2. Findings (none blocking)

### Channel runtime hardening — correct
- `crates/sifr_codegen/src/lib_runtime_needs.rs:202-211` and `:251-260` (sender/receiver loops) use the canonical tokio "register notified first, check state under lock, then await" pattern. Because `Notified` snapshots the notify counter at construction and consumes any stored permit on first poll, no wake-up can be lost between `try_push_ref`/`try_pop` and `.await`.
- `try_push_ref` / `push` / `try_pop` issue `notify_one()` inside `with_state` (lib_runtime_needs.rs:148, 160, 167). Holding the sync `Mutex` across `notify_one()` is safe — Notify's internal lock is independent and there is no await — so capacity/availability wakeups are paired with state mutations atomically.
- `close()`, `release_sender()`, `release_receiver()` (lib_runtime_needs.rs:100-137) set `closed=true` (or sender_count→0 / receiver_alive=false) before calling `notify_waiters()`. Any racing endpoint re-enters the loop, calls `try_push_ref`/`try_pop`, observes Closed, and returns `ClosedError`. `notify_waiters()`'s counter increment is honored by `Notified` futures created before the close (they complete on poll), and futures created after see Closed via state. No stuck wait paths.
- FIFO ordering preserved: tokio `Notify` waiter queue is FIFO and notify_one wakes the oldest waiter; the buffer push/pop ordering is preserved by the state mutex.
- Cancellation no-loss preserved: dropping a `Notified` future is a deregister-only operation; the buffer state is untouched. `channel_cancel_receive_no_loss.sifr` exercises this.

### No raw Tokio types leak publicly
- Sifr-level `Channel` (lib/sifr/sync.sifr:21-57) exposes only `_buffer`/`_closed`/`_capacity`; the generated Rust adds private `_send_notify`/`_recv_notify` Arcs (lib_runtime_needs.rs:57-58) that have no corresponding Sifr field, so user code cannot reach them. `Notify` is exposed only as the Sifr-level `sync.Notify` wrapper (sync.sifr:194-) with `notified()/notify_one()/notify_all()` — no Tokio type surfaces in any signature.
- `is_share_safe_sync_wrapper` (task_scope_calls.rs:417-422) correctly includes `Notify`, `ChannelSender`, `ChannelReceiver` for cross-task sharing.

### SemaphorePermit guard policy — precise and isolated
- `sync_guard_type_label_by_name` (task_scope_calls.rs:408-415) classifies SemaphorePermit alongside LockGuard/RwLockReadGuard/RwLockWriteGuard with distinct labels.
- `await` diagnostic (async_await.rs:23-25) and `return` diagnostic (return_lowering.rs:69-72) gate on `!ctx.allow_intrinsic_imports`, so `lib/sifr/sync.sifr`'s `Semaphore.acquire`/`try_acquire` returning `SemaphorePermit` remain valid intrinsic code.
- Live cargo runs confirm exact codes/messages:
  - SIFR-OWN-0009 `semaphore permit `permit` cannot cross await; release the guard before awaiting`
  - SIFR-OWN-0003 `cannot return semaphore permit: synchronization guards cannot escape their local critical section`
- Lock/RwLock diagnostics remain intact: same SIFR-OWN-0009/0003 codes, only the trailing wording changed from "lock guards cannot escape" to "synchronization guards cannot escape". `test_lock_guard_return_escape_rejected` asserts `.contains("cannot return lock guard")` which still holds.

### M2 surface disposition recorded consistently
- `internal_docs/async_concurrency_model.md:590` defers `AsyncMutex`/`AsyncRwLock`; line 612 records permit guard rules and Notify edge-trigger model; line 612 also defers public `Barrier` and keeps `Once` internal-only.
- `verification/stdlib/concurrency_runtime_substrate_inventory.md:58` and `.json` line 163 carry the matching policy outcome.
- `verification/stdlib/concurrency_runtime_m2_sync_traceability.md` records: Notify edge-trigger acceptance, deferred async lock/Barrier/public Once, rejected Python queue accounting, and "no raw Tokio sync types" in Open Follow-up Boundaries.

### Sendability/shareability coverage retained
- `non_send_reason_inner` (task_scope_calls.rs:347-350) now reports semaphore permits via the new label, in addition to lock guards.
- Channel non-send element diagnostics (`channel_non_send_element_rejected`, `channel_send_wrong_type_rejected`, `ownership_and_async::test_channel_send_rejects_non_send_element`) and `shared_mut_without_lock_rejected` are all covered in the M2 traceability table.

### Ledger / manifest accuracy
- `issues/...execution.md` lines 252, 295-333, 491 add the M2 ledger, traceability entry, and review-pending note.
- `verification/platform/supported_host_matrix.md:13-15` moves the relevant lanes from `blocked-on-concurrency-runtime-m1/m2` to `supported`, with M3/M4/M6 lanes still blocked — accurate per scope.
- `verification/validation_lanes/merge_e2e_manifest.json` adds `channel_backpressure`, `channel_cancel_receive_no_loss`, `lock_basic`, `semaphore_basic`, `notify_basic` to the merge lane.
- Fail fixtures `semaphore_permit_across_await_rejected.sifr` and `semaphore_permit_escape_rejected.sifr` are present and exercised by the 399-fixture fail harness.

## 3. Required Remediation
None.

## 4. Disposition
**M2 is ready to PR after this review is recorded.** Channel runtime correctly replaces yield-loop polling with `tokio::sync::Notify` wakeups with no public Tokio leakage; close/drop/FIFO/backpressure/cancellation semantics are preserved; `SemaphorePermit` is enforced as guard-like with precise diagnostics that don't regress existing lock guard checks; deferred surfaces (`AsyncMutex`/`AsyncRwLock`/public `Barrier`/public `Once`/Python queue accounting) are recorded; supported host matrix, inventory, async model doc, traceability, and merge manifest are aligned.
