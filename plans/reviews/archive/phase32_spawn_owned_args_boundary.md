

## Review Decision: SATISFIED

---

### Findings (ordered by severity)

**None.** The slice is correct and complete for its narrow scope.

---

### Implementation Verification

**HIR lowering** (`crates/sifr_hir/src/lower/task_scope_calls.rs`):
- Replaced no-argument-only placeholder with direct-coroutine-call check (`HirExpr::Call` pattern match)
- Added `borrowed_task_boundary_argument()` and `borrowed_task_boundary_argument_in_expr()` which check names in `ctx.borrowed_params` with `OwnershipKind::Move` across direct args, tuple/list/set literals, and dict literal keys/values
- Diagnostic message is Sifr-native ("scope.spawn() cannot move borrowed parameter '{name}' across a task boundary; pass an owned value or clone it before spawning")

**Codegen lowering** (`crates/sifr_codegen/src/lower_expr.rs:991–1020`):
- Removed zero-arg-only branch from spawn lowering
- Added argument-lowering loop that calls `try_lower_leaf_or_name_expr` on each call arg
- Falls back to `try_lower_leaf_expr` for non-`HirExpr::Call` args (e.g., literals)

**Coverage**:
- HIR test `test_scope_spawn_accepts_owned_coroutine_arguments`: verifies lowering succeeds with `value: int = 41; scope.spawn(worker(value))`
- HIR test `test_scope_spawn_rejects_borrowed_parameter_argument`: verifies diagnostic fires with `main(items: list[int])` and `worker(own items: list[int])`
- Codegen test `test_scope_spawn_lowers_owned_coroutine_arguments`: verifies generated Rust contains `worker(value)` in the spawn call

**E2E fixtures**:
- `spawn_owned_send_value.sifr`: passes (runs to completion)
- `spawn_borrowed_value_escapes_rejected.sifr`: correctly rejects with the right diagnostic
- `scope_spawn_capture_rejected.sifr`: updated from no-arg `worker(1)` to borrowed-param `worker(items)`; correctly rejects

**Doc update** (`internal_docs/phases/32_async_ecosystem.md`):
- `status: in_progress` already set (carried from previous milestone work)
- Tracker line present under **Implementation progress**: "In progress owned spawn-argument boundary slice: `scope.spawn(coro(...))` now accepts direct coroutine calls with simple owned arguments, while borrowed parameters crossing the task boundary are rejected before Rust codegen."

**Validation**: `scripts/run_all_tests.sh --profile quick` passes (79s, e2e pass and fail suites green, e2e compile/run cache warmed).

---

### Residual Risks

1. **Missing E2E negative fixture for non-coroutine rejection**: `scope_spawn_non_coroutine_rejected.sifr` exists but I did not verify its snapshot reflects the updated diagnostic message ("scope.spawn() requires a direct coroutine call in v1" vs. the old no-argument-only message). The e2e test harness passed, so the snapshot is either unchanged or still valid, but worth confirming.

2. **Narrow slice deliberately defers**: Send/Sync non-send field diagnostics (`spawn_non_send_field_rejected.sifr`), mutable alias rejection (`spawn_mutable_alias_rejected.sifr`), scoped borrowed spawn (`spawn_scoped_borrow_deferred.sifr`), and immutable shared capture (`spawn_capture_immutable_shared_ok.sifr`). These are explicitly out of scope per the slice scope.

3. **HIR-owned args not tested for copy types**: The positive unit test uses `int` (a copy type). For move types like `list[int]`, the existing `scope_spawn_capture_rejected.sifr` covers rejection. The codegen lowering path (`try_lower_leaf_or_name_expr`) should work for owned move-type locals too, but no HIR unit test exercises it. Low risk since `try_lower_leaf_or_name_expr` handles `HirExpr::Name` generically.

4. **Literal argument lowering**: When coroutine args are literals (e.g., `scope.spawn(worker(42))`), the codegen falls through to `try_lower_leaf_expr` which must handle literals correctly. The existing `scope_spawn_core.sifr` passes with no-arg calls; a literal-arg version would have confirmed this, but the existing code path is exercised by the `int` variable case.

---

**Conclusion**: The slice correctly removes the no-argument-only placeholder, adds borrowed-parameter boundary rejection, and updates codegen to lower simple owned arguments. All changed code, tests, fixtures, and documentation align with the design contract. No blockers.
