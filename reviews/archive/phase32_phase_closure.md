

The 21 failures are pre-existing snapshot mismatches unrelated to this PR. The try_finally tests all pass.

**Review summary:**

The codegen fix correctly:
1. Dispatches top-level `HirStmt::TryFinally` through structured lowering
2. Falls back to `lower_stmt_expr_for_ir` for block-level `Let`/`Assign`/`Expr` statements
3. Infers try/finally error type from nested try/except metadata when no `Result` return exists
4. Avoids emitting `return Err(...)` from non-`Result` functions (uses `unreachable!` instead)

The regression test covers the cleanup try/except with `?` calls pattern.

Phase closure docs are correctly updated with all milestones completed and the Phase 32 summary on the roadmap.

**SATISFIED.**
