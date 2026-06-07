# Concurrency Runtime Workload Database

Status: M3 active; implementation milestones update validation evidence as APIs land.

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
| sifr.runtime.spawn_cpu | M3 | @cpu_heavy offload boundary | spawn_cpu typed WorkerError fixture |
| sifr.parallel.map/try_map | M3 | @cpu_heavy synchronous, typed worker-runtime boundary | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, async direct-call diagnostic fixture |
| sifr.process.run/output/wait sync | M4 | @blocking_io plus optional @shell_exec | process blocking-in-async and shell-effect fixtures |
| sifr.process async spawn/wait/communicate | M4 | async-suspension plus optional @shell_exec | async process loopback fixture |
| sifr.signal.shutdown_stream/ctrl_c/terminate | M5 | async-suspension host-limited | signal host matrix fixture |
| sifr.resource.AsyncExitStack | M5 | async cleanup under cancellation | async cleanup cancellation fixture |
| sifr.ipc.Connection send/receive | M6 | async-suspension backpressure serialization | IPC frame/malformed/cancel fixtures |

## Rules

Sync APIs that can wait on channels, locks, processes, pipes, or external runtime state are classified as blocking and remain invalid in `async def` unless explicitly offloaded. CPU-heavy APIs use `@cpu_heavy` and must route through `spawn_cpu` in async contexts. Shell subprocess APIs carry `@shell_exec` in addition to blocking or async suspension classification.
