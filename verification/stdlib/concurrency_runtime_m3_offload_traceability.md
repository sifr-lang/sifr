# Concurrency Runtime M3 Offload Traceability

Milestone: `milestone_concurrency_runtime_3`

Status: M3 implementation waves active; scoped owner offload wave merged in PR #2323.

## Production Surface Traceability

| Surface | M3 evidence | Notes |
| --- | --- | --- |
| `sifr.parallel.map` | `parallel_map_basic`; `parallel_map_worker_panic_typed`; `parallel_map_async_direct_rejected`; `parallel_map_non_send_item_rejected` | Owned homogeneous list input, ordered output, typed `WorkerRuntimeError` runtime boundary, direct async-call diagnostic, and non-send item rejection. |
| `sifr.parallel.try_map` | `parallel_try_map_basic`; `parallel_try_map_user_error_typed` | Ordered successful output and user worker errors wrapped in typed `WorkerError` evidence. |
| `sifr.parallel.Pool` / `PoolConfig` | `parallel_pool_map_basic` | Configured pools use private Rayon thread pools and do not configure Rayon's global pool. Pool construction failure is recorded on the pool and surfaced through the first map/try_map result rather than aborting. |
| `task.spawn_cpu` | `spawn_cpu_basic`; `spawn_cpu_user_error_typed`; `spawn_cpu_worker_panic_typed`; `spawn_cpu_unannotated_rejected`; `spawn_cpu_blocking_io_rejected`; `spawn_cpu_non_send_rejected` | Async CPU-heavy offload returns a linear blocking-task handle, requires `@cpu_heavy` sync workers, maps worker/runtime failures into `WorkerRuntimeError` or `WorkerError`, rejects blocking-I/O workers, and rejects non-send output/error types. |
| `TaskScope`/`TaskGroup` scoped offload | `task_scope_spawn_blocking`; `task_group_spawn_cpu`; `task_group_spawn_cpu_user_error`; `task_scope_spawn_cpu_unannotated_rejected`; `task_group_spawn_blocking_error_mismatch_rejected` | Scoped owner offload methods return scoped `Task[T, E]` handles so owner exit can observe/cancel the child while callers retain affine observation. `TaskGroup` enforces the same open-state and error-homogeneity checks as async child spawn; scoped CPU offload maps runtime/user failures into `WorkerRuntimeError`/`WorkerError` and keeps Rayon gated on scoped CPU use. |
| `task.JoinSet[T, E]` | `join_set_add_task_join_all`; `join_set_spawn_cpu_join_all_ordered`; `join_set_cancel_all_evidence`; `join_set_cancel_all_task_cancelled`; `join_set_spawn_blocking`; `join_set_bound_terminal_await`; `join_set_add_type_mismatch_rejected`; `join_set_spawn_cpu_worker_error_required`; `join_set_reassign_live_rejected`; `join_set_unconsumed_rejected`; `join_set_terminal_must_be_awaited_rejected` | Dynamically-growable homogeneous task/offload collection with opaque `JoinItemId`, submission-order `join_all().await`, submission-order `cancel_all().await`, handle-consuming `add`, bound terminal awaitables that are valid when later awaited, and compile-time diagnostics for type mismatch, CPU worker-error boundary mismatch, reassigning/dropping live sets, and creating terminal awaitables without awaiting them. This wave returns `list[TaskResult[T, E]]` from `join_all()` to align with existing task observation evidence; `spawn_cpu` requires `JoinSet[T, WorkerError]` until generic `WorkerError[E]` lands. |
| Generated dependency gating | `crates/sifr_stdlib/src/features.rs`; `crates/sifr_codegen/src/lib_modules_and_codegen.rs` | Importing `sifr.parallel` or using `task.spawn_cpu` adds `rayon = "1.12.0"` to generated projects only when Rayon-backed work is used. |
| Worker panic boundary | `parallel_map_worker_panic_typed`; `spawn_cpu_worker_panic_typed` | Rayon worker calls are wrapped in `catch_unwind` and converted into `WorkerRuntimeError`/`WorkerError` instead of propagating a process panic. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative M3 fixtures |
| --- | --- | --- |
| `Lib/test/test_concurrent_futures/` map ordering and worker failure behavior | `adapted-for-sifr-api` | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `join_set_spawn_cpu_join_all_ordered`, `join_set_add_task_join_all`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await` |
| `Executor.map`, `as_completed`, `ThreadPoolExecutor` public APIs | `rejected` / `unsupported-with-diagnostic` | M0a legacy-surface diagnostics; production APIs are `sifr.runtime`, `sifr.parallel`, and `JoinSet`. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `join_set_spawn_cpu_join_all_ordered`, `join_set_add_task_join_all`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await` |
| Merge | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `join_set_spawn_cpu_join_all_ordered`, `join_set_add_task_join_all`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await` |
| Fail suite | `parallel_map_async_direct_rejected`, `parallel_map_non_send_item_rejected`, `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`, `spawn_cpu_non_send_rejected`, `task_scope_spawn_cpu_unannotated_rejected`, `task_group_spawn_blocking_error_mismatch_rejected`, `join_set_add_type_mismatch_rejected`, `join_set_spawn_cpu_worker_error_required`, `join_set_reassign_live_rejected`, `join_set_unconsumed_rejected`, `join_set_terminal_must_be_awaited_rejected` |

## Open Follow-up Boundaries

Remaining M3 work before milestone closure:

- Full `WorkerError[E]` typing for homogeneous offload collections if the type system can preserve the user error parameter without nominal erasure.
- Closure capture sendability diagnostics beyond item/output/error sendability.
- OS thread creation failure handling for `task.spawn_cpu` if the per-call `std::thread::spawn` bridge is replaced during the `JoinSet` or lazy-pool work.
- Per-call panic-hook suppression that does not rely on global `std::panic::set_hook` state when independent OS threads call `sifr.parallel` concurrently.
- A lazy private default pool shutdown design for top-level `sifr.parallel` calls; the current first wave uses fresh private top-level pools to avoid global Rayon pool mutation and to keep shutdown deterministic.
