

## Review Complete

**SATISFIED**

The scope-failure exit surface implementation is complete and correct. Key validation points:

1. **Generated error types registered**: `ScopeFailure` and `TimeoutError` are now properly collected as referenced error classes when task scope/group or timeout constructs appear (error_refs.rs lines 265-277).

2. **Upcast impl generation**: `impl From<TimeoutError> for Error` and `impl From<ScopeFailure> for Error` are generated when both `Error` and the concrete errors are referenced, excluding IO pseudo-subclasses (lib.rs lines 577-586).

3. **Codegen preserves error surface**: Timeout context returns `Err(TimeoutError::new(...).into())` (render.rs line 900), and scope exit propagates `Err(__sifr_scope_failure.into())` (stmt_support_emitter.rs lines 7344-7360, lower_stmt.rs lines 1938-1945).

4. **HIR rejection**: Functions with task spawns in scope bodies must return `Result[..., ScopeFailure]` or `Result[..., Error]` (async_with.rs lines 608-618).

5. **All tests pass**: unit tests (4 error_refs, 1 async_generated, 14 task), e2e pass/fail suites, quick profile validation (65s wall time), and `cargo clippy`.

6. **Code quality**: formatting clean, no lint issues, HIR maintainability guardrails pass.

The implementation is consistent with `async_concurrency_model.md` (decisions 25, 79-82, 100, 451-462) and the Phase 32 ecosystem doc. No correctness bugs, semantic gaps, or user-triggerable panic risks identified.
