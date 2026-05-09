

## Review Findings

**SATISFIED** — no blocking issues.

### HIR Recognition (`hir_nodes.rs`, `async_with.rs`)
- `HirAsyncWithKind::TaskGroup` variant added alongside `TaskScope`
- `lower_async_with` handles `"TaskGroup"` identifier, validates zero arguments, defines variable in scope with correct type
- Error messages updated to mention all three supported async-with forms

### Spawn on TaskGroup (`task_scope_calls.rs`, `lower_expr.rs`)
- `is_task_scope_type` correctly includes both `TaskScope` and `TaskGroup`
- This is **correctly conservative** — existing infallible/no-arg spawn validation (`"until task error plumbing lands"`, `"until task-boundary checking lands"`) applies uniformly, which is the intended design for this first slice

### Codegen (`lib.rs`, `lower_stmt.rs`, `stmt_support_emitter.rs`)
- `module_uses_task_scope` includes `TaskGroup` in its check, so `__SifrTaskScope` runtime is included when needed
- Both async-with lowering sites emit `__sifr_join_all().await` on normal exit for both `TaskScope` and `TaskGroup`
- Emitted Rust correctly shows `group.spawn()` returning `__SifrTask`, then `group.__sifr_join_all().await` before block exit

### Test Coverage
- **Unit test** (`test_task_group_basic_lowers_to_scope_runtime_substrate`): verifies `struct __SifrTaskScope`, `group.spawn(worker())`, `group.__sifr_join_all().await`, and `tokio` dependency
- **E2E fixture** (`task_group_basic.sifr`): full end-to-end with spawn, join, and normal exit

### PR Manifest (`pr_e2e_manifest.json`)
- JSON valid, `"task_group_basic"` added correctly, trailing comma acceptable

### Phase Doc (`32_async_ecosystem.md`)
- Progress note accurately states: conservative infallible/no-capture children, group error policy and sibling cancellation deferred to follow-up slices

### Alignment with Design Doc (`async_concurrency_model.md`)
- Current slice correctly defers all TaskGroup-specific semantics (heterogeneous errors, sibling cancellation, group failure policy) per the model — this is a pure surface/recognition slice

**No additional changes required.**
