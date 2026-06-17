# Concurrency Runtime synchronization capability Sync Traceability

Capability: `concurrency-runtime synchronization primitives`

## Production Surface Traceability

| Surface | Evidence | Notes |
| --- | --- | --- |
| `sync.channel[T]()` / `sync.bounded_channel[T](capacity)` | `channel_factory_basic`; `bounded_channel_factory_basic`; `channel_backpressure`; `channel_cancel_pending_receive`; `channel_cancel_receive_no_loss` | Sender/receiver endpoint factories, bounded backpressure, cancellation, and no-loss receive behavior. Generated runtime uses explicit `tokio::sync::Notify` wakeups rather than yield-loop polling for full/empty states, and wait loops enable `Notified` before state checks to preserve the no-lost-wake pattern for multi-waiter backpressure. |
| `ChannelSender[T]` / `ChannelReceiver[T]` close/drop/FIFO | `channel_close`; `channel_fifo_order`; `channel_drop_last_sender_closes_after_drain`; `channel_drop_receiver_closes_senders`; `channel_sender_close_clone_closes_all`; `async_for_channel`; `async_effect_summary_channel_receive` | Explicit close, sender clone close, receiver drop, last-sender drain, FIFO, direct receive terminal `ClosedError`, and async-iteration terminal `None` behavior. |
| Channel element sendability | `channel_non_send_element_rejected`; `channel_send_wrong_type_rejected`; `ownership_and_async::test_channel_send_rejects_non_send_element` | Non-send values cannot cross channel boundaries; wrong element types are rejected before codegen. |
| `sync.Lock[T]` / `sync.RwLock[T]` | `lock_basic`; `rwlock_readers`; `lock_guard_across_await_rejected`; `lock_across_task_boundary_rejected`; `lock_guard_escape_rejected`; `ownership_and_async::test_lock_guard_across_await_rejected`; `ownership_and_async::test_lock_guard_return_escape_rejected` | Sync lock guards cannot cross `await`, task boundaries, or return boundaries. Async lock surfaces remain deferred. |
| `sync.Shared[T]` share safety | `shared_mut_without_lock_rejected`; `ownership_and_async::test_shared_rejects_mutable_list_value` | Mutable unsynchronized values require explicit synchronization wrappers. |
| `sync.Semaphore` / `SemaphorePermit` | `semaphore_basic`; `semaphore_permit_across_await_rejected`; `semaphore_permit_escape_rejected`; `ownership_and_async::test_semaphore_permit_across_await_rejected`; `ownership_and_async::test_semaphore_permit_return_escape_rejected` | `SemaphorePermit` is guard-like and await-forbidden in synchronization capability; permits cannot escape through returns. |
| `sync.Notify` | `notify_basic` | `Notify` is the accepted edge-triggered event primitive; level-triggered `Event` behavior uses explicit state plus `Notify` in the first model. |
| `Barrier` / public `Once` | This traceability artifact; capability decision register | Deferred/internal-only for synchronization capability. Channels, locks, semaphores, and notify cover the near-term production coordination model. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative fixtures |
| --- | --- | --- |
| `Lib/test/test_queue.py` | `adapted-for-sifr-api` | `channel_factory_basic`, `bounded_channel_factory_basic`, `channel_fifo_order`, `channel_backpressure`, `channel_drop_last_sender_closes_after_drain` |
| `Lib/test/test_asyncio/test_queues.py` | `adapted-for-sifr-api` | `async_for_channel`, `channel_cancel_pending_receive`, `channel_cancel_receive_no_loss`, `channel_sender_close_clone_closes_all` |
| `Lib/test/test_asyncio/test_locks.py` | `adapted-for-sifr-api` | `lock_basic`, `rwlock_readers`, `lock_guard_across_await_rejected`, `semaphore_basic`, `notify_basic` |
| `queue.task_done()` / `queue.join()` accounting | `rejected` | Sifr channels use explicit ownership and close/backpressure evidence; Python queue accounting is not part of the production model. |
| `sifr.asyncio.Queue` and CPython-shaped queue modules | `unsupported-with-diagnostic` / `rejected` | legacy-subprocess rejection capability removes legacy public adapters; production APIs use `sifr.sync`. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `channel_backpressure`, `channel_cancel_pending_receive`, `channel_cancel_receive_no_loss`, `semaphore_basic`, `notify_basic`, lock/channel basic fixtures |
| Merge | `channel_backpressure`, `channel_cancel_receive_no_loss`, `lock_basic`, `semaphore_basic`, `notify_basic` |
| Fail suite | `channel_non_send_element_rejected`, `channel_send_wrong_type_rejected`, `lock_guard_across_await_rejected`, `lock_across_task_boundary_rejected`, `lock_guard_escape_rejected`, `semaphore_permit_across_await_rejected`, `semaphore_permit_escape_rejected`, `shared_mut_without_lock_rejected` |

## Open Capability Boundaries

synchronization capability does not add `sync.AsyncMutex[T]`, `sync.AsyncRwLock[T]`, public `Barrier`, public `Once`, Python-shaped queue accounting, or raw Tokio sync types. Those require a later capability amendment or capability with explicit await-safe guard and lifecycle semantics.
