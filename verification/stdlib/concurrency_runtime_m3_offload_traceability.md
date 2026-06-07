# Concurrency Runtime M3 Offload Traceability

Milestone: `milestone_concurrency_runtime_3`

Status: M3 implementation waves active.

## Production Surface Traceability

| Surface | M3 evidence | Notes |
| --- | --- | --- |
| `sifr.parallel.map` | `parallel_map_basic`; `parallel_map_worker_panic_typed`; `parallel_map_async_direct_rejected`; `parallel_map_non_send_item_rejected` | Owned homogeneous list input, ordered output, typed `WorkerRuntimeError` runtime boundary, direct async-call diagnostic, and non-send item rejection. |
| `sifr.parallel.try_map` | `parallel_try_map_basic`; `parallel_try_map_user_error_typed` | Ordered successful output and user worker errors wrapped in typed `WorkerError` evidence. |
| `sifr.parallel.Pool` / `PoolConfig` | `parallel_pool_map_basic` | Configured pools use private Rayon thread pools and do not configure Rayon's global pool. Pool construction failure is recorded on the pool and surfaced through the first map/try_map result rather than aborting. |
| `task.spawn_cpu` | `spawn_cpu_basic`; `spawn_cpu_user_error_typed`; `spawn_cpu_worker_panic_typed`; `spawn_cpu_unannotated_rejected`; `spawn_cpu_blocking_io_rejected`; `spawn_cpu_non_send_rejected` | Async CPU-heavy offload returns a linear blocking-task handle, requires `@cpu_heavy` sync workers, maps worker/runtime failures into `WorkerRuntimeError` or `WorkerError`, rejects blocking-I/O workers, and rejects non-send output/error types. |
| Generated dependency gating | `crates/sifr_stdlib/src/features.rs`; `crates/sifr_codegen/src/lib_modules_and_codegen.rs` | Importing `sifr.parallel` or using `task.spawn_cpu` adds `rayon = "1.12.0"` to generated projects only when Rayon-backed work is used. |
| Worker panic boundary | `parallel_map_worker_panic_typed`; `spawn_cpu_worker_panic_typed` | Rayon worker calls are wrapped in `catch_unwind` and converted into `WorkerRuntimeError`/`WorkerError` instead of propagating a process panic. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative M3 fixtures |
| --- | --- | --- |
| `Lib/test/test_concurrent_futures/` map ordering and worker failure behavior | `adapted-for-sifr-api` | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed` |
| `Executor.map`, `as_completed`, `ThreadPoolExecutor` public APIs | `rejected` / `unsupported-with-diagnostic` | M0a legacy-surface diagnostics; production APIs are `sifr.runtime`, `sifr.parallel`, and `JoinSet`. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed` |
| Merge | `parallel_map_basic`, `parallel_try_map_basic`, `parallel_pool_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, `spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed` |
| Fail suite | `parallel_map_async_direct_rejected`, `parallel_map_non_send_item_rejected`, `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`, `spawn_cpu_non_send_rejected` |

## Open Follow-up Boundaries

This first wave does not close all of M3. Remaining M3 work before milestone closure:

- Scoped CPU offload owner methods beyond the module-level affine `task.spawn_cpu` handle.
- `JoinSet[T, E]`, `JoinItemId`, ordered `join_all().await`, `cancel_all().await`, and live-set drop diagnostics.
- Full `WorkerError[E]` typing for homogeneous offload collections if the type system can preserve the user error parameter without nominal erasure.
- Closure capture sendability diagnostics beyond item/output/error sendability.
- OS thread creation failure handling for `task.spawn_cpu` if the per-call `std::thread::spawn` bridge is replaced during the `JoinSet` or lazy-pool work.
- Per-call panic-hook suppression that does not rely on global `std::panic::set_hook` state when independent OS threads call `sifr.parallel` concurrently.
- A lazy private default pool shutdown design for top-level `sifr.parallel` calls; the current first wave uses fresh private top-level pools to avoid global Rayon pool mutation and to keep shutdown deterministic.
