

The code review is complete. I wrote my findings to `reviews/phase32_async_with_named_context_state.md` and the final line is `REVIEW_STATUS: SATISFIED`.

**Summary of findings:**

1. **Direct-name lowering (`stmt_support_emitter.rs`)**: For `async with name as x` where `name` is a `HirExpr::Name`, codegen now calls `name.__aenter__()` and `name.__aexit__()` directly instead of materializing a `__sifr_async_cm` temporary. The generated Rust uses `let mut name` (via the mutation marking) so the `&mut self` receiver requirement is satisfied. The context variable is never moved or consumed — it remains accessible after the block for post-body assertions. Sound.

2. **Mutation marking (`queries.rs`)**: Named async-with context variables are added to `collect_mutated_vars` so local binding codegen emits `let mut`. This only affects the binding's `let mut` annotation in the local scope and has no cascading effects on parameter conventions, task boundary sendability, or borrow rules beyond what's already needed for the method calls.

3. **Fixture coverage**: `async_with_nested_cleanup_order.sifr` asserts state visibility inside blocks, after inner block (inner exited, outer still active), and after outer block (outer exited). The LIFO ordering is verified by generated Rust emit. The fixture scope is appropriate — abnormal-exit cleanup is intentionally deferred per milestone_async_7a.

4. **No regressions**: Non-name contexts, built-in forms, and the original `async_with_basic.sifr` behavior are all preserved. The slice is a strict improvement for the named case.
