# Ad hoc issue: Complete abnormal async cleanup

Status: planned external prerequisite

Owner: core language and async runtime

## Objective

Complete the accepted async resource contracts that SQL transactions and streams
require before runtime implementation starts.

## Owning architecture

- `internal_docs/async_concurrency_model.md`
- `internal_docs/architecture.md`

## Scope

- abnormal body exit through `async with`
- cancellation-specific `AsyncExitCause` delivery
- secondary cleanup evidence for errors and timeouts
- `AsyncClosable.aclose()` on early `async for` exit

## Acceptance criteria

- [ ] `async with` calls `__aexit__` for return, ordinary error, timeout,
  cancellation, and runtime-fault paths.
- [ ] `__aexit__` receives the exact accepted `AsyncExitCause` value.
- [ ] Cleanup errors preserve the primary result and add typed secondary evidence.
- [ ] Cleanup timeout records `SecondaryError.CleanupTimedOut` and invalidates the
  resource.
- [ ] Early `async for` exit calls `AsyncClosable.aclose()` exactly once.
- [ ] Nested cleanup uses last-in, first-out order for every exit path.
- [ ] Compiler, lowering, code generation, runtime, and end-to-end evidence pass.
- [ ] An exact-SHA review approves the implementation.

## SQL dependency

Milestone `sql_9_postgresql_runtime` cannot start until this issue has merged
evidence. The SQL phase records that merge in its verification inventory.
