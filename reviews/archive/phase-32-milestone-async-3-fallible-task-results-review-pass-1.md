

Reviewing the fallible task-result plumbing slice...

**SATISFIED**

---

## Key Changes

| File | Change |
|------|--------|
| `preamble.rs` | `__SifrTask` receiver: `Receiver<T>` → `Receiver<__SifrTaskResult<T, E>>` |
| `preamble.rs` | Added `__sifr_spawn_result` for fallible coroutines; `__sifr_spawn_infallible` wraps raw success as `TaskResult::Ok` |
| `preamble.rs` | Updated `join()`, `__sifr_timeout`, `__sifr_task_gather`, `__sifr_task_race`, `__sifr_task_select` to handle new `TaskResult` receiver shape |
| `task_scope_calls.rs` | Removed infallible-only guard; dispatches to `__sifr_spawn_infallible` or `__sifr_spawn_result` based on coroutine error type |
| `lower_expr.rs` | Recognizes both private spawn helpers in codegen |
| `pr_e2e_manifest.json` + `32_async_ecosystem.md` | Fixture and progress tracking |

---

## Cancellation & Timeout Behavior

**Cancellation** is preserved across all updated helpers:
- `__sifr_task_race`: aborts losers before awaiting their receivers → correct
- `__sifr_task_select`: aborts loser before awaiting → correct
- `__sifr_timeout`: abort then await for cleanup (`let _ = receiver.await`) → correct

**Timeout** correctly distinguishes:
- `TaskResult::Ok(value)` → child succeeded before deadline → returns `TaskResult::Ok(value)`
- `TaskResult::Err(err)` → child failed before deadline → wraps as `TimeoutResult::Inner(err)`
- `TaskResult::Cancelled` or `Err(_)` → child was cancelled/aborter → returns `Cancelled`

The `Err(_)` case in the timeout select arm correctly maps both "receiver closed without send" (sender dropped) and "task was aborted" to `Cancelled`. When `abort()` is called, the oneshot becomes closed, so the await returns `Err(RecvError)`.

---

## Fallible Spawn Soundness

`__sifr_spawn_result` maintains the conservative no-capture boundary:

```rust
F: std::future::Future<Output = Result<T, E>> + Send + 'static
```

The HIR layer preserves the no-argument check (`if !matches!(&coroutine, HirExpr::Call { args, .. } if args.is_empty())`), so fallible spawn is sound within the existing conservative boundary.

---

## Non-Blocking Notes

**Note 1 — E2E fixture has no result assertion:**
`task_spawn_fallible_result.sifr` spawns a fallible coroutine and awaits the handle, but never matches/asserts on `TaskResult.Err`. The test passes by running without panicking, but a follow-up slice should add the match block to validate `ValueError` propagates as `TaskResult.Err`.

**Note 2 — `__sifr_task_gather` returns type:**
The signature returns `TaskResult<Vec<T>, E>` with `E` as the child error type. After this slice, child tasks return `TaskResult[T, E]` from `join()`. This is correct for the homogeneous gather case (all children share error type `E`).
