I've completed the implementation-readiness review. Here are the findings:

## Implementation-Readiness Review: M3 task.spawn_cpu Wave

### Verification against the five contract items

**1. Validation gate (async-only, named zero-arg sync worker, @cpu_heavy, rejects others)** — `crates/sifr_lowering/src/lower/task_calls.rs:97-153`
- Async-only enforced at L98-104 (SIFR-TYPE-0002).
- Keyword args rejected at L106-112; arity exactly 1 enforced at L114-120.
- Function-type and zero-params enforced at L127-145.
- `@cpu_heavy` classification enforced via `reject_offload_target_without_kind(... WorkloadKind::CpuHeavy)` at L146-153 — requires a `Name` target (rejects lambdas), then matches the annotation against `WorkloadKind::CpuHeavy` (SIFR-ASYNC-0005 for missing / mismatched workload, including `@blocking_io`).

**2. Affine `BlockingTask` shape with WorkerRuntimeError / WorkerError typing** — `task_calls.rs:155-198`
- Infallible workers route to `__sifr_spawn_cpu_infallible` and produce `BlockingTask[T, WorkerRuntimeError]`.
- Fallible workers route to `__sifr_spawn_cpu_result` and produce `BlockingTask[T, WorkerError]`.
- `BlockingTask` is the existing affine handle struct from `task_runtime.rs`; `Await` on it lowers to `.join().await` (`leaves_and_plain_calls.rs:437-449`).
- Matches `issues/...-substrate.md:580-581` and traceability table.

**3. Typed worker / pool failure evidence; no user-triggerable abort/panic/unwrap/expect** — `crates/sifr_codegen/src/preamble/cpu_offload_runtime.rs`
- Rayon pool construction failure becomes `WorkerRuntimeError::new("cpu worker pool could not start: ...")` / `WorkerError::new(...)` — no `unwrap`/`expect`/`abort` on the failure leg.
- User worker panic is caught by `std::panic::catch_unwind(AssertUnwindSafe(work))` and converted to typed evidence.
- The remaining `panic::resume_unwind` inside `__sifr_with_silent_cpu_panic_hook` is unreachable in practice: the wrapped body is `match catch_unwind(work) { … }` whose tail expressions are struct constructions (`WorkerRuntimeError::new`, `__SifrFailure::new`, `__SifrTaskResult::Ok/Err`) that cannot panic. The well-known global panic-hook race is explicitly carried as a remaining-work item, not a regression introduced by this wave.

**4. Dependency feature gating** — `crates/sifr_codegen/src/lib_modules_and_codegen.rs`
- `uses_spawn_cpu = module_uses_spawn_cpu(module)` (`lib_runtime_needs.rs:717-769`) drives `referenced_error_classes` to include both `WorkerRuntimeError`/`WorkerError` (L427-430), invokes `build_cpu_offload_items()` (L568-570), and adds `StdlibFeature::Rayon` (L740-742).
- Tokio is reached because `module_uses_task_scope` now includes `__sifr_spawn_cpu_infallible/_result` (`lib_runtime_needs.rs:653`), and the feature gate also picks up `tokio::` in the preamble (L730-735).
- Non-spawn_cpu modules: `module_uses_spawn_cpu` returns false and the stdlib `features_for_stdlib_module` only adds Rayon for `sifr.parallel`, so projects that neither use `task.spawn_cpu` nor import `sifr.parallel` do not pull Rayon from this path.

**5. Sendability + honest traceability** — `task_calls.rs:163-186`
- `non_send_reason(&ok_ty)` rejects non-send output types.
- `non_send_reason(&err_ty)` rejects non-send error types when `err_ty != Never`.
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md` "Open Follow-up Boundaries" enumerates scoped CPU offload owner methods, `JoinSet`/`JoinItemId`/ordering, generic `WorkerError[E]`, closure-capture sendability, global panic-hook follow-up, and lazy default-pool shutdown design.

### Fixtures and manifests
- Pass fixtures (`spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`) and fail fixtures (`spawn_cpu_unannotated_rejected` → SIFR-ASYNC-0005, `spawn_cpu_blocking_io_rejected` → SIFR-ASYNC-0005, `spawn_cpu_non_send_rejected` → SIFR-TYPE-0002) are wired into both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` and match the diagnostic-code expectations.
- `WorkerRuntimeError`/`WorkerError` are listed in `BUILTIN_ERROR_CLASSES`, emitted via `build_error_type_items` with `Display`/`Error` impls so the `E: Display` bound on `__sifr_spawn_cpu_result` is satisfied. User error subclasses receive Display via `class_emitter.rs:385-393`.

### Findings

No BLOCKER findings.

### Non-blocking observations (not blockers; flagged for the record)
- `__sifr_with_silent_cpu_panic_hook` uses process-global `panic::take_hook`/`set_hook`; concurrent spawn_cpu work from independent OS threads can race the hook state. The traceability doc records this as a remaining M3 follow-up.
- `std::thread::spawn` panics if the OS refuses thread creation; this is consistent with the existing `sifr.parallel` thread-creation surface (Rayon's pool build returns `Result` so that surface is handled, while `tokio::spawn`/`std::thread::spawn` retain stdlib panic semantics on infrastructure failure). Not specific to this wave.
- The CPU offload runtime functions are emitted as a single multi-line raw string inside `RustExpr::Ident`, bypassing the typed IR validation — a stylistic precedent shared with `parallel_runtime_rust_code()`, not a defect.

RESULT: PASS
