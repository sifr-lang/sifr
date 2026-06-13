Findings:

**Correctness — clean**
- `class_type_collection.rs:649-655`: class-method workload collection is gated by the same decorator pass as top-level functions, producing qualified `Child.wait` / `Child.kill` keys in `ctx.function_workload_annotations`. The qualified keys flow into `LoweringResult.function_workloads` via `mod_impl.rs:755`.
- `workload_annotations.rs:119-130`: `reject_async_direct_method_call` early-returns unless `object_ty` is `Type::Class` or `Type::Protocol`, then delegates to `reject_async_direct_call` which gates on `current_function_is_async` and explicit workload presence. Non-annotated methods and sync contexts are no-ops.
- `methods_lambdas_and_comprehensions.rs:126-131`: the new check runs after `parallel_calls`, `task_join_set_calls`, `task_scope_offload_calls`, `task_scope_spawn`, and `is_task_handle_type` paths, so async-wrapper method handling is not affected.

**Import propagation — clean**
- `bootstrap.rs:107-114` filters by `should_export_callable(module_name, owner_name)`, correctly preserving the `_Child` exclusion semantics.
- `imports.rs:115-124` and `mod_impl.rs:452-461`/`649-658` only call `import_class_method_workloads` inside the `module_classes.get(name)` arm, so workload propagation is anchored to actual class imports.
- `imported_defaults.rs:79-97`: dual-inserts under both `{local_name}.{suffix}` and `{external_name}.{suffix}`. The external-name insert is what makes the diagnostic fire after `from sifr.process import Child` (since `Type::Class { name }` carries the external `"Child"`). This is intentional — `import_class_method_varargs`/`defaults` would actually have been broken for aliased class imports for the same reason, but that's pre-existing behavior not touched here.

**Tests — clean**
- Both fail fixtures produce `SIFR-ASYNC-0003` exactly at the method name token (`child.wait()` line 6 col 20; `child.kill()` line 6 col 15).
- Pass fixtures `process_spawn_wait_status.sifr` and `process_child_kill_wait.sifr` (both call these methods inside `def main()`) still type-check clean — no false-positive regression.
- Stdlib grep confirms only `Child.wait` and `Child.kill` are class-method-annotated in `lib/sifr/`, so the diagnostic surface is bounded.

**Doc honesty — clean**
- `concurrency_runtime_m4_process_traceability.md` removes the method-form follow-up bullet, adds both new fixtures to the fail-suite row, and threads "method-form `Child.wait()`/`Child.kill()` are `@blocking_io` and direct async calls are rejected" through the relevant traceability rows.
- Does not claim owned pipes, async spawn/wait/communicate, terminate/escalation, scoped supervision, or text-mode closeout — those remain explicitly listed as follow-up work.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` adds an honest implementation + validation block scoped to method-form diagnostics.

**One non-blocker note (operational, not correctness)**
- Working tree also contains unrelated edits: `issues/ad-hoc-production-network-http-platform-substrate-execution.md`, `issues/ad-hoc-production-network-http-platform-substrate.md`, and two untracked `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-{1,2}.md` files. These are clearly from a parallel HTTP-substrate slice and must not be committed with this slice's PR. Not a correctness or regression issue for the M4 process diagnostics — just a staging reminder.

RESULT: PASS
