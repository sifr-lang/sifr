# M3 Scoped Owner Offload — Review Pass 1

Scope: working-tree changes for the M3 scoped-owner-offload wave only (network HTTP phase
files and reviews intentionally excluded).

## Verifications independently re-run

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_scope_spawn_blocking.sifr` → cache hit, exit 0.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr` → cache hit, exit 0.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_group_spawn_cpu_user_error.sifr` → cache hit, exit 0.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_scope_spawn_cpu_unannotated_rejected.sifr` → `error[SIFR-ASYNC-0005]` as expected, anchored at `compute_value`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_group_spawn_blocking_error_mismatch_rejected.sifr` → `error[SIFR-TYPE-0002]` as expected, expected `ValueError`, got `IOError`.
- `cargo run -q -p sifr -- build` for `task_scope_spawn_blocking.sifr` → produces `sifr_output/` whose `Cargo.toml` contains only `tokio`; no `rayon` dependency (gating confirmed).
- `cargo run -q -p sifr -- build` for `task_group_spawn_cpu.sifr` → `rayon = "1.12.0"` present in generated Cargo.toml; emitted Rust contains `__sifr_with_silent_scope_cpu_panic_hook`, `__sifr_scope_spawn_cpu_infallible`, and Rayon `ThreadPoolBuilder` invocation. Final binary runs and asserts hold.

## Soundness / correctness assessment

- Lowering (`crates/sifr_lowering/src/lower/task_scope_offload_calls.rs:14-103`) reuses `is_task_scope_type`/`is_task_group_type`, `validate_sync_worker`, `validate_sendable_result`, and `enforce_task_group_is_open` / `enforce_task_group_error_type`. The async-context guard, zero-arg worker requirement, send-safety check, and group homogeneity check are all enforced. Group homogeneity for `spawn_blocking` registers the user-provided `E`; for `spawn_cpu` it registers `WorkerError` / `WorkerRuntimeError`, which is consistent with the public boundary erasure for CPU work.
- Workload classification is correctly differentiated:
  - `spawn_blocking` uses `reject_unclassified_offload_target` (accepts any classification, matching `task.spawn_blocking` at `task_calls.rs:261-267`).
  - `spawn_cpu` uses `reject_offload_target_without_kind(..., WorkloadKind::CpuHeavy)` (strict `@cpu_heavy`).
- Observation invariant: the four new methods are added to `task_group_spawn_owner` (`task_scope_calls.rs:432-440`). Let-bindings produced by `group.spawn_blocking`/`group.spawn_cpu` therefore register the group as owner so that subsequent observation flips `task_groups_not_proven_open`, mirroring the existing async-spawn observation behavior. `expressions/methods_lambdas_and_comprehensions.rs:109-116` dispatches the offload-call lowering before generic method resolution, so it shadows generic method calls cleanly.
- Codegen runtime (`crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs`):
  - Blocking variant uses a `tokio::sync::oneshot` channel + a tokio `spawn_blocking` join handle wrapped in `__SifrScopeChild`. The closure builds the `__SifrTaskResult`, sends it through the oneshot, and returns the matching `__SifrScopeChildOutcome` so `__sifr_join_all` can detect unobserved failure/cancel.
  - CPU variant additionally builds a per-call Rayon pool inside the spawn_blocking thread and wraps the worker in `__sifr_with_silent_scope_cpu_panic_hook` + `catch_unwind`, converting panics into `WorkerError` / `WorkerRuntimeError`. Pool construction failure is converted into a typed failure rather than aborting.
  - The returned `__SifrTask` reuses the existing affine-task struct (oneshot receiver, abort handle, observed flag), so `join`, `cancel`, `cancel_and_join`, and `__sifr_timeout` flow unchanged. This matches the wave's stated intent of returning scoped `Task[T, E]` rather than module-level `BlockingTask`.
- Dependency gating (`lib_modules_and_codegen.rs:413-414, 434-437, 575-580, 760-765` and `entrypoints.rs:73-78`):
  - `module_uses_task_scope_offload` → emits `build_task_scope_offload_items` (blocking-only impl).
  - `module_uses_task_scope_spawn_cpu` → additionally emits `build_task_scope_cpu_offload_items` and adds `WorkerRuntimeError`/`WorkerError` to referenced classes and `Rayon` to required features. Confirmed empirically that scoped-blocking-only emit pulls in no Rayon symbols and no Rayon Cargo entry.
- Fixtures and manifests: traceability (`verification/stdlib/concurrency_runtime_m3_offload_traceability.md:15, 31-33`) and workload database (`concurrency_runtime_workload_database.md:16`) both name the five new fixtures consistently, and the create-pr / merge manifests include the three new pass fixtures. The two fail fixtures live in the e2e fail suite and are exercised by the standard `--check` path.

## Non-blocking observations

These do not block this wave but are worth recording for follow-ups:

1. Diagnostic prefix is hard-coded to `scope.spawn_blocking()` / `scope.spawn_cpu()` even when the receiver is a `TaskGroup` (`task_scope_offload_calls.rs:35, 39, 58, 69, 73, 95`). Async-spawn already threads a `callable_name` parameter for the `task.spawn_scoped()` vs `scope.spawn()` case; doing the same here would let `group.spawn_blocking(...)` errors say `group.spawn_blocking()`.
2. `module_uses_task_scope_offload` is a superset of `module_uses_task_scope_spawn_cpu`, so when a module uses only `scope.spawn_cpu`, the blocking impl block is emitted but unreferenced. rustc's `dead_code` heuristic does not flag it (other methods on `__SifrTaskScope` are used in the same module), so this is silent waste rather than a warning. Could be split into `module_uses_task_scope_spawn_blocking` + `module_uses_task_scope_spawn_cpu` for clarity, mirroring the JoinSet split.
3. `lower_scope_spawn_blocking`/`lower_scope_spawn_cpu` re-pattern-match `Type::Function` after `validate_sync_worker` has already verified the worker shape (`task_scope_offload_calls.rs:43-45, 78-80`). The fallthrough `return None;` is unreachable in practice but silently emits no diagnostic if it ever did fire. Returning the `FunctionType` from `validate_sync_worker` would remove the duplication.
4. Fixture coverage is asymmetric: pass fixtures exist for `scope.spawn_blocking` and `group.spawn_cpu` (twice), but not `group.spawn_blocking` or `scope.spawn_cpu`. The lowering paths are shared and indirectly exercised by the failure fixtures, so this is not a soundness gap — but a future wave could add the symmetric pass fixtures alongside non-send / send-error rejection fixtures (parallel to the existing `spawn_cpu_non_send_rejected`).
5. `tokio::task::spawn_blocking` cannot be cancelled mid-flight, so `Task::cancel()` / `cancel_and_join()` on scoped blocking/cpu handles only signal abort and then await the underlying worker. This matches the existing module-level `task.spawn_blocking` and `task.spawn_cpu` semantics, so it is consistent rather than new — worth documenting in user-facing docs at some point.

## Documentation honesty

- The wave description in `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:600` and the traceability row accurately describe what was implemented: scoped owner offload returns scoped `Task[T, E]`, the TaskGroup open-state and error-homogeneity checks are reused, and CPU offload maps runtime/user failures into `WorkerRuntimeError` / `WorkerError`. The Rayon gating claim matches the implementation.
- The execution doc records the same targeted local-validation commands I re-ran, all matching the observed results.

---

RESULT: PASS

Rationale: lowering and codegen are correct, the affine-task observation invariant is preserved across the new offload methods, Rayon/WorkerError gating is structurally enforced, scoped-blocking-only emit cleanly omits CPU/Rayon items, the five new fixtures (three pass, two fail) all behave as advertised, and the docs/manifests honestly reflect what landed. The non-blocking observations above are quality-of-implementation polish, not soundness or correctness blockers.
