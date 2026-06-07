# M4 Process Timeout Status Wave — Review Pass 1

Branch: `codex/concurrency-runtime-m4-timeout-status`
Scope: incremental wave on top of merged PR #2331 sync process foundation; adds timeout-aware `output`/`run`/`output_shell` variants and timeout `Status` evidence. Does not claim closure of M4 child/pipe/async/signal/supervision.

## RESULT: CHANGES_REQUESTED

## Blocker 1 — `Duration::from_secs_f64` overflow panic is user-triggerable

File/line: `crates/sifr_codegen/src/intrinsics/registry/process.rs:403-420` (`timeout_invalid_expr`), used by `timeout_guard` in `crates/sifr_codegen/src/intrinsics/registry/process.rs:421-441` and consumed at the `from_secs_f64` site in `timeout_poll_stmts` (`crates/sifr_codegen/src/intrinsics/registry/process.rs:466-475`).

The generated guard is:

```rust
let __timeout_seconds: f64 = seconds;
if !__timeout_seconds.is_finite() || (__timeout_seconds < 0.0) {
    Err(ProcessError { message: format!(...) })
} else {
    ...
    let __deadline = std::time::Instant::now()
        + std::time::Duration::from_secs_f64(__timeout_seconds);
    ...
}
```

`Duration::from_secs_f64` panics on three classes of input: NaN/non-finite, negative, **and overflow** (any positive finite value above roughly `u64::MAX` seconds ≈ 1.844e19). The current guard only rejects the first two classes. A user passing `seconds = 1e30` (or any positive finite value over the `u64::MAX` boundary) passes the guard and then panics inside generated code:

```
thread 'main' panicked at .../core/src/time.rs: cannot convert float seconds to Duration: value is either too big or NaN
```

(Reproduced directly via `Duration::from_secs_f64(1e30)`; `is_finite() == true`, `> 0.0`, and the call still panics.)

Why it matters: this violates the project's "if it compiles, it works" / no-user-triggerable-runtime-panics guarantee called out in `AGENTS.md`. It is reachable from any Sifr program that does `output_timeout(cmd, 1e30)`, `run_timeout(cmd, 1e30)`, or `output_shell_timeout(script, 1e30)`. The wave specifically advertises that "invalid negative/non-finite timeout values" return typed `ProcessError`, so overflow values that today panic are a directly in-scope hole — the contract claim in `verification/stdlib/concurrency_runtime_m4_process_traceability.md:16` is inaccurate as written.

Suggested fix (smallest possible): either

- Replace `Duration::from_secs_f64(__timeout_seconds)` with `Duration::try_from_secs_f64(__timeout_seconds)` and map `Err` to `ProcessError`, or
- Extend `timeout_invalid_expr` with an upper-bound clause (`|| __timeout_seconds > (u64::MAX as f64)`) so the same `ProcessError` path covers overflow.

Either approach also needs a regression fixture in `crates/sifr/tests/e2e/pass/process_timeout_status.sifr` (or a sibling fixture) that asserts `output_timeout(cmd, 1e30)` returns `Err(ProcessError)` rather than panicking.

## Non-blocking observations (not required for PASS)

These are recorded for honesty and to keep follow-up boundaries explicit; none of them gate this wave.

- Poll cadence: `timeout_poll_stmts` sleeps a fixed 1 ms between `try_wait` calls. Fine for short-lived test fixtures, but for multi-second timeouts this is ~1000 wakeups/s per process. Worth revisiting alongside the later async/spawn lifecycle wave that should replace the polling shape with proper event-driven waiting; not in scope here.
- Output truncation under fill: while polling, stdout/stderr are not drained. A child that fills the 64 KB pipe buffer will block writes; after `kill` and `wait_with_output`, the captured `stdout`/`stderr` will be truncated at the kernel pipe buffer. Behavior is acceptable for a timeout path, but worth documenting as a known limitation in the follow-up boundaries when full pipe lifecycle lands.
- Direct-async rejection coverage: `output_shell_timeout` has its own SIFR-ASYNC-0007 fixture (`process_shell_timeout_direct_async_rejected.sifr`). `output_timeout` / `run_timeout` are `@blocking_io` and rely on the existing `process_blocking_direct_async_rejected.sifr` decorator path — that path is mechanism-uniform, so this is acceptable, but a one-line negative fixture for `output_timeout` directly would make the timeout slice symmetric with the shell slice.
- `__status_code` sentinel is `-1` on timeout. Same value used elsewhere by `code().unwrap_or(-1)` for signal-killed exits, so a consumer cannot today distinguish "killed by external signal" from "killed by our timeout" via `code` alone — must use `status.kind`/`status.timed_out`. That is the intended contract, and the test asserts on `kind == "timeout"` and `timed_out == True`, so the wave is internally consistent. Just worth a note when the explicit signal/kill wave lands.

## What I verified

1. **User-triggerable panic paths**: Found the `Duration::from_secs_f64` overflow case above. Inspected the rest of the emitted timeout path (`Instant::now`, `try_wait`, `kill`, `wait_with_output`, `write_all`) — every fallible call is wrapped through `process_map_err` and the `?` operator, no `unwrap`/`expect` on user data, no `panic!` macros. The `Status` construction in `lib/sifr/process.sifr:117-123` does not perform arithmetic that can panic.
2. **Child timeout behavior**: `timeout_poll_stmts` polls `try_wait` and on deadline sends `kill()` (SIGKILL on Unix), then proceeds to `wait_with_output` which reaps the child and drains piped stdout/stderr. `__status_code = -1, __timed_out = true` on timeout; `__status_code = exit code, __timed_out = false` on normal exit. The `_status_from(code, timed_out)` helper in `lib/sifr/process.sifr:117-123` correctly produces `Status(kind="timeout", success=False, timed_out=True)` for the timeout case.
3. **Invalid timeout handling**: NaN, ±Inf, and negative values are rejected by `timeout_invalid_expr` and surface as typed `ProcessError`. Overflow is **not** — see Blocker 1.
4. **Effect/workload diagnostics**:
   - `output_timeout` / `run_timeout` carry `@blocking_io` (`lib/sifr/process.sifr:126,141,159,176`), matching the foundation wave's pattern.
   - `output_shell_timeout` carries both `@blocking_io` and `@shell_exec` (`lib/sifr/process.sifr:235-241`), matching `output_shell` / `output_shell_text`.
   - New fail fixture `crates/sifr/tests/e2e/fail/process_shell_timeout_direct_async_rejected.sifr` expects `SIFR-ASYNC-0007` for direct-async shell timeout calls. Confirmed it's a sibling of the existing `process_shell_exec_direct_async_rejected.sifr` shape, and the user reports 418 fail tests pass.
5. **M4 overclaim check**: `verification/stdlib/concurrency_runtime_m4_process_traceability.md` keeps status as "In progress; sync process foundation wave reviewed and merged in PR #2331." The Follow-up Boundaries section still lists `spawn`/`Child`/owned pipes, async spawn/wait/communicate, terminate/signal, `TaskGroup.spawn_process`, full text-mode closeout, stdin-setter semantics, and legacy intrinsic cleanup as remaining work. The new rows for `run_timeout`/`output_timeout`/`output_shell_timeout` make narrow, accurate claims and do not assert child-lifecycle/pipe-ownership/signal-handling completion. Honest scoping is preserved.
6. **Test/manifest sufficiency for this slice**:
   - Pass fixture `crates/sifr/tests/e2e/pass/process_timeout_status.sifr` covers: fast happy-path with status.success+kind=="success", `output_timeout` deadline expiry, `run_timeout` deadline expiry, `output_shell_timeout` deadline expiry, and negative-timeout rejection via `ProcessError`.
   - Both create-pr and merge e2e manifests are updated to include `process_timeout_status` and JSON is valid.
   - Codegen unit test `lowers_process_timeout_intrinsics_via_registry` (`crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:113-152`) covers the rendered emission shape (Instant::now, Duration::from_secs_f64, try_wait, kill, 4-tuple).
   - Gaps: overflow timeout (see Blocker 1); no explicit fixture for `output_timeout` direct-async rejection (decorator-uniform with existing `process_blocking_direct_async_rejected.sifr`, so not blocking).
7. **Regressions to existing sync foundation**: The codegen refactor extracted `spawn_child_stmt`, `write_stdin_stmt`, `wait_output_stmt` out of the previous monolithic `spawn_output_stmts`. The new `spawn_output_stmts` simply composes the three. I verified the composed shape is equivalent to the prior emission, and the file-size guardrail still passes. No changes to `process_run`, `process_output`, `process_output_text`, `process_shell_run`, `process_shell_output`, `process_shell_output_text` lowering call sites. Risk to the existing foundation is low and consistent with the user-reported PASS of fmt/check/codegen tests/e2e fail suite.

## How to clear the blocker

1. Patch the guard (either widen `timeout_invalid_expr` to reject `> u64::MAX as f64`, or switch the deadline computation to `try_from_secs_f64` + `?` through `process_map_err`).
2. Add a positive-overflow case to `crates/sifr/tests/e2e/pass/process_timeout_status.sifr` (assert `Err(ProcessError)` for `seconds = 1e30`), or a dedicated fail fixture.
3. Re-emit and confirm the generated code no longer contains a bare `from_secs_f64(__timeout_seconds)`; alternatively grep for `from_secs_f64` in the emitted sample to verify it has been replaced or guarded.
4. Update `verification/stdlib/concurrency_runtime_m4_process_traceability.md:16` only if its wording needs to remain unchanged after the fix (it already says "reject invalid negative/non-finite timeout values" — after the fix this should read "reject invalid negative, non-finite, or out-of-range timeout values" so the traceability is exact).

After that change, this wave looks ready to PR.
