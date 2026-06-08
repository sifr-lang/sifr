# Concurrency Runtime M4 Process Traceability

Milestone: `milestone_concurrency_runtime_4`

Status: In progress; sync process foundation merged in PR #2331, sync child wait merged in PR #2334, timeout status evidence merged in PR #2336, sync child kill support merged in PR #2337, Unix signal-status evidence merged in PR #2341, legacy subprocess intrinsic cleanup merged in PR #2344, async run/output loopback merged in PR #2345, stdin append semantics merged in PR #2348, method-form blocking diagnostics merged in PR #2350, and async run timeout is in progress.

## Production Surface Traceability

| Surface | M4 evidence | Notes |
| --- | --- | --- |
| `sifr.process.Command` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin` | Native argv command builder with ordered arguments, explicit env entries, cwd selection, and owned stdin byte payload capture. Repeated `stdin_bytes(...)` calls append in call order. This is the production `sifr.process` path and does not use `sifr.subprocess` or legacy shell-shaped helpers. |
| `sifr.process.Status` | `process_sync_output_text`; `process_shell_exec_output`; `process_spawn_wait_status`; `process_timeout_status`; `process_signal_status`; `process_async_run_output`; `process_async_run_timeout` | Status evidence distinguishes normal success, nonzero exit, timeout, and Unix signal termination through `success`, `code`, `kind`, `timed_out`, and `signal`. When `kind == "signal"`, `signal` carries the meaningful termination evidence and `code` is a platform sentinel. Cancellation status remains open for later lifecycle waves. |
| `sifr.process.Output` / `TextOutput` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_shell_exec_output`; `process_async_run_output` | Byte output captures stdout/stderr as `bytes`; text output requires an explicit encoding argument and currently accepts UTF-8/UTF8 through the text/i18n substrate boundary. Async byte output supports argv run/output loopback and explicitly rejects `Command.stdin_bytes(...)` until the owned pipe/communicate wave. Non-UTF-8 text-process policy remains open for the full M4 text-mode closeout. |
| `sifr.process.Stdio` constants | Public `PIPE`, `INHERIT`, `NULL` definitions | Constants reserve the production namespace for the later owned pipe/spawn wave. Pipe ownership APIs are not claimed complete by this foundation wave. |
| Sync `run`, `output`, `output_text` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_blocking_direct_async_rejected` | Sync process APIs are `@blocking_io`, return typed `Result[..., ProcessError]`, and direct async calls are rejected through imported stdlib workload metadata. |
| Async `async_run`, `async_run_timeout`, `async_output` | `process_async_run_output`; `process_async_run_timeout` | Async argv process APIs lower to `tokio::process::Command` on the current-thread runtime and are not marked `@blocking_io`. Status-only async run timeout validates timeout duration input, races a cancel-safe child wait against a Tokio sleep, kills and reaps the child on timeout, and returns typed timeout `Status` evidence. Async output timeout remains tied to the later owned pipe/communicate wave so stdout/stderr draining semantics stay explicit. Async spawn/wait/communicate, owned pipes, shell async APIs, cancellation, and scoped supervision remain later M4 work. |
| Sync `run_timeout`, `output_timeout` | `process_timeout_status` | Timeout APIs kill and reap timed-out children, return typed `Status` evidence instead of panicking, and reject invalid negative, non-finite, or out-of-range timeout values through `ProcessError`. |
| Sync `spawn`, `wait`, `Child.wait` | `process_spawn_wait_status`; `process_wait_direct_async_rejected`; `process_child_wait_method_direct_async_rejected` | Sync child lifecycle stores `std::process::Child` behind a private generated handle table. `wait(child)` and `Child.wait()` are one-shot observation paths; top-level `wait(child)` and method-form `Child.wait()` are `@blocking_io` and direct async calls are rejected. Owned pipe access, async wait, termination, timeout, and scoped supervision remain later M4 work. |
| Sync `kill`, `Child.kill` | `process_child_kill_wait`; `process_kill_direct_async_rejected`; `process_child_kill_method_direct_async_rejected` | Sync kill requests forceful immediate-child termination through `std::process::Child::kill`; it does not provide process-group or descendant supervision. Callers must still observe the final status with `wait`. Top-level `kill(child)` and method-form `Child.kill()` are `@blocking_io` and direct async calls are rejected. Graceful `terminate`, timeout escalation, structured cancellation, and non-Unix signal-status evidence remain later M4 work. |
| Sync `run_shell`, `output_shell`, `output_shell_text` | `process_shell_exec_output`; `process_shell_exec_direct_async_rejected` | Shell execution is explicit and classified as `@shell_exec` in addition to source-level `@blocking_io`; direct async calls use `SIFR-ASYNC-0007`. |
| Sync `output_shell_timeout` | `process_timeout_status`; `process_shell_timeout_direct_async_rejected` | Shell timeout execution preserves the explicit shell-exec effect and timeout status evidence. |
| Imported workload metadata | `process_blocking_direct_async_rejected`; `process_shell_exec_direct_async_rejected`; `process_child_wait_method_direct_async_rejected`; `process_child_kill_method_direct_async_rejected` | Lowering exports workload labels from stdlib/project modules and reimports them for user modules, including qualified class-method workload labels, so stdlib process APIs participate in the existing direct-async/offload diagnostic model. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative M4 fixtures |
| --- | --- | --- |
| `Lib/test/test_subprocess.py` sync argv, env, cwd, stdin/stdout/stderr, return-code behavior | `adapted-for-sifr-api` | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output` |
| `subprocess.getoutput` / `getstatusoutput` shell helpers | `unsupported-with-diagnostic` / rejected legacy helper shape | M0a legacy subprocess diagnostics; M4 exposes explicit `output_shell_text` instead of CPython helper compatibility. |
| `Lib/test/test_asyncio/test_subprocess.py` async process lifecycle | `adapted-for-sifr-api` / `planned-for-M4-follow-up` | `process_async_run_output` covers initial async argv run/output loopback and `process_async_run_timeout` covers status-only async process timeout. Async output timeout, async spawn/wait/communicate, async stdin/pipes, cancellation, and scoped supervision fixtures remain open. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_spawn_wait_status`, `process_timeout_status`, `process_signal_status`, `process_async_run_output`, `process_async_run_timeout`, `process_child_kill_wait` |
| Merge | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_spawn_wait_status`, `process_timeout_status`, `process_signal_status`, `process_async_run_output`, `process_async_run_timeout`, `process_child_kill_wait` |
| Fail suite | `process_blocking_direct_async_rejected`, `process_shell_exec_direct_async_rejected`, `process_wait_direct_async_rejected`, `process_child_wait_method_direct_async_rejected`, `process_shell_timeout_direct_async_rejected`, `process_kill_direct_async_rejected`, `process_child_kill_method_direct_async_rejected`, existing `legacy_sifr_subprocess_removed`, existing `async_popen_unsupported`, existing `bare_cpython_subprocess_import` |

## Follow-up Boundaries

Intentional remaining M4 work after this foundation wave:

- `PipeReader`, `PipeWriter`, owned stdout/stderr/stdin pipe lifecycle, double-close/use-after-close diagnostics, and handle sendability/shareability checks beyond the one-shot sync `Child.wait()` state.
- Native async spawn/wait/communicate, async stdin/owned pipes, async output timeout, and cancellation-safe process observation.
- Graceful `terminate`, termination escalation, non-Unix signal status evidence, parent cancellation evidence, and supported-host matrix updates for process termination behavior.
- Scoped process supervision entry point accepted by M0: `TaskGroup.spawn_process` returns a distinct `ProcessHandle` preserving pipe access.
- Full subprocess text mode closeout beyond UTF-8-only text output, consuming the text/i18n M1 evidence explicitly.
- Dropping an unwaited sync `Child` keeps the private `std::process::Child` table entry for the process lifetime and may leave host child reaping to a later lifecycle wave; termination, timeout, cancellation, and drop cleanup semantics remain M4 follow-up work.
- If a future stdlib module re-exports a workload-annotated callable, mirror project-module re-export workload metadata in stdlib bootstrap export collection before relying on that shape.
