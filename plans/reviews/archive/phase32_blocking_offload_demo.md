

## Review: milestone_async_6 blocking offload demo slice

### Scope

**Files reviewed (uncommitted only):**
- `demos/m32_blocking_offload_demo.sifr`
- `internal_docs/phases/32_async_ecosystem.md` (doc update only)

### Demo Coverage Assessment

The demo correctly exercises all milestone_async_6 surfaces:

| Surface | Demo Location | Coverage |
|---|---|---|
| `@cpu_bound` annotation | line 8 | `compute_score()` |
| `@io_bound` annotation | line 17 | `read_legacy_counter()` |
| `task.spawn_blocking` | line 45 | `score_handle = task.spawn_blocking(compute_score)` |
| `BlockingTask` observation | lines 46-47 | `join()` + assertion |
| `ThreadPoolExecutor` | lines 49-52 | `submit(double_counter)` |
| `BlockingTask` cancellation | lines 54-56 | `cancel()` + `cancel_and_join()` |
| `sifr.threading.Thread` | lines 58-62 | lifecycle |
| `sifr.threading.Event` | lines 64-68 | set/clear/wait |
| `sifr.threading.Lock` | lines 70-71 | acquire/release/get |
| `sifr.threading.Condition` | lines 73-76 | acquire/notify/wait |

### No Blocking Issues

- **Panics**: None found. The `Event.wait()` is an async call, and all `assert` expressions are compile-time determinable.
- **Docs**: The phase doc update at line 780 correctly records this slice.
- **Semantics**: All uses match the model contract in `async_concurrency_model.md`.

### Low-Severity Observation

**Lines 47, 52**: `assert str(score_result) == "Ok(21)"` uses string comparison for `TaskResult` output. This is acceptable for demo validation but is slightly more brittle than value comparisons (e.g., `assert score_result == TaskResult.Ok(21)`). Other demos in this phase use the same pattern (`m32_sync_channel_demo.sifr` lines 19-20), so this is consistent with existing conventions. Not a blocker.

### Validation Confirmation

- Demo executes: `cache_hit=true` ✓
- Phase doc incremental update is correctly scoped ✓
- No user-visible panic paths ✓

---

REVIEW_STATUS: SATISFIED
