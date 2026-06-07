# Concurrency Runtime M4 Process Traceability

Milestone: `milestone_concurrency_runtime_4`

Status: In progress; first sync native-process wave.

## Production Surface Traceability

| Surface | M4 evidence | Notes |
| --- | --- | --- |
| `sifr.process.Command` | `process_sync_output_status` | Native process command object records program, argv, optional current directory, and whether shell execution is explicit. |
| `sifr.process.output` | `process_sync_output_status` | Sync process output returns typed `Result[Output, ProcessError]`; `Output.stdout` and `Output.stderr` are binary `bytes`, and `Output.status` wraps the exit status. |
| `sifr.process.run` | `process_sync_output_status` | Sync process run returns typed `Result[Status, ProcessError]` without throwing host subprocess failures as panics. |
| `sifr.process.shell` | `process_sync_output_status` | Shell execution requires an explicit constructor and does not reintroduce `sifr.subprocess` compatibility. |
| `sifr.subprocess` and bare `subprocess` | `legacy_sifr_subprocess_removed`; `bare_cpython_subprocess_import` | Legacy CPython-shaped process modules remain rejected or unsupported with diagnostics. |

## Follow-up Boundaries

Intentional remaining M4 work:

- Async process spawn/wait/communicate.
- Owned `Child`, `PipeReader`, and `PipeWriter` resources.
- Timeout, termination, kill, structured cancellation, and scoped process supervision.
- Environment override support beyond inherited process environment.
- Text-mode subprocess integration through text/i18n APIs.
- Shell-effect diagnostics integrated with async/offload validation.
- Cross-platform shell argument semantics: POSIX `sh -c` treats forwarded args as shell positional parameters, while Windows `cmd /C` appends command arguments differently. Later M4 docs/diagnostics must define or reject `shell(...).arg(...)` portability before shell APIs are considered closed.
- Host-matrix evidence for signal/termination behavior.
