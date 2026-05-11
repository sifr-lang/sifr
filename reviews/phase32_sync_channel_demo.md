

The demo is solid. Let me verify the phase doc mentions this file correctly.

The demo is comprehensive and correct. Here's my review:

---

**Demo correctness**

- `shared_and_guard_demo()` (lines 24-41) is a sync function — guards are acquired and released before any `await`. Correctly demonstrates the lock-guard-across-await rule.
- `drain_two` (lines 15-21) receives two values with an intermediate `await task.sleep(0.0)`, but the receiver is consumed before the await, so no guard crosses the await. Correct.
- Return value `2` in `drain_two` matches the assertion `str(drained) == "Ok(2)"` since the return type is `Result[int, ClosedError]` (lines 69-76 backpressure section). Correct.

**Coverage against milestone_async_5 scope**

| Surface | Demo location | State |
|---|---|---|
| `Shared[T]` | line 26 | ✓ |
| `Lock[T]` + guard | lines 30-32 | ✓ |
| `RwLock[T]` + read/write guards | lines 34-40 | ✓ |
| `Semaphore` + `acquire()` | lines 48-49 | ✓ |
| `Notify` + `notified()`/`notify_one`/`notify_all` | lines 51-54 | ✓ |
| `channel()` factory + sender/receiver | lines 57-63 | ✓ |
| Sender clone sharing | line 59 | ✓ |
| Bounded backpressure | lines 65-75 | ✓ |
| Close + drain remaining | lines 78-86 | ✓ |
| Cancellation no-loss | lines 89-105 | ✓ |

**No unsupported semantics exhibited**

Every deferred surface (scoped borrow spawn, `AsyncLock`, `Barrier`, `Condvar`, `ProcessPoolExecutor`, subprocess, etc.) is absent. The demo uses only built-in runtime-backed primitives.

**Phase doc**

`milestone_async_5` status is `in_progress`. The current slice description reads: *"add `demos/m32_sync_channel_demo.sifr` to showcase..."* The doc is accurate and up to date.

**Validation gaps**

None. All surfaces have dedicated pass fixtures in the e2e suite:
- `channel_backpressure.sifr` — bounded backpressure
- `channel_cancel_receive_no_loss.sifr` — cancellation no-loss
- `channel_sender_close_clone_closes_all.sifr` — clone + close
- `channel_cancel_pending_receive.sifr` — pending receive cancellation
- `lock_basic.sifr`, `rwlock_readers.sifr`, `shared_basic.sifr`, `semaphore_basic.sifr`, `notify_basic.sifr`, `lock_guard_across_await_rejected.sifr` (negative)

---

REVIEW_STATUS: SATISFIED
