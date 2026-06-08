RESULT: PASS

I reviewed the uncommitted diff for the M4 async wait cancellation-safe observation wave end-to-end and wrote the full review artifact at `reviews/ad-hoc-production-concurrency-runtime-m4-async-wait-cancellation-review-pass-1.md` (the pass-1 file existed but was empty from the prior aborted run).

Headline findings:

- **Cancellation safety is real.** `process_async_wait_body` (`crates/sifr_codegen/src/preamble/process_async_child_runtime.rs:520`) atomically moves a `Running` child into a `Waiting(Arc<Mutex<Option<…>>>)` slot under a single children-table lock with no `.await` held, spawns a background Tokio wait task that writes the result once, and polls the slot from the caller side via `tokio::time::sleep(1ms).await`. Dropping the caller future (e.g., `async with task.timeout(...)`) leaves the slot, the spawned task, and the child untouched, and a later `child.wait()` reattaches to the same slot and observes the final status.
- **One-shot cleanup is preserved.** Successful observation calls `__children.remove(&handle)`; a third wait returns the typed `"closed or unknown"` `ProcessError`. No `unwrap()` on data, all mutex `lock()`s recover from poison, no nested locks.
- **Pipe/kill/terminate during waiting** return typed `ProcessError("…already being waited")` instead of reaching into the moved child — non-panicking and consistent with the public surface (which never promised kill-during-wait).
- **`kill_on_drop(true)`** is correctly limited to a backstop: it propagates from `__cmd` to the spawned `Child`, so runtime shutdown / aborted wait tasks reap host processes. Issue ledger and traceability describe it as a backstop, not a public cancellation semantic.
- **Fixture and bookkeeping are honest.** `process_async_wait_timeout_retry.sifr` exercises timeout-cancellation, retry-observation of exit code 7, and closed-handle on third wait; it is wired into both create-pr and merge manifests, M4 traceability, the supported-host matrix row, the CPython lifecycle row, and the execution ledger; the "Cancellation-safe process observation" item is removed from follow-ups.
- File-size guardrail OK (624/900 lines).

**No required changes before PR.** Non-blocking notes captured in the artifact: switch the 1ms polling to `tokio::sync::Notify`/`oneshot`/`watch` in a later wave, consider shared `Arc<Mutex<Child>>` only if a future use case needs kill-during-wait, the slot's `String` error clones per poll are negligible but avoidable, and the generic "cancellation" wording in the `async_run` / `async_output` row could be tightened in a doc-only follow-up.

I trusted the recorded targeted-validation results (the `third_party/ruff` submodule isn't linked in this review worktree, so `cargo run -- emit` could not be re-derived here); the merge-gate `scripts/run_all_tests.sh` profile is still owed before PR per AGENTS.md, separately from this review.
