

## Code Review: Phase 32 `spawn_blocking` Slice

### Scope Covered
- HIR lowering for `task.spawn_blocking` in `task_calls.rs`
- Task handle method support in `task_handle_calls.rs`
- Codegen for `__SifrBlockingTask` in `preamble.rs`
- Method lowering in `lower_expr.rs`
- Await lowering in `stmt_support_emitter.rs`
- Runtime dependency wiring in `lib.rs`
- Test fixtures and unit tests
- Phase progress documentation

---

### Review Findings

#### 1. CORRECT: HIR Lowering (`task_calls.rs:50-133`)

The `lower_task_spawn_blocking_call` function implements the v1 contract correctly:
- Async context validation at line 55
- Exactly-one-argument check at line 66
- Zero-parameter function restriction at lines 74-80 with clear diagnostic message
- Return type detection (`Result` vs non-`Result`) at lines 82-93
- Non-send `ok_ty` rejection at lines 95-103
- Non-send `err_ty` rejection at lines 104-115 (only when not `Never`)
- Correct `Type::BlockingTask` result type at line 132

#### 2. CORRECT: Codegen for `__SifrBlockingTask` (`preamble.rs:378-400`)

The struct matches the model contract:
- `handle: Option<JoinHandle<__SifrTaskResult<T, E>>>` — stores the OS thread join handle, `Option` for "already joined/abandoned" case
- `observed: Arc<AtomicBool>` — tracks whether the observer consumed the result
- `_error: PhantomData<E>` — binds the error type without runtime cost

#### 3. CORRECT: `BlockingTask` Methods (`preamble.rs:628-677`)

- `join()`: Consumes `self`, marks observed, awaits handle or returns `cancelled()`
- `cancel()`: Borrows `self`, calls `handle.abort()` — does NOT mark observed (caller may later join)
- `cancel_and_join()`: Calls `abort()` then `join().await` — consistent with `__SifrTask` behavior

Cancellation behavior aligns with `async_concurrency_model.md:634-645`: cancellation requests abandonment but does not forcibly abort the OS thread once running.

#### 4. CORRECT: Helper Functions (`preamble.rs:1092-1151`)

Both `__sifr_spawn_blocking_infallible` and `__sifr_spawn_blocking_result` have correct type parameter bounds:
- `T: Send + 'static`
- `E: Send + 'static` (for the result variant)
- `F: FnOnce() -> T/Result<T, E> + Send + 'static`

#### 5. CORRECT: Integration Points

- `task_handle_calls.rs:9-12`: `is_task_handle_type` correctly includes `BlockingTask`
- `task_handle_calls.rs:22`: `cancel_and_join` added to allowed methods
- `lower_expr.rs:407-410`: Await-to-join desugaring includes `BlockingTask`
- `lower_expr.rs:1035-1051`: `join`/`cancel_and_join`/`cancel` method lowering includes `BlockingTask`
- `stmt_support_emitter.rs:276`: Await lowering in statement context includes `BlockingTask`
- `lib.rs:1363`: Runtime dependency detection includes both spawn_blocking helpers

#### 6. CORRECT: Non-Send Rejection

The check in `task_calls.rs:95-115` calls `non_send_reason(&ok_ty)` which traverses:
- Classes with `NonSend` marker inheritance
- Field types recursively
- Container element types

This is consistent with how `scope.spawn` handles non-send rejection.

#### 7. MINOR: Fixture Coverage Gap

The pass fixture `spawn_blocking_basic.sifr` only tests `await handle`. It does not exercise `handle.join()`, `handle.cancel()`, or `handle.cancel_and_join()`.

The unit tests verify that the type lowering is correct, and existing e2e fixtures (e.g., `task_handle_join.sifr`, `task_cancel_basic.sifr`) demonstrate these methods for `Task[T, E]`. Adding one e2e pass fixture that uses `handle.join()` or `handle.cancel_and_join()` on a `BlockingTask` would improve confidence.

The fail fixture `spawn_blocking_non_send_rejected.sifr` correctly tests non-send result type rejection.

#### 8. NOTE (not a finding): Future Capture Consideration

In v1, the zero-argument function restriction means the function has no parameters to capture non-send values. A future extension allowing parameterized sync functions would need to additionally validate that captured values are sendable. This is not a gap for the current v1 scope.

---

### Summary

The implementation is sound:

| Aspect | Status |
|--------|--------|
| HIR lowering correctness | ✓ |
| V1 zero-arg restriction | ✓ |
| `__SifrBlockingTask` struct | ✓ |
| `join`/`cancel`/`cancel_and_join` methods | ✓ |
| Non-send rejection for return types | ✓ |
| Async concurrency model alignment | ✓ |
| Integration with task-handle lowering | ✓ |
| Test fixtures | Minor gap in e2e coverage for method variants |

No blocking findings remain. The single gap is a suggestion to add an e2e pass fixture exercising `BlockingTask.join()` or `BlockingTask.cancel_and_join()` for production-level validation of those code paths, but this is not required for the slice to be considered complete.

REVIEW_STATUS: SATISFIED
