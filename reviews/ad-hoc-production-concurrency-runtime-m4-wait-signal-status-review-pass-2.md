Pass 2 — M4 wait signal status evidence wave.

Both pass-1 blockers are resolved and re-verified against the actual diff.

RESULT: PASS

## Pass-1 blocker verification

### 1. `lower_process_run` regression — fixed
`crates/sifr_codegen/src/intrinsics/registry/process.rs:585-606` now emits `Ok(status_code(__status))` again; only `lower_process_wait` at `crates/sifr_codegen/src/intrinsics/registry/process.rs:703-709` returns the `(status_code, status_signal)` tuple. `crates/sifr_stdlib/src/process.rs:27-39` still types `process_run` as `result_ty(Type::Int, "ProcessError")` while `crates/sifr_stdlib/src/process.rs:53-59` correctly types `process_wait` as `process_status_signal_tuple()` (`Tuple(Int, Int)`). Confirmed locally by re-running `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_output_text.sifr` and `process_sync_bytes_env_cwd_stdin.sifr` — both PASS where pass-1 saw E0308.

### 2. Full create-pr local validation — re-run
`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` now records `scripts/run_all_tests.sh --profile create-pr` -> PASS against the reverted diff with `pass=95, fail=0, cache_hits=19/25, report_signature=d8d730bd5475756c`. Warm wall-time and warm-cache-hit advisories are noted but are non-blocking advisory budgets, not gate failures. The targeted pass fixtures and the `lowers_process_wait_signal_tuple_via_registry` registry test are also re-recorded. The wave is now gated by the same `scripts/run_all_tests.sh --profile create-pr` lane that CI uses.

## Correctness, panics, types

- `lower_process_wait` returns `Ok(( __status.code().unwrap_or(-1) as i64, { #[cfg(unix)] { ExitStatusExt::signal(&__status).unwrap_or(-1) as i64 } #[cfg(not(unix))] { -1i64 } } ))`. Emitted Rust verified by `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr`; the cfg-attribute on the trailing block expression is the standard cross-platform pattern and `unwrap_or(-1)` on both `code()` and `signal()` removes data-dependent panics. No `expect()`/`unwrap()` is introduced in the generated runtime path.
- `lib/sifr/process.sifr:41-47` extends `Status.__init__` with an optional `signal: int | None = None`; default preserves all existing callers (`_status`, `_status_from`) that omit `signal`. `lib/sifr/process.sifr:153-158` (`_status_from_wait`) keys on `signal >= 0`, so the non-Unix `-1` sentinel cannot misclassify a process as signal-killed and correctly falls through to `_status(code)`.
- Public surface invariants hold: `lib/sifr/process.sifr:77-85` (`Child.wait`) and `lib/sifr/process.sifr:181-187` (top-level `wait`) both still return `Result[Status, ProcessError]`. The "already been waited" guard at `lib/sifr/process.sifr:78-80` and the handle ownership semantics around `Child.kill` are unchanged. `run()` at `lib/sifr/process.sifr:190-202` continues to consume a plain `int` from `process_run`.

## Test adequacy

- `crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr:18-44` forks on `status.code == -1` to exercise both the Unix signal-evidence branch (`kind == "signal"`, `signal is not None`) and the non-Unix fallback (`kind == "nonzero"`, `signal is None`) for both the top-level `wait(child)` and the method-form `Child.wait()`. The "after wait kill" `ProcessError` assertion at line 36-40 is preserved, so kill-after-wait diagnostic coverage is not lost. The final assertion expands to 7 elements and matches the new appends.
- `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:158-168` asserts the new lowering contains both `#[cfg(unix)]` and `#[cfg(not(unix))]` branches, the `ExitStatusExt::signal(&__status)` call, the `-1i64` non-Unix sentinel, and the `(__status.code().unwrap_or(-1) as i64` tuple-head prefix; it is a structural check on the emitted IR, sufficient for this milestone.

## Documentation honesty

- `verification/stdlib/concurrency_runtime_m4_process_traceability.md:12,18,45` adds `process_child_kill_wait` evidence for `Status` and rewrites the kill row to explicitly state "Unix wait status records signal evidence; non-Unix signal value falls back to the `-1` sentinel". The intentional-open list still calls out graceful `terminate`, escalation, parent cancellation evidence, async process, pipes, scoped supervision, and method-form `@blocking_io` enforcement, matching the wave's stated scope.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` records both the change summary, the targeted validation, the post-review collateral-regression check after reverting `process_run`, and the pass-1 review loop with the fix.

## Non-blocking note (carry forward)

`crates/sifr_codegen/src/intrinsics/registry/process.rs:185-189` still uses a raw-string `RustExpr::Ident(format!(...))` shim to inject the platform-gated block. It compiles to valid Rust via `prettyplease` and the codegen test pins both branches, but it bypasses the structured `RustExpr` builders used elsewhere in this file. A future small refactor introducing a `RustExpr::CfgBranch` (or a `RustExpr::Block` with two cfg-attributed statements) would be safer; not blocking phase closure.

RESULT: PASS
