RESULT: PASS

## Verified

**Code/lowering correctness**
- `lib/sifr/process.sifr:401-410` calls `process_async_output_timeout` with 7 positional args matching the intrinsic signature in `crates/sifr_stdlib/src/process.rs:256-270`.
- `crates/sifr_codegen/src/intrinsics/registry.rs:619-623` and `registry/process_async.rs:66-77` lower to `Box::pin(__sifr_process_async_output_timeout(prog.clone(), args.clone(), env.clone(), cwd.clone(), has_cwd, has_stdin, seconds))`. Argument order matches the helper params in `preamble/process_async_runtime.rs:188-195`.
- Generated Tokio helper (verified by `cargo run -p sifr -- emit`):
  - Rejects `has_stdin` with typed `ProcessError` citing owned pipe support deferral.
  - Rejects non-finite/negative duration with typed `ProcessError`.
  - Uses `Duration::try_from_secs_f64(...)?` with `map_err`, so out-of-range is also typed.
  - Sets `kill_on_drop(true)` on `__cmd` before awaiting `__cmd.output()`; on `tokio::time::timeout` Err the future is dropped, which drops the spawned `Child` and triggers SIGKILL via the inherited kill_on_drop flag.
  - Returns `Output::new(Vec::new(), Vec::new(), Status{ kind="timeout", success=false, timed_out=true })` on timeout, matching the `Status`/`Output` Sifr classes (`lib/sifr/process.sifr:43-71`).
  - Only Copy-safe `unwrap_or(-1)` on `ExitStatus::code`; no user-triggerable `.unwrap()/.expect()`.
- No public Tokio leak: helper is `Visibility::Private`.

**Gating + plumbing**
- `SharedPreludeProcessAsyncNeeds.needs_output_timeout` added in `stdlib_filter/implementation.rs:53-58`, set by both AST visitor (`:382-386`) and text scan (`:329-338`), and included in `is_shared_prelude_item` (`:419-425`).
- `lib_modules_and_codegen.rs:393-396, 422-426, 595-602` wires it into the OR for `needs_process_async` and forwards to `build_process_async_items(..., needs_output_timeout)`.

**Traceability / manifests**
- Added to `verification/validation_lanes/{create_pr,merge}_e2e_manifest.json`.
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md` updates Status, Output, async, and CPython rows with the new fixture; supported-host matrix row renamed and now lists all three async fixtures.
- Execution ledger records the new fixture, the cargo/check/guardrail/e2e/create-pr profile evidence (with `report_signature=9212e77abfa82acc`), and the in-review status.

**Local replay**
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` -> exit 0 (cache hit on the prebuilt artifact). The fixture covers success, timeout (empty stdout/stderr + `kind="timeout"` + `timed_out`), invalid negative duration, and the stdin-bytes deferral message.

## Non-blocking notes

- Out-of-scope dirty files in the working tree must NOT be staged for this PR:
  - `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
  - `issues/ad-hoc-production-network-http-platform-substrate.md`
  - `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md` (untracked)
  - `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-2.md` (untracked)
  - `reviews/ad-hoc-production-concurrency-runtime-m4-async-output-timeout-review-pass-1.md` is an empty (0-byte) placeholder; either populate with the review record or omit from the commit.
- `output_timeout` relies on `kill_on_drop` Drop semantics rather than an explicit awaited `child.kill().await` (as `run_timeout` does). That divergence is honest in the traceability ("kill_on_drop(true) for timeout cancellation … partial async output timeout capture through communicate … remain follow-up"), so this is acceptable for the milestone but worth keeping flagged.
- Existing raw-Rust-via-`RustExpr::Ident` pattern (now reused in `output_timeout_body`) is a pre-existing escape hatch in `process_async_runtime.rs`; not a blocker, but consider modelling `tokio::time::timeout` as proper `RustExpr` in a future cleanup so generated-code quality stays auditable.
