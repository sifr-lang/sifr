

Now I have sufficient information. Let me formulate my review.

---

## Review: Phase 32 `milestone_async_2` — Task Cancellation (`handle.cancel()`)

**Verdict: SATISFIED**

### 1. Blocking Findings
None.

### 2. Non-Blocking Observations

**O1 — Missing negative validation fixture for cancel-after-await.**
The HIR test `test_task_handle_cancel_does_not_consume_handle_binding` (expressions_tests.rs:1539) proves `cancel()` borrows, but there is no corresponding negative fixture asserting that `await handle` (which consumes the handle) followed by `handle.cancel()` is rejected. The join path has `task_handle_double_await_rejected.sifr`; a cancel counterpart like `task_handle_cancel_after_await_rejected.sifr` would mirror that coverage.

**O2 — Codegen test obfuscation is unnecessary.**
`lib_codegen_tests.rs:3767–3770` uses `format!("fn {}{}", "can", "cel(&self)")` to spell `fn cancel(&self)`. This prevents plain-text search and adds cognitive load. The test could use `contains("fn cancel(&self)")` directly — no policy reason for obfuscation here unlike tests that guard against generated code that might differ subtly.

**O3 — `__SifrTaskResult::Cancelled` variant is not tuple-like in the generated code.**
The emit shows `Cancelled` with no fields (line: `Cancelled,` in the generated enum). The design doc says `Cancelled(Failure[CancellationError])`, but the conservative milestone_async_2 implementation uses a zero-variant `Cancelled`. This is fine for the current milestone because the conservative infallible-spawn variant uses `std::convert::Infallible` as `E` and stores no error payload. However, milestone_async_3's `TaskResult.Cancelled(Failure[CancellationError])` will need a schema migration. Mark this for tracking.

**O4 — No `cancel()` diagnostic coverage.**
There is no HIR diagnostic test asserting that `handle.cancel(arg)` (with arguments) or `handle.cancel(key=value)` produces a correct-arity diagnostic. The arity/path is guarded by the shared `call.arguments.args.is_empty()` check (task_handle_calls.rs:29–35), but unlike the join path, there is no negative unit test exercising those code paths for `cancel` specifically. The diagnostic messages are parameterized with `method_name`, so the infrastructure is correct — the gap is in test coverage.

**O5 — Tokio `abort()` semantics are sound.**
`tokio::task::AbortHandle::abort()` is idempotent and returns immediately. It is a no-op on an already-completed task, and repeated calls are safe. The generated `fn cancel(&self)` calls `self.abort_handle.abort()` — this satisfies the model contract (async_concurrency_model.md:361) without needing additional state management.

### 3. Design Consistency

| Concern | Status |
|---|---|
| `cancel()` borrows, does not consume handle | ✅ HIR: no `mark_moved`, codegen: `&self` |
| `join()` consumes, marks handle moved | ✅ HIR: `mark_moved` on name binding, codegen: `self` by value |
| `cancel()` returns `None` (no return type in HIR) | ✅ `ty: Type::None` |
| After cancel, `await handle` still valid for cleanup | ✅ Verified by HIR test + e2e fixture |
| Cancel after task completion is a no-op | ✅ Tokio `abort()` handles this naturally |
| Repeated cancel is a no-op | ✅ Tokio `abort()` handles this naturally |
| Runtime substrate uses `tokio::task::AbortHandle` | ✅ Stored on `__SifrTask`, captured from `JoinHandle` at spawn |
| Codegen lowers to `handle.cancel()` method call | ✅ `lower_expr.rs:991–997` |
| Doc reflects cancellation as in-progress | ✅ 32_async_ecosystem.md:396–397, roadmap.md |

### 4. Extra Validation Recommended Before PR

- Add `task_handle_cancel_after_await_rejected.sifr` negative fixture (O1)
- Add `cancel()` arity-diagnostic unit test in `expressions_tests.rs` or a dedicated `task_handle_cancel_diagnostics_tests.rs` (O4)
- Track the `Cancelled(Failure[CancellationError])` schema migration for milestone_async_3 (O3) — not blocking this PR
- Consider simplifying `lib_codegen_tests.rs:3767` to use plain string search (O2 — low priority)

Otherwise the implementation is correct, minimal, and consistent with the async concurrency model contract. The ownership flow has no holes: cancel borrows, join consumes, await consumes — each behavior is isolated and verified.
