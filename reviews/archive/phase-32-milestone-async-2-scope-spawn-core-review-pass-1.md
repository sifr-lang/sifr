# Phase 32 milestone_async_2 scope.spawn core review — pass 1

## Verdict: SATISFIED

## Changes reviewed

The staged diff on `phase32-scope-spawn-core` adds conservative `scope.spawn(coro)` lowering for `TaskScope` values:

| File | Role |
|---|---|
| `sifr_hir/src/lower/task_scope_calls.rs` | New HIR module: `lower_task_scope_spawn_call` with all phase-gate rejections |
| `sifr_hir/src/lower/mod.rs` | Registers the new module |
| `sifr_hir/src/lower/task_calls.rs` | Rejects `task.spawn(...)` with phase-specific diagnostic |
| `sifr_hir/src/lower/expressions.rs` | Wires `spawn` method on `TaskScope` to the new HIR lowering path |
| `sifr_codegen/src/lower_expr.rs` | Lowers `scope.spawn(coro)` method call to Rust |
| `sifr_codegen/src/lower_stmt.rs` | Injects `scope.__sifr_join_all().await` on `async with task.scope()` normal exit |
| `sifr_codegen/src/stmt_support_emitter.rs` | Same injection in the general emitter path |
| `sifr_codegen/src/preamble.rs` | Emits `__SifrTask<T, E>` and `__SifrTaskScope` with `spawn` and `__sifr_join_all` |
| `sifr_codegen/src/lib.rs` | Wires Tokio `sync` feature when task scope is used |
| `sifr/tests/e2e/pass/scope_spawn_core.sifr` | Positive fixture |
| `sifr/tests/e2e/fail/{scope_spawn_capture_rejected,scope_spawn_non_coroutine_rejected,detached_spawn_not_available}.sifr` | Negative fixtures |
| `sifr_codegen/src/lib_codegen_tests.rs` | Codegen unit tests for the spawn substrate |
| `internal_docs/phases/32_async_ecosystem.md` | Tracks the slice |
| `internal_docs/roadmap.md` | Updated milestone description |

---

## Validation performed

| Check | Result |
|---|---|
| `cargo check -p sifr_hir -p sifr_codegen -p sifr` | PASS |
| `cargo clippy -p sifr_hir -p sifr_codegen -- -D warnings` | PASS (no warnings) |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS |
| `cargo fmt --check` | PASS |
| `sifr check` on `scope_spawn_core.sifr` | PASS (no errors) |
| `sifr check` on `scope_spawn_capture_rejected.sifr` | Diagnostic: "scope.spawn() currently accepts only no-argument coroutine calls" |
| `sifr check` on `scope_spawn_non_coroutine_rejected.sifr` | Diagnostic: "scope.spawn() requires a coroutine argument, got 'int'" |
| `sifr check` on `detached_spawn_not_available.sifr` | Diagnostic: "task.spawn() is not available in v1; use scope.spawn(...)" |
| `cargo run -q -p sifr -- run scope_spawn_core.sifr` | PASS |
| `cargo emit` on `scope_spawn_core.sifr` | Generated code verified |
| `cargo test -p sifr_codegen scope_spawn` | PASS |
| `cargo test -p sifr_codegen task_scope_context` | PASS |
| `scripts/run_all_tests.sh --profile quick` | PASS |

---

## Blocking findings: none

## Non-blocking observations

### 1. `__SifrTask` receiver is allocated but never polled in this slice

The generated `__SifrTask<T, E>` struct holds a `tokio::sync::oneshot::Receiver<T>` alongside a `PhantomData<E>`. The `spawn` implementation sends the task result through the channel:

```rust
let child = tokio::spawn(async move {
    let result = future.await;
    let _ = sender.send(result);
});
```

The receiver is placed in the returned `__SifrTask` observer handle, but neither the handle nor its receiver is ever awaited in this milestone — the handle is dropped and `__sifr_join_all` only polls the `JoinHandle<()>`, not the oneshot receiver. The spawned task continues (and the sender is dropped) when the channel is closed by the task completing normally.

This is intentional per the design ("Joining/awaiting task handles and `TaskResult` materialization are upcoming slices"). Future slices that materialize `TaskResult` will poll the receiver. The current structure is sound for that path.

### 2. Both emitter paths modified for `__sifr_join_all` injection

`lower_stmt.rs:try_lower_simple_async_with_stmt` and `stmt_support_emitter.rs:emit_async_with` both now inject `scope.__sifr_join_all().await` at the end of a `TaskScope` body. This duplication is deliberate — `stmt_support_emitter.rs` handles the general emitter path used for more complex constructs, and `lower_stmt.rs` handles the simplified fast-path. Both are correct.

### 3. Tokio feature string updated in two places

`lib.rs` and `tests/e2e.rs` both update the `tokio` dependency spec to include `"sync"`. The test file mirrors the runtime string to keep the dependency spec test accurate. This is fine.

### 4. HIR `task_scope_calls.rs` helper naming

The new module follows the existing pattern of `task_calls.rs` for module-level task primitives. The `is_task_scope_type` helper is public to `super` (the `expressions` module) and used only at the call site. The helper and lowering function are well-scoped.

---

## Design compliance

| Design contract point | Status |
|---|---|
| `scope.spawn(coro)` returns typed `Task[T, E]` in HIR | Correct |
| Fallible coroutines rejected until task error plumbing lands | Correct |
| Non-empty-call captures rejected until task-boundary checking lands | Correct |
| `task.spawn(...)` rejected with phase-specific diagnostic | Correct |
| Normal scope exit joins spawned tasks via `__sifr_join_all` | Correct |
| `__SifrTaskScope` owns `JoinHandle<()>` children | Correct |
| `__SifrTask` observer handle backed by oneshot + PhantomData | Correct |
| Tokio `sync` feature added only when task scope is used | Correct |
| `mutable: true` on `let mut scope` so `__sifr_join_all` can borrow mutably | Correct |
| `#[tokio::main]` bootstrap not affected for non-async entrypoints | Correct |

---

## Conclusion

The implementation is correct, well-scoped, and consistent with the documented design and phase plan. All validations pass. Ready to land.