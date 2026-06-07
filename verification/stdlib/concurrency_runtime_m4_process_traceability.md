# Concurrency Runtime M4 Process Traceability

Milestone: `milestone_concurrency_runtime_4`

Status: In progress; sync process foundation merged in PR #2331, sync child wait merged in PR #2334, and timeout status evidence merged in PR #2336.

## Production Surface Traceability

| Surface | M4 evidence | Notes |
| --- | --- | --- |
| `sifr.process.Command` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin` | Native argv command builder with ordered arguments, explicit env entries, cwd selection, and owned stdin byte payload capture. This is the production `sifr.process` path and does not use `sifr.subprocess` or legacy shell-shaped helpers. |
| `sifr.process.Status` | `process_sync_output_text`; `process_shell_exec_output`; `process_spawn_wait_status`; `process_timeout_status`; `process_child_kill_status` | Sync status evidence distinguishes normal success from nonzero exit through `success`, `code`, and `kind`. Child wait reuses the same status evidence for one-shot sync observation. Timeout status evidence sets `kind == "timeout"` and `timed_out == True`; forced child kill sets `kind == "signal"` and records signal evidence. Cancellation fields remain open for later lifecycle waves. |
| `sifr.process.Output` / `TextOutput` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_shell_exec_output` | Byte output captures stdout/stderr as `bytes`; text output requires an explicit encoding argument and currently accepts UTF-8/UTF8 through the text/i18n substrate boundary. Non-UTF-8 text-process policy remains open for the full M4 text-mode closeout. |
| `sifr.process.Stdio` constants | Public `PIPE`, `INHERIT`, `NULL` definitions | Constants reserve the production namespace for the later owned pipe/spawn wave. Pipe ownership APIs are not claimed complete by this foundation wave. |
| Sync `run`, `output`, `output_text` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_blocking_direct_async_rejected` | Sync process APIs are `@blocking_io`, return typed `Result[..., ProcessError]`, and direct async calls are rejected through imported stdlib workload metadata. |
| Sync `run_timeout`, `output_timeout` | `process_timeout_status` | Timeout APIs kill and reap timed-out children, return typed `Status` evidence instead of panicking, and reject invalid negative, non-finite, or out-of-range timeout values through `ProcessError`. |
| Sync `spawn`, `wait`, `Child.wait`, `kill`, `Child.kill` | `process_spawn_wait_status`; `process_child_kill_status`; `process_wait_direct_async_rejected`; `process_kill_direct_async_rejected` | Sync child lifecycle stores `std::process::Child` behind a private generated handle table. `wait(child)`, `Child.wait()`, `kill(child)`, and `Child.kill()` are one-shot observation paths; sync observation/termination APIs are `@blocking_io` and direct async calls are rejected. Owned pipe access, async wait, graceful terminate, timeout on spawned children, and scoped supervision remain later M4 work. |
| Sync `run_shell`, `output_shell`, `output_shell_text` | `process_shell_exec_output`; `process_shell_exec_direct_async_rejected` | Shell execution is explicit and classified as `@shell_exec` in addition to source-level `@blocking_io`; direct async calls use `SIFR-ASYNC-0007`. |
| Sync `output_shell_timeout` | `process_timeout_status`; `process_shell_timeout_direct_async_rejected` | Shell timeout execution preserves the explicit shell-exec effect and timeout status evidence. |
| Imported workload metadata | `process_blocking_direct_async_rejected`; `process_shell_exec_direct_async_rejected` | Lowering exports workload labels from stdlib/project modules and reimports them for user modules, so stdlib process APIs participate in the existing direct-async/offload diagnostic model. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative M4 fixtures |
| --- | --- | --- |
| `Lib/test/test_subprocess.py` sync argv, env, cwd, stdin/stdout/stderr, return-code behavior | `adapted-for-sifr-api` | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output` |
| `subprocess.getoutput` / `getstatusoutput` shell helpers | `unsupported-with-diagnostic` / rejected legacy helper shape | M0a legacy subprocess diagnostics; M4 exposes explicit `output_shell_text` instead of CPython helper compatibility. |
| `Lib/test/test_asyncio/test_subprocess.py` async process lifecycle | `planned-for-M4-follow-up` | Async spawn/wait/communicate and scoped supervision fixtures remain open. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_spawn_wait_status`, `process_timeout_status`, `process_child_kill_status` |
| Merge | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_spawn_wait_status`, `process_timeout_status`, `process_child_kill_status` |
| Fail suite | `process_blocking_direct_async_rejected`, `process_shell_exec_direct_async_rejected`, `process_wait_direct_async_rejected`, `process_kill_direct_async_rejected`, `process_shell_timeout_direct_async_rejected`, existing `legacy_sifr_subprocess_removed`, existing `async_popen_unsupported`, existing `bare_cpython_subprocess_import` |

## Follow-up Boundaries

Intentional remaining M4 work after this foundation wave:

- `PipeReader`, `PipeWriter`, owned stdout/stderr/stdin pipe lifecycle, double-close/use-after-close diagnostics, and handle sendability/shareability checks beyond the one-shot sync `Child.wait()` state.
- Native async spawn/wait/communicate and cancellation-safe process observation.
- Graceful `terminate`, parent cancellation evidence, and supported-host matrix updates for process termination behavior.
- Non-Unix forced-kill signal evidence currently uses the `-1` sentinel; richer host-specific termination representation remains a supported-host matrix follow-up.
- Scoped process supervision entry point accepted by M0: `TaskGroup.spawn_process` returns a distinct `ProcessHandle` preserving pipe access.
- Full subprocess text mode closeout beyond UTF-8-only text output, consuming the text/i18n M1 evidence explicitly.
- Dropping an unwaited sync `Child` keeps the private `std::process::Child` table entry for the process lifetime and may leave host child reaping to a later lifecycle wave; termination, timeout, cancellation, and drop cleanup semantics remain M4 follow-up work.
- Decide whether repeated `Command.stdin_bytes(...)` calls append or replace payload data when the spawn/pipe wave finalizes stdin ownership semantics.
- Delete the unused legacy `_sifr.sys.subprocess_*` intrinsic registry paths once no test or diagnostic still needs them as M4 cleanup.
- If a future stdlib module re-exports a workload-annotated callable, mirror project-module re-export workload metadata in stdlib bootstrap export collection before relying on that shape.
