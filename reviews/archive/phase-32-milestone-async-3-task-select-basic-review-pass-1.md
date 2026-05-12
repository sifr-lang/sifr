

**SATISFIED**

This is a clean, well-scoped implementation of `task.select(a, b)` for milestone_async_3. All validations pass, including the HIR guardrail at exactly 1200 lines.

### What was implemented:

**Type system** (`types.rs`, `union.rs`):
- Added `Type::Select2(A, B)` as a discriminated union for heterogeneous binary select results
- Properly integrated into `contains_any`, `eq`, `display_name`, `rust_type`, `short_name`, and sort ordering

**HIR lowering** (`task_calls.rs`, `expressions_tests.rs`):
- `lower_task_select_call` validates: async-only, no kwargs, exactly 2 task handle args, marks handles as moved
- Returns `Awaitable(Select2(TaskResult[A, EA], TaskResult[B, EB]))`
- Added `test_task_select_consumes_handle_bindings` to verify double-use rejection

**Codegen** (`preamble.rs`, `lower_expr.rs`, `lib.rs`, plus 4 small pattern matches):
- `__SifrSelect2<A, B>` enum with `First(A)` and `Second(B)` variants
- `__sifr_task_select` function using `tokio::select! { biased; }` for deterministic first-argument priority
- Loser: `abort_handle.abort()` then `receiver.await` for cleanup
- `lower_expr.rs` handles `__sifr_task_select` as a simple 2-arg call

**Tests**:
- HIR unit test: `test_task_select_consumes_handle_bindings`
- Codegen unit test: `test_task_select_lowers_to_private_select_helper`
- E2E: `task_select_first_completion.sifr` validates the losing task (slow 200ms) does not write its marker file when the fast task wins

### Non-blocking notes:
1. The `union.rs` sort ordering change shifted the priority of several type variants (Select2 inserted at 22, everything after shifted by 1). This is consistent but worth noting — future diffs should watch for unintended reordering.
2. Loser cleanup error handling is documented as deferred to a follow-up slice per the phase plan.
