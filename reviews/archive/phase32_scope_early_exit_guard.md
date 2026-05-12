

## Code Review: Phase 32 Scope Early-Exit Guard

### Findings

#### 1. Over-conservative `break`/`continue` handling (Moderate)

`stmt_contains_scope_early_exit` at lines 482-527 treats `Break`/`Continue` as scope-early-exit unconditionally, but `break`/`continue` only exit the **innermost** enclosing loop, not the `async with task.scope()`. A break inside a local `while`/`for` loop should be allowed.

Example that incorrectly triggers the guard:
```sifr
async with task.scope() as scope:
    handle = scope.spawn(worker())
    while True:
        break  # This exits the while, NOT the scope
```

This is inconsistent with the documented rationale: the concern is early exit from the scope's control flow. Break/continue don't escape the scope.

**Fix location**: Either exclude `HirStmt::Break`/`HirStmt::Continue` from the guard, or track the enclosing loop context and only flag breaks/continues that would exit the scope.

#### 2. Missing HIR statement variants (Minor)

`stmt_contains_scope_early_exit` doesn't handle:
- `HirStmt::Lambda` — lambda bodies are their own scope so this is fine
- `HirStmt::AsyncWith` — should recurse since it has its own body

This is a latent gap for future nested async-with scenarios.

**Fix location**: Add `HirStmt::Lambda { body, .. } => body.iter().any(stmt_contains_scope_early_exit)` and `HirStmt::AsyncWith { body, .. } => body.iter().any(stmt_contains_scope_early_exit)` patterns.

#### 3. Missing tests for `break`/`continue` in loops

The diff only adds one e2e fail test (`task_scope_early_return_rejected.sifr`). The remaining early-exit variants (`break`, `continue`, `yield`) have no direct negative coverage beyond `task_scope_early_return_rejected.sifr`.

- `task_scope_break_in_loop_rejected.sifr` — correctly created but removed due to false-positive behavior (see finding #1)
- `task_scope_continue_in_loop_rejected.sifr` — same as above

#### 4. Diagnostic code choice (`TYPE_MISMATCH`)

Line 676 uses `DiagnosticCode::TYPE_MISMATCH`. This is used throughout `async_with.rs` for consistency, but this particular issue is more about control flow than type mismatch. Consider whether a more specific diagnostic code would improve error categorization (e.g., `ASYNC_CONTROL_FLOW`).

### What's Correct

- **Return/raise/yield handling**: Correctly identifies these as scope-early-exit.
- **Nested constructs**: Properly recurses into `If`, `While`, `For`, `TryExcept`, `Match`, `AsyncWith`, `With`.
- **Guard logic**: The AND condition (has spawn AND has early exit) is correct — empty scopes or scopes without spawns are allowed.
- **Phase doc**: Correctly documents the conservative approach.
- **Documentation**: Clear diagnostic message explains the limitation.

### Recommendations

1. **Either** remove `Break`/`Continue` from the guard (simpler, less conservative), **or** add loop context tracking to only flag breaks/continues that would exit the scope.

2. Add explicit test coverage once finding #1 is resolved:
   - `task_scope_break_in_loop_rejected.sifr`
   - `task_scope_continue_in_loop_rejected.sifr`
   - `task_scope_yield_in_scope_rejected.sifr`

3. Add the minor missing patterns to `stmt_contains_scope_early_exit`.

### Verdict

The slice correctly identifies the correctness gap and implements the documented conservative guard. However, finding #1 (break/continue over-conservatism) should be addressed before merge. The implementation can proceed once that issue is resolved.

**SATISFIED** pending resolution of finding #1 (break/continue handling) and addition of test coverage for `break`/`continue`/`yield` variants once the behavioral issue is fixed.
