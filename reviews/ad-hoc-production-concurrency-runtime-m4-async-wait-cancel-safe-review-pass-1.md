## Review: M4 async wait cancel-safe - **PASS**

**Correctness (`process_async_child_runtime.rs:478-517`)**
- Lock is scoped to inner blocks; `await` on `__child.wait()` is held only via `&mut` through the guard, never under a `MutexGuard`. [ok]
- `Drop::drop` re-inserts the child if `Option` still has it - covers cancellation (future dropped mid-`await`) and the wait-error path; only successful observation calls `__guard.child.take()` before return. [ok]
- Mutex poisoning handled via `unwrap_or_else(|e| e.into_inner())`; no data-dependent `unwrap`/`expect`/panics. [ok]
- The `None` arm of `match __guard.child.as_mut()` is dead (we just set `Some(__child)` two lines above with no intervening await/move). Harmless defensive code, not a blocker.

**Fixture (`process_async_wait_cancel_safe.sifr`)**
- Spawns `sleep 30`, cancels `child.wait()` via `task.timeout(0.0)` -> asserts `TimeoutError` path, then re-kills, re-waits, validates SIGKILL signal status. Exercises exactly the property the guard enforces (handle survives cancellation, second wait still observable). [ok]

**Docs / manifests**
- `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` both include `process_async_wait_cancel_safe`. [ok]
- `concurrency_runtime_m4_process_traceability.md`: AsyncChild row updated to reflect the cancellation-preservation invariant, fixture listed in validation lanes + CPython adapted table; "Cancellation-safe process observation" removed from Follow-up Boundaries; AsyncPipeReader/Writer row scrubbed of stale "cancellation-safe observation" wording. [ok]
- `issues/...substrate-execution.md`: adds "M4 async wait cancellation-safe observation: in progress." line - consistent with "under review" status. [ok]

**Minor (non-blocking)**
- The `Async async_run/...` row in traceability still mentions "cancellation ... remain later M4 work" - that refers to argv-helper cancellation rather than `AsyncChild.wait`, so it's compatible with this PR's scope, but worth a follow-up tightening if you want zero ambiguity.

**Verdict: PASS** - ship.
