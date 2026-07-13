## PR #2958 — M7 Wave 2 Post-Push Review

### Prioritized Findings

**None material.** All three pre-PR review rounds (checked in at `plans/reviews/active/ad-hoc-python-interop-m7-wave2-review-round{1,2,3}.md`) were re-audited against the shipped diff and the fixes remain in place.

**Nits (non-blocking, do not gate merge):**

1. **NIT — misleading test `.expect()` message.** `crates/sifr_runtime/src/python/coroutine_ops.rs:71` — `concurrent_raw_coroutines_share_one_owned_loop_and_thread` calls `initialize_runtime(config).expect("init should start the owned loop")`, but `config` is derived from `test_config()` which defaults `start_async_loop: false` (`python_test_support.rs:77`). Init deliberately does **not** start the loop here — the test's whole point is proving the two workers race on lazy `ensure_started()`. Message contradicts the test's intent; message-only, no behavior change.
2. **NIT — round-3 residual, unchanged.** `crates/sifr_runtime/src/python/async_runtime.rs:161-166` — three `?`-gates between "handles taken" and `join()` can locally drop `loop_object`/`loop_thread` without invoking `stop()`/`join()` (`cancel_registered_submissions()?`, second `lock_state()?`, `wait_for_change()?`). Only reachable when `Python::try_attach` returns `None` (CPython finalized) or a mutex is poisoned — implausible in shipped paths (`PythonRuntimeGuard::drop` runs before finalize; state mutex is held briefly and never panics). Acknowledged by round 3 as non-blocking; defer to a future hardening pass.
3. **NIT — round-2 concern B, unchanged.** `crates/sifr_runtime/src/python/async_runtime.rs:255-259` — post-readiness self-failure sets `lifecycle = Failed` but leaves `loop_object`/`loop_thread` populated with stale references. Only reachable if `run_forever` / `shutdown_asyncgens` / `close` unexpectedly returns Err on the loop thread. A subsequent `shutdown()` will attempt `call_soon_threadsafe` on the dead loop and surface the `AsyncRuntimeFailed(previous_failure)` before propagating the stop error. Non-blocking.

### Verified

- **Round-1 HIGH lock-order inversion (shutdown ↔ GIL) — FIXED.** `async_runtime.rs:146-159` takes `loop_object`/`loop_thread` out under `ASYNC_STATE` then releases the guard at the block boundary before line 168's `Python::try_attach`. `cancel_registered_submissions` (`async_runtime.rs:310-329`) uses GIL→state, matching the worker path (`coroutine_ops.rs:6` acquires GIL via `super::attach`, worker helpers acquire state inside). No inverted acquisition order remains.
- **Round-1 MEDIUM concurrent lazy start — FIXED.** `start()` (`async_runtime.rs:61-77`) loops on lifecycle: `Running→Ok`, `Starting→wait_for_change`, `Stopping→Err`, `Disabled|Stopped|Failed→transition & spawn`. Second concurrent lazy caller waits on the condvar for the initiator, exercised by `concurrent_raw_coroutines_share_one_owned_loop_and_thread`.
- **Round-1 LOW test coverage — ADDRESSED.** Concurrent lazy-start test uses `start_async_loop=false` (default) and races two workers via a 3-party `Barrier` (`coroutine_ops.rs:60-106`); `shutdown_cancels_and_joins_an_in_flight_raw_coroutine` (`coroutine_ops.rs:108-149`) polls to registration (5000×1ms, with explicit cleanup on timeout) then races shutdown against a waiting future.
- **Conditional bootstrap.** `python_interop_plan.rs:83, 96, 130, 158-203` sets `requires_async_loop` when a module has an async declaration (`PythonInteropEffect::Async`) OR imports `sifr.python.run_coroutine_blocking` (including aliased names) AND calls it. Three plan tests (`python_interop_plan_tests.rs:99-158`) cover unaliased, aliased, and negative.
- **Cache identity.** `python.requires_async_loop=yes|no` appended in `push_python_plan_cache_key` (`python_interop_plan.rs:239-245`); folded into `InteropBuildPlan::cache_key_fragment` via `rust_interop_plan.rs:19-21`; asserted at `python_interop_plan_tests.rs:120-124`.
- **Codegen wiring.** `apply_package_runtime_metadata` (`project_codegen.rs:107`) sets `start_async_loop` from `interop.python.requires_async_loop`; `render_python_runtime_prelude` (`python_runtime.rs:191, 217`) renders it into generated `main.rs`; `package_python_runtime_starts_owned_loop_only_when_planned` (`project_codegen.rs:179-202`) asserts both `start_async_loop: true` and `start_async_loop: false` render.
- **Raw API semantics preserved.** `stdlib/sifr/python.sifr:778-781` still `@blocking_io def run_coroutine_blocking(...)`. Runtime now delegates to `async_runtime::run_coroutine_blocking` which uses `asyncio.run_coroutine_threadsafe(coro, loop)` + `Future.result()` (`async_runtime.rs:117-142`) — no per-call `asyncio.run` remains anywhere in the raw path (`coroutine_ops.rs` no longer imports `asyncio`).
- **Lifecycle & no-thread-leak.** Single named OS thread (`sifr-python-asyncio`), started only after CPython + bridge init + boundary-error registration (`python.rs:250-256`), readiness published via bounded `sync_channel(1)` before admission (`async_runtime.rs:80-105`). Six-state machine (`Disabled/Starting/Running/Stopping/Stopped/Failed`). Init failure via `fail_start` clears both handles (`async_runtime.rs:331-340`); `loop_setup_failure_is_joined_and_leaves_no_live_thread` (`async_runtime.rs:372-395`) asserts diagnostics normalize and a second shutdown surfaces `AsyncRuntimeFailed`.
- **Shutdown separation.** `stop_result` and `join_result` are computed independently (`async_runtime.rs:168-193`); `join()` unconditionally runs when `loop_thread` is `Some`; lifecycle normalized to `Stopped`, `loop_object` cleared, `failure` propagated before `stop_result.and(join_result)`.
- **Submission accounting.** Monotonic `u64` id (`saturating_add`), symmetric reserve/register/finish/release (`async_runtime.rs:262-308`). Two-phase drain: pending==0 then submissions.is_empty(), between which `cancel_registered_submissions` runs without holding state.
- **PythonRuntimeGuard drop-time shutdown.** `python.rs:117` invokes `async_runtime::shutdown()` before `drain_pending_releases`; workers releasing the GIL inside `Future.result()` allow cancellation to land without inversion.
- **Wave-2 gate discipline preserved.** `@python.coroutine` still hard-errors with `SIFR-PYRES-0002` (`python_lowering/python_interop.rs:410, 435, 639`); `cleanup=async_close` still hard-errors via `reserved_cleanup` (line 434-440). Wave-1 frontend contract from PR #2956 remains gated.
- **No panics in user paths.** No `.unwrap()`/`.expect()` outside `#[cfg(test)]` in the new async_runtime; all error paths use `?` or `.map_err`. `saturating_add`/`saturating_sub` guard the counters.
- **File-size guardrail.** `async_runtime.rs` 396 lines; `python.rs` 862 lines; `coroutine_ops.rs` 195 lines; `python_interop_plan.rs` 380 lines; `python_runtime.rs` 464 lines. All well under 900.
- **Validation report accurate.** The described `create-pr` profile run — all lanes green, 130/130 e2e fixtures, one transient LSP-smoke timeout after 23 successful requests, immediate unchanged repeat passed in 11s — matches the user-provided authoritative gate.

### PR-body sanity

- Summary bullets accurately describe the substrate.
- Explicit gate statement is present: "The typed `@python.coroutine` and `cleanup=async_close` frontend remains behind `SIFR-PYRES-0002`; this PR is M7 Wave 2 substrate only."
- Validation section matches shipped tests and the create-pr run; the transient LSP timeout is disclosed.
- Review history (rounds 1-3) is linked and matches the checked-in artifacts.

VERDICT: SATISFIED
