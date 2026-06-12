# M4 Process Timeout Status Wave — Review Pass 2

Branch: `codex/concurrency-runtime-m4-timeout-status`
Scope: pass-2 review after remediation of the pass-1 `Duration::from_secs_f64` overflow blocker. Same wave scope as pass 1 (timeout-aware `output`/`run`/`output_shell` variants + timeout `Status` evidence).

## RESULT: PASS

## Pass-1 blocker — fixed

Pass 1 flagged that `Duration::from_secs_f64(__timeout_seconds)` would panic for positive finite f64 values above `Duration`'s representable range (e.g., `1e30`), bypassing the guard that only rejected NaN/non-finite/negative inputs.

The remediation in the working tree closes that hole:

- `crates/sifr_codegen/src/intrinsics/registry/process.rs:89-94` introduces `duration_try_from_secs_f64`, which emits `std::time::Duration::try_from_secs_f64(seconds).map_err(|e| ProcessError { message: e.to_string() })?` via the shared `process_map_err` plumbing.
- `crates/sifr_codegen/src/intrinsics/registry/process.rs:450-462` (in `timeout_poll_stmts`) replaces the previous `Duration::from_secs_f64` deadline construction with the new fallible helper. The inner `?` propagates to the surrounding `Result<_, ProcessError>` body produced by `timeout_guard`, so an out-of-range positive finite timeout returns a typed `ProcessError` instead of panicking.
- `crates/sifr/tests/e2e/pass/process_timeout_status.sifr:37-41` adds the regression: `output_timeout(fast_cmd, 1e30)` is asserted to surface `ProcessError` with a non-empty message, alongside the existing negative-timeout case.
- `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:135` updates the codegen expectation to assert the rendered shape contains `std::time::Duration::try_from_secs_f64(__timeout_seconds)` (the bare `from_secs_f64` is no longer emitted).
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md:16` widens the contract phrasing to "reject invalid negative, non-finite, or **out-of-range** timeout values through `ProcessError`", matching the implementation.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:617` extends the emit grep to include `try_from_secs_f64`, so the execution ledger evidence now covers the checked conversion.

I confirmed the emitted Rust shape by running `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_timeout_status.sifr`. The relevant deadline line now reads:

```rust
let __deadline = std::time::Instant::now()
    + std::time::Duration::try_from_secs_f64(__timeout_seconds)
        .map_err(|__sifr_process_error| ProcessError {
            message: __sifr_process_error.to_string(),
        })?;
```

No bare `from_secs_f64` remains in the timeout path. `Duration::try_from_secs_f64` rejects negative, NaN, non-finite, and overflow inputs uniformly with `TryFromFloatSecsError`, and the `?` propagates it to the function-level `Result<_, ProcessError>` exactly like the existing `spawn`/`write_all`/`try_wait`/`kill`/`wait_with_output` failure paths in the same emitted function. The `timeout_invalid_expr` finiteness/negative guard is now defense-in-depth on top of the checked conversion (still useful for producing a more specific operator-facing error message before the conversion is attempted).

## No new blockers

I re-inspected the wave with the pass-1 acceptance criteria in mind. No new user-triggerable panic, no contract overclaim, no regression to the merged sync foundation:

- **Panic surface**: every fallible call in the timeout path (`spawn`, `write_all`, `try_from_secs_f64`, `try_wait`, `kill`, `wait_with_output`) is wrapped through `process_map_err` and `?`. No `unwrap`/`expect` on user data, no `panic!` macros, no arithmetic on user-controlled integers without bounds. `_status_from` in `lib/sifr/process.sifr:117-123` does not perform arithmetic that can panic.
- **Effect/workload diagnostics**: `output_timeout` / `run_timeout` keep `@blocking_io`; `output_shell_timeout` keeps `@blocking_io` + `@shell_exec`. The new fail fixture `crates/sifr/tests/e2e/fail/process_shell_timeout_direct_async_rejected.sifr` exercises `SIFR-ASYNC-0007` for the shell timeout direct-async case.
- **Status evidence**: timeout path sets `__status_code = -1`, `__timed_out = true`, and `_status_from` maps to `Status(kind="timeout", success=false, timed_out=true)`. The pass fixture asserts these for `output_timeout`, `run_timeout`, and `output_shell_timeout`.
- **Test/manifest coverage**: `process_timeout_status` appears in both `verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json`. The codegen unit test `lowers_process_timeout_intrinsics_via_registry` covers the rendered emission shape including the new `try_from_secs_f64`.
- **Scope honesty**: `verification/stdlib/concurrency_runtime_m4_process_traceability.md` Follow-up Boundaries still list child/pipe ownership, async spawn/wait/communicate, terminate/signal, `TaskGroup.spawn_process`, full text-mode closeout, stdin-setter semantics, and legacy intrinsic cleanup as remaining M4 work. The new traceability rows make narrow claims about timeout-status evidence only.
- **Foundation regression**: the sync `run`/`output`/`output_text`/`run_shell`/`output_shell`/`output_shell_text` lowering call sites are untouched by the remediation. The previously-extracted `spawn_child_stmt`/`write_stdin_stmt`/`wait_output_stmt` helpers continue to compose `spawn_output_stmts` for the non-timeout paths.

## Non-blocking observations (not required for PASS)

Recorded for honesty; none gate this wave.

- **Instant + Duration overflow at the next layer**: `std::time::Instant::now() + Duration::from_secs(s)` panics if `s` is large enough that the underlying clock representation overflows (in practice, around `s ≈ i64::MAX` seconds). `Duration::try_from_secs_f64` accepts inputs up to `u64::MAX` seconds, so a hyperextreme finite f64 timeout in the narrow band `[~9.2e18, ~1.844e19]` seconds (~30 billion years and up) would pass `try_from_secs_f64` and then panic at the `+`. This is the same defect class as the pass-1 blocker, but the realistic and pathological inputs called out in pass 1 (including the explicit `1e30` case) are now correctly rejected by `try_from_secs_f64` *before* the `Instant` addition, so the user-facing fixture is honest about its claim. A future hardening pass could switch the deadline computation to `Instant::checked_add(...).ok_or_else(|| ProcessError { ... })?` to close this narrow theoretical band; not in scope here.
- **Poll cadence (carried over from pass 1)**: `timeout_poll_stmts` still sleeps a fixed 1 ms between `try_wait` calls. Acceptable for sync timeout fixtures but expected to be revisited alongside the async/spawn lifecycle wave.
- **Output truncation under fill (carried over from pass 1)**: stdout/stderr are not drained during the polling loop, so a child that fills the 64 KB pipe buffer after a timeout will block writes and surface a truncated capture. Acceptable for the timeout slice; worth documenting when the full pipe lifecycle wave lands.
- **Direct-async coverage asymmetry (carried over from pass 1)**: `output_timeout` / `run_timeout` rely on the existing `process_blocking_direct_async_rejected` decorator-uniform path rather than a dedicated SIFR-ASYNC-0007 fixture. Mechanism-uniform, so not blocking.
- **`__status_code = -1` sentinel (carried over from pass 1)**: shared with the signal-killed exit code path. The wave's contract documents this and the fixtures discriminate via `status.kind`/`status.timed_out`.

## What I verified for this pass

1. **Re-emitted code**: confirmed `Duration::try_from_secs_f64` replaces `Duration::from_secs_f64` in the deadline computation, the `.map_err(...)?` chain routes through `ProcessError`, and the surrounding `Result<_, ProcessError>` function return type makes the `?` legal.
2. **Pass fixture coverage**: confirmed `crates/sifr/tests/e2e/pass/process_timeout_status.sifr` exercises both negative (`-1.0`) and overflow (`1e30`) cases and asserts `ProcessError`. The fast happy-path, deadline expiry for `output_timeout`/`run_timeout`/`output_shell_timeout`, and the timeout-status fields remain covered.
3. **Codegen unit test**: confirmed `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:135` greps for the new `try_from_secs_f64` shape, so an accidental future revert to `from_secs_f64` would fail the test.
4. **Traceability accuracy**: confirmed `verification/stdlib/concurrency_runtime_m4_process_traceability.md:16` says "negative, non-finite, or out-of-range", matching what the emitter now actually enforces.
5. **Validation lanes**: confirmed `process_timeout_status` appears in both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json`. JSON validates as reported.
6. **Issue ledger**: confirmed the emit-grep step in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:617` includes `try_from_secs_f64`, so future evidence runs prove the checked conversion is still emitted.
7. **No regressions to merged foundation**: the remediation diff touches only the timeout helpers and their tests/fixtures; the sync foundation lowering call sites in `lower_process_run`/`lower_process_output`/`lower_process_output_text`/`lower_process_shell_run`/`lower_process_shell_output`/`lower_process_shell_output_text` are unchanged.

This wave is ready to PR.
