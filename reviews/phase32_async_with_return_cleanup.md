

Based on my thorough code review, here is my assessment:

---

## Code Review: Phase 32 Async-With Return Cleanup Slice

### Summary

I reviewed the changed files focusing on correctness, phase semantics, return value/error evaluation, nested async-with handling, HIR guard behavior, and the `body_always_exits` optimization.

### Finding 1: Missing `TryExcept`/`TryFinally` handling in return injection (Minor)

**File:** `crates/sifr_codegen/src/stmt_support_emitter.rs:9839`

`inject_async_with_return_cleanup_stmt` handles `Return`, `If`, `IfLet`, `Match`, `With`, `Block`, `For`, `While`, `Loop`, but not `TryExcept` or `TryFinally`.

However, `return` inside `try`/`except` blocks is captured by the `try` desugaring (wrapping in `Result`), and the return injection applies to the outer `match` of the desugared try. So no return can escape the `TryExcept` without going through the desugared control flow that the injection already handles.

This is correct by construction of the try lowering, but the code lacks explicit coverage. Future code changes to try lowering could break this assumption silently. **Recommendation:** Add explicit `TryExcept`/`TryFinally` cases with a comment explaining why they're safe passthroughs.

### Finding 2: Nested `async with` return cleanup correctness (Verified Correct)

I tested a nested async-with scenario where both inner and outer have return statements. The generated code correctly:
1. Preserves the return value (`Ok(42)`)
2. Calls inner `__aexit__(&AsyncExitCause::Return).await?`
3. Calls outer `__aexit__(&AsyncExitCause::Return).await?`
4. Returns the preserved payload

This proves the LIFO cleanup order is maintained through nested returns.

### Finding 3: `try` blocks inside `async with` with return (Verified Correct)

When `return` is inside a `try` block within `async with`:
- The return goes through the desugared try path
- Both the `Ok` branch and the `Err` branch get return cleanup injected
- The cleanup runs on each path before returning

### Finding 4: `body_always_exits` optimization (Correct)

**File:** `crates/sifr_codegen/src/stmt_support_emitter.rs:7512,7548,7595`

The `body_always_exits` check uses `queries::block_control_flow_effect(body).always_exits()`. The `always_exits()` method returns `true` for `AlwaysReturns`, `AlwaysRaises`, and `AlwaysExits` — all non-fallthrough exits. This is sound: if the body always exits, the normal-exit path is unreachable and can be omitted.

However, I want to note a theoretical edge case: if HIR changes introduce new statement types that can return with a value but don't get handled by `stmt_contains_user_async_with_blocked_exit`, the body might have a return that `always_exits()` doesn't detect. The current implementation correctly handles all existing statement types.

### Finding 5: HIR guard behavior (Correct)

**File:** `crates/sifr_hir/src/lower/async_with.rs:216-223`

The check `body.iter().any(stmt_contains_user_async_with_blocked_exit)` uses `stmt_contains_scope_exit(stmt, false)` which:
- Returns `false` for `Return` (because `include_return = false`)
- Returns `true` for `Raise` and `Yield`

This correctly allows `return` in user-defined async-with bodies while blocking `raise`/`yield` until abnormal-exit cleanup is implemented.

### Finding 6: Task scope/task-group guard unchanged (Correct)

**File:** `crates/sifr_hir/src/lower/async_with.rs:854-865`

The task scope/group guard still uses `stmt_contains_scope_early_exit` with `include_return = true`, blocking `return` when spawned children are present. This is correct and unchanged.

### Finding 7: Error propagation semantics (Correct)

The generated code follows the model specification:
- `let __sifr_async_with_return = Ok(value)` captures the return value
- `__aexit__(&AsyncExitCause::Return).await?` awaits cleanup with `?` (short-circuits on failure)
- If `__aexit__` fails, the return is not performed (primary error is the cleanup failure)
- If cleanup succeeds, the preserved payload is returned

This matches the "Body performs explicit `return`, exit succeeds → `Ok(None)`, return proceeds" and "Body performs explicit `return`, exit fails → `Err(ExitE)`, the return is not performed" rows in the model.

### Finding 8: Fail fixture validation

**File:** `crates/sifr/tests/e2e/fail/async_with_raise_cleanup_rejected.sifr`

The test correctly expects an error when `raise` is used inside user-defined async-with.

---

## REVIEW_STATUS: SATISFIED

The implementation is correct. The minor gap in `TryExcept`/`TryFinally` handling is safe by construction of the try lowering, but adding explicit cases with comments would improve maintainability. All other aspects — return value preservation, error propagation, nested cleanup order, HIR guards, and the `body_always_exits` optimization — are implemented correctly per the model specification.
