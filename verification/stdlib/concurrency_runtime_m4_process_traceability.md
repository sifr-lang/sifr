# Concurrency Runtime M4 Process Traceability

Milestone: `milestone_concurrency_runtime_4`

Status: In progress; sync process foundation wave reviewed and merged in PR #2331.

## Production Surface Traceability

| Surface | M4 evidence | Notes |
| --- | --- | --- |
| `sifr.process.Command` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin` | Native argv command builder with ordered arguments, explicit env entries, cwd selection, and owned stdin byte payload capture. This is the production `sifr.process` path and does not use `sifr.subprocess` or legacy shell-shaped helpers. |
| `sifr.process.Status` | `process_sync_output_text`; `process_shell_exec_output` | Sync status evidence distinguishes normal success from nonzero exit through `success`, `code`, and `kind`. Signal, timeout, and cancellation fields are present on the status object but remain unimplemented until the process lifecycle/cancellation waves. |
| `sifr.process.Output` / `TextOutput` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_shell_exec_output` | Byte output captures stdout/stderr as `bytes`; text output requires an explicit encoding argument and currently accepts UTF-8/UTF8 through the text/i18n substrate boundary. Non-UTF-8 text-process policy remains open for the full M4 text-mode closeout. |
| `sifr.process.Stdio` constants | Public `PIPE`, `INHERIT`, `NULL` definitions | Constants reserve the production namespace for the later owned pipe/spawn wave. Pipe ownership APIs are not claimed complete by this foundation wave. |
| Sync `run`, `output`, `output_text` | `process_sync_output_text`; `process_sync_bytes_env_cwd_stdin`; `process_blocking_direct_async_rejected` | Sync process APIs are `@blocking_io`, return typed `Result[..., ProcessError]`, and direct async calls are rejected through imported stdlib workload metadata. |
| Sync `run_shell`, `output_shell`, `output_shell_text` | `process_shell_exec_output`; `process_shell_exec_direct_async_rejected` | Shell execution is explicit and classified as `@shell_exec` in addition to source-level `@blocking_io`; direct async calls use `SIFR-ASYNC-0007`. |
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
| Create PR | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output` |
| Merge | `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output` |
| Fail suite | `process_blocking_direct_async_rejected`, `process_shell_exec_direct_async_rejected`, existing `legacy_sifr_subprocess_removed`, existing `async_popen_unsupported`, existing `bare_cpython_subprocess_import` |

## Follow-up Boundaries

Intentional remaining M4 work after this foundation wave:

- Production `spawn`, `Child`, `wait`, `PipeReader`, `PipeWriter`, owned stdout/stderr/stdin pipe lifecycle, double-close/use-after-close diagnostics, and handle sendability/shareability checks.
- Native async spawn/wait/communicate and cancellation-safe process observation.
- Timeout handling plus `terminate`, `kill`, signal termination evidence, parent cancellation evidence, and supported-host matrix updates for process termination behavior.
- Scoped process supervision entry point accepted by M0: `TaskGroup.spawn_process` returns a distinct `ProcessHandle` preserving pipe access.
- Full subprocess text mode closeout beyond UTF-8-only text output, consuming the text/i18n M1 evidence explicitly.
- Decide whether repeated `Command.stdin_bytes(...)` calls append or replace payload data when the spawn/pipe wave finalizes stdin ownership semantics.
- Delete the unused legacy `_sifr.sys.subprocess_*` intrinsic registry paths once no test or diagnostic still needs them as M4 cleanup.
- If a future stdlib module re-exports a workload-annotated callable, mirror project-module re-export workload metadata in stdlib bootstrap export collection before relying on that shape.
