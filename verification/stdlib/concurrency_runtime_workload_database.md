# Concurrency Runtime Workload Database

Status: M7 inventory audited; implementation milestones have recorded validation evidence for accepted concurrency/runtime surfaces.

| API | Owner | Workload/effect classification | Validation |
| --- | --- | --- | --- |
| sifr.task.sleep | M1 | async-suspension | task sleep fixture |
| sifr.task.timeout/deadline | M1 | async-suspension cancellation | timeout/deadline evidence fixture |
| sifr.task.cancel_scope | M1 | async-suspension cancellation | cancel-scope fixture |
| sifr.sync.Channel.send/receive async forms | M2 | async-suspension backpressure | channel backpressure and cancellation fixtures |
| sifr.sync.Channel send/receive sync forms | M2 | @blocking_io-equivalent sync wait | blocking-in-async diagnostic fixture |
| sifr.sync.Mutex/RwLock sync lock | M2 | @blocking_io-equivalent sync wait | lock direct async diagnostic fixture |
| sifr.sync.AsyncMutex/AsyncRwLock/Semaphore/Event | M2 | async-suspension | async sync primitive fixtures |
| sifr.runtime.spawn_blocking | M3 | @blocking_io offload boundary | spawn_blocking typed WorkerError fixture |
| sifr.task.spawn_cpu | M3 | @cpu_heavy offload boundary with typed runtime/worker evidence | `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`, `spawn_cpu_non_send_rejected` |
| sifr.task.TaskScope/TaskGroup scoped offload | M3 | @blocking_io/@cpu_heavy scoped owner offload with typed task evidence | `task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `task_scope_spawn_cpu_unannotated_rejected`, `task_group_spawn_blocking_error_mismatch_rejected` |
| sifr.task.JoinSet | M3 | homogeneous task/offload collection with explicit observation/cancellation | `join_set_add_task_join_all`, `join_set_spawn_cpu_join_all_ordered`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await`, `join_set_reassign_live_rejected`, `join_set_unconsumed_rejected`, `join_set_terminal_must_be_awaited_rejected` |
| sifr.parallel.map/try_map | M3 | @cpu_heavy synchronous, typed worker-runtime boundary | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, async direct-call diagnostic fixture |
| sifr.process.run/output/wait sync | M4 | @blocking_io plus optional @shell_exec | process blocking-in-async and shell-effect fixtures |
| sifr.process async spawn/wait/communicate | M4 | async-suspension plus optional @shell_exec | async process loopback fixture |
| sifr.signal.shutdown_stream/ctrl_c/terminate | M5 | async-suspension host-limited | signal host matrix fixture |
| sifr.resource.AsyncExitStack | M5 | async cleanup under cancellation | async cleanup cancellation fixture |
| sifr.ipc.Connection send/receive | M6 | async-suspension backpressure serialization | IPC frame/malformed/cancel fixtures |

## Rules

Sync APIs that can wait on channels, locks, processes, pipes, or external runtime state are classified as blocking and remain invalid in `async def` unless explicitly offloaded. CPU-heavy APIs use `@cpu_heavy` and must route through `spawn_cpu` in async contexts. Shell subprocess APIs carry `@shell_exec` in addition to blocking or async suspension classification.
