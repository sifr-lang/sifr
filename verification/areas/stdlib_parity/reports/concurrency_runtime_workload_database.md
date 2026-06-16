# Concurrency Runtime Workload Database

Status: concurrency runtime inventory audited; implementation contracts have recorded validation evidence for accepted concurrency/runtime surfaces.

| API | Owner contract | Workload/effect classification | Validation |
| --- | --- | --- | --- |
| sifr.task.sleep | concurrency_runtime_structured_tasks | async-suspension | task sleep fixture |
| sifr.task.timeout/deadline | concurrency_runtime_structured_tasks | async-suspension cancellation | timeout/deadline evidence fixture |
| sifr.task.cancel_scope | concurrency_runtime_structured_tasks | async-suspension cancellation | cancel-scope fixture |
| sifr.sync.Channel.send/receive async forms | concurrency_runtime_sync_primitives | async-suspension backpressure | channel backpressure and cancellation fixtures |
| sifr.sync.Channel send/receive sync forms | concurrency_runtime_sync_primitives | @blocking_io-equivalent sync wait | blocking-in-async diagnostic fixture |
| sifr.sync.Mutex/RwLock sync lock | concurrency_runtime_sync_primitives | @blocking_io-equivalent sync wait | lock direct async diagnostic fixture |
| sifr.sync.AsyncMutex/AsyncRwLock/Semaphore/Event | concurrency_runtime_sync_primitives | async-suspension | async sync primitive fixtures |
| sifr.runtime.spawn_blocking | concurrency_runtime_offload | @blocking_io offload boundary | spawn_blocking typed WorkerError fixture |
| sifr.task.spawn_cpu | concurrency_runtime_offload | @cpu_heavy offload boundary with typed runtime/worker evidence | `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`, `spawn_cpu_non_send_rejected` |
| sifr.task.TaskScope/TaskGroup scoped offload | concurrency_runtime_offload | @blocking_io/@cpu_heavy scoped owner offload with typed task evidence | `task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `task_scope_spawn_cpu_unannotated_rejected`, `task_group_spawn_blocking_error_mismatch_rejected` |
| sifr.task.JoinSet | concurrency_runtime_offload | homogeneous task/offload collection with explicit observation/cancellation | `join_set_add_task_join_all`, `join_set_spawn_cpu_join_all_ordered`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await`, `join_set_reassign_live_rejected`, `join_set_unconsumed_rejected`, `join_set_terminal_must_be_awaited_rejected` |
| sifr.parallel.map/try_map | concurrency_runtime_offload | @cpu_heavy synchronous, typed worker-runtime boundary | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, async direct-call diagnostic fixture |
| sifr.process.run/output/wait sync | concurrency_runtime_process | @blocking_io plus optional @shell_exec | process blocking-in-async and shell-effect fixtures |
| sifr.process async spawn/wait/communicate | concurrency_runtime_process | async-suspension plus optional @shell_exec | async process loopback fixture |
| sifr.signal.shutdown_stream/ctrl_c/terminate | concurrency_runtime_shutdown | async-suspension host-limited | signal host matrix fixture |
| sifr.resource.AsyncExitStack | concurrency_runtime_shutdown | async cleanup under cancellation | async cleanup cancellation fixture |
| sifr.ipc.Connection send/receive | concurrency_runtime_typed_ipc | async-suspension backpressure serialization | IPC frame/malformed/cancel fixtures |

## Rules

Sync APIs that can wait on channels, locks, processes, pipes, or external runtime state are classified as blocking and remain invalid in `async def` unless explicitly offloaded. CPU-heavy APIs use `@cpu_heavy` and must route through `spawn_cpu` in async contexts. Shell subprocess APIs carry `@shell_exec` in addition to blocking or async suspension classification.
