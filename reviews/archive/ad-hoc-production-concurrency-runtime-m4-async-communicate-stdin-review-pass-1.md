I have enough information to provide a thorough review. Let me compose the verdict.

# Review Verdict: PASS

The M4 async stdin-byte communicate wave is implemented correctly and within its declared scope. Argument plumbing, generated Tokio I/O orchestration, guard rails, and docs all line up.

## What I verified

**1. Public wrapper / intrinsic metadata / lowerer / helper argument order (matches spec):**
- `lib/sifr/process.sifr:394-417`: `async_output` passes `(program, args, env, cwd, has_cwd, stdin_mode, stdin_data, has_stdin_data)`; `async_output_timeout` adds `seconds` at the tail. ✓
- `crates/sifr_stdlib/src/process.rs:236-260`: intrinsic signatures gained `stdin: Bytes` and `has_stdin: Bool` for both output and output_timeout. ✓
- `crates/sifr_codegen/src/intrinsics/registry/process_async.rs:55-80`: arities raised to 8 / 9, stdin bytes are cloned through `RustExpr::Clone` and trailing flags / timeout passed in-place. ✓
- `crates/sifr_codegen/src/preamble/process_async_runtime.rs:39-77`: `process_async_params(include_stdin=true)` appends `stdin: Vec<u8>` *before* `has_stdin: Bool`. ✓

**2. Deadlock-free communicate orchestration:**
- Output and output_timeout helpers stdin/stdout/stderr-pipe via `Stdio::piped()` only when needed; `__stdin = __child.stdin.take()` is moved into the inner async block via `__stdin.take()`, and `__pipe` drops at end of the `if let Some(...)` arm — that's what gives `cat` an EOF without an explicit `shutdown`/close. ✓
- All four futures (`__child.wait()`, stdin write, stdout read, stderr read) are concurrent via `tokio::try_join!`, so the OS pipe buffers cannot stall the child. ✓
- Borrow surface is clean: `__stdin`, `__stdout`, `__stderr`, `__stdout_bytes`, `__stderr_bytes` are independent locals; `__child.wait()` only borrows `__child` (whose stdin/stdout/stderr Options are already `None`).

**3. Guardrails preserved:**
- `process_async_stdin_mode_guard` still rejects `stdin_mode != "inherit"` with a typed `ProcessError("async process stdin mode requires owned pipe support")`, so `Stdio("pipe")` / `Stdio("null")` keep deferring (`process_async_runtime.rs:79-94`). Tests exercise this for both helpers and for `async_run`/`async_run_timeout`. ✓
- The previous `has_stdin → Err(owned pipe deferral)` was removed from the output helpers and replaced by actual consumption. ✓
- `async_run` / `async_run_timeout` keep their pre-existing signatures (no stdin bytes plumbed, no overclaim).

**4. Timeout helper still kills/reaps:** `process_async_runtime.rs:461-468` still `__child.kill().await` then `__child.wait().await`, returning typed `Status(kind="timeout", timed_out=true)` and an `Output` with empty buffers. ✓

**5. Helper gating remains minimal:** Four independent `needs_*` flags emit four helpers; emission audit recorded in the ledger confirms `process_async_run_timeout` does *not* drag in output / output_timeout helpers.

**6. Tests:**
- `process_async_run_output.sifr` asserts `b"async-" + b"stdin"` → stdout `b"async-stdin"`, status success, empty stderr, plus `Stdio("pipe")` rejection for both `async_output` and `async_run`. ✓
- `process_async_output_timeout.sifr` asserts the analogous communicate path plus the existing timeout/`finite`/`Stdio("pipe")` cases. ✓

**7. Docs honest about scope:** Traceability + matrix updated; remaining work is correctly narrowed to "async spawn/wait, public async owned pipes, shell async APIs, cancellation, scoped supervision". No Windows / text / async-pipe support claimed. File-size guardrail OK (largest touched file is `process_async_runtime.rs` at 625 lines).

## Non-blocking residual risks worth flagging

- **`supported_host_matrix.md:21` (Async subprocess output timeout row) is not updated** to mention stdin-byte communicate coverage — only the run/output row was. The fixture *does* exercise stdin-byte communicate inside `process_async_output_timeout.sifr`, so the matrix narrative under-reports current coverage. Minor docs nit; consider mirroring the same "and one-shot stdin-byte communicate" phrase you added to the run/output row.
- **`BrokenPipe` surfaces as `ProcessError`.** If a child closes stdin before consuming all bytes (e.g. `head -c 5` style), `write_all` will fail and `try_join!` will short-circuit, masking a successful child exit. Acceptable for the cooperating-child happy path the wave targets, but Python `subprocess.communicate` swallows this — call out as a known shape if/when public async pipes land.
- **Cancellation hygiene for the non-timeout output helper.** In the non-timeout path, if one of the I/O futures errors first, `try_join!` cancels `__child.wait()` and `__child` is dropped without `kill_on_drop`. The child can keep running and become a zombie until natural exit. The prior `Command::output()` path had the same property (Tokio's `wait_with_output` also doesn't kill on cancel), so this is not a regression — but a follow-up wave that adds cancellation safety should address it.
- **Silent ignore for `stdin_bytes` + `async_run` / `async_run_timeout`.** Consistent with sync `run()`'s behavior and the user-stated "do not overclaim" rule, but a future wave may want a parity guard with sync `spawn(...)`'s typed `"does not consume Command.stdin_bytes"` rejection so users don't quietly lose data.

Nothing above is a blocker for this narrowly-scoped wave.
