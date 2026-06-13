

Review complete. Here are my findings:

---

## Review: milestone_async_3 task.gather slice

**Result: SATISFIED**

### 1. HIR Type Checking (`task_calls.rs:42-99`)

- `task.gather()` correctly requires exactly one argument that must be `list[Task[T, E]]` (`task_calls.rs:68-88`)
- Return type is `Awaitable[TaskResult[list[T], E]]` as specified in the model (`task_calls.rs:94-97`)
- Rejects non-list arguments with a clear diagnostic
- Rejects lists whose elements are not `Task[T, E]`
- Rejects calls outside async functions
- **Note on error type handling:** For the conservative infallible slice, `E` resolves to `Infallible` (never type). The code correctly passes through `result_ok_ty` and `result_err_ty` derived from the list element type, which works correctly for the current infallible-only spawn. Once fallible spawn lands, `E` will properly carry the group error type.

### 2. Handle Consumption (`task_calls.rs:219-238`)

- `mark_task_handle_names_moved` walks the argument expression and marks any `Name` bindings of type `Task` or `List` as moved
- This prevents reuse after `gather` consumes the handles
- Covers: direct names, list literals, tuple literals, and nested combinations
- **Correct:** The conservative slice only supports `scope.spawn(no-arg infallible coroutine)` which always produces `HirExpr::Call` nodes for list literal elements, so the list literal case in `mark_task_handle_names_moved` is the primary path

### 3. Codegen Preamble (`preamble.rs:479-503`)

- `__sifr_task_gather<T, E>` iterates handles in input order and awaits each via `handle.join().await`
- Fail-fast: first `Err` or `Cancelled` immediately returns, cancelling unwaited siblings (Rust drops the `Vec`)
- Returns `__SifrTaskResult::Ok(values)` preserving input order on success
- Uses generic type params matching the model's `list[T]` signature
- **Correct:** Cancellation of remaining siblings happens implicitly via Rust's `Vec` drop — handles that were moved into the function but not yet joined are dropped, which drops the `__SifrTask`, which calls the `AbortHandle` on scope children. This is sound because the scope's `__sifr_join_all` will have already awaited any children spawned before `gather` returned.

### 4. Runtime Helper Inclusion (`lib.rs:887-941`)

- `module_uses_task_scope` now detects `__sifr_task_gather` calls and emits `build_task_scope_items()` when gather is used
- This ensures `__SifrTask`, `__SifrTaskResult`, `__sifr_task_gather`, and Tokio are all emitted together
- **Correct:** Works regardless of whether `async with task.scope()` and `gather` appear in the same function or different functions (though in this slice, `gather` must appear inside a scope due to spawn requirements)

### 5. Codegen Test (`lib_codegen_tests.rs:3729-3751`)

- Verifies `async fn __sifr_task_gather` appears in generated code
- Verifies `__sifr_task_gather(vec![...])` call pattern
- Verifies the result type `__SifrTaskResult<Vec<i64>, std::convert::Infallible>`
- Verifies Tokio dependency is included

### 6. E2E Fixture (`task_gather_ordered.sifr`)

- Spawns two tasks, gathers them, verifies the ordered-success path
- Simple but meaningful — validates end-to-end compilation and execution
- Confirmed passing: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_gather_ordered.sifr`

### 7. Manifest (`pr_e2e_manifest.json`)

- Added `task_gather_ordered` to the fixture list per the `manifest_update_required` validation rule

### 8. Phase Doc Update (`32_async_ecosystem.md:501`)

- One-line addition to `Implementation progress` for `milestone_async_3`
- Correctly scoped to this slice's conservative deliverable
- Explicitly notes that fail-fast fallible child behavior is deferred

### No Blocking Findings

| Check | Status |
|---|---|
| Type correctness | ✓ |
| Handle consumption soundness | ✓ |
| Runtime helper inclusion | ✓ |
| Deterministic input ordering | ✓ |
| Fail-fast error branch | ✓ |
| Manifest/doc accuracy | ✓ |
| Validation signal | ✓ |

This is a clean, minimal, correctly-scoped slice. The deferred items (fallible spawn, collect-all semantics) are explicitly documented. Ready to open the PR.
